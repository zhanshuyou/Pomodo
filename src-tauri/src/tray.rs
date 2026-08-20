use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::windows;

pub const TRAY_ID: &str = "pomodo-tray";

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "打开 Pomodo", true, None::<&str>)?;
    let prefs = MenuItem::with_id(app, "prefs", "设置…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出 Pomodo", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &prefs, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().expect("bundled icon"))
        .icon_as_template(true)
        .menu(&menu)
        // The menu is the right-click affordance only; a left click opens the popover.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => {
                let _ = windows::show_main(app);
            }
            "prefs" => {
                let _ = windows::show_prefs(app);
            }
            "quit" => {
                if let Some(state) = app.try_state::<crate::state::AppState>() {
                    state.flush();
                }
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let visible = app
                    .get_webview_window("tray")
                    .and_then(|w| w.is_visible().ok())
                    .unwrap_or(false);
                let _ = if visible {
                    windows::hide_tray(app)
                } else {
                    windows::show_tray(app)
                };
            }
        })
        .build(app)?;

    Ok(())
}

/// Refresh the countdown shown next to the menu-bar icon.
pub fn set_title(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_title(Some(text));
    }
}
