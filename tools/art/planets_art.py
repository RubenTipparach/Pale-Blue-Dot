"""Paint the art for docs/mockups/planets.html in the friend's hand-drawn manner.

    python tools/art/planets_art.py            # every PNG into docs/mockups/planets/
    python tools/art/planets_art.py --sheet    # also a contact sheet beside the friend's icons

The crayon kit (.claude/skills/hand-drawn-illustration/scripts/crayon.py) is
used as it is. Three kinds of picture come out:

- `map_<body>.png`, 1024 x 512 equirectangular surface maps for the three.js
  spheres, painted on a 1024 canvas (the kit's canvas size is a module global,
  set here for the maps and put back after). Shapes near the seam are painted
  twice, a world apart, and the overlap is cross-faded so the sphere has no
  seam at longitude 0.
- `lights_<body>.png`, the same size, black with the night lights (towns,
  lava, glowing caves), added on the dark side of the sphere.
- `portrait_<body>.png` and `res_<resource>.png`, 128 px icons in the friend's
  own format: a transparent ground, no frame.

Everything here is painted by code copying the friend's method. None of it is
the friend's drawing.
"""

import math
import os
import random
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)

import crayon  # noqa: E402
from crayon import Icon, circle, ellipse, rect, star, rotate, translate, scale, mix, darker, lighter, tone  # noqa: E402
from PIL import Image, ImageChops, ImageDraw, ImageFilter  # noqa: E402
from PIL.PngImagePlugin import PngInfo  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "planets")
MW = 1024          # the map canvas
WRAP = 896         # columns that are one full turn of longitude; the rest overlaps the start


def meta():
    m = PngInfo()
    m.add_text("source", "pbd-crayon (tools/art/planets_art.py)")
    return m


def blob(cx, cy, r, seed, n=44, rough=0.30, aspect=1.0, rot=0.0):
    """A coastline: a circle pushed in and out by a few random harmonics."""
    rnd = random.Random(seed)
    harm = [(k, rough / k * rnd.uniform(0.5, 1.2), rnd.uniform(0, 6.28)) for k in range(2, 7)]
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        rr = r * (1 + sum(amp * math.sin(k * a + ph) for k, amp, ph in harm))
        pts.append((rr * math.cos(a) * aspect, rr * math.sin(a)))
    return translate(rotate(pts, rot), cx, cy)


def band(y0, y1, seed, amp=14, waves=(3, 7, 13)):
    """A horizontal band right round the map with wavy top and bottom edges."""
    rnd = random.Random(seed)
    ph = [rnd.uniform(0, 6.28) for _ in range(6)]
    top, bot = [], []
    for i in range(0, 41):
        x = -60 + (MW + 120) * i / 40
        u = 2 * math.pi * x / WRAP
        top.append((x, y0 + amp * sum(math.sin(w * u + ph[k]) / (k + 1) for k, w in enumerate(waves))))
        bot.append((x, y1 + amp * sum(math.sin(w * u + ph[k + 3]) / (k + 1) for k, w in enumerate(waves))))
    return top + bot[::-1]


def wiggle(x0, y0, length, ang, seed, n=9, jit=26):
    rnd = random.Random(seed)
    pts = []
    for i in range(n):
        t = i / (n - 1)
        a = ang + rnd.uniform(-0.5, 0.5)
        pts.append((x0 + math.cos(ang) * length * t + rnd.uniform(-jit, jit) * math.sin(a),
                    y0 + math.sin(ang) * length * t + rnd.uniform(-jit, jit) * math.cos(a)))
    return pts


# ---------- maps -------------------------------------------------------------------

