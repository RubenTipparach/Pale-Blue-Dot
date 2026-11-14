"""Food, ingredients, kitchen stations, new crops and kitchen textures for the cooking mockup
(docs/mockups/cooking.html, "Hearth and Kitchen").

Pixel art drawn in code with the balatro-pixel-art kit
(.claude/skills/balatro-pixel-art/scripts/pixelkit.py), the style of the owner's solitaire collection:
a tiny native canvas, one bold subject, 3 to 5 tone ramps lit from the top left, dark seams, a 1 px
outline, single-pixel glints, no anti-aliasing, Sweaft-64 colours only (`save()` refuses anything else),
alpha 0 or 255.

What is drawn here, and at what size:
  - item icons (ingredients, meat cuts by state, fish fillets, dishes, drinks): 24 x 24, the size of the
    farming mockup's produce (M8) and the animals mockup's drops (M7), shown at 24, 48 or 72 px;
  - kitchen station icons (`st_*`): 32 x 32, the size of the crafting mockup's station icons (M3);
  - new crops: four growth stages `crop_<id>_0..3` at 32 x 32 in M8's layout (stage 0 the seeded mound,
    the plant standing on the bottom centre) and the produce `produce_<id>` at 24 x 24;
  - kitchen textures and cards for the 3D scenes (`tex_*` 32 x 32 tiles, `card_*` cut-outs).

NOT drawn here (copied byte-identical instead, see docs/mockups/cooking/README.md): the game's terrain atlas,
settlement textures and item icons (cooking/game/), M7's animal portraits and drops (cooking/m7/), M8's
produce (cooking/m8/), M25's water life (cooking/m25/) and M3's materials (cooking/m3/).

    python tools/art/cooking_art.py              # writes docs/mockups/cooking/*.png
    python tools/art/cooking_art.py --sheet S    # also a contact sheet (keep it out of the repo)
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "cooking")
IW = 24    # item canvas
SW = 32    # station and crop canvas
BASE = 31  # the ground row of a crop stage
MID = 16
WHITE = "#fffefe"
STEAM = "#bfcad7"

R = dict(RAMPS)
R.update({
    # plants (M8's ramps, so the new crops sit beside the old ones)
    "leaf": ["#003227", "#297537", "#559642", "#9bb675"],
    "young": ["#05582d", "#559642", "#9bb675", "#c1f1d8"],
    "stalk": ["#05582d", "#297537", "#9bb675"],
    "grain": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "durum": ["#733126", "#8d4526", "#c56d25", "#fb902f"],
    "straw": ["#8d4526", "#c56d25", "#cfaa81"],
    "dirt": ["#2f1525", "#41322c", "#725244", "#ab8465"],
    "blue_leaf": ["#0c2f44", "#204f63", "#53998b", "#84e3b5"],
    "silver_leaf": ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "tomato": ["#4a051e", "#a61e1f", "#d4462b", "#fe6420"],
    "onion": ["#561c27", "#733126", "#a8633e", "#e69d69"],
    "garlic": ["#92a3b2", "#bfcad7", "#eab798", "#fffefe"],
    "potato": ["#41322c", "#725244", "#ab8465", "#cfaa81"],
    "beet": ["#230c44", "#4f1d83", "#9e3ba2", "#c34988"],
    "cane": ["#05582d", "#297537", "#9bb675", "#c1f1d8"],
    "olive": ["#1c020d", "#2b1f35", "#3e3551", "#69608b"],
    "olive_g": ["#003227", "#05582d", "#559642", "#9bb675"],
    "bark": ["#2e0d06", "#41322c", "#725244", "#ab8465"],
    "soy": ["#05582d", "#559642", "#9bb675", "#c1f1d8"],
    "bean": ["#8d4526", "#ab8465", "#cfaa81", "#eab798"],
    "flower_w": ["#92a3b2", "#bfcad7", "#fffefe"],
    "flower_y": ["#c56d25", "#fb902f", "#faaf5c"],
    "flower_v": ["#4f1d83", "#7d2dbd", "#9e3ba2"],
    # food
    "crust": ["#2e0d06", "#733126", "#a8633e", "#e69d69"],
    "gold_crust": ["#733126", "#c56d25", "#fb902f", "#faaf5c"],
    "crumb": ["#ab8465", "#cfaa81", "#eab798", "#fffefe"],
    "dough": ["#a09f91", "#cfaa81", "#eab798", "#fffefe"],
    "egg_dough": ["#c56d25", "#fb902f", "#faaf5c", "#eab798"],
    "flour": ["#92a3b2", "#bfcad7", "#fffefe"],
    "semolina": ["#cfaa81", "#eab798", "#faaf5c"],
    "sack": ["#725244", "#ab8465", "#cfaa81", "#eab798"],
    "butter": ["#c56d25", "#fb902f", "#faaf5c", "#fffefe"],
    "cheese": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "cream": ["#a09f91", "#bfcad7", "#fffefe"],
    "sauce": ["#4a051e", "#841c1d", "#a61e1f", "#d4462b"],
    "broth": ["#681712", "#8d4526", "#c56d25", "#fb902f"],
    "dark": ["#010000", "#1c020d", "#2e0d06", "#561c27"],
    "oil": ["#05582d", "#559642", "#9bb675", "#faaf5c"],
    "glass": ["#0c2f44", "#204f63", "#53998b", "#84e3b5", "#c1f1d8"],
    "plate": ["#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "bowl": ["#2e0d06", "#733126", "#a8633e", "#e69d69"],
    "glaze": ["#191954", "#15578a", "#1096c3", "#74effe"],
    "noodle": ["#ab8465", "#cfaa81", "#eab798", "#fffefe"],
    "rice_c": ["#92a3b2", "#bfcad7", "#fffefe"],
    "green": ["#003227", "#297537", "#559642", "#9bb675"],
    "herb": ["#05582d", "#297537", "#559642", "#9bb675"],
    "raw": ["#4a051e", "#a61e1f", "#d12d41", "#dd576d"],
    "cooked": ["#2e0d06", "#733126", "#a8633e", "#e69d69"],
    "smoked": ["#170916", "#2e0d06", "#561c27", "#733126"],
    "cured": ["#561c27", "#841c1d", "#dd576d", "#eab798"],
    "fat": ["#a09f91", "#cfaa81", "#eab798", "#fffefe"],
    "fish_raw": ["#a8633e", "#e69d69", "#eab798", "#fffefe"],
    "fish_cooked": ["#8d4526", "#c56d25", "#cfaa81", "#eab798"],
    "fish_smoked": ["#681712", "#8d4526", "#c56d25", "#fb902f"],
    "fish_salted": ["#92a3b2", "#bfcad7", "#eab798", "#fffefe"],
    "fish_skin": ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2"],
    "shrimp": ["#841c1d", "#d4462b", "#fe6420", "#faaf5c"],
    "shell_red": ["#4a051e", "#a61e1f", "#d4462b", "#fe6420"],
    "batter": ["#8d4526", "#c56d25", "#faaf5c", "#eab798"],
    "fried": ["#733126", "#c56d25", "#fb902f", "#faaf5c"],
    "squid": ["#a09f91", "#cfaa81", "#eab798", "#fffefe"],
    "octo": ["#4a051e", "#841c1d", "#d4462b", "#dd576d"],
    "nori": ["#010e1a", "#0c2f44", "#204f63"],
    "tea": ["#2e0d06", "#681712", "#8d4526", "#c56d25"],
    "melon_j": ["#8d4526", "#fb902f", "#faaf5c", "#fffefe"],
    "ale": ["#681712", "#c56d25", "#fb902f", "#faaf5c"],
    "foam": ["#bfcad7", "#fffefe"],
    "tin": ["#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "label": ["#841c1d", "#d4462b", "#fe6420"],
    "kale_s": ["#003227", "#05582d", "#297537", "#559642"],
    "glowsoup": ["#0c2f44", "#15578a", "#1096c3", "#74effe", "#d2f8ff"],
    "porridge": ["#ab8465", "#cfaa81", "#eab798", "#fffefe"],
    "chowder": ["#a09f91", "#cfaa81", "#eab798", "#fffefe"],
    "stew": ["#2e0d06", "#733126", "#8d4526", "#a8633e"],
    "melon": ["#003227", "#297537", "#559642", "#9bb675"],
    "melon_in": ["#c56d25", "#fb902f", "#faaf5c"],
    "skin": ["#7d4636", "#a8633e", "#e69d69", "#eab798"],
    # stations
    "brick": ["#2e0d06", "#681712", "#841c1d", "#a8633e"],
    "clay": ["#561c27", "#733126", "#a8633e", "#e69d69"],
    "stone": ["#2c2c25", "#434137", "#a09f91", "#bfcad7"],
    "wood": ["#2e0d06", "#41322c", "#725244", "#ab8465"],
    "plank": ["#41322c", "#725244", "#ab8465", "#cfaa81"],
    "iron": ["#010e1a", "#1f3b52", "#3e586e", "#5d778d"],
    "steel": ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "copper": ["#2e0d06", "#733126", "#c56d25", "#fb902f"],
    "fire": ["#a61e1f", "#fe6420", "#fb902f", "#faaf5c", "#fffefe"],
    "ember": ["#4a051e", "#a61e1f", "#fe6420", "#fb902f"],
    "smoke": ["#3e586e", "#5d778d", "#92a3b2"],
    "ice": ["#15578a", "#1096c3", "#74effe", "#d2f8ff"],
    "panel": ["#011c34", "#1f3b52", "#3e586e", "#5d778d"],
    "led": ["#05582d", "#559642", "#c1f1d8"],
    "linen": ["#92a3b2", "#bfcad7", "#eab798", "#fffefe"],
    "kelp": ["#2e0d06", "#733126", "#8d4526", "#c56d25"],
})


# ---------- helpers ------------------------------------------------------------------------------

def leaf(sp, x, y, ang, length, width):
    """A pointed leaf from (x, y) out along `ang` (radians, 0 = right, -pi/2 = up); M8's shape."""
    pts = []
    n = 10
    for i in range(n + 1):
        t = i / n
        pts.append((t * length, -math.sin(t * math.pi) * width * (1 - 0.3 * t)))
    for i in range(n, -1, -1):
        t = i / n
        pts.append((t * length, math.sin(t * math.pi) * width * (1 - 0.3 * t) * 0.6))
    c, s = math.cos(ang), math.sin(ang)
    return sp.poly([(x + px * c - py * s, y + px * s + py * c) for px, py in pts])


def frill(sp, x, y, ang, length, width):
    """A frilly leaf: overlapping balls along a line (M8's kale)."""
    m = sp.empty()
    n = max(2, int(length / 3))
    for i in range(n + 1):
        t = i / n
        r = width * (0.55 + 0.45 * math.sin(t * math.pi * 0.9 + 0.3))
        m |= sp.circle(x + math.cos(ang) * length * t, y + math.sin(ang) * length * t, r)
    return m


def outline(sp):
    sp.outline()
    sp.outlined = True


def spark(sp, x, y, ramp):
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], ramp[-2], glow=True)
    sp.plot(x, y, WHITE, glow=True)


def steam(sp, xs=(8, 13), top=1):
    """Two steam wisps above a hot dish: light, never outlined."""
    for i, x in enumerate(xs):
        for k in range(4):
            sp.plot(x + (1 if (k + i) % 2 else 0), top + k, STEAM, glow=True)


def glint(sp, x, y):
    sp.plot(x, y, WHITE)


# ---------- item templates (24 x 24) -------------------------------------------------------------

