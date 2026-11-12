"""Commodity icons and company marks for docs/mockups/market.html, painted in
the friend's hand-drawn manner by the crayon kit
(.claude/skills/hand-drawn-illustration).

    python tools/art/market_art.py               # writes docs/mockups/market/*.png
    python tools/art/market_art.py --sheet OUT.png

Painted by code, not by hand. Eight commodities (the trade terminal's rows)
and five listed companies (the share board).
"""

import math
import os
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)
import crayon  # noqa: E402
from crayon import (Icon, INK, circle, ellipse, rect, roundrect, star, rotate,  # noqa: E402
                    translate, scale, lighter, darker, tone)

OUT_DIR = os.path.join(ROOT, "docs", "mockups", "market")
crayon.OUT = 128


# ---- commodities ---------------------------------------------------------------

def iron_ore(ic):
    for cx, cy, r, s in ((200, 300, 130, 2), (330, 330, 100, 5), (290, 200, 80, 8)):
        ic.drawn(star(cx, cy, r, r * 0.84, 7, jag=0.2, seed=s), (160, 72, 58), shade=0.3, edge_w=8)
        ic.dot(cx - r * 0.2, cy - r * 0.2, 9, (220, 220, 230), ink=(80, 30, 25))


def water_ice(ic):
    ic.drawn(roundrect(90, 140, 420, 420, 40), (120, 210, 250), rim_color=(230, 250, 255), edge=(30, 90, 150), edge_w=10)
    ic.line([(250, 160), (230, 250), (280, 300), (260, 400)], (40, 120, 180), 6)
    ic.line([(230, 250), (150, 290)], (40, 120, 180), 5)
    ic.line([(140, 200), (220, 180)], (240, 252, 255), 10)
    ic.line([(150, 240), (180, 232)], (240, 252, 255), 8)
    for x in (160, 256, 350):
        ic.drawn([(x - 22, 420), (x + 22, 420), (x, 480)], (120, 210, 250), highlight=0, edge=(30, 90, 150), edge_w=5)
    ic.sparkle(400, 110, 24, (220, 250, 255))


def fuel_cells(ic):
    for x, col in ((90, (250, 190, 40)), (210, (250, 200, 50)), (330, (250, 190, 40))):
        ic.drawn(roundrect(x, 150, x + 100, 440, 24), col, edge=(130, 80, 10), edge_w=8)
        ic.drawn(rect(x + 26, 120, x + 74, 150), (120, 128, 140), highlight=0, edge=INK, edge_w=5)
        ic.zap([(x + 62, 220), (x + 34, 300), (x + 64, 300), (x + 36, 380)], (255, 255, 210), 7, jag=3, seed=x)


def grain(ic):
    ic.drawn(roundrect(120, 230, 392, 470, 60), (200, 160, 90), edge=(110, 70, 30), edge_w=9)   # the sack
    ic.line([(170, 260), (342, 260)], (130, 90, 40), 8)
    for a in (-0.5, -0.2, 0.15, 0.45):
        x0, y0 = 256, 260
        x1, y1 = x0 + 220 * math.sin(a), y0 - 220 * math.cos(a)
        ic.line([(x0, y0), (x1, y1)], (130, 160, 60), 7)
        for k in range(4):
            t = 0.55 + 0.12 * k
            px, py = x0 + (x1 - x0) * t, y0 + (y1 - y0) * t
            ic.drawn(ellipse(px, py, 14, 26, 16, a), (250, 210, 80), highlight=0, shade=0.2, edge=(150, 100, 20), edge_w=4)


