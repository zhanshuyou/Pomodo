# Momo 02 — Rust Timer Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust model that owns Momo's timer, tasks and settings — a tested state machine, a debounced JSON store, an actor guarded by a mutex, a monotonic tick thread, and the command and event surface every webview will use.

**Architecture:** `Model` is a plain serialisable struct with no I/O. `core/timer.rs` advances it by an elapsed-seconds delta so sleep/wake and clock changes are just a large delta, which makes every transition unit-testable without sleeping in tests. `AppState` wraps `Mutex<Model>` and an `AppHandle`, emitting typed events on change. A background thread computes elapsed time from `std::time::Instant` and calls `advance` once per second.

**Tech Stack:** Rust 2021, Tauri 2, serde / serde_json, chrono.

**Spec:** `docs/superpowers/specs/2026-08-19-momo-design.md`

## Global Constraints

- Rust owns all state. No timer logic may live in the frontend.
- Elapsed time is derived from `std::time::Instant`, never from counting ticks — a laptop that sleeps for an hour must resume correctly.
- Defaults from spec §6.1: focus 1500s, short break 300s, long break 900s, 4 rounds per cycle, accent `Terracotta`, tone `Playful`.
- Persistence is a single JSON document at `app_data_dir()/momo/state.json`, written atomically (temp file + rename), debounced to at most one write per second, carrying a top-level `schema_version`.
- No panics on the tick thread or in any command — a poisoned mutex must be recovered, not unwrapped.
- `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test` must all pass. CI runs them on Linux, macOS and Windows.
- Nothing in this plan is macOS-specific.

---

## File Structure

| Path | Responsibility |
| --- | --- |
| `src-tauri/src/model.rs` | `Model`, `Settings`, `Task`, `Phase`, `Accent`, `Tone`, `PetFlags` — data only |
| `src-tauri/src/core/mod.rs` | Re-exports the core submodules |
| `src-tauri/src/core/timer.rs` | `Timer` state machine + `PhaseChange` |
| `src-tauri/src/core/task.rs` | Task list mutations |
| `src-tauri/src/store.rs` | Load / atomic save / schema migration |
| `src-tauri/src/events.rs` | Event names and payload structs |
| `src-tauri/src/state.rs` | `AppState` — mutex, emit, debounced save, `tick` entry point |
| `src-tauri/src/commands.rs` | `#[tauri::command]` surface |
| `src-tauri/src/lib.rs` | Builder wiring + tick thread |

---

### Task 1: Model types

**Files:**
- Create: `src-tauri/src/model.rs`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub enum Phase { Focus, ShortBreak, LongBreak }`
  - `pub enum Accent { Terracotta, Blue, Green, Magenta }`
  - `pub enum Tone { Professional, Gentle, Playful }`
  - `pub struct PetFlags { snap_edges, click_interact, hide_fullscreen, sleep_animation: bool }`
  - `pub struct Settings { accent, tone, focus_secs, short_break_secs, long_break_secs, rounds_per_cycle, pet_flags }`
  - `pub type TaskId = u32`
  - `pub struct Task { id: TaskId, name: String, estimate: u8, spent: u8, done: bool }`
  - `pub struct Model { timer: Timer, tasks: Vec<Task>, settings: Settings, next_task_id: TaskId }`
  - `impl Default` for all of them, and `Settings::duration_for(&self, phase: Phase) -> u32`

`Model` gains `reminders`, `pet` and `stats` fields in plans 4 and 5. It is defined here
with only what plan 3 needs so this plan stays independently shippable.

- [ ] **Step 1: Add chrono**

In `src-tauri/Cargo.toml`, under `[dependencies]`:

```toml
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: Write the failing test**

Create `src-tauri/src/model.rs` containing only the test module for now:

```rust
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
        assert_eq!(serde_json::to_string(&Phase::ShortBreak).unwrap(), "\"shortBreak\"");
        assert_eq!(serde_json::to_string(&Accent::Terracotta).unwrap(), "\"terracotta\"");
        assert_eq!(serde_json::to_string(&Tone::Playful).unwrap(), "\"playful\"");
    }
}
```

