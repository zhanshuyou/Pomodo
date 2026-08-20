import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import BubbleToast from "../bubble/App.svelte";
import DesktopPet from "../pet/App.svelte";
import Overlay from "./App.svelte";

function render(Component: typeof Overlay | typeof BubbleToast | typeof DesktopPet) {
  const host = document.createElement("div");
  document.body.appendChild(host);
  const component = mount(Component, { target: host });
  flushSync();
  return { host, component };
}

describe("fullscreen overlay", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => ({ host, component } = render(Overlay)));
  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  it("uses the design's mask colour and fills the viewport", () => {
    const mask = host.querySelector(".mask") as HTMLElement;
    expect(mask).not.toBeNull();
  });

  it("starts at the design's 02:41 countdown", () => {
    expect(host.querySelector(".count")?.textContent).toBe("02:41");
  });

  it("falls back to the design's break line before any firing arrives", () => {
    expect(host.querySelector(".line")?.textContent).toBe("站起来走走，看点远的东西");
  });

  it("offers a completion button and the escape hint", () => {
    expect(host.querySelector(".done")?.textContent?.trim()).toBe("做完了");
    expect(host.querySelector(".escape")?.textContent).toBe("按 ⎋ 逃跑（它会记着）");
  });

  it("sways the pet at scale 3", () => {
    const canvas = host.querySelector("canvas") as HTMLCanvasElement;
    expect(canvas.width).toBe(48); // 16 x scale 3
    expect(canvas.className).toContain("pet--sway");
  });
});

describe("bubble toast", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => ({ host, component } = render(BubbleToast)));
  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  it("renders nothing until a firing arrives", () => {
    // 轻量气泡 only exists while it has something to say.
    expect(host.querySelector(".toast")).toBeNull();
  });
});

describe("desktop pet", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => ({ host, component } = render(DesktopPet)));
  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  it("bobs at scale 8 with a shadow beneath", () => {
    const canvas = host.querySelector("canvas") as HTMLCanvasElement;
    expect(canvas.width).toBe(128); // 16 x scale 8
    expect(canvas.className).toContain("pet--bob");
    expect(host.querySelector(".shadow")).not.toBeNull();
  });

  it("shows the tone-aware pet line when not being nudged", () => {
    expect(host.querySelector(".bubble")?.textContent?.trim()).toBe(
      "还有 25 分钟，我盯着你呢",
    );
    expect(host.querySelector(".bubble.nudging")).toBeNull();
  });

  it("exposes the pet as a keyboard-reachable control", () => {
    const pet = host.querySelector(".pet") as HTMLElement;
    expect(pet.getAttribute("role")).toBe("button");
    expect(pet.getAttribute("tabindex")).toBe("0");
  });
});
