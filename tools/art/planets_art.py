"""Draw the art for docs/mockups/planets.html as pixel art (balatro-pixel-art kit).

    python tools/art/planets_art.py            # every PNG into docs/mockups/planets/
    python tools/art/planets_art.py --sheet    # also contact sheets (beside six of the
                                               # solitaire collection's sprites) in
                                               # $ART_SHEET_DIR, default target/art-sheets

The style is the solitaire collection's (`.claude/skills/balatro-pixel-art`): a tiny
native canvas, short ramps lit from the top left, a 1 px outline, single-pixel glints,
no anti-aliasing, every colour from Sweaft-64 (the kit's save() refuses anything else).
Four kinds of picture come out:

- `map_<body>.png`, 128 x 64 equirectangular surface maps for the three.js globes,
  drawn per pixel from noise sampled on the sphere (no seam at longitude 0), soft
  boundaries as 4x4 Bayer dithers. The page samples them nearest, no mipmaps.
- `lights_<body>.png`, the same grid, near-black with the night lights (towns, lava,
  cave glow) as single palette pixels.
- `portrait_<body>.png`, 48 x 48: the body's own map wrapped on a lit disc (day side
  top left, a dithered terminator, the night lights on the dark side), outlined.
- `res_<resource>.png`, 32 x 32 item icons.
"""

import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts")
REFS = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "references")
sys.path.insert(0, KIT)

from pixelkit import BAYER4, INK, PALETTE, RAMPS, SWEAFT_64, Sprite, rgb, save, sheet  # noqa: E402,F401

SCRATCH = os.environ.get("ART_SHEET_DIR") or os.path.join(ROOT, "target", "art-sheets")


def hexc(h):
    return np.array(rgb(h), np.uint8)


# ---------- noise on the sphere ---------------------------------------------------------------

def _hash(ix, iy, iz, seed):
    s = np.sin(ix * 127.1 + iy * 311.7 + iz * 74.7 + seed * 13.37) * 43758.5453
    return s - np.floor(s)


def vnoise3(p, seed):
    """Value noise in 3D, p shape (..., 3), smooth interpolation, 0..1."""
    i = np.floor(p)
    f = p - i
    u = f * f * (3 - 2 * f)
    ix, iy, iz = i[..., 0], i[..., 1], i[..., 2]
    out = 0
    for dx in (0, 1):
        wx = u[..., 0] if dx else 1 - u[..., 0]
        for dy in (0, 1):
            wy = u[..., 1] if dy else 1 - u[..., 1]
            for dz in (0, 1):
                wz = u[..., 2] if dz else 1 - u[..., 2]
                out = out + wx * wy * wz * _hash(ix + dx, iy + dy, iz + dz, seed)
    return out


def fbm(p, seed, freq=2.0, octaves=4):
    tot, amp, norm = 0.0, 1.0, 0.0
    for o in range(octaves):
        tot = tot + amp * vnoise3(p * freq * (2.03 ** o), seed + o * 17)
        norm += amp
        amp *= 0.5
    return tot / norm


class Grid:
    """An equirectangular map grid: pixel centres as longitude, latitude and sphere points."""

    def __init__(self, w=128, h=64):
        self.w, self.h = w, h
        ys, xs = np.mgrid[0:h, 0:w]
        self.x, self.y = xs, ys
        self.lon = 2 * math.pi * (xs + 0.5) / w
        self.lat = math.pi / 2 - math.pi * (ys + 0.5) / h
        cl = np.cos(self.lat)
        self.p = np.stack([cl * np.cos(self.lon), cl * np.sin(self.lon), np.sin(self.lat)], -1)
        self.bayer = (BAYER4[ys % 4, xs % 4] + 0.5) / 16.0
        self.img = np.zeros((h, w, 3), np.uint8)

    def noise(self, seed, freq=2.0, octaves=4, stretch=(1, 1, 1)):
        return fbm(self.p * np.array(stretch, float), seed, freq, octaves)

    def put(self, mask, h):
        self.img[mask] = hexc(h)

    def ramp(self, v, cols, mask=None, spread=1.0):
        """Fill with a dithered ramp: v 0..1 picks a colour of cols (dark to light)."""
        n = len(cols)
        idx = np.clip(np.floor(v * n + (self.bayer - 0.5) * spread), 0, n - 1).astype(int)
        m = np.ones(v.shape, bool) if mask is None else mask
        for i, c in enumerate(cols):
            self.put(m & (idx == i), c)

    def edge(self, v, t, width=0.03):
        """Pixels of a field near the threshold t, dithered over `width`: a soft boundary."""
        return (v > t - width) & (v < t + width) & (self.bayer < (v - (t - width)) / (2 * width))

    def wrapdx(self, cx):
        return ((self.x - cx + self.w / 2) % self.w) - self.w / 2

    def disc(self, cx, cy, r, aspect=1.0):
        dx = self.wrapdx(cx) / aspect
        dy = self.y - cy
        return dx, dy, (dx * dx + dy * dy) <= r * r

    def crater(self, cx, cy, r, floor, shadow, lit):
        """A pit: its floor, the shadowed wall on the upper left, the lit wall lower right."""
        dx, dy, m = self.disc(cx, cy, r + 0.3)
        self.put(m, floor)
        inner = (dx * dx + dy * dy) <= (r - 1.1) ** 2
        rim = m & ~inner
        self.put(rim & (dx + dy < 0), shadow)
        self.put(rim & (dx + dy > 0.8), lit)

    def save(self, path):
        from PIL import Image
        save(Image.fromarray(self.img, "RGB").convert("RGBA"), path, source="pixelkit (tools/art/planets_art.py)")


