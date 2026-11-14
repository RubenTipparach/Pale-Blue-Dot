"""Draw the art for the star-systems additions to docs/mockups/planets.html (balatro-pixel-art kit).

    python tools/art/star_systems_art.py            # every new PNG into docs/mockups/planets/
    python tools/art/star_systems_art.py --sheet    # also contact sheets in $ART_SHEET_DIR
                                                    # (default target/art-sheets), beside six of
                                                    # the solitaire collection's sprites

It extends tools/art/planets_art.py (imported, its helpers reused: the map Grid, the lit globe,
the portrait, the night-light towns) and writes only NEW files, so the twelve Aubade bodies'
art is never redrawn:

- `map_<body>.png` (128 x 64) and `portrait_<body>.png` (48 x 48) for the ported Tenebris worlds
  (Nocturne system) and the bodies of Tallow and Gemel; `lights_<body>.png` where a world has
  night lights (Tenebris's homesteads, Sequoia's glowing mycelium, Wick's lanternbloom ring,
  Fallow's camps).
- `species/<id>.png` (32 x 32): an icon per species on the ported and new worlds.
- `res_<resource>.png` (32 x 32): the new resources (redwood, spores, blackwood, emberglass,
  platinum, fleece).
- `species/fauna_tex.png` (32 px a column): box textures for the vignettes' animals that Tenebris's
  own animal atlas (planets/tenebris/animals.png) does not have, laid out as that atlas is
  (body 0..15, head 17..23, legs 25..31).

Every colour is from Sweaft-64 (the kit's save() refuses anything else). Mockup-only art: the
ported worlds' textures are Tenebris's own, copied byte-identical into planets/tenebris/.
"""

import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

import planets_art as PA  # noqa: E402
from pixelkit import RAMPS, Sprite, save, sheet  # noqa: E402

OUT = PA.OUT
SPECIES_OUT = os.path.join(OUT, "species")
SRC = "pixelkit (tools/art/star_systems_art.py)"
WHITE = "#fffefe"
Grid, hexc = PA.Grid, PA.hexc


# ---------- maps: the ported Tenebris worlds (Nocturne) -----------------------------------------

def map_tenebris():
    """Tenebris: blue seas, the yellow-green pasture of its Fields, deserts with red crag rock,
    jungle, mountains and tundra caps (planet_gen.rs biomes)."""
    g = Grid()
    e = g.noise(2101, 1.8, 5)
    m = g.noise(2102, 2.6, 3)
    sea = e < 0.50
    g.ramp(np.clip((e - 0.30) / 0.22, 0, 1), ["#15578a", "#15578a", "#1096c3"], sea, spread=1.2)
    land = ~sea
    g.put(land, "#9bb675")                                  # DryGrass pasture
    g.put(land & (m > 0.56), "#559642")
    g.put(land & (m > 0.64), "#297537")                     # jungle
    des = land & (m < 0.36) & (np.abs(g.lat) < 0.8)
    g.put(des, "#cfaa81")
    g.put(des & (g.noise(2103, 9, 2) > 0.66), "#a8633e")    # crag rock outcrops in the desert
    g.put(land & (e > 0.68), "#a09f91")
    g.put(land & (e > 0.73), WHITE)
    PA.caps(g, 22, 24, 2104)
    lights = PA.towns(g, land & (np.abs(g.lat) < 1.0), 40, 2105, big=0.08)
    PA.clouds(g, 2106, 0.68, freq=2.4)
    return g, lights


def map_sequoia():
    """Sequoia: an archipelago in dark green water (~55% sea), tan sod (it has no green grass),
    rust-red redwood groves, pale lavender mycelium flats, slate mountains, small caps, brown cloud."""
    g = Grid()
    e = g.noise(2201, 3.0, 5)
    m = g.noise(2202, 3.6, 3)
    sea = e < 0.53
    g.ramp(np.clip((e - 0.32) / 0.22, 0, 1), ["#0c2f44", "#204f63", "#204f63", "#53998b"], sea, spread=1.2)
    river = (np.abs(g.noise(2207, 2.2, 3) - 0.5) < 0.012) & ~sea
    land = ~sea
    g.put(land, "#ab8465")                                  # tan sequoia sod
    g.put(land & (m > 0.50), "#8d4526")                     # redwood groves, rust crowns
    g.put(land & (m > 0.50) & (g.bayer < 0.3), "#a8633e")
    g.put(land & (m > 0.62), "#733126")                     # the alien jungle, dense
    myc = land & (m < 0.36)
    g.put(myc, "#69608b")                                   # Sporewood / fungal mycelium
    g.put(myc & (g.bayer < 0.25), "#92a3b2")
    g.put(land & (e > 0.70), "#5d778d")                     # slate
    g.put(land & (e > 0.76), WHITE)
    g.put(river, "#204f63")
    PA.caps(g, 9, 10, 2203, cols=("#c1f1d8", WHITE))
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    rnd = np.random.default_rng(2204)
    ys, xs = np.nonzero(myc)
    for k in rnd.choice(len(ys), size=min(150, len(ys)), replace=False):
        lights[ys[k], xs[k]] = hexc("#7d2dbd" if rnd.random() < 0.7 else "#c34988")
    ys, xs = np.nonzero(land & (m > 0.62))
    for k in rnd.choice(len(ys), size=min(40, len(ys)), replace=False):
        lights[ys[k], xs[k]] = hexc("#84e3b5")               # glow-vines under the jungle canopy
    c = g.noise(2205, 2.6, 4, stretch=(1, 1, 3.2))          # brown cloud decks
    g.put((c > 0.66) & (g.bayer < 0.5), "#ab8465")
    g.put(c > 0.70, "#cfaa81")
    return g, lights


