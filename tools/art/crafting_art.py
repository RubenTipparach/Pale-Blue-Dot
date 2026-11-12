"""Item, resource and station icons for the crafting mockup (docs/mockups/crafting.html).

Painted in code with the hand-drawn-illustration skill's crayon kit, in the manner of
the owner's friend's upgrade icons: flat saturated blocks, loose hatching in each
shape's own tones, form-following crescents, highlight dabs, edges in a deep tone of
the same hue, light from the upper left. Nothing here is drawn by hand.

    python tools/art/crafting_art.py            # writes docs/mockups/crafting/*.png
    python tools/art/crafting_art.py --sheet    # also a contact sheet beside the friend's icons
"""

import math
import os
import random
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)
from crayon import (INK, Icon, circle, darker, ellipse, lighter, mix, rect,  # noqa: E402
                    rotate, roundrect, scale, star, tone, translate)

OUT = os.path.join(ROOT, "docs", "mockups", "crafting")


class FillIcon(Icon):
    """The kit's Icon, framed to its subject: the friend's drawings fill most of the
    square, so the drawing is cropped to what was painted (plus a margin) before it
    is shrunk to 128. The kit itself is left as it is."""

    margin = 22

    def final(self):
        from PIL import Image, ImageChops, ImageDraw
        import crayon
        box = self.img.getchannel("A").point(lambda v: 255 if v > 40 else 0).getbbox() or (0, 0, crayon.S, crayon.S)
        x0, y0, x1, y1 = box
        side = max(x1 - x0, y1 - y0) + 2 * self.margin
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        crop = (int(cx - side / 2), int(cy - side / 2), int(cx + side / 2), int(cy + side / 2))
        n = crayon.OUT
        im = self.img.crop(crop).resize((n, n), Image.LANCZOS)
        a = im.getchannel("A").point(lambda v: 0 if v < 40 else (255 if v > 200 else v))
        im.putalpha(a)
        halo = self.halo.crop(crop).resize((n, n), Image.LANCZOS)
        fade = Image.new("L", (n, n), 0)
        fd = ImageDraw.Draw(fade)
        band = max(6, n // 8)
        for i in range(band):
            fd.rectangle((i, i, n - 1 - i, n - 1 - i), outline=int(255 * (i / band) ** 1.5))
        fd.rectangle((band, band, n - 1 - band, n - 1 - band), fill=255)
        halo.putalpha(ImageChops.multiply(halo.getchannel("A"), fade))
        halo.alpha_composite(im)
        return halo


# ---------- shared shapes -------------------------------------------------------------

def lump(cx, cy, r, seed, n=11, jag=0.22, squash=0.82):
    rnd = random.Random(seed)
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n + rnd.uniform(-0.12, 0.12)
        rr = r * (1 + rnd.uniform(-jag, jag))
        pts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr * squash))
    return pts


def rock(ic, cx, cy, r, color, seed, flecks=None, fleck_n=6):
    poly = lump(cx, cy, r, seed)
    ic.drawn(poly, color, shade=0.26, edge_w=9)
    # facet lines, the way a hand marks a chunk of stone
    rnd = random.Random(seed + 5)
    for _ in range(2):
        a = rnd.uniform(0, 6.28)
        x0, y0 = cx + math.cos(a) * r * 0.15, cy + math.sin(a) * r * 0.1
        ic.line([(x0, y0), (x0 + math.cos(a) * r * 0.5, y0 + math.sin(a) * r * 0.4)], darker(color, 0.45), 5)
    if flecks:
        for k in range(fleck_n):
            a = 2 * math.pi * k / fleck_n + rnd.uniform(-0.3, 0.3)
            d = r * rnd.uniform(0.15, 0.62)
            fx, fy = cx + math.cos(a) * d, cy + math.sin(a) * d * 0.8
            fr = r * rnd.uniform(0.11, 0.19)
            ic.drawn(lump(fx, fy, fr, seed + k * 13, n=7, jag=0.3, squash=0.9), flecks,
                     hatch=0.6, shade=0.2, highlight=0.25, edge_w=5)
    return poly


