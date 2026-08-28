import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { app } from "../../lib/state.svelte";
import TimerPane from "./TimerPane.svelte";

const setBodyGoals = vi.hoisted(() =>
  vi.fn(async (_g: { waterGoal: number; standGoal: number; sitGoalMins: number }) => {}),
);
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  setBodyGoals,
}));

describe("TimerPane", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    app.model.settings = {
      ...app.model.settings,
      focusSecs: 1500,
      shortBreakSecs: 300,
      longBreakSecs: 900,
      roundsPerCycle: 4,
    };
    app.model.body = { ...app.model.body, waterGoal: 8, standGoal: 6, sitGoalMins: 90 };
    setBodyGoals.mockClear();
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(TimerPane, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  function inputs() {
    return [...host.querySelectorAll("input")] as HTMLInputElement[];
  }

  it("shows the current durations in minutes and rounds as a plain count", () => {
    const values = inputs().map((i) => i.value);
    expect(values.slice(0, 4)).toEqual(["25", "5", "15", "4"]);
  });

  it("follows a settings change made elsewhere while untouched", () => {
    app.model.settings = { ...app.model.settings, focusSecs: 1800 };
    flushSync();
    expect(inputs()[0].value).toBe("30");
  });

  // Committing an edit goes through set_timer_durations, which invokes into
  // Tauri and cannot run under jsdom — so this, like the field's own onchange,
  // stops at the oninput handler that marks the field unsaved. set_timer_durations'
  // own clamping is covered on the Rust side.
  it("does not clobber an unsaved edit when settings change elsewhere before it commits", () => {
    const focus = inputs()[0];
    focus.value = "40";
    focus.dispatchEvent(new Event("input", { bubbles: true }));
    flushSync();
    expect(focus.value).toBe("40"); // sanity: the edit itself took

    // A change from another window lands before this field's onchange fires.
    app.model.settings = { ...app.model.settings, roundsPerCycle: 3 };
    flushSync();

    expect(focus.value).toBe("40");
  });

  it("shows the body goals and commits them together on change", () => {
    const water = host.querySelector<HTMLInputElement>('input[aria-label="每天喝水目标"]');
    const stand = host.querySelector<HTMLInputElement>('input[aria-label="每天站起目标"]');
    const sit = host.querySelector<HTMLInputElement>('input[aria-label="久坐上限"]');
    if (!water || !stand || !sit) throw new Error("missing body goal inputs");
    expect([water.value, stand.value, sit.value]).toEqual(["8", "6", "90"]);

    water.value = "10";
    water.dispatchEvent(new Event("input", { bubbles: true }));
    water.dispatchEvent(new Event("change", { bubbles: true }));
    expect(setBodyGoals).toHaveBeenCalledWith({ waterGoal: 10, standGoal: 6, sitGoalMins: 90 });
  });

  it("follows a body goal change made elsewhere while untouched", () => {
    app.model.body = { ...app.model.body, standGoal: 4 };
    flushSync();
    const stand = host.querySelector<HTMLInputElement>('input[aria-label="每天站起目标"]');
    expect(stand?.value).toBe("4");
  });
});
