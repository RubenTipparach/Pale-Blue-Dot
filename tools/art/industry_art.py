"""Machine, generator and item icons, and the mockup-only textures, for the industry mockup
(docs/mockups/industry.html): strip mining, refining, ether mining, oil and power.

Pixel art drawn in code with the balatro-pixel-art kit (`.claude/skills/balatro-pixel-art`), the
solitaire collection's style: a 32 x 32 native canvas, one bold subject, 3 to 5 tone ramps lit from
the top left, dark seams, a 1 px outline, single-pixel glints, no anti-aliasing, every colour from
Sweaft-64 (the kit's save() refuses anything else). Shown only at 32, 64 or 96 px, nearest.

Only what the game does not have is drawn here. The ground, ore, coal and stone are the game's own
atlas tiles, walls and floors its settlement textures, and the tools and lamps its item icons, all
copied byte for byte into docs/mockups/industry/game/. The raw ore, coal and ingot icons are the
crafting mockup's (tools/art/crafting_art.py), reused, not redrawn.

    python tools/art/industry_art.py                 # writes docs/mockups/industry/*.png
    python tools/art/industry_art.py --sheet OUT.png # and a 4x contact sheet beside the references
"""

import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art")
sys.path.insert(0, os.path.join(SKILL, "scripts"))
from pixelkit import RAMPS, Sprite, rgb, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "industry")
WHITE = "#fffefe"
R = dict(RAMPS)
R.update({
    "paint": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],          # machine yellow-orange
    "brick": ["#4a051e", "#841c1d", "#d4462b", "#fe6420"],
    "stone": ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "conc": ["#434137", "#a09f91", "#bfcad7", "#fffefe"],
    "white": ["#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "copper": ["#561c27", "#8d4526", "#a8633e", "#e69d69", "#eab798"],
    "iron": ["#2c2c25", "#434137", "#a09f91", "#bfcad7", "#fffefe"],
    "water": ["#191954", "#15578a", "#1096c3", "#74effe"],
    "murky": ["#003227", "#204f63", "#53998b", "#84e3b5"],
    "ether": ["#230c44", "#4f1d83", "#7d2dbd", "#c34988", "#dd576d"],
    "etherglow": ["#4f1d83", "#7d2dbd", "#c34988", "#fa3a64", WHITE],
    "uran": ["#003227", "#05582d", "#559642", "#9bb675", "#c1f1d8"],
    "steam": ["#92a3b2", "#bfcad7", WHITE],
    "smoke": ["#2c2c25", "#434137", "#a09f91"],
})


def darker(ramp):
    return [ramp[0]] + ramp[:-1]


def spark(sp, x, y, c="#faaf5c"):
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], c, glow=True)
    sp.plot(x, y, WHITE, glow=True)


def hexagon(cx, cy, r, squash=1.0):
    return [(cx + r * math.cos(k * math.pi / 3), cy + r * squash * math.sin(k * math.pi / 3)) for k in range(6)]


# ---------------------------------------------------------------- machines

def crusher():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(3, 3), (28, 3), (22, 12), (9, 12)]), R["iron"], "rim")       # the hopper
    sp.part(sp.rect(5, 12, 26, 26), R["paint"], "rim")                              # the body
    for x in (10, 15, 20):                                                          # the jaws' teeth
        sp.part(sp.poly([(x, 12), (x + 4, 12), (x + 2, 17)]), R["iron"], "flat", base=2)
    sp.part(sp.circle(9.5, 21, 3.5), R["iron"], "sphere")                           # flywheel
    sp.part(sp.rect(4, 26, 27, 29), R["soot"], "rim")                               # the skid
    for x in range(6, 26, 4):
        sp.fill(sp.rect(x, 24, x + 1, 25), "#161612")                               # hazard band
    sp.plot(6, 4, WHITE)
    sp.outline()
    return sp.image()


def smelter():
    sp = Sprite(32, 32)
    sp.part(sp.rect(20, 1, 25, 12), R["brick"], "cyl_v")                           # chimney
    sp.part(sp.poly([(4, 30), (4, 12), (9, 7), (22, 7), (27, 12), (27, 30)]), R["brick"], "rim")
    for k, y in enumerate(range(11, 30, 4)):                                         # mortar
        sp.fill(sp.rect(5, y, 26, y), "#4a051e")
    sp.part(sp.rect(10, 17, 21, 27), R["soot"], "flat", base=0)                     # the mouth
    sp.outline()
    sp.part(sp.rect(11, 20, 20, 27), R["flame"], "inset", glow=True)                # the melt
    sp.part(sp.rect(12, 28, 19, 29), ["#a61e1f", "#fe6420", "#faaf5c"], "flat", base=2, glow=True)
    spark(sp, 22, 3, "#a09f91")
    return sp.image()


def refinery():
    sp = Sprite(32, 32)
    sp.part(sp.rect(17, 2, 23, 29), R["white"], "cyl_v")                           # the column
    for y in (8, 14, 20):
        sp.part(sp.rect(16, y, 24, y + 1), R["metal"], "flat", base=2)
    sp.part(sp.ellipse(2, 13, 15, 29), R["metal"], "cyl_v")                         # the tank
    sp.part(sp.rect(4, 19, 13, 21), R["paint"], "flat", base=2)                     # stencil band
    sp.part(sp.rect(13, 23, 18, 25), R["iron"], "rim")                              # pipe between
    sp.part(sp.rect(24, 24, 29, 26), R["iron"], "rim")
    sp.plot(18, 3, WHITE)
    sp.plot(5, 15, WHITE)
    sp.outline()
    sp.part(sp.rect(19, 0, 21, 1), ["#a61e1f", "#fe6420", "#faaf5c"], "flat", base=2, glow=True)   # flare
    return sp.image()


def press():
    sp = Sprite(32, 32)
    sp.part(sp.rect(3, 3, 7, 28) | sp.rect(24, 3, 28, 28), R["paint"], "rim")      # the frame
    sp.part(sp.rect(3, 2, 28, 7), R["paint"], "rim")
    sp.part(sp.rect(9, 8, 22, 13), R["iron"], "cyl_h")                              # upper roll
    sp.part(sp.rect(9, 17, 22, 22), R["iron"], "cyl_h")                             # lower roll
    sp.part(sp.rect(1, 14, 30, 16), R["copper"], "rim", base=3)                     # the plate coming through
    sp.part(sp.rect(2, 28, 29, 30), R["soot"], "rim")
    sp.plot(10, 9, WHITE)
    sp.outline()
    return sp.image()


