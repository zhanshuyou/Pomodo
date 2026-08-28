import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Reminder, SoundSetting } from "../../lib/ipc";
import { app } from "../../lib/state.svelte";
import SoundPane from "./SoundPane.svelte";

const ipc = vi.hoisted(() => ({
  setAllSounds: vi.fn(async (_s: import("../../lib/ipc").SoundSetting) => {}),
  setPhaseSound: vi.fn(
    async (_w: import("../../lib/ipc").PhaseEnd, _s: import("../../lib/ipc").SoundSetting) => {},
  ),
  previewSound: vi.fn(async (_s: import("../../lib/ipc").SoundSetting) => {}),
}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  ...ipc,
}));

function reminder(id: number, sound: SoundSetting): Reminder {
  return {
    id,
    builtin: null,
    name: `r${id}`,
    color: "oklch(0.63 0.13 40)",
    detail: "",
    message: "x",
    hint: "",
    messageEdited: true,
    schedule: { kind: "every", minutes: 30 },
    intensity: "bubble",
    enabled: true,
    rules: {
      activeFromMin: 570,
      activeToMin: 1110,
      weekdays: [true, true, true, true, true, false, false],
      duringFocus: "defer",
      silenceInMeeting: true,
      escalateAfter: 3,
      sound,
      mustComplete: false,
    },
    remainingSecs: 1800,
    consecutiveIgnores: 0,
    deferred: false,
    lastDailyFire: null,
    durationSecs: 60,
  };
}

function row(host: HTMLElement, label: string): HTMLElement {
  const el = host.querySelector<HTMLElement>(`[role="group"][aria-label="${label}"]`);
  if (!el) throw new Error(`no row ${label}`);
  return el;
}

function tones(el: HTMLElement): string[] {
  return [...el.querySelectorAll<HTMLButtonElement>(".tone[aria-pressed]")].map(
    (b) => b.textContent?.trim() ?? "",
  );
}

function pressed(el: HTMLElement): string[] {
  return [...el.querySelectorAll<HTMLButtonElement>('.tone[aria-pressed="true"]')].map(
    (b) => b.textContent?.trim() ?? "",
  );
}

describe("SoundPane", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  function render() {
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(SoundPane, { target: host });
    flushSync();
  }

  beforeEach(() => {
    ipc.setAllSounds.mockClear();
    ipc.setPhaseSound.mockClear();
    ipc.previewSound.mockClear();
    app.model.settings.phaseSounds = {
      focusEnd: { tone: "chime", volume: 40 },
      breakEnd: { tone: "woodblock", volume: 30 },
    };
    app.model.reminders = [
      reminder(0, { tone: "woodblock", volume: 30 }),
      reminder(1, { tone: "woodblock", volume: 30 }),
    ];
    render();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.model.reminders = [];
  });

  it("shows the two phase-end rows with their current tones", () => {
    expect(pressed(row(host, "专注结束"))).toEqual(["风铃"]);
    expect(pressed(row(host, "休息结束"))).toEqual(["木鱼"]);
    expect(tones(row(host, "专注结束"))).toEqual(["无", "木鱼", "风铃", "滴"]);
  });

  it("changes a phase-end sound and previews it", () => {
    [...row(host, "休息结束").querySelectorAll<HTMLButtonElement>(".tone")]
      .find((b) => b.textContent?.trim() === "滴")
      ?.click();
    expect(ipc.setPhaseSound).toHaveBeenCalledWith("breakEnd", { tone: "beep", volume: 30 });
    expect(ipc.previewSound).toHaveBeenCalledWith({ tone: "beep", volume: 30 });
    expect(ipc.setAllSounds).not.toHaveBeenCalled();
  });

  it("commits a phase-end volume on change", () => {
    const slider = row(host, "专注结束").querySelector<HTMLInputElement>('input[type="range"]');
    if (!slider) throw new Error("no slider");
    slider.value = "65";
    slider.dispatchEvent(new Event("change", { bubbles: true }));
    expect(ipc.setPhaseSound).toHaveBeenCalledWith("focusEnd", { tone: "chime", volume: 65 });
  });

  it("shows the shared reminder setting", () => {
    expect(pressed(row(host, "所有提醒"))).toEqual(["木鱼"]);
    expect(row(host, "所有提醒").querySelector(".pct")?.textContent).toBe("30%");
  });

  it("applies a tone to every reminder and previews it", () => {
    [...row(host, "所有提醒").querySelectorAll<HTMLButtonElement>(".tone")]
      .find((b) => b.textContent?.trim() === "滴")
      ?.click();
    expect(ipc.setAllSounds).toHaveBeenCalledWith({ tone: "beep", volume: 30 });
    expect(ipc.previewSound).toHaveBeenCalledWith({ tone: "beep", volume: 30 });
  });

  it("flags when reminders disagree", () => {
    app.model.reminders[1].rules.sound = { tone: "chime", volume: 30 };
    flushSync();
    expect(row(host, "所有提醒").textContent).toContain("部分提醒不同");
  });

  it("says so instead of inventing a setting when there are no reminders", () => {
    void unmount(component);
    host.remove();
    app.model.reminders = [];
    render();
    expect(host.querySelector('[role="group"][aria-label="所有提醒"]')).toBeNull();
    expect(host.querySelector(".empty")?.textContent).toContain("还没有提醒");
  });
});
