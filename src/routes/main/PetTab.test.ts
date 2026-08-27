import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { app } from "../../lib/state.svelte";
import PetTab from "./PetTab.svelte";
import StatsTab from "./StatsTab.svelte";

/** Both tabs take no props; naming them exactly keeps mount()'s generics happy. */
function render(Component: typeof PetTab | typeof StatsTab) {
  const host = document.createElement("div");
  document.body.appendChild(host);
  const component = mount(Component, { target: host });
  flushSync();
  return { host, component };
}

describe("PetTab", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    app.model.pet = {
      selected: 0,
      lifetimePomodoros: 0,
      custom: { focus: null, rest: null, nag: null },
      useCustom: false,
    };
    ({ host, component } = render(PetTab));
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  it("renders all six pets", () => {
    expect(host.querySelectorAll(".petcard")).toHaveLength(6);
    expect([...host.querySelectorAll(".petname")].map((e) => e.textContent)).toEqual([
      "MOCHI",
      "PUDDING",
      "TOFU",
      "BEAN",
      "PEEP",
      "BOO",
    ]);
  });

  it("locks and disables exactly PEEP and BOO at zero pomodoros", () => {
    const locked = [...host.querySelectorAll(".petcard.locked")];
    expect(locked).toHaveLength(2);
    expect(locked.map((e) => e.querySelector(".petname")?.textContent)).toEqual([
      "PEEP",
      "BOO",
    ]);
    for (const card of locked) {
      expect((card as HTMLButtonElement).disabled).toBe(true);
    }
  });

  it("reproduces the artboard's Lv.7 hero figures", () => {
    void unmount(component);
    host.remove();
    app.model.pet.lifetimePomodoros = 86; // 6 levels + 8
    ({ host, component } = render(PetTab));

    expect(host.querySelector(".plevel")?.textContent).toBe("Lv.7 · 好奇期");
    expect(host.querySelector(".hint")?.textContent?.trim()).toBe(
      "再专注 5 个番茄升到 Lv.8，解锁「披风」",
    );
    const width = (host.querySelector(".fill") as HTMLElement).style.width;
    expect(Math.round(parseFloat(width))).toBe(62);
  });

  it("unlocks PEEP at 150 lifetime pomodoros", () => {
    void unmount(component);
    host.remove();
    app.model.pet.lifetimePomodoros = 150;
    ({ host, component } = render(PetTab));
    expect(host.querySelectorAll(".petcard.locked")).toHaveLength(1);
  });

  it("shows the custom-pet drop prompt and the three state chips", () => {
    expect(host.querySelector(".drop")?.textContent?.trim()).toBe(
      "拖入你的宠物 PNG / GIF",
    );
    const chips = [...host.querySelectorAll(".chip")].map((e) => e.textContent?.trim());
    expect(chips).toEqual(
      expect.arrayContaining(["专注", "休息", "催你站起来"]),
    );
  });

  it("shows thumbnails for imported slots and a use-custom chip", () => {
    void unmount(component);
    host.remove();
    app.model.pet.custom = { focus: "/pets/focus.png", rest: null, nag: "/pets/nag.gif" };
    app.model.pet.useCustom = true;
    ({ host, component } = render(PetTab));
    expect(host.querySelectorAll(".chip .thumb")).toHaveLength(2);
    expect(host.querySelector(".slot img")?.getAttribute("src")).toBe("/pets/focus.png");
    const chips = [...host.querySelectorAll(".chip")].map((e) => e.textContent?.trim());
    expect(chips).toContain("用自己的形象");
  });

  it("asks for a second click before clearing an imported slot", () => {
    vi.useFakeTimers();
    void unmount(component);
    host.remove();
    app.model.pet.custom = { focus: "/pets/focus.png", rest: null, nag: null };
    app.model.pet.useCustom = true;
    ({ host, component } = render(PetTab));
    const focusChip = [...host.querySelectorAll<HTMLButtonElement>(".chip")].find(
      (e) => e.textContent?.trim() === "专注",
    )!;
    focusChip.click();
    flushSync();
    expect(focusChip.textContent?.trim()).toBe("再点一次清除「专注」");
    // Walking away resets the confirmation.
    vi.advanceTimersByTime(3000);
    flushSync();
    expect(focusChip.textContent?.trim()).toBe("专注");
    vi.useRealTimers();
  });

  it("renders the four behaviour flags with the first three on", () => {
    const chips = [...host.querySelectorAll(".chip")];
    const flags = chips.slice(-4);
    expect(flags.map((e) => e.textContent?.trim())).toEqual([
      "贴边吸附",
      "点击互动",
      "全屏时隐藏",
      "睡眠动画",
    ]);
    expect(flags.filter((e) => e.classList.contains("selected"))).toHaveLength(3);
  });
});

