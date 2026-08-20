use objc2_app_kit::NSScreen;
use objc2_foundation::MainThreadMarker;
use tauri::WebviewWindow;

use super::PlatformWindows;

pub struct MacOs;

impl PlatformWindows for MacOs {
    fn make_desktop_layer(&self, window: &WebviewWindow) {
        let _ = window.set_always_on_top(true);
        let _ = window.set_skip_taskbar(true);
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(false);
        // Follow the user between spaces instead of living on the one it was born on.
        let _ = window.set_visible_on_all_workspaces(true);
    }

    fn make_overlay_layer(&self, window: &WebviewWindow) {
        let _ = window.set_always_on_top(true);
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(false);
        let _ = window.set_visible_on_all_workspaces(true);
        // Sized and placed per monitor by windows.rs rather than set_fullscreen:
        // a native fullscreen window would get its own Space.
    }

    fn set_click_through(&self, window: &WebviewWindow, ignore: bool) {
        let _ = window.set_ignore_cursor_events(ignore);
    }

    /// When an app goes full-screen the menu bar hides, so the main screen's
    /// visible frame grows to the full frame. That is a cheap, permission-free
    /// signal that something is occupying the whole screen.
    fn fullscreen_app_frontmost(&self) -> bool {
        let Some(mtm) = MainThreadMarker::new() else {
            return false;
        };
        let Some(screen) = NSScreen::mainScreen(mtm) else {
            return false;
        };
        let frame = screen.frame();
        let visible = screen.visibleFrame();
        (frame.size.height - visible.size.height).abs() < 1.0
            && (frame.size.width - visible.size.width).abs() < 1.0
    }

    fn microphone_in_use(&self) -> bool {
        microphone::is_running_somewhere()
    }
}

/// Ask CoreAudio whether the default input device is currently being read by
/// anyone. This is how 检测到会议 / 通话 is detected without any entitlement.
mod microphone {
    use std::mem::size_of;
    use std::ptr;

    use coreaudio_sys::{
        kAudioDevicePropertyDeviceIsRunningSomewhere, kAudioHardwarePropertyDefaultInputDevice,
        kAudioObjectPropertyElementMain, kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject,
        AudioDeviceID, AudioObjectGetPropertyData, AudioObjectPropertyAddress, OSStatus, UInt32,
    };

    fn address(selector: u32) -> AudioObjectPropertyAddress {
        AudioObjectPropertyAddress {
            mSelector: selector,
            mScope: kAudioObjectPropertyScopeGlobal,
            mElement: kAudioObjectPropertyElementMain,
        }
    }

    fn default_input_device() -> Option<AudioDeviceID> {
        let addr = address(kAudioHardwarePropertyDefaultInputDevice);
        let mut device: AudioDeviceID = 0;
        let mut size = size_of::<AudioDeviceID>() as UInt32;
        let status: OSStatus = unsafe {
            AudioObjectGetPropertyData(
                kAudioObjectSystemObject,
                &addr,
                0,
                ptr::null(),
                &mut size,
                &mut device as *mut _ as *mut _,
            )
        };
        if status == 0 && device != 0 {
            Some(device)
        } else {
            None
        }
    }

    pub fn is_running_somewhere() -> bool {
        let Some(device) = default_input_device() else {
            return false;
        };
        let addr = address(kAudioDevicePropertyDeviceIsRunningSomewhere);
        let mut running: UInt32 = 0;
        let mut size = size_of::<UInt32>() as UInt32;
        let status: OSStatus = unsafe {
            AudioObjectGetPropertyData(
                device,
                &addr,
                0,
                ptr::null(),
                &mut size,
                &mut running as *mut _ as *mut _,
            )
        };
        status == 0 && running != 0
    }
}
