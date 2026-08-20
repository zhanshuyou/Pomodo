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

use serde::Serialize;
use tauri::{Emitter, LogicalPosition, LogicalSize, WebviewUrl, WebviewWindowBuilder};

use crate::core::desk::{clamp_to_screen, PetPlacement, ScreenRect};
use crate::platform::platform;

pub const PET_SIZE: (f64, f64) = (380.0, 200.0);

pub fn primary_screen_rect(app: &AppHandle) -> ScreenRect {
    app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| {
            let scale = m.scale_factor();
            let pos = m.position().to_logical::<f64>(scale);
            let size = m.size().to_logical::<f64>(scale);
            ScreenRect {
                x: pos.x,
                y: pos.y,
                width: size.width,
                height: size.height,
            }
        })
        .unwrap_or(ScreenRect {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        })
}

/// True unless the user has dismissed the pet.
fn pet_wanted(app: &AppHandle) -> bool {
    app.try_state::<crate::state::AppState>()
        .map(|s| s.with(|m| m.settings.pet_visible))
        .unwrap_or(true)
}

/// Create the pet window if needed, give it the desktop layer treatment, and put
/// it back where the user left it. Does nothing while the pet is dismissed.
pub fn ensure_pet(app: &AppHandle) -> tauri::Result<()> {
    if !pet_wanted(app) {
        hide_pet(app);
        return Ok(());
    }
    let window = match app.get_webview_window("pet") {
        Some(w) => w,
        None => WebviewWindowBuilder::new(app, "pet", WebviewUrl::App("pet.html".into()))
            .inner_size(PET_SIZE.0, PET_SIZE.1)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .focused(false)
            .visible(false)
            .build()?,
    };

    platform().make_desktop_layer(&window);

    let screen = primary_screen_rect(app);
    let stored = app
        .try_state::<crate::state::AppState>()
        .and_then(|s| s.with(|m| m.pet_placement));
    // Default to the design's bottom-left corner placement.
    let placement = stored.unwrap_or(PetPlacement {
        x: screen.x + 118.0,
        y: screen.y + screen.height - PET_SIZE.1 - 92.0,
    });
    let placement = clamp_to_screen(placement, PET_SIZE, screen);
    window.set_position(LogicalPosition::new(placement.x, placement.y))?;
    window.show()?;
    Ok(())
}

pub fn hide_pet(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("pet") {
        let _ = window.hide();
    }
}

/// Hide the pet while a full-screen app owns the screen, when 全屏时隐藏 is on.
/// The macOS check needs the main thread, so it is hopped there.
pub fn sync_pet_visibility(app: &AppHandle, hide_when_fullscreen: bool) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(window) = handle.get_webview_window("pet") else {
            return;
        };
        let should_hide = hide_when_fullscreen && platform().fullscreen_app_frontmost();
        let _ = if should_hide {
            window.hide()
        } else {
            window.show()
        };
    });
}

/// 轻量气泡 — slide in at the top-right of the primary screen.
pub fn show_bubble<P: Serialize + Clone>(app: &AppHandle, payload: P) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("bubble") else {
        return Ok(());
    };
    platform().make_desktop_layer(&window);
    let screen = primary_screen_rect(app);
    window.set_position(LogicalPosition::new(
        screen.x + screen.width - 360.0 - 24.0,
        screen.y + 40.0,
    ))?;
    window.show()?;
    app.emit_to("bubble", "bubble:show", payload)?;
    Ok(())
}

pub fn hide_bubble(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("bubble") {
        let _ = window.hide();
    }
}

/// 全屏遮罩 — one window per connected monitor, torn down on dismissal.
pub fn show_overlay<P: Serialize + Clone>(app: &AppHandle, payload: P) -> tauri::Result<()> {
    dismiss_overlays(app);
    for (index, monitor) in app.available_monitors()?.iter().enumerate() {
        let label = format!("overlay-{index}");
        let scale = monitor.scale_factor();
        let pos = monitor.position().to_logical::<f64>(scale);
        let size = monitor.size().to_logical::<f64>(scale);

        let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("overlay.html".into()))
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .visible(false)
            .build()?;

        platform().make_overlay_layer(&window);
        window.set_position(LogicalPosition::new(pos.x, pos.y))?;
        window.set_size(LogicalSize::new(size.width, size.height))?;
        window.show()?;
        window.set_focus()?;
        app.emit_to(label.as_str(), "overlay:show", payload.clone())?;
    }
    Ok(())
}

pub fn dismiss_overlays(app: &AppHandle) {
    for (label, window) in app.webview_windows() {
        if label.starts_with("overlay-") {
            let _ = window.close();
        }
    }
}
