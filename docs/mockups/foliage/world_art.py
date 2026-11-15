"""The foliage mockup's in-world pixel art, without outlines (owner, 2026-11-15).

The owner: "outlines are for sprites in the ui inventory system, there is no need for that in
the world, this game looks much better without harsh outlines" (CLAUDE.md, "No outlines in the
world; one art style"). tools/art/foliage_art.py draws every PNG here in the balatro-pixel-art
style, with a 1 px ink outline and a dark seam between parts. This script redraws the ones the
3D world wears from the same drawing functions, with:

- no outline ring: the silhouette's edge is the plant's own tones;
- no seams between parts (a seam is the same dark ring, inside the sprite);
- the birch's lenticels in the bark's own slate (#5d778d, a #434137 core) rather than ink, and
  the birch impostor's trunk bands in #434137.

It overwrites flower_*.png, imp_<species>.png and bark_birch.png. The biome picker's
icon_*.png are UI icons and keep their outlines, so run this AFTER foliage_art.py:

    python tools/art/foliage_art.py          # every PNG, icons included (outlined)
    python docs/mockups/foliage/world_art.py # then the world's versions, outline-free
"""

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "tools", "art"))
import foliage_art as fa  # noqa: E402
from pixelkit import Sprite, RAMPS, save  # noqa: E402

SOFT_DARK = "#434137"   # Sweaft-64; replaces INK where a world sprite used it as detail

_part = Sprite.part
_outline = Sprite.outline


def _part_no_seam(self, m, ramp, mode="rim", base=None, depth=1, seam=True, glow=False, seam_color=None):
    return _part(self, m, ramp, mode, base, depth, False, glow, seam_color)


def tile_birch_soft():
    """tile_birch with the lenticels in the bark's own darkest tones, not ink."""
    import random
    sp = Sprite(32, 32)
    sp.fill(sp.rect(0, 0, 31, 31), "#bfcad7")
    rnd = random.Random(4)
    for x in range(0, 32, 4):
        sp.fill(sp.rect(x, 0, x, 31), "#fffefe" if x % 8 else "#92a3b2")
    for _ in range(9):
        x, y, w = rnd.randrange(1, 24), rnd.randrange(1, 30), rnd.randrange(3, 7)
        sp.fill(sp.rect(x, y, x + w, y + 1), "#5d778d")
        sp.fill(sp.rect(x + 1, y, x + w - 1, y), SOFT_DARK)
    return sp


def world_items():
    Sprite.part = _part_no_seam
    Sprite.outline = lambda self, *a, **k: None
    fa.INK = SOFT_DARK
    try:
        items = [("bark_birch", tile_birch_soft()),
                 ("flower_red", fa.flower(RAMPS["red"], RAMPS["soot"], 5, 3, 4)),
                 ("flower_yellow", fa.flower(RAMPS["gold"][:4], RAMPS["rust"], 8, 2, 4)),
                 ("flower_blue", fa.flower(RAMPS["blue"], RAMPS["gold"][:4], 5, 3, 4)),
                 ("flower_white", fa.flower(RAMPS["paper"], RAMPS["gold"][:4], 10, 2, 5)),
                 ("flower_glow", fa.flower_glow())]
        for name, (fn, _ground) in fa.SPECIES.items():
            items.append(("imp_" + name, fa.impostor(fn)))
        return items
    finally:
        Sprite.part = _part
        Sprite.outline = _outline
        fa.INK = "#161612"


def main():
    items = world_items()
    for name, sp in items:
        save(sp.image(), os.path.join(HERE, name + ".png"), source="pixelkit, docs/mockups/foliage/world_art.py (no outline)")
    print("wrote", len(items), "outline-free world pngs to", HERE)


if __name__ == "__main__":
    main()
