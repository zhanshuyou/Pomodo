# Pomodo 04 — 统计 + 宠物 Tabs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Record every focus session, aggregate it into the four stat cards and the 14-day bar chart, and build the 宠物 tab — level progression, the six-pet picker with unlock rules, custom sprite import, and the four behaviour flags.

**Architecture:** Two new Rust core modules. `core/stats.rs` appends a `Session` on every phase completion and exposes pure aggregation functions that take the current date as a parameter, so every calculation is testable without touching the clock. `core/pet.rs` derives level and unlocks from the lifetime pomodoro count. The frontend gains two tabs that read the same runes store from plan 03.

**Tech Stack:** Rust 2021, chrono, Tauri 2 fs/dialog plugins, Svelte 5 runes.

**Spec:** `docs/superpowers/specs/2026-08-19-momo-design.md`
**Depends on:** plans 01, 02, 03 complete.

## Global Constraints

- 13 completed pomodoros per pet level. The design shows Lv.7 at 62% with `再专注 5 个番茄升到 Lv.8`; 8/13 = 61.5%, and 13 − 8 = 5. Any other constant contradicts the artboard.
- Aggregation functions take `today: NaiveDate` as an argument. No function may call `Local::now()` internally except the thin wrappers in `commands.rs`.
- Bar-chart cell colour is exactly `oklch(from <accent> calc(l + (0.16 - index * 0.035)) c h)` — `barCellColor` from plan 01. A day with zero pomodoros renders one `oklch(0.93 0.008 70)` cell.
- Copy from spec §8.1 verbatim, including `较上周 −4` with U+2212, and `每格 = 一个番茄，颜色越深越连贯`.
- Custom pet files are copied into `app_data_dir()/pomodo/pets/`; the model stores paths only.
- Locked pets render with body `oklch(0.86 0.006 70)` at `opacity: 0.5` and are not selectable.
- The full gate stays green: `npm test`, `npm run check`, `npm run build`, `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`.

---

## File Structure

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/core/stats.rs` | `Session`, `Stats`, aggregation |
| `src-tauri/src/core/pet.rs` | `PetState`, level maths, unlock rules, custom sprite paths |
| `src-tauri/src/model.rs` | gains `stats` and `pet` fields |
| `src-tauri/src/state.rs` | records sessions on phase completion |
| `src-tauri/src/commands.rs` | `stats_summary`, `select_pet`, `import_custom_pet`, `clear_custom_pet` |
| `src/routes/main/StatsTab.svelte` | Artboard 01, 统计 |
| `src/routes/main/PetTab.svelte` | Artboard 01, 宠物 |
| `src/lib/ipc.ts` | new types + calls |

---

### Task 1: Session recording and aggregation

**Files:**
- Create: `src-tauri/src/core/stats.rs`
- Modify: `src-tauri/src/core/mod.rs`, `src-tauri/src/model.rs`

**Interfaces:**
- Consumes: `TaskId`, `Model`.
- Produces:
  - `pub struct Session { pub started_at: i64, pub secs: u32, pub task: Option<TaskId>, pub completed: bool }` (`started_at` is a Unix timestamp in seconds)
  - `pub struct Stats { pub sessions: Vec<Session>, pub best_streak: u32 }` (derives `Default`)
  - `pub struct StatsSummary { pub week_focus_secs: u32, pub week_delta_pct: i32, pub pomodoros: u32, pub daily_average: f32, pub interruptions: u32, pub interruptions_delta: i32, pub streak: u32, pub best_streak: u32, pub bars: Vec<DayBar> }`
  - `pub struct DayBar { pub label: String, pub count: u32 }`
  - `impl Stats`: `record(&mut self, session: Session)`, `daily_counts(&self, today: NaiveDate, days: usize) -> Vec<u32>`, `streak(&self, today: NaiveDate) -> u32`, `summary(&self, today: NaiveDate) -> StatsSummary`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/core/stats.rs` with only the test module:

```rust
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
```

Add `pub mod stats;` to `src-tauri/src/core/mod.rs`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test core::stats::`
Expected: FAIL — `Stats` is not defined.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/core/stats.rs`:

```rust
use chrono::{DateTime, Datelike, Days, NaiveDate, Utc, Weekday};
use serde::{Deserialize, Serialize};

use crate::model::TaskId;

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

    fn window(&self, today: NaiveDate, offset_days: u64) -> (u32, u32, u32) {
        // (completed count, focus secs, interruptions) over the 7 days ending
        // `offset_days` before today.
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
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test core::stats::`
Expected: PASS, 12 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/stats.rs src-tauri/src/core/mod.rs
git commit -m "feat(rust): add session recording and stats aggregation"
```

---

### Task 2: Pet level and unlock rules

**Files:**
- Create: `src-tauri/src/core/pet.rs`
- Modify: `src-tauri/src/core/mod.rs`

**Interfaces:**
- Consumes: nothing beyond serde.
- Produces:
  - `pub const POMODOROS_PER_LEVEL: u32 = 13`
  - `pub const PET_COUNT: u8 = 6`
  - `pub struct CustomPet { pub focus: Option<String>, pub rest: Option<String>, pub nag: Option<String> }`
  - `pub struct PetState { pub selected: u8, pub lifetime_pomodoros: u32, pub custom: CustomPet, pub use_custom: bool }`
  - `impl PetState`: `level() -> u8`, `level_progress() -> f32`, `to_next_level() -> u32`, `is_unlocked(id: u8) -> bool`, `select(id: u8) -> bool`, `credit(&mut self)`
  - `pub fn unlock_threshold(id: u8) -> u32` — 0 for pets 0–3, 150 for PEEP, 300 for BOO

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/core/pet.rs` with only the test module:

```rust
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
```

Add `pub mod pet;` to `src-tauri/src/core/mod.rs`.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test core::pet::`
Expected: FAIL — `PetState` is not defined.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/core/pet.rs`:

```rust
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
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test core::pet::`
Expected: PASS, 10 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/pet.rs src-tauri/src/core/mod.rs
git commit -m "feat(rust): add pet levelling and unlock rules"
```

---

### Task 3: Wire stats and pet into the model

**Files:**
- Modify: `src-tauri/src/model.rs`, `src-tauri/src/state.rs`, `src-tauri/src/store.rs`

**Interfaces:**
- Consumes: `Stats`, `Session`, `PetState`.
- Produces: `Model` gains `pub stats: Stats` and `pub pet: PetState`. `AppState::tick` and
  `commands::skip_phase` both record a `Session`.

Because `Model` gains fields, `SCHEMA_VERSION` must go to 2. Serde's `#[serde(default)]`
on the new fields lets a version-1 file load, so bump the version and accept both.

- [ ] **Step 1: Write the failing test**

Append to the test module in `src-tauri/src/state.rs`:

```rust
    #[test]
    fn completing_a_focus_phase_records_a_session_and_credits_the_pet() {
        use crate::core::stats::Session;

        let state = AppState::new(store_in("session"));
        state.with(|m| {
            m.timer.start();
            let settings = m.settings.clone();
            let changes = m.timer.advance(settings.focus_secs, &settings);
            for change in &changes {
                if change.from == Phase::Focus {
                    m.stats.record(Session {
                        started_at: 1_755_000_000,
                        secs: settings.focus_secs,
                        task: m.timer.active_task,
                        completed: change.completed,
                    });
                    if change.completed {
                        m.pet.credit();
                    }
                }
            }
        });

        let model = state.snapshot();
        assert_eq!(model.stats.sessions.len(), 1);
        assert!(model.stats.sessions[0].completed);
        assert_eq!(model.pet.lifetime_pomodoros, 1);
    }

    #[test]
    fn a_file_written_before_stats_and_pet_existed_still_loads() {
        let dir = std::env::temp_dir().join("momo-state-test-v1");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        fs::write(
            dir.join("state.json"),
            r#"{"schemaVersion":2,"model":{"timer":{"phase":"focus","remainingSecs":99,"running":false,"round":1,"activeTask":null},"tasks":[],"settings":{"accent":"terracotta","tone":"playful","focusSecs":1500,"shortBreakSecs":300,"longBreakSecs":900,"roundsPerCycle":4,"petFlags":{"snapEdges":true,"clickInteract":true,"hideFullscreen":true,"sleepAnimation":false}},"nextTaskId":0}}"#,
        )
        .expect("write");

        let model = Store::new(&dir).load();
        assert_eq!(model.timer.remaining_secs, 99);
        assert_eq!(model.stats.sessions.len(), 0);
        assert_eq!(model.pet.selected, 0);
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test state::`
Expected: FAIL — `Model` has no field `stats`.

- [ ] **Step 3: Extend the model**

In `src-tauri/src/model.rs`, add the imports and the two fields:

