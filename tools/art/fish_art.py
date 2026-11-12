"""Pixel-art fish for the fish and aquarium mockup (docs/mockups/fish.html).

Drawn in code with the balatro-pixel-art kit (.claude/skills/balatro-pixel-art/scripts/pixelkit.py),
the solitaire collection's style: a tiny native canvas, one bold subject filling it, 3 to 5 tone
ramps lit from the top left, dark seams between parts, a 1 px outline, single-pixel glints, no
anti-aliasing, every colour from Sweaft-64 (the kit's save() refuses anything else).

    python tools/art/fish_art.py [OUT_DIR]          (default docs/mockups/fish)
    python tools/art/fish_art.py OUT_DIR --sheet SHEET.png

Writes a 16 x 16 icon for each of the five new species, drawn in the manner of the game's own fish
icons (the other eight species use those icons, copied into docs/mockups/fish/game/), one swatch per
pattern type (32 x 32) and one icon per aquarium decor item (24 x 24). The page shows them at whole
multiples with nearest-neighbour scaling.
"""

import math
import os
import random
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts")
sys.path.insert(0, KIT)

from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402

# ---------------------------------------------------------------- ramps -------
# Every ramp dark to light, every entry a Sweaft-64 colour.
R = dict(RAMPS)
R.update({
    "slate":    ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2"],
    "pale":     ["#92a3b2", "#bfcad7", "#fffefe"],
    "ice":      ["#5d778d", "#92a3b2", "#bfcad7", "#d2f8ff"],
    "sea":      ["#011c34", "#15578a", "#1096c3", "#74effe"],
    "deepsea":  ["#010e1a", "#011c34", "#1f3b52", "#3e586e"],
    "yolk":     ["#733126", "#c56d25", "#fb902f", "#faaf5c"],
    "cream":    ["#ab8465", "#cfaa81", "#eab798", "#fffefe"],
    "ember":    ["#a61e1f", "#d4462b", "#fe6420"],
    "teal":     ["#0c2f44", "#204f63", "#53998b", "#84e3b5"],
    "mint":     ["#53998b", "#84e3b5", "#c1f1d8"],
    "moss":     ["#003227", "#05582d", "#297537", "#559642"],
    "lichen":   ["#297537", "#559642", "#9bb675"],
    "bruise":   ["#170916", "#2b1f35", "#3e3551", "#69608b"],
    "dusk":     ["#2b1f35", "#3e3551", "#69608b"],
    "basalt":   ["#010000", "#161612", "#2c2c25", "#434137"],
    "ash":      ["#2c2c25", "#434137", "#a09f91"],
    "coal":     ["#2e0d06", "#522b2f", "#7d4636"],
    "lure":     ["#15578a", "#1096c3", "#74effe", "#d2f8ff", "#fffefe"],
    "photo":    ["#1096c3", "#74effe", "#d2f8ff"],
    "coral":    ["#841c1d", "#d12d41", "#dd576d", "#fa3a64"],
    "kelp":     ["#003227", "#05582d", "#297537", "#559642", "#9bb675"],
    "rock":     ["#2b1f35", "#3e3551", "#69608b", "#92a3b2"],
    "bubble":   ["#15578a", "#74effe", "#d2f8ff"],
})

# ---------------------------------------------------------------- species icons -----
# The game already ships 16 x 16 icons for eight fish (assets/items/fish; the page shows copies from
# fish/game/). Only the five new species are drawn here, in those icons' manner: 16 x 16, head to the
# right, a mid body tone with a darker rim of the same hue inside the silhouette, one light patch, a
# 1 px dark eye, a transparent ground. The colours are Sweaft-64.
from pixelkit import erode  # noqa: E402

DARK_EYE = "#010000"


def body16(s, m, dark, mid, light_m=None, light="#fffefe"):
    """Fill m in mid with a 1 px rim of dark inside it, as the game's fish icons are drawn."""
    s.fill(m, mid)
    s.fill(m & ~erode(m), dark)
    if light_m is not None:
        s.fill(light_m & erode(m), light)


