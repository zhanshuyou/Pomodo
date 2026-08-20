pub mod commands;
pub mod core;
pub mod events;
pub mod model;
pub mod platform;
pub mod state;
pub mod store;
pub mod tray;
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
        .plugin(tauri_plugin_positioner::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("app data dir")
                .join("pomodo");
            std::fs::create_dir_all(&dir)?;
            app.manage(AppState::new(Store::new(&dir)));

            tray::build(&app.handle().clone())?;
            let _ = windows::ensure_pet(&app.handle().clone());

            // Pomodo lives in the menu bar; once the main window is closed there is
            // no reason for a Dock icon.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

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
        .on_window_event(|window, event| match event {
            // Closing a real window hides it: the timer and reminders keep running,
            // and quitting happens from the tray menu.
            tauri::WindowEvent::CloseRequested { api, .. }
                if matches!(window.label(), "main" | "prefs") =>
            {
                api.prevent_close();
                let _ = window.hide();
            }
            tauri::WindowEvent::Focused(false) if window.label() == "tray" => {
                let _ = window.hide();
            }
            _ => {}
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
            commands::up_next,
            commands::today_summary,
            commands::quit_app,
            commands::show_main,
            commands::set_pet_placement,
            commands::show_pet,
            commands::hide_pet,
            commands::hide_bubble,
            commands::dismiss_overlay,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the Pomodo application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                app.state::<AppState>().flush();
            }
        });
}