def bowl(sp, soup, rim="bowl", hot=True):
    """A bowl seen from a little above: the body, the rim, the soup's surface. Returns the surface mask."""
    body = sp.ellipse(1, 7, 22, 22) & sp.rect(0, 12, 23, 23)
    sp.part(body, R[rim], "cyl_v")
    sp.part(sp.rect(7, 21, 16, 22), R[rim], "rim")
    sp.part(sp.ellipse(1, 8, 22, 16), R[rim], "rim")
    surf = sp.ellipse(3, 9, 20, 15)
    sp.part(surf, R[soup], "sphere", seam=False)
    if hot:
        steam(sp, (8, 14), 1)
    return surf


def plate(sp, y0=13):
    sp.part(sp.ellipse(0, y0, 23, 23), R["plate"], "rim")
    sp.part(sp.ellipse(3, y0 + 2, 20, 21), R["plate"], "flat", base=1, seam=False)


def sack(sp, fill):
    sp.part(sp.poly([(6, 3), (17, 3), (21, 22), (2, 22)]), R["sack"], "rim", depth=2)
    sp.part(sp.ellipse(5, 2, 18, 8), R[fill], "sphere")
    sp.part(sp.rect(7, 9, 16, 10), R["red"], "flat")


def jar(sp, fill, lid="wood"):
    sp.part(sp.rect(6, 3, 17, 6), R[lid], "rim")
    body = sp.ellipse(3, 5, 20, 23) & sp.rect(0, 7, 23, 23)
    sp.part(body, R["glass"], "cyl_v")
    inner = sp.ellipse(5, 9, 18, 22) & sp.rect(0, 10, 23, 23)
    sp.part(inner, R[fill], "cyl_v", seam=False)
    glint(sp, 6, 10)
    return inner


def bottle(sp, fill):
    sp.part(sp.rect(10, 1, 13, 3), R["wood"], "rim")
    sp.part(sp.rect(9, 4, 14, 9), R["glass"], "cyl_v")
    body = sp.rect(5, 9, 18, 22) | sp.ellipse(5, 7, 18, 12)
    sp.part(body, R["glass"], "cyl_v")
    sp.part(sp.rect(7, 12, 16, 21), R[fill], "cyl_v", seam=False)
    sp.part(sp.rect(7, 14, 16, 17), R["paper"], "flat")
    glint(sp, 7, 10)


def nest(sp, ramp, cx=12, cy=14, rx=9, ry=6):
    """A nest of noodles: loops of 1 px strands in alternating tones."""
    base = sp.ellipse(cx - rx, cy - ry, cx + rx, cy + ry)
    sp.part(base, R[ramp], "sphere")
    for k in range(5):
        a0 = k * 1.3
        pts = [(cx + math.cos(a0 + t * 0.5) * rx * (0.3 + 0.12 * t), cy + math.sin(a0 + t * 0.5) * ry * (0.3 + 0.12 * t)) for t in range(7)]
        sp.fill(sp.line(pts, 1) & base, R[ramp][k % 2 + 1] if k % 2 else R[ramp][-1])


# ---------- ingredients --------------------------------------------------------------------------

def i_flour(sp):
    sack(sp, "flour")


def i_semolina(sp):
    sack(sp, "semolina")


def i_dough(sp):
    sp.part(sp.ellipse(2, 9, 21, 22), R["dough"], "sphere")
    sp.fill(sp.line([(6, 15), (10, 14), (14, 16)], 1), R["dough"][0])
    glint(sp, 7, 11)
    for x, y in ((15, 11), (11, 18)):
        sp.plot(x, y, R["flour"][1])


def i_pasta_dough(sp):
    sp.part(sp.ellipse(2, 9, 21, 22), R["egg_dough"], "sphere")
    sp.part(sp.line([(3, 6), (20, 3)], 3), R["wood"], "cyl_h")
    glint(sp, 7, 12)


def i_leaven(sp):
    inner = jar(sp, "dough")
    for x, y in ((9, 13), (13, 16), (11, 19), (15, 12)):
        sp.plot(x, y, R["dough"][0])
    sp.fill(sp.rect(6, 3, 17, 3), R["cream"][-1])


def i_butter(sp):
    sp.part(sp.poly([(2, 13), (8, 9), (21, 11), (21, 19), (15, 22), (2, 20)]), R["plate"], "rim")
    sp.part(sp.poly([(5, 11), (10, 7), (19, 9), (19, 15), (14, 18), (5, 16)]), R["butter"], "rim")
    sp.fill(sp.line([(5, 11), (14, 13), (19, 9)], 1), R["butter"][-1])
    glint(sp, 10, 9)


def i_cheese(sp):
    wedge = sp.poly([(2, 12), (20, 5), (22, 9), (22, 20), (2, 21)])
    sp.part(wedge, R["cheese"], "rim", depth=2)
    sp.part(sp.poly([(2, 12), (20, 5), (22, 9), (4, 14)]), R["butter"], "flat", base=2)
    for x, y, r in ((8, 17, 1.5), (15, 15, 2), (18, 19, 1)):
        sp.fill(sp.circle(x, y, r), R["cheese"][0])
    glint(sp, 12, 9)


def i_sugar(sp):
    cone = sp.poly([(12, 2), (19, 21), (5, 21)])
    sp.part(cone, R["flour"], "cyl_v")
    sp.part(sp.rect(4, 17, 20, 22), R["glaze"], "cyl_h")
    glint(sp, 11, 5)


def i_salt(sp):
    sp.part(sp.ellipse(1, 14, 22, 22), R["wood"], "rim")
    heap = sp.poly([(4, 17), (9, 8), (13, 6), (17, 9), (20, 17)])
    sp.part(heap, R["flour"], "sphere", seam=False)
    for x, y in ((8, 12), (14, 10), (12, 15), (17, 14)):
        sp.plot(x, y, R["ice"][1])
    glint(sp, 11, 8)


def i_oil(sp):
    bottle(sp, "oil")


def i_soy_sauce(sp):
    bottle(sp, "dark")


def i_tomato_sauce(sp):
    jar(sp, "sauce")


def i_stock(sp):
    jar(sp, "broth", lid="iron")


def i_batter(sp):
    s = bowl(sp, "batter", rim="glaze", hot=False)
    sp.part(sp.line([(15, 12), (21, 2)], 2), R["steel"], "flat")
    sp.part(sp.ellipse(12, 10, 17, 15), R["steel"], "rim")


def i_noodles_wheat(sp):
    nest(sp, "noodle")


def i_noodles_rice(sp):
    nest(sp, "rice_c")


def i_pasta_dried(sp):
    for k in range(7):
        x = 6 + k * 1.8
        sp.part(sp.line([(x, 2), (x + 3, 22)], 1), R["egg_dough"] if k % 2 else R["semolina"], "flat")
    sp.part(sp.rect(4, 11, 19, 14), R["red"], "rim")
    glint(sp, 7, 3)


def i_rice_cooked(sp):
    s = bowl(sp, "rice_c", rim="glaze")
    for x, y in ((7, 11), (11, 10), (15, 12), (9, 13), (13, 13)):
        sp.plot(x, y, R["rice_c"][0])


def i_tea_leaves(sp):
    """Dried tea in a lacquered caddy, heaped over the brim."""
    sp.part(sp.rect(4, 9, 19, 22), R["red"], "cyl_v")
    sp.part(sp.rect(4, 13, 19, 14), R["gold"], "flat")
    for x, y, a in ((5, 10, -0.4), (9, 8, -1.2), (14, 9, -2.5), (11, 6, -0.3), (7, 5, -1.7)):
        sp.part(leaf(sp, x, y, a, 7, 2.2), R["herb"] if (x + y) % 2 else R["tea"], "rim")
    glint(sp, 6, 16)


def i_pickles(sp):
    inner = jar(sp, "glass")
    for x, y in ((8, 12), (13, 14), (9, 18), (14, 19)):
        sp.part(sp.ellipse(x - 2, y - 3, x + 2, y + 3), R["green"], "cyl_v")


def i_olives_brined(sp):
    inner = jar(sp, "glass")
    for x, y in ((8, 12), (14, 13), (10, 17), (15, 19), (7, 20)):
        sp.part(sp.ellipse(x - 2, y - 2, x + 2, y + 2), R["olive"], "sphere")
        sp.plot(x - 1, y - 1, R["olive"][-1])


def i_kelp(sp):
    """Dried kelp sheets (nori), from M25's kelp: a proposal."""
    for k in range(3):
        y = 5 + k * 5
        sp.part(sp.rect(3 + k, y, 20 - k, y + 9), R["nori"], "rim")
    glint(sp, 6, 16)


# ---------- meat cuts, by state ------------------------------------------------------------------

STATES = ("raw", "cooked", "smoked", "cured")


def cut_poultry(sp, st):
    sp.part(sp.line([(15, 14), (21, 21)], 3), R["fat"], "flat")
    sp.part(sp.circle(21, 21, 2), R["fat"], "sphere")
    sp.part(sp.circle(19, 22, 1.5), R["fat"], "sphere")
    meat = sp.ellipse(2, 2, 17, 17)
    sp.part(meat, R[st], "sphere", depth=2)
    glint(sp, 6, 5)


def cut_pork(sp, st):
    chop = sp.poly([(2, 6), (10, 2), (20, 4), (22, 12), (17, 21), (6, 21), (2, 14)])
    sp.part(chop, R["fat"], "rim")
    sp.part(sp.poly([(5, 8), (11, 5), (18, 7), (19, 12), (15, 18), (7, 18), (5, 13)]), R[st], "sphere")
    sp.part(sp.line([(12, 6), (12, 19)], 2), R["fat"], "flat")
    glint(sp, 7, 9)


def cut_beef(sp, st):
    steak = sp.ellipse(1, 4, 22, 21)
    sp.part(steak, R["fat"], "rim")
    sp.part(sp.ellipse(3, 6, 20, 19), R[st], "sphere")
    sp.fill(sp.line([(6, 10), (11, 12), (17, 9)], 1), R["fat"][1])
    sp.fill(sp.line([(8, 15), (14, 16)], 1), R["fat"][1])
    glint(sp, 6, 8)


def cut_mutton(sp, st):
    for k in range(3):
        x = 5 + k * 5
        sp.part(sp.rect(x + 1, 2, x + 2, 9), R["fat"], "flat")
        sp.part(sp.ellipse(x - 2, 7, x + 5, 21), R[st], "sphere")
        sp.part(sp.rect(x - 1, 17, x + 4, 20), R["fat"], "rim")
    glint(sp, 4, 10)


def cut_game(sp, st):
    """A haunch of game (hare, shagback): a long leg with the bone out."""
    sp.part(sp.line([(16, 16), (22, 22)], 2), R["fat"], "flat")
    leg = sp.poly([(2, 4), (9, 1), (15, 6), (18, 14), (16, 18), (10, 16), (4, 11)])
    sp.part(leg, R[st], "sphere", depth=2)
    sp.fill(sp.line([(6, 6), (12, 10)], 1), R[st][-1])
    glint(sp, 6, 4)


CUTS = {"poultry": cut_poultry, "pork": cut_pork, "beef": cut_beef, "mutton": cut_mutton, "game": cut_game}


def i_sausage(sp):
    """A string of links."""
    for k, (x0, y0, x1, y1) in enumerate(((1, 4, 12, 13), (11, 10, 22, 19))):
        sp.part(sp.ellipse(x0, y0, x1, y1), R["cooked"], "sphere")
        sp.plot(x0 + 3, y0 + 2, R["cooked"][-1])
    sp.part(sp.line([(11, 11), (13, 12)], 1), R["sack"], "flat")
    sp.part(sp.line([(1, 5), (0, 2)], 1), R["sack"], "flat")
    sp.part(sp.line([(21, 18), (23, 21)], 1), R["sack"], "flat")
    glint(sp, 4, 6)


