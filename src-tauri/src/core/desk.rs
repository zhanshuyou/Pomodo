use serde::{Deserialize, Serialize};

use crate::model::Phase;

/// Top-left corner of the pet window, in logical screen coordinates.
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
        let out = snap(
            PetPlacement {
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
        let narrow = ScreenRect {
            x: 0.0,
            y: 0.0,
            width: 200.0,
            height: 900.0,
        };
        let out = snap(PetPlacement { x: 60.0, y: 400.0 }, PET, narrow);
        // Left gap 60, right gap 200 - 128 - 60 = 12 -> snap right.
        assert_eq!(out.x, 72.0);
    }

    #[test]
    fn clamp_pulls_an_off_screen_pet_back_into_view() {
        let out = clamp_to_screen(
            PetPlacement {
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
        let out = clamp_to_screen(PetPlacement { x: 1000.0, y: 0.0 }, PET, secondary);
        assert_eq!(out.x, 1440.0);
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
