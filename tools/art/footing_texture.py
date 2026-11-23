#!/usr/bin/env python3
"""Draw the footing's texture, assets/textures/settlement/footing.png
(`player-building` design B7): dressed stone blocks laid in courses, the
joints staggered course to course, the mortar recessed and darker.

32 x 32, as every town texture is, seamless both ways. Every colour is taken
from the stones the town kits already use (fieldstone, stone and rubble):
each pixel is drawn as a tone and snapped to the nearest colour of theirs, so
the footing sits in their palette and in the ground's. No outline.
`builds::tests::the_footing_is_drawn_in_the_stones_palette` (pbd-app) holds
the palette and the manifest row; `--check` holds the file to this script.

    python3 tools/art/footing_texture.py            # write the PNG
    python3 tools/art/footing_texture.py --check    # fail if it differs

Deterministic: a fixed seed, integer maths, and the sources' colours sorted.
"""

import random
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2] / "assets" / "textures" / "settlement"
SOURCES = ("fieldstone", "stone", "rubble")
OUT = ROOT / "footing.png"
SIZE = 32
COURSE = 8  # four courses in a repeat


def palette():
    colours = set()
    for name in SOURCES:
        image = Image.open(ROOT / f"{name}.png").convert("RGB")
        colours.update(image.getdata())
    return sorted(colours)


def nearest(colours, rgb):
    return min(colours, key=lambda c: sum((a - b) ** 2 for a, b in zip(c, rgb)))


def course_widths(rng):
    """Blocks 9 to 14 px long that wrap round the 32 px exactly."""
    while True:
        widths = []
        while sum(widths) < SIZE:
            widths.append(rng.randrange(9, 15))
        widths[-1] -= sum(widths) - SIZE
        if widths[-1] >= 7:
            return widths


def draw():
    rng = random.Random(1107)
    colours = palette()
    # The mortar: a dark grey-brown of the fieldstone's joints.
    mortar = (88, 83, 74)
    tone = {}
    for course in range(SIZE // COURSE):
        y0 = course * COURSE
        # Each course starts half a block along from the one under it, so
        # no joint runs on up the wall.
        x = (course * 6 + rng.randrange(3)) % SIZE
        for w in course_widths(rng):
            base = (
                rng.randrange(132, 160),
                rng.randrange(124, 148),
                rng.randrange(108, 128),
            )
            # A block's face is a hair uneven: one end of its top a pixel
            # lower than the other.
            dip = rng.randrange(2)
            for dx in range(w):
                px = (x + dx) % SIZE
                for dy in range(COURSE):
                    py = y0 + dy
                    if dx == w - 1 or dy == COURSE - 1:
                        rgb = mortar
                    else:
                        # Lit from above: the top row light, the bottom two
                        # in shade, the left column a little lit.
                        top = dy == 0 or (dy == 1 and dip and dx < w // 2)
                        lift = 20 if top else (-14 if dy >= COURSE - 3 else 0)
                        if dx == 0:
                            lift += 6
                        # The stone's grain: a few pixels a shade off.
                        grain = rng.choice((0, 0, 0, 0, -8, 8, -5))
                        rgb = tuple(max(0, min(255, c + lift + grain)) for c in base)
                    tone[(px, py)] = rgb
            x += w
    image = Image.new("RGB", (SIZE, SIZE))
    cache = {}
    for (px, py), rgb in tone.items():
        if rgb not in cache:
            cache[rgb] = nearest(colours, rgb)
        image.putpixel((px, py), cache[rgb])
    return image.convert("RGBA")


def main():
    image = draw()
    if "--check" in sys.argv[1:]:
        have = Image.open(OUT).convert("RGBA")
        if list(have.getdata()) != list(image.getdata()):
            print(f"stale: {OUT}")
            sys.exit(1)
        print("footing.png is current")
        return
    image.save(OUT)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
