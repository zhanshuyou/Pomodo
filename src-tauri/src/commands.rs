use tauri::{AppHandle, Emitter, State};

use crate::events::{self, Section};
use crate::model::{Accent, Model, TaskId, Tone};
use crate::state::AppState;

#[tauri::command]
pub fn list_model(state: State<'_, AppState>) -> Model {
    state.snapshot()
}

#[tauri::command]
pub fn start(state: State<'_, AppState>, app: AppHandle) {
    state.with(|m| {
        m.timer.start();
        m.touch();
    });
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Timer);
    state.flush();
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>, app: AppHandle) {
    state.with(|m| {
        m.timer.pause();
        m.touch();
    });
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Timer);
    state.flush();
}

#[tauri::command]
pub fn skip_phase(state: State<'_, AppState>, app: AppHandle) {
    let change = state.with(|m| {
        let settings = m.settings.clone();
        let was_focus = m.timer.phase == crate::model::Phase::Focus;
        // Credit only the time actually served before the user bailed.
        let elapsed = settings.focus_secs.saturating_sub(m.timer.remaining_secs);
        let change = m.timer.skip(&settings);
        if was_focus {
            m.record_focus_phase(false, elapsed);
        }
        m.touch();
        change
    });
    let _ = app.emit(events::PHASE, change);
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Tasks);
    state.flush();
}

#[tauri::command]
pub fn set_active_task(state: State<'_, AppState>, app: AppHandle, id: Option<TaskId>) {
    state.with(|m| m.timer.active_task = id);
    state.emit_changed(&app, Section::Timer);
    state.flush();
}

#[tauri::command]
pub fn add_task(state: State<'_, AppState>, app: AppHandle, name: String, estimate: u8) -> TaskId {
    let id = state.with(|m| m.add_task(name, estimate));
    state.emit_changed(&app, Section::Tasks);
    state.flush();
    id
}

#[tauri::command]
pub fn toggle_task(state: State<'_, AppState>, app: AppHandle, id: TaskId) {
    state.with(|m| m.toggle_task(id));
    state.emit_changed(&app, Section::Tasks);
    state.flush();
}

#[tauri::command]
pub fn rename_task(state: State<'_, AppState>, app: AppHandle, id: TaskId, name: String) {
    if state.with(|m| m.rename_task(id, &name)) {
        state.emit_changed(&app, Section::Tasks);
        state.flush();
    }
}

#[tauri::command]
pub fn set_task_estimate(state: State<'_, AppState>, app: AppHandle, id: TaskId, estimate: u8) {
    if state.with(|m| m.set_task_estimate(id, estimate)) {
        state.emit_changed(&app, Section::Tasks);
        state.flush();
    }
}

#[tauri::command]
pub fn reorder_tasks(state: State<'_, AppState>, app: AppHandle, ids: Vec<TaskId>) {
    state.with(|m| m.reorder_tasks(&ids));
    state.emit_changed(&app, Section::Tasks);
    state.flush();
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, app: AppHandle, id: TaskId) {
    state.with(|m| m.delete_task(id));
    state.emit_changed(&app, Section::Tasks);
    state.flush();
}