def ingot(ic, color, cx=256, cy=290, w=300, h=96, d=120, edge=None):
    """An ingot in three quarter view: the top, the front and the right end."""
    hw = w / 2
    top = [(cx - hw + 40, cy - h), (cx + hw - 40, cy - h), (cx + hw - 40 + d * 0.55, cy - h - d * 0.5),
           (cx - hw + 40 + d * 0.55, cy - h - d * 0.5)]
    front = [(cx - hw, cy), (cx + hw, cy), (cx + hw - 40, cy - h), (cx - hw + 40, cy - h)]
    side = [(cx + hw, cy), (cx + hw + d * 0.55, cy - d * 0.5), (cx + hw - 40 + d * 0.55, cy - h - d * 0.5),
            (cx + hw - 40, cy - h)]
    ic.drawn(translate(side, 0, 0), darker(color, 0.28), highlight=0, shade=0.2, edge=edge or tone(color), edge_w=8)
    ic.drawn(front, color, edge=edge or tone(color), edge_w=8)
    ic.drawn(top, lighter(color, 0.3), rim_color=lighter(color, 0.7), edge=edge or tone(color), edge_w=8)
    return top, front, side


def pick(ic, head, handle=(150, 100, 55), seed=0):
    """A pickaxe: the handle on the diagonal, the head across its top."""
    sh = rotate(rect(238, 100, 274, 470), math.radians(-35), (256, 285))
    ic.drawn(sh, handle, highlight=0.12, edge_w=8)
    grip = rotate(rect(234, 380, 278, 470), math.radians(-35), (256, 285))
    ic.drawn(grip, (120, 60, 50), highlight=0.1, edge_w=7)
    hp = [(56, 226), (140, 128), (256, 96), (372, 128), (456, 226), (380, 186), (300, 168), (300, 200),
          (212, 200), (212, 168), (132, 186)]
    hp = rotate(hp, math.radians(-35), (256, 200))
    hp = translate(hp, 30, 8)
    ic.drawn(hp, head, rim_color=lighter(head, 0.6), edge_w=11)
    tip = rotate([(456, 226)], math.radians(-35), (256, 200))[0]
    ic.sparkle(tip[0] + 30, tip[1] + 8, 26, (255, 240, 170))
    ic.speed(tip[0] + 10, tip[1] + 60, 50, 80, (240, 240, 240), 6)


# ---------- raw resources ---------------------------------------------------------------

def stone(ic):
    rock(ic, 250, 270, 170, (138, 146, 160), 3)
    ic.sparkle(400, 120, 26, (230, 236, 255))


def flint(ic):
    shard = [(150, 120), (300, 70), (410, 200), (370, 380), (220, 440), (110, 300)]
    ic.drawn(shard, (70, 78, 105), shade=0.25, edge_w=9)
    facet = [(170, 140), (290, 98), (330, 230), (200, 270)]
    ic.drawn(facet, (120, 132, 170), shade=0, highlight=0.3, edge=None)
    ic.line([(300, 100), (340, 240), (370, 370)], (40, 44, 64), 5)
    ic.sparkle(120, 110, 24, (220, 236, 255))


def wood_log(ic):
    body = [(100, 200), (380, 120), (420, 270), (140, 350)]
    ic.drawn(body, (150, 96, 52), edge_w=9)
    for k in range(3):
        y = 175 + k * 50
        ic.line([(130, y + 40), (390, y - 32)], (110, 66, 36), 5)
    end = ellipse(400, 196, 62, 80, rot=math.radians(-16))
    ic.drawn(end, (232, 190, 120), shade=0.2, edge=(130, 80, 40), edge_w=9)
    for r in (40, 22):
        ring = ellipse(400, 196, r * 0.8, r, rot=math.radians(-16))
        ic.outline(ring, (176, 120, 66), 5, passes=1)
    ic.drawn(ellipse(140, 360, 40, 30), (70, 160, 70), shade=0.2, edge_w=6)  # a leaf on the cut


