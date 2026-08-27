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

/// ⌘⌥M anywhere flips 迷你模式 — the point of the mode is that you are inside
/// some other app, so an in-window binding alone would not reach you. A hotkey
/// another app already owns is a warning, never a failed launch.
fn register_mini_hotkey(app: &tauri::AppHandle) {
    use tauri_plugin_global_shortcut::{
        Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
    };

    let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyM);
    let plugin = tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |app, pressed, event| {
            if pressed == &shortcut && event.state() == ShortcutState::Pressed {
                let _ = windows::toggle_mini(app);
            }
        })
        .build();

    if let Err(err) = app.plugin(plugin) {
        eprintln!("pomodo: global shortcut plugin unavailable: {err}");
        return;
    }
    if let Err(err) = app.global_shortcut().register(shortcut) {
        eprintln!("pomodo: ⌘⌥M is taken by another app: {err}");
    }
}

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
            // 迷你模式 is remembered across launches: come back up as the bar,
            // with the main window still put away.
            let handle = app.handle().clone();
            if handle.state::<AppState>().with(|m| m.mini_enabled) {
                windows::hide_main(&handle);
                let _ = windows::ensure_mini(&handle);
            }
            register_mini_hotkey(&handle);

            // Pomodo lives in the menu bar; once the main window is closed there is
            // no reason for a Dock icon.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Click-through for the pet window: see windows::poll_pet_hit.
            let handle = app.handle().clone();
            thread::spawn(move || loop {
                thread::sleep(windows::PET_HIT_POLL);
                windows::poll_pet_hit(&handle);
            });

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
            // Clicking anywhere outside the popover dismisses it. The guard keeps
            // this to blurs that actually close a visible popover, so a stale
            // timestamp cannot swallow a later opening click.
            tauri::WindowEvent::Focused(false)
                if window.label() == "tray" && window.is_visible().unwrap_or(false) =>
            {
                let _ = window.hide();
                tray::note_blur_hide();
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
            commands::rename_task,
            commands::set_task_estimate,
            commands::reorder_tasks,
            commands::set_accent,
            commands::set_tone,
            commands::set_pet_flag,
            commands::set_timer_durations,
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
            commands::snooze_overlay,
            commands::set_deep_work,
            commands::open_prefs,
            commands::up_next,
            commands::today_summary,
            commands::quit_app,
            commands::show_main,
            commands::set_mini_mode,
            commands::toggle_mini_mode,
            commands::set_mini_height,
            commands::set_window_height,
            commands::set_mini_placement,
            commands::pet_interacted,
            commands::set_pet_hit_rects,
            commands::set_pet_dragging,
            commands::set_pet_placement,
            commands::show_pet,
            commands::hide_pet,
            commands::set_pet_visible,
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
