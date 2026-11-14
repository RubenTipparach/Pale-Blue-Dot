"""Sprites for docs/mockups/minigames.html: pixel art drawn in code with the balatro-pixel-art kit
(.claude/skills/balatro-pixel-art), the solitaire collection's style.

    python tools/art/minigames_art.py                 # writes docs/mockups/minigames/*.png
    python tools/art/minigames_art.py --sheet OUT.png # and a contact sheet beside the kit's references

Groups: the ore sorter's ores and drill, the Reactor Roll's reel symbols (also the four suits of
Orbit 21), dice, chips, a card back, the race ships, and the job board's badges; and (2026-11-13)
the casino's themed machines (Deep Vein dv_*, Aubade Frontier pl_*, Smuggler's Hold sm_*), the
Star Chart keno markers (kn_*, 16 x 16), small card pips (pip_*, 16 x 16), the dealer button and
the four Hold'em opponents' portraits (ai_*). Every sprite is 32 x 32 at native size except the
card back (22 x 32, a card's 66 x 96 at 3x), the 16 x 16 markers and pips, and pl_wild_col (32 x 96,
a whole reel); the page shows them only at whole-number scales with nearest-neighbour sampling.
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


# ---- Deep Vein (5 x 3 video slot, ore and mining) --------------------------------------------------

def dv_geode():
    """The scatter: a split geode, violet crystals in a rock rind."""
    sp = Sprite(N, N)
    sp.part(lumpy(sp, 16, 17, 14, 12, n=12, jag=0.12, seed=3), ROCK, 'sphere')
    sp.part(sp.ellipse(7, 9, 25, 25), ['#230c44', '#4f1d83'], 'flat', base=0)
    for pts in ([(9, 22), (11, 12), (14, 20)], [(13, 23), (16, 10), (19, 23)], [(18, 21), (21, 12), (23, 22)],
                [(10, 24), (13, 17), (16, 24)], [(16, 24), (19, 17), (22, 24)]):
        sp.part(sp.poly(pts), RAMPS['violet'][1:], 'cyl_v')
    sp.plots([(15, 13), (11, 15)], WHITE)
    sp.outline()
    spark(sp, 27, 5, '#c34988')
    spark(sp, 4, 27, '#c34988')
    return sp


def dv_pick():
    """A miner's pick: a steel head on a wooden haft."""
    sp = Sprite(N, N)
    sp.part(sp.line([(8, 28), (22, 9)], 3), RAMPS['wood'], 'rim')
    head = sp.poly([(4, 10), (10, 5), (17, 4), (24, 5), (30, 9), (28, 11), (22, 9), (17, 8), (11, 9), (6, 12)])
    sp.part(head, RAMPS['metal'], 'cyl_h')
    sp.part(sp.rect(19, 4, 23, 11), RAMPS['soot'], 'rim')
    sp.plots([(12, 6), (13, 6)], WHITE)
    sp.outline()
    return sp


def dv_cart():
    """An ore cart heaped with amber ore."""
    sp = Sprite(N, N)
    for x, y in ((10, 10), (16, 8), (22, 10)):
        sp.part(sp.circle(x, y, 4), RAMPS['amber'], 'sphere')
    sp.part(sp.poly([(3, 12), (29, 12), (26, 24), (6, 24)]), RAMPS['rust'], 'rim', depth=2)
    sp.fill(sp.rect(5, 16, 27, 16), '#2e0d06')
    for x in (8, 23):
        sp.part(sp.circle(x, 26, 3), RAMPS['soot'], 'sphere')
        sp.plot(x, 26, '#a09f91')
    sp.plots([(14, 6), (5, 13)], WHITE)
    sp.outline()
    return sp


def dv_lamp():
    """A miner's lantern, lit."""
    sp = Sprite(N, N)
    sp.part(sp.rect(13, 1, 18, 3) & ~sp.rect(15, 2, 16, 3), RAMPS['steel'], 'rim')
    sp.part(sp.rect(9, 4, 22, 7), RAMPS['brass'], 'cyl_v')
    sp.part(sp.rect(10, 8, 21, 24), ['#0c2f44', '#204f63', '#53998b'], 'cyl_v')
    sp.part(sp.rect(8, 25, 23, 29), RAMPS['brass'], 'cyl_v')
    for x in (10, 21):
        sp.fill(sp.rect(x, 8, x, 24), '#733126')
    sp.outline()
    sp.part(sp.ellipse(12, 11, 19, 22), RAMPS['flame'], 'inset', glow=True)
    sp.plots([(15, 15), (15, 16)], WHITE, glow=True)
    return sp


