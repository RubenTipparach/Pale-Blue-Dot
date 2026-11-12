"""Icons for docs/mockups/minigames.html, painted in the friend's hand-drawn
manner by the crayon kit (.claude/skills/hand-drawn-illustration).

    python tools/art/minigames_art.py            # writes docs/mockups/minigames/*.png
    python tools/art/minigames_art.py --sheet OUT.png

Painted by code, not by hand. Groups: the ore sorter's ores and drill, the
Reactor Roll's reel symbols (also the four suits of Orbit 21), dice, chips, a
card back, the race ships, and the job board's badges.
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
                    translate, scale, lighter, darker, tone, mix)

OUT_DIR = os.path.join(ROOT, "docs", "mockups", "minigames")
crayon.OUT = 160


def crystal(ic, cx, cy, h, w, rot, col, edge_w=7):
    pts = rotate([(cx - w, cy + h * 0.2), (cx - w * 0.9, cy - h * 0.6), (cx, cy - h), (cx + w * 0.9, cy - h * 0.6), (cx + w, cy + h * 0.2), (cx, cy + h * 0.35)], rot, (cx, cy))
    ic.drawn(pts, col, rim_color=lighter(col, 0.55), edge_w=edge_w)
    ic.line(rotate([(cx, cy - h * 0.9), (cx, cy + h * 0.3)], rot, (cx, cy)), lighter(col, 0.5), 5)


def chunk(ic, col, seed, fleck=None, r=170):
    body = star(256, 270, r, r * 0.86, 8, jag=0.18, seed=seed)
    ic.drawn(body, col, shade=0.28, edge_w=10)
    if fleck:
        for x, y, s in ((200, 230, 22), (300, 300, 26), (250, 340, 16), (320, 210, 18), (190, 320, 14)):
            ic.drawn(star(x, y, s, s * 0.5, 4, jag=0.2, seed=x), fleck, highlight=0.1, shade=0, edge=tone(fleck), edge_w=5)


def ore_iron(ic):
    chunk(ic, (150, 70, 60), 2, (210, 210, 220))


def ore_copper(ic):
    chunk(ic, (215, 110, 50), 4, (90, 220, 170))


def ore_gold(ic):
    chunk(ic, (110, 96, 90), 6, (255, 210, 50))
    ic.sparkle(380, 130, 30)


def ore_ice(ic):
    crystal(ic, 200, 330, 200, 60, -0.25, (90, 210, 250))
    crystal(ic, 320, 340, 160, 52, 0.3, (130, 230, 255))
    ic.sparkle(120, 120, 26, (210, 250, 255))


def slag(ic):
    chunk(ic, (100, 100, 104), 9, None, 150)
    ic.line([(200, 230), (240, 270), (220, 310)], (60, 60, 66), 6)


def drill(ic):
    ic.glow((256, 420, 70), (255, 150, 40), radius=26, strength=0.6)
    ic.drawn(roundrect(170, 40, 342, 170, 20), (230, 180, 50), edge=INK, edge_w=8)
    ic.drawn(rect(206, 170, 306, 220), (120, 128, 140), highlight=0, edge=INK, edge_w=7)
    bit = [(200, 220), (312, 220), (256, 450)]
    ic.drawn(bit, (190, 200, 214), rim_color=(250, 250, 255), edge=(50, 60, 80), edge_w=8)
    for y in (260, 310, 360):
        k = (y - 220) / 230
        ic.line([(200 + 56 * k, y), (312 - 56 * k, y + 22)], (90, 100, 120), 6)
    for a in (200, 240, 300, 340):
        ic.speed(256 + 70 * math.cos(math.radians(a)), 440 + 50 * math.sin(math.radians(a)), 40, a, (255, 190, 80), 6)
    for x, y in ((150, 400), (370, 420), (330, 470)):
        ic.dot(x, y, 9, (255, 170, 60), ink=tone((255, 150, 40)))


# ---- Reactor Roll reels (and Orbit 21 suits) ------------------------------------

def sym_core(ic):
    ic.glow((256, 256, 150), (90, 255, 140), radius=34, strength=0.6)
    ic.drawn(circle(256, 256, 120), (70, 230, 110), rim_color=(200, 255, 200), edge=(20, 120, 50))
    for rot in (0.0, 1.05, 2.1):
        ic.line(ellipse(256, 256, 200, 70, 60, rot) + [ellipse(256, 256, 200, 70, 60, rot)[0]], (180, 255, 210), 7)
    ic.dot(256, 256, 30, (240, 255, 220), ink=(20, 120, 50))


def sym_ore(ic):
    crystal(ic, 256, 330, 250, 130, 0.0, (255, 140, 40), 9)
    ic.sparkle(390, 140, 28)


def sym_ice(ic):
    """Coolant: a fat drop of it, with a snowflake on its face."""
    drop = [(256, 40)] + [(256 + 150 * math.cos(a), 320 + 150 * math.sin(a)) for a in [-0.35 + i * (math.pi + 0.7) / 30 for i in range(31)]]
    ic.drawn(drop, (60, 170, 250), rim_color=(200, 240, 255), edge=(20, 70, 140), edge_w=10)
    for k in range(3):
        a = k * math.pi / 3
        ic.line([(256 - 70 * math.cos(a), 320 - 70 * math.sin(a)), (256 + 70 * math.cos(a), 320 + 70 * math.sin(a))], (235, 250, 255), 10)
    ic.sparkle(400, 120, 24, (210, 250, 255))


def sym_fuel(ic):
    ic.drawn(roundrect(156, 90, 356, 440, 40), (250, 200, 40), edge=(130, 80, 10), edge_w=10)
    ic.drawn(rect(206, 50, 306, 96), (120, 128, 140), highlight=0, edge=INK, edge_w=7)
    ic.zap([(290, 140), (220, 270), (290, 270), (222, 400)], (255, 255, 200), 12, jag=6, seed=3)


def sym_star(ic):
    ic.glow((256, 256, 160), (255, 220, 80), radius=30, strength=0.5)
    ic.drawn(star(256, 270, 210, 92, 5), (255, 200, 40), rim_color=(255, 245, 170), edge=(170, 90, 10), edge_w=10)
    ic.dot(220, 250, 16, INK)
    ic.dot(292, 250, 16, INK)


def sym_meltdown(ic):
    ic.glow((256, 256, 170), (180, 60, 255), radius=34, strength=0.6)
    ic.drawn(star(256, 256, 230, 150, 14, jag=0.25, seed=5), (130, 50, 230), edge=(60, 10, 120), edge_w=8)
    ic.drawn(circle(256, 256, 120), (60, 20, 90), highlight=0, shade=0.1, edge=(30, 6, 50), edge_w=7)
    for k in range(3):   # the trefoil
        a0 = -math.pi / 2 + k * 2 * math.pi / 3 - 0.5
        pts = [(256, 256)] + [(256 + 100 * math.cos(a0 + i * 0.1), 256 + 100 * math.sin(a0 + i * 0.1)) for i in range(11)]
        ic.drawn(pts, (255, 220, 60), highlight=0, shade=0, edge=(120, 80, 10), edge_w=5)
    ic.dot(256, 256, 22, (60, 20, 90), ink=(255, 220, 60))


# ---- dice, chips, cards ---------------------------------------------------------

PIPS = {1: [(0, 0)], 2: [(-1, -1), (1, 1)], 3: [(-1, -1), (0, 0), (1, 1)], 4: [(-1, -1), (1, -1), (-1, 1), (1, 1)],
        5: [(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)], 6: [(-1, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (1, 1)]}


def die(n, col=(225, 50, 60)):
    def fn(ic):
        ic.drawn(roundrect(70, 70, 442, 442, 70), col, rim_color=lighter(col, 0.5), edge_w=12)
        for dx, dy in PIPS[n]:
            ic.dot(256 + dx * 105, 256 + dy * 105, 34, (255, 250, 236), ink=tone(col))
    return fn


def chip(col, mark):
    def fn(ic):
        ic.drawn(circle(256, 256, 200), col, rim_color=lighter(col, 0.5), edge_w=10)
        for k in range(8):
            a = k * math.pi / 4
            p = rotate(rect(236, 56, 276, 106), a, (256, 256))
            ic.drawn(p, (250, 246, 230), highlight=0, shade=0, edge=None)
        ic.drawn(circle(256, 256, 120), lighter(col, 0.15), highlight=0.1, edge=tone(col), edge_w=7)
        mark(ic)
    return fn


def hexmark(ic):
    ic.drawn(circle(256, 256, 64, 6, start=math.pi / 6), (250, 246, 230), highlight=0, shade=0, edge=INK, edge_w=6)


def card_back(ic):
    ic.drawn(roundrect(96, 30, 416, 482, 34), (34, 60, 130), edge=(14, 22, 50), edge_w=12)
    ic.drawn(roundrect(126, 60, 386, 452, 22), (52, 92, 180), highlight=0.1, edge=(250, 200, 60), edge_w=6)
    ic.line(circle(256, 256, 90, 40) + [circle(256, 256, 90, 40)[0]], (250, 200, 60), 8)
    ic.line(ellipse(256, 256, 150, 46, 50, -0.4) + [ellipse(256, 256, 150, 46, 50, -0.4)[0]], (250, 200, 60), 6)
    ic.dot(256, 256, 30, (120, 200, 255), ink=(14, 22, 50))
    ic.sparkle(170, 120, 18)
    ic.sparkle(350, 400, 14)


# ---- the race and the job board --------------------------------------------------

def racer(col):
    def fn(ic):
        for y in (210, 260, 310):
            ic.speed(30, y, 90, 0, (240, 240, 240), 7)
        body = [(140, 256), (210, 196), (400, 220), (470, 256), (400, 292), (210, 316)]
        ic.drawn(body, col, rim_color=lighter(col, 0.5), edge_w=9)
        ic.drawn([(220, 200), (300, 120), (330, 120), (300, 214)], darker(col, 0.15), highlight=0, edge=tone(col), edge_w=7)
        ic.drawn([(220, 312), (300, 392), (330, 392), (300, 298)], darker(col, 0.15), highlight=0, edge=tone(col), edge_w=7)
        ic.drawn(ellipse(380, 250, 46, 22, 30), (150, 230, 255), highlight=0.2, shade=0.2, edge=INK, edge_w=6)
        ic.glow((130, 256, 50), (255, 190, 80), radius=20, strength=0.8)
        ic.drawn(star(120, 256, 50, 24, 6, jag=0.3, seed=2), (255, 200, 80), highlight=0, edge=(170, 80, 20), edge_w=5)
    return fn


def bounty(ic):
    ic.drawn(star(256, 256, 220, 150, 12, rot=0), (230, 60, 50), edge=(110, 20, 20), edge_w=9)
    ic.drawn(circle(256, 256, 130), (250, 220, 120), edge=(120, 80, 20), edge_w=8)
    ic.line([(180, 180), (332, 332)], (110, 20, 20), 14)          # crosshair over the face
    ic.line(circle(256, 256, 70, 40) + [circle(256, 256, 70, 40)[0]], (110, 20, 20), 10)
    ic.line([(256, 150), (256, 362)], (110, 20, 20), 8)
    ic.line([(150, 256), (362, 256)], (110, 20, 20), 8)


def salvage(ic):
    ic.drawn(rotate(rect(110, 170, 400, 400), 0.15, (256, 285)), (100, 130, 120), edge=INK, edge_w=9)
    ic.line(rotate([(130, 250), (390, 250)], 0.15, (256, 285)), (230, 180, 60), 12)
    ic.drawn(rotate([(300, 120), (440, 170), (430, 210), (290, 160)], 0.1, (360, 160)), (150, 156, 170), edge=INK, edge_w=7)
    ic.zap([(160, 130), (200, 90), (230, 120), (270, 70)], (120, 220, 255), 8, jag=6, seed=2)
    ic.sparkle(420, 380, 22)


def cargo(ic):
    ic.drawn(rect(80, 160, 432, 420), (240, 150, 40), edge=(120, 60, 10), edge_w=10)
    for x in (170, 256, 342):
        ic.line([(x, 170), (x, 410)], (180, 100, 20), 8)
    ic.drawn(rect(80, 120, 432, 170), (60, 64, 76), highlight=0, edge=INK, edge_w=7)
    ic.drawn(rect(190, 250, 322, 330), (250, 246, 230), highlight=0, shade=0, edge=INK, edge_w=5)
    ic.line([(206, 275), (306, 275)], INK, 5)
    ic.line([(206, 300), (280, 300)], INK, 5)
    ic.speed(440, 300, 50, 0, (240, 240, 240), 7)
    ic.speed(440, 360, 40, 0, (240, 240, 240), 7)


ICONS = {
    "ore_iron": (ore_iron, 1), "ore_copper": (ore_copper, 2), "ore_gold": (ore_gold, 3),
    "ore_ice": (ore_ice, 4), "slag": (slag, 5), "drill": (drill, 6),
    "sym_core": (sym_core, 7), "sym_ore": (sym_ore, 8), "sym_ice": (sym_ice, 9),
    "sym_fuel": (sym_fuel, 10), "sym_star": (sym_star, 11), "sym_meltdown": (sym_meltdown, 12),
    **{f"die_{n}": (die(n), 20 + n) for n in range(1, 7)},
    "chip_10": (chip((40, 110, 220), hexmark), 30), "chip_50": (chip((220, 50, 60), hexmark), 31),
    "chip_100": (chip((30, 30, 40), hexmark), 32),
    "card_back": (card_back, 33),
    "racer_red": (racer((225, 60, 50)), 40), "racer_blue": (racer((50, 120, 230)), 41),
    "racer_green": (racer((60, 190, 80)), 42), "racer_yellow": (racer((240, 190, 40)), 43),
    "bounty": (bounty, 50), "salvage": (salvage, 51), "cargo": (cargo, 52),
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
        pick = [p for p in paths if not os.path.basename(p).startswith("die_") or p.endswith("die_5.png")]
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), out, *pick, "--ref", "inferno", "--ref", "death aura"], check=True)


if __name__ == "__main__":
    main()
