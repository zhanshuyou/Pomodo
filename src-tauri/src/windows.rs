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

/// Put the main window away without closing it — the timer keeps running.
pub fn hide_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{Emitter, LogicalPosition, LogicalSize, WebviewUrl, WebviewWindowBuilder};

pub use crate::core::desk::MINI_SIZE;
use crate::core::desk::{
    clamp_mini_height, clamp_to_screen, pet_should_show, Placement, ScreenRect,
};
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

/// True unless the user has dismissed the pet — or mini mode is on, in which
/// case the bar's own cat stands in for it.
fn pet_wanted(app: &AppHandle) -> bool {
    app.try_state::<crate::state::AppState>()
        .map(|s| s.with(|m| pet_should_show(m.settings.pet_visible, m.mini_enabled)))
        .unwrap_or(true)
}

fn mini_wanted(app: &AppHandle) -> bool {
    app.try_state::<crate::state::AppState>()
        .map(|s| s.with(|m| m.mini_enabled))
        .unwrap_or(false)
}

/// Create the mini bar if needed, give it the same desktop-layer treatment the
/// pet gets, and put it back where the user left it.
pub fn ensure_mini(app: &AppHandle) -> tauri::Result<()> {
    if !mini_wanted(app) {
        hide_mini(app);
        return Ok(());
    }
    let window = match app.get_webview_window("mini") {
        Some(w) => w,
        None => WebviewWindowBuilder::new(app, "mini", WebviewUrl::App("mini.html".into()))
            .inner_size(MINI_SIZE.0, MINI_SIZE.1)
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

    // A bar left expanded by a reminder must come back at its resting height.
    window.set_size(LogicalSize::new(MINI_SIZE.0, MINI_SIZE.1))?;

    let screen = primary_screen_rect(app);
    let stored = app
        .try_state::<crate::state::AppState>()
        .and_then(|s| s.with(|m| m.mini_placement));
    // Default to the artboard's top-right corner, clear of the menu bar.
    let placement = stored.unwrap_or(Placement {
        x: screen.x + screen.width - MINI_SIZE.0 - 24.0,
        y: screen.y + 40.0,
    });
    let placement = clamp_to_screen(placement, MINI_SIZE, screen);
    window.set_position(LogicalPosition::new(placement.x, placement.y))?;
    if !window.is_visible().unwrap_or(false) {
        window.show()?;
    }
    Ok(())
}

/// Enter or leave 迷你模式. Entering puts the main window away and stands the
/// bar's own cat in for the desktop pet; leaving restores both. Lives here
/// rather than in `commands` because the tray and the global hotkey flip it too.
pub fn set_mini(app: &AppHandle, value: bool) -> tauri::Result<()> {
    if let Some(state) = app.try_state::<crate::state::AppState>() {
        state.with(|m| m.mini_enabled = value);
        state.emit_changed(app, crate::events::Section::Settings);
        state.flush();
    }
    if value {
        hide_main(app);
        hide_pet(app);
        ensure_mini(app)?;
    } else {
        hide_mini(app);
        // ensure_pet consults pet_visible itself, so a dismissed pet stays gone.
        let _ = ensure_pet(app);
        show_main(app)?;
    }
    Ok(())
}

pub fn toggle_mini(app: &AppHandle) -> tauri::Result<()> {
    let next = app
        .try_state::<crate::state::AppState>()
        .map(|s| s.with(|m| !m.mini_enabled))
        .unwrap_or(true);
    set_mini(app, next)
}

pub fn hide_mini(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("mini") {
        let _ = window.hide();
    }
}

/// Resize the bar to the height it just measured for itself. The top-left
/// corner stays put, so a bar parked against the top edge grows downward.
pub fn set_mini_height(app: &AppHandle, height: f64) {
    let Some(window) = app.get_webview_window("mini") else {
        return;
    };
    let _ = window.set_size(LogicalSize::new(MINI_SIZE.0, clamp_mini_height(height)));
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
    let placement = stored.unwrap_or(Placement {
        x: screen.x + 118.0,
        y: screen.y + screen.height - PET_SIZE.1 - 92.0,
    });
    let placement = clamp_to_screen(placement, PET_SIZE, screen);
    window.set_position(LogicalPosition::new(placement.x, placement.y))?;
    if !window.is_visible().unwrap_or(false) {
        window.show()?;
    }
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
    let wanted = pet_wanted(app);
    let _ = app.run_on_main_thread(move || {
        let Some(window) = handle.get_webview_window("pet") else {
            return;
        };
        // A dismissed pet stays dismissed; only then does full-screen matter.
        let should_show =
            wanted && !(hide_when_fullscreen && platform().fullscreen_app_frontmost());
        // Only act on a real change. `show()` maps to makeKeyAndOrderFront on
        // macOS, so calling it every tick repeatedly stole key status from
        // whatever was focused — which closed the tray popover a second after
        // it opened.
        if should_show == window.is_visible().unwrap_or(false) {
            return;
        }
        let _ = if should_show {
            window.show()
        } else {
            window.hide()
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

/// Tauri's `close()` is async, so a same-tick double-fire can call `show_overlay`
/// again before the previous windows have actually gone away. Tagging every
/// call with its own generation keeps the new labels from colliding with a
/// predecessor still mid-close, instead of racing `WebviewWindowBuilder::build`
/// against it under the same label.
static OVERLAY_GENERATION: AtomicU64 = AtomicU64::new(0);

fn overlay_label(generation: u64, index: usize) -> String {
    format!("overlay-{generation}-{index}")
}

/// 全屏遮罩 — one window per connected monitor, torn down on dismissal.
pub fn show_overlay<P: Serialize + Clone>(app: &AppHandle, payload: P) -> tauri::Result<()> {
    dismiss_overlays(app);
    let generation = OVERLAY_GENERATION.fetch_add(1, Ordering::SeqCst);
    for (index, monitor) in app.available_monitors()?.iter().enumerate() {
        let label = overlay_label(generation, index);
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

/// Every window label the app creates. A label missing from the capability file
/// gets no permissions at all, and the only symptom is an invoke that quietly
/// fails inside a window nobody can attach a debugger to — so it is checked here.
#[cfg(test)]
const CREATED_LABELS: &[&str] = &[
    "main",
    "prefs",
    "tray",
    "pet",
    "bubble",
    "mini",
    "overlay-0",
    "overlay-1",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Tauri matches capability windows as globs; `*` stands for any run of
    /// characters. Only the trailing form is used here.
    fn label_matches(pattern: &str, label: &str) -> bool {
        match pattern.strip_suffix('*') {
            Some(prefix) => label.starts_with(prefix),
            None => pattern == label,
        }
    }

    fn capability_windows() -> Vec<String> {
        let raw = include_str!("../capabilities/default.json");
        let value: serde_json::Value = serde_json::from_str(raw).expect("capability json");
        value["windows"]
            .as_array()
            .expect("windows array")
            .iter()
            .map(|v| v.as_str().expect("window label").to_string())
            .collect()
    }

    #[test]
    fn every_window_the_app_creates_is_covered_by_the_capability() {
        let patterns = capability_windows();
        for label in CREATED_LABELS {
            assert!(
                patterns.iter().any(|p| label_matches(p, label)),
                "window {label:?} has no capability entry; it would launch with no permissions"
            );
        }
    }

    #[test]
    fn a_bare_prefix_does_not_match_a_suffixed_label() {
        assert!(!label_matches("overlay", "overlay-0"));
        assert!(label_matches("overlay-*", "overlay-0"));
        assert!(label_matches("mini", "mini"));
    }

    #[test]
    fn overlay_labels_never_repeat_across_generations() {
        // Two fullscreen reminders firing in the same tick call show_overlay
        // twice; each call must mint labels the other cannot still be using.
        assert_ne!(overlay_label(0, 0), overlay_label(1, 0));
        assert_ne!(overlay_label(0, 1), overlay_label(1, 1));
    }

    #[test]
    fn a_generation_suffixed_overlay_label_still_matches_the_capability_glob() {
        assert!(label_matches("overlay-*", &overlay_label(3, 1)));
    }
}
