"""Textures, cards, impostors and species icons for the foliage playground
(docs/mockups/foliage.html).

Painted by code with the hand-drawn-illustration kit
(.claude/skills/hand-drawn-illustration/scripts/crayon.py), copying the owner's
friend's method: a flat block, hatching in the shape's own tones, form
crescents, highlight dabs and a same-hue edge, light from the upper left. It is
not hand drawn and the page says so.

Four kinds of output, all 128 x 128 PNG beside the page:

- tiles (ground_*, side_*, bark_*, rock, crystal, cap_*): full squares made to
  wrap (the seam is cross-faded with a half-offset copy), sampled nearest in
  the page as the game samples terrain;
- cards (leaf_*, needle, grass_*, flower_*, fern, vine, litter, twig, kelp,
  reed): shapes on a transparent ground, alpha-tested in the page;
- impostors (imp_*): one tree per species, standing on the bottom edge, for
  the far LOD billboard and the mid LOD crossed cards;
- icons (icon_*): the same species as a picker icon, on a little mound.

    python tools/art/foliage_art.py                # writes docs/mockups/foliage/*.png
    python tools/art/foliage_art.py --sheet S.png  # also a contact sheet beside the friend's icons
"""

import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)
from crayon import Icon, circle, ellipse, rect, rotate, translate, star, mix, darker, lighter, tone  # noqa: E402
from PIL import Image, ImageChops, ImageDraw  # noqa: E402
from PIL.PngImagePlugin import PngInfo  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "foliage")
FULL = rect(-30, -30, 542, 542)


# ---------- shape helpers ----------------------------------------------------------

def leaf(cx, cy, L, W, ang, tip=0.8):
    """A pointed leaf from its stalk end (cx, cy) out along `ang` radians."""
    pts = []
    n = 14
    for i in range(n + 1):
        t = i / n
        pts.append((t * L, -W * math.sin(math.pi * t) ** tip))
    for i in range(n - 1, 0, -1):
        t = i / n
        pts.append((t * L, W * math.sin(math.pi * t) ** tip))
    return translate(rotate(pts, ang), cx, cy)


def blob(cx, cy, rx, ry, bumps=7, amp=0.12, seed=0, n=90):
    rnd = random.Random(seed)
    ph = [rnd.uniform(0, 6.28) for _ in range(3)]
    pts = []
    for i in range(n):
        t = 2 * math.pi * i / n
        r = 1 + amp * (0.6 * math.sin(bumps * t + ph[0]) + 0.4 * abs(math.sin((bumps + 3) * t / 2 + ph[1])))
        pts.append((cx + rx * r * math.cos(t), cy + ry * r * math.sin(t)))
    return pts


def strip(pts, w0, w1):
    """A tapering ribbon along a polyline, w0 wide at the start, w1 at the end."""
    left, right = [], []
    n = len(pts)
    for i, (x, y) in enumerate(pts):
        a = pts[min(i + 1, n - 1)]
        b = pts[max(i - 1, 0)]
        dx, dy = a[0] - b[0], a[1] - b[1]
        dl = math.hypot(dx, dy) or 1
        nx, ny = -dy / dl, dx / dl
        w = (w0 + (w1 - w0) * i / max(1, n - 1)) / 2
        left.append((x + nx * w, y + ny * w))
        right.append((x - nx * w, y - ny * w))
    return left + right[::-1]


def curve(p0, p1, p2, n=12):
    """A quadratic curve through control point p1."""
    out = []
    for i in range(n + 1):
        t = i / n
        a = (1 - t) ** 2
        b = 2 * (1 - t) * t
        c = t * t
        out.append((a * p0[0] + b * p1[0] + c * p2[0], a * p0[1] + b * p1[1] + c * p2[1]))
    return out


# ---------- tiles -------------------------------------------------------------------

