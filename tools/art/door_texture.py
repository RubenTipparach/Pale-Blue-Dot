#!/usr/bin/env python3
"""Draw a player's door leaf, assets/textures/settlement/door.png
(`player-building` design B8): four vertical planks, two ledges and a brace
across them, the iron strap hinges on the hinge side (the left edge, where
`building::door::leaf_mesh` puts the hinge) and a ring pull on the other.
The hinges, ledges and pull are painted into the face, not modelled
(CLAUDE.md, "No obvious primitives").

32 x 64, mapped once over each face of the leaf. Every colour is taken from
the town textures the door is made of (`plank`, `timber` and `iron`): each
pixel is drawn as a tone and snapped to the nearest of theirs. Lit from the
top left, as the town textures are; no outline.

    python3 tools/art/door_texture.py            # write the PNG
    python3 tools/art/door_texture.py --check    # fail if it differs

Deterministic: a fixed seed, integer maths, the sources' colours sorted.
"""

import random
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2] / "assets" / "textures" / "settlement"
OUT = ROOT / "door.png"
W, H = 32, 64


def palette(names):
    colours = set()
    for name in names:
        image = Image.open(ROOT / f"{name}.png").convert("RGBA")
        colours.update(c[:3] for c in image.getdata() if c[3] == 255)
    return sorted(colours)


def nearest(colours, rgb):
    return min(colours, key=lambda c: sum((a - b) ** 2 for a, b in zip(c, rgb)))


def clamp(rgb):
    return tuple(max(0, min(255, int(c))) for c in rgb)


def draw():
    rng = random.Random(2211)
    wood = palette(("plank", "timber"))
    iron = palette(("iron",))
    tone = [[None] * W for _ in range(H)]
    metal = [[False] * W for _ in range(H)]

    # Four planks, each its own shade, with a grain that runs down it.
    for plank in range(4):
        x0 = plank * 8
        base = (rng.randrange(128, 150), rng.randrange(92, 106), rng.randrange(54, 64))
        streaks = {x0 + rng.randrange(1, 7): rng.choice((-14, -10, 10)) for _ in range(2)}
        for x in range(x0, x0 + 8):
            for y in range(H):
                lift = streaks.get(x, 0)
                if (y + x * 7) % 23 == 0:
                    lift -= 8
                if x == x0 + 7:
                    # The joint between planks, in shade.
                    rgb = (78, 50, 30)
                elif x == x0:
                    rgb = clamp(c + 10 + lift for c in base)
                else:
                    rgb = clamp(c + lift for c in base)
                tone[y][x] = rgb

    def ledge(y0):
        # A batten across the planks: lit along its top, shaded under it.
        for x in range(W):
            for dy in range(6):
                if dy == 0:
                    rgb = (160, 116, 68)
                elif dy == 5:
                    rgb = (82, 54, 32)
                else:
                    rgb = (138 - dy * 2, 98 - dy * 2, 58)
                tone[y0 + dy][x] = rgb

    def brace(xa, ya, xb, yb):
        # A diagonal batten between the ledges, four pixels across.
        steps = max(abs(xb - xa), abs(yb - ya))
        for k in range(steps + 1):
            x = xa + (xb - xa) * k // steps
            y = ya + (yb - ya) * k // steps
            for d in range(4):
                xx = x + d - 1
                if 0 <= xx < W:
                    rgb = (156, 112, 66) if d == 0 else ((84, 56, 34) if d == 3 else (136, 96, 56))
                    tone[y][xx] = rgb

    brace(4, 50, 26, 14)
    ledge(8)
    ledge(50)

    def strap(y0, length):
        # An iron strap hinge from the hinge edge along a ledge, its end
        # rounded, nails along it.
        for x in range(length):
            for dy in range(3):
                if x == length - 1 and dy != 1:
                    continue
                rgb = (78, 78, 84) if dy == 0 else ((50, 50, 56) if dy == 2 else (64, 64, 70))
                if dy == 1 and x in (2, 8, 14, length - 3):
                    rgb = (96, 96, 104)
                tone[y0 + dy][x] = rgb
                metal[y0 + dy][x] = True

    strap(9, 22)
    strap(51, 22)

    # The ring pull on the free side, at hand height.
    for (x, y) in ((26, 31), (27, 31), (25, 32), (28, 32), (25, 33), (28, 33), (26, 34), (27, 34)):
        tone[y][x] = (90, 90, 98) if y < 33 else (52, 52, 58)
        metal[y][x] = True
    tone[30][26] = (70, 70, 76)
    tone[30][27] = (70, 70, 76)
    metal[30][26] = metal[30][27] = True

    image = Image.new("RGBA", (W, H))
    cache = {}
    for y in range(H):
        for x in range(W):
            rgb = tone[y][x]
            key = (rgb, metal[y][x])
            if key not in cache:
                cache[key] = nearest(iron if metal[y][x] else wood, rgb)
            image.putpixel((x, y), cache[key] + (255,))
    return image


def main():
    image = draw()
    if "--check" in sys.argv[1:]:
        have = Image.open(OUT).convert("RGBA")
        if list(have.getdata()) != list(image.getdata()):
            print(f"stale: {OUT}")
            sys.exit(1)
        print("door.png is current")
        return
    image.save(OUT)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
