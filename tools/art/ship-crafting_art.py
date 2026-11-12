"""Module icons and hull textures for the ship-crafting mockup (docs/mockups/ship-crafting.html).

Painted in code with the hand-drawn-illustration skill's crayon kit, in the manner of
the owner's friend's upgrade icons. Nothing here is drawn by hand. Two kinds of output:

- module icons (128 px, transparent, framed to the subject like the crafting icons);
- hull textures (256 px, full frame) that the page's three.js ships and shipyard wear,
  sampled with nearest filtering so the marker strokes stay visible.

    python tools/art/ship-crafting_art.py
    python tools/art/ship-crafting_art.py --sheet OUT.png
"""

import math
import os
import random
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from crafting_art import KIT, ROOT, FillIcon, lump  # noqa: E402

import crayon  # noqa: E402
from crayon import (INK, Icon, circle, darker, ellipse, lighter, mix, rect,  # noqa: E402
                    rotate, roundrect, star, tone, translate)

OUT = os.path.join(ROOT, "docs", "mockups", "ship-crafting")
S = crayon.S


# ---------- module icons ----------------------------------------------------------------

def cockpit(ic):
    body = [(70, 380), (110, 240), (220, 150), (380, 140), (450, 220), (450, 380)]
    ic.drawn(body, (200, 210, 225), shade=0.25, edge=(40, 50, 70), edge_w=11)
    glass = [(130, 250), (226, 172), (360, 166), (410, 230), (300, 262)]
    ic.drawn(glass, (60, 150, 220), rim_color=(170, 225, 255), edge=(20, 50, 90), edge_w=9)
    ic.line([(190, 200), (240, 236)], (220, 245, 255), 9)
    ic.line([(90, 330), (440, 330)], (240, 180, 40), 9)
    ic.sparkle(400, 90, 28, (230, 245, 255))


def cargo(ic):
    box = [(80, 170), (360, 170), (360, 430), (80, 430)]
    top = [(80, 170), (160, 100), (440, 100), (360, 170)]
    side = [(360, 170), (440, 100), (440, 360), (360, 430)]
    ic.drawn(side, (190, 100, 30), highlight=0, shade=0.2, edge=(100, 50, 10), edge_w=10)
    ic.drawn(box, (240, 140, 40), edge=(110, 56, 14), edge_w=10)
    ic.drawn(top, (255, 186, 90), shade=0.1, edge=(110, 56, 14), edge_w=10)
    for x in (140, 220, 300):
        ic.line([(x, 180), (x, 420)], (200, 104, 30), 7)
    ic.drawn(rect(150, 250, 290, 310), (40, 40, 50), highlight=0, shade=0, edge=None)
    for k in range(4):
        ic.line([(160 + k * 34, 300), (186 + k * 34, 258)], (250, 210, 60), 9)


def thruster(ic):
    ic.glow((256, 420, 110), (120, 200, 255), 40, 0.7)
    flame = [(160, 380), (220, 470), (256, 520), (292, 470), (352, 380)]
    ic.drawn(flame, (120, 210, 255), shade=0, rim_color=(230, 250, 255), edge=None)
    bell = [(200, 200), (312, 200), (380, 400), (132, 400)]
    ic.drawn(bell, (80, 84, 100), shade=0.25, rim_color=(150, 160, 190), edge=INK, edge_w=11)
    ic.drawn(ellipse(256, 400, 124, 26), (255, 170, 60), shade=0, highlight=0, edge=(150, 60, 20), edge_w=8)
    can = roundrect(170, 60, 342, 210, 20)
    ic.drawn(can, (200, 206, 220), edge=(40, 50, 70), edge_w=10)
    ic.line([(180, 130), (332, 130)], (240, 180, 40), 9)
    for x in (130, 382):
        ic.speed(x, 440, 60, 90, (200, 235, 255), 7)


