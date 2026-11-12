"""Station portraits for docs/mockups/stations.html, painted in the friend's
hand-drawn manner by the crayon kit (.claude/skills/hand-drawn-illustration).

    python tools/art/stations_art.py            # writes docs/mockups/stations/*.png
    python tools/art/stations_art.py --sheet    # and a contact sheet beside the friend's icons

Painted by code, not by hand: the kit copies the friend's method stroke by
stroke (block, hatch, crescents, dabs, tone edge, energy last). Each portrait is
one station type from the stations mockup, light from the upper left.
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

OUT_DIR = os.path.join(ROOT, "docs", "mockups", "stations")
crayon.OUT = 256  # portraits are shown bigger than an upgrade icon

STEEL = (120, 146, 186)
NAVY = (40, 52, 82)
AMBER = (255, 196, 70)
WARM = (255, 220, 130)
CYAN = (70, 220, 235)
RUST = (170, 92, 52)
ROCK = (138, 104, 84)


def ell_ring(cx, cy, rx, ry, k, n=64, rot=0.0):
    """An elliptical ring as one polygon (a ring seen at an angle)."""
    a = ellipse(cx, cy, rx, ry, n, rot)
    b = ellipse(cx, cy, rx * k, ry * k, n, rot)
    return a + [a[0], b[0]] + b[::-1] + [b[0]]


def windows(ic, pts, color=WARM, r=6):
    for x, y in pts:
        ic.dot(x, y, r + 3, tone(color, 0.7))
        ic.dot(x, y, r, color)


def trade_ring(ic):
    """The orbital trade hub: a spinning ring on spokes round a still hub."""
    ic.glow((256, 270, 150), (90, 150, 255), radius=30, strength=0.25)
    # the far half of the ring first, then hub and spokes, then the near half on top
    ring = ell_ring(256, 280, 236, 132, 0.72)
    ic.drawn(ring, STEEL, edge_w=9)
    for a in (0.5, 2.1, 3.7, 5.3):
        x, y = 256 + 186 * math.cos(a), 280 + 104 * math.sin(a)
        ic.line([(256, 272), (x, y)], (95, 112, 140), 12, ink=tone(STEEL), ink_w=8)
    hub = roundrect(206, 150, 306, 360, 34)
    ic.drawn(hub, (150, 170, 205), edge_w=8)
    ic.drawn(roundrect(228, 100, 284, 156, 12), (200, 90, 70), highlight=0, edge_w=6)   # the docking collar
    ic.line([(256, 100), (256, 44)], (210, 214, 220), 6, ink=INK, ink_w=6)                # mast
    ic.sparkle(256, 38, 24, (255, 90, 80))
    # the near half of the ring, drawn over the hub
    near = [p for p in ellipse(256, 280, 236, 132, 64)][0:33]
    inner = [p for p in ellipse(256, 280, 170, 95, 64)][0:33]
    band = near + inner[::-1]
    ic.drawn(band, (132, 158, 200), shade=0.3, edge_w=9)
    windows(ic, [(256 + 203 * math.cos(t), 280 + 113 * math.sin(t)) for t in [0.3 + 0.32 * i for i in range(9)]], r=7)
    for x, y in ((238, 196), (274, 196), (238, 240), (274, 240), (256, 320)):
        ic.dot(x, y, 6, AMBER, ink=INK)
    ic.sparkle(70, 120, 18)
    ic.sparkle(452, 400, 14, (160, 230, 255))


def lumpy(cx, cy, r, n=22, jag=0.16, seed=3):
    p = star(cx, cy, r, r * 0.93, n // 2, jag=jag, seed=seed)
    return p


def mining_outpost(ic):
    """A mining outpost bolted onto a rock: a derrick, a dome, ore spilling out."""
    rock = lumpy(256, 320, 190, 26, 0.12, seed=11)
    ic.drawn(rock, ROCK, shade=0.3, edge_w=10)
    for x, y, r in ((180, 360, 28), (310, 400, 22), (350, 300, 18), (200, 280, 14)):
        ic.drawn(circle(x, y, r, 20), darker(ROCK, 0.25), highlight=0, shade=0.4, edge=tone(ROCK, 0.6), edge_w=5)
    # the derrick, a lattice tower
    leg = [(290, 210), (318, 210), (300, 60), (296, 60)]
    ic.drawn([(270, 214), (300, 60), (306, 60), (338, 214)], (230, 180, 60), highlight=0.1, edge=INK, edge_w=7)
    for y in (180, 140, 100):
        ic.line([(278 + (214 - y) * 0.03, y), (330 - (214 - y) * 0.03, y)], (60, 50, 40), 5)
    ic.line([(303, 60), (303, 250)], (80, 80, 90), 6)
    ic.sparkle(303, 48, 18, (255, 100, 80))
    # the dome with lit windows
    dome = [(p[0], p[1]) for p in circle(170, 214, 70, 40) if p[1] <= 216]
    ic.drawn(dome, (110, 200, 220), rim_color=(200, 250, 255), edge_w=8)
    windows(ic, [(140, 196), (170, 186), (200, 196)], r=6)
    ic.drawn(rect(96, 210, 246, 236), (90, 96, 110), highlight=0, edge=INK, edge_w=6)
    # ore crystals breaking out of the rock
    for cx, cy, h, rot, col in ((390, 250, 70, 0.4, (255, 140, 40)), (420, 300, 50, 0.8, (255, 170, 60)), (120, 380, 46, -0.5, CYAN)):
        cr = rotate([(cx - 14, cy + 10), (cx - 12, cy - h), (cx, cy - h - 18), (cx + 12, cy - h), (cx + 14, cy + 10)], rot, (cx, cy))
        ic.drawn(cr, col, edge_w=6)
    for x, y in ((440, 210), (460, 250), (90, 330)):
        ic.dot(x, y, 7, (255, 170, 60), ink=tone((255, 150, 40)))


def refinery(ic):
    """A refinery: stacked tanks and a cracking tower with a gas flare."""
    ic.glow((256, 96, 90), (255, 130, 40), radius=26, strength=0.45)
    # the long spine
    ic.drawn(rect(60, 320, 460, 364), (110, 118, 130), highlight=0.1, edge=INK, edge_w=7)
    for cx, cy, r, col in ((120, 400, 76, (230, 230, 220)), (280, 410, 84, (230, 230, 220)), (430, 390, 56, (200, 80, 60))):
        ic.drawn(circle(cx, cy, r, 40), col, rim_color=lighter(col, 0.6), edge_w=8)
        ic.line([(cx - r * 0.6, cy - r * 0.15), (cx + r * 0.6, cy - r * 0.15)], darker(col, 0.25), 6)
    # the cracking tower
    tower = [(222, 320), (218, 150), (296, 150), (292, 320)]
    ic.drawn(tower, (230, 186, 64), edge=INK, edge_w=8)
    for y in (180, 220, 260, 300):
        ic.line([(220, y), (294, y)], (120, 80, 30), 6)
    ic.line([(296, 190), (370, 190), (370, 320)], (150, 156, 170), 10, ink=INK, ink_w=6)
    # the flare: a little inferno on the stack
    flame = star(257, 100, 84, 46, 8, jag=0.3, seed=4)
    ic.drawn(flame, (255, 120, 30), rim_color=(255, 230, 90), edge=(170, 50, 10), edge_w=7)
    ic.drawn(scale(flame, 0.45, (257, 112)), (255, 230, 100), highlight=0, shade=0, edge=None)
    windows(ic, [(90, 342), (150, 342), (340, 342), (400, 342)], r=6)
    for x, y, a in ((350, 70, -20), (160, 80, 200)):
        ic.speed(x, y, 40, a, (255, 200, 90), 6)


def customs_post(ic):
    """A military customs post: an armoured hex, red stripe, guns and a beacon."""
    hexa = circle(256, 270, 190, 6, start=math.pi / 6)
    ic.drawn(hexa, (88, 104, 140), edge=(20, 26, 46), edge_w=10)
    inner = scale(hexa, 0.78, (256, 270))
    ic.drawn(inner, (44, 52, 74), highlight=0, shade=0.12, edge=(18, 20, 32), edge_w=6)
    for p in scale(hexa, 0.89, (256, 270)):
        ic.dot(p[0], p[1], 7, (170, 178, 196), ink=(16, 18, 26))
    face = scale(hexa, 0.6, (256, 270))
    ic.drawn(face, (70, 98, 150), edge=(25, 35, 60), edge_w=6)
    # the stripe and the customs gate (a lit slot)
    ic.drawn(rect(150, 248, 362, 282), (220, 60, 50), highlight=0.1, edge=(110, 20, 20), edge_w=5)
    ic.drawn(roundrect(216, 300, 296, 350, 10), (255, 214, 110), highlight=0, shade=0.2, edge=(120, 80, 20), edge_w=6)
    # turrets
    for x, a in ((120, -0.5), (392, 0.5)):
        ic.drawn(circle(x, 150, 34, 24), (130, 140, 160), edge=INK, edge_w=6)
        ic.line([(x, 150), (x + 70 * math.sin(a), 150 - 70 * math.cos(a))], (60, 64, 76), 14, ink=INK, ink_w=6)
    ic.line([(256, 80), (256, 30)], (210, 214, 220), 6, ink=INK, ink_w=6)
    ic.sparkle(256, 30, 26, (255, 80, 70))
    ic.speed(200, 22, 30, 200, (255, 120, 110), 5)
    ic.speed(312, 22, 30, -20, (255, 120, 110), 5)


def smuggler_den(ic):
    """A smuggler den in a hollow rock: a neon sign, a lit cave mouth, crates."""
    rock = lumpy(256, 280, 200, 28, 0.16, seed=7)
    ic.drawn(rock, (96, 82, 100), shade=0.3, edge_w=10)
    mouth = ellipse(250, 300, 110, 74, 40)
    ic.glow((250, 300, 80), (255, 120, 220), radius=28, strength=0.6)
    ic.drawn(mouth, (40, 24, 50), highlight=0, shade=0, edge=(20, 10, 30), edge_w=8)
    ic.drawn(scale(mouth, 0.7, (250, 312)), (255, 150, 90), highlight=0.2, shade=0.25, edge=None)
    # crates in the mouth
    for x, y, c in ((200, 320, (190, 130, 70)), (246, 330, (80, 160, 90)), (226, 300, (190, 130, 70))):
        ic.drawn(rect(x, y, x + 40, y + 34), c, highlight=0, edge=INK, edge_w=5)
    # the neon sign above, a crooked bar of magenta
    sign = rotate(roundrect(150, 150, 360, 200, 18), -0.12, (255, 175))
    ic.drawn(sign, (40, 20, 50), highlight=0, shade=0, edge=(255, 70, 200), edge_w=8)
    ic.line([(178, 186), (200, 162), (222, 186)], (255, 90, 210), 7)    # a neon zigzag
    ic.line([(236, 180), (262, 164), (290, 172), (320, 156)], (90, 255, 230), 7)
    ic.sparkle(388, 120, 22, (255, 120, 220))
    ic.sparkle(112, 210, 14, (120, 255, 230))
    ic.line([(380, 260), (460, 230)], (120, 120, 130), 6, ink=INK, ink_w=5)   # a mooring line
    ic.drawn(rotate(roundrect(440, 200, 500, 236, 10), -0.35, (470, 218)), (200, 70, 60), edge=INK, edge_w=6)


def research_station(ic):
    """A research station: a sphere lab, solar wings and a big dish."""
    for x0, x1 in ((24, 170), (342, 488)):
        wing = rect(x0, 236, x1, 296)
        ic.drawn(wing, (40, 120, 200), highlight=0.12, edge=(15, 40, 80), edge_w=7)
        for k in range(1, 4):
            x = x0 + (x1 - x0) * k / 4
            ic.line([(x, 240), (x, 292)], (140, 210, 255), 4)
        ic.line([(x0, 266), (x1, 266)], (140, 210, 255), 4)
    ic.line([(170, 266), (342, 266)], (150, 156, 170), 12, ink=INK, ink_w=6)
    ic.glow((256, 270, 110), (120, 255, 170), radius=30, strength=0.4)
    sph = circle(256, 270, 104, 48)
    ic.drawn(sph, (230, 236, 240), rim_color=(255, 255, 255), edge=(60, 80, 100), edge_w=9)
    ic.line([(160, 250), (352, 250)], (90, 110, 130), 6)
    windows(ic, [(212, 230), (246, 222), (282, 224), (316, 234)], (140, 255, 190), 7)
    # the dish
    dish = [p for p in ellipse(316, 130, 74, 38, 40, rot=-0.5)]
    ic.line([(290, 180), (318, 136)], (150, 156, 170), 10, ink=INK, ink_w=6)
    ic.drawn(dish, (220, 224, 230), edge=(60, 70, 90), edge_w=7)
    ic.line([(316, 130), (350, 100)], (80, 80, 90), 5)
    for r in (40, 64, 88):
        pts = [(350 + r * math.cos(a), 100 + r * math.sin(a)) for a in [-1.3 + 0.1 * i for i in range(9)]]
        ic.line(pts, (120, 255, 190), 5)
    ic.sparkle(120, 120, 20, (140, 255, 200))


ICONS = {
    "trade_ring": (trade_ring, 3),
    "mining_outpost": (mining_outpost, 5),
    "refinery": (refinery, 8),
    "customs_post": (customs_post, 9),
    "smuggler_den": (smuggler_den, 12),
    "research_station": (research_station, 14),
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
        print("wrote", p)
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1] if len(sys.argv) > sys.argv.index("--sheet") + 1 else "stations_sheet.png"
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), out, *paths, "--ref", "plating", "--ref", "power output"], check=True)


if __name__ == "__main__":
    main()