# ---------- the palette, darker and lighter -----------------------------------------------------

_PAL = np.array(PALETTE, float)


def _lum(c):
    c = np.asarray(c, float)
    return 0.3 * c[..., 0] + 0.59 * c[..., 1] + 0.11 * c[..., 2]


def nearest(c, darker_than=None, lighter_than=None):
    c = np.asarray(c, float)
    d = ((_PAL - c) ** 2).sum(1)
    if darker_than is not None:
        d = np.where(_lum(_PAL) < _lum(darker_than) - 6, d, 1e12)
    if lighter_than is not None:
        d = np.where(_lum(_PAL) > _lum(lighter_than) + 6, d, 1e12)
    return tuple(int(v) for v in _PAL[int(np.argmin(d))])


_SHADE = {}


def shade(c, k):
    """The palette colour for colour c (an rgb tuple) at brightness k (<1 darker, >1 lighter)."""
    key = (tuple(c), k)
    if key not in _SHADE:
        c = np.asarray(c, float)
        if k < 1:
            # towards a night blue, so the dark side reads as night, not as mud
            t = c * k + np.array([2, 8, 26]) * (1 - k)
            _SHADE[key] = nearest(t, darker_than=c)
        else:
            t = np.minimum(c * k + 18, 255)
            out = nearest(t, lighter_than=c)
            o = np.array(out, float)
            same_hue = (int(np.argmax(o)) == int(np.argmax(c)) and np.abs(o - c).sum() < 150
                        and abs((o[0] - o[2]) - (c[0] - c[2])) < 30 and abs((o[1] - o[2]) - (c[1] - c[2])) < 30)
            _SHADE[key] = out if same_hue else tuple(int(v) for v in c)
    return _SHADE[key]


# ---------- a globe sprite from a map -----------------------------------------------------------

LIGHT = np.array([-0.55, -0.62, 0.56])
LIGHT = LIGHT / np.linalg.norm(LIGHT)


def globe(sp, mapimg, cx, cy, r, lon0=0.0, tilt=0.35, lights=None, lumpy=None, air=None):
    """Paint a lit globe onto sprite sp from a map (an HxWx3 array): day side top left,
    a dithered terminator, night side bottom right with the map's night lights on it."""
    h, w = sp.h, sp.w
    ys, xs = np.mgrid[0:h, 0:w]
    nx = (xs + 0.5 - cx) / r
    ny = (ys + 0.5 - cy) / r
    rr = nx * nx + ny * ny
    if lumpy is not None:
        ang = np.arctan2(ny, nx)
        edge = lumpy(ang)
        inside = rr <= edge * edge
        rr = np.minimum(rr / (edge * edge), 0.999)
        nx, ny = nx / edge, ny / edge
    else:
        inside = rr <= 1.0
    nz = np.sqrt(np.clip(1 - nx * nx - ny * ny, 0, 1))
    # tip the north pole towards the viewer
    wy = -ny * math.cos(tilt) + nz * math.sin(tilt)
    wz = ny * math.sin(tilt) + nz * math.cos(tilt)
    lat = np.arcsin(np.clip(wy, -1, 1))
    lon = (np.arctan2(nx, wz) + lon0) % (2 * math.pi)
    mh, mw = mapimg.shape[:2]
    mx = np.clip((lon / (2 * math.pi) * mw).astype(int), 0, mw - 1)
    my = np.clip(((0.5 - lat / math.pi) * mh).astype(int), 0, mh - 1)
    ndl = nx * LIGHT[0] + ny * LIGHT[1] + nz * LIGHT[2]
    bay = (BAYER4[ys % 4, xs % 4] + 0.5) / 16.0
    # bands: lit rim, day, dusk (dithered), night, deep night
    level = np.where(ndl > 0.80, 1.25, 1.0)
    dusk = (ndl < 0.12) & (ndl > -0.12) & (bay < (0.12 - ndl) / 0.24)
    level = np.where(dusk | (ndl <= -0.12), 0.55, level)
    deep = (ndl < -0.45) | ((ndl < -0.30) & (bay < (-0.30 - ndl) / 0.15))
    level = np.where(deep, 0.28, level)
    for y, x in zip(*np.nonzero(inside)):
        c = tuple(int(v) for v in mapimg[my[y, x], mx[y, x]])
        k = float(level[y, x])
        col = c if k == 1.0 else shade(c, k)
        if lights is not None and k < 1.0:
            lc = tuple(int(v) for v in lights[my[y, x], mx[y, x]])
            if sum(lc) > 60:
                col = lc
        sp.rgb[y, x] = col
        sp.alpha[y, x] = True
    return inside