def fibre(ic):
    rnd = random.Random(4)
    for k in range(9):
        x = 170 + k * 20 + rnd.uniform(-6, 6)
        ic.line([(256 + (x - 256) * 0.2, 470), (256, 300), (x + (x - 256) * 1.1, 60 + rnd.uniform(0, 40))],
                mix((150, 200, 70), (220, 210, 90), k / 9), 14, ink=(50, 80, 30), ink_w=8)
    band = rect(196, 280, 316, 330)
    ic.drawn(band, (200, 150, 80), highlight=0.1, edge_w=7)


def clay(ic):
    body = lump(256, 300, 170, 9, n=12, jag=0.08, squash=0.66)
    ic.drawn(body, (204, 112, 72), rim_color=(240, 170, 130), edge_w=9)
    ic.drawn(ellipse(300, 270, 44, 26, rot=0.3), (170, 86, 56), highlight=0, shade=0, edge=(140, 70, 44), edge_w=5)


def ore(color, seed):
    def f(ic):
        rock(ic, 256, 270, 175, (112, 112, 124), seed, flecks=color, fleck_n=7)
    return f


def coal(ic):
    for i, (x, y, r) in enumerate(((190, 310, 120), (330, 300, 110), (260, 200, 110))):
        ic.drawn(lump(x, y, r, 30 + i), (52, 54, 66), shade=0.25, rim_color=(110, 130, 170), edge=(16, 16, 24), edge_w=8)
    ic.sparkle(380, 130, 22, (170, 200, 255))


def sand(ic):
    pile = [(70, 420), (140, 300), (230, 200), (290, 190), (370, 280), (450, 420)]
    ic.drawn(pile, (232, 200, 120), shade=0.22, edge=(170, 120, 50), edge_w=9)
    rnd = random.Random(7)
    for _ in range(18):
        ic.dot(rnd.uniform(140, 380), rnd.uniform(270, 400), 5, (190, 150, 80))


# ---------- worked materials ------------------------------------------------------------

def planks(ic):
    for ang, col in ((-28, (190, 132, 72)), (14, (214, 156, 90))):
        p = rotate(rect(80, 220, 432, 300), math.radians(ang), (256, 260))
        ic.drawn(p, col, edge_w=9)
        ic.line(rotate([(110, 260), (400, 260)], math.radians(ang), (256, 260)), darker(col, 0.3), 4)
        for x in (110, 400):
            c = rotate([(x, 245)], math.radians(ang), (256, 260))[0]
            ic.dot(c[0], c[1], 7, (70, 60, 60))


def bar(color):
    def f(ic):
        ingot(ic, color)
        ic.sparkle(120, 120, 26, (255, 245, 200))
    return f


def glass(ic):
    pane = [(130, 90), (390, 120), (380, 420), (120, 390)]
    ic.drawn(pane, (110, 210, 230), rim_color=(220, 250, 255), edge=(40, 110, 150), edge_w=9)
    for a, b in (((180, 150), (250, 230)), ((220, 140), (330, 270)), ((290, 300), (330, 350))):
        ic.line([a, b], (235, 252, 255), 9)
    ic.sparkle(400, 100, 28, (240, 250, 255))


def brick(ic):
    rows = [(60, 320, 250, 420), (262, 320, 452, 420), (160, 210, 350, 310), (110, 100, 300, 200)]
    for k, (x0, y0, x1, y1) in enumerate(rows):
        ic.drawn(roundrect(x0, y0, x1, y1, 12), (196, 78, 54), shade=0.25, rim_color=(236, 130, 100),
                 edge=(110, 40, 30), edge_w=10)
        for j in range(2):
            ic.dot(x0 + 40 + j * 90, y0 + 50, 7, (150, 56, 40))


