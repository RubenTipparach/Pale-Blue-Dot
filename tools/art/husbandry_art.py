"""Pixel art for the husbandry mockup (docs/mockups/husbandry.html, "Pens, Nests and Pedigrees").

Drawn in code with the balatro-pixel-art kit (.claude/skills/balatro-pixel-art/scripts/pixelkit.py),
the solitaire collection's style: a tiny native canvas, one bold subject, short ramps lit from the
top left, a dark seam between parts, a 1 px outline, single-pixel glints, hard alpha, every colour
from Sweaft-64 (`save()` refuses anything else).

The animals reuse the animals mockup's (M7) body-plan functions (`tools/art/animals_art.py`,
imported, not copied), so a yard hen or a rootling is drawn by the same `quad` and `bird` as M7's
meadow hen and tusk boar, and its hide strip by the same `hide_strip`. Only what no other mockup has
is drawn here:

- portraits, 36 x 36 (shown at 2x): the new domestic breeds and the new wild species;
- hides, 256 x 32: eight seamless 32 px tiles per animal for the 3D farmstead's box-built bodies;
- products, feed and tools, 24 x 24 (shown at 1x, 2x);
- needs, 16 x 16 (shown at 1x, 2x);
- two 32 x 32 floor textures (straw bedding, sty mud).

    python tools/art/husbandry_art.py            # writes docs/mockups/husbandry/*.png
    python tools/art/husbandry_art.py --sheet S  # also a contact sheet (keep it out of the repo)
"""

import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
sys.path.insert(0, HERE)
from pixelkit import Sprite, save, sheet  # noqa: E402
import animals_art as aa  # noqa: E402  (M7's body plans and hide tiles)

OUT = os.path.join(ROOT, "docs", "mockups", "husbandry")
SRC = "tools/art/husbandry_art.py (balatro-pixel-art kit)"
R = aa.R
WHITE = aa.WHITE
EYE = aa.EYE

# New ramps, every tone from Sweaft-64, dark to light.
R.update({
    "pig": ["#841c1d", "#c34988", "#dd576d", "#e69d69", "#eab798"],
    "pigsnout": ["#841c1d", "#dd576d", "#eab798"],
    "cream": ["#ab8465", "#cfaa81", "#eab798", "#fffefe"],
    "chestnut": ["#2e0d06", "#561c27", "#733126", "#a8633e"],
    "mane": ["#010000", "#161612", "#2c2c25"],
    "collie": ["#010000", "#161612", "#2c2c25", "#434137"],
    "tabby": ["#41322c", "#725244", "#ab8465", "#cfaa81"],
    "mouse": ["#2c2c25", "#434137", "#a09f91", "#bfcad7"],
    "duckegg": ["#53998b", "#84e3b5", "#c1f1d8"],
    "honey": ["#681712", "#c56d25", "#fb902f", "#faaf5c"],
    "dung": ["#2e0d06", "#41322c", "#725244"],
    "straw": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "leaf": ["#05582d", "#297537", "#559642", "#9bb675"],
    "beebody": ["#161612", "#c56d25", "#fb902f", "#faaf5c"],
})


def Q(**p):
    return (aa.quad, p)


def B(**p):
    return (aa.bird, p)


