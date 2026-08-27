import { describe, expect, it } from "vitest";
import {
  miniLabel,
  petLine,
  petVerdict,
  phaseLabel,
  runLabel,
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
  it("interpolates the minute count per tone", () => {
    expect(petLine("professional", 12)).toBe("本轮剩余 12 分钟。");
    expect(petLine("gentle", 12)).toBe("再 12 分钟就好，我陪着你。");
    expect(petLine("playful", 12)).toBe("还有 12 分钟，我盯着你呢");
  });
});

describe("petVerdict", () => {
  it("matches the spec for every tone", () => {
    expect(petVerdict("professional")).toBe("本周专注 14h20m，较上周 +12%，中断率下降。");
    expect(petVerdict("gentle")).toBe("这周你比上周多专注了 1 小时 40 分，很稳。");
    expect(petVerdict("playful")).toBe("这周表现不错，我勉为其难地允许你今晚多睡半小时。");
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