def i_bacon(sp):
    for k in range(3):
        y = 4 + k * 6
        pts = [(2, y + 3), (7, y), (12, y + 3), (17, y), (22, y + 3)]
        sp.part(sp.line(pts, 4), R["cured"], "cyl_h")
        sp.fill(sp.line([(p[0], p[1] + 1) for p in pts], 1), R["fat"][2])


def i_jerky(sp):
    for k, (x0, y0, x1, y1) in enumerate(((3, 4, 19, 9), (2, 11, 21, 15), (5, 17, 20, 21))):
        sp.part(sp.poly([(x0, y0 + 1), (x1 - 2, y0), (x1, y1 - 1), (x0 + 2, y1)]), R["smoked"], "rim")
    glint(sp, 6, 5)


def fillet(sp, st):
    body = sp.poly([(1, 12), (6, 6), (16, 5), (22, 9), (21, 14), (14, 18), (5, 18)])
    sp.part(body, R["fish_skin"], "rim")
    sp.part(sp.poly([(3, 12), (7, 8), (16, 7), (20, 10), (19, 13), (13, 16), (6, 16)]), R[st], "rim")
    for x in (8, 12, 16):
        sp.fill(sp.line([(x, 8), (x - 2, 15)], 1), R[st][1])
    glint(sp, 7, 9)


# ---------- dishes -------------------------------------------------------------------------------

def d_flatbread(sp):
    sp.part(sp.ellipse(1, 7, 22, 21), R["crumb"], "rim")
    for x, y in ((6, 11), (12, 9), (17, 13), (9, 16), (15, 17)):
        sp.fill(sp.circle(x, y, 1.2), R["crust"][1])
    glint(sp, 5, 9)


def d_bread(sp):
    loaf = sp.ellipse(1, 5, 22, 22) & sp.rect(0, 0, 23, 20)
    sp.part(loaf, R["gold_crust"], "sphere")
    sp.part(sp.rect(1, 18, 22, 21), R["crust"], "rim")
    for x in (6, 11, 16):
        sp.fill(sp.line([(x, 9), (x + 3, 14)], 1), R["crumb"][2])
    glint(sp, 5, 8)


def d_rolls(sp):
    for x, y in ((3, 10), (12, 10), (7, 4)):
        bun = sp.ellipse(x, y, x + 9, y + 9)
        sp.part(bun, R["gold_crust"], "sphere")
        sp.fill(sp.line([(x + 3, y + 3), (x + 6, y + 3)], 1), R["crumb"][2])
        sp.plot(x + 2, y + 2, WHITE)
    sp.part(sp.ellipse(1, 18, 22, 23), R["sack"], "rim")


def d_pastry(sp):
    """A crescent pastry, layered."""
    for k, (x0, x1, y0, y1) in enumerate(((1, 9, 9, 21), (14, 22, 9, 21), (5, 18, 4, 20))):
        sp.part(sp.ellipse(x0, y0, x1, y1), R["gold_crust"], "sphere")
    for x in (8, 11, 14):
        sp.fill(sp.line([(x, 5), (x - 1, 19)], 1), R["crust"][1])
    glint(sp, 7, 6)


def pie(sp, filling, lattice=True):
    sp.part(sp.ellipse(1, 7, 22, 22) & sp.rect(0, 12, 23, 23), R["crust"], "cyl_h")
    sp.part(sp.ellipse(1, 6, 22, 17), R["gold_crust"], "rim")
    inner = sp.ellipse(4, 8, 19, 15)
    sp.part(inner, R[filling], "sphere", seam=False)
    if lattice:
        for x in (7, 11, 15):
            sp.fill(sp.line([(x, 8), (x + 2, 15)], 1) & inner, R["gold_crust"][2])
        sp.fill(sp.line([(4, 11), (19, 12)], 1) & inner, R["gold_crust"][2])
    else:
        sp.part(inner, R["gold_crust"], "sphere", seam=False)
        for x, y in ((9, 11), (14, 11), (11, 13)):
            sp.plot(x, y, R["crust"][0])
    glint(sp, 5, 9)


def d_melon_pie(sp):
    pie(sp, "melon_in")


def d_meat_pie(sp):
    pie(sp, "stew", lattice=False)
    steam(sp, (9, 14), 1)


def d_cake(sp):
    sp.part(sp.ellipse(0, 17, 23, 23), R["plate"], "rim")
    body = sp.rect(3, 7, 20, 19) | sp.ellipse(3, 15, 20, 21)
    sp.part(body, R["crumb"], "cyl_v")
    sp.part(sp.rect(3, 12, 20, 13), R["red"], "flat")
    sp.part(sp.ellipse(3, 4, 20, 10), R["cream"], "rim")
    for x in (6, 11, 16):
        sp.part(sp.circle(x, 5, 1.5), R["red"], "sphere")
    glint(sp, 5, 8)


def d_sweet_bun(sp):
    bun = sp.ellipse(2, 5, 21, 21)
    sp.part(bun, R["gold_crust"], "sphere")
    sp.part(sp.ellipse(5, 6, 18, 14), R["cream"], "rim")
    sp.fill(sp.line([(7, 13), (6, 16)], 1), R["cream"][-1])
    sp.fill(sp.line([(15, 13), (16, 15)], 1), R["cream"][-1])
    glint(sp, 8, 8)


def pizza(sp, toppings):
    sp.part(sp.ellipse(0, 4, 23, 23), R["gold_crust"], "rim")
    sauce = sp.ellipse(3, 7, 20, 20)
    sp.part(sauce, R["sauce"], "flat", base=2, seam=False)
    for x, y, r in ((7, 10, 2.5), (14, 9, 2.5), (10, 15, 3), (16, 15, 2), (6, 16, 1.5)):
        sp.fill(sp.circle(x, y, r) & sauce, R["butter"][2])
        sp.plot(round(x - 1), round(y - 1), R["butter"][-1])
    for kind, x, y in toppings:
        if kind == "herb":
            sp.fill(sp.rect(x, y, x + 1, y), R["herb"][2])
            sp.plot(x, y, R["herb"][3])
        elif kind == "sausage":
            sp.part(sp.circle(x, y, 1.6), R["cured"], "sphere")
        elif kind == "shrimp":
            sp.part(sp.ellipse(x - 2, y - 1, x + 2, y + 1), R["shrimp"], "rim")
        elif kind == "ring":
            sp.fill(sp.circle(x, y, 2) & ~sp.circle(x, y, 1), R["squid"][2])
    sp.fill(sp.line([(12, 6), (12, 21)], 1), R["gold_crust"][0])
    sp.fill(sp.line([(3, 13), (20, 13)], 1), R["gold_crust"][0])


def d_pizza(sp):
    pizza(sp, [("herb", 9, 12), ("herb", 15, 12), ("herb", 12, 17), ("herb", 7, 9)])


def d_pizza_meat(sp):
    pizza(sp, [("sausage", 8, 11), ("sausage", 15, 10), ("sausage", 11, 16), ("sausage", 17, 16), ("herb", 6, 15)])


def d_pizza_sea(sp):
    pizza(sp, [("shrimp", 8, 11), ("shrimp", 15, 16), ("ring", 15, 10), ("ring", 8, 16), ("herb", 12, 13)])


def d_spaghetti(sp):
    plate(sp, 10)
    nest(sp, "egg_dough", 12, 15, 9, 5)
    sp.part(sp.ellipse(7, 10, 17, 16), R["sauce"], "sphere")
    sp.plots([(10, 12), (13, 11)], R["herb"][3])
    steam(sp, (9, 14), 2)


def d_ravioli(sp):
    plate(sp, 10)
    for x, y in ((4, 12), (11, 12), (7, 16), (14, 16)):
        sq = sp.poly([(x + 3, y), (x + 7, y + 3), (x + 3, y + 6), (x - 1, y + 3)])
        sp.part(sq, R["egg_dough"], "rim")
        sp.plot(x + 3, y + 3, R["egg_dough"][0])
    sp.plots([(9, 13), (16, 18)], R["herb"][3])


def d_lasagne(sp):
    block = sp.rect(2, 6, 21, 20)
    sp.part(block, R["iron"], "rim")
    sp.part(sp.rect(4, 8, 19, 18), R["sauce"], "flat", base=2, seam=False)
    for y in (10, 14):
        sp.fill(sp.rect(4, y, 19, y), R["egg_dough"][2])
        sp.fill(sp.rect(4, y + 1, 19, y + 1), R["cream"][1])
    sp.part(sp.rect(4, 8, 19, 9), R["cheese"], "flat", base=2, seam=False)
    sp.dither(sp.rect(4, 8, 19, 9), R["cheese"][2], R["crust"][2], 0.3)
    steam(sp, (8, 14), 1)


def d_ramen(sp):
    surf = bowl(sp, "broth", rim="glaze")
    nest(sp, "egg_dough", 11, 12, 6, 2)
    sp.part(sp.ellipse(14, 9, 19, 13), R["cream"], "sphere")     # half an egg
    sp.part(sp.circle(16, 11, 1.2), R["butter"], "flat")
    sp.part(sp.rect(4, 10, 8, 13), R["cured"], "rim")             # pork
    sp.fill(sp.rect(9, 9, 10, 9), R["herb"][3])


def d_noodle_soup(sp):
    surf = bowl(sp, "broth", rim="bowl")
    nest(sp, "rice_c", 11, 12, 6, 2)
    sp.part(sp.rect(13, 10, 18, 12), R["cooked"], "rim")
    for x, y in ((6, 11), (8, 13), (15, 13)):
        sp.fill(sp.rect(x, y, x + 1, y), R["herb"][2])


def wok(sp):
    sp.part(sp.line([(18, 12), (23, 9)], 2), R["wood"], "flat")
    body = sp.ellipse(1, 6, 21, 22) & sp.rect(0, 13, 23, 23)
    sp.part(body, R["iron"], "cyl_v")
    sp.part(sp.ellipse(1, 8, 21, 17), R["iron"], "rim")
    inner = sp.ellipse(3, 9, 19, 15)
    sp.part(inner, R["iron"], "flat", base=0, seam=False)
    return inner


def d_stir_fry_veg(sp):
    inner = wok(sp)
    for x, y, rmp in ((6, 11, "green"), (10, 10, "tomato"), (14, 11, "green"), (8, 13, "onion"), (12, 13, "flower_y"), (16, 13, "green")):
        sp.part(sp.ellipse(x - 1.5, y - 1, x + 1.5, y + 1), R[rmp], "rim", seam=False)
    steam(sp, (8, 13), 1)


def d_stir_fry_shrimp(sp):
    inner = wok(sp)
    for x, y in ((6, 11), (13, 10), (10, 13)):
        sp.part(sp.ellipse(x - 2, y - 1, x + 2, y + 1), R["shrimp"], "rim", seam=False)
    for x, y in ((9, 10), (16, 12), (14, 13)):
        sp.part(sp.ellipse(x - 1, y - 1, x + 1, y + 1), R["green"], "rim", seam=False)
    sp.plots([(11, 11), (7, 13)], R["chili"][2] if "chili" in R else R["red"][2])
    steam(sp, (8, 13), 1)


