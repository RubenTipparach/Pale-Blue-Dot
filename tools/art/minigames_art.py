"""Sprites for docs/mockups/minigames.html: pixel art drawn in code with the balatro-pixel-art kit
(.claude/skills/balatro-pixel-art), the solitaire collection's style.

    python tools/art/minigames_art.py                 # writes docs/mockups/minigames/*.png
    python tools/art/minigames_art.py --sheet OUT.png # and a contact sheet beside the kit's references

Groups: the ore sorter's ores and drill, the Reactor Roll's reel symbols (also the four suits of
Orbit 21), dice, chips, a card back, the race ships, and the job board's badges. Every sprite is
32 x 32 at native size except the card back (22 x 32, a card's 66 x 96 at 3x); the page shows them
only at whole-number scales with nearest-neighbour sampling.
"""

import math
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts")
sys.path.insert(0, KIT)
from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402
from PIL import Image  # noqa: E402

OUT_DIR = os.path.join(ROOT, "docs", "mockups", "minigames")
N = 32
WHITE = '#fffefe'
WARM = '#faaf5c'

ROCK = ['#2c2c25', '#41322c', '#725244', '#ab8465']
SLAG = ['#161612', '#2c2c25', '#434137', '#a09f91']
COPPER = ['#561c27', '#8d4526', '#c56d25', '#e69d69']
GOLDEN = ['#c56d25', '#fb902f', '#faaf5c']


def lumpy(sp, cx, cy, rx, ry, n=10, jag=0.16, seed=1):
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n + 0.3
        k = 1 + jag * math.sin(seed * 12.9898 + i * 78.233)
        pts.append((cx + rx * k * math.cos(a), cy + ry * k * math.sin(a)))
    return sp.poly(pts)


def spark(sp, x, y, color=WARM):
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], color, glow=True)
    sp.plot(x, y, WHITE, glow=True)


def flecks(sp, pts, ramp):
    for x, y in pts:
        sp.part(sp.rect(x, y, x + 2, y + 2), ramp, 'rim')
        sp.plot(x, y, ramp[-1])


# ---- the ore sorter -----------------------------------------------------------------------------

def ore(body, seed, fleck=None, pts=()):
    def draw():
        sp = Sprite(N, N)
        sp.part(lumpy(sp, 16, 17, 13, 11, seed=seed), body, 'sphere')
        if fleck:
            flecks(sp, pts, fleck)
        sp.outline()
        return sp
    return draw


ore_iron = ore(RAMPS['rust'], 2, RAMPS['metal'][2:], [(9, 11), (18, 18), (13, 21), (21, 10)])
ore_copper = ore(COPPER, 4, RAMPS['glass'][1:4], [(10, 12), (19, 17), (12, 20), (20, 9)])


def ore_gold():
    sp = Sprite(N, N)
    sp.part(lumpy(sp, 16, 17, 13, 11, seed=6), ROCK, 'sphere')
    for x, y, r in ((12, 13, 3), (20, 19, 3), (11, 21, 2)):
        sp.part(sp.circle(x, y, r), RAMPS['gold'][:4], 'sphere')
        sp.plot(x - 1, y - 1, WHITE)
    sp.outline()
    spark(sp, 27, 5)
    return sp


def ore_ice():
    sp = Sprite(N, N)
    for pts, ramp in (([(5, 27), (6, 12), (11, 5), (15, 12), (15, 27)], RAMPS['cyan']),
                      ([(14, 28), (16, 15), (21, 10), (26, 15), (27, 28)], RAMPS['cyan'][:-1]),
                      ([(24, 28), (25, 21), (28, 18), (30, 22), (30, 28)], RAMPS['cyan'])):
        sp.part(sp.poly(pts), ramp, 'cyl_v')
    sp.plots([(9, 9), (19, 13)], WHITE)
    sp.outline()
    spark(sp, 26, 5, '#74effe')
    return sp


