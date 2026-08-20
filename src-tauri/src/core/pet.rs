use serde::{Deserialize, Serialize};

/// Completed pomodoros per level. Fixed by the artboard: Lv.7 shows 62% progress
/// with `再专注 5 个番茄升到 Lv.8`, which is 8/13 and 13 - 8.
pub const POMODOROS_PER_LEVEL: u32 = 13;

pub const PET_COUNT: u8 = 6;

/// Lifetime pomodoros required before a pet becomes selectable.
/// The first four ship unlocked; PEEP and BOO are the design's greyed-out cards.
pub fn unlock_threshold(id: u8) -> u32 {
    match id {
        0..=3 => 0,
        4 => 150,
        5 => 300,
        _ => u32::MAX,
    }
}

/// Paths to user-supplied sprites, one per pet state.
/// Empty slots fall back to the built-in pet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CustomPet {
    pub focus: Option<String>,
    pub rest: Option<String>,
    pub nag: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PetState {
    pub selected: u8,
    pub lifetime_pomodoros: u32,
    pub custom: CustomPet,
    pub use_custom: bool,
}

impl PetState {
    pub fn credit(&mut self) {
        self.lifetime_pomodoros = self.lifetime_pomodoros.saturating_add(1);
    }

    pub fn level(&self) -> u8 {
        ((self.lifetime_pomodoros / POMODOROS_PER_LEVEL) + 1).min(u8::MAX as u32) as u8
    }

    /// Fraction of the way to the next level, 0.0..1.0.
    pub fn level_progress(&self) -> f32 {
        (self.lifetime_pomodoros % POMODOROS_PER_LEVEL) as f32 / POMODOROS_PER_LEVEL as f32
    }

    pub fn to_next_level(&self) -> u32 {
        POMODOROS_PER_LEVEL - (self.lifetime_pomodoros % POMODOROS_PER_LEVEL)
    }

    pub fn is_unlocked(&self, id: u8) -> bool {
        id < PET_COUNT && self.lifetime_pomodoros >= unlock_threshold(id)
    }

    /// Select a pet. Returns false — and changes nothing — if it is locked or unknown.
    pub fn select(&mut self, id: u8) -> bool {
        if !self.is_unlocked(id) {
            return false;
        }
        self.selected = id;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_pet_state_is_level_one_with_mochi_selected() {
        let p = PetState::default();
        assert_eq!(p.selected, 0);
        assert_eq!(p.level(), 1);
        assert_eq!(p.lifetime_pomodoros, 0);
        assert!(!p.use_custom);
    }

    #[test]
    fn thirteen_pomodoros_per_level() {
        let mut p = PetState::default();
        for _ in 0..13 {
            p.credit();
        }
        assert_eq!(p.level(), 2);
        for _ in 0..13 {
            p.credit();
        }
        assert_eq!(p.level(), 3);
    }

    #[test]
    fn the_designs_level_seven_figures_reproduce_exactly() {
        // Artboard 01 shows Lv.7 at 62% with 再专注 5 个番茄升到 Lv.8.
        let mut p = PetState::default();
        p.lifetime_pomodoros = 6 * POMODOROS_PER_LEVEL + 8; // 86
        assert_eq!(p.level(), 7);
        assert_eq!(p.to_next_level(), 5);
        assert!((p.level_progress() - 8.0 / 13.0).abs() < 1e-6);
        assert_eq!((p.level_progress() * 100.0).round() as u32, 62);
    }

    #[test]
    fn the_first_four_pets_are_unlocked_from_the_start() {
        let p = PetState::default();
        for id in 0..4 {
            assert!(p.is_unlocked(id), "pet {id} should be unlocked");
        }
    }

    #[test]
    fn peep_and_boo_stay_locked_at_level_seven() {
        let mut p = PetState::default();
        p.lifetime_pomodoros = 86;
        assert!(!p.is_unlocked(4));
        assert!(!p.is_unlocked(5));
    }

    #[test]
    fn peep_unlocks_at_one_hundred_and_fifty_and_boo_at_three_hundred() {
        let mut p = PetState::default();
        p.lifetime_pomodoros = 150;
        assert!(p.is_unlocked(4));
        assert!(!p.is_unlocked(5));

        p.lifetime_pomodoros = 300;
        assert!(p.is_unlocked(5));
    }

    #[test]
    fn selecting_a_locked_pet_is_refused() {
        let mut p = PetState::default();
        assert!(!p.select(4));
        assert_eq!(p.selected, 0);
    }

    #[test]
    fn selecting_an_unlocked_pet_succeeds() {
        let mut p = PetState::default();
        assert!(p.select(2));
        assert_eq!(p.selected, 2);
    }

    #[test]
    fn selecting_an_out_of_range_pet_is_refused() {
        let mut p = PetState::default();
        assert!(!p.select(99));
        assert_eq!(p.selected, 0);
    }

    #[test]
    fn level_progress_is_zero_immediately_after_levelling() {
        let mut p = PetState::default();
        p.lifetime_pomodoros = POMODOROS_PER_LEVEL;
        assert_eq!(p.level(), 2);
        assert_eq!(p.level_progress(), 0.0);
        assert_eq!(p.to_next_level(), POMODOROS_PER_LEVEL);
    }
}
