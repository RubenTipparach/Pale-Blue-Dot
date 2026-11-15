"""Draw the water life's atlas: the reef's tiles and M9's fish skins.

`ocean-environments` design B14. The owner, 2026-11-15: "reef needs some
textures", and of the fish, "use the fish species mockup to create proper
fish": M9, Fish Lab and Aquariums (docs/mockups/fish.html).

Writes, beside each other in assets/textures/water_life/:

- atlas.png, 512 x 512 sRGB, sampled nearest with no mipmaps;
- atlas.ron, the manifest the game reads (`fish::bodies`, `fish::places`):
  every rect, and every fish variant's genome and body plan;
- and a copy of both for the mockup, docs/mockups/fish/water_life_atlas.png
  and water_life_atlas.json, so M9 and the game wear the same pixels.

The fish skins are M9's own `paintSkin`, ported line for line from
fish.html (its `genome`, `palette`, `snap`, Bayer dither and patterns, and
the three.js r128 colour arithmetic they lean on: sRGB values, HSL offsets).
A species gets four variants, `genome(species, seed)` for seeds 1 to 4; a
creature wears variant `id % 4`. M9's one-in-64 morphs are not baked.

The reef's tiles are multiply masks, not palette art: neutral greys near
white (mean luminance about 0.87, the darkest marks about 0.6), because the
piece's vertex colour carries its kind's palette and the depth's dimming,
and the texture only lays its surface over that. No outlines (the world's
art rule, docs/mockups/foliage/world_art.py).

    python tools/water_life_atlas.py          # write
    python tools/water_life_atlas.py --check  # the committed files match
"""

import json
import math
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "textures" / "water_life"
MOCK = ROOT / "docs" / "mockups" / "fish"
SIZE = 512
VARIANTS = 4

# ---------------------------------------------------------------- M9's numbers
# fish.html PROFILES and SPECIES (the fields the body and the skin read).
PROFILES = {
    "fusiform": dict(D=0.12, W=0.075, uMax=0.33, ped=0.20, body=0.80),
    "compressed": dict(D=0.19, W=0.060, uMax=0.38, ped=0.18, body=0.82),
    "disc": dict(D=0.29, W=0.055, uMax=0.44, ped=0.14, body=0.82),
    "eel": dict(D=0.042, W=0.036, uMax=0.10, ped=0.55, body=1.00, eel=True),
    "flat": dict(D=0.05, W=0.44, uMax=0.40, ped=0.10, body=1.00, flat=True),
}