def reactor(ic):
    ic.glow((256, 270, 150), (90, 230, 255), 50, 0.6)
    shell = roundrect(110, 80, 402, 460, 60)
    ic.drawn(shell, (90, 96, 116), shade=0.25, edge=INK, edge_w=11)
    core = circle(256, 270, 100)
    ic.drawn(core, (90, 220, 250), rim_color=(220, 255, 255), edge=(20, 80, 110), edge_w=9)
    ic.zap([(190, 240), (240, 300), (270, 230), (320, 300)], (240, 255, 255), 8, jag=10, seed=3)
    for y in (110, 430):
        ic.drawn(rect(160, y - 14, 352, y + 14), (240, 180, 40), highlight=0, edge=(110, 70, 10), edge_w=7)
    ic.sparkle(420, 90, 26, (210, 250, 255))


def fuel(ic):
    tank = roundrect(140, 60, 372, 460, 110)
    ic.drawn(tank, (220, 70, 60), rim_color=(255, 160, 140), edge=(110, 24, 20), edge_w=11)
    ic.drawn(rect(146, 220, 366, 290), (240, 236, 226), shade=0.15, edge=None)
    ic.line([(146, 220), (366, 220)], (110, 24, 20), 6)
    ic.line([(146, 290), (366, 290)], (110, 24, 20), 6)
    ic.drawn(rect(226, 30, 286, 70), (120, 126, 140), edge_w=8)
    drop = [(256, 230)] + [(256 + 24 * math.sin(t), 266 - 24 * math.cos(t)) for t in
                           [math.pi * 0.25 + i * math.pi * 1.5 / 10 for i in range(11)]]
    ic.drawn(drop, (240, 150, 40), shade=0, highlight=0, edge=(120, 60, 10), edge_w=5)


def shield(ic):
    ic.glow((256, 250, 170), (110, 170, 255), 50, 0.5)
    for r, w in ((210, 9), (170, 7)):
        ic.outline([(256 + r * math.cos(a), 250 + r * math.sin(a)) for a in
                    [math.pi * (1.1 + i * 0.8 / 20) for i in range(21)]], (150, 210, 255), w, closed=False, passes=1)
    emitter = [(150, 460), (180, 330), (332, 330), (362, 460)]
    ic.drawn(emitter, (120, 130, 150), shade=0.25, edge=INK, edge_w=10)
    dish = ellipse(256, 300, 130, 50)
    ic.drawn(dish, (90, 150, 240), rim_color=(200, 230, 255), edge=(20, 40, 100), edge_w=9)
    ic.dot(256, 300, 22, (230, 245, 255), ink=(20, 40, 100))
    hexes = [(256 + 150 * math.cos(a), 200 + 110 * math.sin(a)) for a in (-2.4, -1.57, -0.74)]
    for x, y in hexes:
        ic.outline(circle(x, y, 26, n=6, start=math.pi / 6), (180, 225, 255), 6, passes=1)


def bar_along(p0, p1, half):
    """A straight bar from p0 to p1, `half` wide either side."""
    dx, dy = p1[0] - p0[0], p1[1] - p0[1]
    L = math.hypot(dx, dy)
    nx, ny = -dy / L * half, dx / L * half
    return [(p0[0] + nx, p0[1] + ny), (p1[0] + nx, p1[1] + ny), (p1[0] - nx, p1[1] - ny), (p0[0] - nx, p0[1] - ny)]


def mining_laser(ic):
    ic.glow((410, 150, 70), (255, 90, 60), 30, 0.7)
    ic.drawn(bar_along((330, 220), (470, 100), 16), (255, 110, 80), shade=0, rim_color=(255, 235, 210), edge=None)
    ic.drawn(bar_along((210, 320), (340, 210), 36), (70, 74, 88), shade=0.2, edge=INK, edge_w=10)
    ic.drawn(bar_along((330, 222), (352, 203), 46), (120, 126, 146), edge=INK, edge_w=8)
    ic.drawn(bar_along((70, 430), (240, 290), 82), (245, 200, 50), edge=(120, 90, 10), edge_w=11)
    for k in range(3):
        c = (110 + k * 45, 400 - k * 38)
        ic.line(bar_along((c[0] - 30, c[1] - 36), (c[0] + 30, c[1] + 36), 1)[:2], (40, 36, 40), 14)
    rock = lump(450, 110, 56, 9)
    ic.drawn(rock, (150, 90, 70), shade=0.25, edge_w=8)
    for x, y in ((380, 60), (500, 190), (400, 190)):
        ic.sparkle(x, y, 18, (255, 220, 120))


