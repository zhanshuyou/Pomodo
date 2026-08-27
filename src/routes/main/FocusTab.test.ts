import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { app } from "../../lib/state.svelte";
import FocusTab from "./FocusTab.svelte";

/**
 * Mounts the real component tree in jsdom. Outside Tauri the store keeps its
 * fallback model, so this exercises rendering and derived state without any IPC.
 * jsdom has no canvas backend; PetCanvas guards on a null context, and this test
 * would fail if that guard ever regressed.
 */
describe("FocusTab", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(FocusTab, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  it("renders the countdown from the store", () => {
    expect(host.querySelector(".mmss")?.textContent).toBe("25:00");
  });

  it("renders ten belly cells, none filled at a full phase", () => {
    const cells = host.querySelectorAll(".cell");
    expect(cells).toHaveLength(10);
    expect(host.querySelectorAll(".cell.filled")).toHaveLength(0);
  });

  it("fills belly cells as the tick payload advances", () => {
    app.bellyCells = 7;
    flushSync();
    expect(host.querySelectorAll(".cell.filled")).toHaveLength(7);
  });

  it("renders one round pip per configured round", () => {
    expect(host.querySelectorAll(".pip")).toHaveLength(
      app.settings.roundsPerCycle,
    );
  });

  it("shows the paused primary label and both secondary buttons", () => {
    const labels = [...host.querySelectorAll("button")].map((b) =>
      b.textContent?.trim(),
    );
    expect(labels).toContain("开始专注");
    expect(labels).toContain("跳过");
    expect(labels).toContain("迷你模式");
  });

  it("labels the mini button from the real mode, not a local toggle", () => {
    const button = [...host.querySelectorAll("button")].find((b) =>
      b.textContent?.trim().endsWith("迷你模式"),
    );
    expect(button?.textContent?.trim()).toBe("迷你模式");

    app.model.miniEnabled = true;
    flushSync();
    expect(button?.textContent?.trim()).toBe("退出迷你模式");
    app.model.miniEnabled = false;
  });

  it("renders the status pill with phase, round and total", () => {
    const status = host.querySelector(".status")?.textContent?.replace(/\s+/g, " ");
    expect(status).toContain("专注中");
    expect(status).toContain("第 1/4 轮");
  });

  it("renders the pet speech bubble and the ends-at line", () => {
    // A full, never-started focus round: the pet asks what is up, not "还有 25 分钟".
    expect(host.querySelector(".bubble")?.textContent?.trim()).toBe(
      "今天要啃点什么？我准备好了",
    );
    // Paused: a clock-time ETA would go stale, so it is not shown at all.
    expect(host.querySelector(".ends")?.textContent).toBe("已暂停");
    app.model.timer.running = true;
    flushSync();
    expect(host.querySelector(".ends")?.textContent).toMatch(
      /^预计 \d{2}:\d{2} 结束$/,
    );
    app.model.timer.running = false;
  });

  it("renders a canvas for the pet", () => {
    const canvas = host.querySelector("canvas");
    expect(canvas).not.toBeNull();
    expect(canvas?.width).toBe(128); // 16 x scale 8
  });

  it("renders the task sidebar shell", () => {
    expect(host.querySelector(".sidebar")).not.toBeNull();
    expect(host.querySelector(".add")?.textContent?.trim()).toBe(
      "＋ 加一件事（⌘N）",
    );
    expect(host.querySelector(".label")?.textContent).toBe("身体这边的账");
  });
});
