"""Part icons, livery decals and textures for the hangar mockup (docs/mockups/hangar.html, M24).

Pixel art drawn in code with the balatro-pixel-art kit (`.claude/skills/balatro-pixel-art`), the
solitaire collection's style, every colour from Sweaft-64 (the kit's save() refuses anything else).
Four kinds of output, all into docs/mockups/hangar/:

- part icons: 32 x 32, transparent, one bold subject, 1 px outline, light from the top left, drawn
  chibi (fat bodies, big canopies, fat blades); the page shows them only at whole multiples of 32 px.
  Each part is drawn once in every livery paint (its painted panels in that paint's ramp, nothing
  else changed), into `<part>-paints.png`, a 288 x 32 strip in PAINTS order, so the builder's grid
  shows each piece in its colour (the owner: "would be nice for the grid pieces to actually get
  colors too"); `<part>.png` is the red one, for the part cards and the hotbar;
- livery decals: the shark mouth (32 x 16), three roundels of the player's own (16 x 16: no real air
  force's insignia) and a 3 x 5 digit sheet for nose numbers (50 x 7), hard alpha, used by the page's
  three.js aircraft as cut-out cards with alphaTest 0.5;
- textures: 32 x 32 tiles that repeat seamlessly, opaque, no outline: the livery paints (one per
  colour, panel seams at 0.5 m), canopy glass, rotor blade, tyre, the hangar's corrugated door,
  runway asphalt, markings and the windsock's stripes;
- nothing of the game's: the ground, the apron, the hangar's walls and roof, the struts and the
  Kestrel wear the game's own atlas, settlement textures and model, copied byte for byte into
  docs/mockups/hangar/game/.

    python tools/art/hangar_art.py
    python tools/art/hangar_art.py --sheet OUT.png
"""

import glob
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art")
sys.path.insert(0, os.path.join(SKILL, "scripts"))
from pixelkit import RAMPS, Sprite, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "hangar")
N = 32
GLINT = "#fffefe"

R = dict(RAMPS)
R.update({
    "white": ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "dark": ["#010e1a", "#011c34", "#1f3b52", "#3e586e", "#5d778d"],
    "tyre": ["#010000", "#161612", "#2c2c25", "#434137"],
    "plume": ["#15578a", "#1096c3", "#74effe", "#d2f8ff", "#fffefe"],
})