# ---------- portraits (36 x 36), M7's body plans with new numbers ----------------------------------
PORTRAITS = {
    # Tenebris domestic breeds
    "yard_hen": B(col="cream", wingc="sand", tailc="soot", crest="comb", beak_len=0.5, k=1.0),
    "cockerel": B(col="rust", wingc="brass", tailc="green", headc="brass", crest="comb", beak_len=0.5, k=1.08, lift=1),
    "chick": B(col="yolk", wingc="yolk", tailc="yolk", beak_len=0.4, k=0.62),
    "rootling": Q(col="pig", ears="flop", earc="pig", tail="thin", tailc="pig", legs=0.5, long=1.1, tall=1.1, snout=0.55, head=1.15, snoutc="pigsnout", hoof=1, lw=4),
    "fellrunner": Q(col="chestnut", mane="mane", legs=1.25, long=1.0, tall=0.85, neck=0.75, head=0.95, snout=1.0, hoof=1, ears="point", tail="tuft", tailc="mane", dx=1),
    "herd_hound": Q(col="collie", headc="collie", snoutc="paper", legc="paper", ears="point", tail="bush", tipc="paper", legs=0.75, tall=0.8, snout=1.0, dx=-1),
    "hedge_cat": Q(col="tabby", pattern="stripes", patc="#41322c", ears="point", tail="long", legs=0.55, long=0.85, tall=0.8, snout=0.2, head=1.05, glow_eyes="#9bb675", dx=1),
    "yard_dabbler": B(col="paper", wingc="paper", tailc="paper", beakc="brass", legc="brass", beak_len=0.9, k=1.0),
    "clover_bee": (aa.insect, dict(col="beebody", abc="beebody", wingc="paper", wing=0.75, bands="ink")),
    # Tenebris new wild species
    "field_mouse": Q(col="mouse", ears="round", tail="long", legs=0.35, long=0.7, tall=0.75, snout=0.6, head=1.1, dx=2),
}

# Hides for the farmstead's box-built bodies (same eight-tile layout as M7's).
HIDES = ["yard_hen", "cockerel", "chick", "rootling", "fellrunner", "herd_hound", "hedge_cat", "yard_dabbler"]


# ---------- products, feed and tools (24 x 24) ----------------------------------------------------
def egg_mask(sp, cx=12, cy=12.5, w=7, h=10):
    return sp.poly([(cx + w * math.sin(t) * (1 - 0.2 * math.cos(t)), cy - h * math.cos(t)) for t in [2 * math.pi * i / 40 for i in range(40)]])


def p_duck_egg(sp):
    sp.part(egg_mask(sp), R["duckegg"], "sphere")
    sp.plots([(8, 5), (9, 4)], WHITE)


def p_fertile_egg(sp):
    # an egg on a twist of nest straw: kept for the incubator, never eaten
    sp.part(sp.ellipse(1, 16, 22, 22), R["straw"], "rim")
    for x in range(3, 21, 3):
        sp.plot(x, 19, R["straw"][0])
    sp.part(egg_mask(sp, 12, 10.5, 6, 9), R["sand"], "sphere")
    sp.plots([(9, 4), (10, 3)], WHITE)


def p_honey(sp):
    sp.part(sp.rect(7, 2, 16, 5), R["wood"], "rim")
    jar = sp.poly([(6, 6), (17, 6), (19, 9), (19, 21), (4, 21), (4, 9)])
    sp.part(jar, R["honey"], "cyl_v")
    sp.part(sp.rect(5, 12, 18, 16) & jar, R["paper"], "flat")
    sp.fill(sp.rect(9, 13, 14, 15), R["honey"][1])
    sp.fill(sp.rect(6, 8, 7, 11), R["honey"][-1])
    sp.plot(6, 8, WHITE)


def p_wax(sp):
    m = sp.poly([(3, 9), (12, 4), (21, 9), (21, 17), (12, 22), (3, 17)])
    sp.part(m, R["straw"], "rim", depth=2)
    for cy in (9, 15):
        for cx in (8, 13, 18):
            c = sp.poly([(cx - 2, cy - 1), (cx, cy - 2), (cx + 2, cy - 1), (cx + 2, cy + 1), (cx, cy + 2), (cx - 2, cy + 1)]) & m
            sp.fill(c, R["straw"][1])
    sp.plot(8, 7, WHITE)


def p_manure(sp):
    for x0, y0, x1, y1 in ((2, 13, 13, 21), (10, 12, 22, 21), (6, 6, 17, 15)):
        sp.part(sp.ellipse(x0, y0, x1, y1), R["dung"], "sphere")
    sp.plots([(9, 8), (14, 14)], R["dung"][-1])
    # a straw stalk through it
    sp.part(sp.line([(15, 5), (20, 10)], 1), R["straw"], "flat")