def air_rim(sp, inside, colour):
    """After the outline: a 1 px band of air outside it on the lit side, drawn as light."""
    from pixelkit import dilate
    ring2 = dilate(dilate(inside)) & ~dilate(inside)
    ys, xs = np.nonzero(inside)
    cy, cx = ys.mean(), xs.mean()
    yy, xx = np.mgrid[0:sp.h, 0:sp.w]
    lit = (xx - cx) * LIGHT[0] + (yy - cy) * LIGHT[1] > -2
    m = ring2 & lit & ~sp.alpha
    sp.fill(m, colour, glow=True)


# ---------- maps --------------------------------------------------------------------------------

OUT = os.path.join(ROOT, "docs", "mockups", "planets")
MAPS_DONE = {}
LIGHTS_DONE = {}


def towns(g, land, n, seed, warm="#faaf5c", core="#fffefe", big=0.2):
    """Night lights on land: single warm pixels, a few big towns with a white core and a neighbour."""
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    rnd = np.random.default_rng(seed)
    ys, xs = np.nonzero(land & (np.abs(g.lat) < 1.2))
    if not len(ys):
        return lights
    pick = rnd.choice(len(ys), size=min(n, len(ys)), replace=False)
    for k in pick:
        y, x = ys[k], xs[k]
        if rnd.random() < big:
            lights[y, x] = hexc(core)
            for dx, dy in ((1, 0), (0, 1), (-1, 0), (0, -1)):
                if rnd.random() < 0.6 and 0 <= y + dy < g.h:
                    lights[y + dy, (x + dx) % g.w] = hexc(warm)
        else:
            lights[y, x] = hexc(warm)
    return lights


def caps(g, top, bot, seed, cols=("#bfcad7", "#fffefe")):
    n = g.noise(seed, 6, 2)
    north = g.lat > math.radians(90 - top) - n * 0.25
    south = g.lat < -math.radians(90 - bot) + n * 0.25
    m = north | south
    g.put(m, cols[1])
    g.put(m & g.edge(np.abs(g.lat) - (math.radians(90 - top) - n * 0.25), 0.05, 0.05), cols[0])
    return m


def clouds(g, seed, amount=0.62, freq=3.0, col="#fffefe", soft="#bfcad7"):
    c = g.noise(seed, freq, 4, stretch=(1, 1, 3.2))
    g.put((c > amount - 0.04) & (g.bayer < 0.5), soft)
    g.put(c > amount, col)


def map_home():
    g = Grid()
    e = g.noise(101, 1.6, 5)
    m = g.noise(102, 2.4, 3)
    sea = e < 0.52
    g.ramp(np.clip((e - 0.30) / 0.24, 0, 1), ["#15578a", "#15578a", "#1096c3"], sea, spread=1.2)
    land = ~sea
    g.put(land, "#559642")
    g.put(land & (m > 0.55), "#297537")
    g.put(land & (m > 0.66), "#05582d")
    g.put(land & (m < 0.38) & (np.abs(g.lat) < 0.7), "#cfaa81")
    g.put(land & (m < 0.38) & (m > 0.35) & (g.bayer < 0.5) & (np.abs(g.lat) < 0.7), "#9bb675")
    g.put(land & (e > 0.66), "#725244")
    g.put(land & (e > 0.71), "#a09f91")
    g.put(land & (e > 0.75), "#fffefe")
    g.put(land & ~(np.roll(land, 1, 0) & np.roll(land, -1, 0) & np.roll(land, 1, 1) & np.roll(land, -1, 1)) & (e < 0.535), "#cfaa81")
    caps(g, 20, 24, 103)
    lights = towns(g, land & ~(np.abs(g.lat) > 1.1), 120, 104)
    clouds(g, 105, 0.67, freq=2.4)
    return g, lights


def map_verdance():
    g = Grid()
    e = g.noise(201, 3.2, 5)
    m = g.noise(202, 4.0, 3)
    sea = e < 0.56
    g.ramp(np.clip((e - 0.36) / 0.22, 0, 1), ["#204f63", "#204f63", "#53998b", "#53998b"], sea, spread=1.2)
    land = ~sea
    g.put(land, "#297537")
    g.put(land & (m > 0.52), "#559642")
    g.put(land & (m < 0.40), "#05582d")
    g.put(land & (m > 0.62), "#434137")
    g.put(land & (m > 0.62) & (g.bayer < 0.35), "#9bb675")
    caps(g, 14, 16, 203, cols=("#84e3b5", "#c1f1d8"))
    lights = towns(g, land, 50, 204, warm="#faaf5c")
    clouds(g, 205, 0.64, freq=2.8)
    return g, lights