def cloth(ic):
    roll = [(90, 170), (330, 170), (330, 360), (90, 360)]
    ic.drawn(roll, (70, 140, 200), edge_w=9)
    for y in (210, 250, 290, 330):
        ic.line([(100, y), (320, y)], (110, 180, 230), 5)
    end = ellipse(330, 265, 58, 96)
    ic.drawn(end, (100, 170, 225), shade=0.15, edge=(30, 70, 120), edge_w=9)
    ic.outline(ellipse(330, 265, 18, 32), (40, 80, 130), 6, passes=1)
    tail = [(90, 330), (60, 430), (180, 440), (200, 360)]
    ic.drawn(tail, (80, 150, 210), edge_w=8)


def rope(ic):
    for k, r in enumerate((150, 110, 70)):
        ic.outline(ellipse(256, 270, r, r * 0.72), (120, 80, 40), 30, passes=1)
        ic.outline(ellipse(256, 270, r, r * 0.72), (216, 170, 100), 20, passes=1)
    ic.line([(380, 300), (440, 420)], (216, 170, 100), 20, ink=(120, 80, 40), ink_w=10)


# ---------- advanced and alien ---------------------------------------------------------

def circuit(ic):
    board = roundrect(80, 110, 432, 400, 24)
    ic.drawn(board, (50, 160, 90), edge=(20, 70, 40), edge_w=9)
    for pts in (((110, 160), (200, 160), (230, 200)), ((110, 350), (190, 350), (230, 300)),
                ((400, 150), (330, 150), (300, 200)), ((400, 360), (320, 360), (290, 300))):
        ic.line(list(pts), (240, 200, 70), 9)
        ic.dot(pts[0][0], pts[0][1], 12, (250, 220, 90), ink=(80, 60, 20))
    chip = rect(200, 190, 320, 310)
    ic.drawn(chip, (46, 50, 62), highlight=0.1, edge=INK, edge_w=8)
    for k in range(4):
        y = 205 + k * 28
        ic.line([(180, y), (200, y)], (200, 200, 210), 6)
        ic.line([(320, y), (340, y)], (200, 200, 210), 6)
    ic.dot(230, 220, 9, (120, 230, 255))


def vial(ic):
    neck = rect(220, 70, 292, 200)
    ic.drawn(neck, (190, 220, 230), shade=0.1, edge=(60, 90, 110), edge_w=8)
    body = [(220, 190), (292, 190), (420, 420), (92, 420)]
    ic.drawn(body, (190, 225, 235), shade=0.12, edge=(60, 90, 110), edge_w=9)
    liquid = [(160, 300), (352, 300), (410, 414), (102, 414)]
    ic.glow((256, 360, 120), (120, 255, 120), 40, 0.5)
    ic.drawn(liquid, (110, 220, 80), rim_color=(190, 255, 150), edge=None)
    for x, y, r in ((200, 360, 14), (290, 330, 10), (250, 260, 9), (270, 210, 7)):
        ic.dot(x, y, r, (210, 255, 190), ink=(50, 120, 40))
    ic.drawn(rect(206, 52, 306, 86), (150, 100, 60), highlight=0, edge_w=7)


def alloy(ic):
    plate = [(110, 130), (400, 100), (420, 380), (100, 410)]
    ic.drawn(plate, (120, 150, 190), rim_color=(200, 220, 240), edge=(30, 40, 70), edge_w=10)
    for x, y in ((140, 160), (370, 135), (390, 350), (135, 375)):
        ic.dot(x, y, 10, (180, 190, 205), ink=(20, 24, 34))
    ic.line([(160, 300), (260, 220), (360, 300)], (250, 205, 40), 8)


def polymer(ic):
    drop = [(256, 70)] + [(256 + 140 * math.sin(t), 300 - 140 * math.cos(t)) for t in
                          [math.pi * 0.25 + i * math.pi * 1.5 / 20 for i in range(21)]]
    ic.glow((256, 300, 110), (255, 170, 60), 40, 0.45)
    ic.drawn(drop, (240, 150, 40), rim_color=(255, 220, 150), edge=(150, 70, 20), edge_w=9)
    ic.drawn(ellipse(210, 270, 26, 50, rot=0.4), (255, 230, 170), shade=0, highlight=0, edge=None)


