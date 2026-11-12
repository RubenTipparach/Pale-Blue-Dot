"""Hand-drawn fish art for the fish and aquarium mockup (docs/mockups/fish.html).

Painted in code by the hand-drawn-illustration skill's crayon kit, in the
manner of the owner's friend's upgrade icons: flat saturated blocks, loose
marker hatching in each shape's own tones, form-following crescents, a deep
same-hue edge, light from the upper left, no frame. It is not the friend's
work and the page says so.

    python tools/art/fish_art.py [OUT_DIR]          (default docs/mockups/fish)
    python tools/art/fish_art.py OUT_DIR --sheet SHEET.png

Writes one portrait per species (256 px, head to the right, the Tenebris icon
rule), one swatch per pattern type and one icon per aquarium decor item
(128 px). Each species' outline comes from the same body numbers the page's
three.js generator uses (depth, width, where the body is deepest, the tail
stalk, the tail and fin shapes), so the portrait and the 3D fish agree.
"""

import math
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
KIT = os.path.join(ROOT, ".claude", "skills", "hand-drawn-illustration", "scripts")
sys.path.insert(0, KIT)

import crayon  # noqa: E402
from crayon import (Icon, circle, ellipse, rect, roundrect, star, rotate, translate,  # noqa: E402
                    mix, darker, lighter, tone, INK)
from PIL import Image, ImageChops, ImageDraw  # noqa: E402

# ---------------------------------------------------------------- species -----
# Body numbers are fractions of the fish's length (the same as fish.html's
# PROFILES); colours are the species' palette; `pat` is the pattern type.
SPECIES = {
    "minnow":     dict(D=0.12, uMax=0.33, ped=0.22, body=0.80, tail="forked", dorsal=[(0.38, 0.55, 0.10, "soft")], anal=[(0.62, 0.74, 0.06)],
                       base=(78, 112, 150), belly=(226, 232, 236), acc=(52, 74, 104), fin=(150, 175, 196), pat="gradient", eye=0.030),
    "silverfin":  dict(D=0.10, uMax=0.35, ped=0.16, body=0.78, tail="deepfork", dorsal=[(0.36, 0.50, 0.12, "soft")], anal=[(0.60, 0.72, 0.07)],
                       base=(64, 126, 160), belly=(238, 244, 248), acc=(30, 66, 98), fin=(196, 222, 234), pat="stripes", eye=0.032),
    "perch":      dict(D=0.17, uMax=0.38, ped=0.20, body=0.82, tail="truncate", dorsal=[(0.22, 0.48, 0.14, "spiny"), (0.50, 0.68, 0.11, "soft")], anal=[(0.62, 0.74, 0.08)],
                       base=(206, 168, 52), belly=(244, 226, 156), acc=(58, 70, 30), fin=(232, 98, 44), pat="bars", bars=(5, 0.045, 0.55), eye=0.034),
    "ray":        dict(view="top", D=0.05, base=(186, 160, 118), belly=(240, 236, 226), acc=(104, 80, 52), fin=(160, 132, 92), pat="spots", eye=0.026),
    "eel":        dict(D=0.07, uMax=0.12, ped=0.55, body=1.0, tail="eel", wave=22, ex=1.8, base=(108, 122, 58), belly=(204, 194, 120), acc=(40, 46, 18), fin=(132, 140, 70), pat="mottled", eye=0.022),
    "reef":       dict(D=0.27, uMax=0.42, ped=0.14, body=0.80, tail="lunate", dorsal=[(0.18, 0.66, 0.20, "tall")], anal=[(0.50, 0.74, 0.16)],
                       base=(36, 170, 172), belly=(150, 226, 210), acc=(255, 206, 44), fin=(32, 120, 150), pat="bars", bars=(3, 0.06, 0.9), eye=0.034),
    "deepback":   dict(D=0.15, uMax=0.36, ped=0.12, body=0.80, tail="lunate", dorsal=[(0.34, 0.48, 0.12, "soft")], anal=[(0.62, 0.72, 0.07)],
                       base=(44, 64, 98), belly=(176, 196, 206), acc=(24, 34, 56), fin=(70, 96, 130), pat="gradient", eye=0.036),
    "serpent":    dict(D=0.075, uMax=0.10, ped=0.50, body=1.0, tail="eel", wave=36, ex=1.8, crest=True, base=(40, 128, 110), belly=(196, 222, 170), acc=(18, 64, 58), fin=(214, 64, 52), pat="bars", bars=(9, 0.025, 0.5), eye=0.026),
    "bloatfin":   dict(D=0.30, uMax=0.48, ped=0.18, body=0.80, tail="rounded", dorsal=[(0.56, 0.68, 0.10, "soft")], anal=[(0.58, 0.70, 0.09)],
                       base=(228, 196, 86), belly=(250, 244, 220), acc=(84, 58, 30), fin=(238, 160, 60), pat="spots", eye=0.050, spines=True),
    "sunplate":   dict(D=0.30, uMax=0.44, ped=0.14, body=0.82, tail="truncate", dorsal=[(0.26, 0.66, 0.14, "spiny")], anal=[(0.50, 0.72, 0.12)],
                       base=(226, 142, 44), belly=(250, 214, 120), acc=(50, 122, 206), fin=(170, 96, 40), pat="stripes", wavy=True, eye=0.034, earspot=True),
    "lanternjaw": dict(D=0.26, uMax=0.26, ped=0.20, body=0.82, tail="rounded", dorsal=[(0.56, 0.68, 0.08, "soft")], anal=[(0.58, 0.70, 0.07)],
                       base=(56, 42, 72), belly=(92, 74, 110), acc=(90, 244, 255), fin=(74, 58, 96), pat="biolum", eye=0.024, lure=True, teeth=True),
    "glassback":  dict(D=0.14, uMax=0.36, ped=0.15, body=0.78, tail="deepfork", dorsal=[(0.34, 0.50, 0.11, "soft")], anal=[(0.58, 0.70, 0.08)],
                       base=(170, 214, 238), belly=(226, 242, 250), acc=(110, 170, 255), fin=(200, 230, 246), pat="biolum", eye=0.040, glass=True),
    "emberback":  dict(D=0.17, uMax=0.34, ped=0.18, body=0.82, tail="truncate", dorsal=[(0.24, 0.58, 0.12, "spiny")], anal=[(0.60, 0.72, 0.08)],
                       base=(70, 48, 46), belly=(120, 86, 72), acc=(255, 128, 40), fin=(110, 60, 48), pat="mottled", eye=0.030, ember=True),
}