def siphon():
    sp = Sprite(32, 32)
    sp.part(sp.rect(0, 22, 31, 31), R["water"], "flat", base=1)                     # the lake
    for x in range(1, 31, 6):
        sp.fill(sp.rect(x, 24, x + 2, 24), "#74effe")
    sp.part(sp.rect(4, 6, 8, 27), R["iron"], "cyl_v")                               # the intake pipe
    sp.part(sp.rect(4, 6, 22, 10), R["iron"], "cyl_h")
    sp.part(sp.rect(16, 10, 29, 22), R["paint"], "rim")                             # pump house
    sp.part(sp.circle(22.5, 16, 3.5), R["iron"], "sphere")                          # the impeller
    sp.plot(5, 7, WHITE)
    sp.outline()
    return sp.image()


def filter_bed():
    sp = Sprite(32, 32)
    sp.part(sp.rect(3, 4, 28, 29), R["conc"], "rim")                                # the box
    sp.part(sp.rect(6, 7, 25, 11), R["murky"], "flat", base=2)                      # raw water on top
    sp.part(sp.rect(6, 12, 25, 16), R["sand"], "flat", base=2)                      # sand
    sp.dither(sp.rect(6, 17, 25, 21), "#725244", "#a09f91", 0.5)                    # gravel
    sp.part(sp.rect(6, 22, 25, 26), R["soot"], "flat", base=1)                      # charcoal
    sp.part(sp.rect(26, 23, 30, 26), R["water"], "rim", base=3)                     # clean water out
    sp.plot(4, 5, WHITE)
    sp.outline()
    return sp.image()


def tank():
    sp = Sprite(32, 32)
    sp.part(sp.rect(5, 6, 26, 28), R["white"], "cyl_v")
    sp.part(sp.ellipse(5, 2, 26, 9), R["white"], "sphere")
    sp.part(sp.rect(8, 12, 10, 26), R["water"], "flat", base=3)                     # the sight glass
    sp.part(sp.rect(8, 12, 10, 17), R["metal"], "flat", base=1)
    for y in (11, 18, 25):
        sp.part(sp.rect(5, y, 26, y), R["metal"], "flat", base=2, seam=False)
    sp.part(sp.rect(3, 28, 28, 30), R["soot"], "rim")
    sp.plot(9, 4, WHITE)
    sp.outline()
    return sp.image()


def extractor():
    sp = Sprite(32, 32)
    sp.part(sp.poly(hexagon(15.5, 27, 13, 0.3)) | sp.rect(3, 27, 28, 30), R["night"], "rim")   # the plinth
    sp.part(sp.rect(13, 6, 18, 27), R["metal"], "cyl_v")                            # the mast
    for y, w in ((9, 9), (16, 7), (22, 5)):                                         # the rings
        sp.part(sp.ellipse(15.5 - w, y - 1, 15.5 + w, y + 2), R["iron"], "rim")
    sp.outline()
    sp.part(sp.star(15.5, 4, 4, 1.6, 4), R["etherglow"], "inset", glow=True)        # the tap
    sp.part(sp.ellipse(11, 25, 20, 28), R["etherglow"], "inset", glow=True)
    spark(sp, 4, 10, "#c34988")
    spark(sp, 27, 15, "#c34988")
    spark(sp, 7, 20, "#dd576d")
    return sp.image()


# ---------------------------------------------------------------- generators

def coal_plant():
    sp = Sprite(32, 32)
    sp.part(sp.rect(20, 2, 25, 18), R["brick"], "cyl_v")                           # the stack
    sp.part(sp.rect(2, 14, 29, 29), R["conc"], "rim")                               # the hall
    sp.part(sp.poly([(3, 14), (8, 9), (14, 14)]), R["iron"], "rim")                 # the coal hopper
    sp.dither(sp.rect(5, 11, 11, 13), "#161612", "#2c2c25", 0.5)
    for x in (16, 21, 26):
        sp.part(sp.rect(x - 1, 19, x + 1, 23), R["soot"], "flat", base=0)
    sp.outline()
    for x in (16, 21, 26):
        sp.part(sp.rect(x - 1, 20, x + 1, 22), ["#c56d25", "#fb902f", "#faaf5c"], "flat", base=1, glow=True)
    sp.part(sp.circle(24, 1, 1.5), R["smoke"], "flat", base=2, glow=True)
    return sp.image()


def wind():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(14, 30), (15, 12), (17, 12), (18, 30)]), R["white"], "rim")   # the tower
    sp.part(sp.rect(13, 9, 19, 13), R["white"], "rim")                              # nacelle
    for a in (-90, 30, 150):                                                        # three blades
        t = math.radians(a)
        sp.part(sp.taper(16, 11, 16 + 14 * math.cos(t), 11 + 14 * math.sin(t), 3, 1), R["white"], "rim")
    sp.part(sp.circle(16, 11, 2), R["paint"], "sphere")
    sp.part(sp.rect(10, 29, 22, 31), R["soot"], "rim")
    sp.plot(15, 10, WHITE)
    sp.outline()
    return sp.image()


def geothermal():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(0, 31), (0, 24), (8, 21), (24, 21), (31, 24), (31, 31)]), R["soot"], "rim")   # basalt
    sp.part(sp.rect(4, 13, 15, 24), R["conc"], "rim")                               # turbine house
    sp.part(sp.rect(15, 16, 29, 18), R["iron"], "cyl_h")                            # the bore pipe
    sp.part(sp.rect(24, 16, 27, 25), R["iron"], "cyl_v")
    sp.outline()
    sp.part(sp.ellipse(20, 24, 30, 28), R["flame"], "inset", glow=True)             # the hot vent
    for x, y, r in ((22, 9, 3), (25, 4, 2.5), (9, 7, 3), (12, 2, 2)):                # steam
        sp.part(sp.circle(x, y, r), R["steam"], "sphere", glow=True)
    return sp.image()


def hydro():
    sp = Sprite(32, 32)
    sp.part(sp.rect(0, 6, 31, 12), R["water"], "flat", base=2)                      # the high water
    sp.part(sp.poly([(4, 12), (27, 12), (29, 28), (2, 28)]), R["conc"], "rim")      # the dam
    sp.part(sp.rect(0, 26, 31, 31), R["water"], "flat", base=1)                     # tail water
    for x in (9, 15, 21):
        sp.part(sp.rect(x, 18, x + 2, 26), R["metal"], "flat", base=1)              # the penstocks
    sp.part(sp.rect(2, 10, 29, 12), R["iron"], "rim")
    sp.plot(5, 13, WHITE)
    sp.outline()
    for x in (9, 15, 21):
        sp.part(sp.rect(x, 26, x + 2, 28), ["#1096c3", "#74effe", "#d2f8ff"], "flat", base=2, glow=True)
    return sp.image()