class Map:
    """One surface map. Every call paints the shape, and again a turn of
    longitude to the left or right when it reaches past the seam."""

    def __init__(self, seed, base, angle=0, spread=10):
        crayon.S = MW
        self.ic = Icon(seed)
        self.rnd = random.Random(seed * 7 + 1)
        full = rect(-12, -12, MW + 12, MW + 12)
        self.ic.flat(full, base, wobble=0)
        self.ic.scribble(full, base, var=0.20, width=10, length=(70, 180), angle=angle, spread=spread, density=0.85)
        self.ic.scribble(full, base, var=0.12, width=7, length=(40, 110), angle=angle + 50, spread=20, density=0.25)
        self.lights = Image.new("RGBA", (MW, MW), (0, 0, 0, 255))
        self.land = []

    def copies(self, poly):
        xs = [p[0] for p in poly]
        out = [poly]
        if max(xs) - min(xs) >= WRAP:      # a band right round the map needs no copy
            return out
        if min(xs) < MW - WRAP:
            out.append(translate(poly, WRAP, 0))
        if max(xs) > WRAP:
            out.append(translate(poly, -WRAP, 0))
        return out

    def drawn(self, poly, color, shade=0.10, highlight=0.0, edge_w=9, angle=None, hatch=1.0, edge="tone"):
        for p in self.copies(poly):
            self.ic.drawn(p, color, shade=shade, highlight=highlight, edge_w=edge_w, angle=angle, hatch=hatch, edge=edge)

    def patch(self, poly, color, angle=None, edge=None, edge_w=6, density=0.9):
        """Flat colour with hatching, an optional thin edge: a biome inside a continent."""
        for p in self.copies(poly):
            q = self.ic.flat(p, color)
            self.ic.scribble(q, color, var=0.24, width=8, angle=angle if angle is not None else self.rnd.uniform(20, 70),
                             length=(40, 110), spread=14, density=density)
            if edge:
                self.ic.outline(q, edge, edge_w, passes=1)

    def line(self, pts, color, width=8):
        for p in self.copies(pts):
            self.ic.line(p, color, width)

    def crater(self, cx, cy, r, color):
        c = circle(cx, cy, r, n=28)
        for p in self.copies(c):
            self.ic.drawn(p, color, shade=0.0, highlight=0.0, edge_w=max(3, int(r / 7)), hatch=0.6)
            # the lit wall on the lower right, the shadowed wall upper left (a pit)
            self.ic.scribble_in(self.ic.crescent(p, (-1, -1), 0.35), darker(color, 0.35), width=5, density=1.3, length=(10, 30))
            self.ic.scribble_in(self.ic.crescent(p, (1, 1), 0.28), lighter(color, 0.4), width=4, density=1.1, length=(10, 30))

    def light(self, x, y, r, color, core=1.0):
        """A night light on the lights map: a soft glow and a bright core."""
        d = ImageDraw.Draw(self.lights)
        for xx in (x, x + WRAP, x - WRAP):
            if -40 < xx < MW + 40:
                g = Image.new("L", (MW, MW), 0)
                ImageDraw.Draw(g).ellipse((xx - r * 2.6, y - r * 2.6, xx + r * 2.6, y + r * 2.6), fill=120)
                g = g.filter(ImageFilter.GaussianBlur(r * 1.6))
                self.lights.paste(Image.new("RGBA", (MW, MW), color + (255,)), (0, 0), ImageChops.multiply(g, Image.new("L", (MW, MW), int(200 * core))))
                d.ellipse((xx - r * 0.55, y - r * 0.55, xx + r * 0.55, y + r * 0.55), fill=lighter(color, 0.5) + (255,))

    def towns(self, polys, n, seed, color=(255, 190, 110)):
        """Scatter towns on land: a few big clusters and many small ones."""
        rnd = random.Random(seed)
        m = Image.new("L", (MW, MW), 0)
        for p in polys:
            ImageDraw.Draw(m).polygon([(round(x), round(y)) for x, y in scale(p, 0.86, (sum(q[0] for q in p) / len(p), sum(q[1] for q in p) / len(p)))], fill=255)
        placed = 0
        tries = 0
        while placed < n and tries < n * 60:
            tries += 1
            x, y = rnd.uniform(0, WRAP), rnd.uniform(90, MW - 90)
            if m.getpixel((int(x), int(y))) > 0:
                big = rnd.random() < 0.18
                self.light(x, y, 9 if big else 4.5, color, 1.0 if big else 0.7)
                placed += 1

    def save(self, name):
        crayon.S = MW
        img = self.ic.halo.copy()
        img.alpha_composite(self.ic.img)
        finish(img).save(os.path.join(OUT, "map_" + name + ".png"), pnginfo=meta())
        if self.lights.getbbox() and ImageChops.difference(self.lights.convert("RGB"), Image.new("RGB", (MW, MW))).getbbox():
            finish(self.lights).save(os.path.join(OUT, "lights_" + name + ".png"), pnginfo=meta())
        crayon.S = 512
        print("map", name)


def finish(img):
    """Cross-fade the overlap onto the start of the turn, then squash to 2:1."""
    band_w = MW - WRAP
    a = img.crop((0, 0, WRAP, MW)).convert("RGBA")
    tail = img.crop((WRAP, 0, MW, MW)).convert("RGBA")
    head = a.crop((0, 0, band_w, MW))
    m = Image.new("L", (band_w, MW))
    dm = ImageDraw.Draw(m)
    for x in range(band_w):
        dm.line([(x, 0), (x, MW)], fill=int(255 * (1 - x / band_w)))
    a.paste(Image.composite(tail, head, m), (0, 0))
    return a.resize((1024, 512), Image.LANCZOS).convert("RGB")


def caps(mp, color, edge, top=70, bot=70, seed=1):
    mp.drawn(band(-60, top, seed, amp=18), color, shade=0.0, edge_w=8)
    mp.drawn(band(MW - bot, MW + 60, seed + 1, amp=18), color, shade=0.0, edge_w=8)


def clouds(mp, n, seed, color=(245, 248, 252), lat=(140, 880)):
    rnd = random.Random(seed)
    for i in range(n):
        x, y = rnd.uniform(0, WRAP), rnd.uniform(*lat)
        L = rnd.uniform(60, 170)
        pts = [(x + L * t, y + 18 * math.sin(t * 5 + i)) for t in [k / 7 for k in range(8)]]
        mp.line(pts, color, rnd.choice((7, 9, 11)))


def map_home():
    mp = Map(11, (38, 104, 186))
    conts = [blob(150, 360, 120, 1, aspect=0.9), blob(170, 620, 95, 2, aspect=0.8, rot=0.4), blob(420, 300, 150, 3, aspect=1.3),
             blob(520, 560, 90, 4), blob(720, 420, 135, 5, aspect=1.1, rot=-0.3), blob(820, 680, 70, 6)]
    for c in conts:
        mp.patch(scale(c, 1.12, (sum(p[0] for p in c) / len(c), sum(p[1] for p in c) / len(c))), (64, 146, 206), density=0.6)
    for c in conts:
        mp.drawn(c, (88, 168, 66), edge_w=9)
    mp.patch(blob(430, 400, 55, 21, aspect=1.6), (214, 182, 98), edge=(150, 110, 50))     # a desert
    mp.patch(blob(730, 470, 48, 22, aspect=1.4), (214, 182, 98), edge=(150, 110, 50))
    for cx, cy, r, s in ((150, 330, 55, 23), (420, 250, 60, 24), (700, 380, 50, 25), (180, 610, 40, 26)):
        mp.patch(blob(cx, cy, r, s), (42, 118, 58), edge=(22, 70, 34))                   # forest
    for x, y, L, a, s in ((360, 270, 140, 0.3, 31), (690, 360, 120, -0.6, 32), (110, 380, 90, 1.2, 33)):
        mp.line(wiggle(x, y, L, a, s), (128, 104, 82), 12)                               # ranges
        for px, py in wiggle(x, y, L, a, s)[1::2]:
            mp.ic.dot(px, py, 6, (246, 248, 252))
    caps(mp, (236, 244, 250), None, 64, 74, 41)
    clouds(mp, 26, 42)
    mp.towns(conts, 140, 43)
    mp.save("home")


