import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { StatsSummary } from "../../lib/ipc";
import { app } from "../../lib/state.svelte";
import StatsTab from "./StatsTab.svelte";

const addQuietWindow = vi.hoisted(() => vi.fn(async (_from: number, _to: number) => 1));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  addQuietWindow,
}));

function summary(): StatsSummary {
  return {
    weekFocusSecs: 3600,
    weekDeltaPct: 12,
    pomodoros: 4,
    dailyAverage: 1,
    interruptions: 2,
    interruptionsDelta: -1,
    streak: 1,
    bestStreak: 2,
    bars: [],
    interruptionHotspot: { startHour: 15, endHour: 16, interruptions: 3, total: 5 },
  };
}

describe("StatsTab", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    addQuietWindow.mockClear();
    app.summary = summary();
    app.model.quietHours = [];
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(StatsTab, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.summary = null;
    app.model.quietHours = [];
  });

  it("turns the interruption hotspot into a quiet window on click", () => {
    const button = host.querySelector<HTMLButtonElement>("button.iact");
    expect(button?.textContent?.trim()).toBe("设为安静时段");
    button?.click();
    expect(addQuietWindow).toHaveBeenCalledWith(15 * 60, 16 * 60);
  });

  it("says so once a quiet window already covers the hotspot", () => {
    app.model.quietHours = [{ id: 0, fromMin: 14 * 60, toMin: 17 * 60 }];
    flushSync();
    expect(host.querySelector("button.iact")).toBeNull();
    expect(host.querySelector(".iact.done")?.textContent).toContain("已设为安静时段 15:00–16:00");
  });
});
