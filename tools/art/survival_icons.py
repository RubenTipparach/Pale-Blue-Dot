#!/usr/bin/env python3
"""The pack icons of `survival-mode` task 4.10: the goods of design 15 to 17
(raw meat, hide, bone, a feather, the straw, the spears, the bow, the arrows,
the hunting axes, the swords, the straw hat, the wool cloak, the bedroll and
the meat stew) and the two new stations (the tanning rack and the still).

They are M33's Survival tab icons (docs/mockups/mission-tree.html#survival),
which the owner has seen there, drawn by the same functions in
tools/mockup_survival_icons.py and written here into the game's item set: a
16 x 16 slot icon, a ramp lit from the top left and the 1 px dark outline the
inventory icons keep (CLAUDE.md: outlines belong to icons, never the world).
Where M33 drew one tier, the others are the same drawing with the head's
ramp swapped for that tier's (stone, bronze, iron), as the tools' tiers are
tinted; the arrows' heads take bone and bronze the same way.

    python3 tools/art/survival_icons.py            # write the icons
    python3 tools/art/survival_icons.py --check    # fail if any differs
"""

import importlib.util
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location("m33", ROOT / "tools" / "mockup_survival_icons.py")
m33 = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(m33)
C = m33.C

GOODS = ROOT / "assets" / "items" / "goods"
STATIONS = ROOT / "assets" / "items" / "stations"

# The heads' ramps, light then dark, by tier: M33's own colour letters.
STONE = ("S", "s")
BRONZE = ("Z", "x")
IRON = ("I", "j")
BONE = ("B", "b")


def recolour(fn, swap):
    """Draw `fn`, then swap palette letters for others: `swap` maps a letter
    to the letter it becomes."""

    def draw(d):
        fn(d)
        im = d._image
        px = im.load()
        table = {C[a] + (255,): C[b] + (255,) for a, b in swap.items()}
        for y in range(im.height):
            for x in range(im.width):
                if px[x, y] in table:
                    px[x, y] = table[px[x, y]]

    return draw


def arrows_tipped(light, dark):
    """M33's arrows, their stone heads in another ramp."""
    return recolour(m33.arrows, {"s": light})


ICONS = {
    GOODS / "raw-meat.png": m33.raw_meat,
    GOODS / "hide.png": m33.hide,
    GOODS / "bone.png": m33.bone,
    GOODS / "feather.png": m33.feather,
    GOODS / "straw.png": m33.straw,
    GOODS / "stone-spear.png": m33.spear,
    GOODS / "bronze-spear.png": lambda d: m33.spear(d, *BRONZE),
    GOODS / "iron-spear.png": lambda d: m33.spear(d, *IRON),
    GOODS / "bow.png": m33.bow,
    GOODS / "stone-arrow.png": m33.arrows,
    GOODS / "bone-arrow.png": arrows_tipped(*BONE),
    GOODS / "bronze-arrow.png": arrows_tipped(*BRONZE),
    GOODS / "bronze-hunting-axe.png": m33.hunting_axe,
    GOODS / "iron-hunting-axe.png": lambda d: m33.hunting_axe(d, *IRON),
    GOODS / "bronze-sword.png": m33.sword,
    GOODS / "iron-sword.png": lambda d: m33.sword(d, *IRON),
    GOODS / "straw-hat.png": m33.hat,
    GOODS / "wool-cloak.png": m33.cloak,
    GOODS / "bedroll.png": m33.bedroll,
    GOODS / "meat-stew.png": m33.stew,
    STATIONS / "tanning-rack.png": m33.rack,
    STATIONS / "still.png": m33.still,
}


def render(fn):
    im = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    fn(ImageDraw.Draw(im))
    return m33.outlined(im)


def main():
    check = "--check" in sys.argv
    bad = []
    for path, fn in ICONS.items():
        im = render(fn)
        if check:
            if not path.exists() or Image.open(path).convert("RGBA").tobytes() != im.tobytes():
                bad.append(path)
            continue
        im.save(path)
    if bad:
        for path in bad:
            print(f"{path} differs from tools/art/survival_icons.py")
        sys.exit(1)
    print(f"{len(ICONS)} icons {'match' if check else 'written'}")


if __name__ == "__main__":
    main()
