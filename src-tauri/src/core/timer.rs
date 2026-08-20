use serde::{Deserialize, Serialize};

use crate::model::{Phase, Settings, TaskId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timer {
    pub phase: Phase,
    pub remaining_secs: u32,
    pub running: bool,
    /// 1..=rounds_per_cycle, shown in the design as 第 {round}/4 轮.
    pub round: u8,
    pub active_task: Option<TaskId>,
}

impl Default for Timer {
    fn default() -> Self {
        Self::new(&Settings::default())
    }
}

/// One phase boundary crossed. `completed` is false when the user skipped,
/// which is how the stats layer tells a finished pomodoro from an abandoned one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseChange {
    pub from: Phase,
    pub to: Phase,
    pub round: u8,
    pub completed: bool,
}

impl Timer {
    pub fn new(settings: &Settings) -> Self {
        Self {
            phase: Phase::Focus,
            remaining_secs: settings.focus_secs,
            running: false,
            round: 1,
            active_task: None,
        }
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.running = false;
    }

    /// The phase that follows `self.phase`, and the round it lands on.
    fn next_phase(&self, settings: &Settings) -> (Phase, u8) {
        match self.phase {
            // A focus round ends in the long break only after the last round of the cycle.
            Phase::Focus => {
                if self.round >= settings.rounds_per_cycle {
                    (Phase::LongBreak, self.round)
                } else {
                    (Phase::ShortBreak, self.round)
                }
            }
            // Breaks hand back to focus; the round counter moves on the way out of a break.
            Phase::ShortBreak => (Phase::Focus, self.round.saturating_add(1)),
            Phase::LongBreak => (Phase::Focus, 1),
        }
    }

    fn transition(&mut self, settings: &Settings, completed: bool) -> PhaseChange {
        let from = self.phase;
        let (to, round) = self.next_phase(settings);
        self.phase = to;
        self.round = round.min(settings.rounds_per_cycle.max(1));
        self.remaining_secs = settings.duration_for(to);
        PhaseChange {
            from,
            to,
            round: self.round,
            completed,
        }
    }

    /// Move the clock forward by `elapsed_secs` of real time.
    ///
    /// Taking a delta rather than ticking once per call means a machine that slept
    /// for an hour resumes correctly: the caller passes the whole gap and every phase
    /// boundary it crossed comes back in order.
    pub fn advance(&mut self, elapsed_secs: u32, settings: &Settings) -> Vec<PhaseChange> {
        let mut changes = Vec::new();
        if !self.running {
            return changes;
        }
        let mut left = elapsed_secs;
        // Guard against a zero-length phase configuration spinning forever.
        while left >= self.remaining_secs && self.remaining_secs > 0 {
            left -= self.remaining_secs;
            changes.push(self.transition(settings, true));
            if settings.duration_for(self.phase) == 0 {
                break;
            }
        }
        self.remaining_secs = self.remaining_secs.saturating_sub(left);
        changes
    }

    /// Jump to the next phase without crediting the current one.
    pub fn skip(&mut self, settings: &Settings) -> PhaseChange {
        self.transition(settings, false)
    }

    /// Fraction of the current phase already elapsed, 0.0..=1.0.
    pub fn progress(&self, settings: &Settings) -> f32 {
        let total = settings.duration_for(self.phase);
        if total == 0 {
            return 1.0;
        }
        let elapsed = total.saturating_sub(self.remaining_secs) as f32;
        (elapsed / total as f32).clamp(0.0, 1.0)
    }

    /// Filled cells of the ten on the pet's belly — the design's `round(pct / 10)`.
    pub fn belly_cells(&self, settings: &Settings) -> u8 {
        (self.progress(settings) * 10.0).round() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Phase, Settings};

    fn settings() -> Settings {
        Settings::default()
    }

    #[test]
    fn starts_paused_on_a_full_focus_round_one() {
        let t = Timer::new(&settings());
        assert_eq!(t.phase, Phase::Focus);
        assert_eq!(t.remaining_secs, 1500);
        assert_eq!(t.round, 1);
        assert!(!t.running);
    }

    #[test]
    fn advance_does_nothing_while_paused() {
        let mut t = Timer::new(&settings());
        assert!(t.advance(60, &settings()).is_empty());
        assert_eq!(t.remaining_secs, 1500);
    }

    #[test]
    fn advance_decrements_while_running() {
        let mut t = Timer::new(&settings());
        t.start();
        assert!(t.advance(90, &settings()).is_empty());
        assert_eq!(t.remaining_secs, 1410);
    }

    #[test]
    fn focus_completing_goes_to_a_short_break_without_advancing_the_round() {
        let mut t = Timer::new(&settings());
        t.start();
        let changes = t.advance(1500, &settings());
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].from, Phase::Focus);
        assert_eq!(changes[0].to, Phase::ShortBreak);
        assert!(changes[0].completed);
        assert_eq!(t.phase, Phase::ShortBreak);
        assert_eq!(t.remaining_secs, 300);
        assert_eq!(t.round, 1);
    }

    #[test]
    fn a_short_break_completing_advances_the_round() {
        let mut t = Timer::new(&settings());
        t.start();
        t.advance(1500 + 300, &settings());
        assert_eq!(t.phase, Phase::Focus);
        assert_eq!(t.round, 2);
        assert_eq!(t.remaining_secs, 1500);
    }

    #[test]
    fn the_fourth_focus_is_followed_by_a_long_break() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.round = 4;
        t.start();
        let changes = t.advance(1500, &s);
        assert_eq!(changes[0].to, Phase::LongBreak);
        assert_eq!(t.remaining_secs, 900);
    }

    #[test]
    fn a_long_break_completing_resets_to_round_one() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.round = 4;
        t.start();
        t.advance(1500 + 900, &s);
        assert_eq!(t.phase, Phase::Focus);
        assert_eq!(t.round, 1);
    }

    #[test]
    fn a_long_sleep_rolls_through_every_phase_it_crossed() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.start();
        // 25 focus + 5 break + 25 focus + 5 break = 3600s exactly.
        let changes = t.advance(3600, &s);
        assert_eq!(changes.len(), 4);
        assert_eq!(t.round, 3);
        assert_eq!(t.phase, Phase::Focus);
        assert_eq!(t.remaining_secs, 1500);
    }

    #[test]
    fn skipping_a_phase_is_not_a_completion() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.start();
        t.advance(60, &s);
        let change = t.skip(&s);
        assert!(!change.completed);
        assert_eq!(change.to, Phase::ShortBreak);
        assert_eq!(t.remaining_secs, 300);
    }

    #[test]
    fn skipping_a_break_still_advances_the_round() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.phase = Phase::ShortBreak;
        t.remaining_secs = 300;
        let change = t.skip(&s);
        assert_eq!(change.to, Phase::Focus);
        assert_eq!(t.round, 2);
    }

    #[test]
    fn progress_and_belly_cells_track_the_elapsed_fraction() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.start();
        t.advance(750, &s);
        assert!((t.progress(&s) - 0.5).abs() < 1e-6);
        assert_eq!(t.belly_cells(&s), 5);

        t.advance(300, &s);
        assert_eq!(t.belly_cells(&s), 7); // 1050/1500 = 70%
    }

    #[test]
    fn pausing_stops_the_clock() {
        let s = settings();
        let mut t = Timer::new(&s);
        t.start();
        t.advance(60, &s);
        t.pause();
        t.advance(600, &s);
        assert_eq!(t.remaining_secs, 1440);
    }
}