Declare the module in `src-tauri/src/lib.rs` above the existing code:

```rust
pub mod model;
```

- [ ] **Step 3: Run the test to verify it fails**

Run: `cd src-tauri && cargo test model::`
Expected: FAIL — `Settings`, `Phase`, `Model` etc. are not defined.

- [ ] **Step 4: Write the implementation**

Prepend to `src-tauri/src/model.rs`, above the test module:

```rust
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
```

`Settings` needs `Default` derived manually (done above), so remove `Settings` from the
`#[derive(Default)]` chain if the compiler complains — `Model`'s derived `Default` uses
`Settings::default()` automatically.

- [ ] **Step 5: Run the test to verify it passes**

Run: `cd src-tauri && cargo test model::`
Expected: PASS, 5 tests. (`core::timer::Timer` does not exist yet — Task 2 creates it; if
you are executing tasks strictly in order, write a minimal `Timer` stub now and let Task 2
replace it via its own failing test.)

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/model.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat(rust): add the Momo model types"
```

---

### Task 2: Timer state machine

**Files:**
- Create: `src-tauri/src/core/mod.rs`, `src-tauri/src/core/timer.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `Phase`, `Settings`, `TaskId` from `model.rs`.
- Produces:
  - `pub struct Timer { pub phase: Phase, pub remaining_secs: u32, pub running: bool, pub round: u8, pub active_task: Option<TaskId> }`
  - `pub struct PhaseChange { pub from: Phase, pub to: Phase, pub round: u8, pub completed: bool }`
  - `Timer::new(&Settings) -> Timer`
  - `Timer::advance(&mut self, elapsed_secs: u32, &Settings) -> Vec<PhaseChange>`
  - `Timer::start(&mut self)`, `Timer::pause(&mut self)`
  - `Timer::skip(&mut self, &Settings) -> PhaseChange`
  - `Timer::progress(&self, &Settings) -> f32` — 0.0..=1.0 elapsed fraction
  - `Timer::belly_cells(&self, &Settings) -> u8` — 0..=10, the design's `round(pct / 10)`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/core/timer.rs` with only the test module:

```rust
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
```

Create `src-tauri/src/core/mod.rs`:

```rust
pub mod timer;
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod core;
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test core::timer::`
Expected: FAIL — `Timer` is not defined.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/core/timer.rs`:

```rust
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
/// which is how plan 4's stats layer tells a finished pomodoro from an abandoned one.
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
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test core::timer::`
Expected: PASS, 12 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core
git commit -m "feat(rust): add the timer state machine"
```

---

### Task 3: Task list operations

**Files:**
- Create: `src-tauri/src/core/task.rs`
- Modify: `src-tauri/src/core/mod.rs`

**Interfaces:**
- Consumes: `Model`, `Task`, `TaskId` from `model.rs`.
- Produces, all as `impl Model`:
  - `pub fn add_task(&mut self, name: String, estimate: u8) -> TaskId`
  - `pub fn toggle_task(&mut self, id: TaskId) -> bool` — returns the new `done` value; false if missing
  - `pub fn delete_task(&mut self, id: TaskId)`
  - `pub fn credit_task(&mut self, id: TaskId)` — increments `spent`, saturating at `u8::MAX`
  - `pub fn done_count(&self) -> usize`
  - `pub fn seed_demo_tasks(&mut self)` — the five tasks from spec §8.1

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/core/task.rs` with only the test module:

```rust
#[cfg(test)]
mod tests {
    use crate::model::Model;

    #[test]
    fn add_task_assigns_increasing_ids() {
        let mut m = Model::default();
        let a = m.add_task("写产品需求文档".into(), 3);
        let b = m.add_task("回 Sarah 的邮件".into(), 1);
        assert_ne!(a, b);
        assert_eq!(m.tasks.len(), 2);
        assert_eq!(m.tasks[0].name, "写产品需求文档");
        assert_eq!(m.tasks[0].estimate, 3);
        assert_eq!(m.tasks[0].spent, 0);
        assert!(!m.tasks[0].done);
    }

