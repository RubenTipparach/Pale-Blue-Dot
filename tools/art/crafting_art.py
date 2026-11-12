"""Item, resource, station and tool icons for the crafting mockup (docs/mockups/crafting.html).

Pixel art drawn in code with the balatro-pixel-art kit (`.claude/skills/balatro-pixel-art`), the
solitaire collection's style: a 32 x 32 native canvas, one bold subject filling it, 3 to 5 tone
ramps lit from the top left, a dark seam between parts, a 1 px outline, single-pixel glints, no
anti-aliasing, every colour from Sweaft-64 (the kit's save() refuses anything else). The page shows
these only at whole multiples of 32 px (32, 64, 96) with `image-rendering: pixelated`.

    python tools/art/crafting_art.py                 # writes docs/mockups/crafting/*.png
    python tools/art/crafting_art.py --sheet OUT.png # also a 4x contact sheet beside the references
"""

import glob
import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art")
sys.path.insert(0, os.path.join(SKILL, "scripts"))
from pixelkit import RAMPS, Sprite, erode, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "crafting")
N = 32  # native size of every icon

# Ramps beyond the kit's, dark to light, every entry a Sweaft-64 colour.
R = dict(RAMPS)
R.update({
    "stone": ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "orerock": ["#2b1f35", "#3e3551", "#69608b", "#92a3b2"],
    "copper": ["#561c27", "#8d4526", "#a8633e", "#e69d69", "#eab798"],
    "bronze": ["#733126", "#c56d25", "#fb902f", "#faaf5c", "#fffefe"],
    "iron": ["#2c2c25", "#434137", "#a09f91", "#bfcad7", "#fffefe"],
    "steelb": ["#011c34", "#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "clay": ["#522b2f", "#7d4636", "#a8633e", "#e69d69"],
    "brick": ["#4a051e", "#841c1d", "#d4462b", "#fe6420"],
    "terra": ["#2e0d06", "#681712", "#a61e1f", "#d4462b"],
    "cloth": ["#191954", "#15578a", "#1096c3", "#74effe"],
    "board": ["#003227", "#05582d", "#297537", "#559642"],
    "obsidian": ["#010e1a", "#170916", "#2b1f35", "#3e3551"],
    "staris": ["#191954", "#230c44", "#4f1d83", "#69608b", "#92a3b2"],
    "coal": ["#010000", "#161612", "#2c2c25", "#434137", "#5d778d"],
    "flint": ["#170916", "#2b1f35", "#3e3551", "#69608b", "#bfcad7"],
    "rope": ["#41322c", "#8d4526", "#ab8465", "#cfaa81"],
    "fibre": ["#003227", "#297537", "#559642", "#9bb675"],
    "polymer": ["#681712", "#c56d25", "#fb902f", "#faaf5c", "#fffefe"],
    "white": ["#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "liquid": ["#05582d", "#297537", "#559642", "#9bb675", "#c1f1d8"],
})
GLINT = "#fffefe"


def lump(sp, cx, cy, rx, ry, seed, n=10, jag=0.18):
    """A seeded irregular blob: a rock, a lump of clay."""
    rnd = random.Random(seed)
    pts = []
    for i in range(n):
        a = i / n * 2 * math.pi + rnd.uniform(-0.15, 0.15)
        k = 1 + rnd.uniform(-jag, jag)
        pts.append((cx + math.cos(a) * rx * k, cy + math.sin(a) * ry * k))
    return sp.poly(pts)


def nugget(sp, x, y, ramp, big=False):
    """A 3 x 3 (or 4 x 3) mineral fleck set into a rock, glinted top left."""
    m = sp.rect(x, y, x + (3 if big else 2), y + 2)
    sp.part(m, ramp, "rim")
    sp.plot(x, y, ramp[-1])


# ---------------------------------------------------------------- raw, by hand

def stone(sp):
    sp.part(lump(sp, 16, 16, 14, 12, 4, jag=0.12), R["stone"], "sphere")
    sp.fill(sp.poly([(7, 11), (13, 6), (19, 6), (13, 12)]), R["stone"][4])
    sp.plots([(10, 11), (11, 10)], GLINT)
    sp.outline()


def flint(sp):
    shard = sp.poly([(3, 26), (11, 4), (23, 1), (30, 13), (22, 30)])
    sp.part(shard, R["flint"], "rim", base=2, depth=2)
    facet = sp.poly([(12, 5), (22, 3), (18, 16), (9, 16)])
    sp.part(facet, R["flint"], "flat", base=3)
    sp.part(sp.poly([(19, 17), (28, 14), (22, 28)]), R["flint"], "flat", base=1)
    sp.plots([(14, 7), (15, 7), (14, 8)], GLINT)
    sp.outline()


def log(sp):
    body = sp.rect(2, 8, 23, 25)
    sp.part(body, R["wood"], "cyl_h")
    for x in (8, 15):
        sp.fill(sp.rect(x, 12, x + 3, 12), R["wood"][0])
    sp.fill(sp.rect(10, 20, 14, 20), R["wood"][0])
    end = sp.ellipse(19, 7, 30, 26)
    sp.part(end, R["sand"], "flat", base=2)
    sp.fill(sp.ellipse(21, 10, 28, 23) & ~sp.ellipse(22, 12, 27, 21), R["sand"][1])
    sp.fill(sp.ellipse(23, 14, 26, 19) & ~sp.ellipse(24, 15, 25, 18), R["sand"][1])
    sp.plots([(21, 9), (20, 11)], R["bone"][-1])
    leaf = sp.ellipse(4, 2, 10, 7)
    sp.part(leaf, R["green"], "rim")
    sp.fill(sp.rect(7, 7, 7, 8), R["wood"][1])
    sp.plot(5, 3, R["green"][-1])
    sp.outline()


def fibre(sp):
    for i, (x0, x1) in enumerate([(13, 4), (14, 9), (16, 16), (18, 23), (19, 28)]):
        blade = sp.taper(x0, 20, x1 + (x1 - 16) // 4, 1 + abs(16 - x1) // 2, 4, 1)
        sp.part(blade, R["fibre"] if i % 2 else R["olive"], "rim")
    stems = sp.rect(12, 18, 19, 29)
    sp.part(stems, R["fibre"], "cyl_v")
    band = sp.rect(11, 20, 20, 23)
    sp.part(band, R["rope"], "cyl_h")
    sp.plot(12, 21, R["rope"][-1])
    sp.outline()


def clay(sp):
    sp.part(lump(sp, 16, 18, 14, 11, 11, jag=0.08), R["clay"], "sphere")
    sp.fill(sp.rect(10, 18, 15, 18), R["clay"][1])
    sp.fill(sp.rect(18, 22, 22, 22), R["clay"][1])
    sp.plots([(9, 13), (10, 12), (11, 12)], R["clay"][-1])
    sp.plot(10, 13, GLINT)
    sp.outline()


def sand(sp):
    pile = sp.poly([(1, 28), (8, 14), (14, 5), (18, 5), (24, 14), (30, 28)])
    sp.part(pile, R["sand"], "sphere")
    rnd = random.Random(13)
    for _ in range(14):
        x, y = rnd.randint(7, 25), rnd.randint(14, 25)
        if pile[y, x] and pile[y, x - 1]:
            sp.plot(x, y, R["sand"][rnd.choice([0, 3])])
    sp.plots([(15, 6), (14, 7)], R["bone"][-1])
    sp.outline()


# ---------------------------------------------------------------- raw, by pick

def ore(vein, seed):
    def draw(sp):
        sp.part(lump(sp, 16, 16, 14, 12, seed, jag=0.14), R["orerock"], "sphere")
        rnd = random.Random(seed)
        spots = [(9, 10), (17, 9), (21, 15), (11, 17), (16, 21), (22, 22), (7, 20)]
        for i, (x, y) in enumerate(spots):
            nugget(sp, x + rnd.randint(-1, 1), y + rnd.randint(-1, 1), vein, big=i % 3 == 0)
        sp.outline()
    return draw


def coal(sp):
    for (cx, cy, r, s) in [(11, 20, 8, 1), (21, 21, 8, 2), (16, 12, 8, 3)]:
        sp.part(lump(sp, cx, cy, r, r - 1, s, n=7, jag=0.2), R["coal"], "sphere")
    sp.plots([(13, 7), (8, 16), (19, 17)], R["coal"][-1])
    sp.plot(14, 7, GLINT)
    sp.outline()


# ---------------------------------------------------------------- worked

def planks(sp):
    a = sp.taper(2, 8, 29, 21, 7, 7)
    sp.part(a, R["wood"], "rim")
    b = sp.taper(2, 23, 29, 10, 7, 7)
    sp.part(b, R["wood"], "rim", base=2)
    sp.fill(sp.line([(7, 21), (25, 12)]), R["wood"][1])
    sp.plots([(6, 21), (26, 12)], R["steel"][-1])
    sp.outline()


def ingot(ramp):
    def draw(sp):
        top = sp.poly([(8, 6), (24, 6), (28, 14), (4, 14)])
        front = sp.poly([(4, 15), (28, 15), (30, 26), (2, 26)])
        sp.part(front, ramp, "rim", base=len(ramp) - 4)
        sp.part(top, ramp, "flat", base=len(ramp) - 2)
        sp.fill(sp.rect(9, 7, 22, 7), ramp[-1])
        sp.plots([(8, 8), (9, 9), (7, 9)], GLINT)
        sp.fill(sp.rect(4, 17, 4, 23), ramp[-2])
        sp.outline()
    return draw


def glass(sp):
    pane = sp.rect(5, 4, 26, 27)
    sp.part(pane, R["glass"], "rim", base=2)
    sp.fill(sp.rect(7, 6, 24, 25) & ~sp.rect(8, 7, 23, 24), R["glass"][3])
    for k in (0, 5):
        sp.fill(sp.line([(10 + k, 20), (18 + k, 9)], 2) & sp.rect(8, 7, 23, 24), R["glass"][4])
    sp.plots([(8, 7), (9, 7), (8, 8)], GLINT)
    sp.outline()


def brick(sp):
    for (x0, y0) in [(3, 18), (16, 18), (9, 9)]:
        top = sp.poly([(x0 + 2, y0), (x0 + 13, y0), (x0 + 11, y0 + 3), (x0, y0 + 3)])
        face = sp.rect(x0, y0 + 3, x0 + 11, y0 + 9)
        side = sp.poly([(x0 + 12, y0 + 3), (x0 + 13, y0 + 1), (x0 + 13, y0 + 7), (x0 + 12, y0 + 9)])
        sp.part(face, R["brick"], "rim", base=2)
        sp.part(side, R["brick"], "flat", base=1, seam=False)
        sp.part(top, R["brick"], "flat", base=3, seam=False)
        sp.plot(x0 + 3, y0 + 1, R["flame"][3])
    sp.outline()


def cloth(sp):
    bolt = sp.rect(4, 9, 22, 25)
    sp.part(bolt, R["cloth"], "cyl_h")
    end = sp.ellipse(19, 8, 28, 26)
    sp.part(end, R["cloth"], "flat", base=2)
    sp.fill(sp.ellipse(21, 12, 26, 22), R["cloth"][1])
    sp.fill(sp.ellipse(23, 15, 24, 19), R["ink"][1])
    sp.fill(sp.rect(4, 22, 9, 28) & ~sp.poly([(4, 28), (9, 28), (6, 25)]), R["cloth"][2])
    for x in (9, 14):
        sp.fill(sp.rect(x, 10, x, 23), R["cloth"][3])
    sp.plots([(6, 11), (7, 11)], GLINT)
    sp.outline()


def rope(sp):
    for r, ramp_i in [(13, 0), (9, 1), (5, 2)]:
        ring = sp.circle(16, 16, r) & ~sp.circle(16, 16, r - 3)
        sp.part(ring, R["rope"], "rim", base=2)
    for a in range(0, 360, 30):
        x = 16 + math.cos(math.radians(a)) * 11.5
        y = 16 + math.sin(math.radians(a)) * 11.5
        sp.plot(round(x), round(y), R["rope"][1])
    tail = sp.taper(25, 22, 30, 29, 3, 3)
    sp.part(tail, R["rope"], "rim", base=2)
    sp.plot(11, 4, R["rope"][-1])
    sp.outline()


# ---------------------------------------------------------------- advanced

def circuit(sp):
    board = sp.rect(3, 5, 28, 26)
    sp.part(board, R["board"], "rim", base=2)
    gold = R["gold"]
    for pts in [[(4, 9), (10, 9), (12, 12)], [(4, 21), (9, 21), (11, 18)], [(27, 10), (21, 10)],
                [(27, 21), (21, 21), (20, 19)], [(15, 6), (15, 11)], [(17, 25), (17, 20)]]:
        sp.fill(sp.line(pts), gold[2])
    for x, y in [(4, 9), (4, 21), (27, 10), (27, 21), (15, 6), (17, 25)]:
        sp.fill(sp.rect(x - 1, y - 1, x, y), gold[3])
    chip = sp.rect(11, 12, 20, 19)
    sp.part(chip, R["soot"], "rim", base=1)
    sp.plots([(12, 13), (13, 13)], R["soot"][-1])
    sp.plot(6, 23, R["red"][-1])
    sp.outline()


def reagent(sp):
    neck = sp.rect(13, 3, 18, 12)
    bulb = sp.poly([(13, 11), (18, 11), (27, 26), (25, 28), (6, 28), (4, 26)])
    sp.part(bulb, R["paper"], "rim")
    liquid = erode(bulb) & sp.rect(0, 18, 31, 31)
    sp.part(liquid, R["liquid"], "rim", base=2, seam=False)
    sp.fill(sp.rect(9, 18, 22, 18), R["liquid"][4])
    sp.part(neck, R["paper"], "cyl_v")
    sp.part(sp.rect(12, 2, 19, 4), R["wood"], "rim")
    sp.plots([(14, 6), (14, 7), (10, 20)], GLINT)
    sp.plots([(14, 23), (19, 21)], R["liquid"][4])
    sp.outline()


def alloy_plate(sp):
    plate = sp.poly([(4, 8), (24, 4), (28, 23), (8, 28)])
    sp.part(plate, R["steelb"], "rim", base=3, depth=2)
    sp.fill(sp.line([(8, 18), (14, 12), (19, 17), (24, 11)]), R["bronze"][3])
    for x, y in [(7, 9), (23, 6), (26, 22), (9, 26)]:
        sp.plot(x, y, R["steelb"][-1])
        sp.plot(x + 1, y + 1, R["steelb"][0])
    sp.plots([(6, 9), (5, 9)], GLINT)
    sp.outline()


def polymer(sp):
    drop = sp.circle(16, 20, 9) | sp.poly([(16, 2), (24, 16), (8, 16)])
    sp.part(drop, R["polymer"], "sphere", base=3)
    sp.fill(sp.rect(11, 14, 12, 18), R["polymer"][4])
    sp.plot(11, 13, GLINT)
    sp.outline()


# ---------------------------------------------------------------- alien

def frost_crystal(sp):
    sp.part(sp.taper(10, 26, 5, 10, 6, 1), R["cyan"], "rim")
    sp.part(sp.taper(22, 26, 27, 9, 6, 1), R["cyan"], "rim")
    sp.part(sp.taper(16, 28, 16, 2, 8, 1), R["cyan"], "rim", base=3)
    sp.fill(sp.line([(15, 25), (15, 6)]), R["cyan"][4])
    base = sp.ellipse(6, 25, 26, 30)
    sp.part(base, R["white"], "rim")
    sp.plots([(14, 6), (15, 4)], GLINT)
    sp.outline()
    for x, y in [(4, 4), (27, 4)]:
        sp.plot(x, y, GLINT, glow=True)
        sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], R["cyan"][3], glow=True)


def glowcap(sp):
    stem = sp.poly([(13, 15), (19, 15), (20, 28), (12, 28)])
    sp.part(stem, R["bone"], "cyl_v")
    cap = sp.ellipse(3, 4, 28, 22) & sp.rect(0, 0, 31, 16)
    sp.part(cap, R["violet"], "sphere", base=3)
    sp.part(sp.rect(4, 15, 27, 17), R["violet"], "flat", base=1)
    sp.outline()
    for x, y in [(9, 9), (16, 7), (22, 10), (12, 13), (19, 13)]:
        sp.fill(sp.rect(x, y, x + 1, y + 1), R["cyan"][3], glow=True)
        sp.plot(x, y, GLINT, glow=True)


def magma_glass(sp):
    rock = lump(sp, 16, 17, 13, 11, 65, n=8, jag=0.2)
    sp.part(rock, R["obsidian"], "sphere", base=2)
    sp.part(sp.poly([(8, 9), (17, 6), (15, 13)]), R["obsidian"], "flat", base=3)
    sp.outline()
    for pts in [[(6, 18), (11, 16), (15, 20), (20, 17), (26, 19)], [(15, 20), (14, 26)], [(20, 17), (22, 11)]]:
        sp.fill(sp.line(pts) & erode(rock), R["flame"][1], glow=True)
    for x, y in [(11, 16), (20, 17), (15, 20)]:
        sp.plot(x, y, R["flame"][3], glow=True)
    sp.plots([(10, 9), (11, 8)], R["glass"][3])


def star_iron(sp):
    sp.part(lump(sp, 16, 17, 13, 11, 67, n=9, jag=0.16), R["staris"], "sphere", base=3)
    for x, y in [(9, 12), (19, 10), (14, 19), (22, 20)]:
        nugget(sp, x, y, R["iron"])
    sp.outline()
    for x, y in [(5, 6), (26, 4)]:
        sp.plot(x, y, GLINT, glow=True)
        sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], R["violet"][4], glow=True)