def frost_crystal(ic):
    ic.glow((256, 300, 140), (120, 220, 255), 46, 0.5)
    for ang, h, col in ((-30, 300, (110, 200, 245)), (25, 260, (150, 225, 255)), (0, 360, (90, 180, 240))):
        p = [(-34, 0), (-30, -h * 0.75), (0, -h), (30, -h * 0.75), (34, 0)]
        p = translate(rotate(p, math.radians(ang)), 256, 440)
        ic.drawn(p, col, rim_color=(230, 250, 255), edge=(30, 80, 140), edge_w=8)
    ic.sparkle(120, 150, 26, (230, 250, 255))
    ic.sparkle(390, 110, 20, (230, 250, 255))


def glowcap(ic):
    ic.glow((256, 220, 150), (200, 120, 255), 50, 0.55)
    stem = [(224, 250), (290, 250), (300, 450), (210, 450)]
    ic.drawn(stem, (230, 220, 200), edge=(120, 100, 90), edge_w=8)
    cap = [(256 + 180 * math.cos(t), 260 - 160 * math.sin(t)) for t in [i * math.pi / 24 for i in range(25)]]
    ic.drawn(cap, (170, 80, 230), rim_color=(230, 170, 255), edge=(70, 20, 110), edge_w=9)
    for x, y, r in ((190, 200, 16), (270, 150, 20), (330, 215, 14)):
        ic.dot(x, y, r, (180, 255, 230), ink=(60, 30, 90))
    for x, y in ((110, 120), (410, 90), (420, 330)):
        ic.dot(x, y, 8, (190, 255, 230))


def magma_glass(ic):
    body = lump(256, 280, 175, 41, n=9, jag=0.28, squash=0.85)
    ic.glow((256, 280, 160), (255, 120, 40), 40, 0.45)
    ic.drawn(body, (44, 34, 48), shade=0.2, rim_color=(110, 90, 130), edge=(14, 10, 18), edge_w=9)
    ic.zap([(150, 200), (250, 280), (300, 230), (380, 330)], (255, 140, 40), 10, jag=12, seed=4)
    ic.zap([(250, 280), (230, 390)], (255, 190, 60), 8, jag=10, seed=6)


def star_iron(ic):
    rock(ic, 256, 280, 170, (70, 66, 96), 51, flecks=(170, 120, 255), fleck_n=5)
    ic.sparkle(110, 110, 30, (220, 200, 255))
    ic.sparkle(400, 140, 22, (220, 200, 255))
    ic.speed(390, 60, 90, 135, (220, 200, 255), 7)


# ---------- stations --------------------------------------------------------------------

def workbench(ic):
    for x in (100, 372):
        ic.drawn(rect(x, 260, x + 44, 460), (130, 84, 46), highlight=0.1, edge_w=10)
    ic.drawn(rect(110, 370, 400, 396), (120, 78, 44), highlight=0, edge_w=8)
    top = [(50, 220), (462, 220), (446, 280), (66, 280)]
    ic.drawn(top, (204, 146, 84), rim_color=(244, 206, 146), edge_w=11)
    ic.line([(70, 250), (440, 250)], (160, 108, 60), 5)
    # a hammer leaning on the bench and a saw lying on it
    ic.drawn(rotate(rect(150, 40, 184, 220), math.radians(18), (167, 210)), (170, 116, 62), highlight=0.1, edge_w=8)
    head = rotate(rect(96, 30, 236, 92), math.radians(18), (167, 210))
    ic.drawn(head, (150, 160, 182), rim_color=(220, 228, 240), edge_w=10)
    saw = [(250, 214), (430, 150), (446, 190), (270, 222)]
    ic.drawn(saw, (210, 220, 232), shade=0.1, edge=(60, 70, 90), edge_w=8)
    for k in range(6):
        x, y = 280 + k * 26, 216 - k * 9
        ic.line([(x, y), (x + 8, y + 12)], (60, 70, 90), 4)
    ic.drawn(roundrect(420, 120, 486, 196, 16), (210, 64, 52), edge_w=9)
    ic.sparkle(60, 120, 26, (255, 236, 140))


