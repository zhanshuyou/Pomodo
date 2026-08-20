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