SPECIES = [
    dict(id="minnow", prof="fusiform", tail="forked", dorsal=[[0.38, 0.55, 0.10, "soft"]],
         anal=[[0.62, 0.74, 0.06]], pec=0.07, eye=0.03, pat="gradient", den=1, con=0.7,
         cols=dict(back="#4e7096", belly="#e2e8ec", acc="#344a68", fin="#96afc4"), hueVar=8),
    dict(id="silverfin", prof="fusiform", dMul=0.9, tail="deepfork", dorsal=[[0.36, 0.50, 0.12, "soft"]],
         anal=[[0.60, 0.72, 0.07]], pec=0.07, eye=0.032, pat="stripes", den=0.5, con=0.85,
         cols=dict(back="#407ea0", belly="#eef4f8", acc="#1e4262", fin="#c4dee9"), hueVar=6),
    dict(id="perch", prof="compressed", dMul=0.95, tail="truncate",
         dorsal=[[0.22, 0.48, 0.14, "spiny"], [0.50, 0.68, 0.11, "soft"]], anal=[[0.62, 0.74, 0.08]],
         pec=0.08, eye=0.034, pat="bars", bars=[5, 0.045, 0.55], den=1, con=0.9,
         cols=dict(back="#cea834", belly="#f4e29c", acc="#3a461e", fin="#e8622c"), hueVar=10),
    dict(id="ray", prof="flat", tail="whip", dorsal=[], anal=[], pec=0, eye=0.026, pat="spots", den=1,
         con=0.8, cols=dict(back="#8c6e46", belly="#f0ece2", acc="#4a3622", fin="#7a6040"), hueVar=8),
    dict(id="eel", prof="eel", tail="eel", dorsal=[], anal=[], pec=0.035, eye=0.022, pat="mottled", den=1,
         con=0.8, cols=dict(back="#6c7a3a", belly="#ccc278", acc="#282e12", fin="#4a5428"), hueVar=8),
    dict(id="reef", prof="disc", dMul=0.95, tail="lunate", dorsal=[[0.18, 0.66, 0.20, "tall"]],
         anal=[[0.50, 0.74, 0.16]], pec=0.08, eye=0.034, pat="bars", bars=[3, 0.06, 0.9], den=1, con=1,
         cols=dict(back="#24aaac", belly="#96e2d2", acc="#ffce2c", fin="#207896"), hueVar=14),
    dict(id="deepback", prof="fusiform", dMul=1.2, tail="lunate", dorsal=[[0.34, 0.48, 0.12, "soft"]],
         anal=[[0.62, 0.72, 0.07]], pec=0.07, eye=0.036, pat="gradient", den=1, con=0.9,
         cols=dict(back="#2c4062", belly="#b0c4ce", acc="#182238", fin="#466082"), hueVar=6),
    dict(id="serpent", prof="eel", dMul=1.5, tail="eel", crest=True, dorsal=[], anal=[], pec=0.04,
         eye=0.026, pat="bars", bars=[9, 0.025, 0.5], den=1, con=0.85,
         cols=dict(back="#28806e", belly="#c4deaa", acc="#12403a", fin="#d64034"), hueVar=10),
    # The reef's two ambient fish (not species, never caught), added to M9 in
    # the same form (design B14): the clownfish-like darter and the anthias.
    dict(id="darter", prof="compressed", dMul=1.1, tail="rounded", dorsal=[[0.22, 0.68, 0.12, "spiny"]],
         anal=[[0.58, 0.74, 0.09]], pec=0.08, eye=0.04, pat="bars", bars=[3, 0.07, 1.0], den=1, con=1,
         cols=dict(back="#c56d25", belly="#faaf5c", acc="#fffefe", fin="#fb902f"), hueVar=6),
    dict(id="anthias", prof="compressed", dMul=0.85, tail="lunate", dorsal=[[0.24, 0.66, 0.10, "soft"]],
         anal=[[0.58, 0.74, 0.08]], pec=0.07, eye=0.036, pat="stripes", den=0.5, con=0.9,
         cols=dict(back="#dd576d", belly="#faaf5c", acc="#fb902f", fin="#dd576d"), hueVar=10),
]

# ---------------------------------------------------------------- JS arithmetic
M32 = 0xFFFFFFFF


def imul(a, b):
    return ((a & M32) * (b & M32)) & M32


def to_i32(x):
    x &= M32
    return x - (1 << 32) if x & 0x80000000 else x


def mulberry32(a):
    state = [a & M32]

    def r():
        state[0] = (state[0] + 0x6D2B79F5) & M32
        a_ = state[0]
        t = imul(a_ ^ (a_ >> 15), 1 | a_)
        t = ((t + imul(t ^ (t >> 7), 61 | t)) & M32) ^ t
        return ((t ^ (t >> 14)) & M32) / 4294967296

    return r


def hash_str(s):
    h = 2166136261
    for ch in s:
        h ^= ord(ch)
        h = imul(h, 16777619)
    return h & M32


def jsround(x):
    return math.floor(x + 0.5)


class Col:
    """three.js r128's Color: sRGB values 0..1, no colour management."""

    def __init__(self, r, g, b):
        self.r, self.g, self.b = r, g, b

    @staticmethod
    def hex(h):
        v = int(h[1:], 16)
        return Col(((v >> 16) & 255) / 255, ((v >> 8) & 255) / 255, (v & 255) / 255)

    def clone(self):
        return Col(self.r, self.g, self.b)

    def get_hsl(self):
        r, g, b = self.r, self.g, self.b
        mx, mn = max(r, g, b), min(r, g, b)
        light = (mn + mx) / 2
        if mn == mx:
            return 0.0, 0.0, light
        d = mx - mn
        sat = d / (mx + mn) if light <= 0.5 else d / (2 - mx - mn)
        if mx == r:
            hue = (g - b) / d + (6 if g < b else 0)
        elif mx == g:
            hue = (b - r) / d + 2
        else:
            hue = (r - g) / d + 4
        return hue / 6, sat, light

    def set_hsl(self, h, s, l):
        h = ((h % 1) + 1) % 1
        s = min(max(s, 0), 1)
        l = min(max(l, 0), 1)
        if s == 0:
            self.r = self.g = self.b = l
            return self

        def hue2rgb(p, q, t):
            if t < 0:
                t += 1
            if t > 1:
                t -= 1
            if t < 1 / 6:
                return p + (q - p) * 6 * t
            if t < 1 / 2:
                return q
            if t < 2 / 3:
                return p + (q - p) * 6 * (2 / 3 - t)
            return p

        p = l * (1 + s) if l <= 0.5 else l + s - l * s
        q = 2 * l - p
        self.r = hue2rgb(q, p, h + 1 / 3)
        self.g = hue2rgb(q, p, h)
        self.b = hue2rgb(q, p, h - 1 / 3)
        return self

    def offset_hsl(self, h, s, l):
        hh, ss, ll = self.get_hsl()
        return self.set_hsl(hh + h, ss + s, ll + l)

    def lerp(self, o, k):
        self.r += (o.r - self.r) * k
        self.g += (o.g - self.g) * k
        self.b += (o.b - self.b) * k
        return self

    def mul(self, k):
        self.r *= k
        self.g *= k
        self.b *= k
        return self