def nuclear():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(17, 30), (19, 16), (17, 3), (30, 3), (28, 16), (30, 30)]), R["conc"], "rim")   # cooling tower
    sp.part(sp.ellipse(1, 12, 18, 26), R["white"], "sphere")                        # the dome
    sp.part(sp.rect(1, 20, 18, 30), R["white"], "rim")
    sp.part(sp.rect(7, 24, 12, 30), R["metal"], "flat", base=1)
    sp.plot(6, 14, WHITE)
    sp.outline()
    for x, y, r in ((22, 1, 2.5), (26, 1, 2)):
        sp.part(sp.circle(x, y, r), R["steam"], "sphere", glow=True)
    sp.part(sp.star(9.5, 17, 3, 1, 3), R["uran"], "flat", base=3, glow=True)       # the hazard trefoil
    return sp.image()


def pole():
    sp = Sprite(32, 32)
    sp.part(sp.rect(14, 3, 17, 30), R["wood"], "cyl_v")
    sp.part(sp.rect(5, 6, 26, 8), R["wood"], "rim")                                 # cross arm
    for x in (6, 15, 24):
        sp.part(sp.rect(x, 3, x + 1, 5), R["glass"], "flat", base=3)                # insulators
    sp.part(sp.line([(0, 4), (6, 5)], 1) | sp.line([(25, 5), (31, 4)], 1), R["soot"], "flat", base=1, seam=False)
    sp.plot(14, 4, WHITE)
    sp.outline()
    return sp.image()


# ---------------------------------------------------------------- items

def crushed(ramp, name):
    def f():
        sp = Sprite(32, 32)
        for (x, y, r) in ((9, 20, 5), (19, 22, 5.5), (14, 13, 5), (24, 15, 4), (7, 11, 3.5), (25, 24, 3.5)):
            sp.part(sp.circle(x, y, r), R["stone"], "sphere")
            sp.part(sp.rect(int(x - 2), int(y - 1), int(x), int(y + 1)), ramp, "rim", base=len(ramp) - 2)
        sp.plot(12, 10, WHITE)
        sp.outline()
        return sp.image()
    f.__name__ = name
    return f


def plate(ramp, name):
    def f():
        sp = Sprite(32, 32)
        for k in range(3):
            y = 20 - k * 5
            sp.part(sp.poly([(3, y + 4), (19, y + 8), (29, y + 2), (13, y - 2)]), ramp if k == 2 else darker(ramp), "rim")
            sp.part(sp.poly([(3, y + 4), (19, y + 8), (19, y + 10), (3, y + 6)]), darker(darker(ramp)), "flat", base=1)
        sp.plot(14, 9, WHITE)
        sp.plot(15, 9, WHITE)
        sp.outline()
        return sp.image()
    f.__name__ = name
    return f


def canister(ramp, name, label):
    def f():
        sp = Sprite(32, 32)
        sp.part(sp.rect(12, 2, 19, 5), R["iron"], "rim")                            # cap
        sp.part(sp.rect(7, 6, 24, 29), R["glass"], "cyl_v", base=4)                 # glass
        sp.part(sp.rect(8, 11, 23, 28), ramp, "cyl_v")                              # what is in it
        sp.part(sp.rect(7, 17, 24, 20), label, "flat", base=2)
        sp.plot(9, 8, WHITE)
        sp.plot(9, 9, WHITE)
        sp.outline()
        return sp.image()
    f.__name__ = name
    return f


def ether_shard():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(16, 2), (24, 12), (19, 29), (11, 29), (7, 12)]), R["ether"], "inset")
    sp.part(sp.poly([(16, 2), (16, 29), (11, 29), (7, 12)]), R["etherglow"], "rim", base=2)
    sp.plots([(13, 8), (12, 10)], WHITE)
    sp.outline()
    spark(sp, 26, 5, "#c34988")
    spark(sp, 5, 24, "#dd576d")
    return sp.image()


def fuel_rod():
    sp = Sprite(32, 32)
    for x in (6, 13, 20):
        sp.part(sp.rect(x, 4, x + 5, 27), R["iron"], "cyl_v")
        sp.part(sp.rect(x + 1, 9, x + 4, 22), R["uran"], "cyl_v", base=2)
    sp.part(sp.rect(4, 26, 27, 29), R["metal"], "rim")
    sp.part(sp.rect(4, 2, 27, 5), R["metal"], "rim")
    sp.plot(7, 6, WHITE)
    sp.outline()
    return sp.image()


def wire():
    sp = Sprite(32, 32)
    sp.part(sp.circle(15.5, 15.5, 13), R["wood"], "sphere")                         # spool flange
    sp.part(sp.circle(15.5, 15.5, 10), R["copper"], "rim", base=3)                  # the wound wire
    for r in (7.5, 5):
        sp.fill(sp.circle(15.5, 15.5, r) & ~sp.circle(15.5, 15.5, r - 1), "#8d4526")
    sp.part(sp.circle(15.5, 15.5, 3), R["wood"], "rim", base=0)
    sp.part(sp.line([(26, 20), (31, 28)], 2), R["copper"], "flat", base=3, seam=False)
    sp.plot(10, 9, WHITE)
    sp.outline()
    return sp.image()


def marker():
    """The survey flag: what the player plants to mark cells for the digger."""
    sp = Sprite(32, 32)
    sp.part(sp.poly(hexagon(15.5, 26, 12, 0.35)), R["cyan"], "flat", base=1)       # the marked cell
    sp.part(sp.rect(9, 3, 11, 27), R["wood"], "cyl_v")
    sp.part(sp.poly([(12, 4), (27, 8), (12, 13)]), R["cyan"], "rim")
    sp.plot(10, 4, WHITE)
    sp.outline()
    return sp.image()


# ---------------------------------------------------------------- textures (32 x 32, wrapping)

def tile(fn):
    a = np.zeros((32, 32, 4), np.uint8)
    for y in range(32):
        for x in range(32):
            a[y, x, :3] = rgb(fn(x, y))
            a[y, x, 3] = 255
    return Image.fromarray(a, "RGBA")


def h(x, y, s=0):
    v = (x * 374761393 + y * 668265263 + s * 2246822519) & 0xFFFFFFFF
    v = ((v ^ (v >> 13)) * 1274126177) & 0xFFFFFFFF
    return ((v ^ (v >> 16)) & 0xFFFF) / 65536


