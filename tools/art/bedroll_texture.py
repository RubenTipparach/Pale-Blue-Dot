#!/usr/bin/env python3
"""The laid bedroll's texture (`survival-mode` task 6.3): one 32 px tile in
the game's palette, sampled nearest, written to
assets/textures/settlement/bedroll.png beside the towns' set with its
manifest row. The model maps it once across (u) and twice along (v):

- the wool blanket in the woolback's cream (the `wool` tile's ramp), its
  weave a soft two-texel check with a few darker flecks;
- the hide binding down both long edges (x 0..3 and 28..31), in the
  tanning rack's hide ramp (`weapon_textures.py`'s `hide`);
- a darker band across at y 13..15, the rolled blanket's tie, which lands on
  the head's roll at the model's mapping.

Detail is painted here, never modelled (CLAUDE.md, "Every model is
textured").

    python3 tools/art/bedroll_texture.py            # write it
    python3 tools/art/bedroll_texture.py --check    # fail if it differs
"""

import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "textures" / "settlement" / "bedroll.png"
M32 = 0xFFFFFFFF


def h(x, y, s=0):
    v = (x * 374761393 + y * 668265263 + s * 2246822519) & M32
    v = ((v ^ (v >> 13)) * 1274126177) & M32
    return ((v ^ (v >> 16)) & M32) / 4294967296.0


def clamp(v):
    return max(0, min(255, int(round(v))))


WOOL = (214, 202, 176)
HIDE = (176, 120, 72)


def texel(x, y):
    if x < 4 or x >= 28:
        # The hide binding: mottled, its stitch line a texel in.
        v = 0.86 + 0.16 * h(x >> 1, y >> 1, 31)
        if x in (3, 28) and y % 4 == 1:
            v *= 0.72
        return tuple(c * v for c in HIDE)
    if 13 <= y <= 15:
        # The tie across the roll: a strip of hide.
        v = 0.7 + 0.1 * h(x, y, 32)
        return tuple(c * v for c in HIDE)
    weave = 0.95 if ((x >> 1) + (y >> 1)) % 2 else 1.03
    v = weave * (0.94 + 0.08 * h(x, y, 33))
    if h(x, y, 34) > 0.95:
        v *= 0.85
    return tuple(c * v for c in WOOL)


def render():
    im = Image.new("RGBA", (32, 32))
    px = im.load()
    for y in range(32):
        for x in range(32):
            r, g, b = texel(x, y)
            px[x, y] = (clamp(r), clamp(g), clamp(b), 255)
    return im


def main():
    im = render()
    if "--check" in sys.argv:
        if not OUT.exists() or Image.open(OUT).convert("RGBA").tobytes() != im.tobytes():
            print(f"{OUT} differs from tools/art/bedroll_texture.py")
            sys.exit(1)
        print("bedroll texture matches")
        return
    im.save(OUT)
    print("bedroll texture written")


if __name__ == "__main__":
    main()
