"""Furniture's painted textures (`furniture` design 2; owner, 2026-11-23:
"Make sure things have textures", painted detail in the texture and not
stuck-on boxes): a chest of drawers' front, a closet's doors, and chests'
boards with their bands and locks. Each is mapped once over its face
(`repeat_m: 0` in the manifest), drawn over the settlement set's own planks
and boards in their palette, the iron in the iron texture's, sampled
nearest: pixel art at the walls' texel size.

    python3 tools/art/furniture_textures.py
"""

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
SET = ROOT / "assets" / "textures" / "settlement"

SEAM = (52, 37, 22)
DARK = (75, 49, 28)
LIGHT = (150, 110, 70)
IRON0 = (40, 41, 46)
IRON1 = (78, 80, 88)
IRON2 = (128, 132, 140)
IRON3 = (170, 174, 180)


def base(name, w=32, h=32):
    tile = Image.open(SET / f"{name}.png").convert("RGB")
    im = Image.new("RGB", (w, h))
    for y in range(0, h, tile.height):
        for x in range(0, w, tile.width):
            im.paste(tile, (x, y))
    return im


def rect(im, x0, y0, x1, y1, c):
    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            if 0 <= x < im.width and 0 <= y < im.height:
                im.putpixel((x, y), c)


def frame(im):
    w, h = im.size
    rect(im, 0, 0, w - 1, 0, SEAM)
    rect(im, 0, h - 1, w - 1, h - 1, SEAM)
    rect(im, 0, 0, 0, h - 1, SEAM)
    rect(im, w - 1, 0, w - 1, h - 1, SEAM)


def pull(im, x, y):
    # an iron D-pull: a dark outline, a lit top
    rect(im, x, y, x + 3, y + 1, IRON1)
    rect(im, x, y, x + 3, y, IRON2)
    im.putpixel((x + 1, y), IRON3)
    rect(im, x, y + 2, x + 3, y + 2, IRON0)


def drawer_front():
    im = base("plank")
    frame(im)
    for top in (1, 11, 21):
        bottom = top + 9
        rect(im, 1, bottom, 30, bottom, SEAM)
        rect(im, 1, top, 30, top, LIGHT)
        pull(im, 7, top + 4)
        pull(im, 21, top + 4)
    return im


def closet_front():
    im = base("plank", 32, 64)
    frame(im)
    rect(im, 15, 0, 16, 63, SEAM)
    for x0, x1 in ((3, 12), (19, 28)):
        for y0, y1 in ((4, 28), (34, 59)):
            # a sunk panel: lit on its lower right, shaded on its upper left
            rect(im, x0, y0, x1, y0, DARK)
            rect(im, x0, y0, x0, y1, DARK)
            rect(im, x0, y1, x1, y1, LIGHT)
            rect(im, x1, y0, x1, y1, LIGHT)
    for x in (12, 18):
        # ring pulls by the meeting doors
        rect(im, x, 30, x + 1, 30, IRON2)
        rect(im, x - 1 if x == 12 else x + 2, 31, x - 1 if x == 12 else x + 2, 32, IRON1)
        rect(im, x, 33, x + 1, 33, IRON0)
    return im


def bands(im, spans, iron):
    for x0, x1 in spans:
        if iron:
            rect(im, x0, 0, x1, 31, IRON1)
            rect(im, x0, 0, x0, 31, IRON2)
            rect(im, x1, 0, x1, 31, IRON0)
            for y in range(3, 32, 7):
                im.putpixel(((x0 + x1) // 2, y), IRON3)
        else:
            rect(im, x0, 0, x1, 31, DARK)
            rect(im, x0, 0, x0, 31, LIGHT)
            rect(im, x1, 0, x1, 31, SEAM)
            for y in range(4, 32, 8):
                im.putpixel(((x0 + x1) // 2, y), IRON1)


def lock(im):
    rect(im, 13, 1, 18, 9, IRON1)
    rect(im, 13, 1, 18, 1, IRON2)
    rect(im, 13, 9, 18, 9, IRON0)
    rect(im, 15, 4, 16, 6, IRON0)
    im.putpixel((15, 3), IRON0)


def chest(front, iron):
    im = base("boards")
    spans = [(2, 4), (10, 12), (19, 21), (27, 29)] if iron else [(4, 7), (24, 27)]
    bands(im, spans, iron)
    rect(im, 0, 0, 31, 0, SEAM)
    if front:
        lock(im)
    return im


def main():
    made = {
        "drawerfront": drawer_front(),
        "closetfront": closet_front(),
        "chestside": chest(False, False),
        "chestfront": chest(True, False),
        "bandedside": chest(False, True),
        "bandedfront": chest(True, True),
    }
    for name, im in made.items():
        im.save(SET / f"{name}.png")
    manifest = SET / "manifest.ron"
    text = manifest.read_text()
    for name, im in made.items():
        line = f'    (name: "{name}", width: {im.width}, height: {im.height}, repeat_m: 0.0),'
        if f'"{name}"' not in text:
            text = text.replace("  ],\n)", line + "\n  ],\n)")
    manifest.write_text(text)


if __name__ == "__main__":
    main()