NOSE, LPX, CY = 466.0, 404.0, 262.0   # head to the right; the fish spans x 62..466


def profile(u, D, uMax, ped):
    """Half depth at u (0 = nose, 1 = tail stalk), as fish.html's profile()."""
    if u <= uMax:
        t = u / uMax
        return D * math.sqrt(max(0.0, 2 * t - t * t))
    t = (u - uMax) / (1 - uMax)
    return D * (ped + (1 - ped) * 0.5 * (1 + math.cos(math.pi * t)))


class Fish:
    """A side-view fish laid out in canvas pixels."""

    def __init__(self, sp):
        # A portrait exaggerates a slender body so it reads at 48 px; the 3D
        # generator keeps the true depth.
        sp = dict(sp, D=sp["D"] * sp.get("ex", 1.0 if sp["D"] > 0.2 else 1.3))
        self.sp = sp
        self.wave = sp.get("wave", 6)
        self.blen = sp.get("body", 0.8)
        n = 60
        self.top, self.bot = [], []
        for i in range(n + 1):
            u = i / n
            x, cy = self.at(u)
            h = profile(u, sp["D"], sp["uMax"], sp["ped"]) * LPX
            if sp.get("tail") == "eel":
                h = max(h * (1 - u ** 3), 2.0)
            self.top.append((x, cy - h))
            self.bot.append((x, cy + h * 0.94))
        self.outline = self.top + self.bot[::-1]

    def at(self, u):
        x = NOSE - u * self.blen * LPX
        return x, CY + self.wave * math.sin(u * math.pi * (2.2 if self.sp.get("tail") == "eel" else 1.0) + 0.4) * u

    def h(self, u):
        return profile(u, self.sp["D"], self.sp["uMax"], self.sp["ped"]) * LPX

    def slab(self, u0, u1, y0, y1, n=14):
        """A band across the body between u0 and u1, from y0 to y1 (fractions of
        the local half depth, -1 top, +1 belly), following the body's curve."""
        up, dn = [], []
        for i in range(n + 1):
            u = u0 + (u1 - u0) * i / n
            x, cy = self.at(u)
            h = self.h(u)
            up.append((x, cy + y0 * h))
            dn.append((x, cy + y1 * h))
        return up + dn[::-1]


def clip(ic, poly, body_mask, color, hatch=True, var=0.14):
    """Paint `poly` cut to the body: a flat block and its own hatching."""
    m = ImageChops.multiply(ic._mask(ic._wobble(poly, amp=2.5)), body_mask)
    ic.img.paste(Image.new("RGBA", (crayon.S, crayon.S), color + (255,)), (0, 0), m)
    if hatch:
        ic.scribble_in(m, color, var=var, width=5, density=0.9, length=(26, 70), spread=16, angle=25)
    return m


def fin(ic, poly, color, rays=None, alpha_edge=6):
    ic.drawn(poly, color, hatch=0.7, shade=0.18, highlight=0.0, edge="tone", edge_w=alpha_edge)
    for a, b in (rays or []):
        ic.line([a, b], darker(color, 0.28), 3, amp=1.2)


