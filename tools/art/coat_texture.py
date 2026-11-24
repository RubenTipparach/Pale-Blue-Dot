#!/usr/bin/env python3
"""Draw the animals' coat texture, assets/textures/settlement/coat.png
(`land-transport` design 12, `animal-husbandry` design 16).

A strip of six 32 x 32 tiles, side by side, each seamless both ways:

    0 hair     short hair laid along the body, a darker underside
    1 mane     long strands for a mane and a tail
    2 hoof     horn grown in bands
    3 feather  overlapping feathers in rows
    4 fleece   wool in tight curls
    5 skin     bare skin: a muzzle, a comb, a beak, legs

The game lays a tile once round a lofted part's section (x, "around") and
repeats it along the part (y, "along"), so a tile's top-to-bottom is along
the body and its left-to-right goes round it from the right side over the
top (x = 8), down the left and under the belly (x = 24). The tiles are light
neutral tones with the detail painted in; the animal's colours from its
genome tint them as vertex colours, so a coat's variation tints its texture
rather than replacing it. No outline, and no harsh contrast (CLAUDE.md, "No
outlines in the world"): every tone is from one warm grey ramp.

    python3 tools/art/coat_texture.py            # write the PNG
    python3 tools/art/coat_texture.py --check    # fail if it differs

Deterministic: a fixed seed and integer maths.
"""

import math
import random
import sys
from pathlib import Path

from PIL import Image

OUT = Path(__file__).resolve().parents[2] / "assets" / "textures" / "settlement" / "coat.png"
T = 32
TILES = ("hair", "mane", "hoof", "feather", "fleece", "skin")

# One warm grey ramp, light to dark: the tint brings the colour.
RAMP = [
    (255, 253, 248),
    (243, 239, 231),
    (229, 223, 213),
    (214, 207, 196),
    (198, 190, 178),
    (181, 172, 160),
    (163, 154, 142),
    (145, 136, 125),
]


def tone(level):
    return RAMP[max(0, min(len(RAMP) - 1, level))]


def under(x):
    """Ramp steps darker round the section: lit on top (x = 8), shaded under
    the belly (x = 24), half way on the sides."""
    return round(1.0 - math.sin(2.0 * math.pi * (x + 0.5) / T))


def hair(rng):
    px = {}
    base = [[rng.randrange(2) for _ in range(T)] for _ in range(T)]
    for x in range(T):
        for y in range(T):
            px[x, y] = 1 + under(x) + base[y][x]
    # Short strokes along the body: runs of 3 to 5 px a step lighter or
    # darker, each starting where the last left off on its column.
    for _ in range(70):
        x, y = rng.randrange(T), rng.randrange(T)
        d = rng.choice((-1, 1))
        for k in range(rng.randrange(3, 6)):
            px[x, (y + k) % T] += d
    return px


def mane(rng):
    px = {}
    strand = [rng.randrange(0, 4) for _ in range(T)]
    for x in range(T):
        for y in range(T):
            wobble = 1 if (y + x * 7) % 11 == 0 else 0
            px[x, y] = 1 + strand[x] + wobble
    # A dark parting between some strands.
    for x in range(0, T, 5):
        for y in range(T):
            px[x, y] = max(px[x, y], 5)
    return px


def hoof(rng):
    px = {}
    band = [rng.randrange(0, 3) for _ in range(T)]
    for x in range(T):
        for y in range(T):
            # Growth bands across the hoof, a lit stripe down its front.
            lit = -1 if 6 <= x <= 9 else 0
            px[x, y] = 3 + band[y] + lit + under(x) // 2
    return px


def feather(rng):
    px = {}
    for x in range(T):
        for y in range(T):
            px[x, y] = 1 + under(x)
    # Rows of feathers 8 px long, each row offset half a feather; a feather
    # is lighter at its root and has a soft darker rim at its tip.
    for row in range(T // 8):
        off = (row % 2) * 4
        for col in range(T // 8):
            cx = col * 8 + off
            for dx in range(8):
                for dy in range(8):
                    x, y = (cx + dx) % T, (row * 8 + dy) % T
                    edge = abs(dx - 3.5)
                    tip = dy - (5.0 - 0.25 * edge * edge)
                    if tip > 0.6:
                        px[x, y] += 2
                    elif dy < 3:
                        px[x, y] -= 1
            # A quill down the feather's middle.
            for dy in range(1, 5):
                px[(cx + 3) % T, (row * 8 + dy) % T] -= 1
    for _ in range(40):
        x, y = rng.randrange(T), rng.randrange(T)
        px[x, y] += rng.choice((-1, 1))
    return px


def fleece(rng):
    px = {}
    for x in range(T):
        for y in range(T):
            px[x, y] = 1 + under(x) // 2
    # Curls: a lit crown and a shaded crescent under it, packed tight.
    for _ in range(48):
        cx, cy = rng.randrange(T), rng.randrange(T)
        r = rng.choice((2, 2, 3))
        for dx in range(-r, r + 1):
            for dy in range(-r, r + 1):
                d = dx * dx + dy * dy
                if d > r * r:
                    continue
                x, y = (cx + dx) % T, (cy + dy) % T
                if dy < 0 and d < r * r - 1:
                    px[x, y] = min(px[x, y], 0)
                elif dy >= r - 1 or d >= r * r - 1:
                    px[x, y] = max(px[x, y], 3)
    return px


def skin(rng):
    px = {}
    for x in range(T):
        for y in range(T):
            px[x, y] = 1 + under(x) // 2 + (1 if rng.randrange(5) == 0 else 0)
    return px


def draw():
    rng = random.Random(1123)
    image = Image.new("RGB", (T * len(TILES), T))
    painters = {
        "hair": hair,
        "mane": mane,
        "hoof": hoof,
        "feather": feather,
        "fleece": fleece,
        "skin": skin,
    }
    for i, name in enumerate(TILES):
        px = painters[name](rng)
        for (x, y), level in px.items():
            image.putpixel((i * T + x, y), tone(level))
    return image


def main(argv):
    image = draw()
    if "--check" in argv:
        have = Image.open(OUT).convert("RGB")
        if have.size != image.size or have.tobytes() != image.tobytes():
            print(f"{OUT} differs from tools/art/coat_texture.py; run it to redraw")
            return 1
        print("coat.png matches its script")
        return 0
    image.save(OUT)
    print(f"wrote {OUT} ({image.width} x {image.height})")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
