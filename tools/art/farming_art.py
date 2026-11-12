"""Crop growth stages, produce and farm tools for the farming mockup (docs/mockups/farming.html).

Pixel art drawn in code with the balatro-pixel-art kit
(.claude/skills/balatro-pixel-art/scripts/pixelkit.py), the style of the owner's solitaire
collection: a tiny native canvas, one bold subject, 3 to 5 tone ramps lit from the top left, dark
seams, a 1 px outline, single-pixel glints, no anti-aliasing, Sweaft-64 colours only (`save()`
refuses anything else). Alpha is 0 or 255, so the 3D plot draws the stages with an alpha test and no
blending.

Ground is NOT drawn here: the mockup uses the game's own terrain atlas (assets/tilesets/atlas.png,
whose fields sheet has a "tilled soil" tile) and settlement textures, copied to
docs/mockups/farming/game/. Only what the game lacks is drawn.

Each crop has four stages, `<crop>_0..3` (seeded, sprout, growing, ripe), 32 x 32, the plant standing
on the bottom centre so the 3D scene stands the sprite on its cell. Produce and tools are 24 x 24.

    python tools/art/farming_art.py              # writes docs/mockups/farming/*.png
    python tools/art/farming_art.py --sheet S    # also a contact sheet (keep it out of the repo)
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "farming")
CW = 32   # crop stage canvas
IW = 24   # produce and tool canvas
BASE = 31  # the ground row of a crop stage
MID = 16
WHITE = "#fffefe"

R = dict(RAMPS)
R.update({
    "leaf": ["#003227", "#297537", "#559642", "#9bb675"],
    "young": ["#05582d", "#559642", "#9bb675", "#c1f1d8"],
    "stalk": ["#05582d", "#297537", "#9bb675"],
    "grain": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "ricegrn": ["#0c2f44", "#204f63", "#53998b", "#84e3b5"],
    "ricehead": ["#8d4526", "#ab8465", "#cfaa81", "#eab798"],
    "straw": ["#8d4526", "#c56d25", "#cfaa81"],
    "dirt": ["#2f1525", "#41322c", "#725244", "#ab8465"],
    "turnip": ["#230c44", "#4f1d83", "#9e3ba2", "#c34988"],
    "root": ["#92a3b2", "#bfcad7", "#fffefe"],
    "melon": ["#003227", "#297537", "#559642", "#9bb675"],
    "flower": ["#c56d25", "#fb902f", "#faaf5c"],
    "chili": ["#4a051e", "#a61e1f", "#d4462b", "#fa3a64"],
    "kale": ["#0c2f44", "#204f63", "#53998b", "#84e3b5"],
    "kale2": ["#191954", "#15578a", "#53998b", "#84e3b5"],
    "stem": ["#a09f91", "#cfaa81", "#eab798"],
    "capgrey": ["#2b1f35", "#3e3551", "#69608b", "#92a3b2"],
    "capglow": ["#0c2f44", "#15578a", "#1096c3", "#74effe", "#d2f8ff"],
    "sack": ["#725244", "#ab8465", "#cfaa81", "#eab798"],
    "fert": ["#2f1525", "#41322c", "#725244", "#ab8465"],
    "can": ["#0c2f44", "#15578a", "#1096c3", "#74effe"],
    "skin": ["#7d4636", "#a8633e", "#e69d69", "#eab798"],
    "lamp": ["#c56d25", "#fb902f", "#faaf5c", "#fffefe"],
})


def leaf(sp, x, y, ang, length, width):
    """A pointed leaf mask from (x, y) out along `ang` (radians, 0 = right, -pi/2 = up)."""
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
    """A frilly kale leaf: a row of overlapping balls along a line."""
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


# ---------- stage 0: a seeded mound ----------------------------------------------------------

def mound(sp, seed_ramp):
    m = sp.ellipse(4, 25, 27, 34) & sp.rect(0, 0, CW, BASE)
    sp.part(m, R["dirt"], "sphere")
    for x, y in ((9, 28), (14, 26), (19, 28), (23, 29), (12, 30)):
        sp.fill(sp.rect(x, y, x + 1, y), R[seed_ramp][-2])
        sp.plot(x, y, R[seed_ramp][-1])


# ---------- crops: fn(sp, stage) for stage 1..3 ----------------------------------------------

def wheat(sp, s, droop=False):
    green = R["ricegrn"] if droop else R["stalk"]
    ripe = R["ricehead"] if droop else R["grain"]
    n = (0, 3, 5, 6)[s]
    h = (0, 10, 18, 21)[s] * (0.95 if droop else 1)
    stalk = R["young"] if s == 1 else (green if s == 2 else (R["straw"] if not droop else R["stalk"]))
    heads = []
    for i in range(n):
        f = (i - (n - 1) / 2) / max(1, (n - 1) / 2)
        x0 = MID + f * 3
        tx = MID + f * (11 if s == 3 else 8)
        ty = BASE - h * (0.85 + 0.15 * math.cos(i * 1.7))
        sp.part(sp.line([(x0, BASE), (tx, ty)], 1), stalk, "flat")
        if s >= 2 and i % 2 == 0:
            sp.part(leaf(sp, (x0 * 2 + tx) / 3, (BASE * 2 + ty) / 3, -math.pi / 2 + f * 1.2 + (0.6 if f >= 0 else -0.6), h * 0.35, 2), green, "rim")
        if s == 1:
            sp.part(leaf(sp, x0, BASE, -math.pi / 2 + f * 0.5, h, 2.2), R["young"], "rim")
        heads.append((tx, ty, f))
    for tx, ty, f in heads:
        side = 1 if f >= 0 else -1
        if s == 3:
            if droop:
                # a ripe rice head bows over: a tapered plume hanging off the stalk's tip
                sp.part(sp.taper(tx, ty - 1, tx + side * 4, ty + 6, 3, 1), ripe, "rim")
                sp.plot(round(tx), round(ty - 1), ripe[-1])
            else:
                sp.part(sp.ellipse(tx - 1.5, ty - 7, tx + 1.5, ty), ripe, "cyl_v")
                sp.plot(round(tx), round(ty - 8), ripe[-1])
        elif s == 2:
            sp.part(sp.ellipse(tx - 1, ty - 4, tx + 1, ty), R["young"], "flat")


def rice(sp, s):
    wheat(sp, s, droop=True)
    if s >= 1:
        # standing water at the foot of a paddy plant
        sp.part(sp.rect(4, BASE - 1, 27, BASE), R["glass"], "flat", seam=False)


def turnip(sp, s):
    k = (0, 0.5, 0.72, 0.85)[s]
    by = BASE - 5 if s == 3 else BASE
    angs = (-2.3, -1.9, -1.57, -1.25, -0.85)
    for i, a in enumerate(angs):
        if s == 1 and i in (0, 2, 4):
            continue
        sp.part(leaf(sp, MID, by, a, 20 * k, 5 * k), R["leaf"] if i % 2 else R["young"], "rim")
    if s == 3:
        bulb = sp.ellipse(MID - 7, BASE - 9, MID + 7, BASE + 2) & sp.rect(0, 0, CW, BASE)
        sp.part(bulb, R["root"], "sphere")
        sp.part(bulb & sp.rect(0, 0, CW, BASE - 6), R["turnip"], "sphere")
        sp.plot(MID - 4, BASE - 7, WHITE)


def melon(sp, s):
    sp.part(sp.line([(2, BASE - 1), (9, BASE - 3), (17, BASE - 1), (29, BASE - 3)], 2), R["stalk"], "flat")
    leaves = ((6, -2.0), (13, -1.2), (21, -1.9), (27, -1.0))[: (0, 2, 3, 4)[s]]
    for x, a in leaves:
        sp.part(leaf(sp, x, BASE - 2, a, 10, 5), R["leaf"], "rim")
    if s == 2:
        for x in (11, 22):
            sp.part(sp.star(x, BASE - 9, 3.5, 1.5, 5), R["flower"], "rim")
            sp.plot(x, BASE - 9, R["grain"][0])
    if s == 3:
        body = sp.ellipse(MID - 11, BASE - 15, MID + 11, BASE)
        sp.part(body, R["melon"], "sphere")
        for dx in (-7, -3, 1, 5, 9):
            stripe = sp.line([(MID + dx * 0.4, BASE - 15), (MID + dx, BASE - 8), (MID + dx * 0.4, BASE)], 1) & body
            sp.fill(stripe, R["melon"][0])
        sp.part(sp.line([(MID, BASE - 15), (MID + 2, BASE - 18)], 2), R["wood"], "flat")
        sp.plots([(MID - 7, BASE - 12), (MID - 6, BASE - 13)], R["melon"][-1])
        sp.plot(MID - 7, BASE - 13, WHITE)


def chili(sp, s):
    h = (0, 11, 19, 23)[s]
    k = (0, 0.55, 0.8, 1.0)[s]
    sp.part(sp.line([(MID, BASE), (MID - 1, BASE - h * 0.5), (MID + 1, BASE - h)], 2), R["stalk"], "flat")
    spots = [(-1, 0.4), (1, 0.5), (-1, 0.7), (1, 0.8), (-1, 0.95), (1, 1.0)][: (0, 2, 4, 6)[s]]
    for side, t in spots:
        y = BASE - h * t
        sp.part(leaf(sp, MID, y, -math.pi / 2 + side * 1.25, 9 * k, 3 * k), R["leaf"] if side < 0 else R["young"], "rim")
    if s == 2:
        for side, t in ((-1, 0.55), (1, 0.85)):
            x, y = MID + side * 6, BASE - h * t
            sp.part(sp.star(x, y, 2.6, 1.2, 5), R["paper"], "rim")
            sp.plot(round(x), round(y), R["gold"][2])
    if s == 3:
        for side, t, d in ((-1, 0.45, 5), (1, 0.6, 6), (-1, 0.82, 9), (1, 0.92, 9), (1, 0.3, 4)):
            x, y = MID + side * d, BASE - h * t
            pod = sp.poly([(x - 1.5, y), (x + 1.5, y), (x + 1, y + 5), (x + side * 1, y + 7), (x - 1, y + 5)])
            sp.part(pod, R["chili"], "cyl_v")
            sp.part(sp.rect(x - 1, y - 1, x + 1, y - 1), R["stalk"], "flat")
            sp.plot(round(x - 1), round(y + 1), R["chili"][-1])


def kale(sp, s):
    k = (0, 0.55, 0.78, 1.0)[s]
    sp.part(sp.rect(MID - 1, BASE - 8 * k, MID + 1, BASE), R["stem"], "cyl_v")
    angs = (-2.6, -0.55, -2.1, -1.05, -1.57)
    for i, a in enumerate(angs):
        if s == 1 and i in (0, 1):
            continue
        sp.part(frill(sp, MID, BASE - 6 * k, a, 12 * k, 3.4 * k), R["kale2"] if i % 2 else R["kale"], "sphere")
        x, y = MID + math.cos(a) * 9 * k, BASE - 6 * k + math.sin(a) * 9 * k
        sp.plot(round(x - 1), round(y - 1), R["kale"][-1])
    if s == 3:
        outline(sp)
        spark(sp, 4, 5, R["cyan"])
        spark(sp, 27, 9, R["cyan"])


def glowcap(sp, s):
    caps = [(-8, 0.6, 0.75), (0, 1.0, 1.0), (8, 0.75, 0.8), (-4, 0.4, 0.55), (4, 0.35, 0.5)][: (0, 3, 4, 5)[s]]
    k = (0, 0.4, 0.7, 1.0)[s]
    ramp = R["capglow"] if s == 3 else R["capgrey"]
    order = sorted(caps, key=lambda c: -c[1])
    for dx, h, r in order:
        x = MID + dx
        top = BASE - 16 * h * k - 3
        sw = 1 if r * k > 0.7 else 0
        sp.part(sp.rect(x - sw, top, x + sw, BASE), R["stem"], "cyl_v")
        cw, ch = 6.5 * r * k + 1.5, 4.5 * r * k + 1
        cap = sp.ellipse(x - cw, top - ch, x + cw, top + ch) & sp.rect(0, 0, CW, top + 1)
        sp.part(cap, ramp, "sphere")
        if s == 3:
            sp.plot(round(x - cw * 0.4), round(top - ch * 0.5), ramp[-1])
            sp.plot(round(x + cw * 0.3), round(top - ch * 0.2), ramp[-1])
    if s == 3:
        outline(sp)
        spark(sp, 3, 4, R["capglow"])
        spark(sp, 28, 7, R["capglow"])


CROPS = {
    "wheat": (wheat, "grain"),
    "turnip": (turnip, "turnip"),
    "sun_melon": (melon, "bone"),
    "chili": (chili, "bone"),
    "rice": (rice, "paper"),
    "frost_kale": (kale, "soot"),
    "glowcap": (glowcap, "cyan"),
}


# ---------- produce (24 x 24) --------------------------------------------------------------------

def p_wheat(sp):
    for i in range(5):
        f = (i - 2) / 2
        sp.part(sp.line([(12 + f * 1.5, 22), (12 + f * 8, 9)], 1), R["straw"], "flat")
    for i in (0, 4, 1, 3, 2):
        f = (i - 2) / 2
        x = 12 + f * 8
        sp.part(sp.ellipse(x - 1.5, 2 + abs(f) * 3, x + 1.5, 9 + abs(f) * 3), R["grain"], "cyl_v")
    sp.part(sp.rect(8, 14, 16, 16), R["red"], "rim")


def p_turnip(sp):
    for a in (-2.1, -1.57, -1.05):
        sp.part(leaf(sp, 12, 9, a, 8, 3), R["leaf"], "rim")
    bulb = sp.ellipse(4, 7, 20, 20)
    sp.part(bulb, R["root"], "sphere")
    sp.part(bulb & sp.rect(0, 0, IW, 12), R["turnip"], "sphere")
    sp.part(sp.line([(12, 20), (12, 23)], 1), R["root"], "flat")
    sp.plot(7, 9, WHITE)


def p_melon(sp):
    body = sp.ellipse(1, 5, 22, 21)
    sp.part(body, R["melon"], "sphere")
    for dx in (-8, -4, 0, 4, 8):
        sp.fill(sp.line([(12 + dx * 0.4, 5), (12 + dx, 13), (12 + dx * 0.4, 21)], 1) & body, R["melon"][0])
    sp.part(sp.line([(12, 5), (14, 1)], 2), R["wood"], "flat")
    sp.plots([(5, 9), (6, 8)], R["melon"][-1])
    sp.plot(5, 8, WHITE)


def p_chili(sp):
    pod = sp.poly([(6, 5), (10, 4), (15, 10), (19, 18), (20, 22), (17, 21), (12, 15), (7, 10)])
    sp.part(pod, R["chili"], "rim", depth=2)
    sp.part(sp.poly([(3, 3), (9, 2), (10, 6), (5, 7)]), R["leaf"], "rim")
    sp.part(sp.line([(5, 3), (3, 0)], 2), R["stalk"], "flat")
    sp.plots([(9, 7), (10, 8)], R["chili"][-1])
    sp.plot(10, 7, WHITE)


def p_rice(sp):
    sp.part(sp.ellipse(4, 6, 19, 14), R["ricehead"], "sphere")
    sp.part(sp.poly([(1, 12), (22, 12), (19, 20), (4, 20)]), R["wood"], "cyl_h")
    sp.part(sp.rect(1, 11, 22, 12), R["rust"], "rim")
    for x, y in ((7, 8), (11, 7), (15, 9), (9, 10)):
        sp.plot(x, y, R["ricehead"][1])
    sp.part(sp.line([(15, 1), (11, 9)], 1), R["wood"], "flat")
    sp.part(sp.line([(18, 2), (13, 9)], 1), R["wood"], "flat")
    sp.plot(6, 7, WHITE)


def p_kale(sp):
    m = frill(sp, 7, 18, -0.9, 14, 4.2)
    sp.part(m, R["kale"], "sphere")
    sp.fill(sp.line([(3, 22), (7, 18), (15, 7)], 1) & m, R["kale"][-1])
    sp.part(sp.line([(2, 23), (5, 20)], 2), R["stem"], "flat")


def p_glowcap(sp):
    sp.part(sp.rect(9, 11, 14, 22), R["stem"], "cyl_v")
    cap = sp.ellipse(1, 2, 22, 16) & sp.rect(0, 0, IW, 12)
    sp.part(cap, R["capglow"], "sphere")
    sp.fill(sp.rect(3, 12, 20, 12), R["capglow"][0])
    outline(sp)
    for x, y in ((6, 6), (12, 4), (16, 8)):
        sp.fill(sp.rect(x, y, x + 1, y + 1), R["capglow"][-2], glow=True)
        sp.plot(x, y, WHITE, glow=True)


# ---------- tools (24 x 24) ----------------------------------------------------------------------

def t_hoe(sp):
    sp.part(sp.line([(3, 22), (17, 5)], 2), R["wood"], "flat")
    sp.part(sp.poly([(14, 3), (21, 5), (21, 11), (18, 10), (17, 7), (13, 6)]), R["metal"], "rim")
    sp.plot(15, 4, WHITE)


def t_can(sp):
    sp.part(sp.line([(15, 13), (22, 5)], 2), R["can"], "flat")
    sp.part(sp.rect(20, 3, 23, 5), R["can"], "rim")
    sp.part(sp.line([(4, 9), (5, 4), (12, 3), (16, 6), (16, 9)], 2), R["can"], "flat")
    body = sp.rect(2, 9, 17, 22)
    sp.part(body, R["can"], "cyl_v")
    sp.fill(sp.rect(2, 12, 17, 12), R["can"][0])
    sp.plot(4, 10, WHITE)


def t_seeds(sp):
    sp.part(sp.poly([(7, 2), (16, 2), (17, 6), (6, 6)]), R["sack"], "rim")
    sack = sp.poly([(6, 6), (17, 6), (21, 22), (2, 22)])
    sp.part(sack, R["sack"], "rim", depth=2)
    sp.part(sp.rect(6, 6, 17, 7), R["red"], "flat")
    for x, y in ((9, 13), (13, 12), (11, 17), (15, 17), (7, 18)):
        sp.fill(sp.rect(x, y, x + 1, y), R["wood"][1])
        sp.plot(x, y, R["wood"][2])


def t_fert(sp):
    sack = sp.poly([(4, 5), (19, 5), (21, 22), (2, 22)])
    sp.part(sack, R["fert"], "rim", depth=2)
    sp.part(sp.ellipse(4, 3, 19, 7), R["dirt"], "rim")
    sp.part(leaf(sp, 11, 17, -1.0, 8, 3), R["young"], "rim")
    sp.part(leaf(sp, 11, 17, -2.1, 8, 3), R["leaf"], "rim")
    outline(sp)
    spark(sp, 21, 2, R["gold"])


def t_sickle(sp):
    outer = sp.ellipse(4, 2, 22, 20)
    inner = sp.ellipse(4, 6, 19, 22)
    blade = outer & ~inner & sp.rect(0, 0, IW, 13)
    sp.part(blade, R["metal"], "rim")
    sp.part(sp.line([(6, 12), (2, 22)], 3), R["wood"], "flat")
    sp.plot(12, 3, WHITE)


def t_hand(sp):
    """The look tool: an open hand, as the game's hotbar shows one."""
    skin = R["skin"]
    for x0, top in ((6, 4), (9, 2), (12, 2), (15, 4)):
        sp.part(sp.rect(x0, top, x0 + 2, 13), skin, "cyl_v")
    sp.part(sp.taper(3, 10, 8, 16, 3, 3), skin, "rim")
    sp.part(sp.rect(6, 12, 17, 21), skin, "rim", depth=2)
    sp.plot(10, 3, WHITE)