SWEAFT_64 = ["#010e1a", "#011c34", "#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7", "#fffefe", "#a09f91",
             "#434137", "#2c2c25", "#161612", "#010000", "#1c020d", "#4a051e", "#841c1d", "#d4462b", "#fe6420",
             "#fb902f", "#faaf5c", "#003227", "#05582d", "#297537", "#559642", "#9bb675", "#0c2f44", "#204f63",
             "#53998b", "#84e3b5", "#c1f1d8", "#191954", "#15578a", "#1096c3", "#74effe", "#d2f8ff", "#230c44",
             "#4f1d83", "#7d2dbd", "#9e3ba2", "#c34988", "#dd576d", "#2b1f35", "#3e3551", "#69608b", "#170916",
             "#2f1525", "#522b2f", "#7d4636", "#a8633e", "#e69d69", "#eab798", "#41322c", "#725244", "#ab8465",
             "#cfaa81", "#561c27", "#733126", "#8d4526", "#c56d25", "#2e0d06", "#681712", "#a61e1f", "#d12d41",
             "#fa3a64"]
SWEAFT_RGB = [(int(h[1:3], 16), int(h[3:5], 16), int(h[5:7], 16)) for h in SWEAFT_64]


def snap(c):
    r, g, b = c.r * 255, c.g * 255, c.b * 255
    best, bd = 0, 1e9
    for i, p in enumerate(SWEAFT_RGB):
        d = (p[0] - r) ** 2 * 2 + (p[1] - g) ** 2 * 4 + (p[2] - b) ** 2 * 3
        if d < bd:
            bd, best = d, i
    return SWEAFT_RGB[best]


BAYER4 = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]


def bay(x, y):
    return (BAYER4[int(y) & 3][int(x) & 3] + 0.5) / 16


def smooth(a, b, x):
    t = min(max((x - a) / (b - a), 0), 1)
    return t * t * (3 - 2 * t)


# ---------------------------------------------------------------- M9's genome and skin
def genome(sp, seed):
    r = mulberry32(to_i32(seed ^ hash_str(sp["id"])))
    len_k = 0.85 + 0.40 * r()
    age = (len_k - 0.85) / 0.40
    g = dict(
        sp=sp, seed=seed, prof=sp["prof"], tail=sp["tail"], pat=sp["pat"], len_k=len_k,
        depth=sp.get("dMul", 1) * (1 + (r() - 0.5) * 0.14 + (age - 0.5) * 0.12),
        width=1 + (r() - 0.5) * 0.14 + (age - 0.5) * 0.08,
        fin=1 + (r() - 0.5) * 0.24,
        den=sp["den"] * (1 + (r() - 0.5) * 0.3),
        con=sp["con"],
        hue=(r() - 0.5) * 2 * sp["hueVar"],
        light=(r() - 0.5) * 0.08,
        patSeed=math.floor(r() * 1e9),
    )
    return g


def palette(g):
    c = {}
    for k in ("back", "belly", "acc", "fin"):
        col = Col.hex(g["sp"]["cols"][k])
        col.offset_hsl(g["hue"] / 360, 0, g["light"])
        c[k] = col
    return c


