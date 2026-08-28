import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

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
  petVisible: boolean;
  phaseSounds: PhaseSounds;
}

/** What a finished focus / break rings with; see `setPhaseSound`. */
export interface PhaseSounds {
  focusEnd: SoundSetting;
  breakEnd: SoundSetting;
}
export type PhaseEnd = keyof PhaseSounds;

/**
 * The subset of Rust's `Model` the views read. Rust serialises more (id
 * counters, raw sessions, window placements); anything not listed here is
 * simply ignored on the way in.
 */
export interface Model {
  timer: Timer;
  tasks: Task[];
  settings: Settings;
  pet: PetState;
  reminders: Reminder[];
  body: BodyCounters;
  deepWork: boolean;
  quietHours: QuietWindow[];
  miniEnabled: boolean;
  /** Derived in Rust from timer / nudge / idle time; see `onPetState`. */
  petMood: PetMood;
}

export type PetMood = "focus" | "break" | "nagging" | "sleeping";

export interface PetStatePayload {
  state: PetMood;
}

export interface TickPayload {
  remainingSecs: number;
  phase: Phase;
  running: boolean;
  round: number;
  bellyCells: number;
  /** 省电模式 is on; see AppStore.still. */
  lowPower: boolean;
}

export interface PhaseChange {
  from: Phase;
  to: Phase;
  round: number;
  completed: boolean;
}

