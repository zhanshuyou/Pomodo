use serde::{Deserialize, Serialize};

use crate::core::reminder_copy::{self, Builtin};
use crate::model::Tone;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Intensity {
    /// 轻量气泡 — 右上角滑入，6 秒自动收起
    Bubble,
    /// 宠物来闹你 — 它蹦起来说话，一个窗口都不遮
    Pet,
    /// 全屏遮罩 — 盖住所有屏幕
    Fullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Schedule {
    Every { minutes: u32 },
    DailyAt { hour: u8, minute: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FocusBehavior {
    /// 推迟到本轮结束
    Defer,
    Silence,
    Interrupt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rules {
    /// Minutes past midnight, inclusive.
    pub active_from_min: u16,
    pub active_to_min: u16,
    /// Monday-first.
    pub weekdays: [bool; 7],
    pub during_focus: FocusBehavior,
    pub silence_in_meeting: bool,
    pub escalate_after: u8,
    pub sound: String,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            active_from_min: 9 * 60 + 30,
            active_to_min: 18 * 60 + 30,
            weekdays: [true, true, true, true, true, false, false],
            during_focus: FocusBehavior::Defer,
            silence_in_meeting: true,
            escalate_after: 3,
            sound: "木鱼 · 30%".to_string(),
        }
    }
}

/// Everything the engine needs to know about "now" to decide whether to fire.
/// Passed in rather than read from the clock so every rule is unit-testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireContext {
    pub minute_of_day: u16,
    /// 0 = Monday.
    pub weekday_index: usize,
    /// Days since a fixed epoch (`NaiveDate::num_days_from_ce`). Used, together with
    /// `last_daily_fire`, to tell "already fired today" apart from "the clock is past
    /// the target minute" — the two are conflated if you only track the minute, which
    /// is what let a `DailyAt` reminder go silent forever once the machine slept
    /// through its exact target minute.
    pub day_ordinal: i32,
    pub in_focus: bool,
    pub in_meeting: bool,
    pub deep_work: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickOutcome {
    Idle,
    Deferred,
    Fire(Intensity),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminder {
    pub id: u32,
    pub builtin: Option<Builtin>,
    pub name: String,
    pub color: String,
    pub detail: String,
    pub message: String,
    pub hint: String,
    /// True once the user has typed their own message; retone() then leaves it alone.
    pub message_edited: bool,
    pub schedule: Schedule,
    pub intensity: Intensity,
    pub enabled: bool,
    pub rules: Rules,
    /// Countdown for `Schedule::Every`.
    pub remaining_secs: u32,
    pub consecutive_ignores: u8,
    pub deferred: bool,
    /// Day-ordinal (`FireContext::day_ordinal`) a `DailyAt` reminder last fired on,
    /// so it fires once per day — including on the tick that first notices the
    /// machine woke up past the target minute.
    pub last_daily_fire: Option<i32>,
}

fn seed_schedule(b: Builtin) -> Schedule {
    match b {
        Builtin::Stand => Schedule::Every { minutes: 45 },
        Builtin::Water => Schedule::Every { minutes: 30 },
        Builtin::Eyes => Schedule::Every { minutes: 20 },
        Builtin::Review => Schedule::DailyAt {
            hour: 17,
            minute: 30,
        },
    }
}

fn seed_intensity(b: Builtin) -> Intensity {
    match b {
        Builtin::Stand => Intensity::Pet,
        Builtin::Water | Builtin::Eyes => Intensity::Bubble,
        Builtin::Review => Intensity::Fullscreen,
    }
}

fn interval_secs(schedule: Schedule) -> u32 {
    match schedule {
        Schedule::Every { minutes } => minutes.saturating_mul(60).max(1),
        Schedule::DailyAt { .. } => u32::MAX,
    }
}

impl Reminder {
    pub fn seed(builtin: Builtin, id: u32, tone: Tone) -> Self {
        let schedule = seed_schedule(builtin);
        Self {
            id,
            builtin: Some(builtin),
            name: reminder_copy::name(builtin).to_string(),
            color: reminder_copy::color(builtin).to_string(),
            detail: reminder_copy::detail(builtin).to_string(),
            message: reminder_copy::message(builtin, tone).to_string(),
            hint: reminder_copy::hint(builtin, tone).to_string(),
            message_edited: false,
            schedule,
            intensity: seed_intensity(builtin),
            enabled: true,
            rules: Rules::default(),
            remaining_secs: interval_secs(schedule),
            consecutive_ignores: 0,
            deferred: false,
            last_daily_fire: None,
        }
    }

    /// A reminder the user created from a template chip or from ＋ 空白.
    pub fn blank(id: u32, name: String, color: String) -> Self {
        let schedule = Schedule::Every { minutes: 45 };
        Self {
            id,
            builtin: None,
            detail: "每 45 分钟 · 宠物提示 · 工作时段".to_string(),
            name,
            color,
            message: String::new(),
            hint: "自定义提醒：时间、文案、方式都可改。".to_string(),
            message_edited: true,
            schedule,
            intensity: Intensity::Pet,
            enabled: true,
            rules: Rules::default(),
            remaining_secs: interval_secs(schedule),
            consecutive_ignores: 0,
            deferred: false,
            last_daily_fire: None,
        }
    }

    /// Rewrite the copy for a new tone, unless the user has edited it.
    pub fn retone(&mut self, tone: Tone) {
        let Some(b) = self.builtin else { return };
        self.hint = reminder_copy::hint(b, tone).to_string();
        if !self.message_edited {
            self.message = reminder_copy::message(b, tone).to_string();
        }
    }

    pub fn is_active_now(&self, ctx: &FireContext) -> bool {
        if !self
            .rules
            .weekdays
            .get(ctx.weekday_index)
            .copied()
            .unwrap_or(false)
        {
            return false;
        }
        ctx.minute_of_day >= self.rules.active_from_min
            && ctx.minute_of_day <= self.rules.active_to_min
    }

    /// The intensity this firing should actually use.
    ///
    /// Deep work flattens everything to a bubble ("深度工作时全部自动降到最轻那档").
    /// Otherwise, a run of ignores promotes one firing to fullscreen and then resets.
    fn fire(&mut self, ctx: &FireContext) -> TickOutcome {
        if ctx.deep_work {
            return TickOutcome::Fire(Intensity::Bubble);
        }
        if self.consecutive_ignores >= self.rules.escalate_after {
            self.consecutive_ignores = 0;
            return TickOutcome::Fire(Intensity::Fullscreen);
        }
        TickOutcome::Fire(self.intensity)
    }

    /// Advance this reminder by `elapsed_secs` of real time.
    ///
    /// The countdown is decremented and rearmed *before* the suppression checks, so a
    /// reminder silenced by a meeting or an inactive window consumes its interval
    /// rather than piling up a backlog that all fires at 09:30 the next morning.
    pub fn tick(&mut self, elapsed_secs: u32, ctx: &FireContext) -> TickOutcome {
        if !self.enabled {
            return TickOutcome::Idle;
        }

        match self.schedule {
            Schedule::Every { .. } => {
                self.remaining_secs = self.remaining_secs.saturating_sub(elapsed_secs);
                if self.remaining_secs > 0 {
                    return TickOutcome::Idle;
                }
                self.remaining_secs = interval_secs(self.schedule);
            }
            Schedule::DailyAt { hour, minute } => {
                let target = hour as u16 * 60 + minute as u16;
                // Fires on the first tick at or after the target minute each day, so a
                // machine that slept through the exact minute still gets it on wake.
                if ctx.minute_of_day < target || self.last_daily_fire == Some(ctx.day_ordinal) {
                    return TickOutcome::Idle;
                }
                self.last_daily_fire = Some(ctx.day_ordinal);
            }
        }

        if self.deferred {
            return TickOutcome::Idle;
        }
        if !self.is_active_now(ctx) {
            return TickOutcome::Idle;
        }
        if ctx.in_meeting && self.rules.silence_in_meeting {
            return TickOutcome::Idle;
        }
        if ctx.in_focus {
            match self.rules.during_focus {
                FocusBehavior::Silence => return TickOutcome::Idle,
                FocusBehavior::Defer => {
                    self.deferred = true;
                    return TickOutcome::Deferred;
                }
                FocusBehavior::Interrupt => {}
            }
        }

        self.fire(ctx)
    }

    /// Called at the end of a focus round. Returns true if this reminder had something
    /// waiting, in which case the caller should fire it now.
    pub fn release_deferred(&mut self) -> bool {
        if !self.deferred {
            return false;
        }
        self.deferred = false;
        true
    }

    pub fn acknowledge(&mut self) {
        self.consecutive_ignores = 0;
        self.deferred = false;
    }

    /// Seconds until this reminder next wants attention, for the tray's 接下来轮到 list.
    /// `None` means it will not fire again today.
    pub fn seconds_until_due(&self, ctx: &FireContext) -> Option<u32> {
        if !self.enabled {
            return None;
        }
        if self.deferred {
            return Some(0);
        }
        match self.schedule {
            Schedule::Every { .. } => Some(self.remaining_secs),
            Schedule::DailyAt { hour, minute } => {
                let target = hour as u32 * 60 + minute as u32;
                let now = ctx.minute_of_day as u32;
                if target < now {
                    None
                } else {
                    Some((target - now) * 60)
                }
            }
        }
    }

    pub fn ignore(&mut self) {
        self.consecutive_ignores = self.consecutive_ignores.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Tone;

    /// Wednesday 14:00, not focusing, no meeting, deep work off.
    fn ctx() -> FireContext {
        FireContext {
            minute_of_day: 14 * 60,
            weekday_index: 2,
            day_ordinal: 100,
            in_focus: false,
            in_meeting: false,
            deep_work: false,
        }
    }

    fn water() -> Reminder {
        Reminder::seed(Builtin::Water, 1, Tone::Playful)
    }

    #[test]
    fn seeding_fills_name_color_message_and_interval_from_the_copy_table() {
        let r = water();
        assert_eq!(r.name, "喝水");
        assert_eq!(r.color, "oklch(0.66 0.09 195)");
        assert_eq!(r.message, "你的杯子在喊你，它说它很空。");
        assert_eq!(r.schedule, Schedule::Every { minutes: 30 });
        assert!(r.enabled);
        assert!(!r.message_edited);
    }

    #[test]
    fn each_builtin_seeds_its_designed_interval_and_intensity() {
        assert_eq!(
            Reminder::seed(Builtin::Stand, 0, Tone::Playful).schedule,
            Schedule::Every { minutes: 45 }
        );
        assert_eq!(
            Reminder::seed(Builtin::Eyes, 0, Tone::Playful).schedule,
            Schedule::Every { minutes: 20 }
        );
        assert_eq!(
            Reminder::seed(Builtin::Review, 0, Tone::Playful).schedule,
            Schedule::DailyAt {
                hour: 17,
                minute: 30
            }
        );
        assert_eq!(
            Reminder::seed(Builtin::Stand, 0, Tone::Playful).intensity,
            Intensity::Pet
        );
        assert_eq!(
            Reminder::seed(Builtin::Water, 0, Tone::Playful).intensity,
            Intensity::Bubble
        );
        assert_eq!(
            Reminder::seed(Builtin::Review, 0, Tone::Playful).intensity,
            Intensity::Fullscreen
        );
    }

    #[test]
    fn an_interval_reminder_stays_idle_until_its_countdown_expires() {
        let mut r = water();
        assert_eq!(r.tick(1799, &ctx()), TickOutcome::Idle);
        assert_eq!(r.tick(1, &ctx()), TickOutcome::Fire(Intensity::Bubble));
    }

    #[test]
    fn firing_rearms_the_countdown() {
        let mut r = water();
        r.tick(1800, &ctx());
        assert_eq!(r.remaining_secs, 1800);
    }

    #[test]
    fn a_disabled_reminder_never_fires() {
        let mut r = water();
        r.enabled = false;
        assert_eq!(r.tick(99999, &ctx()), TickOutcome::Idle);
    }

    #[test]
    fn a_reminder_outside_its_active_window_does_not_fire() {
        let mut r = water();
        let mut c = ctx();
        c.minute_of_day = 7 * 60; // 07:00, before 09:30
        assert_eq!(r.tick(1800, &c), TickOutcome::Idle);
    }

    #[test]
    fn a_reminder_on_an_excluded_weekday_does_not_fire() {
        let mut r = water();
        let mut c = ctx();
        c.weekday_index = 6; // Sunday; defaults are 周一 – 周五
        assert_eq!(r.tick(1800, &c), TickOutcome::Idle);
    }

    #[test]
    fn focus_defers_rather_than_interrupting() {
        let mut r = water();
        let mut c = ctx();
        c.in_focus = true;
        assert_eq!(r.tick(1800, &c), TickOutcome::Deferred);
        assert!(r.deferred);
        // It does not fire again while still deferred.
        assert_eq!(r.tick(1800, &c), TickOutcome::Idle);
    }

    #[test]
    fn a_deferred_reminder_releases_at_the_end_of_the_round() {
        let mut r = water();
        let mut c = ctx();
        c.in_focus = true;
        r.tick(1800, &c);
        assert!(r.release_deferred());
        assert!(!r.deferred);
        assert!(!r.release_deferred()); // only once
    }

    #[test]
    fn focus_behavior_silence_drops_the_firing_entirely() {
        let mut r = water();
        r.rules.during_focus = FocusBehavior::Silence;
        let mut c = ctx();
        c.in_focus = true;
        assert_eq!(r.tick(1800, &c), TickOutcome::Idle);
        assert!(!r.deferred);
    }

    #[test]
    fn focus_behavior_interrupt_fires_through_focus() {
        let mut r = water();
        r.rules.during_focus = FocusBehavior::Interrupt;
        let mut c = ctx();
        c.in_focus = true;
        assert_eq!(r.tick(1800, &c), TickOutcome::Fire(Intensity::Bubble));
    }

    #[test]
    fn a_meeting_silences_a_reminder_that_asks_for_it() {
        let mut r = water();
        let mut c = ctx();
        c.in_meeting = true;
        assert_eq!(r.tick(1800, &c), TickOutcome::Idle);
    }

    #[test]
    fn deep_work_demotes_every_intensity_to_a_bubble() {
        let mut r = Reminder::seed(Builtin::Review, 3, Tone::Playful);
        r.schedule = Schedule::Every { minutes: 1 };
        r.remaining_secs = 60;
        let mut c = ctx();
        c.deep_work = true;
        assert_eq!(r.tick(60, &c), TickOutcome::Fire(Intensity::Bubble));
    }

    #[test]
    fn three_consecutive_ignores_escalate_the_next_firing_to_fullscreen() {
        let mut r = water();
        for _ in 0..3 {
            r.tick(1800, &ctx());
            r.ignore();
        }
        assert_eq!(
            r.tick(1800, &ctx()),
            TickOutcome::Fire(Intensity::Fullscreen)
        );
    }

    #[test]
    fn escalation_resets_after_it_fires_once() {
        let mut r = water();
        for _ in 0..3 {
            r.tick(1800, &ctx());
            r.ignore();
        }
        r.tick(1800, &ctx()); // the escalated firing
        assert_eq!(r.consecutive_ignores, 0);
        assert_eq!(r.tick(1800, &ctx()), TickOutcome::Fire(Intensity::Bubble));
    }

    #[test]
    fn acknowledging_clears_the_ignore_streak() {
        let mut r = water();
        r.ignore();
        r.ignore();
        r.acknowledge();
        assert_eq!(r.consecutive_ignores, 0);
    }

    #[test]
    fn a_daily_reminder_fires_when_the_clock_reaches_its_minute() {
        let mut r = Reminder::seed(Builtin::Review, 3, Tone::Playful);
        let mut c = ctx();
        c.minute_of_day = 17 * 60 + 29;
        assert_eq!(r.tick(60, &c), TickOutcome::Idle);
        c.minute_of_day = 17 * 60 + 30;
        assert_eq!(r.tick(60, &c), TickOutcome::Fire(Intensity::Fullscreen));
    }

    #[test]
    fn a_daily_reminder_fires_only_once_per_day() {
        let mut r = Reminder::seed(Builtin::Review, 3, Tone::Playful);
        let mut c = ctx();
        c.minute_of_day = 17 * 60 + 30;
        assert_eq!(r.tick(60, &c), TickOutcome::Fire(Intensity::Fullscreen));
        assert_eq!(r.tick(60, &c), TickOutcome::Idle);
        // Next day.
        c.day_ordinal += 1;
        c.minute_of_day = 9 * 60 + 40;
        r.tick(60, &c);
        c.minute_of_day = 17 * 60 + 30;
        assert_eq!(r.tick(60, &c), TickOutcome::Fire(Intensity::Fullscreen));
    }

    #[test]
    fn a_daily_reminder_still_fires_after_sleeping_through_its_minute() {
        // The machine sleeps from 17:00 to 18:00, skipping 17:30 entirely; the next
        // tick after wake should still deliver today's firing rather than going silent.
        let mut r = Reminder::seed(Builtin::Review, 3, Tone::Playful);
        let mut c = ctx();
        c.minute_of_day = 17 * 60;
        assert_eq!(r.tick(60, &c), TickOutcome::Idle);
        c.minute_of_day = 18 * 60;
        assert_eq!(r.tick(3600, &c), TickOutcome::Fire(Intensity::Fullscreen));
        // Still only once, even though the clock stays past the target the rest of the day.
        c.minute_of_day = 18 * 60 + 15;
        assert_eq!(r.tick(900, &c), TickOutcome::Idle);
    }

    #[test]
    fn retone_rewrites_an_unedited_message_and_leaves_an_edited_one_alone() {
        let mut r = water();
        r.retone(Tone::Professional);
        assert_eq!(r.message, "补充 200ml 水，今日 6/8 杯。");

        r.message = "我自己写的".into();
        r.message_edited = true;
        r.retone(Tone::Gentle);
        assert_eq!(r.message, "我自己写的");
    }

    #[test]
    fn a_blank_reminder_defaults_to_a_forty_five_minute_pet_nudge() {
        let r = Reminder::blank(9, "肩颈拉伸".into(), "oklch(0.7 0.12 60)".into());
        assert_eq!(r.schedule, Schedule::Every { minutes: 45 });
        assert_eq!(r.intensity, Intensity::Pet);
        assert!(r.builtin.is_none());
        assert!(r.message_edited);
    }

    #[test]
    fn default_rules_match_the_spec() {
        let r = Rules::default();
        assert_eq!(r.active_from_min, 9 * 60 + 30);
        assert_eq!(r.active_to_min, 18 * 60 + 30);
        assert_eq!(r.weekdays, [true, true, true, true, true, false, false]);
        assert_eq!(r.during_focus, FocusBehavior::Defer);
        assert!(r.silence_in_meeting);
        assert_eq!(r.escalate_after, 3);
        assert_eq!(r.sound, "木鱼 · 30%");
    }

    #[test]
    fn seconds_until_due_reports_the_interval_countdown() {
        let mut r = water();
        r.tick(600, &ctx());
        assert_eq!(r.seconds_until_due(&ctx()), Some(1200));
    }

    #[test]
    fn seconds_until_due_is_none_for_a_disabled_reminder() {
        let mut r = water();
        r.enabled = false;
        assert_eq!(r.seconds_until_due(&ctx()), None);
    }

    #[test]
    fn seconds_until_due_counts_forward_to_a_daily_time() {
        let r = Reminder::seed(Builtin::Review, 3, Tone::Playful);
        let mut c = ctx();
        c.minute_of_day = 17 * 60; // 17:00, target 17:30
        assert_eq!(r.seconds_until_due(&c), Some(30 * 60));
    }

    #[test]
    fn a_daily_time_already_past_reports_none() {
        let r = Reminder::seed(Builtin::Review, 3, Tone::Playful);
        let mut c = ctx();
        c.minute_of_day = 18 * 60;
        assert_eq!(r.seconds_until_due(&c), None);
    }

    #[test]
    fn a_deferred_reminder_reports_zero_seconds() {
        let mut r = water();
        let mut c = ctx();
        c.in_focus = true;
        r.tick(1800, &c);
        assert_eq!(r.seconds_until_due(&ctx()), Some(0));
    }
}