    #[test]
    fn toggle_task_flips_done_and_reports_the_new_value() {
        let mut m = Model::default();
        let id = m.add_task("整理用研笔记".into(), 2);
        assert!(m.toggle_task(id));
        assert!(m.tasks[0].done);
        assert!(!m.toggle_task(id));
        assert!(!m.tasks[0].done);
    }

    #[test]
    fn toggling_a_missing_task_is_a_no_op() {
        let mut m = Model::default();
        assert!(!m.toggle_task(999));
    }

    #[test]
    fn delete_task_removes_it() {
        let mut m = Model::default();
        let id = m.add_task("改登录页文案".into(), 1);
        m.delete_task(id);
        assert!(m.tasks.is_empty());
    }

    #[test]
    fn credit_task_increments_spent_and_saturates() {
        let mut m = Model::default();
        let id = m.add_task("周会前更新看板".into(), 1);
        m.credit_task(id);
        m.credit_task(id);
        assert_eq!(m.tasks[0].spent, 2);

        m.tasks[0].spent = u8::MAX;
        m.credit_task(id);
        assert_eq!(m.tasks[0].spent, u8::MAX);
    }

    #[test]
    fn done_count_counts_only_finished_tasks() {
        let mut m = Model::default();
        let a = m.add_task("a".into(), 1);
        m.add_task("b".into(), 1);
        m.toggle_task(a);
        assert_eq!(m.done_count(), 1);
    }

    #[test]
    fn seed_demo_tasks_matches_the_design() {
        let mut m = Model::default();
        m.seed_demo_tasks();
        assert_eq!(m.tasks.len(), 5);
        assert_eq!(m.tasks[0].name, "写产品需求文档");
        assert_eq!(m.tasks[0].spent, 3);
        assert_eq!(m.done_count(), 2);
        assert!(m.tasks[3].done);
        assert!(m.tasks[4].done);
    }

    #[test]
    fn seeding_twice_does_not_duplicate() {
        let mut m = Model::default();
        m.seed_demo_tasks();
        m.seed_demo_tasks();
        assert_eq!(m.tasks.len(), 5);
    }
}
```

Add to `src-tauri/src/core/mod.rs`:

```rust
pub mod task;
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test core::task::`
Expected: FAIL — `add_task` is not a method on `Model`.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/core/task.rs`:

```rust
use crate::model::{Model, Task, TaskId};

impl Model {
    pub fn add_task(&mut self, name: String, estimate: u8) -> TaskId {
        let id = self.next_task_id;
        self.next_task_id = self.next_task_id.saturating_add(1);
        self.tasks.push(Task {
            id,
            name,
            estimate,
            spent: 0,
            done: false,
        });
        id
    }

    /// Flip a task's done flag. Returns the new value, or false if the id is unknown.
    pub fn toggle_task(&mut self, id: TaskId) -> bool {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => {
                task.done = !task.done;
                task.done
            }
            None => false,
        }
    }

    pub fn delete_task(&mut self, id: TaskId) {
        self.tasks.retain(|t| t.id != id);
        if self.timer.active_task == Some(id) {
            self.timer.active_task = None;
        }
    }

    /// Record one completed pomodoro against a task.
    pub fn credit_task(&mut self, id: TaskId) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.spent = task.spent.saturating_add(1);
        }
    }

    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }

    /// First-run content, copied from the design's task list.
    pub fn seed_demo_tasks(&mut self) {
        if !self.tasks.is_empty() {
            return;
        }
        let seeds: [(&str, u8, u8, bool); 5] = [
            ("写产品需求文档", 3, 3, false),
            ("回 Sarah 的邮件", 1, 0, false),
            ("整理用研笔记", 2, 0, false),
            ("改登录页文案", 1, 1, true),
            ("周会前更新看板", 1, 1, true),
        ];
        for (name, estimate, spent, done) in seeds {
            let id = self.add_task(name.to_string(), estimate);
            if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
                task.spent = spent;
                task.done = done;
            }
        }
        self.timer.active_task = self.tasks.first().map(|t| t.id);
    }
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test core::task::`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/core/task.rs src-tauri/src/core/mod.rs
git commit -m "feat(rust): add task list operations"
```

---

### Task 4: JSON store

**Files:**
- Create: `src-tauri/src/store.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `Model`.
- Produces:
  - `pub const SCHEMA_VERSION: u32 = 1`
  - `pub struct Store { path: PathBuf }`
  - `Store::new(dir: &Path) -> Store` — the file lands at `dir/state.json`
  - `Store::load(&self) -> Model` — returns `Model::default()` seeded with demo tasks when the file is missing or unreadable, and backs up an unknown-version file to `state.json.bak` before doing so
  - `Store::save(&self, model: &Model) -> std::io::Result<()>` — atomic temp-file + rename

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/store.rs` with only the test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("momo-store-test-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn load_seeds_a_fresh_model_when_no_file_exists() {
        let store = Store::new(&temp_dir("fresh"));
        let model = store.load();
        assert_eq!(model.tasks.len(), 5);
        assert_eq!(model.settings.focus_secs, 1500);
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = temp_dir("roundtrip");
        let store = Store::new(&dir);
        let mut model = store.load();
        model.timer.remaining_secs = 42;
        model.add_task("新任务".into(), 2);
        store.save(&model).expect("save");

        let back = Store::new(&dir).load();
        assert_eq!(back.timer.remaining_secs, 42);
        assert_eq!(back.tasks.len(), 6);
        assert_eq!(back.tasks[5].name, "新任务");
    }

    #[test]
    fn save_writes_the_schema_version() {
        let dir = temp_dir("version");
        let store = Store::new(&dir);
        store.save(&store.load()).expect("save");
        let raw = fs::read_to_string(dir.join("state.json")).expect("read");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(value["schemaVersion"], SCHEMA_VERSION);
    }

    #[test]
    fn an_unknown_schema_version_is_backed_up_not_overwritten() {
        let dir = temp_dir("migrate");
        fs::write(
            dir.join("state.json"),
            r#"{"schemaVersion":999,"model":{"whatever":true}}"#,
        )
        .expect("write");

        let model = Store::new(&dir).load();
        assert_eq!(model.tasks.len(), 5); // fell back to a fresh model
        assert!(dir.join("state.json.bak").exists());
        let backup = fs::read_to_string(dir.join("state.json.bak")).expect("read backup");
        assert!(backup.contains("999"));
    }

    #[test]
    fn a_corrupt_file_falls_back_without_panicking() {
        let dir = temp_dir("corrupt");
        fs::write(dir.join("state.json"), "{ not json at all").expect("write");
        let model = Store::new(&dir).load();
        assert_eq!(model.tasks.len(), 5);
    }

    #[test]
    fn save_leaves_no_temp_file_behind() {
        let dir = temp_dir("atomic");
        let store = Store::new(&dir);
        store.save(&store.load()).expect("save");
        let leftovers: Vec<_> = fs::read_dir(&dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "left behind {leftovers:?}");
    }
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod store;
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test store::`
Expected: FAIL — `Store` is not defined.

- [ ] **Step 3: Write the implementation**

Prepend to `src-tauri/src/store.rs`:

