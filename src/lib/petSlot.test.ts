import { describe, expect, it } from "vitest";

import { integerScale, petSlotFor, resolveSlot } from "./petSlot";

describe("petSlotFor", () => {
  it("wears the nag sprite whenever the pet is nagging", () => {
    expect(petSlotFor("nagging", "focus")).toBe("nag");
    expect(petSlotFor("nagging", "shortBreak")).toBe("nag");
  });

  it("wears the focus sprite only during a focus phase", () => {
    expect(petSlotFor("focus", "focus")).toBe("focus");
  });

  it("rests during breaks, while dozing, and whenever the phase is not focus", () => {
    expect(petSlotFor("break", "shortBreak")).toBe("rest");
    expect(petSlotFor("sleeping", "focus")).toBe("rest");
    expect(petSlotFor("focus", "longBreak")).toBe("rest");
  });
});

describe("resolveSlot", () => {
  const only = (slot: "focus" | "rest" | "nag") => ({
    focus: null,
    rest: null,
    nag: null,
    [slot]: `/pets/${slot}.png`,
  });

  it("returns the requested slot when it is filled", () => {
    expect(resolveSlot(only("rest"), "rest")).toBe("/pets/rest.png");
  });

  it("falls back to focus, then rest, then nag", () => {
    expect(resolveSlot(only("focus"), "nag")).toBe("/pets/focus.png");
    expect(resolveSlot(only("rest"), "focus")).toBe("/pets/rest.png");
    expect(resolveSlot(only("nag"), "focus")).toBe("/pets/nag.png");
  });

  it("is null when nothing is imported", () => {
    expect(resolveSlot({ focus: null, rest: null, nag: null }, "focus")).toBeNull();
  });
});

describe("integerScale", () => {
  it("picks the largest whole multiple that still fits", () => {
    expect(integerScale(16, 16, 128)).toBe(8);
    expect(integerScale(32, 24, 128)).toBe(4);
    expect(integerScale(50, 50, 128)).toBe(2);
  });

  it("never shrinks below 1:1, even when the image is bigger than the box", () => {
    expect(integerScale(300, 300, 128)).toBe(1);
    expect(integerScale(0, 0, 128)).toBe(128);
  });
});