def habitation(ic):
    body = roundrect(60, 130, 452, 400, 80)
    ic.drawn(body, (236, 236, 228), shade=0.25, edge=(70, 80, 100), edge_w=11)
    for x in (130, 220, 310):
        ic.glow((x + 30, 230, 30), (255, 200, 90), 18, 0.6)
        ic.drawn(roundrect(x, 200, x + 60, 260, 16), (255, 200, 90), rim_color=(255, 240, 180),
                 edge=(110, 70, 20), edge_w=7)
    ic.line([(80, 320), (432, 320)], (90, 160, 210), 10)
    ic.drawn(rect(380, 180, 430, 300), (200, 204, 210), shade=0.1, edge=(70, 80, 100), edge_w=7)
    leaf = ellipse(150, 360, 40, 18, rot=-0.4)
    ic.drawn(leaf, (90, 190, 80), edge_w=6)


def gyro(ic):
    for r, col, rot in ((200, (240, 180, 40), 0.0), (160, (90, 200, 240), 1.0)):
        ic.outline(ellipse(256, 256, r, r * 0.42, rot=rot), tone(col), 22, passes=1)
        ic.outline(ellipse(256, 256, r, r * 0.42, rot=rot), col, 13, passes=1)
    ic.drawn(circle(256, 256, 80), (200, 206, 220), rim_color=(240, 245, 255), edge=(40, 50, 70), edge_w=10)
    ic.dot(256, 256, 22, (60, 66, 84))
    for a in (0.6, 2.6, 4.4):
        ic.line([(256 + 230 * math.cos(a), 256 + 230 * math.sin(a)),
                 (256 + 250 * math.cos(a + 0.35), 256 + 250 * math.sin(a + 0.35))], (240, 240, 240), 8)


def hull_plate(ic):
    plate = [(90, 120), (420, 90), (440, 400), (80, 420)]
    ic.drawn(plate, (170, 180, 196), rim_color=(225, 232, 245), edge=(40, 50, 70), edge_w=11)
    ic.line([(260, 108), (262, 410)], (110, 120, 140), 6)
    for x, y in ((120, 150), (390, 125), (410, 370), (115, 385), (240, 140), (280, 390)):
        ic.dot(x, y, 10, (210, 214, 224), ink=(30, 34, 46))


def landing_gear(ic):
    ic.drawn(rect(150, 50, 362, 110), (120, 126, 140), edge=(40, 50, 70), edge_w=10)
    ic.drawn(bar_along((256, 100), (256, 330), 34), (190, 196, 210), edge=(40, 50, 70), edge_w=10)
    ic.drawn(bar_along((256, 200), (256, 330), 20), (230, 236, 245), shade=0.1, edge=(40, 50, 70), edge_w=7)
    ic.drawn(bar_along((180, 110), (240, 280), 14), (240, 180, 40), edge_w=8)
    foot = [(110, 400), (160, 330), (352, 330), (402, 400)]
    ic.drawn(foot, (70, 74, 88), shade=0.2, rim_color=(140, 150, 170), edge=INK, edge_w=11)
    ic.line([(120, 420), (392, 420)], (240, 180, 40), 10)
    for x in (90, 422):
        ic.speed(x, 330, 50, 90, (230, 230, 230), 7)