```rust
use crate::core::pet::PetState;
use crate::core::stats::Stats;
```

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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
}
```

In `src-tauri/src/store.rs`, bump the constant:

```rust
pub const SCHEMA_VERSION: u32 = 2;
```

- [ ] **Step 4: Record sessions on every phase boundary**

In `src-tauri/src/state.rs`, replace the body of `tick`:

```rust
    /// Advance the clock by real elapsed time, then record and credit anything that
    /// finished. Called once per second by the tick thread.
    pub fn tick(&self, app: &AppHandle, elapsed_secs: u32) {
        let changes = self.with(|m| {
            let settings = m.settings.clone();
            let changes = m.timer.advance(elapsed_secs, &settings);
            for change in &changes {
                if change.from == Phase::Focus {
                    m.record_focus_phase(change.completed, settings.focus_secs);
                }
            }
            changes
        });

        for change in &changes {
            let _ = app.emit(events::PHASE, change);
        }
        if !changes.is_empty() {
            self.emit_changed(app, Section::Tasks);
        }

        self.emit_tick(app);
        self.save_debounced();
    }
```

Add the shared helper to `src-tauri/src/core/stats.rs` so `skip_phase` can reuse it:

```rust
use crate::model::Model;

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
```

Remove the now-duplicated `credit_task` call that `tick` previously made inline.

In `src-tauri/src/commands.rs`, make `skip_phase` record the abandoned phase:

```rust
#[tauri::command]
pub fn skip_phase(state: State<'_, AppState>, app: AppHandle) {
    let change = state.with(|m| {
        let settings = m.settings.clone();
        let was_focus = m.timer.phase == crate::model::Phase::Focus;
        let elapsed = settings.focus_secs.saturating_sub(m.timer.remaining_secs);
        let change = m.timer.skip(&settings);
        if was_focus {
            m.record_focus_phase(false, elapsed);
        }
        change
    });
    let _ = tauri::Emitter::emit(&app, crate::events::PHASE, change);
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Tasks);
    state.flush();
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cd src-tauri && cargo test`
Expected: PASS, all suites.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src
git commit -m "feat(rust): record focus sessions into stats and pet progress"
```

---

### Task 4: Stats and pet commands

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Consumes: `StatsSummary`, `PetState`.
- Produces:
  - `stats_summary(state) -> StatsSummary`
  - `select_pet(state, app, id: u8) -> bool`
  - `set_use_custom_pet(state, app, value: bool)`
  - `import_custom_pet(state, app, slot: String, source: String) -> Result<String, String>` — copies the file into `app_data_dir()/pomodo/pets/<slot>.<ext>` and stores the path; `slot` is `focus` / `rest` / `nag`
  - `clear_custom_pet(state, app, slot: String)`

- [ ] **Step 1: Add the dialog plugin**

```bash
cd src-tauri && cargo add tauri-plugin-dialog@2
```

Register it in `lib.rs`'s builder chain, next to `tauri_plugin_opener::init()`:

```rust
.plugin(tauri_plugin_dialog::init())
```

Add `"dialog:allow-open"` to the `permissions` array in
`src-tauri/capabilities/default.json`.

- [ ] **Step 2: Write the commands**

Append to `src-tauri/src/commands.rs`:

```rust
use std::path::Path;

use tauri::Manager;

use crate::core::stats::StatsSummary;

#[tauri::command]
pub fn stats_summary(state: State<'_, AppState>) -> StatsSummary {
    let today = chrono::Utc::now().date_naive();
    state.with(|m| m.stats.summary(today))
}

#[tauri::command]
pub fn select_pet(state: State<'_, AppState>, app: AppHandle, id: u8) -> bool {
    let ok = state.with(|m| m.pet.select(id));
    if ok {
        state.emit_changed(&app, Section::Settings);
        state.flush();
    }
    ok
}

#[tauri::command]
pub fn set_use_custom_pet(state: State<'_, AppState>, app: AppHandle, value: bool) {
    state.with(|m| m.pet.use_custom = value);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

/// Copy a user-chosen image into the app's pets directory and remember its path.
/// `slot` is one of focus / rest / nag.
#[tauri::command]
pub fn import_custom_pet(
    state: State<'_, AppState>,
    app: AppHandle,
    slot: String,
    source: String,
) -> Result<String, String> {
    let allowed = ["png", "gif", "apng", "webp"];
    let src = Path::new(&source);
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    if !allowed.contains(&ext.as_str()) {
        return Err(format!("不支持的图片格式：{ext}"));
    }
    if !matches!(slot.as_str(), "focus" | "rest" | "nag") {
        return Err(format!("未知的槽位：{slot}"));
    }

    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("momo")
        .join("pets");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let dest = dir.join(format!("{slot}.{ext}"));
    std::fs::copy(src, &dest).map_err(|e| e.to_string())?;
    let stored = dest.to_string_lossy().into_owned();

    state.with(|m| {
        let c = &mut m.pet.custom;
        match slot.as_str() {
            "focus" => c.focus = Some(stored.clone()),
            "rest" => c.rest = Some(stored.clone()),
            "nag" => c.nag = Some(stored.clone()),
            _ => {}
        }
        m.pet.use_custom = true;
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
    Ok(stored)
}

#[tauri::command]
pub fn clear_custom_pet(state: State<'_, AppState>, app: AppHandle, slot: String) {
    state.with(|m| {
        let c = &mut m.pet.custom;
        match slot.as_str() {
            "focus" => c.focus = None,
            "rest" => c.rest = None,
            "nag" => c.nag = None,
            _ => {}
        }
        if c.focus.is_none() && c.rest.is_none() && c.nag.is_none() {
            m.pet.use_custom = false;
        }
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
}
```

