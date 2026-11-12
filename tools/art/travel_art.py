"""Draw the icons for docs/mockups/travel.html as pixel art (balatro-pixel-art kit).

    python tools/art/travel_art.py            # 32 x 32 PNGs into docs/mockups/travel/
    python tools/art/travel_art.py --sheet    # also a contact sheet beside six of the
                                              # solitaire collection's sprites, in
                                              # $ART_SHEET_DIR (default target/art-sheets)

Same kit and rules as tools/art/planets_art.py: a tiny native canvas, short ramps lit
from the top left, a 1 px outline, light drawn after the outline, Sweaft-64 only.
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from planets_art import RAMPS, ROOT, SCRATCH, Sprite, refs, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "travel")
HULL = ["#3e586e", "#92a3b2", "#bfcad7", "#fffefe"]
PLUME = ["#15578a", "#1096c3", "#74effe", "#d2f8ff", "#fffefe"]


def ship():
    """The runabout: a wedge hull, an amber stripe, a canopy, two engine pods, blue plumes."""
    sp = Sprite(32, 32)
    for y in (4, 24):
        sp.part(sp.rect(8, y, 17, y + 4), RAMPS["metal"], "cyl_h")
    sp.part(sp.poly([(7, 7), (20, 5), (31, 15), (31, 17), (20, 27), (7, 25), (5, 16)]), HULL, "cyl_h")
    sp.part(sp.poly([(11, 12), (21, 11), (27, 16), (21, 21), (11, 20)]), RAMPS["brass"], "flat", base=2)
    sp.part(sp.ellipse(19, 13, 27, 19), RAMPS["cyan"], "sphere")
    sp.plot(21, 14, "#fffefe")
    sp.outline()
    for y in (4, 24):
        sp.part(sp.taper(7, y + 2, 0, y + 2, 5, 1), PLUME, "inset", glow=True)
    return sp


def gate():
    """A lane gate: a ring of plates with a field across it, amber lamps on the ring."""
    sp = Sprite(32, 32)
    ring = sp.circle(16, 16, 14) & ~sp.circle(16, 16, 10)
    sp.part(ring, RAMPS["metal"], "sphere")
    for k in range(8):
        a = k * math.pi / 4
        x, y = 16 + 12 * math.cos(a), 16 + 12 * math.sin(a)
        sp.fill(sp.line([(16 + 10 * math.cos(a), 16 + 10 * math.sin(a)), (16 + 14 * math.cos(a), 16 + 14 * math.sin(a))], 1) & ring, "#1f3b52")
        del x, y
    sp.outline()
    sp.part(sp.circle(16, 16, 9), ["#15578a", "#1096c3", "#74effe", "#d2f8ff"], "inset", glow=True)
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        sp.plot(round(16 + 12 * math.cos(a)), round(16 + 12 * math.sin(a)), "#faaf5c", glow=True)
    sp.plots([(16, 16), (15, 16), (17, 16), (16, 15), (16, 17)], "#fffefe", glow=True)
    return sp


def station():
    """A ring station: a lit habitat ring, four spokes, an orange hub, a docking arm."""
    sp = Sprite(32, 32)
    ring = sp.circle(16, 16, 14) & ~sp.circle(16, 16, 11)
    sp.part(ring, HULL, "sphere")
    for k in range(4):
        a = k * math.pi / 2 + math.pi / 4
        sp.part(sp.line([(16 + 5 * math.cos(a), 16 + 5 * math.sin(a)), (16 + 11 * math.cos(a), 16 + 11 * math.sin(a))], 2), RAMPS["metal"], "flat", base=2)
    sp.part(sp.rect(15, 3, 17, 10), RAMPS["metal"], "cyl_v")
    sp.part(sp.circle(16, 16, 5), RAMPS["brass"], "sphere")
    sp.plot(14, 14, "#fffefe")
    sp.outline()
    for k in range(12):
        a = k * math.pi / 6 + 0.26
        sp.plot(round(16 + 12.5 * math.cos(a)), round(16 + 12.5 * math.sin(a)), "#faaf5c", glow=True)
    return sp


def fuel():
    """A hydrogen cell: a squat red canister with a white H, a grey cap."""
    sp = Sprite(32, 32)
    sp.part(sp.rect(12, 2, 19, 6), RAMPS["metal"], "cyl_v")
    sp.part(sp.ellipse(6, 5, 25, 30) & sp.rect(6, 6, 25, 30), RAMPS["red"], "cyl_v")
    for x0, y0, x1, y1 in ((11, 12, 12, 23), (19, 12, 20, 23), (13, 17, 18, 18)):
        sp.fill(sp.rect(x0, y0, x1, y1), "#fffefe")
    sp.plots([(9, 9), (9, 10)], "#fffefe")
    sp.outline()
    return sp


def dampener():
    """Dampeners: a ship-shaped dot held still by four arrows pressing in."""
    sp = Sprite(32, 32)
    sp.part(sp.circle(16, 16, 7), RAMPS["green"], "sphere")
    for k in range(4):
        a = k * math.pi / 2
        c, s = math.cos(a), math.sin(a)
        pts = [(-15, -2), (-12, -2), (-12, -5), (-8, 0), (-12, 5), (-12, 2), (-15, 2)]
        sp.part(sp.poly([(16 + x * c - y * s, 16 + x * s + y * c) for x, y in pts]), RAMPS["gold"][:4], "rim")
    sp.part(sp.circle(16, 16, 2), RAMPS["paper"], "flat")
    sp.plot(13, 12, "#fffefe")
    sp.outline()
    return sp


def skimmer():
    """The gas scoop: a funnel with streams of gas pulled in."""
    sp = Sprite(32, 32)
    sp.part(sp.poly([(10, 6), (19, 12), (27, 12), (27, 20), (19, 20), (10, 26)]), RAMPS["metal"], "cyl_h")
    sp.part(sp.ellipse(7, 6, 13, 26), ["#161612", "#2c2c25", "#434137"], "flat", base=1)
    sp.part(sp.rect(25, 11, 30, 21), RAMPS["brass"], "cyl_h")
    sp.plot(18, 13, "#fffefe")
    sp.outline()
    for y0, y1 in ((8, 12), (16, 16), (24, 20)):
        sp.fill(sp.line([(0, y0), (4, (y0 + y1) / 2), (7, y1)], 1) & ~sp.alpha, "#faaf5c", glow=True)
    return sp


def cloud_amber():
    """Amber's cloud deck seen from just above it: a 64 x 64 tile that repeats both ways, wavy
    bands in the giant's own ramp, dithered between tones, a few bright puff tops."""
    import numpy as np
    from PIL import Image
    from pixelkit import BAYER4, rgb
    n = 64
    ys, xs = np.mgrid[0:n, 0:n]
    u, v = xs / n * 2 * math.pi, ys / n * 2 * math.pi
    # every term has a whole number of periods across the tile, so it repeats without a seam
    b = np.sin(4 * v + 0.9 * np.sin(2 * u) + 0.4 * np.sin(5 * u + v)) * 0.5 + 0.5
    b = b * 0.8 + 0.2 * (np.sin(3 * u + 2 * v) * 0.5 + 0.5)
    ramp = ["#8d4526", "#a8633e", "#e69d69", "#cfaa81", "#eab798"]
    bay = (BAYER4[ys % 4, xs % 4] + 0.5) / 16.0
    idx = np.clip(np.floor(b * len(ramp) + (bay - 0.5) * 1.4), 0, len(ramp) - 1).astype(int)
    img = np.zeros((n, n, 3), np.uint8)
    for i, c in enumerate(ramp):
        img[idx == i] = rgb(c)
    for x, y in ((9, 13), (40, 29), (25, 50), (54, 6)):          # puff tops, lit from the top left
        for dx, dy, c in ((0, 0, "#fffefe"), (1, 0, "#eab798"), (0, 1, "#eab798"), (1, 1, "#cfaa81")):
            img[(y + dy) % n, (x + dx) % n] = rgb(c)
    return Image.fromarray(img, "RGB").convert("RGBA")


ICONS = {"ship": ship, "gate": gate, "station": station, "fuel": fuel, "dampener": dampener, "skimmer": skimmer}


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    items = []
    for name, fn in ICONS.items():
        im = fn().image()
        save(im, os.path.join(OUT, "icon_" + name + ".png"), source="pixelkit (tools/art/travel_art.py)")
        items.append((name, im))
        print("icon", name)
    deck = cloud_amber()
    save(deck, os.path.join(OUT, "cloud_amber.png"), source="pixelkit (tools/art/travel_art.py)")
    print("texture cloud_amber")
    if "--sheet" in sys.argv:
        from PIL import Image
        tiled = Image.new("RGBA", (128, 128))
        for i in range(2):
            for j in range(2):
                tiled.paste(deck, (64 * i, 64 * j))
        items.append(("cloud deck 2x2", tiled))
        os.makedirs(SCRATCH, exist_ok=True)
        sheet(items + refs(), scale=3, cols=6, cell=(288, 192)).save(os.path.join(SCRATCH, "sheet_travel.png"))
