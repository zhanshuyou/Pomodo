import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { FirePayload } from "../../lib/ipc";
import { app } from "../../lib/state.svelte";
import MiniBar from "./App.svelte";

const ipcSpies = vi.hoisted(() => ({
  ackReminder: vi.fn(async () => {}),
  ignoreReminder: vi.fn(async () => {}),
}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  ...ipcSpies,
}));

const FOCUS_TASK = {
  id: 0,
  name: "写产品需求文档",
  estimate: 3,
  spent: 0,
  done: false,
};

/** The bar exports receiveNudge so a test can fire a reminder at it directly. */
type MiniInstance = { receiveNudge: (payload: FirePayload) => void };

function render() {
  const host = document.createElement("div");
  document.body.appendChild(host);
  const component = mount(MiniBar, { target: host }) as unknown as MiniInstance;
  flushSync();
  return { host, component };
}

describe("mini bar", () => {
  let host: HTMLDivElement;
  let component: MiniInstance;

  beforeEach(() => {
    vi.useFakeTimers();
    app.model.tasks = [FOCUS_TASK];
    // Half-way through a 25-minute focus phase, the way the artboard shows it.
    app.model.timer = {
      phase: "focus",
      remainingSecs: 750,
      running: true,
      round: 2,
      activeTask: 0,
    };
    ({ host, component } = render());
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    vi.useRealTimers();
    app.model.tasks = [];
    app.model.timer = {
      phase: "focus",
      remainingSecs: 1500,
      running: false,
      round: 1,
      activeTask: null,
    };
  });

  it("renders the countdown and the active task name", () => {
    expect(host.querySelector(".mmss")?.textContent).toBe("12:30");
    expect(host.querySelector(".task")?.textContent).toBe("写产品需求文档");
  });

  it("fills the progress bar by the elapsed fraction of the phase", () => {
    // 1500 - 750 elapsed of 1500 = 50%.
    const fill = host.querySelector(".fill") as HTMLElement;
    expect(fill.style.width).toBe("50%");
  });

  it("draws the pet at scale 2, matching the artboard's 32px sprite", () => {
    const canvas = host.querySelector("canvas") as HTMLCanvasElement;
    expect(canvas.width).toBe(32); // 16 x scale 2
  });

  it("offers pause, skip and back-to-main, hidden until the bar is hovered", () => {
    const labels = [...host.querySelectorAll(".act")].map((b) =>
      b.getAttribute("aria-label"),
    );
    expect(labels).toEqual(["暂停", "跳过", "回主窗口"]);
    expect(host.querySelector(".acts")?.classList.contains("shown")).toBe(false);

    host.querySelector(".bar")?.dispatchEvent(new Event("pointerenter"));
    flushSync();
    expect(host.querySelector(".acts")?.classList.contains("shown")).toBe(true);
  });

  it("labels the run button 继续 while the timer is paused", () => {
    app.model.timer = { ...app.model.timer, running: false };
    flushSync();
    expect(host.querySelector(".act")?.getAttribute("aria-label")).toBe("继续");
  });

  it("swells to carry a reminder instead of opening a second window", () => {
    expect(host.querySelector(".nudge")).toBeNull();

    component.receiveNudge({
      id: 1,
      name: "站立",
      message: "已连续坐着 45 分钟，请起身活动 2 分钟。",
      intensity: "pet",
      color: "oklch(0.63 0.13 40)",
    });
    flushSync();

    expect(host.querySelector(".nudge-name")?.textContent).toBe("站立");
    expect(host.querySelector(".nudge-text")?.textContent).toBe(
      "已连续坐着 45 分钟，请起身活动 2 分钟。",
    );
  });

  it("collapses again once the reminder has been on screen long enough", () => {
    component.receiveNudge({
      id: 1,
      name: "站立",
      message: "起来动动",
      intensity: "pet",
      color: "oklch(0.63 0.13 40)",
    });
    flushSync();
    expect(host.querySelector(".nudge")).not.toBeNull();

    vi.advanceTimersByTime(12_000);
    flushSync();
    expect(host.querySelector(".nudge")).toBeNull();
    // Nobody answered, so this one counts toward 连续忽略 N 次.
    expect(ipcSpies.ignoreReminder).toHaveBeenCalledWith(1);
    expect(ipcSpies.ackReminder).not.toHaveBeenCalled();
  });

  it("answering the nudge acknowledges rather than ignores", () => {
    ipcSpies.ackReminder.mockClear();
    ipcSpies.ignoreReminder.mockClear();
    component.receiveNudge({
      id: 2,
      name: "喝水",
      message: "喝口水",
      intensity: "pet",
      color: "oklch(0.66 0.09 195)",
    });
    flushSync();
    host.querySelector<HTMLElement>(".nudge")?.click();
    flushSync();
    vi.advanceTimersByTime(12_000);
    flushSync();
    expect(ipcSpies.ackReminder).toHaveBeenCalledWith(2);
    expect(ipcSpies.ignoreReminder).not.toHaveBeenCalled();
  });
});
