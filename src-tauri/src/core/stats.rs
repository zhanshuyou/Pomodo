use chrono::{DateTime, Datelike, Days, Local, NaiveDate, Timelike, Utc, Weekday};
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
    fn local(&self) -> Option<DateTime<Local>> {
        DateTime::<Utc>::from_timestamp(self.started_at, 0).map(|dt| dt.with_timezone(&Local))
    }

    fn date(&self) -> Option<NaiveDate> {
        self.local().map(|dt| dt.date_naive())
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

/// The hour-of-day bucket with the worst interruption rate, for the
/// 「被打断最多的时段」 insight card. `end_hour` is `start_hour + 1`, wrapped at midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterruptionHotspot {
    pub start_hour: u8,
    pub end_hour: u8,
    pub interruptions: u32,
    pub total: u32,
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
    /// `None` when no hour bucket has enough sessions yet to say anything meaningful,
    /// rather than showing a made-up conclusion.
    pub interruption_hotspot: Option<InterruptionHotspot>,
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

    /// Sum of each completed session's own recorded length on `date`. Unlike
    /// `daily_counts(date, 1).0 * settings.focus_secs`, this stays correct even
    /// after `focus_secs` has been changed, since every session carries the
    /// duration it actually ran under.
    pub fn day_focus_secs(&self, date: NaiveDate) -> u32 {
        self.sessions
            .iter()
            .filter(|s| s.completed && s.date() == Some(date))
            .map(|s| s.secs)
            .sum()
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

    /// The hour-of-day bucket with the worst interruption rate over the 14 days
    /// ending `today` (matching the two-week chart above the card). Only considers
    /// buckets with at least `MIN_SESSIONS` sessions, so a single bad round early on
    /// doesn't read as a pattern.
    fn interruption_hotspot(&self, today: NaiveDate) -> Option<InterruptionHotspot> {
        const MIN_SESSIONS: u32 = 3;
        let start = today.checked_sub_days(Days::new(13)).unwrap_or(today);

        // (interruptions, total) per hour of day.
        let mut buckets = [(0u32, 0u32); 24];
        for session in &self.sessions {
            let Some(local) = session.local() else {
                continue;
            };
            let date = local.date_naive();
            if date < start || date > today {
                continue;
            }
            let bucket = &mut buckets[local.hour() as usize];
            bucket.1 += 1;
            if !session.completed {
                bucket.0 += 1;
            }
        }

        buckets
            .iter()
            .enumerate()
            .filter(|&(_, &(_, total))| total >= MIN_SESSIONS)
            .max_by(|&(_, &(a_int, a_total)), &(_, &(b_int, b_total))| {
                let a_rate = f64::from(a_int) / f64::from(a_total);
                let b_rate = f64::from(b_int) / f64::from(b_total);
                a_rate
                    .total_cmp(&b_rate)
                    .then_with(|| a_total.cmp(&b_total))
            })
            .map(|(hour, &(interruptions, total))| InterruptionHotspot {
                start_hour: hour as u8,
                end_hour: ((hour + 1) % 24) as u8,
                interruptions,
                total,
            })
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
            interruption_hotspot: self.interruption_hotspot(today),
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
        let today = Local::now().date_naive();
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

    /// A session at the given local hour on the given day, built in local time
    /// (rather than UTC noon like `session_on`) so hour-bucketing tests don't
    /// depend on the machine's timezone offset.
    fn session_at(date: NaiveDate, hour: u32, completed: bool) -> Session {
        Session {
            started_at: Local
                .from_local_datetime(&date.and_hms_opt(hour, 0, 0).unwrap())
                .single()
                .expect("unambiguous local time")
                .timestamp(),
            secs: 1500,
            task: None,
            completed,
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
    fn day_focus_secs_sums_each_sessions_own_length() {
        let mut stats = Stats::default();
        // Simulates focus_secs having been 1500s for one session, then changed to
        // 1200s for the next — the total should reflect what actually ran, not
        // `count * current_focus_secs`.
        stats.record(session_on(day(2026, 8, 19)));
        let mut shorter = session_on(day(2026, 8, 19));
        shorter.secs = 1200;
        stats.record(shorter);
        stats.record(session_on(day(2026, 8, 18)));

        assert_eq!(stats.day_focus_secs(day(2026, 8, 19)), 1500 + 1200);
    }

    #[test]
    fn day_focus_secs_ignores_incomplete_sessions() {
        let mut stats = Stats::default();
        let mut skipped = session_on(day(2026, 8, 19));
        skipped.completed = false;
        stats.record(skipped);
        assert_eq!(stats.day_focus_secs(day(2026, 8, 19)), 0);
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
    fn interruption_hotspot_picks_the_bucket_with_the_worst_rate() {
        let mut stats = Stats::default();
        let today = day(2026, 8, 19);
        // 15:00 bucket: 3 sessions, 2 interrupted — a 67% rate.
        stats.record(session_at(today, 15, false));
        stats.record(session_at(today, 15, false));
        stats.record(session_at(today, 15, true));
        // 10:00 bucket: 3 sessions, 1 interrupted — a lower rate.
        stats.record(session_at(today, 10, false));
        stats.record(session_at(today, 10, true));
        stats.record(session_at(today, 10, true));

        let hotspot = stats
            .summary(today)
            .interruption_hotspot
            .expect("a hotspot with enough data");
        assert_eq!(hotspot.start_hour, 15);
        assert_eq!(hotspot.end_hour, 16);
        assert_eq!(hotspot.interruptions, 2);
        assert_eq!(hotspot.total, 3);
    }

    #[test]
    fn interruption_hotspot_is_none_without_enough_sessions_in_any_bucket() {
        let mut stats = Stats::default();
        let today = day(2026, 8, 19);
        stats.record(session_at(today, 15, false));
        stats.record(session_at(today, 15, false));
        assert!(stats.summary(today).interruption_hotspot.is_none());
    }

    #[test]
    fn interruption_hotspot_ignores_sessions_outside_the_two_week_window() {
        let mut stats = Stats::default();
        let today = day(2026, 8, 19);
        let too_old = today.checked_sub_days(Days::new(14)).unwrap();
        for _ in 0..5 {
            stats.record(session_at(too_old, 15, false));
        }
        assert!(stats.summary(today).interruption_hotspot.is_none());
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