def paint_skin(g):
    """fish.html's paintSkin: the 64 x 32 skin and the 8 x 8 fin tile."""
    W, H = 64, 32
    sp = g["sp"]
    c = palette(g)
    r = mulberry32(g["patSeed"])
    skin = [None] * (W * H)

    def wrap_y(y):
        return ((jsround(y) % H) + H) % H

    def put(x, y, col):
        x = jsround(x)
        if x < 0 or x >= W:
            return
        skin[wrap_y(y) * W + x] = col

    def put_both(x, y, col):
        put(x, y, col)
        put(x, H - 1 - jsround(y), col)

    def keep(x, y):
        return bay(x, wrap_y(y)) < g["con"]

    tones = [snap(c["back"].clone().lerp(c["belly"], k)) for k in (0, 1 / 3, 2 / 3, 1)]
    for y in range(H):
        t = (1 - math.cos(2 * math.pi * (y + 0.5) / H)) / 2
        k = (smooth(0.46, 0.56, t) if g["prof"] == "flat" else smooth(0.35, 0.8, t)) * 3
        i = math.floor(k)
        f = k - i
        for x in range(W):
            skin[y * W + x] = tones[min(3, i + 1 if f > bay(x, y) else i)]

    def y_at(th):
        return th / (2 * math.pi) * H

    acc = snap(c["acc"])

    def n(k):
        return max(1, jsround(k * g["den"]))

    pat = g["pat"]
    if pat == "gradient":
        band = H * 0.22
        y = 0
        while y < band:
            for x in range(W):
                if bay(x, y) < (1 - y / band) * g["con"] * 1.2:
                    put_both(x, y, acc)
            y += 1
        x = jsround(W * 0.1)
        while x < W * 0.98:
            if g["con"] > 0.3:
                put_both(x, y_at(math.pi / 2), acc)
            x += 2
    elif pat == "stripes":
        wavy = sp.get("wavy", False)
        k = n(5 if wavy else 2)
        for i in range(k):
            th = 0.3 + i * (2.3 / k) if wavy else math.pi * (0.38 + i * 0.16)
            ph = r() * 6
            amp = 0.8 + r() * 0.8 if wavy else 0
            w = 1 if wavy else 1 + jsround(r())
            for x in range(1, W):
                yy = y_at(th) + amp * math.sin(x * 0.44 + ph)
                for d in range(w):
                    if keep(x, yy + d):
                        put_both(x, yy + d, acc)
    elif pat == "bars":
        nb, bw, low = sp["bars"]
        k = n(nb)
        ylow = y_at(math.acos(-low))
        for i in range(k):
            u = 0.2 + 0.72 * (i + 0.5) / k + (r() - 0.5) * 0.03
            w = max(2, jsround(bw * W * (0.8 + r() * 0.5)))
            lean = (r() - 0.5) * 2
            y = 0
            while y <= ylow:
                x0 = jsround(u * W - w / 2 + lean * y / max(1, ylow))
                for x in range(x0, x0 + w):
                    if keep(x, y):
                        put_both(x, y, acc)
                y += 1
    elif pat == "spots":
        k = n(60 if g["prof"] == "flat" else 40)
        bel = snap(c["belly"])
        for _ in range(k):
            px = math.floor(W * (0.08 + r() * 0.9))
            yy = math.floor(H * r())
            t = (1 - math.cos(2 * math.pi * yy / H)) / 2
            big = r() < 0.45
            col = acc if r() < 0.72 else bel
            if t > 0.72 or r() > g["con"] + 0.05:
                continue
            put(px, yy, col)
            if big:
                put(px + 1, yy, col)
                put(px, yy + 1, col)
                put(px + 1, yy + 1, col)
    elif pat == "mottled":
        k = n(26)
        for _ in range(k):
            px, yy = W * r(), H * r()
            rx, ry = 1 + r() * 2.2, 0.8 + r() * 1.4
            t = (1 - math.cos(2 * math.pi * yy / H)) / 2
            if t > 0.8:
                continue
            y = math.floor(yy - ry)
            while y <= yy + ry:
                x = math.floor(px - rx)
                while x <= px + rx:
                    if ((x + 0.5 - px) / rx) ** 2 + ((y + 0.5 - yy) / ry) ** 2 <= 1 and keep(x, y):
                        put(x, y, acc)
                    x += 1
                y += 1
    img = np.array(skin, dtype=np.uint8).reshape(H, W, 3)
    f_mid = snap(c["fin"])
    f_dark = snap(c["fin"].clone().mul(0.62))
    f_lit = snap(c["fin"].clone().lerp(Col(1, 1, 1), 0.3))
    fin = np.zeros((8, 8, 3), np.uint8)
    for y in range(8):
        for x in range(8):
            fin[y, x] = f_dark if x % 4 == 1 else (f_lit if (x % 4 == 2 and y % 2 == 0) else f_mid)
    return img, fin