```rust
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::Model;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    schema_version: u32,
    model: Model,
}

pub struct Store {
    path: PathBuf,
}

impl Store {
    pub fn new(dir: &Path) -> Self {
        Self {
            path: dir.join("state.json"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Read the model from disk. Any problem — missing file, corrupt JSON, a schema
    /// version we do not understand — falls back to a freshly seeded model rather than
    /// failing to launch. An unrecognised version is preserved as `state.json.bak` first
    /// so a downgrade never silently destroys the user's data.
    pub fn load(&self) -> Model {
        let Ok(raw) = fs::read_to_string(&self.path) else {
            return fresh();
        };

        let version = serde_json::from_str::<serde_json::Value>(&raw)
            .ok()
            .and_then(|v| v.get("schemaVersion").and_then(|v| v.as_u64()));

        match version {
            Some(v) if v == SCHEMA_VERSION as u64 => serde_json::from_str::<Envelope>(&raw)
                .map(|e| e.model)
                .unwrap_or_else(|_| fresh()),
            Some(_) => {
                let _ = fs::write(self.path.with_extension("json.bak"), &raw);
                fresh()
            }
            None => fresh(),
        }
    }

    /// Write the model atomically: a sibling temp file, then a rename, so a crash
    /// mid-write can never leave a half-written state.json.
    pub fn save(&self, model: &Model) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let envelope = Envelope {
            schema_version: SCHEMA_VERSION,
            model: model.clone(),
        };
        let json = serde_json::to_string_pretty(&envelope)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

fn fresh() -> Model {
    let mut model = Model::default();
    model.seed_demo_tasks();
    model
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd src-tauri && cargo test store::`
Expected: PASS, 6 tests.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/store.rs src-tauri/src/lib.rs
git commit -m "feat(rust): add the atomic JSON store"
```

---

### Task 5: Events and AppState

**Files:**
- Create: `src-tauri/src/events.rs`, `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `Model`, `Timer`, `PhaseChange`, `Store`.
- Produces:
  - `events::TICK = "timer:tick"`, `PHASE = "timer:phase"`, `CHANGED = "model:changed"`
  - `pub struct TickPayload { remaining_secs: u32, phase: Phase, running: bool, round: u8, belly_cells: u8 }`
  - `pub struct ChangedPayload { section: &'static str }`
  - `pub enum Section { Tasks, Settings, Timer }` with `as_str()`
  - `pub struct AppState { model: Mutex<Model>, store: Store, last_save: Mutex<Instant> }`
  - `AppState::new(store: Store) -> AppState`
  - `AppState::with<R>(&self, f: impl FnOnce(&mut Model) -> R) -> R` — locks, recovering from poisoning
  - `AppState::snapshot(&self) -> Model`
  - `AppState::save_debounced(&self)` — writes at most once per second
  - `AppState::flush(&self)` — unconditional save, for app exit
  - `AppState::tick(&self, app: &AppHandle, elapsed_secs: u32)` — advances, emits, credits tasks, saves

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/state.rs` with only the test module. These tests exercise the
non-Tauri half; emission is verified manually in plan 3.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Phase;
    use std::fs;

    fn store_in(tag: &str) -> Store {
        let dir = std::env::temp_dir().join(format!("momo-state-test-{tag}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        Store::new(&dir)
    }

    #[test]
    fn new_loads_the_model_from_the_store() {
        let state = AppState::new(store_in("load"));
        assert_eq!(state.snapshot().tasks.len(), 5);
    }

    #[test]
    fn with_mutates_under_the_lock_and_returns_a_value() {
        let state = AppState::new(store_in("with"));
        let id = state.with(|m| m.add_task("新的".into(), 1));
        assert_eq!(state.snapshot().tasks.len(), 6);
        assert!(state.snapshot().tasks.iter().any(|t| t.id == id));
    }

    #[test]
    fn advance_credits_the_active_task_when_a_focus_phase_completes() {
        let state = AppState::new(store_in("credit"));
        let before = state.snapshot();
        let active = before.timer.active_task.expect("seeded active task");
        let spent_before = before
            .tasks
            .iter()
            .find(|t| t.id == active)
            .map(|t| t.spent)
            .unwrap();

        let changes = state.with(|m| {
            m.timer.start();
            let changes = m.timer.advance(m.settings.focus_secs, &m.settings.clone());
            for change in &changes {
                if change.completed && change.from == Phase::Focus {
                    if let Some(id) = m.timer.active_task {
                        m.credit_task(id);
                    }
                }
            }
            changes
        });

        assert_eq!(changes.len(), 1);
        let after = state.snapshot();
        let spent_after = after.tasks.iter().find(|t| t.id == active).unwrap().spent;
        assert_eq!(spent_after, spent_before + 1);
    }

    #[test]
    fn flush_writes_to_disk() {
        let state = AppState::new(store_in("flush"));
        state.with(|m| m.timer.remaining_secs = 7);
        state.flush();
        let reloaded = state.store.load();
        assert_eq!(reloaded.timer.remaining_secs, 7);
    }

    #[test]
    fn save_debounced_writes_at_most_once_per_second() {
        let state = AppState::new(store_in("debounce"));
        state.with(|m| m.timer.remaining_secs = 11);
        state.save_debounced();
        state.with(|m| m.timer.remaining_secs = 22);
        state.save_debounced(); // suppressed
        assert_eq!(state.store.load().timer.remaining_secs, 11);
    }

    #[test]
    fn a_poisoned_lock_is_recovered_rather_than_panicking() {
        use std::sync::Arc;
        let state = Arc::new(AppState::new(store_in("poison")));
        let clone = Arc::clone(&state);
        let _ = std::thread::spawn(move || {
            clone.with(|_| panic!("poison the mutex"));
        })
        .join();
        // Must not panic.
        assert_eq!(state.snapshot().tasks.len(), 5);
    }
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd src-tauri && cargo test state::`
Expected: FAIL — `AppState` is not defined.

