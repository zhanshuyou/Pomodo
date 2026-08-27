import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { app } from "../state.svelte";
import Pet from "./Pet.svelte";

/**
 * Pet is the one place that decides between the built-in sprite and a
 * user-imported picture, so every window gets the same answer.
 */
describe("Pet", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  function render(props: Record<string, unknown> = {}) {
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(Pet, { target: host, props });
    flushSync();
  }

  beforeEach(() => {
    app.model.pet.custom = { focus: "/pets/focus.png", rest: "/pets/rest.png", nag: null };
    app.model.pet.useCustom = true;
    app.model.petMood = "focus";
    app.model.timer.phase = "focus";
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.model.pet.custom = { focus: null, rest: null, nag: null };
    app.model.pet.useCustom = false;
    app.model.petMood = "focus";
  });

  it("draws the built-in canvas when custom sprites are switched off", () => {
    app.model.pet.useCustom = false;
    render({ scale: 4 });
    expect(host.querySelector("canvas")).not.toBeNull();
    expect(host.querySelector("img")).toBeNull();
  });

  it("shows the focus picture in a box sized like the canvas would be", () => {
    render({ scale: 4, anim: "bob" });
    const frame = host.querySelector<HTMLElement>(".frame")!;
    expect(frame.style.width).toBe("64px");
    expect(frame.classList.contains("frame--bob")).toBe(true);
    expect(host.querySelector("img")?.getAttribute("src")).toBe("/pets/focus.png");
  });

  it("follows the mood: resting during a break, nagging falls back to focus", () => {
    app.model.petMood = "break";
    app.model.timer.phase = "shortBreak";
    render({ scale: 4 });
    expect(host.querySelector("img")?.getAttribute("src")).toBe("/pets/rest.png");

    app.model.petMood = "nagging";
    flushSync();
    expect(host.querySelector("img")?.getAttribute("src")).toBe("/pets/focus.png");
  });

  it("honours an explicit slot", () => {
    render({ scale: 4, slot: "rest" });
    expect(host.querySelector("img")?.getAttribute("src")).toBe("/pets/rest.png");
  });

  it("upscales by whole multiples only once the image size is known", () => {
    render({ scale: 8 });
    const img = host.querySelector<HTMLImageElement>("img")!;
    Object.defineProperty(img, "naturalWidth", { value: 20 });
    Object.defineProperty(img, "naturalHeight", { value: 30 });
    img.dispatchEvent(new Event("load"));
    flushSync();
    // 128px box, 30px tall → ×4, not the 4.27 that object-fit would use.
    expect(img.style.width).toBe("80px");
    expect(img.style.height).toBe("120px");
  });
});