# ---- Aubade Frontier (5 x 3 video slot, the Aubade worlds) -------------------------------------

def planet(ramp, extra=None, ring=None):
    def draw():
        sp = Sprite(N, N)
        if ring:
            sp.part(sp.ellipse(1, 13, 30, 20) & ~sp.ellipse(5, 15, 26, 18), ring, 'flat', base=1)
        body = sp.circle(16, 16, 11)
        sp.part(body, ramp, 'sphere')
        if extra:
            extra(sp, body)
        if ring:
            front = sp.ellipse(1, 13, 30, 20) & ~sp.ellipse(5, 15, 26, 18) & sp.rect(0, 17, 31, 31)
            sp.part(front, ring, 'flat', base=2)
        sp.plots([(11, 9), (10, 10)], WHITE)
        sp.outline()
        return sp
    return draw


def _verdance(sp, body):
    for pts in ([(9, 12), (14, 9), (17, 13), (13, 17), (9, 16)], [(17, 18), (23, 16), (25, 21), (20, 25)]):
        sp.part(sp.poly(pts) & body, RAMPS['blue'][1:], 'flat', base=1, seam=False)


def _cinder(sp, body):
    for pts in ([(8, 12), (14, 15), (12, 21)], [(16, 9), (19, 16), (25, 19)], [(14, 22), (20, 20), (22, 25)]):
        sp.fill(sp.line(pts, 1) & body, '#fe6420')
    sp.plots([(14, 15), (19, 16)], '#faaf5c')


def _rime(sp, body):
    sp.part(sp.rect(0, 0, 31, 9) & body, ['#bfcad7', WHITE], 'flat', base=1, seam=False)
    sp.part(sp.rect(0, 24, 31, 31) & body, ['#92a3b2', '#bfcad7'], 'flat', base=1, seam=False)
    sp.fill(sp.line([(8, 16), (14, 14), (20, 17), (25, 15)], 1) & body, '#1096c3')


def _tidewater(sp, body):
    for pts in ([(7, 13), (13, 11), (19, 13)], [(12, 19), (19, 17), (25, 19)], [(9, 23), (15, 23)]):
        sp.fill(sp.line(pts, 2) & body, '#d2f8ff')


def _moon(sp, body):
    for x, y, r in ((12, 13, 2), (19, 18, 3), (13, 21, 1), (21, 11, 1)):
        sp.part(sp.circle(x, y, r) & body, ['#434137', '#a09f91'], 'flat', base=0, seam=False)
        sp.plot(x + 1, y + 1, '#bfcad7')


def pl_aubade():
    """The wild: Aubade, the system's sun, a hot disc with eight rays."""
    sp = Sprite(N, N)
    rays = sp.star(16, 16, 15, 8, points=8, turn=-90)
    sp.part(rays, ['#c56d25', '#fb902f', '#faaf5c'], 'inset', glow=True)
    sp.part(sp.circle(16, 16, 9), RAMPS['gold'], 'sphere')
    sp.plots([(12, 11), (13, 11), (12, 12)], WHITE)
    return sp


def pl_wild_col():
    """The expanded wild: Aubade's light filling a whole reel (32 x 96, three cells)."""
    sp = Sprite(N, 96)
    sp.dither(sp.rect(3, 0, 28, 95), '#c56d25', '#fb902f', 0.5, glow=True)
    sp.dither(sp.rect(8, 0, 23, 95), '#fb902f', '#faaf5c', 0.5, glow=True)
    sp.fill(sp.rect(13, 0, 18, 95), '#faaf5c', glow=True)
    sp.part(sp.star(16, 48, 15, 8, points=8), ['#c56d25', '#fb902f', '#faaf5c'], 'inset', glow=True)
    sp.part(sp.circle(16, 48, 9), RAMPS['gold'], 'sphere', glow=True)
    sp.plots([(12, 43), (13, 43), (12, 44)], WHITE, glow=True)
    for y in (12, 80):
        spark(sp, 16, y)
    return sp


# ---- Smuggler's Hold (3 x 3 classic, contraband) -------------------------------------------------

