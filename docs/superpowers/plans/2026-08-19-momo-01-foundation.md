# Pomodo 01 — Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Tauri scaffold with Pomodo's visual foundation — design tokens, vendored fonts, the pixel sprite renderer, the shared component kit, and a five-entry Vite build — all verifiable in a plain browser without Tauri.

**Architecture:** Pure frontend. Two dependency-free modules (`sprites.ts`, `theme.ts`) hold every value copied from the design and are unit-tested with vitest. Svelte components consume them. A dev-only gallery route renders every component so the work can be eyeballed against the artboards before any Rust exists.

**Tech Stack:** Svelte 5 (runes), TypeScript, Vite 8, vitest, IBM Plex Sans/Mono + Silkscreen (vendored WOFF2).

**Spec:** `docs/superpowers/specs/2026-08-19-momo-design.md`

## Global Constraints

- Every colour is `oklch()` copied verbatim from the spec §3.1. Never substitute a hex approximation.
- UI language is Chinese. Every user-facing string comes from spec §5 verbatim, including punctuation (`＋` is fullwidth, `−` in `较上周 −4` is U+2212 minus, not a hyphen).
- Fonts are vendored under `src/assets/fonts/` and loaded with local `@font-face`. No `fonts.googleapis.com` request may survive into the app.
- Sprite character maps in spec §4.1 are copied verbatim — 16 strings of 16 characters each, per pet.
- All animation must be suppressed under `@media (prefers-reduced-motion: reduce)`.
- Existing CI must stay green: `npm run check` (svelte-check + tsc) and `npm run build`.
- Node 18+, no new runtime dependencies beyond what is listed in this plan.

---

## File Structure

| Path | Responsibility |
| --- | --- |
| `src/lib/sprites.ts` | Pet character maps, palette resolution, oklch→sRGB, 16×16 `ImageData` production |
| `src/lib/theme.ts` | Accent + tone types, design tokens as TS constants, `tone()` copy selector, stats colour ramp |
| `src/styles/tokens.css` | CSS custom properties for every token in spec §3.1 |
| `src/styles/fonts.css` | `@font-face` declarations for the three vendored families |
| `src/styles/base.css` | Reset, body defaults, the five keyframes, reduced-motion block |
| `src/lib/components/PetCanvas.svelte` | Canvas sprite renderer |
| `src/lib/components/PixelButton.svelte` | Primary / secondary button |
| `src/lib/components/Toggle.svelte` | 38×22 switch |
| `src/lib/components/Chip.svelte` | Selectable chip (intervals, templates, flags) |
| `src/lib/components/StatBar.svelte` | Label / value / progress track |
| `src/lib/components/SpeechBubble.svelte` | Tail-cornered bubble |
| `src/lib/components/TitleBar.svelte` | Traffic lights + title + slot |
| `src/lib/components/SectionHeading.svelte` | Silkscreen number + heading + caption |
| `src/routes/gallery/App.svelte` | Dev-only component gallery |
| `index.html`, `prefs.html`, `tray.html`, `pet.html`, `overlay.html`, `gallery.html` | Vite entry points |
| `src/lib/sprites.test.ts`, `src/lib/theme.test.ts` | vitest suites |

Deleted: `src/App.svelte`, `src/app.css`, `src/assets/svelte.svg`, `public/vite.svg`, `public/tauri.svg`.

---

### Task 1: Test tooling and scaffold removal

**Files:**
- Modify: `package.json`
- Create: `vitest.config.ts`
- Delete: `src/App.svelte`, `src/app.css`, `src/assets/svelte.svg`, `public/vite.svg`, `public/tauri.svg`
- Modify: `src/main.ts`, `index.html`

**Interfaces:**
- Consumes: nothing.
- Produces: `npm test` runs vitest; `src/main.ts` mounts `src/routes/main/App.svelte`.

- [ ] **Step 1: Install vitest**

```bash
npm install -D vitest@^3 jsdom@^25
```

- [ ] **Step 2: Create the vitest config**

Create `vitest.config.ts`:

```ts
import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte({ hot: false })],
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
```

- [ ] **Step 3: Add the test script**

In `package.json`, add to `"scripts"`:

```json
"test": "vitest run",
"test:watch": "vitest"
```

- [ ] **Step 4: Delete the scaffold files**

```bash
git rm src/App.svelte src/app.css src/assets/svelte.svg public/vite.svg public/tauri.svg
```

- [ ] **Step 5: Point the entry at the new main route**

Create `src/routes/main/App.svelte` as a temporary stub:

```svelte
<script lang="ts"></script>

<main>Pomodo</main>
```

Replace `src/main.ts`:

```ts
import { mount } from "svelte";
import "./styles/fonts.css";
import "./styles/tokens.css";
import "./styles/base.css";
import App from "./routes/main/App.svelte";

export default mount(App, { target: document.getElementById("app")! });
```

The three CSS files do not exist yet — create them empty for now:

```bash
mkdir -p src/styles && touch src/styles/fonts.css src/styles/tokens.css src/styles/base.css
```

Update `index.html` to drop the vite favicon reference:

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Pomodo</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 6: Verify the build still works**

Run: `npm run check && npm run build`
Expected: both succeed with no errors.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "chore: remove Tauri scaffold, add vitest"
```

---

### Task 2: oklch → sRGB conversion

**Files:**
- Create: `src/lib/sprites.ts`
- Test: `src/lib/sprites.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `export type Rgba = [number, number, number, number]`
  - `export function oklchToRgba(css: string): Rgba` — accepts `oklch(L C H)` and `oklch(L C H / A)`, L/C unitless, H in degrees, A in 0..1. Returns 0–255 integers.
  - `export function shadeOf(body: string): string` — returns the body colour with L reduced by 0.12, as an `oklch()` string.

The sprite palette uses `oklch(from <body> calc(l - 0.12) c h)` in the design. Relative colour syntax cannot be read back from JS, so both functions are needed to resolve it ourselves.

- [ ] **Step 1: Write the failing test**

Create `src/lib/sprites.test.ts`:

```ts
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- src/lib/sprites.test.ts`
Expected: FAIL — cannot resolve `./sprites`.

- [ ] **Step 3: Write the implementation**

Create `src/lib/sprites.ts`:

```ts
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
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- src/lib/sprites.test.ts`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/sprites.ts src/lib/sprites.test.ts
git commit -m "feat: add oklch to sRGB conversion for sprite palettes"
```

---

### Task 3: Pet character maps and palette

**Files:**
- Modify: `src/lib/sprites.ts`
- Test: `src/lib/sprites.test.ts`

**Interfaces:**
- Consumes: `oklchToRgba`, `shadeOf`, `Rgba` from Task 2.
- Produces:
  - `export type PetId = 0 | 1 | 2 | 3 | 4 | 5`
  - `export interface PetDef { id: PetId; name: string; map: readonly string[]; body: string; unlockedByDefault: boolean }`
  - `export const PETS: readonly PetDef[]` — six entries in spec §4.1 order
  - `export const LOCKED_BODY = "oklch(0.86 0.006 70)"`
  - `export function paletteFor(body: string): Record<string, Rgba>` — keys `o s b e w p`
  - `export const SPRITE_SIZE = 16`

- [ ] **Step 1: Write the failing test**

Append to `src/lib/sprites.test.ts`:

```ts
import { LOCKED_BODY, PETS, SPRITE_SIZE, paletteFor } from "./sprites";

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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- src/lib/sprites.test.ts`
Expected: FAIL — `PETS` is not exported.

- [ ] **Step 3: Write the implementation**

Append to `src/lib/sprites.ts`. The six maps are copied verbatim from the design's script block.

```ts
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
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- src/lib/sprites.test.ts`
Expected: PASS, 15 tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/sprites.ts src/lib/sprites.test.ts
git commit -m "feat: add pet character maps and palette resolution"
```

---

### Task 4: Sprite rasterisation

**Files:**
- Modify: `src/lib/sprites.ts`
- Test: `src/lib/sprites.test.ts`

**Interfaces:**
- Consumes: `PETS`, `paletteFor`, `SPRITE_SIZE`, `Rgba`.
- Produces: `export function rasterize(map: readonly string[], body: string): Uint8ClampedArray` — a 16×16 RGBA buffer, 1024 bytes, `.` pixels fully transparent.

Returning the raw buffer rather than `ImageData` keeps the module testable under jsdom, where `ImageData` construction is unreliable. `PetCanvas` wraps it.

- [ ] **Step 1: Write the failing test**

Append to `src/lib/sprites.test.ts`:

```ts
import { rasterize } from "./sprites";

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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- src/lib/sprites.test.ts`
Expected: FAIL — `rasterize` is not exported.

- [ ] **Step 3: Write the implementation**

Append to `src/lib/sprites.ts`:

```ts
/**
 * Rasterise a character map into a 16x16 RGBA buffer.
 * '.' and any unknown character become fully transparent.
 */
export function rasterize(
  map: readonly string[],
  body: string,
): Uint8ClampedArray {
  const pal = paletteFor(body);
  const buf = new Uint8ClampedArray(SPRITE_SIZE * SPRITE_SIZE * 4);
  for (let y = 0; y < SPRITE_SIZE; y++) {
    const row = map[y] ?? "";
    for (let x = 0; x < SPRITE_SIZE; x++) {
      const px = pal[row[x]];
      if (!px) continue;
      const i = (y * SPRITE_SIZE + x) * 4;
      buf[i] = px[0];
      buf[i + 1] = px[1];
      buf[i + 2] = px[2];
      buf[i + 3] = px[3];
    }
  }
  return buf;
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- src/lib/sprites.test.ts`
Expected: PASS, 20 tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/sprites.ts src/lib/sprites.test.ts
git commit -m "feat: rasterize pet character maps to RGBA buffers"
```

---

### Task 5: Theme tokens and the tone selector

**Files:**
- Create: `src/lib/theme.ts`
- Test: `src/lib/theme.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `export type Accent = "terracotta" | "blue" | "green" | "magenta"`
  - `export type Tone = "professional" | "gentle" | "playful"`
  - `export const ACCENTS: Record<Accent, string>`
  - `export const DEFAULT_ACCENT: Accent`, `export const DEFAULT_TONE: Tone`
  - `export function tone<T>(t: Tone, professional: T, gentle: T, playful: T): T`
  - `export function barCellColor(accent: string, index: number): string` — the stats ramp
  - `export const REMINDER_COLORS: Record<string, string>`

- [ ] **Step 1: Write the failing test**

Create `src/lib/theme.test.ts`:

```ts
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
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `npm test -- src/lib/theme.test.ts`
Expected: FAIL — cannot resolve `./theme`.

- [ ] **Step 3: Write the implementation**

Create `src/lib/theme.ts`:

```ts
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
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `npm test -- src/lib/theme.test.ts`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit**

```bash
git add src/lib/theme.ts src/lib/theme.test.ts
git commit -m "feat: add accent, tone and stats ramp theme helpers"
```

---

### Task 6: Vendored fonts and CSS token layer

**Files:**
- Create: `src/assets/fonts/` (WOFF2 files)
- Modify: `src/styles/fonts.css`, `src/styles/tokens.css`, `src/styles/base.css`

**Interfaces:**
- Consumes: the colour values in spec §3.1, the keyframes in §3.3.
- Produces: CSS custom properties `--bg --card --surface-2 --ink --dim --faint --line --line-soft --track --good --accent`, families `--font-ui --font-mono --font-pixel`, and the five keyframes.

- [ ] **Step 1: Vendor the font files**

Download the Latin subsets and place them in `src/assets/fonts/`:

```bash
mkdir -p src/assets/fonts
curl -L -o src/assets/fonts/ibm-plex-sans-400.woff2 \
  "https://cdn.jsdelivr.net/fontsource/fonts/ibm-plex-sans@latest/latin-400-normal.woff2"
curl -L -o src/assets/fonts/ibm-plex-sans-500.woff2 \
  "https://cdn.jsdelivr.net/fontsource/fonts/ibm-plex-sans@latest/latin-500-normal.woff2"
curl -L -o src/assets/fonts/ibm-plex-sans-600.woff2 \
  "https://cdn.jsdelivr.net/fontsource/fonts/ibm-plex-sans@latest/latin-600-normal.woff2"
curl -L -o src/assets/fonts/ibm-plex-mono-400.woff2 \
  "https://cdn.jsdelivr.net/fontsource/fonts/ibm-plex-mono@latest/latin-400-normal.woff2"
curl -L -o src/assets/fonts/ibm-plex-mono-500.woff2 \
  "https://cdn.jsdelivr.net/fontsource/fonts/ibm-plex-mono@latest/latin-500-normal.woff2"
curl -L -o src/assets/fonts/silkscreen-400.woff2 \
  "https://cdn.jsdelivr.net/fontsource/fonts/silkscreen@latest/latin-400-normal.woff2"
```

Verify each file is a real WOFF2 and not an HTML error page:

```bash
file src/assets/fonts/*.woff2
```
Expected: every line reports `Web Open Font Format (Version 2)`.

CJK glyphs are intentionally not vendored — `PingFang SC` covers them on macOS and the
fallback stack covers other platforms. Both IBM Plex and Silkscreen are OFL-licensed;
add `src/assets/fonts/OFL.txt` with the licence text.

- [ ] **Step 2: Write the @font-face declarations**

Replace `src/styles/fonts.css`:

```css
@font-face {
  font-family: "IBM Plex Sans";
  src: url("../assets/fonts/ibm-plex-sans-400.woff2") format("woff2");
  font-weight: 400;
  font-display: block;
}
@font-face {
  font-family: "IBM Plex Sans";
  src: url("../assets/fonts/ibm-plex-sans-500.woff2") format("woff2");
  font-weight: 500;
  font-display: block;
}
@font-face {
  font-family: "IBM Plex Sans";
  src: url("../assets/fonts/ibm-plex-sans-600.woff2") format("woff2");
  font-weight: 600;
  font-display: block;
}
@font-face {
  font-family: "IBM Plex Mono";
  src: url("../assets/fonts/ibm-plex-mono-400.woff2") format("woff2");
  font-weight: 400;
  font-display: block;
}
@font-face {
  font-family: "IBM Plex Mono";
  src: url("../assets/fonts/ibm-plex-mono-500.woff2") format("woff2");
  font-weight: 500;
  font-display: block;
}
@font-face {
  font-family: "Silkscreen";
  src: url("../assets/fonts/silkscreen-400.woff2") format("woff2");
  font-weight: 400;
  font-display: block;
}
```

- [ ] **Step 3: Write the token layer**

Replace `src/styles/tokens.css`:

```css
:root {
  --bg: oklch(0.95 0.008 70);
  --card: oklch(0.99 0.004 80);
  --surface-2: oklch(0.965 0.006 70);
  --ink: oklch(0.24 0.012 60);
  --dim: oklch(0.5 0.012 60);
  --faint: oklch(0.6 0.012 60);
  --line: oklch(0.88 0.008 70);
  --line-soft: oklch(0.93 0.008 70);
  --track: oklch(0.9 0.008 70);
  --good: oklch(0.55 0.11 145);

  --accent: oklch(0.63 0.13 40);

  --font-ui: "IBM Plex Sans", "PingFang SC", "Helvetica Neue", sans-serif;
  --font-mono: "IBM Plex Mono", ui-monospace, monospace;
  --font-pixel: "Silkscreen", "IBM Plex Mono", monospace;

  --radius-window: 16px;
  --radius-card: 13px;
  --radius-control: 11px;
  --radius-chip: 9px;

  --inset-press: inset 0 -3px 0 oklch(0.24 0.012 60 / 0.18);
  --shadow-window: 0 28px 56px -28px oklch(0.24 0.012 60 / 0.45);
  --shadow-bubble: 0 8px 20px -12px oklch(0.24 0.012 60 / 0.5);
}

:root[data-accent="blue"] { --accent: oklch(0.58 0.11 250); }
:root[data-accent="green"] { --accent: oklch(0.58 0.12 150); }
:root[data-accent="magenta"] { --accent: oklch(0.55 0.14 320); }
```

- [ ] **Step 4: Write the base layer and keyframes**

Replace `src/styles/base.css`:

```css
*,
*::before,
*::after {
  box-sizing: border-box;
}

html,
body {
  margin: 0;
  padding: 0;
  background: var(--bg);
}

body {
  font-family: var(--font-ui);
  color: var(--ink);
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
}

button {
  font-family: inherit;
}

@keyframes momo-bob {
  0%,
  100% { transform: translateY(0) rotate(-1.5deg); }
  50% { transform: translateY(-6px) rotate(1.5deg); }
}

@keyframes momo-hop {
  0%,
  60%,
  100% { transform: translateY(0); }
  25% { transform: translateY(-14px); }
  40% { transform: translateY(-3px); }
}

@keyframes momo-rise {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: none; }
}

@keyframes momo-pulse {
  0%,
  100% { opacity: 0.4; transform: scale(1); }
  50% { opacity: 0.06; transform: scale(1.45); }
}

@keyframes momo-sway {
  0%,
  100% { transform: rotate(-3deg); }
  50% { transform: rotate(3deg); }
}

@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.001ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.001ms !important;
  }
}
```

- [ ] **Step 5: Verify no network font request remains**

Run: `grep -rn "fonts.googleapis\|fonts.gstatic" src/ index.html`
Expected: no matches.

- [ ] **Step 6: Commit**

```bash
git add src/assets/fonts src/styles
git commit -m "feat: vendor fonts and add the design token layer"
```

---

### Task 7: PetCanvas component

**Files:**
- Create: `src/lib/components/PetCanvas.svelte`

**Interfaces:**
- Consumes: `rasterize`, `SPRITE_SIZE` from `src/lib/sprites.ts`.
- Produces: a component with props
  `{ map: readonly string[]; body: string; scale?: number; anim?: "none" | "bob" | "hop" | "sway"; alt?: string }`.
  Default `scale = 8`, `anim = "none"`. Renders a `16 * scale` square canvas.