def tail_poly(f, kind):
    x0, cy = f.at(1.0)
    h = max(f.h(1.0), 10)
    L = (1 - f.blen) * LPX + 6
    xe = x0 - L
    if kind == "forked":
        return [(x0 + 6, cy - h), (xe, cy - h - L * 0.75), (x0 - L * 0.55, cy), (xe, cy + h + L * 0.75), (x0 + 6, cy + h)]
    if kind == "deepfork":
        return [(x0 + 6, cy - h), (xe - 8, cy - h - L * 0.95), (xe + L * 0.15, cy - h - L * 0.7), (x0 - L * 0.45, cy),
                (xe + L * 0.15, cy + h + L * 0.7), (xe - 8, cy + h + L * 0.95), (x0 + 6, cy + h)]
    if kind == "lunate":
        pts = [(x0 + 6, cy - h), (xe - 6, cy - h - L * 1.05), (xe + L * 0.42, cy - h * 0.5), (xe + L * 0.52, cy), (xe + L * 0.42, cy + h * 0.5), (xe - 6, cy + h + L * 1.05), (x0 + 6, cy + h)]
        return pts
    if kind == "rounded":
        pts = [(x0 + 6, cy - h)]
        for i in range(13):
            a = -math.pi / 2 + math.pi * i / 12
            pts.append((x0 - L * 0.35 - L * 0.75 * math.cos(a), cy + (h + L * 0.55) * math.sin(a)))
        return pts + [(x0 + 6, cy + h)]
    # truncate: a square-cut fan
    return [(x0 + 6, cy - h), (xe, cy - h - L * 0.6), (xe - 6, cy), (xe, cy + h + L * 0.6), (x0 + 6, cy + h)]


