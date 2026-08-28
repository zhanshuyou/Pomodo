import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const overlayIpc = vi.hoisted(() => ({
  onOverlayShow: vi.fn((_cb: (p: unknown) => void) => Promise.resolve(() => {})),
  dismissOverlay: vi.fn(async () => true),
  snoozeOverlay: vi.fn(async () => true),
}));
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  ...overlayIpc,
}));

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

  it("shows nothing — no mask, no countdown — until a firing arrives", () => {
    expect(host.querySelector(".mask")).toBeNull();
    expect(host.querySelector(".count")).toBeNull();
  });

  it("offers a completion button and the escape hint", () => {
    void unmount(component);
    host.remove();
    let deliver: ((p: unknown) => void) | undefined;
    overlayIpc.onOverlayShow.mockImplementationOnce((cb) => {
      deliver = cb;
      return Promise.resolve(() => {});
    });
    ({ host, component } = render(Overlay));
    deliver?.({
      id: 1,
      name: "收工前复盘",
      message: "先夸自己一句，再写下明天要干的事。",
      intensity: "fullscreen",
      color: "oklch(0.68 0.1 300)",
      mustComplete: false,
      durationSecs: 300,
    });
    flushSync();
    expect(host.querySelector(".count")?.textContent).toBe("05:00");
    expect(host.querySelector(".line")?.textContent).toBe("先夸自己一句，再写下明天要干的事。");
    expect(host.querySelector(".name")?.textContent?.trim()).toBe("收工前复盘");
    expect((host.querySelector(".name .dot") as HTMLElement).style.background).toBe(
      "oklch(0.68 0.1 300)",
    );
    expect(host.querySelector(".done")?.textContent?.trim()).toBe("做完了");
    expect(host.querySelector(".escape")?.textContent).toBe("按 ⎋ 逃跑（它会记着）");
  });

  it("counts down the firing's own duration and reports completion once", () => {
    vi.useFakeTimers();
    void unmount(component);
    host.remove();
    let deliver: ((p: unknown) => void) | undefined;
    overlayIpc.onOverlayShow.mockImplementationOnce((cb) => {
      deliver = cb;
      return Promise.resolve(() => {});
    });
    overlayIpc.dismissOverlay.mockClear();
    ({ host, component } = render(Overlay));
    deliver?.({
      id: 2,
      name: "远眺护眼",
      message: "看看远方",
      intensity: "fullscreen",
      color: "oklch(0.7 0.1 145)",
      mustComplete: false,
      durationSecs: 20,
    });
    flushSync();
    expect(host.querySelector(".count")?.textContent).toBe("00:20");
    vi.advanceTimersByTime(25_000);
    flushSync();
    expect(overlayIpc.dismissOverlay).toHaveBeenCalledTimes(1);
    expect(overlayIpc.dismissOverlay).toHaveBeenCalledWith(2, true);
    vi.useRealTimers();
  });

  it("hides every exit for a 必须完成 firing until the countdown ends", () => {
    vi.useFakeTimers();
    void unmount(component);
    host.remove();
    let deliver: ((p: unknown) => void) | undefined;
    overlayIpc.onOverlayShow.mockImplementationOnce((cb) => {
      deliver = cb;
      return Promise.resolve(() => {});
    });
    overlayIpc.dismissOverlay.mockClear();
    ({ host, component } = render(Overlay));
    deliver?.({
      id: 4,
      name: "收工前复盘",
      message: "先夸自己一句",
      intensity: "fullscreen",
      color: "oklch(0.68 0.1 300)",
      mustComplete: true,
      durationSecs: 161,
    });
    flushSync();
    expect(host.querySelector(".later")).toBeNull();
    expect(host.querySelector(".done")).toBeNull();
    expect(host.querySelector(".escape")?.textContent).toBe("这条得做完才能走");

    window.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(overlayIpc.dismissOverlay).not.toHaveBeenCalled();

    vi.advanceTimersByTime(161_000);
    flushSync();
    expect(host.querySelector(".done")).not.toBeNull();
    vi.useRealTimers();
  });

  it("sways the pet at scale 3", () => {
    void unmount(component);
    host.remove();
    let deliver: ((p: unknown) => void) | undefined;
    overlayIpc.onOverlayShow.mockImplementationOnce((cb) => {
      deliver = cb;
      return Promise.resolve(() => {});
    });
    ({ host, component } = render(Overlay));
    deliver?.({
      id: 3,
      name: "站起来动一动",
      message: "起来！",
      intensity: "fullscreen",
      color: "oklch(0.63 0.13 40)",
      mustComplete: false,
      durationSecs: 120,
    });
    flushSync();
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
      "今天要啃点什么？我准备好了",
    );
    expect(host.querySelector(".bubble.nudging")).toBeNull();
  });

  it("exposes the pet as a keyboard-reachable control", () => {
    const wrap = host.querySelector(".petwrap") as HTMLElement;
    expect(wrap.getAttribute("role")).toBe("button");
    expect(wrap.getAttribute("tabindex")).toBe("0");
    expect(wrap.getAttribute("aria-label")).toContain("双击打开");
  });

  it("offers a close button that is hidden until hover", () => {
    const close = host.querySelector(".close") as HTMLButtonElement;
    expect(close).not.toBeNull();
    expect(close.getAttribute("aria-label")).toBe("隐藏桌面宠物");
  });

  it("does not start a drag from a press alone, so clicks still register", () => {
    const wrap = host.querySelector(".petwrap") as HTMLElement;
    // A pointerdown with no movement must not put the pet into the drag state;
    // startDragging would otherwise swallow the click that follows.
    // jsdom has no PointerEvent; a MouseEvent of the same type dispatches fine.
    wrap.dispatchEvent(
      new MouseEvent("pointerdown", { button: 0, clientX: 10, clientY: 10, bubbles: true }),
    );
    flushSync();
    expect(host.querySelector(".pet.dragging")).toBeNull();
  });
});