# ---------------------------------------------------------------- stations

def workbench(sp):
    for x in (5, 23):
        sp.part(sp.rect(x, 16, x + 3, 29), R["wood"], "cyl_v")
    sp.part(sp.rect(8, 22, 23, 23), R["wood"], "flat", base=1)
    top = sp.rect(2, 12, 29, 16)
    sp.part(top, R["wood"], "cyl_h")
    sp.part(sp.rect(16, 5, 18, 11), R["wood"], "cyl_v")
    sp.part(sp.rect(12, 3, 22, 6), R["iron"], "rim", base=2)
    board = sp.rect(3, 9, 11, 11)
    sp.part(board, R["sand"], "rim", base=2)
    sp.plots([(24, 10), (26, 10)], R["iron"][2])
    sp.plots([(13, 4), (14, 4)], GLINT)
    sp.outline()


def forge(sp):
    sp.part(sp.rect(20, 2, 25, 10), R["stone"], "cyl_v")
    body = sp.rect(3, 9, 28, 29)
    sp.part(body, R["stone"], "rim", base=2, depth=2)
    for y in (14, 19, 24):
        off = 0 if y % 2 else 4
        sp.fill(sp.rect(4, y, 27, y), R["stone"][1])
        for x in range(4 + off, 27, 8):
            sp.fill(sp.rect(x, y - 4, x, y), R["stone"][1])
    mouth = sp.ellipse(9, 14, 22, 30) & sp.rect(0, 0, 31, 27)
    sp.part(mouth, R["ink"], "flat", base=0)
    sp.outline()
    sp.part(sp.poly([(11, 26), (13, 19), (15, 22), (16, 16), (18, 22), (20, 20), (21, 26)]), R["flame"], "inset", glow=True)
    sp.plots([(4, 10), (5, 10)], R["stone"][-1])


