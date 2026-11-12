"""Station portraits for docs/mockups/stations.html: pixel art drawn in code with the
balatro-pixel-art kit (.claude/skills/balatro-pixel-art), the solitaire collection's style.

    python tools/art/stations_art.py                 # writes docs/mockups/stations/*.png
    python tools/art/stations_art.py --sheet OUT.png # and a contact sheet beside the kit's references

Each portrait is one station type from the stations mockup, 96 x 64 at native size (the size of the
reference set's boss portraits), shown on the page at 2x with nearest-neighbour scaling. One bold
subject, short ramps from Sweaft-64 lit from the top left, dark seams, a 1 px outline, lamps and
flames drawn last as light (never outlined).
"""

import math
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts")
sys.path.insert(0, KIT)
from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402
from PIL import Image  # noqa: E402

OUT_DIR = os.path.join(ROOT, "docs", "mockups", "stations")
W, H = 96, 64

ROCK = ['#2c2c25', '#41322c', '#725244', '#ab8465']
DUSK_ROCK = ['#2b1f35', '#3e3551', '#69608b', '#92a3b2']
NAVY = ['#011c34', '#1f3b52', '#3e586e', '#5d778d']
WARM = '#faaf5c'
WHITE = '#fffefe'


def far(ramp):
    """The far side of a thing: one ramp step darker."""
    return [ramp[0]] + ramp[:-1]


def lumpy(sp, cx, cy, rx, ry, n=18, jag=0.14, seed=1, top_flat=None):
    """A lumpy rock outline: an ellipse with a deterministic wobble on its radius."""
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n
        k = 1 + jag * math.sin(seed * 12.9898 + i * 78.233) * 1.0
        k += jag * 0.5 * math.sin(seed * 4.1 + i * 2.7)
        pts.append((cx + rx * k * math.cos(a), cy + ry * k * math.sin(a)))
    return sp.poly(pts)


def spark(sp, x, y, color=WARM):
    """A sparkle: a white pixel in a plus of the palest warm tone (light, never outlined)."""
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], color, glow=True)
    sp.plot(x, y, WHITE, glow=True)


def lamps(sp, pts, color=WARM, core=None):
    """Lit windows: 2 x 1 pixels of warm light, after the outline."""
    for x, y in pts:
        sp.plots([(x, y), (x + 1, y)], color, glow=True)
        if core:
            sp.plot(x, y, core, glow=True)


def trade_ring():
    """The orbital trade hub: a spinning ring on spokes round a still hub."""
    sp = Sprite(W, H)
    cy = 38
    band = sp.ellipse(3, 16, 92, 60) & ~sp.ellipse(14, 24, 81, 52)
    rows = sp.empty()
    rows[:cy, :] = True
    sp.part(band & rows, far(RAMPS['metal']), 'cyl_h')
    for a in (0.55, 2.6, 3.7, 5.75):
        x, y = 48 + 40 * math.cos(a), cy + 18 * math.sin(a)
        sp.part(sp.line([(48, cy - 4), (x, y)], 2), RAMPS['steel'], 'flat', base=1, seam=False)
    hub = sp.rect(40, 12, 55, 50)
    sp.part(hub, RAMPS['metal'], 'cyl_v')
    for y in (20, 30, 40):
        sp.fill(sp.rect(40, y, 55, y), '#3e586e')
    sp.part(sp.rect(42, 6, 53, 12), RAMPS['red'], 'cyl_v')
    sp.part(sp.rect(46, 1, 49, 6), RAMPS['steel'], 'cyl_v')
    sp.part(band & ~rows, RAMPS['metal'], 'cyl_h')
    # the ring's plating joints
    for t in range(0, 360, 30):
        a = math.radians(t)
        if math.sin(a) > 0.05:
            x = round(48 + 40.5 * math.cos(a))
            ys = [y for y in range(cy, H) if band[y, x]]
            for y in ys[1:-1]:
                sp.plot(x, y, '#3e586e')
    sp.plots([(41, 14), (42, 14), (44, 7)], WHITE)
    sp.outline()
    lamps(sp, [(round(48 + 40 * math.cos(math.radians(t))), round(cy + 16 + 1 * math.sin(math.radians(t))) - 1)
               for t in range(20, 170, 18)])
    lamps(sp, [(44, 24), (50, 24), (44, 34), (50, 34), (47, 44)])
    sp.plots([(47, 0), (48, 0)], '#fa3a64', glow=True)
    spark(sp, 10, 7)
    spark(sp, 86, 8, '#74effe')
    return sp


