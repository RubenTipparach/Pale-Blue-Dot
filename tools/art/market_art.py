"""Commodity icons and company marks for docs/mockups/market.html: pixel art drawn in code with the
balatro-pixel-art kit (.claude/skills/balatro-pixel-art), the solitaire collection's style.

    python tools/art/market_art.py                 # writes docs/mockups/market/*.png
    python tools/art/market_art.py --sheet OUT.png # and a contact sheet beside the kit's references

Eight commodities (the trade terminal's rows) and five listed companies (the share board), each
28 x 28 at native size: the page shows them at 1x in its tables and contracts, nearest neighbour.
"""

import math
import os
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts")
sys.path.insert(0, KIT)
from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402
from PIL import Image  # noqa: E402

OUT_DIR = os.path.join(ROOT, "docs", "mockups", "market")
N = 28
WHITE = '#fffefe'
WARM = '#faaf5c'


def lumpy(sp, cx, cy, rx, ry, n=9, jag=0.16, seed=1):
    pts = []
    for i in range(n):
        a = 2 * math.pi * i / n + 0.3
        k = 1 + jag * math.sin(seed * 12.9898 + i * 78.233)
        pts.append((cx + rx * k * math.cos(a), cy + ry * k * math.sin(a)))
    return sp.poly(pts)


def spark(sp, x, y, color=WARM):
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], color, glow=True)
    sp.plot(x, y, WHITE, glow=True)


def hexagon(sp, cx, cy, r, turn=30):
    return sp.poly([(cx + r * math.cos(math.radians(a)), cy + r * math.sin(math.radians(a)))
                    for a in range(turn, turn + 360, 60)])


# ---- commodities ------------------------------------------------------------------------------

def iron_ore():
    sp = Sprite(N, N)
    for cx, cy, rx, ry, s in ((17, 12, 8, 7, 8), (9, 18, 8, 7, 2), (19, 20, 7, 6, 5)):
        sp.part(lumpy(sp, cx, cy, rx, ry, seed=s), RAMPS['rust'], 'sphere')
        sp.part(sp.rect(cx - 3, cy - 3, cx - 2, cy - 2), RAMPS['metal'][2:], 'flat', base=2, seam_color='#2e0d06')
    sp.outline()
    return sp


def water_ice():
    sp = Sprite(N, N)
    sp.part(sp.poly([(3, 9), (10, 3), (25, 3), (18, 9)]), RAMPS['cyan'], 'flat', base=4)
    sp.part(sp.poly([(18, 9), (25, 3), (25, 18), (18, 25)]), RAMPS['cyan'], 'flat', base=1)
    sp.part(sp.rect(3, 9, 18, 25), RAMPS['cyan'], 'rim', base=3)
    sp.fill(sp.line([(11, 10), (9, 15), (13, 18), (11, 24)], 1), '#1096c3')
    sp.plots([(5, 11), (5, 12), (6, 11)], WHITE)
    sp.outline()
    spark(sp, 25, 23, '#74effe')
    return sp


def fuel_cells():
    sp = Sprite(N, N)
    for x0, ramp in ((3, RAMPS['green']), (14, RAMPS['gold'][:4])):
        sp.part(sp.rect(x0, 7, x0 + 10, 25), ramp, 'cyl_v')
        sp.part(sp.rect(x0 + 3, 4, x0 + 7, 6), RAMPS['steel'], 'cyl_v')
        sp.fill(sp.rect(x0, 11, x0 + 10, 11), ramp[0])
        sp.fill(sp.rect(x0, 21, x0 + 10, 21), ramp[0])
    sp.part(sp.poly([(21, 12), (17, 17), (20, 17), (18, 21), (23, 15), (20, 15), (22, 12)]), ['#2e0d06', '#733126'], 'flat', base=1, seam=False)
    sp.plots([(4, 8), (15, 8)], WHITE)
    sp.outline()
    return sp


def grain():
    """Grain: a tied sack with ears of wheat standing out of its neck."""
    sp = Sprite(N, N)
    for x, top in ((10, 1), (14, 0), (18, 2)):
        sp.part(sp.rect(x, top + 5, x, 10), RAMPS['olive'], 'flat', base=2, seam=False)
        sp.part(sp.ellipse(x - 1, top, x + 1, top + 5), ['#c56d25', '#fb902f', '#faaf5c'], 'rim', seam=False)
    sack = sp.poly([(6, 10), (21, 10), (24, 18), (23, 25), (4, 25), (3, 18)])
    sp.part(sack, RAMPS['sand'], 'sphere')
    sp.part(sp.rect(9, 8, 18, 10), RAMPS['wood'], 'rim')
    sp.part(sp.rect(5, 17, 22, 19) & sack, RAMPS['wood'], 'flat', base=1, seam=False)
    sp.plots([(7, 12), (6, 13)], WHITE)
    sp.outline()
    return sp