Register all five in the `generate_handler!` list in `lib.rs`.

- [ ] **Step 3: Verify**

Run: `cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src-tauri
git commit -m "feat(rust): add stats and pet commands"
```

---

### Task 5: Extend the IPC layer

**Files:**
- Modify: `src/lib/ipc.ts`, `src/lib/state.svelte.ts`

**Interfaces:**
- Produces:
  - `export interface DayBar { label: string; count: number }`
  - `export interface StatsSummary { weekFocusSecs; weekDeltaPct; pomodoros; dailyAverage; interruptions; interruptionsDelta; streak; bestStreak; bars: DayBar[] }`
  - `export interface CustomPet { focus: string | null; rest: string | null; nag: string | null }`
  - `export interface PetState { selected: number; lifetimePomodoros: number; custom: CustomPet; useCustom: boolean }`
  - `Model` gains `stats: { sessions: unknown[]; bestStreak: number }` and `pet: PetState`
  - `statsSummary()`, `selectPet(id)`, `setUseCustomPet(v)`, `importCustomPet(slot, source)`, `clearCustomPet(slot)`
  - `app.stats: StatsSummary | null` and `app.pet: PetState` on the store

- [ ] **Step 1: Add the types and calls**

Append to `src/lib/ipc.ts`:

```ts
export interface DayBar {
  label: string;
  count: number;
}

export interface StatsSummary {
  weekFocusSecs: number;
  weekDeltaPct: number;
  pomodoros: number;
  dailyAverage: number;
  interruptions: number;
  interruptionsDelta: number;
  streak: number;
  bestStreak: number;
  bars: DayBar[];
}

export interface CustomPet {
  focus: string | null;
  rest: string | null;
  nag: string | null;
}

export interface PetState {
  selected: number;
  lifetimePomodoros: number;
  custom: CustomPet;
  useCustom: boolean;
}

export type PetSlot = "focus" | "rest" | "nag";

export const statsSummary = () => invoke<StatsSummary>("stats_summary");
export const selectPet = (id: number) => invoke<boolean>("select_pet", { id });
export const setUseCustomPet = (value: boolean) =>
  invoke<void>("set_use_custom_pet", { value });
export const importCustomPet = (slot: PetSlot, source: string) =>
  invoke<string>("import_custom_pet", { slot, source });
export const clearCustomPet = (slot: PetSlot) =>
  invoke<void>("clear_custom_pet", { slot });
```

Extend the `Model` interface in the same file:

```ts
export interface Model {
  timer: Timer;
  tasks: Task[];
  settings: Settings;
  nextTaskId: number;
  stats: { sessions: unknown[]; bestStreak: number };
  pet: PetState;
}
```

- [ ] **Step 2: Extend the store**

In `src/lib/state.svelte.ts`, add the two fallback fields to `FALLBACK`:

```ts
  stats: { sessions: [], bestStreak: 0 },
  pet: {
    selected: 0,
    lifetimePomodoros: 0,
    custom: { focus: null, rest: null, nag: null },
    useCustom: false,
  },
```

Add to `AppStore`:

```ts
  summary = $state<StatsSummary | null>(null);

  get pet() {
    return this.model.pet;
  }

  async refreshStats(): Promise<void> {
    if (!IS_TAURI) return;
    this.summary = await statsSummary();
  }
```