def map_verdance():
    mp = Map(12, (26, 116, 128))
    isl = []
    rnd = random.Random(5)
    for i in range(16):
        cx, cy = rnd.uniform(0, WRAP), rnd.uniform(220, 800)
        isl.append(blob(cx, cy, rnd.uniform(30, 85), 100 + i, aspect=rnd.uniform(0.8, 1.6), rot=rnd.uniform(0, 3)))
    for c in isl:
        mp.patch(scale(c, 1.2, (sum(p[0] for p in c) / len(c), sum(p[1] for p in c) / len(c))), (52, 160, 150), density=0.6)
    for c in isl:
        mp.drawn(c, (40, 150, 58), edge_w=8)
    for c in isl[::2]:
        cx, cy = sum(p[0] for p in c) / len(c), sum(p[1] for p in c) / len(c)
        mp.patch(blob(cx + 6, cy, 22, int(cx)), (104, 122, 46), edge=(60, 72, 24), edge_w=4)  # swamp
    for c in isl[1::3]:
        cx, cy = sum(p[0] for p in c) / len(c), sum(p[1] for p in c) / len(c)
        mp.patch(blob(cx, cy - 4, 20, int(cy)), (24, 104, 44))                              # deep jungle
    caps(mp, (214, 236, 232), None, 44, 50, 51)
    clouds(mp, 40, 52, lat=(100, 920))
    mp.towns(isl, 60, 53, color=(255, 214, 140))
    mp.save("verdance")


def map_ochre():
    mp = Map(13, (192, 104, 58))
    for i, (cx, cy, r) in enumerate(((150, 400, 120), (480, 600, 150), (760, 330, 130), (330, 230, 80))):
        mp.patch(blob(cx, cy, r, 200 + i, aspect=1.3), (154, 72, 44), edge=(100, 40, 26))       # highlands
    for i, (cx, cy, r) in enumerate(((300, 480, 60), (620, 400, 45), (820, 620, 55))):
        mp.patch(blob(cx, cy, r, 210 + i, aspect=1.6), (210, 176, 94), edge=(150, 116, 52))     # steppe
    mp.patch(blob(560, 470, 34, 220, aspect=1.7), (238, 228, 204), edge=(170, 150, 120))       # salt flat
    sea = blob(380, 700, 70, 221, aspect=1.5)
    mp.drawn(sea, (58, 112, 146), edge_w=8)                                                     # an inland sea
    for x, y, L, a, s in ((100, 520, 260, -0.2, 61), (520, 300, 220, 0.5, 62), (700, 560, 200, -0.7, 63)):
        mp.line(wiggle(x, y, L, a, s, n=12, jit=30), (98, 38, 28), 13)                          # canyons
        mp.line(wiggle(x, y, L, a, s, n=12, jit=30), (226, 140, 92), 3)
    oases = [(300, 470), (640, 390), (560, 660), (180, 560), (820, 610)]
    for x, y in oases:
        mp.ic.dot(x, y, 12, (70, 160, 70))
        mp.ic.dot(x, y, 5, (70, 150, 200))
    caps(mp, (240, 234, 226), None, 34, 40, 64)
    clouds(mp, 8, 65, color=(250, 236, 220))
    for x, y in oases:
        mp.light(x, y, 7, (255, 186, 100))
    mp.towns([sea], 10, 66)
    mp.save("ochre")


def map_lantern():
    mp = Map(14, (150, 150, 162), angle=40, spread=40)
    for i, (cx, cy, r) in enumerate(((220, 380, 130), (520, 520, 110), (760, 340, 90), (380, 700, 70))):
        mp.patch(blob(cx, cy, r, 300 + i, aspect=1.2), (98, 98, 114), edge=(70, 70, 86))      # maria
    rnd = random.Random(9)
    for i in range(34):
        r = rnd.choice((10, 14, 18, 26, 40))
        mp.crater(rnd.uniform(0, WRAP), rnd.uniform(80, MW - 80), r, (136, 136, 150))
    for x in (120, 420, 700):
        mp.patch(blob(x, 40, 40, x), (220, 236, 248))                                           # polar ice in shadow
        mp.patch(blob(x + 140, MW - 40, 36, x + 1), (220, 236, 248))
    mp.save("lantern")


def map_rime():
    mp = Map(15, (208, 232, 244), angle=20)
    for i, (cx, cy, r) in enumerate(((200, 300, 140), (600, 640, 160), (760, 260, 90))):
        mp.patch(blob(cx, cy, r, 400 + i, aspect=1.4), (166, 208, 236))
    rnd = random.Random(15)
    for i in range(16):
        x, y = rnd.uniform(0, WRAP), rnd.uniform(120, 900)
        pts = wiggle(x, y, rnd.uniform(160, 360), rnd.uniform(-0.6, 0.6), 500 + i, n=10, jit=18)
        mp.line(pts, (70, 132, 196), 7)                                                         # the ice's cracks
        mp.line(pts, (240, 250, 255), 2)
    for i in range(8):
        mp.crater(rnd.uniform(0, WRAP), rnd.uniform(120, 900), rnd.choice((12, 18, 26)), (190, 220, 238))
    for x, y in ((300, 860), (420, 900), (520, 870)):                                           # geysers at the south pole
        mp.ic.dot(x, y, 10, (255, 255, 255))
    mp.save("rime")


def map_cinder():
    mp = Map(16, (228, 194, 66), angle=30, spread=30)
    rnd = random.Random(16)
    for i in range(10):
        mp.patch(blob(rnd.uniform(0, WRAP), rnd.uniform(120, 900), rnd.uniform(50, 110), 600 + i, aspect=1.3), rnd.choice(((232, 140, 52), (240, 232, 196), (196, 150, 40))))
    for i in range(9):
        mp.drawn(blob(rnd.uniform(0, WRAP), rnd.uniform(140, 880), rnd.uniform(30, 70), 620 + i, aspect=1.5, rough=0.45), (46, 34, 34), edge_w=6)  # black flows
    lakes = [(rnd.uniform(0, WRAP), rnd.uniform(160, 860), rnd.uniform(10, 22)) for _ in range(16)]
    for x, y, r in lakes:
        mp.drawn(circle(x, y, r * 1.6, n=24), (120, 40, 26), shade=0, edge_w=4, hatch=0.5)
        mp.drawn(circle(x, y, r, n=24), (255, 96, 30), shade=0, edge_w=3, hatch=0.6, edge=(255, 210, 80))
        mp.light(x, y, r * 0.9, (255, 110, 40))
    mp.save("cinder")