def d_fried_rice(sp):
    surf = bowl(sp, "egg_dough", rim="glaze")
    for x, y in ((6, 11), (10, 10), (14, 12), (9, 13), (16, 11)):
        sp.plot(x, y, R["green"][2])
    for x, y in ((8, 12), (12, 11), (15, 13)):
        sp.plot(x, y, R["cured"][2])


def d_spit_roast(sp):
    sp.part(sp.line([(0, 9), (23, 9)], 1), R["steel"], "flat")
    body = sp.ellipse(3, 4, 20, 19)
    sp.part(body, R["gold_crust"], "sphere")
    sp.part(sp.ellipse(1, 12, 7, 20), R["gold_crust"], "sphere")
    sp.part(sp.ellipse(16, 12, 22, 20), R["gold_crust"], "sphere")
    sp.part(sp.rect(4, 19, 6, 22), R["fat"], "flat")
    sp.part(sp.rect(17, 19, 19, 22), R["fat"], "flat")
    glint(sp, 7, 6)


def d_grilled_steak(sp):
    plate(sp, 9)
    sp.part(sp.ellipse(2, 9, 15, 20), R["cooked"], "sphere")
    for x in (5, 8, 11):
        sp.fill(sp.line([(x, 11), (x + 2, 18)], 1), R["cooked"][0])
    sp.part(sp.ellipse(13, 11, 21, 19), R["potato"], "sphere")
    sp.part(sp.rect(16, 12, 18, 13), R["butter"], "flat")
    steam(sp, (6, 11), 3)


def d_stew(sp):
    bowl(sp, "stew")
    for x, y, rmp in ((6, 11, "cooked"), (10, 12, "potato"), (14, 11, "onion"), (16, 13, "cooked"), (8, 13, "tomato")):
        sp.part(sp.rect(x - 1, y - 1, x + 1, y), R[rmp], "rim", seam=False)


def d_skewers(sp):
    for k, y in enumerate((5, 13)):
        sp.part(sp.line([(1, y + 5), (22, y)], 1), R["wood"], "flat")
        for j, x in enumerate((5, 10, 15)):
            yy = y + 4 - (x / 22) * 5
            sp.part(sp.rect(x - 2, yy - 2, x + 2, yy + 2), R["cooked"] if j % 2 == 0 else R["green"], "rim")


def d_sausage_mash(sp):
    plate(sp, 9)
    sp.part(sp.ellipse(3, 10, 13, 19), R["cream"], "sphere")
    sp.part(sp.taper(10, 12, 21, 16, 4, 4), R["cooked"], "cyl_h")
    sp.part(sp.taper(8, 16, 20, 19, 4, 4), R["cooked"], "cyl_h")
    sp.plots([(6, 12), (8, 13)], R["herb"][3])


def d_grilled_fish(sp):
    plate(sp, 9)
    fish = sp.poly([(2, 15), (6, 11), (14, 10), (18, 14), (14, 18), (6, 18)])
    sp.part(fish, R["fish_cooked"], "rim")
    sp.part(sp.poly([(17, 14), (22, 10), (22, 19)]), R["fish_cooked"], "rim")
    for x in (7, 10, 13):
        sp.fill(sp.line([(x, 11), (x + 1, 17)], 1), R["cooked"][0])
    sp.plot(4, 13, R["ink"][0])
    sp.plots([(9, 20), (11, 20)], R["herb"][3])


def d_fish_chips(sp):
    cone = sp.poly([(1, 10), (22, 10), (17, 23), (6, 23)])
    sp.part(cone, R["paper"], "rim")
    sp.part(sp.ellipse(3, 2, 14, 12), R["fried"], "sphere")
    for k in range(4):
        x = 13 + k * 2
        sp.part(sp.rect(x, 3 + k, x + 1, 12), R["butter"], "cyl_v")
    sp.fill(sp.rect(1, 14, 22, 14), R["red"][2])
    glint(sp, 6, 4)


def d_sushi(sp):
    sp.part(sp.rect(0, 15, 23, 21), R["plank"], "rim")
    for k, x in enumerate((2, 9, 16)):
        sp.part(sp.ellipse(x, 9, x + 6, 16), R["rice_c"], "sphere")
        sp.part(sp.ellipse(x, 7, x + 6, 12), R["fish_raw"] if k != 1 else R["fish_salted"], "rim")
        sp.part(sp.rect(x + 2, 8, x + 3, 15), R["nori"], "flat")


def d_calamari(sp):
    plate(sp, 9)
    for x, y in ((6, 12), (12, 11), (17, 13), (9, 16), (15, 17)):
        ring = sp.circle(x, y, 3.2) & ~sp.circle(x, y, 1.4)
        sp.part(ring, R["fried"], "rim")
    sp.plots([(4, 17), (19, 17)], R["herb"][3])


def d_shrimp_skewer(sp):
    sp.part(sp.line([(1, 20), (22, 3)], 1), R["wood"], "flat")
    for x, y in ((6, 15), (11, 11), (16, 7)):
        body = sp.circle(x, y, 3.5) & ~sp.circle(x + 1, y - 1, 1.5)
        sp.part(body, R["shrimp"], "rim")
        sp.plot(x - 2, y - 2, WHITE)


def d_tempura(sp):
    plate(sp, 11)
    for k, (x0, y0, x1, y1) in enumerate(((2, 13, 15, 6), (5, 19, 19, 11))):
        sp.part(sp.taper(x0, y0, x1, y1, 5, 3), R["fried"], "sphere")
        sp.part(sp.poly([(x1, y1 - 2), (x1 + 4, y1 - 4), (x1 + 3, y1 + 1)]), R["shrimp"], "rim")
    sp.dither(sp.rect(3, 8, 18, 18) & sp.taper(2, 13, 15, 6, 5, 3), R["fried"][2], R["fried"][3], 0.3)


def d_crab_boil(sp):
    body = sp.ellipse(4, 8, 19, 19)
    for s in (-1, 1):
        for k in range(3):
            x = 12 + s * (6 + k)
            sp.part(sp.line([(x, 14 + k * 2), (x + s * 3, 18 + k * 2)], 1), R["shell_red"], "flat")
        sp.part(sp.circle(12 + s * 9, 7, 3), R["shell_red"], "sphere")
    sp.part(body, R["shell_red"], "sphere")
    sp.plots([(10, 9), (14, 9)], R["ink"][0])
    glint(sp, 8, 11)


def d_lobster(sp):
    plate(sp, 10)
    tail = sp.poly([(2, 15), (8, 11), (17, 12), (21, 16), (15, 20), (6, 20)])
    sp.part(tail, R["shell_red"], "rim")
    sp.part(sp.poly([(5, 15), (9, 13), (16, 14), (18, 16), (14, 18), (7, 18)]), R["fish_salted"], "sphere")
    sp.part(sp.rect(17, 9, 20, 11), R["butter"], "rim")


def d_chowder(sp):
    """Clam chowder in a bread bowl."""
    body = sp.ellipse(1, 6, 22, 22) & sp.rect(0, 11, 23, 23)
    sp.part(body, R["gold_crust"], "sphere")
    sp.part(sp.ellipse(1, 6, 22, 15), R["gold_crust"], "rim")
    surf = sp.ellipse(4, 8, 19, 13)
    sp.part(surf, R["chowder"], "sphere", seam=False)
    for x, y in ((8, 10), (12, 11), (15, 10)):
        sp.fill(sp.rect(x, y, x + 1, y), R["potato"][2])
    sp.plot(10, 9, R["herb"][3])
    steam(sp, (8, 14), 1)


def d_octopus(sp):
    plate(sp, 10)
    for k, (x0, y0, x1, y1) in enumerate(((3, 18, 12, 11), (10, 19, 20, 13), (5, 13, 14, 15))):
        sp.part(sp.taper(x0, y0, x1, y1, 4, 2), R["octo"], "cyl_h")
    for x, y in ((6, 17), (13, 17), (16, 15)):
        sp.plot(x, y, R["cream"][-1])
    sp.fill(sp.line([(4, 12), (19, 12)], 1), R["oil"][3])


def d_veg_soup(sp):
    bowl(sp, "tomato")
    sp.plots([(8, 11), (13, 12), (10, 13)], R["herb"][3])
    sp.fill(sp.ellipse(11, 10, 15, 12), R["cream"][1])


def d_kale_soup(sp):
    bowl(sp, "kale_s")
    for x, y in ((7, 11), (12, 10), (15, 12)):
        sp.fill(sp.rect(x, y, x + 1, y + 1), R["potato"][3])


def d_porridge(sp):
    bowl(sp, "porridge")
    sp.fill(sp.line([(7, 11), (11, 13), (15, 11)], 1), R["butter"][1])


def d_glowcap_soup(sp):
    surf = bowl(sp, "glowsoup", hot=False)
    for x, y in ((8, 11), (13, 10), (11, 13)):
        sp.fill(sp.rect(x, y, x + 1, y), R["glowsoup"][-1], glow=True)
    outline(sp)
    spark(sp, 4, 3, R["cyan"])
    spark(sp, 19, 4, R["cyan"])


def d_chips(sp):
    box = sp.poly([(4, 10), (19, 10), (17, 22), (6, 22)])
    for k in range(6):
        x = 5 + k * 2.4
        sp.part(sp.rect(x, 2 + (k % 3), x + 1, 12), R["butter"], "cyl_v")
    sp.part(box, R["red"], "rim")
    sp.fill(sp.rect(8, 14, 15, 15), R["paper"][-1])


def d_baked_potato(sp):
    sp.part(sp.ellipse(1, 7, 22, 21), R["potato"], "sphere")
    sp.part(sp.poly([(7, 9), (12, 12), (17, 9), (14, 15), (9, 15)]), R["cream"], "rim")
    sp.part(sp.rect(10, 9, 13, 11), R["butter"], "rim")
    sp.plot(4, 10, WHITE)


def d_salad(sp):
    bowl(sp, "green", rim="glaze", hot=False)
    for x, y in ((7, 11), (14, 10), (11, 13)):
        sp.part(sp.circle(x, y, 1.6), R["tomato"], "sphere", seam=False)
    for x, y in ((9, 10), (16, 12)):
        sp.plot(x, y, R["oil"][3])


def d_tea(sp):
    cup = sp.rect(3, 9, 16, 20) | sp.ellipse(3, 16, 16, 22)
    sp.part(sp.circle(18, 14, 4) & ~sp.circle(18, 14, 2), R["plate"], "rim")
    sp.part(cup, R["plate"], "cyl_v")
    sp.part(sp.ellipse(3, 7, 16, 12), R["plate"], "rim")
    sp.part(sp.ellipse(5, 8, 14, 11), R["tea"], "flat", base=2, seam=False)
    steam(sp, (7, 11), 1)


def d_melon_juice(sp):
    gl = sp.poly([(5, 4), (18, 4), (16, 22), (7, 22)])
    sp.part(gl, R["glass"], "cyl_v")
    sp.part(sp.poly([(6, 8), (17, 8), (15, 21), (8, 21)]), R["melon_j"], "cyl_v", seam=False)
    sp.part(sp.poly([(15, 2), (22, 6), (19, 9)]), R["melon"], "rim")
    sp.part(sp.line([(10, 1), (12, 12)], 1), R["red"], "flat")
    glint(sp, 7, 6)


def d_ale(sp):
    mug = sp.rect(3, 7, 16, 22)
    sp.part(sp.circle(18, 14, 4) & ~sp.circle(18, 14, 2), R["wood"], "rim")
    sp.part(mug, R["wood"], "cyl_v")
    sp.part(sp.rect(3, 10, 16, 11), R["iron"], "flat")
    sp.part(sp.rect(3, 18, 16, 19), R["iron"], "flat")
    sp.part(sp.ellipse(2, 3, 17, 9), R["foam"], "sphere")
    sp.plot(5, 4, WHITE)


