"""Build-piece icons and material textures for the building mockup
(docs/mockups/building.html).

Painted by code with the hand-drawn-illustration skill's crayon kit, in the
manner of the owner's friend's upgrade icons. Nothing here is drawn by hand.

    python tools/art/building_art.py            # writes docs/mockups/building/*.png
    python tools/art/building_art.py --sheet    # and a contact sheet beside the friend's icons

Icons (icon_*.png) are the palette's pieces on a transparent ground. Textures
(tex_*.png) fill the square and are made to wrap: the painting is crossfaded
with itself shifted half a tile, so its edges come from its own middle and
meet seamlessly when the mockup repeats it.
"""

import math
import os
import subprocess
import sys

from PIL import Image, ImageDraw

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)
from crayon import Icon, INK, S, circle, rect, roundrect, rotate, translate, lighter, darker, tone, mix  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "building")


def hexagon(cx, cy, r, squash=1.0, rot=0.0):
    return [(cx + r * math.cos(rot + k * math.pi / 3), cy + r * squash * math.sin(rot + k * math.pi / 3)) for k in range(6)]


# ---------- piece icons ---------------------------------------------------------------

def icon_floor(ic):
    top = hexagon(256, 230, 200, 0.55)
    side = [top[0], top[1], top[2], top[3], (top[3][0], top[3][1] + 60), (top[2][0], top[2][1] + 60), (top[1][0], top[1][1] + 60), (top[0][0], top[0][1] + 60)]
    ic.drawn(side, (140, 92, 50), hatch=0.8, highlight=0, shade=0.2)
    ic.drawn(top, (204, 150, 88), hatch=1.0, shade=0.2, highlight=0.18, angle=0)
    for k in (-1, 0, 1):
        ic.line([(90, 230 + k * 50), (422, 230 + k * 50)], (150, 100, 56), 5)


def icon_wall(ic):
    face = [(100, 160), (380, 120), (380, 400), (100, 440)]
    edge = [(380, 120), (420, 140), (420, 410), (380, 400)]
    ic.drawn(edge, (150, 90, 60), hatch=0.7, highlight=0, shade=0.2)
    ic.drawn(face, (200, 96, 70), hatch=1.0, shade=0.22, highlight=0.16)
    for k in range(1, 6):
        y0 = 160 + k * 47
        ic.line([(100, y0), (380, y0 - 40 * k / 6 * 0 - 40)], (240, 220, 200), 5, amp=1)
    for k in range(6):
        off = 0 if k % 2 else 70
        y = 182 + k * 47 - 20
        for x in range(130 + off, 380, 140):
            ic.line([(x, y - 18), (x, y + 18)], (240, 220, 200), 5, amp=1)
    ic.sparkle(440, 90, 22)


def icon_roof(ic):
    # a gable roof seen from the front corner: the gable end, the long slope, the eave's shadow
    gable = [(70, 300), (190, 120), (310, 300)]
    slope = [(190, 120), (440, 160), (470, 330), (310, 300)]
    ic.drawn(rect(90, 300, 300, 440), (226, 214, 186), hatch=0.7, shade=0.2, highlight=0.1, edge_w=6)
    ic.drawn(rect(300, 300, 450, 430), (190, 176, 150), hatch=0.7, shade=0.15, highlight=0, edge_w=6)
    ic.drawn(gable, (196, 150, 96), hatch=0.9, shade=0.2, highlight=0.15)
    ic.drawn(slope, (220, 110, 60), hatch=1.2, shade=0.22, highlight=0.2, angle=10)
    for t in (0.3, 0.55, 0.8):
        ic.line([(190 + (310 - 190) * t, 120 + (300 - 120) * t), (440 + (470 - 440) * t, 160 + (330 - 160) * t)], (150, 66, 40), 6)
    ic.line([(60, 310), (190, 110), (450, 152)], (110, 50, 30), 12)


def icon_door(ic):
    frame = roundrect(130, 50, 382, 470, 26)
    ic.drawn(frame, (130, 90, 56), hatch=0.8, highlight=0.1, shade=0.2)
    leaf = roundrect(160, 80, 352, 470, 16)
    ic.drawn(leaf, (190, 128, 70), hatch=1.1, shade=0.22, highlight=0.18, angle=90)
    for x in (224, 288):
        ic.line([(x, 92), (x, 462)], (140, 92, 50), 5)
    for y in (150, 380):
        ic.drawn(rect(160, y - 14, 352, y + 14), (70, 70, 80), hatch=0.4, highlight=0.1, edge=INK, edge_w=5)
    ic.dot(326, 270, 14, (250, 200, 70), ink=INK)