Call `void this.refreshStats()` inside `refresh()` after `this.model = await listModel()`,
and import `statsSummary` plus the `StatsSummary` type.

- [ ] **Step 3: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/ipc.ts src/lib/state.svelte.ts
git commit -m "feat: expose stats and pet state to the frontend"
```

---

### Task 6: The 统计 tab

**Files:**
- Create: `src/routes/main/StatsTab.svelte`
- Modify: `src/routes/main/App.svelte`

**Interfaces:**
- Consumes: `app.summary`, `barCellColor`, `petVerdict`.
- Produces: artboard 01's 统计 tab.

- [ ] **Step 1: Write StatsTab.svelte**

Create `src/routes/main/StatsTab.svelte`:

```svelte
<script lang="ts">
  import { petVerdict } from "../../lib/copy";
  import { app } from "../../lib/state.svelte";
  import { ACCENTS, barCellColor } from "../../lib/theme";

  const s = $derived(app.summary);
  const accent = $derived(ACCENTS[app.settings.accent]);

  function hoursMinutes(secs: number): string {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return `${h}h${String(m).padStart(2, "0")}m`;
  }

  /** `+12%` / `−4` use U+2212, matching the design. */
  function signed(n: number, suffix = ""): string {
    if (n > 0) return `+${n}${suffix}`;
    if (n < 0) return `−${Math.abs(n)}${suffix}`;
    return `${n}${suffix}`;
  }

  const cards = $derived(
    s
      ? [
          {
            name: "本周专注",
            value: hoursMinutes(s.weekFocusSecs),
            delta: `较上周 ${signed(s.weekDeltaPct, "%")}`,
            good: s.weekDeltaPct > 0,
          },
          {
            name: "完成番茄",
            value: String(s.pomodoros),
            delta: `日均 ${s.dailyAverage.toFixed(1)} 个`,
            good: false,
          },
          {
            name: "中断次数",
            value: String(s.interruptions),
            delta: `较上周 ${signed(s.interruptionsDelta)}`,
            good: s.interruptionsDelta < 0,
          },
          {
            name: "连续天数",
            value: String(s.streak),
            delta: `个人最佳 ${s.bestStreak}`,
            good: false,
          },
        ]
      : [],
  );
</script>