def sm_cutter():
    """The wild: a smuggler's cutter, soot-black with a red stripe."""
    sp = Sprite(N, N)
    sp.part(sp.poly([(9, 12), (17, 12), (12, 4), (7, 4)]), RAMPS['red'], 'rim')
    sp.part(sp.poly([(5, 11), (24, 11), (31, 16), (24, 21), (5, 21)]), RAMPS['soot'], 'cyl_h')
    sp.part(sp.poly([(19, 12), (24, 12), (28, 15), (19, 15)]), ['#4a051e', '#a61e1f', '#fa3a64'], 'flat', base=2)
    sp.part(sp.rect(6, 17, 24, 18), RAMPS['red'], 'flat', base=2, seam=False)
    sp.part(sp.poly([(9, 21), (17, 21), (12, 28), (7, 28)]), RAMPS['red'], 'rim')
    sp.plots([(21, 13), (22, 13)], WHITE)
    sp.outline()
    sp.part(sp.taper(3, 16, -3, 16, 6, 2), ['#4f1d83', '#7d2dbd', '#c34988', '#fffefe'], 'inset', glow=True)
    return sp


def sm_vial():
    """Contraband: a glowing vial with a steel cap."""
    sp = Sprite(N, N)
    sp.part(sp.rect(12, 2, 19, 6), RAMPS['steel'], 'cyl_v')
    sp.part(sp.rect(13, 7, 18, 10), RAMPS['paper'], 'cyl_v')
    sp.part(sp.circle(16, 20, 10) | sp.rect(12, 9, 19, 14), RAMPS['paper'][:2], 'flat', base=0)
    sp.outline()
    sp.part(sp.circle(16, 21, 8), ['#05582d', '#297537', '#559642', '#9bb675', '#c1f1d8'], 'inset', glow=True)
    sp.plots([(11, 16), (11, 17), (12, 15)], WHITE, glow=True)
    spark(sp, 26, 6, '#84e3b5')
    return sp


def sm_badge():
    """Customs: a navy shield with a gold hex."""
    sp = Sprite(N, N)
    sp.part(sp.poly([(4, 4), (28, 4), (28, 16), (16, 29), (4, 16)]), RAMPS['blue'], 'rim', depth=2)
    hexa = sp.poly([(16 + 6 * math.cos(math.radians(a)), 14 + 6 * math.sin(math.radians(a))) for a in range(30, 390, 60)])
    sp.part(hexa, RAMPS['gold'][:4], 'rim', seam_color='#191954')
    sp.plots([(7, 6), (6, 7)], WHITE)
    sp.outline()
    return sp


def sm_stash():
    """The scatter: a lockbox, lid up, credit chips inside."""
    sp = Sprite(N, N)
    sp.part(sp.poly([(5, 13), (8, 4), (27, 4), (27, 13)]), RAMPS['metal'][:4], 'rim')
    sp.part(sp.rect(4, 13, 27, 28), RAMPS['metal'], 'rim', depth=2)
    sp.part(sp.rect(6, 11, 25, 15), RAMPS['gold'][:4], 'flat', base=3)
    sp.part(sp.rect(13, 19, 18, 23), RAMPS['brass'], 'rim', seam_color='#1f3b52')
    sp.plots([(7, 6), (5, 15)], WHITE)
    sp.outline()
    spark(sp, 10, 9)
    spark(sp, 22, 8)
    return sp


# ---- Star Chart keno markers (16 x 16) and card pips ---------------------------------------------

def kn_pick():
    """The player's pick: a mint hex marker."""
    sp = Sprite(16, 16)
    hexa = sp.poly([(7.5 + 7 * math.cos(math.radians(a)), 7.5 + 7 * math.sin(math.radians(a))) for a in range(30, 390, 60)])
    sp.part(hexa & ~sp.poly([(7.5 + 4 * math.cos(math.radians(a)), 7.5 + 4 * math.sin(math.radians(a))) for a in range(30, 390, 60)]),
            RAMPS['glass'][1:], 'rim')
    sp.outline()
    return sp


def kn_drawn():
    """A drawn sector: a cyan probe ping."""
    sp = Sprite(16, 16)
    sp.part(sp.star(7.5, 7.5, 7, 3, points=4, turn=-90), RAMPS['cyan'][1:], 'inset', glow=True)
    sp.plot(7, 7, WHITE, glow=True)
    return sp