def icon_window(ic):
    ic.glow((256, 250, 160), (255, 190, 80), radius=40, strength=0.6)
    frame = rect(110, 90, 402, 420)
    ic.drawn(frame, (130, 90, 56), hatch=0.8, highlight=0.12)
    pane = rect(140, 120, 372, 390)
    ic.drawn(pane, (255, 200, 90), hatch=0.9, shade=0.18, highlight=0.22, rim_color=(255, 240, 180), edge=(150, 90, 30))
    ic.line([(256, 120), (256, 390)], (110, 74, 44), 12)
    ic.line([(140, 255), (372, 255)], (110, 74, 44), 12)
    ic.drawn(rect(90, 410, 422, 452), (170, 160, 150), hatch=0.6, highlight=0.12, edge_w=6)


def icon_stairs(ic):
    for k in range(5):
        x0, y0 = 90 + k * 64, 420 - k * 64
        ic.drawn(rect(x0, y0 - 64, x0 + 80 + (4 - k) * 0, 450), (180, 124, 70) if k % 2 else (200, 140, 80), hatch=0.9, shade=0.2, highlight=0.15, edge_w=6)
    ic.line([(110, 250), (200, 160)], (255, 255, 240), 12, ink=INK, ink_w=8)
    ic.drawn([(230, 130), (150, 150), (210, 210)], (255, 255, 240), hatch=0.2, highlight=0, shade=0, edge=INK, edge_w=6)


def icon_lantern(ic):
    ic.glow((256, 280, 150), (255, 170, 60), radius=44, strength=0.8)
    ic.line([(256, 30), (256, 110)], (60, 60, 70), 9)
    ic.drawn(roundrect(200, 100, 312, 150, 12), (70, 70, 84), hatch=0.5, highlight=0.15, edge=INK, edge_w=6)
    ic.drawn(rect(196, 150, 316, 380), (255, 196, 80), hatch=0.9, shade=0.15, highlight=0.25, rim_color=(255, 245, 190), edge=(140, 80, 20))
    for x in (196, 256, 316):
        ic.line([(x, 150), (x, 380)], (60, 60, 70), 10)
    ic.drawn(roundrect(180, 376, 332, 420, 10), (70, 70, 84), hatch=0.5, highlight=0.1, edge=INK, edge_w=6)
    ic.drawn([(256, 200), (282, 270), (256, 320), (230, 270)], (255, 130, 40), hatch=0.4, highlight=0.3, shade=0, edge=None)
    for (x, y) in ((110, 180), (400, 220), (380, 430)):
        ic.sparkle(x, y, 20)


def icon_airlock(ic):
    body = roundrect(80, 80, 432, 440, 60)
    ic.drawn(body, (150, 162, 176), hatch=1.0, shade=0.24, highlight=0.18, edge=(30, 36, 50))
    ring = circle(256, 260, 140)
    ic.drawn(ring, (90, 100, 116), hatch=0.6, highlight=0.12, edge=(20, 24, 34))
    ic.drawn(circle(256, 260, 104), (70, 180, 200), hatch=0.8, highlight=0.2, rim_color=(170, 240, 250), edge=(20, 70, 90))
    ic.line([(256, 170), (256, 350)], (240, 190, 50), 10)
    ic.line([(166, 260), (346, 260)], (240, 190, 50), 10)
    ic.drawn(circle(256, 260, 28), (240, 190, 50), hatch=0.4, highlight=0.15, edge=INK, edge_w=6)
    for k in range(8):
        a = k * math.pi / 4 + 0.4
        ic.dot(256 + 160 * math.cos(a), 260 + 160 * math.sin(a), 8, (200, 206, 214), ink=INK)
    for x in (110, 402):
        ic.drawn(rect(x - 10, 110, x + 10, 160), (240, 190, 50), hatch=0.3, highlight=0, shade=0, edge=INK, edge_w=5)


