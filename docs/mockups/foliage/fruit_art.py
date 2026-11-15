"""Wild fruit for the foliage playground (docs/mockups/foliage.html), world style: no outlines.

Owner, 2026-11-15: "we also need fruits to be found in the wild, berries, strawberries, apple,
watermelon, tomatoes, etc", and "mango, kiwi, regional fruits would be nice as well"; and, the same
day, "outlines are for sprites in the ui inventory system, there is no need for that in the world".

Drawn with the balatro-pixel-art kit (pixelkit.py), Sweaft-64 colours, alpha 0 or 255, sampled
nearest and alpha-tested, but in the world style: no outline ring, no seams between parts, no
near-black tone anywhere. Two kinds of sprite, into docs/mockups/foliage/fruit/:

- hang_<fruit>.png, 16 x 16: a cluster of fruit as it hangs in a host tree's crown (apple on an
  oak, mango on a kapok, dates on a palm ...), and, laid flat, as the few fallen under it;
- plant_<fruit>.png, 32 x 32: a forageable plant standing on the bottom edge (berry bushes, wild
  strawberries, tomato, watermelon and kiwi vines, prickly pear, the alien fruit).

Leaves, stems and vines are drawn in a KEY ramp (#05582d, #559642, #9bb675, dark to light). The
page swaps those three colours for the biome's own ground ramp (groundRamp: its ground tile's
dark, mid and light colours) when it loads a plant, so a plant's leaves sit in the ground's palette
as the grass tufts do. Fruit keeps its own colours, bright enough to spot.

    python docs/mockups/foliage/fruit_art.py
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
from pixelkit import Sprite, save  # noqa: E402

OUT = os.path.join(HERE, "fruit")
KEY = ["#05582d", "#559642", "#9bb675"]          # recoloured to the ground ramp by the page
WOOD = ["#725244", "#ab8465", "#cfaa81"]
F = {  # fruit ramps, dark to light; no near-black tone
    "apple": ["#a61e1f", "#d12d41", "#fa3a64"], "pear": ["#c56d25", "#faaf5c", "#eab798"],
    "plum": ["#4f1d83", "#7d2dbd", "#9e3ba2"], "cherry": ["#841c1d", "#d12d41", "#fa3a64"],
    "mango": ["#d4462b", "#fb902f", "#faaf5c"], "papaya": ["#c56d25", "#fe6420", "#fb902f"],
    "banana": ["#c56d25", "#faaf5c", "#eab798"], "coconut": ["#41322c", "#725244", "#ab8465"],
    "date": ["#733126", "#a8633e", "#e69d69"], "baobab": ["#a09f91", "#cfaa81", "#eab798"],
    "marula": ["#c56d25", "#faaf5c", "#eab798"], "pinecone": ["#522b2f", "#7d4636", "#ab8465"],
    "glowfruit": ["#1096c3", "#74effe", "#d2f8ff"],
    "blueberry": ["#191954", "#15578a", "#1096c3"], "raspberry": ["#a61e1f", "#d12d41", "#fa3a64"],
    "blackberry": ["#2b1f35", "#3e3551", "#69608b"], "strawberry": ["#a61e1f", "#d12d41", "#fa3a64"],
    "lingonberry": ["#a61e1f", "#d4462b", "#fa3a64"], "cloudberry": ["#c56d25", "#fb902f", "#faaf5c"],
    "cranberry": ["#841c1d", "#a61e1f", "#d12d41"], "elderberry": ["#3e3551", "#69608b", "#9e3ba2"],
    "tomato": ["#a61e1f", "#d4462b", "#fe6420"], "melon": ["#297537", "#53998b", "#84e3b5"],
    "kiwi": ["#522b2f", "#7d4636", "#ab8465"], "pear_cactus": ["#204f63", "#53998b", "#84e3b5"],
    "tuna": ["#9e3ba2", "#c34988", "#dd576d"], "gem": ["#4f1d83", "#7d2dbd", "#c34988"],
    "gem2": ["#15578a", "#1096c3", "#74effe"], "pod": ["#8d4526", "#c56d25", "#faaf5c"],
}


def sp_(w, h):
    return Sprite(w, h)


def part(sp, m, ramp, mode="sphere", glow=False):
    sp.part(m, ramp, mode, seam=False, glow=glow)


def berry(sp, x, y, ramp, r=1.2, glow=False):
    part(sp, sp.circle(x, y, r), ramp, "sphere", glow)
    sp.plot(round(x - r * 0.4), round(y - r * 0.4), ramp[-1], glow)


# ---------- hanging clusters, 16 x 16 --------------------------------------------------------------

def hang(kind):
    sp, r = sp_(16, 16), F[kind]
    stem = lambda pts: part(sp, sp.line(pts, 1), WOOD, "flat")
    if kind in ("apple", "plum", "marula"):
        stem([(8, 0), (8, 3)]); part(sp, sp.circle(5, 7, 3.5), r); part(sp, sp.circle(11, 9, 3.5), r)
        part(sp, sp.ellipse(8, 2, 12, 4), KEY, "rim")
    elif kind == "pear":
        for cx in (5, 11):
            stem([(cx, 0), (cx, 3)]); part(sp, sp.circle(cx, 5, 2), r); part(sp, sp.ellipse(cx - 3.5, 6, cx + 3.5, 14), r)
    elif kind == "cherry":
        stem([(8, 0), (4, 9)]); stem([(8, 0), (12, 10)]); berry(sp, 4, 11, r, 2.6); berry(sp, 12, 12, r, 2.6)
        part(sp, sp.ellipse(7, 0, 13, 3), KEY, "rim")
    elif kind == "mango":
        stem([(8, 0), (6, 4)]); part(sp, sp.ellipse(2, 4, 11, 15), r); sp.plot(4, 7, "#fa3a64"); sp.plot(5, 6, "#fa3a64")
        stem([(8, 0), (12, 6)]); part(sp, sp.ellipse(10, 6, 15, 14), r)
    elif kind == "papaya":
        for cx in (4, 8, 12):
            part(sp, sp.ellipse(cx - 2.5, 3, cx + 2.5, 15), r, "cyl_v")
        part(sp, sp.rect(2, 0, 14, 2), WOOD, "flat")
    elif kind == "banana":
        part(sp, sp.rect(7, 0, 8, 4), WOOD, "flat")
        for k in range(4):
            y = 3 + k * 3
            part(sp, sp.taper(2, y + 2, 14, y, 3, 2), r, "rim")
    elif kind == "coconut":
        for cx, cy in ((5, 6), (11, 6), (8, 11)):
            part(sp, sp.circle(cx, cy, 3.6), r)
    elif kind == "date":
        stem([(8, 0), (3, 8)]); stem([(8, 0), (13, 8)]); stem([(8, 0), (8, 9)])
        for x, y in ((3, 10), (5, 13), (8, 11), (10, 14), (13, 10), (7, 15), (11, 12)):
            part(sp, sp.ellipse(x - 1, y - 2, x + 1, y + 1), r)
    elif kind == "baobab":
        for cx, top in ((5, 2), (11, 4)):
            stem([(cx, 0), (cx, top)]); part(sp, sp.ellipse(cx - 2.5, top, cx + 2.5, top + 10), r, "cyl_v")
    elif kind == "pinecone":
        for cx in (5, 11):
            part(sp, sp.poly([(cx - 3, 4), (cx + 3, 4), (cx + 1, 14), (cx - 1, 14)]), r, "cyl_v")
            for y in (6, 9, 12):
                sp.plot(cx - 1, y, r[2]); sp.plot(cx + 1, y + 1, r[0])
    elif kind == "glowfruit":
        stem([(8, 0), (5, 6)]); stem([(8, 0), (11, 8)])
        berry(sp, 5, 9, r, 3, glow=True); berry(sp, 11, 11, r, 2.5, glow=True)
    return sp


# ---------- forage plants, 32 x 32, standing on the bottom row ------------------------------------

B = 31


def leaves(sp, cx, cy, rx, ry, n=7, seed=1):
    for k in range(n):
        a = k / n * math.tau + seed
        x, y = cx + math.cos(a) * rx * 0.6, cy + math.sin(a) * ry * 0.6
        part(sp, sp.ellipse(x - rx * 0.45, y - ry * 0.4, x + rx * 0.45, y + ry * 0.4), KEY)
    part(sp, sp.ellipse(cx - rx * 0.5, cy - ry * 0.5, cx + rx * 0.5, cy + ry * 0.5), KEY)


def bush(kind, h=22, w=13, n=9, rr=1.3):
    sp = sp_(32, 32)
    for dx in (-4, 0, 4):
        part(sp, sp.line([(16, B), (16 + dx * 1.5, B - h * 0.6)], 1), KEY, "flat")
    leaves(sp, 16, B - h * 0.55, w, h * 0.55, 9)
    for k in range(n):
        a = k * 2.4
        berry(sp, 16 + math.cos(a) * w * 0.7, B - h * 0.55 + math.sin(a) * h * 0.4, F[kind], rr)
    return sp


def elderberry():
    sp = bush("elderberry", 24, 13, 0)
    for cx, cy in ((8, 10), (16, 6), (24, 10), (12, 16), (21, 17)):
        for dx, dy in ((-2, 0), (0, -1), (2, 0), (-1, 1), (1, 1), (-3, 1), (3, 1), (0, 1)):
            sp.plot(cx + dx, cy + dy, F["elderberry"][1 + (dx + dy) % 2])
    return sp


def raspberry():
    sp = bush("raspberry", 24, 12, 0)
    for k in range(7):
        a = k * 2.4
        x, y = 16 + math.cos(a) * 9, B - 13 + math.sin(a) * 9
        part(sp, sp.ellipse(x - 1.5, y - 1, x + 1.5, y + 2), F["raspberry"])
    return sp


def low(kind, n=5, rr=1.4, spread=13):
    """A low ground plant: a mat of leaves, berries on and between them (strawberry, lingonberry ...)."""
    sp = sp_(32, 32)
    for k in range(5):
        x = 16 + (k - 2) * spread * 0.45
        part(sp, sp.ellipse(x - 4, B - 8 - (k % 2) * 2, x + 4, B), KEY)
    for k in range(n):
        x = 16 + (k - (n - 1) / 2) * spread * 2 / max(n - 1, 1) * 0.9
        y = B - 3 - (k % 2) * 4
        if kind == "strawberry":
            part(sp, sp.poly([(x - 2, y - 2), (x + 2, y - 2), (x, y + 2)]), F[kind])
            sp.plot(round(x), round(y - 1), "#faaf5c"); sp.fill(sp.rect(x - 1, y - 3, x + 1, y - 3), KEY[0])
        else:
            berry(sp, x, y, F[kind], rr)
    return sp


def tomato():
    sp = sp_(32, 32)
    part(sp, sp.line([(16, B), (15, B - 14), (17, B - 24)], 2), KEY, "flat")
    for side, y in ((-1, 10), (1, 15), (-1, 20), (1, 24)):
        part(sp, sp.ellipse(16 + side * 3 - 4, B - y - 3, 16 + side * 3 + 4, B - y + 2), KEY)
    for x, y in ((10, 9), (21, 13), (12, 19), (20, 22), (15, 5)):
        berry(sp, x, B - y, F["tomato"], 2.4)
    return sp


def vine(fruit, big):
    sp = sp_(32, 32)
    part(sp, sp.line([(1, B - 1), (10, B - 3), (20, B - 1), (31, B - 3)], 1), KEY, "flat")
    for x in (4, 13, 24, 29):
        part(sp, sp.ellipse(x - 3, B - 8, x + 3, B - 1), KEY)
    if big:   # a watermelon, striped in its own rind greens
        body = sp.ellipse(7, B - 12, 25, B)
        part(sp, body, F["melon"])
        for dx in (-6, -2, 2, 6):
            sp.fill(sp.line([(16 + dx * 0.5, B - 12), (16 + dx, B - 6), (16 + dx * 0.5, B)], 1) & body, F["melon"][0])
        sp.plot(11, B - 9, "#84e3b5")
    else:     # kiwi: fuzzy brown ovals along the vine, on a low tangle
        part(sp, sp.line([(6, B - 2), (10, B - 14), (20, B - 16), (26, B - 3)], 1), WOOD, "flat")
        for x, y in ((9, 11), (14, 15), (19, 14), (23, 9), (16, 9)):
            part(sp, sp.ellipse(x - 2, B - y - 1.5, x + 2, B - y + 1.5), F["kiwi"])
        for x in (12, 18):
            part(sp, sp.ellipse(x - 3, B - 21, x + 3, B - 15), KEY)
    return sp


def prickly_pear():
    sp = sp_(32, 32)
    for (x0, y0, x1, y1) in ((10, 14, 22, 31), (4, 6, 14, 20), (18, 3, 28, 18)):
        part(sp, sp.ellipse(x0, y0, x1, y1), F["pear_cactus"])
    for x, y in ((6, 5), (10, 4), (20, 2), (24, 2), (27, 5)):
        part(sp, sp.ellipse(x - 1.5, y - 2, x + 1.5, y + 2), F["tuna"])
    return sp


def glowfruit_plant():
    sp = sp_(32, 32)
    for dx, h in ((-6, 16), (0, 24), (6, 19)):
        part(sp, sp.line([(16 + dx * 0.4, B), (16 + dx, B - h)], 1), KEY, "flat")
        berry(sp, 16 + dx, B - h - 2, F["glowfruit"], 2.6, glow=True)
    part(sp, sp.ellipse(8, B - 5, 24, B), KEY)
    return sp


def gem_berry():
    sp = sp_(32, 32)
    for x0, x1, top in ((12, 15, 8), (18, 20, 13), (8, 10, 16)):
        part(sp, sp.poly([(x0, B), (x1, B), ((x0 + x1) / 2, top)]), ["#69608b", "#92a3b2", "#bfcad7"], "inset")
    for k, (x, y) in enumerate(((7, 14), (13, 7), (21, 12), (24, 20), (5, 22), (18, 20))):
        berry(sp, x, y, F["gem"] if k % 2 else F["gem2"], 1.8, glow=True)
    return sp


def bladder_pod():
    sp = sp_(32, 32)
    for k, x0 in enumerate((10, 17, 23)):
        pts = [(x0 + 2 * math.sin(i * 0.9 + k), B - i * 3) for i in range(9 - k)]
        part(sp, sp.line(pts, 2), KEY, "rim")
        for i in range(2, len(pts), 2):
            px, py = pts[i]
            part(sp, sp.ellipse(px + 1, py - 2, px + 5, py + 2), F["pod"])
    return sp


HANG = ["apple", "pear", "plum", "cherry", "mango", "papaya", "banana", "coconut", "date", "baobab", "marula", "pinecone", "glowfruit"]
PLANTS = {
    "blueberry": lambda: bush("blueberry", 18, 12, 11, 1.4), "raspberry": raspberry, "blackberry": lambda: bush("blackberry", 22, 13, 9, 1.8),
    "elderberry": elderberry, "strawberry": lambda: low("strawberry", 4), "lingonberry": lambda: low("lingonberry", 7, 1.3),
    "cloudberry": lambda: low("cloudberry", 3, 2.4), "cranberry": lambda: low("cranberry", 8, 1.3, 14),
    "tomato": tomato, "watermelon": lambda: vine("melon", True), "kiwi": lambda: vine("kiwi", False),
    "prickly_pear": prickly_pear, "glowfruit": glowfruit_plant, "gem_berry": gem_berry, "bladder_pod": bladder_pod,
}


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    src = "docs/mockups/foliage/fruit_art.py (world style, no outline)"
    items = [("hang_" + k, hang(k)) for k in HANG] + [("plant_" + k, fn()) for k, fn in PLANTS.items()]
    for name, sp in items:
        save(sp.image(), os.path.join(OUT, name + ".png"), src)
    print("wrote", len(items), "pngs to", OUT)
    if "--sheet" in argv:
        from pixelkit import sheet
        sheet([(n, sp.image()) for n, sp in items], scale=4, cols=8, cell=(128, 128)).save(argv[argv.index("--sheet") + 1])


if __name__ == "__main__":
    main(sys.argv[1:])
