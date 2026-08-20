use tauri::{AppHandle, Manager};

/// Show the preferences window, creating it if the config-declared one is gone.
pub fn show_prefs(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("prefs") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(app, "prefs", tauri::WebviewUrl::App("prefs.html".into()))
        .title("设置")
        .inner_size(1180.0, 640.0)
        .min_inner_size(1180.0, 606.0)
        .build()?;
    Ok(())
}

pub fn show_tray(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("tray") {
        // Anchor the popover under the menu-bar item.
        let _ = tauri_plugin_positioner::WindowExt::move_window(
            &window,
            tauri_plugin_positioner::Position::TrayBottomCenter,
        );
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

pub fn hide_tray(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("tray") {
        window.hide()?;
    }
    Ok(())
}

pub fn show_main(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
    }
    Ok(())
}