def mining_outpost():
    """A mining outpost bolted onto a rock: a derrick, a dome, ore breaking out."""
    sp = Sprite(W, H)
    rock = lumpy(sp, 48, 46, 44, 16, n=20, jag=0.1, seed=3)
    sp.part(rock, ROCK, 'sphere')
    for cx, cy, r in ((30, 52, 4), (60, 55, 3), (74, 46, 3), (44, 57, 2)):
        sp.part(sp.ellipse(cx - r, cy - r + 1, cx + r, cy + r - 1), far(ROCK), 'inset')
    # the derrick, a lattice tower over a drill hole
    sp.part(sp.taper(57, 36, 64, 4, 3, 2), RAMPS['brass'], 'flat', base=2)
    sp.part(sp.taper(73, 36, 66, 4, 3, 2), RAMPS['brass'], 'flat', base=1)
    for y, hw in ((30, 7), (22, 5), (14, 3)):
        sp.part(sp.rect(65 - hw, y, 65 + hw, y + 1), RAMPS['brass'], 'flat', base=2, seam=False)
    sp.part(sp.rect(62, 2, 68, 5), RAMPS['steel'], 'rim')
    sp.part(sp.rect(64, 6, 65, 38), RAMPS['ink'], 'flat', base=2, seam=False)
    # the dome and its deck
    sp.part(sp.rect(8, 33, 46, 37), NAVY, 'cyl_h')
    dome = sp.ellipse(12, 16, 42, 48)
    dome[34:, :] = False
    sp.part(dome, RAMPS['glass'], 'sphere')
    sp.part(sp.rect(26, 16, 27, 33), RAMPS['glass'][:2], 'flat', base=1, seam=False)
    sp.part(sp.rect(13, 26, 41, 26), RAMPS['glass'][:2], 'flat', base=1, seam=False)
    sp.plots([(18, 20), (19, 20), (17, 21)], WHITE)
    # ore crystals breaking out of the rock
    for pts, ramp in (([(78, 42), (80, 26), (84, 22), (87, 28), (86, 42)], RAMPS['amber']),
                      ([(86, 44), (89, 34), (92, 33), (93, 45)], RAMPS['amber']),
                      ([(6, 50), (6, 40), (10, 36), (13, 41), (12, 50)], RAMPS['cyan'])):
        sp.part(sp.poly(pts), ramp, 'inset')
    sp.plots([(83, 24), (9, 39)], WHITE)
    sp.outline()
    lamps(sp, [(16, 30), (21, 29), (31, 29), (36, 30)])
    lamps(sp, [(12, 35), (20, 35), (28, 35), (36, 35)], '#fb902f')
    sp.plots([(64, 1), (65, 1)], '#fa3a64', glow=True)
    spark(sp, 90, 14)
    spark(sp, 4, 30, '#74effe')
    return sp