- [ ] **Step 1: Write the component**

Create `src/lib/components/PetCanvas.svelte`:

```svelte
<script lang="ts">
  import { SPRITE_SIZE, rasterize } from "../sprites";

  interface Props {
    map: readonly string[];
    body: string;
    scale?: number;
    anim?: "none" | "bob" | "hop" | "sway";
    alt?: string;
  }

  let { map, body, scale = 8, anim = "none", alt = "" }: Props = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);

  const size = $derived(SPRITE_SIZE * scale);

  // Redraw only when the pixels or the size actually change — never per frame.
  // The bob/hop/sway motion is a CSS transform on the element, not a repaint.
  $effect(() => {
    const el = canvas;
    if (!el) return;
    const buf = rasterize(map, body);
    const px = size;

    const off = document.createElement("canvas");
    off.width = SPRITE_SIZE;
    off.height = SPRITE_SIZE;
    const octx = off.getContext("2d");
    if (!octx) return;
    octx.putImageData(new ImageData(buf, SPRITE_SIZE, SPRITE_SIZE), 0, 0);

    const ctx = el.getContext("2d");
    if (!ctx) return;
    ctx.clearRect(0, 0, px, px);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(off, 0, 0, px, px);
  });
</script>

<canvas
  bind:this={canvas}
  width={size}
  height={size}
  style:width="{size}px"
  style:height="{size}px"
  class="pet pet--{anim}"
  role="img"
  aria-label={alt}
></canvas>

<style>
  .pet {
    display: block;
    image-rendering: pixelated;
  }
  .pet--bob {
    animation: momo-bob 4.2s ease-in-out infinite;
  }
  .pet--hop {
    animation: momo-hop 1.6s ease-in-out infinite;
  }
  .pet--sway {
    animation: momo-sway 3s ease-in-out infinite;
  }
</style>
```

- [ ] **Step 2: Verify it type-checks**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/PetCanvas.svelte
git commit -m "feat: add PetCanvas sprite renderer"
```

---

### Task 8: Pixel component kit

**Files:**
- Create: `src/lib/components/PixelButton.svelte`, `Toggle.svelte`, `Chip.svelte`, `StatBar.svelte`, `SpeechBubble.svelte`, `TitleBar.svelte`, `SectionHeading.svelte`

**Interfaces:**
- Consumes: the CSS tokens from Task 6.
- Produces:
  - `PixelButton` — `{ variant?: "primary" | "secondary"; onclick?: () => void; children }`
  - `Toggle` — `{ checked: boolean; onchange: (v: boolean) => void; label?: string }`
  - `Chip` — `{ selected?: boolean; dot?: string; onclick?: () => void; children }`
  - `StatBar` — `{ name: string; value: string; pct: number; color: string }` (`pct` is 0–100)
  - `SpeechBubble` — `{ tail?: "bottom-left" | "none"; maxWidth?: number; children }`
  - `TitleBar` — `{ title: string; children? }`
  - `SectionHeading` — `{ index: string; title: string; caption?: string }`

- [ ] **Step 1: Write PixelButton**

Create `src/lib/components/PixelButton.svelte`:

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "primary" | "secondary";
    onclick?: () => void;
    children: Snippet;
  }

  let { variant = "primary", onclick, children }: Props = $props();
</script>

<button class="btn btn--{variant}" type="button" {onclick}>{@render children()}</button>

<style>
  .btn {
    padding: 14px 20px;
    border-radius: 12px;
    font-size: 15px;
    cursor: pointer;
    border: none;
  }
  .btn--primary {
    background: var(--accent);
    color: var(--card);
    font-weight: 600;
    box-shadow: var(--inset-press);
  }
  .btn--primary:hover {
    filter: brightness(1.07);
  }
  .btn--secondary {
    background: var(--card);
    color: oklch(0.4 0.012 60);
    border: 1px solid var(--line);
  }
  .btn--secondary:hover {
    background: oklch(0.96 0.006 70);
  }
</style>
```

- [ ] **Step 2: Write Toggle**

Create `src/lib/components/Toggle.svelte`:

```svelte
<script lang="ts">
  interface Props {
    checked: boolean;
    onchange: (value: boolean) => void;
    label?: string;
  }

  let { checked, onchange, label = "" }: Props = $props();
</script>

<button
  class="switch"
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  onclick={(e) => {
    e.stopPropagation();
    onchange(!checked);
  }}
>
  <span class="knob"></span>
</button>

<style>
  .switch {
    width: 38px;
    height: 22px;
    border: none;
    border-radius: 13px;
    padding: 2px;
    display: flex;
    justify-content: flex-start;
    background: oklch(0.86 0.008 70);
    cursor: pointer;
    flex: none;
  }
  .switch[aria-checked="true"] {
    background: var(--accent);
    justify-content: flex-end;
  }
  .knob {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--card);
    box-shadow: 0 1px 3px oklch(0.24 0.012 60 / 0.3);
  }
</style>
```

- [ ] **Step 3: Write Chip**

Create `src/lib/components/Chip.svelte`:

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    selected?: boolean;
    dot?: string;
    onclick?: () => void;
    children: Snippet;
  }

  let { selected = false, dot, onclick, children }: Props = $props();
</script>

<button class="chip" class:selected type="button" {onclick}>
  {#if dot}<span class="dot" style:background={dot}></span>{/if}
  {@render children()}
</button>

<style>
  .chip {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 7px 11px;
    border: 1px solid var(--line);
    border-radius: var(--radius-chip);
    background: var(--card);
    color: var(--dim);
    font-size: 12.5px;
    cursor: pointer;
  }
  .chip:hover {
    background: oklch(0.96 0.006 70);
  }
  .chip.selected {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
    color: var(--ink);
  }
  .dot {
    width: 7px;
    height: 7px;
    flex: none;
  }
</style>
```

- [ ] **Step 4: Write StatBar**

Create `src/lib/components/StatBar.svelte`:

```svelte
<script lang="ts">
  interface Props {
    name: string;
    value: string;
    pct: number;
    color: string;
  }

  let { name, value, pct, color }: Props = $props();
</script>

<div class="stat">
  <div class="row">
    <span>{name}</span>
    <span class="value">{value}</span>
  </div>
  <div class="track">
    <div class="fill" style:width="{Math.max(0, Math.min(100, pct))}%" style:background={color}></div>
  </div>
</div>

<style>
  .stat {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    display: flex;
    justify-content: space-between;
    font-size: 12.5px;
  }
  .value {
    font-family: var(--font-mono);
    color: var(--dim);
  }
  .track {
    height: 6px;
    border-radius: 3px;
    background: var(--line-soft);
    overflow: hidden;
  }
  .fill {
    height: 100%;
  }
</style>
```

- [ ] **Step 5: Write SpeechBubble**

Create `src/lib/components/SpeechBubble.svelte`:

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    tail?: "bottom-left" | "none";
    maxWidth?: number;
    children: Snippet;
  }

  let { tail = "bottom-left", maxWidth, children }: Props = $props();
</script>

<div
  class="bubble"
  class:tailed={tail === "bottom-left"}
  style:max-width={maxWidth ? `${maxWidth}px` : undefined}
>
  {@render children()}
</div>

<style>
  .bubble {
    padding: 11px 16px;
    border-radius: 13px;
    background: var(--card);
    box-shadow: var(--shadow-bubble);
    font-size: 14px;
    line-height: 1.45;
  }
  .bubble.tailed {
    border-bottom-left-radius: 4px;
  }
</style>
```

- [ ] **Step 6: Write TitleBar**

Create `src/lib/components/TitleBar.svelte`:

```svelte
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    children?: Snippet;
  }

  let { title, children }: Props = $props();
</script>

<div class="bar" data-tauri-drag-region>
  <span class="light" style:background="oklch(0.72 0.15 25)"></span>
  <span class="light" style:background="oklch(0.82 0.13 85)"></span>
  <span class="light" style:background="oklch(0.78 0.14 145)"></span>
  <span class="title">{title}</span>
  {#if children}{@render children()}{/if}
</div>

<style>
  .bar {
    height: 46px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 16px;
    border-bottom: 1px solid oklch(0.91 0.008 70);
    background: var(--surface-2);
  }
  .light {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    flex: none;
  }
  .title {
    margin-left: 14px;
    font-size: 13px;
    font-weight: 600;
    color: oklch(0.42 0.012 60);
  }
</style>
```

- [ ] **Step 7: Write SectionHeading**

Create `src/lib/components/SectionHeading.svelte`:

```svelte
<script lang="ts">
  interface Props {
    index: string;
    title: string;
    caption?: string;
  }

  let { index, title, caption }: Props = $props();
</script>

<div class="heading">
  <span class="index">{index}</span>
  <h2>{title}</h2>
  {#if caption}<span class="caption">{caption}</span>{/if}
</div>

<style>
  .heading {
    display: flex;
    align-items: baseline;
    gap: 16px;
  }
  .index {
    font-family: var(--font-pixel);
    font-size: 12px;
    letter-spacing: 0.14em;
    color: var(--accent);
  }
  h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 600;
  }
  .caption {
    font-size: 14px;
    color: var(--dim);
  }
</style>
```

- [ ] **Step 8: Verify**

Run: `npm run check`
Expected: no errors.

- [ ] **Step 9: Commit**

```bash
git add src/lib/components
git commit -m "feat: add the pixel component kit"
```

---

### Task 9: Multi-entry build and the dev gallery

**Files:**
- Modify: `vite.config.ts`
- Create: `prefs.html`, `tray.html`, `pet.html`, `overlay.html`, `gallery.html`
- Create: `src/entries/prefs.ts`, `tray.ts`, `pet.ts`, `overlay.ts`, `gallery.ts`
- Create: `src/routes/prefs/App.svelte`, `src/routes/tray/App.svelte`, `src/routes/pet/App.svelte`, `src/routes/overlay/App.svelte`, `src/routes/gallery/App.svelte`

**Interfaces:**
- Consumes: every component from Tasks 7–8, `PETS` from Task 3, `ACCENTS`/`tone` from Task 5.
- Produces: `npm run build` emits six HTML entry points into `dist/`. Later plans attach Tauri windows to them.

- [ ] **Step 1: Add the entry points to the Vite config**

In `vite.config.ts`, add a `build.rollupOptions.input` map inside the exported config:

```ts
import { resolve } from "node:path";

// inside defineConfig({ ... })
build: {
  rollupOptions: {
    input: {
      main: resolve(__dirname, "index.html"),
      prefs: resolve(__dirname, "prefs.html"),
      tray: resolve(__dirname, "tray.html"),
      pet: resolve(__dirname, "pet.html"),
      overlay: resolve(__dirname, "overlay.html"),
      gallery: resolve(__dirname, "gallery.html"),
    },
  },
},
```

Keep the existing `server`, `clearScreen` and plugin settings untouched.

- [ ] **Step 2: Create the shared entry helper**

Create `src/entries/mount.ts`:

```ts
import { mount } from "svelte";
import type { Component } from "svelte";
import "../styles/fonts.css";
import "../styles/tokens.css";
import "../styles/base.css";

export function mountApp(App: Component): unknown {
  return mount(App, { target: document.getElementById("app")! });
}
```

Rewrite `src/main.ts` to use it:

```ts
import { mountApp } from "./entries/mount";
import App from "./routes/main/App.svelte";

export default mountApp(App);
```

- [ ] **Step 3: Create the four window entries**

For each of `prefs`, `tray`, `pet`, `overlay`, create `src/entries/<name>.ts`:

```ts
import { mountApp } from "./mount";
import App from "../routes/<name>/App.svelte";

export default mountApp(App);
```

and a placeholder `src/routes/<name>/App.svelte`:

```svelte
<script lang="ts"></script>

<main>Pomodo · <name></main>
```

Replace `<name>` with the actual route name in both files. These are deliberate
placeholders — plans 5, 6 and 7 fill them in. Then create `<name>.html` at the repo root:

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Pomodo</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/entries/<name>.ts"></script>
  </body>
</html>
```

- [ ] **Step 4: Build the gallery**

Create `src/entries/gallery.ts` the same way, then `src/routes/gallery/App.svelte`:

```svelte
<script lang="ts">
  import PetCanvas from "../../lib/components/PetCanvas.svelte";
  import PixelButton from "../../lib/components/PixelButton.svelte";
  import Toggle from "../../lib/components/Toggle.svelte";
  import Chip from "../../lib/components/Chip.svelte";
  import StatBar from "../../lib/components/StatBar.svelte";
  import SpeechBubble from "../../lib/components/SpeechBubble.svelte";
  import TitleBar from "../../lib/components/TitleBar.svelte";
  import SectionHeading from "../../lib/components/SectionHeading.svelte";
  import { LOCKED_BODY, PETS } from "../../lib/sprites";
  import { ACCENTS, type Accent, type Tone, tone } from "../../lib/theme";

  let accent = $state<Accent>("terracotta");
  let activeTone = $state<Tone>("playful");
  let switched = $state(true);
  let picked = $state(0);

  $effect(() => {
    document.documentElement.dataset.accent = accent;
  });
</script>

<div class="page">
  <SectionHeading index="00" title="组件画廊" caption="仅用于开发时比对设计稿" />

  <div class="controls">
    {#each Object.keys(ACCENTS) as key (key)}
      <Chip
        selected={accent === key}
        dot={ACCENTS[key as Accent]}
        onclick={() => (accent = key as Accent)}
      >
        {key}
      </Chip>
    {/each}
    {#each [["professional", "克制专业"], ["gentle", "温和陪伴"], ["playful", "俏皮拟人"]] as [key, label] (key)}
      <Chip selected={activeTone === key} onclick={() => (activeTone = key as Tone)}>
        {label}
      </Chip>
    {/each}
  </div>

  <p class="tagline">
    {tone(
      activeTone,
      "菜单栏与主窗口双入口的番茄计时器，含可自定义的身体提醒与桌面宠物。",
      "一个陪你专注的番茄钟：它记得提醒你站立喝水，也记得在你完成时替你高兴。",
      "它负责计时、催你喝水、盯你站起来，并在你摸鱼时用眼神谴责你。",
    )}
  </p>

  <div class="pets">
    {#each PETS as pet (pet.id)}
      <button class="petcard" class:sel={picked === pet.id} onclick={() => (picked = pet.id)}>
        <PetCanvas
          map={pet.map}
          body={pet.unlockedByDefault ? pet.body : LOCKED_BODY}
          scale={4}
          alt={pet.name}
        />
        <span>{pet.name}</span>
      </button>
    {/each}
  </div>

  <div class="row">
    <PetCanvas map={PETS[picked].map} body={PETS[picked].body} scale={8} anim="bob" />
    <PetCanvas map={PETS[picked].map} body={PETS[picked].body} scale={4} anim="hop" />
    <PetCanvas map={PETS[picked].map} body={PETS[picked].body} scale={3} anim="sway" />
  </div>

  <div class="row">
    <PixelButton>让它歇会儿</PixelButton>
    <PixelButton variant="secondary">跳过</PixelButton>
    <Toggle checked={switched} onchange={(v) => (switched = v)} label="示例开关" />
  </div>

  <SpeechBubble maxWidth={340}>还有 12 分钟，我盯着你呢</SpeechBubble>

  <div class="bars">
    <StatBar name="喝水" value="6 / 8 杯" pct={75} color="oklch(0.66 0.09 195)" />
    <StatBar name="站立" value="4 / 6 次" pct={66} color="oklch(0.63 0.13 40)" />
    <StatBar name="久坐最长" value="68 分钟" pct={76} color="oklch(0.7 0.12 60)" />
  </div>

  <div class="window">
    <TitleBar title="Pomodo" />
  </div>
</div>

<style>
  .page {
    padding: 40px;
    display: flex;
    flex-direction: column;
    gap: 28px;
    max-width: 900px;
  }
  .controls,
  .row {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .tagline {
    margin: 0;
    color: var(--dim);
    font-size: 17px;
    line-height: 1.6;
  }
  .pets {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 10px;
  }
  .petcard {
    padding: 18px 8px 11px;
    border: 1.5px solid var(--line);
    border-radius: 12px;
    background: var(--card);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    cursor: pointer;
    font-family: var(--font-pixel);
    font-size: 11px;
  }
  .petcard.sel {
    border-color: var(--accent);
    background: oklch(0.975 0.008 70);
  }
  .bars {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 320px;
  }
  .window {
    border: 1px solid var(--line);
    border-radius: var(--radius-window);
    overflow: hidden;
    background: var(--card);
  }
</style>
```

- [ ] **Step 5: Verify the gallery renders**

Run: `npm run dev`, open `http://localhost:1420/gallery.html`.
Expected: six pixel pets render crisply; switching accent recolours buttons, chips and the
selected pet card; switching tone rewrites the tagline; the bob/hop/sway animations run.
Compare the pets against artboard 01's pet grid — pixels must be sharp, never blurred.

- [ ] **Step 6: Verify the whole toolchain**

Run: `npm test && npm run check && npm run build`
Expected: all pass. `dist/` contains `index.html`, `prefs.html`, `tray.html`, `pet.html`,
`overlay.html`, `gallery.html`.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add multi-entry build and the component gallery"
```

---

## Definition of Done

- `npm test`, `npm run check`, `npm run build` all pass.
- `gallery.html` renders every component; accent and tone switching both work live.
- No `fonts.googleapis.com` reference anywhere in `src/` or the HTML entries.
- All six sprite maps render pixel-sharp at scales 3, 4, 8 and 9.
- `dist/` contains six HTML entry points.
- Nothing in this plan imports `@tauri-apps/api` — the whole plan is verifiable in a browser.