def d_ration(sp):
    can = sp.rect(4, 6, 19, 21)
    sp.part(can, R["tin"], "cyl_v")
    sp.part(sp.ellipse(4, 3, 19, 9), R["tin"], "rim")
    sp.part(sp.rect(4, 11, 19, 17), R["label"], "cyl_v")
    sp.part(sp.circle(11, 14, 2), R["paper"], "flat")
    glint(sp, 6, 5)


def d_hardtack(sp):
    for k, (x, y) in enumerate(((2, 9), (10, 5), (8, 13))):
        sq = sp.rect(x, y, x + 11, y + 8)
        sp.part(sq, R["crumb"], "rim")
        for dx, dy in ((3, 2), (7, 2), (3, 5), (7, 5)):
            sp.plot(x + dx, y + dy, R["crust"][1])


def d_omelette(sp):
    plate(sp, 9)
    sp.part(sp.ellipse(3, 10, 20, 19), R["butter"], "sphere")
    sp.fill(sp.line([(5, 14), (18, 14)], 1), R["cheese"][1])
    sp.plots([(8, 12), (14, 12)], R["herb"][3])


def d_cheese_toast(sp):
    sp.part(sp.poly([(2, 8), (21, 6), (22, 19), (3, 21)]), R["gold_crust"], "rim")
    sp.part(sp.poly([(4, 9), (19, 8), (20, 16), (5, 18)]), R["cheese"], "sphere")
    sp.dither(sp.poly([(4, 9), (19, 8), (20, 16), (5, 18)]), R["cheese"][2], R["crust"][2], 0.25)


def i_kelp_frond(sp):
    """A cut kelp frond (M25's kelp forest has the plant; this is the item)."""
    sp.part(sp.line([(4, 22), (8, 14), (7, 6), (11, 1)], 2), R["wood"], "flat")
    for k, (x, y, a) in enumerate(((7, 18, -0.4), (8, 12, -2.6), (8, 8, -0.3), (9, 4, -2.8), (6, 15, -2.9))):
        sp.part(leaf(sp, x, y, a, 13 - k, 3.4), R["olive"] if k % 2 else R["kelp"] if "kelp" in R else R["olive"], "rim")
    sp.plot(12, 11, R["olive"][-1])


def i_seawater(sp):
    inner = jar(sp, "glaze", lid="plank")
    for x, y in ((8, 13), (13, 17)):
        sp.fill(sp.line([(x, y), (x + 3, y)], 1), R["glaze"][-1])


# ---------- proposed catchables (24 x 24, M25's layout: side view head right, crawlers from above) --

def n_sand_prawn(sp):
    """A sand prawn: M25's cleaner shrimp is a reef cleaner the size of a finger; this is the table shrimp."""
    body = sp.circle(11, 13, 9) & ~sp.circle(14, 10, 3) & ~sp.rect(14, 7, 23, 11)
    sp.part(body, R["fish_salted"], "rim")
    for k in range(4):
        a = 2.4 + k * 0.55
        sp.fill(sp.line([(11 + math.cos(a) * 5, 13 + math.sin(a) * 5), (11 + math.cos(a) * 9, 13 + math.sin(a) * 9)], 1) & body, R["fish_salted"][0])
    sp.part(sp.poly([(18, 18), (23, 21), (19, 23)]), R["fish_salted"], "rim")
    for k in range(3):
        sp.part(sp.line([(6, 6 + k), (1, 1 + k * 2)], 1), R["fish_salted"], "flat")
    sp.plot(7, 8, R["ink"][0])


def n_sand_clam(sp):
    shell = sp.ellipse(1, 4, 22, 21) & sp.rect(0, 0, 23, 19)
    sp.part(shell, R["sand"], "sphere")
    for k in range(5):
        sp.fill(sp.line([(12, 18), (3 + k * 4.5, 5 + abs(k - 2) * 2)], 1) & shell, R["sand"][1])
    sp.part(sp.rect(4, 18, 19, 20), R["sand"], "rim", base=0)
    sp.plot(6, 7, WHITE)


def n_rock_mussel(sp):
    for k, (x, y, a) in enumerate(((3, 18, -0.6), (9, 21, -1.1))):
        m = sp.taper(x, y, x + math.cos(a) * 15, y + math.sin(a) * 15, 4, 9)
        sp.part(m, R["night"] + [R["glaze"][1]], "rim")
        sp.fill(sp.line([(x, y), (x + math.cos(a) * 14, y + math.sin(a) * 14)], 1) & m, R["night"][0])
    sp.plot(12, 6, R["glaze"][-1])


def d_mussel_pot(sp):
    body = sp.ellipse(1, 9, 22, 23) & sp.rect(0, 14, 23, 23)
    sp.part(body, R["iron"], "cyl_v")
    sp.part(sp.ellipse(1, 10, 22, 18), R["iron"], "rim")
    for x, y, a in ((5, 14, -0.5), (10, 12, -1.2), (14, 15, -2.4), (17, 12, -0.3), (8, 16, -2.0)):
        sp.part(sp.taper(x, y, x + math.cos(a) * 5, y + math.sin(a) * 5, 2, 4), R["night"], "rim")
        sp.plot(x, y, R["butter"][1])
    sp.plots([(12, 14), (16, 16)], R["herb"][3])
    steam(sp, (8, 14), 3)


ICONS = {
    # basics and processed
    "flour": i_flour, "semolina": i_semolina, "dough": i_dough, "pasta_dough": i_pasta_dough, "leaven": i_leaven,
    "butter": i_butter, "cheese": i_cheese, "sugar": i_sugar, "salt": i_salt, "oil": i_oil, "soy_sauce": i_soy_sauce,
    "tomato_sauce": i_tomato_sauce, "stock": i_stock, "batter": i_batter, "noodles_wheat": i_noodles_wheat,
    "noodles_rice": i_noodles_rice, "pasta_dried": i_pasta_dried, "rice_cooked": i_rice_cooked, "tea_leaves": i_tea_leaves,
    "pickles": i_pickles, "olives_brined": i_olives_brined, "kelp_sheets": i_kelp,
    "sausage": i_sausage, "bacon": i_bacon, "jerky": i_jerky,
    # dishes
    "flatbread": d_flatbread, "bread": d_bread, "rolls": d_rolls, "pastry": d_pastry, "melon_pie": d_melon_pie,
    "meat_pie": d_meat_pie, "cake": d_cake, "sweet_bun": d_sweet_bun, "pizza": d_pizza, "pizza_meat": d_pizza_meat,
    "pizza_sea": d_pizza_sea, "spaghetti": d_spaghetti, "ravioli": d_ravioli, "lasagne": d_lasagne, "ramen": d_ramen,
    "noodle_soup": d_noodle_soup, "stir_fry_veg": d_stir_fry_veg, "stir_fry_shrimp": d_stir_fry_shrimp,
    "fried_rice": d_fried_rice, "spit_roast": d_spit_roast, "grilled_steak": d_grilled_steak, "stew": d_stew,
    "skewers": d_skewers, "sausage_mash": d_sausage_mash, "grilled_fish": d_grilled_fish, "fish_chips": d_fish_chips,
    "sushi": d_sushi, "calamari": d_calamari, "shrimp_skewer": d_shrimp_skewer, "tempura": d_tempura,
    "crab_boil": d_crab_boil, "lobster": d_lobster, "chowder": d_chowder, "octopus": d_octopus,
    "veg_soup": d_veg_soup, "kale_soup": d_kale_soup, "porridge": d_porridge, "glowcap_soup": d_glowcap_soup,
    "chips": d_chips, "baked_potato": d_baked_potato, "salad": d_salad, "tea": d_tea, "melon_juice": d_melon_juice,
    "ale": d_ale, "ration": d_ration, "hardtack": d_hardtack, "omelette": d_omelette, "cheese_toast": d_cheese_toast,
    "seawater": i_seawater, "kelp_frond": i_kelp_frond, "sand_prawn": n_sand_prawn, "sand_clam": n_sand_clam, "rock_mussel": n_rock_mussel,
    "mussel_pot": d_mussel_pot,
}


# ---------- kitchen stations (32 x 32) -----------------------------------------------------------

def flames(sp, x0, x1, y_base, h):
    for k, x in enumerate(range(x0, x1 + 1, 3)):
        hh = h * (0.7 + 0.3 * ((k * 7) % 3) / 2)
        sp.part(sp.taper(x, y_base, x + 1, y_base - hh, 4, 0), R["fire"], "inset", glow=True)


def st_campfire(sp):
    for (a, b) in (((3, 28), (28, 22)), ((4, 22), (28, 28))):
        sp.part(sp.line([a, b], 3), R["wood"], "cyl_h")
    for x in (4, 27):
        sp.part(sp.line([(x, 8), (x, 28)], 2), R["wood"], "flat")
    sp.part(sp.line([(2, 9), (29, 9)], 1), R["steel"], "flat")
    sp.part(sp.ellipse(10, 9, 21, 17), R["gold_crust"], "sphere")
    outline(sp)
    flames(sp, 9, 21, 25, 9)


def st_rack(sp):
    for x in (4, 27):
        sp.part(sp.line([(x, 3), (x, 30)], 2), R["wood"], "flat")
    for y in (6, 17):
        sp.part(sp.line([(3, y), (28, y)], 1), R["wood"], "flat")
    for k, x in enumerate((8, 14, 20)):
        sp.part(sp.poly([(x - 2, 7), (x + 2, 7), (x + 3, 14), (x, 16), (x - 3, 14)]), R["fish_smoked"], "rim")
    for k, x in enumerate((8, 15, 22)):
        sp.part(sp.poly([(x - 2, 18), (x + 2, 18), (x + 3, 26), (x - 3, 26)]), R["herb"] if k != 1 else R["chili"] if "chili" in R else R["red"], "rim")


def st_quern(sp):
    sp.part(sp.ellipse(2, 18, 29, 30), R["stone"], "cyl_v")
    sp.part(sp.ellipse(2, 15, 29, 24), R["stone"], "rim")
    sp.part(sp.ellipse(5, 10, 26, 18) | (sp.rect(5, 14, 26, 19) & sp.ellipse(5, 10, 26, 24)), R["stone"], "cyl_v")
    sp.part(sp.ellipse(5, 9, 26, 15), R["stone"], "rim")
    sp.part(sp.ellipse(13, 11, 18, 13), R["ink"], "flat")
    sp.part(sp.line([(22, 11), (23, 2)], 3), R["wood"], "cyl_v")
    sp.part(sp.ellipse(1, 25, 9, 31), R["flour"], "sphere")


def st_hearth(sp):
    sp.part(sp.rect(2, 4, 29, 30), R["stone"], "rim")
    for y in (9, 15, 21, 27):
        sp.fill(sp.rect(2, y, 29, y), R["stone"][0])
    sp.part(sp.rect(6, 13, 25, 30), R["ink"], "flat")
    sp.part(sp.line([(6, 13), (25, 13)], 1), R["iron"], "flat")
    sp.part(sp.line([(15, 13), (15, 16)], 1), R["iron"], "flat")
    sp.part(sp.ellipse(9, 16, 22, 27), R["iron"], "sphere")
    sp.part(sp.rect(9, 16, 22, 18), R["iron"], "rim")
    outline(sp)
    flames(sp, 8, 23, 30, 4)
    steam(sp, (13, 17), 8)