def refinery():
    """A refinery: a spine of tanks and a cracking tower with a gas flare."""
    sp = Sprite(W, H)
    sp.part(sp.rect(3, 42, 92, 48), RAMPS['metal'], 'cyl_h')
    sp.part(sp.line([(56, 28), (68, 28), (68, 42)], 3), RAMPS['steel'], 'flat', base=2)
    tower = sp.rect(38, 19, 54, 45)
    sp.part(tower, RAMPS['brass'], 'cyl_v')
    for y in (25, 31, 37):
        sp.fill(sp.rect(38, y, 54, y), '#733126')
    sp.part(sp.rect(36, 17, 56, 20), RAMPS['steel'], 'cyl_h')
    sp.part(sp.rect(44, 11, 48, 17), RAMPS['soot'], 'cyl_v')
    for cx, cy, r, ramp in ((17, 50, 12, RAMPS['bone']), (82, 51, 10, RAMPS['red']), (61, 53, 9, RAMPS['bone'])):
        sp.part(sp.circle(cx, cy, r), ramp, 'sphere')
        sp.fill(sp.rect(cx - r + 2, cy - 1, cx + r - 2, cy - 1), ramp[0])
        sp.plot(cx - r // 2, cy - r // 2 - 1, WHITE)
    sp.outline()
    flame = sp.taper(46, 11, 46, 0, 9, 1) | sp.taper(46, 11, 51, 3, 5, 0) | sp.taper(46, 11, 42, 4, 4, 0)
    sp.part(flame, RAMPS['flame'], 'inset', glow=True)
    lamps(sp, [(28, 45), (72, 45), (90, 45)])
    lamps(sp, [(41, 22), (49, 22), (41, 33), (49, 39)], '#fffefe')
    spark(sp, 8, 14)
    spark(sp, 84, 24)
    return sp


def customs_post():
    """A military customs post: an armoured hex, red stripe, two turrets and a beacon."""
    sp = Sprite(W, H)
    hexa = lambda r, ry: sp.poly([(48 + r * math.cos(math.radians(a)), 38 + ry * math.sin(math.radians(a)))
                                  for a in range(0, 360, 60)])
    for x, s in ((14, -1), (82, 1)):
        sp.part(sp.taper(x, 16, x + 9 * s, 3, 4, 3), RAMPS['soot'], 'flat', base=2)
        sp.part(sp.circle(x, 17, 6), RAMPS['steel'], 'sphere')
    sp.part(sp.rect(18, 18, 26, 22), RAMPS['soot'], 'rim')
    sp.part(sp.rect(70, 18, 78, 22), RAMPS['soot'], 'rim')
    sp.part(hexa(40, 25), NAVY, 'rim', depth=2)
    sp.part(hexa(31, 19), RAMPS['metal'], 'rim')
    for a in range(0, 360, 60):
        x, y = 48 + 36 * math.cos(math.radians(a)), 38 + 22.5 * math.sin(math.radians(a))
        sp.plot(round(x), round(y), '#bfcad7')
    sp.part(sp.rect(20, 33, 76, 37), RAMPS['red'], 'cyl_h')
    sp.part(sp.rect(40, 42, 56, 51), RAMPS['ink'], 'flat', base=0)
    sp.part(sp.rect(46, 4, 49, 13), RAMPS['steel'], 'cyl_v')
    sp.plots([(38, 23), (39, 22), (12, 13)], WHITE)
    sp.outline()
    sp.part(sp.rect(42, 44, 54, 49), RAMPS['amber'], 'inset', glow=True)
    sp.plots([(47, 2), (48, 2), (47, 3), (48, 3)], '#fa3a64', glow=True)
    sp.plot(47, 2, WHITE, glow=True)
    for x in (40, 44, 51, 55):
        sp.plot(x, 1, '#d12d41', glow=True)
    lamps(sp, [(28, 39), (64, 39), (24, 29), (68, 29)])
    spark(sp, 6, 50, '#74effe')
    return sp


def smuggler_den():
    """A smuggler den in a hollow rock: a neon sign, a lit cave mouth, crates, a cutter moored."""
    sp = Sprite(W, H)
    rock = lumpy(sp, 46, 38, 42, 24, n=22, jag=0.12, seed=7)
    sp.part(rock, DUSK_ROCK, 'sphere')
    sp.part(sp.ellipse(16, 36, 22, 40), far(DUSK_ROCK), 'inset')
    sp.part(sp.ellipse(64, 48, 72, 52), far(DUSK_ROCK), 'inset')
    mouth = sp.ellipse(26, 30, 66, 58)
    sp.part(mouth, ['#170916'], 'flat', base=0)
    sp.part(sp.ellipse(31, 36, 61, 58) & ~sp.rect(0, 57, W, H), ['#841c1d', '#d4462b', '#fe6420', '#fb902f'], 'inset')
    for x, y, ramp in ((34, 48, RAMPS['wood']), (49, 48, RAMPS['green']), (41, 42, RAMPS['wood'])):
        sp.part(sp.rect(x, y, x + 8, y + 7), ramp, 'rim')
        sp.fill(sp.rect(x, y + 3, x + 8, y + 3), ramp[0])
    # the neon sign, a crooked bar
    sign = sp.poly([(24, 10), (70, 6), (71, 17), (25, 21)])
    sp.part(sign, ['#170916', '#2b1f35'], 'flat', base=0)
    # a cutter moored at the right
    sp.part(sp.line([(84, 34), (92, 30)], 1), RAMPS['steel'], 'flat', base=1, seam=False)
    sp.part(sp.poly([(80, 26), (92, 22), (95, 25), (92, 30), (80, 31)]), RAMPS['red'], 'cyl_h')
    sp.plot(90, 24, WHITE)
    sp.outline()
    for pts, c in (([(29, 18), (32, 13), (35, 18), (38, 13), (41, 17)], '#fa3a64'),
                   ([(45, 15), (50, 12), (55, 15), (60, 11), (66, 13)], '#74effe')):
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            sp.fill(sp.line([(x0, y0), (x1, y1)], 2), c, glow=True)
    sp.plots([(32, 13), (60, 11)], WHITE, glow=True)
    spark(sp, 80, 8, '#fa3a64')
    spark(sp, 8, 22, '#74effe')
    sp.plots([(78, 28), (77, 28)], '#fe6420', glow=True)
    return sp


def research_station():
    """A research station: a sphere lab on a truss, solar wings, a dish talking home."""
    sp = Sprite(W, H)
    sp.part(sp.rect(18, 37, 78, 40), RAMPS['steel'], 'cyl_h')
    for x0, x1 in ((2, 26), (70, 93)):
        sp.part(sp.rect(x0, 30, x1, 46), RAMPS['blue'], 'rim')
        for x in range(x0 + 6, x1, 6):
            sp.fill(sp.rect(x, 30, x, 46), '#191954')
        sp.fill(sp.rect(x0, 38, x1, 38), '#191954')
        sp.plot(x0 + 2, 32, '#d2f8ff')
    sp.part(sp.line([(56, 22), (66, 12)], 2), RAMPS['steel'], 'flat', base=1)
    dish = sp.ellipse(58, 2, 82, 14)
    sp.part(dish, RAMPS['bone'], 'rim')
    sp.part(sp.ellipse(62, 5, 80, 12), RAMPS['paper'], 'flat', base=0)
    sp.part(sp.line([(70, 8), (76, 3)], 1), RAMPS['ink'], 'flat', base=2, seam=False)
    lab = sp.circle(48, 38, 18)
    sp.part(lab, RAMPS['paper'] + [WHITE], 'sphere', base=2)
    sp.part(sp.rect(31, 40, 65, 41) & lab, RAMPS['metal'], 'flat', base=2)
    sp.plots([(40, 26), (41, 26), (39, 27)], WHITE)
    sp.outline()
    lamps(sp, [(38, 34), (44, 32), (50, 32), (56, 34)], '#84e3b5', '#c1f1d8')
    lamps(sp, [(40, 46), (52, 46)], WARM)
    for r in (5, 9, 13):
        pts = [(76 + r * math.cos(a), 3 - 0 + r * math.sin(a) * 0.9) for a in [k * 0.08 - 0.3 for k in range(14)]]
        for x, y in pts:
            if 0 <= y < H and 0 <= x < W and not sp.alpha[int(round(y)), int(round(x))]:
                sp.plot(int(round(x)), int(round(y)), '#84e3b5', glow=True)
    spark(sp, 10, 12, '#84e3b5')
    spark(sp, 88, 56)
    return sp


SPRITES = {
    "trade_ring": trade_ring,
    "mining_outpost": mining_outpost,
    "refinery": refinery,
    "customs_post": customs_post,
    "smuggler_den": smuggler_den,
    "research_station": research_station,
}

# ---- hull textures for the 3D scene ---------------------------------------------------------------
# 32 x 32 tiles that repeat (the scene maps one tile to TEX_M metres of hull, nearest, no mipmaps).
# Drawn with the same kit and palette; no outline, since a texture tiles edge to edge.

T = 32


def tile():
    sp = Sprite(T, T)
    sp.fill(sp.rect(0, 0, T - 1, T - 1), '#5d778d')
    return sp


def panels(sp, pw, ph, body, light, dark, rivet, offset=0):
    """Plates pw x ph with a lit top/left edge, a dark seam bottom/right and corner rivets."""
    for py in range(0, T, ph):
        off = offset if (py // ph) % 2 else 0
        for px in range(-pw, T + pw, pw):
            x0 = px + off
            sp.fill(sp.rect(x0, py, x0 + pw - 1, py + ph - 1), body)
            sp.fill(sp.rect(x0, py, x0 + pw - 1, py), light)
            sp.fill(sp.rect(x0, py, x0, py + ph - 1), light)
            sp.fill(sp.rect(x0, py + ph - 1, x0 + pw - 1, py + ph - 1), dark)
            sp.fill(sp.rect(x0 + pw - 1, py, x0 + pw - 1, py + ph - 1), dark)
            for rx, ry in ((x0 + 2, py + 2), (x0 + pw - 3, py + 2), (x0 + 2, py + ph - 3), (x0 + pw - 3, py + ph - 3)):
                sp.plot(rx, ry, rivet)


def tex_hull():
    sp = tile()
    panels(sp, 16, 16, '#92a3b2', '#bfcad7', '#3e586e', '#5d778d')
    sp.dither(sp.rect(17, 17, 30, 30), '#92a3b2', '#5d778d', 0.12)
    sp.fill(sp.rect(4, 7, 11, 8), '#5d778d')   # a vent on one plate
    return sp


def tex_armour():
    sp = tile()
    panels(sp, 32, 16, '#3e586e', '#5d778d', '#011c34', '#92a3b2', offset=16)
    return sp


def tex_redband():
    sp = tile()
    panels(sp, 16, 16, '#a61e1f', '#d12d41', '#4a051e', '#fa3a64', offset=8)
    return sp


def tex_solar():
    sp = tile()
    sp.fill(sp.rect(0, 0, T - 1, T - 1), '#191954')
    for y in range(0, T, 8):
        for x in range(0, T, 8):
            sp.fill(sp.rect(x + 1, y + 1, x + 7, y + 7), '#15578a')
            sp.fill(sp.rect(x + 1, y + 1, x + 7, y + 1), '#1096c3')
            sp.plot(x + 2, y + 2, '#74effe')
    sp.fill(sp.rect(0, 15, T - 1, 16), '#bfcad7')
    return sp


def tex_hazard():
    sp = tile()
    sp.fill(sp.rect(0, 0, T - 1, T - 1), '#fb902f')
    for y in range(T):
        for x in range(T):
            if (x + y) % 16 < 8:
                sp.plot(x, y, '#161612')
    return sp


def tex_glass():
    sp = tile()
    sp.fill(sp.rect(0, 0, T - 1, T - 1), '#204f63')
    for y in range(0, T, 16):
        for x in range(0, T, 16):
            sp.fill(sp.rect(x, y, x + 15, y), '#92a3b2')
            sp.fill(sp.rect(x, y, x, y + 15), '#92a3b2')
            sp.fill(sp.line([(x + 3, y + 12), (x + 12, y + 3)], 1), '#53998b')
            sp.plot(x + 4, y + 3, '#84e3b5')
    return sp


TEXTURES = {"tex_hull": tex_hull, "tex_armour": tex_armour, "tex_redband": tex_redband,
            "tex_solar": tex_solar, "tex_hazard": tex_hazard, "tex_glass": tex_glass}


REFS = ["furnace_barge", "foreman_okafor", "captain_zefir", "dreadnought_ascendant", "glass_seraph", "brine_hive"]


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    items = []
    for name, fn in SPRITES.items():
        img = fn().image()
        path = os.path.join(OUT_DIR, name + ".png")
        save(img, path, "tools/art/stations_art.py (balatro-pixel-art kit)")
        items.append((name, img))
        print("wrote", path)
    for name, fn in TEXTURES.items():
        img = fn().image()
        path = os.path.join(OUT_DIR, name + ".png")
        save(img, path, "tools/art/stations_art.py (balatro-pixel-art kit)")
        items.append((name, img))
        print("wrote", path)
    if "--sheet" in sys.argv:
        i = sys.argv.index("--sheet")
        out = sys.argv[i + 1] if len(sys.argv) > i + 1 else "stations_sheet.png"
        ref_dir = os.path.join(os.path.dirname(KIT), "references")
        refs = [("ref " + n, Image.open(os.path.join(ref_dir, n + ".png")).convert("RGBA")) for n in REFS]
        sheet(items + refs, scale=3, cols=4).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main()
