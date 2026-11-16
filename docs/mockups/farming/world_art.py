"""The farming mockup's crops as the 3D world draws them: no outlines (owner, 2026-11-15).

The owner: "outlines are for sprites in the ui inventory system, there is no need for that in the
world, this game looks much better without harsh outlines" (CLAUDE.md, "No outlines in the world;
one art style"). tools/art/farming_art.py draws the crop stages <crop>_<0..3>.png with a 1 px ink
outline and dark seams; the page's crop table, cell panel and seed belt keep those (they are UI).
This script redraws the same stages from the same functions for the world:

- no outline ring and no seams between parts: a silhouette's edge is the plant's own tones;
- the darkest tone of each ramp lifted one step where it is near-black (#003227 -> #297537 and
  the like), so stripes and shadows no longer read as harsh dark detail against the soft ground;
- no frost sparkles, and the lightest tone of each ramp of four or more drawn as the one below
  (owner, 2026-11-16: "they are two contrasting colors ontop of eachother";
  `farming-and-smelting` decision 16), so a plant's light and shade are the scene's, not a lit
  top baked over a dark underside.

It writes docs/mockups/farming/world/<crop>_<s>.png, which the farm plot draws, and the same bytes
to docs/mockups/foliage/crops/<crop>_<s>.png, which the foliage playground's "Crops and gardens"
layer draws, so both pages show a crop the same way.

    python tools/art/farming_art.py            # icons and UI stages (outlined)
    python docs/mockups/farming/world_art.py   # the world's stages (outline-free)
"""

import os
import shutil
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "tools", "art"))
import farming_art as fa  # noqa: E402
from pixelkit import Sprite, save  # noqa: E402

OUTS = [os.path.join(HERE, "world"), os.path.join(ROOT, "docs", "mockups", "foliage", "crops")]


def hexrgb(h):
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


def lum(c):
    r, g, b = [(v / 255) ** 2.2 for v in c]
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def dark_lift():
    """ramp[0] -> ramp[1] for every ramp whose darkest tone is near-black."""
    lift = {}
    for ramp in fa.R.values():
        if len(ramp) >= 3 and lum(hexrgb(ramp[0])) < 0.035 and hexrgb(ramp[0]) not in lift:
            lift[hexrgb(ramp[0])] = hexrgb(ramp[1])
    return lift


def top_fold():
    """ramp[-1] -> ramp[-2] for every ramp of four or more tones (the first ramp to name a tone
    decides it)."""
    fold = {}
    for ramp in fa.R.values():
        # White is a flower's petal (the chili's), never a lit top: it keeps its colour.
        if ramp[-1].lower() == fa.WHITE:
            continue
        if len(ramp) >= 4 and hexrgb(ramp[-1]) not in fold:
            fold[hexrgb(ramp[-1])] = hexrgb(ramp[-2])
    return fold


def world_stage(fn, *args):
    part, outline, spark = Sprite.part, Sprite.outline, fa.spark

    def part_no_seam(self, m, ramp, mode="rim", base=None, depth=1, seam=True, glow=False, seam_color=None):
        return part(self, m, ramp, mode, base, depth, False, glow, seam_color)
    Sprite.part = part_no_seam
    Sprite.outline = lambda self, *a, **k: None
    fa.spark = lambda *a, **k: None
    try:
        sp = Sprite(fa.CW, fa.CW)
        fn(sp, *args)
    finally:
        Sprite.part, Sprite.outline, fa.spark = part, outline, spark
    img = np.array(sp.image())
    # Both maps read the drawing as it came out, so a lifted dark is never folded again.
    rgb = img[..., :3].copy()
    solid = img[..., 3] == 255
    for table in (dark_lift(), top_fold()):
        for src, dst in table.items():
            m = solid & np.all(rgb == src, axis=-1)
            img[m, :3] = dst
    from PIL import Image
    return Image.fromarray(img, "RGBA")


def main():
    for d in OUTS:
        os.makedirs(d, exist_ok=True)
    n = 0
    for name, (fn, seed) in fa.CROPS.items():
        for s in range(4):
            img = world_stage(fa.mound, seed) if s == 0 else world_stage(fn, s)
            first = os.path.join(OUTS[0], f"{name}_{s}.png")
            save(img, first, "docs/mockups/farming/world_art.py (outline-free world stage)")
            for d in OUTS[1:]:
                shutil.copyfile(first, os.path.join(d, f"{name}_{s}.png"))
            n += 1
    print("wrote", n, "world crop stages to", ", ".join(OUTS))


if __name__ == "__main__":
    main()
