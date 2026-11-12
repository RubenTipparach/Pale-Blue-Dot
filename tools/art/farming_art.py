"""Crop growth stages, produce, farm tools and soil textures for the farming mockup
(docs/mockups/farming.html).

Painted by code with the hand-drawn-illustration kit
(.claude/skills/hand-drawn-illustration/scripts/crayon.py), in the manner of the
owner's friend's icons. It is not hand drawn and the page says so.

Each crop has four sprites, `<crop>_0..3` (seeded, sprout, growing, ripe). The plant
stands on the bottom centre of the square, so the mockup's three.js scene can use a
sprite as a billboard standing on its cell.

    python tools/art/farming_art.py            # writes docs/mockups/farming/*.png
    python tools/art/farming_art.py --sheet S  # also a contact sheet beside the friend's icons
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)
from crayon import Icon, circle, ellipse, rect, rotate, translate, mix, darker, lighter, tone  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "farming")
BASE = (256, 492)
GREEN = (80, 170, 60)
YOUNG = (130, 210, 80)
GOLD = (236, 190, 60)


def leaf_poly(x, y, ang, length, width, n=24):
    """A pointed leaf from (x, y) out along `ang` (radians, 0 = right, -pi/2 = up)."""
    pts = []
    for i in range(n + 1):
        t = i / n
        pts.append((t * length, -math.sin(t * math.pi) * width * (1 - 0.3 * t)))
    for i in range(n, -1, -1):
        t = i / n
        pts.append((t * length, math.sin(t * math.pi) * width * (1 - 0.3 * t) * 0.6))
    return translate(rotate(pts, ang), x, y)


def leaf(ic, x, y, ang, length, width, col, vein=True):
    ic.drawn(leaf_poly(x, y, ang, length, width), col, edge_w=6, highlight=0.12)
    if vein:
        ic.line([(x, y), (x + math.cos(ang) * length * 0.85, y + math.sin(ang) * length * 0.85)], tone(col, 0.3), 4)


def frilly(cx, cy, rx, ry, rot=0.0, bumps=9):
    pts = []
    for i in range(90):
        t = 2 * math.pi * i / 90
        r = 1 + 0.1 * math.sin(bumps * t)
        pts.append((rx * r * math.cos(t), ry * r * math.sin(t)))
    return translate(rotate(pts, rot), cx, cy)


def mound(ic, seed_col):
    ic.drawn(ellipse(256, 450, 200, 52), (120, 82, 52), shade=0.25, highlight=0.1, edge_w=7)
    for dx, dy in ((-90, -6), (-30, -18), (30, -10), (90, -2), (0, 6)):
        ic.drawn(ellipse(256 + dx, 450 + dy, 15, 10, rot=0.4), seed_col, hatch=0.3, shade=0, highlight=0, edge_w=4)


# ---------- crops: fn(ic, stage) for stage 1..3 -------------------------------------

def wheat(ic, s, ripe=GOLD, droop=False, green=GREEN):
    n = (0, 3, 5, 7)[s] + (2 if droop else 0)
    h = (0, 150, 290, 400)[s] * (0.8 if droop else 1)
    col = (YOUNG, YOUNG, green, mix(green, ripe, 0.75 if not droop else 0.45))[s]
    for i in range(n):
        f = (i - (n - 1) / 2) / max(1, n - 1)
        x0 = 256 + f * 70
        top = (x0 + f * 120, BASE[1] - h * (0.85 + 0.15 * math.cos(i * 1.7)))
        ic.line([(x0, BASE[1]), ((x0 + top[0]) / 2, (BASE[1] + top[1]) / 2), top], col, 9, ink=tone(col), ink_w=5)
        leaf(ic, (x0 * 2 + top[0]) / 3, (BASE[1] * 2 + top[1]) / 3, -math.pi / 2 + f * 1.6 + (0.7 if i % 2 else -0.7), h * 0.4, 18, col, vein=False)
        if s == 3:
            if droop:
                head = ellipse(top[0] + 26 * (1 if f >= 0 else -1), top[1] + 40, 22, 54, rot=0.5 * (1 if f >= 0 else -1))
            else:
                head = ellipse(top[0], top[1] - 30, 20, 56, rot=f * 0.5)
            ic.drawn(head, ripe, edge_w=6, highlight=0.2)
        elif s == 2:
            ic.drawn(ellipse(top[0], top[1] - 18, 12, 32, rot=f * 0.5), YOUNG, edge_w=5, highlight=0)


def rice(ic, s):
    wheat(ic, s, ripe=(240, 210, 120), droop=True, green=(60, 160, 120))


def turnip(ic, s):
    sz = (0, 0.45, 0.75, 1.0)[s]
    if s == 3:
        ic.drawn(ellipse(256, 440, 100, 70), (238, 232, 236), edge=(120, 60, 110), edge_w=8)
        ic.drawn(ellipse(256, 410, 98, 40), (170, 70, 160), shade=0.2, edge=(90, 30, 90), edge_w=6)
    for i, a in enumerate((-2.4, -1.9, -1.55, -1.2, -0.7)):
        if s == 1 and i in (0, 4):
            continue
        leaf(ic, 256, 420 if s == 3 else BASE[1] - 4, a, 300 * sz, 60 * sz, (GREEN, YOUNG)[i % 2])


def melon(ic, s):
    ic.line([(80, 470), (180, 440), (300, 470), (440, 445)], (70, 140, 50), 10, ink=(30, 70, 30), ink_w=5)
    for x, a in ((120, -2.0), (220, -1.2), (360, -1.9), (430, -1.0))[: (0, 2, 3, 4)[s]]:
        ic.drawn(frilly(x + math.cos(a) * 50, 450 + math.sin(a) * 50, 60, 44, rot=a, bumps=5), GREEN, edge_w=6)
    if s == 2:
        for x in (190, 320):
            ic.drawn(circle(x, 420, 22), (250, 220, 60), edge_w=5, highlight=0)
    if s == 3:
        ic.drawn(ellipse(256, 410, 160, 100), (130, 190, 70), rim_color=(220, 250, 170), edge_w=9)
        for dx in (-100, -50, 0, 50, 100):
            ic.line([(256 + dx * 0.5, 315), (256 + dx, 410), (256 + dx * 0.5, 505)], (60, 120, 40), 7)


def chili(ic, s):
    h = (0, 160, 300, 380)[s]
    ic.line([(256, BASE[1]), (250, BASE[1] - h * 0.5), (262, BASE[1] - h)], (70, 120, 40), 12, ink=(30, 60, 20), ink_w=5)
    k = (0, 0.5, 0.8, 1.0)[s]
    spots = [(-1, 0.35), (1, 0.45), (-1, 0.65), (1, 0.75), (-1, 0.92), (1, 1.0)][: (0, 2, 4, 6)[s]]
    for side, t in spots:
        y = BASE[1] - h * t
        leaf(ic, 256, y, -math.pi / 2 + side * 1.1, 150 * k, 44 * k, GREEN)
    if s == 2:
        for side, t in ((-1, 0.55), (1, 0.85)):
            ic.drawn(circle(256 + side * 70, BASE[1] - h * t, 16), (250, 250, 240), edge_w=4, highlight=0)
    if s == 3:
        for side, t in ((-1, 0.5), (1, 0.6), (-1, 0.8), (1, 0.88), (1, 0.3)):
            x, y = 256 + side * 90, BASE[1] - h * t
            pod = [(x - 14, y), (x + 14, y), (x + 8, y + 70), (x + side * 10, y + 96), (x - 6, y + 66)]
            ic.drawn(pod, (230, 40, 40), edge_w=6, rim_color=(255, 150, 130))
            ic.line([(x, y - 12), (x, y + 2)], (60, 120, 40), 6)


def kale(ic, s):
    k = (0, 0.55, 0.78, 1.0)[s]
    ic.line([(256, BASE[1]), (256, BASE[1] - 140 * k)], (120, 160, 120), 14, ink=(50, 80, 60), ink_w=5)
    col = (90, 150, 140)
    for i, a in enumerate((-2.6, -2.1, -1.57, -1.05, -0.55)):
        if s == 1 and i in (0, 4):
            continue
        x = 256 + math.cos(a) * 110 * k
        y = BASE[1] - 120 * k + math.sin(a) * 140 * k
        ic.drawn(frilly(x, y, 110 * k, 60 * k, rot=a, bumps=11), mix(col, (60, 110, 160), (i % 2) * 0.4), edge_w=6, rim_color=lighter(col, 0.5) if s == 3 else None)
    if s == 3:
        ic.sparkle(120, 140, 24, (220, 240, 255))
        ic.sparkle(400, 200, 18, (220, 240, 255))


def glowcap(ic, s):
    caps = [(-90, 0.6, 0.7), (0, 1.0, 1.0), (95, 0.75, 0.8), (-40, 0.45, 0.5), (55, 0.4, 0.45)][: (0, 5, 4, 5)[s]]
    k = (0, 0.25, 0.6, 1.0)[s]
    glow = (120, 240, 255)
    for dx, h, r in caps:
        x = 256 + dx
        top = BASE[1] - 230 * h * k - 20
        if s == 3:
            ic.glow((x, top, 90 * r), glow, radius=26, strength=0.7)
        ic.drawn(rect(x - 12 * r - 4, top, x + 12 * r + 4, BASE[1]), (230, 226, 210), highlight=0, edge_w=5)
        cap = [(x - 90 * r * k - 10, top + 10)] + [(x + math.cos(a) * (90 * r * k + 10), top + 10 - math.sin(a) * (60 * r * k + 10)) for a in [math.pi * i / 16 for i in range(17)]][::-1]
        ic.drawn(cap, (70, 200, 230) if s == 3 else (120, 150, 170), rim_color=(200, 255, 255) if s == 3 else None, edge_w=6)
        if s == 3:
            for j in (-0.4, 0.1, 0.5):
                ic.dot(x + j * 70 * r, top - 16 * r, 7, (230, 255, 255))


CROPS = {
    "wheat": (wheat, (236, 200, 110)),
    "turnip": (turnip, (180, 90, 160)),
    "sun_melon": (melon, (240, 230, 190)),
    "chili": (chili, (240, 220, 160)),
    "rice": (rice, (240, 236, 220)),
    "frost_kale": (kale, (60, 50, 40)),
    "glowcap": (glowcap, (200, 220, 230)),
}


# ---------- produce, tools ------------------------------------------------------------

def p_wheat(ic):
    for i in range(7):
        f = (i - 3) / 3
        ic.line([(256 + f * 30, 470), (256 + f * 130, 150)], (220, 180, 70), 10, ink=(130, 90, 30), ink_w=5)
        ic.drawn(ellipse(256 + f * 140, 120, 24, 60, rot=f * 0.4), GOLD, edge_w=6)
    ic.drawn(rect(186, 320, 326, 360), (200, 70, 50), highlight=0, edge_w=6)


def p_turnip(ic):
    for a in (-2.0, -1.57, -1.1):
        leaf(ic, 256, 200, a, 170, 40, GREEN)
    ic.drawn(ellipse(256, 320, 140, 130), (240, 234, 238), edge=(120, 60, 110), rim_color=(255, 255, 255))
    ic.drawn(ellipse(256, 250, 136, 60), (170, 70, 160), shade=0.2, edge=(90, 30, 90), edge_w=6)
    ic.line([(256, 450), (250, 500)], (200, 180, 190), 6)


def p_melon(ic):
    ic.drawn(ellipse(256, 280, 200, 150), (130, 190, 70), rim_color=(220, 250, 170), edge_w=9)
    for dx in (-140, -70, 0, 70, 140):
        ic.line([(256 + dx * 0.5, 135), (256 + dx, 280), (256 + dx * 0.5, 425)], (60, 120, 40), 8)
    ic.line([(256, 135), (276, 90)], (90, 70, 40), 10)


def p_chili(ic):
    pod = [(150, 140), (230, 120), (330, 230), (400, 400), (380, 430), (280, 330), (180, 230)]
    ic.drawn(pod, (230, 40, 40), rim_color=(255, 150, 130), edge_w=9)
    ic.drawn([(120, 110), (190, 90), (200, 160), (140, 160)], (70, 150, 50), highlight=0, edge_w=6)
    ic.line([(150, 110), (110, 60)], (70, 120, 40), 10)


def p_rice(ic):
    ic.drawn([(110, 250), (402, 250), (370, 420), (142, 420)], (180, 110, 70), edge_w=9)
    ic.drawn(ellipse(256, 250, 150, 50), (246, 242, 230), highlight=0, edge=(150, 140, 120), edge_w=6)
    ic.drawn(ellipse(256, 236, 120, 46), (255, 252, 242), rim_color=(255, 255, 255), edge=(170, 160, 140), edge_w=4)
    for i in range(9):
        ic.dot(180 + (i * 47) % 150, 220 + (i * 29) % 30, 5, (225, 218, 200))
    ic.line([(330, 150), (250, 236)], (150, 110, 60), 8)
    ic.line([(350, 160), (270, 240)], (150, 110, 60), 8)


def p_kale(ic):
    ic.drawn(frilly(256, 240, 170, 130, rot=-0.5, bumps=13), (90, 150, 140), rim_color=(170, 220, 210), edge_w=8)
    ic.line([(140, 440), (256, 240), (350, 140)], (190, 220, 200), 10)


def p_glowcap(ic):
    ic.glow((256, 220, 160), (120, 240, 255), radius=34, strength=0.7)
    ic.drawn(rect(220, 230, 292, 450), (230, 226, 210), edge_w=7)
    cap = [(60, 260)] + [(256 + math.cos(a) * 196, 260 - math.sin(a) * 160) for a in [math.pi * i / 24 for i in range(25)]][::-1]
    ic.drawn(cap, (70, 200, 230), rim_color=(200, 255, 255), edge_w=8)
    for x, y in ((180, 180), (300, 150), (360, 220), (250, 220)):
        ic.dot(x, y, 12, (230, 255, 255))


def t_hoe(ic):
    ic.line([(140, 470), (380, 100)], (170, 120, 70), 20, ink=(80, 50, 30), ink_w=8)
    ic.drawn([(330, 80), (450, 110), (440, 160), (360, 150)], (150, 160, 175), edge=(40, 46, 60), edge_w=8)


def t_can(ic):
    ic.drawn([(300, 220), (450, 120), (470, 150), (330, 270)], (70, 150, 200), edge_w=7)
    ic.drawn(roundbody := [(140, 200), (330, 200), (340, 430), (130, 430)], (70, 150, 200), rim_color=(170, 220, 240), edge_w=9)
    ic.line([(150, 210), (130, 150), (230, 120), (320, 150), (320, 210)], (40, 90, 130), 12)
    for x, y in ((470, 190), (490, 240), (460, 260)):
        ic.drawn(ellipse(x, y, 10, 18, rot=0.4), (140, 210, 255), hatch=0.3, shade=0, highlight=0, edge_w=3)


def t_seeds(ic):
    sack = [(170, 180), (340, 180), (400, 420), (110, 420)]
    ic.drawn(sack, (210, 180, 120), edge_w=9)
    ic.drawn([(190, 120), (320, 120), (340, 180), (170, 180)], (190, 160, 100), highlight=0, edge_w=7)
    ic.line([(170, 185), (340, 185)], (140, 60, 40), 10)
    for x, y in ((220, 300), (290, 280), (260, 350), (320, 360), (190, 370)):
        ic.drawn(ellipse(x, y, 16, 11, rot=0.5), (120, 80, 40), hatch=0.3, shade=0, highlight=0, edge_w=4)


def t_fert(ic):
    sack = [(150, 160), (360, 160), (390, 440), (120, 440)]
    ic.drawn(sack, (130, 100, 70), edge_w=9)
    ic.drawn(ellipse(255, 170, 105, 30), (70, 50, 36), highlight=0, edge_w=6)
    leaf(ic, 255, 360, -1.0, 110, 30, GREEN)
    leaf(ic, 255, 360, -2.1, 110, 30, YOUNG)
    ic.sparkle(410, 120, 26)


def t_sickle(ic):
    blade = [(200, 300)] + [(260 + math.cos(a) * 170, 250 - math.sin(a) * 170) for a in [math.pi * (0.1 + 0.8 * i / 20) for i in range(21)]] + \
            [(260 + math.cos(a) * 130, 270 - math.sin(a) * 120) for a in [math.pi * (0.9 - 0.8 * i / 20) for i in range(21)]]
    ic.drawn(blade, (180, 190, 205), rim_color=(240, 246, 255), edge=(40, 46, 60), edge_w=8)
    ic.line([(200, 300), (110, 460)], (170, 120, 70), 22, ink=(80, 50, 30), ink_w=8)


def t_lantern(ic):
    ic.glow((256, 290, 130), (255, 190, 90), radius=40, strength=0.8)
    ic.drawn([(196, 190), (316, 190), (330, 400), (182, 400)], (255, 210, 110), rim_color=(255, 240, 200), edge=(90, 60, 30), edge_w=9)
    ic.drawn(rect(176, 160, 336, 196), (60, 56, 60), highlight=0, edge=(20, 20, 24), edge_w=6)
    ic.drawn(rect(176, 396, 336, 430), (60, 56, 60), highlight=0, edge=(20, 20, 24), edge_w=6)
    ic.line([(206, 160), (256, 100), (306, 160)], (60, 56, 60), 9)
    ic.drawn(ellipse(256, 320, 22, 40), (255, 140, 40), hatch=0.4, shade=0, highlight=0, edge_w=4)


# ---------- soil and ground, full squares for the hex tops ------------------------------

def soil(ic, col, furrows=None, flecks=None, wet=False):
    sq = rect(-40, -40, 552, 552)
    ic.flat(sq, col, wobble=0)
    ic.scribble(sq, col, var=0.28, angle=20, density=1.0, length=(40, 110), spread=16)
    ic.scribble(sq, col, var=0.18, angle=80, density=0.4, length=(30, 80), spread=18)
    if furrows:
        for y in range(30, 512, 86):
            ic.line([(-20, y), (540, y + 10)], darker(col, 0.35), 22, amp=4)
            ic.line([(-20, y + 30), (540, y + 40)], lighter(col, 0.18), 12, amp=4)
    if flecks:
        for i in range(70):
            x, y = (i * 137) % 512, (i * 211) % 512
            ic.dot(x, y, 4 + i % 4, flecks)
    if wet:
        for i in range(10):
            x, y = (i * 173 + 40) % 480, (i * 97 + 60) % 480
            ic.dot(x, y, 10, lighter(col, 0.35))


def grass_tex(ic):
    soil(ic, (96, 150, 64))
    for i in range(120):
        x, y = (i * 131) % 520, (i * 197) % 520
        ic.line([(x, y), (x + 6, y - 26)], (140, 200, 90) if i % 2 else (60, 110, 46), 5)


GROUND = {
    "ground_grass": grass_tex,
    "ground_tilled": lambda ic: soil(ic, (140, 100, 64), furrows=True),
    "ground_wet": lambda ic: soil(ic, (86, 60, 42), furrows=True, wet=True),
    "ground_fertile": lambda ic: soil(ic, (100, 70, 48), furrows=True, flecks=(40, 30, 24)),
    "ground_sand": lambda ic: soil(ic, (222, 196, 140)),
    "ground_regolith": lambda ic: soil(ic, (130, 130, 136), flecks=(90, 90, 96)),
}

ICONS = {
    "produce_wheat": p_wheat, "produce_turnip": p_turnip, "produce_sun_melon": p_melon, "produce_chili": p_chili,
    "produce_rice": p_rice, "produce_frost_kale": p_kale, "produce_glowcap": p_glowcap,
    "tool_hoe": t_hoe, "tool_can": t_can, "tool_seeds": t_seeds, "tool_fert": t_fert, "tool_sickle": t_sickle, "tool_lantern": t_lantern,
}


def save_square(ic, path):
    """A ground texture keeps its full square: skip the icon's halo fade."""
    from PIL import Image
    from PIL.PngImagePlugin import PngInfo
    meta = PngInfo()
    meta.add_text("source", "armory-crayon")
    ic.img.resize((128, 128), Image.LANCZOS).convert("RGB").save(path, pnginfo=meta)


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    written = []
    seed = 500
    for name, (fn, seed_col) in CROPS.items():
        for s in range(4):
            ic = Icon(seed=seed)
            seed += 1
            if s == 0:
                mound(ic, seed_col)
            else:
                fn(ic, s)
            p = os.path.join(OUT, f"{name}_{s}.png")
            ic.save(p)
            written.append(p)
    for name, fn in ICONS.items():
        ic = Icon(seed=seed)
        seed += 1
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        ic.save(p)
        written.append(p)
    for name, fn in GROUND.items():
        ic = Icon(seed=seed)
        seed += 1
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        save_square(ic, p)
        written.append(p)
    print("wrote", len(written), "pngs to", OUT)
    if "--sheet" in argv:
        import sheet
        sheet.main([argv[argv.index("--sheet") + 1]] + written + ["--ref", "vitality", "--ref", "power output", "--ref", "plating"])


if __name__ == "__main__":
    main(sys.argv[1:])
