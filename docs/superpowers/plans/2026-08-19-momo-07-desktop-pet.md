# Pomodo 07 — Desktop Pet + Reminder Overlays Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put Pomodo on the desktop — a draggable, edge-snapping, always-on-top pet that hides behind fullscreen apps — and render the three reminder intensities from artboard 04 as real windows: the corner bubble, the pet's hop-and-speak nudge, and the multi-display fullscreen overlay.

**Architecture:** A `PlatformWindows` trait isolates every macOS-specific behaviour behind one interface, with a real implementation for macOS and a plain always-on-top fallback that keeps Linux and Windows compiling. The pet lives in its own transparent window whose position is persisted; the bubble is a small top-right window; the overlay spawns one window per connected monitor and tears them down on dismissal. Reminder firings from plan 05 finally reach the screen.

**Tech Stack:** Rust 2021, Tauri 2 multi-window, `objc2` / `objc2-app-kit` (macOS only), `coreaudio-sys` (macOS only), Svelte 5.

**Spec:** `docs/superpowers/specs/2026-08-19-momo-design.md`
**Depends on:** plans 01–06 complete.

## Global Constraints

- macOS-first. Every macOS-only crate goes behind `[target.'cfg(target_os = "macos")'.dependencies]` and every macOS-only call behind `#[cfg(target_os = "macos")]`. The Linux and Windows CI jobs must keep building and the fallback must be a working, if plainer, experience.
- Behaviour flags from spec §8.1 gate the features: 贴边吸附 → edge snapping, 点击互动 → click responses, 全屏时隐藏 → hide behind fullscreen apps, 睡眠动画 → idle sleep pose.
- Bubble: dark `oklch(0.31 0.025 258)`, pet at scale 3, slides in top-right, auto-dismisses after 6 s.
- Pet nudge: pet at scale 4 with `momo-hop`, speech bubble beside it, covers no window.
- Fullscreen overlay: `oklch(0.29 0.025 258)`, pet at scale 3 with `momo-sway`, 30px mono countdown, `站起来走走，看点远的东西`, corner note `按 ⎋ 逃跑（它会记着）`. Escape records an ignore.
- Overlay covers every connected display, one window each.
- The full gate stays green on all three platforms.

---