ICONS = {
    "cockpit": (cockpit, 3), "cargo": (cargo, 5), "thruster": (thruster, 7), "reactor": (reactor, 9),
    "fuel": (fuel, 11), "shield": (shield, 13), "mining-laser": (mining_laser, 15),
    "habitation": (habitation, 17), "gyro": (gyro, 19), "hull-plate": (hull_plate, 21),
    "landing-gear": (landing_gear, 23),
}


# ---------- textures (full frame) --------------------------------------------------------

def ground(ic, color, angle=30, hatch=1.0):
    full = rect(-20, -20, S + 20, S + 20)
    ic.flat(full, color, wobble=0)
    ic.scribble(full, color, var=0.18, angle=angle, density=hatch * 0.9, length=(50, 140), spread=10)
    ic.scribble(full, color, var=0.12, angle=angle + 60, density=hatch * 0.3, length=(30, 90), spread=14)


def seams(ic, color, cols=2, rows=2, rivets=True, w=7):
    for i in range(1, cols):
        x = S * i / cols
        ic.line([(x, -10), (x, S + 10)], color, w, amp=1.5)
    for j in range(1, rows):
        y = S * j / rows
        ic.line([(-10, y), (S + 10, y)], color, w, amp=1.5)
    ic.line([(0, 3), (S, 3)], color, w)
    ic.line([(3, 0), (3, S)], color, w)
    if rivets:
        for i in range(cols):
            for j in range(rows):
                x0, y0 = S * i / cols, S * j / rows
                for dx, dy in ((22, 22), (S / cols - 22, 22), (22, S / rows - 22), (S / cols - 22, S / rows - 22)):
                    ic.dot(x0 + dx, y0 + dy, 7, lighter(color, 0.55), ink=darker(color, 0.4))


def tex_hull(ic):
    ground(ic, (178, 188, 204))
    seams(ic, (110, 120, 142))
    ic.dabs(rect(0, 0, S, S), (225, 232, 245), n=10, size=14)


def tex_hull_dark(ic):
    ground(ic, (70, 76, 92))
    seams(ic, (40, 44, 56), 2, 3)


def tex_cargo(ic):
    ground(ic, (236, 136, 40), angle=80)
    for x in (S * 0.25, S * 0.5, S * 0.75):
        ic.line([(x, -10), (x, S + 10)], (190, 96, 24), 12)
    for k in range(-2, 8):
        x = k * 80
        ic.line([(x, S - 10), (x + 60, S - 70)], (40, 36, 40), 26)
    ic.line([(0, S - 80), (S, S - 80)], (90, 46, 12), 7)


def tex_reactor(ic):
    ground(ic, (60, 66, 84))
    for y in (90, 200, 310, 420):
        ic.glow((256, y, 40), (90, 230, 255), 20, 0.6)
        ic.drawn(roundrect(70, y - 22, 442, y + 22, 18), (90, 220, 250), shade=0, rim_color=(220, 255, 255),
                 edge=(20, 70, 100), edge_w=6, hatch=0.6)
    seams(ic, (30, 34, 46), 1, 1, rivets=True)


def tex_thruster(ic):
    ground(ic, (84, 88, 104))
    for k, y in enumerate(range(40, S, 70)):
        ic.line([(-10, y), (S + 10, y)], (50, 52, 64) if k % 2 else (130, 136, 156), 14)


def tex_fuel(ic):
    ground(ic, (220, 70, 60), angle=20)
    ic.flat(rect(-10, 190, S + 10, 320), (240, 236, 226), wobble=1)
    ic.scribble(rect(-10, 190, S + 10, 320), (240, 236, 226), var=0.12, angle=20, density=0.8)
    ic.line([(-10, 190), (S + 10, 190)], (110, 24, 20), 8)
    ic.line([(-10, 320), (S + 10, 320)], (110, 24, 20), 8)


def tex_shield(ic):
    ground(ic, (60, 110, 200), angle=50)
    r = 46
    for j in range(-1, 8):
        for i in range(-1, 8):
            x = i * r * 1.75 + (j % 2) * r * 0.87
            y = j * r * 1.5
            ic.outline(circle(x, y, r, n=6, start=math.pi / 6), (150, 210, 255), 6, passes=1, amp=1.2)