- [ ] **Step 3: Write events.rs**

Create `src-tauri/src/events.rs`:

```rust
use serde::Serialize;

use crate::model::Phase;

pub const TICK: &str = "timer:tick";
pub const PHASE: &str = "timer:phase";
pub const CHANGED: &str = "model:changed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickPayload {
    pub remaining_secs: u32,
    pub phase: Phase,
    pub running: bool,
    pub round: u8,
    pub belly_cells: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum Section {
    Tasks,
    Settings,
    Timer,
}

impl Section {
    pub fn as_str(self) -> &'static str {
        match self {
            Section::Tasks => "tasks",
            Section::Settings => "settings",
            Section::Timer => "timer",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangedPayload {
    pub section: &'static str,
}
```

- [ ] **Step 4: Write state.rs**

Prepend to `src-tauri/src/state.rs`:

```rust
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::events::{self, ChangedPayload, Section, TickPayload};
use crate::model::{Model, Phase};
use crate::store::Store;

const SAVE_INTERVAL: Duration = Duration::from_secs(1);

pub struct AppState {
    model: Mutex<Model>,
    pub store: Store,
    last_save: Mutex<Instant>,
}

impl AppState {
    pub fn new(store: Store) -> Self {
        let model = store.load();
        Self {
            model: Mutex::new(model),
            store,
            // Start a full interval in the past so the first save is not suppressed.
            last_save: Mutex::new(Instant::now() - SAVE_INTERVAL),
        }
    }

    /// Run `f` against the model under the lock.
    ///
    /// A panic inside a previous `with` poisons the mutex; recovering the inner value
    /// keeps a single bad command from bricking the app for the rest of the session.
    pub fn with<R>(&self, f: impl FnOnce(&mut Model) -> R) -> R {
        let mut guard = self.model.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut guard)
    }

    pub fn snapshot(&self) -> Model {
        self.with(|m| m.clone())
    }

    pub fn save_debounced(&self) {
        let mut last = self.last_save.lock().unwrap_or_else(|e| e.into_inner());
        if last.elapsed() < SAVE_INTERVAL {
            return;
        }
        *last = Instant::now();
        drop(last);
        self.flush();
    }

    pub fn flush(&self) {
        let model = self.snapshot();
        if let Err(err) = self.store.save(&model) {
            eprintln!("momo: failed to persist state: {err}");
        }
    }

    pub fn emit_changed(&self, app: &AppHandle, section: Section) {
        let _ = app.emit(
            events::CHANGED,
            ChangedPayload {
                section: section.as_str(),
            },
        );
    }

    fn tick_payload(&self) -> TickPayload {
        self.with(|m| TickPayload {
            remaining_secs: m.timer.remaining_secs,
            phase: m.timer.phase,
            running: m.timer.running,
            round: m.timer.round,
            belly_cells: m.timer.belly_cells(&m.settings),
        })
    }

    pub fn emit_tick(&self, app: &AppHandle) {
        let _ = app.emit(events::TICK, self.tick_payload());
    }

    /// Advance the clock by real elapsed time, credit any completed focus phase to the
    /// active task, then emit. Called once per second by the tick thread.
    pub fn tick(&self, app: &AppHandle, elapsed_secs: u32) {
        let changes = self.with(|m| {
            let settings = m.settings.clone();
            let changes = m.timer.advance(elapsed_secs, &settings);
            for change in &changes {
                if change.completed && change.from == Phase::Focus {
                    if let Some(id) = m.timer.active_task {
                        m.credit_task(id);
                    }
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
}
```

