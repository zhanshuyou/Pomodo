import { describe, expect, it } from "vitest";
import {
  ACCENTS,
  DEFAULT_ACCENT,
  DEFAULT_TONE,
  REMINDER_COLORS,
  barCellColor,
  tone,
} from "./theme";

describe("ACCENTS", () => {
  it("holds the four accents from the spec", () => {
    expect(ACCENTS).toEqual({
      terracotta: "oklch(0.63 0.13 40)",
      blue: "oklch(0.58 0.11 250)",
      green: "oklch(0.58 0.12 150)",
      magenta: "oklch(0.55 0.14 320)",
    });
  });

  it("defaults to terracotta and the playful tone", () => {
    expect(DEFAULT_ACCENT).toBe("terracotta");
    expect(DEFAULT_TONE).toBe("playful");
  });
});

describe("tone", () => {
  it("selects the matching variant", () => {
    expect(tone("professional", "a", "b", "c")).toBe("a");
    expect(tone("gentle", "a", "b", "c")).toBe("b");
    expect(tone("playful", "a", "b", "c")).toBe("c");
  });

  it("works with non-string values", () => {
    expect(tone("gentle", 1, 2, 3)).toBe(2);
  });
});

describe("barCellColor", () => {
  it("brightens the accent by 0.16 for the bottom cell", () => {
    expect(barCellColor(ACCENTS.terracotta, 0)).toBe(
      "oklch(from oklch(0.63 0.13 40) calc(l + 0.16) c h)",
    );
  });

  it("steps down by 0.035 per cell", () => {
    expect(barCellColor(ACCENTS.terracotta, 2)).toBe(
      "oklch(from oklch(0.63 0.13 40) calc(l + 0.09) c h)",
    );
  });

  it("goes negative past the fifth cell", () => {
    expect(barCellColor(ACCENTS.terracotta, 6)).toBe(
      "oklch(from oklch(0.63 0.13 40) calc(l + -0.05) c h)",
    );
  });
});

describe("REMINDER_COLORS", () => {
  it("holds every reminder category colour from the spec", () => {
    expect(REMINDER_COLORS).toEqual({
      stand: "oklch(0.63 0.13 40)",
      water: "oklch(0.66 0.09 195)",
      eyes: "oklch(0.7 0.1 145)",
      breathe: "oklch(0.68 0.1 300)",
      stretch: "oklch(0.7 0.12 60)",
      note: "oklch(0.62 0.07 250)",
    });
  });
});
