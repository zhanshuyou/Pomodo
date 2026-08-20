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