def kn_hit():
    """A hit: the probe found the player's star, a gold burst."""
    sp = Sprite(16, 16)
    sp.part(sp.star(7.5, 7.5, 8, 3, points=8, turn=-90), ['#c56d25', '#fb902f', '#faaf5c', WHITE], 'inset', glow=True)
    sp.plots([(7, 7), (8, 7), (7, 8)], WHITE, glow=True)
    return sp


def pip(src):
    """A 16 x 16 card pip, drawn again small from the suit's shape (never resampled)."""
    def draw():
        sp = Sprite(16, 16)
        if src == 'ore':
            sp.part(sp.poly([(4, 14), (5, 6), (8, 1), (11, 6), (11, 14)]), RAMPS['amber'], 'cyl_v')
            sp.plot(7, 4, WHITE)
        elif src == 'ice':
            sp.part(sp.circle(7.5, 9.5, 5) | sp.poly([(3, 8), (7.5, 1), (12, 8)]), RAMPS['cyan'][:-1], 'sphere')
            sp.plot(5, 7, WHITE)
        elif src == 'fuel':
            sp.part(sp.rect(4, 3, 11, 14), RAMPS['green'], 'cyl_v')
            sp.part(sp.rect(3, 1, 12, 3), RAMPS['brass'], 'cyl_v')
            sp.fill(sp.poly([(8, 5), (5, 9), (8, 9), (7, 12), (10, 8), (8, 8), (9, 5)]), '#faaf5c')
        else:
            sp.part(sp.circle(7.5, 7.5, 7) & ~sp.circle(7.5, 7.5, 4), RAMPS['metal'], 'sphere')
            sp.outline()
            sp.part(sp.circle(7.5, 7.5, 3), ['#297537', '#559642', '#9bb675', '#c1f1d8'], 'inset', glow=True)
            return sp
        sp.outline()
        return sp
    return draw


def dealer_btn():
    """The dealer button: a white puck with a black D."""
    sp = Sprite(16, 16)
    sp.part(sp.circle(7.5, 7.5, 7), RAMPS['paper'], 'sphere')
    d = (sp.rect(5, 4, 6, 11) | sp.rect(5, 4, 8, 5) | sp.rect(5, 10, 8, 11) | sp.rect(9, 5, 10, 10))
    sp.fill(d, '#161612')
    sp.outline()
    return sp


# ---- Hold'em opponents (32 x 32 portraits) -------------------------------------------------------

SKIN_A = ['#7d4636', '#a8633e', '#e69d69', '#eab798']
SKIN_B = ['#522b2f', '#7d4636', '#a8633e', '#e69d69']


def face(sp, skin, cx=16):
    sp.part(sp.rect(cx - 9, 26, cx + 9, 31), RAMPS['soot'], 'rim')
    sp.part(sp.ellipse(cx - 8, 7, cx + 8, 26), skin, 'sphere')


def eyes(sp, y, color='#161612', cx=16):
    for x in (cx - 4, cx + 3):
        sp.fill(sp.rect(x, y, x + 1, y + 1), color)


def ai_harrow():
    """Old Harrow, a retired driller: hard hat, grey beard. Tight and patient."""
    sp = Sprite(N, N)
    face(sp, SKIN_A)
    sp.part(sp.ellipse(7, 15, 25, 29) & ~sp.rect(0, 0, 31, 19), ['#434137', '#a09f91', '#bfcad7'], 'rim')
    sp.part(sp.rect(12, 20, 19, 21), SKIN_A, 'flat', base=1, seam=False)
    sp.fill(sp.rect(13, 21, 18, 21), '#41322c')
    sp.part(sp.ellipse(6, 2, 26, 14) & sp.rect(0, 0, 31, 10), RAMPS['gold'][:4], 'sphere')
    sp.part(sp.rect(4, 10, 27, 11), RAMPS['brass'], 'flat', base=1)
    eyes(sp, 14)
    sp.plots([(11, 4), (12, 4)], WHITE)
    sp.outline()
    sp.part(sp.circle(16, 6, 2), RAMPS['flame'][1:], 'inset', glow=True)
    return sp