def tail_rays(f, poly):
    x0, cy = f.at(1.0)
    pts = poly[1:-1]
    step = max(1, len(pts) // 5)
    return [((x0 - 4, cy), p) for p in pts[::step]]


def dorsal_poly(f, u0, u1, hgt, kind):
    hp = hgt * LPX
    base = [(f.at(u)[0], f.at(u)[1] - f.h(u) * 0.86) for u in (u0 + (u1 - u0) * i / 10 for i in range(11))]
    if kind == "spiny":
        tips = []
        for i, (x, y) in enumerate(base):
            tips.append((x, y - hp * (0.9 if i % 2 == 0 else 0.55) * (1 - 0.3 * i / 10)))
        return base[:1] + tips + base[::-1]
    if kind == "tall":
        return [base[0], (base[1][0] - hp * 0.2, base[1][1] - hp * 1.3), (base[-1][0] - hp * 0.5, base[-1][1] - hp * 0.6), base[-1]] + base[::-1][1:-1]
    # soft: a rounded sail, highest near the front
    pk = (base[3][0] - hp * 0.2, base[3][1] - hp)
    return [base[0], pk, (base[-1][0] - hp * 0.35, base[-1][1] - hp * 0.25), base[-1]] + base[::-1][1:-1]


def anal_poly(f, u0, u1, hgt):
    hp = hgt * LPX
    base = [(f.at(u)[0], f.at(u)[1] + f.h(u) * 0.84) for u in (u0 + (u1 - u0) * i / 8 for i in range(9))]
    return [base[0], (base[2][0] - hp * 0.3, base[2][1] + hp), (base[-1][0] - hp * 0.3, base[-1][1] + hp * 0.3), base[-1]] + base[::-1][1:-1]


def pectoral_poly(f, sp):
    u = 0.24 if sp.get("tail") != "eel" else 0.09
    x, cy = f.at(u)
    h = f.h(u)
    s = max(h * 0.75, 26)
    p = [(0, 0), (-s * 1.3, -s * 0.15), (-s * 1.6, s * 0.35), (-s * 0.9, s * 0.5), (0, s * 0.18)]
    return translate(rotate(p, 0.35), x, cy + h * 0.25)


def pattern(ic, f, sp, bm):
    """The species' pattern, laid over the body block and cut to it."""
    rnd = ic.rnd
    pat, acc = sp["pat"], sp["acc"]
    if pat == "gradient":
        clip(ic, f.slab(0.0, 1.0, -1.2, -0.25), bm, acc)                       # the dark back
        clip(ic, f.slab(0.02, 1.0, -0.25, -0.05), bm, mix(acc, sp["base"], 0.5))
    elif pat == "stripes":
        if sp.get("wavy"):
            for k in range(5):
                y = -0.7 + k * 0.32
                pts_up, pts_dn = [], []
                for i in range(15):
                    u = 0.08 + 0.86 * i / 14
                    x, cy = f.at(u)
                    h = f.h(u)
                    w = math.sin(u * 22 + k) * 0.06
                    pts_up.append((x, cy + (y + w) * h))
                    pts_dn.append((x, cy + (y + w + 0.1) * h))
                clip(ic, pts_up + pts_dn[::-1], bm, acc, var=0.2)
        else:
            clip(ic, f.slab(0.0, 1.0, -1.2, -0.35), bm, mix(acc, sp["base"], 0.35))
            clip(ic, f.slab(0.12, 1.0, -0.08, 0.12), bm, acc)                  # the lateral stripe
    elif pat == "bars":
        nb, w, low = sp["bars"]
        for k in range(nb):
            u = 0.2 + 0.72 * (k + 0.5) / nb
            clip(ic, f.slab(u - w, u + w * 0.8, -1.2, low), bm, acc)
    elif pat == "spots":
        for _ in range(26):
            u = rnd.uniform(0.12, 0.95)
            x, cy = f.at(u)
            h = f.h(u)
            y = cy + rnd.uniform(-0.9, 0.5) * h
            r = rnd.uniform(6, 13)
            clip(ic, circle(x, y, r, 14), bm, acc, hatch=False)
    elif pat == "mottled":
        for _ in range(16):
            u = rnd.uniform(0.06, 0.98)
            x, cy = f.at(u)
            h = f.h(u)
            y = cy + rnd.uniform(-0.9, 0.4) * h
            blob = star(x, y, rnd.uniform(14, 26), rnd.uniform(8, 14), points=5, rot=rnd.uniform(0, 6), jag=0.3, seed=rnd.randint(0, 999))
            clip(ic, blob, bm, acc if not sp.get("ember") else darker(sp["base"], 0.4), hatch=False)
        if sp.get("ember"):   # glowing cracks between the plates
            for _ in range(9):
                u = rnd.uniform(0.1, 0.9)
                x, cy = f.at(u)
                h = f.h(u)
                y = cy + rnd.uniform(-0.6, 0.5) * h
                pts = [(x, y)]
                for _ in range(3):
                    x -= rnd.uniform(10, 24)
                    y += rnd.uniform(-12, 12)
                    pts.append((x, y))
                ic.glow((pts[1][0], pts[1][1], 16), acc, radius=10, strength=0.6)
                lay = Image.new("RGBA", (crayon.S, crayon.S), (0, 0, 0, 0))
                ImageDraw.Draw(lay).line(pts, fill=acc + (255,), width=5, joint="curve")
                ic._comp(lay, bm)
    elif pat == "biolum":
        pass   # dots go on after the shading, so they stay bright


def biolum_dots(ic, f, sp):
    acc = sp["acc"]
    for row, y in ((0, 0.62), (1, 0.15)):
        n = 9 if row == 0 else 6
        for k in range(n):
            u = 0.18 + 0.7 * k / (n - 1)
            x, cy = f.at(u)
            yy = cy + y * f.h(u)
            ic.glow((x, yy, 12), acc, radius=7, strength=0.7)
            ic.dot(x, yy, 5 if row == 0 else 4, lighter(acc, 0.3), ink=tone(sp["base"]))


def eye(ic, f, sp):
    sp = f.sp
    u = 0.08 if sp.get("tail") != "eel" else 0.045
    x, cy = f.at(u)
    r = sp["eye"] * LPX
    y = cy - f.h(min(0.2, sp["uMax"])) * 0.32
    ic.dot(x, y, r + 3, (250, 248, 236), ink=tone(sp["base"]))
    ic.dot(x + r * 0.15, y, r * 0.62, (18, 16, 24))
    ic.dot(x - r * 0.15, y - r * 0.3, max(2, r * 0.22), (255, 255, 255))


def bubbles(ic, sp, n=3):
    for i in range(n):
        x = 470 - i * 18 + ic.rnd.uniform(-6, 6)
        y = 150 - i * 46
        r = 12 - i * 2.5
        ic.outline(circle(x, y, r, 18), lighter((120, 200, 240), 0.2), 4, amp=0.8, passes=1)
        ic.dot(x - r * 0.35, y - r * 0.35, 2.5, (240, 250, 255))


def portrait_side(ic, sp):
    f = Fish(sp)
    # behind the body: far pectoral, tail, the fins on the back and belly
    if sp.get("tail") == "eel":
        # one continuous fin round the back half and the tail
        up, dn = [], []
        for i in range(25):
            u = 0.35 + 0.65 * i / 24
            x, cy = f.at(u)
            h = max(f.h(u) * (1 - u ** 3), 2)
            fh = 9 + 10 * (u - 0.35) / 0.65
            up.append((x, cy - h - fh))
            dn.append((x, cy + h + fh))
        x, cy = f.at(1.0)
        ring = up + [(x - 18, cy)] + dn[::-1]
        fin(ic, ring, sp["fin"], alpha_edge=5)
        if sp.get("crest"):
            crest = []
            for i in range(19):
                u = 0.02 + 0.5 * i / 18
                x, cy = f.at(u)
                crest.append((x, cy - f.h(u) * 0.9))
            spikes = [crest[0]]
            for i, (x, y) in enumerate(crest[1:], 1):
                spikes.append((x + 6, y - (28 if i % 2 else 10) * (1 - i / 22)))
            fin(ic, spikes + crest[::-1], sp["fin"], alpha_edge=5)
    else:
        tp = tail_poly(f, sp["tail"])
        fin(ic, tp, sp["fin"], rays=tail_rays(f, tp))
        for (u0, u1, hh, kind) in sp.get("dorsal", []):
            dp = dorsal_poly(f, u0, u1, hh, kind)
            fin(ic, dp, sp["fin"])
        for (u0, u1, hh) in sp.get("anal", []):
            fin(ic, anal_poly(f, u0, u1, hh), sp["fin"])
    if sp.get("spines"):
        for k in range(14):
            u = 0.15 + 0.75 * k / 13
            x, cy = f.at(u)
            for s in (-1, 1):
                y = cy + s * f.h(u) * 0.95
                ic.line([(x, y), (x - 6, y + s * 16)], tone(sp["base"]), 4, amp=0.6)
    if sp.get("lure"):
        x, cy = f.at(0.12)
        top = cy - f.h(0.12)
        stem = [(x, top), (x + 30, top - 70), (x + 70, top - 88), (x + 92, top - 70)]
        ic.line(stem, (90, 76, 110), 5, amp=1.0)
        ic.glow((x + 94, top - 60, 44), sp["acc"], radius=22, strength=0.9)
        ic.drawn(circle(x + 94, top - 58, 15, 20), lighter(sp["acc"], 0.25), hatch=0.4, shade=0.15, highlight=0.2, edge=tone(sp["acc"], 0.4), edge_w=4)

    # the body: block and hatch, belly, pattern, then shade, rim and edge
    sp = f.sp
    body = ic.drawn(f.outline, sp["base"], shade=0, highlight=0, edge=None)
    bm = ic._mask(body)
    clip(ic, f.slab(0.0, 1.0, 0.25, 1.3), bm, sp["belly"])
    pattern(ic, f, sp, bm)
    if sp.get("earspot"):
        x, cy = f.at(0.17)
        clip(ic, ellipse(x - 4, cy - f.h(0.17) * 0.05, 16, 11), bm, (30, 30, 40), hatch=False)
        clip(ic, ellipse(x + 6, cy - f.h(0.17) * 0.05, 6, 9), bm, (220, 50, 40), hatch=False)
    ic.scribble_in(ic.crescent(body, (1, 1), 0.16), darker(sp["base"], 0.35), var=0.1, density=0.7, length=(30, 80), spread=14, angle=30)
    ic.scribble_in(ic.crescent(body, (-1, -1), 0.12), lighter(sp["base"], 0.5), var=0.1, density=0.6, width=5, length=(26, 60), spread=14, angle=40)
    ic.dabs(body, lighter(sp["base"], 0.65), (-1, -1), n=4, size=11, reach=0.35)
    if sp.get("glass"):
        ic.rim(body, (250, 254, 255), width=6, inset=8)
        # the spine and ribs, seen through the glassy flesh
        pts = [f.at(u) for u in (0.12 + 0.82 * i / 20 for i in range(21))]
        ic.line([(x, y + 4) for x, y in pts], (140, 170, 200), 5, amp=0.6)
        for i in range(2, 19, 2):
            x, y = pts[i]
            ic.line([(x, y - f.h(0.12 + 0.82 * i / 20) * 0.7), (x - 10, y + f.h(0.12 + 0.82 * i / 20) * 0.6)], (160, 196, 222), 3, amp=0.5)
    else:
        ic.rim(body, lighter(sp["base"], 0.35), width=5, inset=7)
    ic.outline(body, tone(sp["base"]), 10)
    if sp["pat"] == "biolum":
        biolum_dots(ic, f, sp)
    # mouth, teeth, gill, eye, the near pectoral
    x, cy = f.at(0.0)
    if sp.get("teeth"):
        mouth = [(x + 4, cy - 18), (x - 70, cy + 6), (x + 2, cy + 30)]
        ic.drawn(mouth, (26, 14, 30), hatch=0.3, shade=0, highlight=0, edge=tone(sp["base"]), edge_w=5)
        for k in range(5):
            tx = x - 8 - k * 12
            ic.drawn([(tx, cy - 12 + k * 3.5), (tx - 5, cy + 4), (tx - 10, cy - 10 + k * 3.5)], (240, 236, 220), hatch=0, shade=0, highlight=0, edge=(120, 110, 100), edge_w=2)
            ic.drawn([(tx - 2, cy + 24 - k * 3), (tx - 7, cy + 8), (tx - 12, cy + 22 - k * 3)], (240, 236, 220), hatch=0, shade=0, highlight=0, edge=(120, 110, 100), edge_w=2)
    else:
        ic.line([(x - 2, cy + 4), (x - 26, cy + 8)], tone(sp["base"], 0.7), 4, amp=0.8)
    if sp.get("tail") != "eel":
        gu = 0.19
        gx, gy = f.at(gu)
        gh = f.h(gu)
        ic.line([(gx + 8, gy - gh * 0.6), (gx - 4, gy), (gx + 6, gy + gh * 0.6)], tone(sp["base"], 0.45), 4, amp=1.0)
    eye(ic, f, sp)
    if sp.get("tail") != "eel":
        pp = pectoral_poly(f, sp)
        fin(ic, pp, lighter(sp["fin"], 0.1), rays=[(pp[0], pp[2]), (pp[0], pp[3])], alpha_edge=5)
    if sp.get("glass") or sp["pat"] == "biolum":
        ic.sparkle(120, 120, 16, lighter(sp["acc"], 0.4), ink=tone(sp["acc"]))
    if not sp.get("lure"):
        bubbles(ic, sp)


def portrait_ray(ic, sp):
    cx, cy = 280, 262
    # a diamond: blunt snout right, wing tips pointed out to the sides
    key = [(cx + 128, cy), (cx + 70, cy - 70), (cx + 10, cy - 196), (cx - 40, cy - 150), (cx - 120, cy - 50),
           (cx - 132, cy), (cx - 120, cy + 50), (cx - 40, cy + 150), (cx + 10, cy + 196), (cx + 70, cy + 70)]
    disc = []
    for i in range(len(key)):   # round the corners a little with midpoints pulled in
        a, b = key[i], key[(i + 1) % len(key)]
        disc += [a, ((a[0] * 0.5 + b[0] * 0.5 + cx * 0.0), (a[1] * 0.5 + b[1] * 0.5))]
    tail = [(cx - 110, cy - 7), (40, cy - 2), (40, cy + 2), (cx - 110, cy + 7)]
    ic.drawn(tail, sp["fin"], hatch=0.5, shade=0.1, highlight=0, edge_w=5)
    for s in (-1, 1):     # the small pelvic lobes at the back
        ic.drawn(ellipse(cx - 112, cy + s * 26, 30, 18, 20, s * 0.5), sp["fin"], hatch=0.6, shade=0.1, highlight=0, edge_w=5)
    body = ic.drawn(disc, sp["base"], shade=0, highlight=0, edge=None)
    bm = ic._mask(body)
    for _ in range(30):
        x = ic.rnd.uniform(cx - 130, cx + 120)
        y = ic.rnd.uniform(cy - 150, cy + 150)
        r = ic.rnd.uniform(7, 14)
        clip(ic, circle(x, y, r, 14), bm, sp["acc"] if ic.rnd.random() < 0.65 else lighter(sp["base"], 0.5), hatch=False)
    # the raised spine down the middle
    clip(ic, ellipse(cx - 10, cy, 110, 16), bm, darker(sp["base"], 0.15))
    ic.scribble_in(ic.crescent(body, (1, 1), 0.12), darker(sp["base"], 0.35), var=0.1, density=0.7, angle=30)
    ic.scribble_in(ic.crescent(body, (-1, -1), 0.1), lighter(sp["base"], 0.5), var=0.1, density=0.6, width=5, angle=40)
    ic.dabs(body, lighter(sp["base"], 0.65), (-1, -1), n=5, size=12, reach=0.4)
    ic.rim(body, lighter(sp["base"], 0.35), width=5, inset=8)
    ic.outline(body, tone(sp["base"]), 8)
    for s in (-1, 1):
        ic.dot(cx + 66, cy + s * 26, 10, (250, 246, 230), ink=tone(sp["base"]))
        ic.dot(cx + 69, cy + s * 26, 6, (20, 18, 24))
        ic.dot(cx + 52, cy + s * 30, 6, darker(sp["base"], 0.5))   # spiracles
    bubbles(ic, sp)


# ---------------------------------------------------------------- swatches -----
PATTERNS = {
    "gradient": ((64, 126, 160), (30, 60, 96), (234, 240, 244)),
    "stripes":  ((226, 142, 44), (50, 122, 206), None),
    "bars":     ((206, 168, 52), (58, 70, 30), None),
    "spots":    ((186, 160, 118), (104, 80, 52), None),
    "mottled":  ((108, 122, 58), (56, 68, 30), None),
    "biolum":   ((44, 34, 60), (90, 244, 255), None),
}


def swatch(ic, name):
    base, acc, extra = PATTERNS[name]
    tile = roundrect(70, 90, 442, 422, 70)
    body = ic.drawn(tile, base, shade=0, highlight=0, edge=None, angle=30)
    bm = ic._mask(body)
    r = ic.rnd
    if name == "gradient":
        clip(ic, rect(40, 60, 470, 210), bm, acc)
        clip(ic, rect(40, 210, 470, 290), bm, mix(acc, base, 0.5))
        clip(ic, rect(40, 330, 470, 460), bm, extra)
    elif name == "stripes":
        for k in range(5):
            y0 = 120 + k * 64
            pts = [(60 + i * 30, y0 + 10 * math.sin(i * 0.9 + k)) for i in range(14)]
            clip(ic, pts + [(x, y + 20) for x, y in pts[::-1]], bm, acc)
    elif name == "bars":
        for k in range(4):
            x0 = 110 + k * 88
            clip(ic, [(x0, 60), (x0 + 38, 60), (x0 + 30, 460), (x0 - 6, 460)], bm, acc)
    elif name == "spots":
        for _ in range(22):
            clip(ic, circle(r.uniform(90, 420), r.uniform(110, 400), r.uniform(12, 24), 14), bm,
                 acc if r.random() < 0.7 else lighter(base, 0.5), hatch=False)
    elif name == "mottled":
        for _ in range(12):
            clip(ic, star(r.uniform(90, 420), r.uniform(110, 400), r.uniform(30, 50), r.uniform(16, 26), 5,
                          r.uniform(0, 6), jag=0.35, seed=r.randint(0, 999)), bm, acc, hatch=False)
    ic.scribble_in(ic.crescent(body, (1, 1), 0.14), darker(base, 0.35), var=0.1, density=0.6, angle=30)
    ic.dabs(body, lighter(base, 0.6), (-1, -1), n=4, size=12)
    ic.rim(body, lighter(base, 0.35), width=5, inset=8)
    ic.outline(body, tone(base), 8)
    if name == "biolum":
        for row in range(4):
            for k in range(5):
                x, y = 120 + k * 68 + (row % 2) * 30, 140 + row * 74
                ic.glow((x, y, 22), acc, radius=12, strength=0.8)
                ic.dot(x, y, 10, lighter(acc, 0.3), ink=tone(base))


# ---------------------------------------------------------------- decor --------
def d_rock(ic):
    ic.drawn([(70, 420), (110, 260), (190, 170), (300, 150), (390, 220), (450, 420)], (128, 124, 132))
    ic.drawn([(190, 420), (210, 330), (260, 300), (310, 330), (330, 420)], (34, 30, 40), hatch=0.4, shade=0, highlight=0, edge_w=6)
    ic.drawn(ellipse(160, 250, 28, 18), (90, 150, 70), hatch=0.5, shade=0.1, highlight=0.1, edge_w=5)


def d_kelp(ic):
    for k, (x0, c) in enumerate(((180, (70, 160, 60)), (280, (96, 186, 70)), (360, (60, 140, 64)))):
        pts = []
        for i in range(12):
            t = i / 11
            pts.append((x0 + 34 * math.sin(t * 5 + k), 460 - t * (340 - k * 40)))
        left = [(x - 16 * (1 - (i / 11) * 0.6), y) for i, (x, y) in enumerate(pts)]
        right = [(x + 16 * (1 - (i / 11) * 0.6), y) for i, (x, y) in enumerate(pts)]
        ic.drawn(left + right[::-1], c, highlight=0.12, edge_w=6)
        for i in (3, 6, 9):
            x, y = pts[i]
            ic.drawn(ellipse(x + 30, y, 26, 11, 20, -0.4), lighter(c, 0.15), hatch=0.5, shade=0.1, highlight=0, edge_w=5)


def d_coral(ic):
    def branch(x, y, a, L, depth):
        x2, y2 = x + math.cos(a) * L, y + math.sin(a) * L
        ic.line([(x, y), (x2, y2)], (240, 110, 120), int(26 - depth * 6), amp=2, ink=(150, 40, 60), ink_w=8)
        if depth < 2:
            for da in (-0.5, 0.45):
                branch(x2, y2, a + da, L * 0.72, depth + 1)
        else:
            ic.dot(x2, y2, 12, (255, 170, 170), ink=(150, 40, 60))
    branch(256, 460, -math.pi / 2, 130, 0)
    ic.drawn(ellipse(256, 450, 120, 26), (220, 200, 150), hatch=0.6, shade=0.1, highlight=0, edge_w=6)


def d_sand(ic):
    ic.drawn([(50, 380), (140, 300), (260, 320), (360, 280), (462, 360), (462, 440), (50, 440)], (226, 198, 130))
    for _ in range(14):
        ic.dot(ic.rnd.uniform(90, 430), ic.rnd.uniform(370, 425), 4, (180, 150, 90))
    sh = [(200, 330), (240, 280), (290, 330)]
    ic.drawn(sh, (250, 226, 214), hatch=0.4, shade=0.1, highlight=0.1, edge_w=5)


def d_driftwood(ic):
    ic.drawn([(60, 360), (180, 300), (330, 290), (450, 220), (470, 250), (350, 340), (180, 360), (80, 400)], (150, 112, 72))
    ic.drawn([(300, 300), (330, 190), (360, 180), (350, 300)], (136, 100, 64), highlight=0.1, edge_w=6)
    for x in (140, 230, 310):
        ic.line([(x, 345), (x + 50, 330)], (100, 70, 44), 4)


def d_heater(ic):
    ic.drawn(roundrect(196, 80, 316, 430, 50), (96, 96, 108), rim_color=(170, 170, 186))
    ic.drawn(roundrect(216, 200, 296, 400, 34), (255, 120, 40), highlight=0.2, rim_color=(255, 210, 120))
    ic.glow((256, 300, 110), (255, 140, 60), radius=34, strength=0.6)
    for y in (230, 300, 370):
        ic.speed(330, y, 70, -10, (255, 200, 120), 6)


def d_chiller(ic):
    ic.drawn(roundrect(140, 150, 372, 420, 30), (120, 170, 210), rim_color=(220, 240, 255))
    for k in range(5):
        ic.line([(180 + k * 38, 200), (180 + k * 38, 380)], (60, 100, 150), 7)
    ic.sparkle(330, 130, 40, (220, 246, 255), ink=(60, 100, 150))
    ic.sparkle(150, 120, 26, (220, 246, 255), ink=(60, 100, 150))


def d_lamp(ic):
    ic.glow((256, 340, 150), (140, 220, 255), radius=40, strength=0.55)
    ic.drawn(roundrect(90, 170, 422, 240, 26), (70, 74, 90), rim_color=(150, 156, 176))
    ic.drawn(rect(120, 236, 392, 262), (190, 240, 255), hatch=0.4, shade=0, highlight=0.2, edge_w=5)
    for x in (150, 256, 362):
        ic.line([(x, 280), (x + (x - 256) * 0.5, 440)], (180, 236, 255), 8, amp=1)


def d_bubbler(ic):
    ic.drawn(roundrect(150, 400, 362, 460, 20), (90, 96, 104))
    for i in range(9):
        x = 256 + 50 * math.sin(i * 1.3)
        y = 380 - i * 36
        r = 10 + i * 2.2
        ic.outline(circle(x, y, r, 18), (150, 214, 245), 6, amp=1, passes=1)
        ic.dot(x - r * 0.35, y - r * 0.35, 4, (240, 252, 255))


DECOR = {"rock-cave": d_rock, "kelp": d_kelp, "coral": d_coral, "sand-bed": d_sand,
         "driftwood": d_driftwood, "heater": d_heater, "chiller": d_chiller, "lamp": d_lamp, "bubbler": d_bubbler}


def save(ic, path, size):
    crayon.OUT = size
    try:
        ic.save(path)
    finally:
        crayon.OUT = 128


def main(argv):
    out = argv[0] if argv and not argv[0].startswith("--") else os.path.join(ROOT, "docs", "mockups", "fish")
    os.makedirs(out, exist_ok=True)
    written = []
    for i, (name, sp) in enumerate(SPECIES.items()):
        ic = Icon(seed=100 + i)
        (portrait_ray if sp.get("view") == "top" else portrait_side)(ic, sp)
        p = os.path.join(out, f"species-{name}.png")
        save(ic, p, 256)
        written.append(p)
    for i, name in enumerate(PATTERNS):
        ic = Icon(seed=300 + i)
        swatch(ic, name)
        p = os.path.join(out, f"pattern-{name}.png")
        save(ic, p, 128)
        written.append(p)
    for i, (name, fn) in enumerate(DECOR.items()):
        ic = Icon(seed=500 + i)
        fn(ic)
        p = os.path.join(out, f"decor-{name}.png")
        save(ic, p, 128)
        written.append(p)
    print("wrote", len(written), "PNGs to", out)
    if "--sheet" in argv:
        sheet = argv[argv.index("--sheet") + 1]
        subprocess.run([sys.executable, os.path.join(KIT, "sheet.py"), sheet] + written
                       + ["--ref", "inferno", "--ref", "vitality", "--ref", "magnetic field"], check=True)


if __name__ == "__main__":
    main(sys.argv[1:])
