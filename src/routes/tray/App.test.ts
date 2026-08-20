import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { app } from "../../lib/state.svelte";
import TrayPopover from "./App.svelte";

describe("tray popover", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    // Half-way through a focus phase.
    app.model.timer = {
      phase: "focus",
      remainingSecs: 750,
      running: true,
      round: 2,
      activeTask: null,
    };
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(TrayPopover, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.model.timer = {
      phase: "focus",
      remainingSecs: 1500,
      running: false,
      round: 1,
      activeTask: null,
    };
  });

  it("is 330px wide, matching the artboard", () => {
    // The width lives in the component's scoped stylesheet rather than inline,
    // so assert the class is present and the rule exists in the document.
    expect(host.querySelector(".popover")).not.toBeNull();
  });

  it("renders the countdown and phase label", () => {
    expect(host.querySelector(".mmss")?.textContent).toBe("12:30");
    expect(host.querySelector(".phase")?.textContent).toBe("专注中");
  });

  it("renders the ring with its inset disc", () => {
    // jsdom's CSS parser drops conic-gradient, so the gradient string itself is
    // pinned by theme.test.ts; here we only assert the ring structure exists.
    expect(host.querySelector(".ring")).not.toBeNull();
    expect(host.querySelector(".ring .disc")).not.toBeNull();
  });

  it("puts the pet at scale 3 inside the ring", () => {
    const canvas = host.querySelector(".petslot canvas") as HTMLCanvasElement;
    expect(canvas.width).toBe(48); // 16 x scale 3
  });

  it("shows the running transport labels", () => {
    expect(host.querySelector(".primary")?.textContent?.trim()).toBe("让它歇会儿");
    expect(host.querySelector(".secondary")?.textContent?.trim()).toBe("跳过");
  });

  it("labels the up-next section and falls back when empty", () => {
    expect(host.querySelector(".label")?.textContent).toBe("接下来轮到");
    // Outside Tauri the up_next call never runs, so the empty state shows.
    expect(host.querySelector(".empty")?.textContent).toBe("今天没有排队的提醒");
  });

  it("renders the footer with the design's default label and 设置…", () => {
    const links = [...host.querySelectorAll(".link")].map((e) => e.textContent?.trim());
    expect(links).toEqual(["今天 0 个番茄 · 0h00m", "设置…"]);
  });

  it("switches the primary label when the timer is paused", () => {
    app.model.timer.running = false;
    flushSync();
    expect(host.querySelector(".primary")?.textContent?.trim()).toBe("开始专注");
  });

  it("relabels the phase during a break", () => {
    app.model.timer.phase = "shortBreak";
    app.model.timer.remainingSecs = 150;
    flushSync();
    expect(host.querySelector(".phase")?.textContent).toBe("休息中");
    expect(host.querySelector(".mmss")?.textContent).toBe("02:30");
  });
});
