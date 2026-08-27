use serde::{Deserialize, Serialize};

use crate::model::Phase;

/// Top-left corner of a desktop-layer window (the pet, the mini bar), in
/// logical screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
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
pub fn clamp_to_screen(placement: Placement, pet: (f64, f64), screen: ScreenRect) -> Placement {
    let max_x = screen.x + (screen.width - pet.0).max(0.0);
    let max_y = screen.y + (screen.height - pet.1).max(0.0);
    Placement {
        x: placement.x.clamp(screen.x, max_x),
        y: placement.y.clamp(screen.y, max_y),
    }
}

/// 贴边吸附 — pull the pet flush against whichever edge it was dropped near.
/// Horizontal and vertical are decided independently, and when both edges on one
/// axis are within reach the nearer one wins.
pub fn snap(placement: Placement, pet: (f64, f64), screen: ScreenRect) -> Placement {
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

    Placement { x, y }
}

/// 条形 — the mini bar's resting size, straight off the artboard.
pub const MINI_SIZE: (f64, f64) = (260.0, 52.0);
/// However long a reminder is, the bar stops being a bar past this.
pub const MINI_MAX_HEIGHT: f64 = 260.0;

/// The bar measures itself and asks for a height, because how tall a reminder
/// renders depends on the message, the font and the user's text size — none of
/// which Rust can see. This only keeps that request sane.
pub fn clamp_mini_height(height: f64) -> f64 {
    height.clamp(MINI_SIZE.1, MINI_MAX_HEIGHT)
}

/// Fixed width and the sane height range for each window that measures its
/// own content and asks to be resized. `None` for a window that must not.
pub fn window_height_bounds(label: &str) -> Option<(f64, f64, f64)> {
    match label {
        "mini" => Some((MINI_SIZE.0, MINI_SIZE.1, MINI_MAX_HEIGHT)),
        // The popover: ring row + up to three up-next rows + footer.
        "tray" => Some((330.0, 200.0, 600.0)),
        // A toast; a long message wraps rather than clips.
        "bubble" => Some((360.0, 60.0, 320.0)),
        _ => None,
    }
}

/// A rectangle inside the pet window that should receive clicks, in logical
/// pixels relative to the window's top-left. The webview measures these (the
/// sprite, the speech bubble); Rust only needs to know where they are.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HitRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Is the point (window-local, logical) over anything clickable?
pub fn hits(rects: &[HitRect], x: f64, y: f64) -> bool {
    rects
        .iter()
        .any(|r| x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height)
}

