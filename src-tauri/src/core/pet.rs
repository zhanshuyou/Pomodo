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

/// Life stage by level. The artboard's Lv.7 is 好奇期; the rest fan out
/// around it so the label changes every few levels rather than never.
pub fn stage_name(level: u8) -> &'static str {
    match level {
        0..=3 => "幼崽期",
        4..=8 => "好奇期",
        9..=14 => "顽皮期",
        15..=24 => "稳重期",
        _ => "老江湖",
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetState {
    pub selected: u8,
    pub lifetime_pomodoros: u32,
    pub custom: CustomPet,
    pub use_custom: bool,
    /// Everything below is derived from `lifetime_pomodoros` by `refresh` and
    /// only travels outward, so the webviews do not each keep their own copy
    /// of the level maths and the unlock table.
    #[serde(skip_deserializing)]
    pub level: u8,
    #[serde(skip_deserializing)]
    pub stage: &'static str,
    #[serde(skip_deserializing)]
    pub to_next_level: u32,
    /// 0–100, rounded.
    #[serde(skip_deserializing)]
    pub progress_pct: u32,
    /// Lifetime pomodoros needed for each of the six pets.
    #[serde(skip_deserializing)]
    pub unlock_at: [u32; 6],
}

impl Default for PetState {
    fn default() -> Self {
        let mut p = Self {
            selected: 0,
            lifetime_pomodoros: 0,
            custom: CustomPet::default(),
            use_custom: false,
            level: 0,
            stage: "",
            to_next_level: 0,
            progress_pct: 0,
            unlock_at: [0; 6],
        };
        p.refresh();
        p
    }
}

impl PetState {
    pub fn credit(&mut self) {
        self.lifetime_pomodoros = self.lifetime_pomodoros.saturating_add(1);
        self.refresh();
    }

    /// Recompute the derived fields. Called after every change to
    /// `lifetime_pomodoros` and once after loading from disk.
    pub fn refresh(&mut self) {
        self.level = self.level();
        self.stage = stage_name(self.level);
        self.to_next_level = self.to_next_level();
        self.progress_pct = (self.level_progress() * 100.0).round() as u32;
        self.unlock_at = std::array::from_fn(|i| unlock_threshold(i as u8));
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
        // Artboard figure: 6 full levels plus 8 into the seventh.
        let p = PetState {
            lifetime_pomodoros: 6 * POMODOROS_PER_LEVEL + 8, // 86
            ..PetState::default()
        };
        assert_eq!(p.level(), 7);
        assert_eq!(p.to_next_level(), 5);
        assert!((p.level_progress() - 8.0 / 13.0).abs() < 1e-6);
        assert_eq!((p.level_progress() * 100.0).round() as u32, 62);
    }

    #[test]
    fn derived_fields_follow_lifetime_pomodoros_and_are_never_read_back() {
        let mut p = PetState {
            lifetime_pomodoros: 86,
            ..PetState::default()
        };
        p.refresh();
        assert_eq!(p.level, 7);
        assert_eq!(p.stage, "好奇期");
        assert_eq!(p.to_next_level, 5);
        assert_eq!(p.progress_pct, 62);
        assert_eq!(p.unlock_at, [0, 0, 0, 0, 150, 300]);

        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"level\":7"));
        // A file that claims a different level is ignored; the count is the truth.
        let forged = json.replace("\"level\":7", "\"level\":99");
        let mut back: PetState = serde_json::from_str(&forged).unwrap();
        back.refresh();
        assert_eq!(back.level, 7);
    }

    #[test]
    fn stages_change_every_few_levels_and_lv7_is_curious() {
        assert_eq!(stage_name(1), "幼崽期");
        assert_eq!(stage_name(7), "好奇期");
        assert_eq!(stage_name(10), "顽皮期");
        assert_eq!(stage_name(20), "稳重期");
        assert_eq!(stage_name(40), "老江湖");
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
        let p = PetState {
            lifetime_pomodoros: 86,
            ..PetState::default()
        };
        assert!(!p.is_unlocked(4));
        assert!(!p.is_unlocked(5));
    }

    #[test]
    fn peep_unlocks_at_one_hundred_and_fifty_and_boo_at_three_hundred() {
        let mut p = PetState {
            lifetime_pomodoros: 150,
            ..PetState::default()
        };
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
        let p = PetState {
            lifetime_pomodoros: POMODOROS_PER_LEVEL,
            ..PetState::default()
        };
        assert_eq!(p.level(), 2);
        assert_eq!(p.level_progress(), 0.0);
        assert_eq!(p.to_next_level(), POMODOROS_PER_LEVEL);
    }
}
