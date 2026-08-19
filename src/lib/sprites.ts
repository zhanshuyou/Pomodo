export type Rgba = [number, number, number, number];

const OKLCH = /^oklch\(\s*([\d.]+)\s+([\d.]+)\s+([\d.]+)\s*(?:\/\s*([\d.]+)\s*)?\)$/;

function parseOklch(css: string): { l: number; c: number; h: number; a: number } {
  const m = OKLCH.exec(css.trim());
  if (!m) throw new Error(`not an oklch() colour: ${css}`);
  return {
    l: parseFloat(m[1]),
    c: parseFloat(m[2]),
    h: parseFloat(m[3]),
    a: m[4] === undefined ? 1 : parseFloat(m[4]),
  };
}

function encodeChannel(linear: number): number {
  const v =
    linear <= 0.0031308
      ? 12.92 * linear
      : 1.055 * Math.pow(Math.max(linear, 0), 1 / 2.4) - 0.055;
  return Math.max(0, Math.min(255, Math.round(v * 255)));
}

/** Convert an `oklch(L C H)` or `oklch(L C H / A)` string to 0-255 sRGB + alpha. */
export function oklchToRgba(css: string): Rgba {
  const { l: L, c: C, h: hDeg, a: alpha } = parseOklch(css);
  const h = (hDeg * Math.PI) / 180;
  const a = C * Math.cos(h);
  const b = C * Math.sin(h);

  const lp = L + 0.3963377774 * a + 0.2158037573 * b;
  const mp = L - 0.1055613458 * a - 0.0638541728 * b;
  const sp = L - 0.0894841775 * a - 1.291485548 * b;

  const l3 = lp * lp * lp;
  const m3 = mp * mp * mp;
  const s3 = sp * sp * sp;

  return [
    encodeChannel(4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3),
    encodeChannel(-1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3),
    encodeChannel(-0.0041960863 * l3 - 0.7034186147 * m3 + 1.707614701 * s3),
    Math.round(alpha * 255),
  ];
}

/** The design's `oklch(from <body> calc(l - 0.12) c h)`, resolved in JS. */
export function shadeOf(body: string): string {
  const { l, c, h } = parseOklch(body);
  const dimmed = Math.max(0, Math.round((l - 0.12) * 1000) / 1000);
  return `oklch(${dimmed} ${c} ${h})`;
}

export const SPRITE_SIZE = 16;

const CAT = [
  "................",
  "..oo........oo..",
  "..obo......obo..",
  "..obbo....obbo..",
  "..obbboooobbbo..",
  "..obbbbbbbbbbo..",
  ".obbbbbbbbbbbbo.",
  ".obbebbbbbbebbo.",
  ".obbbbbbbbbbbbo.",
  ".obbpbwwwwbpbbo.",
  ".obbbbwwwwbbbbo.",
  ".obbbbbwwbbbbbo.",
  "..obbbbbbbbbbo..",
  "..obboooooobbo..",
  "..oo.o....o.oo..",
  "................",
] as const;

const BIRD = [
  "................",
  ".....oooo.......",
  "....obbbbo......",
  "...obbbbbbo.....",
  "...obebbbbo.oo..",
  "...obbbbbboobo..",
  "..obbbbbbbbbbo..",
  "..obbbbbbbbbbo..",
  ".obbbbwwwwbbbo..",
  ".obbbwwwwwwbbo..",
  ".obbbbwwwwbbbo..",
  "..obbbbbbbbbo...",
  "...obbbbbbbo....",
  "....oo.oo.......",
  "....p...p.......",
  "................",
] as const;

const FROG = [
  "................",
  "..oo......oo....",
  ".obwo....obwo...",
  ".obeo....obeo...",
  ".obbbooooobbbo..",
  "obbbbbbbbbbbbbo.",
  "obbbbbbbbbbbbbo.",
  "obbboowwwwoobbo.",
  "obbbbbbbbbbbbbo.",
  ".obbbbbbbbbbbo..",
  "..obbbbbbbbbo...",
  "..obboooooobo...",
  ".ooo......ooo...",
  "op.o......o.po..",
  "................",
  "................",
] as const;