def alloy(ic):
    def ingot(dx, dy, col):
        top = translate([(130, 250), (330, 250), (390, 200), (190, 200)], dx, dy)
        front = translate([(130, 250), (330, 250), (310, 330), (110, 330)], dx, dy)
        side = translate([(330, 250), (390, 200), (370, 280), (310, 330)], dx, dy)
        ic.drawn(front, col, edge=tone(col), edge_w=7)
        ic.drawn(side, darker(col, 0.25), highlight=0, edge=tone(col), edge_w=7)
        ic.drawn(top, lighter(col, 0.25), shade=0.1, edge=tone(col), edge_w=7)
    ingot(-10, 120, (170, 186, 210))
    ingot(30, 20, (190, 206, 228))
    ic.sparkle(120, 140, 26)


def medicine(ic):
    ic.drawn(roundrect(110, 110, 402, 440, 50), (240, 244, 248), rim_color=(255, 255, 255), edge=(80, 96, 120), edge_w=10)
    ic.drawn(rect(110, 110, 402, 170), (60, 180, 120), highlight=0.1, edge=(20, 90, 50), edge_w=7)
    cross = [(226, 220), (286, 220), (286, 270), (336, 270), (336, 330), (286, 330), (286, 380), (226, 380), (226, 330), (176, 330), (176, 270), (226, 270)]
    ic.drawn(cross, (230, 60, 60), edge=(120, 20, 20), edge_w=7)


def electronics(ic):
    ic.drawn(rect(130, 130, 382, 382), (40, 150, 90), edge=(10, 60, 30), edge_w=9)
    for k in range(5):
        v = 160 + k * 48
        for a, b in (((v, 130), (v, 80)), ((v, 382), (v, 432)), ((130, v), (80, v)), ((382, v), (432, v))):
            ic.line([a, b], (220, 200, 120), 9, ink=INK, ink_w=5)
    ic.drawn(rect(196, 196, 316, 316), (40, 44, 56), highlight=0.1, edge=INK, edge_w=6)
    ic.line([(160, 170), (196, 210)], (250, 220, 120), 5)
    ic.dot(256, 256, 16, (120, 255, 200), ink=INK)


def spice(ic):
    jar = [(170, 150), (342, 150), (380, 220), (370, 440), (142, 440), (132, 220)]
    ic.drawn(jar, (180, 60, 140), rim_color=(250, 170, 230), edge=(80, 20, 70), edge_w=10)
    ic.drawn(rect(180, 100, 332, 154), (230, 180, 60), highlight=0, edge=(120, 80, 10), edge_w=7)
    ic.drawn(roundrect(170, 270, 342, 370, 16), (250, 240, 210), highlight=0, shade=0, edge=(80, 20, 70), edge_w=5)
    ic.dot(256, 320, 22, (220, 90, 40), ink=(100, 30, 10))
    for x, y in ((420, 140), (100, 200), (430, 330)):
        ic.sparkle(x, y, 20, (255, 170, 230))


# ---- company marks --------------------------------------------------------------

def hexplate(ic, col, inner):
    h = circle(256, 256, 220, 6, start=math.pi / 6)
    ic.drawn(h, col, edge=tone(col), edge_w=10)
    ic.drawn(scale(h, 0.8), inner, highlight=0.1, shade=0.15, edge=tone(inner), edge_w=6)


def kessler(ic):
    """Kessler Deep Mining: a pick across a hex."""
    hexplate(ic, (230, 140, 40), (70, 50, 40))
    ic.line([(150, 400), (330, 150)], (235, 175, 90), 34, ink=INK, ink_w=10)
    head = [(170, 150), (250, 100), (340, 100), (420, 150), (450, 230), (390, 170), (320, 150), (250, 160), (180, 200)]
    ic.drawn(head, (210, 216, 226), rim_color=(250, 250, 255), edge=INK, edge_w=7)
    ic.sparkle(150, 160, 22)


