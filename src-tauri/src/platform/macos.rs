use std::ffi::c_void;

use objc2_app_kit::{
    NSApplication, NSApplicationPresentationOptions, NSFloatingWindowLevel,
    NSScreenSaverWindowLevel, NSWindow, NSWindowCollectionBehavior, NSWindowLevel,
};
use objc2_core_foundation::{CFDictionary, CFNumber, CFNumberType, CFString, CGRect};
use objc2_core_graphics::{
    kCGNullWindowID, kCGWindowBounds, kCGWindowLayer, kCGWindowOwnerPID,
    CGRectMakeWithDictionaryRepresentation, CGWindowListCopyWindowInfo, CGWindowListOption,
};
use objc2_foundation::MainThreadMarker;
use tauri::WebviewWindow;

use super::PlatformWindows;
use crate::core::desk::ScreenRect;

pub struct MacOs;

/// Apply the AppKit settings Tauri's window builder cannot express. Runs on
/// the main thread because that is where NSWindow may be touched; the raw
/// pointer travels as an address so the closure stays `Send`.
fn configure(window: &WebviewWindow, level: NSWindowLevel, behaviour: NSWindowCollectionBehavior) {
    let Ok(ptr) = window.ns_window() else {
        return;
    };
    let addr = ptr as usize;
    let _ = window.run_on_main_thread(move || {
        // SAFETY: Tauri handed out the pointer for a window it still owns, and
        // AppKit window objects are only touched on the main thread — which
        // run_on_main_thread guarantees.
        let ns: &NSWindow = unsafe { &*(addr as *const NSWindow) };
        ns.setLevel(level);
        ns.setCollectionBehavior(behaviour);
        // Never vanish just because Pomodo stopped being the active app.
        ns.setHidesOnDeactivate(false);
    });
}

impl PlatformWindows for MacOs {
    fn make_desktop_layer(&self, window: &WebviewWindow) {
        let _ = window.set_always_on_top(true);
        let _ = window.set_skip_taskbar(true);
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(false);
        let _ = window.set_visible_on_all_workspaces(true);
        // Floating level keeps it above documents but under menus and
        // alerts; CanJoinAllSpaces + FullScreenAuxiliary let it follow the
        // user into another app's full-screen Space instead of staying
        // behind on the desktop; Stationary keeps Mission Control from
        // sweeping it up. A true non-activating NSPanel would need the
        // window created as a panel, which Tauri does not offer — the
        // `focused(false)` builder flag covers the activation half.
        configure(
            window,
            NSFloatingWindowLevel,
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
    }

    fn make_overlay_layer(&self, window: &WebviewWindow) {
        let _ = window.set_always_on_top(true);
        let _ = window.set_decorations(false);
        let _ = window.set_shadow(false);
        let _ = window.set_visible_on_all_workspaces(true);
        // Sized and placed per monitor by windows.rs rather than set_fullscreen:
        // a native fullscreen window would get its own Space. Screen-saver
        // level is what it takes to sit over a full-screen app's own Space.
        configure(
            window,
            NSScreenSaverWindowLevel,
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary,
        );
    }

    fn set_click_through(&self, window: &WebviewWindow, ignore: bool) {
        let _ = window.set_ignore_cursor_events(ignore);
    }

    /// Two checks, cheapest first. The active app's presentation options say
    /// whether *any* full-screen Space is up — a menu-bar auto-hide user does
    /// not trip this, which the old "is the menu bar gone" heuristic did.
    /// Then the on-screen window list says whether it is on *this* display.
    fn fullscreen_app_covering(&self, screen: ScreenRect) -> bool {
        let Some(mtm) = MainThreadMarker::new() else {
            return false;
        };
        let options = NSApplication::sharedApplication(mtm).currentSystemPresentationOptions();
        if !options.contains(NSApplicationPresentationOptions::FullScreen) {
            return false;
        }
        foreign_window_fills(screen)
    }

    fn microphone_in_use(&self) -> bool {
        microphone::is_running_somewhere()
    }
}

fn dict_number(dict: &CFDictionary, key: &CFString) -> Option<i64> {
    // SAFETY: CGWindowList dictionaries are keyed by these CFStrings and hold
    // CFNumbers for layer and owner pid; a missing key comes back null.
    let ptr = unsafe { dict.value(key as *const CFString as *const c_void) };
    if ptr.is_null() {
        return None;
    }
    let number: &CFNumber = unsafe { &*(ptr as *const CFNumber) };
    let mut value: i64 = 0;
    let ok = unsafe {
        number.value(
            CFNumberType::SInt64Type,
            &mut value as *mut i64 as *mut c_void,
        )
    };
    ok.then_some(value)
}

/// Does any other process have a normal-layer window exactly the size of
/// `screen` on screen right now? That is what a full-screen app looks like
/// from outside. Bounds and Tauri monitors share the same points /
/// top-left-origin space.
fn foreign_window_fills(screen: ScreenRect) -> bool {
    let options =
        CGWindowListOption::OptionOnScreenOnly | CGWindowListOption::ExcludeDesktopElements;
    let Some(list) = CGWindowListCopyWindowInfo(options, kCGNullWindowID) else {
        return false;
    };
    let me = std::process::id() as i64;
    let close = |a: f64, b: f64| (a - b).abs() < 1.0;
    for i in 0..list.count() {
        // SAFETY: the array holds CFDictionary entries for the lifetime of `list`.
        let entry = unsafe { list.value_at_index(i) } as *const CFDictionary;
        if entry.is_null() {
            continue;
        }
        let dict: &CFDictionary = unsafe { &*entry };
        // SAFETY: reading extern statics exported by CoreGraphics.
        let (layer_key, pid_key, bounds_key) =
            unsafe { (kCGWindowLayer, kCGWindowOwnerPID, kCGWindowBounds) };
        if dict_number(dict, layer_key) != Some(0) || dict_number(dict, pid_key) == Some(me) {
            continue;
        }
        let bounds = unsafe { dict.value(bounds_key as *const CFString as *const c_void) }
            as *const CFDictionary;
        if bounds.is_null() {
            continue;
        }
        let mut rect = CGRect::default();
        // SAFETY: `bounds` is the CFDictionary CoreGraphics wrote for this key.
        let ok = unsafe { CGRectMakeWithDictionaryRepresentation(Some(&*bounds), &mut rect) };
        if !ok {
            continue;
        }
        if close(rect.origin.x, screen.x)
            && close(rect.origin.y, screen.y)
            && close(rect.size.width, screen.width)
            && close(rect.size.height, screen.height)
        {
            return true;
        }
    }
    false
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