def alloy():
    sp = Sprite(N, N)

    def ingot(x, y, w):
        top = sp.poly([(x + 2, y), (x + w - 2, y), (x + w, y + 3), (x, y + 3)])
        sp.part(top, RAMPS['metal'], 'flat', base=4)
        sp.part(sp.rect(x, y + 4, x + w, y + 8), RAMPS['metal'], 'cyl_h')
    ingot(2, 15, 12)
    ingot(14, 15, 12)
    ingot(8, 7, 12)
    sp.plots([(11, 8), (12, 8)], WHITE)
    sp.outline()
    spark(sp, 24, 5, '#d2f8ff')
    return sp


def medicine():
    sp = Sprite(N, N)
    sp.part(sp.rect(10, 4, 17, 7) & ~sp.rect(12, 6, 15, 7), RAMPS['steel'], 'rim')
    sp.part(sp.rect(2, 7, 25, 25), RAMPS['paper'] + [WHITE], 'rim', base=2, depth=2)
    sp.part(sp.rect(11, 10, 16, 22) | sp.rect(7, 13, 20, 18), RAMPS['red'], 'rim', seam_color='#92a3b2')
    sp.outline()
    return sp


def electronics():
    sp = Sprite(N, N)
    for i in range(4):
        x = 7 + i * 4
        for r in (sp.rect(x, 2, x + 1, 5), sp.rect(x, 22, x + 1, 25), sp.rect(2, x, 5, x + 1), sp.rect(22, x, 25, x + 1)):
            sp.part(r, RAMPS['gold'][1:4], 'flat', base=2, seam=False)
    sp.part(sp.rect(5, 5, 22, 22), RAMPS['green'], 'rim')
    sp.part(sp.rect(9, 9, 18, 18), RAMPS['soot'], 'rim')
    sp.plots([(6, 6), (7, 6)], '#9bb675')
    sp.plot(10, 10, WHITE)
    sp.outline()
    sp.plots([(16, 16), (17, 16)], '#74effe', glow=True)
    return sp


def spice():
    sp = Sprite(N, N)
    jar = sp.rect(5, 11, 22, 25) & ~sp.rect(5, 25, 5, 25) & ~sp.rect(22, 25, 22, 25)
    sp.part(jar, RAMPS['violet'][:4], 'cyl_v')
    sp.part(sp.rect(5, 15, 22, 20), RAMPS['bone'], 'rim', seam_color='#230c44')
    sp.part(sp.ellipse(6, 4, 21, 13), ['#681712', '#c56d25', '#fe6420', '#fb902f'], 'sphere')
    sp.plots([(9, 6), (10, 5)], WHITE)
    sp.fill(sp.rect(11, 17, 16, 18), '#7d2dbd')
    sp.outline()
    spark(sp, 24, 4, '#c34988')
    return sp


# ---- company marks ----------------------------------------------------------------------------

def plate(sp, ramp):
    sp.part(hexagon(sp, 14, 14, 13), ramp, 'rim', depth=2)
    sp.part(hexagon(sp, 14, 14, 10), [ramp[0]] + ramp[:-1], 'flat', base=1)


def kessler():
    """Kessler Deep Mining: a pick across a hex."""
    sp = Sprite(N, N)
    plate(sp, RAMPS['metal'])
    sp.part(sp.line([(8, 22), (19, 9)], 3), RAMPS['wood'], 'rim')
    head = sp.poly([(10, 5), (17, 6), (22, 9), (26, 14), (21, 12), (16, 10), (11, 8)])
    sp.part(head, RAMPS['gold'][:4], 'rim')
    sp.plots([(12, 6), (13, 6)], WHITE)
    sp.outline()
    return sp