export interface ChangedPayload {
  section: "tasks" | "settings" | "timer" | "reminders" | "body";
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
export const renameTask = (id: number, name: string) =>
  invoke<void>("rename_task", { id, name });
export const setTaskEstimate = (id: number, estimate: number) =>
  invoke<void>("set_task_estimate", { id, estimate });
export const reorderTasks = (ids: number[]) =>
  invoke<void>("reorder_tasks", { ids });
export const setAccent = (accent: Accent) => invoke<void>("set_accent", { accent });
export const setTone = (tone: Tone) => invoke<void>("set_tone", { tone });
export const setPetFlag = (flag: keyof PetFlags, value: boolean) =>
  invoke<void>("set_pet_flag", { flag, value });
export const setTimerDurations = (durations: {
  focusSecs: number;
  shortBreakSecs: number;
  longBreakSecs: number;
  roundsPerCycle: number;
}) => invoke<void>("set_timer_durations", durations);

/**
 * Every window subscribes in onMount, including the ones rendered outside Tauri
 * — vitest and gallery.html. There is no event bridge there, so subscribing has
 * to be a no-op handing back a no-op unlisten; otherwise each mount leaves a
 * rejection nobody is positioned to catch.
 */
function subscribe<T>(event: string, cb: (p: T) => void): Promise<UnlistenFn> {
  if (!IS_TAURI) return Promise.resolve(() => {});
  return listen<T>(event, (e) => cb(e.payload));
}

export const onTick = (cb: (p: TickPayload) => void): Promise<UnlistenFn> =>
  subscribe<TickPayload>("timer:tick", cb);
export const onPhase = (cb: (p: PhaseChange) => void): Promise<UnlistenFn> =>
  subscribe<PhaseChange>("timer:phase", cb);
export const onChanged = (cb: (p: ChangedPayload) => void): Promise<UnlistenFn> =>
  subscribe<ChangedPayload>("model:changed", cb);

export interface DayBar {
  label: string;
  count: number;
}

export interface InterruptionHotspot {
  startHour: number;
  endHour: number;
  interruptions: number;
  total: number;
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
  interruptionHotspot: InterruptionHotspot | null;
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
  /** Derived in Rust from lifetimePomodoros — see core/pet.rs. */
  level: number;
  stage: string;
  toNextLevel: number;
  progressPct: number;
  unlockAt: number[];
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

/** Ask the user for a sprite file. Returns null when they cancel. */
export async function pickPetImage(): Promise<string | null> {
  const picked = await openDialog({
    multiple: false,
    directory: false,
    filters: [{ name: "宠物图片", extensions: ["png", "gif", "apng", "webp"] }],
  });
  return typeof picked === "string" ? picked : null;
}

export const PET_IMAGE_EXTENSIONS = ["png", "gif", "apng", "webp"] as const;

export type FileDropEvent =
  | { type: "enter" | "drop"; paths: string[] }
  | { type: "over" | "leave" };

/**
 * OS file drag-and-drop into this window. Tauri delivers it as a webview
 * event rather than the DOM's dragover/drop, which wry swallows. No-op
 * outside Tauri, like every other subscription.
 */
export async function onFileDrop(cb: (e: FileDropEvent) => void): Promise<UnlistenFn> {
  if (!IS_TAURI) return () => {};
  const { getCurrentWebview } = await import("@tauri-apps/api/webview");
  return getCurrentWebview().onDragDropEvent((event) => {
    const p = event.payload;
    if (p.type === "enter" || p.type === "drop") cb({ type: p.type, paths: p.paths });
    else cb({ type: p.type });
  });
}

/** Convert a stored absolute path into something an <img src> can load. */
import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * asset:// URL for a file in the pets directory. Outside Tauri there is no
 * asset protocol (and convertFileSrc throws), so the raw path is handed back
 * and jsdom / gallery.html get a plain broken <img>.
 */
export const petImageSrc = (path: string): string =>
  IS_TAURI ? convertFileSrc(path) : path;

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
  sound: SoundSetting;
  mustComplete: boolean;
}

export type SoundTone = "none" | "woodblock" | "chime" | "beep";
export interface SoundSetting {
  tone: SoundTone;
  /** 0–100 */
  volume: number;
}
export const SOUND_TONES: { key: SoundTone; label: string }[] = [
  { key: "none", label: "无" },
  { key: "woodblock", label: "木鱼" },
  { key: "chime", label: "风铃" },
  { key: "beep", label: "滴" },
];
/** The rule row's text, mirroring SoundSetting::label in Rust. */
export function soundLabel(s: SoundSetting): string {
  if (s.tone === "none") return "无";
  const name = SOUND_TONES.find((t) => t.key === s.tone)?.label ?? s.tone;
  return `${name} · ${s.volume}%`;
}
export const previewSound = (sound: SoundSetting): Promise<void> =>
  IS_TAURI ? invoke<void>("preview_sound", { sound }) : Promise.resolve();
export const setAllSounds = (sound: SoundSetting) =>
  invoke<void>("set_all_sounds", { sound });
/** 身体这边的账 — the bars' denominators. Clamped in Rust; see BodyCounters::set_goals. */
export const setBodyGoals = (goals: {
  waterGoal: number;
  standGoal: number;
  sitGoalMins: number;
}) => invoke<void>("set_body_goals", goals);
/**
 * Braces a reminder message may carry; Rust fills them from the body
 * counters at fire time (reminder_copy::fill). Mirrored here for the editor's
 * caption only.
 */
export const MESSAGE_PLACEHOLDERS: { key: string; label: string }[] = [
  { key: "{cups}", label: "今日已喝杯数" },
  { key: "{goal}", label: "每日目标杯数" },
  { key: "{next}", label: "下一杯是第几杯" },
  { key: "{stands}", label: "今日站起次数" },
  { key: "{standGoal}", label: "每日站起目标" },
];
export const setPhaseSound = (which: PhaseEnd, sound: SoundSetting) =>
  invoke<void>("set_phase_sound", { which, sound });

export interface Reminder {
  id: number;
  builtin: "stand" | "water" | "eyes" | "review" | null;
  name: string;
  color: string;
  detail: string;
  /** The authored third clause of `detail`; the rest is derived. */
  note: string;
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
  durationSecs: number;
}

export interface BodyCounters {
  waterCups: number;
  waterGoal: number;
  stands: number;
  standGoal: number;
  longestSitMins: number;
  sitGoalMins: number;
  day: string;
}

export interface FirePayload {
  id: number;
  name: string;
  message: string;
  intensity: Intensity;
  color: string;
  /** 必须完成: the fullscreen overlay offers no ⎋ and no 稍后. */
  mustComplete?: boolean;
  /** The overlay's countdown; absent only in tests that predate it. */
  durationSecs?: number;
}

/**
 * 安静时段: inside it 直接打断 becomes 推迟到本轮结束 and nothing rings louder
 * than 宠物提示. Minutes past midnight; `toMin` exclusive, wraps midnight.
 */
export interface QuietWindow {
  id: number;
  fromMin: number;
  toMin: number;
}
export const addQuietWindow = (fromMin: number, toMin: number) =>
  invoke<number>("add_quiet_window", { fromMin, toMin });
export const deleteQuietWindow = (id: number) =>
  invoke<void>("delete_quiet_window", { id });

export interface ReminderPatch {
  name?: string;
  color?: string;
  /** The detail line's third clause. */
  note?: string;
  message?: string;
  /** Shorthand for `schedule: { kind: "every", minutes }`. */
  intervalMinutes?: number;
  schedule?: Schedule;
  intensity?: Intensity;
  durationSecs?: number;
  enabled?: boolean;
  rules?: Rules;
}

/** `color` paints a non-builtin template; builtin templates keep their own. */
export const addReminder = (template: string | null, color: string | null = null) =>
  invoke<number>("add_reminder", { template, color });
export const updateReminder = (id: number, patch: ReminderPatch) =>
  invoke<void>("update_reminder", { id, patch });
export const toggleReminder = (id: number) => invoke<void>("toggle_reminder", { id });
export const deleteReminder = (id: number) => invoke<void>("delete_reminder", { id });
export const ackReminder = (id: number) => invoke<void>("ack_reminder", { id });
export const ignoreReminder = (id: number) => invoke<void>("ignore_reminder", { id });
/** Mirrors core::reminder::SNOOZE_MINUTES. */
export const SNOOZE_MINUTES = 10;
export const snoozeReminder = (id: number, minutes: number = SNOOZE_MINUTES) =>
  invoke<void>("snooze_reminder", { id, minutes });
export const snoozeOverlay = (id: number, minutes: number = SNOOZE_MINUTES) =>
  invoke<void>("snooze_overlay", { id, minutes });
export const setDeepWork = (value: boolean) => invoke<void>("set_deep_work", { value });
export const openPrefs = () => invoke<void>("open_prefs");


export interface UpNextItem {
  id: number;
  name: string;
  color: string;
  due: string;
}

export interface TodaySummary {
  pomodoros: number;
  focusSecs: number;
  label: string;
}

export const upNext = () => invoke<UpNextItem[]>("up_next");
export const todaySummary = () => invoke<TodaySummary>("today_summary");
export const showMain = () => invoke<void>("show_main");

export const setMiniMode = (value: boolean) =>
  invoke<void>("set_mini_mode", { value });
export const toggleMiniMode = () => invoke<void>("toggle_mini_mode");
/**
 * A window that lays out variable content asks to be exactly as tall as it
 * rendered — Rust cannot measure the webview. No-op outside Tauri.
 */
export const setWindowHeight = (label: "tray" | "bubble", height: number): Promise<void> =>
  IS_TAURI ? invoke<void>("set_window_height", { label, height }) : Promise.resolve();
export const setMiniHeight = (height: number) =>
  invoke<void>("set_mini_height", { height });
export const setMiniPlacement = (x: number, y: number) =>
  invoke<void>("set_mini_placement", { x, y });

export const setPetPlacement = (x: number, y: number) =>
  invoke<void>("set_pet_placement", { x, y });
export const hidePet = () => invoke<void>("hide_pet");
export const hideBubble = () => invoke<void>("hide_bubble");
export const setPetVisible = (value: boolean) =>
  invoke<void>("set_pet_visible", { value });
export const dismissOverlay = (id: number, acknowledged: boolean) =>
  invoke<void>("dismiss_overlay", { id, acknowledged });

export const onPetNudge = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  subscribe<FirePayload>("pet:nudge", cb);
export const onPetState = (
  cb: (p: PetStatePayload) => void,
): Promise<UnlistenFn> => subscribe<PetStatePayload>("pet:state", cb);
export interface HitRect {
  x: number;
  y: number;
  width: number;
  height: number;
}
/**
 * Where the pet window is clickable; everywhere else lets the mouse through.
 * Fired from a layout effect on every mount, so outside Tauri (vitest, the
 * gallery) it has to be a no-op rather than an unhandled rejection.
 */
export const setPetHitRects = (rects: HitRect[]): Promise<void> =>
  IS_TAURI ? invoke<void>("set_pet_hit_rects", { rects }) : Promise.resolve();
export const setPetDragging = (dragging: boolean): Promise<void> =>
  IS_TAURI ? invoke<void>("set_pet_dragging", { dragging }) : Promise.resolve();
/**
 * 开机自动启动 lives in the OS (a LaunchAgent), not in state.json, so it is
 * read back from the plugin rather than mirrored in Settings.
 */
export async function autostartEnabled(): Promise<boolean> {
  if (!IS_TAURI) return false;
  const { isEnabled } = await import("@tauri-apps/plugin-autostart");
  return isEnabled();
}
export async function setAutostart(value: boolean): Promise<void> {
  if (!IS_TAURI) return;
  const { enable, disable } = await import("@tauri-apps/plugin-autostart");
  await (value ? enable() : disable());
}
/** The user poked the pet — wakes it from 睡眠动画. */
export const petInteracted = () => invoke<void>("pet_interacted");
export const onMiniNudge = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  subscribe<FirePayload>("mini:nudge", cb);
export const onBubbleShow = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  subscribe<FirePayload>("bubble:show", cb);
export const onOverlayShow = (cb: (p: FirePayload) => void): Promise<UnlistenFn> =>
  subscribe<FirePayload>("overlay:show", cb);
