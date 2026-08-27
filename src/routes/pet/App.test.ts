import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { app } from "../../lib/state.svelte";
import Pet from "./App.svelte";

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