def st_barrel(sp):
    body = sp.ellipse(4, 3, 27, 30) & sp.rect(0, 5, 31, 28)
    sp.part(body, R["plank"], "cyl_v")
    for x in (10, 16, 22):
        sp.fill(sp.line([(x, 6), (x, 27)], 1) & body, R["plank"][0])
    for y in (8, 25):
        sp.part(sp.rect(4, y, 27, y + 1) & body, R["iron"], "flat")
    sp.part(sp.ellipse(6, 2, 25, 9), R["plank"], "rim")
    sp.part(sp.ellipse(8, 3, 23, 8), R["glass"], "flat", base=1, seam=False)
    for x, y in ((11, 5), (17, 6)):
        sp.plot(x, y, R["flour"][-1])


def st_dairy(sp):
    """Butter churn and cheese press."""
    sp.part(sp.poly([(4, 10), (13, 10), (14, 30), (3, 30)]), R["plank"], "cyl_v")
    sp.part(sp.rect(4, 15, 13, 16), R["iron"], "flat")
    sp.part(sp.line([(8, 1), (8, 10)], 2), R["wood"], "flat")
    sp.part(sp.ellipse(4, 8, 13, 11), R["plank"], "rim")
    for x in (17, 28):
        sp.part(sp.line([(x, 6), (x, 30)], 2), R["wood"], "flat")
    sp.part(sp.rect(16, 6, 29, 8), R["wood"], "rim")
    sp.part(sp.line([(23, 8), (23, 16)], 2), R["iron"], "flat")
    sp.part(sp.rect(18, 16, 27, 18), R["plank"], "rim")
    sp.part(sp.rect(18, 19, 27, 27), R["cheese"], "cyl_v")
    sp.part(sp.rect(16, 28, 29, 30), R["plank"], "rim")


def st_oven(sp):
    """A brick bake oven: a squat dome with a dark mouth and a fire inside."""
    sp.part(sp.rect(1, 22, 30, 30), R["stone"], "rim")
    dome = sp.ellipse(2, 3, 29, 30) & sp.rect(0, 0, 31, 22)
    sp.part(dome, R["brick"], "sphere")
    for y in (8, 13, 18):
        sp.fill(sp.line([(3, y), (28, y)], 1) & dome, R["brick"][0])
    mouth = sp.ellipse(9, 11, 22, 26) & sp.rect(0, 0, 31, 21)
    sp.part(mouth, R["ink"], "flat")
    sp.part(sp.rect(20, 1, 25, 6), R["brick"], "rim")
    outline(sp)
    flames(sp, 11, 20, 21, 5)
    sp.fill(sp.rect(10, 20, 21, 21), R["ember"][2], glow=True)


def st_smoker(sp):
    sp.part(sp.rect(5, 8, 26, 30), R["plank"], "rim")
    for x in (10, 15, 20):
        sp.fill(sp.line([(x, 9), (x, 30)], 1), R["plank"][0])
    sp.part(sp.poly([(3, 9), (15, 2), (28, 9)]), R["iron"], "rim")
    sp.part(sp.rect(21, 1, 24, 6), R["iron"], "rim")
    sp.part(sp.rect(10, 21, 21, 29), R["ink"], "flat")
    for x in (12, 16, 20):
        sp.part(sp.poly([(x - 1, 12), (x + 1, 12), (x + 2, 18), (x - 2, 18)]), R["fish_smoked"], "rim")
    outline(sp)
    flames(sp, 12, 19, 29, 3)


def st_pasta(sp):
    """The pasta and noodle table: a board, a rolling pin, a drying rack hung with strands."""
    for x in (3, 28):
        sp.part(sp.line([(x, 2), (x, 17)], 2), R["wood"], "flat")
    sp.part(sp.line([(2, 3), (29, 3)], 1), R["wood"], "flat")
    for x in range(6, 27, 2):
        sp.part(sp.line([(x, 4), (x, 14 - (x % 4))], 1), R["egg_dough"], "flat", seam=False)
    sp.part(sp.rect(1, 18, 30, 21), R["plank"], "rim")
    for x in (4, 27):
        sp.part(sp.rect(x - 1, 22, x + 1, 30), R["wood"], "rim")
    sp.part(sp.ellipse(7, 14, 18, 19), R["egg_dough"], "sphere")
    sp.part(sp.line([(14, 16), (29, 12)], 3), R["plank"], "cyl_h")


def st_pizza(sp):
    """A wood-fired pizza oven: a taller dome on a stand, with a wide mouth."""
    sp.part(sp.rect(4, 22, 27, 30), R["stone"], "rim")
    sp.fill(sp.rect(6, 25, 25, 29), R["wood"][1])
    dome = sp.ellipse(3, 2, 28, 34) & sp.rect(0, 0, 31, 21)
    sp.part(dome, R["clay"], "sphere")
    sp.part(sp.rect(13, 0, 18, 5), R["clay"], "rim")
    mouth = sp.ellipse(8, 9, 23, 30) & sp.rect(0, 0, 31, 20)
    sp.part(mouth, R["ink"], "flat")
    sp.part(sp.line([(20, 19), (31, 14)], 2), R["wood"], "flat")
    outline(sp)
    flames(sp, 9, 15, 20, 7)
    sp.fill(sp.ellipse(15, 17, 22, 20), R["gold_crust"][2], glow=True)


def st_stove(sp):
    """An iron stove with a wok on its hob."""
    sp.part(sp.rect(2, 13, 29, 30), R["iron"], "rim")
    sp.part(sp.rect(5, 19, 26, 27), R["iron"], "rim", base=1)
    sp.part(sp.rect(23, 2, 27, 13), R["iron"], "cyl_v")
    body = sp.ellipse(5, 4, 22, 16) & sp.rect(0, 9, 31, 16)
    sp.part(body, R["iron"], "cyl_v", base=2)
    sp.part(sp.ellipse(5, 6, 22, 12), R["iron"], "rim", base=2)
    sp.part(sp.line([(0, 8), (5, 9)], 2), R["wood"], "flat")
    for x, y, rmp in ((10, 8, "green"), (14, 9, "shrimp"), (17, 8, "tomato")):
        sp.part(sp.ellipse(x - 1, y - 1, x + 1, y + 1), R[rmp], "rim", seam=False)
    outline(sp)
    sp.fill(sp.rect(7, 22, 24, 24), R["ember"][2], glow=True)
    flames(sp, 8, 19, 13, 2)


def st_fryer(sp):
    sp.part(sp.rect(3, 12, 28, 30), R["steel"], "rim")
    sp.part(sp.rect(3, 9, 28, 13), R["steel"], "rim", base=3)
    sp.part(sp.rect(5, 10, 26, 12), R["oil"], "flat", base=3, seam=False)
    sp.part(sp.rect(9, 3, 20, 10), R["steel"], "rim")
    sp.part(sp.line([(20, 4), (29, 1)], 2), R["wood"], "flat")
    for x in (11, 14, 17):
        sp.fill(sp.rect(x, 5, x, 9), R["steel"][0])
    sp.part(sp.rect(7, 18, 24, 26), R["steel"], "rim", base=2)
    sp.fill(sp.rect(10, 21, 12, 23), R["red"][2])


def st_press(sp):
    """An oil press: a screw over a basket, oil running to a jar."""
    for x in (4, 27):
        sp.part(sp.line([(x, 2), (x, 28)], 3), R["wood"], "cyl_v")
    sp.part(sp.rect(3, 2, 28, 5), R["wood"], "rim")
    sp.part(sp.line([(16, 5), (16, 14)], 3), R["iron"], "cyl_v")
    sp.part(sp.line([(8, 7), (24, 7)], 1), R["iron"], "flat")
    sp.part(sp.rect(9, 14, 23, 16), R["plank"], "rim")
    sp.part(sp.rect(10, 17, 22, 25), R["olive"], "cyl_v")
    sp.part(sp.rect(3, 26, 28, 29), R["plank"], "rim")
    sp.part(sp.rect(22, 27, 23, 30), R["oil"], "flat")


def st_icebox(sp):
    sp.part(sp.rect(4, 4, 27, 30), R["plank"], "rim")
    sp.part(sp.rect(6, 6, 25, 16), R["plank"], "rim", base=1)
    sp.part(sp.rect(6, 18, 25, 28), R["plank"], "rim", base=1)
    for y in (11, 23):
        sp.part(sp.rect(21, y - 1, 23, y + 1), R["steel"], "rim")
    sp.part(sp.rect(8, 1, 23, 4), R["ice"], "rim")
    glint(sp, 10, 2)


def st_galley(sp):
    """A ship's galley unit: a powered hob, a sealed oven and a freezer, one block."""
    sp.part(sp.rect(1, 3, 30, 30), R["panel"], "rim")
    sp.part(sp.rect(3, 5, 28, 11), R["steel"], "rim", base=1)
    for x in (8, 19):
        sp.part(sp.ellipse(x - 4, 6, x + 4, 10), R["iron"], "rim")
    sp.part(sp.rect(3, 13, 16, 28), R["steel"], "rim", base=2)
    sp.part(sp.rect(5, 15, 14, 22), R["ink"], "flat")
    sp.part(sp.rect(18, 13, 28, 28), R["ice"], "rim", base=1)
    sp.fill(sp.rect(25, 15, 26, 16), R["led"][2], glow=False)
    sp.fill(sp.rect(10, 24, 11, 25), R["red"][3])
    outline(sp)
    sp.fill(sp.rect(6, 19, 13, 21), R["ember"][2], glow=True)


STATIONS = {
    "st_campfire": st_campfire, "st_rack": st_rack, "st_quern": st_quern, "st_hearth": st_hearth, "st_barrel": st_barrel,
    "st_dairy": st_dairy, "st_oven": st_oven, "st_smoker": st_smoker, "st_pasta": st_pasta, "st_pizza": st_pizza,
    "st_stove": st_stove, "st_fryer": st_fryer, "st_press": st_press, "st_icebox": st_icebox, "st_galley": st_galley,
}


# ---------- new crops: stages 1..3 at 32 x 32 (stage 0 is M8's seeded mound) --------------------

def mound(sp, seed_ramp):
    m = sp.ellipse(4, 25, 27, 34) & sp.rect(0, 0, SW, BASE)
    sp.part(m, R["dirt"], "sphere")
    for x, y in ((9, 28), (14, 26), (19, 28), (23, 29), (12, 30)):
        sp.fill(sp.rect(x, y, x + 1, y), R[seed_ramp][-2])
        sp.plot(x, y, R[seed_ramp][-1])


def c_tomato(sp, s):
    h = (0, 10, 20, 25)[s]
    sp.part(sp.line([(MID - 6, BASE), (MID - 6, BASE - h - 2)], 1), R["wood"], "flat")   # the cane it climbs
    sp.part(sp.line([(MID, BASE), (MID - 2, BASE - h * 0.5), (MID + 1, BASE - h)], 2), R["stalk"], "flat")
    for k, t in enumerate((0.35, 0.6, 0.85, 1.0)[: (0, 2, 3, 4)[s]]):
        side = -1 if k % 2 else 1
        sp.part(frill(sp, MID, BASE - h * t, -math.pi / 2 + side * 1.2, 8 * (0.6 + 0.2 * s), 2.2), R["leaf"] if k % 2 else R["young"], "rim")
    if s == 2:
        for x, y in ((MID + 5, BASE - 12), (MID - 4, BASE - 17)):
            sp.part(sp.star(x, y, 2.4, 1, 5), R["flower_y"], "rim")
    if s == 3:
        for x, y in ((MID + 6, BASE - 9), (MID - 5, BASE - 14), (MID + 4, BASE - 19), (MID - 1, BASE - 6)):
            sp.part(sp.circle(x, y, 3), R["tomato"], "sphere")
            sp.plot(x - 1, y - 1, R["tomato"][-1])
            sp.plot(x, y - 3, R["leaf"][2])