def t_lantern(sp):
    sp.part(sp.line([(8, 4), (11, 1), (13, 1), (16, 4)], 1), R["soot"], "flat")
    sp.part(sp.rect(6, 4, 17, 6), R["soot"], "rim")
    sp.part(sp.rect(6, 19, 17, 21), R["soot"], "rim")
    sp.part(sp.rect(7, 7, 16, 18), R["lamp"], "flat")
    sp.part(sp.rect(7, 7, 7, 18) | sp.rect(16, 7, 16, 18), R["soot"], "flat", seam=False)
    outline(sp)
    sp.fill(sp.rect(8, 7, 15, 18), R["lamp"][1], glow=True)
    sp.fill(sp.ellipse(9, 9, 14, 17), R["lamp"][2], glow=True)
    sp.fill(sp.ellipse(10, 11, 13, 16), R["lamp"][3], glow=True)
    spark(sp, 21, 9, R["flame"])


ICONS = {
    "produce_wheat": p_wheat, "produce_turnip": p_turnip, "produce_sun_melon": p_melon, "produce_chili": p_chili,
    "produce_rice": p_rice, "produce_frost_kale": p_kale, "produce_glowcap": p_glowcap,
    "tool_hand": t_hand, "tool_hoe": t_hoe, "tool_can": t_can, "tool_seeds": t_seeds, "tool_fert": t_fert, "tool_sickle": t_sickle, "tool_lantern": t_lantern,
}


def draw(size, fn, *args):
    sp = Sprite(size, size)
    fn(sp, *args)
    if not getattr(sp, "outlined", False):
        outline(sp)
    return sp.image()


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    items = []
    src = "tools/art/farming_art.py (balatro-pixel-art kit)"
    for name, (fn, seed) in CROPS.items():
        for s in range(4):
            img = draw(CW, mound, seed) if s == 0 else draw(CW, fn, s)
            save(img, os.path.join(OUT, f"{name}_{s}.png"), src)
            items.append((f"{name}_{s}", img))
    for name, fn in ICONS.items():
        img = draw(IW, fn)
        save(img, os.path.join(OUT, name + ".png"), src)
        items.append((name, img))
    print("wrote", len(items), "pngs to", OUT)
    if "--sheet" in argv:
        out = argv[argv.index("--sheet") + 1]
        sheet(items, scale=4, cols=8, cell=(CW * 4, CW * 4)).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main(sys.argv[1:])
