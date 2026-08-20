"""Render Pomodo's app icon and menu-bar template from the design's 16x16 grids.

No third-party imaging libraries are available, so this writes PNGs directly:
zlib-compressed scanlines with a filter byte, which is all a PNG needs.
Everything is drawn at an integer multiple of the 16x16 grid, so the pixel art
stays exact — no resampling anywhere.
"""

import math
import struct
import zlib

# ---------------------------------------------------------------- colour ----

def _srgb(linear: float) -> int:
    v = 12.92 * linear if linear <= 0.0031308 else 1.055 * max(linear, 0.0) ** (1 / 2.4) - 0.055
    return max(0, min(255, round(v * 255)))


def oklch(l: float, c: float, h_deg: float):
    """oklch -> 8-bit sRGB, matching src/lib/sprites.ts."""
    h = math.radians(h_deg)
    a, b = c * math.cos(h), c * math.sin(h)
    lp, mp, sp = (
        l + 0.3963377774 * a + 0.2158037573 * b,
        l - 0.1055613458 * a - 0.0638541728 * b,
        l - 0.0894841775 * a - 1.2914855480 * b,
    )
    l3, m3, s3 = lp ** 3, mp ** 3, sp ** 3
    return (
        _srgb(4.0767416621 * l3 - 3.3077115913 * m3 + 0.2309699292 * s3),
        _srgb(-1.2684380046 * l3 + 2.6097574011 * m3 - 0.3413193965 * s3),
        _srgb(-0.0041960863 * l3 - 0.7034186147 * m3 + 1.7076147010 * s3),
    )


# ------------------------------------------------------------------- png ----

def write_png(path: str, w: int, h: int, px) -> None:
    raw = bytearray()
    for y in range(h):
        raw.append(0)  # filter: none
        for x in range(w):
            raw += bytes(px[y][x])

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(bytes(raw), 9)))
        f.write(chunk(b"IEND", b""))


def blank(w, h):
    return [[(0, 0, 0, 0) for _ in range(w)] for _ in range(h)]


def over(dst, src):
    """Source-over compositing, both premultiplied on the fly."""
    sr, sg, sb, sa = src
    dr, dg, db, da = dst
    if sa == 0:
        return dst
    a = sa / 255
    ia = 1 - a
    out_a = sa + round(da * ia)
    return (
        round(sr * a + dr * ia),
        round(sg * a + dg * ia),
        round(sb * a + db * ia),
        min(255, out_a),
    )


# ------------------------------------------------------------- the marks ----
# 猫头 — the design's recommended direction, its default selection.
CAT = [
    "................",
    "..o..........o..",
    "..oo........oo..",
    "..opo......opo..",
    "..obbo....obbo..",
    "..obbboooobbbo..",
    ".obbbbbbbbbbbbo.",
    "obbbebbbbbbebbbo",
    "obbbbbbbbbbbbbbo",
    "obbpbbwwwwbbpbbo",
    "obbbbbwwwwbbbbbo",
    ".obbbbbwwbbbbbo.",
    "..obbbbbbbbbbo..",
    "...oobbbbbboo...",
    ".....oooooo.....",
    "................",
]

BODY = oklch(0.86, 0.09, 82)
PALETTE = {
    "o": oklch(0.26, 0.02, 60),
    "b": BODY,
    "e": oklch(0.20, 0.015, 60),
    "w": oklch(0.98, 0.006, 80),
    "p": oklch(0.76, 0.12, 22),
}

# 16x16 menu-bar body mask, eyes knocked out. Solid cat head with wedge ears.
MB_BODY = [
    "................",
    "..xx........xx..",
    "..xxx......xxx..",
    "..xxxx....xxxx..",
    "..xxxxxxxxxxxx..",
    "..xxxxxxxxxxxx..",
    ".xxxxxxxxxxxxxx.",
    ".xxxxxxxxxxxxxx.",
    ".xxxxxxxxxxxxxx.",
    ".xxxxxxxxxxxxxx.",
    ".xxxxxxxxxxxxxx.",
    "..xxxxxxxxxxxx..",
    "..xxxxxxxxxxxx..",
    "...xxxxxxxxxx...",
    "................",
    "................",
]
MB_EYES = [(7, 4), (7, 11)]


# ------------------------------------------------------------ app icon -----

