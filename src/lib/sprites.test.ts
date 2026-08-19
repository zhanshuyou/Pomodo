import { describe, expect, it } from "vitest";
import { oklchToRgba, shadeOf } from "./sprites";

describe("oklchToRgba", () => {
  it("converts pure white", () => {
    expect(oklchToRgba("oklch(1 0 0)")).toEqual([255, 255, 255, 255]);
  });

  it("converts pure black", () => {
    expect(oklchToRgba("oklch(0 0 0)")).toEqual([0, 0, 0, 255]);
  });

  it("converts the default accent to its sRGB neighbourhood", () => {
    const [r, g, b, a] = oklchToRgba("oklch(0.63 0.13 40)");
    expect(a).toBe(255);
    expect(r).toBeGreaterThan(g);
    expect(g).toBeGreaterThan(b);
    expect(r).toBeGreaterThan(180);
    expect(r).toBeLessThan(215);
  });

  it("reads the alpha channel", () => {
    expect(oklchToRgba("oklch(0.24 0.012 60 / 0.5)")[3]).toBe(128);
  });

  it("clamps out-of-gamut channels into 0..255", () => {
    const px = oklchToRgba("oklch(0.9 0.4 140)");
    for (const c of px) {
      expect(c).toBeGreaterThanOrEqual(0);
      expect(c).toBeLessThanOrEqual(255);
    }
  });

  it("rejects a non-oklch colour", () => {
    expect(() => oklchToRgba("#ff0000")).toThrow();
  });
});

describe("shadeOf", () => {
  it("darkens lightness by 0.12 and keeps chroma and hue", () => {
    expect(shadeOf("oklch(0.84 0.09 80)")).toBe("oklch(0.72 0.09 80)");
  });

  it("never goes below zero lightness", () => {
    expect(shadeOf("oklch(0.05 0.02 60)")).toBe("oklch(0 0.02 60)");
  });
});