def ai_wren():
    """Wren Askew, smuggler: red hair, an eye patch. Loose, aggressive, bluffs."""
    sp = Sprite(N, N)
    sp.part(sp.ellipse(5, 3, 27, 27), ['#4a051e', '#841c1d', '#d4462b', '#fe6420'], 'sphere')
    face(sp, SKIN_A)
    sp.part(sp.poly([(7, 6), (25, 6), (25, 11), (17, 9), (8, 13)]), ['#4a051e', '#841c1d', '#d4462b', '#fe6420'], 'rim')
    sp.fill(sp.rect(11, 15, 12, 16), '#161612')
    sp.part(sp.rect(18, 14, 21, 17), RAMPS['ink'], 'flat', base=1, seam=False)
    sp.fill(sp.line([(8, 11), (24, 17)], 1) & sp.ellipse(8, 7, 24, 26), '#161612')
    sp.fill(sp.rect(13, 22, 18, 22), '#841c1d')
    sp.plot(15, 22, '#fa3a64')
    sp.outline()
    return sp


def ai_dace():
    """Officer Dace, off duty from Gatehouse Kappa: peaked cap. Tight and aggressive."""
    sp = Sprite(N, N)
    face(sp, SKIN_B)
    sp.part(sp.rect(7, 26, 25, 31), RAMPS['blue'][:3], 'rim')
    sp.part(sp.rect(14, 26, 17, 31), RAMPS['gold'][1:4], 'flat', base=1)
    sp.part(sp.ellipse(5, 2, 27, 10), RAMPS['blue'][:3], 'sphere')
    sp.part(sp.rect(7, 9, 25, 11), RAMPS['ink'], 'flat', base=1)
    sp.part(sp.rect(14, 4, 17, 7), RAMPS['gold'][1:4], 'rim', seam_color='#191954')
    eyes(sp, 15)
    sp.fill(sp.rect(12, 21, 19, 21), '#522b2f')
    sp.outline()
    return sp


def ai_bolt():
    """Bolt, a cargo loader drone: steel head, cyan eyes. Calls everything."""
    sp = Sprite(N, N)
    sp.part(sp.rect(10, 26, 21, 31), RAMPS['soot'], 'rim')
    sp.part(sp.rect(15, 1, 16, 5), RAMPS['steel'], 'rim')
    sp.part(sp.rect(5, 6, 26, 25), RAMPS['metal'], 'rim', depth=2)
    sp.part(sp.rect(8, 11, 23, 18), RAMPS['night'], 'flat', base=0)
    sp.part(sp.rect(9, 20, 22, 22), RAMPS['soot'], 'flat', base=1)
    for x in (10, 13, 16, 19):
        sp.plot(x, 21, '#a09f91')
    sp.plots([(7, 8), (8, 8)], WHITE)
    sp.outline()
    for x in (10, 18):
        sp.part(sp.rect(x, 13, x + 3, 15), RAMPS['cyan'][1:], 'inset', glow=True)
    sp.plot(15, 0, '#fa3a64', glow=True)
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
    # casino 2026-11-13: themed slot machines, keno, card pips, Hold'em
    "dv_geode": dv_geode, "dv_pick": dv_pick, "dv_cart": dv_cart, "dv_lamp": dv_lamp,
    "pl_verdance": planet(RAMPS['green'], _verdance), "pl_cinder": planet(['#1c020d', '#4a051e', '#841c1d', '#d4462b'], _cinder),
    "pl_rime": planet(RAMPS['cyan'][1:], _rime), "pl_tidewater": planet(RAMPS['blue'], _tidewater),
    "pl_ochre": planet(['#41322c', '#725244', '#ab8465', '#cfaa81'], ring=['#733126', '#c56d25', '#faaf5c']),
    "pl_moon": planet(['#2c2c25', '#434137', '#a09f91', '#bfcad7'], _moon),
    "pl_aubade": pl_aubade, "pl_wild_col": pl_wild_col,
    "sm_cutter": sm_cutter, "sm_vial": sm_vial, "sm_badge": sm_badge, "sm_stash": sm_stash,
    "kn_pick": kn_pick, "kn_drawn": kn_drawn, "kn_hit": kn_hit,
    "pip_ore": pip('ore'), "pip_ice": pip('ice'), "pip_fuel": pip('fuel'), "pip_core": pip('core'),
    "dealer_btn": dealer_btn,
    "ai_harrow": ai_harrow, "ai_wren": ai_wren, "ai_dace": ai_dace, "ai_bolt": ai_bolt,
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