const GHOST = [
  "................",
  ".....oooo.......",
  "...oobbbboo.....",
  "..obbbbbbbbo....",
  "..obbbbbbbbo....",
  ".obbebbbbebbo...",
  ".obbbbbbbbbbo...",
  ".obbbbwwwwbbo...",
  ".obbbbbbbbbbo...",
  ".obbbbbbbbbbo...",
  ".obbbbbbbbbbo...",
  ".obbbbbbbbbbo...",
  ".obobbobbobbo...",
  ".o.oo.oo.oo.o...",
  "................",
  "................",
] as const;

const BEAR = [
  "................",
  ".ooo......ooo...",
  "obbbo....obbbo..",
  "obpbo....obpbo..",
  ".obbboooobbbo...",
  ".obbbbbbbbbbo...",
  "obbbbbbbbbbbbo..",
  "obbebbbbbbebbo..",
  "obbbbbbbbbbbbo..",
  "obbbowwwwobbbo..",
  "obbbwwwwwwbbbo..",
  ".obbbwwwwbbbo...",
  ".obbbbbbbbbbo...",
  "..oboooooobo....",
  "..o.o....o.o....",
  "................",
] as const;

const SLIME = [
  "................",
  "................",
  ".....oooo.......",
  "...oobbbboo.....",
  "..obbbbbbbbo....",
  ".obbbbbbbbbbo...",
  ".obbebbbbebbo...",
  "obbbbbbbbbbbbo..",
  "obbbbwwwwbbbbo..",
  "obbbbbwwbbbbbo..",
  "obbbbbbbbbbbbo..",
  "obbbbbbbbbbbbo..",
  ".obbbbbbbbbbo...",
  "..oooooooooo....",
  "................",
  "................",
] as const;

export type PetId = 0 | 1 | 2 | 3 | 4 | 5;

export interface PetDef {
  id: PetId;
  name: string;
  map: readonly string[];
  body: string;
  unlockedByDefault: boolean;
}

export const PETS: readonly PetDef[] = [
  { id: 0, name: "MOCHI", map: CAT, body: "oklch(0.84 0.09 80)", unlockedByDefault: true },
  { id: 1, name: "PUDDING", map: SLIME, body: "oklch(0.82 0.08 195)", unlockedByDefault: true },
  { id: 2, name: "TOFU", map: FROG, body: "oklch(0.82 0.1 145)", unlockedByDefault: true },
  { id: 3, name: "BEAN", map: BEAR, body: "oklch(0.72 0.06 55)", unlockedByDefault: true },
  { id: 4, name: "PEEP", map: BIRD, body: "oklch(0.85 0.11 95)", unlockedByDefault: false },
  { id: 5, name: "BOO", map: GHOST, body: "oklch(0.9 0.02 280)", unlockedByDefault: false },
];

/** Body colour used to render a still-locked pet in the picker. */
export const LOCKED_BODY = "oklch(0.86 0.006 70)";

const FIXED = {
  o: "oklch(0.26 0.02 60)",
  e: "oklch(0.2 0.015 60)",
  w: "oklch(0.98 0.006 80)",
  p: "oklch(0.78 0.11 20)",
} as const;

const paletteCache = new Map<string, Record<string, Rgba>>();

/** Resolve the seven sprite palette entries for one body colour. Memoised. */
export function paletteFor(body: string): Record<string, Rgba> {
  const hit = paletteCache.get(body);
  if (hit) return hit;
  const pal: Record<string, Rgba> = {
    o: oklchToRgba(FIXED.o),
    e: oklchToRgba(FIXED.e),
    w: oklchToRgba(FIXED.w),
    p: oklchToRgba(FIXED.p),
    b: oklchToRgba(body),
    s: oklchToRgba(shadeOf(body)),
  };
  paletteCache.set(body, pal);
  return pal;
}
