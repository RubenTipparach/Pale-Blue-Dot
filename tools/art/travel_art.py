"""Paint the icons for docs/mockups/travel.html in the friend's hand-drawn manner.

    python tools/art/travel_art.py            # PNGs into docs/mockups/travel/
    python tools/art/travel_art.py --sheet    # also a contact sheet beside the friend's icons

Same kit and same rules as tools/art/planets_art.py: painted by code copying
the friend's method, never the friend's own drawing.
"""

import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from planets_art import KIT, blob, meta  # noqa: E402
import crayon  # noqa: E402
from crayon import Icon, circle, ellipse, rect, roundrect, rotate, star, translate, lighter, darker  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(ROOT, "docs", "mockups", "travel")


def ship(ic):
    """The runabout: a wedge hull, a canopy, two engine pods, a blue flame."""
    for y in (206, 318):
        ic.glow((100, y, 40), (120, 180, 255), radius=16, strength=0.7)
        ic.drawn([(28, y), (80, y - 30), (136, y - 22), (136, y + 22), (80, y + 30)], (90, 160, 255), shade=0, highlight=0.25, rim_color=(210, 235, 255), edge=(30, 60, 170), edge_w=6)
    hull = [(130, 190), (300, 168), (440, 248), (440, 276), (300, 356), (130, 334), (110, 262)]
    ic.drawn(hull, (226, 228, 232), shade=0.3, highlight=0.18)
    ic.drawn([(210, 236), (330, 214), (400, 256), (330, 300), (210, 290)], (240, 170, 50), shade=0.12, highlight=0.0, edge_w=6)
    ic.drawn(ellipse(330, 256, 46, 26, n=30), (60, 170, 230), shade=0.15, highlight=0.25, rim_color=(200, 240, 255), edge=(20, 70, 120), edge_w=6)
    for y in (206, 318):
        ic.drawn(roundrect(130, y - 26, 236, y + 26, 18), (120, 126, 140), shade=0.25, highlight=0.15, edge=(36, 40, 52), edge_w=7)
    for y, L in ((140, 70), (390, 70)):
        ic.speed(40, y, L, 0, (230, 240, 255), width=7)


def gate(ic):
    """A lane gate: a ring of plates with a field across it."""
    ic.glow((256, 256, 150), (90, 230, 255), radius=30, strength=0.6)
    ic.drawn(circle(256, 256, 130), (70, 200, 240), shade=0.2, highlight=0.25, rim_color=(210, 250, 255), edge=(20, 90, 140), edge_w=6, hatch=0.7)
    ic.drawn(crayon.ring(256, 256, 214, 160), (126, 132, 150), shade=0.25, highlight=0.15, edge=(36, 40, 56))
    for k in range(8):
        a = k * math.pi / 4 + math.pi / 8
        x, y = 256 + 187 * math.cos(a), 256 + 187 * math.sin(a)
        ic.dot(x, y, 12, (255, 196, 60), ink=(90, 60, 20))
    ic.sparkle(256, 256, 54, (240, 255, 255), ink=(20, 90, 140))


def station(ic):
    """A ring station: a hub, four spokes, a lit habitat ring, a docking arm."""
    ic.drawn(crayon.ring(256, 256, 214, 168), (196, 200, 214), shade=0.3, highlight=0.15, edge=(40, 46, 64))
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        spoke = rotate(rect(-14, 60, 14, 176), a - math.pi / 2)
        ic.drawn(translate(spoke, 256, 256), (150, 156, 172), shade=0.1, highlight=0, edge=(40, 46, 64), edge_w=6)
    for k in range(12):
        a = k * math.pi / 6
        ic.dot(256 + 191 * math.cos(a), 256 + 191 * math.sin(a), 8, (255, 214, 120), ink=(110, 70, 20))
    ic.drawn(circle(256, 256, 70), (224, 120, 60), shade=0.25, highlight=0.2, edge=(120, 50, 20))
    ic.drawn(rect(240, 120, 272, 200), (150, 156, 172), edge=(40, 46, 64), edge_w=6)
    ic.dot(256, 256, 24, (255, 236, 160), ink=(120, 60, 20))


def fuel(ic):
    """A hydrogen cell: a squat canister with an H and a gauge."""
    ic.drawn(roundrect(140, 110, 372, 440, 50), (230, 70, 60), shade=0.3, highlight=0.18, rim_color=(255, 170, 150))
    ic.drawn(rect(212, 64, 300, 112), (120, 126, 140), edge=(36, 40, 52))
    for x0, y0, x1, y1 in ((196, 190, 226, 360), (286, 190, 316, 360), (196, 262, 316, 290)):
        ic.drawn(rect(x0, y0, x1, y1), (250, 240, 220), shade=0, highlight=0, edge=(120, 30, 20), edge_w=5)
    ic.sparkle(420, 130, 26, (255, 236, 160))


def dampener(ic):
    """Dampeners: a ship-shaped dot held still by four arrows pressing in."""
    ic.glow((256, 256, 100), (120, 255, 170), radius=26, strength=0.5)
    ic.drawn(circle(256, 256, 96), (70, 210, 120), shade=0.25, highlight=0.2, rim_color=(200, 255, 210), edge=(20, 100, 50))
    for k in range(4):
        a = k * math.pi / 2
        arrow = [(-236, -34), (-176, -34), (-176, -66), (-112, 0), (-176, 66), (-176, 34), (-236, 34)]
        arrow = translate(rotate(arrow, a), 256, 256)
        ic.drawn(arrow, (255, 206, 60), shade=0.1, highlight=0.1, edge=(150, 90, 10), edge_w=6)
    ic.drawn(circle(256, 256, 30), (240, 248, 240), shade=0, highlight=0, edge=(20, 100, 50), edge_w=5)


def skimmer(ic):
    """The gas scoop: a funnel with streams of gas pulled in."""
    for k, y in enumerate((190, 256, 322)):
        ic.line([(30, y + 30 * (k - 1)), (120, y + 10 * (k - 1)), (200, y)], (255, 196, 120), 9, ink=(160, 90, 40), ink_w=5)
    funnel = [(200, 150), (300, 216), (420, 216), (420, 296), (300, 296), (200, 362)]
    ic.drawn(funnel, (126, 132, 150), shade=0.3, highlight=0.15, edge=(36, 40, 56))
    ic.drawn(ellipse(200, 256, 30, 106, n=36), (60, 40, 30), shade=0, highlight=0, edge=(36, 40, 56), edge_w=7)
    ic.drawn(roundrect(400, 196, 480, 316, 20), (240, 170, 50), shade=0.2, highlight=0.2)


ICONS = [("ship", ship, 41), ("gate", gate, 42), ("station", station, 43), ("fuel", fuel, 44), ("dampener", dampener, 45), ("skimmer", skimmer, 46)]


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    crayon.S = 512
    for name, fn, seed in ICONS:
        ic = Icon(seed)
        fn(ic)
        ic.final().save(os.path.join(OUT, "icon_" + name + ".png"), pnginfo=meta())
        print("icon", name)
    if "--sheet" in sys.argv:
        scratch = os.environ.get("ART_SHEET_DIR", OUT)
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), os.path.join(scratch, "sheet_travel.png")]
                       + [os.path.join(OUT, "icon_" + n + ".png") for n, _, _ in ICONS] + ["--ref", "power output", "--ref", "eagle eye"])
