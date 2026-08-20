import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import type { Reminder } from "../../lib/ipc";
import { app } from "../../lib/state.svelte";
import RemindersPane from "./RemindersPane.svelte";

/** A stand-in for what Rust's Reminder::seed produces, with spec copy. */
function seed(
  id: number,
  name: string,
  color: string,
  detail: string,
  message: string,
  hint: string,
  overrides: Partial<Reminder> = {},
): Reminder {
  return {
    id,
    builtin: null,
    name,
    color,
    detail,
    message,
    hint,
    messageEdited: false,
    schedule: { kind: "every", minutes: 30 },
    intensity: "bubble",
    enabled: true,
    rules: {
      activeFromMin: 9 * 60 + 30,
      activeToMin: 18 * 60 + 30,
      weekdays: [true, true, true, true, true, false, false],
      duringFocus: "defer",
      silenceInMeeting: true,
      escalateAfter: 3,
      sound: "木鱼 · 30%",
    },
    remainingSecs: 1800,
    consecutiveIgnores: 0,
    deferred: false,
    lastDailyFire: null,
    ...overrides,
  };
}

describe("RemindersPane", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    app.model.reminders = [
      seed(
        0,
        "站起来动一动",
        "oklch(0.63 0.13 40)",
        "每 45 分钟 · 宠物提示 · 工作时段",
        "再坐下去你就要跟椅子长在一起了，起来！",
        "我不打断你，但下课钟一响我就扑上来。",
        { schedule: { kind: "every", minutes: 45 }, intensity: "pet" },
      ),
      seed(
        1,
        "喝水",
        "oklch(0.66 0.09 195)",
        "每 30 分钟 · 轻量气泡 · 计入每日 8 杯",
        "你的杯子在喊你，它说它很空。",
        "我偷偷在小本本上记你喝了几杯。",
      ),
      seed(
        2,
        "远眺护眼",
        "oklch(0.7 0.1 145)",
        "每 20 分钟 · 轻量气泡 · 20-20-20",
        "眼睛快冒烟了，看看远方压压火。",
        "我数到 20 就放你走，说好了。",
        { schedule: { kind: "every", minutes: 20 }, enabled: false },
      ),
    ];
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(RemindersPane, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
    app.model.reminders = [];
  });

  it("renders the six template chips plus a blank button", () => {
    const chips = [...host.querySelectorAll(".col2 .chip")].map((e) =>
      e.textContent?.trim(),
    );
    expect(chips).toEqual([
      "站立",
      "喝水",
      "护眼",
      "深呼吸",
      "肩颈拉伸",
      "记一句想法",
    ]);
    expect(host.querySelector(".blank")?.textContent?.trim()).toBe("＋ 空白");
  });

  it("lists every reminder with its detail line", () => {
    expect(host.querySelectorAll(".rem")).toHaveLength(3);
    expect([...host.querySelectorAll(".remdetail")].map((e) => e.textContent)).toEqual([
      "每 45 分钟 · 宠物提示 · 工作时段",
      "每 30 分钟 · 轻量气泡 · 计入每日 8 杯",
      "每 20 分钟 · 轻量气泡 · 20-20-20",
    ]);
  });

  it("counts only enabled reminders and fades the disabled tile", () => {
    expect(host.querySelector(".oncount")?.textContent).toBe("2 条开启");
    const tiles = [...host.querySelectorAll(".tile")] as HTMLElement[];
    expect(tiles[2].style.opacity).toBe("0.28");
    expect(tiles[0].style.opacity).toBe("1");
  });

  it("edits the first reminder by default, with its interval selected", () => {
    expect(host.querySelector(".col3 .sectitle")?.textContent).toBe(
      "编辑「站起来动一动」",
    );
    const selected = [...host.querySelectorAll(".col3 .chip.selected")].map((e) =>
      e.textContent?.trim(),
    );
    expect(selected).toEqual(["45 min"]);
  });

  it("offers the four interval chips from the design", () => {
    const chips = [...host.querySelectorAll(".col3 .chip")].map((e) =>
      e.textContent?.trim(),
    );
    expect(chips).toEqual(["20 min", "30 min", "45 min", "60 min"]);
  });

  it("offers the three intensity cards and marks the active one", () => {
    const cards = [...host.querySelectorAll(".style")];
    expect(cards.map((e) => e.querySelector(".slabel")?.textContent)).toEqual([
      "气泡",
      "宠物",
      "全屏",
    ]);
    expect(cards.map((e) => e.querySelector(".shint")?.textContent)).toEqual([
      "角落一闪",
      "它跳给你看",
      "躲不掉",
    ]);
    expect(host.querySelector(".style.sel .slabel")?.textContent).toBe("宠物");
  });

  it("keeps the advanced rules collapsed until disclosed", () => {
    expect(host.querySelector(".rules")).toBeNull();
    expect(host.querySelector(".disclose")?.textContent?.trim()).toContain(
      "还要更精细？展开规则",
    );
  });

  it("reveals the six rule rows with the spec defaults when disclosed", () => {
    (host.querySelector(".disclose") as HTMLButtonElement).click();
    flushSync();

    const names = [...host.querySelectorAll(".rname")].map((e) => e.textContent);
    const values = [...host.querySelectorAll(".rvalue")].map((e) => e.textContent);
    expect(names).toEqual([
      "生效时段",
      "生效日期",
      "专注中",
      "检测到会议 / 通话",
      "连续忽略 3 次",
      "声音",
    ]);
    expect(values).toEqual([
      "09:30 – 18:30",
      "周一 – 周五",
      "推迟到本轮结束",
      "静默",
      "升级为全屏",
      "木鱼 · 30%",
    ]);
    expect(host.querySelector(".disclose")?.textContent?.trim()).toContain(
      "收起精细规则",
    );
  });

  it("shows the tone-aware hint beside the small pet", () => {
    expect(host.querySelector(".hintcard span")?.textContent).toBe(
      "我不打断你，但下课钟一响我就扑上来。",
    );
    const canvas = host.querySelector(".hintcard canvas") as HTMLCanvasElement;
    expect(canvas.width).toBe(48); // 16 x scale 3
  });

  it("puts the editable message in a textarea", () => {
    const field = host.querySelector(".message") as HTMLTextAreaElement;
    expect(field.value).toBe("再坐下去你就要跟椅子长在一起了，起来！");
  });
});
