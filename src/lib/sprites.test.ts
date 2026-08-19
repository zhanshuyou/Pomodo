import { describe, expect, it } from "vitest";
import {
  LOCKED_BODY,
  PETS,
  SPRITE_SIZE,
  oklchToRgba,
  paletteFor,
  rasterize,
  shadeOf,
} from "./sprites";

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

describe("PETS", () => {
  it("has the six pets from the spec in order", () => {
    expect(PETS.map((p) => p.name)).toEqual([
      "MOCHI",
      "PUDDING",
      "TOFU",
      "BEAN",
      "PEEP",
      "BOO",
    ]);
  });

  it("locks only PEEP and BOO by default", () => {
    expect(PETS.filter((p) => !p.unlockedByDefault).map((p) => p.name)).toEqual([
      "PEEP",
      "BOO",
    ]);
  });

  it("gives every pet a square 16x16 map", () => {
    for (const pet of PETS) {
      expect(pet.map, pet.name).toHaveLength(SPRITE_SIZE);
      for (const row of pet.map) {
        expect(row, `${pet.name}: ${row}`).toHaveLength(SPRITE_SIZE);
      }
    }
  });

  it("uses only known palette characters", () => {
    for (const pet of PETS) {
      for (const row of pet.map) {
        expect(row).toMatch(/^[.osbewp]+$/);
      }
    }
  });

  it("assigns each pet a distinct id matching its index", () => {
    expect(PETS.map((p) => p.id)).toEqual([0, 1, 2, 3, 4, 5]);
  });
});

describe("paletteFor", () => {
  it("maps b to the body colour and s to its shade", () => {
    const pal = paletteFor("oklch(0.84 0.09 80)");
    expect(pal.b).toEqual(oklchToRgba("oklch(0.84 0.09 80)"));
    expect(pal.s).toEqual(oklchToRgba("oklch(0.72 0.09 80)"));
  });

  it("uses the fixed outline, eye, white and blush colours", () => {
    const pal = paletteFor(LOCKED_BODY);
    expect(pal.o).toEqual(oklchToRgba("oklch(0.26 0.02 60)"));
    expect(pal.e).toEqual(oklchToRgba("oklch(0.2 0.015 60)"));
    expect(pal.w).toEqual(oklchToRgba("oklch(0.98 0.006 80)"));
    expect(pal.p).toEqual(oklchToRgba("oklch(0.78 0.11 20)"));
  });
});

describe("rasterize", () => {
  const mochi = PETS[0];

  it("produces a 16x16 RGBA buffer", () => {
    expect(rasterize(mochi.map, mochi.body)).toHaveLength(
      SPRITE_SIZE * SPRITE_SIZE * 4,
    );
  });

  it("leaves '.' pixels fully transparent", () => {
    const buf = rasterize(mochi.map, mochi.body);
    // row 0 of CAT is all dots
    for (let x = 0; x < SPRITE_SIZE; x++) {
      expect(buf[x * 4 + 3]).toBe(0);
    }
  });

  it("writes the outline colour at a known 'o' pixel", () => {
    const buf = rasterize(mochi.map, mochi.body);
    // CAT row 1 is "..oo........oo..", so (2,1) is 'o'
    const i = (1 * SPRITE_SIZE + 2) * 4;
    const pal = paletteFor(mochi.body);
    expect([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]).toEqual(pal.o);
  });

  it("writes the body colour at a known 'b' pixel", () => {
    const buf = rasterize(mochi.map, mochi.body);
    // CAT row 5 is "..obbbbbbbbbbo..", so (3,5) is 'b'
    const i = (5 * SPRITE_SIZE + 3) * 4;
    const pal = paletteFor(mochi.body);
    expect([buf[i], buf[i + 1], buf[i + 2], buf[i + 3]]).toEqual(pal.b);
  });

  it("recolours the same map when given a different body", () => {
    const a = rasterize(mochi.map, mochi.body);
    const b = rasterize(mochi.map, LOCKED_BODY);
    expect(a).not.toEqual(b);
  });
});