def forge(ic):
    body = roundrect(90, 150, 422, 450, 30)
    ic.drawn(body, (120, 116, 124), shade=0.25, edge_w=10)
    for y in (220, 300, 380):
        ic.line([(100, y), (412, y)], (80, 78, 88), 5)
    ic.drawn(rect(220, 60, 292, 160), (100, 96, 104), edge_w=8)
    mouth = [(170, 440), (170, 330)] + [(256 - 86 * math.cos(t), 330 - 70 * math.sin(t)) for t in
                                         [i * math.pi / 12 for i in range(13)]] + [(342, 440)]
    ic.drawn(mouth, (40, 24, 20), highlight=0, shade=0, edge=INK, edge_w=8)
    ic.glow((256, 390, 90), (255, 140, 40), 30, 0.6)
    fl = star(256, 390, 70, 30, 6, jag=0.3, seed=3)
    ic.drawn(fl, (255, 140, 30), rim_color=(255, 220, 90), edge=(170, 60, 20), edge_w=6)
    ic.drawn(star(256, 400, 34, 14, 5, seed=4), (255, 230, 110), shade=0, highlight=0, edge=None)


def kiln(ic):
    dome = [(256 + 170 * math.cos(t), 420 - 300 * math.sin(t)) for t in [i * math.pi / 24 for i in range(25)]]
    ic.drawn(dome, (200, 120, 80), shade=0.25, edge_w=10)
    for y in (180, 260, 340):
        ic.line([(110 + (420 - y) * 0.05, y), (400 - (420 - y) * 0.05, y)], (160, 86, 56), 5)
    door = [(200, 420)] + [(256 - 56 * math.cos(t), 330 - 50 * math.sin(t)) for t in
                           [i * math.pi / 10 for i in range(11)]] + [(312, 420)]
    ic.glow((256, 380, 70), (255, 170, 60), 26, 0.6)
    ic.drawn(door, (255, 170, 60), rim_color=(255, 230, 140), edge=(120, 50, 20), edge_w=7)
    ic.drawn(rect(236, 80, 276, 140), (150, 90, 60), edge_w=7)


def loom(ic):
    for x in (110, 380):
        ic.drawn(rect(x, 90, x + 30, 450), (160, 106, 60), highlight=0.1, edge_w=8)
    for y in (100, 400):
        ic.drawn(rect(90, y, 430, y + 32), (190, 130, 74), edge_w=8)
    for k in range(9):
        x = 160 + k * 25
        ic.line([(x, 132), (x, 400)], (240, 230, 210), 5)
    cloth = rect(150, 260, 372, 398)
    ic.drawn(cloth, (210, 70, 80), edge=(110, 30, 40), edge_w=7)
    for y in (290, 330, 370):
        ic.line([(152, y), (370, y)], (240, 140, 140), 5)
    ic.drawn(ellipse(256, 240, 110, 14), (240, 200, 90), highlight=0, edge_w=6)


def chem_lab(ic):
    ic.drawn(rect(70, 400, 442, 440), (120, 90, 70), edge_w=8)
    for x0, col, h in ((100, (240, 90, 140), 170), (320, (90, 200, 240), 210)):
        f = [(x0 + 30, 400 - h), (x0 + 60, 400 - h), (x0 + 90, 400), (x0, 400)]
        ic.drawn(f, (210, 230, 238), shade=0.1, edge=(60, 90, 110), edge_w=7)
        ic.drawn([(x0 + 14, 340), (x0 + 76, 340), (x0 + 88, 398), (x0 + 2, 398)], col, edge=None)
    ic.glow((256, 270, 70), (150, 255, 120), 30, 0.5)
    bulb = circle(256, 270, 70)
    ic.drawn(bulb, (120, 230, 90), rim_color=(210, 255, 180), edge=(40, 110, 40), edge_w=8)
    ic.drawn(rect(240, 110, 272, 210), (200, 225, 232), shade=0.1, edge=(60, 90, 110), edge_w=7)
    ic.line([(256, 110), (256, 70), (380, 70), (380, 190)], (200, 225, 232), 12, ink=(60, 90, 110), ink_w=8)
    ic.drawn(ellipse(256, 380, 50, 16), (255, 140, 40), shade=0, highlight=0, edge=(150, 60, 20), edge_w=5)


