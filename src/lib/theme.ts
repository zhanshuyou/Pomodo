export type Accent = "terracotta" | "blue" | "green" | "magenta";
export type Tone = "professional" | "gentle" | "playful";

export const ACCENTS: Record<Accent, string> = {
  terracotta: "oklch(0.63 0.13 40)",
  blue: "oklch(0.58 0.11 250)",
  green: "oklch(0.58 0.12 150)",
  magenta: "oklch(0.55 0.14 320)",
};

export const DEFAULT_ACCENT: Accent = "terracotta";
export const DEFAULT_TONE: Tone = "playful";

/** Reminder category colours, keyed by the template they seed. */
export const REMINDER_COLORS = {
  stand: "oklch(0.63 0.13 40)",
  water: "oklch(0.66 0.09 195)",
  eyes: "oklch(0.7 0.1 145)",
  breathe: "oklch(0.68 0.1 300)",
  stretch: "oklch(0.7 0.12 60)",
  note: "oklch(0.62 0.07 250)",
} as const;

/**
 * Pick the copy variant for the active tone.
 * professional = 克制专业, gentle = 温和陪伴, playful = 俏皮拟人.
 */
export function tone<T>(t: Tone, professional: T, gentle: T, playful: T): T {
  return t === "professional" ? professional : t === "gentle" ? gentle : playful;
}

/**
 * Colour for cell `index` (0 = bottom) of a stats bar, per the design:
 * `oklch(from <accent> calc(l + (0.16 - index * 0.035)) c h)`.
 * Emitted as relative-colour CSS so the browser resolves it against the
 * live accent; the ramp deliberately goes negative for tall bars.
 */
export function barCellColor(accent: string, index: number): string {
  const delta = Math.round((0.16 - index * 0.035) * 1000) / 1000;
  return `oklch(from ${accent} calc(l + ${delta}) c h)`;
}
