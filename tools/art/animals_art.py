"""Animal portraits and drop icons for the animals mockup (docs/mockups/animals.html).

Painted by code with the hand-drawn-illustration kit
(.claude/skills/hand-drawn-illustration/scripts/crayon.py), copying the owner's
friend's method: a flat block, hatching in the shape's own tones, form crescents,
dabs and a same-hue edge. It is not hand drawn and the page says so.

Every quadruped is one function, `quad`, driven by body-plan parameters (length,
height, leg length, neck, ears, horns, tail, wool, hump, pattern), the same
parameters the mockup's three.js creatures are built from.

    python tools/art/animals_art.py            # writes docs/mockups/animals/*.png
    python tools/art/animals_art.py --sheet S  # also a contact sheet beside the friend's icons
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)
from crayon import Icon, circle, ellipse, rotate, translate, star, mix, darker, lighter, tone, INK  # noqa: E402

OUT = os.path.join(ROOT, "docs", "mockups", "animals")
GROUND = 470


def bumpy(cx, cy, rx, ry, bumps=13, amp=0.07, n=120, rot=0.0):
    pts = []
    for i in range(n):
        t = 2 * math.pi * i / n
        r = 1 + amp * abs(math.sin(bumps * t / 2))
        pts.append((rx * r * math.cos(t), ry * r * math.sin(t)))
    return translate(rotate(pts, rot), cx, cy)


def leg(x, top, bottom, w, lean=0.0):
    return [(x - w / 2, top), (x + w / 2, top), (x + w * 0.38 + lean, bottom), (x - w * 0.38 + lean, bottom)]


def tri(a, b, c):
    return [a, b, c]


def quad(ic, P):
    """A four-legged body plan, facing left, light from the upper left."""
    k = P.get("k", 1.0)
    col = P["col"]
    legc = P.get("legc", darker(col, 0.12))
    rx = 172 * k * P.get("long", 1.0)
    ry = 92 * k * P.get("tall", 1.0)
    leg_len = 140 * k * P.get("legs", 1.0)
    ground = GROUND
    cy = ground - leg_len - ry * 0.55
    cx = 284 + (1 - k) * 10
    lw = 38 * k * P.get("leg_w", 1.0)
    fx, bx = cx - rx * 0.58, cx + rx * 0.58
    # far legs, darker, a step behind
    for x in (fx + 22, bx + 22):
        ic.drawn(leg(x, cy, ground - 10, lw * 0.9, -6), darker(legc, 0.3), highlight=0, shade=0.1, edge_w=6)
    # tail
    tail = P.get("tail", "thin")
    tx, ty = cx + rx * 0.92, cy - ry * 0.2
    if tail == "bush":
        ic.drawn(ellipse(tx + 34 * k, ty + 44 * k, 66 * k, 28 * k, rot=1.0), P.get("tailc", col), shade=0.2, edge_w=6)
        ic.drawn(ellipse(tx + 62 * k, ty + 94 * k, 22 * k, 16 * k, rot=1.0), P.get("tipc", (250, 246, 236)), highlight=0, edge_w=5)
    elif tail == "tuft":
        ic.line([(tx, ty), (tx + 30 * k, ty + 50 * k), (tx + 36 * k, ty + 96 * k)], tone(col, 0.2), 9, ink=tone(col), ink_w=6)
        ic.drawn(ellipse(tx + 36 * k, ty + 104 * k, 12 * k, 20 * k), darker(col, 0.45), highlight=0, edge_w=5)
    elif tail == "stub":
        ic.drawn(circle(tx + 6, ty - 4, 20 * k), P.get("tailc", lighter(col, 0.5)), highlight=0, edge_w=5)
    elif tail == "long":
        pts = [(tx, ty - 8), (tx + 70 * k, ty - 40 * k), (tx + 120 * k, ty - 70 * k), (tx + 128 * k, ty - 60 * k),
               (tx + 72 * k, ty - 18 * k), (tx, ty + 14)]
        ic.drawn(pts, col, shade=0.2, highlight=0.1, edge_w=6)
    else:
        ic.line([(tx, ty), (tx + 40 * k, ty + 30 * k), (tx + 46 * k, ty + 80 * k)], darker(col, 0.2), 10, ink=tone(col), ink_w=6)
    # body
    if P.get("wool"):
        body = bumpy(cx, cy, rx, ry * 1.08, bumps=18, amp=0.09)
    else:
        body = ellipse(cx, cy, rx, ry, rot=P.get("tilt", 0.0))
    ic.drawn(body, col, rim_color=P.get("rim"), edge_w=8)
    if P.get("hump"):
        ic.drawn(ellipse(cx + 8, cy - ry * 0.85, rx * 0.42, ry * 0.62), col, shade=0.15, edge_w=7)
    if P.get("mane"):
        mane = bumpy(cx - rx * 0.62, cy - ry * 0.35, rx * 0.36, ry * 0.95, bumps=10, amp=0.12)
        ic.drawn(mane, P["mane"], shade=0.25, edge_w=7)
    # pattern on the body
    pat = P.get("pattern")
    pc = P.get("patc", darker(col, 0.5))
    if pat == "stripes":
        for i in range(-2, 4):
            x = cx + i * rx * 0.24
            dx = (x - cx) / rx
            if abs(dx) > 0.85:
                continue
            h = ry * math.sqrt(1 - dx * dx) * 0.92
            ic.drawn([(x - 9, cy - h), (x + 9, cy - h), (x + 15, cy - h * 0.15), (x + 2, cy + h * 0.35)], pc,
                     hatch=0.5, shade=0, highlight=0, edge_w=4)
    elif pat == "spots":
        for (dx, dy, r) in ((-0.3, -0.3, 0.17), (0.15, -0.45, 0.13), (0.42, -0.1, 0.16), (0.0, 0.1, 0.12), (-0.5, 0.15, 0.1), (0.3, 0.32, 0.11)):
            ic.drawn(ellipse(cx + dx * rx, cy + dy * ry, r * rx, r * rx * 0.75), pc, hatch=0.5, shade=0, highlight=0, edge_w=4)
    elif pat == "saddle":
        ic.drawn(ellipse(cx + rx * 0.1, cy - ry * 0.5, rx * 0.55, ry * 0.42), pc, hatch=0.6, shade=0.1, highlight=0, edge_w=5)
    # near legs
    for x in (fx, bx):
        ic.drawn(leg(x, cy + ry * 0.2, ground, lw), legc, highlight=0.08, edge_w=7)
        if P.get("hoof"):
            ic.drawn([(x - lw * 0.42, ground - 18), (x + lw * 0.42, ground - 18), (x + lw * 0.46, ground + 2), (x - lw * 0.46, ground + 2)],
                     P["hoof"], hatch=0.4, highlight=0, shade=0, edge_w=5)
    # neck and head
    hr = 64 * k * P.get("head", 1.0)
    neck = P.get("neck", 0.3)
    hx = cx - rx * 0.92 - hr * 0.35
    hy = cy - ry * 0.4 - neck * 150 * k
    if neck > 0.05:
        ic.drawn([(cx - rx * 0.55, cy - ry * 0.75), (cx - rx * 0.78, cy + ry * 0.2), (hx + hr * 0.2, hy + hr * 0.6), (hx + hr * 0.75, hy - hr * 0.1)],
                 col, highlight=0.1, edge_w=7)
    # ears behind the head
    ears = P.get("ears", "point")
    ec = P.get("earc", col)
    if ears == "long":
        for dx, rot in ((8, -0.15), (30, 0.12)):
            ic.drawn(ellipse(hx + dx, hy - hr * 1.15, hr * 0.22, hr * 0.85, rot=rot), ec, shade=0.15, edge_w=6)
    elif ears == "point":
        for dx in (6, 34):
            ic.drawn(tri((hx + dx - 18, hy - hr * 0.55), (hx + dx + 16, hy - hr * 0.6), (hx + dx - 4, hy - hr * 1.45)), ec, highlight=0, edge_w=6)
    elif ears == "round":
        for dx in (0, 34):
            ic.drawn(circle(hx + dx, hy - hr * 0.8, hr * 0.32), ec, highlight=0, edge_w=6)
    elif ears == "flop":
        ic.drawn(ellipse(hx + 34, hy - hr * 0.15, hr * 0.5, hr * 0.22, rot=0.6), darker(ec, 0.15), highlight=0, edge_w=6)
    head = ellipse(hx, hy, hr * 1.05, hr * 0.88, rot=-0.15)
    ic.drawn(head, P.get("headc", col), edge_w=8)
    snout = P.get("snout", 0.6)
    sn = ellipse(hx - hr * 0.75 - snout * 20, hy + hr * 0.3, hr * (0.4 + snout * 0.35), hr * 0.38, rot=0.15)
    ic.drawn(sn, P.get("snoutc", lighter(col, 0.25)), shade=0.15, highlight=0.1, edge_w=6)
    ic.dot(hx - hr * 1.05 - snout * 40, hy + hr * 0.2, 6 * k + 2, (30, 22, 26))
    # eye
    eye = P.get("eyec", (24, 20, 28))
    ic.dot(hx - hr * 0.25, hy - hr * 0.15, 9 * k + 3, (250, 246, 236), ink=tone(col, 0.7))
    ic.dot(hx - hr * 0.3, hy - hr * 0.13, 6 * k + 2, eye)
    if P.get("glow_eyes"):
        ic.glow((hx - hr * 0.3, hy - hr * 0.13, 22), P["glow_eyes"], radius=10, strength=0.8)
    # horns on top
    horns = P.get("horns")
    hc = P.get("hornc", (236, 222, 180))
    if horns == "curl":
        pts = [(hx - 10, hy - hr * 0.6), (hx + 40, hy - hr * 1.5), (hx + 115, hy - hr * 1.2), (hx + 120, hy - hr * 0.2),
               (hx + 92, hy - hr * 0.3), (hx + 88, hy - hr * 0.95), (hx + 44, hy - hr * 1.05), (hx + 18, hy - hr * 0.42)]
        ic.drawn(pts, hc, edge_w=6)
    elif horns == "spike":
        for dx in (4, 30):
            ic.drawn([(hx + dx - 9, hy - hr * 0.6), (hx + dx + 9, hy - hr * 0.6), (hx + dx + 30, hy - hr * 1.7)], hc, highlight=0, edge_w=5)
    elif horns == "tusk":
        sx, sy = hx - hr * 1.1, hy + hr * 0.45
        ic.drawn([(sx + 14, sy), (sx - 10, sy - 50), (sx - 30, sy - 70), (sx - 20, sy - 40), (sx - 4, sy + 8)], hc, highlight=0, edge_w=5)
    elif horns == "droop":
        ic.drawn([(hx - 40, hy - hr * 0.75), (hx + 60, hy - hr * 0.95), (hx + 110, hy - hr * 0.4), (hx + 100, hy + hr * 0.6),
                  (hx + 80, hy + hr * 0.5), (hx + 82, hy - hr * 0.3), (hx + 40, hy - hr * 0.5), (hx - 34, hy - hr * 0.4)], hc, edge_w=6)
    elif horns == "wide":
        ic.drawn([(hx - 30, hy - hr * 0.55), (hx + 50, hy - hr * 0.75), (hx + 110, hy - hr * 0.35), (hx + 118, hy + hr * 0.1),
                  (hx + 90, hy - hr * 0.2), (hx + 40, hy - hr * 0.35), (hx - 24, hy - hr * 0.25)], hc, edge_w=6)


def bird(ic, P):
    col, k = P["col"], P.get("k", 1.0)
    cx, cy = 270, 270
    fly = P.get("fly", False)
    if not fly:
        for dx in (-10, 24):
            ic.line([(cx + dx, cy + 60 * k), (cx + dx - 6, GROUND - 4)], P.get("legc", (230, 160, 60)), 9, ink=tone(P.get("legc", (230, 160, 60))), ink_w=6)
    tail = [(cx + 80 * k, cy - 10), (cx + 190 * k, cy + 30 * k), (cx + 180 * k, cy + 70 * k), (cx + 70 * k, cy + 40)]
    ic.drawn(tail, P.get("tailc", darker(col, 0.2)), highlight=0.05, edge_w=6)
    body = ellipse(cx, cy, 120 * k, 78 * k, rot=-0.18)
    ic.drawn(body, col, rim_color=P.get("rim"), edge_w=8)
    if P.get("belly"):
        ic.drawn(ellipse(cx - 30 * k, cy + 30 * k, 70 * k, 40 * k, rot=-0.2), P["belly"], shade=0.1, edge_w=5)
    wc = P.get("wingc", darker(col, 0.15))
    if fly:
        wing = [(cx - 40, cy - 30), (cx + 30, cy - 40), (cx + 160 * k, cy - 200 * k), (cx + 60 * k, cy - 170 * k), (cx - 30 * k, cy - 120 * k)]
    else:
        wing = [(cx - 40 * k, cy - 30 * k), (cx + 60 * k, cy - 40 * k), (cx + 150 * k, cy + 20 * k), (cx + 30 * k, cy + 45 * k)]
    ic.drawn(wing, wc, edge_w=7)
    for i in range(3):
        t = 0.3 + i * 0.22
        a, b = wing[1], wing[2]
        x, y = a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t
        ic.line([(x - 40 * k, y + 22 * k), (x, y)], tone(wc, 0.3), 5)
    hx, hy = cx - 110 * k, cy - 70 * k
    ic.drawn(circle(hx, hy, 50 * k), P.get("headc", col), edge_w=8)
    bl = P.get("beak_len", 1.0)
    beak = [(hx - 38 * k, hy - 14 * k), (hx - 38 * k - 70 * k * bl, hy + 8 * k * P.get("hook", 1)), (hx - 36 * k, hy + 16 * k)]
    ic.drawn(beak, P.get("beakc", (250, 190, 50)), highlight=0, shade=0.1, edge_w=6)
    ic.dot(hx - 10 * k, hy - 10 * k, 10 * k, (250, 246, 236), ink=tone(col, 0.7))
    ic.dot(hx - 13 * k, hy - 9 * k, 6 * k, (24, 20, 28))
    if P.get("crest"):
        ic.drawn([(hx + 4, hy - 44 * k), (hx + 50 * k, hy - 100 * k), (hx + 30 * k, hy - 40 * k)], P["crest"], highlight=0, edge_w=5)


def insect(ic, P):
    col = P["col"]
    cx, cy = 256, 280
    glow = P.get("glow")
    if glow:
        ic.glow((cx + 90, cy + 20, 110), glow, radius=40, strength=0.75)
    wc = P.get("wingc", (220, 236, 250))
    ws = P.get("wing", 1.0)
    for rot, dy in ((-0.5, -10), (-0.2, 10)):
        ic.drawn(ellipse(cx + 10, cy - 110 * ws, 70 * ws, 130 * ws, rot=rot), wc, hatch=0.6, shade=0.1, edge_w=6)
    for i, dx in enumerate((-40, 10, 60)):
        ic.line([(cx + dx, cy + 30), (cx + dx - 30, cy + 110), (cx + dx - 46 + i * 10, cy + 150)], (40, 34, 40), 7)
    ab = ellipse(cx + 90, cy + 20, 100, 62, rot=0.25)
    ic.drawn(ab, P.get("abc", col), rim_color=lighter(glow, 0.5) if glow else None, edge_w=7)
    if P.get("bands"):
        for i in range(3):
            x = cx + 70 + i * 34
            ic.drawn(ellipse(x, cy + 22 + i * 8, 10, 54, rot=0.25), P["bands"], hatch=0.5, shade=0, highlight=0, edge_w=4)
    ic.drawn(circle(cx - 20, cy, 52), col, edge_w=7)
    ic.drawn(circle(cx - 92, cy - 18, 40), darker(col, 0.2), edge_w=7)
    for dx in (-110, -86):
        ic.line([(cx + dx, cy - 50), (cx + dx - 40, cy - 130), (cx + dx - 70, cy - 140)], (40, 34, 40), 5)
    ic.dot(cx - 112, cy - 22, 10, (250, 246, 236), ink=(24, 20, 28))
    if glow:
        ic.sparkle(cx + 190, cy - 70, 22, lighter(glow, 0.4))
        ic.sparkle(cx - 160, cy + 90, 16, lighter(glow, 0.4))


def scarab(ic, P):
    col = P["col"]
    cx, cy = 256, 270
    for i, a in enumerate((-0.6, 0.0, 0.6)):
        for s in (-1, 1):
            x0, y0 = cx + s * 80, cy - 40 + i * 60
            ic.line([(x0, y0), (x0 + s * 70, y0 + 30 * a * 3 + 10), (x0 + s * 100, y0 + 70)], (40, 30, 30), 9)
    ic.drawn(ellipse(cx, cy + 30, 120, 150), col, rim_color=lighter(col, 0.55), edge_w=8)
    ic.line([(cx, cy - 110), (cx, cy + 175)], tone(col, 0.4), 6)
    ic.drawn(ellipse(cx, cy - 145, 82, 48), darker(col, 0.15), edge_w=7)
    ic.drawn([(cx - 30, cy - 180), (cx, cy - 250), (cx + 30, cy - 180)], P.get("hornc", darker(col, 0.25)), highlight=0, edge_w=6)
    ic.sparkle(cx + 150, cy - 160, 24)


def crab(ic, P):
    col = P["col"]
    cx, cy = 256, 300
    for s in (-1, 1):
        for i in range(3):
            x0 = cx + s * (70 + i * 22)
            ic.line([(x0, cy + 10), (x0 + s * 50, cy + 40 + i * 6), (x0 + s * 64, GROUND - 6)], darker(col, 0.2), 10, ink=tone(col), ink_w=6)
        ic.line([(cx + s * 90, cy - 20), (cx + s * 150, cy - 90)], col, 16, ink=tone(col), ink_w=6)
        claw = [(cx + s * 120, cy - 90), (cx + s * 200, cy - 170), (cx + s * 210, cy - 110), (cx + s * 175, cy - 70)]
        ic.drawn(claw, col, edge_w=7)
        ic.drawn([(cx + s * 160, cy - 140), (cx + s * 175, cy - 210), (cx + s * 195, cy - 160)], lighter(col, 0.1), highlight=0, edge_w=6)
    ic.drawn(ellipse(cx, cy, 140, 82), col, rim_color=lighter(col, 0.5), edge_w=8)
    for s in (-1, 1):
        ic.line([(cx + s * 30, cy - 70), (cx + s * 36, cy - 120)], darker(col, 0.3), 7)
        ic.dot(cx + s * 36, cy - 126, 14, (250, 246, 236), ink=(24, 20, 28))
        ic.dot(cx + s * 38, cy - 126, 7, (24, 20, 28))


def toad(ic, P):
    col = P["col"]
    cx, cy = 256, 330
    for s in (-1, 1):
        ic.drawn(ellipse(cx + s * 120, GROUND - 26, 70, 26), darker(col, 0.15), highlight=0, edge_w=6)
    body = ellipse(cx, cy, 170, 105)
    ic.drawn(body, col, edge_w=8)
    ic.drawn(ellipse(cx, cy + 50, 120, 50), P.get("belly", lighter(col, 0.4)), shade=0.1, edge_w=5)
    glow = P.get("glow")
    for (dx, dy, r) in ((-80, -40, 16), (-20, -70, 13), (50, -55, 17), (110, -20, 12), (0, -20, 10)):
        if glow:
            ic.glow((cx + dx, cy + dy, r * 2.2), glow, radius=10, strength=0.9)
        ic.drawn(circle(cx + dx, cy + dy, r), glow or darker(col, 0.4), hatch=0.4, shade=0, highlight=0, edge_w=4)
    for s in (-1, 1):
        ic.drawn(circle(cx + s * 70, cy - 100, 40), col, edge_w=7)
        ic.dot(cx + s * 70, cy - 104, 22, (250, 220, 90), ink=(24, 20, 28))
        ic.drawn(ellipse(cx + s * 70, cy - 104, 6, 16), (24, 20, 28), hatch=0, shade=0, highlight=0, edge=None)
    ic.line([(cx - 70, cy - 30), (cx, cy - 18), (cx + 70, cy - 30)], tone(col, 0.5), 6)


def croc(ic, P):
    col = P["col"]
    cy = 330
    for x in (150, 330):
        ic.drawn(leg(x, cy, GROUND, 40, -20), darker(col, 0.2), highlight=0, edge_w=6)
    body = [(20, cy - 14), (120, cy - 66), (330, cy - 72), (420, cy - 36), (505, cy + 14), (420, cy + 26), (330, cy + 56), (120, cy + 52), (20, cy + 18)]
    ic.drawn(body, col, edge_w=8)
    for i in range(9):
        x = 140 + i * 30
        ic.drawn([(x - 12, cy - 64), (x, cy - 92), (x + 12, cy - 64)], darker(col, 0.25), hatch=0.4, highlight=0, shade=0, edge_w=4)
    ic.drawn(ellipse(140, cy - 50, 26, 20), lighter(col, 0.15), edge_w=6)
    ic.dot(138, cy - 54, 9, (250, 220, 90), ink=(24, 20, 28))
    ic.drawn(ellipse(138, cy - 54, 3, 9), (24, 20, 28), hatch=0, shade=0, highlight=0, edge=None)
    for x in range(44, 110, 16):
        ic.drawn([(x, cy + 2), (x + 6, cy + 16), (x + 12, cy + 2)], (250, 246, 236), hatch=0, shade=0, highlight=0, edge_w=3)
    for x in (200, 300):
        ic.line([(x, GROUND), (x - 20, GROUND)], tone(col), 4)


# ---------- drops --------------------------------------------------------------------

def d_wool(ic):
    ic.drawn(bumpy(256, 270, 180, 130, bumps=16, amp=0.13), (246, 240, 226), rim_color=(255, 255, 250), edge=(150, 130, 110))
    for x, y in ((190, 250), (260, 220), (320, 280), (230, 320)):
        ic.line([(x, y), (x + 20, y - 18), (x + 34, y)], (200, 190, 170), 6)


def d_hide(ic):
    pts = [(110, 150), (200, 120), (256, 160), (320, 115), (410, 150), (395, 260), (430, 350), (330, 360), (256, 400), (180, 360), (85, 350), (120, 260)]
    ic.drawn(pts, (176, 112, 64), edge_w=9)
    for x, y in ((180, 220), (300, 230), (240, 310)):
        ic.drawn(ellipse(x, y, 30, 20), (120, 72, 40), hatch=0.4, shade=0, highlight=0, edge_w=4)


def d_meat(ic):
    ic.drawn(ellipse(330, 330, 22, 90, rot=-0.8), (246, 238, 220), edge=(140, 120, 100), edge_w=7)
    for dx, dy in ((385, 375), (400, 350)):
        ic.drawn(circle(dx, dy, 26), (246, 238, 220), edge=(140, 120, 100), edge_w=6)
    ic.drawn(ellipse(220, 220, 140, 110, rot=-0.7), (214, 70, 60), rim_color=(255, 160, 140), edge_w=8)
    ic.drawn(ellipse(200, 200, 90, 60, rot=-0.7), (238, 120, 100), shade=0.1, edge=None)


def d_egg(ic):
    pts = [(256 + 120 * math.sin(t) * (1 - 0.18 * math.cos(t)), 270 - 165 * math.cos(t)) for t in [2 * math.pi * i / 80 for i in range(80)]]
    ic.drawn(pts, (240, 214, 170), rim_color=(255, 245, 220), edge_w=9)
    for x, y in ((220, 200), (300, 260), (250, 340), (310, 180)):
        ic.dot(x, y, 8, (190, 150, 100))


def d_feather(ic):
    pts = [(120, 420), (170, 300), (260, 170), (380, 80), (400, 100), (330, 220), (230, 340)]
    ic.drawn(pts, (70, 170, 230), edge_w=8)
    ic.line([(110, 440), (390, 90)], (240, 240, 230), 7, ink=(30, 60, 90), ink_w=4)
    for i in range(6):
        t = 0.25 + i * 0.11
        x, y = 110 + 280 * t, 440 - 350 * t
        ic.line([(x, y), (x + 50, y + 26)], (40, 110, 170), 5)


def d_glow(ic):
    ic.glow((256, 300, 150), (200, 255, 120), radius=40, strength=0.75)
    jar = [(170, 180), (342, 180), (360, 230), (360, 420), (152, 420), (152, 230)]
    ic.drawn(jar, (180, 236, 140), rim_color=(240, 255, 210), edge=(50, 110, 60), edge_w=8)
    ic.drawn([(180, 130), (332, 130), (342, 180), (170, 180)], (150, 100, 60), highlight=0, edge_w=7)
    for x, y in ((220, 280), (300, 320), (250, 370), (300, 250)):
        ic.dot(x, y, 12, (250, 255, 170))
    ic.sparkle(400, 150, 28, (240, 255, 150))


def d_milk(ic):
    pail = [(150, 200), (362, 200), (340, 420), (172, 420)]
    ic.drawn(pail, (150, 170, 190), edge_w=9)
    ic.drawn(ellipse(256, 200, 106, 30), (250, 248, 240), highlight=0, edge=(120, 130, 150), edge_w=6)
    ic.line(list(reversed([(150, 200), (180, 110), (256, 90), (332, 110), (362, 200)])), (90, 90, 100), 7)
    for y in (260, 360):
        ic.line([(158, y), (354, y)], (90, 104, 124), 8)


def d_horn(ic):
    pts = [(120, 400), (180, 260), (300, 160), (420, 150), (440, 180), (330, 220), (230, 330), (190, 420)]
    ic.drawn(pts, (236, 220, 176), rim_color=(255, 250, 230), edge=(130, 110, 80), edge_w=8)
    for t in (0.3, 0.45, 0.6, 0.75):
        x, y = 150 + 270 * t, 410 - 250 * t
        ic.line([(x - 20, y - 30), (x + 20, y + 20)], (180, 160, 120), 5)


def d_shell(ic):
    fan = [(256, 430)] + [(256 + 200 * math.cos(a) * (1 + 0.05 * math.sin(9 * a)), 300 - 210 * math.sin(a)) for a in [math.pi * (0.08 + 0.84 * i / 30) for i in range(31)]]
    ic.drawn(fan, (246, 170, 130), rim_color=(255, 226, 200), edge=(150, 70, 50), edge_w=9)
    for i in range(7):
        a = math.pi * (0.16 + 0.68 * i / 6)
        ic.line([(256, 420), (256 + 180 * math.cos(a), 300 - 190 * math.sin(a))], (200, 110, 80), 6)
    ic.drawn([(210, 400), (302, 400), (290, 450), (222, 450)], (230, 140, 100), highlight=0, edge=(150, 70, 50), edge_w=6)


# ---------- the roster's portraits ----------------------------------------------------

def Q(**p):
    return (quad, p)


PORTRAITS = {
    # fields
    "woolback": Q(col=(238, 232, 214), wool=True, legc=(70, 60, 60), headc=(70, 60, 60), snoutc=(100, 88, 86), ears="flop", earc=(70, 60, 60), tail="stub", legs=0.7, k=0.95, snout=0.3),
    "plodder": Q(col=(176, 110, 70), pattern="spots", patc=(248, 242, 230), horns="spike", tail="tuft", legs=0.75, long=1.12, tall=1.15, hoof=(60, 46, 40), k=1.0, snout=0.7, ears="flop"),
    "hen": (bird, dict(col=(222, 120, 60), wingc=(186, 86, 40), tailc=(60, 50, 50), crest=(230, 40, 40), beak_len=0.5, k=0.95)),
    "meadow_hare": Q(col=(176, 140, 100), ears="long", tail="stub", tailc=(250, 246, 236), legs=0.45, long=0.7, tall=0.95, k=0.85, snout=0.2, head=0.9),
    "red_fox": Q(col=(232, 112, 40), ears="point", tail="bush", legs=0.65, long=1.0, tall=0.8, k=0.9, snout=1.0, snoutc=(250, 240, 226), legc=(60, 40, 36)),
    "lark": (bird, dict(col=(170, 130, 90), belly=(236, 214, 170), crest=(140, 100, 70), beak_len=0.5, k=0.85, fly=True)),
    "firefly": (insect, dict(col=(60, 50, 60), abc=(220, 240, 90), glow=(210, 255, 90), wing=0.8)),
    # desert
    "sand_strider": Q(col=(214, 170, 100), hump=True, neck=0.85, legs=1.15, long=1.0, tall=0.85, k=0.85, tail="tuft", ears="round", snout=0.8, head=0.85, hoof=(120, 90, 60)),
    "dust_hound": Q(col=(200, 160, 110), ears="point", pattern="spots", patc=(110, 80, 60), tail="thin", legs=0.85, tall=0.75, k=0.85, snout=1.0, glow_eyes=(255, 200, 80)),
    "scarab": (scarab, dict(col=(40, 140, 150))),
    # jungle
    "stripe_cat": Q(col=(240, 150, 40), pattern="stripes", patc=(40, 30, 30), tail="long", ears="round", legs=0.7, long=1.1, tall=0.78, k=0.95, snout=0.4, glow_eyes=(150, 255, 160)),
    "tusk_boar": Q(col=(90, 76, 80), mane=(60, 50, 52), horns="tusk", hornc=(246, 238, 210), tail="thin", legs=0.55, tall=1.05, k=0.95, snout=1.0, ears="point"),
    "parrot": (bird, dict(col=(40, 190, 80), wingc=(40, 110, 220), tailc=(230, 50, 50), headc=(240, 210, 40), beakc=(240, 240, 220), beak_len=0.6, hook=2.5)),
    "glowmoth": (insect, dict(col=(110, 70, 160), abc=(170, 120, 230), wingc=(150, 220, 240), glow=(120, 220, 255), wing=1.15, bands=(70, 40, 120))),
    # swamp
    "bog_toad": (toad, dict(col=(90, 130, 70), glow=(150, 255, 210))),
    "heron": (bird, dict(col=(170, 190, 210), wingc=(110, 130, 160), headc=(230, 236, 240), beak_len=1.6, k=0.85, legc=(200, 170, 80), crest=(40, 50, 70))),
    "mire_lurker": (croc, dict(col=(80, 120, 70))),
    # mountains
    "ridge_goat": Q(col=(206, 200, 190), horns="curl", hornc=(120, 96, 70), tail="stub", legs=0.85, long=0.95, k=0.9, snout=0.6, hoof=(50, 44, 44), ears="point", legc=(150, 140, 130), headc=(236, 232, 224)),
    "crag_eagle": (bird, dict(col=(110, 80, 50), wingc=(80, 56, 36), headc=(246, 242, 232), beak_len=0.6, hook=2.5, fly=True)),
    # tundra
    "shagback": Q(col=(150, 112, 78), wool=True, mane=(96, 70, 50), headc=(70, 52, 40), snoutc=(120, 100, 84), horns="droop", hornc=(236, 226, 200), hump=True, legs=0.5, long=1.1, tall=1.15, k=1.0, snout=0.6, tail="stub", tailc=(80, 60, 46), ears=None),
    "pale_wolf": Q(col=(220, 226, 236), ears="point", tail="bush", tipc=(140, 150, 170), legs=0.8, tall=0.82, k=0.95, snout=1.1, legc=(190, 196, 210), glow_eyes=(150, 220, 255)),
    "snow_hare": Q(col=(244, 246, 250), ears="long", earc=(244, 246, 250), tail="stub", legs=0.45, long=0.7, tall=0.95, k=0.85, snout=0.2, head=0.9, legc=(220, 224, 236)),
    # beach
    "shore_crab": (crab, dict(col=(230, 90, 50))),
    "gull": (bird, dict(col=(240, 244, 246), wingc=(150, 164, 180), tailc=(40, 44, 50), beak_len=0.8, k=0.9)),
}

DROPS = {
    "drop_wool": d_wool, "drop_hide": d_hide, "drop_meat": d_meat, "drop_egg": d_egg, "drop_feather": d_feather,
    "drop_glow": d_glow, "drop_milk": d_milk, "drop_horn": d_horn, "drop_shell": d_shell,
}


def main(argv):
    os.makedirs(OUT, exist_ok=True)
    written = []
    for i, (name, (fn, p)) in enumerate(PORTRAITS.items()):
        ic = Icon(seed=100 + i)
        fn(ic, p)
        path = os.path.join(OUT, name + ".png")
        ic.save(path)
        written.append(path)
    for i, (name, fn) in enumerate(DROPS.items()):
        ic = Icon(seed=300 + i)
        fn(ic)
        path = os.path.join(OUT, name + ".png")
        ic.save(path)
        written.append(path)
    print("wrote", len(written), "pngs to", OUT)
    if "--sheet" in argv:
        out = argv[argv.index("--sheet") + 1]
        sys.path.insert(0, KIT)
        import sheet
        sheet.main([out] + written + ["--ref", "vitality", "--ref", "eagle eye", "--ref", "inferno"])


if __name__ == "__main__":
    main(sys.argv[1:])