def tex_paint():
    """Machine paint: 16 px panels of chipped orange-yellow, a dark seam, rivets, worn edges."""
    seam, shadow, body, lit, chip = "#733126", "#c56d25", "#fb902f", "#faaf5c", "#434137"

    def px(x, y):
        u, v = x % 16, y % 16
        if u == 0 or v == 0:
            return seam
        if (u, v) in ((2, 2), (13, 2), (2, 13), (13, 13)):
            return "#2c2c25"
        if u == 15 or v == 15:
            return shadow
        if u == 1 or v == 1:
            return lit
        if h(x, y, 1) < 0.035:
            return chip
        if h(x // 2, y // 2, 2) < 0.06:
            return shadow
        return body
    return tile(px)


def tex_belt():
    """Conveyor belt: dark rubber with a cleat every 8 px across u (the belt runs along u)."""
    def px(x, y):
        if x % 8 == 0:
            return "#434137"
        if x % 8 == 1:
            return "#161612"
        if y in (0, 31):
            return "#010000"
        return "#2c2c25" if h(x, y, 3) > 0.12 else "#161612"
    return tile(px)


def tex_concrete():
    """Concrete: pale grey, form-board lines every 16 px, tie holes, dithered stain."""
    def px(x, y):
        if y % 16 == 0:
            return "#a09f91"
        if (x % 16, y % 16) in ((4, 8), (12, 8)):
            return "#434137"
        n = h(x, y, 4)
        if n < 0.08:
            return "#a09f91"
        if n > 0.93:
            return "#fffefe"
        return "#bfcad7" if (x + y) % 2 == 0 or n > 0.3 else "#a09f91"
    return tile(px)


def tex_hazard():
    def px(x, y):
        return "#fb902f" if ((x + y) // 8) % 2 == 0 else "#161612"
    return tile(px)


def tex_white():
    """White painted cladding (turbines, the reactor dome, the cooling tower): 8 px courses."""
    def px(x, y):
        if y % 8 == 0:
            return "#92a3b2"
        if y % 8 == 1:
            return "#fffefe"
        return "#bfcad7" if h(x, y, 5) > 0.06 else "#92a3b2"
    return tile(px)


def tex_grate():
    """A furnace grate: iron bars over the glow. Shown self-lit at night."""
    def px(x, y):
        if x % 8 in (0, 1):
            return "#2c2c25"
        if y % 16 == 0:
            return "#434137"
        n = h(x, y, 6)
        return "#faaf5c" if n > 0.8 else "#fb902f" if n > 0.35 else "#fe6420"
    return tile(px)


def tex_ether():
    """The ether field's cells: violet crystal facets, hard edges, a few white glints."""
    def px(x, y):
        u, v = x % 16, y % 16
        d = (u + 2 * v) % 16
        if u == 0 or v == 0:
            return "#230c44"
        if h(x, y, 7) > 0.97:
            return "#fffefe"
        if d < 4:
            return "#c34988"
        if d < 9:
            return "#7d2dbd"
        return "#4f1d83"
    return tile(px)


# ---------------------------------------------------------------- the late tiers: fusion, antimatter, the swarm

def fusion():
    sp = Sprite(32, 32)
    sp.part(sp.rect(1, 20, 30, 30), R["conc"], "rim")                               # the hall's base
    ring = sp.ellipse(3, 8, 28, 26) & ~sp.ellipse(10, 13, 21, 21)
    sp.part(ring, R["metal"], "rim", base=3)                                        # the torus
    for x in (5, 10, 16, 21, 26):                                                   # field coils
        sp.part(sp.rect(x - 1, 9 if x in (10, 16, 21) else 12, x + 1, 25 if x in (10, 16, 21) else 22) & ring, R["paint"], "flat", base=2)
    sp.plot(8, 10, WHITE)
    sp.outline()
    sp.part(sp.ellipse(7, 11, 24, 23) & ~sp.ellipse(9, 13, 22, 21), R["etherglow"], "flat", base=3, glow=True)   # the plasma
    return sp.image()


def collider():
    sp = Sprite(32, 32)
    sp.part(sp.ellipse(1, 7, 30, 27) & ~sp.ellipse(5, 11, 26, 23), R["conc"], "rim")  # the ring
    sp.part(sp.rect(19, 19, 29, 29), R["white"], "rim")                             # the detector hall
    sp.part(sp.rect(21, 22, 27, 26), R["metal"], "flat", base=1)
    sp.plot(4, 12, WHITE)
    sp.outline()
    sp.part(sp.ellipse(3, 9, 28, 25) & ~sp.ellipse(4, 10, 27, 24), R["cyan"], "flat", base=3, glow=True)   # the beam
    spark(sp, 6, 13, "#74effe")
    spark(sp, 15, 8, "#74effe")
    return sp.image()


def antimatter():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(0, 31), (3, 12), (28, 12), (31, 31)]), R["soot"], "rim")    # shielding berm
    for x in (7, 16, 25):                                                           # containment vessels
        sp.part(sp.rect(x - 3, 5, x + 3, 27), R["white"], "cyl_v")
        for y in (9, 15, 21):
            sp.part(sp.rect(x - 4, y, x + 4, y + 1), R["metal"], "flat", base=1)
    sp.plot(5, 6, WHITE)
    sp.outline()
    for x in (7, 16, 25):
        sp.part(sp.rect(x - 1, 11, x, 13) | sp.rect(x - 1, 17, x, 19), R["etherglow"], "flat", base=3, glow=True)
    return sp.image()


def am_cell():
    sp = Sprite(32, 32)
    sp.part(sp.rect(8, 2, 23, 6) | sp.rect(8, 26, 23, 30), R["iron"], "rim")        # end caps
    sp.part(sp.rect(10, 6, 21, 26), R["glass"], "cyl_v", base=4)
    for y in (10, 16, 22):
        sp.part(sp.rect(9, y, 22, y + 1), R["paint"], "flat", base=2)               # the trap's coils
    sp.plot(11, 7, WHITE)
    sp.outline()
    sp.part(sp.circle(15.5, 16, 2.5), R["etherglow"], "inset", glow=True)           # the held speck
    spark(sp, 26, 8, "#c34988")
    return sp.image()