def map_ochre():
    g = Grid()
    e = g.noise(301, 2.0, 5)
    g.ramp(np.clip((e - 0.25) / 0.5, 0, 1), ["#7d4636", "#a8633e", "#a8633e", "#c56d25"], spread=1.0)
    hi = g.noise(302, 2.4, 3)
    g.put(hi > 0.60, "#7d4636")
    g.put((hi > 0.57) & (hi <= 0.60) & (g.bayer < 0.5), "#7d4636")
    st = g.noise(303, 3.0, 3)
    g.put((st < 0.36) & (hi < 0.55), "#cfaa81")
    _, _, salt = g.disc(70, 34, 4, aspect=1.8)
    g.put(salt, "#eab798")
    _, _, sea = g.disc(54, 46, 5, aspect=1.6)
    g.put(sea, "#15578a")
    can = np.abs(g.noise(304, 2.2, 4) - 0.5) < 0.012
    g.put(can & (np.abs(g.lat) < 1.1), "#522b2f")
    oases = [(38, 30), (80, 26), (70, 42), (22, 36), (103, 40)]
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    for x, y in oases:
        g.put(g.disc(x, y, 1.2)[2], "#297537")
        g.img[y, x] = hexc("#1096c3")
        lights[y, x] = hexc("#fffefe")
        lights[y, (x + 1) % g.w] = hexc("#faaf5c")
        lights[y + 1, x] = hexc("#faaf5c")
    caps(g, 8, 10, 305, cols=("#eab798", "#fffefe"))
    lights = np.maximum(lights, towns(g, sea | g.disc(54, 46, 9, aspect=1.6)[2], 8, 306))
    return g, lights


def map_lantern():
    g = Grid()
    n = g.noise(401, 3.0, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ["#5d778d", "#92a3b2", "#92a3b2", "#bfcad7"], spread=1.0)
    maria = g.noise(402, 1.6, 3)
    g.put(maria < 0.42, "#3e586e")
    g.put(g.edge(maria, 0.42, 0.03), "#3e586e")
    rnd = np.random.default_rng(403)
    for _ in range(26):
        r = float(rnd.choice([1.5, 2, 2.5, 3, 4]))
        g.crater(rnd.uniform(0, 128), rnd.uniform(6, 58), r, "#5d778d", "#1f3b52", "#bfcad7")
    pole = np.abs(g.lat) + g.noise(404, 6, 2) * 0.2
    g.put(pole > 1.42, "#d2f8ff")
    g.put(g.edge(pole, 1.42, 0.04), "#bfcad7")
    return g, None


def map_rime():
    g = Grid()
    n = g.noise(501, 2.4, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ["#92a3b2", "#bfcad7", "#d2f8ff", "#d2f8ff", "#fffefe"], spread=1.2)
    for s in (502,):
        crack = np.abs(g.noise(s, 1.6, 3) - 0.5) < 0.010
        g.put(crack, "#15578a")
        g.put(np.roll(crack, -1, 0) & ~crack & (g.bayer < 0.5), "#74effe")
    rnd = np.random.default_rng(504)
    for _ in range(7):
        g.crater(rnd.uniform(0, 128), rnd.uniform(10, 54), float(rnd.choice([1.5, 2, 3])), "#bfcad7", "#5d778d", "#fffefe")
    for x in (40, 52, 60):                       # geysers at the south pole
        g.img[60, x] = hexc("#fffefe")
        g.img[59, x] = hexc("#74effe")
    return g, None


def map_cinder():
    g = Grid()
    n = g.noise(601, 2.6, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ["#c56d25", "#fb902f", "#faaf5c", "#faaf5c", "#eab798"], spread=1.3)
    p = g.noise(602, 2.0, 3)
    g.put(p > 0.62, "#fe6420")
    g.put(p < 0.34, "#eab798")
    flow = g.noise(603, 2.8, 4)
    g.put(flow > 0.64, "#2c2c25")
    g.put(g.edge(flow, 0.64, 0.025), "#161612")
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    rnd = np.random.default_rng(604)
    for _ in range(18):
        x, y, r = rnd.uniform(0, 128), rnd.uniform(8, 56), float(rnd.choice([1.0, 1.5, 2.0]))
        _, _, outer = g.disc(x, y, r + 1)
        _, _, core = g.disc(x, y, r)
        g.put(outer, "#a61e1f")
        g.put(core, "#fe6420")
        g.img[int(y), int(x) % 128] = hexc("#faaf5c")
        lights[outer] = hexc("#a61e1f")
        lights[core] = hexc("#fe6420")
        lights[int(y), int(x) % 128] = hexc("#faaf5c")
    return g, lights


def map_murk():
    g = Grid()
    warp = g.noise(701, 3.0, 3)
    b = np.sin(g.lat * 7 + warp * 3.0) * 0.5 + 0.5
    g.ramp(b, ["#8d4526", "#a8633e", "#c56d25", "#c56d25", "#e69d69"], spread=1.4)
    sea = g.noise(702, 3.0, 3)
    polar = np.abs(g.lat) > 1.0
    g.put(polar & (sea > 0.5), "#522b2f")
    g.put(polar & (sea > 0.58), "#2f1525")
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    lights[22, 60] = hexc("#fffefe")
    lights[22, 61] = hexc("#faaf5c")
    return g, lights


