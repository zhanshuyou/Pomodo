# Pomodo 05 — Reminder Engine + 设置 Window Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the reminder engine — interval and daily schedules, active windows, defer-to-round-end, three-strike escalation, deep-work demotion — and the 设置 window's three-layer 提醒 pane from artboard 03. Wire the acknowledged counts back into the 专注 tab's body stats.

**Architecture:** `core/reminder.rs` holds a `Reminder` list and a pure `due(&self, ctx) -> Option<Intensity>` decision per reminder, driven by the same tick that runs the timer. Firing emits `reminder:fire`; plan 07 renders the three intensities as real windows, so this plan surfaces them in-window and logs to the console. Reminder copy lives in Rust because the engine needs the resolved string when it fires; the frontend just displays what the model carries.

**Tech Stack:** Rust 2021, chrono, Svelte 5 runes, Tauri 2 multi-window.

**Spec:** `docs/superpowers/specs/2026-08-19-momo-design.md`
**Depends on:** plans 01–04 complete.

## Global Constraints

- The four seeded reminders and all 24 of their tone variants come from spec §5.4 verbatim. A single character out of place fails this plan.
- Template chip names from spec §5.5: 站立, 喝水, 护眼, 深呼吸, 肩颈拉伸, 记一句想法, and `＋ 空白` with a fullwidth plus.
- Advanced rule defaults from spec §6.3: 09:30–18:30, 周一–周五, 推迟到本轮结束, 静默, 升级为全屏, 木鱼 · 30%.
- Interval chips are exactly 20 / 30 / 45 / 60 min. Intensity cards are 气泡 (角落一闪) / 宠物 (它跳给你看) / 全屏 (躲不掉).
- Scheduling decisions take the current time as a parameter. No core function may call `Local::now()` internally.
- Changing the tone rewrites the message of every reminder the user has *not* edited, and leaves edited ones alone.
- Meeting detection is macOS-only and lands in plan 07; here the hook exists and always reports "no meeting".
- The full gate stays green: `npm test`, `npm run check`, `npm run build`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`.

---

## File Structure

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/core/reminder.rs` | `Reminder`, `Schedule`, `Intensity`, `Rules`, the due/fire decision |
| `src-tauri/src/core/reminder_copy.rs` | The tone table for the four built-in reminders |
| `src-tauri/src/model.rs` | gains `reminders`, `body`, `deep_work` |
| `src-tauri/src/state.rs` | ticks the reminder engine, emits `reminder:fire` |
| `src-tauri/src/commands.rs` | reminder CRUD, ack/snooze, deep work |
| `src-tauri/src/windows.rs` | opens the `prefs` window |
| `src/routes/prefs/App.svelte` | 设置 chrome + sidebar |
| `src/routes/prefs/RemindersPane.svelte` | Artboard 03, all three layers |
| `src/routes/prefs/TimerPane.svelte`, `PetPane.svelte`, `SoundPane.svelte`, `GeneralPane.svelte` | The other four sidebar entries |
| `src/routes/main/TaskSidebar.svelte` | body stats now read real counters |

---

### Task 1: Reminder copy table

**Files:**
- Create: `src-tauri/src/core/reminder_copy.rs`
- Modify: `src-tauri/src/core/mod.rs`

**Interfaces:**
- Consumes: `Tone` from `model.rs`.
- Produces:
  - `pub enum Builtin { Stand, Water, Eyes, Review }`
  - `pub fn name(b: Builtin) -> &'static str`
  - `pub fn color(b: Builtin) -> &'static str`
  - `pub fn detail(b: Builtin) -> &'static str`
  - `pub fn message(b: Builtin, tone: Tone) -> &'static str`
  - `pub fn hint(b: Builtin, tone: Tone) -> &'static str`
  - `pub const ALL: [Builtin; 4]`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/core/reminder_copy.rs` with only the test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Tone;

    #[test]
    fn stand_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Stand, Tone::Professional),
            "已连续坐着 45 分钟，请起身活动 2 分钟。"
        );
        assert_eq!(
            message(Builtin::Stand, Tone::Gentle),
            "坐久了，陪我一起站起来伸个懒腰？"
        );
        assert_eq!(
            message(Builtin::Stand, Tone::Playful),
            "再坐下去你就要跟椅子长在一起了，起来！"
        );
    }

    #[test]
    fn water_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Water, Tone::Professional),
            "补充 200ml 水，今日 6/8 杯。"
        );
        assert_eq!(message(Builtin::Water, Tone::Gentle), "喝口水吧，今天第 7 杯了。");
        assert_eq!(
            message(Builtin::Water, Tone::Playful),
            "你的杯子在喊你，它说它很空。"
        );
    }

    #[test]
    fn eyes_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Eyes, Tone::Professional),
            "看向 6 米外物体并保持 20 秒。"
        );
        assert_eq!(message(Builtin::Eyes, Tone::Gentle), "抬头看看窗外，20 秒就好。");
        assert_eq!(
            message(Builtin::Eyes, Tone::Playful),
            "眼睛快冒烟了，看看远方压压火。"
        );
    }

    #[test]
    fn review_carries_all_three_message_tones() {
        assert_eq!(
            message(Builtin::Review, Tone::Professional),
            "用 5 分钟复盘今天并规划明天。"
        );
        assert_eq!(
            message(Builtin::Review, Tone::Gentle),
            "收工前，和我一起理一理今天？"
        );
        assert_eq!(
            message(Builtin::Review, Tone::Playful),
            "先夸自己一句，再写下明天要干的事。"
        );
    }

    #[test]
    fn every_builtin_has_three_distinct_hints() {
        for b in ALL {
            let a = hint(b, Tone::Professional);
            let g = hint(b, Tone::Gentle);
            let p = hint(b, Tone::Playful);
            assert_ne!(a, g);
            assert_ne!(g, p);
            assert_ne!(a, p);
        }
    }

    #[test]
    fn hints_match_the_spec_for_stand() {
        assert_eq!(hint(Builtin::Stand, Tone::Professional), "专注进行中时会推迟到本轮结束。");
        assert_eq!(hint(Builtin::Stand, Tone::Gentle), "我会等你这轮结束再叫你。");
        assert_eq!(
            hint(Builtin::Stand, Tone::Playful),
            "我不打断你，但下课钟一响我就扑上来。"
        );
    }

    #[test]
    fn names_details_and_colors_match_the_spec() {
        assert_eq!(name(Builtin::Stand), "站起来动一动");
        assert_eq!(name(Builtin::Water), "喝水");
        assert_eq!(name(Builtin::Eyes), "远眺护眼");
        assert_eq!(name(Builtin::Review), "收工前复盘");

        assert_eq!(color(Builtin::Water), "oklch(0.66 0.09 195)");
        assert_eq!(detail(Builtin::Eyes), "每 20 分钟 · 轻量气泡 · 20-20-20");
        assert_eq!(detail(Builtin::Review), "每天 17:30 · 全屏 · 仅工作日");
    }
}
```