def c_durum(sp, s):
    """Durum wheat: M8's wheat drawn taller, with long dark awns."""
    n = (0, 3, 5, 6)[s]
    h = (0, 11, 19, 23)[s]
    stalk = R["young"] if s == 1 else (R["stalk"] if s == 2 else R["straw"])
    for i in range(n):
        f = (i - (n - 1) / 2) / max(1, (n - 1) / 2)
        x0, tx = MID + f * 3, MID + f * (10 if s == 3 else 7)
        ty = BASE - h * (0.85 + 0.15 * math.cos(i * 1.7))
        sp.part(sp.line([(x0, BASE), (tx, ty)], 1), stalk, "flat")
        if s == 1:
            sp.part(leaf(sp, x0, BASE, -math.pi / 2 + f * 0.5, h, 2.2), R["young"], "rim")
        if s >= 2:
            sp.part(sp.ellipse(tx - 1.5, ty - 6, tx + 1.5, ty), R["durum"] if s == 3 else R["young"], "cyl_v")
        if s == 3:
            for a in (-0.4, 0.4):
                sp.fill(sp.line([(tx, ty - 5), (tx + a * 6, ty - 11)], 1), R["durum"][0])


def c_soy(sp, s):
    k = (0, 0.5, 0.8, 1.0)[s]
    for a in (-2.4, -1.9, -1.2, -0.7)[: (0, 2, 4, 4)[s]]:
        x, y = MID + math.cos(a) * 7 * k, BASE - 4 - math.sin(-a) * 9 * k
        sp.part(sp.line([(MID, BASE), (x, y)], 1), R["stalk"], "flat")
        for d in (-0.5, 0, 0.5):
            sp.part(leaf(sp, x, y, a + d, 6 * k + 1, 2.6 * k + 0.4), R["soy"] if d else R["leaf"], "rim")
    if s == 3:
        for x, y in ((MID - 5, BASE - 6), (MID + 4, BASE - 8), (MID, BASE - 12)):
            sp.part(sp.taper(x, y, x + 2, y + 7, 3, 2), R["bean"], "cyl_v")


def c_bok_choy(sp, s):
    k = (0, 0.5, 0.78, 1.0)[s]
    for i, a in enumerate((-2.2, -1.9, -1.57, -1.25, -0.95)):
        if s == 1 and i in (0, 4):
            continue
        L = 20 * k
        sp.part(sp.taper(MID, BASE, MID + math.cos(a) * L * 0.5, BASE + math.sin(a) * L * 0.5, 4 * k + 1, 3 * k), R["flower_w"], "cyl_v")
        sp.part(leaf(sp, MID + math.cos(a) * L * 0.4, BASE + math.sin(a) * L * 0.4, a, L * 0.6, 5 * k), R["leaf"] if i % 2 else R["young"], "rim")


def c_onion(sp, s, garlic=False):
    k = (0, 0.5, 0.8, 1.0)[s]
    for a in (-1.85, -1.6, -1.35, -1.1)[: (0, 2, 3, 4)[s]]:
        sp.part(sp.taper(MID, BASE - 4, MID + math.cos(a) * 22 * k, BASE - 4 + math.sin(a) * 22 * k, 2, 1), R["blue_leaf"] if garlic else R["leaf"], "rim")
    if s == 3:
        bulb = sp.ellipse(MID - 7, BASE - 11, MID + 7, BASE + 1) & sp.rect(0, 0, SW, BASE)
        sp.part(bulb, R["garlic"] if garlic else R["onion"], "sphere")
        if garlic:
            for x in (MID - 3, MID + 2):
                sp.fill(sp.line([(x, BASE - 9), (x, BASE - 1)], 1) & bulb, R["garlic"][0])
        sp.plot(MID - 4, BASE - 8, WHITE)


def c_garlic(sp, s):
    c_onion(sp, s, garlic=True)


def c_sugar_cane(sp, s):
    h = (0, 12, 22, 29)[s]
    for k, dx in enumerate((-5, 0, 5)[: (0, 2, 3, 3)[s]]):
        x = MID + dx
        sp.part(sp.rect(x - 1, BASE - h + k * 2, x + 1, BASE), R["cane"], "cyl_v")
        for y in range(BASE - 4, BASE - h + k * 2, -5):
            sp.fill(sp.rect(x - 1, y, x + 1, y), R["cane"][0])
        sp.part(leaf(sp, x, BASE - h + k * 2 + 2, -math.pi / 2 + (k - 1) * 0.9 - 0.3, 9, 2), R["leaf"], "rim")
        sp.part(leaf(sp, x, BASE - h * 0.6, -math.pi / 2 + (1 - k) * 1.1 + 0.4, 9, 2), R["young"], "rim")


def c_sugar_beet(sp, s):
    k = (0, 0.5, 0.72, 0.85)[s]
    by = BASE - 5 if s == 3 else BASE
    for i, a in enumerate((-2.3, -1.9, -1.57, -1.25, -0.85)):
        if s == 1 and i in (0, 2, 4):
            continue
        sp.part(leaf(sp, MID, by, a, 18 * k, 6 * k), R["leaf"] if i % 2 else R["young"], "rim")
    if s == 3:
        bulb = sp.ellipse(MID - 7, BASE - 9, MID + 7, BASE + 2) & sp.rect(0, 0, SW, BASE)
        sp.part(bulb, R["crumb"], "sphere")
        sp.plot(MID - 4, BASE - 7, WHITE)


def c_herbs(sp, s):
    """A pot-herb bush (basil and thyme): small leaves in clusters, white flowers when ripe."""
    k = (0, 0.5, 0.8, 1.0)[s]
    for i, (dx, h) in enumerate(((-6, 0.6), (0, 1.0), (6, 0.7), (-3, 0.8), (3, 0.85))[: (0, 3, 5, 5)[s]]):
        x, top = MID + dx * k, BASE - 18 * h * k - 2
        sp.part(sp.line([(MID, BASE), (x, top)], 1), R["stalk"], "flat")
        for j in range(3):
            y = top + j * 4
            for side in (-1, 1):
                sp.part(leaf(sp, x, y, -math.pi / 2 + side * 1.0, 4 * k + 1.5, 1.8), R["herb"] if (i + j) % 2 else R["young"], "rim")
        if s == 3:
            sp.part(sp.circle(x, top - 1, 1.3), R["flower_v"] if i % 2 else R["flower_w"], "rim")


def c_potato(sp, s):
    k = (0, 0.5, 0.8, 1.0)[s]
    for i, a in enumerate((-2.4, -2.0, -1.57, -1.1, -0.7)[: (0, 3, 5, 5)[s]]):
        x, y = MID + math.cos(a) * 9 * k, BASE - 3 + math.sin(a) * 11 * k
        sp.part(sp.line([(MID, BASE), (x, y)], 1), R["stalk"], "flat")
        sp.part(frill(sp, x, y, a, 5 * k + 1, 2.2), R["leaf"] if i % 2 else R["young"], "sphere")
    if s >= 2:
        for x, y in ((MID - 6, BASE - 16), (MID + 5, BASE - 15)):
            sp.part(sp.star(x, y, 2.4, 1.1, 5), R["flower_w"] if s == 2 else R["flower_v"], "rim")
    if s == 3:
        for x, y in ((MID - 8, BASE - 1), (MID + 7, BASE - 1)):
            sp.part(sp.ellipse(x - 3, y - 3, x + 3, y + 1) & sp.rect(0, 0, SW, BASE), R["potato"], "sphere")


def c_olive(sp, s):
    """An olive tree: perennial, so it grows in three stages and then fruits each year."""
    h = (0, 12, 20, 22)[s]
    sp.part(sp.poly([(MID - 2, BASE), (MID + 2, BASE), (MID + 1, BASE - h * 0.6), (MID - 1, BASE - h * 0.6)]), R["bark"], "cyl_v")
    r = (0, 5, 9, 11)[s]
    for dx, dy in ((-6, 0), (6, -1), (0, -5), (-3, 3), (4, 3))[: (0, 2, 4, 5)[s]]:
        sp.part(frill(sp, MID + dx - r * 0.5, BASE - h + dy + 4, 0, r, 3.2), R["silver_leaf"], "sphere")
    if s == 3:
        for x, y in ((MID - 6, BASE - 15), (MID + 5, BASE - 13), (MID - 1, BASE - 20), (MID + 8, BASE - 18), (MID - 8, BASE - 11)):
            sp.part(sp.ellipse(x - 1.5, y - 2, x + 1.5, y + 2), R["olive"], "sphere")


def c_tea(sp, s):
    """A tea bush: a low rounded shrub, its tips the pale leaves you pick."""
    k = (0, 0.5, 0.8, 1.0)[s]
    sp.part(sp.line([(MID, BASE), (MID, BASE - 6 * k)], 2), R["bark"], "flat")
    for dx, dy in ((-7, 0), (7, 0), (-3, -5), (3, -5), (0, -9))[: (0, 3, 5, 5)[s]]:
        sp.part(frill(sp, MID + dx * k - 4 * k, BASE - 8 * k + dy * k, 0, 8 * k, 3.4 * k + 0.5), R["leaf"], "sphere")
    if s == 3:
        for x, y in ((MID - 8, BASE - 14), (MID + 5, BASE - 16), (MID - 1, BASE - 19), (MID + 9, BASE - 11)):
            sp.part(leaf(sp, x, y, -math.pi / 2 - 0.3, 4, 1.6), R["young"], "rim")


NEW_CROPS = {
    "tomato": (c_tomato, "tomato"), "durum": (c_durum, "durum"), "soy": (c_soy, "bean"), "bok_choy": (c_bok_choy, "young"),
    "onion": (c_onion, "onion"), "garlic": (c_garlic, "garlic"), "sugar_cane": (c_sugar_cane, "cane"),
    "sugar_beet": (c_sugar_beet, "crumb"), "herbs": (c_herbs, "herb"), "potato": (c_potato, "potato"),
    "olive": (c_olive, "olive"), "tea": (c_tea, "tea"),
}


def p_tomato(sp):
    sp.part(sp.circle(12, 13, 9), R["tomato"], "sphere")
    sp.part(sp.star(12, 5, 4, 1.5, 5), R["leaf"], "rim")
    sp.plot(8, 9, WHITE)


def p_durum(sp):
    for i in range(5):
        f = (i - 2) / 2
        sp.part(sp.line([(12 + f * 1.5, 22), (12 + f * 8, 9)], 1), R["straw"], "flat")
    for i in (0, 4, 1, 3, 2):
        f = (i - 2) / 2
        x = 12 + f * 8
        sp.part(sp.ellipse(x - 1.5, 3 + abs(f) * 3, x + 1.5, 9 + abs(f) * 3), R["durum"], "cyl_v")
        sp.fill(sp.line([(x, 3 + abs(f) * 3), (x + f * 2, abs(f) * 3)], 1), R["durum"][0])
    sp.part(sp.rect(8, 14, 16, 16), R["glaze"], "rim")