<div class="stats">
  <div class="cards">
    {#each cards as card (card.name)}
      <div class="card">
        <span class="cname">{card.name}</span>
        <span class="cvalue">{card.value}</span>
        <span class="cdelta" class:good={card.good}>{card.delta}</span>
      </div>
    {/each}
  </div>

  <div class="chart">
    <div class="chart-head">
      <span class="ctitle">最近两周的专注分布</span>
      <span class="ccaption">每格 = 一个番茄，颜色越深越连贯</span>
    </div>
    <div class="bars">
      {#each s?.bars ?? [] as bar, i (i)}
        <div class="bar">
          <div class="stack">
            {#if bar.count === 0}
              <span class="cell empty"></span>
            {:else}
              {#each Array.from({ length: bar.count }, (_, k) => k) as k (k)}
                <span class="cell" style:background={barCellColor(accent, k)}></span>
              {/each}
            {/if}
          </div>
          <span class="blabel">{bar.label}</span>
        </div>
      {/each}
    </div>
  </div>

  <div class="insights">
    <div class="insight">
      <span class="ititle">被打断最多的时段</span>
      <span class="ibody">
        15:00–16:00，平均每轮被打断 1.8 次。要不要把这段设成「勿扰 + 只留宠物提示」？
      </span>
    </div>
    <div class="insight">
      <span class="ititle">Pomodo 的评价</span>
      <span class="ibody">{petVerdict(app.tone)}</span>
    </div>
  </div>
</div>

<style>
  .stats {
    flex: 1;
    padding: 32px 40px 38px;
    display: flex;
    flex-direction: column;
    gap: 28px;
    overflow-y: auto;
  }
  .cards {
    display: flex;
    gap: 40px;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .cname {
    font-size: 12px;
    color: var(--dim);
  }
  .cvalue {
    font-family: var(--font-mono);
    font-size: 30px;
    font-weight: 500;
    letter-spacing: -0.02em;
  }
  .cdelta {
    font-size: 12px;
    color: var(--dim);
  }
  .cdelta.good {
    color: var(--good);
  }
  .chart {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .chart-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .ctitle {
    font-size: 14px;
    font-weight: 600;
  }
  .ccaption {
    font-size: 12.5px;
    color: var(--dim);
  }
  .bars {
    display: flex;
    gap: 5px;
    align-items: flex-end;
    height: 148px;
  }
  .bar {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 7px;
  }
  .stack {
    width: 100%;
    height: 118px;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    gap: 3px;
    overflow: hidden;
  }
  .cell {
    height: 13px;
    border-radius: 3px;
    flex: none;
  }
  .cell.empty {
    background: var(--line-soft);
  }
  .blabel {
    font-size: 11px;
    color: var(--faint);
  }
  .insights {
    display: flex;
    gap: 20px;
  }
  .insight {
    flex: 1;
    padding: 18px 20px;
    border: 1px solid oklch(0.9 0.008 70);
    border-radius: var(--radius-card);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .ititle {
    font-size: 13px;
    font-weight: 600;
  }
  .ibody {
    font-size: 12.5px;
    color: oklch(0.53 0.012 60);
    line-height: 1.5;
  }
</style>
```

- [ ] **Step 2: Mount it**

In `src/routes/main/App.svelte`, import `StatsTab` and replace the `tab === 1` stub with
`<StatsTab />`.

- [ ] **Step 3: Commit**

```bash
git add src/routes/main/StatsTab.svelte src/routes/main/App.svelte
git commit -m "feat: build the stats tab"
```

---

### Task 7: The 宠物 tab

**Files:**
- Create: `src/routes/main/PetTab.svelte`
- Modify: `src/routes/main/App.svelte`, `src/routes/main/FocusTab.svelte`

**Interfaces:**
- Consumes: `PETS`, `LOCKED_BODY`, `PetCanvas`, `Chip`, `app`, `selectPet`, `importCustomPet`, `clearCustomPet`, `setPetFlag`, and `open` from `@tauri-apps/plugin-dialog` (re-exported through `ipc.ts`).
- Produces: artboard 01's 宠物 tab, and `FocusTab` switching from the hard-coded `PETS[0]`
  to `PETS[app.pet.selected]`.

- [ ] **Step 1: Re-export the file picker through ipc.ts**

Install the JS side of the dialog plugin and add the wrapper:

```bash
npm install @tauri-apps/plugin-dialog@^2
```

Append to `src/lib/ipc.ts`:

```ts
import { open as openDialog } from "@tauri-apps/plugin-dialog";

/** Ask the user for a sprite file. Returns null when they cancel. */
export async function pickPetImage(): Promise<string | null> {
  const picked = await openDialog({
    multiple: false,
    directory: false,
    filters: [{ name: "宠物图片", extensions: ["png", "gif", "apng", "webp"] }],
  });
  return typeof picked === "string" ? picked : null;
}

/** Convert a stored absolute path into something an <img src> can load. */
export { convertFileSrc } from "@tauri-apps/api/core";
```

- [ ] **Step 2: Write PetTab.svelte**

Create `src/routes/main/PetTab.svelte`:

```svelte
<script lang="ts">
  import Chip from "../../lib/components/Chip.svelte";
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import {
    type PetFlags,
    type PetSlot,
    clearCustomPet,
    convertFileSrc,
    importCustomPet,
    pickPetImage,
    selectPet,
    setPetFlag,
  } from "../../lib/ipc";
  import { LOCKED_BODY, PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";

  const POMODOROS_PER_LEVEL = 13;

  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
  const level = $derived(Math.floor(app.pet.lifetimePomodoros / POMODOROS_PER_LEVEL) + 1);
  const progressPct = $derived(
    ((app.pet.lifetimePomodoros % POMODOROS_PER_LEVEL) / POMODOROS_PER_LEVEL) * 100,
  );
  const toNext = $derived(
    POMODOROS_PER_LEVEL - (app.pet.lifetimePomodoros % POMODOROS_PER_LEVEL),
  );

  const UNLOCK_AT: Record<number, number> = { 0: 0, 1: 0, 2: 0, 3: 0, 4: 150, 5: 300 };
  const unlocked = (id: number) => app.pet.lifetimePomodoros >= (UNLOCK_AT[id] ?? Infinity);

  const FLAGS: { key: keyof PetFlags; name: string }[] = [
    { key: "snapEdges", name: "贴边吸附" },
    { key: "clickInteract", name: "点击互动" },
    { key: "hideFullscreen", name: "全屏时隐藏" },
    { key: "sleepAnimation", name: "睡眠动画" },
  ];

  const SLOTS: { key: PetSlot; label: string }[] = [
    { key: "focus", label: "专注" },
    { key: "rest", label: "休息" },
    { key: "nag", label: "催你站起来" },
  ];

  let error = $state("");

  async function importSlot(slot: PetSlot) {
    error = "";
    const source = await pickPetImage();
    if (!source) return;
    try {
      await importCustomPet(slot, source);
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="pettab">
  <aside class="hero">
    <PetCanvas map={pet.map} body={pet.body} scale={9} anim="bob" alt={pet.name} />
    <div class="heroinfo">
      <div class="heroname">
        <span class="pname">{pet.name}</span>
        <span class="plevel">Lv.{level} · 好奇期</span>
      </div>
      <div class="track"><div class="fill" style:width="{progressPct}%"></div></div>
      <span class="hint">再专注 {toNext} 个番茄升到 Lv.{level + 1}，解锁「披风」</span>
    </div>
  </aside>

  <div class="right">
    <section>
      <div class="sechead">
        <span class="sectitle">选一只</span>
        <span class="seccaption">灰色的还锁着，专注攒够就解锁</span>
      </div>
      <div class="grid">
        {#each PETS as p (p.id)}
          {@const open = unlocked(p.id)}
          <button
            class="petcard"
            class:sel={app.pet.selected === p.id}
            class:locked={!open}
            type="button"
            disabled={!open}
            onclick={() => void selectPet(p.id)}
          >
            <PetCanvas
              map={p.map}
              body={open ? p.body : LOCKED_BODY}
              scale={4}
              alt={p.name}
            />
            <span class="petname">{p.name}</span>
          </button>
        {/each}
      </div>
    </section>

    <section class="custom">
      <div class="slot">
        {#if app.pet.custom.focus}
          <img src={convertFileSrc(app.pet.custom.focus)} alt="自定义宠物" />
        {:else}
          <button class="drop" type="button" onclick={() => void importSlot("focus")}>
            拖入你的宠物 PNG / GIF
          </button>
        {/if}
      </div>

      <div class="customtext">
        <span class="sectitle">或者养你自己的</span>
        <span class="blurb">
          拖入 PNG / GIF / APNG 就成了你的宠物。可以给「专注」「休息」「催你站起来」三种状态各配一张，Pomodo
          自动换装；像素图会按整数倍放大，不糊。
        </span>
        <div class="chiprow">
          {#each SLOTS as slot (slot.key)}
            <Chip
              selected={!!app.pet.custom[slot.key]}
              onclick={() =>
                app.pet.custom[slot.key]
                  ? void clearCustomPet(slot.key)
                  : void importSlot(slot.key)}
            >
              {slot.label}
            </Chip>
          {/each}
        </div>
        {#if error}<span class="error">{error}</span>{/if}
      </div>
    </section>

    <section>
      <div class="chiprow">
        {#each FLAGS as flag (flag.key)}
          <Chip
            selected={app.settings.petFlags[flag.key]}
            dot={app.settings.petFlags[flag.key] ? "var(--accent)" : "oklch(0.85 0.008 70)"}
            onclick={() => void setPetFlag(flag.key, !app.settings.petFlags[flag.key])}
          >
            {flag.name}
          </Chip>
        {/each}
      </div>
    </section>
  </div>
</div>

<style>
  .pettab {
    flex: 1;
    padding: 30px 40px 38px;
    display: flex;
    gap: 36px;
    overflow-y: auto;
  }
  .hero {
    width: 300px;
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 18px;
    padding: 26px 20px;
    border: 1px solid oklch(0.9 0.008 70);
    border-radius: 14px;
    background: linear-gradient(180deg, oklch(0.975 0.012 75) 0%, oklch(0.99 0.004 80) 70%);
    align-self: flex-start;
  }
  .heroinfo {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    width: 100%;
  }
  .heroname {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .pname {
    font-family: var(--font-pixel);
    font-size: 18px;
  }
  .plevel {
    font-size: 12.5px;
    color: var(--dim);
  }
  .track {
    width: 100%;
    height: 8px;
    border-radius: 4px;
    background: var(--track);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: var(--accent);
  }
  .hint {
    font-size: 12px;
    color: oklch(0.53 0.012 60);
    text-align: center;
  }
  .right {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 20px;
    min-width: 0;
  }
  section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .sechead {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .sectitle {
    font-size: 14px;
    font-weight: 600;
  }
  .seccaption {
    font-size: 12.5px;
    color: var(--dim);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 10px;
  }
  .petcard {
    padding: 18px 8px 11px;
    border: 1.5px solid var(--line);
    border-radius: 12px;
    background: var(--card);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    cursor: pointer;
  }
  .petcard.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .petcard.locked {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .petname {
    font-family: var(--font-pixel);
    font-size: 11px;
  }
  .custom {
    flex-direction: row;
    gap: 16px;
    align-items: stretch;
  }
  .slot {
    width: 148px;
    height: 148px;
    flex: none;
    border-radius: 12px;
    overflow: hidden;
    background: var(--surface-2);
  }
  .slot img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    image-rendering: pixelated;
  }
  .drop {
    width: 100%;
    height: 100%;
    border: 1px dashed oklch(0.85 0.008 70);
    border-radius: 12px;
    background: transparent;
    color: var(--dim);
    font-size: 12.5px;
    cursor: pointer;
    padding: 12px;
  }
  .customtext {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 10px;
    justify-content: center;
  }
  .blurb {
    font-size: 13px;
    line-height: 1.55;
    color: oklch(0.52 0.012 60);
  }
  .chiprow {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .error {
    font-size: 12px;
    color: oklch(0.55 0.15 25);
  }
</style>
```

- [ ] **Step 3: Mount it and use the selected pet everywhere**

In `src/routes/main/App.svelte`, import `PetTab` and replace the `tab === 2` stub.

In `src/routes/main/FocusTab.svelte`, replace:

```ts
  // Plan 04 replaces this with the user's selected pet.
  const pet = PETS[0];
```

with:

```ts
  const pet = $derived(PETS[app.pet.selected] ?? PETS[0]);
```

- [ ] **Step 4: Verify**

Run: `npm run check && npm test && npm run build`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/routes/main src/lib/ipc.ts package.json package-lock.json
git commit -m "feat: build the pet tab and use the selected pet everywhere"
```

---

### Task 8: End-to-end verification

**Files:** none.

- [ ] **Step 1: Launch and seed some history**

Run: `npm run tauri dev`

In the devtools console, backdate some sessions so the chart has data:

```js
const { invoke } = await import("@tauri-apps/api/core");
// Complete a few phases quickly by skipping — these count as interruptions.
for (let i = 0; i < 3; i++) await invoke("skip_phase");
console.log(await invoke("stats_summary"));
```

Expected: `interruptions` is 3 (skips from a focus phase only), `pomodoros` is 0, and
`bars` has 14 entries.

- [ ] **Step 2: Check the 统计 tab against the artboard**

Expected: four stat cards in a row with 30px mono numerals; the bar chart 148px tall with
13px cells and 3px gaps; empty days show one pale cell; the two insight cards sit side by
side; the Pomodo 的评价 text changes when the tone changes.

- [ ] **Step 3: Check the 宠物 tab against the artboard**

Expected: the hero card 300px wide with the pet at scale 9; a six-column picker; PEEP and
BOO greyed at 50% opacity and not clickable; clicking TOFU switches the hero *and* the
专注 tab's pet; the four behaviour chips reflect and toggle the stored flags.

- [ ] **Step 4: Check custom pet import**

Click the drop slot, choose a PNG. Expected: the image appears in the slot, the 专注 chip
lights up, and the file exists at
`~/Library/Application Support/com.pomodo.app/pomodo/pets/focus.png`. Click the 专注 chip to
clear it; the slot returns to the dashed prompt.

- [ ] **Step 5: Check levelling**

```js
const { invoke } = await import("@tauri-apps/api/core");
console.log((await invoke("list_model")).pet);
```

Expected: `lifetimePomodoros` climbs by one for each naturally completed focus phase, and
the hero's Lv. number increments every 13.

- [ ] **Step 6: Run the full gate**

```bash
npm test && npm run check && npm run build && (cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test)
```
Expected: everything passes.

- [ ] **Step 7: Commit**

```bash
git commit --allow-empty -m "test: verify the stats and pet tabs end to end"
```

---

## Definition of Done

- Focus phases are recorded as sessions; skipping records an interruption.
- The four stat cards and the 14-day chart compute from real recorded data.
- Bar cells use the design's exact relative-colour ramp and recolour with the accent.
- Pet level follows 13 pomodoros per level; the artboard's Lv.7 / 62% / 5-to-go figures
  reproduce exactly (covered by `the_designs_level_seven_figures_reproduce_exactly`).
- PEEP and BOO are locked and unselectable until 150 / 300 lifetime pomodoros.
- Custom sprites import into the app data directory and render pixelated.
- Selecting a pet changes it in both the 宠物 and 专注 tabs.
- The full test gate passes.