def halyard(ic):
    """Halyard Shipyards: a hull on a slipway under a gantry."""
    hexplate(ic, (70, 110, 180), (30, 44, 70))
    ic.line([(140, 130), (372, 130)], (240, 190, 50), 14, ink=INK, ink_w=6)
    ic.line([(160, 130), (160, 360)], (240, 190, 50), 12, ink=INK, ink_w=6)
    ic.line([(352, 130), (352, 360)], (240, 190, 50), 12, ink=INK, ink_w=6)
    hull = [(160, 260), (330, 230), (390, 280), (330, 330), (160, 310)]
    ic.drawn(hull, (220, 226, 236), rim_color=(255, 255, 255), edge=(40, 50, 70), edge_w=7)
    ic.line([(256, 130), (256, 236)], (200, 200, 210), 5)


def greenbelt(ic):
    """Greenbelt Co-op: a sprout in a dome."""
    hexplate(ic, (80, 170, 70), (230, 240, 210))
    ic.drawn(rect(150, 330, 362, 370), (140, 100, 60), highlight=0, edge=(70, 40, 20), edge_w=6)
    ic.line([(256, 340), (256, 200)], (60, 140, 50), 12)
    ic.drawn(ellipse(200, 220, 60, 30, 24, -0.5), (90, 200, 70), edge=(30, 90, 30), edge_w=6)
    ic.drawn(ellipse(312, 190, 64, 32, 24, 0.5), (90, 200, 70), edge=(30, 90, 30), edge_w=6)
    dome = [(150 + 106 - 106 * math.cos(t), 330 - 170 * math.sin(t)) for t in [i * math.pi / 30 for i in range(31)]]
    ic.line(dome, (120, 200, 240), 7)


def ventra(ic):
    """Ventra Power: a bolt in a ring."""
    ic.drawn(circle(256, 256, 220), (240, 110, 40), edge=(120, 40, 10), edge_w=10)
    ic.drawn(circle(256, 256, 160), (60, 30, 30), highlight=0.05, edge=(30, 10, 10), edge_w=6)
    bolt = [(290, 100), (180, 280), (250, 280), (210, 420), (340, 220), (270, 220), (320, 100)]
    ic.drawn(bolt, (255, 220, 60), rim_color=(255, 250, 200), edge=(140, 80, 0), edge_w=8)
    for a in (0.3, 2.2, 4.0, 5.4):
        ic.sparkle(256 + 200 * math.cos(a), 256 + 200 * math.sin(a), 18, (255, 230, 120))


def lumen(ic):
    """Lumen Labs: a glowing flask."""
    ic.glow((256, 330, 110), (150, 120, 255), radius=30, strength=0.5)
    flask = [(220, 80), (292, 80), (292, 220), (400, 420), (112, 420), (220, 220)]
    ic.drawn(flask, (220, 230, 245), rim_color=(255, 255, 255), edge=(60, 70, 100), edge_w=9)
    liquid = [(176, 320), (336, 320), (392, 412), (120, 412)]
    ic.drawn(liquid, (150, 110, 250), highlight=0.2, edge=None)
    for x, y, r in ((220, 360, 14), (290, 380, 10), (250, 290, 8)):
        ic.dot(x, y, r, (230, 220, 255))
    ic.drawn(rect(206, 50, 306, 86), (120, 128, 140), highlight=0, edge=INK, edge_w=6)


ICONS = {
    "iron_ore": (iron_ore, 1), "water_ice": (water_ice, 2), "fuel_cells": (fuel_cells, 3), "grain": (grain, 4),
    "alloy": (alloy, 5), "medicine": (medicine, 6), "electronics": (electronics, 7), "spice": (spice, 8),
    "co_kessler": (kessler, 20), "co_halyard": (halyard, 21), "co_greenbelt": (greenbelt, 22),
    "co_ventra": (ventra, 23), "co_lumen": (lumen, 24),
}


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    paths = []
    for name, (fn, seed) in ICONS.items():
        ic = Icon(seed)
        fn(ic)
        p = os.path.join(OUT_DIR, name + ".png")
        ic.save(p)
        paths.append(p)
    print("wrote", len(paths), "icons to", OUT_DIR)
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1]
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), out, *paths, "--ref", "laser drill", "--ref", "plating"], check=True)


if __name__ == "__main__":
    main()