def drone():
    sp = Sprite(32, 32)
    for x0 in (1, 19):                                                              # two collector wings
        sp.part(sp.rect(x0, 9, x0 + 11, 22), R["blue"], "rim")
        for x in range(x0 + 3, x0 + 11, 4):
            sp.fill(sp.rect(x, 10, x, 21), "#191954")
        sp.fill(sp.rect(x0 + 1, 15, x0 + 10, 15), "#191954")
    sp.part(sp.rect(12, 11, 19, 20), R["gold"], "rim")                              # the core
    sp.part(sp.rect(14, 5, 17, 11), R["iron"], "flat", base=2)                      # the emitter
    sp.plot(2, 10, WHITE)
    sp.outline()
    sp.part(sp.rect(15, 2, 16, 4), ["#c56d25", "#fb902f", "#faaf5c"], "flat", base=2, glow=True)
    return sp.image()


def dyson():
    """A relay station: a ring, a dish aimed down, panels."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(0, 12, 8, 19) | sp.rect(23, 12, 31, 19), R["blue"], "rim")    # panels
    sp.part(sp.circle(15.5, 15.5, 8), R["white"], "sphere")                         # the hub
    sp.part(sp.poly([(9, 22), (22, 22), (19, 28), (12, 28)]), R["iron"], "rim")     # the dish
    sp.plot(12, 11, WHITE)
    sp.outline()
    sp.part(sp.rect(14, 28, 17, 31), R["flame"], "inset", glow=True)                # the beam
    spark(sp, 27, 5)
    return sp.image()


def tex_plasma():
    def px(x, y):
        n = h(x // 2, y // 2, 8)
        if (x + 3 * y) % 11 == 0:
            return "#fffefe"
        return "#fa3a64" if n > 0.7 else "#dd576d" if n > 0.35 else "#c34988"
    return tile(px)


def tex_panel():
    """Collector panel: deep blue cells in a gold frame, 8 px."""
    def px(x, y):
        if x % 8 == 0 or y % 8 == 0:
            return "#c56d25"
        if (x % 8, y % 8) in ((2, 2), (3, 2)):
            return "#74effe"
        return "#15578a" if (x // 8 + y // 8) % 2 else "#191954"
    return tile(px)


# ---------------------------------------------------------------- oil (steer-and-sail design 5, M20)

R.update({
    "crude": ["#010000", "#161612", "#2b1f35", "#3e3551", "#69608b"],   # black with a violet sheen
    "redpaint": ["#4a051e", "#a61e1f", "#d12d41", "#fa3a64"],
    "tar": ["#010000", "#161612", "#2c2c25", "#434137"],
})


def pumpjack():
    """The nodding donkey: skid, A-frame samson post, walking beam, horsehead over the well, crank."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(1, 27, 30, 30), R["soot"], "rim")                                # the skid
    sp.part(sp.rect(25, 23, 30, 27), R["iron"], "rim")                               # wellhead
    sp.part(sp.rect(3, 21, 11, 27), R["paint"], "rim")                               # gearbox
    sp.part(sp.line([(9, 27), (15, 11)], 2) | sp.line([(21, 27), (16, 11)], 2), R["iron"], "rim")   # samson post
    sp.part(sp.circle(7, 19, 4.5), R["soot"], "sphere")                              # the crank's weight
    sp.part(sp.line([(7, 18), (5, 10)], 2), R["iron"], "flat", base=2)               # pitman arm
    sp.part(sp.poly([(2, 7), (26, 9), (26, 12), (2, 10)]), R["paint"], "rim")        # walking beam
    sp.part(sp.poly([(24, 6), (29, 7), (31, 11), (30, 18), (27, 18), (27, 12), (24, 11)]), R["paint"], "rim")   # horsehead
    sp.part(sp.rect(28, 18, 28, 23), R["iron"], "flat", base=1, seam=False)          # polished rod
    sp.plot(16, 11, "#bfcad7")
    sp.plot(4, 8, WHITE)
    sp.plot(5, 17, WHITE)
    sp.outline()
    return sp.image()


def oil_refinery():
    """The distillation column with its trays, a product tank beside it and the flare stack."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(3, 6, 6, 29), R["redpaint"], "cyl_v")                           # flare stack
    sp.part(sp.rect(20, 17, 30, 29), R["metal"], "cyl_v")                           # product tank
    sp.part(sp.ellipse(20, 15, 30, 19), R["metal"], "sphere")
    sp.part(sp.rect(23, 21, 27, 22), R["amber"], "flat", base=2)
    sp.part(sp.rect(10, 4, 19, 29), R["white"], "cyl_v")                            # the column
    sp.part(sp.ellipse(10, 2, 19, 6), R["white"], "sphere")
    for y in (10, 16, 22):                                                           # trays and draw-offs
        sp.part(sp.rect(9, y, 20, y + 1), R["iron"], "flat", base=2)
    sp.part(sp.rect(19, 12, 22, 13) | sp.rect(19, 24, 21, 25), R["iron"], "rim")
    sp.part(sp.rect(1, 28, 31, 30), R["soot"], "rim")
    sp.plot(12, 5, WHITE)
    sp.plot(22, 18, WHITE)
    sp.outline()
    sp.part(sp.taper(4.5, 5, 4.5, 0, 3, 1), R["flame"], "inset", glow=True)          # the flare
    return sp.image()


def oil_plant():
    """A boiler and turbine hall on fuel oil: a steel stack, a red-banded fuel oil tank, lit windows."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(16, 3, 20, 16), R["steel"], "cyl_v")                            # the stack
    sp.part(sp.rect(15, 6, 21, 7), R["redpaint"], "flat", base=2)
    sp.part(sp.rect(1, 14, 22, 29), R["conc"], "rim")                               # the hall
    sp.part(sp.poly([(1, 14), (6, 10), (12, 14)]), R["iron"], "rim")                # roof vent
    sp.part(sp.rect(21, 18, 31, 29), R["white"], "cyl_v")                           # fuel oil tank
    sp.part(sp.ellipse(21, 16, 31, 20), R["white"], "sphere")
    sp.part(sp.rect(21, 23, 31, 25), R["redpaint"], "flat", base=2)
    for x in (4, 9, 14):
        sp.part(sp.rect(x, 19, x + 2, 23), R["soot"], "flat", base=0)
    sp.plot(24, 17, WHITE)
    sp.outline()
    for x in (4, 9, 14):
        sp.part(sp.rect(x, 20, x + 2, 22), ["#c56d25", "#fb902f", "#faaf5c"], "flat", base=1, glow=True)
    sp.part(sp.circle(18, 1, 1.5), R["tar"], "flat", base=2, glow=True)            # black smoke
    return sp.image()


def crude():
    """Crude oil: a fat black drop with a violet sheen."""
    sp = Sprite(32, 32)
    sp.part(sp.circle(15.5, 20, 9.5) | sp.poly([(15.5, 2), (7, 17), (24, 17)]), R["crude"], "sphere")
    sp.part(sp.ellipse(9, 13, 13, 22), ["#3e3551", "#69608b", "#9e3ba2"], "flat", base=1, seam=False)
    sp.plots([(11, 13), (11, 14)], WHITE)
    sp.outline()
    return sp.image()


