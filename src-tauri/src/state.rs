use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::core::desk::HitRect;
use crate::core::reminder::{Intensity, TickOutcome};
use crate::core::reminder_copy;
use crate::events::{self, ChangedPayload, FirePayload, PetStatePayload, Section, TickPayload};
use crate::model::{Model, Phase};
use crate::store::Store;

const SAVE_INTERVAL: Duration = Duration::from_secs(1);

pub struct AppState {
    model: Mutex<Model>,
    pub store: Store,
    last_save: Mutex<Instant>,
    /// Click-through bookkeeping for the pet window; see `windows::poll_pet_hit`.
    pub pet_hit: Mutex<PetHit>,
}

/// The pet window is transparent almost everywhere, so it ignores the mouse by
/// default and is switched back on only while the cursor is over something the
/// webview reported as clickable. Once the window ignores cursor events it stops
/// receiving pointer events, so the webview cannot notice the cursor coming
/// back — Rust polls the global cursor instead.
#[derive(Debug, Default)]
pub struct PetHit {
    pub rects: Vec<HitRect>,
    /// A native drag is in progress; leave the window alone until it ends.
    pub dragging: bool,
    /// What the window was last told, so the poll only calls into AppKit on change.
    pub ignoring: Option<bool>,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        let mut model = store.load();
        model.seed_reminders();
        // The derived level fields are never read from disk.
        model.pet.refresh();
        Self {
            model: Mutex::new(model),
            store,
            // Start a full interval in the past so the first save is not suppressed.
            last_save: Mutex::new(Instant::now() - SAVE_INTERVAL),
            pet_hit: Mutex::new(PetHit::default()),
        }
    }

    /// Run `f` against the model under the lock.
    ///
    /// A panic inside a previous `with` poisons the mutex; recovering the inner value
    /// keeps a single bad command from bricking the app for the rest of the session.
    pub fn with<R>(&self, f: impl FnOnce(&mut Model) -> R) -> R {
        let mut guard = self.model.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard)
    }

    pub fn snapshot(&self) -> Model {
        self.with(|m| m.clone())
    }

    pub fn with_pet_hit<R>(&self, f: impl FnOnce(&mut PetHit) -> R) -> R {
        let mut guard = self.pet_hit.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard)
    }

    pub fn save_debounced(&self) {
        let mut last = self.last_save.lock().unwrap_or_else(|e| e.into_inner());
        if last.elapsed() < SAVE_INTERVAL {
            return;
        }
        *last = Instant::now();
        drop(last);
        self.flush();
    }

    pub fn flush(&self) {
        let model = self.snapshot();
        if let Err(err) = self.store.save(&model) {
            eprintln!("momo: failed to persist state: {err}");
        }
    }

    pub fn emit_changed(&self, app: &AppHandle, section: Section) {
        let _ = app.emit(
            events::CHANGED,
            ChangedPayload {
                section: section.as_str(),
            },
        );
        // Nearly every command that changes the model can change the mood too
        // (start, pause, ack, the 睡眠动画 flag…), so this is the one choke point.
        self.sync_mood(app);
    }

    /// Emit `pet:state` if the mood moved. Cheap enough to call on every change.
    pub fn sync_mood(&self, app: &AppHandle) {
        let (changed, state) = self.with(|m| (m.sync_mood(), m.pet_mood));
        if changed {
            let _ = app.emit(events::PET_STATE, PetStatePayload { state });
        }
    }

    fn tick_payload(&self) -> TickPayload {
        self.with(|m| TickPayload {
            remaining_secs: m.timer.remaining_secs,
            phase: m.timer.phase,
            running: m.timer.running,
            round: m.timer.round,
            belly_cells: m.timer.belly_cells(&m.settings),
        })
    }

    pub fn emit_tick(&self, app: &AppHandle) {
        let _ = app.emit(events::TICK, self.tick_payload());
        let title = self.with(|m| {
            let secs = m.timer.remaining_secs;
            format!("{:02}:{:02}", secs / 60, secs % 60)
        });
        crate::tray::set_title(app, &title);
    }

    /// Advance the clock by real elapsed time, credit any completed focus phase to the
    /// active task, then emit. Called once per second by the tick thread.
    pub fn tick(&self, app: &AppHandle, elapsed_secs: u32) {
        let (changes, abandoned) = self.with(|m| {
            let settings = m.settings.clone();
            m.advance_presence(elapsed_secs);
            let abandoned = m.advance_pause(elapsed_secs);
            let changes = m.timer.advance(elapsed_secs, &settings);
            for change in &changes {
                if change.from == Phase::Focus {
                    m.record_focus_phase(change.completed, settings.focus_secs);
                }
            }
            (changes, abandoned)
        });

        for change in &changes {
            let _ = app.emit(events::PHASE, change);
        }
        // Only a phase that ran its course rings; a skip is the user's own
        // doing and needs no announcing. A sleep that replays several phases
        // at once rings only the last so wake-up is not a drum roll.
        if let Some(change) = changes.iter().rev().find(|c| c.completed) {
            let sound = self.with(|m| m.settings.phase_sounds.for_end_of(change.from));
            crate::audio::play(sound);
        }
        // An abandoned pause reset the phase without a phase change; the
        // windows still need to hear that stats and the countdown moved.
        if !changes.is_empty() || abandoned {
            self.emit_changed(app, Section::Tasks);
        }

        self.emit_tick(app);
        // A focus phase that just ended releases anything parked during it.
        let round_ended = changes.iter().any(|c| c.from == Phase::Focus);
        self.run_reminders(app, elapsed_secs, round_ended);
        self.sync_mood(app);
        self.save_debounced();
    }

    /// Advance every reminder and emit whatever wants attention.
    fn run_reminders(&self, app: &AppHandle, elapsed_secs: u32, round_ended: bool) {
        let now = chrono::Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        // 检测到会议 / 通话 — reads the default input device's run state.
        let in_meeting = crate::platform::platform().microphone_in_use();

        let sit_changed = self.with(|m| {
            m.roll_body_day(&today);
            // Sitting happens whether or not the timer runs.
            m.body.advance_sit(elapsed_secs)
        });
        if sit_changed {
            self.emit_changed(app, Section::Body);
        }

        let fires = self.with(|m| {
            let ctx = m.fire_context(now, in_meeting);
            let mut ids: Vec<(u32, crate::core::reminder::Intensity)> = Vec::new();
            let mut interrupted: Option<u32> = None;

            for reminder in &mut m.reminders {
                if round_ended {
                    if let Some(intensity) = reminder.release_deferred(&ctx) {
                        ids.push((reminder.id, intensity));
                    }
                }
                match reminder.tick(elapsed_secs, &ctx) {
                    TickOutcome::Fire(intensity) => {
                        if ctx.in_focus {
                            // Only 直接打断 gets this far while focusing; remember
                            // it so the session can say what broke it.
                            interrupted = Some(reminder.id);
                        }
                        ids.push((reminder.id, intensity));
                    }
                    TickOutcome::Idle | TickOutcome::Deferred => {}
                }
            }
            if interrupted.is_some() {
                m.interrupted_by = interrupted;
            }

            ids.into_iter()
                .filter_map(|(id, intensity)| {
                    m.reminders
                        .iter()
                        .find(|r| r.id == id)
                        .map(|r| FirePayload {
                            id: r.id,
                            name: r.name.clone(),
                            message: reminder_copy::fill(&r.message, &m.body),
                            intensity,
                            color: r.color.clone(),
                            must_complete: r.rules.must_complete,
                            duration_secs: r.duration_secs,
                        })
                })
                .collect::<Vec<_>>()
        });

        let mini = self.with(|m| m.mini_enabled);

        let sounds: Vec<(u32, crate::core::sound::SoundSetting)> =
            self.with(|m| m.reminders.iter().map(|r| (r.id, r.rules.sound)).collect());

        for payload in fires {
            // Every window hears the raw event; the surface that renders it depends
            // on the intensity the engine chose for this particular firing.
            let _ = app.emit(events::REMINDER_FIRE, payload.clone());
            if let Some((_, sound)) = sounds.iter().find(|(id, _)| *id == payload.id) {
                crate::audio::play(*sound);
            }
            match payload.intensity {
                // In 迷你模式 the bar swells to carry the message itself. You have
                // already handed the screen over to whatever you are working in —
                // covering it with a second window would take it back.
                Intensity::Bubble | Intensity::Pet if mini => {
                    // The bar grows itself once it knows how tall the message
                    // rendered — see set_mini_height.
                    if payload.intensity == Intensity::Pet {
                        self.with(|m| m.begin_nag(payload.id));
                    }
                    let _ = app.emit_to("mini", "mini:nudge", payload.clone());
                }
                Intensity::Bubble => {
                    let _ = crate::windows::show_bubble(app, payload.clone());
                }
                Intensity::Pet => {
                    self.with(|m| m.begin_nag(payload.id));
                    let _ = crate::windows::ensure_pet(app);
                    let _ = app.emit_to("pet", "pet:nudge", payload.clone());
                }
                // 全屏遮罩 is the "you must deal with this" tier; shrinking it into a
                // 260px bar would quietly disable the loudest reminder there is.
                Intensity::Fullscreen => {
                    let _ = crate::windows::show_overlay(app, payload.clone());
                }
            }
        }

        // The pet ducks out of the way of full-screen apps between firings.
        let hide = self.with(|m| m.settings.pet_flags.hide_fullscreen);
        crate::windows::sync_pet_visibility(app, hide);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Phase;
    use std::fs;

    fn store_in(tag: &str) -> Store {
        let dir = std::env::temp_dir().join(format!("momo-state-test-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        Store::new(&dir)
    }

    #[test]
    fn new_loads_the_model_from_the_store() {
        let state = AppState::new(store_in("load"));
        assert!(state.snapshot().tasks.is_empty());
        assert_eq!(state.snapshot().reminders.len(), 4);
    }

    #[test]
    fn with_mutates_under_the_lock_and_returns_a_value() {
        let state = AppState::new(store_in("with"));
        let id = state.with(|m| m.add_task("新的".into(), 1));
        assert_eq!(state.snapshot().tasks.len(), 1);
        assert!(state.snapshot().tasks.iter().any(|t| t.id == id));
    }

    #[test]
    fn advance_credits_the_active_task_when_a_focus_phase_completes() {
        let state = AppState::new(store_in("credit"));
        let active = state.with(|m| {
            let id = m.add_task("写产品需求文档".into(), 3);
            m.timer.active_task = Some(id);
            id
        });
        let before = state.snapshot();
        let spent_before = before
            .tasks
            .iter()
            .find(|t| t.id == active)
            .map(|t| t.spent)
            .unwrap();

        let changes = state.with(|m| {
            m.timer.start();
            let settings = m.settings.clone();
            let changes = m.timer.advance(settings.focus_secs, &settings);
            for change in &changes {
                if change.completed && change.from == Phase::Focus {
                    if let Some(id) = m.timer.active_task {
                        m.credit_task(id);
                    }
                }
            }
            changes
        });

        assert_eq!(changes.len(), 1);
        let after = state.snapshot();
        let spent_after = after.tasks.iter().find(|t| t.id == active).unwrap().spent;
        assert_eq!(spent_after, spent_before + 1);
    }

    #[test]
    fn flush_writes_to_disk() {
        let state = AppState::new(store_in("flush"));
        state.with(|m| m.timer.remaining_secs = 7);
        state.flush();
        let reloaded = state.store.load();
        assert_eq!(reloaded.timer.remaining_secs, 7);
    }

    #[test]
    fn save_debounced_writes_at_most_once_per_second() {
        let state = AppState::new(store_in("debounce"));
        state.with(|m| m.timer.remaining_secs = 11);
        state.save_debounced();
        state.with(|m| m.timer.remaining_secs = 22);
        state.save_debounced(); // suppressed
        assert_eq!(state.store.load().timer.remaining_secs, 11);
    }

    #[test]
    fn a_poisoned_lock_is_recovered_rather_than_panicking() {
        use std::sync::Arc;
        let state = Arc::new(AppState::new(store_in("poison")));
        let clone = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            clone.with(|_| panic!("poison the mutex"));
        })
        .join();
        // Must not panic.
        assert_eq!(state.snapshot().reminders.len(), 4);
    }
}