def _barren(seed, ramp, n_craters, crater_cols, tint_spots=None):
    g = Grid()
    n = g.noise(seed, 3.0, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ramp, spread=1.0)
    rnd = np.random.default_rng(seed + 1)
    for _ in range(n_craters):
        g.crater(rnd.uniform(0, 128), rnd.uniform(6, 58), float(rnd.choice([1.5, 2, 2.5, 3, 4])), *crater_cols)
    if tint_spots:
        col, k = tint_spots
        for _ in range(k):
            g.img[int(rnd.uniform(4, 60)), int(rnd.uniform(0, 128))] = hexc(col)
    return g


def map_luna():
    return _barren(2301, ["#434137", "#a09f91", "#a09f91", "#bfcad7"], 28, ("#a09f91", "#2c2c25", "#bfcad7")), None


def map_selene():
    return _barren(2311, ["#2c2c25", "#434137", "#434137", "#a09f91"], 22, ("#434137", "#161612", "#a09f91")), None


def map_verdant():
    return _barren(2321, ["#2c2c25", "#434137", "#a09f91", "#9bb675"], 18, ("#434137", "#161612", "#a09f91")), None


def map_sprout():
    return _barren(2331, ["#434137", "#434137", "#a09f91", "#bfcad7"], 12, ("#434137", "#2c2c25", "#bfcad7")), None


def map_quartz():
    return _barren(2341, ["#434137", "#a09f91", "#bfcad7", "#bfcad7"], 14, ("#a09f91", "#434137", WHITE), ("#d2f8ff", 40)), None


def map_crag():
    """Crag: Tenebris's rusty red barren planet (moon.png tinted 1.30, 0.55, 0.35)."""
    g = _barren(2401, ["#561c27", "#733126", "#8d4526", "#a8633e"], 30, ("#733126", "#2e0d06", "#e69d69"))
    hi = g.noise(2403, 2.2, 3)
    g.put(hi > 0.64, "#a8633e")
    g.put(g.edge(hi, 0.64, 0.03), "#8d4526")
    return g, None


def _ice(seed, ramp, cracks=2):
    g = Grid()
    n = g.noise(seed, 2.4, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ramp, spread=1.2)
    for s in range(cracks):
        crack = np.abs(g.noise(seed + 10 + s, 1.6, 3) - 0.5) < 0.010
        g.put(crack, "#191954")
        g.put(np.roll(crack, -1, 0) & ~crack & (g.bayer < 0.5), "#74effe")
    return g


def map_frostbite():
    """Frostbite: snow crust over deep glacial ice (ice.png tinted 0.55, 0.80, 1.25)."""
    g = _ice(2501, ["#15578a", "#1096c3", "#1096c3", "#74effe", "#d2f8ff"], 3)
    PA.caps(g, 30, 30, 2502, cols=("#d2f8ff", WHITE))
    return g, None


def map_tenrime():
    g = _ice(2511, ["#92a3b2", "#bfcad7", "#d2f8ff", "#d2f8ff", WHITE], 1)
    rnd = np.random.default_rng(2512)
    for _ in range(8):
        g.crater(rnd.uniform(0, 128), rnd.uniform(8, 56), float(rnd.choice([1.5, 2, 3])), "#bfcad7", "#5d778d", WHITE)
    return g, None


def map_nbelt():
    return _barren(2601, ["#2c2c25", "#434137", "#434137", "#a09f91"], 36, ("#434137", "#161612", "#a09f91"), ("#bfcad7", 10)), None


# ---------- maps: Tallow (a red dwarf) ----------------------------------------------------------

def map_sconce():
    """Sconce: a flare-stripped iron world, airless; black iron plains, rust, molten seams."""
    g = Grid()
    n = g.noise(3101, 2.6, 4)
    g.ramp(np.clip((n - 0.3) / 0.4, 0, 1), ["#161612", "#2c2c25", "#41322c", "#522b2f"], spread=1.2)
    rnd = np.random.default_rng(3102)
    for _ in range(20):
        g.crater(rnd.uniform(0, 128), rnd.uniform(6, 58), float(rnd.choice([1.5, 2, 3])), "#2c2c25", "#161612", "#725244")
    seam = (np.abs(g.noise(3103, 2.0, 3) - 0.5) < 0.014)
    g.put(seam, "#a61e1f")
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    lights[seam] = hexc("#fe6420")
    return g, lights


