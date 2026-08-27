import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { app } from "../../lib/state.svelte";
import Pet from "./App.svelte";

const ipcSpies = vi.hoisted(() => ({
  ackReminder: vi.fn(async () => {}),
  ignoreReminder: vi.fn(async () => {}),
  onPetNudge: vi.fn(),
}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  ...ipcSpies,
}));

/**
 * The desktop pet takes its mood from Rust (`pet:state`) rather than deciding
 * for itself; these tests drive `app.model.petMood` directly and check that
 * each mood lands on the right animation class and bubble copy.
 */
describe("desktop pet mood", () => {
  let host: HTMLDivElement;
  let component: ReturnType<typeof mount>;

  function render() {
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(Pet, { target: host });
    flushSync();
  }

  beforeEach(() => {
    app.model.petMood = "focus";
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.model.petMood = "focus";
  });

  it("bobs while focused and keeps the countdown line", () => {
    render();
    expect(host.querySelector("canvas")?.classList.contains("pet--bob")).toBe(
      true,
    );
    expect(host.querySelector(".bubble")?.textContent).toContain("25");
  });

  it("hops while nagging", () => {
    app.model.petMood = "nagging";
    render();
    expect(host.querySelector("canvas")?.classList.contains("pet--hop")).toBe(
      true,
    );
  });

  it("dozes off when Rust says it is sleeping", () => {
    app.model.petMood = "sleeping";
    render();
    const canvas = host.querySelector("canvas");
    expect(canvas?.classList.contains("pet--sleep")).toBe(true);
    expect(host.querySelector(".bubble")?.textContent?.trim()).toBe("zzz…");
  });

  it("records an ignore when a nudge times out unanswered", () => {
    vi.useFakeTimers();
    let deliver: ((p: unknown) => void) | undefined;
    ipcSpies.onPetNudge.mockImplementation((cb: (p: unknown) => void) => {
      deliver = cb;
      return Promise.resolve(() => {});
    });
    ipcSpies.ignoreReminder.mockClear();
    render();
    deliver?.({
      id: 5,
      name: "站立",
      message: "起来动动",
      intensity: "pet",
      color: "oklch(0.63 0.13 40)",
    });
    flushSync();
    expect(host.querySelector(".bubble.nudging")).not.toBeNull();
    vi.advanceTimersByTime(12_000);
    flushSync();
    expect(ipcSpies.ignoreReminder).toHaveBeenCalledWith(5);
    expect(host.querySelector(".bubble.nudging")).toBeNull();
    vi.useRealTimers();
  });

  it("wakes up again when the mood changes back", () => {
    app.model.petMood = "sleeping";
    render();
    app.model.petMood = "focus";
    flushSync();
    expect(host.querySelector("canvas")?.classList.contains("pet--bob")).toBe(
      true,
    );
    expect(host.querySelector(".bubble")?.textContent?.trim()).not.toBe("zzz…");
  });
});
