import { flushSync, mount, unmount } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { Reminder } from "../../lib/ipc";
import { app } from "../../lib/state.svelte";
import RemindersPane from "./RemindersPane.svelte";

const updateReminder = vi.hoisted(() =>
  vi.fn(async (_id: number, _patch: import("../../lib/ipc").ReminderPatch) => {}),
);
vi.mock("../../lib/ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/ipc")>()),
  updateReminder,
}));

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
      mustComplete: false,
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

  it("reveals the six rule rows when disclosed", () => {
    (host.querySelector(".disclose") as HTMLButtonElement).click();
    flushSync();

    const names = [...host.querySelectorAll(".rname")].map(
      (e) => e.firstChild?.textContent?.trim(),
    );
    expect(names).toEqual([
      "生效时段",
      "生效日期",
      "专注中",
      "检测到会议 / 通话时静默",
      "连续忽略",
      "声音",
    ]);
    expect(host.querySelector(".rvalue")?.textContent).toBe("木鱼 · 30%");
    expect(host.querySelector(".disclose")?.textContent?.trim()).toContain(
      "收起精细规则",
    );
  });

  describe("name and schedule editor", () => {
    beforeEach(() => updateReminder.mockClear());

    const patchSent = () => updateReminder.mock.calls.at(-1)![1];

    it("renames through the name field, ignoring blanks and no-ops", () => {
      const name = host.querySelector<HTMLInputElement>(".name")!;
      expect(name.value).toBe("站起来动一动");
      name.value = "  ";
      name.dispatchEvent(new Event("change", { bubbles: true }));
      name.value = "站起来动一动";
      name.dispatchEvent(new Event("change", { bubbles: true }));
      expect(updateReminder).not.toHaveBeenCalled();
      name.value = " 起立 ";
      name.dispatchEvent(new Event("change", { bubbles: true }));
      expect(patchSent()).toEqual({ name: "起立" });
    });

    it("sends a schedule for a preset chip and clamps a custom interval", () => {
      [...host.querySelectorAll<HTMLButtonElement>(".col3 .chip")]
        .find((c) => c.textContent?.trim() === "20 min")
        ?.click();
      expect(patchSent()).toEqual({ schedule: { kind: "every", minutes: 20 } });
      const custom = host.querySelector<HTMLInputElement>(".minutes")!;
      custom.value = "900";
      custom.dispatchEvent(new Event("change", { bubbles: true }));
      expect(patchSent()).toEqual({ schedule: { kind: "every", minutes: 480 } });
    });

    it("switches to a daily time and edits it without touching the interval chips", () => {
      [...host.querySelectorAll<HTMLButtonElement>(".mode")]
        .find((b) => b.textContent?.trim() === "每天定时")
        ?.click();
      expect(patchSent()).toEqual({
        schedule: { kind: "dailyAt", hour: 17, minute: 30 },
      });
      app.model.reminders[0].schedule = { kind: "dailyAt", hour: 17, minute: 30 };
      flushSync();
      expect(host.querySelector(".col3 .chip")).toBeNull();
      const time = host.querySelector<HTMLInputElement>(".daily")!;
      expect(time.value).toBe("17:30");
      time.value = "09:05";
      time.dispatchEvent(new Event("change", { bubbles: true }));
      expect(patchSent()).toEqual({ schedule: { kind: "dailyAt", hour: 9, minute: 5 } });
    });

    it("warns when the message is blank", () => {
      expect(host.querySelector(".warn")).toBeNull();
      app.model.reminders[0].message = "   ";
      flushSync();
      expect(host.querySelector(".warn")?.textContent).toContain("不会响");
    });
  });

  describe("精细规则 editor", () => {
    beforeEach(() => {
      updateReminder.mockClear();
      host.querySelector<HTMLButtonElement>(".disclose")?.click();
      flushSync();
    });

    const rulesSent = () => updateReminder.mock.calls.at(-1)![1].rules!;

    it("shows the defaults as editable controls", () => {
      const times = [...host.querySelectorAll<HTMLInputElement>(".time")].map((e) => e.value);
      expect(times).toEqual(["09:30", "18:30"]);
      expect(host.querySelectorAll(".day.on")).toHaveLength(5);
      expect(host.querySelector(".segbtn.on")?.textContent?.trim()).toBe("推迟到本轮结束");
      expect(host.querySelector<HTMLInputElement>(".esc")?.value).toBe("3");
      expect(host.querySelector(".rsub")?.textContent).toBe("周一 – 周五");
    });

    it("patches the whole rules block when a weekday is toggled", () => {
      host.querySelector<HTMLButtonElement>('.day[aria-label="周六"]')?.click();
      flushSync();
      expect(updateReminder).toHaveBeenCalledTimes(1);
      expect(updateReminder.mock.calls[0][0]).toBe(0);
      expect(rulesSent().weekdays).toEqual([true, true, true, true, true, true, false]);
      expect(rulesSent().duringFocus).toBe("defer");
    });

    it("sends the new window when a time input changes", () => {
      const from = host.querySelector<HTMLInputElement>(".time")!;
      from.value = "08:00";
      from.dispatchEvent(new Event("change", { bubbles: true }));
      expect(rulesSent().activeFromMin).toBe(480);
    });

    it("switches the during-focus behaviour and the meeting toggle", () => {
      [...host.querySelectorAll<HTMLButtonElement>(".segbtn")]
        .find((b) => b.textContent?.trim() === "直接打断")
        ?.click();
      expect(rulesSent().duringFocus).toBe("interrupt");
      host.querySelector<HTMLButtonElement>(".rules .switch")?.click();
      expect(rulesSent().silenceInMeeting).toBe(false);
    });

    it("clamps the escalation count and explains 0 as never", () => {
      const num = host.querySelector<HTMLInputElement>(".esc")!;
      num.value = "42";
      num.dispatchEvent(new Event("change", { bubbles: true }));
      expect(rulesSent().escalateAfter).toBe(10);
      app.model.reminders[0].rules.escalateAfter = 0;
      flushSync();
      expect([...host.querySelectorAll(".rsub")].at(-1)?.textContent).toBe("不升级");
    });
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

  // The delete itself goes through invoke, which cannot run under jsdom, so
  // these cover everything up to the point of no return. delete_reminder's own
  // behaviour is covered on the Rust side.
  it("offers a delete control on every reminder, resting out of the way", () => {
    expect(host.querySelectorAll(".rem .del")).toHaveLength(3);
    expect(host.querySelector(".rem.confirming")).toBeNull();
  });

  it("asks before deleting, naming the reminder, and deletes nothing yet", () => {
    const del = host.querySelectorAll(".rem .del")[1] as HTMLButtonElement;
    del.click();
    flushSync();

    expect(host.querySelector(".confirm-ask")?.textContent).toBe("删掉「喝水」？");
    expect(host.querySelectorAll(".rem.confirming")).toHaveLength(1);
    // Still all three: asking is not doing.
    expect(host.querySelectorAll(".rem")).toHaveLength(3);
    expect(app.reminders).toHaveLength(3);
  });

  it("backs out of the confirmation without touching the list", () => {
    (host.querySelectorAll(".rem .del")[1] as HTMLButtonElement).click();
    flushSync();
    (host.querySelector(".confirm-no") as HTMLButtonElement).click();
    flushSync();

    expect(host.querySelector(".rem.confirming")).toBeNull();
    expect(host.querySelectorAll(".rem")).toHaveLength(3);
  });

  it("does not select a reminder for editing when its delete is clicked", () => {
    (host.querySelectorAll(".rem .del")[1] as HTMLButtonElement).click();
    flushSync();
    // Still editing the first one, not the one being deleted.
    expect(host.querySelector(".col3 .sectitle")?.textContent).toBe(
      "编辑「站起来动一动」",
    );
  });

  it("drops the confirmation when another reminder is selected", () => {
    (host.querySelectorAll(".rem .del")[1] as HTMLButtonElement).click();
    flushSync();
    (host.querySelectorAll(".rem")[2] as HTMLElement).click();
    flushSync();

    expect(host.querySelector(".rem.confirming")).toBeNull();
  });
});

describe("RemindersPane with nothing left", () => {
  let host: HTMLDivElement;
  let component: Record<string, unknown>;

  beforeEach(() => {
    app.model.reminders = [];
    host = document.createElement("div");
    document.body.appendChild(host);
    component = mount(RemindersPane, { target: host });
    flushSync();
  });

  afterEach(() => {
    void unmount(component);
    host.remove();
  });

  it("says so rather than leaving the editor column blank and broken-looking", () => {
    expect(host.querySelectorAll(".rem")).toHaveLength(0);
    expect(host.querySelector(".remempty")?.textContent).toBe(
      "还没有提醒，从上面抓一个模板",
    );
  });
});