def map_murk():
    mp = Map(17, (198, 136, 58), angle=0, spread=6)
    ys = [0, 120, 230, 330, 420, 520, 600, 700, 800, 900, MW]
    cols = [(214, 158, 78), (186, 122, 52), (206, 148, 70), (176, 112, 48), (210, 154, 76)]
    for i in range(len(ys) - 1):
        if i % 2:
            mp.patch(band(ys[i], ys[i + 1], 700 + i, amp=10), cols[i % len(cols)], angle=0, density=0.8)
    for i, (x, y, r) in enumerate(((150, 110, 60), (420, 90, 80), (700, 130, 50), (300, 920, 70), (640, 900, 55))):
        mp.drawn(blob(x, y, r, 720 + i, aspect=1.6, rough=0.4), (58, 46, 44), edge_w=7)      # methane seas, dark
    mp.light(420, 260, 6, (255, 200, 120))                                                   # one outpost
    mp.save("murk")


def map_hollow():
    mp = Map(18, (112, 100, 124), angle=50, spread=30)
    rnd = random.Random(18)
    for i in range(8):
        mp.patch(blob(rnd.uniform(0, WRAP), rnd.uniform(100, 920), rnd.uniform(60, 120), 800 + i, aspect=1.4), (136, 124, 146))
    for i in range(14):
        x, y, r = rnd.uniform(0, WRAP), rnd.uniform(140, 880), rnd.uniform(14, 34)
        mp.drawn(blob(x, y, r, 820 + i, aspect=1.3), (34, 26, 44), shade=0, edge_w=5)         # sinkholes into the caves
        mp.ic.dot(x + 2, y + 2, r * 0.25, (178, 120, 240))
        mp.light(x, y, r * 0.5, (176, 112, 255), 0.8)
    for i in range(10):
        mp.crater(rnd.uniform(0, WRAP), rnd.uniform(100, 920), rnd.choice((10, 16, 22)), (122, 110, 134))
    mp.save("hollow")


def map_tidewater():
    mp = Map(19, (22, 64, 146), angle=15)
    rnd = random.Random(19)
    for i in range(9):
        mp.patch(blob(rnd.uniform(0, WRAP), rnd.uniform(160, 860), rnd.uniform(60, 130), 900 + i, aspect=1.8), (40, 98, 172), density=0.6)
    reef = blob(448, 512, 110, 950, aspect=1.2)                     # the face that always looks at the giant
    mp.patch(reef, (56, 164, 170), density=0.7)
    mp.patch(scale(reef, 0.7, (448, 512)), (32, 110, 170), density=0.6)
    isl = [blob(430 + 30 * k, 470 + 25 * math.sin(k), 18 + 4 * (k % 3), 960 + k) for k in range(5)]
    isl += [blob(rnd.uniform(0, WRAP), rnd.uniform(250, 780), rnd.uniform(14, 26), 970 + k) for k in range(6)]
    for c in isl:
        mp.drawn(c, (96, 168, 82), edge_w=5)
    caps(mp, (220, 236, 246), None, 50, 56, 71)
    clouds(mp, 30, 72)
    mp.towns(isl, 14, 73, color=(255, 210, 140))
    mp.save("tidewater")


def map_pebble():
    mp = Map(20, (122, 106, 94), angle=60, spread=40)
    rnd = random.Random(20)
    for i in range(6):
        mp.patch(blob(rnd.uniform(0, WRAP), rnd.uniform(100, 920), rnd.uniform(60, 140), 1000 + i), (98, 86, 78))
    for i in range(40):
        mp.crater(rnd.uniform(0, WRAP), rnd.uniform(80, 940), rnd.choice((8, 12, 16, 24, 36)), (116, 100, 90))
    for i in range(10):
        mp.ic.dot(rnd.uniform(0, WRAP), rnd.uniform(100, 920), 5, (196, 150, 96))           # metal glints
    mp.save("pebble")


def map_amber():
    mp = Map(21, (232, 206, 156), angle=0, spread=4)
    ys = [-40, 70, 150, 230, 300, 360, 430, 500, 560, 640, 720, 800, 880, 960, MW + 40]
    cols = [(206, 128, 66), (232, 206, 156), (170, 92, 52), (226, 178, 112), (214, 146, 80), (240, 222, 180), (180, 104, 58)]
    for i in range(len(ys) - 1):
        mp.patch(band(ys[i], ys[i + 1], 1100 + i, amp=9, waves=(4, 9, 17)), cols[i % len(cols)], angle=0, density=0.8)
    for i in range(8):                                                     # band edge eddies
        mp.ic.dot(random.Random(i).uniform(0, WRAP), ys[3 + i % 8], 9, (244, 230, 200))
    storm = ellipse(560, 620, 90, 48, n=40)
    mp.drawn(scale(storm, 1.35, (560, 620)), (238, 214, 172), shade=0, edge_w=6, angle=0)
    mp.drawn(storm, (204, 72, 50), shade=0.12, edge_w=7, angle=0)
    mp.drawn(scale(storm, 0.5, (560, 620)), (232, 120, 80), shade=0, edge_w=0, edge=None, angle=0)
    mp.save("amber")


def map_glaucus():
    mp = Map(22, (112, 192, 216), angle=0, spread=4)
    ys = [-40, 110, 260, 420, 600, 760, 900, MW + 40]
    cols = [(92, 168, 204), (132, 208, 226), (100, 180, 212)]
    for i in range(len(ys) - 1):
        if i % 2 == 0:
            mp.patch(band(ys[i], ys[i + 1], 1200 + i, amp=7, waves=(3, 5, 9)), cols[i % 3], angle=0, density=0.7)
    spot = ellipse(300, 600, 70, 34, n=36)
    mp.drawn(spot, (36, 74, 150), shade=0, edge_w=6, angle=0)
    for i in range(10):
        x = random.Random(50 + i).uniform(0, WRAP)
        mp.line([(x, 560 + i % 3 * 30), (x + 120, 556 + i % 3 * 30)], (240, 250, 255), 6)
    mp.save("glaucus")


