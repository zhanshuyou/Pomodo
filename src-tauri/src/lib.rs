pub mod commands;
pub mod core;
pub mod events;
pub mod model;
pub mod state;
pub mod store;
pub mod windows;

use std::thread;
use std::time::{Duration, Instant};

use tauri::Manager;

use crate::state::AppState;
use crate::store::Store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("app data dir")
                .join("momo");
            std::fs::create_dir_all(&dir)?;
            app.manage(AppState::new(Store::new(&dir)));

            let handle = app.handle().clone();
            thread::spawn(move || {
                // Elapsed time comes from a monotonic instant, not from counting
                // iterations: if the machine sleeps for an hour, the next wake passes
                // the whole gap to advance() and every crossed phase is reported.
                let mut previous = Instant::now();
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let now = Instant::now();
                    let elapsed = now.duration_since(previous).as_secs();
                    previous = now;
                    if elapsed == 0 {
                        continue;
                    }
                    let state = handle.state::<AppState>();
                    state.tick(&handle, elapsed as u32);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_model,
            commands::start,
            commands::pause,
            commands::skip_phase,
            commands::set_active_task,
            commands::add_task,
            commands::toggle_task,
            commands::delete_task,
            commands::set_accent,
            commands::set_tone,
            commands::set_pet_flag,
            commands::stats_summary,
            commands::select_pet,
            commands::set_use_custom_pet,
            commands::import_custom_pet,
            commands::clear_custom_pet,
            commands::add_reminder,
            commands::update_reminder,
            commands::toggle_reminder,
            commands::delete_reminder,
            commands::ack_reminder,
            commands::ignore_reminder,
            commands::snooze_reminder,
            commands::set_deep_work,
            commands::open_prefs,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the Momo application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                app.state::<AppState>().flush();
            }
        });
}
