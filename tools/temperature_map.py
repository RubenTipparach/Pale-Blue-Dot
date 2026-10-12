#!/usr/bin/env python3
"""A surface temperature map of the planet, from the fish-range instrument.

Reads the fields `cargo run --release -p pbd-core --example fish_ranges`
writes and draws one year's mean surface temperature as an equirectangular
map. Over the sea the temperature is the sea surface. On land it is the
ground at sea level, which is what the instrument records (`ground_k`).

- Colour runs from deep blue at -30 C through white at 0 C to red at +35 C.
- A thin line is drawn every 5 C, and a heavier one at freezing (-1.8 C, the
  sea's freezing point).
- Coasts are traced from the altitude plane, and land is shaded a little
  darker than sea at the same temperature.
- Latitude lines at 30 and 60 degrees.

It also prints the area-weighted mean over the whole surface and over the sea,
which is the number `climate-balance` holds at 15 C.

    python3 tools/temperature_map.py FIELDS.bin OUT.png [--year N] [--title T]

The measurement instrument for `openspec/changes/climate-balance`. Nothing in
the game reads it.
"""

import math
import struct
import sys

from PIL import Image, ImageDraw, ImageFont

FREEZE_C = -1.8
LOW_C, HIGH_C = -30.0, 35.0
# Colour stops (deg C, RGB), cold to hot.
STOPS = [
    (-30.0, (32, 44, 110)),
    (-15.0, (58, 110, 190)),
    (-1.8, (205, 228, 245)),
    (0.0, (245, 245, 240)),
    (10.0, (250, 214, 130)),
    (20.0, (240, 140, 60)),
    (35.0, (170, 30, 40)),
]


def load(path):
    with open(path, "rb") as f:
        data = f.read()
    if data[:8] != b"PBDFISH1":
        sys.exit(f"{path}: not a fish-range field file")
    width, height, planes = struct.unpack_from("<III", data, 8)
    n = width * height
    off = 20
    out = []
    for p in range(planes):
        out.append(struct.unpack_from(f"<{n}f", data, off + p * n * 4))
    return width, height, out


def colour(t):
    t = max(LOW_C, min(HIGH_C, t))
    for (t0, c0), (t1, c1) in zip(STOPS, STOPS[1:]):
        if t <= t1:
            f = (t - t0) / (t1 - t0) if t1 > t0 else 0.0
            return tuple(round(a + (b - a) * f) for a, b in zip(c0, c1))
    return STOPS[-1][1]


def main():
    args = sys.argv[1:]
    if len(args) < 2:
        sys.exit(__doc__)
    src, out = args[0], args[1]
    year = int(args[args.index("--year") + 1]) if "--year" in args else 1
    title = args[args.index("--title") + 1] if "--title" in args else None
    width, height, planes = load(src)
    alt = planes[0]
    years = (len(planes) - 2) // 3
    if not 1 <= year <= years:
        sys.exit(f"{src} holds {years} year(s)")
    mean = planes[2 + (year - 1) * 3]

    # Area-weighted means: a row's weight is the cosine of its latitude.
    tot = sea = w_tot = w_sea = 0.0
    lo, hi = 1e9, -1e9
    for y in range(height):
        lat = math.pi * (0.5 - (y + 0.5) / height)
        w = math.cos(lat)
        for x in range(width):
            i = y * width + x
            t = mean[i]
            tot += t * w
            w_tot += w
            lo, hi = min(lo, t), max(hi, t)
            if alt[i] < 0.0:
                sea += t * w
                w_sea += w
    print(f"year {year}: whole surface {tot / w_tot:.1f} C, sea {sea / w_sea:.1f} C, "
          f"range {lo:.1f} to {hi:.1f} C")

    img = Image.new("RGB", (width, height))
    px = img.load()
    for y in range(height):
        for x in range(width):
            i = y * width + x
            r, g, b = colour(mean[i])
            if alt[i] >= 0.0:
                r, g, b = round(r * 0.82), round(g * 0.82), round(b * 0.82)
            px[x, y] = (r, g, b)

    # Isotherms every 5 C and the freezing line, where the band changes
    # between a pixel and its right or lower neighbour.
    def band(t):
        return math.floor(t / 5.0)

    for y in range(height - 1):
        for x in range(width - 1):
            i = y * width + x
            t, tr, td = mean[i], mean[i + 1], mean[i + width]
            if (t < FREEZE_C) != (tr < FREEZE_C) or (t < FREEZE_C) != (td < FREEZE_C):
                px[x, y] = (20, 60, 140)
            elif band(t) != band(tr) or band(t) != band(td):
                r, g, b = px[x, y]
                px[x, y] = (round(r * 0.55), round(g * 0.55), round(b * 0.55))
            # Coast.
            if (alt[i] >= 0.0) != (alt[i + 1] >= 0.0) or (alt[i] >= 0.0) != (alt[i + width] >= 0.0):
                px[x, y] = (30, 30, 30)

    draw = ImageDraw.Draw(img)
    for lat in (-60, -30, 0, 30, 60):
        y = round((0.5 - lat / 180.0) * height)
        dash = 6 if lat else 12
        for x in range(0, width, dash * 2):
            draw.line([(x, y), (x + dash, y)], fill=(60, 60, 60), width=1)

    # Legend under the map.
    font = ImageFont.load_default()
    pad, bar_h, label_h = 16, 18, 44 if title else 30
    canvas = Image.new("RGB", (width, height + bar_h + label_h + pad * 2), (250, 250, 247))
    canvas.paste(img, (0, 0))
    d = ImageDraw.Draw(canvas)
    x0, x1, yb = pad, width - pad, height + pad
    for x in range(x0, x1):
        t = LOW_C + (HIGH_C - LOW_C) * (x - x0) / (x1 - x0)
        d.line([(x, yb), (x, yb + bar_h)], fill=colour(t))
    for t in range(-30, 36, 5):
        x = x0 + (x1 - x0) * (t - LOW_C) / (HIGH_C - LOW_C)
        d.line([(x, yb + bar_h), (x, yb + bar_h + 4)], fill=(40, 40, 40))
        d.text((x - 8, yb + bar_h + 6), f"{t}", fill=(40, 40, 40), font=font)
    if title:
        d.text((x0, yb + bar_h + 22), title, fill=(20, 20, 20), font=font)
    canvas.save(out)
    print(f"wrote {out}")


if __name__ == "__main__":
    main()