def kiln(sp):
    dome = sp.ellipse(3, 4, 28, 40) & sp.rect(0, 0, 31, 28)
    sp.part(dome, R["clay"], "sphere", base=2)
    for y in (10, 16, 22):
        sp.fill(dome & sp.rect(0, y, 31, y), R["clay"][1])
    for y, xs in [(13, (10, 18)), (19, (7, 14, 22)), (25, (10, 18, 25))]:
        for x in xs:
            sp.fill(dome & sp.rect(x, y - 2, x, y), R["clay"][1])
    sp.part(sp.rect(2, 27, 29, 29), R["stone"], "rim", base=2)
    door = sp.ellipse(11, 15, 20, 34) & sp.rect(0, 0, 31, 26)
    sp.part(door, R["terra"], "flat", base=0)
    sp.outline()
    sp.part(sp.ellipse(12, 18, 19, 32) & sp.rect(0, 0, 31, 26), R["amber"], "inset", glow=True)
    sp.plots([(9, 7), (10, 6)], R["clay"][-1])


def loom(sp):
    for x in (3, 26):
        sp.part(sp.rect(x, 3, x + 2, 29), R["wood"], "cyl_v")
    sp.part(sp.rect(3, 3, 28, 6), R["wood"], "cyl_h")
    sp.part(sp.rect(3, 25, 28, 27), R["wood"], "cyl_h")
    for x in range(7, 25, 3):
        sp.fill(sp.rect(x, 7, x, 15), R["paper"][1])
    cloth = sp.rect(7, 15, 24, 24)
    sp.part(cloth, R["red"], "rim", base=2)
    for y in (17, 20, 23):
        sp.fill(sp.rect(7, y, 24, y), R["red"][1])
    sp.part(sp.rect(6, 12, 25, 13), R["sand"], "cyl_h")
    sp.plots([(4, 4), (5, 4)], R["wood"][-1])
    sp.outline()