def tex_laser(ic):
    ground(ic, (245, 200, 50), angle=40)
    for k in range(-4, 8):
        x = k * 110
        ic.flat([(x, -10), (x + 55, -10), (x + 55 + S, S + 10), (x + S, S + 10)], (40, 36, 40), wobble=1.5)


def tex_hab(ic):
    ground(ic, (232, 232, 224))
    seams(ic, (150, 156, 170), 2, 2, rivets=False)
    for x in (60, 316):
        ic.drawn(roundrect(x, 190, x + 136, 300, 24), (255, 200, 90), rim_color=(255, 240, 180),
                 edge=(80, 70, 60), edge_w=8, shade=0.1)
    ic.line([(-10, 380), (S + 10, 380)], (90, 160, 210), 16)


def tex_glass(ic):
    ground(ic, (40, 90, 150), angle=60)
    for a, b, w in (((60, 300), (260, 80), 22), ((140, 360), (360, 130), 10), ((330, 460), (470, 300), 14)):
        ic.line([a, b], (170, 225, 255), w)


def tex_pad(ic):
    ground(ic, (128, 128, 122), angle=10)
    r = 74
    for j in range(-1, 6):
        for i in range(-1, 6):
            x = i * r * 1.75 + (j % 2) * r * 0.87
            y = j * r * 1.5
            ic.outline(circle(x, y, r, n=6, start=math.pi / 6), (86, 86, 82), 6, passes=1, amp=1.5)
    ic.line([(0, 470), (S, 470)], (245, 200, 50), 18)


def tex_girder(ic):
    ground(ic, (190, 110, 60), angle=70)
    ic.line([(-10, 30), (S + 10, 30)], (120, 60, 30), 30)
    ic.line([(-10, S - 30), (S + 10, S - 30)], (120, 60, 30), 30)
    for k in range(-1, 5):
        x = k * 140
        ic.line([(x, 40), (x + 140, S - 40)], (150, 80, 40), 22, ink=(90, 44, 20), ink_w=8)
        ic.line([(x + 140, 40), (x, S - 40)], (150, 80, 40), 22, ink=(90, 44, 20), ink_w=8)


TEXTURES = {
    "tex-hull": (tex_hull, 101), "tex-hull-dark": (tex_hull_dark, 103), "tex-cargo": (tex_cargo, 105),
    "tex-reactor": (tex_reactor, 107), "tex-thruster": (tex_thruster, 109), "tex-fuel": (tex_fuel, 111),
    "tex-shield": (tex_shield, 113), "tex-laser": (tex_laser, 115), "tex-hab": (tex_hab, 117),
    "tex-glass": (tex_glass, 119), "tex-pad": (tex_pad, 121), "tex-girder": (tex_girder, 123),
}


def save_texture(ic, path):
    from PIL import Image
    from PIL.PngImagePlugin import PngInfo
    im = ic.img.convert("RGB").resize((256, 256), Image.LANCZOS)
    meta = PngInfo()
    meta.add_text("source", "armory-crayon")
    im.save(path, pnginfo=meta)


def main():
    os.makedirs(OUT, exist_ok=True)
    paths = []
    for name, (fn, seed) in ICONS.items():
        ic = FillIcon(seed)
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        ic.save(p)
        paths.append(p)
    for name, (fn, seed) in TEXTURES.items():
        ic = Icon(seed)
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        save_texture(ic, p)
        paths.append(p)
    print("wrote", len(paths), "files to", OUT)
    if "--sheet" in sys.argv:
        sheet = sys.argv[sys.argv.index("--sheet") + 1]
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), sheet] + paths +
                       ["--ref", "laser drill", "--ref", "plating", "--ref", "power output"], check=True)


if __name__ == "__main__":
    main()