def map_wick():
    """Wick: tidally locked to Tallow. A scorched dayside (centred on longitude 0), a ring of
    black-leaved life and ring seas at the terminator, an ice cap over the whole night side."""
    g = Grid()
    # angle from the substellar point (lon 0, lat 0)
    sub = np.degrees(np.arccos(np.clip(g.p[..., 0], -1, 1)))
    n = g.noise(3201, 2.8, 4)
    a = sub + (n - 0.5) * 28
    g.ramp(np.clip(a / 60, 0, 1), ["#e69d69", "#cfaa81", "#ab8465"], a < 60, spread=1.0)
    ring = (a >= 60) & (a < 108)
    g.put(ring, "#2f1525")
    g.put(ring & (g.noise(3202, 5, 3) > 0.55), "#522b2f")
    g.put(ring & (g.noise(3203, 3, 3) < 0.38), "#204f63")      # the ring seas
    g.put((a >= 108), "#bfcad7")
    g.put((a >= 116), "#d2f8ff")
    g.put(g.edge(a, 108, 3), "#92a3b2")
    lights = np.zeros_like(g.img)
    lights[:] = hexc("#010000")
    rnd = np.random.default_rng(3204)
    ys, xs = np.nonzero(ring & (g.noise(3203, 3, 3) >= 0.38))
    for k in rnd.choice(len(ys), size=min(120, len(ys)), replace=False):
        lights[ys[k], xs[k]] = hexc("#faaf5c" if rnd.random() < 0.6 else "#fb902f")
    return g, lights


def map_censer():
    g = Grid()
    warp = g.noise(3301, 3.0, 4)
    b = np.sin(g.lat * 9 + warp * 2.0) * 0.5 + 0.5
    g.ramp(b, ["#2e0d06", "#681712", "#841c1d", "#a61e1f", "#d4462b"], spread=1.5)
    g.put(g.disc(40, 24, 4, aspect=1.8)[2], "#fe6420")
    g.put(g.disc(40, 24, 1.6, aspect=1.8)[2], "#fb902f")
    return g, None