def halyard():
    """Halyard Shipyards: a hull on a slipway under a gantry."""
    sp = Sprite(N, N)
    sp.part(sp.rect(3, 3, 5, 24) | sp.rect(22, 3, 24, 24) | sp.rect(3, 3, 24, 5), RAMPS['gold'][:4], 'rim')
    sp.part(sp.rect(13, 6, 14, 10), RAMPS['steel'], 'flat', base=1)
    hull = sp.poly([(6, 12), (19, 12), (22, 15), (19, 18), (6, 18)])
    sp.part(hull, RAMPS['blue'], 'cyl_h')
    sp.part(sp.rect(9, 10, 13, 12), RAMPS['paper'], 'rim')
    sp.part(sp.rect(1, 22, 26, 25), RAMPS['steel'], 'cyl_h')
    sp.plots([(8, 13), (9, 13)], '#d2f8ff')
    sp.outline()
    return sp


def greenbelt():
    """Greenbelt Co-op: a sprout under a dome."""
    sp = Sprite(N, N)
    sp.part(sp.rect(3, 21, 24, 25), RAMPS['wood'], 'cyl_h')
    sp.part(sp.ellipse(4, 3, 23, 38) & sp.rect(0, 0, N, 20), RAMPS['glass'], 'sphere', base=2)
    sp.part(sp.rect(13, 12, 14, 20), RAMPS['green'], 'flat', base=1, seam=False)
    sp.part(sp.ellipse(6, 9, 13, 14), RAMPS['green'], 'sphere', seam_color='#003227')
    sp.part(sp.ellipse(14, 7, 21, 12), RAMPS['green'], 'sphere', seam_color='#003227')
    sp.plots([(8, 7), (8, 8)], WHITE)
    sp.outline()
    return sp


def ventra():
    """Ventra Power: a bolt in a ring."""
    sp = Sprite(N, N)
    sp.part(sp.circle(14, 14, 13) & ~sp.circle(14, 14, 10), RAMPS['cyan'], 'sphere')
    sp.part(sp.poly([(17, 2), (6, 16), (13, 16), (10, 26), (22, 11), (15, 11), (19, 2)]), RAMPS['gold'][:4], 'rim', seam_color='#0c2f44')
    sp.plots([(16, 4), (15, 5)], WHITE)
    sp.outline()
    return sp


def lumen():
    """Lumen Labs: a glowing flask."""
    sp = Sprite(N, N)
    flask = sp.poly([(11, 9), (16, 9), (24, 23), (22, 26), (5, 26), (3, 23)]) | sp.rect(11, 3, 16, 9)
    sp.part(flask, RAMPS['paper'], 'rim')
    sp.part(sp.rect(10, 2, 17, 4), RAMPS['wood'], 'rim')
    sp.outline()
    liquid = flask & sp.rect(0, 16, N, N) & ~sp.rect(0, 26, N, N)
    liquid &= ~(sp.empty() | (flask & ~sp.poly([(10, 16), (17, 16), (23, 24), (21, 25), (6, 25), (4, 24)])))
    sp.part(liquid, ['#05582d', '#559642', '#84e3b5', '#c1f1d8'], 'inset', glow=True)
    sp.plots([(11, 21), (15, 19), (18, 22)], WHITE, glow=True)
    sp.plots([(12, 5), (12, 6), (12, 7)], WHITE)
    spark(sp, 23, 6, '#84e3b5')
    return sp


SPRITES = {
    "iron_ore": iron_ore, "water_ice": water_ice, "fuel_cells": fuel_cells, "grain": grain,
    "alloy": alloy, "medicine": medicine, "electronics": electronics, "spice": spice,
    "co_kessler": kessler, "co_halyard": halyard, "co_greenbelt": greenbelt,
    "co_ventra": ventra, "co_lumen": lumen,
}

REFS = ["emp_burst", "drop_tanks", "armour_plating", "afterburner_kit", "all_in", "ejection_seat"]


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    items = []
    for name, fn in SPRITES.items():
        img = fn().image()
        path = os.path.join(OUT_DIR, name + ".png")
        save(img, path, "tools/art/market_art.py (balatro-pixel-art kit)")
        items.append((name, img))
    print("wrote", len(items), "sprites to", OUT_DIR)
    if "--sheet" in sys.argv:
        i = sys.argv.index("--sheet")
        out = sys.argv[i + 1] if len(sys.argv) > i + 1 else "market_sheet.png"
        ref_dir = os.path.join(os.path.dirname(KIT), "references")
        refs = [("ref " + n, Image.open(os.path.join(ref_dir, n + ".png")).convert("RGBA")) for n in REFS]
        sheet(items + refs, scale=4, cols=7, cell=(224, 224)).save(out)
        print("sheet", out)


if __name__ == "__main__":
    main()