def slag():
    sp = Sprite(N, N)
    sp.part(lumpy(sp, 16, 19, 13, 9, n=12, jag=0.2, seed=9), SLAG, 'sphere')
    for x, y in ((10, 15), (18, 13), (14, 21), (22, 20), (8, 21)):
        sp.fill(sp.rect(x, y, x + 1, y + 1), '#161612')
    sp.outline()
    sp.plots([(19, 22), (11, 18)], '#fe6420', glow=True)
    return sp


def drill():
    """A drill head: a brass motor housing over a grooved steel bit."""
    sp = Sprite(N, N)
    sp.part(sp.rect(10, 2, 21, 4), RAMPS['steel'], 'cyl_v')
    sp.part(sp.rect(7, 5, 24, 14), RAMPS['gold'][:4], 'cyl_v')
    sp.fill(sp.rect(7, 9, 24, 9), '#8d4526')
    sp.part(sp.rect(9, 15, 22, 17), RAMPS['soot'], 'cyl_v')
    bit = sp.poly([(10, 18), (21, 18), (16, 30), (15, 30)])
    sp.part(bit, RAMPS['metal'], 'cyl_v')
    for y in (20, 23, 26):
        sp.fill(sp.line([(11 + (y - 18) // 3, y + 1), (20 - (y - 18) // 3, y - 1)], 1) & bit, '#1f3b52')
    sp.plots([(9, 6), (12, 19)], WHITE)
    sp.outline()
    for x, y in ((6, 28), (25, 28), (22, 31)):
        sp.plot(x, y, WARM, glow=True)
    spark(sp, 26, 22)
    return sp


# ---- reel symbols and the four suits --------------------------------------------------------------

def sym_core():
    """A reactor core: a green glowing heart in a steel ring."""
    sp = Sprite(N, N)
    ring = sp.circle(16, 16, 14) & ~sp.circle(16, 16, 9)
    sp.part(ring, RAMPS['metal'], 'sphere')
    for a in range(45, 360, 90):
        x, y = 16 + 11.5 * math.cos(math.radians(a)), 16 + 11.5 * math.sin(math.radians(a))
        sp.plot(round(x), round(y), '#1f3b52')
    sp.plot(8, 7, WHITE)
    sp.outline()
    sp.part(sp.circle(16, 16, 8), ['#05582d', '#297537', '#559642', '#9bb675', '#c1f1d8'], 'inset', glow=True)
    sp.plots([(13, 13), (14, 13), (13, 14)], WHITE, glow=True)
    return sp


def sym_ore():
    """Ore: a fat amber crystal cluster."""
    sp = Sprite(N, N)
    for pts, ramp in (([(4, 28), (5, 15), (10, 10), (14, 16), (13, 28)], RAMPS['amber'][:-1]),
                      ([(18, 28), (19, 14), (24, 9), (28, 14), (28, 28)], RAMPS['amber'][:-1]),
                      ([(9, 29), (11, 9), (16, 2), (21, 9), (22, 29)], RAMPS['amber'])):
        sp.part(sp.poly(pts), ramp, 'cyl_v')
    sp.part(sp.rect(4, 26, 28, 29), ROCK, 'rim')
    sp.plots([(14, 6), (13, 8)], WHITE)
    sp.outline()
    return sp


def sym_ice():
    """Coolant: a fat drop of it, a snowflake on its face."""
    sp = Sprite(N, N)
    drop = sp.circle(16, 19, 11) | sp.poly([(6, 15), (16, 1), (26, 15)])
    sp.part(drop, RAMPS['cyan'][:-1], 'sphere')
    for a in (0, 60, 120):
        dx, dy = 6 * math.cos(math.radians(a)), 6 * math.sin(math.radians(a))
        sp.fill(sp.line([(16 - dx, 19 - dy), (16 + dx, 19 + dy)], 1), '#d2f8ff')
    sp.plots([(11, 11), (10, 12), (10, 13)], WHITE)
    sp.outline()
    return sp


def sym_fuel():
    """Fuel: a green fuel cell with brass caps and a bolt."""
    sp = Sprite(N, N)
    sp.part(sp.rect(8, 6, 23, 28), RAMPS['green'], 'cyl_v')
    sp.part(sp.rect(7, 3, 24, 6), RAMPS['brass'], 'cyl_v')
    sp.part(sp.rect(7, 27, 24, 30), RAMPS['brass'], 'cyl_v')
    sp.part(sp.rect(12, 0, 19, 2), RAMPS['steel'], 'cyl_v')
    sp.part(sp.poly([(17, 9), (11, 18), (15, 18), (13, 25), (20, 15), (16, 15), (19, 9)]), RAMPS['gold'][1:4], 'flat', base=2)
    sp.plots([(9, 8), (9, 9)], '#9bb675')
    sp.outline()
    return sp


def sym_star():
    sp = Sprite(N, N)
    sp.part(sp.star(16, 17, 15, 7), RAMPS['gold'][:4], 'inset')
    sp.plots([(14, 9), (14, 10)], WHITE)
    sp.outline()
    spark(sp, 28, 4)
    return sp


def sym_meltdown():
    """Meltdown: the radiation trefoil, black on a hot red disc."""
    sp = Sprite(N, N)
    sp.part(sp.circle(16, 16, 14), RAMPS['red'], 'sphere')
    sp.part(sp.circle(16, 16, 11), ['#a61e1f', '#fb902f', '#faaf5c'], 'flat', base=2)
    blades = sp.empty()
    for a in (-90, 30, 150):
        a0, a1 = math.radians(a - 30), math.radians(a + 30)
        pts = [(16, 16)] + [(16 + 10 * math.cos(a0 + (a1 - a0) * k / 6), 16 + 10 * math.sin(a0 + (a1 - a0) * k / 6))
                            for k in range(7)]
        blades |= sp.poly(pts)
    blades &= ~sp.circle(16, 16, 3)
    blades |= sp.circle(16, 16, 1)
    sp.fill(blades, '#161612')
    sp.plots([(8, 7), (7, 8)], WHITE)
    sp.outline()
    return sp


# ---- dice, chips, the card back -----------------------------------------------------------------

PIPS = {1: [(0, 0)], 2: [(-1, -1), (1, 1)], 3: [(-1, -1), (0, 0), (1, 1)],
        4: [(-1, -1), (1, -1), (-1, 1), (1, 1)], 5: [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)],
        6: [(-1, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (1, 1)]}


def die(n):
    def draw():
        sp = Sprite(N, N)
        side = sp.poly([(3, 26), (6, 29), (29, 29), (29, 6), (26, 3)])
        sp.part(side, RAMPS['red'][:2], 'flat', base=0)
        face = sp.rect(3, 3, 26, 26) & ~sp.rect(3, 3, 3, 3) & ~sp.rect(26, 3, 26, 3) & ~sp.rect(3, 26, 3, 26) & ~sp.rect(26, 26, 26, 26)
        sp.part(face, RAMPS['red'], 'rim', depth=1)
        for dx, dy in PIPS[n]:
            x, y = 14 + dx * 7, 14 + dy * 7
            sp.part(sp.rect(x - 1, y - 1, x + 2, y + 2), ['#bfcad7', WHITE], 'rim', base=1, seam_color='#4a051e')
        sp.outline()
        return sp
    return draw


def chip(ramp, stripe=WHITE):
    def draw():
        sp = Sprite(N, N)
        disc = sp.circle(16, 16, 14)
        sp.part(disc, ramp, 'sphere')
        for a in range(0, 360, 60):
            wedge = sp.poly([(16, 16), (16 + 18 * math.cos(math.radians(a - 9)), 16 + 18 * math.sin(math.radians(a - 9))),
                             (16 + 18 * math.cos(math.radians(a + 9)), 16 + 18 * math.sin(math.radians(a + 9)))])
            sp.fill(wedge & disc & ~sp.circle(16, 16, 11), stripe)
        sp.part(sp.circle(16, 16, 9), ramp, 'flat', seam_color='#161612')
        hexa = sp.poly([(16 + 5 * math.cos(math.radians(a)), 16 + 5 * math.sin(math.radians(a))) for a in range(30, 390, 60)])
        sp.part(hexa, ['#bfcad7', WHITE], 'rim', base=1, seam=False)
        sp.plots([(7, 8), (8, 7)], WHITE)
        sp.outline()
        return sp
    return draw


def card_back():
    sp = Sprite(22, 32)
    card = sp.rect(0, 0, 21, 31) & ~sp.rect(0, 0, 0, 0) & ~sp.rect(21, 0, 21, 0) & ~sp.rect(0, 31, 0, 31) & ~sp.rect(21, 31, 21, 31)
    sp.part(card, ['#a61e1f'], 'flat', base=0)
    sp.part(sp.rect(2, 2, 19, 29), ['#191954', '#15578a'], 'flat', base=0, seam=False)
    sp.dither(sp.rect(3, 3, 18, 28), '#191954', '#15578a', 0.25)
    sp.fill(sp.rect(2, 2, 19, 2) | sp.rect(2, 29, 19, 29) | sp.rect(2, 2, 2, 29) | sp.rect(19, 2, 19, 29), '#c56d25')
    hexa = sp.poly([(10.5 + 6 * math.cos(math.radians(a)), 15.5 + 6 * math.sin(math.radians(a))) for a in range(30, 390, 60)])
    sp.part(hexa, RAMPS['gold'][:4], 'rim', seam_color='#011c34')
    sp.part(sp.poly([(10.5 + 3 * math.cos(math.radians(a)), 15.5 + 3 * math.sin(math.radians(a))) for a in range(30, 390, 60)]),
            ['#191954', '#15578a'], 'flat', base=0)
    sp.plots([(8, 11), (9, 10)], WHITE)
    for x, y in ((5, 5), (16, 5), (5, 26), (16, 26)):
        sp.plot(x, y, '#faaf5c')
    return sp


# ---- race ships ---------------------------------------------------------------------------------

def racer(team):
    def draw():
        sp = Sprite(N, N)
        sp.part(sp.poly([(10, 12), (17, 12), (13, 3), (8, 3)]), team, 'rim')
        sp.part(sp.poly([(7, 11), (25, 11), (31, 16), (25, 21), (7, 21)]), RAMPS['paper'], 'cyl_h')
        sp.part(sp.poly([(19, 11), (24, 11), (28, 15), (19, 15)]), RAMPS['glass'][1:], 'flat', base=2)
        sp.part(sp.rect(8, 16, 25, 18), team, 'flat', base=len(team) - 2, seam=False)
        sp.part(sp.poly([(10, 21), (18, 21), (14, 29), (8, 29)]), team, 'rim')
        sp.part(sp.rect(4, 12, 7, 20), RAMPS['soot'], 'rim')
        sp.plots([(21, 12), (22, 12)], WHITE)
        sp.outline()
        sp.part(sp.taper(3, 16, -3, 16, 6, 2), RAMPS['flame'], 'inset', glow=True)
        sp.plots([(2, 16), (2, 15)], WHITE, glow=True)
        return sp
    return draw


# ---- job board badges ---------------------------------------------------------------------------

def bounty():
    """Wanted: a skull in a red reticle."""
    sp = Sprite(N, N)
    sp.part(sp.ellipse(9, 7, 23, 20), RAMPS['bone'], 'sphere')
    sp.part(sp.rect(12, 19, 20, 24), RAMPS['bone'], 'rim')
    for x in (13, 16, 19):
        sp.fill(sp.rect(x, 22, x, 24), '#a09f91')
    for x in (11, 17):
        sp.fill(sp.rect(x, 12, x + 3, 15), '#161612')
    sp.fill(sp.rect(15, 17, 16, 18), '#41322c')
    sp.plots([(12, 9), (11, 10)], WHITE)
    sp.outline()
    ring = sp.circle(16, 16, 15) & ~sp.circle(16, 16, 13)
    sp.fill(ring, '#d12d41', glow=True)
    for r in (sp.rect(15, 0, 16, 5), sp.rect(15, 26, 16, 31), sp.rect(0, 15, 5, 16), sp.rect(26, 15, 31, 16)):
        sp.fill(r, '#fa3a64', glow=True)
    return sp


def salvage():
    """Salvage: a wrench over a torn hull plate."""
    sp = Sprite(N, N)
    plate = sp.poly([(3, 6), (24, 3), (27, 12), (22, 16), (26, 22), (20, 28), (4, 27)])
    sp.part(plate, RAMPS['rust'], 'rim', depth=2)
    for x, y in ((6, 9), (20, 7), (7, 24)):
        sp.plot(x, y, '#2e0d06')
    sp.part(sp.line([(8, 25), (22, 11)], 4), RAMPS['metal'], 'rim')
    jaw = sp.circle(24, 8, 6) & ~sp.poly([(24, 8), (31, 1), (32, 6)]) & ~sp.circle(24, 8, 2)
    sp.part(jaw, RAMPS['metal'], 'rim')
    sp.part(sp.circle(7, 26, 3) & ~sp.circle(7, 26, 1), RAMPS['metal'], 'rim')
    sp.plots([(20, 4), (12, 19)], WHITE)
    sp.outline()
    spark(sp, 28, 26)
    return sp


def cargo():
    """Cargo: a banded crate with brass corners."""
    sp = Sprite(N, N)
    top = sp.poly([(4, 9), (12, 3), (29, 3), (21, 9)])
    sp.part(top, RAMPS['wood'], 'flat', base=3)
    side = sp.poly([(21, 9), (29, 3), (29, 21), (21, 28)])
    sp.part(side, RAMPS['wood'], 'flat', base=1)
    front = sp.rect(3, 9, 21, 28)
    sp.part(front, RAMPS['wood'], 'rim')
    for y in (14, 20):
        sp.fill(sp.rect(4, y, 20, y), '#41322c')
    sp.part(sp.rect(10, 12, 14, 24), RAMPS['olive'], 'flat', base=2, seam_color='#41322c')
    for x, y in ((3, 9), (19, 9), (3, 26), (19, 26)):
        sp.part(sp.rect(x, y, x + 2, y + 2), RAMPS['brass'], 'rim', seam=False)
    sp.plots([(5, 11), (14, 4)], WHITE)
    sp.outline()
    return sp


SPRITES = {
    "ore_iron": ore_iron, "ore_copper": ore_copper, "ore_gold": ore_gold, "ore_ice": ore_ice,
    "slag": slag, "drill": drill,
    "sym_core": sym_core, "sym_ore": sym_ore, "sym_ice": sym_ice, "sym_fuel": sym_fuel,
    "sym_star": sym_star, "sym_meltdown": sym_meltdown,
    **{f"die_{n}": die(n) for n in range(1, 7)},
    "chip_10": chip(RAMPS['blue']), "chip_50": chip(RAMPS['red']),
    "chip_100": chip(['#010000', '#161612', '#2c2c25', '#434137'], '#faaf5c'),
    "card_back": card_back,
    "racer_red": racer(RAMPS['red']), "racer_blue": racer(RAMPS['blue']),
    "racer_green": racer(RAMPS['green']), "racer_yellow": racer(RAMPS['gold'][:4]),
    "bounty": bounty, "salvage": salvage, "cargo": cargo,
}

REFS = ["all_in", "emp_burst", "cluster_bombs", "armour_plating", "drop_tanks", "cruise_missile"]


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    items = []
    for name, fn in SPRITES.items():
        img = fn().image()
        path = os.path.join(OUT_DIR, name + ".png")
        save(img, path, "tools/art/minigames_art.py (balatro-pixel-art kit)")
        items.append((name, img))
    print("wrote", len(items), "sprites to", OUT_DIR)
    if "--sheet" in sys.argv:
        i = sys.argv.index("--sheet")
        out = sys.argv[i + 1] if len(sys.argv) > i + 1 else "minigames_sheet.png"
        ref_dir = os.path.join(os.path.dirname(KIT), "references")
        refs = [("ref " + n, Image.open(os.path.join(ref_dir, n + ".png")).convert("RGBA")) for n in REFS]
        sheet(items + refs, scale=4, cols=7, cell=(224, 224)).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main()