def map_snuff():
    g = _ice(3401, ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7"], 1)
    soot = g.noise(3402, 2, 3)
    g.put(soot > 0.6, "#2c2c25")
    g.put(g.edge(soot, 0.6, 0.03), "#434137")
    return g, None


# ---------- maps: Gemel (an orange dwarf with a white-dwarf companion) --------------------------

def map_fallow():
    """Fallow: golden steppe, cold desert, a lake country and glacier highlands; few clouds."""
    g = Grid()
    e = g.noise(4101, 2.0, 5)
    m = g.noise(4102, 2.8, 3)
    g.put(np.ones_like(e, bool), "#ab8465")
    g.put(m > 0.42, "#cfaa81")                               # the steppe
    g.put((m > 0.42) & (g.bayer < 0.3), "#9bb675")
    g.put(m > 0.62, "#559642")
    lake = (e < 0.36)
    g.put(lake, "#15578a")
    g.put(g.edge(e, 0.36, 0.02), "#1096c3")
    g.put(e > 0.68, "#a09f91")
    g.put(e > 0.73, WHITE)
    PA.caps(g, 28, 30, 4103)
    lights = PA.towns(g, ~lake & (np.abs(g.lat) < 0.9), 18, 4104, big=0.05)
    PA.clouds(g, 4105, 0.72, freq=2.2)
    return g, lights


def map_shale():
    """Shale: layered slate, tide-sheared into cliffs; airless."""
    g = Grid()
    warp = g.noise(4201, 2.4, 3)
    b = (np.sin(g.lat * 22 + warp * 6) * 0.5 + 0.5)
    g.ramp(b, ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2"], spread=1.4)
    rnd = np.random.default_rng(4202)
    for _ in range(14):
        g.crater(rnd.uniform(0, 128), rnd.uniform(6, 58), float(rnd.choice([1.5, 2, 3])), "#3e586e", "#1f3b52", "#bfcad7")
    return g, None


def map_lode():
    g = _barren(4301, ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7"], 30, ("#5d778d", "#1f3b52", WHITE), (WHITE, 26))
    return g, None


MAPS = {
    "tenebris": map_tenebris, "sequoia": map_sequoia, "luna": map_luna, "selene": map_selene,
    "verdant": map_verdant, "sprout": map_sprout, "crag": map_crag, "quartz": map_quartz,
    "frostbite": map_frostbite, "tenrime": map_tenrime, "nbelt": map_nbelt,
    "sconce": map_sconce, "wick": map_wick, "censer": map_censer, "snuff": map_snuff,
    "fallow": map_fallow, "shale": map_shale, "lode": map_lode,
}


def write_maps():
    from PIL import Image
    for name, fn in MAPS.items():
        g, lights = fn()
        g.save(os.path.join(OUT, "map_" + name + ".png"))
        PA.MAPS_DONE[name] = g.img.copy()
        if lights is not None:
            save(Image.fromarray(lights, "RGB").convert("RGBA"), os.path.join(OUT, "lights_" + name + ".png"), source=SRC)
            PA.LIGHTS_DONE[name] = lights


# ---------- portraits ---------------------------------------------------------------------------

def star_portrait(ramp_rays, ramp_disc, r_out=23, r_in=15, r_disc=15, spots=None, companion=None):
    def draw():
        sp = Sprite(48, 48)
        sp.part(sp.star(24, 24, r_out, r_in, 12), ramp_rays, "inset", glow=True)
        sp.part(sp.circle(24, 24, r_disc), ramp_disc, "sphere", base=len(ramp_disc) - 2)
        for x, y in (spots or []):
            sp.fill(sp.rect(x, y, x + 1, y), ramp_disc[1])
        sp.plots([(24 - r_disc // 2, 24 - r_disc // 2 + 1), (24 - r_disc // 2 + 1, 24 - r_disc // 2)], WHITE)
        if companion:
            cx, cy = companion
            sp.part(sp.star(cx, cy, 6, 2, 4), ["#74effe", "#d2f8ff", WHITE], "inset", glow=True)
            sp.fill(sp.circle(cx, cy, 1), WHITE, glow=True)
        return sp
    return draw


def lumpy(seed):
    return lambda a: 0.82 + 0.10 * np.sin(a * 3 + seed) + 0.06 * np.sin(a * 5 + seed * 2)


def censer_portrait():
    return PA.portrait("censer", r=21, tilt=0.15)


PORTRAITS = {
    "nocturne": star_portrait(RAMPS["flame"], ["#841c1d", "#c56d25", "#fb902f", "#faaf5c", WHITE], spots=[(18, 26), (28, 30)]),
    "tallow": star_portrait(["#4a051e", "#a61e1f", "#d4462b", "#fe6420"], ["#4a051e", "#841c1d", "#d4462b", "#fe6420", "#fb902f"],
                            r_out=22, r_in=17, r_disc=17, spots=[(16, 24), (27, 31), (30, 20)]),
    "gemel": star_portrait(["#8d4526", "#c56d25", "#fb902f", "#faaf5c"], ["#8d4526", "#c56d25", "#fb902f", "#faaf5c", WHITE],
                           r_out=19, r_in=13, r_disc=13, companion=(40, 9)),
    "gemelb": star_portrait(["#15578a", "#1096c3", "#74effe", "#d2f8ff"], ["#74effe", "#d2f8ff", WHITE], r_out=14, r_in=6, r_disc=6),
    "tenebris": lambda: PA.portrait("tenebris", air="#74effe", lon0=0.8),
    "sequoia": lambda: PA.portrait("sequoia", r=22, air="#e69d69", lon0=1.6),
    "luna": lambda: PA.portrait("luna", r=16),
    "selene": lambda: PA.portrait("selene", r=12),
    "verdant": lambda: PA.portrait("verdant", r=12),
    "sprout": lambda: PA.portrait("sprout", r=12, lumpy=lumpy(1.0)),
    "crag": lambda: PA.portrait("crag", r=19, lon0=0.4),
    "quartz": lambda: PA.portrait("quartz", r=12),
    "frostbite": lambda: PA.portrait("frostbite", r=19, tilt=0.2),
    "tenrime": lambda: PA.portrait("tenrime", r=12),
    "nbelt": lambda: PA.portrait("nbelt", r=17, lumpy=lumpy(2.0), tilt=0.2),
    "sconce": lambda: PA.portrait("sconce", r=18, lon0=0.2),
    "wick": lambda: PA.portrait("wick", r=20, air="#e69d69", lon0=-0.9, tilt=0.1),
    "censer": censer_portrait,
    "snuff": lambda: PA.portrait("snuff", r=14),
    "fallow": lambda: PA.portrait("fallow", r=20, air="#74effe", lon0=2.0),
    "shale": lambda: PA.portrait("shale", r=18),
    "lode": lambda: PA.portrait("lode", r=17, lumpy=lumpy(4.0), tilt=0.15),
}


# ---------- species icons (32 x 32) -------------------------------------------------------------
# A few body plans, each drawn back to front with masks, one ramp per material, light top left.

def eye(sp, x, y, big=False, col=WHITE):
    if big:
        sp.fill(sp.rect(x, y, x + 2, y + 2), col)
        sp.fill(sp.rect(x + 1, y + 1, x + 2, y + 2), "#010000")
        sp.plot(x, y, WHITE)
    else:
        sp.fill(sp.rect(x, y, x + 1, y + 1), "#010000")
        sp.plot(x, y, col)


def biped(body, belly=None, legs=None, head_r=7, body_box=(9, 13, 23, 26), extra=None, after=None, big_eye=True):
    """A compact two-legged creature: big head, round body, short arms (Sequoia's line)."""
    def draw():
        sp = Sprite(32, 32)
        lg = legs or [body[0]] + body[:-1]
        if extra:
            extra(sp, "back")
        x0, y0, x1, y1 = body_box
        sp.part(sp.rect(x0 + 2, y1 - 1, x0 + 4, 30) | sp.rect(x1 - 4, y1 - 1, x1 - 2, 30), lg, "rim")
        sp.part(sp.rect(x0, 30, x0 + 5, 30) | sp.rect(x1 - 5, 30, x1, 30), lg, "flat", base=0)
        b = sp.ellipse(x0, y0, x1, y1)
        sp.part(b, body, "sphere")
        if belly:
            sp.part(sp.ellipse(x0 + 4, y0 + 5, x1 - 3, y1 - 1) & b, belly, "rim", seam=False)
        sp.part(sp.rect(x0 - 2, y0 + 6, x0 + 1, y0 + 8), body, "rim")            # arm
        hx, hy = (x0 + x1) // 2 + 1, y0 - head_r + 3
        h = sp.circle(hx, hy, head_r)
        sp.part(h, body, "sphere")
        eye(sp, hx + 1, hy - 2, big=big_eye)
        if after:
            after(sp, hx, hy)
        sp.outline()
        if extra:
            extra(sp, "glow")
        return sp
    return draw


def quad(body, legs=None, box=(4, 12, 26, 24), head=(26, 10, 6), extra=None, after=None, leg_len=5):
    """A four-legged grazer seen from the side, facing right."""
    def draw():
        sp = Sprite(32, 32)
        lg = legs or [body[0]] + body[:-1]
        x0, y0, x1, y1 = box
        for lx in (x0 + 2, x0 + 7, x1 - 8, x1 - 3):
            sp.part(sp.rect(lx, y1 - 2, lx + 2, min(31, y1 + leg_len)), lg, "rim")
        b = sp.ellipse(x0, y0, x1, y1)
        sp.part(b, body, "cyl_h")
        if extra:
            extra(sp, b)
        hx, hy, hr = head
        sp.part(sp.ellipse(hx - hr, hy - hr + 1, hx + hr, hy + hr - 1), body, "sphere")
        eye(sp, hx + 1, hy - 2, big=False)
        if after:
            after(sp, hx, hy)
        sp.outline()
        return sp
    return draw


def bird(body, wing, legs="#c56d25", tall=False, spread=False, beak="#fb902f", after=None):
    def draw():
        sp = Sprite(32, 32)
        if tall:
            sp.part(sp.rect(13, 19, 14, 30) | sp.rect(17, 19, 18, 30), [legs, legs], "flat")
            sp.part(sp.ellipse(7, 11, 23, 21), body, "sphere")
            sp.part(sp.poly([(9, 13), (21, 13), (16, 19)]), wing, "rim")
            sp.part(sp.rect(20, 3, 22, 13), body, "rim")
            sp.part(sp.circle(23, 4, 3), body, "sphere")
            sp.part(sp.poly([(26, 3), (31, 5), (26, 6)]), [beak, beak], "flat")
            eye(sp, 23, 3)
        elif spread:
            sp.part(sp.poly([(1, 8), (14, 13), (14, 18), (3, 15)]), wing, "rim")
            sp.part(sp.poly([(30, 8), (17, 13), (17, 18), (28, 15)]), wing, "rim")
            sp.part(sp.ellipse(11, 10, 20, 26), body, "sphere")
            sp.part(sp.circle(15, 9, 4), body, "sphere")
            sp.part(sp.poly([(14, 12), (17, 12), (15, 16)]), [beak, beak], "flat")
            eye(sp, 13, 8)
            eye(sp, 17, 8)
        if after:
            after(sp)
        sp.outline()
        return sp
    return draw


def serpent():
    sp = Sprite(32, 32)
    pts = [(2, 26), (8, 20), (14, 25), (20, 18), (24, 10), (27, 6)]
    sp.part(sp.line(pts, 5), RAMPS["glass"], "rim")
    sp.part(sp.line([(4, 27), (8, 23), (14, 27), (20, 21)], 1), ["#84e3b5", "#c1f1d8"], "flat", seam=False)
    sp.part(sp.ellipse(23, 2, 31, 9), RAMPS["glass"], "sphere")
    sp.part(sp.poly([(21, 4), (24, 0), (25, 4)]), ["#4a051e", "#a61e1f", "#d12d41"], "rim")
    eye(sp, 27, 4)
    sp.fill(sp.rect(0, 29, 31, 31) & ~sp.alpha, "#15578a")
    sp.outline()
    return sp


def frog():
    sp = Sprite(32, 32)
    sp.part(sp.ellipse(4, 18, 14, 29) | sp.ellipse(18, 18, 28, 29), RAMPS["green"], "rim")
    sp.part(sp.ellipse(6, 10, 26, 27), RAMPS["green"], "sphere")
    sp.part(sp.ellipse(10, 20, 22, 27), ["#9bb675", "#c1f1d8"], "rim", seam=False)
    for x in (8, 19):
        sp.part(sp.circle(x + 2, 9, 3), RAMPS["green"], "sphere")
        eye(sp, x + 1, 8, big=True)
    sp.fill(sp.line([(11, 18), (21, 18)], 1), "#003227")
    sp.outline()
    return sp


def lizard():
    sp = Sprite(32, 32)
    sp.part(sp.taper(2, 22, 12, 19, 1, 5), RAMPS["olive"], "rim")
    sp.part(sp.ellipse(9, 15, 24, 23), RAMPS["olive"], "cyl_h")
    for lx in (11, 20):
        sp.part(sp.rect(lx, 21, lx + 2, 26), RAMPS["olive"], "rim")
    sp.part(sp.ellipse(21, 12, 30, 19), RAMPS["olive"], "sphere")
    eye(sp, 26, 13)
    sp.outline()
    return sp


def penguin():
    sp = Sprite(32, 32)
    sp.part(sp.ellipse(8, 6, 24, 29), RAMPS["night"], "sphere")
    sp.part(sp.ellipse(11, 12, 22, 28), RAMPS["paper"], "rim")
    sp.part(sp.poly([(19, 11), (25, 12), (19, 13)]), ["#c56d25", "#fb902f"], "flat")
    sp.part(sp.rect(10, 29, 14, 30) | sp.rect(18, 29, 22, 30), ["#c56d25", "#fb902f"], "flat")
    eye(sp, 17, 9)
    sp.outline()
    return sp


def flora(kind):
    def draw():
        sp = Sprite(32, 32)
        if kind == "lanternbloom":
            sp.part(sp.rect(15, 12, 16, 30), ["#2f1525", "#522b2f"], "rim")
            sp.part(sp.poly([(15, 22), (6, 18), (9, 24)]) | sp.poly([(16, 20), (26, 15), (23, 22)]), ["#170916", "#2f1525", "#522b2f"], "rim")
            sp.outline()
            sp.part(sp.circle(16, 9, 6), ["#c56d25", "#fb902f", "#faaf5c", WHITE], "inset", glow=True)
            sp.plots([(14, 7)], WHITE, glow=True)
        return sp
    return draw


def _ears(sp, hx, hy):
    sp.part(sp.poly([(hx - 5, hy - 4), (hx - 3, hy - 13), (hx, hy - 6)]), RAMPS["gold"][:4], "rim")
    sp.part(sp.poly([(hx + 1, hy - 6), (hx + 5, hy - 13), (hx + 6, hy - 4)]), RAMPS["gold"][:4], "rim")
    sp.fill(sp.rect(hx - 3, hy - 11, hx - 3, hy - 10) | sp.rect(hx + 5, hy - 11, hx + 5, hy - 10), "#010000")


def _spark_tail(sp, phase):
    if phase == "back":
        sp.part(sp.line([(8, 24), (4, 20), (6, 16), (2, 12)], 2), RAMPS["gold"][:4], "rim")
    else:
        sp.plots([(2, 11), (1, 12), (3, 12), (2, 13), (2, 12)], "#faaf5c", glow=True)
        sp.plot(2, 12, WHITE, glow=True)


def _shell(sp, phase):
    if phase == "back":
        return
    sp.part(sp.ellipse(4, 12, 20, 25) & ~sp.rect(0, 23, 31, 31), ["#2e0d06", "#733126", "#a8633e", "#e69d69"], "sphere")


def _cap_back(sp, phase):
    if phase == "back":
        return
    if phase == "cap":
        return
    if phase == "glow":
        sp.plots([(7, 13), (12, 11), (17, 12)], "#c34988", glow=True)


def _cap_head(sp, hx, hy):
    """Gloomhopper's mushroom-cap-coloured back, worn as a cap over the head."""
    cap = sp.ellipse(hx - 9, hy - 9, hx + 8, hy - 2)
    sp.part(cap, ["#4a051e", "#9e3ba2", "#c34988", "#dd576d"], "sphere")
    for x, y in ((hx - 5, hy - 6), (hx + 1, hy - 7), (hx + 4, hy - 5)):
        sp.fill(sp.rect(x, y, x + 1, y), "#eab798")


SPECIES = {
    # Sequoia, built in Tenebris (fauna.rs: the roster unique to Sequoia)
    "sparkit": biped(RAMPS["gold"][:4], belly=["#cfaa81", "#eab798"], extra=_spark_tail, after=_ears),
    "shellkin": biped(["#05582d", "#297537", "#559642", "#9bb675"], belly=["#cfaa81", "#eab798"], extra=_shell, big_eye=False),
    "crane": bird(["#92a3b2", "#bfcad7", WHITE], ["#3e586e", "#5d778d", "#92a3b2"], tall=True, legs="#a61e1f", beak="#c56d25"),
    "raptor": bird(["#41322c", "#725244", "#ab8465", "#cfaa81"], ["#2e0d06", "#733126", "#a8633e"], spread=True, beak="#faaf5c"),
    "serpent": serpent,
    # Sequoia, planned in SEQUOIA_PLAN.md (not built in Tenebris)
    "gloomhopper": biped(["#4a051e", "#9e3ba2", "#c34988", "#dd576d"], belly=["#eab798", "#fffefe"], body_box=(7, 15, 25, 28),
                         head_r=6, extra=_cap_back, after=_cap_head),
    "sporeling": biped(["#69608b", "#92a3b2", "#bfcad7", WHITE], body_box=(8, 10, 24, 26), head_r=4, big_eye=False),
    "barkstrider": biped(["#41322c", "#725244", "#ab8465", "#cfaa81"], belly=["#a8633e", "#e69d69"], body_box=(10, 12, 22, 24), head_r=5),
    "mossback": quad(["#41322c", "#725244", "#ab8465"], box=(2, 13, 25, 25), head=(26, 16, 5),
                     extra=lambda sp, b: sp.part(sp.ellipse(3, 6, 23, 19), ["#003227", "#05582d", "#297537", "#559642"], "sphere")),
    # Tenebris, built (fauna.rs: the classic rosters)
    "critter": quad(["#41322c", "#725244", "#ab8465", "#cfaa81"], box=(5, 13, 24, 23), head=(25, 11, 5)),
    "stalker": quad(["#161612", "#2c2c25", "#434137"], box=(3, 11, 25, 22), head=(26, 10, 5), leg_len=7,
                    after=lambda sp, hx, hy: sp.plots([(hx + 1, hy - 2), (hx + 2, hy - 2)], "#d12d41")),
    "sheep": quad(["#a09f91", "#bfcad7", WHITE], legs=["#161612", "#2c2c25", "#434137"], box=(3, 10, 25, 24), head=(26, 12, 5)),
    "frog": frog,
    "lizard": lizard,
    "penguin": penguin,
    # Wick (Tallow), new
    "tallowback": quad(["#725244", "#cfaa81", "#eab798", WHITE], legs=["#2f1525", "#522b2f", "#725244"], box=(2, 11, 25, 25), head=(26, 15, 5)),
    "dimwing": bird(["#170916", "#2f1525", "#522b2f"], ["#2f1525", "#522b2f", "#7d4636"], spread=True, beak="#fb902f",
                    after=lambda sp: sp.plots([(13, 8), (17, 8)], "#faaf5c")),
    "lanternbloom": flora("lanternbloom"),
    # Fallow (Gemel), new
    "drover": quad(["#ab8465", "#cfaa81", "#eab798"], legs=["#41322c", "#725244", "#ab8465"], box=(2, 9, 25, 24), head=(26, 12, 5),
                   after=lambda sp, hx, hy: sp.part(sp.line([(hx - 2, hy - 4), (hx - 4, hy - 9), (hx - 1, hy - 11)], 2), RAMPS["bone"], "rim")),
    "scythewing": bird(["#3e586e", "#5d778d", "#92a3b2", "#bfcad7"], ["#1f3b52", "#3e586e", "#5d778d"], spread=True, beak="#faaf5c"),
    "burrowkit": biped(["#7d4636", "#a8633e", "#e69d69", "#eab798"], belly=["#eab798", WHITE], body_box=(9, 15, 23, 27), head_r=6,
                       after=_ears),
}


# ---------- resource icons (32 x 32) ------------------------------------------------------------

def log_icon(bark, rings):
    def draw():
        sp = Sprite(32, 32)
        sp.part(sp.poly([(3, 12), (22, 6), (28, 20), (9, 26)]), bark, "cyl_h")
        end = sp.ellipse(20, 6, 29, 20)
        sp.part(end, rings, "rim", base=2)
        sp.fill(sp.ellipse(23, 10, 26, 16), rings[1])
        sp.fill(sp.rect(24, 12, 25, 13), rings[0])
        sp.plots([(8, 13), (9, 12)], WHITE)
        sp.outline()
        return sp
    return draw


def r_spores():
    sp = Sprite(32, 32)
    sp.part(sp.ellipse(6, 10, 26, 30), ["#41322c", "#725244", "#ab8465", "#cfaa81"], "sphere")
    sp.part(sp.rect(12, 6, 20, 11), ["#41322c", "#725244", "#ab8465"], "rim")
    sp.outline()
    for x, y in ((10, 4), (15, 2), (21, 4), (18, 7)):
        sp.part(sp.circle(x, y, 1), ["#c34988", "#dd576d"], "flat", glow=True)
    return sp


def r_emberglass():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(16, 2), (26, 12), (22, 29), (10, 29), (6, 12)]), ["#681712", "#d4462b", "#fe6420", "#fb902f", "#faaf5c"], "inset")
    sp.fill(sp.line([(16, 3), (16, 28)], 1), "#faaf5c")
    sp.plots([(12, 9), (11, 10)], WHITE)
    sp.outline()
    return sp


def r_platinum():
    sp = Sprite(32, 32)
    sp.part(sp.poly([(4, 18), (10, 10), (28, 10), (28, 22), (22, 28), (4, 28)]), RAMPS["metal"], "rim", depth=2)
    sp.part(sp.poly([(10, 10), (28, 10), (22, 17), (4, 17)]), RAMPS["paper"], "rim")
    sp.plots([(12, 12), (13, 12)], WHITE)
    sp.outline()
    return sp


def r_fleece():
    sp = Sprite(32, 32)
    for x, y, r in ((10, 16, 7), (20, 14, 8), (16, 22, 8), (24, 22, 6), (8, 23, 5)):
        sp.part(sp.circle(x, y, r), RAMPS["bone"], "sphere")
    sp.plots([(17, 9), (18, 9)], WHITE)
    sp.outline()
    return sp


RESOURCES = {
    "redwood": log_icon(["#2e0d06", "#733126", "#8d4526", "#a8633e"], ["#725244", "#ab8465", "#cfaa81", "#eab798"]),
    "blackwood": log_icon(["#170916", "#2f1525", "#522b2f", "#7d4636"], ["#522b2f", "#7d4636", "#a8633e", "#e69d69"]),
    "spores": r_spores, "emberglass": r_emberglass, "platinum": r_platinum, "fleece": r_fleece,
}


# ---------- fauna_tex: box textures for the vignettes' animals ----------------------------------
# Columns in the same layout as Tenebris's animals.png (body 0..15, head 17..23, legs 25..31).

FAUNA_TEX = {
    "gloomhopper": (["#9e3ba2", "#c34988"], ["#c34988", "#dd576d"], ["#4a051e", "#9e3ba2"]),
    "sporeling": (["#92a3b2", "#bfcad7"], ["#bfcad7", WHITE], ["#41322c", "#725244"]),
    "barkstrider": (["#41322c", "#725244"], ["#725244", "#ab8465"], ["#2e0d06", "#41322c"]),
    "mossback": (["#05582d", "#297537"], ["#725244", "#ab8465"], ["#41322c", "#725244"]),
    "tallowback": (["#cfaa81", "#eab798"], ["#eab798", WHITE], ["#2f1525", "#522b2f"]),
    "dimwing": (["#170916", "#2f1525"], ["#2f1525", "#522b2f"], ["#170916", "#2f1525"]),
    "drover": (["#cfaa81", "#eab798"], ["#ab8465", "#cfaa81"], ["#41322c", "#725244"]),
    "scythewing": (["#3e586e", "#5d778d"], ["#5d778d", "#92a3b2"], ["#c56d25", "#fb902f"]),
    "burrowkit": (["#a8633e", "#e69d69"], ["#e69d69", "#eab798"], ["#41322c", "#7d4636"]),
}


def fauna_tex():
    from PIL import Image
    from pixelkit import BAYER4, rgb
    names = list(FAUNA_TEX)
    img = np.zeros((32, 32 * len(names), 4), np.uint8)
    for i, n in enumerate(names):
        for (y0, y1), (c1, c2) in zip(((0, 16), (16, 24), (24, 32)), FAUNA_TEX[n]):
            for y in range(y0, y1):
                for x in range(32):
                    c = c2 if BAYER4[y % 4, x % 4] < 6 else c1
                    img[y, i * 32 + x, :3] = rgb(c)
                    img[y, i * 32 + x, 3] = 255
        for ex in (10, 21):                                      # eyes on the head strip
            img[19:21, i * 32 + ex:i * 32 + ex + 2, :3] = rgb("#010000")
            img[19, i * 32 + ex, :3] = rgb(WHITE)
    save(Image.fromarray(img, "RGBA"), os.path.join(SPECIES_OUT, "fauna_tex.png"), source=SRC)
    return names


def main():
    from PIL import Image
    os.makedirs(SPECIES_OUT, exist_ok=True)
    write_maps()
    items_p, items_s, items_r = [], [], []
    for name, fn in PORTRAITS.items():
        im = fn().image()
        save(im, os.path.join(OUT, "portrait_" + name + ".png"), source=SRC)
        items_p.append((name, im))
    for name, fn in SPECIES.items():
        im = fn().image()
        save(im, os.path.join(SPECIES_OUT, name + ".png"), source=SRC)
        items_s.append((name, im))
    for name, fn in RESOURCES.items():
        im = fn().image()
        save(im, os.path.join(OUT, "res_" + name + ".png"), source=SRC)
        items_r.append((name, im))
    print("fauna_tex columns:", fauna_tex())
    print(len(MAPS), "maps,", len(items_p), "portraits,", len(items_s), "species,", len(items_r), "resources")
    if "--sheet" in sys.argv:
        scratch = PA.SCRATCH
        os.makedirs(scratch, exist_ok=True)
        sheet(items_p + PA.refs(), scale=3, cols=7, cell=(288, 192)).save(os.path.join(scratch, "ss_portraits.png"))
        sheet(items_s + items_r + PA.refs(), scale=3, cols=8, cell=(220, 160)).save(os.path.join(scratch, "ss_species.png"))
        maps = [(n, Image.fromarray(PA.MAPS_DONE[n], "RGB").convert("RGBA")) for n in MAPS]
        sheet(maps, scale=2, cols=4).save(os.path.join(scratch, "ss_maps.png"))
        print("sheets in", scratch)


if __name__ == "__main__":
    main()
