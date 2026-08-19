use serde::{Deserialize, Serialize};

use crate::core::timer::Timer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Phase {
    Focus,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Accent {
    Terracotta,
    Blue,
    Green,
    Magenta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Tone {
    Professional,
    Gentle,
    Playful,
}

/// 贴边吸附 / 点击互动 / 全屏时隐藏 / 睡眠动画
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetFlags {
    pub snap_edges: bool,
    pub click_interact: bool,
    pub hide_fullscreen: bool,
    pub sleep_animation: bool,
}

impl Default for PetFlags {
    fn default() -> Self {
        // The design shows the first three chips active and the fourth inactive.
        Self {
            snap_edges: true,
            click_interact: true,
            hide_fullscreen: true,
            sleep_animation: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub accent: Accent,
    pub tone: Tone,
    pub focus_secs: u32,
    pub short_break_secs: u32,
    pub long_break_secs: u32,
    pub rounds_per_cycle: u8,
    pub pet_flags: PetFlags,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            accent: Accent::Terracotta,
            tone: Tone::Playful,
            focus_secs: 1500,
            short_break_secs: 300,
            long_break_secs: 900,
            rounds_per_cycle: 4,
            pet_flags: PetFlags::default(),
        }
    }
}

impl Settings {
    pub fn duration_for(&self, phase: Phase) -> u32 {
        match phase {
            Phase::Focus => self.focus_secs,
            Phase::ShortBreak => self.short_break_secs,
            Phase::LongBreak => self.long_break_secs,
        }
    }
}

pub type TaskId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    /// Estimated pomodoros; the design renders at most three pips.
    pub estimate: u8,
    /// Pomodoros already completed against this task.
    pub spent: u8,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub timer: Timer,
    pub tasks: Vec<Task>,
    pub settings: Settings,
    pub next_task_id: TaskId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_match_the_spec() {
        let s = Settings::default();
        assert_eq!(s.focus_secs, 1500);
        assert_eq!(s.short_break_secs, 300);
        assert_eq!(s.long_break_secs, 900);
        assert_eq!(s.rounds_per_cycle, 4);
        assert_eq!(s.accent, Accent::Terracotta);
        assert_eq!(s.tone, Tone::Playful);
    }

    #[test]
    fn default_pet_flags_leave_sleep_animation_off() {
        let f = PetFlags::default();
        assert!(f.snap_edges);
        assert!(f.click_interact);
        assert!(f.hide_fullscreen);
        assert!(!f.sleep_animation);
    }

    #[test]
    fn duration_for_maps_each_phase() {
        let s = Settings::default();
        assert_eq!(s.duration_for(Phase::Focus), 1500);
        assert_eq!(s.duration_for(Phase::ShortBreak), 300);
        assert_eq!(s.duration_for(Phase::LongBreak), 900);
    }

    #[test]
    fn model_round_trips_through_json() {
        let model = Model::default();
        let json = serde_json::to_string(&model).expect("serialize");
        let back: Model = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.settings.focus_secs, model.settings.focus_secs);
        assert_eq!(back.timer.phase, model.timer.phase);
    }

    #[test]
    fn enums_serialize_as_lower_camel_strings() {
        assert_eq!(
            serde_json::to_string(&Phase::ShortBreak).unwrap(),
            "\"shortBreak\""
        );
        assert_eq!(
            serde_json::to_string(&Accent::Terracotta).unwrap(),
            "\"terracotta\""
        );
        assert_eq!(
            serde_json::to_string(&Tone::Playful).unwrap(),
            "\"playful\""
        );
    }
}
