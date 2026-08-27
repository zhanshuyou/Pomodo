import { describe, expect, it } from "vitest";
import {
  emptyTasks,
  miniLabel,
  petLine,
  petVerdict,
  phaseLabel,
  roundsUntilLongBreak,
  runLabel,
  runLabelShort,
  snoozeLabel,
  tagline,
} from "./copy";

describe("tagline", () => {
  it("matches the spec for every tone", () => {
    expect(tagline("professional")).toBe(
      "菜单栏与主窗口双入口的番茄计时器，含可自定义的身体提醒与桌面宠物。",
    );
    expect(tagline("gentle")).toBe(
      "一个陪你专注的番茄钟：它记得提醒你站立喝水，也记得在你完成时替你高兴。",
    );
    expect(tagline("playful")).toBe(
      "它负责计时、催你喝水、盯你站起来，并在你摸鱼时用眼神谴责你。",
    );
  });
});

describe("petLine", () => {
  const focus = { phase: "focus" as const, running: true, minutes: 12 };

  it("interpolates the minute count per tone while focusing", () => {
    expect(petLine("professional", focus)).toBe("本轮剩余 12 分钟。");
    expect(petLine("gentle", focus)).toBe("再 12 分钟就好，我陪着你。");
    expect(petLine("playful", focus)).toBe("还有 12 分钟，我盯着你呢");
  });

  it("does not say 本轮剩余 during a break", () => {
    const rest = { phase: "shortBreak" as const, running: true, minutes: 4 };
    expect(petLine("professional", rest)).toBe("休息剩余 4 分钟。");
    expect(petLine("playful", rest)).toContain("别偷偷干活");
  });

  it("tells paused and never-started apart", () => {
    expect(petLine("gentle", { phase: "focus", running: false, minutes: 25 })).toBe(
      "今天要啃点什么？",
    );
    expect(
      petLine("gentle", { phase: "focus", running: false, minutes: 10, started: true }),
    ).toBe("先停一下，等你回来。");
    expect(petLine("gentle", { phase: "longBreak", running: false, minutes: 15 })).toBe(
      "先停一下，等你回来。",
    );
  });
});

describe("petVerdict", () => {
  // The artboard's week: 14h20m, +12%, fewer interruptions.
  const good = { weekFocusSecs: 14 * 3600 + 20 * 60, weekDeltaPct: 12, interruptionsDelta: -4 };

  it("fills the spec's sentences with the real week", () => {
    expect(petVerdict("professional", good)).toBe("本周专注 14h20m，较上周 +12%，中断减少。");
    // 14h20m is 112% of last week, so the gain is ~1h32m — not the mock's 1h40m.
    expect(petVerdict("gentle", good)).toBe("这周你比上周多专注了 1 小时 32 分，很稳。");
    expect(petVerdict("playful", good)).toBe("这周表现不错，我勉为其难地允许你今晚多睡半小时。");
  });

  it("is honest about a worse week", () => {
    const bad = { weekFocusSecs: 5 * 3600, weekDeltaPct: -30, interruptionsDelta: 2 };
    expect(petVerdict("professional", bad)).toBe("本周专注 5h00m，较上周 −30%，中断增多。");
    expect(petVerdict("gentle", bad)).toContain("少专注了 2 小时 9 分");
    expect(petVerdict("playful", bad)).not.toContain("表现不错");
  });

  it("has a distinct line for an empty week and for no prior week", () => {
    const empty = { weekFocusSecs: 0, weekDeltaPct: 0, interruptionsDelta: 0 };
    expect(petVerdict("professional", empty)).toBe("本周尚无专注记录。");
    const flat = { weekFocusSecs: 3600, weekDeltaPct: 0, interruptionsDelta: 0 };
    expect(petVerdict("gentle", flat)).toBe("这周专注了 1 小时，稳稳的。");
  });
});

describe("runLabelShort", () => {
  it("is the artboard's two-character tray label", () => {
    expect(runLabelShort(true)).toBe("暂停");
    expect(runLabelShort(false)).toBe("开始");
  });
});

describe("emptyTasks", () => {
  it("points at ⌘N in every tone", () => {
    for (const t of ["professional", "gentle", "playful"] as const) {
      expect(emptyTasks(t)).toContain("⌘N");
    }
  });
});

describe("roundsUntilLongBreak", () => {
  it("reproduces the artboard line at round 2 of 4 with a 15-minute long break", () => {
    expect(roundsUntilLongBreak("playful", "focus", 2, 15)).toBe(
      "再 2 轮就能哄它去睡长觉（15 分钟）",
    );
  });

  it("counts down and follows the configured long break", () => {
    expect(roundsUntilLongBreak("professional", "focus", 3, 20)).toBe("再 3 轮进入长休息（20 分钟）。");
    expect(roundsUntilLongBreak("gentle", "focus", 1, 15)).toBe("再 1 轮就能歇个长的（15 分钟）。");
    expect(roundsUntilLongBreak("playful", "focus", 0, 15)).toBe("这轮结束就能哄它去睡长觉（15 分钟）");
  });

  it("speaks to the break phases too", () => {
    expect(roundsUntilLongBreak("professional", "shortBreak", 2, 15)).toBe(
      "短休息中，之后还有 2 轮进入长休息。",
    );
    expect(roundsUntilLongBreak("playful", "longBreak", 0, 15)).toBe("它在睡长觉，15 分钟内别吵。");
  });
});

describe("phaseLabel", () => {
  it("labels focus and both breaks", () => {
    expect(phaseLabel("focus")).toBe("专注中");
    expect(phaseLabel("shortBreak")).toBe("休息中");
    expect(phaseLabel("longBreak")).toBe("休息中");
  });
});

describe("button labels", () => {
  it("swaps on state", () => {
    expect(runLabel(true)).toBe("让它歇会儿");
    expect(runLabel(false)).toBe("开始专注");
    expect(miniLabel(true)).toBe("退出迷你模式");
    expect(miniLabel(false)).toBe("迷你模式");
  });
});

describe("snoozeLabel", () => {
  it("names the delay in every tone", () => {
    expect(snoozeLabel("professional", 10)).toBe("10 分钟后再提醒");
    expect(snoozeLabel("gentle", 10)).toBe("过 10 分钟再叫我");
    expect(snoozeLabel("playful", 10)).toBe("再赖 10 分钟");
  });
});