def rounded_rect_alpha(size: int, radius: float, samples: int = 4):
    """Coverage mask for a rounded square, supersampled for a clean edge."""
    mask = [[0.0] * size for _ in range(size)]
    step = 1.0 / samples
    for y in range(size):
        for x in range(size):
            hits = 0
            for sy in range(samples):
                py = y + (sy + 0.5) * step
                for sx in range(samples):
                    px = x + (sx + 0.5) * step
                    # Distance into the nearest corner's circle, if any.
                    cx = min(max(px, radius), size - radius)
                    cy = min(max(py, radius), size - radius)
                    if math.hypot(px - cx, py - cy) <= radius:
                        hits += 1
            mask[y][x] = hits / (samples * samples)
    return mask


def app_icon(path: str, canvas: int = 1024) -> None:
    # Apple's macOS template leaves the rounded square at ~80% of the canvas.
    plate = round(canvas * 0.805)
    plate = plate - plate % 16  # keep the art's integer scaling exact
    origin = (canvas - plate) // 2
    radius = plate * 0.2235  # matches the design's 50px on a 224px plate

    img = blank(canvas, canvas)
    mask = rounded_rect_alpha(plate, radius)

    # 陶土 — the design's default plate, a 160deg two-stop ramp.
    c0 = oklch(0.74, 0.13, 48)
    c1 = oklch(0.58, 0.14, 32)
    ang = math.radians(160)
    dx, dy = math.sin(ang), -math.cos(ang)
    span = abs(plate * dx) + abs(plate * dy)

    for y in range(plate):
        for x in range(plate):
            cov = mask[y][x]
            if cov <= 0:
                continue
            t = (((x - plate / 2) * dx + (y - plate / 2) * dy) / span) + 0.5
            t = min(1.0, max(0.0, t))
            rgb = tuple(round(c0[i] + (c1[i] - c0[i]) * t) for i in range(3))

            # Gloss: white at the top, a faint darkening at the very bottom.
            v = y / plate
            if v < 0.46:
                g = 0.22 * (1 - v / 0.46)
                rgb = tuple(round(rgb[i] + (255 - rgb[i]) * g) for i in range(3))
            else:
                g = 0.10 * ((v - 0.46) / 0.54)
                rgb = tuple(round(rgb[i] * (1 - g)) for i in range(3))

            img[origin + y][origin + x] = (*rgb, round(255 * cov))

    # The pixel art sits at ~62% of the plate, at an exact integer scale.
    scale = round(plate * 0.62 / 16)
    art = scale * 16
    ax = origin + (plate - art) // 2
    ay = origin + (plate - art) // 2
    for gy in range(16):
        for gx in range(16):
            ch = CAT[gy][gx]
            if ch not in PALETTE:
                continue
            rgba = (*PALETTE[ch], 255)
            for py in range(scale):
                for px in range(scale):
                    yy, xx = ay + gy * scale + py, ax + gx * scale + px
                    img[yy][xx] = over(img[yy][xx], rgba)

    write_png(path, canvas, canvas, img)
    return plate, scale


# ------------------------------------------------- menu-bar template -------

def menubar_glyph(scale: int):
    """Solid 待机 silhouette with the eyes knocked out.

    A macOS template image carries shape in the alpha channel only; the system
    paints it dark on a light menu bar and light on a dark one, and inverts it
    when the menu opens. Baking a colour in would defeat that.
    """
    size = 16 * scale
    img = blank(size, size)
    for gy in range(16):
        for gx in range(16):
            if MB_BODY[gy][gx] != "x":
                continue
            if (gy, gx) in MB_EYES:
                continue
            for py in range(scale):
                for px in range(scale):
                    img[gy * scale + py][gx * scale + px] = (0, 0, 0, 255)
    return size, img


def menubar_icon(path: str, scale: int) -> None:
    size, img = menubar_glyph(scale)
    write_png(path, size, size, img)


if __name__ == "__main__":
    import sys

    out = sys.argv[1]
    plate, scale = app_icon(f"{out}/app-icon-1024.png")
    print(f"app icon: 1024 canvas, {plate}px plate, art at {scale}x (={scale*16}px)")
    # @2x is an exact integer doubling of a pixel grid, so it stays pixel-aligned
    # — the design's warning about redrawing applies to anti-aliased art.
    menubar_icon(f"{out}/tray-icon.png", 1)
    menubar_icon(f"{out}/tray-icon@2x.png", 2)
    print("menu-bar template: 16x16 and 32x32, alpha-only")
