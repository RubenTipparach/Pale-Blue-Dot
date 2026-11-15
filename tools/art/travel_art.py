"""Draw the icons for docs/mockups/travel.html as pixel art (balatro-pixel-art kit), and copy the
art it borrows into docs/mockups/travel/ byte-identical.

    python tools/art/travel_art.py            # 32 x 32 PNGs into docs/mockups/travel/, plus the copies
    python tools/art/travel_art.py --sheet    # also a contact sheet beside six of the
                                              # solitaire collection's sprites, in
                                              # $ART_SHEET_DIR (default target/art-sheets)

Same kit and rules as tools/art/planets_art.py: a tiny native canvas, short ramps lit
from the top left, a 1 px outline, light drawn after the outline, Sweaft-64 only.

2026-11-14 (the owner: "Aubade Transit - you just lifted off with a vtol to another planet that
makes no fucking sense lol"): the page now travels on M21's rockets, so the runabout and dampener
icons (the free lift drive's ship) are gone, an aerostat icon is new, and the rocket part textures
and icons are copied from M21 (docs/mockups/ship-tiers/, drawn by tools/art/ship_tiers_art.py)
into travel/rocket/, the settlement textures the launch field needs into travel/game/.
"""

import math
import os
import shutil
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from planets_art import RAMPS, ROOT, SCRATCH, Sprite, refs, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "travel")
HULL = ["#3e586e", "#92a3b2", "#bfcad7", "#fffefe"]
PLUME = ["#15578a", "#1096c3", "#74effe", "#d2f8ff", "#fffefe"]


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


def aerostat():
    """An aerostat for a giant's airship band: a fat hot-hydrogen envelope with seams, a gondola
    slung under it with two lit windows, and a scoop line hanging below."""
    sp = Sprite(32, 32)
    env = sp.ellipse(3, 3, 28, 19)
    sp.part(env, RAMPS["brass"], "sphere")
    for x in (10, 16, 22):
        sp.fill(sp.line([(x, 4), (x, 18)], 1) & env, "#733126")
    for x0, x1 in ((11, 13), (19, 21)):
        sp.part(sp.line([(x0, 18), (x1 + 1, 22)], 1), RAMPS["metal"], "flat", base=2)
    sp.part(sp.rect(10, 22, 22, 26), RAMPS["metal"], "cyl_h")
    sp.part(sp.rect(15, 27, 16, 30), RAMPS["steel"], "flat", base=2)
    sp.plot(9, 7, "#fffefe")
    sp.outline()
    sp.plots([(13, 24), (19, 24)], "#faaf5c", glow=True)
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


ICONS = {"gate": gate, "station": station, "fuel": fuel, "skimmer": skimmer, "aerostat": aerostat}

# Copied byte-identical, never edited: M21's rocket art (its own pixel art, ship_tiers_art.py) and the
# game's settlement textures for the launch field. Only served image types (no .glb).
ST = os.path.join(ROOT, "docs", "mockups", "ship-tiers")
ROCKET = {n + ".png": os.path.join(ST, n + ".png") for n in (
    "tex-pod", "tex-tank", "tex-engine", "tex-ntr", "tex-booster", "tex-fin", "tex-hazard",
    "pod", "tank", "engine-chem", "engine-ntr", "booster", "legs", "fusion", "antimatter",
    "lander", "fusionship", "navcomp")}
GAME = {n + ".png": os.path.join(ROOT, "assets", "textures", "settlement", n + ".png") for n in ("granite", "stone", "slate")}


def copy_borrowed():
    for folder, files in ((os.path.join(OUT, "rocket"), ROCKET), (os.path.join(OUT, "game"), GAME)):
        os.makedirs(folder, exist_ok=True)
        for name, src in files.items():
            shutil.copyfile(src, os.path.join(folder, name))
        print("copied", len(files), "files to", folder)


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    copy_borrowed()
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
