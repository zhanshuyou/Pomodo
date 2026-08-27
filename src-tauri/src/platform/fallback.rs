use tauri::WebviewWindow;

use super::PlatformWindows;
use crate::core::desk::ScreenRect;

/// Linux and Windows get plain always-on-top windows. The pet still floats, drags
/// and snaps; it just cannot follow spaces or detect a full-screen app.
pub struct Fallback;

impl PlatformWindows for Fallback {
    fn make_desktop_layer(&self, window: &WebviewWindow) {
        let _ = window.set_always_on_top(true);
        let _ = window.set_skip_taskbar(true);
        let _ = window.set_decorations(false);
    }

    fn make_overlay_layer(&self, window: &WebviewWindow) {
        let _ = window.set_always_on_top(true);
        let _ = window.set_decorations(false);
    }

    fn set_click_through(&self, window: &WebviewWindow, ignore: bool) {
        let _ = window.set_ignore_cursor_events(ignore);
    }

    fn fullscreen_app_covering(&self, _screen: ScreenRect) -> bool {
        false
    }

    fn microphone_in_use(&self) -> bool {
        false
    }
}