def p_soy(sp):
    for k, (x, y, a) in enumerate(((3, 18, -0.9), (8, 20, -1.3), (13, 19, -0.6))):
        pod = sp.taper(x, y, x + math.cos(a) * 14, y + math.sin(a) * 14, 5, 3)
        sp.part(pod, R["bean"], "cyl_h")
        for j in range(3):
            t = 0.2 + j * 0.3
            sp.plot(round(x + math.cos(a) * 14 * t), round(y + math.sin(a) * 14 * t), R["bean"][0])


def p_bok_choy(sp):
    for i, a in enumerate((-2.0, -1.57, -1.15)):
        sp.part(sp.taper(12, 22, 12 + math.cos(a) * 9, 22 + math.sin(a) * 9, 5, 3), R["flower_w"], "cyl_v")
        sp.part(leaf(sp, 12 + math.cos(a) * 7, 22 + math.sin(a) * 7, a, 12, 5), R["leaf"] if i % 2 else R["young"], "rim")


def p_onion(sp, garlic=False):
    for a in (-1.8, -1.35):
        sp.part(sp.taper(12, 8, 12 + math.cos(a) * 8, 8 + math.sin(a) * 8, 2, 1), R["straw"], "flat")
    bulb = sp.ellipse(3, 6, 20, 22)
    sp.part(bulb, R["garlic"] if garlic else R["onion"], "sphere")
    if garlic:
        for x in (8, 15):
            sp.fill(sp.line([(x, 8), (x, 20)], 1) & bulb, R["garlic"][0])
    else:
        sp.fill(sp.line([(8, 9), (6, 19)], 1) & bulb, R["onion"][0])
    sp.part(sp.line([(9, 22), (14, 23)], 1), R["sack"], "flat")
    sp.plot(7, 9, WHITE)


def p_garlic(sp):
    p_onion(sp, garlic=True)


def p_sugar_cane(sp):
    for k, x in enumerate((6, 12, 17)):
        sp.part(sp.rect(x - 1, 2 + k, x + 2, 22), R["cane"], "cyl_v")
        for y in (8, 14, 20):
            sp.fill(sp.rect(x - 1, y, x + 2, y), R["cane"][0])
    sp.part(sp.rect(3, 12, 20, 14), R["red"], "rim")


def p_sugar_beet(sp):
    for a in (-2.0, -1.57, -1.1):
        sp.part(leaf(sp, 12, 9, a, 8, 3), R["leaf"], "rim")
    sp.part(sp.ellipse(4, 7, 19, 20), R["crumb"], "sphere")
    sp.part(sp.line([(11, 20), (12, 23)], 1), R["crumb"], "flat")
    sp.plot(7, 9, WHITE)


def p_herbs(sp):
    for k, (x, a) in enumerate(((8, -2.0), (12, -1.6), (16, -1.2))):
        sp.part(sp.line([(12, 22), (x, 8)], 1), R["stalk"], "flat")
        for j in range(3):
            for side in (-1, 1):
                sp.part(leaf(sp, x + (12 - x) * j / 5, 8 + j * 4, -math.pi / 2 + side * 1.0, 5, 2), R["herb"] if (k + j) % 2 else R["young"], "rim")
    sp.part(sp.rect(9, 17, 15, 19), R["red"], "rim")


def p_potato(sp):
    for x, y in ((2, 9), (11, 6), (7, 14)):
        sp.part(sp.ellipse(x, y, x + 11, y + 8), R["potato"], "sphere")
        sp.plot(x + 4, y + 3, R["potato"][0])
        sp.plot(x + 2, y + 2, R["potato"][-1])


def p_olive(sp):
    sp.part(sp.line([(3, 3), (20, 8)], 1), R["bark"], "flat")
    for x, y, a in ((6, 4, -0.2), (14, 6, 0.4)):
        sp.part(leaf(sp, x, y, a, 9, 2.5), R["silver_leaf"], "rim")
    for x, y in ((6, 13), (12, 16), (17, 12)):
        sp.part(sp.ellipse(x - 3, y - 4, x + 3, y + 4), R["olive"], "sphere")
        sp.plot(x - 1, y - 2, R["olive"][-1])


def p_tea(sp):
    for x, y, a in ((4, 20, -1.0), (10, 21, -1.4), (15, 20, -1.9), (9, 14, -0.7), (14, 13, -2.2)):
        sp.part(leaf(sp, x, y, a, 10, 3), R["young"] if (x + y) % 2 else R["leaf"], "rim")


PRODUCE = {
    "tomato": p_tomato, "durum": p_durum, "soy": p_soy, "bok_choy": p_bok_choy, "onion": p_onion, "garlic": p_garlic,
    "sugar_cane": p_sugar_cane, "sugar_beet": p_sugar_beet, "herbs": p_herbs, "potato": p_potato, "olive": p_olive, "tea": p_tea,
}


# ---------- textures and cards for the 3D scenes -------------------------------------------------

def tex_embers(sp):
    """A bed of coals, drawn as light: the 3D scene shows it unlit."""
    sp.dither(sp.rect(0, 0, 31, 31), R["ember"][0], R["ember"][1], 0.45, glow=True)
    for x, y in ((4, 5), (12, 9), (21, 4), (27, 13), (7, 18), (17, 21), (25, 26), (3, 28), (14, 14)):
        sp.fill(sp.ellipse(x - 2, y - 1, x + 2, y + 1), R["ember"][2], glow=True)
        sp.plot(x, y, R["ember"][3], glow=True)


def tex_crust(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["gold_crust"][1])
    sp.dither(sp.rect(0, 0, 31, 31), R["gold_crust"][1], R["gold_crust"][2], 0.35)
    for x in range(2, 32, 8):
        sp.fill(sp.line([(x, 4), (x + 4, 12)], 1), R["crumb"][1])
        sp.fill(sp.line([(x + 3, 18), (x + 7, 26)], 1), R["crumb"][1])


def tex_dough(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["dough"][2])
    sp.dither(sp.rect(0, 0, 31, 31), R["dough"][2], R["dough"][1], 0.2)
    for x, y in ((5, 7), (19, 4), (11, 20), (26, 23), (3, 27)):
        sp.plot(x, y, R["flour"][2])


def tex_steel(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["steel"][2])
    for y in range(0, 32, 4):
        sp.fill(sp.rect(0, y, 31, y), R["steel"][1])
    for x, y in ((3, 2), (28, 2), (3, 29), (28, 29)):
        sp.plot(x, y, R["steel"][4])


def tex_panel(sp):
    """A ship galley's wall panel."""
    sp.fill(sp.rect(0, 0, 31, 31), R["panel"][2])
    sp.fill(sp.rect(0, 0, 31, 0) | sp.rect(0, 0, 0, 31), R["panel"][3])
    sp.fill(sp.rect(0, 31, 31, 31) | sp.rect(31, 0, 31, 31), R["panel"][0])
    sp.fill(sp.rect(0, 15, 31, 15), R["panel"][1])
    for x, y in ((3, 3), (28, 3), (3, 12), (28, 12), (3, 19), (28, 19), (3, 28), (28, 28)):
        sp.plot(x, y, R["panel"][3])


def card_flame(sp):
    sp.part(sp.taper(16, 31, 15, 8, 18, 2), R["fire"], "inset", glow=True)
    sp.part(sp.taper(8, 31, 6, 16, 10, 1), R["fire"], "inset", glow=True)
    sp.part(sp.taper(24, 31, 26, 14, 10, 1), R["fire"], "inset", glow=True)


def card_smoke(sp):
    for x, y, r in ((10, 22, 6), (18, 18, 7), (13, 10, 6), (21, 7, 4)):
        sp.dither(sp.circle(x, y, r), R["smoke"][1], R["smoke"][2], 0.4)


def card_herbs(sp):
    """Bunches of herbs and chilis hung to dry, a cut-out for the drying rack."""
    sp.part(sp.line([(0, 1), (31, 1)], 1), R["sack"], "flat")
    for k, x in enumerate((4, 12, 20, 28)):
        sp.part(sp.line([(x, 1), (x, 6)], 1), R["sack"], "flat")
        if k % 2:
            for j in range(3):
                sp.part(sp.poly([(x - 1 + j - 1, 6), (x + j, 6), (x + j, 16 + j * 2), (x - 1 + j - 1, 14 + j * 2)]), R["red"], "rim")
        else:
            for j in range(4):
                sp.part(leaf(sp, x, 6 + j * 3, math.pi / 2 + (j % 2 - 0.5) * 0.8, 7, 2), R["herb"], "rim")


def card_fish(sp):
    sp.part(sp.line([(0, 1), (31, 1)], 1), R["sack"], "flat")
    for x in (5, 16, 27):
        sp.part(sp.line([(x, 1), (x, 4)], 1), R["sack"], "flat")
        sp.part(sp.poly([(x - 1, 4), (x + 1, 4), (x + 3, 9), (x + 3, 20), (x, 25), (x - 3, 20), (x - 3, 9)]), R["fish_smoked"], "rim")
        sp.part(sp.poly([(x, 25), (x + 3, 30), (x - 3, 30)]), R["fish_smoked"], "rim")


def card_noodles(sp):
    sp.part(sp.line([(0, 1), (31, 1)], 2), R["wood"], "flat")
    for x in range(1, 31, 2):
        sp.part(sp.line([(x, 3), (x, 22 + (x * 7) % 9)], 1), R["egg_dough"] if x % 4 == 1 else R["noodle"], "flat", seam=False)


TEXTURES = {
    "tex_embers": (32, 32, tex_embers), "tex_crust": (32, 32, tex_crust), "tex_dough": (32, 32, tex_dough),
    "tex_steel": (32, 32, tex_steel), "tex_panel": (32, 32, tex_panel), "card_flame": (32, 32, card_flame),
    "card_smoke": (32, 32, card_smoke), "card_herbs": (32, 32, card_herbs), "card_fish": (32, 32, card_fish),
    "card_noodles": (32, 32, card_noodles),
}
NO_OUTLINE = {"tex_embers", "tex_crust", "tex_dough", "tex_steel", "tex_panel", "card_flame", "card_smoke"}


# ---------- out ----------------------------------------------------------------------------------

def draw(w, h, fn, *args, line=True):
    sp = Sprite(w, h)
    fn(sp, *args)
    if line and not getattr(sp, "outlined", False):
        outline(sp)
    return sp.image()


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    items = []
    src = "tools/art/cooking_art.py (balatro-pixel-art kit)"

    def put(name, img):
        save(img, os.path.join(OUT, name + ".png"), src)
        items.append((name, img))

    for name, fn in ICONS.items():
        put(name, draw(IW, IW, fn))
    for cut, fn in CUTS.items():
        for st in STATES:
            put(f"meat_{cut}_{st}", draw(IW, IW, fn, st))
    for st, ramp in (("raw", "fish_raw"), ("cooked", "fish_cooked"), ("smoked", "fish_smoked"), ("salted", "fish_salted")):
        put(f"fillet_{st}", draw(IW, IW, fillet, ramp))
    for name, fn in STATIONS.items():
        put(name, draw(SW, SW, fn))
    for name, (fn, seed) in NEW_CROPS.items():
        for s in range(4):
            put(f"crop_{name}_{s}", draw(SW, SW, mound, seed) if s == 0 else draw(SW, SW, fn, s))
        put(f"produce_{name}", draw(IW, IW, PRODUCE[name]))
    for name, (w, h, fn) in TEXTURES.items():
        put(name, draw(w, h, fn, line=name not in NO_OUTLINE))
    print("wrote", len(items), "pngs to", OUT)
    if "--sheet" in argv:
        out = argv[argv.index("--sheet") + 1]
        sheet(items, scale=3, cols=14, cell=(SW * 3, SW * 3)).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main(sys.argv[1:])
