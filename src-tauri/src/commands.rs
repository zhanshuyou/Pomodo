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
    state.with(|m| m.timer.start());
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Timer);
    state.flush();
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>, app: AppHandle) {
    state.with(|m| m.timer.pause());
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
    state.with(|m| m.settings.tone = tone);
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

use std::path::Path;

use tauri::Manager;

use crate::core::stats::StatsSummary;

#[tauri::command]
pub fn stats_summary(state: State<'_, AppState>) -> StatsSummary {
    let today = chrono::Utc::now().date_naive();
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
        .join("momo")
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