def icon_bloatfin():
    s = Sprite(16, 16)
    tail = s.poly([(4, 7), (1, 5), (1, 10), (4, 8)])
    body16(s, tail, "#8d4526", "#fb902f")
    body = s.ellipse(3, 3, 13, 12)
    body16(s, body, "#c56d25", "#faaf5c", s.ellipse(6, 9, 10, 11), "#eab798")
    for x, y in ((6, 5), (9, 4), (7, 8), (10, 7), (5, 10)):
        s.plot(x, y, "#8d4526")
    for x, y in ((8, 2), (5, 3), (11, 3), (14, 6), (13, 10), (8, 13), (4, 12)):
        s.plot(x, y, "#eab798")
    s.plot(11, 6, DARK_EYE)
    return s


def icon_sunplate():
    s = Sprite(16, 16)
    body16(s, s.rect(1, 6, 3, 10), "#733126", "#a8633e")
    body16(s, s.poly([(5, 4), (11, 3), (5, 3)]) | s.rect(5, 3, 10, 4), "#733126", "#a8633e")
    body = s.ellipse(3, 4, 14, 13)
    body16(s, body, "#c56d25", "#fb902f", s.ellipse(6, 10, 11, 12), "#faaf5c")
    for x, y in ((5, 7), (6, 6), (7, 7), (8, 6), (9, 7), (6, 9), (7, 10), (8, 9), (9, 10), (10, 9)):
        s.plot(x, y, "#1096c3")
    s.plot(11, 8, "#161612")
    s.plot(11, 9, "#d12d41")
    s.plot(12, 6, DARK_EYE)
    return s


def icon_lanternjaw():
    s = Sprite(16, 16)
    body16(s, s.poly([(4, 9), (1, 6), (1, 13), (4, 10)]), "#2b1f35", "#3e3551")
    body = s.ellipse(3, 6, 14, 14)
    body16(s, body, "#2b1f35", "#3e3551", s.rect(6, 12, 10, 12), "#69608b")
    s.fill(s.poly([(14, 9), (10, 11), (14, 13)]), "#170916")
    for x, y in ((13, 10), (12, 12)):
        s.plot(x, y, "#fffefe")
    s.plots([(9, 6), (9, 5), (10, 4), (11, 3), (12, 3)], "#69608b")
    s.plot(13, 4, "#74effe")
    s.plot(13, 3, "#d2f8ff")
    for x in (5, 7, 9):
        s.plot(x, 12, "#74effe")
    s.plot(11, 8, DARK_EYE)
    return s


def icon_glassback():
    s = Sprite(16, 16)
    body16(s, s.poly([(4, 7), (0, 3), (2, 7), (0, 12), (4, 8)]), "#5d778d", "#92a3b2")
    body16(s, s.poly([(7, 5), (10, 3), (10, 5)]), "#5d778d", "#92a3b2")
    body = s.ellipse(3, 4, 14, 11)
    body16(s, body, "#92a3b2", "#bfcad7", s.rect(6, 9, 10, 9), "#d2f8ff")
    s.fill(s.rect(4, 7, 11, 7), "#5d778d")
    for x in (5, 7, 9):
        s.plot(x, 6, "#5d778d")
        s.plot(x + 1, 8, "#5d778d")
    s.plots([(6, 9), (9, 9)], "#1096c3")
    s.plot(12, 6, DARK_EYE)
    return s


def icon_emberback():
    s = Sprite(16, 16)
    body16(s, s.rect(1, 5, 3, 10), "#2e0d06", "#522b2f")
    body16(s, s.poly([(5, 4), (6, 2), (7, 4), (8, 2), (9, 4), (10, 2), (11, 4)]) | s.rect(5, 3, 11, 4), "#2e0d06", "#522b2f")
    body = s.ellipse(3, 3, 14, 11)
    body16(s, body, "#161612", "#2c2c25", s.rect(6, 9, 10, 9), "#434137")
    s.plots([(5, 6), (6, 6), (7, 7), (8, 7), (10, 5), (11, 6), (9, 9)], "#fe6420")
    s.plots([(6, 6), (8, 7)], "#faaf5c")
    s.plot(12, 6, "#fffefe")
    return s


