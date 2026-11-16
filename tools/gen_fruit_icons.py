#!/usr/bin/env python3
"""The fruit goods' icons (`flora-v2` design 12.5): M13's own hanging-fruit
sprites (`docs/mockups/foliage/fruit/hang_<kind>.png`, 16 px, drawn by
`docs/mockups/foliage/fruit_art.py` with no outline for the world) given the
1 px outline an inventory icon carries (CLAUDE.md: "outlines are for sprites
in the ui inventory system"). The outline is the sprite's own darkest tone
darkened again, so it stays in the fruit's palette.

    python3 tools/gen_fruit_icons.py          # write assets/items/goods/<good>.png
    python3 tools/gen_fruit_icons.py --check  # fail if any is stale
"""
import io
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "docs" / "mockups" / "foliage" / "fruit"
OUT = ROOT / "assets" / "items" / "goods"
# FRUIT_KINDS order (pbd_core::flora), the sprite's name and the good's code.
KINDS = [
    ("apple", "apple"), ("pear", "pear"), ("plum", "plum"), ("cherry", "cherry"),
    ("mango", "mango"), ("papaya", "papaya"), ("marula", "marula"),
    ("baobab", "baobab"), ("pinecone", "pine-cone"),
]


def icon(sprite):
    img = Image.open(sprite).convert("RGBA")
    w, h = img.size
    px = img.load()
    solid = [(x, y) for y in range(h) for x in range(w) if px[x, y][3] >= 128]
    darkest = min((px[x, y] for x, y in solid), key=lambda c: c[0] + c[1] + c[2])
    ink = tuple(int(c * 0.45) for c in darkest[:3]) + (255,)
    out = Image.new("RGBA", (w, h))
    o = out.load()
    for y in range(h):
        for x in range(w):
            if px[x, y][3] >= 128:
                o[x, y] = px[x, y][:3] + (255,)
    for y in range(h):
        for x in range(w):
            if o[x, y][3]:
                continue
            if any(0 <= x + dx < w and 0 <= y + dy < h and px[x + dx, y + dy][3] >= 128
                   for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1))):
                o[x, y] = ink
    buf = io.BytesIO()
    out.save(buf, format="PNG", optimize=False)
    return buf.getvalue()


def main():
    check = "--check" in sys.argv[1:]
    stale = []
    for sprite, code in KINDS:
        data = icon(SOURCE / f"hang_{sprite}.png")
        path = OUT / f"{code}.png"
        if check:
            if not path.exists() or path.read_bytes() != data:
                stale.append(str(path))
        else:
            path.write_bytes(data)
    if stale:
        raise SystemExit("stale fruit icons (run tools/gen_fruit_icons.py): " + ", ".join(stale))
    print(("checked " if check else "wrote ") + f"{len(KINDS)} fruit icons")


if __name__ == "__main__":
    main()