#[cfg(test)]
mod model_extension_tests {
    use super::*;
    use crate::model::Phase;
    use std::fs;

    fn store_in(tag: &str) -> Store {
        let dir = std::env::temp_dir().join(format!("momo-state-test-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        Store::new(&dir)
    }

    #[test]
    fn completing_a_focus_phase_records_a_session_and_credits_the_pet() {
        let state = AppState::new(store_in("session"));
        state.with(|m| {
            m.timer.start();
            let settings = m.settings.clone();
            let changes = m.timer.advance(settings.focus_secs, &settings);
            for change in &changes {
                if change.from == Phase::Focus {
                    m.record_focus_phase(change.completed, settings.focus_secs);
                }
            }
        });

        let model = state.snapshot();
        assert_eq!(model.stats.sessions.len(), 1);
        assert!(model.stats.sessions[0].completed);
        assert_eq!(model.pet.lifetime_pomodoros, 1);
    }

    #[test]
    fn a_file_written_before_stats_and_pet_existed_still_loads() {
        let dir = std::env::temp_dir().join("momo-state-test-v1");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        // Current schema, but written before stats/pet/reminders existed: every new
        // field must fall back to its serde default rather than discarding the file.
        let body = r#"{"timer":{"phase":"focus","remainingSecs":99,"running":false,"round":1,"activeTask":null},"tasks":[],"settings":{"accent":"terracotta","tone":"playful","focusSecs":1500,"shortBreakSecs":300,"longBreakSecs":900,"roundsPerCycle":4,"petFlags":{"snapEdges":true,"clickInteract":true,"hideFullscreen":true,"sleepAnimation":false}},"nextTaskId":0}"#;
        fs::write(
            dir.join("state.json"),
            format!(
                r#"{{"schemaVersion":{},"model":{}}}"#,
                crate::store::SCHEMA_VERSION,
                body
            ),
        )
        .expect("write");

        let model = Store::new(&dir).load();
        assert_eq!(model.timer.remaining_secs, 99);
        assert_eq!(model.stats.sessions.len(), 0);
        assert_eq!(model.pet.selected, 0);
    }
}

#[cfg(test)]
mod reminder_wiring_tests {
    use super::*;
    use std::fs;

    fn store_in(tag: &str) -> Store {
        let dir = std::env::temp_dir().join(format!("momo-state-test-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        Store::new(&dir)
    }

    #[test]
    fn a_fresh_model_seeds_the_four_builtin_reminders_in_order() {
        let state = AppState::new(store_in("reminders"));
        let names: Vec<String> = state
            .snapshot()
            .reminders
            .iter()
            .map(|r| r.name.clone())
            .collect();
        assert_eq!(
            names,
            vec!["站起来动一动", "喝水", "远眺护眼", "收工前复盘"]
        );
    }

    #[test]
    fn body_counters_reset_when_the_day_changes() {
        let state = AppState::new(store_in("body"));
        state.with(|m| {
            m.body.water_cups = 5;
            m.body.day = "2020-01-01".into();
            m.roll_body_day("2026-08-20");
        });
        let body = state.snapshot().body;
        assert_eq!(body.water_cups, 0);
        assert_eq!(body.day, "2026-08-20");
        assert_eq!(body.water_goal, 8);
        assert_eq!(body.stand_goal, 6);
    }

    #[test]
    fn body_counters_survive_a_tick_on_the_same_day() {
        let state = AppState::new(store_in("body-same"));
        state.with(|m| {
            m.body.day = "2026-08-20".into();
            m.body.water_cups = 3;
            m.roll_body_day("2026-08-20");
        });
        assert_eq!(state.snapshot().body.water_cups, 3);
    }

    #[test]
    fn changing_tone_retones_every_unedited_reminder() {
        use crate::model::Tone;
        let state = AppState::new(store_in("retone"));
        state.with(|m| {
            m.settings.tone = Tone::Professional;
            m.retone_reminders();
        });
        let model = state.snapshot();
        assert_eq!(
            model.reminders[1].message,
            "补充 200ml 水，今日 {cups}/{goal} 杯。"
        );
    }
}