# ---------- portraits --------------------------------------------------------------

def clip_patch(ic, poly, disc_mask, color):
    m = ImageChops.multiply(ic._mask(ic._wobble(poly, amp=2.5)), disc_mask)
    ic.img.paste(Image.new("RGBA", (512, 512), color + (255,)), (0, 0), m)
    ic.scribble_in(m, color, var=0.22, width=6, density=0.9, length=(30, 90), spread=14)
    layer = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
    w = ic._wobble(poly, amp=2.0)
    ImageDraw.Draw(layer).line(w + [w[0]], fill=tone(color, 0.4) + (255,), width=5, joint="curve")
    ic._comp(layer, disc_mask)


def planet(ic, base, patches=(), air=None, r=178, cx=256, cy=262, lines=()):
    disc = circle(cx, cy, r, n=64)
    if air:
        ic.glow((cx, cy, r + 18), air, radius=22, strength=0.7)
    disc = ic.flat(disc, base)
    a = 30
    ic.scribble(disc, base, var=0.26, angle=a, density=0.8, length=(40, 110), spread=14)
    dm = ic._mask(disc)
    for poly, col in patches:
        clip_patch(ic, poly, dm, col)
    for pts, col, w in lines:
        layer = Image.new("RGBA", (512, 512), (0, 0, 0, 0))
        ImageDraw.Draw(layer).line(ic._wobble(pts, closed=False, amp=2), fill=col + (255,), width=w, joint="curve")
        ic._comp(layer, dm)
    night = mix(darker(base, 0.55), (16, 18, 48), 0.35)
    ic.scribble_in(ic.crescent(disc, (1, 1), 0.46), night, var=0.12, angle=a, density=1.8, length=(36, 90), spread=14)
    ic.scribble_in(ic.crescent(disc, (1, 1), 0.24), mix(night, (8, 8, 26), 0.55), var=0.10, angle=a, density=2.0, length=(30, 80), spread=14)
    ic.scribble_in(ic.crescent(disc, (-1, -1), 0.16), lighter(base, 0.5), var=0.1, angle=a + 20, density=1.2, width=5, length=(30, 80), spread=14)
    ic.dabs(disc, lighter(base, 0.65), (-1, -1), n=4, size=13)
    if air:
        ic.rim(disc, lighter(air, 0.3), width=9, inset=8)
    ic.outline(disc, tone(base), 10)
    return disc


def P(fn):
    def run(name, seed):
        ic = Icon(seed)
        fn(ic)
        ic.save(os.path.join(OUT, "portrait_" + name + ".png"))
        print("portrait", name)
    return run


@P
def p_star(ic):
    ic.glow((256, 256, 200), (255, 200, 80), radius=34, strength=0.9)
    ic.drawn(star(256, 256, 236, 176, 14, jag=0.12, seed=4), (255, 150, 40), shade=0.1, highlight=0, edge=(200, 70, 20), edge_w=7)
    ic.drawn(circle(256, 256, 160), (255, 214, 70), shade=0.2, highlight=0.2, rim_color=(255, 246, 190), edge=(214, 120, 20))
    for x, y in ((210, 210), (300, 300), (296, 200)):
        ic.dot(x, y, 9, (255, 170, 60))


@P
def p_home(ic):
    planet(ic, (40, 108, 196), air=(120, 200, 255), patches=[
        (blob(200, 210, 90, 1, aspect=1.1), (88, 170, 66)), (blob(330, 340, 70, 2), (88, 170, 66)),
        (blob(190, 220, 34, 3, aspect=1.5), (214, 182, 98)), (blob(256, 96, 70, 4, aspect=2.2), (238, 246, 252)),
        (blob(330, 330, 30, 5), (40, 120, 58))],
        lines=[([(120, 300), (210, 290), (290, 312)], (246, 248, 252), 8), ([(260, 170), (360, 180)], (246, 248, 252), 7)])


@P
def p_verdance(ic):
    planet(ic, (28, 124, 132), air=(140, 230, 200), patches=[
        (blob(180, 200, 46, 1), (42, 152, 58)), (blob(300, 250, 60, 2, aspect=1.4), (42, 152, 58)), (blob(220, 340, 40, 3), (42, 152, 58)),
        (blob(340, 380, 30, 4), (42, 152, 58)), (blob(300, 250, 26, 5), (104, 122, 46)), (blob(180, 200, 20, 6), (24, 104, 44))],
        lines=[([(110, 260), (200, 250), (260, 280)], (240, 248, 248), 9), ([(240, 150), (330, 160)], (240, 248, 248), 7), ([(280, 330), (380, 320)], (240, 248, 248), 7)])


@P
def p_ochre(ic):
    planet(ic, (196, 106, 58), air=(255, 196, 150), patches=[
        (blob(190, 220, 70, 1, aspect=1.3), (154, 72, 44)), (blob(330, 330, 60, 2), (154, 72, 44)),
        (blob(270, 290, 26, 3, aspect=1.6), (238, 228, 204)), (blob(200, 350, 34, 4, aspect=1.5), (58, 112, 146)),
        (blob(256, 92, 90, 5, aspect=2.4), (240, 234, 226))],
        lines=[([(120, 260), (190, 250), (250, 280), (330, 270)], (98, 38, 28), 9), ([(300, 180), (360, 230), (400, 240)], (98, 38, 28), 8)])


@P
def p_lantern(ic):
    rnd = random.Random(3)
    patches = [(blob(200, 230, 70, 1), (100, 100, 116)), (blob(310, 330, 54, 2), (100, 100, 116))]
    for i in range(7):
        x, y, r = rnd.uniform(130, 380), rnd.uniform(140, 390), rnd.uniform(12, 26)
        patches.append((circle(x, y, r, n=20), (124, 124, 138)))
    planet(ic, (156, 156, 168), patches=patches)


