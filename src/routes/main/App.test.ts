import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App.svelte";

/**
 * The ⌘N shortcut mirrors TaskSidebar's own "＋ 加一件事" button, so this
 * only has to prove the keydown handler reaches window.prompt — the button
 * itself and addTask are covered by FocusTab.test.ts and ipc.test.ts.
 */
describe("main window ⌘N shortcut", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;
  let prompt: ReturnType<typeof vi.fn<() => string | null>>;

  beforeEach(() => {
    host = document.createElement("div");
    document.body.appendChild(host);
    prompt = vi.fn(() => null);
    window.prompt = prompt;
    component = mount(App, { target: host });
    flushSync();
  });

  const originalPrompt = window.prompt;

  afterEach(() => {
    void unmount(component);
    host.remove();
    window.prompt = originalPrompt;
  });

  it("opens the add-task prompt on ⌘N", () => {
    window.dispatchEvent(
      new KeyboardEvent("keydown", { code: "KeyN", metaKey: true }),
    );
    expect(prompt).toHaveBeenCalledTimes(1);
  });

  it("does nothing while a text input has focus", () => {
    const input = document.createElement("input");
    host.appendChild(input);
    input.focus();

    window.dispatchEvent(
      new KeyboardEvent("keydown", { code: "KeyN", metaKey: true }),
    );
    expect(prompt).not.toHaveBeenCalled();
  });

  it("ignores ⌘⌥N so it doesn't collide with other alt-modified shortcuts", () => {
    window.dispatchEvent(
      new KeyboardEvent("keydown", { code: "KeyN", metaKey: true, altKey: true }),
    );
    expect(prompt).not.toHaveBeenCalled();
  });
});