def chem_lab(sp):
    sp.part(sp.rect(2, 25, 29, 28), R["wood"], "cyl_h")
    # round flask, left
    fl = sp.circle(10, 18, 6) | sp.rect(8, 7, 12, 13)
    sp.part(fl, R["paper"], "rim")
    sp.part(sp.circle(10, 19, 5) & sp.rect(0, 17, 31, 31), R["liquid"], "rim", base=2, seam=False)
    # tall tube, right
    tube = sp.rect(21, 5, 26, 24)
    sp.part(tube, R["paper"], "cyl_v")
    sp.part(sp.rect(22, 13, 25, 23), R["violet"], "rim", base=3, seam=False)
    # the bridge between
    sp.part(sp.line([(11, 7), (11, 4), (23, 4), (23, 5)], 2), R["iron"], "flat", base=2)
    sp.plots([(9, 9), (22, 7)], GLINT)
    sp.outline()
    sp.plots([(10, 1), (12, 2)], R["liquid"][4], glow=True)
    sp.plots([(23, 1)], R["violet"][4], glow=True)


def fabricator(sp):
    body = sp.rect(3, 6, 28, 29)
    sp.part(body, R["white"], "rim", base=2, depth=2)
    sp.part(sp.rect(13, 2, 18, 5), R["steelb"], "rim", base=3)
    screen = sp.rect(6, 9, 20, 17)
    sp.part(screen, R["night"], "flat", base=1)
    sp.fill(sp.line([(7, 15), (10, 15), (12, 11), (15, 15), (19, 12)]), R["glass"][3])
    for i, ramp in enumerate([R["green"], R["amber"], R["red"]]):
        sp.part(sp.rect(23, 9 + i * 3, 25, 10 + i * 3), ramp, "flat", base=len(ramp) - 1)
    bay = sp.rect(7, 20, 24, 27)
    sp.part(bay, R["steelb"], "rim", base=1)
    sp.part(sp.rect(14, 20, 17, 23), R["bronze"], "rim", base=3)
    sp.plots([(4, 7), (5, 7)], GLINT)
    sp.outline()


