"""Animal portraits and drop icons for the animals mockup (docs/mockups/animals.html).

Pixel art drawn in code with the balatro-pixel-art kit
(.claude/skills/balatro-pixel-art/scripts/pixelkit.py), the style of the owner's solitaire
collection: a tiny native canvas, one bold subject, 3 to 5 tone ramps lit from the top left, dark
seams between parts, a 1 px outline, single-pixel glints, no anti-aliasing, Sweaft-64 colours only
(`save()` refuses anything else).

Portraits are 36 x 36 (the page shows them at 2x, 72 px); drops are 24 x 24 (1x and 2x). Each
species with a body in the 3D scene also gets a hide strip, `hide_<species>.png`: eight seamless
32 x 32 tiles (body with its coat pattern, legs, head, horns, tail, mane or wing, snout or beak,
ears or belly) that the scene's box-built creatures are textured with. Every
four-legged portrait is one function, `quad`, fed body-plan numbers, as the mockup's 3D creatures are.

    python tools/art/animals_art.py              # writes docs/mockups/animals/*.png
    python tools/art/animals_art.py --sheet S    # also a contact sheet (keep it out of the repo)
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
from pixelkit import Sprite, RAMPS, save, sheet  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "animals")
PW = 36   # portrait canvas
DW = 24   # drop canvas
GROUND = 33
WHITE = "#fffefe"
EYE = "#010000"

R = dict(RAMPS)
R.update({
    "face": ["#161612", "#2c2c25", "#434137", "#725244"],
    "wool": ["#a09f91", "#cfaa81", "#eab798", "#fffefe"],
    "frost": ["#5d778d", "#92a3b2", "#bfcad7", "#fffefe"],
    "snow": ["#92a3b2", "#bfcad7", "#fffefe"],
    "goat": ["#434137", "#a09f91", "#bfcad7", "#fffefe"],
    "boar": ["#2f1525", "#522b2f", "#7d4636", "#a8633e"],
    "hoof": ["#161612", "#2c2c25", "#434137"],
    "crab": ["#4a051e", "#a61e1f", "#d4462b", "#fe6420"],
    "shell": ["#733126", "#a8633e", "#e69d69", "#eab798"],
    "toad": ["#003227", "#05582d", "#297537", "#559642"],
    "croc": ["#003227", "#297537", "#559642", "#9bb675"],
    "teal": ["#0c2f44", "#204f63", "#53998b", "#84e3b5", "#c1f1d8"],
    "eagle": ["#2e0d06", "#41322c", "#725244", "#ab8465"],
    "comb": ["#4a051e", "#a61e1f", "#d12d41"],
    "yolk": ["#c56d25", "#fb902f", "#faaf5c"],
    "beak": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "lime": ["#05582d", "#559642", "#9bb675", "#c1f1d8"],
    "milk": ["#92a3b2", "#bfcad7", "#fffefe"],
    "meat": ["#4a051e", "#a61e1f", "#d4462b", "#dd576d"],
    "pink": ["#841c1d", "#dd576d", "#e69d69"],
})


def dark(ramp):
    """The far side of a thing: one ramp step darker."""
    return [ramp[0]] + ramp[:-1]


def eye(sp, x, y, glow=None):
    """A 2 x 2 eye, glint top left; glowing eyes are lit in their colour."""
    sp.fill(sp.rect(x, y, x + 1, y + 1), EYE)
    sp.plot(x, y, WHITE)
    if glow:
        sp.plot(x + 1, y, glow)


def blob(sp, circles):
    m = sp.empty()
    for cx, cy, r in circles:
        m |= sp.circle(cx, cy, r)
    return m


# ---------- four legs -------------------------------------------------------------------------

def quad(sp, P):
    """A four-legged body plan, facing left, light from the top left."""
    col = R[P["col"]]
    legc = R[P.get("legc", P["col"])]
    L = 9.5 * P.get("long", 1.0)
    H = 6.0 * P.get("tall", 1.0)
    leg = 8.0 * P.get("legs", 1.0)
    lw = P.get("lw", 3)
    cx = 20 + P.get("dx", 0)
    cy = GROUND - leg - H + 3
    fx, bx = cx - L * 0.55, cx + L * 0.5
    # tail first, behind everything
    tail = P.get("tail", "thin")
    tx, ty = cx + L - 1, cy - H * 0.4
    if tail == "bush":
        sp.part(sp.taper(tx - 2, ty, tx + 6, ty + 9, 6, 3), col, "rim")
        sp.part(sp.ellipse(tx + 4, ty + 9, tx + 8, ty + 13), R[P.get("tipc", "paper")], "rim")
    elif tail == "tuft":
        sp.part(sp.line([(tx, ty), (tx + 3, ty + 4), (tx + 3, ty + 9)], 2), col, "flat")
        sp.part(sp.ellipse(tx + 2, ty + 9, tx + 5, ty + 13), dark(dark(col)), "rim")
    elif tail == "stub":
        sp.part(sp.circle(tx + 1, ty, 2), R[P.get("tailc", P["col"])], "rim")
    elif tail == "long":
        sp.part(sp.line([(tx, ty + 1), (tx + 4, ty - 3), (tx + 6, ty - 9), (tx + 5, ty - 12)], 2), col, "flat")
    else:
        sp.part(sp.line([(tx, ty), (tx + 3, ty + 3), (tx + 4, ty + 8)], 2), dark(col), "flat")
    # far legs, a ramp step darker
    for x in (fx + 2, bx + 2):
        sp.part(sp.rect(x - lw // 2, cy, x - lw // 2 + lw - 1, GROUND - 1), dark(legc), "rim")
        if P.get("hoof"):
            sp.part(sp.rect(x - lw // 2, GROUND - 1, x - lw // 2 + lw - 1, GROUND - 1), R["hoof"], "flat")
    # body
    if P.get("wool"):
        # the fleece is a pile of round curls, each its own seamed ball, back to front
        curls = [(cx + dx * L, cy + dy * H, r * H) for dx, dy, r in
                 ((0.62, 0.25, 0.55), (0.0, 0.4, 0.6), (-0.55, 0.3, 0.55), (0.65, -0.3, 0.55), (0.2, -0.05, 0.6),
                  (-0.25, 0.0, 0.6), (0.3, -0.6, 0.55), (-0.2, -0.6, 0.55), (-0.65, -0.3, 0.55))]
        body = blob(sp, curls)
        for x, y, r in curls:
            sp.part(sp.circle(x, y, r), col, "sphere")
    else:
        body = sp.ellipse(cx - L, cy - H, cx + L, cy + H)
        sp.part(body, col, "sphere")
    if P.get("hump"):
        sp.part(sp.ellipse(cx - L * 0.35, cy - H * 1.55, cx + L * 0.35, cy - H * 0.2) & ~sp.rect(0, cy - H * 0.3, PW, PW), col, "sphere")
    if P.get("mane"):
        sp.part(sp.ellipse(cx - L * 1.05, cy - H * 1.2, cx - L * 0.1, cy + H * 0.7), R[P["mane"]], "sphere")
    pat = P.get("pattern")
    if pat:
        pc = P.get("patc", col[0])
        inner = body & ~sp.rect(0, 0, PW, cy - H + 1)
        if pat == "stripes":
            for i in range(-2, 3):
                x = round(cx + i * L * 0.33)
                sp.fill(sp.poly([(x, cy - H), (x + 1, cy - H), (x + 2, cy + H * 0.2), (x + 1, cy + H * 0.2)]) & body, pc)
        elif pat == "spots":
            for dx, dy, r in ((-0.35, -0.25, 1.6), (0.2, -0.45, 1.2), (0.5, 0.05, 1.5), (0.0, 0.3, 1.1), (-0.6, 0.25, 1.0)):
                m = sp.circle(cx + dx * L, cy + dy * H, r) & inner
                sp.fill(m, pc)
    # near legs
    for x in (fx, bx):
        sp.part(sp.rect(x - lw // 2, cy + H * 0.3, x - lw // 2 + lw - 1, GROUND - 1), legc, "rim")
        if P.get("hoof"):
            sp.part(sp.rect(x - lw // 2, GROUND - 1, x - lw // 2 + lw - 1, GROUND), R["hoof"], "flat")
        else:
            sp.part(sp.rect(x - lw // 2 - 1, GROUND - 1, x - lw // 2 + lw - 1, GROUND), legc, "rim")
    # neck and head
    hr = 4.6 * P.get("head", 1.0)
    neck = P.get("neck", 0.3)
    hx = cx - L - hr * 0.2
    hy = cy - H * 0.5 - neck * 12
    headc = R[P.get("headc", P["col"])]
    if neck > 0.05:
        sp.part(sp.poly([(cx - L * 0.45, cy - H * 0.7), (cx - L * 0.8, cy + H * 0.4), (hx + 1, hy + hr * 0.6), (hx + hr * 0.9, hy - hr * 0.3)]), col, "rim")
    ears = P.get("ears", "point")
    ec = R[P.get("earc", P.get("headc", P["col"]))]
    if ears == "long":
        sp.part(sp.ellipse(hx + 1, hy - hr * 2.6, hx + 3, hy - hr * 0.4), dark(ec), "rim")
        sp.part(sp.ellipse(hx - 2, hy - hr * 2.8, hx + 0, hy - hr * 0.4), ec, "rim")
    elif ears == "point":
        sp.part(sp.poly([(hx + 1, hy - hr * 0.5), (hx + 4, hy - hr * 0.5), (hx + 3, hy - hr * 1.6)]), dark(ec), "rim")
        sp.part(sp.poly([(hx - 2, hy - hr * 0.4), (hx + 1, hy - hr * 0.6), (hx - 1, hy - hr * 1.7)]), ec, "rim")
    elif ears == "round":
        sp.part(sp.circle(hx + 2, hy - hr * 0.8, 1.6), dark(ec), "flat")
        sp.part(sp.circle(hx - 1.5, hy - hr * 0.9, 1.6), ec, "rim")
    elif ears == "flop":
        sp.part(sp.ellipse(hx + 1, hy - 2, hx + 6, hy + 1), dark(ec), "rim")
    head = sp.ellipse(hx - hr, hy - hr * 0.85, hx + hr * 0.9, hy + hr * 0.85)
    sp.part(head, headc, "sphere")
    snout = P.get("snout", 0.6)
    if snout > 0:
        sl = 2 + snout * 3
        sn = sp.ellipse(hx - hr - sl + 1, hy - 0.5, hx - hr * 0.2, hy + hr * 0.75)
        sp.part(sn, R[P.get("snoutc", P.get("headc", P["col"]))], "rim")
        sp.plot(round(hx - hr - sl + 1), round(hy), EYE)
    else:
        sp.plot(round(hx - hr), round(hy + 1), EYE)
    eye(sp, round(hx - hr * 0.35), round(hy - hr * 0.4), P.get("glow_eyes"))
    horns = P.get("horns")
    hc = R[P.get("hornc", "bone")]
    if horns == "curl":
        sp.part(sp.line([(hx - 1, hy - hr * 0.6), (hx + 3, hy - hr * 1.3), (hx + 7, hy - hr * 1.1), (hx + 8, hy - hr * 0.2), (hx + 6, hy + 1)], 3), hc, "flat")
        sp.plots([(round(hx + 3), round(hy - hr * 1.3) - 1)], hc[-1])
    elif horns == "spike":
        sp.part(sp.poly([(hx + 0, hy - hr * 0.6), (hx + 3, hy - hr * 0.6), (hx + 4, hy - hr * 1.9)]), hc, "rim")
        sp.part(sp.poly([(hx - 3, hy - hr * 0.5), (hx - 1, hy - hr * 0.7), (hx - 4, hy - hr * 1.8)]), hc, "rim")
    elif horns == "tusk":
        sx, sy = hx - hr - 2, hy + hr * 0.4
        sp.part(sp.poly([(sx + 2, sy + 1), (sx - 1, sy - 3), (sx - 2, sy - 5), (sx + 0, sy - 3), (sx + 3, sy - 1)]), hc, "rim")
    elif horns == "droop":
        sp.part(sp.line([(hx - 3, hy - hr * 0.7), (hx + 3, hy - hr * 1.1), (hx + 7, hy - hr * 0.4), (hx + 6, hy + hr * 0.7)], 3), hc, "flat")


# ---------- birds, bugs and the odd ones ---------------------------------------------------------

def bird(sp, P):
    col = R[P["col"]]
    k = P.get("k", 1.0)
    cx, cy = 20, 19 - P.get("lift", 0)
    fly = P.get("fly", False)
    if not fly:
        lc = R[P.get("legc", "beak")]
        for dx in (-2, 2):
            sp.part(sp.line([(cx + dx, cy + 6 * k), (cx + dx - 1, GROUND)], 1), lc, "flat")
            sp.part(sp.rect(cx + dx - 3, GROUND, cx + dx, GROUND), lc, "flat")
    sp.part(sp.poly([(cx + 6 * k, cy - 2), (cx + 15 * k, cy + 2 * k), (cx + 14 * k, cy + 6 * k), (cx + 5 * k, cy + 4)]),
            R[P.get("tailc", P["col"])], "rim")
    if fly:
        sp.part(sp.poly([(cx + 2, cy - 2), (cx + 13 * k, cy - 14 * k), (cx + 8 * k, cy - 15 * k), (cx - 3, cy - 4)]), dark(R[P.get("wingc", P["col"])]), "rim")
    body = sp.ellipse(cx - 9 * k, cy - 6 * k, cx + 9 * k, cy + 6 * k)
    sp.part(body, col, "sphere")
    if P.get("belly"):
        sp.part(sp.ellipse(cx - 8 * k, cy, cx + 3 * k, cy + 6 * k) & body, R[P["belly"]], "rim")
    wc = R[P.get("wingc", P["col"])]
    if fly:
        # raised, the primaries fanned into three fingers at the tip
        wing = sp.poly([(cx - 4, cy - 3), (cx + 6, cy - 2), (cx + 16 * k, cy - 11 * k), (cx + 13 * k, cy - 11 * k), (cx + 15 * k, cy - 15 * k),
                        (cx + 11 * k, cy - 14 * k), (cx + 12 * k, cy - 18 * k), (cx + 8 * k, cy - 16 * k), (cx + 6 * k, cy - 19 * k), (cx + 1, cy - 11 * k)])
    else:
        wing = sp.poly([(cx - 4 * k, cy - 3 * k), (cx + 5 * k, cy - 4 * k), (cx + 12 * k, cy + 2 * k), (cx + 2 * k, cy + 4 * k)])
    sp.part(wing, wc, "rim")
    hx, hy = cx - 8 * k, cy - 7 * k
    if P.get("crest"):
        sp.part(sp.poly([(hx - 1, hy - 3 * k), (hx + 3 * k, hy - 8 * k), (hx + 2 * k, hy - 2 * k)]), R[P["crest"]], "rim")
    hr = 4.4 * k
    sp.part(sp.circle(hx, hy, hr), R[P.get("headc", P["col"])], "sphere")
    bl = P.get("beak_len", 1.0)
    hook = P.get("hook", 0)
    sp.part(sp.poly([(hx - hr + 1, hy - 2), (hx - hr - 5 * bl, hy + hook), (hx - hr + 1, hy + 1.5)]), R[P.get("beakc", "beak")], "rim")
    eye(sp, round(hx - 2), round(hy - 2))


def insect(sp, P):
    cx, cy = 15, 20
    ws = P.get("wing", 1.0)
    wc = R[P.get("wingc", "paper")]
    def wing(ox, oy):
        return sp.poly([(cx - 1 + ox, cy - 3 + oy), (cx + 2 + ox, cy - 12 * ws + oy), (cx + 8 * ws + ox, cy - 16 * ws + oy),
                        (cx + 12 * ws + ox, cy - 14 * ws + oy), (cx + 10 * ws + ox, cy - 8 * ws + oy), (cx + 4 + ox, cy - 2 + oy)])
    sp.part(wing(4, 1), dark(wc), "rim")
    for i, dx in enumerate((-3, 1, 5)):
        sp.part(sp.line([(cx + dx, cy + 3), (cx + dx - 2, cy + 8), (cx + dx - 3 + i, cy + 11)], 1), R["ink"], "flat")
    ab = sp.ellipse(cx + 2, cy - 3, cx + 18, cy + 6)
    glow = P.get("glow")
    sp.part(ab, R[P.get("abc", P["col"])], "sphere")
    if P.get("bands"):
        for i in range(3):
            x = cx + 7 + i * 3
            sp.fill(sp.rect(x, cy - 3, x, cy + 6) & ab, R[P["bands"]][0])
    sp.part(sp.ellipse(cx - 4, cy - 4, cx + 4, cy + 4), R[P["col"]], "sphere")
    sp.part(wing(0, 0), wc, "rim")
    sp.plot(cx + 4, round(cy - 12 * ws), wc[-1])
    sp.part(sp.circle(cx - 6, cy - 1, 3), dark(R[P["col"]]), "sphere")
    for dx in (-7, -5):
        sp.part(sp.line([(cx + dx, cy - 3), (cx + dx - 3, cy - 8), (cx + dx - 5, cy - 9)], 1), R["ink"], "flat")
    eye(sp, cx - 8, cy - 2)
    outline(sp)
    if glow:
        g = R[glow]
        # the lit tail glows over the outline, a dithered halo round it
        halo = sp.ellipse(cx + 6, cy - 2, cx + 20, cy + 8) & ~sp.alpha
        sp.dither(halo, g[1], g[2], 0.35, glow=True)
        sp.fill(sp.ellipse(cx + 9, cy, cx + 17, cy + 5), g[-2], glow=True)
        sp.fill(sp.ellipse(cx + 11, cy + 1, cx + 15, cy + 3), g[-1], glow=True)
        for x, y in ((31, 6), (4, 30), (29, 31)):
            spark(sp, x, y, g)


def outline(sp):
    sp.outline()
    sp.outlined = True


def spark(sp, x, y, ramp):
    """A white pixel in a plus of the palest tone: light, so never outlined."""
    sp.plots([(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)], ramp[-2], glow=True)
    sp.plot(x, y, WHITE, glow=True)


def scarab(sp, P):
    col = R[P["col"]]
    cx, cy = 18, 20
    for i in range(3):
        for s in (-1, 1):
            y0 = cy - 4 + i * 5
            sp.part(sp.line([(cx + s * 6, y0), (cx + s * 11, y0 + 2), (cx + s * 13, y0 + 6)], 2), R["soot"], "flat")
    shell = sp.ellipse(cx - 9, cy - 7, cx + 9, cy + 13)
    sp.part(shell, col, "sphere")
    sp.fill(sp.rect(cx, cy - 6, cx, cy + 12) & shell, col[0])
    sp.part(sp.ellipse(cx - 6, cy - 12, cx + 6, cy - 5), dark(col), "sphere")
    sp.part(sp.poly([(cx - 2, cy - 12), (cx, cy - 18), (cx + 2, cy - 12)]), R["gold"], "rim")
    sp.plots([(cx - 5, cy - 3), (cx - 4, cy - 4)], col[-1])
    sp.plot(cx - 5, cy - 4, WHITE)
    outline(sp)
    spark(sp, 30, 5, R["gold"])


def crab(sp, P):
    col = R[P["col"]]
    cx, cy = 18, 23
    for s in (-1, 1):
        for i in range(3):
            x0 = cx + s * (6 + i * 2)
            sp.part(sp.line([(x0, cy + 1), (x0 + s * 4, cy + 4 + i), (x0 + s * 5, GROUND)], 2), dark(col), "flat")
        sp.part(sp.line([(cx + s * 6, cy - 2), (cx + s * 11, cy - 7)], 3), col, "flat")
        claw = sp.poly([(cx + s * 9, cy - 7), (cx + s * 15, cy - 15), (cx + s * 17, cy - 9), (cx + s * 13, cy - 5)])
        sp.part(claw, col, "sphere")
        sp.part(sp.poly([(cx + s * 13, cy - 12), (cx + s * 13, cy - 19), (cx + s * 16, cy - 14)]), col, "rim")
    sp.part(sp.ellipse(cx - 11, cy - 6, cx + 11, cy + 5), col, "sphere")
    for s in (-1, 1):
        sp.part(sp.rect(cx + s * 3 - (1 if s < 0 else 0), cy - 10, cx + s * 3 + (0 if s < 0 else 1) , cy - 6), dark(col), "flat")
        eye(sp, cx + s * 3 - (1 if s < 0 else 0), cy - 12)


def toad(sp, P):
    col = R[P["col"]]
    cx, cy = 18, 24
    for s in (-1, 1):
        sp.part(sp.ellipse(cx + s * 11 - 5, GROUND - 3, cx + s * 11 + 5, GROUND), dark(col), "rim")
    body = sp.ellipse(cx - 14, cy - 8, cx + 14, cy + 8)
    sp.part(body, col, "sphere")
    sp.part(sp.ellipse(cx - 9, cy + 2, cx + 9, cy + 8) & body, R["lime"], "rim")
    for s in (-1, 1):
        sp.part(sp.circle(cx + s * 6, cy - 9, 3.5), col, "sphere")
        sp.fill(sp.rect(cx + s * 6 - 1, cy - 10, cx + s * 6 + 1, cy - 8), R["gold"][3])
        sp.fill(sp.rect(cx + s * 6, cy - 10, cx + s * 6, cy - 8), EYE)
        sp.plot(cx + s * 6 - 1, cy - 10, WHITE)
    sp.fill(sp.line([(cx - 6, cy - 2), (cx, cy - 1), (cx + 6, cy - 2)], 1), col[0])
    outline(sp)
    g = R[P["glow"]]
    for dx, dy in ((-9, -3), (-3, -5), (5, -5), (10, -1), (0, 1)):
        sp.fill(sp.rect(cx + dx, cy + dy, cx + dx + 1, cy + dy + 1), g[-2], glow=True)
        sp.plot(cx + dx, cy + dy, WHITE, glow=True)


def croc(sp, P):
    col = R[P["col"]]
    cy = 24
    for x in (11, 24):
        sp.part(sp.rect(x, cy, x + 3, GROUND), dark(col), "rim")
    body = sp.poly([(1, cy - 1), (8, cy - 5), (24, cy - 6), (30, cy - 3), (35, cy + 1), (30, cy + 2), (24, cy + 4), (8, cy + 4), (1, cy + 1)])
    sp.part(body, col, "cyl_h")
    for i in range(7):
        x = 11 + i * 3
        sp.part(sp.poly([(x - 1, cy - 5), (x, cy - 8), (x + 1, cy - 5)]), dark(col), "flat")
    sp.part(sp.ellipse(8, cy - 8, 13, cy - 3), col, "sphere")
    sp.fill(sp.rect(10, cy - 7, 11, cy - 6), R["gold"][3])
    sp.fill(sp.rect(11, cy - 7, 11, cy - 6), EYE)
    sp.plot(10, cy - 7, WHITE)
    for x in range(3, 9, 2):
        sp.plot(x, cy + 1, WHITE)
    sp.fill(sp.line([(1, cy), (9, cy)], 1), col[0])
    for x in (12, 25):
        sp.part(sp.rect(x - 1, GROUND, x + 4, GROUND), dark(col), "flat")


# ---------- drops (24 x 24) --------------------------------------------------------------------

def d_wool(sp):
    for x, y, r in ((6, 17, 3.5), (18, 17, 3.5), (12, 17, 4.5), (7, 11, 4.5), (17, 11, 4.5), (12, 8, 5)):
        sp.part(sp.circle(x, y, r), R["wool"], "sphere")
    sp.plot(10, 5, WHITE)


def d_hide(sp):
    m = sp.poly([(3, 5), (8, 3), (12, 6), (16, 3), (21, 5), (19, 12), (21, 20), (16, 20), (12, 22), (8, 20), (2, 20), (5, 12)])
    sp.part(m, R["rust"], "rim", depth=2)
    for x, y, r in ((8, 10, 1.6), (15, 11, 1.4), (11, 16, 1.6)):
        sp.fill(sp.circle(x, y, r) & m, R["rust"][1])


def d_meat(sp):
    sp.part(sp.line([(13, 13), (19, 19)], 3), R["bone"], "flat")
    sp.part(sp.circle(20, 19, 2) | sp.circle(18, 21, 2), R["bone"], "rim")
    m = sp.ellipse(2, 3, 16, 15)
    sp.part(m, R["meat"], "sphere")
    sp.part(sp.ellipse(5, 6, 11, 11), R["pink"], "rim")
    sp.plots([(5, 5), (6, 5)], WHITE)


def d_egg(sp):
    m = sp.poly([(12 + 7 * math.sin(t) * (1 - 0.2 * math.cos(t)), 12.5 - 10 * math.cos(t)) for t in [2 * math.pi * i / 40 for i in range(40)]])
    sp.part(m, R["sand"], "sphere")
    for x, y in ((9, 8), (14, 12), (11, 17), (15, 6)):
        sp.plot(x, y, R["sand"][0])
    sp.plots([(8, 5), (9, 4)], WHITE)


def d_feather(sp):
    vane = sp.poly([(4, 21), (6, 15), (11, 8), (18, 2), (20, 3), (17, 10), (11, 17)])
    sp.part(vane, R["blue"], "rim")
    for i in range(4):
        x, y = 9 + i * 3, 16 - i * 4
        sp.plots([(x + 1, y + 1), (x + 2, y + 2)], R["blue"][0])
    sp.fill(sp.line([(3, 22), (19, 3)], 1), R["paper"][2])
    outline(sp)


def d_glow(sp):
    sp.part(sp.rect(7, 3, 16, 5), R["wood"], "rim")
    jar = sp.poly([(6, 6), (17, 6), (19, 9), (19, 21), (4, 21), (4, 9)])
    sp.part(jar, R["glass"], "rim")
    sp.fill(sp.rect(7, 10, 8, 18), R["glass"][-1])
    outline(sp)
    g = R["lime"]
    for x, y in ((11, 12), (15, 15), (12, 18), (15, 10)):
        sp.fill(sp.rect(x, y, x + 1, y + 1), g[-2], glow=True)
        sp.plot(x, y, WHITE, glow=True)
    spark(sp, 21, 2, R["lime"])


def d_milk(sp):
    sp.part(sp.line([(4, 8), (6, 3), (12, 1), (18, 3), (20, 8)], 1), R["soot"], "flat")
    pail = sp.poly([(3, 8), (20, 8), (18, 21), (5, 21)])
    sp.part(pail, R["metal"], "cyl_v")
    for y in (11, 18):
        sp.fill(sp.rect(3, y, 20, y) & pail, R["metal"][0])
    sp.part(sp.ellipse(4, 6, 19, 10), R["milk"], "rim")
    sp.plot(7, 7, WHITE)


def d_horn(sp):
    m = sp.poly([(3, 21), (5, 14), (10, 8), (16, 5), (21, 5), (21, 8), (16, 9), (11, 13), (7, 21)])
    sp.part(m, R["bone"], "rim", depth=2)
    for t in (0.3, 0.5, 0.7):
        x, y = 4 + 15 * t, 20 - 14 * t
        sp.fill(sp.line([(x - 1, y - 2), (x + 1, y + 1)], 1) & m, R["bone"][1])
    sp.plot(19, 6, WHITE)


def d_shell(sp):
    fan = sp.poly([(12, 21)] + [(12 + 10 * math.cos(a), 14 - 11 * math.sin(a)) for a in [math.pi * (0.06 + 0.88 * i / 20) for i in range(21)]])
    sp.part(fan, R["shell"], "sphere")
    for i in range(5):
        a = math.pi * (0.2 + 0.6 * i / 4)
        sp.fill(sp.line([(12, 20), (12 + 9 * math.cos(a), 14 - 10 * math.sin(a))], 1) & fan, R["shell"][1])
    sp.part(sp.rect(9, 19, 15, 22), R["shell"], "rim")
    sp.plot(8, 6, WHITE)


# ---------- the roster ------------------------------------------------------------------------

def Q(**p):
    return (quad, p)


PORTRAITS = {
    # fields
    "woolback": Q(col="wool", wool=True, legc="face", headc="face", earc="face", ears="flop", tail="stub", legs=0.7, tall=1.15, snout=0.3),
    "plodder": Q(col="rust", pattern="spots", patc="#fffefe", horns="spike", tail="tuft", legs=0.75, long=1.15, tall=1.15, hoof=1, snout=0.7, ears="flop", lw=4),
    "hen": (bird, dict(col="brass", wingc="rust", tailc="soot", crest="comb", beak_len=0.5, k=1.0)),
    "meadow_hare": Q(col="wood", ears="long", tail="stub", tailc="paper", legs=0.5, long=0.75, tall=1.0, snout=0.0, head=1.0, dx=2),
    "red_fox": Q(col="brass", ears="point", tail="bush", legc="ink", legs=0.7, long=1.0, tall=0.85, snout=1.0, snoutc="bone", dx=-1),
    "lark": (bird, dict(col="wood", belly="sand", crest="wood", beak_len=0.5, k=0.9, fly=True)),
    "firefly": (insect, dict(col="soot", abc="gold", glow="lime", wing=0.8)),
    # desert
    "sand_strider": Q(col="sand", hump=True, neck=0.8, legs=1.15, long=0.9, tall=0.85, tail="tuft", ears="round", snout=0.8, head=0.9, hoof=1, dx=2),
    "dust_hound": Q(col="sand", ears="point", pattern="spots", patc="#725244", tail="thin", legs=0.85, tall=0.8, snout=1.0, glow_eyes="#faaf5c"),
    "scarab": (scarab, dict(col="teal")),
    # jungle
    "stripe_cat": Q(col="brass", pattern="stripes", patc="#161612", tail="long", ears="round", legs=0.7, long=1.05, tall=0.8, snout=0.3, glow_eyes="#84e3b5", dx=-2),
    "tusk_boar": Q(col="boar", mane="boar", horns="tusk", hornc="bone", tail="thin", legs=0.6, tall=1.05, snout=1.0, ears="point", dx=1),
    "parrot": (bird, dict(col="green", wingc="blue", tailc="red", headc="gold", beakc="paper", beak_len=0.6, hook=3)),
    "glowmoth": (insect, dict(col="violet", abc="violet", wingc="cyan", glow="cyan", wing=1.1, bands="violet")),
    # swamp
    "bog_toad": (toad, dict(col="toad", glow="glass")),
    "heron": (bird, dict(col="frost", wingc="metal", headc="snow", beak_len=1.5, k=0.8, legc="gold", crest="night", lift=4)),
    "mire_lurker": (croc, dict(col="croc")),
    # mountains
    "ridge_goat": Q(col="goat", horns="curl", hornc="wood", tail="stub", legs=0.85, long=0.95, snout=0.6, hoof=1, ears="point", headc="snow"),
    "crag_eagle": (bird, dict(col="eagle", wingc="eagle", headc="paper", beakc="gold", beak_len=0.6, hook=3, fly=True)),
    # tundra
    "shagback": Q(col="wood", wool=True, headc="eagle", horns="droop", hornc="bone", hump=True, legs=0.55, long=1.1, tall=1.15, snout=0.5, tail="stub", tailc="eagle", ears=None, lw=4),
    "pale_wolf": Q(col="frost", ears="point", tail="bush", tipc="metal", legs=0.8, tall=0.85, snout=1.1, glow_eyes="#74effe", dx=-1),
    "snow_hare": Q(col="snow", ears="long", tail="stub", legs=0.5, long=0.75, tall=1.0, snout=0.0, head=1.0, dx=2),
    # beach
    "shore_crab": (crab, dict(col="crab")),
    "gull": (bird, dict(col="snow", wingc="metal", tailc="soot", beak_len=0.8)),
}

DROPS = {
    "drop_wool": d_wool, "drop_hide": d_hide, "drop_meat": d_meat, "drop_egg": d_egg, "drop_feather": d_feather,
    "drop_glow": d_glow, "drop_milk": d_milk, "drop_horn": d_horn, "drop_shell": d_shell,
}


# ---------- hides: the 3D creatures' textures ----------------------------------------------------
# One strip a species, eight 32 x 32 tiles that repeat seamlessly, a tile a metre on the 3D boxes:
#   0 body (its coat pattern baked in), 1 legs, 2 head, 3 horns / crest / eyes, 4 tail,
#   5 mane, wing or tail tip, 6 snout or beak, 7 ears or belly.
# The JS picks the tile by index; a seed varies each animal's tint and the pattern's scale.

import numpy as np  # noqa: E402
from PIL import Image  # noqa: E402

T = 32
HIDE_CELLS = 8


def _rgb(h):
    return np.array([int(h[i:i + 2], 16) for i in (1, 3, 5)], np.uint8)


def _tile(color):
    t = np.zeros((T, T, 3), np.uint8)
    t[:] = _rgb(color)
    return t


def _put(t, x, y, color):
    t[y % T, x % T] = _rgb(color)


def _disc(t, cx, cy, r, color):
    for y in range(int(cy - r) - 1, int(cy + r) + 2):
        for x in range(int(cx - r) - 1, int(cx + r) + 2):
            if (x - cx) ** 2 + (y - cy) ** 2 <= r * r:
                _put(t, x, y, color)


def tex_fur(ramp, seed):
    """Short strokes of the shadow and light tones on the body tone, lying along the body (x)."""
    rs = np.random.RandomState(seed)
    n = len(ramp)
    base, dk, lt = ramp[n - 2], ramp[max(n - 3, 0)], ramp[n - 1]
    t = _tile(base)
    for _ in range(46):
        x, y = rs.randint(T), rs.randint(T)
        for i in range(rs.randint(2, 4)):
            _put(t, x + i, y, dk)
    for _ in range(22):
        x, y = rs.randint(T), rs.randint(T)
        for i in range(rs.randint(1, 3)):
            _put(t, x + i, y, lt)
    return t


def tex_wool(ramp, seed):
    """Round curls in rows, each lit top left with a dark seam, like the portraits' fleece."""
    n = len(ramp)
    t = _tile(ramp[max(n - 3, 0)])
    for row in range(4):
        for col in range(4):
            cx, cy = col * 8 + (4 if row % 2 else 0) + 3.5, row * 8 + 3.5
            _disc(t, cx, cy, 3.6, ramp[0])
            _disc(t, cx, cy, 2.9, ramp[n - 2])
            _put(t, int(cx) - 1, int(cy) - 1, ramp[n - 1])
            _put(t, int(cx), int(cy) - 2, ramp[n - 1])
            _put(t, int(cx) + 1, int(cy) + 2, ramp[max(n - 3, 0)])
    return t