def map_hollow():
    g = Grid()
    n = g.noise(801, 2.6, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ["#3e3551", "#69608b", "#69608b", "#92a3b2"], spread=1.2)
    rnd = np.random.default_rng(802)
    for _ in range(9):
        g.crater(rnd.uniform(0, 128), rnd.uniform(8, 56), float(rnd.choice([1.5, 2, 3])), "#3e3551", "#2b1f35", "#92a3b2")
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    for _ in range(14):
        x, y, r = rnd.uniform(0, 128), rnd.uniform(10, 54), float(rnd.choice([1.0, 1.5, 2.0]))
        _, _, hole = g.disc(x, y, r, aspect=1.3)
        g.put(hole, "#170916")
        cx, cy = int(x) % 128, int(y)
        g.img[cy, cx] = hexc("#7d2dbd")
        lights[hole] = hexc("#4f1d83")
        lights[cy, cx] = hexc("#c34988")
    return g, lights


def map_tidewater():
    g = Grid()
    n = g.noise(901, 2.2, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ["#191954", "#15578a", "#15578a", "#1096c3"], spread=1.2)
    dx, dy, reef = g.disc(64, 32, 13, aspect=1.2)
    d = np.sqrt(dx * dx + dy * dy) / 13
    g.ramp(np.clip(1 - d, 0, 1) * 0.99, ["#1096c3", "#53998b", "#84e3b5"], reef, spread=1.6)
    isl = np.zeros_like(reef)
    for k in range(5):
        isl |= g.disc(58 + 3 * k, 30 + 2 * math.sin(k), 1.2 + 0.5 * (k % 2))[2]
    rnd = np.random.default_rng(902)
    for _ in range(6):
        isl |= g.disc(rnd.uniform(0, 128), rnd.uniform(18, 46), 1.4)[2]
    g.put(isl, "#559642")
    g.put(isl & ~(np.roll(isl, 1, 0) & np.roll(isl, -1, 0) & np.roll(isl, 1, 1) & np.roll(isl, -1, 1)), "#cfaa81")
    caps(g, 14, 16, 903)
    lights = towns(g, isl, 14, 904)
    clouds(g, 905, 0.68, freq=2.4)
    return g, lights


def map_pebble():
    g = Grid()
    n = g.noise(1001, 2.6, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ["#41322c", "#725244", "#725244", "#ab8465"], spread=1.2)
    rnd = np.random.default_rng(1002)
    for _ in range(34):
        g.crater(rnd.uniform(0, 128), rnd.uniform(6, 58), float(rnd.choice([1.5, 2, 2.5, 3.5])), "#725244", "#41322c", "#ab8465")
    for _ in range(12):
        g.img[int(rnd.uniform(6, 58)), int(rnd.uniform(0, 128))] = hexc("#faaf5c")
    return g, None


def map_amber():
    g = Grid()
    warp = g.noise(1101, 3.0, 4)
    b = np.sin(g.lat * 11 + warp * 2.4) * 0.5 + 0.5
    g.ramp(b, ["#8d4526", "#a8633e", "#e69d69", "#cfaa81", "#eab798"], spread=1.5)
    dx, dy, storm = g.disc(80, 42, 5, aspect=1.9)
    g.put(g.disc(80, 42, 6.5, aspect=1.9)[2], "#eab798")
    g.put(storm, "#d4462b")
    g.put(g.disc(80, 42, 2.5, aspect=1.9)[2], "#fe6420")
    g.put(g.disc(78, 41, 1.0, aspect=1.9)[2], "#faaf5c")
    return g, None


def map_glaucus():
    g = Grid()
    warp = g.noise(1201, 2.4, 3)
    b = np.sin(g.lat * 6 + warp * 1.4) * 0.5 + 0.5
    g.ramp(b, ["#1096c3", "#74effe", "#74effe", "#d2f8ff"], spread=1.6)
    g.put(g.disc(36, 38, 4, aspect=2.0)[2], "#15578a")
    g.put(g.disc(36, 38, 2, aspect=2.0)[2], "#191954")
    rnd = np.random.default_rng(1202)
    for _ in range(9):
        x, y = int(rnd.uniform(0, 128)), int(rnd.choice([18, 26, 40, 46]))
        for k in range(int(rnd.uniform(5, 12))):
            g.img[y, (x + k) % 128] = hexc("#fffefe")
    return g, None


MAPS = {"home": map_home, "verdance": map_verdance, "ochre": map_ochre, "lantern": map_lantern, "rime": map_rime,
        "cinder": map_cinder, "murk": map_murk, "hollow": map_hollow, "tidewater": map_tidewater, "pebble": map_pebble,
        "amber": map_amber, "glaucus": map_glaucus}


