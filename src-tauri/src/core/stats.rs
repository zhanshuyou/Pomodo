use chrono::{DateTime, Datelike, Days, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Serialize};

use crate::model::{Model, TaskId};

/// One focus phase. `completed` is false when the user skipped out of it,
/// which is what the design counts under 中断次数.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub started_at: i64,
    pub secs: u32,
    pub task: Option<TaskId>,
    pub completed: bool,
}

impl Session {
    fn date(&self) -> Option<NaiveDate> {
        DateTime::<Utc>::from_timestamp(self.started_at, 0).map(|dt| dt.date_naive())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub sessions: Vec<Session>,
    pub best_streak: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DayBar {
    pub label: String,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsSummary {
    pub week_focus_secs: u32,
    pub week_delta_pct: i32,
    pub pomodoros: u32,
    pub daily_average: f32,
    pub interruptions: u32,
    pub interruptions_delta: i32,
    pub streak: u32,
    pub best_streak: u32,
    pub bars: Vec<DayBar>,
}

fn weekday_label(date: NaiveDate) -> &'static str {
    match date.weekday() {
        Weekday::Mon => "一",
        Weekday::Tue => "二",
        Weekday::Wed => "三",
        Weekday::Thu => "四",
        Weekday::Fri => "五",
        Weekday::Sat => "六",
        Weekday::Sun => "日",
    }
}

impl Stats {
    pub fn record(&mut self, session: Session) {
        self.sessions.push(session);
    }

    /// Completed pomodoros per day for the `days` days ending on `today`, oldest first.
    pub fn daily_counts(&self, today: NaiveDate, days: usize) -> Vec<u32> {
        (0..days)
            .rev()
            .map(|back| {
                let Some(date) = today.checked_sub_days(Days::new(back as u64)) else {
                    return 0;
                };
                self.sessions
                    .iter()
                    .filter(|s| s.completed && s.date() == Some(date))
                    .count() as u32
            })
            .collect()
    }

    /// Consecutive days with at least one completed pomodoro, ending at today or
    /// yesterday. A day still in progress does not break the streak.
    pub fn streak(&self, today: NaiveDate) -> u32 {
        let has = |date: NaiveDate| {
            self.sessions
                .iter()
                .any(|s| s.completed && s.date() == Some(date))
        };

        // Anchor on today if it already has a session, otherwise on yesterday.
        let mut cursor = if has(today) {
            today
        } else {
            match today.checked_sub_days(Days::new(1)) {
                Some(y) if has(y) => y,
                _ => return 0,
            }
        };

        let mut count = 0;
        while has(cursor) {
            count += 1;
            match cursor.checked_sub_days(Days::new(1)) {
                Some(prev) => cursor = prev,
                None => break,
            }
        }
        count
    }

    /// (completed count, focus secs, interruptions) over the 7 days ending
    /// `offset_days` before today.
    fn window(&self, today: NaiveDate, offset_days: u64) -> (u32, u32, u32) {
        let end = today
            .checked_sub_days(Days::new(offset_days))
            .unwrap_or(today);
        let start = end.checked_sub_days(Days::new(6)).unwrap_or(end);

        let mut completed = 0;
        let mut secs = 0;
        let mut interruptions = 0;
        for session in &self.sessions {
            let Some(date) = session.date() else { continue };
            if date < start || date > end {
                continue;
            }
            if session.completed {
                completed += 1;
                secs += session.secs;
            } else {
                interruptions += 1;
            }
        }
        (completed, secs, interruptions)
    }

    pub fn summary(&self, today: NaiveDate) -> StatsSummary {
        let (pomodoros, week_focus_secs, interruptions) = self.window(today, 0);
        let (_, prev_secs, prev_interruptions) = self.window(today, 7);

        let week_delta_pct = if prev_secs == 0 {
            0
        } else {
            (((week_focus_secs as f64 - prev_secs as f64) / prev_secs as f64) * 100.0).round()
                as i32
        };

        let counts = self.daily_counts(today, 14);
        let bars = counts
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let back = (13 - i) as u64;
                let date = today.checked_sub_days(Days::new(back)).unwrap_or(today);
                DayBar {
                    label: weekday_label(date).to_string(),
                    count,
                }
            })
            .collect();

        let streak = self.streak(today);

        StatsSummary {
            week_focus_secs,
            week_delta_pct,
            pomodoros,
            daily_average: pomodoros as f32 / 7.0,
            interruptions,
            interruptions_delta: interruptions as i32 - prev_interruptions as i32,
            streak,
            best_streak: self.best_streak.max(streak),
            bars,
        }
    }
}

