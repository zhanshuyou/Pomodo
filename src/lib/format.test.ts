import { describe, expect, it } from "vitest";
import { endsAt, minutesLeft, mmss } from "./format";

describe("mmss", () => {
  it("zero-pads both fields", () => {
    expect(mmss(0)).toBe("00:00");
    expect(mmss(65)).toBe("01:05");
    expect(mmss(1500)).toBe("25:00");
  });

  it("does not roll over past 60 minutes", () => {
    expect(mmss(3661)).toBe("61:01");
  });

  it("clamps negatives to zero", () => {
    expect(mmss(-5)).toBe("00:00");
  });
});

describe("minutesLeft", () => {
  it("floors to whole minutes", () => {
    expect(minutesLeft(1500)).toBe(25);
    expect(minutesLeft(119)).toBe(1);
    expect(minutesLeft(59)).toBe(0);
  });
});

describe("endsAt", () => {
  it("renders the wall-clock finish time", () => {
    const now = new Date(2026, 7, 19, 14, 26, 0);
    expect(endsAt(1500, now)).toBe("预计 14:51 结束");
  });

  it("zero-pads the hour and minute", () => {
    const now = new Date(2026, 7, 19, 8, 3, 0);
    expect(endsAt(300, now)).toBe("预计 08:08 结束");
  });

  it("wraps past midnight", () => {
    const now = new Date(2026, 7, 19, 23, 50, 0);
    expect(endsAt(1500, now)).toBe("预计 00:15 结束");
  });
});