def fabricator(ic):
    body = roundrect(80, 140, 432, 440, 26)
    ic.drawn(body, (200, 210, 225), shade=0.25, edge=(50, 60, 80), edge_w=10)
    ic.drawn(roundrect(120, 180, 300, 300, 12), (30, 50, 70), highlight=0, shade=0, edge=INK, edge_w=7)
    ic.line([(140, 260), (180, 220), (220, 250), (270, 200)], (120, 240, 255), 7)
    for k, col in enumerate(((255, 90, 70), (250, 200, 60), (100, 230, 120))):
        ic.dot(350 + (k % 2) * 40, 200 + k * 34, 13, col, ink=(30, 30, 40))
    bay = rect(140, 330, 372, 420)
    ic.drawn(bay, (60, 66, 80), highlight=0, shade=0.1, edge=INK, edge_w=7)
    ic.glow((256, 390, 60), (120, 230, 255), 22, 0.6)
    ic.line([(256, 300), (256, 380)], (150, 240, 255), 9)
    ic.sparkle(256, 392, 26, (210, 250, 255))
    ic.drawn(rect(220, 80, 292, 145), (160, 170, 190), edge=(50, 60, 80), edge_w=7)


# ---------- tool tiers -------------------------------------------------------------------

def tier_pick(head):
    def f(ic):
        pick(ic, head)
    return f


ICONS = {
    # raw, by hand
    "stone": (stone, 3), "flint": (flint, 5), "log": (wood_log, 7), "fibre": (fibre, 9), "clay": (clay, 11),
    "sand": (sand, 13),
    # raw, by pick
    "copper-ore": (ore((230, 120, 50), 21), 21), "tin-ore": (ore((210, 225, 235), 23), 23),
    "iron-ore": (ore((190, 70, 50), 25), 25), "coal": (coal, 27),
    # worked
    "planks": (planks, 31), "copper-ingot": (bar((226, 122, 64)), 33), "bronze-ingot": (bar((200, 150, 60)), 35),
    "iron-ingot": (bar((150, 156, 168)), 37), "steel-ingot": (bar((110, 150, 200)), 39), "glass": (glass, 41),
    "brick": (brick, 43), "cloth": (cloth, 45), "rope": (rope, 47),
    # advanced
    "circuit": (circuit, 51), "reagent": (vial, 53), "alloy-plate": (alloy, 55), "polymer": (polymer, 57),
    # alien
    "frost-crystal": (frost_crystal, 61), "glowcap": (glowcap, 63), "magma-glass": (magma_glass, 65),
    "star-iron": (star_iron, 67),
    # stations
    "workbench": (workbench, 71), "forge": (forge, 73), "kiln": (kiln, 75), "loom": (loom, 77),
    "chem-lab": (chem_lab, 79), "fabricator": (fabricator, 81),
    # tool tiers
    "pick-stone": (tier_pick((140, 146, 158)), 91), "pick-bronze": (tier_pick((210, 150, 60)), 93),
    "pick-iron": (tier_pick((170, 176, 190)), 95), "pick-steel": (tier_pick((100, 150, 220)), 97),
}


def main():
    os.makedirs(OUT, exist_ok=True)
    paths = []
    for name, (fn, seed) in ICONS.items():
        ic = FillIcon(seed)
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        ic.save(p)
        paths.append(p)
    print("wrote", len(paths), "icons to", OUT)
    if "--sheet" in sys.argv:
        sheet = sys.argv[sys.argv.index("--sheet") + 1]
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), sheet] + paths +
                       ["--ref", "laser drill", "--ref", "plating", "--ref", "power output", "--ref", "inferno"], check=True)


if __name__ == "__main__":
    main()
