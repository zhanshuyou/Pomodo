use serde::{Deserialize, Serialize};

use chrono::{DateTime, Datelike, Local, Timelike};

use crate::core::desk::Placement;
use crate::core::pet::PetState;
use crate::core::reminder::{FireContext, Reminder};
use crate::core::reminder_copy;
use crate::core::stats::Stats;
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
    /// The user can dismiss the desktop pet; it must stay gone across ticks
    /// and restarts until they ask for it back.
    #[serde(default = "yes")]
    pub pet_visible: bool,
}

fn yes() -> bool {
    true
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
            pet_visible: true,
        }
    }
}

/// 1 minute to 4 hours — generous enough for any real focus/break length, tight
/// enough to keep a mistyped value from wedging the timer.
const MIN_PHASE_SECS: u32 = 60;
const MAX_PHASE_SECS: u32 = 4 * 3600;
const MAX_ROUNDS_PER_CYCLE: u8 = 12;

impl Settings {
    pub fn duration_for(&self, phase: Phase) -> u32 {
        match phase {
            Phase::Focus => self.focus_secs,
            Phase::ShortBreak => self.short_break_secs,
            Phase::LongBreak => self.long_break_secs,
        }
    }

    pub fn set_timer_durations(
        &mut self,
        focus_secs: u32,
        short_break_secs: u32,
        long_break_secs: u32,
        rounds_per_cycle: u8,
    ) {
        self.focus_secs = focus_secs.clamp(MIN_PHASE_SECS, MAX_PHASE_SECS);
        self.short_break_secs = short_break_secs.clamp(MIN_PHASE_SECS, MAX_PHASE_SECS);
        self.long_break_secs = long_break_secs.clamp(MIN_PHASE_SECS, MAX_PHASE_SECS);
        self.rounds_per_cycle = rounds_per_cycle.clamp(1, MAX_ROUNDS_PER_CYCLE);
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

/// 身体这边的账 — the three bars in the 专注 tab sidebar. Reset each calendar day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyCounters {
    pub water_cups: u32,
    pub water_goal: u32,
    pub stands: u32,
    pub stand_goal: u32,
    pub longest_sit_mins: u32,
    /// ISO date the counters belong to.
    pub day: String,
}

impl Default for BodyCounters {
    fn default() -> Self {
        Self {
            water_cups: 0,
            water_goal: 8,
            stands: 0,
            stand_goal: 6,
            longest_sit_mins: 0,
            day: String::new(),
        }
    }
}

// No `Eq`: pet placement carries f64 screen coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub timer: Timer,
    pub tasks: Vec<Task>,
    pub settings: Settings,
    pub next_task_id: TaskId,
    #[serde(default)]
    pub stats: Stats,
    #[serde(default)]
    pub pet: PetState,
    #[serde(default)]
    pub reminders: Vec<Reminder>,
    #[serde(default)]
    pub body: BodyCounters,
    #[serde(default)]
    pub deep_work: bool,
    #[serde(default)]
    pub next_reminder_id: u32,
    #[serde(default)]
    pub pet_placement: Option<Placement>,
    /// 迷你模式 — the main window is put away and only the bar is left.
    #[serde(default)]
    pub mini_enabled: bool,
    #[serde(default)]
    pub mini_placement: Option<Placement>,
}

impl Model {
    pub fn seed_reminders(&mut self) {
        if !self.reminders.is_empty() {
            return;
        }
        for builtin in reminder_copy::ALL {
            let id = self.next_reminder_id;
            self.next_reminder_id += 1;
            self.reminders
                .push(Reminder::seed(builtin, id, self.settings.tone));
        }
    }

    pub fn retone_reminders(&mut self) {
        let tone = self.settings.tone;
        for reminder in &mut self.reminders {
            reminder.retone(tone);
        }
    }

    /// Zero the body counters when the calendar day turns over.
    pub fn roll_body_day(&mut self, today: &str) {
        if self.body.day == today {
            return;
        }
        let goals = (self.body.water_goal, self.body.stand_goal);
        self.body = BodyCounters {
            water_goal: goals.0,
            stand_goal: goals.1,
            day: today.to_string(),
            ..BodyCounters::default()
        };
    }

    pub fn fire_context(&self, now: DateTime<Local>, in_meeting: bool) -> FireContext {
        FireContext {
            minute_of_day: (now.hour() * 60 + now.minute()) as u16,
            weekday_index: now.weekday().num_days_from_monday() as usize,
            day_ordinal: now.date_naive().num_days_from_ce(),
            in_focus: self.timer.running && self.timer.phase == Phase::Focus,
            in_meeting,
            deep_work: self.deep_work,
        }
    }
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
    fn set_timer_durations_applies_valid_values() {
        let mut s = Settings::default();
        s.set_timer_durations(600, 120, 1200, 6);
        assert_eq!(s.focus_secs, 600);
        assert_eq!(s.short_break_secs, 120);
        assert_eq!(s.long_break_secs, 1200);
        assert_eq!(s.rounds_per_cycle, 6);
    }

    #[test]
    fn set_timer_durations_clamps_out_of_range_values() {
        let mut s = Settings::default();
        s.set_timer_durations(0, u32::MAX, 30, 0);
        assert_eq!(s.focus_secs, 60); // clamped up to the 1-minute floor
        assert_eq!(s.short_break_secs, 4 * 3600); // clamped down to the 4-hour ceiling
        assert_eq!(s.long_break_secs, 60);
        assert_eq!(s.rounds_per_cycle, 1); // clamped up from 0

        s.set_timer_durations(600, 120, 1200, 255);
        assert_eq!(s.rounds_per_cycle, 12); // clamped down from 255
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
