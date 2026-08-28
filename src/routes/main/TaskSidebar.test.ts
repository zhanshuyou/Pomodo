import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { app } from "../../lib/state.svelte";
import TaskSidebar from "./TaskSidebar.svelte";

const ipc = vi.hoisted(() => ({
  deleteTask: vi.fn(async () => {}),
  renameTask: vi.fn(async () => {}),
  reorderTasks: vi.fn(async () => {}),
  setTaskEstimate: vi.fn(async () => {}),
  setActiveTask: vi.fn(async (_id: number | null) => {}),
}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  ...ipc,
}));

const key = (k: string) => new KeyboardEvent("keydown", { key: k, bubbles: true });

describe("TaskSidebar task management", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    for (const fn of Object.values(ipc)) fn.mockClear();
    app.model.tasks = [
      { id: 0, name: "写产品需求文档", estimate: 3, spent: 3, done: false },
      { id: 1, name: "回 Sarah 的邮件", estimate: 1, spent: 0, done: false },
      { id: 2, name: "整理用研笔记", estimate: 2, spent: 1, done: false },
    ];
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(TaskSidebar, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.model.tasks = [];
  });

  const row = (i: number) => host.querySelectorAll<HTMLElement>(".task")[i];

  it("shows a tone-aware empty state that opens the editor", () => {
    app.model.tasks = [];
    flushSync();
    const empty = host.querySelector<HTMLButtonElement>(".empty")!;
    expect(empty.textContent).toContain("⌘N");
    empty.click();
    flushSync();
    expect(host.querySelector(".add-input")).not.toBeNull();
    expect(host.querySelector(".empty")).toBeNull();
    host.querySelector<HTMLInputElement>(".add-input")!.dispatchEvent(key("Escape"));
    flushSync();
  });

  it("lets go of the active task when it is picked again", () => {
    ipc.setActiveTask.mockClear();
    app.model.timer.activeTask = null;
    flushSync();
    const rows = () => [...host.querySelectorAll<HTMLElement>(".task")];
    rows()[0].click();
    expect(ipc.setActiveTask).toHaveBeenLastCalledWith(app.model.tasks[0].id);
    app.model.timer.activeTask = app.model.tasks[0].id;
    flushSync();
    rows()[0].click();
    expect(ipc.setActiveTask).toHaveBeenLastCalledWith(null);
    app.model.timer.activeTask = null;
  });

  it("draws one pip per estimated pomodoro, filled up to spent", () => {
    const pips = (i: number) =>
      [...row(i).querySelectorAll(".pip:not(.ghost)")].map((p) => p.classList.contains("on"));
    expect(pips(0)).toEqual([true, true, true]);
    expect(pips(1)).toEqual([false]);
    expect(pips(2)).toEqual([true, false]);
  });

  it("re-estimates when a pip is clicked, without selecting the row", () => {
    row(1).querySelectorAll<HTMLButtonElement>(".pip")[1].click();
    expect(ipc.setTaskEstimate).toHaveBeenCalledWith(1, 2);
  });

  it("deletes from the hover control", () => {
    row(2).querySelector<HTMLButtonElement>(".act.del")!.click();
    expect(ipc.deleteTask).toHaveBeenCalledWith(2);
  });

  it("moves a row up or down by sending the full new order", () => {
    row(1).querySelector<HTMLButtonElement>('[aria-label="上移"]')!.click();
    expect(ipc.reorderTasks).toHaveBeenLastCalledWith([1, 0, 2]);
    row(1).querySelector<HTMLButtonElement>('[aria-label="下移"]')!.click();
    expect(ipc.reorderTasks).toHaveBeenLastCalledWith([0, 2, 1]);
    expect(row(0).querySelector<HTMLButtonElement>('[aria-label="上移"]')!.disabled).toBe(true);
  });

  it("renames on double-click, Enter submits and Escape cancels", () => {
    row(0).querySelector<HTMLElement>(".name")!.dispatchEvent(
      new MouseEvent("dblclick", { bubbles: true }),
    );
    flushSync();
    const input = row(0).querySelector<HTMLInputElement>(".rename")!;
    expect(document.activeElement).toBe(input);
    input.value = " 写 PRD ";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(key("Enter"));
    flushSync();
    expect(ipc.renameTask).toHaveBeenCalledWith(0, "写 PRD");
    expect(row(0).querySelector(".rename")).toBeNull();

    row(1).querySelector<HTMLElement>(".name")!.dispatchEvent(
      new MouseEvent("dblclick", { bubbles: true }),
    );
    flushSync();
    row(1).querySelector<HTMLInputElement>(".rename")!.dispatchEvent(key("Escape"));
    flushSync();
    expect(ipc.renameTask).toHaveBeenCalledTimes(1);
    expect(row(1).querySelector(".rename")).toBeNull();
  });
});