def icon_post(ic):
    ic.drawn(rect(216, 40, 296, 470), (150, 100, 56), hatch=1.0, shade=0.22, highlight=0.18, angle=90)
    ic.drawn(hexagon(256, 470, 90, 0.4), (130, 130, 130), hatch=0.6, highlight=0.12, edge_w=6)
    for y in (120, 260, 400):
        ic.line([(220, y), (292, y + 12)], (110, 70, 40), 5)


# ---------- textures ----------------------------------------------------------------------

BIG = (-40, -40, S + 40, S + 40)


def tex_plank(ic):
    ic.flat(rect(*BIG), (170, 116, 66), wobble=0)
    for k in range(6):
        y0 = k * S / 6
        c = mix((176, 120, 68), (196, 140, 82), (k * 37 % 5) / 5)
        ic.drawn(rect(-30, y0 + 4, S + 30, y0 + S / 6 - 4), c, hatch=1.0, shade=0.12, highlight=0.08, edge=(96, 60, 30), edge_w=6, angle=0)
        x = (k * 173) % S
        ic.line([(x, y0 + 6), (x, y0 + S / 6 - 6)], (96, 60, 30), 6)
        ic.dot((x + 60) % S, y0 + S / 12, 5, (110, 70, 36))


def tex_stone(ic):
    ic.flat(rect(*BIG), (96, 92, 88), wobble=0)
    import random
    rnd = random.Random(5)
    for row in range(5):
        for col in range(5):
            cx = col * S / 4 + (row % 2) * S / 8 + rnd.uniform(-10, 10)
            cy = row * S / 4 + rnd.uniform(-10, 10)
            r = rnd.uniform(48, 60)
            poly = [(cx + r * math.cos(a) * rnd.uniform(0.85, 1.1), cy + r * 0.8 * math.sin(a) * rnd.uniform(0.85, 1.1)) for a in [k * math.pi / 4 for k in range(8)]]
            g = rnd.randint(-14, 14)
            ic.drawn(poly, (150 + g, 146 + g, 138 + g), hatch=0.8, shade=0.25, highlight=0.18, edge=(70, 66, 62), edge_w=6)


def tex_thatch(ic):
    ic.flat(rect(*BIG), (180, 140, 60), wobble=0)
    import random
    rnd = random.Random(9)
    for band in range(6):
        y = band * S / 5 - 20
        ic.scribble(rect(-20, y, S + 20, y + S / 5 + 30), (214, 170, 80), var=0.3, width=6, length=(60, 130), angle=95, spread=8, density=2.2)
        ic.line([(-20, y + S / 5 + 20), (S + 20, y + S / 5 + 20)], (120, 86, 36), 8, amp=4)


def tex_metal(ic):
    ic.flat(rect(*BIG), (120, 128, 140), wobble=0)
    for (x0, y0, x1, y1) in ((0, 0, 256, 256), (256, 0, 512, 256), (0, 256, 256, 512), (256, 256, 512, 512)):
        ic.drawn(rect(x0 + 8, y0 + 8, x1 - 8, y1 - 8), (150, 160, 174), hatch=0.9, shade=0.2, highlight=0.15, edge=(40, 46, 60), edge_w=7, angle=20)
        for (x, y) in ((x0 + 26, y0 + 26), (x1 - 26, y0 + 26), (x0 + 26, y1 - 26), (x1 - 26, y1 - 26)):
            ic.dot(x, y, 7, (200, 206, 214), ink=(40, 46, 60))
    ic.line([(30, 130), (220, 130)], (230, 180, 40), 9)


