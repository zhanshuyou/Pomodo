use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::events::{self, ChangedPayload, Section, TickPayload};
use crate::model::{Model, Phase};
use crate::store::Store;

const SAVE_INTERVAL: Duration = Duration::from_secs(1);

pub struct AppState {
    model: Mutex<Model>,
    pub store: Store,
    last_save: Mutex<Instant>,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        let model = store.load();
        Self {
            model: Mutex::new(model),
            store,
            // Start a full interval in the past so the first save is not suppressed.
            last_save: Mutex::new(Instant::now() - SAVE_INTERVAL),
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
    }

    /// Advance the clock by real elapsed time, credit any completed focus phase to the
    /// active task, then emit. Called once per second by the tick thread.
    pub fn tick(&self, app: &AppHandle, elapsed_secs: u32) {
        let changes = self.with(|m| {
            let settings = m.settings.clone();
            let changes = m.timer.advance(elapsed_secs, &settings);
            for change in &changes {
                if change.from == Phase::Focus {
                    m.record_focus_phase(change.completed, settings.focus_secs);
                }
            }
            changes
        });

        for change in &changes {
            let _ = app.emit(events::PHASE, change);
        }
        if !changes.is_empty() {
            self.emit_changed(app, Section::Tasks);
        }

        self.emit_tick(app);
        self.save_debounced();
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
        assert_eq!(state.snapshot().tasks.len(), 5);
    }

    #[test]
    fn with_mutates_under_the_lock_and_returns_a_value() {
        let state = AppState::new(store_in("with"));
        let id = state.with(|m| m.add_task("新的".into(), 1));
        assert_eq!(state.snapshot().tasks.len(), 6);
        assert!(state.snapshot().tasks.iter().any(|t| t.id == id));
    }

    #[test]
    fn advance_credits_the_active_task_when_a_focus_phase_completes() {
        let state = AppState::new(store_in("credit"));
        let before = state.snapshot();
        let active = before.timer.active_task.expect("seeded active task");
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
        assert_eq!(state.snapshot().tasks.len(), 5);
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
        fs::write(
            dir.join("state.json"),
            r#"{"schemaVersion":2,"model":{"timer":{"phase":"focus","remainingSecs":99,"running":false,"round":1,"activeTask":null},"tasks":[],"settings":{"accent":"terracotta","tone":"playful","focusSecs":1500,"shortBreakSecs":300,"longBreakSecs":900,"roundsPerCycle":4,"petFlags":{"snapEdges":true,"clickInteract":true,"hideFullscreen":true,"sleepAnimation":false}},"nextTaskId":0}}"#,
        )
        .expect("write");

        let model = Store::new(&dir).load();
        assert_eq!(model.timer.remaining_secs, 99);
        assert_eq!(model.stats.sessions.len(), 0);
        assert_eq!(model.pet.selected, 0);
    }
}