def drum(ramp, label, name):
    def f():
        sp = Sprite(32, 32)
        sp.part(sp.rect(6, 5, 25, 29), ramp, "cyl_v")                               # the drum
        sp.part(sp.ellipse(6, 2, 25, 7), R["steel"], "sphere")                      # its lid
        for y in (11, 22):
            sp.part(sp.rect(5, y, 26, y + 1), darker(ramp), "flat", base=1)         # rolling hoops
        sp.part(sp.rect(10, 14, 21, 19), R["paper"], "rim")                         # the label
        sp.part(sp.rect(12, 16, 19, 17), label, "flat", base=len(label) - 2, seam=False)
        sp.plot(12, 3, WHITE)
        sp.plot(8, 7, WHITE)
        sp.outline()
        return sp.image()
    f.__name__ = name
    return f


def asphalt():
    """Asphalt, the column's residue: a slab of road with its yellow line."""
    sp = Sprite(32, 32)
    top = sp.poly([(2, 13), (16, 6), (30, 13), (16, 20)])
    sp.part(sp.poly([(2, 13), (16, 20), (16, 27), (2, 20)]), R["tar"], "flat", base=1)
    sp.part(sp.poly([(16, 20), (30, 13), (30, 20), (16, 27)]), R["tar"], "flat", base=0)
    sp.part(top, R["tar"], "rim", base=2)
    sp.plots([(x, y) for y in range(8, 20) for x in range(4, 29) if top[y, x] and h(x, y, 11) > 0.9], "#a09f91")
    sp.part(sp.line([(8, 11), (13, 13)], 2) | sp.line([(18, 15), (23, 17)], 2), R["amber"], "flat", base=3, seam=False)
    sp.plot(15, 8, WHITE)
    sp.outline()
    return sp.image()


def tex_asphalt():
    """The refinery yard: dark tar with grey and pale gravel specks."""
    def px(x, y):
        n = h(x, y, 9)
        if n > 0.96:
            return "#a09f91"
        if n > 0.78:
            return "#434137"
        if n < 0.12:
            return "#161612"
        return "#2c2c25"
    return tile(px)


def tex_flare():
    """A flare stack's red and white bands, 8 px, a dark ring between."""
    def px(x, y):
        v = y % 16
        if v in (0, 8):
            return "#2c2c25"
        red = v < 8
        lit = x % 32 < 10
        if red:
            return "#d12d41" if lit else "#a61e1f"
        return "#fffefe" if lit else "#bfcad7"
    return tile(px)


def tex_window():
    """A lit window: warm panes in a dark frame, four panes to a tile. Self-lit at night."""
    def px(x, y):
        u, v = x % 16, y % 16
        if u in (0, 15) or v in (0, 15) or u == 7 or v == 7:
            return "#2c2c25"
        if (u, v) in ((2, 2), (3, 2), (2, 3), (9, 2), (10, 2), (9, 3)):
            return "#fffefe"
        return "#faaf5c" if v < 7 else "#fb902f" if (u + v) % 2 else "#faaf5c"
    return tile(px)


ICONS = {
    "crusher": crusher, "smelter": smelter, "refinery": refinery, "press": press,
    "siphon": siphon, "filter": filter_bed, "tank": tank, "extractor": extractor,
    "coal-plant": coal_plant, "wind": wind, "geothermal": geothermal, "hydro": hydro, "nuclear": nuclear, "pole": pole,
    "crushed-copper": crushed(R["copper"], "crushed_copper"), "crushed-iron": crushed(R["rust"], "crushed_iron"),
    "copper-plate": plate(R["copper"], "copper_plate"), "iron-plate": plate(R["iron"], "iron_plate"),
    "raw-water": canister(R["murky"], "raw_water", R["sand"]), "clean-water": canister(R["water"], "clean_water", R["paper"]),
    "ether-shard": ether_shard, "fuel-rod": fuel_rod, "wire": wire, "marker": marker,
    "tex_paint": tex_paint, "tex_belt": tex_belt, "tex_concrete": tex_concrete, "tex_hazard": tex_hazard,
    "tex_white": tex_white, "tex_grate": tex_grate, "tex_ether": tex_ether,
    "fusion": fusion, "collider": collider, "antimatter": antimatter, "am-cell": am_cell, "drone": drone, "dyson": dyson,
    "tex_plasma": tex_plasma, "tex_panel": tex_panel,
    "pumpjack": pumpjack, "oil-refinery": oil_refinery, "oil-plant": oil_plant, "crude": crude,
    "fuel-oil": drum(R["redpaint"], R["soot"], "fuel_oil"), "diesel": drum(R["gold"], R["olive"], "diesel"), "asphalt": asphalt,
    "tex_asphalt": tex_asphalt, "tex_flare": tex_flare, "tex_window": tex_window,
}
REFS = ["furnace_barge", "armour_plating", "emp_burst", "drop_tanks", "glass_seraph"]


# ---- 2026-11-14: mining tiers, the rig and the platform, freighters, the four carriers (industry-tiers)
def worker():
    """A hired townsfolk miner: hard hat, a pick over the shoulder, a barrow beside him."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(18, 22, 30, 26), R["iron"], "rim")                               # the barrow's tub
    sp.part(sp.circle(21, 28, 2.5), R["soot"], "sphere")                             # its wheel
    sp.part(sp.rect(7, 14, 14, 24), R["blue"], "rim")                                # coat
    sp.part(sp.rect(8, 24, 10, 30) | sp.rect(11, 24, 13, 30), R["wood"], "flat", base=1)   # legs
    sp.part(sp.circle(10.5, 10, 3.5), R["sand"], "sphere")                           # face
    sp.part(sp.rect(6, 5, 15, 8), R["gold"], "rim")                                  # hard hat
    sp.part(sp.line([(4, 22), (17, 6)], 2), R["wood"], "flat", base=2)               # pick handle
    sp.part(sp.poly([(13, 3), (21, 6), (20, 8), (14, 6)]), R["iron"], "rim")        # pick head
    sp.plot(7, 6, WHITE)
    sp.outline()
    return sp.image()


def excavator():
    """A tracked excavator: tracks, cab, boom and bucket in machine paint."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(2, 24, 20, 29), R["soot"], "rim")                                # tracks
    sp.part(sp.rect(4, 15, 17, 23), R["paint"], "rim")                               # house
    sp.part(sp.rect(11, 9, 17, 16), R["paint"], "rim")                               # cab
    sp.fill(sp.rect(13, 11, 16, 14), "#74effe")                                      # cab glass
    sp.part(sp.line([(16, 16), (25, 6)], 3), R["paint"], "flat", base=2)            # boom
    sp.part(sp.line([(25, 6), (28, 18)], 2), R["paint"], "flat", base=2)            # stick
    sp.part(sp.poly([(24, 18), (31, 18), (30, 24), (25, 23)]), R["iron"], "rim")    # bucket
    sp.plot(5, 16, WHITE)
    sp.outline()
    return sp.image()


