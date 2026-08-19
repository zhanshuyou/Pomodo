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
