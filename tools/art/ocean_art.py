"""Water-life sprites and mockup-only textures for the seas, rivers and marshes mockup
(docs/mockups/ocean.html).

Pixel art drawn in code with the balatro-pixel-art kit
(.claude/skills/balatro-pixel-art/scripts/pixelkit.py): a tiny native canvas, one bold subject,
3 to 5 tone ramps lit from the top left, dark seams between parts, a 1 px outline, single-pixel
glints, no anti-aliasing, Sweaft-64 colours only (`save()` refuses anything else).

Creatures are 24 x 24. The same picture is the bestiary icon (shown at 2x and 3x) and the card the
3D scene draws: swimmers and fliers are SIDE views with the head to the right (+x), so a card
turned along the creature's heading reads correctly; crawlers on the bed are TOP views with the
head to the right, laid flat. Textures are 32 x 32 tiles (seamless where they repeat); plant cards
have hard alpha.

Only what the game does not have is drawn here. The game's fish icons (16 x 16,
assets/items/fish), the terrain atlas and the settlement textures are copied byte-identical into
docs/mockups/ocean/game/ instead (see its README); the M7 animals and M9 fish-lab portraits the
page shows are copied from those mockups' folders.

    python tools/art/ocean_art.py              # writes docs/mockups/ocean/*.png
    python tools/art/ocean_art.py --sheet S    # also a contact sheet at S (keep it out of the repo)
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
from pixelkit import Sprite, RAMPS, save, sheet, erode  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "ocean")
N = 24
WHITE = "#fffefe"
EYE = "#010000"

R = dict(RAMPS)
R.update({
    "orange": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "coral": ["#841c1d", "#d4462b", "#e69d69", "#eab798"],
    "pink": ["#561c27", "#c34988", "#dd576d", "#eab798"],
    "purple": ["#230c44", "#4f1d83", "#7d2dbd", "#9e3ba2"],
    "teal": ["#0c2f44", "#204f63", "#53998b", "#84e3b5"],
    "sea": ["#011c34", "#15578a", "#1096c3", "#74effe"],
    "kelp": ["#2e0d06", "#725244", "#ab8465", "#cfaa81"],
    "kelpg": ["#003227", "#05582d", "#559642", "#9bb675"],
    "brown": ["#2e0d06", "#41322c", "#725244", "#ab8465"],
    "fur": ["#170916", "#2f1525", "#522b2f", "#7d4636"],
    "otter": ["#2e0d06", "#522b2f", "#7d4636", "#a8633e"],
    "seal": ["#1f3b52", "#3e586e", "#5d778d", "#92a3b2"],
    "grey": ["#2c2c25", "#434137", "#a09f91", "#bfcad7"],
    "white": ["#92a3b2", "#bfcad7", "#fffefe"],
    "red": ["#4a051e", "#a61e1f", "#d12d41", "#fa3a64"],
    "rust": ["#2e0d06", "#733126", "#a8633e", "#e69d69"],
    "yellow": ["#8d4526", "#c56d25", "#faaf5c", "#fffefe"],
    "lime": ["#05582d", "#559642", "#9bb675", "#c1f1d8"],
    "mud": ["#2b1f35", "#41322c", "#725244", "#ab8465"],
    "duckg": ["#003227", "#05582d", "#297537", "#559642"],
    "glass": ["#0c2f44", "#204f63", "#53998b", "#84e3b5", "#c1f1d8"],
    "ice": ["#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "deep": ["#170916", "#2b1f35", "#3e3551", "#69608b"],
    "blue": ["#191954", "#15578a", "#1096c3", "#74effe"],
})


def dark(ramp):
    return [ramp[0]] + ramp[:-1]


def eye(sp, x, y, big=False, glow=None):
    if big:
        sp.fill(sp.rect(x, y, x + 1, y + 1), EYE)
        sp.plot(x, y, WHITE)
    else:
        sp.plot(x, y, EYE)
    if glow:
        sp.plot(x, y, glow, glow=True)


def blob(sp, circles):
    m = sp.empty()
    for cx, cy, r in circles:
        m |= sp.circle(cx, cy, r)
    return m


# ---------- side-view swimmers (head to the right) ---------------------------------------------

def fish(P):
    """A fish in profile, head right. P: body box, ramps, tail shape, fins, pattern."""
    sp = Sprite(N, N)
    x0, y0, x1, y1 = P.get("box", (5, 8, 21, 16))
    col, fin = R[P["col"]], R[P.get("fin", P["col"])]
    cy = (y0 + y1) // 2
    # tail, behind
    t = P.get("tail", "fork")
    if t == "fork":
        sp.part(sp.poly([(x0 + 2, cy), (x0 - 4, cy - 5), (x0 - 2, cy), (x0 - 4, cy + 5)]), fin, "rim")
    elif t == "round":
        sp.part(sp.ellipse(x0 - 4, cy - 3, x0 + 2, cy + 3), fin, "rim")
    elif t == "lunate":
        sp.part(sp.poly([(x0 + 2, cy), (x0 - 3, cy - 6), (x0 - 1, cy), (x0 - 3, cy + 6)]), fin, "rim")
    # dorsal fin
    if P.get("dorsal", True):
        dx = (x0 + x1) // 2
        sp.part(sp.poly([(dx - 4, y0 + 1), (dx - 1, y0 - P.get("dh", 3)), (dx + 3, y0 + 1)]), dark(fin), "rim")
    if P.get("wings"):   # a glider's long pectoral wing
        sp.part(sp.poly([(x1 - 6, cy), (x0, y0 - 5), (x0 + 3, y0 - 6), (x1 - 3, cy - 1)]), R["sea"], "rim")
    body = sp.ellipse(x0, y0, x1, y1)
    sp.part(body, col, "cyl_h")
    belly = P.get("belly")
    if belly:
        sp.part(sp.ellipse(x0 + 3, cy + 1, x1 - 2, y1) & body, R[belly], "rim", seam=False)
    for i, sx in enumerate(P.get("stripes", [])):
        m = sp.rect(sx, y0, sx + P.get("sw", 1), y1) & body
        sp.part(m, R[P.get("stripe", "white")], "flat", seam=False)
    for x, y in P.get("spots", []):
        sp.plot(x, y, R[P.get("spot", "white")][-1])
    if P.get("lateral"):
        sp.part(sp.rect(x0 + 2, cy, x1 - 3, cy) & body, [R[P["lateral"]][0]], "flat", seam=False)
    # pectoral fin
    sp.part(sp.poly([(x1 - 6, cy + 1), (x1 - 9, cy + 3), (x1 - 6, cy + 3)]), dark(fin), "flat")
    if P.get("beak"):
        sp.part(sp.rect(x1 - 1, cy - 1, x1 + 1, cy + 1), R[P["beak"]], "rim")
    eye(sp, x1 - 4, cy - 2, big=P.get("bigeye", True))
    sp.outline()
    for x, y, c in P.get("glow", []):
        sp.plot(x, y, c, glow=True)
    return sp


def squid():
    sp = Sprite(N, N)
    col = R["pink"]
    for i, y in enumerate((10, 12, 14, 16)):   # arms trailing behind (left)
        sp.part(sp.line([(8, y - 1 + i // 2), (3, y - 2 + i), (1, y + i - 1)], 1), dark(col), "flat")
    sp.part(sp.poly([(18, 13), (23, 8), (22, 13), (23, 18)]), col, "rim")   # fins at the mantle tip
    mantle = sp.ellipse(7, 10, 21, 16)
    sp.part(mantle, col, "cyl_h")
    sp.part(sp.ellipse(6, 10, 10, 15), dark(col), "rim")
    for x in (12, 15, 18):
        sp.plot(x, 12, "#fa3a64")
    eye(sp, 8, 11, big=True)
    sp.outline()
    for x, y in ((11, 15), (14, 15), (17, 15), (20, 14)):
        sp.plot(x, y, "#74effe", glow=True)
    return sp


def jelly():
    sp = Sprite(N, N)
    for x in (7, 10, 13, 16):
        sp.part(sp.line([(x, 11), (x - 1, 16), (x + 1, 20), (x, 23)], 1), R["violet"][2:], "flat")
    sp.part(sp.line([(11, 11), (12, 17), (11, 21)], 2), R["pink"], "flat")
    bell = sp.ellipse(4, 3, 19, 14) & ~sp.rect(0, 12, 23, 23)
    sp.part(bell, R["glass"], "sphere")
    sp.part(sp.ellipse(8, 6, 15, 10) & bell, R["violet"][2:], "inset", seam=False)
    sp.plot(7, 5, WHITE)
    sp.outline()
    for x in (5, 9, 14, 18):
        sp.plot(x, 11, "#d2f8ff", glow=True)
    return sp


def turtle():
    sp = Sprite(N, N)
    flip = R["olive"]
    sp.part(sp.poly([(15, 13), (21, 19), (16, 18)]), dark(flip), "rim")   # near front flipper
    sp.part(sp.poly([(5, 13), (2, 17), (6, 16)]), dark(flip), "rim")
    shell = sp.ellipse(4, 6, 18, 15) & ~sp.rect(0, 14, 23, 23)
    sp.part(shell, R["kelp"], "sphere")
    for x, y in ((8, 9), (12, 8), (15, 10), (10, 12)):
        sp.plot(x, y, R["kelp"][0])
    sp.part(sp.rect(4, 13, 18, 14), R["sand"], "flat")
    sp.part(sp.ellipse(17, 9, 22, 13), flip, "rim")
    sp.part(sp.poly([(13, 14), (19, 21), (14, 19)]), flip, "rim")
    eye(sp, 20, 10)
    sp.outline()
    return sp


def seal(r="seal"):
    sp = Sprite(N, N)
    col = R[r]
    body = blob(sp, [(9, 15, 5), (14, 14, 5), (19, 12, 3)]) | sp.taper(4, 16, 0, 18, 4, 2)
    sp.part(body, col, "cyl_h")
    sp.part(sp.poly([(0, 16), (0, 20), (3, 18)]), dark(col), "flat")
    sp.part(sp.poly([(12, 17), (10, 21), (14, 19)]), dark(col), "rim")
    for x, y in ((8, 13), (11, 12), (14, 11), (7, 16)):
        sp.plot(x, y, col[0])
    eye(sp, 20, 11, big=True)
    sp.plot(22, 13, EYE)
    sp.outline()
    return sp


def otter(r="otter", back=True):
    """back=True: floating on its back (seen from the side, belly up, paws on chest)."""
    sp = Sprite(N, N)
    col = R[r]
    if back:
        body = sp.ellipse(3, 10, 19, 16) | sp.circle(19, 11, 3) | sp.taper(3, 13, 0, 11, 3, 2)
        sp.part(body, col, "cyl_h")
        sp.part(sp.ellipse(7, 10, 16, 12) & body, R["sand"], "rim", seam=False)
        sp.part(sp.rect(10, 8, 12, 10), dark(col), "rim")   # paws holding a shell
        sp.part(sp.ellipse(9, 6, 13, 8), R["bone"], "rim")
        sp.part(sp.rect(4, 7, 5, 10), dark(col), "flat")    # feet up
        sp.part(sp.circle(20, 10, 2) & body, R["sand"], "rim", seam=False)
        eye(sp, 20, 9)
        sp.plot(22, 11, EYE)
    else:
        body = sp.ellipse(4, 11, 19, 16) | sp.circle(19, 12, 3) | sp.taper(5, 14, 0, 17, 3, 1)
        sp.part(body, col, "cyl_h")
        sp.part(sp.rect(7, 15, 9, 18), dark(col), "rim")
        sp.part(sp.rect(15, 15, 17, 18), dark(col), "rim")
        sp.part(sp.circle(21, 13, 1), R["sand"], "rim", seam=False)
        eye(sp, 20, 11)
        sp.plot(22, 12, EYE)
    sp.outline()
    return sp


def beaver(branch=False):
    sp = Sprite(N, N)
    col = R["fur"][1:] + ["#a8633e"]
    sp.part(sp.ellipse(0, 15, 8, 19), R["deep"], "rim")             # the flat tail
    for x in (2, 4, 6):
        sp.plot(x, 17, "#2b1f35")
    body = sp.ellipse(4, 9, 18, 19) | sp.circle(18, 12, 4)
    sp.part(body, R["otter"], "sphere")
    sp.part(sp.rect(7, 18, 9, 20), R["fur"], "rim")
    sp.part(sp.rect(14, 18, 16, 20), R["fur"], "rim")
    sp.part(sp.rect(21, 14, 21, 15), R["yellow"], "flat")          # the orange teeth
    sp.plot(15, 8, R["fur"][2])
    eye(sp, 19, 10)
    sp.plot(22, 12, EYE)
    sp.outline()
    if branch:
        sp.part(sp.line([(13, 16), (23, 10)], 2), R["wood"], "flat")
        sp.part(sp.ellipse(19, 5, 23, 9), R["green"], "rim")
    return sp


def duck(kind="drake"):
    """Dabbling and diving ducks in profile on the water; 'fly' wings up, 'sleep' head tucked."""
    sp = Sprite(N, N)
    if kind in ("drake", "hen", "diver", "sleep", "duckling"):
        small = kind == "duckling"
        if small:
            body = sp.ellipse(7, 12, 16, 18)
            sp.part(body, R["yellow"][1:], "sphere")
            sp.part(sp.circle(15, 10, 3), R["yellow"][1:], "sphere")
            sp.part(sp.rect(18, 10, 19, 11), R["brown"], "flat")
            eye(sp, 16, 9)
            sp.outline()
            return sp
        bodyc = {"drake": R["grey"], "hen": R["kelp"], "diver": R["ink"] + ["#434137"], "sleep": R["kelp"]}[kind]
        body = sp.ellipse(3, 11, 19, 19) | sp.poly([(3, 13), (0, 11), (3, 16)])
        sp.part(body, bodyc, "cyl_h")
        if kind == "drake":
            sp.part(sp.ellipse(4, 12, 13, 15) & body, R["kelp"], "rim", seam=False)   # brown breast/back
            sp.part(sp.rect(8, 15, 12, 16) & body, R["blue"], "flat", seam=False)      # wing speculum
        if kind == "diver":
            sp.part(sp.ellipse(6, 14, 15, 18) & body, R["white"], "rim", seam=False)
        if kind == "hen":
            for x, y in ((6, 13), (9, 14), (12, 13), (7, 16), (11, 16), (14, 15)):
                sp.plot(x, y, R["kelp"][0])
            sp.part(sp.rect(8, 15, 11, 15) & body, R["blue"], "flat", seam=False)
        if kind == "sleep":
            sp.part(sp.circle(15, 12, 3), dark(bodyc), "sphere")   # head tucked on the back
            sp.plot(16, 11, "#2c2c25")
            sp.outline()
            return sp
        head = {"drake": R["duckg"], "hen": R["kelp"], "diver": R["ink"] + ["#434137"]}[kind]
        sp.part(sp.rect(16, 8, 18, 13), head, "rim")             # neck
        sp.part(sp.ellipse(15, 4, 21, 10), head, "sphere")
        bill = R["yellow"] if kind == "drake" else (R["orange"] if kind == "hen" else R["seal"])
        sp.part(sp.rect(21, 7, 23, 8), bill, "flat")
        if kind == "drake":
            sp.part(sp.rect(16, 11, 18, 11), R["white"], "flat", seam=False)   # white collar
        eye(sp, 19, 6)
        sp.outline()
        return sp
    # flying, wings up
    body = sp.ellipse(3, 12, 18, 17)
    sp.part(sp.poly([(8, 13), (4, 2), (9, 3), (13, 13)]), R["grey"], "rim")
    sp.part(body, R["grey"], "cyl_h")
    sp.part(sp.ellipse(16, 9, 21, 14), R["duckg"], "sphere")
    sp.part(sp.rect(21, 11, 23, 12), R["yellow"], "flat")
    sp.part(sp.poly([(10, 14), (14, 6), (17, 7), (15, 14)]), R["grey"], "rim")
    sp.part(sp.rect(12, 9, 14, 10), R["blue"], "flat", seam=False)
    eye(sp, 19, 11)
    sp.outline()
    return sp


def bird(kind):
    sp = Sprite(N, N)
    if kind == "kingfisher":
        body = sp.ellipse(6, 9, 16, 17)
        sp.part(sp.poly([(7, 14), (1, 19), (4, 19), (9, 16)]), R["blue"], "rim")
        sp.part(body, R["blue"], "cyl_h")
        sp.part(sp.ellipse(8, 13, 15, 17) & body, R["orange"], "rim", seam=False)
        sp.part(sp.circle(15, 9, 4), R["blue"], "sphere")
        sp.part(sp.poly([(18, 8), (23, 10), (18, 11)]), R["ink"], "flat")
        sp.part(sp.rect(12, 17, 12, 20), R["red"], "flat")
        eye(sp, 16, 8)
        sp.plot(16, 10, WHITE)
    sp.outline()
    return sp


def dragonfly():
    sp = Sprite(N, N)
    for y0, y1 in ((4, 9), (14, 19)):
        sp.part(sp.ellipse(6, y0, 13, y1), R["glass"][2:], "flat")
        sp.part(sp.ellipse(12, y0, 19, y1), R["glass"][2:], "flat")
    sp.part(sp.rect(2, 11, 15, 12), R["blue"], "cyl_h")
    sp.part(sp.ellipse(14, 9, 19, 14), R["teal"], "sphere")
    sp.part(sp.circle(20, 11, 2), R["teal"], "sphere")
    sp.plot(21, 10, WHITE)
    sp.outline()
    return sp


def frog():
    sp = Sprite(N, N)   # top view, head right
    col = R["lime"]
    for (a, b) in (((6, 6), (2, 3)), ((6, 18), (2, 21)), ((15, 8), (19, 4)), ((15, 16), (19, 20))):
        sp.part(sp.line([a, b], 2), dark(col), "flat")
    body = sp.ellipse(4, 7, 18, 17) | sp.ellipse(14, 8, 21, 16)
    sp.part(body, col, "sphere")
    for x, y in ((8, 10), (11, 14), (7, 14)):
        sp.plot(x, y, "#05582d")
    sp.part(sp.circle(18, 9, 1), R["yellow"], "rim")
    sp.part(sp.circle(18, 15, 1), R["yellow"], "rim")
    sp.plot(18, 9, EYE)
    sp.plot(18, 15, EYE)
    sp.outline()
    return sp


# ---------- top-view crawlers (head to the right) ----------------------------------------------

def crustacean(P):
    sp = Sprite(N, N)
    col = R[P["col"]]
    kind = P["kind"]
    if kind == "crab":
        for s in (-1, 1):
            for i in range(3):
                x = 9 + i * 3
                sp.part(sp.line([(x, 12 + s * 3), (x - 2, 12 + s * 7), (x - 3, 12 + s * 9)], 1), dark(col), "flat")
        big = P.get("fiddler")
        for s in (-1, 1):
            r = 4 if (big and s < 0) else 2
            sp.part(sp.line([(15, 12 + s * 3), (18, 12 + s * 6)], 2), col, "flat")
            sp.part(sp.circle(19 + (1 if r > 2 else 0), 12 + s * (6 + (1 if r > 2 else 0)), r), col, "sphere")
        sp.part(sp.ellipse(6, 6, 18, 18), col, "sphere")
        sp.plot(18, 10, EYE)
        sp.plot(18, 14, EYE)
        if P.get("hairy"):
            for x, y in ((8, 8), (11, 7), (14, 8), (9, 16), (13, 17)):
                sp.plot(x, y, WHITE)
        sp.outline()
        return sp
    if kind in ("lobster", "crayfish", "shrimp", "krill"):
        L = {"lobster": 1.0, "crayfish": 0.85, "shrimp": 0.75, "krill": 0.5}[kind]
        cx = 12
        segs = 6
        tail_x = int(cx - 10 * L)
        sp.part(sp.poly([(tail_x, 12), (tail_x - 3, 8), (tail_x - 3, 16)]), dark(col), "rim")   # tail fan
        for i in range(segs):
            x = int(tail_x + i * 2.2 * L)
            sp.part(sp.ellipse(x, int(12 - 3 * L), int(x + 3 * L), int(12 + 3 * L)), col, "sphere")
        head = sp.ellipse(int(cx), int(12 - 4 * L), int(cx + 8 * L), int(12 + 4 * L))
        sp.part(head, col, "sphere")
        hx = int(cx + 8 * L)
        for s in (-1, 1):
            sp.part(sp.line([(hx, 12 + s), (23, 12 + s * 8)], 1), dark(col), "flat")   # antennae
            if kind in ("lobster", "crayfish"):
                sp.part(sp.line([(hx - 2, 12 + s * 3), (hx + 2, 12 + s * 6)], 2), col, "flat")
                sp.part(sp.ellipse(hx + 1, 12 + s * 6 - 2, hx + 6, 12 + s * 6 + 2), col, "sphere")
        sp.plot(hx - 1, 10, EYE)
        sp.plot(hx - 1, 14, EYE)
        sp.outline()
        if P.get("glow"):
            sp.plot(cx + 2, 12, "#74effe", glow=True)
        return sp
    if kind == "hermit":
        sp.part(sp.ellipse(3, 5, 15, 19), R["sand"], "sphere")   # the borrowed shell
        for r in (5, 3):
            sp.part(sp.circle(9, 12, r) & ~sp.circle(9, 12, r - 1), [R["sand"][0]], "flat", seam=False)
        for s in (-1, 1):
            sp.part(sp.line([(15, 12 + s * 2), (20, 12 + s * 5)], 2), col, "flat")
        sp.part(sp.ellipse(14, 8, 21, 16), col, "sphere")
        sp.plot(20, 10, EYE)
        sp.plot(20, 14, EYE)
        sp.outline()
        return sp


def starfish():
    sp = Sprite(N, N)
    st = sp.star(12, 12, 11, 4, 5)
    sp.part(st, R["orange"], "sphere")
    for a in range(5):
        ang = math.radians(-90 + a * 72)
        for k in (3, 6):
            sp.plot(int(12 + k * math.cos(ang)), int(12 + k * math.sin(ang)), "#faaf5c")
    sp.outline()
    return sp


def urchin():
    sp = Sprite(N, N)
    for a in range(16):
        ang = math.radians(a * 22.5)
        sp.part(sp.line([(12, 12), (int(12 + 11 * math.cos(ang)), int(12 + 11 * math.sin(ang)))], 1), R["purple"][:2] + ["#4f1d83"], "flat")
    sp.part(sp.circle(12, 12, 6), R["purple"], "sphere")
    sp.plot(10, 9, WHITE)
    sp.outline()
    return sp


def cucumber():
    sp = Sprite(N, N)
    body = sp.taper(2, 12, 21, 12, 7, 6)
    sp.part(body, R["rust"], "cyl_h")
    for x, y in ((5, 10), (8, 13), (11, 10), (14, 13), (17, 10), (6, 14), (13, 9)):
        sp.plot(x, y, "#e69d69")
    for a in range(5):
        sp.plot(22, 9 + a * 1 + 1, "#eab798")
    sp.outline()
    return sp


def flounder():
    sp = Sprite(N, N)   # top view of a flatfish on sand, head right
    sp.part(sp.poly([(4, 12), (0, 8), (0, 16)]), R["sand"], "rim")
    body = sp.ellipse(3, 4, 21, 20)
    sp.part(body, R["sand"], "rim")
    for x, y in ((8, 8), (12, 15), (15, 9), (9, 14), (17, 13), (6, 11)):
        sp.fill(sp.rect(x, y, x + 1, y), "#725244")
    sp.part(sp.rect(4, 12, 20, 12) & body, ["#ab8465"], "flat", seam=False)
    sp.plot(17, 9, EYE)
    sp.plot(18, 11, EYE)
    sp.outline()
    return sp


def octopus(col="coral"):
    sp = Sprite(N, N)   # top view, mantle right, arms trailing and curling
    c = R[col]
    for i, a in enumerate((-70, -40, -15, 10, 35, 60, 85, -95)):
        ang = math.radians(180 + a * 0.9)
        pts = [(12, 12)]
        for k in (4, 7, 9):
            ang2 = ang + math.radians(15 * (1 if i % 2 else -1)) * (k / 9)
            pts.append((int(12 + k * math.cos(ang2)), int(12 + k * math.sin(ang2))))
        sp.part(sp.line(pts, 2), dark(c), "flat")
    sp.part(sp.ellipse(11, 6, 22, 18), c, "sphere")
    for x, y in ((15, 9), (18, 12), (15, 15)):
        sp.plot(x, y, c[0])
    sp.plot(12, 9, EYE)
    sp.plot(12, 15, EYE)
    sp.plot(12, 8, WHITE)
    sp.outline()
    return sp


def clam():
    sp = Sprite(N, N)   # three-quarter view: two wavy shells, the mantle showing
    sp.part(sp.ellipse(2, 10, 21, 21), R["bone"], "sphere")
    for x in (5, 9, 13, 17):
        sp.part(sp.rect(x, 12, x, 20), ["#a09f91"], "flat", seam=False)
    sp.part(sp.ellipse(3, 8, 20, 13), R["blue"][1:] + ["#d2f8ff"], "rim")   # the blue mantle
    for x in (6, 10, 14, 17):
        sp.plot(x, 10, "#74effe")
    sp.part(sp.ellipse(2, 3, 21, 10) & ~sp.ellipse(3, 8, 20, 13), dark(R["bone"]), "sphere")
    sp.part(sp.circle(11, 10, 1), R["white"], "rim", seam=False)   # a pearl
    sp.outline()
    return sp


def abalone():
    sp = Sprite(N, N)
    sp.part(sp.ellipse(2, 5, 21, 19), R["kelp"], "sphere")
    for r in (8, 5, 2):
        sp.part(sp.ellipse(12 - r, 12 - r // 2 - 1, 12 + r, 12 + r // 2 + 1) & ~sp.ellipse(13 - r, 12 - r // 2, 11 + r, 12 + r // 2), ["#725244"], "flat", seam=False)
    for x in (5, 8, 11, 14, 17):
        sp.plot(x, 7 + (x % 3), "#84e3b5")
    sp.outline()
    return sp


def mussel(col="ink"):
    sp = Sprite(N, N)
    c = R["night"] if col == "ink" else R[col]
    for (x0, y0, x1, y1) in ((2, 8, 12, 14), (11, 5, 21, 11), (9, 13, 19, 19)):
        sp.part(sp.ellipse(x0, y0, x1, y1), c, "sphere")
        sp.plot(x0 + 2, y0 + 1, R["glass"][3])
    sp.outline()
    return sp


def tubeworm():
    sp = Sprite(N, N)
    for x, top in ((5, 7), (11, 3), (17, 9)):
        sp.part(sp.rect(x, top + 4, x + 3, 23), R["white"], "cyl_v")
        for y in range(top + 6, 23, 3):
            sp.part(sp.rect(x, y, x + 3, y), ["#92a3b2"], "flat", seam=False)
        sp.part(sp.ellipse(x - 2, top, x + 5, top + 5), R["red"], "sphere")
    sp.outline()
    return sp


def sparkle():
    sp = Sprite(N, N)
    for x, y, r in ((6, 7, 2), (15, 5, 1), (12, 13, 3), (5, 17, 1), (18, 17, 2), (20, 10, 1)):
        sp.part(sp.circle(x, y, r), R["cyan"], "inset", glow=True)
        sp.plot(x, y, WHITE, glow=True)
    return sp


# ---------- plants and reef pieces (cards and tiles) -------------------------------------------

def kelp_card():
    """16 x 64: one kelp frond, a stipe with blades and gas floats, hard alpha."""
    sp = Sprite(16, 64)
    sp.part(sp.line([(8, 63), (7, 48), (9, 32), (7, 16), (8, 0)], 2), R["kelp"], "flat")
    for i, y in enumerate(range(4, 62, 7)):
        s = 1 if i % 2 else -1
        x0 = 8 + s * 1
        sp.part(sp.taper(x0, y, x0 + s * 7, y + 5, 5, 2), R["kelp"][1:] + ["#9bb675"] if i % 3 else R["kelpg"], "rim")
        sp.part(sp.circle(x0, y + 1, 1), R["orange"], "rim", seam=False)
    return sp


def grass_card(kind):
    sp = Sprite(32, 32)
    if kind == "seagrass":
        for i, x in enumerate(range(2, 31, 4)):
            sp.part(sp.line([(x, 31), (x + (i % 3) - 1, 18), (x + (i % 2) * 2 - 1, 4 + (i * 5) % 8)], 2), R["kelpg"], "flat")
    elif kind == "reed":
        for i, x in enumerate(range(1, 31, 3)):
            sp.part(sp.line([(x, 31), (x + (i % 2), 2 + (i * 7) % 9)], 1), R["olive"] if i % 2 else R["khaki"], "flat")
        for x, y in ((4, 4), (13, 6), (22, 3), (28, 8)):
            sp.part(sp.rect(x, y, x + 1, y + 5), R["brown"], "cyl_v")      # cattail heads
    elif kind == "fan":   # a sea fan: a lattice fan on a stalk
        sp.part(sp.rect(15, 24, 16, 31), R["purple"], "flat")
        m = sp.ellipse(2, 2, 29, 27) & ~sp.rect(0, 26, 31, 31)
        net = sp.empty()
        for x in range(2, 30, 3):
            net |= sp.line([(16, 26), (x, 2)], 1)
        for r in (8, 14, 20):
            net |= sp.ellipse(16 - r, 26 - r, 16 + r, 26 + r) & ~sp.ellipse(17 - r, 27 - r, 15 + r, 25 + r)
        sp.part(m & net, R["purple"][1:] + ["#c34988"], "flat")
    elif kind == "soft":
        for i, x in enumerate((6, 12, 18, 24)):
            sp.part(sp.line([(16, 31), (x, 14 - (i % 2) * 4)], 3), R["pink"], "flat")
            sp.part(blob(sp, [(x, 12 - (i % 2) * 4, 3), (x + 2, 9 - (i % 2) * 4, 2)]), R["pink"][1:] + ["#fffefe"], "sphere")
    elif kind == "anemone":
        for i in range(13):
            x = 3 + i * 2
            sp.part(sp.line([(16, 26), (x, 8 + (i % 3) * 3)], 2), R["coral"][1:] + ["#fffefe"], "flat")
        sp.part(sp.ellipse(7, 24, 25, 31), R["red"], "rim")
    elif kind == "staghorn":
        def branch(x, y, a, L, d):
            if d == 0 or L < 2:
                return sp.empty()
            x2, y2 = x + L * math.cos(a), y - L * math.sin(a)
            m = sp.line([(int(x), int(y)), (int(x2), int(y2))], 2)
            return m | branch(x2, y2, a + 0.45, L * 0.7, d - 1) | branch(x2, y2, a - 0.5, L * 0.72, d - 1)
        sp.part(branch(16, 31, math.pi / 2, 10, 4), R["sand"][1:] + ["#fffefe"], "flat")
    elif kind == "lily":
        sp.part(sp.circle(16, 16, 13) & ~sp.poly([(16, 16), (31, 10), (31, 20)]), R["kelpg"], "rim")
        sp.part(sp.star(12, 14, 5, 2, 6), R["pink"][1:] + ["#fffefe"], "sphere")
        sp.part(sp.circle(12, 14, 1), R["yellow"], "rim", seam=False)
    elif kind == "mangrove":
        for x in (4, 10, 17, 24, 29):
            sp.part(sp.line([(16, 6), (x, 31)], 2), R["brown"], "flat")
        sp.part(sp.rect(14, 0, 17, 10), R["brown"], "cyl_v")
    return sp


def tile(kind):
    """32 x 32 seamless tiles for the reef, the vents and the dam."""
    sp = Sprite(32, 32)
    full = sp.rect(0, 0, 31, 31)
    if kind == "brain":
        sp.fill(full, "#e69d69")
        for y in range(0, 32, 4):
            pts = [(x, y + int(1.5 * math.sin((x + y * 3) / 3.0))) for x in range(0, 33, 2)]
            sp.fill(sp.line(pts, 1), "#a8633e")
            pts2 = [(x, y + 2 + int(1.5 * math.sin((x + y * 3) / 3.0))) for x in range(0, 33, 2)]
            sp.fill(sp.line(pts2, 1), "#eab798")
    elif kind == "table":
        sp.fill(full, "#53998b")
        for y in range(0, 32, 3):
            for x in range((y * 5) % 3, 32, 3):
                sp.plot(x, y, "#84e3b5")
                sp.plot((x + 1) % 32, y, "#204f63")
    elif kind == "branch":
        sp.fill(full, "#cfaa81")
        for y in range(0, 32, 2):
            for x in range((y // 2) % 2, 32, 4):
                sp.plot(x, y, "#eab798")
                sp.plot((x + 2) % 32, (y + 1) % 32, "#ab8465")
    elif kind == "sponge":
        sp.fill(full, "#fb902f")
        for (x, y) in ((4, 4), (14, 6), (24, 3), (8, 16), (20, 15), (28, 22), (3, 26), (15, 26)):
            sp.fill(sp.circle(x, y, 2), "#8d4526")
            sp.plot(x - 1, y - 2, "#faaf5c")
    elif kind == "mat":     # a bacterial mat at the vents
        sp.fill(full, "#cfaa81")
        for i in range(90):
            h = (i * 2654435761) & 0xffffffff
            x, y = h % 32, (h >> 8) % 32
            sp.plot(x, y, "#fffefe" if i % 3 else "#faaf5c")
        sp.dither(sp.rect(0, 20, 31, 31), "#cfaa81", "#eab798", 0.3)
    elif kind == "mussels":
        sp.fill(full, "#2c2c25")
        for j in range(0, 32, 6):
            for i in range(0, 32, 5):
                x, y = (i + (j // 6) * 2) % 32, j
                sp.fill(sp.ellipse(x, y, x + 3, y + 4) & full, "#1f3b52")
                sp.plot(x + 1, y + 1, "#53998b")
    elif kind == "tube":
        sp.fill(full, "#bfcad7")
        for y in range(0, 32, 5):
            sp.fill(sp.rect(0, y, 31, y), "#92a3b2")
            sp.fill(sp.rect(0, y + 1, 31, y + 1), "#fffefe")
    elif kind == "plume":
        sp.fill(full, "#a61e1f")
        for x in range(0, 32, 3):
            sp.fill(sp.rect(x, 0, x, 31), "#d12d41")
        for x in range(1, 32, 6):
            sp.fill(sp.rect(x, 0, x, 31), "#fa3a64")
    elif kind == "sticks":   # a beaver dam's or lodge's face: stripped sticks in mud
        sp.fill(full, "#41322c")
        for i in range(16):
            y = (i * 7) % 32
            x = (i * 11) % 32
            a = (i % 5 - 2) * 0.25
            pts = [(x - 10, int(y - 10 * a)), (x + 10, int(y + 10 * a))]
            sp.fill(sp.line(pts, 2) & full, "#ab8465" if i % 3 else "#cfaa81")
            sp.fill(sp.line([(p[0], p[1] + 1) for p in pts], 1) & full, "#725244")
            # wrap
            sp.fill(sp.line([(pts[0][0] + 32, pts[0][1]), (pts[1][0] + 32, pts[1][1])], 2) & full, "#ab8465")
    elif kind == "wake":
        sp.dither(full, "#bfcad7", "#fffefe", 0.5)
    return sp


# ---------- the roster ----------------------------------------------------------------------------

def sprites():
    out = {}
    out["darter"] = fish({"col": "orange", "fin": "orange", "box": (5, 8, 20, 17), "tail": "round",
                          "stripes": [9, 15], "sw": 1, "stripe": "white"})
    out["beakfish"] = fish({"col": "teal", "fin": "pink", "box": (4, 7, 20, 18), "tail": "lunate",
                            "beak": "white", "spots": [(8, 10), (11, 12), (14, 10), (9, 14)], "spot": "pink"})
    out["kelpfin"] = fish({"col": "orange", "fin": "orange", "box": (4, 7, 20, 17), "tail": "round",
                           "spots": [(9, 10), (12, 13)], "spot": "white"})
    out["glider"] = fish({"col": "sea", "fin": "sea", "box": (3, 11, 21, 15), "tail": "fork", "wings": True,
                          "belly": "white", "dorsal": False})
    out["redrun"] = fish({"col": "red", "fin": "rust", "box": (2, 8, 21, 16), "tail": "fork", "belly": "white",
                          "spots": [(6, 9), (9, 10), (12, 9), (15, 10)], "spot": "ink", "dh": 2})
    out["trout"] = fish({"col": "olive", "fin": "olive", "box": (2, 8, 21, 16), "tail": "fork", "belly": "sand",
                         "spots": [(6, 10), (9, 9), (12, 11), (15, 9), (8, 12)], "spot": "red", "lateral": "pink"})
    out["squid"] = squid()
    out["jelly"] = jelly()
    out["turtle"] = turtle()
    out["seal"] = seal()
    out["otter"] = otter("otter", back=True)
    out["riverotter"] = otter("fur", back=False)
    out["beaver"] = beaver()
    out["beaver_branch"] = beaver(branch=True)
    out["dabbler_drake"] = duck("drake")
    out["dabbler_hen"] = duck("hen")
    out["duckling"] = duck("duckling")
    out["dabbler_sleep"] = duck("sleep")
    out["dabbler_fly"] = duck("fly")
    out["diver"] = duck("diver")
    out["kingfisher"] = bird("kingfisher")
    out["dragonfly"] = dragonfly()
    out["frog"] = frog()
    out["crab"] = crustacean({"kind": "crab", "col": "coral"})
    out["fiddler"] = crustacean({"kind": "crab", "col": "orange", "fiddler": True})
    out["ventcrab"] = crustacean({"kind": "crab", "col": "white", "hairy": True})
    out["lobster"] = crustacean({"kind": "lobster", "col": "rust"})
    out["crayfish"] = crustacean({"kind": "crayfish", "col": "mud"})
    out["shrimp"] = crustacean({"kind": "shrimp", "col": "coral"})
    out["ventshrimp"] = crustacean({"kind": "shrimp", "col": "white", "glow": True})
    out["krill"] = crustacean({"kind": "krill", "col": "orange"})
    out["hermit"] = crustacean({"kind": "hermit", "col": "red"})
    out["starfish"] = starfish()
    out["urchin"] = urchin()
    out["cucumber"] = cucumber()
    out["flounder"] = flounder()
    out["octopus"] = octopus("coral")
    out["clam"] = clam()
    out["abalone"] = abalone()
    out["ventmussel"] = mussel("ink")
    out["rivermussel"] = mussel("brown")
    out["tubeworm"] = tubeworm()
    out["sparkle"] = sparkle()
    return out


def textures():
    out = {"card-kelp": kelp_card()}
    for k in ("seagrass", "reed", "fan", "soft", "anemone", "staghorn", "lily", "mangrove"):
        out["card-" + k] = grass_card(k)
    for k in ("brain", "table", "branch", "sponge", "mat", "mussels", "tube", "plume", "sticks", "wake"):
        out["tex-" + k] = tile(k)
    return out


def main():
    os.makedirs(OUT, exist_ok=True)
    items = []
    for name, sp in list(sprites().items()) + list(textures().items()):
        img = sp.image()
        save(img, os.path.join(OUT, name + ".png"))
        items.append((name, img))
    if "--sheet" in sys.argv:
        path = sys.argv[sys.argv.index("--sheet") + 1]
        sheet(items, scale=4, cols=8).save(path)
        print("sheet", path)
    print(len(items), "images ->", OUT)


if __name__ == "__main__":
    main()