def tex_feather(ramp, seed):
    """Overlapping scallops in offset rows, the trailing edge dark and the leading pixel light."""
    n = len(ramp)
    t = _tile(ramp[n - 2])
    for row in range(T // 4):
        y = row * 4 + 3
        for col in range(T // 8):
            x0 = col * 8 + (4 if row % 2 else 0)
            for dx, dy in ((0, -1), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, -1)):
                _put(t, x0 + dx, y + dy, ramp[max(n - 3, 0)])
            _put(t, x0 + 2, y - 2, ramp[n - 1])
    return t


def tex_scale(ramp, seed):
    """A diamond lattice of scales, each with a lit pixel top left."""
    n = len(ramp)
    t = _tile(ramp[n - 2])
    for y in range(T):
        for x in range(T):
            if (x + y) % 8 == 0 or (x - y) % 8 == 0:
                _put(t, x, y, ramp[max(n - 3, 0)])
    for y in range(0, T, 8):
        for x in range(0, T, 8):
            _put(t, x + 4, y + 2, ramp[n - 1])
    return t


def tex_shell(ramp, seed):
    """Smooth shell: broad bands a tone apart, a few dark pits."""
    rs = np.random.RandomState(seed)
    n = len(ramp)
    t = _tile(ramp[n - 2])
    for y in range(T):
        if y % 8 == 0:
            t[y, :] = _rgb(ramp[n - 1])
        if y % 8 == 7:
            t[y, :] = _rgb(ramp[max(n - 3, 0)])
    for _ in range(8):
        _put(t, rs.randint(T), rs.randint(T), ramp[0])
    return t


def tex_horn(ramp, seed):
    """Ringed horn, beak or leg: a dark ring every 4 px, a lit one every 8."""
    n = len(ramp)
    t = _tile(ramp[n - 2])
    for y in range(0, T, 4):
        t[y, :] = _rgb(ramp[max(n - 3, 0)])
        if y % 8 == 0:
            t[(y + 1) % T, :] = _rgb(ramp[n - 1])
    return t


def pat_spots(t, color, seed):
    rs = np.random.RandomState(seed + 7)
    for _ in range(5):
        cx, cy, r = rs.randint(T), rs.randint(T), 2.5 + rs.rand() * 2
        for ox in (-T, 0, T):
            for oy in (-T, 0, T):
                _disc(t, cx + ox, cy + oy, r, color)
    return t


def pat_stripes(t, color, seed):
    for x0 in (2, 12, 22):
        for y in range(T):
            w = 3 if (y // 5) % 2 else 2
            off = int(round(1.5 * math.sin(y / 5.0 * math.pi * 2 / 6.4)))
            for i in range(w):
                _put(t, x0 + off + i, y, color)
    return t


def hide_strip(name, fn, P):
    """The eight tiles of one species, as the 3D builder in animals.html reads them."""
    seed = sum(map(ord, name))

    def g(k, d):
        return P.get(k) or d

    col = P["col"]
    if fn is quad:
        body = (tex_wool if P.get("wool") else tex_fur)(R[col], seed)
        if P.get("pattern") == "spots":
            pat_spots(body, P.get("patc", R[col][0]), seed)
        elif P.get("pattern") == "stripes":
            pat_stripes(body, P.get("patc", R[col][0]), seed)
        tiles = [body, tex_fur(R[g("legc", col)], seed + 1), tex_fur(R[g("headc", col)], seed + 2), tex_horn(R[g("hornc", "bone")], seed),
                 tex_fur(R[g("tailc", col)], seed + 4), tex_fur(R[g("mane", g("tipc", col))], seed + 5),
                 tex_fur(R[g("snoutc", g("headc", col))], seed + 6), tex_fur(R[g("earc", g("headc", col))], seed + 7)]
    elif fn is bird:
        tiles = [tex_feather(R[col], seed), tex_horn(R[g("legc", "beak")], seed), tex_feather(R[g("headc", col)], seed + 2),
                 tex_fur(R[g("crest", "comb")], seed + 3), tex_feather(R[g("tailc", col)], seed + 4), tex_feather(R[g("wingc", col)], seed + 5),
                 tex_horn(R[g("beakc", "beak")], seed), tex_feather(R[g("belly", col)], seed + 7)]
    elif fn is toad:
        body = pat_spots(tex_scale(R[col], seed), R[col][0], seed)
        tiles = [body, tex_fur(R[col], seed + 1), body, tex_horn(R["gold"], seed), body, body, body, tex_fur(R["lime"], seed + 7)]
    elif fn is croc:
        body = tex_scale(R[col], seed)
        tiles = [body, tex_scale(dark(R[col]), seed + 1), body, tex_horn(R["gold"], seed), body, body, body, tex_fur(R["lime"], seed + 7)]
    else:  # crab, scarab: a shell
        body = tex_shell(R[col], seed)
        tiles = [body, tex_shell(dark(R[col]), seed + 1), body, tex_horn(R["paper"], seed), body, body, body, body]
    return Image.fromarray(np.concatenate(tiles, axis=1), "RGB").convert("RGBA")


def draw(fn, size, *args):
    """Draw one sprite; outline it unless the drawing did (to put light over the outline)."""
    sp = Sprite(size, size)
    fn(sp, *args)
    if not getattr(sp, "outlined", False):
        outline(sp)
    return sp.image()


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    items = []
    for name, (fn, p) in PORTRAITS.items():
        img = draw(fn, PW, p)
        save(img, os.path.join(OUT, name + ".png"), "tools/art/animals_art.py (balatro-pixel-art kit)")
        items.append((name, img))
    for name, fn in DROPS.items():
        img = draw(fn, DW)
        save(img, os.path.join(OUT, name + ".png"), "tools/art/animals_art.py (balatro-pixel-art kit)")
        items.append((name, img))
    hides = []
    for name, (fn, p) in PORTRAITS.items():
        if fn is insect:
            continue   # insects are swarms of light in the 3D scene, no bodies
        img = hide_strip(name, fn, p)
        save(img, os.path.join(OUT, "hide_" + name + ".png"), "tools/art/animals_art.py (balatro-pixel-art kit)")
        hides.append(("hide_" + name, img))
    print("wrote", len(items) + len(hides), "pngs to", OUT)
    if "--sheet" in argv:
        out = argv[argv.index("--sheet") + 1]
        sheet(items, scale=4, cols=8, cell=(PW * 4, PW * 4)).save(out)
        sheet(hides, scale=3, cols=2).save(out.replace(".png", "_hides.png"))
        print("sheet", out)


if __name__ == "__main__":
    main(sys.argv[1:])
