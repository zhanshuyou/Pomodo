import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Reminder } from "../../lib/ipc";
import { app } from "../../lib/state.svelte";
import SoundPane from "./SoundPane.svelte";

const ipc = vi.hoisted(() => ({
  setAllSounds: vi.fn(async (_s: import("../../lib/ipc").SoundSetting) => {}),
  previewSound: vi.fn(async (_s: import("../../lib/ipc").SoundSetting) => {}),
}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  ...ipc,
}));

function reminder(id: number, sound: Reminder["rules"]["sound"]): Reminder {
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

describe("SoundPane", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    ipc.setAllSounds.mockClear();
    ipc.previewSound.mockClear();
    app.model.reminders = [
      reminder(0, { tone: "woodblock", volume: 30 }),
      reminder(1, { tone: "woodblock", volume: 30 }),
    ];
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(SoundPane, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.model.reminders = [];
  });

  it("shows the shared setting and the four tones", () => {
    expect(host.querySelector(".val")?.textContent).toBe("木鱼 · 30%");
    expect([...host.querySelectorAll(".tone.on")].map((e) => e.textContent?.trim())).toEqual([
      "木鱼",
    ]);
    expect([...host.querySelectorAll(".tones .tone")].map((e) => e.textContent?.trim())).toEqual(
      ["无", "木鱼", "风铃", "滴"],
    );
  });

  it("applies a tone to every reminder and previews it", () => {
    [...host.querySelectorAll<HTMLButtonElement>(".tones .tone")]
      .find((b) => b.textContent?.trim() === "滴")
      ?.click();
    expect(ipc.setAllSounds).toHaveBeenCalledWith({ tone: "beep", volume: 30 });
    expect(ipc.previewSound).toHaveBeenCalledWith({ tone: "beep", volume: 30 });
  });

  it("flags when reminders disagree", () => {
    app.model.reminders[1].rules.sound = { tone: "chime", volume: 30 };
    flushSync();
    expect(host.querySelector(".val")?.textContent).toContain("部分提醒不同");
  });
});