def write_maps():
    from PIL import Image
    for name, fn in MAPS.items():
        g, lights = fn()
        g.save(os.path.join(OUT, "map_" + name + ".png"))
        MAPS_DONE[name] = g.img.copy()
        if lights is not None:
            save(Image.fromarray(lights, "RGB").convert("RGBA"), os.path.join(OUT, "lights_" + name + ".png"),
                 source="pixelkit (tools/art/planets_art.py)")
            LIGHTS_DONE[name] = lights
        print("map", name)


# ---------- portraits (48 x 48) -----------------------------------------------------------------

def portrait(name, r=20, lon0=0.0, tilt=0.35, air=None, lumpy=None, extra=None, behind=None):
    sp = Sprite(48, 48)
    if behind:
        behind(sp)
    inside = globe(sp, MAPS_DONE[name], 24, 24, r, lon0=lon0, tilt=tilt, lights=LIGHTS_DONE.get(name), lumpy=lumpy)
    ys, xs = np.nonzero(inside)
    # one glint on the lit shoulder
    k = np.argmin((xs - 24) * 0.7 + (ys - 24))
    sp.plot(int(xs[k]) + 4, int(ys[k]) + 3, "#fffefe")
    if extra:
        extra(sp, "solid")
    sp.outline()
    if air:
        air_rim(sp, inside, air)
    if extra:
        extra(sp, "glow")
    return sp


def p_star():
    sp = Sprite(48, 48)
    sp.part(sp.star(24, 24, 23, 15, 12), RAMPS["flame"], "inset", glow=True)
    sp.part(sp.circle(24, 24, 15), ["#8d4526", "#c56d25", "#fb902f", "#faaf5c", "#fffefe"], "sphere", base=3)
    for x, y in ((18, 26), (28, 31), (29, 20)):
        sp.fill(sp.rect(x, y, x + 1, y), "#fb902f")
    sp.plots([(17, 16), (18, 15)], "#fffefe")
    return sp


def ring_parts(sp, front):
    outer = sp.ellipse(1, 18, 46, 31)
    inner = sp.ellipse(7, 21, 40, 28)
    ring = outer & ~inner
    yy = np.mgrid[0:48, 0:48][0]
    half = ring & ((yy >= 25) if front else (yy < 25))
    sp.part(half, ["#8d4526", "#a8633e", "#cfaa81", "#eab798"], "rim")


def p_amber():
    sp = Sprite(48, 48)
    ring_parts(sp, False)
    inside = globe(sp, MAPS_DONE["amber"], 24, 24, 15, tilt=0.25)
    sp.plot(18, 14, "#fffefe")
    ring_parts(sp, True)
    sp.outline()
    del inside
    return sp


def p_cinder_extra(sp, phase):
    if phase != "glow":
        return
    # a sulfur fountain off the upper-left limb
    for pts in (((10, 12), (6, 5), (3, 9)), ((11, 11), (10, 3), (14, 2))):
        m = sp.line(list(pts), 1)
        sp.fill(m & ~sp.alpha, "#faaf5c", glow=True)
    sp.plots([(3, 10), (14, 3), (6, 4)], "#fffefe", glow=True)


def p_rime_extra(sp, phase):
    if phase != "glow":
        return
    m = sp.taper(24, 44, 24, 47, 2, 5)
    sp.fill(m & ~sp.alpha, "#d2f8ff", glow=True)
    sp.plots([(22, 46), (26, 47)], "#fffefe", glow=True)


