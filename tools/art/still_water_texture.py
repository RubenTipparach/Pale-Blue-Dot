#!/usr/bin/env python3
"""The still's fresh water (`survival-mode` design 15 and 18): one 16 px
tile in the game's palette, sampled nearest, written to
assets/textures/settlement/still_water.png with its manifest row. It is
the clear water standing in the still's cup, drawn as it fills: a clear
river green-blue (fresh water reads greener and clearer than the sea, design
15's "the look"), darker at the cup's rim, with a few pale glints.

    python3 tools/art/still_water_texture.py            # write it
    python3 tools/art/still_water_texture.py --check    # fail if it differs
"""

import math
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "textures" / "settlement" / "still_water.png"
M32 = 0xFFFFFFFF


def h(x, y, s=0):
    v = (x * 374761393 + y * 668265263 + s * 2246822519) & M32
    v = ((v ^ (v >> 13)) * 1274126177) & M32
    return ((v ^ (v >> 16)) & M32) / 4294967296.0


FRESH = (70, 150, 150)


def texel(x, y):
    # The disc is mapped across the whole tile: darker toward its edge.
    r = math.hypot(x - 7.5, y - 7.5) / 7.5
    v = 1.05 - 0.3 * min(r, 1.0) ** 2 + 0.06 * (h(x >> 1, y >> 1, 41) - 0.5)
    c = tuple(k * v for k in FRESH)
    if h(x, y, 42) > 0.94 and r < 0.8:
        c = (200, 232, 224)
    return c


def render():
    im = Image.new("RGBA", (16, 16))
    px = im.load()
    for y in range(16):
        for x in range(16):
            px[x, y] = tuple(max(0, min(255, int(round(k)))) for k in texel(x, y)) + (255,)
    return im


def main():
    im = render()
    if "--check" in sys.argv:
        if not OUT.exists() or Image.open(OUT).convert("RGBA").tobytes() != im.tobytes():
            print(f"{OUT} differs from tools/art/still_water_texture.py")
            sys.exit(1)
        print("still water texture matches")
        return
    im.save(OUT)
    print("still water texture written")


if __name__ == "__main__":
    main()