/// Whether the desktop pet belongs on screen at all. Mini mode carries its own
/// 35px cat in the bar, so a second one on the desktop would just be a duplicate
/// — but a pet the user dismissed stays dismissed once mini mode ends.
pub fn pet_should_show(pet_visible: bool, mini_enabled: bool) -> bool {
    pet_visible && !mini_enabled
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum PetMood {
    #[default]
    Focus,
    Break,
    Nagging,
    Sleeping,
}

/// Seconds the timer has to sit stopped before the pet dozes off, when 睡眠动画
/// is enabled. "Idle" is the timer, not the person — there is no input hook.
pub const SLEEP_AFTER_SECS: u32 = 300;

/// How long a 宠物 nudge counts as "nagging" if nobody answers it. Matches the
/// pet window's own bubble timeout so the hop stops when the bubble does.
pub const NAG_SECS: u32 = 12;

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
        let out = snap(Placement { x: 20.0, y: 400.0 }, PET, screen());
        assert_eq!(out.x, 0.0);
        assert_eq!(out.y, 400.0);
    }

    #[test]
    fn a_pet_near_the_right_edge_snaps_flush_to_it() {
        let out = snap(
            Placement {
                x: 1300.0,
                y: 400.0,
            },
            PET,
            screen(),
        );
        assert_eq!(out.x, 1440.0 - 128.0);
    }

    #[test]
    fn a_pet_near_the_bottom_edge_snaps_flush_to_it() {
        let out = snap(Placement { x: 600.0, y: 800.0 }, PET, screen());
        assert_eq!(out.y, 900.0 - 128.0);
    }

    #[test]
    fn a_pet_in_open_space_does_not_move() {
        let placement = Placement { x: 600.0, y: 400.0 };
        let out = snap(placement, PET, screen());
        assert_eq!(out.x, 600.0);
        assert_eq!(out.y, 400.0);
    }

    #[test]
    fn snapping_prefers_the_nearer_of_two_close_edges() {
        // A 200-wide screen where both edges are within the threshold.
        let narrow = ScreenRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 900.0,
        };
        let out = snap(Placement { x: 60.0, y: 400.0 }, PET, narrow);
        // Left gap 60, right gap 200 - 128 - 60 = 12 -> snap right.
        assert_eq!(out.x, 72.0);
    }

    #[test]
    fn clamp_pulls_an_off_screen_pet_back_into_view() {
        let out = clamp_to_screen(
            Placement {
                x: -400.0,
                y: 5000.0,
            },
            PET,
            screen(),
        );
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
        let out = clamp_to_screen(Placement { x: 1000.0, y: 0.0 }, PET, secondary);
        assert_eq!(out.x, 1440.0);
    }

    const BAR: (f64, f64) = (260.0, 52.0);

    #[test]
    fn the_mini_bar_snaps_flush_to_the_top_right_corner() {
        // Dropped near the top-right, the way the design parks it.
        let out = snap(
            Placement {
                x: 1440.0 - 260.0 - 30.0,
                y: 22.0,
            },
            BAR,
            screen(),
        );
        assert_eq!(out.x, 1440.0 - 260.0);
        assert_eq!(out.y, 0.0);
    }

    #[test]
    fn a_resting_bar_keeps_the_artboards_height() {
        assert_eq!(clamp_mini_height(52.0), 52.0);
    }

    #[test]
    fn a_bar_carrying_a_two_line_reminder_grows_to_fit_it() {
        assert_eq!(clamp_mini_height(127.0), 127.0);
    }

    #[test]
    fn only_the_self_measuring_windows_may_ask_for_a_height() {
        assert!(window_height_bounds("tray").is_some());
        assert!(window_height_bounds("bubble").is_some());
        assert_eq!(window_height_bounds("mini"), Some((260.0, 52.0, 260.0)));
        assert_eq!(window_height_bounds("main"), None);
        assert_eq!(window_height_bounds("overlay-3"), None);
    }

    #[test]
    fn a_bar_can_never_shrink_below_its_own_row() {
        // A measurement taken before layout settles reports 0.
        assert_eq!(clamp_mini_height(0.0), 52.0);
    }

    #[test]
    fn a_runaway_measurement_cannot_turn_the_bar_into_a_curtain() {
        assert_eq!(clamp_mini_height(9000.0), MINI_MAX_HEIGHT);
    }

    #[test]
    fn the_pet_shows_when_it_is_wanted_and_mini_mode_is_off() {
        assert!(pet_should_show(true, false));
    }

    #[test]
    fn mini_mode_takes_the_pet_off_screen_even_when_it_is_wanted() {
        // The mini bar carries its own 35px cat; a second one would be a duplicate.
        assert!(!pet_should_show(true, true));
    }

    #[test]
    fn a_dismissed_pet_stays_dismissed_when_mini_mode_ends() {
        assert!(!pet_should_show(false, false));
    }

    #[test]
    fn hits_is_inclusive_at_the_origin_and_exclusive_at_the_far_edge() {
        let r = [HitRect {
            x: 8.0,
            y: 64.0,
            width: 128.0,
            height: 128.0,
        }];
        assert!(hits(&r, 8.0, 64.0));
        assert!(hits(&r, 135.9, 191.9));
        assert!(!hits(&r, 136.0, 100.0));
        assert!(!hits(&r, 100.0, 192.0));
        assert!(!hits(&r, 7.9, 100.0));
        assert!(!hits(&[], 50.0, 100.0));
    }

    #[test]
    fn mood_is_focus_while_a_focus_phase_runs() {
        assert_eq!(mood(Phase::Focus, true, false, 0, true), PetMood::Focus);
    }

    #[test]
    fn mood_is_break_during_either_break() {
        assert_eq!(
            mood(Phase::ShortBreak, true, false, 0, true),
            PetMood::Break
        );
        assert_eq!(mood(Phase::LongBreak, true, false, 0, true), PetMood::Break);
    }

    #[test]
    fn nagging_beats_every_other_mood() {
        assert_eq!(mood(Phase::Focus, true, true, 0, true), PetMood::Nagging);
    }

    #[test]
    fn a_long_idle_sleeps_when_the_sleep_animation_flag_is_on() {
        assert_eq!(
            mood(Phase::Focus, false, false, 600, true),
            PetMood::Sleeping
        );
    }

    #[test]
    fn a_long_idle_does_not_sleep_when_the_flag_is_off() {
        assert_eq!(mood(Phase::Focus, false, false, 600, false), PetMood::Focus);
    }
}
