import type { UnlistenFn } from "@tauri-apps/api/event";

import {
  IS_TAURI,
  type Model,
  type StatsSummary,
  listModel,
  onChanged,
  onPetState,
  onPhase,
  onTick,
  statsSummary,
} from "./ipc";
import { ACCENTS, DEFAULT_ACCENT, DEFAULT_TONE, type Tone, bellyCellsFor } from "./theme";

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
    petVisible: true,
    phaseSounds: {
      focusEnd: { tone: "chime", volume: 40 },
      breakEnd: { tone: "woodblock", volume: 30 },
    },
  },
  pet: {
    selected: 0,
    lifetimePomodoros: 0,
    custom: { focus: null, rest: null, nag: null },
    useCustom: false,
    level: 1,
    stage: "幼崽期",
    toNextLevel: 13,
    progressPct: 0,
    unlockAt: [0, 0, 0, 0, 150, 300],
  },
  reminders: [],
  body: {
    waterCups: 0,
    waterGoal: 8,
    stands: 0,
    standGoal: 6,
    longestSitMins: 0,
    sitGoalMins: 90,
    day: "",
  },
  deepWork: false,
  miniEnabled: false,
  petMood: "focus",
};

class AppStore {
  model = $state<Model>(FALLBACK);
  ready = $state(false);
  bellyCells = $state(0);
  /** From the last tick — the OS is in 省电模式. */
  lowPower = $state(false);
  /** document.visibilityState; a hidden or fully occluded window reports false. */
  pageVisible = $state(true);
  summary = $state<StatsSummary | null>(null);

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
  get pet() {
    return this.model.pet;
  }
  get reminders() {
    return this.model.reminders;
  }
  get body() {
    return this.model.body;
  }
  get deepWork() {
    return this.model.deepWork;
  }
  get miniEnabled() {
    return this.model.miniEnabled;
  }
  get petMood() {
    return this.model.petMood;
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

    this.#watchVisibility();

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
        this.lowPower = p.lowPower;
      }),
      // A phase boundary can credit a task and change settings-derived durations,
      // so refetch rather than trying to patch every dependent field by hand.
      await onPhase(() => void this.refresh()),
      await onChanged(() => void this.refresh()),
      await onPetState((p) => {
        this.model.petMood = p.state;
      }),
    );

    this.ready = true;
  }

  async refresh(): Promise<void> {
    this.model = await listModel();
    this.bellyCells = bellyCellsFor(this.phaseDurationSecs(), this.model.timer.remainingSecs);
    await this.refreshStats();
  }

  /** settings.duration_for(timer.phase). */
  phaseDurationSecs(): number {
    const s = this.model.settings;
    switch (this.model.timer.phase) {
      case "focus":
        return s.focusSecs;
      case "shortBreak":
        return s.shortBreakSecs;
      case "longBreak":
        return s.longBreakSecs;
    }
  }

  async refreshStats(): Promise<void> {
    if (!IS_TAURI) return;
    this.summary = await statsSummary();
  }

  /**
   * An always-on-top pet that keeps bobbing behind a fullscreen app, or on a
   * battery-saving laptop, is wasted work. `still` is what the pet components
   * consult; it holds the sprite in place without touching the timer.
   */
  get still(): boolean {
    return !this.pageVisible || this.lowPower;
  }

  #onVisibility = () => {
    this.pageVisible = document.visibilityState !== "hidden";
  };

  #watchVisibility(): void {
    if (typeof document === "undefined") return;
    this.#onVisibility();
    document.addEventListener("visibilitychange", this.#onVisibility);
    this.#unlisteners.push(() =>
      document.removeEventListener("visibilitychange", this.#onVisibility),
    );
  }

  dispose(): void {
    for (const un of this.#unlisteners) un();
    this.#unlisteners = [];
    this.#started = false;
  }
}

export const app = new AppStore();