# The livery paints, each a ramp dark to light; the page's swatches are their body tone (index 2).
PAINTS = {
    "red": ["#4a051e", "#a61e1f", "#d12d41", "#fa3a64"],
    "orange": ["#733126", "#c56d25", "#fb902f", "#faaf5c"],
    "sunny": ["#8d4526", "#fb902f", "#faaf5c", "#fffefe"],
    "green": ["#05582d", "#297537", "#559642", "#9bb675"],
    "teal": ["#0c2f44", "#204f63", "#53998b", "#84e3b5"],
    "blue": ["#191954", "#15578a", "#1096c3", "#74effe"],
    "white": ["#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "violet": ["#230c44", "#4f1d83", "#7d2dbd", "#9e3ba2"],
    "tan": ["#41322c", "#725244", "#ab8465", "#cfaa81"],
}

YS, XS = np.mgrid[0:N, 0:N]
SPECK = np.random.default_rng(11).random((N, N)) < 0.04  # fixed grain, the same every run


# ---------------------------------------------------------------- part icons (32 x 32)
# Side views, nose to the right, chibi: everything fat and round-shouldered.

def body(sp, x0, x1, y0, y1, ramp, nose=4, tail=3):
    """A fat fuselage, side on, with a rounded nose and tail."""
    m = sp.poly([(x0, y0 + tail), (x0 + tail, y0), (x1 - nose, y0), (x1, y0 + nose),
                 (x1, y1 - nose), (x1 - nose, y1), (x0 + tail, y1), (x0, y1 - tail)])
    return sp.part(m, ramp, "cyl_h", base=2)


# Every icon takes P, the paint ramp (dark to light, four tones from PAINTS): the part's painted
# panels are drawn in it and nothing else is, so one drawing makes every colour's variant with
# exact palette tones (no hue shifting). The page shows the variant of the piece's colour.

def cockpit(sp, P):
    body(sp, 2, 29, 16, 28, P)
    canopy = sp.poly([(8, 17), (11, 8), (16, 5), (22, 5), (26, 9), (27, 17)])
    sp.part(canopy, R["cyan"], "sphere", base=2)
    sp.fill(sp.line([(13, 14), (17, 8)], 2) & canopy, R["cyan"][4])
    sp.fill(sp.rect(18, 5, 19, 16) & canopy, R["dark"][2])
    sp.plots([(14, 9), (15, 8)], GLINT)
    sp.outline()


def fuselage(sp, P):
    body(sp, 1, 30, 6, 26, P, nose=7, tail=7)
    for x in (11, 20):
        sp.fill(sp.rect(x, 7, x, 25), P[1])
    sp.part(sp.rect(2, 15, 29, 18), R["white"], "flat", base=3)
    sp.plots([(8, 11), (9, 11)], GLINT)
    sp.outline()


def prop_engine(sp, P):
    cowl = sp.poly([(3, 9), (18, 7), (22, 10), (22, 22), (18, 25), (3, 23)])
    sp.part(cowl, P, "cyl_h", base=2)
    for x in (8, 13):
        sp.fill(sp.rect(x, 9, x, 23) & cowl, P[0])
    spinner = sp.poly([(22, 12), (27, 14), (29, 16), (27, 18), (22, 20)])
    sp.part(spinner, R["white"], "cyl_h", base=3)
    sp.part(sp.taper(25, 15, 25, 1, 5, 3), R["soot"], "rim", base=2)
    sp.part(sp.taper(25, 17, 25, 31, 5, 3), R["soot"], "rim", base=2)
    sp.fill(sp.rect(23, 1, 27, 3), R["white"][3])
    sp.fill(sp.rect(23, 29, 27, 31), R["white"][3])
    sp.plots([(6, 10), (7, 10)], GLINT)
    sp.outline()


def jet(sp, P):
    pod = sp.poly([(6, 7), (24, 6), (29, 10), (29, 22), (24, 26), (6, 25)])
    sp.part(pod, P, "cyl_h", base=2)
    sp.part(sp.rect(25, 9, 30, 23), R["dark"], "cyl_h", base=3)
    sp.part(sp.rect(6, 10, 9, 22), R["dark"], "flat", base=1)
    sp.fill(sp.rect(14, 6, 15, 26) & pod, P[0])
    sp.part(sp.rect(16, 13, 22, 19), R["white"], "flat", base=3)
    sp.plots([(10, 8), (11, 8)], GLINT)
    sp.outline()
    sp.part(sp.poly([(0, 11), (5, 10), (5, 22), (0, 21)]), R["flame"], "inset", glow=True)


def wing(sp, P):
    # three-quarter view: a thick stubby wing, leading edge to the right, roundel on its top
    top = sp.poly([(1, 18), (17, 4), (30, 7), (14, 23)])
    sp.part(top, P, "rim", base=2, depth=1)
    edge = sp.poly([(14, 23), (30, 7), (30, 12), (14, 28)])
    sp.part(edge, P, "flat", base=1)
    sp.part(sp.poly([(1, 18), (14, 23), (14, 28), (1, 22)]), P, "flat", base=0)
    sp.part(sp.poly([(17, 4), (22, 5), (9, 20), (5, 19)]), R["white"], "rim", base=3)
    sp.part(sp.ellipse(16, 10, 24, 15), R["blue"], "flat", base=2)
    sp.part(sp.ellipse(19, 11, 21, 13), R["paper"], "flat", base=2)
    sp.plots([(16, 5), (17, 5)], GLINT)
    sp.outline()


def tailplane(sp, P):
    w = sp.poly([(2, 20), (16, 8), (30, 10), (16, 23)])
    sp.part(w, P, "rim", base=2, depth=1)
    sp.part(sp.poly([(16, 23), (30, 10), (30, 14), (16, 27)]), P, "flat", base=0)
    sp.part(sp.poly([(2, 20), (16, 23), (16, 27), (2, 23)]), P, "flat", base=0)
    sp.part(sp.poly([(22, 9), (30, 10), (27, 13), (19, 12)]), R["white"], "rim", base=3)
    sp.plots([(14, 9), (15, 9)], GLINT)
    sp.outline()


def fin(sp, P):
    f = sp.poly([(4, 29), (10, 6), (16, 3), (22, 4), (25, 8), (27, 29)])
    sp.part(f, P, "rim", base=2, depth=2)
    sp.part(sp.poly([(9, 12), (11, 6), (16, 3), (22, 4), (24, 7), (24, 12)]), R["white"], "rim", base=3)
    sp.fill(sp.rect(17, 14, 18, 28) & f, P[0])
    sp.plots([(12, 7), (13, 6)], GLINT)
    sp.outline()


def rotor(sp, P):
    sp.part(sp.poly([(4, 31), (7, 24), (24, 24), (27, 31)]), P, "cyl_h", base=2)
    sp.part(sp.rect(13, 12, 18, 25), R["steel"], "cyl_v", base=2)
    sp.part(sp.rect(11, 8, 20, 13), R["dark"], "rim", base=3)
    sp.part(sp.taper(1, 8, 15, 10, 4, 6), R["soot"], "rim", base=2)
    sp.part(sp.taper(16, 10, 30, 8, 6, 4), R["soot"], "rim", base=2)
    sp.fill(sp.rect(1, 6, 3, 10), P[2])
    sp.fill(sp.rect(28, 6, 30, 10), P[2])
    sp.plots([(14, 13), (14, 14)], GLINT)
    sp.outline()


def tail_rotor(sp, P):
    boom = sp.poly([(1, 18), (18, 16), (18, 23), (1, 22)])
    sp.part(boom, P, "cyl_h", base=2)
    f = sp.poly([(14, 24), (17, 9), (24, 7), (25, 24)])
    sp.part(f, P, "rim", base=2)
    hub = sp.circle(21, 15, 2)
    sp.part(sp.taper(21, 15, 21, 1, 5, 4), R["soot"], "rim", base=2)
    sp.part(sp.taper(21, 15, 21, 30, 5, 4), R["soot"], "rim", base=2)
    sp.fill(sp.rect(19, 1, 23, 3), R["white"][3])
    sp.fill(sp.rect(19, 28, 23, 30), R["white"][3])
    sp.part(hub, R["white"], "sphere", base=3)
    sp.outline()


def nacelle(sp, P):
    pod = sp.poly([(10, 31), (10, 14), (13, 11), (19, 11), (22, 14), (22, 31)])
    sp.part(pod, P, "cyl_v", base=2)
    sp.part(sp.rect(10, 24, 22, 26), R["white"], "cyl_v", base=3)
    sp.part(sp.rect(13, 7, 19, 11), R["dark"], "rim", base=3)
    sp.part(sp.taper(1, 5, 15, 7, 4, 6), R["soot"], "rim", base=2)
    sp.part(sp.taper(17, 7, 31, 5, 6, 4), R["soot"], "rim", base=2)
    sp.fill(sp.rect(1, 3, 3, 7), R["white"][3])
    sp.fill(sp.rect(29, 3, 31, 7), R["white"][3])
    sp.plots([(12, 15), (12, 16)], GLINT)
    sp.outline()


def gear(sp, P):
    sp.part(sp.rect(14, 2, 18, 15), R["steel"], "cyl_v", base=2)
    sp.part(sp.rect(9, 1, 23, 5), P, "rim", base=2)
    wheel = sp.circle(16, 21, 9)
    sp.part(wheel, R["tyre"], "sphere", base=2)
    sp.part(sp.circle(16, 21, 4), P, "sphere", base=2)
    sp.part(sp.circle(16, 21, 1), R["white"], "flat", base=3)
    sp.plots([(11, 15), (12, 14)], R["tyre"][3])
    sp.outline()


def float_(sp, P):
    for x in (9, 22):
        sp.part(sp.rect(x, 4, x + 2, 16), R["steel"], "cyl_v", base=2)
    pont = sp.poly([(2, 17), (24, 16), (29, 18), (31, 21), (27, 26), (4, 26), (2, 23)])
    sp.part(pont, P, "cyl_h", base=2)
    sp.part(sp.rect(3, 23, 28, 25) & pont, R["white"], "flat", base=3)
    sp.part(sp.rect(4, 1, 27, 4), R["steel"], "rim", base=2)
    sp.plots([(7, 18), (8, 18)], GLINT)
    sp.outline()


def fuel(sp, P):
    tank = sp.poly([(2, 16), (6, 10), (24, 10), (30, 16), (24, 22), (6, 22)])
    sp.part(tank, P, "cyl_h", base=2)
    sp.part(sp.rect(12, 10, 17, 22) & tank, R["white"], "cyl_h", base=3)
    sp.part(sp.poly([(13, 13), (16, 13), (17, 16), (14, 19), (13, 16)]), R["flame"], "flat", base=2, seam=False)
    sp.part(sp.rect(18, 5, 21, 9), R["steel"], "rim", base=2)
    sp.plots([(7, 11), (8, 11)], GLINT)
    sp.outline()


def cargo(sp, P):
    top = sp.poly([(7, 5), (29, 5), (25, 10), (3, 10)])
    front = sp.rect(3, 11, 24, 28)
    side = sp.poly([(25, 11), (29, 6), (29, 23), (25, 28)])
    sp.part(front, P, "rim", base=2)
    for x in (8, 14, 19):
        sp.fill(sp.rect(x, 12, x, 27), P[1])
    sp.part(side, P, "flat", base=1)
    sp.part(top, P, "flat", base=3)
    sp.part(sp.rect(4, 17, 23, 21), R["white"], "flat", base=3)
    sp.plots([(8, 6), (9, 6)], GLINT)
    sp.outline()


def utility(sp, P):
    pod = sp.poly([(2, 11), (20, 10), (24, 13), (24, 21), (20, 23), (2, 22)])
    sp.part(pod, P, "cyl_h", base=2)
    sp.part(sp.rect(9, 4, 13, 10), R["steel"], "rim", base=2)
    sp.part(sp.rect(22, 12, 26, 22), R["dark"], "rim", base=3)
    sp.plots([(5, 12), (6, 12)], GLINT)
    sp.outline()
    beam = sp.poly([(27, 13), (31, 9), (31, 25), (27, 21)])
    sp.part(beam, R["gold"], "inset", glow=True)
    sp.plot(27, 17, GLINT, glow=True)


def livery(sp, P):
    # the paint brush tool: a spray can, its band in the brush's colour
    can = sp.poly([(5, 10), (19, 10), (19, 30), (5, 30)])
    sp.part(can, R["white"], "cyl_v", base=3)
    sp.part(sp.rect(5, 16, 19, 23), P, "cyl_v", base=2)
    sp.part(sp.rect(9, 5, 15, 9), R["steel"], "rim", base=2)
    sp.part(sp.rect(10, 2, 14, 4), R["dark"], "rim", base=3)
    sp.plots([(7, 12), (7, 13)], GLINT)
    sp.outline()
    for (x, y), c in zip([(22, 4), (26, 7), (23, 10), (28, 12), (25, 15)], [P[2], P[3], P[2], P[3], P[1]]):
        sp.fill(sp.rect(x, y, x + 2, y + 2), c, glow=True)


ICONS = {
    "cockpit": cockpit, "fuselage": fuselage, "prop": prop_engine, "jet": jet, "wing": wing,
    "tailplane": tailplane, "fin": fin, "rotor": rotor, "tail-rotor": tail_rotor, "nacelle": nacelle,
    "gear": gear, "float": float_, "fuel": fuel, "cargo": cargo, "utility": utility, "livery": livery,
}


# ---------------------------------------------------------------- livery decals

def shark_mouth():
    """32 x 16, the nose to the right, as on the owner's first reference."""
    sp = Sprite(32, 16)
    mouth = sp.poly([(0, 7), (8, 4), (20, 3), (30, 5), (31, 8), (24, 12), (12, 13), (2, 10)])
    sp.part(mouth, ["#4a051e", "#841c1d", "#a61e1f", "#d12d41"], "inset", seam=False)
    sp.fill(sp.poly([(4, 8), (14, 10), (26, 8), (24, 11), (12, 12), (4, 9)]), "#fa3a64")
    for x in range(3, 31, 3):
        top = 3 + (0 if 6 < x < 26 else 1)
        sp.fill(sp.poly([(x, top + 1), (x + 2, top + 1), (x + 1, top + 4)]), "#fffefe")
    for x in range(5, 27, 3):
        sp.fill(sp.poly([(x, 12), (x + 2, 12), (x + 1, 9)]), "#bfcad7")
    sp.outline()
    eye = sp.circle(25, 1, 1) | sp.rect(24, 0, 27, 1)
    sp.fill(eye & ~sp.rect(0, 2, 31, 15), "#fffefe")
    sp.plot(26, 1, "#161612")
    return sp.image()


def roundel(kind):
    sp = Sprite(16, 16)
    disc = sp.circle(7.5, 7.5, 7)
    if kind == "dot":   # the pale blue dot: the game's own name, ringed
        sp.fill(disc, "#15578a")
        sp.fill(sp.circle(7.5, 7.5, 5), "#fffefe")
        sp.fill(sp.circle(7.5, 7.5, 3), "#1096c3")
        sp.fill(sp.circle(6.5, 6.5, 1), "#74effe")
    elif kind == "sun":
        sp.fill(disc, "#a61e1f")
        sp.fill(sp.star(7.5, 7.5, 6.5, 3.5, 8, -90), "#fb902f")
        sp.fill(sp.circle(7.5, 7.5, 2.5), "#faaf5c")
        sp.plot(6, 6, "#fffefe")
    else:   # chevron
        sp.fill(disc, "#fffefe")
        sp.fill(sp.circle(7.5, 7.5, 6), "#bfcad7")
        sp.fill(sp.poly([(3, 5), (7.5, 9), (12, 5), (12, 8), (7.5, 12), (3, 8)]), "#d12d41")
    sp.outline()
    return sp.image()


DIGITS = [  # 3 x 5, row strings top to bottom
    ["111", "101", "101", "101", "111"], ["010", "110", "010", "010", "111"],
    ["111", "001", "111", "100", "111"], ["111", "001", "111", "001", "111"],
    ["101", "101", "111", "001", "001"], ["111", "100", "111", "001", "111"],
    ["111", "100", "111", "101", "111"], ["111", "001", "010", "010", "010"],
    ["111", "101", "111", "101", "111"], ["111", "101", "111", "001", "111"],
]


def digit_sheet():
    """Ten glyphs, each in a 5 x 7 cell: white 3 x 5 strokes with a 1 px ink outline."""
    sp = Sprite(50, 7)
    for d, rows in enumerate(DIGITS):
        m = sp.empty()
        for y, row in enumerate(rows):
            for x, c in enumerate(row):
                if c == "1":
                    m[1 + y, d * 5 + 1 + x] = True
        sp.fill(m, "#fffefe")
    sp.outline()
    return sp.image()


# ---------------------------------------------------------------- textures (32 x 32 tiles)
# Each repeats seamlessly: every pattern's period divides 32. The page maps them at 1 m a repeat on
# the aircraft (paint seams every 0.5 m) and at the metres named in its code on the field.

def tile():
    return Sprite(N, N)


def paint(sp, ramp):
    full = sp.rect(0, 0, N - 1, N - 1)
    sp.fill(full, ramp[2])
    sp.fill(full & SPECK, ramp[1])
    u, v = XS % 16, YS % 32
    sp.fill(u == 0, ramp[1])
    sp.fill(u == 1, ramp[3])
    sp.fill(v == 0, ramp[1])
    sp.fill(v == 1, ramp[3])
    sp.fill((u == 15) & (v > 1), ramp[2])
    sp.fill(((u == 0) & (v % 4 == 2)) | ((v == 0) & (XS % 4 == 2)), ramp[0])


def tex_canopy(sp):
    ramp = R["cyan"]
    sp.fill(sp.rect(0, 0, 31, 31), ramp[1])
    sp.dither(sp.rect(0, 0, 31, 31) & (YS < 16), ramp[1], ramp[2], 0.3)
    d = (XS + YS) % 32
    sp.fill((d == 6) | (d == 7), ramp[3])
    sp.fill(d == 13, ramp[2])
    sp.fill(((XS + YS) % 32 == 7) & (XS % 8 == 0), ramp[4])
    sp.fill((XS % 16 == 0) | (YS % 32 == 0), R["dark"][2])
    sp.fill((XS % 16 == 1), R["dark"][3])


def tex_blade(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["soot"][1])
    sp.fill(YS % 32 < 3, R["soot"][3])
    sp.fill(YS % 32 == 3, R["soot"][2])
    sp.fill(YS % 32 == 31, R["ink"][0])
    sp.fill(SPECK & (YS % 32 > 4), R["soot"][2])


def tex_tyre(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["tyre"][2])
    u, v = XS % 8, YS % 8
    tread = ((u < 3) & (v < 5)) | ((u >= 4) & (u < 7) & (v >= 4))
    sp.fill(tread, R["tyre"][1])
    sp.fill(tread & (v % 8 == 0), R["tyre"][3])
    sp.fill(SPECK, R["tyre"][0])


def tex_door(sp):
    ramp = R["white"]
    u = XS % 4
    sp.fill(u == 0, ramp[0])
    sp.fill(u == 1, ramp[3])
    sp.fill(u == 2, ramp[2])
    sp.fill(u == 3, ramp[1])
    sp.fill((YS % 32 == 0), R["dark"][1])
    sp.fill((YS % 32 == 1), ramp[1])
    band = (YS % 32 >= 14) & (YS % 32 <= 16)
    sp.fill(band & (((XS + YS) // 3) % 2 == 0), R["ink"][1])
    sp.fill(band & (((XS + YS) // 3) % 2 == 1), PAINTS["sunny"][2])


def tex_runway(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["soot"][1])
    grain = np.random.default_rng(23).random((N, N))
    sp.fill(grain < 0.12, R["soot"][2])
    sp.fill(grain > 0.93, R["ink"][1])
    crack = sp.line([(3, 8), (9, 11), (14, 10)], 1) | sp.line([(20, 26), (25, 23), (31, 24)], 1)
    sp.fill(crack, R["ink"][0])


def tex_marking(sp):
    sp.fill(sp.rect(0, 0, 31, 31), "#bfcad7")
    grain = np.random.default_rng(29).random((N, N))
    sp.fill(grain < 0.10, "#92a3b2")
    sp.fill(grain > 0.96, "#fffefe")


def tex_marking_yellow(sp):
    sp.fill(sp.rect(0, 0, 31, 31), "#fb902f")
    grain = np.random.default_rng(31).random((N, N))
    sp.fill(grain < 0.10, "#c56d25")
    sp.fill(grain > 0.95, "#faaf5c")


def tex_sock(sp):
    band = (YS % 16) < 8
    sp.fill(band, "#fe6420")
    sp.fill(~band, "#bfcad7")
    sp.fill(band & (YS % 16 == 0), "#faaf5c")
    sp.fill(~band & (YS % 16 == 8), "#fffefe")
    sp.fill(XS % 16 == 0, "#733126")


TEXTURES = {
    "tex-canopy": tex_canopy, "tex-blade": tex_blade, "tex-tyre": tex_tyre, "tex-door": tex_door,
    "tex-runway": tex_runway, "tex-marking": tex_marking, "tex-marking-yellow": tex_marking_yellow,
    "tex-sock": tex_sock,
}
for _name, _ramp in PAINTS.items():
    TEXTURES["paint-" + _name] = (lambda r: (lambda sp: paint(sp, r)))(_ramp)


def main():
    os.makedirs(OUT, exist_ok=True)
    src = "balatro-pixel-art pixelkit"
    icons, decals, texes, sheets = [], [], [], []
    from PIL import Image
    for name, draw in ICONS.items():
        # one sheet a part: its variant in each paint, PAINTS order, 32 px a frame
        strip = Image.new("RGBA", (N * len(PAINTS), N))
        for i, ramp in enumerate(PAINTS.values()):
            sp = Sprite(N, N)
            draw(sp, ramp)
            img = sp.image()
            strip.paste(img, (i * N, 0))
            if i == 0:
                save(img, os.path.join(OUT, name + ".png"), source=src)   # red: the cards and the hotbar
                icons.append((name, img))
        save(strip, os.path.join(OUT, name + "-paints.png"), source=src)
        sheets.append((name + "-paints", strip))
    for name, img in [("shark-mouth", shark_mouth()), ("roundel-dot", roundel("dot")),
                      ("roundel-sun", roundel("sun")), ("roundel-chevron", roundel("chevron")),
                      ("digits", digit_sheet())]:
        save(img, os.path.join(OUT, name + ".png"), source=src)
        decals.append((name, img))
    for name, draw in TEXTURES.items():
        sp = tile()
        draw(sp)
        img = sp.image()
        assert img.getchannel("A").getextrema() == (255, 255), name + " has holes"
        save(img.convert("RGB"), os.path.join(OUT, name + ".png"), source=src)
        texes.append((name, img))
    print("wrote", len(icons), "icons,", len(sheets), "paint sheets,", len(decals), "decals and", len(texes), "textures to", OUT)
    if "--sheet" in sys.argv:
        from PIL import Image
        path = sys.argv[sys.argv.index("--sheet") + 1]
        refs = sorted(p for p in glob.glob(os.path.join(SKILL, "references", "*.png")) if not p.endswith("_contact.png"))
        refs = [(os.path.basename(p)[:-4], Image.open(p).convert("RGBA")) for p in refs]
        refs = [r for r in refs if r[1].size == (56, 56)][-4:]
        tiled = []
        for name, img in texes:
            t = Image.new("RGBA", (64, 64))
            for ox in (0, 32):
                for oy in (0, 32):
                    t.paste(img, (ox, oy))
            tiled.append((name, t))
        big = []
        for name, img in decals:
            c = Image.new("RGBA", (56, 56))
            c.paste(img, ((56 - img.width) // 2, (56 - img.height) // 2))
            big.append((name, c))
        sheet(refs + icons + big + tiled, scale=3, cols=8).save(path)
        print("sheet", path)
        # the paint variants: every part in every colour, one row a part
        rows = Image.new("RGBA", (N * len(PAINTS), N * len(sheets)), (31, 59, 82, 255))
        for k, (name, strip) in enumerate(sheets):
            rows.alpha_composite(strip, (0, k * N))
        rows.resize((rows.width * 3, rows.height * 3), Image.NEAREST).save(path.replace(".png", "-paints.png"))
        print("sheet", path.replace(".png", "-paints.png"))


if __name__ == "__main__":
    main()