Add to `src-tauri/src/lib.rs`:

```rust
pub mod events;
pub mod state;
```

- [ ] **Step 5: Run the test to verify it passes**

Run: `cd src-tauri && cargo test state::`
Expected: PASS, 6 tests.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/events.rs src-tauri/src/state.rs src-tauri/src/lib.rs
git commit -m "feat(rust): add AppState with debounced persistence and events"
```

---

### Task 6: Command surface

**Files:**
- Create: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `AppState`, `Model`, `Section`.
- Produces these `#[tauri::command]` functions, all taking `State<'_, AppState>` and `AppHandle`:
  - `list_model() -> Model`
  - `start()`, `pause()`, `skip_phase()`
  - `set_active_task(id: Option<TaskId>)`
  - `add_task(name: String, estimate: u8) -> TaskId`
  - `toggle_task(id: TaskId)`, `delete_task(id: TaskId)`
  - `set_accent(accent: Accent)`, `set_tone(tone: Tone)`
  - `set_pet_flag(flag: String, value: bool)`

- [ ] **Step 1: Write the implementation**

Create `src-tauri/src/commands.rs`:

```rust
use tauri::{AppHandle, State};

use crate::events::Section;
use crate::model::{Accent, Model, TaskId, Tone};
use crate::state::AppState;

#[tauri::command]
pub fn list_model(state: State<'_, AppState>) -> Model {
    state.snapshot()
}

#[tauri::command]
pub fn start(state: State<'_, AppState>, app: AppHandle) {
    state.with(|m| m.timer.start());
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Timer);
    state.flush();
}

#[tauri::command]
pub fn pause(state: State<'_, AppState>, app: AppHandle) {
    state.with(|m| m.timer.pause());
    state.emit_tick(&app);
    state.emit_changed(&app, Section::Timer);
    state.flush();
}

#[tauri::command]
pub fn skip_phase(state: State<'_, AppState>, app: AppHandle) {
    let change = state.with(|m| {
        let settings = m.settings.clone();
        m.timer.skip(&settings)
    });
    let _ = tauri::Emitter::emit(&app, crate::events::PHASE, change);
    state.emit_tick(&app);
    state.flush();
}

#[tauri::command]
pub fn set_active_task(state: State<'_, AppState>, app: AppHandle, id: Option<TaskId>) {
    state.with(|m| m.timer.active_task = id);
    state.emit_changed(&app, Section::Timer);
    state.flush();
}

#[tauri::command]
pub fn add_task(
    state: State<'_, AppState>,
    app: AppHandle,
    name: String,
    estimate: u8,
) -> TaskId {
    let id = state.with(|m| m.add_task(name, estimate));
    state.emit_changed(&app, Section::Tasks);
    state.flush();
    id
}

#[tauri::command]
pub fn toggle_task(state: State<'_, AppState>, app: AppHandle, id: TaskId) {
    state.with(|m| m.toggle_task(id));
    state.emit_changed(&app, Section::Tasks);
    state.flush();
}

#[tauri::command]
pub fn delete_task(state: State<'_, AppState>, app: AppHandle, id: TaskId) {
    state.with(|m| m.delete_task(id));
    state.emit_changed(&app, Section::Tasks);
    state.flush();
}

#[tauri::command]
pub fn set_accent(state: State<'_, AppState>, app: AppHandle, accent: Accent) {
    state.with(|m| m.settings.accent = accent);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

#[tauri::command]
pub fn set_tone(state: State<'_, AppState>, app: AppHandle, tone: Tone) {
    state.with(|m| m.settings.tone = tone);
    state.emit_changed(&app, Section::Settings);
    state.flush();
}

/// `flag` is one of snapEdges / clickInteract / hideFullscreen / sleepAnimation.
#[tauri::command]
pub fn set_pet_flag(state: State<'_, AppState>, app: AppHandle, flag: String, value: bool) {
    state.with(|m| {
        let f = &mut m.settings.pet_flags;
        match flag.as_str() {
            "snapEdges" => f.snap_edges = value,
            "clickInteract" => f.click_interact = value,
            "hideFullscreen" => f.hide_fullscreen = value,
            "sleepAnimation" => f.sleep_animation = value,
            _ => {}
        }
    });
    state.emit_changed(&app, Section::Settings);
    state.flush();
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles (the module is not yet declared, so also add `pub mod commands;` to
`lib.rs` before running).

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(rust): add the command surface"
```