def quarry():
    """The quarry frame: an orange gantry over a square hole, the drill head hanging from its cross rail."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(5, 18, 27, 29), R["soot"], "inset")                              # the hole
    for x in (3, 26):
        sp.part(sp.rect(x, 5, x + 2, 28), R["paint"], "flat", base=2)                # legs
    sp.part(sp.rect(3, 4, 28, 6), R["paint"], "rim")                                 # top rail
    sp.part(sp.rect(14, 6, 17, 8), R["iron"], "rim")                                 # carriage
    sp.part(sp.rect(15, 8, 16, 20), R["cyan"], "flat", base=2)                       # the drill shaft
    sp.part(sp.poly([(13, 20), (18, 20), (15.5, 25)]), R["iron"], "rim")            # bit
    sp.fill(sp.rect(4, 5, 4, 5), WHITE)
    sp.outline()
    return sp.image()


def bucketwheel():
    """The bucket-wheel excavator: crawlers, a tall mast with cables, a long boom, the toothed wheel."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(1, 26, 16, 30), R["soot"], "rim")                                # crawlers
    sp.part(sp.rect(3, 20, 14, 26), R["metal"], "rim")                               # house
    sp.part(sp.rect(7, 4, 9, 20), R["metal"], "flat", base=2)                        # mast
    sp.part(sp.line([(9, 5), (25, 14)], 1) | sp.line([(8, 5), (2, 16)], 1), R["steel"], "flat", base=1, seam=False)   # cables
    sp.part(sp.line([(10, 20), (24, 16)], 2), R["metal"], "flat", base=2)           # boom
    sp.part(sp.line([(1, 18), (5, 18)], 2), R["metal"], "flat", base=1)             # counterweight arm
    sp.part(sp.circle(25, 18, 6), R["rust"], "sphere")                               # the wheel
    for a in range(0, 360, 45):
        x, y = 25 + 6.5 * math.cos(math.radians(a)), 18 + 6.5 * math.sin(math.radians(a))
        sp.plot(int(round(x)), int(round(y)), "#2c2c25")                            # buckets' teeth
    sp.fill(sp.circle(25, 18, 2), "#2c2c25")
    sp.plot(4, 21, WHITE)
    sp.outline()
    return sp.image()


def hive():
    """The drone hive: a hex hangar with three bays lit, a drone lifting off its roof."""
    sp = Sprite(32, 32)
    sp.part(sp.poly(hexagon(16, 22, 13, 0.55)), R["steel"], "rim")                   # the hex roof
    sp.part(sp.rect(5, 22, 27, 29), R["metal"], "rim")                               # walls
    for x in (8, 15, 22):
        sp.fill(sp.rect(x, 24, x + 3, 27), "#faaf5c")                                # lit bays
    sp.part(sp.rect(12, 6, 20, 9), R["blue"], "rim")                                 # a drone
    sp.part(sp.rect(9, 5, 11, 6) | sp.rect(21, 5, 23, 6), R["steel"], "flat", base=2)   # its rotors
    sp.plot(15, 10, "#fa3a64")
    sp.plot(7, 20, WHITE)
    sp.outline()
    return sp.image()


def derrick():
    """The onshore drilling rig: a lattice derrick on a substructure, the pipe rack beside it."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(4, 23, 26, 29), R["paint"], "rim")                               # substructure
    sp.part(sp.line([(8, 23), (14, 2)], 2) | sp.line([(22, 23), (16, 2)], 2), R["iron"], "flat", base=2)
    for y in (8, 13, 18):
        x0, x1 = 8 + (23 - y) * 6 / 21, 22 - (23 - y) * 6 / 21
        sp.part(sp.line([(x0, y), (x1, y)], 1), R["iron"], "flat", base=2, seam=False)
    sp.part(sp.rect(13, 1, 17, 3), R["redpaint"], "rim")                             # crown block
    sp.part(sp.rect(24, 26, 31, 29), R["metal"], "cyl_h")                            # pipe rack
    sp.plot(5, 24, WHITE)
    sp.outline()
    return sp.image()


def platform():
    """The offshore platform: legs in the sea, a deck of modules, a derrick, a turbine stack, the flare."""
    sp = Sprite(32, 32)
    sp.fill(sp.rect(0, 26, 31, 31), "#15578a")                                       # the sea
    for x in (6, 22):
        sp.part(sp.rect(x, 17, x + 3, 30), R["rust"], "rim")                         # legs
    sp.part(sp.rect(3, 14, 28, 18), R["metal"], "rim")                               # the deck
    sp.part(sp.rect(4, 9, 12, 14), R["white"], "rim")                                # quarters
    sp.fill(sp.rect(6, 11, 10, 11), "#faaf5c")                                       # lit windows
    sp.part(sp.line([(15, 14), (17, 2)], 1) | sp.line([(20, 14), (18, 2)], 1), R["iron"], "flat", base=2)   # derrick
    sp.part(sp.rect(22, 8, 25, 14), R["steel"], "rim")                               # turbine stack
    sp.part(sp.line([(26, 14), (30, 5)], 1), R["iron"], "flat", base=2)             # flare boom
    sp.part(sp.circle(30, 3, 1.8), R["flame"], "sphere", glow=True)                  # the flare
    sp.plot(5, 10, WHITE)
    sp.outline()
    return sp.image()


def tanker():
    """A tanker: a long low red hull, the deck's pipe runs, the bridge aft."""
    sp = Sprite(32, 32)
    sp.fill(sp.rect(0, 25, 31, 31), "#15578a")
    sp.part(sp.poly([(1, 18), (30, 18), (27, 26), (3, 26)]), R["redpaint"], "rim")   # hull
    sp.part(sp.rect(3, 15, 25, 18), R["metal"], "rim")                               # deck and its tanks
    sp.part(sp.rect(4, 14, 22, 14), R["steel"], "flat", base=2, seam=False)          # the pipe run
    sp.part(sp.rect(24, 8, 29, 18), R["white"], "rim")                               # bridge
    sp.fill(sp.rect(25, 10, 28, 10), "#74effe")
    sp.part(sp.rect(26, 5, 27, 8), R["soot"], "flat", base=1)                        # funnel
    sp.plot(4, 19, WHITE)
    sp.outline()
    return sp.image()