@P
def p_rime(ic):
    rnd = random.Random(4)
    lines = []
    for i in range(6):
        x, y = rnd.uniform(100, 300), rnd.uniform(130, 380)
        lines.append((wiggle(x, y, 200, rnd.uniform(-0.5, 0.5), i, n=7, jit=12), (66, 132, 200), 6))
    planet(ic, (204, 232, 246), patches=[(blob(220, 250, 80, 1, aspect=1.4), (166, 208, 236))], lines=lines)
    ic.sparkle(96, 110, 30, (220, 248, 255), ink=(40, 110, 170))
    ic.sparkle(420, 400, 22, (220, 248, 255), ink=(40, 110, 170))
    ic.glow((262, 470, 34), (220, 245, 255), radius=14, strength=0.8)    # a geyser's spray off the south pole
    for w, L in ((-30, 46), (-10, 62), (10, 66), (30, 48)):
        ic.line([(256 + w * 0.15, 438), (256 + w * 0.5, 438 + L * 0.5), (256 + w, 438 + L)], (236, 248, 255), 5)


@P
def p_cinder(ic):
    rnd = random.Random(5)
    patches = [(blob(200, 220, 60, 1), (232, 140, 52)), (blob(320, 330, 54, 2, aspect=1.4), (46, 34, 34)), (blob(180, 340, 36, 3), (240, 232, 196))]
    for i in range(6):
        x, y = rnd.uniform(140, 370), rnd.uniform(150, 380)
        patches.append((circle(x, y, rnd.uniform(9, 16), n=18), (255, 92, 30)))
    planet(ic, (230, 196, 66), patches=patches)
    # a sulfur fountain off the upper-left limb: arcs that rise and fall back
    bx, by = 150, 140
    for k, (h, w) in enumerate(((110, -70), (130, -20), (100, 40), (80, -100))):
        pts = [(bx + w * t, by - 4 * h * t * (1 - t) * 1.0 - 10 * t) for t in [i / 10 for i in range(11)]]
        ic.line(pts, (250, 230, 130), 9, ink=(170, 110, 30), ink_w=6)
    for x, y in ((96, 70), (150, 40), (190, 60), (70, 120)):
        ic.dot(x, y, 7, (255, 110, 40), ink=(120, 40, 20))


@P
def p_murk(ic):
    patches = [(band(150 + 70 * k, 190 + 70 * k, 30 + k, amp=6), c) for k, c in enumerate(((186, 122, 52), (214, 158, 78), (176, 112, 48), (206, 148, 70)))]
    patches.append((blob(230, 120, 46, 9, aspect=1.6), (58, 46, 44)))
    # band() spans the map's width; on a 512 canvas it simply crosses the disc
    planet(ic, (200, 138, 58), air=(255, 170, 70), patches=patches)


@P
def p_hollow(ic):
    rnd = random.Random(6)
    patches = [(blob(220, 260, 90, 1), (136, 124, 146))]
    holes = []
    for i in range(6):
        x, y, r = rnd.uniform(140, 370), rnd.uniform(150, 380), rnd.uniform(14, 26)
        patches.append((blob(x, y, r, 10 + i, aspect=1.3), (34, 26, 44)))
        holes.append((x, y, r))
    planet(ic, (114, 102, 128), patches=patches)
    for x, y, r in holes[:4]:
        ic.dot(x + 3, y + 3, max(4, r * 0.3), (190, 130, 255))
    ic.sparkle(420, 120, 26, (210, 160, 255), ink=(70, 30, 120))


@P
def p_tidewater(ic):
    planet(ic, (24, 70, 156), air=(120, 200, 255), patches=[
        (blob(256, 262, 90, 1, aspect=1.2), (56, 164, 170)), (blob(256, 262, 60, 2), (32, 110, 170)),
        (circle(236, 250, 14), (96, 168, 82)), (circle(266, 270, 11), (96, 168, 82)), (circle(290, 246, 9), (96, 168, 82)),
        (blob(256, 90, 120, 3, aspect=2.4), (226, 240, 248))],
        lines=[([(120, 330), (200, 320), (260, 350)], (240, 248, 252), 8), ([(300, 160), (390, 170)], (240, 248, 252), 7)])


@P
def p_pebble(ic):
    rnd = random.Random(8)
    lump = blob(256, 262, 150, 77, rough=0.45, n=40)
    ic.drawn(lump, (124, 106, 94), shade=0.32, highlight=0.18)
    for i in range(9):
        x, y, r = rnd.uniform(160, 350), rnd.uniform(170, 360), rnd.uniform(10, 24)
        c = circle(x, y, r, n=18)
        ic.drawn(c, (100, 86, 78), shade=0, highlight=0, edge_w=4, hatch=0.5)
    for x, y in ((210, 220), (300, 300), (250, 330)):
        ic.sparkle(x, y, 14, (255, 210, 120), ink=(90, 60, 30))


@P
def p_amber(ic):
    ring_back = ellipse(256, 270, 240, 56, n=60, rot=-0.18)
    ic.outline(ring_back, (200, 170, 120), 20, passes=1)
    patches = []
    for k, c in enumerate(((206, 128, 66), (170, 92, 52), (214, 146, 80), (180, 104, 58), (206, 128, 66))):
        patches.append((band(110 + 64 * k, 140 + 64 * k, 60 + k, amp=5), c))
    patches.append((ellipse(300, 330, 34, 20, n=30), (204, 72, 50)))
    planet(ic, (232, 206, 156), patches=patches, r=170)
    # the near half of the ring, in front of the disc
    front = [p for p in ellipse(256, 270, 240, 56, n=60, rot=-0.18) if p[1] > 262 + (p[0] - 256) * -0.18]
    ic.line(front, (226, 196, 140), 18, ink=(120, 90, 50), ink_w=8)


