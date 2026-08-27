import { describe, expect, it } from "vitest";

import {
  escalationLabel,
  minutesToTime,
  timeToMinutes,
  weekdaysLabel,
} from "./rules";

describe("minutesToTime / timeToMinutes", () => {
  it("round-trips the defaults", () => {
    expect(minutesToTime(9 * 60 + 30)).toBe("09:30");
    expect(minutesToTime(18 * 60 + 30)).toBe("18:30");
    expect(timeToMinutes("09:30")).toBe(570);
    expect(timeToMinutes("18:30")).toBe(1110);
  });

  it("clamps out-of-range minutes and rejects junk", () => {
    expect(minutesToTime(-5)).toBe("00:00");
    expect(minutesToTime(99_999)).toBe("23:59");
    expect(timeToMinutes("")).toBeNull();
    expect(timeToMinutes("25:00")).toBeNull();
    expect(timeToMinutes("9:75")).toBeNull();
  });
});

describe("weekdaysLabel", () => {
  const days = (...on: number[]) => [0, 1, 2, 3, 4, 5, 6].map((i) => on.includes(i));

  it("names the common shapes", () => {
    expect(weekdaysLabel(days(0, 1, 2, 3, 4))).toBe("周一 – 周五");
    expect(weekdaysLabel(days(0, 1, 2, 3, 4, 5, 6))).toBe("每天");
    expect(weekdaysLabel(days(5, 6))).toBe("周末");
    expect(weekdaysLabel(days())).toBe("从不");
  });

  it("does not call Mon–Sun 周一 – 周五 just because the weekdays are on", () => {
    expect(weekdaysLabel(days(0, 1, 2, 3, 4, 6))).toBe("周一、二、三、四、五、日");
  });

  it("lists an irregular selection", () => {
    expect(weekdaysLabel(days(0, 2, 4))).toBe("周一、三、五");
  });
});

describe("escalationLabel", () => {
  it("says how many ignores it takes, or that it never escalates", () => {
    expect(escalationLabel(3)).toBe("忽略 3 次后升级为全屏");
    expect(escalationLabel(0)).toBe("不升级");
  });
});