---

### Task 7: Wire the builder and the tick thread

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: everything above.
- Produces: a running app that manages `AppState`, ticks once per second from a monotonic
  clock, and flushes on exit.

- [ ] **Step 1: Rewrite lib.rs**

Replace `src-tauri/src/lib.rs` entirely:

```rust
pub mod commands;
pub mod core;
pub mod events;
pub mod model;
pub mod state;
pub mod store;

use std::thread;
use std::time::{Duration, Instant};

use tauri::Manager;

use crate::state::AppState;
use crate::store::Store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("app data dir")
                .join("momo");
            std::fs::create_dir_all(&dir)?;
            app.manage(AppState::new(Store::new(&dir)));

            let handle = app.handle().clone();
            thread::spawn(move || {
                // Elapsed time comes from a monotonic instant, not from counting
                // iterations: if the machine sleeps for an hour, the next wake passes
                // the whole gap to advance() and every crossed phase is reported.
                let mut previous = Instant::now();
                loop {
                    thread::sleep(Duration::from_secs(1));
                    let now = Instant::now();
                    let elapsed = now.duration_since(previous).as_secs();
                    previous = now;
                    if elapsed == 0 {
                        continue;
                    }
                    let state = handle.state::<AppState>();
                    state.tick(&handle, elapsed as u32);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_model,
            commands::start,
            commands::pause,
            commands::skip_phase,
            commands::set_active_task,
            commands::add_task,
            commands::toggle_task,
            commands::delete_task,
            commands::set_accent,
            commands::set_tone,
            commands::set_pet_flag,
        ])
        .build(tauri::generate_context!())
        .expect("error while building the Momo application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                app.state::<AppState>().flush();
            }
        });
}
```

- [ ] **Step 2: Verify the whole crate**

Run: `cd src-tauri && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
Expected: fmt rewrites nothing further, clippy is clean, all 37 tests pass.

- [ ] **Step 3: Verify the app launches and ticks**

Run: `npm run tauri dev`

In the webview devtools console:

```js
const { invoke } = await import("@tauri-apps/api/core");
const { listen } = await import("@tauri-apps/api/event");
await listen("timer:tick", (e) => console.log(e.payload));
console.log(await invoke("list_model"));
await invoke("start");
```

Expected: `list_model` returns the model with five seeded tasks; after `start`, a
`timer:tick` payload logs once per second with `remainingSecs` counting down and
`bellyCells` climbing.

- [ ] **Step 4: Verify persistence survives a restart**

Quit the app, then:

```bash
cat "$HOME/Library/Application Support/com.pomodo.app/momo/state.json" | head -20
```

Expected: `schemaVersion: 1` and a `remainingSecs` below 1500. Relaunch and confirm
`list_model` returns the same `remainingSecs`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(rust): wire the builder, tick thread and exit flush"
```

---

## Definition of Done

- `cargo fmt --check`, `cargo clippy --all-targets -D warnings` and `cargo test` all pass.
- The timer advances correctly across a simulated hour-long sleep (covered by
  `a_long_sleep_rolls_through_every_phase_it_crossed`).
- `state.json` is written atomically, carries `schemaVersion: 1`, and an unknown version
  is backed up rather than clobbered.
- The app launches, emits `timer:tick` once per second, and restores its timer after a
  restart.
- No frontend file was touched by this plan.