def wrap(im):
    """Make a 128 tile wrap: the edges are taken from a half-offset copy, whose
    edges are the original's middle, cross-faded into the original toward the
    centre. A soft cross-fade, so the strokes ghost a little but never seam."""
    w, h = im.size
    rolled = ImageChops.offset(im, w // 2, h // 2)
    m = Image.new("L", (w, h), 0)
    px = m.load()
    band = w * 0.28
    for y in range(h):
        for x in range(w):
            d = min(x, y, w - 1 - x, h - 1 - y)
            px[x, y] = int(255 * max(0.0, min(1.0, (d - 4) / band)))
    return Image.composite(im, rolled, m)


def ground(ic, base, var=0.2, speck=None, speck_n=60, strokes=3, angle=None):
    ic.flat(FULL, base, wobble=0)
    a = angle if angle is not None else ic.rnd.uniform(20, 70)
    for k in range(strokes):
        ic.scribble(FULL, base, var=var, angle=a + k * 55, density=0.9, length=(40, 120), spread=16, width=7)
    if speck:
        d = ImageDraw.Draw(ic.img)
        for _ in range(speck_n):
            c = speck[ic.rnd.randrange(len(speck))]
            x, y = ic.rnd.uniform(0, 512), ic.rnd.uniform(0, 512)
            r = ic.rnd.uniform(4, 10)
            d.ellipse((x - r, y - r * 0.7, x + r, y + r * 0.7), fill=c + (255,))


def t_grass(ic):
    ground(ic, (92, 150, 58), speck=[(132, 190, 80), (60, 104, 40), (170, 206, 96)], speck_n=70)
    for _ in range(26):  # little blade ticks
        x, y = ic.rnd.uniform(0, 512), ic.rnd.uniform(0, 512)
        ic.line([(x, y + 22), (x + ic.rnd.uniform(-8, 8), y)], (150, 204, 92), 6, amp=1)


def t_dry(ic):
    ground(ic, (196, 168, 92), speck=[(222, 198, 120), (160, 128, 70), (140, 150, 80)], angle=80)


def t_litter(ic):
    ground(ic, (104, 78, 50), speck=[(80, 58, 38)], speck_n=30)
    for i in range(22):
        x, y = ic.rnd.uniform(20, 492), ic.rnd.uniform(20, 492)
        col = [(196, 120, 52), (168, 84, 40), (210, 160, 70), (120, 130, 60)][i % 4]
        ic.drawn(leaf(x, y, 70, 22, ic.rnd.uniform(0, 6.28)), col, hatch=0.5, shade=0.2, highlight=0.0, edge_w=4)


def t_jungle(ic):
    ground(ic, (58, 104, 50), speck=[(40, 76, 36), (98, 70, 44)], speck_n=40)
    for i in range(10):
        x, y = ic.rnd.uniform(20, 492), ic.rnd.uniform(20, 492)
        ic.drawn(leaf(x, y, 90, 30, ic.rnd.uniform(0, 6.28)), (70, 128, 54) if i % 2 else (112, 96, 52), hatch=0.5, highlight=0.1, edge_w=4)


def t_snow(ic):
    ground(ic, (226, 236, 246), var=0.08, speck=[(250, 252, 255), (190, 208, 230)], speck_n=50)


def t_sand(ic):
    ground(ic, (226, 196, 140), var=0.12, speck=[(200, 168, 110), (240, 220, 170), (170, 140, 100)], speck_n=90, angle=10)


def t_mud(ic):
    ground(ic, (86, 80, 52), speck=[(64, 60, 40), (110, 110, 64)], speck_n=50)
    ic.drawn(blob(300, 260, 110, 60, seed=3), (74, 92, 88), hatch=0.5, shade=0.15, highlight=0.25, edge_w=5)


def t_fungal(ic):
    ground(ic, (70, 52, 104), speck=[(96, 70, 140), (48, 36, 76)], speck_n=50)
    d = ImageDraw.Draw(ic.img)
    for _ in range(40):
        x, y, r = ic.rnd.uniform(0, 512), ic.rnd.uniform(0, 512), ic.rnd.uniform(5, 10)
        d.ellipse((x - r, y - r, x + r, y + r), fill=(120, 240, 210, 255))
        d.ellipse((x - r / 2.5, y - r / 2.5, x + r / 2.5, y + r / 2.5), fill=(230, 255, 245, 255))


def t_crystal_ground(ic):
    ground(ic, (178, 170, 204), var=0.14, speck=[(210, 204, 236), (140, 130, 170), (150, 220, 236)], speck_n=60)


def t_kelp_ground(ic):
    ground(ic, (70, 120, 116), speck=[(52, 92, 90), (104, 150, 132), (180, 196, 150)], speck_n=70, angle=0)


def t_side_dirt(ic):
    ic.flat(FULL, (120, 84, 54), wobble=0)
    for y, c in ((60, (104, 72, 46)), (180, (134, 96, 62)), (300, (110, 76, 50)), (430, (128, 92, 58))):
        ic.scribble(rect(-20, y - 40, 532, y + 40), c, var=0.2, angle=0, density=1.2, length=(60, 140), spread=6, width=7)
    d = ImageDraw.Draw(ic.img)
    for _ in range(30):
        x, y, r = ic.rnd.uniform(0, 512), ic.rnd.uniform(0, 512), ic.rnd.uniform(6, 14)
        d.ellipse((x - r, y - r * 0.7, x + r, y + r * 0.7), fill=(150, 140, 128, 255))


def t_side_rock(ic):
    ic.flat(FULL, (126, 124, 130), wobble=0)
    ic.scribble(FULL, (126, 124, 130), var=0.22, angle=5, density=1.1, length=(60, 140), spread=8, width=8)
    for y in (100, 250, 400):
        ic.line([(-10, y), (180, y + 14), (330, y - 6), (522, y + 8)], (86, 84, 92), 6, amp=3)


def t_side_sand(ic):
    ic.flat(FULL, (210, 176, 120), wobble=0)
    ic.scribble(FULL, (210, 176, 120), var=0.16, angle=0, density=1.0, length=(60, 150), spread=5, width=8)


def t_rock(ic):
    ic.flat(FULL, (132, 130, 136), wobble=0)
    ic.scribble(FULL, (132, 130, 136), var=0.24, angle=35, density=1.0, length=(40, 110), spread=14)
    ic.scribble(FULL, (132, 130, 136), var=0.16, angle=95, density=0.4, length=(30, 80), spread=18)
    for _ in range(5):
        x, y = ic.rnd.uniform(40, 470), ic.rnd.uniform(40, 470)
        ic.line([(x, y), (x + 40, y + 30), (x + 60, y + 80)], (90, 88, 96), 6, amp=2)
    for _ in range(4):
        ic.drawn(blob(ic.rnd.uniform(60, 450), ic.rnd.uniform(60, 450), 50, 30, seed=ic.rnd.randrange(99)), (110, 140, 70), hatch=0.6, highlight=0.1, edge=None)


def t_crystal(ic):
    ic.flat(FULL, (120, 210, 230), wobble=0)
    for k in range(6):
        x = k * 90 - 20
        ic.drawn([(x, -20), (x + 110, -20), (x + 60, 540), (x - 30, 540)], mix((120, 210, 230), (190, 140, 240), (k % 3) / 3), hatch=0.6, shade=0.25, highlight=0.25, edge_w=5)
    ic.dabs(rect(0, 0, 512, 512), (245, 255, 255), n=10, size=14)


def bark(ic, base, furrow, plate=None, vertical=True):
    ic.flat(FULL, base, wobble=0)
    ic.scribble(FULL, base, var=0.22, angle=90, density=1.2, length=(70, 160), spread=6, width=8)
    for k in range(7):
        x = k * 76 + ic.rnd.uniform(-10, 10)
        pts = [(x + ic.rnd.uniform(-12, 12), y) for y in range(-20, 560, 60)]
        ic.line(pts, furrow, 9, amp=4)
    if plate:
        for _ in range(9):
            x, y = ic.rnd.uniform(0, 512), ic.rnd.uniform(0, 512)
            ic.drawn(blob(x, y, 30, 56, seed=ic.rnd.randrange(99), amp=0.08), plate, hatch=0.5, highlight=0.2, edge_w=4)


def t_bark_oak(ic):
    bark(ic, (112, 80, 54), (62, 42, 30))


def t_bark_birch(ic):
    ic.flat(FULL, (232, 228, 214), wobble=0)
    ic.scribble(FULL, (232, 228, 214), var=0.1, angle=0, density=1.0, length=(60, 140), spread=6)
    for _ in range(26):
        x, y = ic.rnd.uniform(0, 512), ic.rnd.uniform(0, 512)
        L = ic.rnd.uniform(30, 80)
        ic.line([(x, y), (x + L, y + ic.rnd.uniform(-4, 4))], (44, 40, 40), ic.rnd.randrange(6, 12), amp=1.5)


def t_bark_pine(ic):
    bark(ic, (128, 70, 44), (70, 36, 26), plate=(156, 92, 58))


def t_bark_palm(ic):
    ic.flat(FULL, (150, 120, 80), wobble=0)
    for k in range(9):
        y = k * 64 - 10
        ic.drawn([(-20, y), (532, y + 10), (532, y + 46), (-20, y + 40)], (150, 120, 80) if k % 2 else (126, 98, 64), hatch=0.6, highlight=0.15, edge_w=5)


def t_bark_kapok(ic):
    bark(ic, (150, 140, 116), (98, 92, 76))


def t_bark_fungal(ic):
    ic.flat(FULL, (220, 210, 230), wobble=0)
    ic.scribble(FULL, (220, 210, 230), var=0.16, angle=90, density=1.2, length=(80, 160), spread=5)
    for k in range(9):
        x = k * 60
        ic.line([(x, -10), (x + 8, 520)], (170, 140, 200), 6, amp=3)


def t_bark_kelp(ic):
    bark(ic, (92, 110, 46), (60, 74, 32))


def t_bark_cactus(ic):
    ic.flat(FULL, (78, 132, 76), wobble=0)
    for k in range(6):
        x = k * 90 - 10
        ic.drawn(rect(x, -20, x + 70, 540), (88, 148, 84), hatch=0.6, shade=0.25, highlight=0.2, edge_w=6)
    d = ImageDraw.Draw(ic.img)
    for _ in range(50):
        x, y = ic.rnd.uniform(0, 512), ic.rnd.uniform(0, 512)
        d.line((x, y, x + 10, y - 12), fill=(246, 240, 200, 255), width=3)


def cap(ic, base, spots, glow=False):
    ic.flat(FULL, base, wobble=0)
    ic.scribble(FULL, base, var=0.2, angle=40, density=1.0, length=(40, 110), spread=14)
    for _ in range(14):
        x, y = ic.rnd.uniform(20, 492), ic.rnd.uniform(20, 492)
        ic.drawn(blob(x, y, 26, 22, seed=ic.rnd.randrange(99)), spots, hatch=0.4, shade=0.1, highlight=0.0, edge_w=4)


def t_cap_red(ic):
    cap(ic, (206, 50, 40), (250, 244, 230))


def t_cap_brown(ic):
    cap(ic, (150, 100, 60), (200, 160, 110))


def t_cap_glow(ic):
    cap(ic, (110, 70, 190), (140, 255, 220), glow=True)


# ---------- cards -------------------------------------------------------------------

def leaf_cluster(ic, cols, n=14, L=(90, 130), W=(30, 44), tip=0.8, seed=0, twig=(92, 66, 44), spread=1.0):
    rnd = random.Random(seed)
    # twigs first, then the leaves round them
    ic.line([(256, 500), (250, 330), (230, 180)], twig, 14, amp=3)
    ic.line([(250, 360), (360, 250)], twig, 10, amp=2)
    ic.line([(246, 300), (140, 210)], twig, 10, amp=2)
    for i in range(n):
        a = rnd.uniform(0, 2 * math.pi)
        r = math.sqrt(rnd.uniform(0.02, 1.0)) * 175 * spread
        x, y = 256 + math.cos(a) * r * 0.9, 240 + math.sin(a) * r * 0.8
        ang = math.atan2(y - 300, x - 256) + rnd.uniform(-0.5, 0.5)
        col = cols[i % len(cols)]
        ic.drawn(leaf(x, y, rnd.uniform(*L), rnd.uniform(*W), ang, tip), col, hatch=0.7, shade=0.25, highlight=0.18, edge_w=6)
        ic.line(leaf(x, y, rnd.uniform(*L) * 0.7, 0.1, ang)[:8], darker(col, 0.25), 3, amp=0.5)


def c_leaf_broad(ic):
    leaf_cluster(ic, [(84, 150, 60), (104, 168, 64), (70, 132, 54)], n=30, seed=1, spread=1.15)


def c_leaf_birch(ic):
    leaf_cluster(ic, [(150, 196, 80), (128, 180, 70), (170, 206, 96)], n=40, L=(60, 80), W=(28, 36), tip=0.6, seed=2, spread=1.2, twig=(70, 60, 56))


def c_leaf_acacia(ic):
    leaf_cluster(ic, [(128, 146, 64), (110, 132, 56)], n=60, L=(44, 60), W=(14, 20), seed=3, twig=(92, 70, 50), spread=1.3)


def c_needle(ic):
    ic.line([(256, 500), (256, 40)], (96, 60, 40), 14, amp=2)
    for k in range(22):
        y = 470 - k * 20
        for s in (-1, 1):
            L = 120 * (1 - k / 26) + 30
            ic.line([(256, y), (256 + s * L, y - 40)], (44, 98, 70) if k % 2 else (58, 116, 82), 9, amp=1.5)
            ic.line([(256, y + 6), (256 + s * L * 0.8, y - 20)], (34, 78, 58), 6, amp=1.5)


def c_leaf_jungle(ic):
    for i, (x, y, a, c) in enumerate([(256, 470, -1.9, (58, 136, 60)), (256, 470, -1.2, (70, 150, 64)), (256, 470, -2.5, (50, 120, 56))]):
        p = leaf(x, y, 330, 110, a, 0.7)
        ic.drawn(p, c, hatch=0.8, shade=0.25, highlight=0.2, edge_w=7)
        tip = (x + math.cos(a) * 320, y + math.sin(a) * 320)
        ic.line([(x, y), tip], lighter(c, 0.3), 5, amp=1)
        for k in range(1, 6):  # splits
            t = k / 6
            px, py = x + math.cos(a) * 330 * t, y + math.sin(a) * 330 * t
            for s in (-1, 1):
                ax = a + s * 1.2
                ic.line([(px + math.cos(ax) * 60, py + math.sin(ax) * 60), (px + math.cos(ax) * 110, py + math.sin(ax) * 110)], (0, 0, 0), 2, amp=0)
    # cut the splits back to transparent
    a = ic.img.getchannel("A")
    px = ic.img.load()
    for y in range(512):
        for x in range(512):
            r, g, b, al = px[x, y]
            if al and r == 0 and g == 0 and b == 0:
                px[x, y] = (0, 0, 0, 0)


def c_leaf_palm(ic):
    spine = curve((40, 470), (230, 120), (490, 160), 18)
    ic.line(spine, (120, 120, 60), 10, amp=1)
    for i, (x, y) in enumerate(spine[1:-1]):
        for s in (-1, 1):
            a = math.atan2(spine[i + 2][1] - y, spine[i + 2][0] - x) + s * 1.1
            L = 120 * math.sin(math.pi * (i + 1) / 18) + 30
            ic.drawn(leaf(x, y, L, 13, a), (88, 150, 60) if i % 2 else (104, 164, 66), hatch=0.4, shade=0.15, highlight=0.0, edge_w=4)


def c_leaf_willow(ic):
    for k in range(9):
        x = 60 + k * 50
        pts = curve((x, 20), (x + 20, 240), (x - 10, 500), 10)
        ic.line(pts, (110, 120, 70), 5, amp=1.5)
        for j, (px, py) in enumerate(pts[1:]):
            for s in (-1, 1):
                ic.drawn(leaf(px, py, 46, 9, math.pi / 2 + s * 0.6), (128, 170, 82) if (j + k) % 2 else (108, 152, 74), hatch=0.3, shade=0.0, highlight=0.0, edge_w=3)


def c_grass(ic, cols=((96, 160, 60), (120, 180, 70), (78, 138, 52)), n=13, h=(330, 470), lean=60, w=34, seed=7):
    rnd = random.Random(seed)
    for i in range(n):
        x = 60 + (392 * i / (n - 1)) + rnd.uniform(-14, 14)
        H = rnd.uniform(*h)
        tipx = x + rnd.uniform(-lean, lean)
        pts = curve((x, 512), ((x + tipx) / 2 + rnd.uniform(-20, 20), 512 - H * 0.6), (tipx, 512 - H), 10)
        col = cols[i % len(cols)]
        ic.drawn(strip(pts, w, 2), col, hatch=0.5, shade=0.3, highlight=0.25, edge_w=5, angle=80)


def c_grass_green(ic):
    c_grass(ic)


def c_grass_dry(ic):
    c_grass(ic, cols=((210, 180, 96), (190, 160, 80), (226, 204, 120)), n=15, w=26, lean=90, seed=8)


def c_grass_alien(ic):
    c_grass(ic, cols=((150, 100, 210), (110, 200, 190), (170, 120, 230)), n=11, w=38, lean=110, seed=9)


def c_reed(ic):
    c_grass(ic, cols=((110, 140, 70), (96, 126, 62)), n=7, h=(440, 500), lean=20, w=22, seed=10)
    for x in (130, 250, 380):
        ic.drawn(ellipse(x + 5, 120, 18, 60), (110, 70, 44), hatch=0.6, highlight=0.2, edge_w=5)


def c_kelp(ic):
    pts = [(256 + 60 * math.sin(i * 0.7), 512 - i * 34) for i in range(15)]
    ic.drawn(strip(pts, 90, 40), (120, 140, 50), hatch=0.6, shade=0.25, highlight=0.25, edge_w=6, angle=85)
    for i in (3, 7, 11):
        x, y = pts[i]
        ic.drawn(circle(x + 40, y, 20), (190, 170, 70), hatch=0.4, highlight=0.3, edge_w=5)


def flower(ic, petal, centre, n=6, r=150, pr=80, shape=0.6):
    for k in range(n):
        a = 2 * math.pi * k / n + 0.3
        ic.drawn(leaf(256, 256, r + 40, pr * 0.55, a, shape), petal, hatch=0.6, shade=0.22, highlight=0.2, edge_w=6)
    ic.drawn(circle(256, 256, 56), centre, hatch=0.7, shade=0.3, highlight=0.25, edge_w=6)
    d = ImageDraw.Draw(ic.img)
    for _ in range(14):
        a, rr = ic.rnd.uniform(0, 6.28), ic.rnd.uniform(5, 38)
        x, y = 256 + math.cos(a) * rr, 256 + math.sin(a) * rr
        d.ellipse((x - 5, y - 5, x + 5, y + 5), fill=darker(centre, 0.4) + (255,))


def c_flower_red(ic):
    flower(ic, (220, 52, 44), (40, 30, 30), n=5, pr=130, shape=0.5)


def c_flower_yellow(ic):
    flower(ic, (250, 204, 50), (200, 110, 30), n=8, pr=70)


def c_flower_blue(ic):
    flower(ic, (90, 120, 230), (250, 230, 120), n=5, pr=110, shape=0.55)


def c_flower_white(ic):
    flower(ic, (246, 244, 236), (240, 190, 40), n=12, pr=46, r=160)


def c_flower_glow(ic):
    ic.glow((256, 256, 150), (120, 255, 220), radius=30, strength=0.7)
    ic.drawn(circle(256, 256, 120), (110, 240, 210), hatch=0.6, shade=0.25, highlight=0.3, rim_color=(230, 255, 250), edge_w=7)
    ic.sparkle(330, 190, 40, (240, 255, 250))


def c_fern(ic):
    spine = curve((80, 500), (200, 160), (470, 80), 22)
    ic.line(spine, (70, 110, 50), 8, amp=1)
    for i, (x, y) in enumerate(spine[1:-1]):
        nxt = spine[i + 2]
        for s in (-1, 1):
            a = math.atan2(nxt[1] - y, nxt[0] - x) + s * 1.25
            L = 110 * (1 - i / 22) + 20
            ic.drawn(leaf(x, y, L, 16, a, 0.9), (70, 140, 60) if i % 2 else (84, 156, 66), hatch=0.3, shade=0.2, highlight=0.0, edge_w=3)


def c_vine(ic):
    pts = [(256 + 40 * math.sin(i * 0.9), i * 36) for i in range(15)]
    ic.line(pts, (80, 96, 50), 9, amp=2)
    for i, (x, y) in enumerate(pts[1:]):
        s = 1 if i % 2 else -1
        ic.drawn(leaf(x, y, 80, 28, (0.3 if s > 0 else math.pi - 0.3) + 0.5), (76, 140, 60) if i % 3 else (96, 156, 64), hatch=0.4, shade=0.2, highlight=0.1, edge_w=4)


def c_litter(ic):
    for i in range(16):
        x, y = ic.rnd.uniform(80, 432), ic.rnd.uniform(80, 432)
        col = [(196, 120, 52), (168, 84, 40), (210, 160, 70), (140, 100, 56)][i % 4]
        ic.drawn(leaf(x, y, 90, 30, ic.rnd.uniform(0, 6.28)), col, hatch=0.5, shade=0.25, highlight=0.12, edge_w=5)
    ic.line([(120, 380), (400, 300)], (100, 74, 50), 9, amp=2)


def c_twig(ic):
    def br(x, y, a, L, w, d):
        if d == 0 or L < 20:
            return
        x2, y2 = x + math.cos(a) * L, y + math.sin(a) * L
        ic.line([(x, y), (x2, y2)], (120, 96, 72), w, amp=2, ink=(70, 52, 40), ink_w=5)
        br(x2, y2, a - 0.45 + ic.rnd.uniform(-0.2, 0.2), L * 0.7, max(3, w - 3), d - 1)
        br(x2, y2, a + 0.45 + ic.rnd.uniform(-0.2, 0.2), L * 0.7, max(3, w - 3), d - 1)
    br(256, 510, -math.pi / 2, 170, 16, 5)


# ---------- impostors and icons -----------------------------------------------------

def mound(ic, col):
    ic.drawn(ellipse(256, 470, 190, 40), col, hatch=0.6, shade=0.2, highlight=0.15, edge_w=6)


def trunk(ic, x0, y0, x1, y1, w0, w1, col, edge=None):
    a = math.atan2(y1 - y0, x1 - x0)
    nx, ny = -math.sin(a), math.cos(a)
    p = [(x0 + nx * w0, y0 + ny * w0), (x1 + nx * w1, y1 + ny * w1), (x1 - nx * w1, y1 - ny * w1), (x0 - nx * w0, y0 - ny * w0)]
    ic.drawn(p, col, hatch=0.6, shade=0.3, highlight=0.12, edge_w=6, angle=85)


def canopy(ic, blobs, col, seed=0):
    for i, (x, y, rx, ry) in enumerate(blobs):
        c = mix(col, darker(col, 0.3), 0.6) if i < len(blobs) // 3 else col
        ic.drawn(blob(x, y, rx, ry, seed=seed + i, amp=0.14, bumps=8), c, hatch=0.8, shade=0.28, highlight=0.2, edge_w=7)


def s_oak(ic, icon):
    trunk(ic, 256, 512, 250, 260, 30, 20, (112, 80, 54))
    canopy(ic, [(170, 250, 90, 70), (340, 250, 90, 70), (256, 170, 130, 110), (200, 290, 80, 50), (320, 300, 80, 50)], (84, 150, 60))


def s_birch(ic, icon):
    trunk(ic, 256, 512, 262, 120, 18, 10, (232, 228, 214))
    for y in (450, 380, 300, 220):
        ic.line([(244, y), (262, y + 4)], (40, 36, 36), 6, amp=1)
    canopy(ic, [(256, 210, 90, 140), (220, 270, 60, 70), (300, 250, 60, 80)], (150, 196, 80), seed=4)


def s_pine(ic, icon):
    trunk(ic, 256, 512, 256, 420, 18, 14, (128, 70, 44))
    for k in range(6):
        y = 440 - k * 66
        w = 190 - k * 30
        ic.drawn([(256 - w, y), (256 + w, y), (256, y - 120)], (44, 98, 70) if k % 2 else (58, 116, 82), hatch=0.7, shade=0.25, highlight=0.15, edge_w=6)
    if icon:
        for x, y in ((180, 330), (330, 260), (240, 200)):
            ic.dot(x, y, 9, (246, 250, 255))


def s_kapok(ic, icon):
    trunk(ic, 256, 512, 256, 190, 34, 22, (150, 140, 116))
    for s in (-1, 1):
        ic.drawn([(256 + s * 20, 400), (256 + s * 100, 512), (256 + s * 30, 512)], (150, 140, 116), hatch=0.5, highlight=0.0, edge_w=5)
        ic.line([(256, 250), (256 + s * 110, 170)], (150, 140, 116), 16, amp=2, ink=tone((150, 140, 116)), ink_w=6)
    canopy(ic, [(256, 130, 200, 60), (150, 160, 90, 50), (360, 160, 90, 50), (256, 95, 120, 45)], (62, 128, 56), seed=7)


def s_palm(ic, icon):
    pts = curve((250, 512), (230, 300), (300, 120), 12)
    ic.drawn(strip(pts, 40, 24), (150, 120, 80), hatch=0.5, shade=0.3, highlight=0.12, edge_w=6, angle=80)
    for a in (-2.9, -2.3, -1.6, -0.9, -0.3, 0.3, 2.6):
        x, y = 300, 120
        end = (x + math.cos(a) * 190, y + math.sin(a) * 120 + 70)
        mid = (x + math.cos(a) * 110, y + math.sin(a) * 90 - 30)
        ic.drawn(strip(curve((x, y), mid, end, 10), 50, 6), (92, 156, 62), hatch=0.5, shade=0.3, highlight=0.15, edge_w=5)
    ic.drawn(circle(300, 132, 22), (110, 80, 40), hatch=0.4, highlight=0.25, edge_w=5)


def s_acacia(ic, icon):
    trunk(ic, 256, 512, 250, 300, 20, 14, (96, 74, 52))
    ic.line([(250, 320), (160, 210)], (96, 74, 52), 14, amp=2, ink=tone((96, 74, 52)), ink_w=6)
    ic.line([(250, 330), (360, 200)], (96, 74, 52), 14, amp=2, ink=tone((96, 74, 52)), ink_w=6)
    canopy(ic, [(256, 190, 220, 44), (170, 170, 90, 34), (350, 175, 100, 36)], (128, 146, 64), seed=11)


def s_willow(ic, icon):
    trunk(ic, 256, 512, 250, 230, 32, 20, (96, 84, 60))
    canopy(ic, [(256, 180, 170, 90)], (120, 160, 80), seed=13)
    for k in range(11):
        x = 110 + k * 29
        ic.drawn(strip(curve((x, 180), (x + 4, 300), (x - 8, 420 - abs(k - 5) * 18), 8), 30, 8), (112, 156, 74) if k % 2 else (128, 170, 82), hatch=0.3, shade=0.25, highlight=0.0, edge_w=4)


def s_cactus(ic, icon):
    col = (78, 138, 76)
    ic.drawn(strip([(256, 512), (256, 120)], 74, 66) + [], col, hatch=0.6, shade=0.3, highlight=0.2, edge_w=7, angle=88)
    ic.drawn(strip([(225, 330), (150, 330), (150, 210)], 50, 44), col, hatch=0.5, shade=0.3, highlight=0.15, edge_w=6)
    ic.drawn(strip([(290, 280), (360, 280), (360, 170)], 50, 44), col, hatch=0.5, shade=0.3, highlight=0.15, edge_w=6)
    ic.drawn(circle(256, 118, 20), (240, 90, 120), hatch=0.4, highlight=0.3, edge_w=4)


def s_glowcap(ic, icon):
    ic.glow((256, 200, 150), (120, 255, 220), radius=26, strength=0.45)
    trunk(ic, 256, 512, 256, 200, 30, 22, (220, 210, 230))
    ic.drawn([(256 + 200 * math.cos(t), 220 - 150 * abs(math.sin(t)) ** 0.8) for t in [math.pi * i / 30 for i in range(31)]], (110, 70, 190), hatch=0.8, shade=0.3, highlight=0.22, rim_color=(150, 255, 220), edge_w=7)
    for x, y, r in ((180, 160, 18), (260, 110, 22), (330, 170, 16), (220, 200, 12), (300, 205, 12)):
        ic.dot(x, y, r, (150, 255, 220))
        ic.dot(x - r / 3, y - r / 3, r / 3, (240, 255, 250))


def s_crystal(ic, icon):
    ic.glow((256, 280, 120), (150, 220, 255), radius=26, strength=0.35)
    for x, top, w, c in ((256, 60, 60, (130, 210, 240)), (180, 200, 40, (180, 150, 240)), (336, 170, 46, (120, 220, 220)), (130, 330, 30, (150, 200, 250)), (390, 320, 30, (190, 160, 240))):
        ic.drawn([(x, top), (x + w, top + w * 1.4), (x + w * 0.7, 512), (x - w * 0.7, 512), (x - w, top + w * 1.4)], c, hatch=0.6, shade=0.3, highlight=0.3, rim_color=lighter(c, 0.7), edge_w=6)
    ic.sparkle(300, 110, 30, (250, 255, 255))


def s_kelp(ic, icon):
    for k, x in enumerate((180, 256, 330)):
        pts = [(x + 40 * math.sin(i * 0.6 + k), 512 - i * 33) for i in range(14 - k)]
        ic.drawn(strip(pts, 40, 16), (120, 140, 50) if k % 2 else (100, 128, 44), hatch=0.5, shade=0.3, highlight=0.2, edge_w=5, angle=85)
        for i in range(3, len(pts), 3):
            px, py = pts[i]
            ic.drawn(circle(px + 26, py, 15), (196, 176, 70), hatch=0.3, highlight=0.3, edge_w=4)


SPECIES = {
    "oak": (s_oak, (98, 150, 60)), "birch": (s_birch, (98, 150, 60)), "pine": (s_pine, (226, 236, 246)),
    "kapok": (s_kapok, (58, 104, 50)), "palm": (s_palm, (58, 104, 50)), "acacia": (s_acacia, (196, 168, 92)),
    "willow": (s_willow, (86, 80, 52)), "cactus": (s_cactus, (226, 196, 140)), "glowcap": (s_glowcap, (70, 52, 104)),
    "crystal": (s_crystal, (178, 170, 204)), "kelp": (s_kelp, (70, 120, 116)),
}

TILES = {
    "ground_grass": t_grass, "ground_dry": t_dry, "ground_litter": t_litter, "ground_jungle": t_jungle,
    "ground_snow": t_snow, "ground_sand": t_sand, "ground_mud": t_mud, "ground_fungal": t_fungal,
    "ground_crystal": t_crystal_ground, "ground_kelp": t_kelp_ground,
    "side_dirt": t_side_dirt, "side_rock": t_side_rock, "side_sand": t_side_sand,
    "bark_oak": t_bark_oak, "bark_birch": t_bark_birch, "bark_pine": t_bark_pine, "bark_palm": t_bark_palm,
    "bark_kapok": t_bark_kapok, "bark_fungal": t_bark_fungal, "bark_kelp": t_bark_kelp, "bark_cactus": t_bark_cactus,
    "rock": t_rock, "crystal": t_crystal, "cap_red": t_cap_red, "cap_brown": t_cap_brown, "cap_glow": t_cap_glow,
}

CARDS = {
    "leaf_broad": c_leaf_broad, "leaf_birch": c_leaf_birch, "leaf_acacia": c_leaf_acacia, "needle": c_needle,
    "leaf_jungle": c_leaf_jungle, "leaf_palm": c_leaf_palm, "leaf_willow": c_leaf_willow,
    "grass_green": c_grass_green, "grass_dry": c_grass_dry, "grass_alien": c_grass_alien, "reed": c_reed, "kelp": c_kelp,
    "flower_red": c_flower_red, "flower_yellow": c_flower_yellow, "flower_blue": c_flower_blue,
    "flower_white": c_flower_white, "flower_glow": c_flower_glow,
    "fern": c_fern, "vine": c_vine, "litter": c_litter, "twig": c_twig,
}


def save_tile(ic, path):
    im = ic.img.resize((128, 128), Image.LANCZOS)
    im.putalpha(255)
    im = wrap(im)
    meta = PngInfo()
    meta.add_text("source", "armory-crayon")
    im.save(path, pnginfo=meta)


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    only = None
    if "--only" in argv:
        only = set(argv[argv.index("--only") + 1].split(","))
    written = []
    seed = 500
    for name, fn in TILES.items():
        seed += 1
        if only and name not in only:
            continue
        ic = Icon(seed=seed)
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        save_tile(ic, p)
        written.append(p)
    for name, fn in CARDS.items():
        seed += 1
        if only and name not in only:
            continue
        ic = Icon(seed=seed)
        fn(ic)
        p = os.path.join(OUT, name + ".png")
        ic.save(p)
        written.append(p)
    for name, (fn, ground_col) in SPECIES.items():
        seed += 1
        for kind in ("imp", "icon"):
            if only and name not in only and f"{kind}_{name}" not in only:
                continue
            ic = Icon(seed=seed)
            if kind == "icon":
                fn(ic, True)
                im = ic.img
                # shrink the tree onto a mound so it reads as a picker icon
                small = im.resize((400, 400), Image.LANCZOS)
                ic.img = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
                mound(ic, ground_col)
                ic.img.alpha_composite(small, (56, 50))
            else:
                fn(ic, False)
            p = os.path.join(OUT, f"{kind}_{name}.png")
            ic.save(p)
            written.append(p)
    print("wrote", len(written), "pngs to", OUT)
    if "--sheet" in argv:
        out = argv[argv.index("--sheet") + 1]
        import sheet
        sheet.main([out] + written + ["--ref", "vitality", "--ref", "plating", "--ref", "inferno"])


if __name__ == "__main__":
    main(sys.argv[1:])