def freighter():
    """A container ship: dark hull, stacked boxes in three colours, the bridge aft."""
    sp = Sprite(32, 32)
    sp.fill(sp.rect(0, 25, 31, 31), "#15578a")
    sp.part(sp.poly([(1, 19), (30, 19), (28, 26), (3, 26)]), R["soot"], "rim")       # hull
    cols = [R["red"], R["green"], R["blue"], R["gold"]]
    for i, x in enumerate(range(3, 21, 5)):
        for j, y in enumerate((15, 11)):
            sp.part(sp.rect(x, y, x + 4, y + 3), cols[(i + j) % 4], "rim")          # containers
    sp.part(sp.rect(23, 8, 28, 19), R["white"], "rim")                               # bridge
    sp.fill(sp.rect(24, 10, 27, 10), "#74effe")
    sp.plot(4, 12, WHITE)
    sp.outline()
    return sp.image()


def pipe():
    """A pipe run: a flanged elbow on a stand, a valve wheel, a blue water band."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(2, 12, 20, 18), R["steel"], "cyl_h")                             # run
    sp.part(sp.rect(16, 12, 22, 29), R["steel"], "rim")                              # elbow down
    sp.part(sp.rect(7, 11, 8, 19) | sp.rect(15, 20, 23, 21), R["metal"], "flat", base=1)   # flanges
    sp.fill(sp.rect(10, 14, 13, 16), "#1096c3")                                      # the fluid's band
    sp.part(sp.circle(5, 8, 3.5), R["red"], "rim")                                   # valve wheel
    sp.part(sp.rect(4, 10, 5, 12), R["metal"], "flat", base=1)
    sp.plot(3, 13, WHITE)
    sp.outline()
    return sp.image()


def tube():
    """A pneumatic tube: a glass pipe with a brass capsule in it, a send station's mouth."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(1, 11, 24, 19), R["glass"], "cyl_h")                            # the tube
    sp.part(sp.rect(9, 12, 17, 18), R["brass"], "cyl_h")                             # the capsule
    sp.part(sp.rect(22, 6, 30, 26), R["metal"], "rim")                               # station
    sp.fill(sp.rect(24, 12, 28, 18), "#2c2c25")                                      # its mouth
    sp.plot(27, 8, "#9bb675", glow=True)                                             # ready lamp
    sp.plot(3, 12, WHITE)
    sp.outline()
    return sp.image()


def belt():
    """A conveyor belt section with ore riding it, on two legs."""
    sp = Sprite(32, 32)
    sp.part(sp.poly([(1, 20), (31, 12), (31, 16), (1, 24)]), R["soot"], "rim")      # the belt
    for x in (4, 12, 20, 28):
        y = 21 - (x - 1) * 8 / 30
        sp.plot(x, int(y), "#434137")                                                 # cleats
    sp.part(sp.circle(9, 17, 2.5), R["copper"], "sphere")                            # ore
    sp.part(sp.circle(20, 14, 2.5), R["soot"], "sphere")                             # coal
    sp.part(sp.rect(5, 23, 6, 30) | sp.rect(25, 16, 26, 30), R["iron"], "flat", base=2)   # legs
    sp.plot(8, 16, WHITE)
    sp.outline()
    return sp.image()


def printer():
    """The hex printer (M22): a gantry over a print bed, a nozzle laying a hex wall."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(2, 26, 29, 29), R["metal"], "rim")                               # bed
    sp.part(sp.poly(hexagon(16, 22, 9, 0.45)), R["sand"], "rim")                     # printed hex course
    for x in (3, 27):
        sp.part(sp.rect(x, 4, x + 2, 26), R["steel"], "flat", base=2)
    sp.part(sp.rect(3, 4, 29, 6), R["steel"], "rim")
    sp.part(sp.rect(14, 6, 18, 13), R["paint"], "rim")                               # head
    sp.plot(16, 15, "#fe6420", glow=True)
    sp.plot(4, 5, WHITE)
    sp.outline()
    return sp.image()


def launchpad():
    """A launch pad: a tower, a rocket on the pad, fuel tanks and their pipes."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(1, 27, 30, 30), R["metal"], "rim")                               # pad
    sp.part(sp.rect(4, 4, 7, 27), R["red"], "flat", base=2)                          # tower
    sp.part(sp.rect(12, 6, 17, 26), R["white"], "cyl_h")                             # rocket
    sp.part(sp.poly([(12, 6), (17, 6), (14.5, 1)]), R["red"], "rim")
    sp.part(sp.circle(25, 21, 4), R["white"], "sphere")                              # oxygen sphere
    sp.part(sp.rect(18, 24, 22, 25), R["steel"], "flat", base=1)                     # fuel line
    sp.plot(13, 8, WHITE)
    sp.outline()
    return sp.image()


def lox():
    """Liquid oxygen: a pale blue canister frosted white."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(9, 6, 22, 29), R["cyan"], "cyl_h")
    sp.part(sp.rect(12, 2, 19, 6), R["steel"], "rim")
    sp.fill(sp.rect(10, 15, 21, 18), WHITE)
    sp.plot(11, 8, WHITE)
    sp.outline()
    return sp.image()


ICONS.update({"worker": worker, "excavator": excavator, "quarry": quarry, "bucketwheel": bucketwheel, "hive": hive,
              "derrick": derrick, "platform": platform, "tanker": tanker, "freighter": freighter,
              "pipe": pipe, "tube": tube, "belt": belt, "printer": printer, "launchpad": launchpad, "lox": lox})


def main():
    os.makedirs(OUT, exist_ok=True)
    items = []
    for name, fn in ICONS.items():
        img = fn()
        p = os.path.join(OUT, name + ".png")
        save(img, p, source="pixelkit (tools/art/industry_art.py)")
        items.append((name, img))
    print("wrote", len(items), "files to", OUT)
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1]
        tiled = [(n + " 2x2", Image.fromarray(np.tile(np.array(im), (2, 2, 1)), "RGBA")) for n, im in items if n.startswith("tex_")]
        refs = [("ref " + r, Image.open(os.path.join(SKILL, "references", r + ".png")).convert("RGBA")) for r in REFS]
        sheet([i for i in items if not i[0].startswith("tex_")] + tiled + refs, scale=4, cols=8, cell=(64 * 4, 64 * 4)).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main()