def p_tallow(sp):
    # two tallow candles bound with twine: Wick's tallowback fat
    for x0, x1 in ((4, 10), (12, 18)):
        sp.part(sp.rect(x0, 7, x1, 21), R["bone"], "cyl_v")
        sp.part(sp.line([((x0 + x1) // 2, 3), ((x0 + x1) // 2, 6)], 1), R["ink"], "flat")
    sp.part(sp.rect(3, 14, 19, 15), R["rust"], "flat")
    sp.plot(5, 8, WHITE)


def p_steak(sp):
    m = sp.poly([(3, 7), (9, 3), (17, 3), (21, 8), (20, 16), (14, 21), (6, 20), (2, 14)])
    sp.part(m, R["meat"], "rim", depth=2)
    sp.part(sp.ellipse(5, 6, 13, 13) & m, R["pink"], "rim")
    sp.part(sp.line([(14, 5), (14, 19)], 2) | sp.line([(9, 5), (19, 5)], 2), R["bone"], "flat")
    sp.plots([(6, 6), (7, 5)], WHITE)


def p_chop(sp):
    sp.part(sp.line([(14, 12), (21, 4)], 2), R["bone"], "flat")
    sp.part(sp.circle(21, 4, 2), R["bone"], "rim")
    m = sp.ellipse(2, 8, 17, 21)
    sp.part(m, R["meat"], "sphere")
    sp.part(sp.ellipse(4, 15, 14, 21) & m, R["bone"], "rim")
    sp.plots([(6, 10), (7, 10)], WHITE)


def p_ham(sp):
    sp.part(sp.rect(15, 3, 18, 8), R["bone"], "rim")
    sp.part(sp.circle(16.5, 3, 2.5), R["bone"], "rim")
    m = sp.poly([(12, 7), (20, 9), (22, 16), (17, 22), (6, 22), (2, 16), (5, 10)])
    sp.part(m, R["pig"], "sphere")
    sp.part(sp.ellipse(4, 14, 13, 21) & m, R["meat"], "rim")
    sp.plots([(7, 11), (8, 10)], WHITE)


def p_drumstick(sp):
    sp.part(sp.line([(13, 13), (20, 20)], 2), R["bone"], "flat")
    sp.part(sp.circle(20, 20, 2) | sp.circle(21, 18, 1.5), R["bone"], "rim")
    m = sp.ellipse(2, 2, 16, 16)
    sp.part(m, R["pink"], "sphere")
    sp.plots([(6, 5), (7, 5)], WHITE)


def p_grain_feed(sp):
    sack = sp.poly([(5, 6), (18, 6), (21, 13), (20, 21), (3, 21), (2, 13)])
    sp.part(sack, R["bone"], "rim", depth=2)
    sp.part(sp.rect(7, 3, 16, 6), R["bone"], "rim")
    sp.part(sp.rect(6, 6, 17, 7), R["rust"], "flat")
    for x, y in ((8, 12), (12, 15), (15, 11), (10, 18)):
        sp.fill(sp.rect(x, y, x + 1, y), R["straw"][2])
    sp.plot(5, 9, WHITE)


def p_hay(sp):
    m = sp.rect(2, 6, 21, 20)
    sp.part(m, R["straw"], "rim", depth=2)
    for y in range(8, 20, 3):
        for x in range(3, 21, 4):
            sp.plot(x + (y % 2), y, R["straw"][1])
    for x in (7, 16):
        sp.fill(sp.rect(x, 6, x, 20), R["rust"][1])
    sp.plot(4, 7, WHITE)


def t_shears(sp):
    for s in (0, 1):
        blade = sp.poly([(11, 12), (3 + s * 4, 2), (6 + s * 4, 2), (13, 11)])
        sp.part(blade, R["steel"], "rim")
    sp.part(sp.rect(10, 11, 13, 13), R["metal"], "flat")
    sp.part(sp.line([(10, 13), (5, 21)], 3) | sp.line([(13, 13), (18, 21)], 3), R["wood"], "rim")
    sp.plot(5, 4, WHITE)


def t_pail(sp):
    sp.part(sp.line([(4, 8), (6, 3), (12, 1), (18, 3), (20, 8)], 1), R["soot"], "flat")
    pail = sp.poly([(3, 8), (20, 8), (18, 21), (5, 21)])
    sp.part(pail, R["wood"], "cyl_v")
    for y in (11, 18):
        sp.fill(sp.rect(3, y, 20, y) & pail, R["metal"][1])
    sp.part(sp.ellipse(4, 6, 19, 10), R["wood"], "flat")
    sp.fill(sp.ellipse(6, 7, 17, 9), R["wood"][0])


def t_lead(sp):
    ring = sp.circle(12, 12, 9) & ~sp.circle(12, 12, 5)
    sp.part(ring, R["wood"], "rim")
    for a in range(0, 360, 45):
        x, y = 12 + 7 * math.cos(math.radians(a)), 12 + 7 * math.sin(math.radians(a))
        sp.plot(round(x), round(y), R["wood"][0])
    sp.part(sp.line([(18, 18), (22, 22)], 2), R["wood"], "flat")
    sp.part(sp.rect(17, 2, 21, 6), R["steel"], "rim")


def t_brush(sp):
    sp.part(sp.rect(3, 6, 20, 12), R["wood"], "rim")
    for x in range(4, 20, 2):
        sp.part(sp.rect(x, 13, x, 19), R["soot"], "flat")
    sp.plot(4, 7, WHITE)


def t_smoker(sp):
    can = sp.rect(5, 8, 15, 21)
    sp.part(can, R["metal"], "cyl_v")
    sp.part(sp.poly([(5, 8), (10, 3), (15, 8)]), R["metal"], "rim")
    sp.part(sp.rect(16, 12, 21, 19), R["wood"], "rim")
    aa.outline(sp)
    for x, y, r in ((9, 2, 1.6), (6, 1, 1.2), (12, 0.5, 1.0)):
        sp.fill(sp.circle(x, y, r), R["paper"][1], glow=True)


def t_incubator(sp):
    box = sp.rect(2, 7, 21, 21)
    sp.part(box, R["wood"], "rim", depth=2)
    sp.part(sp.rect(5, 10, 18, 18), R["glass"], "flat")
    for x in (7, 12):
        sp.part(egg_mask(sp, x + 2, 15, 2.5, 3.2), R["sand"], "sphere")
    sp.part(sp.rect(9, 3, 14, 7), R["metal"], "rim")
    aa.outline(sp)
    sp.fill(sp.rect(10, 4, 13, 5), R["flame"][3], glow=True)
    sp.plot(11, 4, WHITE, glow=True)


def t_nest(sp):
    sp.part(sp.ellipse(1, 11, 22, 22), R["straw"], "rim")
    for cx in (8, 15, 11):
        sp.part(egg_mask(sp, cx, 13 if cx != 11 else 11, 3.2, 4.2), R["sand"], "sphere")
    for x in range(3, 21, 3):
        sp.plot(x, 19, R["straw"][0])
    sp.plot(7, 10, WHITE)


PRODUCTS = {
    "duck_egg": p_duck_egg, "fertile_egg": p_fertile_egg, "honey": p_honey, "wax": p_wax, "manure": p_manure,
    "tallow": p_tallow, "cut_steak": p_steak, "cut_chop": p_chop, "cut_ham": p_ham, "cut_drumstick": p_drumstick,
    "feed_grain": p_grain_feed, "feed_hay": p_hay,
    "tool_shears": t_shears, "tool_pail": t_pail, "tool_lead": t_lead, "tool_brush": t_brush, "tool_smoker": t_smoker,
    "tool_incubator": t_incubator, "tool_nest": t_nest,
}


# ---------- needs (16 x 16) ----------------------------------------------------------------------
def n_food(sp):
    sp.part(sp.line([(8, 14), (8, 3)], 1), R["straw"], "flat")
    for y in (3, 6, 9):
        sp.part(sp.ellipse(4, y, 7, y + 2) | sp.ellipse(9, y, 12, y + 2), R["straw"], "rim")


def n_water(sp):
    m = sp.poly([(8, 1), (13, 9), (12, 13), (8, 15), (4, 13), (3, 9)])
    sp.part(m, R["cyan"], "sphere")
    sp.plot(6, 8, WHITE)


def n_shelter(sp):
    sp.part(sp.rect(3, 8, 12, 14), R["wood"], "rim")
    sp.part(sp.poly([(1, 8), (8, 2), (14, 8)]), R["straw"], "rim")
    sp.fill(sp.rect(7, 10, 8, 14), R["wood"][0])


def n_space(sp):
    m = sp.poly([(8, 1), (14, 4.5), (14, 11.5), (8, 15), (2, 11.5), (2, 4.5)])
    sp.part(m, R["leaf"], "rim")
    sp.fill(sp.poly([(8, 5), (11, 7), (11, 10), (8, 12), (5, 10), (5, 7)]), R["leaf"][1])


def n_mood(sp):
    m = sp.circle(5, 6, 3.4) | sp.circle(11, 6, 3.4) | sp.poly([(1.8, 7), (14.2, 7), (8, 14)])
    sp.part(m, R["red"], "sphere")
    sp.plot(4, 4, WHITE)


def n_health(sp):
    sp.part(sp.rect(6, 2, 10, 14) | sp.rect(2, 6, 14, 10), R["paper"], "rim")
    sp.fill(sp.rect(7, 3, 9, 13) | sp.rect(3, 7, 13, 9), R["red"][2])


NEEDS = {"need_food": n_food, "need_water": n_water, "need_shelter": n_shelter, "need_space": n_space, "need_mood": n_mood, "need_health": n_health}


# ---------- 32 x 32 floor textures ----------------------------------------------------------------
def tex_straw():
    rs = np.random.RandomState(11)
    t = aa._tile(R["straw"][2])
    for _ in range(70):
        x, y, n = rs.randint(32), rs.randint(32), rs.randint(3, 7)
        c = R["straw"][rs.choice([0, 1, 3])]
        dx = rs.choice([-1, 1])
        for i in range(n):
            aa._put(t, x + i, y + (i * dx) // 3, c)
    return Image.fromarray(t, "RGB").convert("RGBA")


def tex_mud():
    rs = np.random.RandomState(5)
    t = aa._tile(R["dung"][1])
    for _ in range(14):
        aa._disc(t, rs.randint(32), rs.randint(32), 1.5 + rs.rand() * 2.5, R["dung"][0])
    for _ in range(30):
        aa._put(t, rs.randint(32), rs.randint(32), R["dung"][2])
    return Image.fromarray(t, "RGB").convert("RGBA")


def draw(fn, size, *args):
    sp = Sprite(size, size)
    fn(sp, *args)
    if not getattr(sp, "outlined", False):
        aa.outline(sp)
    return sp.image()


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    items, hides = [], []
    for name, (fn, p) in PORTRAITS.items():
        img = draw(fn, aa.PW, p)
        save(img, os.path.join(OUT, name + ".png"), SRC)
        items.append((name, img))
    for name, fn in PRODUCTS.items():
        img = draw(fn, 24)
        save(img, os.path.join(OUT, name + ".png"), SRC)
        items.append((name, img))
    for name, fn in NEEDS.items():
        img = draw(fn, 16)
        save(img, os.path.join(OUT, name + ".png"), SRC)
        items.append((name, img))
    for name in HIDES:
        fn, p = PORTRAITS[name]
        img = aa.hide_strip(name, fn, p)
        save(img, os.path.join(OUT, "hide_" + name + ".png"), SRC)
        hides.append(("hide_" + name, img))
    for name, img in (("tex_straw", tex_straw()), ("tex_mud", tex_mud())):
        save(img, os.path.join(OUT, name + ".png"), SRC)
        hides.append((name, img))
    print("wrote", len(items) + len(hides), "pngs to", OUT)
    if "--sheet" in argv:
        out = argv[argv.index("--sheet") + 1]
        sheet(items, scale=4, cols=8, cell=(36 * 4, 36 * 4)).save(out)
        sheet(hides, scale=3, cols=2).save(out.replace(".png", "_hides.png"))
        print("sheet", out)


if __name__ == "__main__":
    main(sys.argv[1:])