SPECIES_ICONS = {"bloatfin": icon_bloatfin, "sunplate": icon_sunplate, "lanternjaw": icon_lanternjaw,
                 "glassback": icon_glassback, "emberback": icon_emberback}
GAME_FISH = ("minnow", "silverfin", "perch", "ray", "eel", "reef", "deepback", "serpent")


def ramp(x):
    return R[x] if isinstance(x, str) else x


def toned(s, m, rp, t):
    """Paint mask m with ramp rp using an existing tone map t (so overlays keep the base's light)."""
    n = len(rp)
    for i in range(n):
        s.fill(m & (np.clip(t, 0, n - 1) == i), rp[i])


# ---------------------------------------------------------------- swatches -----
PATTERNS = {
    "gradient": ("slate", "pale", ["#011c34", "#1f3b52", "#3e586e", "#5d778d"]),
    "stripes":  ("brass", None, ["#191954", "#15578a", "#1096c3", "#1096c3"]),
    "bars":     ("yolk", None, ["#003227", "#003227", "#05582d", "#297537"]),
    "spots":    ("sand", None, ["#41322c", "#725244", "#725244", "#ab8465"]),
    "mottled":  ("lichen", None, ["#003227", "#05582d", "#297537"]),
    "biolum":   ("bruise", None, None),
}


def swatch(name):
    s = Sprite(32, 32)
    base, low, acc = PATTERNS[name]
    rb = ramp(base)
    tile = s.rect(1, 1, 30, 30)
    for x, y in ((1, 1), (30, 1), (1, 30), (30, 30)):
        tile &= ~s.rect(x, y, x, y)
    s.part(tile, rb, "rim", depth=2)
    t = s.tones(tile, rb, "rim", depth=2)
    rnd = random.Random(300 + list(PATTERNS).index(name))
    pm = s.empty()
    if name == "gradient":
        pm = tile & s.rect(0, 0, 31, 10)
        toned(s, tile & s.rect(0, 20, 31, 31), R["pale"], t - 1)
        s.dither(tile & s.rect(2, 11, 29, 13), rb[1], ramp(acc)[2], 0.5)
        s.dither(tile & s.rect(2, 18, 29, 19), rb[2], R["pale"][1], 0.5)
    elif name == "stripes":
        for k in range(5):
            up = [(x, 4 + k * 5.5 + 1.4 * math.sin(x * 0.45 + k)) for x in range(0, 33, 2)]
            pm |= tile & s.line(up, 2)
    elif name == "bars":
        for k in range(4):
            x0 = 4 + k * 7
            pm |= tile & s.poly([(x0, 0), (x0 + 3, 0), (x0 + 2, 31), (x0 - 1, 31)])
    elif name == "spots":
        for _ in range(16):
            x, y = rnd.uniform(4, 27), rnd.uniform(4, 27)
            pm |= tile & s.circle(x, y, rnd.choice((1, 1, 1.5)))
    elif name == "mottled":
        for _ in range(8):
            x, y = rnd.uniform(4, 27), rnd.uniform(4, 27)
            rx, ry = rnd.uniform(2.5, 4.5), rnd.uniform(1.5, 3)
            pm |= tile & s.ellipse(x - rx, y - ry, x + rx, y + ry)
    if acc:
        toned(s, pm, ramp(acc), t)
    s.outline()
    s.plot(3, 3, "#fffefe") if name != "biolum" else None
    if name == "biolum":
        for row in range(4):
            for k in range(4):
                x, y = 6 + k * 7 + (row % 2) * 3, 6 + row * 7
                if x > 27:
                    continue
                s.plot(x, y, "#d2f8ff", glow=True)
                s.plot(x - 1, y, "#74effe", glow=True)
                s.plot(x + 1, y, "#74effe", glow=True)
                s.plot(x, y - 1, "#74effe", glow=True)
                s.plot(x, y + 1, "#1096c3", glow=True)
    return s