def p_hollow_extra(sp, phase):
    if phase == "glow":
        for x, y in ((40, 7),):
            sp.plots([(x, y), (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], "#c34988", glow=True)
            sp.plot(x, y, "#fffefe", glow=True)


def pebble_edge(a):
    return 0.86 + 0.10 * np.sin(3 * a + 1) + 0.06 * np.sin(5 * a + 2) + 0.04 * np.cos(7 * a)


PORTRAITS = {
    "star": p_star,
    "home": lambda: portrait("home", air="#74effe", lon0=0.4),
    "verdance": lambda: portrait("verdance", air="#84e3b5", lon0=1.0),
    "ochre": lambda: portrait("ochre", air="#e69d69", lon0=2.6),
    "lantern": lambda: portrait("lantern"),
    "rime": lambda: portrait("rime", extra=p_rime_extra, tilt=0.15),
    "cinder": lambda: portrait("cinder", extra=p_cinder_extra, lon0=0.6),
    "murk": lambda: portrait("murk", air="#fb902f"),
    "hollow": lambda: portrait("hollow", extra=p_hollow_extra, lon0=0.3),
    "tidewater": lambda: portrait("tidewater", air="#74effe", lon0=math.pi - 0.2),
    "pebble": lambda: portrait("pebble", r=19, lumpy=pebble_edge, tilt=0.2),
    "amber": p_amber,
    "glaucus": lambda: portrait("glaucus", air="#d2f8ff", tilt=0.2),
}


# ---------- resource icons (32 x 32) ------------------------------------------------------------

ICE = ["#15578a", "#1096c3", "#74effe", "#d2f8ff", "#fffefe"]
ROCK = ["#2c2c25", "#434137", "#a09f91", "#bfcad7"]
DROCK = ["#161612", "#2c2c25", "#434137", "#a09f91"]


def prism(sp, x0, y0, w, h, ramp):
    """A hexagonal crystal standing up: lit left face, front face, dark right face, a pointed top."""
    body = sp.poly([(x0, y0 + w // 2), (x0 + w // 2, y0), (x0 + w, y0 + w // 2), (x0 + w, y0 + h), (x0 + w // 2, y0 + h + 2), (x0, y0 + h)])
    sp.part(body, ramp, "cyl_v")
    return body


def r_ice():
    sp = Sprite(32, 32)
    prism(sp, 3, 3, 12, 24, ICE)
    prism(sp, 15, 10, 12, 18, ICE)
    sp.plots([(6, 7), (18, 14)], "#fffefe")
    sp.outline()
    sp.plots([(26, 5), (25, 5), (27, 5), (26, 4), (26, 6)], "#d2f8ff", glow=True)
    sp.plot(26, 5, "#fffefe", glow=True)
    return sp


def r_iron():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(3, 18), (8, 7), (19, 4), (28, 11), (29, 22), (20, 28), (7, 27)]), ROCK, "sphere")
    for x, y, r in ((11, 13, 3), (19, 19, 3), (21, 10, 2), (10, 21, 2)):
        sp.part(sp.circle(x, y, r), ["#561c27", "#8d4526", "#c56d25", "#e69d69"], "sphere")
    sp.plots([(10, 12), (18, 18)], "#eab798")
    sp.outline()
    return sp


def r_sulfur():
    sp = Sprite(32, 32)
    sp.part(sp.ellipse(3, 22, 29, 30), DROCK, "rim")
    for x, y, h in ((6, 10, 14), (13, 3, 22), (21, 9, 15)):
        sp.part(sp.poly([(x, y + 3), (x + 3, y), (x + 6, y + 3), (x + 6, y + h), (x, y + h)]), RAMPS["gold"][:4], "cyl_v")
    sp.plots([(15, 6), (8, 13)], "#fffefe")
    sp.outline()
    return sp


def r_helium3():
    sp = Sprite(32, 32)
    sp.part(sp.rect(12, 2, 19, 6), RAMPS["metal"], "cyl_v")
    sp.part(sp.ellipse(6, 5, 25, 30), RAMPS["metal"], "cyl_v")
    sp.part(sp.rect(10, 11, 21, 24), ["#0c2f44", "#204f63", "#53998b", "#84e3b5"], "flat", base=0)
    sp.outline()
    sp.part(sp.rect(11, 12, 20, 23), ["#53998b", "#84e3b5", "#c1f1d8"], "inset", glow=True)
    sp.plots([(13, 14), (13, 15)], "#fffefe", glow=True)
    sp.plot(9, 9, "#fffefe")
    return sp


def drop(sp, cx=16, top=2, r=11):
    return sp.circle(cx, top + 2 * r - 2, r) | sp.poly([(cx, top), (cx - r + 2, top + r + 2), (cx + r - 2, top + r + 2)])


def r_methane():
    sp = Sprite(32, 32)
    sp.part(drop(sp), ["#733126", "#c56d25", "#fb902f", "#faaf5c"], "sphere")
    sp.outline()
    sp.part(sp.poly([(16, 12), (20, 20), (19, 25), (16, 27), (13, 25), (12, 20)]), ["#191954", "#15578a", "#1096c3", "#74effe", "#d2f8ff"], "inset", glow=True)
    sp.plot(11, 14, "#fffefe")
    return sp


def r_timber():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(3, 20), (20, 9), (26, 16), (9, 27)]), RAMPS["wood"], "cyl_h")
    sp.part(sp.ellipse(19, 8, 28, 19), ["#725244", "#ab8465", "#cfaa81", "#eab798"], "flat", base=2)
    ring = sp.ellipse(21, 10, 26, 17) & ~sp.ellipse(23, 12, 24, 15)
    sp.fill(ring, "#ab8465")
    sp.part(sp.circle(8, 8, 5), RAMPS["green"], "sphere")
    sp.plots([(6, 6), (21, 10)], "#fffefe")
    sp.outline()
    return sp


def r_fish():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(22, 16), (30, 8), (29, 16), (30, 24)]), RAMPS["glass"], "rim")
    sp.part(sp.poly([(10, 10), (15, 5), (18, 11)]), RAMPS["glass"], "rim")
    sp.part(sp.ellipse(2, 9, 25, 23), ["#204f63", "#1096c3", "#74effe", "#d2f8ff"], "cyl_h")
    sp.fill(sp.rect(6, 13, 8, 15), "#fffefe")
    sp.plot(7, 14, INK)
    sp.outline()
    return sp


