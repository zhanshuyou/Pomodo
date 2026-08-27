import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App.svelte";

const addTask = vi.hoisted(() => vi.fn(async () => {}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  addTask,
}));

/**
 * Task creation is an inline editor, not window.prompt(): WKWebView has no
 * delegate for JS prompts, so prompt() returns null there and the old flow
 * was a silent no-op in the packaged app. These tests drive the real editor.
 */
describe("main window add-task flow", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  const editor = () => host.querySelector<HTMLInputElement>(".add-input");
  // Svelte 5 delegates keydown to the root, so the event has to bubble.
  const key = (k: string) =>
    new KeyboardEvent("keydown", { key: k, bubbles: true });
  const pressCmdN = (init: KeyboardEventInit = {}) =>
    window.dispatchEvent(
      new KeyboardEvent("keydown", { code: "KeyN", metaKey: true, ...init }),
    );

  beforeEach(() => {
    addTask.mockClear();
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(App, { target: host });
    flushSync();
  });

  afterEach(() => {
    // Leave the module-level editor closed for the next test.
    editor()?.dispatchEvent(key("Escape"));
    flushSync();
    void unmount(component);
    host.remove();
  });

  it("opens the inline editor on ⌘N and focuses it", () => {
    expect(editor()).toBeNull();
    pressCmdN();
    flushSync();
    expect(editor()).not.toBeNull();
    expect(document.activeElement).toBe(editor());
  });

  it("opens the editor from the dashed row too", () => {
    host.querySelector<HTMLButtonElement>(".add")?.click();
    flushSync();
    expect(editor()).not.toBeNull();
  });

  it("adds the trimmed name on Enter and closes the editor", () => {
    pressCmdN();
    flushSync();
    const input = editor()!;
    input.value = "  写产品需求文档 ";
    input.dispatchEvent(new Event("input"));
    input.dispatchEvent(key("Enter"));
    flushSync();
    expect(addTask).toHaveBeenCalledWith("写产品需求文档", 1);
    expect(editor()).toBeNull();
  });

  it("cancels on Escape without adding", () => {
    pressCmdN();
    flushSync();
    const input = editor()!;
    input.value = "半途而废";
    input.dispatchEvent(new Event("input"));
    input.dispatchEvent(key("Escape"));
    flushSync();
    expect(addTask).not.toHaveBeenCalled();
    expect(editor()).toBeNull();
  });

  it("does not add an empty name", () => {
    pressCmdN();
    flushSync();
    editor()!.dispatchEvent(key("Enter"));
    flushSync();
    expect(addTask).not.toHaveBeenCalled();
    expect(editor()).toBeNull();
  });

  it("does nothing while another text input has focus", () => {
    const input = document.createElement("input");
    host.appendChild(input);
    input.focus();
    pressCmdN();
    flushSync();
    expect(editor()).toBeNull();
  });

  it("ignores ⌘⌥N so it doesn't collide with other alt-modified shortcuts", () => {
    pressCmdN({ altKey: true });
    flushSync();
    expect(editor()).toBeNull();
  });

  it("switches back to the 专注 tab so the editor is visible", () => {
    const tabs = host.querySelectorAll<HTMLButtonElement>(".tab");
    tabs[1].click();
    flushSync();
    expect(host.querySelector(".add")).toBeNull();
    pressCmdN();
    flushSync();
    expect(editor()).not.toBeNull();
  });
});