def tex_brick(ic):
    ic.flat(rect(*BIG), (210, 200, 186), wobble=0)
    rows = 8
    h = S / rows
    for r in range(rows):
        off = 0 if r % 2 == 0 else S / 4
        for k in range(-1, 3):
            x0 = k * S / 2 + off
            g = ((r * 7 + k * 3) % 5) * 6
            ic.drawn(rect(x0 + 6, r * h + 5, x0 + S / 2 - 6, (r + 1) * h - 5), (180 + g, 80 + g // 2, 58), hatch=0.9, shade=0.2, highlight=0.12, edge=(110, 50, 36), edge_w=5)


def tex_plaster(ic):
    ic.flat(rect(*BIG), (226, 214, 186), wobble=0)
    ic.scribble(rect(*BIG), (226, 214, 186), var=0.12, width=9, length=(60, 140), angle=30, spread=40, density=1.4)
    ic.scribble(rect(*BIG), (210, 196, 166), var=0.1, width=7, length=(30, 80), angle=-40, spread=30, density=0.5)


def tex_regolith(ic):
    ic.flat(rect(*BIG), (128, 124, 118), wobble=0)
    ic.scribble(rect(*BIG), (128, 124, 118), var=0.2, width=7, length=(20, 60), spread=90, density=1.6)
    import random
    rnd = random.Random(3)
    for _ in range(5):
        x, y, r = rnd.uniform(60, 450), rnd.uniform(60, 450), rnd.uniform(24, 50)
        ic.drawn(circle(x, y, r), (110, 106, 100), hatch=0.5, shade=0.3, highlight=0.2, edge=(80, 76, 72), edge_w=5, light=(1, 1))


def tex_glass(ic):
    ic.flat(rect(*BIG), (60, 120, 150), wobble=0)
    ic.scribble(rect(*BIG), (90, 170, 200), var=0.18, width=8, length=(60, 160), angle=60, spread=10, density=1.2)
    for k in range(3):
        ic.line([(80 + k * 50, 420), (260 + k * 50, 90)], (200, 245, 255), 8)


ICONS = {
    "icon_floor": (icon_floor, 31), "icon_wall": (icon_wall, 32), "icon_roof": (icon_roof, 33), "icon_door": (icon_door, 34),
    "icon_window": (icon_window, 35), "icon_stairs": (icon_stairs, 36), "icon_lantern": (icon_lantern, 37), "icon_airlock": (icon_airlock, 38),
    "icon_post": (icon_post, 39),
}
TEXTURES = {
    "tex_plank": (tex_plank, 41), "tex_stone": (tex_stone, 42), "tex_thatch": (tex_thatch, 43), "tex_metal": (tex_metal, 44),
    "tex_brick": (tex_brick, 45), "tex_plaster": (tex_plaster, 46), "tex_regolith": (tex_regolith, 47), "tex_glass": (tex_glass, 48),
}


def seamless(im):
    """Crossfade the painting with itself shifted half a tile: the weight is 1
    in the middle and 0 at the edges, where the shifted copy's own middle
    shows, so the tile wraps without a seam."""
    w, h = im.size
    shifted = Image.new("RGBA", (w, h))
    shifted.paste(im.crop((w // 2, h // 2, w, h)), (0, 0))
    shifted.paste(im.crop((0, h // 2, w // 2, h)), (w - w // 2, 0))
    shifted.paste(im.crop((w // 2, 0, w, h // 2)), (0, h - h // 2))
    shifted.paste(im.crop((0, 0, w // 2, h // 2)), (w - w // 2, h - h // 2))
    mask = Image.new("L", (w, h))
    px = mask.load()
    for y in range(h):
        for x in range(w):
            u = min(x, w - 1 - x) / (w / 2)
            v = min(y, h - 1 - y) / (h / 2)
            t = min(1.0, min(u, v) * 2.2)
            px[x, y] = int(255 * t * t * (3 - 2 * t))
    return Image.composite(im, shifted, mask)


def main():
    os.makedirs(OUT, exist_ok=True)
    paths = []
    for name, (fn, seed) in ICONS.items():
        ic = Icon(seed)
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        ic.save(p)
        paths.append(p)
    for name, (fn, seed) in TEXTURES.items():
        ic = Icon(seed)
        fn(ic)
        im = ic.img.resize((128, 128), Image.LANCZOS)
        im.putalpha(255)
        im = seamless(im)
        from PIL.PngImagePlugin import PngInfo
        meta = PngInfo()
        meta.add_text("source", "armory-crayon")
        p = os.path.join(OUT, name + ".png")
        im.save(p, pnginfo=meta)
        paths.append(p)
    for p in paths:
        print("wrote", p)
    if "--sheet" in sys.argv:
        sheet = os.path.join(os.environ.get("TEMP", "."), "building_sheet.png")
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), sheet] + paths + ["--ref", "plating", "--ref", "power output", "--ref", "laser drill"], check=True)


if __name__ == "__main__":
    main()