@P
def p_glaucus(ic):
    planet(ic, (112, 192, 216), air=(170, 240, 255), patches=[
        (band(170, 210, 1, amp=4), (92, 168, 204)), (band(290, 330, 2, amp=4), (132, 208, 226)),
        (ellipse(210, 300, 40, 18), (36, 74, 150))],
        lines=[([(150, 240), (240, 236)], (240, 250, 255), 6), ([(280, 360), (360, 356)], (240, 250, 255), 6)])


# ---------- resource icons ---------------------------------------------------------

def R(fn):
    def run(name, seed):
        ic = Icon(seed)
        fn(ic)
        ic.save(os.path.join(OUT, "res_" + name + ".png"))
        print("res", name)
    return run


def hexagon(cx, cy, r, rot=0.0):
    return [(cx + r * math.cos(rot + k * math.pi / 3), cy + r * math.sin(rot + k * math.pi / 3)) for k in range(6)]


def prism(ic, cx, cy, w, h, color, edge=None):
    """A hexagonal prism standing up: a side face, a lit face, a top."""
    top = [(cx - w, cy - h), (cx - w * 0.5, cy - h - w * 0.45), (cx + w * 0.5, cy - h - w * 0.45), (cx + w, cy - h), (cx + w * 0.5, cy - h + w * 0.45), (cx - w * 0.5, cy - h + w * 0.45)]
    left = [(cx - w, cy - h), (cx - w * 0.5, cy - h + w * 0.45), (cx - w * 0.5, cy + w * 0.45), (cx - w, cy)]
    mid = [(cx - w * 0.5, cy - h + w * 0.45), (cx + w * 0.5, cy - h + w * 0.45), (cx + w * 0.5, cy + w * 0.45), (cx - w * 0.5, cy + w * 0.45)]
    right = [(cx + w * 0.5, cy - h + w * 0.45), (cx + w, cy - h), (cx + w, cy), (cx + w * 0.5, cy + w * 0.45)]
    e = edge or tone(color)
    ic.drawn(left, lighter(color, 0.25), edge=e, edge_w=7, shade=0.1)
    ic.drawn(mid, color, edge=e, edge_w=7)
    ic.drawn(right, darker(color, 0.3), edge=e, edge_w=7, highlight=0)
    ic.drawn(top, lighter(color, 0.5), edge=e, edge_w=7, shade=0)


@R
def r_ice(ic):
    prism(ic, 200, 400, 70, 190, (120, 200, 240))
    prism(ic, 330, 430, 56, 130, (150, 220, 250))
    ic.sparkle(120, 120, 34, (240, 252, 255), ink=(40, 110, 170))
    ic.sparkle(400, 210, 24, (240, 252, 255), ink=(40, 110, 170))


@R
def r_iron(ic):
    rock = blob(256, 290, 170, 3, rough=0.35, aspect=1.15)
    ic.drawn(rock, (110, 112, 126), shade=0.3, highlight=0.18)
    for x, y, r in ((200, 240, 30), (300, 320, 26), (330, 220, 20), (210, 350, 18)):
        ic.drawn(blob(x, y, r, int(x), rough=0.4), (196, 110, 60), shade=0.1, highlight=0.2, edge_w=5)
    ic.sparkle(400, 130, 26, (255, 230, 170), ink=(110, 60, 30))


@R
def r_sulfur(ic):
    for x, y, h, rot in ((180, 420, 260, -0.35), (290, 430, 320, 0.1), (370, 430, 210, 0.45)):
        crystal = [(x - 36, y), (x - 40, y - h * 0.75), (x, y - h), (x + 40, y - h * 0.75), (x + 36, y)]
        ic.drawn(rotate(crystal, rot, (x, y - 40)), (246, 214, 40), edge=(160, 110, 10), edge_w=8)
    ic.drawn(blob(266, 410, 70, 4, aspect=2.6, rough=0.2), (110, 90, 70), shade=0.25, highlight=0.1)


@R
def r_helium(ic):
    ic.glow((256, 270, 150), (120, 255, 220), radius=26, strength=0.6)
    body = [(170, 140), (342, 140), (342, 430), (170, 430)]
    ic.drawn(roundrect_pts(160, 130, 352, 440, 40), (200, 210, 224), shade=0.28, highlight=0.18, edge=(50, 60, 80))
    ic.drawn(rect(196, 190, 316, 380), (70, 230, 200), shade=0.15, highlight=0.2, edge=(20, 110, 100), edge_w=6, rim_color=(190, 255, 240))
    ic.drawn(rect(220, 86, 292, 134), (110, 120, 140), edge=(40, 46, 60))
    for y in (240, 300):
        ic.line([(214, y), (298, y)], (190, 255, 240), 6)
    ic.sparkle(410, 120, 24, (200, 255, 240), ink=(20, 110, 100))
    del body


def roundrect_pts(x0, y0, x1, y1, r):
    return crayon.roundrect(x0, y0, x1, y1, r)


@R
def r_methane(ic):
    drop = [(256, 70)] + [(256 + 150 * math.sin(t), 330 - 150 * math.cos(t)) for t in [math.pi * (0.25 + 1.5 * k / 30) for k in range(31)]]
    ic.drawn(drop, (232, 132, 44), rim_color=(255, 214, 140), edge=(130, 60, 20))
    flame = [(256, 210), (296, 300), (282, 380), (256, 400), (230, 380), (216, 300)]
    ic.drawn(flame, (70, 150, 255), shade=0.1, highlight=0.2, edge=(30, 60, 160), edge_w=6)


@R
def r_timber(ic):
    log = rotate(crayon.roundrect(90, 210, 400, 330, 50), -0.3, (256, 270))
    ic.drawn(log, (150, 96, 54), shade=0.3, highlight=0.15)
    for y in (250, 290):
        ic.line(rotate([(130, y), (360, y)], -0.3, (256, 270)), (110, 66, 36), 5)
    end = ellipse(392, 222, 48, 64, rot=-0.3)
    ic.drawn(end, (226, 186, 120), shade=0.1, highlight=0, edge=(120, 76, 40))
    ic.outline(ellipse(392, 222, 26, 36, rot=-0.3), (170, 120, 70), 5, passes=1)
    ic.drawn(blob(170, 150, 60, 3), (70, 170, 60), edge_w=6)