describe("StatsTab", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    app.summary = {
      weekFocusSecs: 14 * 3600 + 20 * 60,
      weekDeltaPct: 12,
      pomodoros: 43,
      dailyAverage: 6.142,
      interruptions: 9,
      interruptionsDelta: -4,
      streak: 12,
      bestStreak: 18,
      bars: [3, 5, 2, 6, 4, 0, 1, 5, 7, 4, 5, 6, 2, 5].map((count, i) => ({
        label: "一二三四五六日"[i % 7],
        count,
      })),
      interruptionHotspot: { startHour: 15, endHour: 16, interruptions: 5, total: 9 },
    };
    ({ host, component } = render(StatsTab));
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.summary = null;
  });

  it("renders the four stat cards with the artboard's values", () => {
    const values = [...host.querySelectorAll(".cvalue")].map((e) => e.textContent);
    expect(values).toEqual(["14h20m", "43", "9", "12"]);
  });

  it("formats deltas with a real minus sign and marks the good ones", () => {
    const deltas = [...host.querySelectorAll(".cdelta")].map((e) => e.textContent);
    expect(deltas).toEqual([
      "较上周 +12%",
      "日均 6.1 个",
      "较上周 −4",
      "个人最佳 18",
    ]);
    // U+2212, not a hyphen.
    expect(deltas[2]).toContain("−");
    expect(host.querySelectorAll(".cdelta.good")).toHaveLength(2);
  });

  it("renders fourteen bars, one cell per pomodoro", () => {
    expect(host.querySelectorAll(".bar")).toHaveLength(14);
    // 3+5+2+6+4+1+5+7+4+5+6+2+5 = 55 real cells, plus one empty placeholder.
    expect(host.querySelectorAll(".cell")).toHaveLength(56);
    expect(host.querySelectorAll(".cell.empty")).toHaveLength(1);
  });

  it("colours bar cells with the design's relative-colour ramp", () => {
    const cells = [...host.querySelectorAll(".cell:not(.empty)")] as HTMLElement[];
    // jsdom normalises the commutative calc, emitting `calc(0.16 + l)` for our
    // `calc(l + 0.16)`. Assert on the parts rather than the exact spelling;
    // theme.test.ts pins the literal string barCellColor produces.
    const first = cells[0].style.background;
    expect(first).toContain("from oklch(0.63 0.13 40)");
    expect(first).toContain("0.16");
    expect(first).toMatch(/\bl\b/);
    // Third cell of the first bar steps down twice: 0.16 - 2 * 0.035 = 0.09.
    expect(cells[2].style.background).toContain("0.09");
  });

  it("renders both insight cards, with the hotspot text computed from real data", () => {
    const titles = [...host.querySelectorAll(".ititle")].map((e) => e.textContent);
    expect(titles).toEqual(["被打断最多的时段", "Pomodo 的评价"]);
    expect(host.querySelector(".ccaption")?.textContent).toBe(
      "每格 = 一个番茄，颜色越深越连贯",
    );
    const bodies = [...host.querySelectorAll(".ibody")].map((e) => e.textContent);
    expect(bodies[0]).toBe(
      "15:00–16:00，9 轮里有 5 轮被打断。要不要把这段设成「勿扰 + 只留宠物提示」？",
    );
  });

  it("hides the hotspot card instead of a fabricated conclusion when there is not enough data", () => {
    app.summary = { ...app.summary!, interruptionHotspot: null };
    flushSync();
    const titles = [...host.querySelectorAll(".ititle")].map((e) => e.textContent);
    expect(titles).toEqual(["Pomodo 的评价"]);
  });
});
