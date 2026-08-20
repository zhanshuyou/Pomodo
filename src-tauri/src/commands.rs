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