@R
def r_fish(ic):
    body = ellipse(236, 262, 150, 86, n=50)
    tail = [(370, 262), (450, 186), (440, 262), (450, 338)]
    ic.drawn(tail, (60, 170, 200), edge=(20, 80, 110))
    ic.drawn(body, (90, 196, 220), rim_color=(200, 250, 255), edge=(20, 80, 110))
    ic.drawn([(200, 190), (260, 140), (300, 196)], (60, 170, 200), edge=(20, 80, 110), edge_w=6)
    ic.dot(150, 240, 16, (250, 250, 250), ink=(20, 30, 40))
    ic.dot(154, 240, 8, (20, 30, 40))
    for y, L in ((230, 40), (262, 56), (294, 40)):
        ic.speed(470, y, L, 0, (220, 240, 255), width=6, ink=(20, 80, 110))


@R
def r_crystal(ic):
    ic.glow((256, 300, 160), (180, 110, 255), radius=30, strength=0.7)
    for x, y, h, rot in ((200, 390, 250, -0.3), (280, 400, 300, 0.05), (350, 390, 200, 0.4)):
        c = [(x - 32, y), (x - 36, y - h * 0.78), (x, y - h), (x + 36, y - h * 0.78), (x + 32, y)]
        ic.drawn(rotate(c, rot, (x, y)), (176, 110, 250), rim_color=(230, 210, 255), edge=(80, 30, 140), edge_w=7)
    ic.drawn(blob(270, 412, 70, 5, aspect=2.6, rough=0.2), (80, 70, 90), shade=0.25, highlight=0.1)
    ic.sparkle(110, 140, 28, (240, 220, 255), ink=(80, 30, 140))


@R
def r_obsidian(ic):
    shard = [(150, 420), (120, 250), (220, 90), (330, 120), (400, 260), (360, 420)]
    ic.drawn(shard, (50, 40, 62), shade=0.2, highlight=0.0, edge=(18, 12, 26))
    ic.drawn([(200, 140), (250, 110), (230, 300)], (130, 100, 190), shade=0, highlight=0.2, edge=None)
    ic.line([(280, 150), (350, 280)], (160, 130, 220), 7)
    ic.drawn(blob(256, 430, 40, 2, aspect=3), (255, 110, 40), shade=0, edge=(150, 40, 10), edge_w=5)


@R
def r_regolith(ic):
    pile = [(70, 420), (140, 300), (230, 220), (300, 240), (380, 320), (450, 420)]
    ic.drawn(pile, (176, 172, 180), shade=0.3, highlight=0.18)
    rnd = random.Random(7)
    for i in range(14):
        ic.dot(rnd.uniform(140, 380), rnd.uniform(290, 400), rnd.uniform(4, 9), rnd.choice(((120, 118, 130), (220, 220, 228))))
    ic.sparkle(330, 170, 26, (180, 255, 230), ink=(20, 110, 90))


@R
def r_water(ic):
    drop = [(256, 60)] + [(256 + 150 * math.sin(t), 330 - 150 * math.cos(t)) for t in [math.pi * (0.25 + 1.5 * k / 30) for k in range(31)]]
    ic.drawn(drop, (60, 150, 240), rim_color=(190, 230, 255), edge=(20, 60, 140))
    ic.dabs(drop, (230, 245, 255), (-1, -1), n=3, size=16)


MAPS = [map_home, map_verdance, map_ochre, map_lantern, map_rime, map_cinder, map_murk, map_hollow, map_tidewater, map_pebble, map_amber, map_glaucus]
PORTRAITS = [("star", p_star, 2), ("home", p_home, 3), ("verdance", p_verdance, 4), ("ochre", p_ochre, 5), ("lantern", p_lantern, 6),
             ("rime", p_rime, 7), ("cinder", p_cinder, 8), ("murk", p_murk, 9), ("hollow", p_hollow, 10), ("tidewater", p_tidewater, 11),
             ("pebble", p_pebble, 12), ("amber", p_amber, 13), ("glaucus", p_glaucus, 14)]
RESOURCES = [("ice", r_ice, 21), ("iron", r_iron, 22), ("sulfur", r_sulfur, 23), ("helium3", r_helium, 24), ("methane", r_methane, 25),
             ("timber", r_timber, 26), ("fish", r_fish, 27), ("glowcrystal", r_crystal, 28), ("obsidian", r_obsidian, 29),
             ("regolith", r_regolith, 30), ("water", r_water, 31)]


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    only = [a for a in sys.argv[1:] if not a.startswith("--")]
    if "--no-maps" not in sys.argv:
        for fn in MAPS:
            if not only or fn.__name__[4:] in only:
                fn()
    crayon.S = 512
    for name, fn, seed in PORTRAITS + RESOURCES:
        fn(name, seed)
    if "--sheet" in sys.argv:
        import subprocess
        sheet = os.path.join(KIT, "sheet.py")
        scratch = os.environ.get("ART_SHEET_DIR", OUT)
        pngs = [os.path.join(OUT, "portrait_" + n + ".png") for n, _, _ in PORTRAITS]
        subprocess.run([sys.executable, sheet, os.path.join(scratch, "sheet_portraits.png")] + pngs + ["--ref", "magnetic field", "--ref", "inferno"])
        pngs = [os.path.join(OUT, "res_" + n + ".png") for n, _, _ in RESOURCES]
        subprocess.run([sys.executable, sheet, os.path.join(scratch, "sheet_resources.png")] + pngs + ["--ref", "plating", "--ref", "vitality"])
        maps = sorted(f for f in os.listdir(OUT) if f.startswith("map_"))
        mont = Image.new("RGB", (1024, 256 * ((len(maps) + 1) // 2)), (20, 20, 26))
        for k, f in enumerate(maps):
            mont.paste(Image.open(os.path.join(OUT, f)).resize((512, 256)), ((k % 2) * 512, (k // 2) * 256))
        mont.save(os.path.join(scratch, "sheet_maps.png"))
        print("wrote maps montage")
