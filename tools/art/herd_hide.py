#!/usr/bin/env python3
"""Lay the herd's hides into one texture, assets/textures/settlement/herd_hide.png
(`animal-husbandry` design 17, "The owner's pick": the blocky animals of M26's
Farmstead and Hatchery tabs).

Each row is one species' hide strip exactly as M26 draws it: eight 32 px tiles
side by side (0 body, 1 legs, 2 head, 3 horns or comb, 4 tail, 5 mane or wing,
6 snout or beak, 7 ears or belly), drawn by tools/art/animals_art.py and
tools/art/husbandry_art.py (the balatro-pixel-art kit) into docs/mockups/husbandry/.
The rows are, top to bottom: the horse (M26's fellrunner), the plodder, the
woolback and the meadow hen. `husbandry::forms` lays a tile on every face of a
box by the metre, as the mockup's hide shader does: 0.9 tiles a metre on the
body, 1.3 on every other part.

    python3 tools/art/herd_hide.py            # write the PNG
    python3 tools/art/herd_hide.py --check    # fail if it differs
"""
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "docs" / "mockups" / "husbandry"
OUT = ROOT / "assets" / "textures" / "settlement" / "herd_hide.png"
ROWS = ["hide_fellrunner.png", "m7/hide_plodder.png", "m7/hide_woolback.png", "m7/hide_hen.png"]


def build():
    img = Image.new("RGBA", (256, 32 * len(ROWS)), (0, 0, 0, 0))
    for i, name in enumerate(ROWS):
        strip = Image.open(SRC / name).convert("RGBA")
        assert strip.size == (256, 32), (name, strip.size)
        img.paste(strip, (0, 32 * i))
    # Every texel opaque: the herd's material is opaque.
    px = img.load()
    for y in range(img.height):
        for x in range(img.width):
            r, g, b, _ = px[x, y]
            px[x, y] = (r, g, b, 255)
    return img


def main():
    img = build()
    if "--check" in sys.argv:
        old = Image.open(OUT).convert("RGBA")
        if old.tobytes() != img.tobytes():
            print(f"{OUT} differs from tools/art/herd_hide.py; run it to redraw")
            sys.exit(1)
        print("herd_hide.png matches its script")
        return
    img.save(OUT)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