## File Structure

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/platform/mod.rs` | `PlatformWindows` trait + `platform()` selector |
| `src-tauri/src/platform/macos.rs` | NSPanel-like behaviour, fullscreen detect, mic-in-use |
| `src-tauri/src/platform/fallback.rs` | Plain always-on-top implementation |
| `src-tauri/src/core/desk.rs` | Pet position, edge snapping, pet visual state |
| `src-tauri/src/windows.rs` | pet / bubble / overlay lifecycle |
| `src-tauri/src/commands.rs` | pet position, overlay dismiss, ack from overlay |
| `src/routes/pet/App.svelte` | Desktop pet + nudge bubble |
| `src/routes/bubble/App.svelte`, `bubble.html` | 轻量气泡 |
| `src/routes/overlay/App.svelte` | 全屏遮罩 |

---

### Task 1: Pet position and edge snapping

**Files:**
- Create: `src-tauri/src/core/desk.rs`
- Modify: `src-tauri/src/core/mod.rs`, `src-tauri/src/model.rs`

**Interfaces:**
- Consumes: `Phase`, `PetFlags`.
- Produces:
  - `pub struct PetPlacement { pub x: f64, pub y: f64 }`
  - `pub struct ScreenRect { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }`
  - `pub const SNAP_THRESHOLD: f64 = 48.0`
  - `pub fn snap(placement: PetPlacement, pet: (f64, f64), screen: ScreenRect) -> PetPlacement`
  - `pub fn clamp_to_screen(placement: PetPlacement, pet: (f64, f64), screen: ScreenRect) -> PetPlacement`
  - `pub enum PetMood { Focus, Break, Nagging, Sleeping }`
  - `pub fn mood(phase: Phase, running: bool, nagging: bool, idle_secs: u32, sleep_animation: bool) -> PetMood`
  - `Model` gains `pub pet_placement: Option<PetPlacement>`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/core/desk.rs` with only the test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Phase;

    const PET: (f64, f64) = (128.0, 128.0);

    fn screen() -> ScreenRect {
        ScreenRect {
            x: 0.0,
            y: 0.0,
            width: 1440.0,
            height: 900.0,
        }
    }

    #[test]
    fn a_pet_near_the_left_edge_snaps_flush_to_it() {
        let out = snap(PetPlacement { x: 20.0, y: 400.0 }, PET, screen());
        assert_eq!(out.x, 0.0);
        assert_eq!(out.y, 400.0);
    }

    #[test]
    fn a_pet_near_the_right_edge_snaps_flush_to_it() {
        let out = snap(PetPlacement { x: 1300.0, y: 400.0 }, PET, screen());
        assert_eq!(out.x, 1440.0 - 128.0);
    }

    #[test]
    fn a_pet_near_the_bottom_edge_snaps_flush_to_it() {
        let out = snap(PetPlacement { x: 600.0, y: 800.0 }, PET, screen());
        assert_eq!(out.y, 900.0 - 128.0);
    }

    #[test]
    fn a_pet_in_open_space_does_not_move() {
        let placement = PetPlacement { x: 600.0, y: 400.0 };
        let out = snap(placement, PET, screen());
        assert_eq!(out.x, 600.0);
        assert_eq!(out.y, 400.0);
    }

    #[test]
    fn snapping_prefers_the_nearer_of_two_close_edges() {
        // A 200-wide screen where both edges are within the threshold.
        let narrow = ScreenRect { x: 0.0, y: 0.0, width: 200.0, height: 900.0 };
        let out = snap(PetPlacement { x: 60.0, y: 400.0 }, PET, narrow);
        // Left gap 60, right gap 200 - 128 - 60 = 12 -> snap right.
        assert_eq!(out.x, 72.0);
    }

    #[test]
    fn clamp_pulls_an_off_screen_pet_back_into_view() {
        let out = clamp_to_screen(PetPlacement { x: -400.0, y: 5000.0 }, PET, screen());
        assert_eq!(out.x, 0.0);
        assert_eq!(out.y, 900.0 - 128.0);
    }

    #[test]
    fn clamp_respects_a_screen_origin_offset() {
        let secondary = ScreenRect {
            x: 1440.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        };
        let out = clamp_to_screen(PetPlacement { x: 1000.0, y: 0.0 }, PET, secondary);
        assert_eq!(out.x, 1440.0);
    }

    #[test]
    fn mood_is_focus_while_a_focus_phase_runs() {
        assert_eq!(
            mood(Phase::Focus, true, false, 0, true),
            PetMood::Focus
        );
    }

    #[test]
    fn mood_is_break_during_either_break() {
        assert_eq!(mood(Phase::ShortBreak, true, false, 0, true), PetMood::Break);
        assert_eq!(mood(Phase::LongBreak, true, false, 0, true), PetMood::Break);
    }

    #[test]
    fn nagging_beats_every_other_mood() {
        assert_eq!(mood(Phase::Focus, true, true, 0, true), PetMood::Nagging);
    }

    #[test]
    fn a_long_idle_sleeps_when_the_sleep_animation_flag_is_on() {
        assert_eq!(mood(Phase::Focus, false, false, 600, true), PetMood::Sleeping);
    }

    #[test]
    fn a_long_idle_does_not_sleep_when_the_flag_is_off() {
        assert_eq!(mood(Phase::Focus, false, false, 600, false), PetMood::Focus);
    }
}
```

Add `pub mod desk;` to `src-tauri/src/core/mod.rs`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test core::desk::`
Expected: FAIL — `snap` is not defined.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/core/desk.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::model::Phase;

/// Top-left corner of the pet window, in physical screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetPlacement {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// How close to an edge counts as "near enough to stick".
pub const SNAP_THRESHOLD: f64 = 48.0;

/// Keep the whole pet inside the screen it belongs to.
pub fn clamp_to_screen(
    placement: PetPlacement,
    pet: (f64, f64),
    screen: ScreenRect,
) -> PetPlacement {
    let max_x = screen.x + (screen.width - pet.0).max(0.0);
    let max_y = screen.y + (screen.height - pet.1).max(0.0);
    PetPlacement {
        x: placement.x.clamp(screen.x, max_x),
        y: placement.y.clamp(screen.y, max_y),
    }
}