Add `pub mod reminder_copy;` to `src-tauri/src/core/mod.rs`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test core::reminder_copy::`
Expected: FAIL — `Builtin` is not defined.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/core/reminder_copy.rs`. Every string is copied from spec §5.4.

```rust
use serde::{Deserialize, Serialize};

use crate::model::Tone;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Builtin {
    Stand,
    Water,
    Eyes,
    Review,
}

pub const ALL: [Builtin; 4] = [Builtin::Stand, Builtin::Water, Builtin::Eyes, Builtin::Review];

pub fn name(b: Builtin) -> &'static str {
    match b {
        Builtin::Stand => "站起来动一动",
        Builtin::Water => "喝水",
        Builtin::Eyes => "远眺护眼",
        Builtin::Review => "收工前复盘",
    }
}

pub fn color(b: Builtin) -> &'static str {
    match b {
        Builtin::Stand => "oklch(0.63 0.13 40)",
        Builtin::Water => "oklch(0.66 0.09 195)",
        Builtin::Eyes => "oklch(0.7 0.1 145)",
        Builtin::Review => "oklch(0.68 0.1 300)",
    }
}

pub fn detail(b: Builtin) -> &'static str {
    match b {
        Builtin::Stand => "每 45 分钟 · 宠物提示 · 工作时段",
        Builtin::Water => "每 30 分钟 · 轻量气泡 · 计入每日 8 杯",
        Builtin::Eyes => "每 20 分钟 · 轻量气泡 · 20-20-20",
        Builtin::Review => "每天 17:30 · 全屏 · 仅工作日",
    }
}

fn pick(tone: Tone, professional: &'static str, gentle: &'static str, playful: &'static str) -> &'static str {
    match tone {
        Tone::Professional => professional,
        Tone::Gentle => gentle,
        Tone::Playful => playful,
    }
}

pub fn message(b: Builtin, tone: Tone) -> &'static str {
    match b {
        Builtin::Stand => pick(
            tone,
            "已连续坐着 45 分钟，请起身活动 2 分钟。",
            "坐久了，陪我一起站起来伸个懒腰？",
            "再坐下去你就要跟椅子长在一起了，起来！",
        ),
        Builtin::Water => pick(
            tone,
            "补充 200ml 水，今日 6/8 杯。",
            "喝口水吧，今天第 7 杯了。",
            "你的杯子在喊你，它说它很空。",
        ),
        Builtin::Eyes => pick(
            tone,
            "看向 6 米外物体并保持 20 秒。",
            "抬头看看窗外，20 秒就好。",
            "眼睛快冒烟了，看看远方压压火。",
        ),
        Builtin::Review => pick(
            tone,
            "用 5 分钟复盘今天并规划明天。",
            "收工前，和我一起理一理今天？",
            "先夸自己一句，再写下明天要干的事。",
        ),
    }
}

pub fn hint(b: Builtin, tone: Tone) -> &'static str {
    match b {
        Builtin::Stand => pick(
            tone,
            "专注进行中时会推迟到本轮结束。",
            "我会等你这轮结束再叫你。",
            "我不打断你，但下课钟一响我就扑上来。",
        ),
        Builtin::Water => pick(
            tone,
            "菜单栏会累计今日饮水杯数。",
            "我帮你数着杯数。",
            "我偷偷在小本本上记你喝了几杯。",
        ),
        Builtin::Eyes => pick(
            tone,
            "遵循 20-20-20 护眼规则。",
            "20 分钟、20 英尺、20 秒。",
            "我数到 20 就放你走，说好了。",
        ),
        Builtin::Review => pick(
            tone,
            "自定义提醒：时间、文案、方式都可改。",
            "这条完全是你自己写的。",
            "这条是你自己加的，别怪我。",
        ),
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test core::reminder_copy::`
Expected: PASS, 7 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/reminder_copy.rs src-tauri/src/core/mod.rs
git commit -m "feat(rust): add the built-in reminder copy table"
```

---

### Task 2: The reminder model and scheduling decision

**Files:**
- Create: `src-tauri/src/core/reminder.rs`
- Modify: `src-tauri/src/core/mod.rs`

**Interfaces:**
- Consumes: `Builtin` and the copy functions, `Tone`, `Phase`.
- Produces:
  - `pub enum Intensity { Bubble, Pet, Fullscreen }`
  - `pub enum Schedule { Every { minutes: u32 }, DailyAt { hour: u8, minute: u8 } }`
  - `pub enum FocusBehavior { Defer, Silence, Interrupt }`
  - `pub struct Rules { active_from_min: u16, active_to_min: u16, weekdays: [bool; 7], during_focus: FocusBehavior, silence_in_meeting: bool, escalate_after: u8, sound: String }`
  - `pub struct Reminder { id: u32, builtin: Option<Builtin>, name: String, color: String, detail: String, message: String, hint: String, message_edited: bool, schedule: Schedule, intensity: Intensity, enabled: bool, rules: Rules, remaining_secs: u32, consecutive_ignores: u8, deferred: bool }`
  - `pub struct FireContext { pub minute_of_day: u16, pub weekday_index: usize, pub in_focus: bool, pub in_meeting: bool, pub deep_work: bool }`
  - `pub enum TickOutcome { Idle, Deferred, Fire(Intensity) }`
  - `impl Reminder`: `seed(builtin, id, tone) -> Reminder`, `blank(id, name, color) -> Reminder`, `tick(&mut self, elapsed_secs, ctx) -> TickOutcome`, `release_deferred(&mut self) -> bool`, `acknowledge(&mut self)`, `ignore(&mut self)`, `retone(&mut self, tone)`, `is_active_now(&self, ctx) -> bool`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/core/reminder.rs` with only the test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Tone;

    /// Wednesday 14:00, not focusing, no meeting, deep work off.
    fn ctx() -> FireContext {
        FireContext {
            minute_of_day: 14 * 60,
            weekday_index: 2,
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
            Schedule::DailyAt { hour: 17, minute: 30 }
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
        assert_eq!(r.tick(1800, &ctx()), TickOutcome::Fire(Intensity::Fullscreen));
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
        c.minute_of_day = 9 * 60 + 40;
        r.tick(60, &c);
        c.minute_of_day = 17 * 60 + 30;
        assert_eq!(r.tick(60, &c), TickOutcome::Fire(Intensity::Fullscreen));
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
}
```

Add `pub mod reminder;` to `src-tauri/src/core/mod.rs`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test core::reminder::`
Expected: FAIL — `Reminder` is not defined.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/core/reminder.rs`:

```rust
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
    /// Last minute-of-day a `DailyAt` reminder fired, so it fires once per day.
    pub last_daily_fire: Option<u16>,
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
        if !self.rules.weekdays.get(ctx.weekday_index).copied().unwrap_or(false) {
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
    pub fn tick(&mut self, elapsed_secs: u32, ctx: &FireContext) -> TickOutcome {
        if !self.enabled {
            return TickOutcome::Idle;
        }

        let ready = match self.schedule {
            Schedule::Every { .. } => {
                self.remaining_secs = self.remaining_secs.saturating_sub(elapsed_secs);
                if self.remaining_secs > 0 {
                    return TickOutcome::Idle;
                }
                self.remaining_secs = interval_secs(self.schedule);
                true
            }
            Schedule::DailyAt { hour, minute } => {
                let target = hour as u16 * 60 + minute as u16;
                let hit = ctx.minute_of_day == target;
                // Clear the once-per-day latch as soon as the clock moves off the target.
                if !hit {
                    self.last_daily_fire = None;
                    return TickOutcome::Idle;
                }
                if self.last_daily_fire == Some(target) {
                    return TickOutcome::Idle;
                }
                self.last_daily_fire = Some(target);
                true
            }
        };

        if !ready {
            return TickOutcome::Idle;
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

    pub fn ignore(&mut self) {
        self.consecutive_ignores = self.consecutive_ignores.saturating_add(1);
    }
}
```

Note the deliberate ordering inside `tick`: the countdown is decremented (and rearmed)
*before* any of the suppression checks run. A reminder that is silenced by a meeting or an
inactive window still consumes its interval rather than piling up a backlog that all fires
at 09:30 the next morning.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test core::reminder::`
Expected: PASS, 21 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/reminder.rs src-tauri/src/core/mod.rs
git commit -m "feat(rust): add the reminder scheduling engine"
```

---

### Task 3: Body counters and model wiring

**Files:**
- Modify: `src-tauri/src/model.rs`, `src-tauri/src/state.rs`, `src-tauri/src/events.rs`, `src-tauri/src/store.rs`

**Interfaces:**
- Produces:
  - `pub struct BodyCounters { pub water_cups: u32, pub water_goal: u32, pub stands: u32, pub stand_goal: u32, pub longest_sit_mins: u32, pub day: String }` — `day` is an ISO date so the counters reset daily
  - `Model` gains `pub reminders: Vec<Reminder>`, `pub body: BodyCounters`, `pub deep_work: bool`, `pub next_reminder_id: u32`
  - `Model::seed_reminders(&mut self)` — the four built-ins, in spec order
  - `Model::fire_context(now: DateTime<Local>) -> FireContext`
  - `events::REMINDER_FIRE = "reminder:fire"` with `FirePayload { id, name, message, intensity, color }`
  - `AppState::tick` runs the reminder engine and emits

- [ ] **Step 1: Write the failing test**

Append to the test module in `src-tauri/src/state.rs`:

```rust
    #[test]
    fn a_fresh_model_seeds_the_four_builtin_reminders_in_order() {
        let state = AppState::new(store_in("reminders"));
        let names: Vec<String> = state
            .snapshot()
            .reminders
            .iter()
            .map(|r| r.name.clone())
            .collect();
        assert_eq!(names, vec!["站起来动一动", "喝水", "远眺护眼", "收工前复盘"]);
    }

    #[test]
    fn body_counters_reset_when_the_day_changes() {
        let state = AppState::new(store_in("body"));
        state.with(|m| {
            m.body.water_cups = 5;
            m.body.day = "2020-01-01".into();
            m.roll_body_day("2026-08-19");
        });
        let body = state.snapshot().body;
        assert_eq!(body.water_cups, 0);
        assert_eq!(body.day, "2026-08-19");
        assert_eq!(body.water_goal, 8);
        assert_eq!(body.stand_goal, 6);
    }

    #[test]
    fn body_counters_survive_a_tick_on_the_same_day() {
        let state = AppState::new(store_in("body-same"));
        state.with(|m| {
            m.body.day = "2026-08-19".into();
            m.body.water_cups = 3;
            m.roll_body_day("2026-08-19");
        });
        assert_eq!(state.snapshot().body.water_cups, 3);
    }

    #[test]
    fn changing_tone_retones_every_unedited_reminder() {
        use crate::model::Tone;
        let state = AppState::new(store_in("retone"));
        state.with(|m| {
            m.settings.tone = Tone::Professional;
            m.retone_reminders();
        });
        let model = state.snapshot();
        assert_eq!(model.reminders[1].message, "补充 200ml 水，今日 6/8 杯。");
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test state::`
Expected: FAIL — `Model` has no field `reminders`.

- [ ] **Step 3: Extend the model**

In `src-tauri/src/model.rs`, add:

```rust
use crate::core::reminder::{FireContext, Reminder};
use crate::core::reminder_copy;
use chrono::{DateTime, Datelike, Local, Timelike};

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
```

Add the four fields to `Model`:

```rust
    #[serde(default)]
    pub reminders: Vec<Reminder>,
    #[serde(default)]
    pub body: BodyCounters,
    #[serde(default)]
    pub deep_work: bool,
    #[serde(default)]
    pub next_reminder_id: u32,
```

And the helpers:

```rust
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

    pub fn fire_context(&self, now: DateTime<Local>) -> FireContext {
        FireContext {
            minute_of_day: (now.hour() * 60 + now.minute()) as u16,
            weekday_index: now.weekday().num_days_from_monday() as usize,
            in_focus: self.timer.running && self.timer.phase == Phase::Focus,
            // Meeting detection is macOS-only and lands in plan 07.
            in_meeting: false,
            deep_work: self.deep_work,
        }
    }
}
```

In `src-tauri/src/store.rs`, bump `SCHEMA_VERSION` to `3` and call `seed_reminders` in
`fresh()`:

```rust
fn fresh() -> Model {
    let mut model = Model::default();
    model.seed_demo_tasks();
    model.seed_reminders();
    model
}
```

`AppState::new` must also seed reminders into an older loaded file, so add to its body
after `let model = store.load();`:

```rust
        let mut model = model;
        model.seed_reminders();
```

- [ ] **Step 4: Add the fire event**

Append to `src-tauri/src/events.rs`:

```rust
use crate::core::reminder::Intensity;

pub const REMINDER_FIRE: &str = "reminder:fire";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirePayload {
    pub id: u32,
    pub name: String,
    pub message: String,
    pub intensity: Intensity,
    pub color: String,
}
```

Add `Reminders` to `Section` and its `"reminders"` string, plus `Body` → `"body"`.

- [ ] **Step 5: Run the reminder engine from the tick**

In `src-tauri/src/state.rs`, extend `tick` so it drives reminders after the timer:

```rust
    fn run_reminders(&self, app: &AppHandle, elapsed_secs: u32, round_ended: bool) {
        use crate::core::reminder::TickOutcome;

        let now = chrono::Local::now();
        let today = now.format("%Y-%m-%d").to_string();

        let fires = self.with(|m| {
            m.roll_body_day(&today);
            let ctx = m.fire_context(now);
            let mut fires = Vec::new();

            for reminder in &mut m.reminders {
                // A round that just ended releases anything parked during focus.
                if round_ended && reminder.release_deferred() {
                    fires.push((reminder.id, reminder.intensity));
                }
                match reminder.tick(elapsed_secs, &ctx) {
                    TickOutcome::Fire(intensity) => fires.push((reminder.id, intensity)),
                    TickOutcome::Idle | TickOutcome::Deferred => {}
                }
            }

            fires
                .into_iter()
                .filter_map(|(id, intensity)| {
                    m.reminders.iter().find(|r| r.id == id).map(|r| FirePayload {
                        id: r.id,
                        name: r.name.clone(),
                        message: r.message.clone(),
                        intensity,
                        color: r.color.clone(),
                    })
                })
                .collect::<Vec<_>>()
        });

        for payload in fires {
            let _ = app.emit(events::REMINDER_FIRE, payload);
        }
    }
```

Call it at the end of `tick`, passing `round_ended = changes.iter().any(|c| c.from == Phase::Focus)`.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cd src-tauri && cargo test`
Expected: PASS, all suites.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src
git commit -m "feat(rust): wire reminders, body counters and the fire event"
```

---

### Task 4: Reminder commands and the prefs window

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/windows.rs`

**Interfaces:**
- Produces:
  - `add_reminder(template: Option<String>) -> u32` — templates: 站立 喝水 护眼 深呼吸 肩颈拉伸 记一句想法; `None` is ＋ 空白
  - `update_reminder(id, patch: ReminderPatch)` where `ReminderPatch { name, message, interval_minutes, intensity, enabled, rules }`, all optional
  - `toggle_reminder(id)`, `delete_reminder(id)`
  - `ack_reminder(id)`, `ignore_reminder(id)`, `snooze_reminder(id, minutes)`
  - `set_deep_work(value: bool)`
  - `open_prefs()` — shows the `prefs` window, creating it on first call
  - `set_tone` now calls `retone_reminders`

- [ ] **Step 1: Declare the prefs window**

In `src-tauri/tauri.conf.json`, add a second entry to `app.windows`:

```json
{
  "label": "prefs",
  "url": "prefs.html",
  "title": "设置",
  "width": 1180,
  "height": 640,
  "minWidth": 1180,
  "minHeight": 606,
  "visible": false,
  "resizable": true,
  "titleBarStyle": "Overlay",
  "hiddenTitle": true
}
```

- [ ] **Step 2: Write windows.rs**

Create `src-tauri/src/windows.rs`:

```rust
use tauri::{AppHandle, Manager};

/// Show the preferences window, creating it if the config-declared one is gone.
pub fn show_prefs(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("prefs") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(app, "prefs", tauri::WebviewUrl::App("prefs.html".into()))
        .title("设置")
        .inner_size(1180.0, 640.0)
        .min_inner_size(1180.0, 606.0)
        .build()?;
    Ok(())
}
```

Add `pub mod windows;` to `lib.rs`.

- [ ] **Step 3: Write the commands**

Append to `src-tauri/src/commands.rs`:

```rust
use serde::Deserialize;

use crate::core::reminder::{Intensity, Reminder, Rules, Schedule};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderPatch {
    pub name: Option<String>,
    pub message: Option<String>,
    pub interval_minutes: Option<u32>,
    pub intensity: Option<Intensity>,
    pub enabled: Option<bool>,
    pub rules: Option<Rules>,
}

/// Template chips from the design's 从模板抓一个 row.
fn template_color(name: &str) -> &'static str {
    match name {
        "站立" => "oklch(0.63 0.13 40)",
        "喝水" => "oklch(0.66 0.09 195)",
        "护眼" => "oklch(0.7 0.1 145)",
        "深呼吸" => "oklch(0.68 0.1 300)",
        "肩颈拉伸" => "oklch(0.7 0.12 60)",
        "记一句想法" => "oklch(0.62 0.07 250)",
        _ => "oklch(0.63 0.13 40)",
    }
}

#[tauri::command]
pub fn add_reminder(state: State<'_, AppState>, app: AppHandle, template: Option<String>) -> u32 {
    let id = state.with(|m| {
        let id = m.next_reminder_id;
        m.next_reminder_id += 1;
        let name = template.clone().unwrap_or_else(|| "新提醒".to_string());
        let color = template_color(&name).to_string();
        m.reminders.push(Reminder::blank(id, name, color));
        id
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
    id
}

#[tauri::command]
pub fn update_reminder(
    state: State<'_, AppState>,
    app: AppHandle,
    id: u32,
    patch: ReminderPatch,
) {
    state.with(|m| {
        let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) else {
            return;
        };
        if let Some(name) = patch.name {
            r.name = name;
        }
        if let Some(message) = patch.message {
            r.message = message;
            // Once the user writes their own words, a tone change must not overwrite them.
            r.message_edited = true;
        }
        if let Some(minutes) = patch.interval_minutes {
            r.schedule = Schedule::Every { minutes };
            r.remaining_secs = minutes.saturating_mul(60).max(1);
        }
        if let Some(intensity) = patch.intensity {
            r.intensity = intensity;
        }
        if let Some(enabled) = patch.enabled {
            r.enabled = enabled;
        }
        if let Some(rules) = patch.rules {
            r.rules = rules;
        }
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn toggle_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            r.enabled = !r.enabled;
        }
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn delete_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| m.reminders.retain(|r| r.id != id));
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

/// The user did the thing. Clears the ignore streak and moves the body counters.
#[tauri::command]
pub fn ack_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| {
        let builtin = m
            .reminders
            .iter_mut()
            .find(|r| r.id == id)
            .and_then(|r| {
                r.acknowledge();
                r.builtin
            });
        match builtin {
            Some(crate::core::reminder_copy::Builtin::Water) => m.body.water_cups += 1,
            Some(crate::core::reminder_copy::Builtin::Stand) => m.body.stands += 1,
            _ => {}
        }
    });
    state.emit_changed(&app, Section::Body);
    state.flush();
}

#[tauri::command]
pub fn ignore_reminder(state: State<'_, AppState>, app: AppHandle, id: u32) {
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            r.ignore();
        }
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn snooze_reminder(state: State<'_, AppState>, app: AppHandle, id: u32, minutes: u32) {
    state.with(|m| {
        if let Some(r) = m.reminders.iter_mut().find(|r| r.id == id) {
            r.remaining_secs = minutes.saturating_mul(60).max(1);
            r.deferred = false;
        }
    });
    state.emit_changed(&app, Section::Reminders);
    state.flush();
}

#[tauri::command]
pub fn set_deep_work(state: State<'_, AppState>, app: AppHandle, value: bool) {
    state.with(|m| m.deep_work = value);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

#[tauri::command]
pub fn open_prefs(app: AppHandle) -> Result<(), String> {
    crate::windows::show_prefs(&app).map_err(|e| e.to_string())
}
```

Update `set_tone` to retone:

```rust
#[tauri::command]
pub fn set_tone(state: State<'_, AppState>, app: AppHandle, tone: Tone) {
    state.with(|m| {
        m.settings.tone = tone;
        m.retone_reminders();
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
}
```

Register all ten new commands in `generate_handler!`.

- [ ] **Step 4: Verify**

Run: `cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add src-tauri
git commit -m "feat(rust): add reminder commands and the prefs window"
```

---

### Task 5: Frontend IPC and store extensions

**Files:**
- Modify: `src/lib/ipc.ts`, `src/lib/state.svelte.ts`

**Interfaces:**
- Produces the TS mirrors of every Rust type added above, plus `app.reminders`, `app.body`,
  `app.deepWork`, and `app.onFire(cb)`.

- [ ] **Step 1: Add the types and calls**

Append to `src/lib/ipc.ts`:

```ts
export type Intensity = "bubble" | "pet" | "fullscreen";
export type FocusBehavior = "defer" | "silence" | "interrupt";

export type Schedule =
  | { kind: "every"; minutes: number }
  | { kind: "dailyAt"; hour: number; minute: number };

export interface Rules {
  activeFromMin: number;
  activeToMin: number;
  weekdays: boolean[];
  duringFocus: FocusBehavior;
  silenceInMeeting: boolean;
  escalateAfter: number;
  sound: string;
}

export interface Reminder {
  id: number;
  builtin: "stand" | "water" | "eyes" | "review" | null;
  name: string;
  color: string;
  detail: string;
  message: string;
  hint: string;
  messageEdited: boolean;
  schedule: Schedule;
  intensity: Intensity;
  enabled: boolean;
  rules: Rules;
  remainingSecs: number;
  consecutiveIgnores: number;
  deferred: boolean;
  lastDailyFire: number | null;
}

export interface BodyCounters {
  waterCups: number;
  waterGoal: number;
  stands: number;
  standGoal: number;
  longestSitMins: number;
  day: string;
}

export interface FirePayload {
  id: number;
  name: string;
  message: string;
  intensity: Intensity;
  color: string;
}

export interface ReminderPatch {
  name?: string;
  message?: string;
  intervalMinutes?: number;
  intensity?: Intensity;
  enabled?: boolean;
  rules?: Rules;
}

export const addReminder = (template: string | null) =>
  invoke<number>("add_reminder", { template });
export const updateReminder = (id: number, patch: ReminderPatch) =>
  invoke<void>("update_reminder", { id, patch });
export const toggleReminder = (id: number) => invoke<void>("toggle_reminder", { id });
export const deleteReminder = (id: number) => invoke<void>("delete_reminder", { id });
export const ackReminder = (id: number) => invoke<void>("ack_reminder", { id });
export const ignoreReminder = (id: number) => invoke<void>("ignore_reminder", { id });
export const snoozeReminder = (id: number, minutes: number) =>
  invoke<void>("snooze_reminder", { id, minutes });
export const setDeepWork = (value: boolean) => invoke<void>("set_deep_work", { value });
export const openPrefs = () => invoke<void>("open_prefs");

export const onReminderFire = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  listen<FirePayload>("reminder:fire", (e) => cb(e.payload));
```

Extend `Model` with `reminders: Reminder[]`, `body: BodyCounters`, `deepWork: boolean`,
`nextReminderId: number`, and add matching entries to `FALLBACK` in
`src/lib/state.svelte.ts` (empty array, zeroed counters with goals 8 and 6, `false`, `0`).

Add to `AppStore`:

```ts
  get reminders() {
    return this.model.reminders;
  }
  get body() {
    return this.model.body;
  }
  get deepWork() {
    return this.model.deepWork;
  }
```

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib
git commit -m "feat: expose reminders and body counters to the frontend"
```

---

### Task 6: Real body stats in the 专注 sidebar

**Files:**
- Modify: `src/routes/main/TaskSidebar.svelte`

- [ ] **Step 1: Replace the hard-coded array**

In `src/routes/main/TaskSidebar.svelte`, delete the `BODY_STATS` constant and replace it
with a derived value:

```ts
  const bodyStats = $derived([
    {
      name: "喝水",
      value: `${app.body.waterCups} / ${app.body.waterGoal} 杯`,
      pct: (app.body.waterCups / Math.max(1, app.body.waterGoal)) * 100,
      color: "oklch(0.66 0.09 195)",
    },
    {
      name: "站立",
      value: `${app.body.stands} / ${app.body.standGoal} 次`,
      pct: (app.body.stands / Math.max(1, app.body.standGoal)) * 100,
      color: "oklch(0.63 0.13 40)",
    },
    {
      name: "久坐最长",
      value: `${app.body.longestSitMins} 分钟`,
      pct: Math.min(100, (app.body.longestSitMins / 90) * 100),
      color: "oklch(0.7 0.12 60)",
    },
  ]);
```

and change the `{#each BODY_STATS ...}` block to `{#each bodyStats as stat (stat.name)}`.

- [ ] **Step 2: Commit**

```bash
git add src/routes/main/TaskSidebar.svelte
git commit -m "feat: drive the body stats from real reminder counters"
```

---

### Task 7: The 设置 window shell

**Files:**
- Modify: `src/routes/prefs/App.svelte`
- Create: `src/routes/prefs/TimerPane.svelte`, `PetPane.svelte`, `SoundPane.svelte`, `GeneralPane.svelte`

**Interfaces:**
- Produces the artboard-03 chrome: 46px title bar reading `设置 — 提醒`, a 172px sidebar
  with 计时 / 提醒 / 宠物 / 声音 / 通用, and the active pane to its right.

- [ ] **Step 1: Write App.svelte**

Replace `src/routes/prefs/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import TitleBar from "../../lib/components/TitleBar.svelte";
  import { app } from "../../lib/state.svelte";
  import GeneralPane from "./GeneralPane.svelte";
  import PetPane from "./PetPane.svelte";
  import RemindersPane from "./RemindersPane.svelte";
  import SoundPane from "./SoundPane.svelte";
  import TimerPane from "./TimerPane.svelte";

  const PANES = ["计时", "提醒", "宠物", "声音", "通用"] as const;
  let active = $state(1);

  onMount(() => {
    void app.init();
    return () => app.dispose();
  });

  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });
</script>

<div class="window">
  <TitleBar title="设置 — {PANES[active]}" />
  <div class="body">
    <nav>
      {#each PANES as name, i (name)}
        <button class="nav" class:active={active === i} type="button" onclick={() => (active = i)}>
          {name}
        </button>
      {/each}
    </nav>

    {#if active === 0}
      <TimerPane />
    {:else if active === 1}
      <RemindersPane />
    {:else if active === 2}
      <PetPane />
    {:else if active === 3}
      <SoundPane />
    {:else}
      <GeneralPane />
    {/if}
  </div>
</div>

<style>
  .window {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--card);
    overflow: hidden;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  nav {
    width: 172px;
    flex: none;
    padding: 16px 12px;
    background: var(--surface-2);
    border-right: 1px solid oklch(0.91 0.008 70);
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .nav {
    padding: 8px 12px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--dim);
    font-size: 13.5px;
    font-weight: 400;
    text-align: left;
    cursor: pointer;
  }
  .nav.active {
    background: var(--card);
    color: var(--ink);
    font-weight: 600;
  }
</style>
```

- [ ] **Step 2: Write the four secondary panes**

Create `src/routes/prefs/TimerPane.svelte`:

```svelte
<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import { app } from "../../lib/state.svelte";

  const mins = (secs: number) => Math.round(secs / 60);
</script>

<div class="pane">
  <h3>时长</h3>
  <div class="rows">
    <div class="row"><span>专注</span><span class="val">{mins(app.settings.focusSecs)} min</span></div>
    <div class="row"><span>短休息</span><span class="val">{mins(app.settings.shortBreakSecs)} min</span></div>
    <div class="row"><span>长休息</span><span class="val">{mins(app.settings.longBreakSecs)} min</span></div>
    <div class="row"><span>一轮几个番茄</span><span class="val">{app.settings.roundsPerCycle}</span></div>
  </div>
  <p class="note">时长调整将在后续版本开放编辑。</p>
</div>

<style>
  .pane {
    flex: 1;
    padding: 22px 26px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  h3 {
    margin: 0;
    font-size: 13.5px;
    font-weight: 600;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 13px;
    max-width: 420px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
  }
  .val {
    padding: 5px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .note {
    margin: 0;
    font-size: 12px;
    color: var(--dim);
  }
</style>
```

Create `src/routes/prefs/PetPane.svelte` and `SoundPane.svelte` with the same shell,
substituting their content:

- `PetPane`: the four behaviour chips, reusing the `FLAGS` list and `setPetFlag` call from
  `src/routes/main/PetTab.svelte`, plus the line `宠物形象与自定义图片请在主窗口的「宠物」标签页设置。`
- `SoundPane`: a single read-only row `提示音` / `木鱼 · 30%` and the note
  `声音库将在后续版本开放。`

Create `src/routes/prefs/GeneralPane.svelte`:

```svelte
<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import { setAccent, setDeepWork, setTone } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";
  import { ACCENTS, type Accent, type Tone } from "../../lib/theme";

  const TONES: { key: Tone; label: string }[] = [
    { key: "professional", label: "克制专业" },
    { key: "gentle", label: "温和陪伴" },
    { key: "playful", label: "俏皮拟人" },
  ];
</script>

<div class="pane">
  <section>
    <h3>强调色</h3>
    <div class="chips">
      {#each Object.entries(ACCENTS) as [key, css] (key)}
        <Chip
          selected={app.settings.accent === key}
          dot={css}
          onclick={() => void setAccent(key as Accent)}
        >
          {key}
        </Chip>
      {/each}
    </div>
  </section>

  <section>
    <h3>说话的口气</h3>
    <div class="chips">
      {#each TONES as t (t.key)}
        <Chip selected={app.settings.tone === t.key} onclick={() => void setTone(t.key)}>
          {t.label}
        </Chip>
      {/each}
    </div>
    <p class="note">改口气会重写所有没被你编辑过的提醒文案。</p>
  </section>

  <section>
    <h3>深度工作</h3>
    <div class="row">
      <span>开启后所有提醒自动降到最轻那档</span>
      <Toggle
        checked={app.deepWork}
        onchange={(v) => void setDeepWork(v)}
        label="深度工作"
      />
    </div>
  </section>
</div>

<style>
  .pane {
    flex: 1;
    padding: 22px 26px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }
  h3 {
    margin: 0 0 10px;
    font-size: 13.5px;
    font-weight: 600;
  }
  .chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    max-width: 480px;
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
  }
  .note {
    margin: 8px 0 0;
    font-size: 12px;
    color: var(--dim);
  }
</style>
```

- [ ] **Step 3: Open prefs from the main window**

In `src/routes/main/App.svelte`, make the `⌘,` label a button that calls `openPrefs()`,
and register the shortcut:

```svelte
<button class="prefslink" type="button" onclick={() => void openPrefs()}>⌘,</button>
```

```ts
  onMount(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        void openPrefs();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
```

Style `.prefslink` as a borderless transparent button inheriting the `.meta` colour.

- [ ] **Step 4: Commit**

```bash
git add src/routes/prefs src/routes/main/App.svelte
git commit -m "feat: add the settings window shell and secondary panes"
```

---

### Task 8: The 提醒 pane — all three layers

**Files:**
- Create: `src/routes/prefs/RemindersPane.svelte`

**Interfaces:**
- Consumes: `app.reminders`, `Toggle`, `Chip`, `PetCanvas`, and the reminder commands.
- Produces: artboard 03's two right-hand columns in full.

- [ ] **Step 1: Write RemindersPane.svelte**

Create `src/routes/prefs/RemindersPane.svelte`:

```svelte
<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import {
    type Intensity,
    addReminder,
    toggleReminder,
    updateReminder,
  } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";
  import { REMINDER_COLORS } from "../../lib/theme";

  const TEMPLATES = [
    { name: "站立", color: REMINDER_COLORS.stand },
    { name: "喝水", color: REMINDER_COLORS.water },
    { name: "护眼", color: REMINDER_COLORS.eyes },
    { name: "深呼吸", color: REMINDER_COLORS.breathe },
    { name: "肩颈拉伸", color: REMINDER_COLORS.stretch },
    { name: "记一句想法", color: REMINDER_COLORS.note },
  ];

  const INTERVALS = [20, 30, 45, 60];

  const STYLES: { key: Intensity; label: string; hint: string }[] = [
    { key: "bubble", label: "气泡", hint: "角落一闪" },
    { key: "pet", label: "宠物", hint: "它跳给你看" },
    { key: "fullscreen", label: "全屏", hint: "躲不掉" },
  ];

  let editId = $state<number | null>(null);
  let advanced = $state(false);

  const editing = $derived(
    app.reminders.find((r) => r.id === editId) ?? app.reminders[0] ?? null,
  );
  const onCount = $derived(app.reminders.filter((r) => r.enabled).length);
  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);

  const currentInterval = $derived(
    editing?.schedule.kind === "every" ? editing.schedule.minutes : null,
  );

  function minutesLabel(min: number): string {
    return `${String(Math.floor(min / 60)).padStart(2, "0")}:${String(min % 60).padStart(2, "0")}`;
  }

  const ruleRows = $derived(
    editing
      ? [
          {
            name: "生效时段",
            value: `${minutesLabel(editing.rules.activeFromMin)} – ${minutesLabel(editing.rules.activeToMin)}`,
          },
          {
            name: "生效日期",
            value: editing.rules.weekdays.slice(0, 5).every(Boolean)
              ? "周一 – 周五"
              : "自定义",
          },
          {
            name: "专注中",
            value:
              editing.rules.duringFocus === "defer"
                ? "推迟到本轮结束"
                : editing.rules.duringFocus === "silence"
                  ? "静默"
                  : "直接打断",
          },
          {
            name: "检测到会议 / 通话",
            value: editing.rules.silenceInMeeting ? "静默" : "照常提醒",
          },
          {
            name: `连续忽略 ${editing.rules.escalateAfter} 次`,
            value: "升级为全屏",
          },
          { name: "声音", value: editing.rules.sound },
        ]
      : [],
  );
</script>

<div class="col2">
  <section>
    <div class="sechead">
      <span class="num">01</span>
      <span class="sectitle">从模板抓一个</span>
    </div>
    <div class="chips">
      {#each TEMPLATES as t (t.name)}
        <Chip dot={t.color} onclick={() => void addReminder(t.name)}>{t.name}</Chip>
      {/each}
      <button class="blank" type="button" onclick={() => void addReminder(null)}>
        ＋ 空白
      </button>
    </div>
  </section>

  <div class="divider"></div>

  <section>
    <div class="sechead">
      <span class="num">02</span>
      <span class="sectitle">你的提醒</span>
      <span class="oncount">{onCount} 条开启</span>
    </div>

    {#each app.reminders as r (r.id)}
      <div
        class="rem"
        class:sel={editing?.id === r.id}
        role="button"
        tabindex="0"
        onclick={() => (editId = r.id)}
        onkeydown={(e) => e.key === "Enter" && (editId = r.id)}
      >
        <span class="tile" style:background={r.color} style:opacity={r.enabled ? 1 : 0.28}></span>
        <div class="remtext">
          <span class="remname">{r.name}</span>
          <span class="remdetail">{r.detail}</span>
        </div>
        <Toggle
          checked={r.enabled}
          onchange={() => void toggleReminder(r.id)}
          label="{r.name} 开关"
        />
      </div>
    {/each}
  </section>
</div>

<div class="col3">
  {#if editing}
    <div class="sechead">
      <span class="num">03</span>
      <span class="sectitle">编辑「{editing.name}」</span>
    </div>

    <div class="field">
      <span class="flabel">它会怎么说</span>
      <textarea
        class="message"
        rows="2"
        value={editing.message}
        onchange={(e) =>
          void updateReminder(editing.id, {
            message: (e.currentTarget as HTMLTextAreaElement).value,
          })}
      ></textarea>
    </div>

    <div class="field">
      <span class="flabel">多久一次</span>
      <div class="chips">
        {#each INTERVALS as min (min)}
          <Chip
            selected={currentInterval === min}
            onclick={() => void updateReminder(editing.id, { intervalMinutes: min })}
          >
            <span class="mono">{min} min</span>
          </Chip>
        {/each}
      </div>
    </div>

    <div class="field">
      <span class="flabel">怎么打扰你</span>
      <div class="styles">
        {#each STYLES as s (s.key)}
          <button
            class="style"
            class:sel={editing.intensity === s.key}
            type="button"
            onclick={() => void updateReminder(editing.id, { intensity: s.key })}
          >
            <span class="slabel">{s.label}</span>
            <span class="shint">{s.hint}</span>
          </button>
        {/each}
      </div>
    </div>

    <button class="disclose" type="button" onclick={() => (advanced = !advanced)}>
      <span>{advanced ? "收起精细规则" : "还要更精细？展开规则"}</span>
      <span class="arrow">{advanced ? "▲" : "▼"}</span>
    </button>

    {#if advanced}
      <div class="rules">
        {#each ruleRows as row (row.name)}
          <div class="rrow">
            <span class="rname">{row.name}</span>
            <span class="rvalue">{row.value}</span>
          </div>
        {/each}
      </div>
    {/if}

    <div class="hintcard">
      <PetCanvas map={pet.map} body={pet.body} scale={3} alt={pet.name} />
      <span>{editing.hint}</span>
    </div>
  {/if}
</div>

<style>
  .col2 {
    width: 396px;
    flex: none;
    padding: 22px;
    border-right: 1px solid oklch(0.91 0.008 70);
    display: flex;
    flex-direction: column;
    gap: 18px;
    overflow-y: auto;
  }
  .col3 {
    flex: 1;
    padding: 22px 26px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
    min-width: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 11px;
  }
  .sechead {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .num {
    font-family: var(--font-pixel);
    font-size: 10px;
    color: var(--accent);
  }
  .sectitle {
    font-size: 13.5px;
    font-weight: 600;
  }
  .oncount {
    margin-left: auto;
    font-size: 12px;
    color: var(--faint);
  }
  .chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .blank {
    padding: 7px 11px;
    border: 1px dashed oklch(0.84 0.008 70);
    border-radius: var(--radius-chip);
    background: transparent;
    font-size: 12.5px;
    color: var(--dim);
    cursor: pointer;
  }
  .divider {
    height: 1px;
    background: var(--line-soft);
  }
  .rem {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 13px;
    border: 1px solid var(--line);
    border-radius: var(--radius-control);
    background: var(--card);
    cursor: pointer;
  }
  .rem.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .tile {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    flex: none;
  }
  .remtext {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .remname {
    font-size: 13.5px;
    font-weight: 600;
  }
  .remdetail {
    font-size: 12px;
    color: var(--dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .flabel {
    font-size: 12px;
    color: var(--dim);
  }
  .message {
    padding: 11px 13px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: oklch(0.985 0.004 80);
    font-family: inherit;
    font-size: 13.5px;
    line-height: 1.5;
    color: var(--ink);
    resize: vertical;
  }
  .mono {
    font-family: var(--font-mono);
  }
  .styles {
    display: flex;
    gap: 7px;
  }
  .style {
    flex: 1;
    padding: 11px 12px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--card);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
  }
  .style.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .slabel {
    font-size: 12.5px;
    font-weight: 600;
    color: var(--ink);
  }
  .shint {
    font-size: 11.5px;
    color: var(--faint);
  }
  .disclose {
    display: flex;
    align-items: center;
    gap: 8px;
    border: none;
    background: transparent;
    padding: 0;
    font-family: inherit;
    font-size: 12.5px;
    color: var(--accent);
    cursor: pointer;
  }
  .arrow {
    font-size: 11px;
  }
  .rules {
    padding: 16px 18px;
    border: 1px solid oklch(0.9 0.008 70);
    border-radius: 12px;
    background: oklch(0.975 0.006 70);
    display: flex;
    flex-direction: column;
    gap: 13px;
  }
  .rrow {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .rname {
    font-size: 12.5px;
    color: oklch(0.42 0.012 60);
  }
  .rvalue {
    padding: 5px 10px;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--card);
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .hintcard {
    margin-top: auto;
    padding: 14px 16px;
    border-radius: 12px;
    background: oklch(0.96 0.012 70);
    display: flex;
    gap: 12px;
    align-items: center;
  }
  .hintcard span {
    font-size: 12.5px;
    color: oklch(0.45 0.012 60);
    line-height: 1.5;
  }
</style>
```

- [ ] **Step 2: Verify**

Run: `npm run check && npm run build`
Expected: both pass.

- [ ] **Step 3: Commit**

```bash
git add src/routes/prefs/RemindersPane.svelte
git commit -m "feat: build the three-layer reminders settings pane"
```

---

### Task 9: End-to-end verification

**Files:** none.

- [ ] **Step 1: Launch and open settings**

Run: `npm run tauri dev`, then press ⌘, in the main window.
Expected: the 设置 window opens with 提醒 selected and the title reads `设置 — 提醒`.

- [ ] **Step 2: Check the layout against artboard 03**

Expected: 172px sidebar, 396px middle column, flexible right column; six template chips
plus a dashed `＋ 空白`; four reminder rows with 26px colour tiles and switches; the right
column showing 编辑「站起来动一动」 with the message box, four interval chips, three
intensity cards, the disclosure link, and the pet hint card pinned to the bottom.

- [ ] **Step 3: Check behaviour**

1. Click 喝水 in the list → the right column retitles and shows 30 min selected and 气泡
   highlighted.
2. Click 45 min → `list_model` shows `schedule: { kind: "every", minutes: 45 }`.
3. Click 全屏 → the intensity card highlights and persists across a restart.
4. Toggle a reminder off → its tile fades to 28% and `n 条开启` decreases.
5. Click 还要更精细？展开规则 → six rule rows appear showing `09:30 – 18:30`, `周一 – 周五`,
   `推迟到本轮结束`, `静默`, `升级为全屏`, `木鱼 · 30%`.
6. Click a template chip → a new reminder appears at the bottom of the list.
7. In 通用, switch the tone to 克制专业 → every unedited reminder's message rewrites; edit
   one message by hand, switch tone again, and confirm the edited one is untouched.

- [ ] **Step 4: Check firing**

In the main window's devtools console:

```js
const { invoke } = await import("@tauri-apps/api/core");
const { listen } = await import("@tauri-apps/api/event");
await listen("reminder:fire", (e) => console.log("FIRE", e.payload));
// Make 喝水 fire in a minute; ids come from list_model.
const m = await invoke("list_model");
const water = m.reminders.find((r) => r.name === "喝水");
await invoke("update_reminder", { id: water.id, patch: { intervalMinutes: 1 } });
```

Expected within ~60 s (during the 09:30–18:30 weekday window, timer not running): a `FIRE`
log with `intensity: "bubble"`. Start the timer and repeat: no fire, and `list_model` shows
`deferred: true`. Skip the phase; the deferred reminder fires immediately.

- [ ] **Step 5: Check acknowledgement moves the body stats**

```js
await invoke("ack_reminder", { id: water.id });
```
Expected: the 专注 tab's 喝水 bar advances from `0 / 8 杯` to `1 / 8 杯`.

- [ ] **Step 6: Run the full gate**

```bash
npm test && npm run check && npm run build && (cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test)
```
Expected: everything passes.

- [ ] **Step 7: Commit**

```bash
git commit --allow-empty -m "test: verify the reminder engine and settings pane end to end"
```

---

## Definition of Done

- All four built-in reminders seed with the exact copy from spec §5.4 in all three tones.
- Interval and daily schedules both fire; daily fires once per day.
- Active window, weekday filter, meeting silence and deep-work demotion all work.
- Focus defers rather than interrupts, and the deferred reminder releases at round end.
- Three consecutive ignores escalate one firing to fullscreen, then reset.
- Changing tone rewrites unedited messages and leaves edited ones alone.
- The 设置 window matches artboard 03 and opens with ⌘,.
- Acknowledging 喝水 / 站立 moves the 专注 tab's body stats.
- The full test gate passes.