# ---------------------------------------------------------------- the reef's tiles
GREYS = [255, 232, 206, 168]  # the four tones of a mask, white to its darkest mark


def quant(v):
    """0..1 (1 lightest) to the four greys, by thresholds."""
    idx = np.where(v > 0.72, 0, np.where(v > 0.45, 1, np.where(v > 0.2, 2, 3)))
    return np.array(GREYS, np.uint8)[idx]


def rng(seed):
    return np.random.default_rng(seed)


def tile_polyps():
    t = np.full((32, 32), 0.6)
    rnd = rng(1)
    for row in range(0, 32, 5):
        for col in range(0, 32, 5):
            cx = (col + (2 if (row // 5) % 2 else 0) + rnd.integers(-1, 2)) % 32
            cy = (row + rnd.integers(-1, 2)) % 32
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    t[(cy + dy) % 32, (cx + dx) % 32] = 0.9
            t[cy, cx] = 0.1
    return quant(t)


def tile_knobs():
    t = np.full((32, 32), 0.6)
    rnd = rng(2)
    for _ in range(26):
        cx, cy = rnd.integers(0, 32, 2)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                t[(cy + dy) % 32, (cx + dx) % 32] = 0.95
        t[(cy + 2) % 32, cx] = 0.3
        t[(cy + 2) % 32, (cx + 1) % 32] = 0.3
    return quant(t)


def tile_brain():
    y, x = np.mgrid[0:32, 0:32].astype(float)
    a = np.sin(x * 2 * math.pi / 8 + 1.6 * np.sin(y * 2 * math.pi / 16))
    b = np.sin(y * 2 * math.pi / 32 * 2 + 1.0 * np.sin(x * 2 * math.pi / 32 * 3))
    v = np.abs(a * 0.8 + b * 0.35)
    return quant(np.clip(v, 0, 1) * 0.95 + 0.0)


def tile_plate():
    y, x = np.mgrid[0:32, 0:32].astype(float)
    rr = np.hypot(x - 15.5, y - 15.5)
    ring = (np.floor(rr) % 4 == 0)
    ang = np.arctan2(y - 15.5, x - 15.5)
    spoke = (np.abs(np.sin(ang * 9)) < 0.12) & (rr > 3)
    v = np.where(ring, 0.15, np.where(spoke, 0.4, 0.65))
    v = np.where((np.floor(rr) % 4 == 1), 0.95, v)
    return quant(v)


def tile_pores():
    t = np.full((32, 32), 0.6)
    rnd = rng(4)
    for _ in range(30):
        cx, cy = rnd.integers(0, 32, 2)
        t[cy, cx] = 0.05
        t[cy, (cx + 1) % 32] = 0.05
        t[(cy + 1) % 32, cx] = 0.9
        t[(cy + 1) % 32, (cx + 1) % 32] = 0.9
    return quant(t)


def tile_lattice():
    y, x = np.mgrid[0:32, 0:32]
    net = ((x + y) % 6 == 0) | ((x - y) % 6 == 0)
    v = np.where(net, 0.95, 0.3)
    v = np.where(((x + y) % 6 == 3) & ((x - y) % 6 == 3), 0.1, v)
    return quant(v)


def tile_stripes():
    y, x = np.mgrid[0:32, 0:32]
    v = np.where(y % 4 == 0, 0.25, np.where(y % 4 == 2, 0.95, 0.6))
    v = np.where((y % 8 == 6) & (x % 3 == 0), 0.95, v)
    return quant(v)


def tile_veins():
    y, x = np.mgrid[0:32, 0:32]
    v = np.where(x % 8 == 3, 0.95, np.where(x % 4 == 1, 0.35, 0.6))
    v = np.where((x % 8 == 4) & (y % 5 == 0), 0.3, v)
    return quant(v)


def tile_grit():
    rnd = rng(9)
    v = rnd.choice([0.95, 0.6, 0.6, 0.35, 0.1], size=(32, 32), p=[0.25, 0.3, 0.2, 0.17, 0.08])
    return quant(v)


def tile_ridges():
    y, x = np.mgrid[0:32, 0:32]
    v = np.where(x % 5 == 0, 0.25, np.where(x % 5 == 2, 0.95, 0.6))
    v = np.where(y % 11 == 5, 0.35, v)
    return quant(v)


def tile_basalt():
    rnd = rng(11)
    v = np.full((32, 32), 0.6)
    # Prism faces cracked into blocks, with pale sulfide crust in specks.
    for yy in range(0, 32, 8):
        off = 4 if (yy // 8) % 2 else 0
        v[yy, :] = 0.1
        for xx in range(off, 32, 8):
            v[yy:yy + 8, xx] = 0.1
    specks = rnd.random((32, 32)) < 0.12
    v = np.where(specks & (v > 0.2), 0.95, v)
    rough = rnd.random((32, 32)) < 0.2
    v = np.where(rough & (v > 0.2) & (v < 0.9), 0.35, v)
    return quant(v)


def tile_ice():
    rnd = rng(12)
    y, x = np.mgrid[0:32, 0:32]
    v = np.where((x * 2 + y) % 13 == 0, 0.6, 0.95)
    for _ in range(3):
        cx, cy = rnd.integers(0, 32, 2)
        for s in range(14):
            v[cy % 32, cx % 32] = 0.35
            cx += 1
            cy += int(rnd.integers(-1, 2))
    return quant(v)


def tile_mat():
    rnd = rng(13)
    v = np.full((32, 32), 0.95)
    for _ in range(40):
        cx, cy = rnd.integers(0, 32, 2)
        rr = rnd.integers(1, 3)
        for dy in range(-rr, rr + 1):
            for dx in range(-rr, rr + 1):
                if dx * dx + dy * dy <= rr * rr:
                    v[(cy + dy) % 32, (cx + dx) % 32] = 0.6
    v = np.where(rnd.random((32, 32)) < 0.06, 0.35, v)
    return quant(v)


def tile_white():
    return np.full((32, 32), 255, np.uint8)


REEF_TILES = [
    ("white", tile_white), ("polyps", tile_polyps), ("knobs", tile_knobs), ("brain", tile_brain),
    ("plate", tile_plate), ("pores", tile_pores), ("lattice", tile_lattice), ("stripes", tile_stripes),
    ("veins", tile_veins), ("grit", tile_grit), ("ridges", tile_ridges), ("basalt", tile_basalt),
    ("ice", tile_ice), ("mat", tile_mat),
]


# ---------------------------------------------------------------- the atlas
def place(atlas, img, x, y, pad=2):
    """An image at (x, y), its edge pixels carried `pad` out so nearest
    sampling at a rect's edge never reads a neighbour."""
    h, w = img.shape[:2]
    padded = np.pad(img, ((pad, pad), (pad, pad), (0, 0)), mode="edge")
    atlas[y - pad:y + h + pad, x - pad:x + w + pad] = padded


def build():
    atlas = np.zeros((SIZE, SIZE, 3), np.uint8)
    manifest = {"size": [SIZE, SIZE], "tiles": {}, "fish": []}
    # Reef tiles: a row of 32 px cells on a 36 px stride from (2, 2).
    for i, (name, fn) in enumerate(REEF_TILES):
        x, y = 2 + (i % 14) * 36, 2 + (i // 14) * 36
        grey = fn()
        place(atlas, np.stack([grey] * 3, -1), x, y)
        manifest["tiles"][name] = [x, y, 32, 32]
    # Fish: skins 64 x 32 on a 68 x 36 stride from y 40, fins (the 8 x 8
    # tile four times each way) 32 x 32 on a 36 stride below them.
    skins_per_row = (SIZE - 4) // 68
    n = 0
    skin_y0 = 40
    rows = math.ceil(len(SPECIES) * VARIANTS / skins_per_row)
    fin_y0 = skin_y0 + rows * 36
    fins_per_row = (SIZE - 4) // 36
    for sp in SPECIES:
        p = PROFILES[sp["prof"]]
        for v in range(VARIANTS):
            g = genome(sp, v + 1)
            skin, fin = paint_skin(g)
            sx, sy = 2 + (n % skins_per_row) * 68, skin_y0 + (n // skins_per_row) * 36
            fx, fy = 2 + (n % fins_per_row) * 36, fin_y0 + (n // fins_per_row) * 36
            place(atlas, skin, sx, sy)
            place(atlas, np.tile(fin, (4, 4, 1)), fx, fy)
            n += 1
            manifest["fish"].append({
                "id": sp["id"], "variant": v, "seed": v + 1,
                "skin": [sx, sy, 64, 32], "fin_tile": [fx, fy, 32, 32],
                "profile": sp["prof"], "tail": sp["tail"],
                "d": p["D"], "w": p["W"], "u_max": p["uMax"], "ped": p["ped"], "body": p["body"],
                "eel": p.get("eel", False), "flat": p.get("flat", False),
                "depth": round(g["depth"], 5), "width": round(g["width"], 5), "fin": round(g["fin"], 5),
                "length_k": round(g["len_k"], 5),
                "dorsal": [[d[0], d[1], d[2], d[3]] for d in sp["dorsal"]],
                "anal": [[a[0], a[1], a[2]] for a in sp["anal"]],
                "pec": sp["pec"], "eye": sp["eye"], "crest": sp.get("crest", False),
            })
    assert fin_y0 + math.ceil(n / fins_per_row) * 36 <= SIZE
    return atlas, manifest


def ron(manifest):
    """The manifest as RON for the game (`fish::bodies::Atlas`)."""
    def f(v):
        return repr(float(v))

    def rect(r):
        return "(" + ", ".join(str(int(a)) for a in r) + ")"

    out = ["// Written by tools/water_life_atlas.py; do not edit (ocean-environments B14).", "("]
    out.append(f"    size: ({manifest['size'][0]}, {manifest['size'][1]}),")
    out.append("    tiles: {")
    for k, r in manifest["tiles"].items():
        out.append(f'        "{k}": {rect(r)},')
    out.append("    },")
    out.append("    fish: [")
    for e in manifest["fish"]:
        dorsal = ", ".join(f'({f(d[0])}, {f(d[1])}, {f(d[2])}, "{d[3]}")' for d in e["dorsal"])
        anal = ", ".join(f"({f(a[0])}, {f(a[1])}, {f(a[2])})" for a in e["anal"])
        out.append(
            f'        (id: "{e["id"]}", variant: {e["variant"]}, seed: {e["seed"]}, '
            f'skin: {rect(e["skin"])}, fin_tile: {rect(e["fin_tile"])}, '
            f'profile: "{e["profile"]}", tail: "{e["tail"]}", '
            f'd: {f(e["d"])}, w: {f(e["w"])}, u_max: {f(e["u_max"])}, ped: {f(e["ped"])}, body: {f(e["body"])}, '
            f'eel: {str(e["eel"]).lower()}, flat: {str(e["flat"]).lower()}, '
            f'depth: {f(e["depth"])}, width: {f(e["width"])}, fin: {f(e["fin"])}, length_k: {f(e["length_k"])}, '
            f'dorsal: [{dorsal}], anal: [{anal}], pec: {f(e["pec"])}, eye: {f(e["eye"])}, '
            f'crest: {str(e["crest"]).lower()}),'
        )
    out.append("    ],")
    out.append(")")
    return "\n".join(out) + "\n"


def main():
    atlas, manifest = build()
    targets = {
        OUT / "atlas.ron": ron(manifest).encode(),
        MOCK / "water_life_atlas.json": (json.dumps(manifest, indent=1) + "\n").encode(),
    }
    png = Image.fromarray(atlas)
    if "--check" in sys.argv:
        bad = []
        for path, data in targets.items():
            if not path.exists() or path.read_bytes() != data:
                bad.append(str(path))
        for path in (OUT / "atlas.png", MOCK / "water_life_atlas.png"):
            if not path.exists() or not np.array_equal(np.asarray(Image.open(path).convert("RGB")), atlas):
                bad.append(str(path))
        if bad:
            raise SystemExit("out of date: " + ", ".join(bad))
        print("water_life atlas: up to date")
        return
    OUT.mkdir(parents=True, exist_ok=True)
    for path, data in targets.items():
        path.write_bytes(data)
    png.save(OUT / "atlas.png", optimize=True)
    png.save(MOCK / "water_life_atlas.png", optimize=True)
    print(f"wrote {OUT / 'atlas.png'} ({len(manifest['fish'])} fish skins, {len(manifest['tiles'])} tiles)")


if __name__ == "__main__":
    main()