/// 贴边吸附 — pull the pet flush against whichever edge it was dropped near.
/// Horizontal and vertical are decided independently, and when both edges on one
/// axis are within reach the nearer one wins.
pub fn snap(placement: PetPlacement, pet: (f64, f64), screen: ScreenRect) -> PetPlacement {
    let placement = clamp_to_screen(placement, pet, screen);

    let left_gap = placement.x - screen.x;
    let right_gap = (screen.x + screen.width) - (placement.x + pet.0);
    let x = if left_gap <= SNAP_THRESHOLD || right_gap <= SNAP_THRESHOLD {
        if left_gap <= right_gap {
            screen.x
        } else {
            screen.x + screen.width - pet.0
        }
    } else {
        placement.x
    };

    let top_gap = placement.y - screen.y;
    let bottom_gap = (screen.y + screen.height) - (placement.y + pet.1);
    let y = if top_gap <= SNAP_THRESHOLD || bottom_gap <= SNAP_THRESHOLD {
        if top_gap <= bottom_gap {
            screen.y
        } else {
            screen.y + screen.height - pet.1
        }
    } else {
        placement.y
    };

    PetPlacement { x, y }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PetMood {
    Focus,
    Break,
    Nagging,
    Sleeping,
}

/// Seconds of inactivity before the pet dozes off, when 睡眠动画 is enabled.
const SLEEP_AFTER_SECS: u32 = 300;

pub fn mood(
    phase: Phase,
    running: bool,
    nagging: bool,
    idle_secs: u32,
    sleep_animation: bool,
) -> PetMood {
    if nagging {
        return PetMood::Nagging;
    }
    if sleep_animation && !running && idle_secs >= SLEEP_AFTER_SECS {
        return PetMood::Sleeping;
    }
    match phase {
        Phase::Focus => PetMood::Focus,
        Phase::ShortBreak | Phase::LongBreak => PetMood::Break,
    }
}
```

In `src-tauri/src/model.rs`, add the field and import:

```rust
use crate::core::desk::PetPlacement;
```

```rust
    #[serde(default)]
    pub pet_placement: Option<PetPlacement>,
```

Bump `SCHEMA_VERSION` in `src-tauri/src/store.rs` to `4`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test core::desk::`
Expected: PASS, 12 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/desk.rs src-tauri/src/core/mod.rs src-tauri/src/model.rs src-tauri/src/store.rs
git commit -m "feat(rust): add pet placement, edge snapping and mood"
```

---

### Task 2: The platform abstraction

**Files:**
- Create: `src-tauri/src/platform/mod.rs`, `src-tauri/src/platform/fallback.rs`, `src-tauri/src/platform/macos.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`

**Interfaces:**
- Produces:
  - `pub trait PlatformWindows: Send + Sync { fn make_desktop_layer(&self, window: &WebviewWindow); fn make_overlay_layer(&self, window: &WebviewWindow); fn set_click_through(&self, window: &WebviewWindow, ignore: bool); fn fullscreen_app_frontmost(&self) -> bool; fn microphone_in_use(&self) -> bool; }`
  - `pub fn platform() -> &'static dyn PlatformWindows`

- [ ] **Step 1: Add the macOS-only dependencies**

In `src-tauri/Cargo.toml`:

```toml
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6"
objc2-app-kit = { version = "0.3", features = ["NSScreen", "NSWindow", "NSApplication"] }
objc2-foundation = "0.3"
coreaudio-sys = "0.2"
```

- [ ] **Step 2: Write the trait**

Create `src-tauri/src/platform/mod.rs`:

```rust
use tauri::WebviewWindow;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod fallback;

/// Window behaviours that only macOS can express properly. Every call site goes
/// through this trait so the Linux and Windows builds stay compilable and usable.
pub trait PlatformWindows: Send + Sync {
    /// Float above normal windows, follow the user across spaces, and never steal focus.
    fn make_desktop_layer(&self, window: &WebviewWindow);
    /// Cover everything, including full-screen apps.
    fn make_overlay_layer(&self, window: &WebviewWindow);
    /// Let clicks pass straight through the window's transparent regions.
    fn set_click_through(&self, window: &WebviewWindow, ignore: bool);
    /// True when a full-screen application currently owns the main screen.
    fn fullscreen_app_frontmost(&self) -> bool;
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
```

- [ ] **Step 3: Write the fallback**

Create `src-tauri/src/platform/fallback.rs`:

```rust
use tauri::WebviewWindow;

use super::PlatformWindows;

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
        let _ = window.set_fullscreen(true);
        let _ = window.set_decorations(false);
    }

    fn set_click_through(&self, window: &WebviewWindow, ignore: bool) {
        let _ = window.set_ignore_cursor_events(ignore);
    }

    fn fullscreen_app_frontmost(&self) -> bool {
        false
    }

    fn microphone_in_use(&self) -> bool {
        false
    }
}
```

- [ ] **Step 4: Write the macOS implementation**

Create `src-tauri/src/platform/macos.rs`:

```rust
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
        let _ = window.set_fullscreen(false);
        // The overlay is sized and placed per monitor by windows.rs, not by
        // set_fullscreen — a native fullscreen window would get its own space.
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
```

Add `pub mod platform;` to `src-tauri/src/lib.rs`.

- [ ] **Step 5: Feed the real meeting signal into the engine**

In `src-tauri/src/model.rs`, change `fire_context` to accept the platform signals rather
than hard-coding them:

```rust
    pub fn fire_context(&self, now: DateTime<Local>, in_meeting: bool) -> FireContext {
        FireContext {
            minute_of_day: (now.hour() * 60 + now.minute()) as u16,
            weekday_index: now.weekday().num_days_from_monday() as usize,
            in_focus: self.timer.running && self.timer.phase == Phase::Focus,
            in_meeting,
            deep_work: self.deep_work,
        }
    }
```

Update both call sites (`state.rs::run_reminders` and `commands.rs::up_next`) to pass
`crate::platform::platform().microphone_in_use()` and `false` respectively — the up-next
list should show what is scheduled regardless of whether a meeting is currently silencing it.

- [ ] **Step 6: Verify on every platform**

Run: `cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clean on macOS.

Run: `cargo check --target x86_64-unknown-linux-gnu` if the target is installed; otherwise
rely on CI. Expected: the fallback compiles with no macOS crates pulled in.

- [ ] **Step 7: Commit**

```bash
git add src-tauri
git commit -m "feat(rust): add the platform window abstraction with a macOS implementation"
```

---

### Task 3: Pet, bubble and overlay window lifecycle

**Files:**
- Modify: `src-tauri/src/windows.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/state.rs`, `src-tauri/tauri.conf.json`
- Create: `bubble.html`, `src/entries/bubble.ts`, `src/routes/bubble/App.svelte` (placeholder)

**Interfaces:**
- Produces:
  - `windows::ensure_pet(app) -> tauri::Result<()>` — creates and places the pet window
  - `windows::sync_pet_visibility(app, hide_when_fullscreen: bool)`
  - `windows::show_bubble(app, payload) -> tauri::Result<()>`
  - `windows::show_overlay(app, payload) -> tauri::Result<()>` — one window per monitor
  - `windows::dismiss_overlays(app)`
  - Commands: `set_pet_placement(x, y)`, `dismiss_overlay(id, acknowledged)`, `pet_poke()`
  - Event `pet:nudge` with the `FirePayload` shape, consumed by the pet window

- [ ] **Step 1: Declare the pet and bubble windows**

In `src-tauri/tauri.conf.json`, add to `app.windows`:

```json
{
  "label": "pet",
  "url": "pet.html",
  "width": 380,
  "height": 200,
  "visible": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": false,
  "shadow": false,
  "focus": false
},
{
  "label": "bubble",
  "url": "bubble.html",
  "width": 360,
  "height": 110,
  "visible": false,
  "decorations": false,
  "transparent": true,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": false,
  "shadow": false,
  "focus": false
}
```

Add `"bubble"` to the `windows` array in `src-tauri/capabilities/default.json`, and add
`bubble` to the Vite input map in `vite.config.ts` alongside a root `bubble.html` and
`src/entries/bubble.ts` following the pattern from plan 01 Task 9.

Overlay windows are created at runtime (one per monitor) rather than declared, so the
`overlay` entry stays out of `tauri.conf.json`; only `overlay.html` needs to exist.

- [ ] **Step 2: Write the window lifecycle**

Append to `src-tauri/src/windows.rs`:

```rust
use serde::Serialize;
use tauri::{Emitter, LogicalPosition, LogicalSize, WebviewUrl, WebviewWindowBuilder};

use crate::core::desk::{PetPlacement, ScreenRect, clamp_to_screen};
use crate::platform::platform;

const PET_SIZE: (f64, f64) = (380.0, 200.0);

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

/// Create the pet window if needed, give it the desktop layer treatment, and put it
/// back where the user left it.
pub fn ensure_pet(app: &AppHandle) -> tauri::Result<()> {
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

/// Hide the pet while a full-screen app owns the screen, when 全屏时隐藏 is on.
pub fn sync_pet_visibility(app: &AppHandle, hide_when_fullscreen: bool) {
    let Some(window) = app.get_webview_window("pet") else {
        return;
    };
    let should_hide = hide_when_fullscreen && platform().fullscreen_app_frontmost();
    let _ = if should_hide {
        window.hide()
    } else {
        window.show()
    };
}

pub fn hide_pet(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("pet") {
        let _ = window.hide();
    }
}

/// 轻量气泡 — slide in at the top-right of the primary screen for six seconds.
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
    let monitors = app.available_monitors()?;
    for (index, monitor) in monitors.iter().enumerate() {
        let label = format!("overlay-{index}");
        let scale = monitor.scale_factor();
        let pos = monitor.position().to_logical::<f64>(scale);
        let size = monitor.size().to_logical::<f64>(scale);

        let window =
            WebviewWindowBuilder::new(app, &label, WebviewUrl::App("overlay.html".into()))
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
```

- [ ] **Step 3: Route firings to the right surface**

In `src-tauri/src/state.rs`, add `use crate::core::reminder::Intensity;` to the imports,
then replace the emit loop at the end of `run_reminders`:

```rust
        for payload in fires {
            // Every window hears the raw event; the surface that renders it depends
            // on the intensity the engine chose for this particular firing.
            let _ = app.emit(events::REMINDER_FIRE, payload.clone());
            match payload.intensity {
                Intensity::Bubble => {
                    let _ = crate::windows::show_bubble(app, payload.clone());
                }
                Intensity::Pet => {
                    let _ = crate::windows::ensure_pet(app);
                    let _ = app.emit_to("pet", "pet:nudge", payload.clone());
                }
                Intensity::Fullscreen => {
                    let _ = crate::windows::show_overlay(app, payload.clone());
                }
            }
        }

        // The pet ducks out of the way of full-screen apps between firings.
        let hide = self.with(|m| m.settings.pet_flags.hide_fullscreen);
        crate::windows::sync_pet_visibility(app, hide);
```

- [ ] **Step 4: Add the pet and overlay commands**

Append to `src-tauri/src/commands.rs`:

```rust
use crate::core::desk::{PetPlacement, snap};

#[tauri::command]
pub fn set_pet_placement(state: State<'_, AppState>, app: AppHandle, x: f64, y: f64) {
    let snap_edges = state.with(|m| m.settings.pet_flags.snap_edges);
    let screen = crate::windows::primary_screen_rect(&app);
    let mut placement = PetPlacement { x, y };
    if snap_edges {
        placement = snap(placement, (380.0, 200.0), screen);
    }
    state.with(|m| m.pet_placement = Some(placement));
    if let Some(window) = tauri::Manager::get_webview_window(&app, "pet") {
        let _ = window.set_position(tauri::LogicalPosition::new(placement.x, placement.y));
    }
    state.flush();
}

#[tauri::command]
pub fn show_pet(app: AppHandle) -> Result<(), String> {
    crate::windows::ensure_pet(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn hide_pet(app: AppHandle) {
    crate::windows::hide_pet(&app);
}

#[tauri::command]
pub fn hide_bubble(app: AppHandle) {
    crate::windows::hide_bubble(&app);
}

/// Close the overlay on every display. `acknowledged` distinguishes 完成 from ⎋.
#[tauri::command]
pub fn dismiss_overlay(
    state: State<'_, AppState>,
    app: AppHandle,
    id: u32,
    acknowledged: bool,
) {
    crate::windows::dismiss_overlays(&app);
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            if acknowledged {
                r.acknowledge();
            } else {
                r.ignore();
            }
        }
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}
```

Register the five new commands in `generate_handler!`.

- [ ] **Step 5: Show the pet at launch**

In `src-tauri/src/lib.rs`'s `setup`, after the tray build, add:

```rust
            let _ = windows::ensure_pet(&app.handle().clone());
```

- [ ] **Step 6: Verify**

Run: `cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clean.

- [ ] **Step 7: Commit**

```bash
git add src-tauri vite.config.ts bubble.html src/entries/bubble.ts src/routes/bubble
git commit -m "feat(rust): add pet, bubble and overlay window lifecycle"
```

---

### Task 4: The desktop pet UI

**Files:**
- Modify: `src/routes/pet/App.svelte`, `src/lib/ipc.ts`

**Interfaces:**
- Consumes: `app`, `PetCanvas`, `SpeechBubble`, `onPetNudge`, `setPetPlacement`, `ackReminder`, `showMain`.
- Produces: the desktop pet from artboard 02 — bob animation, blurred shadow, drag to move,
  speech bubble on nudge.

- [ ] **Step 1: Add the IPC pieces**

Append to `src/lib/ipc.ts`:

```ts
export const setPetPlacement = (x: number, y: number) =>
  invoke<void>("set_pet_placement", { x, y });
export const showPet = () => invoke<void>("show_pet");
export const hidePet = () => invoke<void>("hide_pet");
export const hideBubble = () => invoke<void>("hide_bubble");
export const dismissOverlay = (id: number, acknowledged: boolean) =>
  invoke<void>("dismiss_overlay", { id, acknowledged });

export const onPetNudge = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  listen<FirePayload>("pet:nudge", (e) => cb(e.payload));
export const onBubbleShow = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  listen<FirePayload>("bubble:show", (e) => cb(e.payload));
export const onOverlayShow = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  listen<FirePayload>("overlay:show", (e) => cb(e.payload));
```

- [ ] **Step 2: Write the pet window**

Replace `src/routes/pet/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import { petLine } from "../../lib/copy";
  import { minutesLeft } from "../../lib/format";
  import {
    type FirePayload,
    ackReminder,
    onPetNudge,
    setPetPlacement,
    showMain,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  const NUDGE_MS = 12_000;

  let nudge = $state<FirePayload | null>(null);
  let dragging = $state(false);

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  const bubbleText = $derived(
    nudge?.message ?? petLine(app.tone, minutesLeft(app.timer.remainingSecs)),
  );

  let nudgeTimer: ReturnType<typeof setTimeout> | undefined;

  onMount(() => {
    void app.init();
    const un = onPetNudge((payload) => {
      nudge = payload;
      clearTimeout(nudgeTimer);
      nudgeTimer = setTimeout(() => (nudge = null), NUDGE_MS);
    });
    return () => {
      clearTimeout(nudgeTimer);
      void un.then((f) => f());
      app.dispose();
    };
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });

  /**
   * Tauri's startDragging moves the window natively; when it finishes we read the
   * final position back and hand it to Rust, which applies edge snapping and stores it.
   */
  async function onPointerDown(event: PointerEvent) {
    if (event.button !== 0) return;
    dragging = true;
    const win = getCurrentWindow();
    await win.startDragging();
    const pos = await win.outerPosition();
    const scale = await win.scaleFactor();
    dragging = false;
    await setPetPlacement(pos.x / scale, pos.y / scale);
  }

  function onPoke() {
    if (!app.settings.petFlags.clickInteract) return;
    if (nudge) {
      void ackReminder(nudge.id);
      nudge = null;
      return;
    }
    void showMain();
  }
</script>

<div class="stage">
  <div
    class="pet"
    class:dragging
    role="button"
    tabindex="0"
    aria-label="Pomodo"
    onpointerdown={onPointerDown}
    onclick={onPoke}
    onkeydown={(e) => e.key === "Enter" && onPoke()}
  >
    <div class="sprite" class:hop={!!nudge}>
      <PetCanvas
        map={pet.map}
        body={pet.body}
        scale={8}
        anim={nudge ? "hop" : "bob"}
        alt={pet.name}
      />
    </div>
    <div class="shadow"></div>
  </div>

  <div class="bubble" class:nudging={!!nudge}>{bubbleText}</div>
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent;
    overflow: hidden;
  }
  .stage {
    display: flex;
    align-items: flex-end;
    gap: 12px;
    padding: 8px;
    height: 100vh;
  }
  .pet {
    position: relative;
    width: 128px;
    height: 128px;
    flex: none;
    cursor: grab;
    background: transparent;
    border: none;
    padding: 0;
  }
  .pet.dragging {
    cursor: grabbing;
  }
  .shadow {
    position: absolute;
    bottom: -6px;
    left: 8px;
    width: 112px;
    height: 10px;
    border-radius: 50%;
    background: oklch(0.2 0.02 260 / 0.4);
    filter: blur(4px);
  }
  .bubble {
    margin-bottom: 4px;
    padding: 10px 14px;
    border-radius: 13px 13px 13px 4px;
    background: oklch(0.985 0.004 80 / 0.95);
    box-shadow: 0 12px 28px -12px oklch(0.2 0.02 260 / 0.6);
    font-size: 13.5px;
    line-height: 1.45;
    max-width: 220px;
    color: var(--ink);
  }
  .bubble.nudging {
    border: 1.5px solid var(--accent);
  }
</style>
```

- [ ] **Step 3: Commit**

```bash
git add src/routes/pet/App.svelte src/lib/ipc.ts
git commit -m "feat: build the desktop pet window"
```

---

### Task 5: The bubble and overlay UIs

**Files:**
- Modify: `src/routes/bubble/App.svelte`, `src/routes/overlay/App.svelte`

- [ ] **Step 1: Write the bubble**

Replace `src/routes/bubble/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import { type FirePayload, ackReminder, hideBubble, onBubbleShow } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  /** 右上角滑入，6 秒自动收起。 */
  const AUTO_DISMISS_MS = 6000;

  let fire = $state<FirePayload | null>(null);
  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  let timer: ReturnType<typeof setTimeout> | undefined;

  onMount(() => {
    void app.init();
    const un = onBubbleShow((payload) => {
      fire = payload;
      clearTimeout(timer);
      timer = setTimeout(() => void hideBubble(), AUTO_DISMISS_MS);
    });
    return () => {
      clearTimeout(timer);
      void un.then((f) => f());
      app.dispose();
    };
  });
</script>

{#if fire}
  <div class="toast">
    <PetCanvas map={pet.map} body={pet.body} scale={3} alt={pet.name} />
    <div class="text">
      <span class="title">{fire.name}</span>
      <span class="body">{fire.message}</span>
    </div>
    <button
      class="ack"
      type="button"
      onclick={() => {
        if (fire) void ackReminder(fire.id);
        void hideBubble();
      }}
    >
      好
    </button>
  </div>
{/if}

<style>
  :global(html),
  :global(body) {
    background: transparent;
    overflow: hidden;
  }
  .toast {
    margin: 8px;
    padding: 15px;
    border-radius: 13px;
    background: oklch(0.31 0.025 258);
    color: oklch(0.97 0.004 80);
    display: flex;
    gap: 12px;
    align-items: center;
    box-shadow: 0 18px 40px -16px oklch(0.2 0.02 260 / 0.7);
    animation: momo-rise 0.35s ease both;
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 13.5px;
    font-weight: 600;
  }
  .body {
    font-size: 12.5px;
    opacity: 0.8;
    line-height: 1.4;
  }
  .ack {
    border: 1px solid oklch(0.97 0.004 80 / 0.3);
    border-radius: 8px;
    background: transparent;
    color: inherit;
    font-family: inherit;
    font-size: 12.5px;
    padding: 6px 12px;
    cursor: pointer;
    flex: none;
  }
  .ack:hover {
    background: oklch(0.97 0.004 80 / 0.14);
  }
</style>
```

- [ ] **Step 2: Write the overlay**

Replace `src/routes/overlay/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import { mmss } from "../../lib/format";
  import { type FirePayload, dismissOverlay, onOverlayShow } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  /** The design's overlay shows 02:41 — a short forced break. */
  const BREAK_SECS = 161;

  let fire = $state<FirePayload | null>(null);
  let left = $state(BREAK_SECS);

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  onMount(() => {
    void app.init();

    const un = onOverlayShow((payload) => {
      fire = payload;
      left = BREAK_SECS;
    });

    const ticker = setInterval(() => {
      if (left > 0) {
        left -= 1;
      } else if (fire) {
        // Sitting through the whole countdown counts as doing the thing.
        void dismissOverlay(fire.id, true);
      }
    }, 1000);

    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && fire) {
        void dismissOverlay(fire.id, false);
      }
    };
    window.addEventListener("keydown", onKey);

    return () => {
      clearInterval(ticker);
      window.removeEventListener("keydown", onKey);
      void un.then((f) => f());
      app.dispose();
    };
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });
</script>

<div class="mask">
  <PetCanvas map={pet.map} body={pet.body} scale={3} anim="sway" alt={pet.name} />
  <span class="count">{mmss(left)}</span>
  <span class="line">{fire?.message ?? "站起来走走，看点远的东西"}</span>
  <button
    class="done"
    type="button"
    onclick={() => fire && void dismissOverlay(fire.id, true)}
  >
    做完了
  </button>
  <span class="escape">按 ⎋ 逃跑（它会记着）</span>
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    overflow: hidden;
  }
  .mask {
    position: relative;
    width: 100vw;
    height: 100vh;
    background: oklch(0.29 0.025 258);
    color: oklch(0.97 0.004 80);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
  }
  .count {
    font-family: var(--font-mono);
    font-size: 64px;
    font-weight: 500;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
  }
  .line {
    font-size: 15px;
    opacity: 0.85;
  }
  .done {
    margin-top: 14px;
    padding: 12px 28px;
    border: none;
    border-radius: 12px;
    background: var(--accent);
    color: oklch(0.99 0.004 80);
    font-family: inherit;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    box-shadow: var(--inset-press);
  }
  .escape {
    position: absolute;
    bottom: 18px;
    right: 20px;
    font-size: 11px;
    opacity: 0.5;
  }
</style>
```

The design's overlay renders the countdown at 30px because the artboard shows a 400px
card; a real full-screen mask uses 64px so it reads from across the room. Everything else —
colour, pet scale, sway, copy, corner note — matches artboard 04 exactly.

- [ ] **Step 3: Verify**

Run: `npm run check && npm run build`
Expected: both pass, with six HTML entries plus `bubble.html` in `dist/`.

- [ ] **Step 4: Commit**

```bash
git add src/routes/bubble src/routes/overlay
git commit -m "feat: build the bubble and fullscreen overlay UIs"
```

---

### Task 6: End-to-end verification

**Files:** none.

- [ ] **Step 1: Launch**

Run: `npm run tauri dev`
Expected: the pet appears near the bottom-left of the screen, bobbing, with a speech bubble
showing the remaining minutes. It has no window frame or shadow.

- [ ] **Step 2: Check dragging and snapping**

Drag the pet to the middle of the screen and release. Expected: it stays where you dropped
it. Drag it to within ~40px of the right edge. Expected: it snaps flush. Turn off 贴边吸附
in the 宠物 tab and repeat. Expected: no snapping. Quit and relaunch. Expected: the pet
returns to its last position.

- [ ] **Step 3: Check the three intensities**

In the main window's devtools console:

```js
const { invoke } = await import("@tauri-apps/api/core");
const m = await invoke("list_model");
const water = m.reminders.find((r) => r.name === "喝水");
const stand = m.reminders.find((r) => r.name === "站起来动一动");
const review = m.reminders.find((r) => r.name === "收工前复盘");

await invoke("update_reminder", { id: water.id, patch: { intervalMinutes: 1, intensity: "bubble" } });
```

Expected within a minute: a dark toast slides in at the top right with the pet at scale 3
and the water message, dismissing itself after six seconds.

Repeat with `intensity: "pet"` on `stand`. Expected: the desktop pet hops and its bubble
switches to the stand-up message for twelve seconds.

Repeat with `intensity: "fullscreen"` on `review` (set `intervalMinutes` and change its
schedule with the same patch). Expected: every connected display is covered by a dark mask
with a swaying pet and a counting-down clock.

- [ ] **Step 4: Check overlay dismissal semantics**

Press ⎋ on the overlay. Expected: every overlay window closes, and `list_model` shows the
reminder's `consecutiveIgnores` incremented. Trigger it again and click 做完了. Expected:
`consecutiveIgnores` is back to 0.

- [ ] **Step 5: Check the escalation path reaches the screen**

Set a reminder to a one-minute bubble, ignore three firings by letting them auto-dismiss
without clicking 好, then wait for the fourth. Expected: the fourth firing arrives as a
fullscreen overlay rather than a bubble, and `consecutiveIgnores` resets afterwards.

- [ ] **Step 6: Check fullscreen hiding and meeting silence**

Put any app into macOS fullscreen with 全屏时隐藏 on. Expected: the pet disappears within a
tick and returns when you leave fullscreen.

Start a recording in QuickTime (or join a call) with a reminder due. Expected: it does not
fire; `microphone_in_use` is reporting true. Stop recording and it fires normally.

- [ ] **Step 7: Check deep work demotion end to end**

Turn on 深度工作 in 设置 · 通用, then trigger the fullscreen reminder. Expected: it arrives
as a corner bubble instead of a mask.

- [ ] **Step 8: Check click interaction**

With 点击互动 on, click the pet with no nudge pending. Expected: the main window comes
forward. Click it while a nudge is showing. Expected: the reminder is acknowledged, the
bubble reverts, and the body counter moves. Turn 点击互动 off and confirm clicks do nothing.

- [ ] **Step 9: Run the full gate**

```bash
npm test && npm run check && npm run build && (cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test)
```
Expected: everything passes.

- [ ] **Step 10: Verify the non-macOS build**

Push the branch and confirm the Linux and Windows CI jobs go green. Expected: the fallback
platform compiles with no macOS crates and the app builds on all three targets.

- [ ] **Step 11: Commit**

```bash
git commit --allow-empty -m "test: verify the desktop pet and reminder overlays end to end"
```

---

## Definition of Done

- The pet floats on the desktop, bobs, drags, snaps to edges when 贴边吸附 is on, and
  restores its position across restarts.
- 全屏时隐藏 hides it behind fullscreen apps; 点击互动 gates click responses.
- All three reminder intensities render as real windows matching artboard 04.
- The fullscreen overlay covers every connected display and tears down cleanly.
- ⎋ records an ignore; 做完了 and sitting through the countdown record an acknowledgement.
- Three ignores escalate the next firing to fullscreen, visibly.
- A microphone in use silences reminders; deep work demotes them all to bubbles.
- Linux and Windows still build and run with the fallback platform.
- The full test gate passes on macOS and CI is green on all three platforms.