def r_glowcrystal():
    sp = Sprite(32, 32)
    sp.part(sp.ellipse(3, 23, 29, 30), ["#170916", "#2b1f35", "#3e3551", "#69608b"], "rim")
    sp.outline()
    for x, y, h in ((5, 11, 13), (12, 3, 21), (20, 9, 15)):
        sp.part(sp.poly([(x, y + 3), (x + 3, y), (x + 6, y + 3), (x + 6, y + h), (x, y + h)]), RAMPS["violet"], "cyl_v", glow=True)
    sp.plots([(14, 6), (7, 14)], "#fffefe", glow=True)
    sp.plots([(27, 4), (26, 4), (28, 4), (27, 3), (27, 5)], "#c34988", glow=True)
    return sp


def r_obsidian():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(7, 27), (4, 15), (12, 3), (22, 5), (27, 15), (24, 27)]), ["#170916", "#2b1f35", "#3e3551", "#69608b"], "rim")
    sp.part(sp.poly([(10, 8), (14, 5), (12, 18)]), ["#4f1d83", "#7d2dbd", "#9e3ba2"], "flat", base=1)
    sp.fill(sp.line([(17, 8), (22, 16)], 1), "#69608b")
    sp.plot(12, 7, "#fffefe")
    sp.outline()
    sp.part(sp.ellipse(9, 27, 23, 30) & ~sp.alpha, RAMPS["flame"][:4], "inset", glow=True)
    return sp


def r_regolith():
    sp = Sprite(32, 32)
    pile = sp.poly([(2, 28), (8, 18), (14, 12), (19, 13), (25, 19), (30, 28)])
    sp.part(pile, ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7"], "sphere")
    for x, y in ((9, 24), (15, 20), (20, 25), (24, 23), (12, 27)):
        sp.fill(sp.rect(x, y, x + 1, y), "#3e586e")
    sp.part(sp.ellipse(3, 23, 10, 29), ["#2c2c25", "#434137", "#a09f91", "#bfcad7"], "sphere")
    sp.part(sp.ellipse(21, 24, 27, 29), ["#2c2c25", "#434137", "#a09f91", "#bfcad7"], "sphere")
    sp.plots([(14, 14), (15, 14), (5, 24)], "#fffefe")
    sp.outline()
    sp.plots([(24, 6), (23, 6), (25, 6), (24, 5), (24, 7)], "#84e3b5", glow=True)
    sp.plot(24, 6, "#fffefe", glow=True)
    return sp


def r_water():
    sp = Sprite(32, 32)
    sp.part(drop(sp), ["#191954", "#15578a", "#1096c3", "#74effe", "#d2f8ff"], "sphere", base=2)
    sp.plots([(11, 16), (11, 17), (12, 15)], "#fffefe")
    sp.outline()
    return sp


RESOURCES = {"ice": r_ice, "iron": r_iron, "sulfur": r_sulfur, "helium3": r_helium3, "methane": r_methane, "timber": r_timber,
             "fish": r_fish, "glowcrystal": r_glowcrystal, "obsidian": r_obsidian, "regolith": r_regolith, "water": r_water}

SHEET_REFS = ["armour_plating", "drop_tanks", "emp_burst", "glass_seraph", "cluster_bombs", "afterburner_kit"]


def refs():
    from PIL import Image
    return [("ref " + n, Image.open(os.path.join(REFS, n + ".png")).convert("RGBA")) for n in SHEET_REFS]


def main():
    from PIL import Image
    os.makedirs(OUT, exist_ok=True)
    write_maps()
    items_p, items_r = [], []
    for name, fn in PORTRAITS.items():
        im = fn().image()
        save(im, os.path.join(OUT, "portrait_" + name + ".png"), source="pixelkit (tools/art/planets_art.py)")
        items_p.append((name, im))
    for name, fn in RESOURCES.items():
        im = fn().image()
        save(im, os.path.join(OUT, "res_" + name + ".png"), source="pixelkit (tools/art/planets_art.py)")
        items_r.append((name, im))
    print("portraits and resources written")
    if "--sheet" in sys.argv:
        os.makedirs(SCRATCH, exist_ok=True)
        sheet(items_p + refs(), scale=3, cols=7, cell=(288, 192)).save(os.path.join(SCRATCH, "sheet_portraits.png"))
        sheet(items_r + refs(), scale=3, cols=6, cell=(288, 192)).save(os.path.join(SCRATCH, "sheet_resources.png"))
        maps = [(n, Image.fromarray(MAPS_DONE[n], "RGB").convert("RGBA")) for n in MAPS]
        sheet(maps, scale=3, cols=3).save(os.path.join(SCRATCH, "sheet_maps.png"))
        lights = [(n, Image.fromarray(LIGHTS_DONE[n], "RGB").convert("RGBA")) for n in LIGHTS_DONE]
        sheet(lights, scale=3, cols=3).save(os.path.join(SCRATCH, "sheet_lights.png"))
        print("sheets in", SCRATCH)


if __name__ == "__main__":
    main()