impl Model {
    /// Record a finished (or abandoned) focus phase against stats, the pet and the task.
    pub fn record_focus_phase(&mut self, completed: bool, secs: u32) {
        self.stats.record(Session {
            started_at: Utc::now().timestamp(),
            secs,
            task: self.timer.active_task,
            completed,
        });
        if completed {
            self.pet.credit();
            if let Some(id) = self.timer.active_task {
                self.credit_task(id);
            }
        }
        let today = Utc::now().date_naive();
        self.stats.best_streak = self.stats.best_streak.max(self.stats.streak(today));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// A completed 25-minute focus session at noon on the given day.
    fn session_on(date: NaiveDate) -> Session {
        Session {
            started_at: Utc
                .from_utc_datetime(&date.and_hms_opt(12, 0, 0).unwrap())
                .timestamp(),
            secs: 1500,
            task: None,
            completed: true,
        }
    }

    #[test]
    fn record_appends_and_tracks_the_best_streak() {
        let mut stats = Stats::default();
        stats.record(session_on(day(2026, 8, 19)));
        assert_eq!(stats.sessions.len(), 1);
    }

    #[test]
    fn daily_counts_returns_oldest_first_and_pads_empty_days() {
        let mut stats = Stats::default();
        stats.record(session_on(day(2026, 8, 19)));
        stats.record(session_on(day(2026, 8, 19)));
        stats.record(session_on(day(2026, 8, 17)));

        let counts = stats.daily_counts(day(2026, 8, 19), 4);
        // 8/16, 8/17, 8/18, 8/19
        assert_eq!(counts, vec![0, 1, 0, 2]);
    }

    #[test]
    fn daily_counts_ignores_incomplete_sessions() {
        let mut stats = Stats::default();
        let mut skipped = session_on(day(2026, 8, 19));
        skipped.completed = false;
        stats.record(skipped);
        assert_eq!(stats.daily_counts(day(2026, 8, 19), 1), vec![0]);
    }

    #[test]
    fn streak_counts_consecutive_days_ending_today() {
        let mut stats = Stats::default();
        for d in 17..=19 {
            stats.record(session_on(day(2026, 8, d)));
        }
        assert_eq!(stats.streak(day(2026, 8, 19)), 3);
    }

    #[test]
    fn streak_survives_a_day_with_no_session_yet_today() {
        // Nothing today, but yesterday and the day before: the streak is still alive.
        let mut stats = Stats::default();
        stats.record(session_on(day(2026, 8, 17)));
        stats.record(session_on(day(2026, 8, 18)));
        assert_eq!(stats.streak(day(2026, 8, 19)), 2);
    }

    #[test]
    fn streak_breaks_on_a_two_day_gap() {
        let mut stats = Stats::default();
        stats.record(session_on(day(2026, 8, 15)));
        stats.record(session_on(day(2026, 8, 19)));
        assert_eq!(stats.streak(day(2026, 8, 19)), 1);
    }

    #[test]
    fn streak_is_zero_with_no_sessions() {
        assert_eq!(Stats::default().streak(day(2026, 8, 19)), 0);
    }

    #[test]
    fn summary_totals_the_last_seven_days_and_compares_to_the_week_before() {
        let mut stats = Stats::default();
        // Previous week (8/6 - 8/12): 4 pomodoros.
        for d in 6..=9 {
            stats.record(session_on(day(2026, 8, d)));
        }
        // This week (8/13 - 8/19): 8 pomodoros.
        for d in 13..=16 {
            stats.record(session_on(day(2026, 8, d)));
            stats.record(session_on(day(2026, 8, d)));
        }

        let s = stats.summary(day(2026, 8, 19));
        assert_eq!(s.pomodoros, 8);
        assert_eq!(s.week_focus_secs, 8 * 1500);
        assert_eq!(s.week_delta_pct, 100); // doubled
        assert_eq!(s.bars.len(), 14);
    }

    #[test]
    fn summary_reports_zero_delta_when_the_previous_week_was_empty() {
        let mut stats = Stats::default();
        stats.record(session_on(day(2026, 8, 19)));
        assert_eq!(stats.summary(day(2026, 8, 19)).week_delta_pct, 0);
    }

    #[test]
    fn summary_counts_interruptions_from_incomplete_sessions() {
        let mut stats = Stats::default();
        let mut skipped = session_on(day(2026, 8, 19));
        skipped.completed = false;
        stats.record(skipped.clone());
        stats.record(skipped);
        stats.record(session_on(day(2026, 8, 19)));

        let s = stats.summary(day(2026, 8, 19));
        assert_eq!(s.interruptions, 2);
        assert_eq!(s.pomodoros, 1);
    }

    #[test]
    fn summary_labels_the_bars_with_chinese_weekday_characters() {
        let s = Stats::default().summary(day(2026, 8, 19)); // 2026-08-19 is a Wednesday
        assert_eq!(s.bars.len(), 14);
        assert_eq!(s.bars[13].label, "三");
        assert_eq!(s.bars[12].label, "二");
    }

    #[test]
    fn daily_average_divides_the_week_by_seven() {
        let mut stats = Stats::default();
        for d in 13..=19 {
            stats.record(session_on(day(2026, 8, d)));
        }
        let s = stats.summary(day(2026, 8, 19));
        assert!((s.daily_average - 1.0).abs() < 1e-6);
    }
}