#[tauri::command]
pub fn set_accent(state: State<'_, AppState>, app: AppHandle, accent: Accent) {
    state.with(|m| m.settings.accent = accent);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

#[tauri::command]
pub fn set_tone(state: State<'_, AppState>, app: AppHandle, tone: Tone) {
    state.with(|m| {
        m.settings.tone = tone;
        m.retone_reminders();
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

/// `flag` is one of snapEdges / clickInteract / hideFullscreen / sleepAnimation.
#[tauri::command]
pub fn set_pet_flag(state: State<'_, AppState>, app: AppHandle, flag: String, value: bool) {
    state.with(|m| {
        let f = &mut m.settings.pet_flags;
        match flag.as_str() {
            "snapEdges" => f.snap_edges = value,
            "clickInteract" => f.click_interact = value,
            "hideFullscreen" => f.hide_fullscreen = value,
            "sleepAnimation" => f.sleep_animation = value,
            _ => {}
        }
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

/// Changing a duration only takes effect on the *next* phase transition — the
/// round already in flight keeps counting down against the duration it started
/// with, the same way `Timer::advance` always has. `belly_cells` is the one
/// exception: it reads `settings.duration_for` fresh every tick, so it will
/// jump immediately to reflect the new denominator.
#[tauri::command]
pub fn set_timer_durations(
    state: State<'_, AppState>,
    app: AppHandle,
    focus_secs: u32,
    short_break_secs: u32,
    long_break_secs: u32,
    rounds_per_cycle: u8,
) {
    state.with(|m| {
        m.settings.set_timer_durations(
            focus_secs,
            short_break_secs,
            long_break_secs,
            rounds_per_cycle,
        )
    });
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

use std::path::Path;

use tauri::Manager;

use crate::core::stats::StatsSummary;

#[tauri::command]
pub fn stats_summary(state: State<'_, AppState>) -> StatsSummary {
    let today = chrono::Local::now().date_naive();
    state.with(|m| m.stats.summary(today))
}

#[tauri::command]
pub fn select_pet(state: State<'_, AppState>, app: AppHandle, id: u8) -> bool {
    let ok = state.with(|m| m.pet.select(id));
    if ok {
        state.emit_changed(&app, Section::Settings);
        state.flush();
    }
    ok
}

#[tauri::command]
pub fn set_use_custom_pet(state: State<'_, AppState>, app: AppHandle, value: bool) {
    state.with(|m| m.pet.use_custom = value);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

/// Copy a user-chosen image into the app's pets directory and remember its path.
/// `slot` is one of focus / rest / nag.
#[tauri::command]
pub fn import_custom_pet(
    state: State<'_, AppState>,
    app: AppHandle,
    slot: String,
    source: String,
) -> Result<String, String> {
    let allowed = ["png", "gif", "apng", "webp"];
    let src = Path::new(&source);
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !allowed.contains(&ext.as_str()) {
        return Err(format!("不支持的图片格式：{ext}"));
    }
    if !matches!(slot.as_str(), "focus" | "rest" | "nag") {
        return Err(format!("未知的槽位：{slot}"));
    }

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("pomodo")
        .join("pets");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let dest = dir.join(format!("{slot}.{ext}"));
    std::fs::copy(src, &dest).map_err(|e| e.to_string())?;
    let stored = dest.to_string_lossy().into_owned();

    state.with(|m| {
        let c = &mut m.pet.custom;
        match slot.as_str() {
            "focus" => c.focus = Some(stored.clone()),
            "rest" => c.rest = Some(stored.clone()),
            "nag" => c.nag = Some(stored.clone()),
            _ => {}
        }
        m.pet.use_custom = true;
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
    Ok(stored)
}

#[tauri::command]
pub fn clear_custom_pet(state: State<'_, AppState>, app: AppHandle, slot: String) {
    state.with(|m| {
        let c = &mut m.pet.custom;
        match slot.as_str() {
            "focus" => c.focus = None,
            "rest" => c.rest = None,
            "nag" => c.nag = None,
            _ => {}
        }
        if c.focus.is_none() && c.rest.is_none() && c.nag.is_none() {
            m.pet.use_custom = false;
        }
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

use serde::{Deserialize, Serialize};

use crate::core::reminder::{Intensity, Reminder, Rules, Schedule};
use crate::core::reminder_copy::Builtin;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderPatch {
    pub name: Option<String>,
    pub message: Option<String>,
    /// Older shorthand for `schedule: Every { minutes }`; `schedule` wins if both are set.
    pub interval_minutes: Option<u32>,
    pub schedule: Option<Schedule>,
    pub intensity: Option<Intensity>,
    pub enabled: Option<bool>,
    pub rules: Option<Rules>,
}

/// Template chips from the design's 从模板抓一个 row.
fn template_color(name: &str) -> &'static str {
    match name {
        "站立" => "oklch(0.63 0.13 40)",
        "喝水" => "oklch(0.66 0.09 195)",
        "护眼" => "oklch(0.7 0.1 145)",
        "深呼吸" => "oklch(0.68 0.1 300)",
        "肩颈拉伸" => "oklch(0.7 0.12 60)",
        "记一句想法" => "oklch(0.62 0.07 250)",
        _ => "oklch(0.63 0.13 40)",
    }
}

#[tauri::command]
pub fn add_reminder(state: State<'_, AppState>, app: AppHandle, template: Option<String>) -> u32 {
    let id = state.with(|m| {
        let id = m.next_reminder_id;
        m.next_reminder_id += 1;
        let name = template.clone().unwrap_or_else(|| "新提醒".to_string());
        let color = template_color(&name).to_string();
        m.reminders.push(Reminder::blank(id, name, color));
        id
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
    id
}

#[tauri::command]
pub fn update_reminder(state: State<'_, AppState>, app: AppHandle, id: u32, patch: ReminderPatch) {
    use chrono::{Datelike, Timelike};
    let now = chrono::Local::now();
    let now_minute = (now.hour() * 60 + now.minute()) as u16;
    let today = now.date_naive().num_days_from_ce();
    state.with(|m| {
        let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) else {
            return;
        };
        if let Some(name) = patch.name {
            r.name = name;
        }
        if let Some(message) = patch.message {
            r.message = message;
            // Once the user writes their own words, a tone change must not overwrite them.
            r.message_edited = true;
        }
        let schedule = patch.schedule.or(patch
            .interval_minutes
            .map(|minutes| Schedule::Every { minutes }));
        if let Some(schedule) = schedule {
            r.set_schedule(schedule, now_minute, today);
        }
        if let Some(intensity) = patch.intensity {
            r.intensity = intensity;
        }
        if let Some(enabled) = patch.enabled {
            r.enabled = enabled;
        }
        if let Some(rules) = patch.rules {
            r.rules = rules;
        }
        r.refresh_detail();
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn toggle_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            r.enabled = !r.enabled;
        }
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn delete_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| m.reminders.retain(|r| r.id != id));
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

/// The user did the thing. Clears the ignore streak and moves the body counters.
#[tauri::command]
pub fn ack_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| {
        let builtin = m.reminders.iter_mut().find(|r| r.id == id).and_then(|r| {
            r.acknowledge();
            r.builtin
        });
        match builtin {
            Some(Builtin::Water) => m.body.water_cups += 1,
            Some(Builtin::Stand) => m.body.stand_up(),
            _ => {}
        }
        m.end_nag(id);
        m.touch();
    });
    state.emit_changed(&app, Section::Body);
    state.flush();
}

#[tauri::command]
pub fn ignore_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            r.ignore();
        }
        m.end_nag(id);
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn snooze_reminder(state: State<'_, AppState>, app: AppHandle, id: u32, minutes: u32) {
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            r.remaining_secs = minutes.saturating_mul(60).max(1);
            r.deferred = false;
        }
        m.end_nag(id);
        m.touch();
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn set_deep_work(state: State<'_, AppState>, app: AppHandle, value: bool) {
    state.with(|m| m.deep_work = value);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

#[tauri::command]
pub fn open_prefs(app: AppHandle) -> Result<(), String> {
    crate::windows::show_prefs(&app).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpNextItem {
    pub id: u32,
    pub name: String,
    pub color: String,
    pub due: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TodaySummary {
    pub pomodoros: u32,
    pub focus_secs: u32,
    pub label: String,
}

/// Human wait time for the tray's 接下来轮到 column.
fn due_label(secs: u32) -> String {
    let mins = secs / 60;
    if mins == 0 {
        return "马上".to_string();
    }
    if mins < 60 {
        return format!("{mins} 分钟后");
    }
    let hours = mins / 60;
    let rest = mins % 60;
    if rest == 0 {
        format!("{hours} 小时后")
    } else {
        format!("{hours} 小时 {rest} 分后")
    }
}

/// The design's footer line: 今天 5 个番茄 · 2h05m
fn today_label(pomodoros: u32, focus_secs: u32) -> String {
    let h = focus_secs / 3600;
    let m = (focus_secs % 3600) / 60;
    format!("今天 {pomodoros} 个番茄 · {h}h{m:02}m")
}

#[tauri::command]
pub fn up_next(state: State<'_, AppState>) -> Vec<UpNextItem> {
    let now = chrono::Local::now();
    state.with(|m| {
        // The list shows what is scheduled, regardless of whether a meeting would
        // currently silence it.
        let ctx = m.fire_context(now, false);
        let mut items: Vec<(u32, UpNextItem)> = m
            .reminders
            .iter()
            .filter_map(|r| {
                r.seconds_until_due(&ctx).map(|secs| {
                    (
                        secs,
                        UpNextItem {
                            id: r.id,
                            name: r.name.clone(),
                            color: r.color.clone(),
                            due: due_label(secs),
                        },
                    )
                })
            })
            .collect();
        items.sort_by_key(|(secs, _)| *secs);
        items.into_iter().take(3).map(|(_, item)| item).collect()
    })
}

#[tauri::command]
pub fn today_summary(state: State<'_, AppState>) -> TodaySummary {
    let today = chrono::Local::now().date_naive();
    state.with(|m| {
        let counts = m.stats.daily_counts(today, 1);
        let pomodoros = counts.first().copied().unwrap_or(0);
        // Sums each session's own recorded length rather than multiplying by the
        // *current* focus_secs, which goes wrong the moment the duration is ever
        // changed — today's earlier sessions may have run under a different one.
        let focus_secs = m.stats.day_focus_secs(today);
        TodaySummary {
            pomodoros,
            focus_secs,
            label: today_label(pomodoros, focus_secs),
        }
    })
}

#[tauri::command]
pub fn quit_app(state: State<'_, AppState>, app: AppHandle) {
    state.flush();
    app.exit(0);
}

#[tauri::command]
pub fn show_main(app: AppHandle) -> Result<(), String> {
    crate::windows::show_main(&app).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_label_renders_minutes_for_short_waits() {
        assert_eq!(due_label(4 * 60), "4 分钟后");
        assert_eq!(due_label(59), "马上");
        assert_eq!(due_label(0), "马上");
    }

    #[test]
    fn due_label_renders_hours_and_minutes_for_long_waits() {
        assert_eq!(due_label(90 * 60), "1 小时 30 分后");
        assert_eq!(due_label(120 * 60), "2 小时后");
    }

    #[test]
    fn today_label_matches_the_design_footer() {
        assert_eq!(today_label(5, 2 * 3600 + 5 * 60), "今天 5 个番茄 · 2h05m");
        assert_eq!(today_label(0, 0), "今天 0 个番茄 · 0h00m");
    }
}

use crate::core::desk::{snap, Placement};

#[tauri::command]
pub fn set_mini_mode(app: AppHandle, value: bool) -> Result<(), String> {
    crate::windows::set_mini(&app, value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_mini_mode(app: AppHandle) -> Result<(), String> {
    crate::windows::toggle_mini(&app).map_err(|e| e.to_string())
}

/// The bar reports the height it rendered at, growing around a reminder and
/// shrinking back once that reminder is answered or times out.
#[tauri::command]
pub fn set_mini_height(app: AppHandle, height: f64) {
    crate::windows::set_mini_height(&app, height);
}

#[tauri::command]
pub fn set_mini_placement(state: State<'_, AppState>, app: AppHandle, x: f64, y: f64) {
    let snap_edges = state.with(|m| m.settings.pet_flags.snap_edges);
    let screen = crate::windows::primary_screen_rect(&app);
    let mut placement = Placement { x, y };
    if snap_edges {
        placement = snap(placement, crate::windows::MINI_SIZE, screen);
    }
    state.with(|m| m.mini_placement = Some(placement));
    if let Some(window) = tauri::Manager::get_webview_window(&app, "mini") {
        let _ = window.set_position(tauri::LogicalPosition::new(placement.x, placement.y));
    }
    state.flush();
}

/// The user poked the pet. Wakes it if it was dozing; nothing else changes.
#[tauri::command]
pub fn pet_interacted(state: State<'_, AppState>, app: AppHandle) {
    state.with(|m| m.touch());
    state.sync_mood(&app);
}

#[tauri::command]
pub fn set_pet_placement(state: State<'_, AppState>, app: AppHandle, x: f64, y: f64) {
    let snap_edges = state.with(|m| m.settings.pet_flags.snap_edges);
    let screen = crate::windows::primary_screen_rect(&app);
    let mut placement = Placement { x, y };
    if snap_edges {
        placement = snap(placement, crate::windows::PET_SIZE, screen);
    }
    state.with(|m| m.pet_placement = Some(placement));
    if let Some(window) = tauri::Manager::get_webview_window(&app, "pet") {
        let _ = window.set_position(tauri::LogicalPosition::new(placement.x, placement.y));
    }
    state.flush();
}

#[tauri::command]
pub fn show_pet(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    set_pet_visible(state, app.clone(), true);
    crate::windows::ensure_pet(&app).map_err(|e| e.to_string())
}

/// Dismiss the desktop pet. The choice persists, so neither the per-tick
/// visibility sync nor the next launch brings it back uninvited.
#[tauri::command]
pub fn hide_pet(state: State<'_, AppState>, app: AppHandle) {
    set_pet_visible(state, app.clone(), false);
    crate::windows::hide_pet(&app);
}

#[tauri::command]
pub fn set_pet_visible(state: State<'_, AppState>, app: AppHandle, value: bool) {
    state.with(|m| m.settings.pet_visible = value);
    state.emit_changed(&app, Section::Settings);
    state.flush();
    if value {
        let _ = crate::windows::ensure_pet(&app);
    } else {
        crate::windows::hide_pet(&app);
    }
}

#[tauri::command]
pub fn hide_bubble(app: AppHandle) {
    crate::windows::hide_bubble(&app);
}

/// Close the overlay on every display. `acknowledged` distinguishes 做完了 from ⎋.
#[tauri::command]
pub fn dismiss_overlay(state: State<'_, AppState>, app: AppHandle, id: u32, acknowledged: bool) {
    crate::windows::dismiss_overlays(&app);
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            if acknowledged {
                r.acknowledge();
            } else {
                r.ignore();
            }
        }
        m.end_nag(id);
        m.touch();
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}