# ---------------------------------------------------------------- tool tiers

def pick(head):
    def draw(sp):
        handle = sp.taper(5, 29, 21, 9, 4, 4)
        sp.part(handle, R["wood"], "rim")
        sp.part(sp.taper(5, 29, 8, 25, 5, 5), R["rope"], "rim")
        blade = sp.poly([(1, 11), (6, 5), (14, 2), (23, 3), (29, 7), (31, 14), (29, 15),
                         (25, 10), (19, 8), (11, 8), (3, 12)])
        sp.part(blade, head, "rim", base=len(head) - 2, depth=1)
        sp.part(sp.rect(17, 3, 22, 9), R["wood"], "rim", base=1)
        sp.plots([(8, 5), (9, 4), (7, 6)], GLINT)
        sp.outline()
    return draw


ICONS = {
    # raw, by hand
    "stone": stone, "flint": flint, "log": log, "fibre": fibre, "clay": clay, "sand": sand,
    # raw, by pick
    "copper-ore": ore(R["copper"], 21), "tin-ore": ore(R["white"], 23), "iron-ore": ore(R["terra"], 25),
    "coal": coal,
    # worked
    "planks": planks, "copper-ingot": ingot(R["copper"]), "bronze-ingot": ingot(R["bronze"]),
    "iron-ingot": ingot(R["iron"]), "steel-ingot": ingot(R["steelb"]), "glass": glass, "brick": brick,
    "cloth": cloth, "rope": rope,
    # advanced
    "circuit": circuit, "reagent": reagent, "alloy-plate": alloy_plate, "polymer": polymer,
    # alien
    "frost-crystal": frost_crystal, "glowcap": glowcap, "magma-glass": magma_glass, "star-iron": star_iron,
    # stations
    "workbench": workbench, "forge": forge, "kiln": kiln, "loom": loom, "chem-lab": chem_lab,
    "fabricator": fabricator,
    # tool tiers
    "pick-stone": pick(R["stone"]), "pick-bronze": pick(R["bronze"]), "pick-iron": pick(R["iron"]),
    "pick-steel": pick(R["cyan"]),
}


def references(n=6):
    from PIL import Image
    refs = sorted(p for p in glob.glob(os.path.join(SKILL, "references", "*.png")) if not p.endswith("_contact.png"))
    imgs = [(os.path.basename(p)[:-4], Image.open(p).convert("RGBA")) for p in refs]
    return [(k, im) for k, im in imgs if im.size == (56, 56)][:n]


def main():
    os.makedirs(OUT, exist_ok=True)
    items = []
    for name, draw in ICONS.items():
        sp = Sprite(N, N)
        draw(sp)
        img = sp.image()
        save(img, os.path.join(OUT, name + ".png"), source="balatro-pixel-art pixelkit")
        items.append((name, img))
    print("wrote", len(items), "icons to", OUT)
    if "--sheet" in sys.argv:
        path = sys.argv[sys.argv.index("--sheet") + 1]
        sheet(references() + items, scale=3, cols=9).save(path)
        print("sheet", path)


if __name__ == "__main__":
    main()
