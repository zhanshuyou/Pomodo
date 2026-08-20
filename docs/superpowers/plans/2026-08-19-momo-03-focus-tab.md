# Pomodo 03 — 主窗口 · 专注 Tab Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Connect the frontend to the Rust core and build the 专注 tab from artboard 01 — the pet-as-timer with belly progress, the countdown, the tone-aware speech bubble, the task sidebar, and the body-stats footer. This plan produces the first genuinely usable Pomodoro.

**Architecture:** A typed `ipc.ts` layer is the only place that touches `@tauri-apps/api`. A single runes store in `state.svelte.ts` holds the mirrored model, hydrates from `list_model` and patches on `timer:tick` / `timer:phase` / `model:changed`. Components read the store and call `ipc` functions; none of them invoke directly. All copy lives in `copy.ts` as tone-aware functions.

**Tech Stack:** Svelte 5 runes, TypeScript, `@tauri-apps/api` v2, vitest.

**Spec:** `docs/superpowers/specs/2026-08-19-momo-design.md`
**Depends on:** plan 01 (foundation) and plan 02 (timer core), both complete.

## Global Constraints

- Every user-facing string comes from spec §5 verbatim. `＋ 加一件事（⌘N）` uses the fullwidth `＋` and fullwidth parentheses.
- Only `src/lib/ipc.ts` may import from `@tauri-apps/api`. Components import from `ipc.ts` and `state.svelte.ts`.
- Colours are the CSS tokens from plan 01 (`var(--accent)`, `var(--line)`, …). No new raw `oklch()` literals except the ones spec §8.1 names explicitly for this screen.
- The accent must react live: setting `document.documentElement.dataset.accent` is the only mechanism.
- `npm test`, `npm run check`, `npm run build`, `cargo test`, `cargo clippy -D warnings` all stay green.
- Layout numbers in spec §8.1 are exact: window 1180 wide, sidebar 372, title bar 46, timer 78px, belly cells 11×11, pips 7×7, task checkbox 17px.

---

## File Structure

| Path | Responsibility |
| --- | --- |
| `src/lib/ipc.ts` | Typed `invoke` + `listen` wrappers; the only Tauri import site |
| `src/lib/copy.ts` | Every tone-aware string from spec §5 |
| `src/lib/format.ts` | `mmss`, `endsAt`, minute extraction |
| `src/lib/state.svelte.ts` | The runes store: hydrate, patch, expose derived values |
| `src/routes/main/App.svelte` | Window chrome, tab switching |
| `src/routes/main/FocusTab.svelte` | Left column of artboard 01 |
| `src/routes/main/TaskSidebar.svelte` | Right column of artboard 01 |
| `src/lib/format.test.ts`, `src/lib/copy.test.ts` | vitest suites |
| `src-tauri/tauri.conf.json` | Window size + the `prefs` window definition |
| `src-tauri/capabilities/default.json` | Event + core permissions |

---

### Task 1: Formatting helpers

**Files:**
- Create: `src/lib/format.ts`
- Test: `src/lib/format.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `export function mmss(totalSecs: number): string` — zero-padded `MM:SS`, clamped at 0
  - `export function minutesLeft(totalSecs: number): number` — floor of minutes, used in the pet line
  - `export function endsAt(remainingSecs: number, now?: Date): string` — `预计 HH:MM 结束`

- [ ] **Step 1: Write the failing test**

Create `src/lib/format.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { endsAt, minutesLeft, mmss } from "./format";

describe("mmss", () => {
  it("zero-pads both fields", () => {
    expect(mmss(0)).toBe("00:00");
    expect(mmss(65)).toBe("01:05");
    expect(mmss(1500)).toBe("25:00");
  });

  it("does not roll over past 60 minutes", () => {
    expect(mmss(3661)).toBe("61:01");
  });

  it("clamps negatives to zero", () => {
    expect(mmss(-5)).toBe("00:00");
  });
});

describe("minutesLeft", () => {
  it("floors to whole minutes", () => {
    expect(minutesLeft(1500)).toBe(25);
    expect(minutesLeft(119)).toBe(1);
    expect(minutesLeft(59)).toBe(0);
  });
});

