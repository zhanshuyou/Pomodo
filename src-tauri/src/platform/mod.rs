use tauri::WebviewWindow;

use crate::core::desk::ScreenRect;

#[cfg(not(target_os = "macos"))]
mod fallback;
#[cfg(target_os = "macos")]
mod macos;

/// Window behaviours that only macOS can express properly. Every call site goes
/// through this trait so the Linux and Windows builds stay compilable and usable.
pub trait PlatformWindows: Send + Sync {
    /// Float above normal windows, follow the user across spaces, and never steal focus.
    fn make_desktop_layer(&self, window: &WebviewWindow);
    /// Cover everything, including full-screen apps.
    fn make_overlay_layer(&self, window: &WebviewWindow);
    /// Let clicks pass straight through the window's transparent regions.
    fn set_click_through(&self, window: &WebviewWindow, ignore: bool);
    /// True when another application's full-screen window covers `screen`
    /// (logical points, top-left origin — the same space Tauri monitors use).
    fn fullscreen_app_covering(&self, screen: ScreenRect) -> bool;
    /// True when some application is capturing audio input — a meeting or a call.
    fn microphone_in_use(&self) -> bool;
}

#[cfg(target_os = "macos")]
pub fn platform() -> &'static dyn PlatformWindows {
    &macos::MacOs
}

#[cfg(not(target_os = "macos"))]
pub fn platform() -> &'static dyn PlatformWindows {
    &fallback::Fallback
}