# ---------------------------------------------------------------- decor (24 x 24) ----
def d_rock():
    s = Sprite(24, 24)
    s.part(s.poly([(1, 22), (3, 12), (8, 5), (15, 4), (20, 9), (22, 22)]), R["rock"], "sphere")
    s.part(s.poly([(8, 22), (9, 16), (12, 14), (15, 16), (16, 22)]), R["deepsea"][:2], "flat", base=0)
    s.part(s.ellipse(5, 8, 10, 11), R["moss"], "rim")
    s.outline()
    s.plot(10, 6, "#fffefe")
    return s


def d_kelp():
    s = Sprite(24, 24)
    for k, (x0, top, rp) in enumerate(((7, 3, R["kelp"][:4]), (15, 6, R["kelp"][1:]))):
        pts = [(x0 + 2.2 * math.sin(t * 5 + k * 2), 22 - t * (22 - top)) for t in (i / 10 for i in range(11))]
        for i in (2, 4, 6, 8):
            x, y = pts[i]
            sd = 1 if (i // 2 + k) % 2 else -1
            s.part(s.poly([(x, y), (x + sd * 5, y - 3), (x + sd * 4, y - 1)]), R["lichen"], "rim")
        s.part(s.line(pts, 3), rp, "cyl_v")
    s.outline()
    s.plot(6, 4, "#9bb675")
    return s


def d_coral():
    s = Sprite(24, 24)
    s.part(s.ellipse(3, 19, 20, 23), R["sand"], "rim")
    m = s.line([(12, 20), (12, 12)], 4)
    m |= s.line([(12, 14), (6, 9), (5, 4)], 3) | s.line([(12, 13), (18, 8), (18, 3)], 3)
    m |= s.line([(6, 9), (2, 7)], 2) | s.line([(18, 8), (22, 6)], 2) | s.line([(12, 12), (12, 5)], 3)
    for x, y in ((5, 3), (18, 2), (12, 4), (2, 6), (21, 5)):
        m |= s.circle(x, y, 1.2)
    s.part(m, R["coral"], "cyl_v")
    s.outline()
    s.plot(4, 3, "#fffefe")
    return s


def d_sand():
    s = Sprite(24, 24)
    s.part(s.poly([(0, 18), (5, 13), (11, 14), (17, 11), (23, 15), (23, 22), (0, 22)]), R["sand"], "cyl_h")
    for x, y in ((4, 19), (9, 20), (14, 18), (19, 20), (7, 17)):
        s.plot(x, y, "#725244")
    s.part(s.poly([(10, 14), (12, 10), (15, 14)]), R["cream"], "rim")
    s.outline()
    s.plot(12, 11, "#fffefe")
    return s


def d_driftwood():
    s = Sprite(24, 24)
    s.part(s.poly([(14, 15), (16, 5), (18, 5), (18, 15)]), [R["wood"][0]] + R["wood"][:-1], "cyl_v")
    s.part(s.poly([(1, 18), (8, 14), (16, 13), (22, 9), (23, 12), (17, 17), (9, 19), (2, 21)]), R["wood"], "cyl_h")
    for x in (6, 11, 15):
        s.plot(x, 17, "#41322c")
        s.plot(x + 1, 17, "#41322c")
    s.outline()
    s.plot(10, 15, "#eab798")
    return s


def d_heater():
    s = Sprite(24, 24)
    s.part(s.rect(8, 2, 15, 22), R["metal"], "cyl_v")
    s.outline()
    s.part(s.rect(10, 9, 13, 20), R["flame"][:4], "cyl_v", glow=True)
    for y in (10, 14, 18):
        s.plot(17, y, "#fb902f", glow=True)
        s.plot(18, y - 1, "#faaf5c", glow=True)
        s.plot(19, y - 1, "#fb902f", glow=True)
    s.plot(9, 3, "#fffefe")
    return s


def d_chiller():
    s = Sprite(24, 24)
    s.part(s.rect(4, 7, 19, 21), R["ice"], "rim", depth=2)
    for k in range(4):
        s.fill(s.rect(7 + k * 3, 10, 7 + k * 3, 18), "#3e586e")
    s.outline()
    s.plot(5, 8, "#fffefe")
    for cx, cy in ((19, 3), (5, 3)):
        s.plot(cx, cy, "#fffefe", glow=True)
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            s.plot(cx + dx, cy + dy, "#74effe", glow=True)
    return s


def d_lamp():
    s = Sprite(24, 24)
    s.part(s.rect(2, 5, 21, 9), R["metal"], "cyl_h")
    s.outline()
    s.part(s.rect(4, 10, 19, 11), R["lure"][2:], "flat", base=1, glow=True)
    for x0, dx in ((6, -2), (12, 0), (17, 2)):
        s.dither(s.line([(x0, 13), (x0 + dx, 22)], 2), "#1096c3", "#74effe", 0.5, glow=True)
    s.plot(4, 6, "#fffefe")
    return s


def d_bubbler():
    s = Sprite(24, 24)
    s.part(s.rect(6, 19, 17, 22), R["soot"], "rim")
    s.outline()
    for x, y, r in ((11, 15, 2), (14, 9, 3), (10, 3, 2)):
        s.fill(s.circle(x, y, r) & ~s.circle(x, y, r - 1), "#74effe", glow=True)
        s.fill(s.circle(x, y, r - 1), "#15578a", glow=True)
        s.plot(x - 1, y - 1, "#fffefe", glow=True)
    return s


DECOR = {"rock-cave": d_rock, "kelp": d_kelp, "coral": d_coral, "sand-bed": d_sand,
         "driftwood": d_driftwood, "heater": d_heater, "chiller": d_chiller, "lamp": d_lamp, "bubbler": d_bubbler}


def main(argv):
    out = argv[0] if argv and not argv[0].startswith("--") else os.path.join(ROOT, "docs", "mockups", "fish")
    os.makedirs(out, exist_ok=True)
    items = []
    for name, fn in SPECIES_ICONS.items():
        img = fn().image()
        save(img, os.path.join(out, f"species-{name}.png"), "tools/art/fish_art.py (pixelkit)")
        items.append((name, img))
    for name in PATTERNS:
        img = swatch(name).image()
        save(img, os.path.join(out, f"pattern-{name}.png"), "tools/art/fish_art.py (pixelkit)")
        items.append((name, img))
    for name, fn in DECOR.items():
        img = fn().image()
        save(img, os.path.join(out, f"decor-{name}.png"), "tools/art/fish_art.py (pixelkit)")
        items.append((name, img))
    print("wrote", len(items), "PNGs to", out)
    if "--sheet" in argv:
        from PIL import Image
        path = argv[argv.index("--sheet") + 1]
        game = [("game " + n, Image.open(os.path.join(ROOT, "assets", "items", "fish", n + ".png")).convert("RGBA")) for n in GAME_FISH]
        sheet(game + items[:5], scale=6, cols=7).save(path)
        refs = []
        rdir = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "references")
        for r in ("brood_mother", "glass_seraph", "furnace_barge", "cruise_missile", "afterburner_kit", "brine_hive"):
            refs.append(("ref " + r, Image.open(os.path.join(rdir, r + ".png")).convert("RGBA")))
        sheet(items[5:] + refs, scale=3, cols=8).save(path.replace(".png", "-icons.png"))


if __name__ == "__main__":
    main(sys.argv[1:])