describe("endsAt", () => {
  it("renders the wall-clock finish time", () => {
    const now = new Date(2026, 7, 19, 14, 26, 0);
    expect(endsAt(1500, now)).toBe("预计 14:51 结束");
  });

  it("zero-pads the hour and minute", () => {
    const now = new Date(2026, 7, 19, 8, 3, 0);
    expect(endsAt(300, now)).toBe("预计 08:08 结束");
  });

  it("wraps past midnight", () => {
    const now = new Date(2026, 7, 19, 23, 50, 0);
    expect(endsAt(1500, now)).toBe("预计 00:15 结束");
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- src/lib/format.test.ts`
Expected: FAIL — cannot resolve `./format`.

- [ ] **Step 3: Write the implementation**

Create `src/lib/format.ts`:

```ts
function pad(n: number): string {
  return String(n).padStart(2, "0");
}

/** `MM:SS`, where MM is not capped at 60. */
export function mmss(totalSecs: number): string {
  const secs = Math.max(0, Math.floor(totalSecs));
  return `${pad(Math.floor(secs / 60))}:${pad(secs % 60)}`;
}

/** Whole minutes remaining, as interpolated into the pet's line. */
export function minutesLeft(totalSecs: number): number {
  return Math.floor(Math.max(0, totalSecs) / 60);
}

/** The design's `预计 HH:MM 结束` line under the countdown. */
export function endsAt(remainingSecs: number, now: Date = new Date()): string {
  const end = new Date(now.getTime() + Math.max(0, remainingSecs) * 1000);
  return `预计 ${pad(end.getHours())}:${pad(end.getMinutes())} 结束`;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- src/lib/format.test.ts`
Expected: PASS, 7 tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/format.ts src/lib/format.test.ts
git commit -m "feat: add timer formatting helpers"
```

---

### Task 2: Tone-aware copy

**Files:**
- Create: `src/lib/copy.ts`
- Test: `src/lib/copy.test.ts`

**Interfaces:**
- Consumes: `Tone`, `tone` from `src/lib/theme.ts`.
- Produces:
  - `export function tagline(t: Tone): string`
  - `export function petLine(t: Tone, minutes: number): string`
  - `export function petVerdict(t: Tone): string`
  - `export function phaseLabel(phase: Phase): string` where `Phase = "focus" | "shortBreak" | "longBreak"`
  - `export function runLabel(running: boolean): string`
  - `export function miniLabel(mini: boolean): string`
  - `export type Phase = "focus" | "shortBreak" | "longBreak"`

Reminder copy (spec §5.4) arrives in plan 05; this task covers only what the 专注 tab and
统计 tab need.

- [ ] **Step 1: Write the failing test**

Create `src/lib/copy.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { miniLabel, petLine, petVerdict, phaseLabel, runLabel, tagline } from "./copy";

describe("tagline", () => {
  it("matches the spec for every tone", () => {
    expect(tagline("professional")).toBe(
      "菜单栏与主窗口双入口的番茄计时器，含可自定义的身体提醒与桌面宠物。",
    );
    expect(tagline("gentle")).toBe(
      "一个陪你专注的番茄钟：它记得提醒你站立喝水，也记得在你完成时替你高兴。",
    );
    expect(tagline("playful")).toBe(
      "它负责计时、催你喝水、盯你站起来，并在你摸鱼时用眼神谴责你。",
    );
  });
});

describe("petLine", () => {
  it("interpolates the minute count per tone", () => {
    expect(petLine("professional", 12)).toBe("本轮剩余 12 分钟。");
    expect(petLine("gentle", 12)).toBe("再 12 分钟就好，我陪着你。");
    expect(petLine("playful", 12)).toBe("还有 12 分钟，我盯着你呢");
  });
});

describe("petVerdict", () => {
  it("matches the spec for every tone", () => {
    expect(petVerdict("professional")).toBe("本周专注 14h20m，较上周 +12%，中断率下降。");
    expect(petVerdict("gentle")).toBe("这周你比上周多专注了 1 小时 40 分，很稳。");
    expect(petVerdict("playful")).toBe("这周表现不错，我勉为其难地允许你今晚多睡半小时。");
  });
});

describe("phaseLabel", () => {
  it("labels focus and both breaks", () => {
    expect(phaseLabel("focus")).toBe("专注中");
    expect(phaseLabel("shortBreak")).toBe("休息中");
    expect(phaseLabel("longBreak")).toBe("休息中");
  });
});

describe("button labels", () => {
  it("swaps on state", () => {
    expect(runLabel(true)).toBe("让它歇会儿");
    expect(runLabel(false)).toBe("开始专注");
    expect(miniLabel(true)).toBe("退出迷你模式");
    expect(miniLabel(false)).toBe("迷你模式");
  });
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- src/lib/copy.test.ts`
Expected: FAIL — cannot resolve `./copy`.

- [ ] **Step 3: Write the implementation**

Create `src/lib/copy.ts`:

```ts
import { type Tone, tone } from "./theme";

export type Phase = "focus" | "shortBreak" | "longBreak";

export function tagline(t: Tone): string {
  return tone(
    t,
    "菜单栏与主窗口双入口的番茄计时器，含可自定义的身体提醒与桌面宠物。",
    "一个陪你专注的番茄钟：它记得提醒你站立喝水，也记得在你完成时替你高兴。",
    "它负责计时、催你喝水、盯你站起来，并在你摸鱼时用眼神谴责你。",
  );
}

export function petLine(t: Tone, minutes: number): string {
  return tone(
    t,
    `本轮剩余 ${minutes} 分钟。`,
    `再 ${minutes} 分钟就好，我陪着你。`,
    `还有 ${minutes} 分钟，我盯着你呢`,
  );
}

export function petVerdict(t: Tone): string {
  return tone(
    t,
    "本周专注 14h20m，较上周 +12%，中断率下降。",
    "这周你比上周多专注了 1 小时 40 分，很稳。",
    "这周表现不错，我勉为其难地允许你今晚多睡半小时。",
  );
}

export function phaseLabel(phase: Phase): string {
  return phase === "focus" ? "专注中" : "休息中";
}

export function runLabel(running: boolean): string {
  return running ? "让它歇会儿" : "开始专注";
}

export function miniLabel(mini: boolean): string {
  return mini ? "退出迷你模式" : "迷你模式";
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- src/lib/copy.test.ts`
Expected: PASS, 6 tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/copy.ts src/lib/copy.test.ts
git commit -m "feat: add tone-aware copy for the focus and stats tabs"
```

---

### Task 3: Typed IPC layer

**Files:**
- Create: `src/lib/ipc.ts`
- Modify: `src-tauri/capabilities/default.json`

**Interfaces:**
- Consumes: the commands and events from plan 02.
- Produces:
  - `export interface Task { id: number; name: string; estimate: number; spent: number; done: boolean }`
  - `export interface Timer { phase: Phase; remainingSecs: number; running: boolean; round: number; activeTask: number | null }`
  - `export interface PetFlags { snapEdges: boolean; clickInteract: boolean; hideFullscreen: boolean; sleepAnimation: boolean }`
  - `export interface Settings { accent: Accent; tone: Tone; focusSecs: number; shortBreakSecs: number; longBreakSecs: number; roundsPerCycle: number; petFlags: PetFlags }`
  - `export interface Model { timer: Timer; tasks: Task[]; settings: Settings; nextTaskId: number }`
  - `export interface TickPayload { remainingSecs: number; phase: Phase; running: boolean; round: number; bellyCells: number }`
  - `export interface PhaseChange { from: Phase; to: Phase; round: number; completed: boolean }`
  - `export interface ChangedPayload { section: "tasks" | "settings" | "timer" }`
  - Functions: `listModel`, `start`, `pause`, `skipPhase`, `setActiveTask`, `addTask`, `toggleTask`, `deleteTask`, `setAccent`, `setTone`, `setPetFlag`
  - `export function onTick(cb)`, `onPhase(cb)`, `onChanged(cb)` — each returns `Promise<UnlistenFn>`
  - `export const IS_TAURI: boolean`

- [ ] **Step 1: Write the implementation**

Create `src/lib/ipc.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";

import type { Phase } from "./copy";
import type { Accent, Tone } from "./theme";

export interface Task {
  id: number;
  name: string;
  estimate: number;
  spent: number;
  done: boolean;
}

export interface Timer {
  phase: Phase;
  remainingSecs: number;
  running: boolean;
  round: number;
  activeTask: number | null;
}

export interface PetFlags {
  snapEdges: boolean;
  clickInteract: boolean;
  hideFullscreen: boolean;
  sleepAnimation: boolean;
}

export interface Settings {
  accent: Accent;
  tone: Tone;
  focusSecs: number;
  shortBreakSecs: number;
  longBreakSecs: number;
  roundsPerCycle: number;
  petFlags: PetFlags;
}

export interface Model {
  timer: Timer;
  tasks: Task[];
  settings: Settings;
  nextTaskId: number;
}

export interface TickPayload {
  remainingSecs: number;
  phase: Phase;
  running: boolean;
  round: number;
  bellyCells: number;
}

export interface PhaseChange {
  from: Phase;
  to: Phase;
  round: number;
  completed: boolean;
}

export interface ChangedPayload {
  section: "tasks" | "settings" | "timer";
}

/** False when the page is opened in a plain browser, e.g. gallery.html. */
export const IS_TAURI =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export const listModel = () => invoke<Model>("list_model");
export const start = () => invoke<void>("start");
export const pause = () => invoke<void>("pause");
export const skipPhase = () => invoke<void>("skip_phase");
export const setActiveTask = (id: number | null) =>
  invoke<void>("set_active_task", { id });
export const addTask = (name: string, estimate: number) =>
  invoke<number>("add_task", { name, estimate });
export const toggleTask = (id: number) => invoke<void>("toggle_task", { id });
export const deleteTask = (id: number) => invoke<void>("delete_task", { id });
export const setAccent = (accent: Accent) => invoke<void>("set_accent", { accent });
export const setTone = (tone: Tone) => invoke<void>("set_tone", { tone });
export const setPetFlag = (flag: keyof PetFlags, value: boolean) =>
  invoke<void>("set_pet_flag", { flag, value });

export const onTick = (cb: (p: TickPayload) => void): Promise<UnlistenFn> =>
  listen<TickPayload>("timer:tick", (e) => cb(e.payload));
export const onPhase = (cb: (p: PhaseChange) => void): Promise<UnlistenFn> =>
  listen<PhaseChange>("timer:phase", (e) => cb(e.payload));
export const onChanged = (cb: (p: ChangedPayload) => void): Promise<UnlistenFn> =>
  listen<ChangedPayload>("model:changed", (e) => cb(e.payload));
```

- [ ] **Step 2: Grant the event permissions**

Replace `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the Pomodo windows",
  "windows": ["main", "prefs", "tray", "pet", "overlay"],
  "permissions": [
    "core:default",
    "core:event:default",
    "core:window:allow-start-dragging",
    "opener:default"
  ]
}
```

- [ ] **Step 3: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/lib/ipc.ts src-tauri/capabilities/default.json
git commit -m "feat: add the typed IPC layer"
```

---

### Task 4: The runes store

**Files:**
- Create: `src/lib/state.svelte.ts`

**Interfaces:**
- Consumes: everything from `ipc.ts`, `theme.ts`, `format.ts`.
- Produces a module-level singleton `export const app` with:
  - `app.model: Model | null`, `app.ready: boolean`
  - `app.timer`, `app.tasks`, `app.settings` — throw-free getters that fall back to defaults before hydration
  - `app.bellyCells: number`, `app.accentCss: string`, `app.tone: Tone`
  - `app.activeTaskName: string`
  - `app.init(): Promise<void>` — hydrates and subscribes; safe to call twice
  - `app.dispose(): void` — unlistens

- [ ] **Step 1: Write the implementation**

Create `src/lib/state.svelte.ts`:

```svelte-ts
import type { UnlistenFn } from "@tauri-apps/api/event";

import {
  IS_TAURI,
  type Model,
  listModel,
  onChanged,
  onPhase,
  onTick,
} from "./ipc";
import { ACCENTS, DEFAULT_ACCENT, DEFAULT_TONE, type Tone } from "./theme";

const FALLBACK: Model = {
  timer: {
    phase: "focus",
    remainingSecs: 1500,
    running: false,
    round: 1,
    activeTask: null,
  },
  tasks: [],
  settings: {
    accent: DEFAULT_ACCENT,
    tone: DEFAULT_TONE,
    focusSecs: 1500,
    shortBreakSecs: 300,
    longBreakSecs: 900,
    roundsPerCycle: 4,
    petFlags: {
      snapEdges: true,
      clickInteract: true,
      hideFullscreen: true,
      sleepAnimation: false,
    },
  },
  nextTaskId: 0,
};

class AppStore {
  model = $state<Model>(FALLBACK);
  ready = $state(false);
  bellyCells = $state(0);

  #unlisteners: UnlistenFn[] = [];
  #started = false;

  get timer() {
    return this.model.timer;
  }
  get tasks() {
    return this.model.tasks;
  }
  get settings() {
    return this.model.settings;
  }
  get tone(): Tone {
    return this.model.settings.tone;
  }
  get accentCss(): string {
    return ACCENTS[this.model.settings.accent];
  }
  get activeTaskName(): string {
    const id = this.model.timer.activeTask;
    return this.model.tasks.find((t) => t.id === id)?.name ?? "";
  }

  async init(): Promise<void> {
    if (this.#started) return;
    this.#started = true;

    if (!IS_TAURI) {
      // gallery.html and vitest run outside Tauri; keep the fallback model.
      this.ready = true;
      return;
    }

    await this.refresh();

    this.#unlisteners.push(
      await onTick((p) => {
        this.model.timer.remainingSecs = p.remainingSecs;
        this.model.timer.phase = p.phase;
        this.model.timer.running = p.running;
        this.model.timer.round = p.round;
        this.bellyCells = p.bellyCells;
      }),
      // A phase boundary can credit a task and change settings-derived durations,
      // so refetch rather than trying to patch every dependent field by hand.
      await onPhase(() => void this.refresh()),
      await onChanged(() => void this.refresh()),
    );

    this.ready = true;
  }

  async refresh(): Promise<void> {
    this.model = await listModel();
  }

  dispose(): void {
    for (const un of this.#unlisteners) un();
    this.#unlisteners = [];
    this.#started = false;
  }
}

export const app = new AppStore();
```

Rename the file extension in the create step to `state.svelte.ts` (Svelte's runes compiler
requires the `.svelte.ts` suffix for `$state` outside components). The fenced language above
is illustrative only — the file is plain TypeScript.

- [ ] **Step 2: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/state.svelte.ts
git commit -m "feat: add the runes store mirroring the Rust model"
```

---

### Task 5: Window chrome and tab switching

**Files:**
- Modify: `src/routes/main/App.svelte`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: `app` store, `TitleBar`.
- Produces: `App.svelte` rendering the 46px title bar with the three-tab segmented control
  and mounting the active tab. `StatsTab` and `PetTab` are placeholders until plan 04.

- [ ] **Step 1: Size the window**

In `src-tauri/tauri.conf.json`, replace the `app.windows` array:

```json
"windows": [
  {
    "label": "main",
    "title": "Pomodo",
    "width": 1180,
    "height": 700,
    "minWidth": 1180,
    "minHeight": 660,
    "resizable": true,
    "titleBarStyle": "Overlay",
    "hiddenTitle": true
  }
]
```

`titleBarStyle: "Overlay"` with `hiddenTitle` keeps the native traffic lights while letting
our own 46px bar own the area — this is what makes the artboard's custom title bar work on
macOS. On Linux and Windows these two keys are ignored and the native frame is used.

- [ ] **Step 2: Write App.svelte**

Replace `src/routes/main/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import TitleBar from "../../lib/components/TitleBar.svelte";
  import { app } from "../../lib/state.svelte";
  import FocusTab from "./FocusTab.svelte";

  const TABS = ["专注", "统计", "宠物"] as const;
  let tab = $state(0);

  onMount(() => {
    void app.init();
    return () => app.dispose();
  });

  // The accent lives on the root element so every token resolves against it.
  $effect(() => {
    document.documentElement.dataset.accent = app.settings.accent;
  });
</script>

<div class="window">
  <TitleBar title="Pomodo">
    <div class="tabs">
      {#each TABS as name, i (name)}
        <button class="tab" class:active={tab === i} type="button" onclick={() => (tab = i)}>
          {name}
        </button>
      {/each}
    </div>
    <div class="meta">
      <span>连续 12 天</span>
      <span class="sep"></span>
      <span>⌘,</span>
    </div>
  </TitleBar>

  {#if tab === 0}
    <FocusTab />
  {:else if tab === 1}
    <div class="stub">统计（计划 04）</div>
  {:else}
    <div class="stub">宠物（计划 04）</div>
  {/if}
</div>

<style>
  .window {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--card);
    overflow: hidden;
  }
  .tabs {
    margin-left: 20px;
    display: flex;
    gap: 4px;
    padding: 3px;
    border-radius: 9px;
    background: oklch(0.92 0.008 70);
  }
  .tab {
    padding: 5px 14px;
    border: none;
    border-radius: 7px;
    background: transparent;
    color: var(--dim);
    font-size: 12.5px;
    font-weight: 400;
    cursor: pointer;
  }
  .tab.active {
    background: var(--card);
    color: var(--ink);
    font-weight: 600;
  }
  .meta {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 14px;
    font-size: 12.5px;
    color: oklch(0.52 0.012 60);
  }
  .sep {
    width: 1px;
    height: 14px;
    background: var(--line);
  }
  .stub {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--dim);
  }
</style>
```

- [ ] **Step 3: Verify**

Run: `npm run check`
Expected: fails only on the missing `FocusTab.svelte`, which Task 6 creates. Create an
empty placeholder to keep the checkpoint green:

```bash
printf '<script lang="ts"></script>\n\n<div></div>\n' > src/routes/main/FocusTab.svelte
```

Re-run `npm run check`; expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src/routes/main/App.svelte src/routes/main/FocusTab.svelte src-tauri/tauri.conf.json
git commit -m "feat: add the main window chrome and tab switching"
```

---

### Task 6: The 专注 tab left column

**Files:**
- Modify: `src/routes/main/FocusTab.svelte`

**Interfaces:**
- Consumes: `app`, `PetCanvas`, `PixelButton`, `SpeechBubble`, `PETS`, `mmss`, `minutesLeft`, `endsAt`, `petLine`, `phaseLabel`, `runLabel`, `miniLabel`, and `ipc.start / pause / skipPhase`.
- Produces: the full left column of artboard 01, plus a slot for `TaskSidebar` (Task 7).

**Known gap — 迷你模式.** The artboard shows the button and both of its labels, but never
shows what mini mode looks like. This task wires the button to local state so the label
toggles as designed; it does not resize or restyle the window, because there is no design
to build against. Raise this with the designer before plan 07 — the natural home for a real
mini mode is the desktop-pet window, which plan 07 builds anyway.

- [ ] **Step 1: Write FocusTab.svelte**

Replace `src/routes/main/FocusTab.svelte`:

```svelte
<script lang="ts">
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import PixelButton from "../../lib/components/PixelButton.svelte";
  import SpeechBubble from "../../lib/components/SpeechBubble.svelte";
  import { miniLabel, petLine, phaseLabel, runLabel } from "../../lib/copy";
  import { endsAt, minutesLeft, mmss } from "../../lib/format";
  import { pause, skipPhase, start } from "../../lib/ipc";
  import { PETS } from "../../lib/sprites";
  import { app } from "../../lib/state.svelte";
  import TaskSidebar from "./TaskSidebar.svelte";

  // Plan 04 replaces this with the user's selected pet.
  const pet = PETS[0];

  let mini = $state(false);

  const remaining = $derived(app.timer.remainingSecs);
  const cells = $derived(app.bellyCells);
  const roundsTotal = $derived(app.settings.roundsPerCycle);

  function toggleRun() {
    void (app.timer.running ? pause() : start());
  }
</script>

<div class="body">
  <section class="stage">
    <div class="status">
      <span class="dot"></span>
      <span>
        {phaseLabel(app.timer.phase)} · 第 {app.timer.round}/{roundsTotal} 轮 · {app.activeTaskName}
      </span>
    </div>

    <div class="petwrap">
      <div class="ring"></div>
      <PetCanvas map={pet.map} body={pet.body} scale={8} anim="bob" alt={pet.name} />
      <div class="shadow"></div>
      <div class="belly">
        {#each Array.from({ length: 10 }, (_, i) => i) as i (i)}
          <span class="cell" class:filled={i < cells}></span>
        {/each}
      </div>
    </div>

    <div class="clock">
      <span class="mmss">{mmss(remaining)}</span>
      <span class="ends">{endsAt(remaining)}</span>
    </div>

    <SpeechBubble maxWidth={340}>
      {petLine(app.tone, minutesLeft(remaining))}
    </SpeechBubble>

    <div class="actions">
      <PixelButton onclick={toggleRun}>{runLabel(app.timer.running)}</PixelButton>
      <PixelButton variant="secondary" onclick={() => void skipPhase()}>跳过</PixelButton>
      <PixelButton variant="secondary" onclick={() => (mini = !mini)}>
        {miniLabel(mini)}
      </PixelButton>
    </div>

    <div class="rounds">
      {#each Array.from({ length: roundsTotal }, (_, i) => i) as i (i)}
        <span class="pip" class:on={i < app.timer.round}></span>
      {/each}
      <span class="hint">再 2 轮就能哄它去睡长觉（15 分钟）</span>
    </div>
  </section>

  <TaskSidebar />
</div>

<style>
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  .stage {
    flex: 1;
    padding: 40px 44px 34px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 24px;
    background: linear-gradient(
      180deg,
      oklch(0.975 0.012 75) 0%,
      oklch(0.99 0.004 80) 70%
    );
    overflow-y: auto;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 14px;
    border-radius: 20px;
    background: oklch(0.99 0.004 80 / 0.8);
    font-size: 13px;
    color: oklch(0.4 0.012 60);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
  }
  .petwrap {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .ring {
    position: absolute;
    inset: -18px -34px;
    border-radius: 50%;
    border: 2px solid var(--accent);
    animation: momo-pulse 3.6s ease-in-out infinite;
    pointer-events: none;
  }
  .shadow {
    width: 124px;
    height: 11px;
    border-radius: 50%;
    background: oklch(0.24 0.012 60 / 0.14);
    filter: blur(4px);
  }
  .belly {
    display: flex;
    gap: 4px;
    margin-top: 4px;
  }
  .cell {
    width: 11px;
    height: 11px;
    background: var(--track);
  }
  .cell.filled {
    background: var(--accent);
  }
  .clock {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }
  .mmss {
    font-family: var(--font-mono);
    font-size: 78px;
    font-weight: 500;
    letter-spacing: -0.05em;
    line-height: 1;
    font-variant-numeric: tabular-nums;
  }
  .ends {
    font-size: 13px;
    color: var(--dim);
  }
  .actions {
    display: flex;
    gap: 10px;
    width: 100%;
    max-width: 420px;
  }
  .actions :global(.btn--primary) {
    flex: 1;
    padding-left: 0;
    padding-right: 0;
  }
  .rounds {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 12.5px;
    color: var(--dim);
  }
  .pip {
    width: 10px;
    height: 10px;
    background: var(--line);
  }
  .pip.on {
    background: var(--accent);
  }
  .hint {
    margin-left: 6px;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add src/routes/main/FocusTab.svelte
git commit -m "feat: build the focus tab stage column"
```

---

### Task 7: The task sidebar

**Files:**
- Create: `src/routes/main/TaskSidebar.svelte`

**Interfaces:**
- Consumes: `app`, `StatBar`, `ipc.addTask / toggleTask / setActiveTask`.
- Produces: the 372px right column of artboard 01.

The three body stats (喝水 / 站立 / 久坐最长) are hard-coded to the design's values in this
plan. Plan 05 replaces them with counters driven by the reminder engine — that is the only
source that can know how many glasses of water were actually acknowledged.

- [ ] **Step 1: Write TaskSidebar.svelte**

Create `src/routes/main/TaskSidebar.svelte`:

```svelte
<script lang="ts">
  import StatBar from "../../lib/components/StatBar.svelte";
  import { addTask, setActiveTask, toggleTask } from "../../lib/ipc";
  import { app } from "../../lib/state.svelte";

  // Placeholder until plan 05 wires the reminder engine's counters.
  const BODY_STATS = [
    { name: "喝水", value: "6 / 8 杯", pct: 75, color: "oklch(0.66 0.09 195)" },
    { name: "站立", value: "4 / 6 次", pct: 66, color: "oklch(0.63 0.13 40)" },
    { name: "久坐最长", value: "68 分钟", pct: 76, color: "oklch(0.7 0.12 60)" },
  ];

  const doneCount = $derived(app.tasks.filter((t) => t.done).length);

  function meta(task: { estimate: number; spent: number; done: boolean }): string {
    if (task.done) return `已完成 · ${task.spent} 个番茄`;
    if (task.spent > 0) return `进行中 · 已投入 ${task.spent} 个番茄`;
    return `预计 ${task.estimate} 个番茄`;
  }

  async function onAdd() {
    const name = window.prompt("要啃什么？");
    if (!name?.trim()) return;
    await addTask(name.trim(), 1);
  }
</script>

<aside class="sidebar">
  <header>
    <span class="title">今天要啃的</span>
    <span class="count">{doneCount} / {app.tasks.length} 完成</span>
  </header>

  <div class="list">
    {#each app.tasks as task (task.id)}
      <div
        class="task"
        class:selected={app.timer.activeTask === task.id && !task.done}
        role="button"
        tabindex="0"
        onclick={() => void setActiveTask(task.id)}
        onkeydown={(e) => e.key === "Enter" && void setActiveTask(task.id)}
      >
        <button
          class="box"
          class:checked={task.done}
          type="button"
          aria-label={task.done ? "标记为未完成" : "标记为完成"}
          onclick={(e) => {
            e.stopPropagation();
            void toggleTask(task.id);
          }}
        >
          <span class="tick"></span>
        </button>

        <div class="text">
          <span class="name" class:done={task.done}>{task.name}</span>
          <span class="meta">{meta(task)}</span>
        </div>

        <div class="pips">
          {#each [0, 1, 2] as i (i)}
            <span class="pip" class:on={i < Math.min(task.spent, 3)}></span>
          {/each}
        </div>
      </div>
    {/each}
  </div>

  <button class="add" type="button" onclick={onAdd}>＋ 加一件事（⌘N）</button>

  <div class="body-stats">
    <span class="label">身体这边的账</span>
    {#each BODY_STATS as stat (stat.name)}
      <StatBar name={stat.name} value={stat.value} pct={stat.pct} color={stat.color} />
    {/each}
  </div>
</aside>

<style>
  .sidebar {
    width: 372px;
    flex: none;
    border-left: 1px solid oklch(0.91 0.008 70);
    padding: 26px 24px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .title {
    font-size: 14px;
    font-weight: 600;
  }
  .count {
    font-size: 12.5px;
    color: var(--dim);
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .task {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 11px 13px;
    border: 1px solid var(--line);
    border-radius: var(--radius-control);
    background: var(--card);
    cursor: pointer;
  }
  .task.selected {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .box {
    width: 17px;
    height: 17px;
    flex: none;
    border: 1.5px solid oklch(0.82 0.008 70);
    border-radius: 5px;
    background: var(--card);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
  }
  .box.checked {
    border-color: var(--accent);
    background: var(--accent);
  }
  .tick {
    width: 7px;
    height: 7px;
    border-radius: 1px;
    background: transparent;
  }
  .box.checked .tick {
    background: var(--card);
  }
  .text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .name {
    font-size: 13.5px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .name.done {
    text-decoration: line-through;
    color: oklch(0.62 0.012 60);
  }
  .meta {
    font-size: 11.5px;
    color: var(--faint);
  }
  .pips {
    display: flex;
    gap: 3px;
  }
  .pip {
    width: 7px;
    height: 7px;
    background: var(--track);
  }
  .pip.on {
    background: var(--accent);
  }
  .add {
    padding: 10px 13px;
    border: 1px dashed oklch(0.85 0.008 70);
    border-radius: var(--radius-control);
    background: transparent;
    font-size: 13px;
    color: var(--dim);
    cursor: pointer;
    text-align: left;
  }
  .add:hover {
    background: oklch(0.97 0.006 70);
  }
  .body-stats {
    margin-top: auto;
    padding-top: 16px;
    border-top: 1px solid var(--line-soft);
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .label {
    font-size: 11px;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--faint);
  }
</style>
```

- [ ] **Step 2: Verify**

Run: `npm run check && npm test`
Expected: both pass.

- [ ] **Step 3: Commit**

```bash
git add src/routes/main/TaskSidebar.svelte
git commit -m "feat: build the focus tab task sidebar"
```

---

### Task 8: End-to-end verification

**Files:** none — this task only verifies.

- [ ] **Step 1: Launch**

Run: `npm run tauri dev`

- [ ] **Step 2: Check the layout against artboard 01**

Expected, side by side with the design file:
- Title bar 46px tall, three tabs, 专注 active, right side reads `连续 12 天 | ⌘,`
- Pet bobs inside a slowly pulsing accent ring, blurred ellipse shadow underneath
- Ten belly cells, all empty at 25:00
- Countdown in IBM Plex Mono at 78px with tight negative tracking
- Speech bubble with a squared bottom-left corner
- Sidebar exactly 372px, five seeded tasks, the last two struck through
- Three body-stat bars pinned to the bottom

- [ ] **Step 3: Check behaviour**

1. Click 开始专注 → label becomes 让它歇会儿, countdown starts, belly cells fill roughly every
   2.5 minutes, `预计 HH:MM 结束` stays consistent.
2. Click 让它歇会儿 → countdown freezes.
3. Click 跳过 → phase becomes 休息中, clock resets to 05:00, belly empties.
4. Tick a task checkbox → it strikes through and the `n / 5 完成` counter moves.
5. Click a task row → its border turns accent and the status pill's task name changes.
6. Click ＋ 加一件事 and enter a name → the task appears immediately.
7. Quit and relaunch → the timer resumes where it stopped and the new task is still there.

- [ ] **Step 4: Check the accent and tone react**

In the devtools console:

```js
const { invoke } = await import("@tauri-apps/api/core");
await invoke("set_accent", { accent: "blue" });
await invoke("set_tone", { tone: "professional" });
```

Expected: every accent-coloured element recolours immediately, and the speech bubble
switches to `本轮剩余 N 分钟。`.

- [ ] **Step 5: Run the full gate**

Run:

```bash
npm test && npm run check && npm run build && (cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test)
```
Expected: everything passes.

- [ ] **Step 6: Commit**

```bash
git commit --allow-empty -m "test: verify the focus tab end to end"
```

---

## Definition of Done

- The 专注 tab matches artboard 01 at 1180px width.
- Start / pause / skip all work and survive a restart.
- Belly cells, round pips, status pill and `预计 … 结束` all track the Rust timer.
- Completing a focus phase increments the active task's pip count.
- Changing accent or tone updates the UI live with no reload.
- Only `src/lib/ipc.ts` imports `@tauri-apps/api`.
- The full test gate passes.
