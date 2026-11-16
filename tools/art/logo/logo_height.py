"""The Pale Blue Dot logo's stone, as height fields (owner, 2026-11-16).

The owner: "can you make me a logo? that little blue marble you have on the menu is perfect for it,
but for the logo I'd like it to be a screenshot of the planet from the game, inside the marble. and
rendered is a high quality version of that rock surrounding the blue marble. with the dot being the
moon", laid out as

             O
    PALE BLUE
          D(moon)T

and "pale blue is etched into the stone with beautiful markings depicting hunting, fishing,
farming, sailing and rocketry", "use blender sculpting skills".

The `blender-organic-panels` method: every stone form is z = h(x, y), computed here in numpy with
masks (the tablet, the emblem's rock ring and its socket, the DOT stone), cut with V-groove chisel
profiles (letters and pictographs drawn with PIL, then shaped by their distance transform), and
weathered with layered noise and cracks. `logo_render.py` turns the field into one mesh and renders
it in Cycles with the marble and the moon as real spheres.

    <venv with numpy, pillow, scipy>/bin/python tools/art/logo/logo_height.py <out dir>
writes field.npz (height, masks) and preview.png (a hillshade, for judging the cuts quickly).
"""

import math
import os
import sys

import numpy as np
from PIL import Image, ImageDraw, ImageFont
from scipy import ndimage

W, H = 3200, 2150              # canvas, px (the render is the same size)
FONT = "/mnt/skills/examples/canvas-design/canvas-fonts/YoungSerif-Regular.ttf"   # (owner: "previous font was better")
FONT_DOT = FONT

# Where things stand (px, canvas origin top left).
TABLET = (230, 760, 2970, 1440)          # x0, y0, x1, y1
DOTSTONE = (900, 1290, 2300, 1900)
EMBLEM = (1600, 470, 435)                # cx, cy, radius
MARBLE_R = 268                           # the planet marble's radius
MOON = (1600, 1590, 118)                 # cx, cy, radius
MOON_SOCKET = 150

rng = np.random.default_rng(7)
PLACED = []   # where each frieze scene was etched, for the key
YY, XX = np.mgrid[0:H, 0:W].astype(np.float32)


def fbm(scale, octaves=5, seed=0):
    """Value noise summed over octaves, 0..1, at `scale` px for the first octave."""
    r = np.random.default_rng(seed)
    out = np.zeros((H, W), np.float32)
    amp, total = 1.0, 0.0
    s = scale
    for _ in range(octaves):
        gh, gw = int(H / s) + 3, int(W / s) + 3
        grid = r.random((gh, gw)).astype(np.float32)
        layer = ndimage.zoom(grid, (H / (gh - 3) * 1.0, W / (gw - 3) * 1.0), order=3)[:H, :W]
        out += amp * layer
        total += amp
        amp *= 0.5
        s /= 2.0
    return out / total


def rounded_rect(x0, y0, x1, y1, r):
    dx = np.maximum(np.maximum(x0 + r - XX, XX - (x1 - r)), 0)
    dy = np.maximum(np.maximum(y0 + r - YY, YY - (y1 - r)), 0)
    return r - np.hypot(dx, dy)              # >0 inside, distance to the rounded edge


def slab(rect, radius, wobble, seed):
    """A stone slab: its signed inset (px) with a hand-broken edge."""
    d = rounded_rect(*rect, radius)
    edge = (fbm(140, 4, seed) - 0.5) * wobble + (fbm(30, 3, seed + 1) - 0.5) * wobble * 0.35
    return d + edge


def bevel(inset, width, height):
    """The slab's body: rises over `width` px from its edge to `height`, rounded like worn stone."""
    t = np.clip(inset / width, 0, 1)
    return height * np.sin(t * math.pi / 2) ** 0.8


def groove(mask, depth, width):
    """A V-cut chisel groove from a drawn mask: deepest on the stroke's middle line."""
    d = ndimage.distance_transform_edt(mask)
    return -depth * np.clip(d / width, 0, 1) ** 0.85


def raised(mask, height, round_px, channel_px):
    """A letter standing proud of the stone in a deep cut channel: its face rounded at the edge over
    `round_px`, the channel `channel_px` wide round it, deepest against the letter."""
    inside = ndimage.distance_transform_edt(mask)
    outside = ndimage.distance_transform_edt(~mask)
    face = height * np.clip(inside / round_px, 0, 1) ** 0.6
    chan = np.where(~mask & (outside < channel_px), -0.16 * (1 - outside / max(channel_px, 1e-6)) ** 1.4, 0) if channel_px > 0 else 0
    return np.where(mask, face, chan)


def text_mask(text, size, cx, cy, font=FONT, spacing=0.0):
    img = Image.new("L", (W, H), 0)
    dr = ImageDraw.Draw(img)
    f = ImageFont.truetype(font, size)
    # letter by letter, so spacing is ours
    widths = [dr.textlength(c, font=f) for c in text]
    total = sum(widths) + spacing * size * (len(text) - 1)
    x = cx - total / 2
    for c, w in zip(text, widths):
        dr.text((x, cy), c, font=f, fill=255, anchor="lm")
        x += w + spacing * size
    return np.array(img) > 127, x


# ---------------- the petroglyphs: hunting, fishing, farming, sailing, rocketry ----------------
def glyphs(draw, x, y, s, which, w):
    """One scene at (x, y), the band's middle line, `s` px tall, strokes `w` px wide."""
    L = lambda *p: draw.line([(x + a * s, y + b * s) for a, b in p], fill=255, width=w, joint="curve")
    C = lambda a, b, r: draw.ellipse([x + (a - r) * s, y + (b - r) * s, x + (a + r) * s, y + (b + r) * s], outline=255, width=w)
    D = lambda a, b, r: draw.ellipse([x + (a - r) * s, y + (b - r) * s, x + (a + r) * s, y + (b + r) * s], fill=255)
    A = lambda a, b, r, s0, s1: draw.arc([x + (a - r) * s, y + (b - r) * s, x + (a + r) * s, y + (b + r) * s], s0, s1, fill=255, width=w)

    def man(a, b, arms):
        D(a, b - 0.42, 0.1)                           # head
        draw.polygon([(x + (a - 0.09) * s, y + (b - 0.32) * s), (x + (a + 0.09) * s, y + (b - 0.32) * s),
                      (x + (a + 0.05) * s, y + (b + 0.06) * s), (x + (a - 0.05) * s, y + (b + 0.06) * s)], fill=255)
        L((a, b + 0.05), (a - 0.12, b + 0.42))        # legs
        L((a, b + 0.05), (a + 0.12, b + 0.42))
        L(*arms)

    if which == "hunting":
        # a hunter drawing a bow at a stag
        man(0.25, 0.0, [(0.25, -0.2), (0.45, -0.22)])
        A(0.47, -0.22, 0.2, 270, 90)                  # the bow
        L((0.47, -0.42), (0.47, -0.02))               # its string
        L((0.47, -0.22), (0.95, -0.22))               # the arrow
        L((0.95, -0.22), (0.88, -0.28)); L((0.95, -0.22), (0.88, -0.16))
        # the stag: body, legs, neck, head, antlers
        draw.ellipse([x + 1.16 * s, y - 0.14 * s, x + 1.68 * s, y + 0.1 * s], fill=255)
        L((1.24, -0.05), (1.2, 0.38)); L((1.32, -0.05), (1.34, 0.38))
        L((1.52, -0.05), (1.5, 0.38)); L((1.6, -0.05), (1.66, 0.38))
        L((1.62, -0.05), (1.72, -0.3)); D(1.76, -0.32, 0.07)
        L((1.72, -0.36), (1.62, -0.62)); L((1.66, -0.5), (1.55, -0.56))
        L((1.78, -0.38), (1.88, -0.64)); L((1.84, -0.52), (1.96, -0.58))
        return 2.1
    if which == "fishing":
        # a fisher sitting in a canoe on the water, the rod out over the bow, the line down to a fish
        for k in range(5):
            A(0.2 + k * 0.38, 0.12, 0.19, 200, 340)    # the water
        draw.polygon([(x + 0.12 * s, y - 0.1 * s), (x + 1.08 * s, y - 0.1 * s), (x + 0.92 * s, y + 0.06 * s),
                      (x + 0.28 * s, y + 0.06 * s)], fill=255)          # the canoe
        D(0.5, -0.5, 0.1)                                               # the fisher's head
        draw.polygon([(x + 0.41 * s, y - 0.4 * s), (x + 0.59 * s, y - 0.4 * s), (x + 0.62 * s, y - 0.1 * s),
                      (x + 0.38 * s, y - 0.1 * s)], fill=255)           # sitting: the body down into the canoe
        L((0.56, -0.3), (0.84, -0.42))                                  # the arm to the rod
        L((0.78, -0.36), (1.42, -0.86))                                 # the rod
        L((1.42, -0.86), (1.5, 0.36))                                   # the line
        draw.ellipse([x + 1.32 * s, y + 0.36 * s, x + 1.72 * s, y + 0.56 * s], fill=255)   # the fish
        draw.polygon([(x + 1.7 * s, y + 0.46 * s), (x + 1.86 * s, y + 0.34 * s), (x + 1.86 * s, y + 0.58 * s)], fill=255)
        return 2.0
    if which == "farming":
        # the sun, a farmer at the hoe, three stalks of wheat
        C(0.2, -0.55, 0.13)
        for k in range(8):
            a = k * math.pi / 4
            L((0.2 + math.cos(a) * 0.19, -0.55 + math.sin(a) * 0.19), (0.2 + math.cos(a) * 0.28, -0.55 + math.sin(a) * 0.28))
        man(0.6, 0.0, [(0.6, -0.2), (0.9, 0.05)])
        L((0.82, -0.18), (1.0, 0.38)); L((1.0, 0.38), (1.12, 0.32))   # the hoe
        for k in range(3):
            a = 1.35 + k * 0.22
            L((a, 0.42), (a + 0.02, -0.38))
            for j in range(4):
                yy = -0.32 + j * 0.12
                L((a + 0.01, yy + 0.06), (a - 0.08, yy - 0.04)); L((a + 0.01, yy + 0.06), (a + 0.1, yy - 0.04))
        L((0.95, 0.42), (2.0, 0.42))                   # the furrow
        return 2.1
    if which == "sailing":
        # a ship under sail on the waves, a bird over it
        L((0.15, 0.12), (1.45, 0.12)); L((0.15, 0.12), (0.35, 0.36)); L((1.45, 0.12), (1.25, 0.36)); L((0.35, 0.36), (1.25, 0.36))
        L((0.8, 0.12), (0.8, -0.78))                   # the mast
        draw.polygon([(x + 0.85 * s, y - 0.72 * s), (x + 1.35 * s, y - 0.02 * s), (x + 0.85 * s, y - 0.02 * s)], fill=255)
        draw.polygon([(x + 0.75 * s, y - 0.62 * s), (x + 0.32 * s, y - 0.02 * s), (x + 0.75 * s, y - 0.02 * s)], fill=255)
        L((0.8, -0.78), (0.95, -0.72)); L((0.95, -0.72), (0.8, -0.66))   # the pennant
        for k in range(4):
            A(0.1 + k * 0.42, 0.5, 0.21, 200, 340)
        A(1.65, -0.6, 0.1, 200, 340); A(1.85, -0.6, 0.1, 200, 340)          # a bird
        return 2.0
    if which == "rocketry":
        # stars, the moon, a rocket rising on its flame
        C(1.55, -0.45, 0.22)
        D(1.48, -0.5, 0.04); D(1.62, -0.38, 0.05)
        for a, b in ((0.25, -0.65), (0.6, -0.8), (1.05, -0.72), (1.95, -0.1), (0.15, -0.15)):
            L((a - 0.06, b), (a + 0.06, b)); L((a, b - 0.06), (a, b + 0.06))
        # the rocket, tilted toward the moon
        ang = -0.55
        c, si = math.cos(ang), math.sin(ang)
        R = lambda u, v: (0.75 + u * c - v * si, 0.05 + u * si + v * c)
        body = [R(-0.32, -0.09), R(0.18, -0.09), R(0.38, 0.0), R(0.18, 0.09), R(-0.32, 0.09)]
        draw.polygon([(x + a * s, y + b * s) for a, b in body], fill=255)
        draw.polygon([(x + a * s, y + b * s) for a, b in (R(-0.32, -0.09), R(-0.44, -0.2), R(-0.22, -0.09))], fill=255)
        draw.polygon([(x + a * s, y + b * s) for a, b in (R(-0.32, 0.09), R(-0.44, 0.2), R(-0.22, 0.09))], fill=255)
        D(*R(0.06, 0.0), 0.04)
        for k, (u, v) in enumerate(((-0.5, 0.0), (-0.62, -0.06), (-0.62, 0.06), (-0.78, 0.0))):
            L(R(-0.34, 0.0), R(u, v))                  # the flame
        return 2.1
    if which == "spiral":
        pts = [(0.45 + math.cos(t) * t * 0.045, -0.2 + math.sin(t) * t * 0.045) for t in np.linspace(0, 4.2 * math.pi, 60)]
        L(*pts)
        return 1.0
    if which == "sun":
        C(0.4, -0.2, 0.16)
        for k in range(10):
            a = k * math.tau / 10
            L((0.4 + math.cos(a) * 0.23, -0.2 + math.sin(a) * 0.23), (0.4 + math.cos(a) * 0.36, -0.2 + math.sin(a) * 0.36))
        return 0.9
    if which == "fish":
        draw.ellipse([x + 0.05 * s, y - 0.32 * s, x + 0.6 * s, y - 0.06 * s], fill=255)
        L((0.6, -0.19), (0.8, -0.36)); L((0.6, -0.19), (0.8, -0.02)); L((0.8, -0.36), (0.8, -0.02))
        D(0.16, -0.22, 0.035)
        return 0.9
    if which == "bird":
        A(0.25, -0.2, 0.18, 200, 340); A(0.55, -0.2, 0.18, 200, 340)
        return 0.8
    if which == "star":
        for k in range(4):
            a = k * math.pi / 4
            L((0.3 - math.cos(a) * 0.18, -0.2 - math.sin(a) * 0.18), (0.3 + math.cos(a) * 0.18, -0.2 + math.sin(a) * 0.18))
        return 0.6
    if which == "hand":
        draw.ellipse([x + 0.15 * s, y - 0.25 * s, x + 0.47 * s, y + 0.1 * s], fill=255)
        for k, (a, b) in enumerate(((0.12, -0.38), (0.22, -0.55), (0.32, -0.58), (0.42, -0.52), (0.55, -0.2))):
            L((0.18 + k * 0.07 if k < 4 else 0.46, -0.2 if k < 4 else -0.05), (a, b))
        return 0.75
    if which == "stag":
        draw.ellipse([x + 0.06 * s, y - 0.14 * s, x + 0.58 * s, y + 0.1 * s], fill=255)
        L((0.14, -0.05), (0.1, 0.38)); L((0.22, -0.05), (0.24, 0.38))
        L((0.42, -0.05), (0.4, 0.38)); L((0.5, -0.05), (0.56, 0.38))
        L((0.52, -0.05), (0.62, -0.3)); D(0.66, -0.32, 0.07)
        L((0.62, -0.36), (0.52, -0.62)); L((0.68, -0.38), (0.78, -0.64))
        return 0.95
    if which == "comet":
        # the story's start: a comet breaking apart, its pieces scattering knowledge over the world
        D(0.25, -0.62, 0.12)
        for k, (a, b) in enumerate(((0.32, -0.7), (0.4, -0.58), (0.36, -0.5))):
            L((a, b), (a + 0.42, b - 0.2 + k * 0.06))          # the tail
        for a, b, r in ((0.55, -0.3, 0.05), (0.75, -0.12, 0.04), (0.95, 0.05, 0.05), (0.62, 0.0, 0.035), (1.05, -0.25, 0.035)):
            D(a, b, r)                                          # the pieces
            L((a - 0.05, b - 0.08), (a - 0.12, b - 0.16))
        A(0.85, 0.75, 0.45, 210, 330)                           # the world they fall on
        return 1.4
    if which == "rafting":
        man(0.55, -0.18, [(0.55, -0.4), (0.85, -0.6)])
        L((0.85, -0.65), (1.0, 0.3))                            # the pole into the river
        for k in range(3):
            L((0.15, 0.22 + k * 0.0), (1.05, 0.22 + k * 0.0))
        draw.rectangle([x + 0.12 * s, y + 0.2 * s, x + 1.08 * s, y + 0.3 * s], fill=255)
        for k in range(4):
            A(0.15 + k * 0.32, 0.45, 0.16, 200, 340)
        return 1.3
    if which == "flying":
        # a biplane over the clouds
        draw.ellipse([x + 0.15 * s, y - 0.38 * s, x + 1.15 * s, y - 0.18 * s], fill=255)
        L((0.45, -0.55), (0.85, -0.55)); L((0.45, -0.05), (0.85, -0.05))
        L((0.55, -0.55), (0.55, -0.05)); L((0.78, -0.55), (0.78, -0.05))
        L((0.15, -0.28), (0.05, -0.55)); L((1.15, -0.28), (1.22, -0.45)); L((1.15, -0.28), (1.22, -0.1))
        A(0.4, 0.3, 0.14, 180, 360); A(0.62, 0.3, 0.18, 180, 360); A(0.86, 0.3, 0.14, 180, 360)
        return 1.3
    if which == "shipbuilding":
        # a hull on the slipway, its ribs up, a builder with a hammer
        A(0.75, -0.55, 0.62, 30, 150)
        for k in range(5):
            a = 0.35 + k * 0.2
            L((a, -0.25 + abs(a - 0.75) * 0.25), (a, -0.68))
        L((0.1, 0.2), (1.4, 0.0))
        man(1.45, -0.1, [(1.45, -0.3), (1.3, -0.55)])
        L((1.25, -0.6), (1.38, -0.5))
        return 1.65
    if which == "mining":
        A(0.45, 0.1, 0.42, 180, 360)                            # the mine's mouth
        L((0.03, 0.1), (0.03, 0.4)); L((0.87, 0.1), (0.87, 0.4))
        man(1.2, 0.0, [(1.2, -0.25), (0.98, -0.42)])
        A(0.98, -0.42, 0.14, 120, 300)                          # the pick
        for a, b in ((0.3, 0.3), (0.5, 0.22), (0.62, 0.35)):
            D(a, b, 0.04)
        return 1.45
    if which == "windmill":
        draw.polygon([(x + 0.4 * s, y + 0.4 * s), (x + 0.8 * s, y + 0.4 * s), (x + 0.68 * s, y - 0.3 * s), (x + 0.52 * s, y - 0.3 * s)], fill=255)
        for k in range(4):
            a = math.pi / 4 + k * math.pi / 2
            L((0.6, -0.36), (0.6 + math.cos(a) * 0.55, -0.36 + math.sin(a) * 0.55))
            draw.polygon([(x + (0.6 + math.cos(a) * 0.2) * s, y + (-0.36 + math.sin(a) * 0.2) * s),
                          (x + (0.6 + math.cos(a) * 0.55) * s, y + (-0.36 + math.sin(a) * 0.55) * s),
                          (x + (0.6 + math.cos(a + 0.35) * 0.5) * s, y + (-0.36 + math.sin(a + 0.35) * 0.5) * s)], fill=255)
        return 1.2
    if which == "watermill":
        C(0.45, -0.1, 0.36)
        for k in range(8):
            a = k * math.pi / 4
            L((0.45, -0.1), (0.45 + math.cos(a) * 0.36, -0.1 + math.sin(a) * 0.36))
        draw.rectangle([x + 0.82 * s, y - 0.5 * s, x + 1.3 * s, y + 0.25 * s], fill=255)
        draw.polygon([(x + 0.76 * s, y - 0.5 * s), (x + 1.06 * s, y - 0.8 * s), (x + 1.36 * s, y - 0.5 * s)], fill=255)
        for k in range(4):
            A(0.15 + k * 0.32, 0.42, 0.16, 200, 340)
        return 1.45
    if which == "nuclear":
        D(0.6, -0.2, 0.09)
        for k in range(3):
            pts = []
            for j in range(41):
                t = j / 40 * math.tau
                ex, ey = math.cos(t) * 0.5, math.sin(t) * 0.18
                a = k * math.pi / 3
                pts.append((0.6 + ex * math.cos(a) - ey * math.sin(a), -0.2 + ex * math.sin(a) + ey * math.cos(a)))
            L(*pts)
        return 1.25
    if which == "fusion":
        # a star held in a ring: the torus, and the fire at its heart
        draw.ellipse([x + 0.1 * s, y - 0.48 * s, x + 1.2 * s, y + 0.1 * s], outline=255, width=w)
        draw.ellipse([x + 0.32 * s, y - 0.34 * s, x + 0.98 * s, y - 0.04 * s], outline=255, width=w)
        for k in range(6):
            a = k * math.pi / 3
            L((0.65 + math.cos(a) * 0.04, -0.19 + math.sin(a) * 0.04), (0.65 + math.cos(a) * 0.13, -0.19 + math.sin(a) * 0.13))
        L((0.3, 0.1), (0.2, 0.42)); L((1.0, 0.1), (1.1, 0.42))
        return 1.3
    if which == "antimatter":
        C(0.35, -0.2, 0.2); C(1.05, -0.2, 0.2)
        L((0.27, -0.2), (0.43, -0.2)); L((0.35, -0.28), (0.35, -0.12))   # +
        L((0.97, -0.2), (1.13, -0.2))                                     # -
        for k in range(8):
            a = k * math.pi / 4
            L((0.7 + math.cos(a) * 0.06, -0.2 + math.sin(a) * 0.06), (0.7 + math.cos(a) * 0.15, -0.2 + math.sin(a) * 0.15))
        return 1.4
    if which == "dyson":
        D(0.65, -0.2, 0.12)
        r2 = np.random.default_rng(4)
        for ring, n in ((0.3, 10), (0.45, 16), (0.58, 22)):
            for k in range(n):
                a = k * math.tau / n + r2.uniform(0, 0.2)
                D(0.65 + math.cos(a) * ring, -0.2 + math.sin(a) * ring * 0.8, 0.025)
        return 1.3
    if which == "computer":
        # racks of a great computer, its lights, the one who tends it
        for k in range(3):
            x0r = 0.1 + k * 0.32
            draw.rectangle([x + x0r * s, y - 0.7 * s, x + (x0r + 0.24) * s, y + 0.4 * s], outline=255, width=w)
            for j in range(5):
                D(x0r + 0.12, -0.55 + j * 0.2, 0.03)
        man(1.25, 0.0, [(1.25, -0.2), (1.05, -0.2)])
        return 1.45
    if which == "solar":
        C(1.2, -0.6, 0.1)
        for k in range(3):
            x0p = 0.1 + k * 0.38
            draw.polygon([(x + x0p * s, y + 0.0 * s), (x + (x0p + 0.32) * s, y - 0.18 * s), (x + (x0p + 0.32) * s, y - 0.02 * s), (x + x0p * s, y + 0.16 * s)], fill=255)
            L((x0p + 0.16, 0.08), (x0p + 0.16, 0.42))
        return 1.4
    if which == "station":
        C(0.6, -0.2, 0.36)
        C(0.6, -0.2, 0.08)
        for k in range(4):
            a = k * math.pi / 2
            L((0.6 + math.cos(a) * 0.08, -0.2 + math.sin(a) * 0.08), (0.6 + math.cos(a) * 0.36, -0.2 + math.sin(a) * 0.36))
        draw.rectangle([x + 1.0 * s, y - 0.32 * s, x + 1.3 * s, y - 0.08 * s], fill=255)
        L((0.96, -0.2), (1.0, -0.2))
        return 1.35
    if which == "colony":
        A(0.7, 0.9, 0.95, 220, 320)                             # the new world's curve
        A(0.6, -0.05, 0.36, 180, 360)                           # the dome
        draw.rectangle([x + 0.45 * s, y - 0.25 * s, x + 0.55 * s, y - 0.05 * s], fill=255)
        draw.rectangle([x + 0.62 * s, y - 0.3 * s, x + 0.74 * s, y - 0.05 * s], fill=255)
        for a, b in ((1.2, -0.6), (0.15, -0.65), (1.35, -0.25)):
            L((a - 0.05, b), (a + 0.05, b)); L((a, b - 0.05), (a, b + 0.05))
        return 1.45
    if which == "starship":
        # the story's end: the starship, built from all of it, leaving for the stars
        draw.polygon([(x + 0.15 * s, y - 0.2 * s), (x + 1.25 * s, y - 0.32 * s), (x + 1.45 * s, y - 0.2 * s),
                      (x + 1.25 * s, y - 0.08 * s)], fill=255)
        draw.polygon([(x + 0.55 * s, y - 0.25 * s), (x + 0.3 * s, y - 0.55 * s), (x + 0.75 * s, y - 0.26 * s)], fill=255)
        draw.polygon([(x + 0.55 * s, y - 0.15 * s), (x + 0.3 * s, y + 0.15 * s), (x + 0.75 * s, y - 0.14 * s)], fill=255)
        for k in range(3):
            L((0.12, -0.2 + (k - 1) * 0.06), (-0.05, -0.2 + (k - 1) * 0.12))
        for a, b in ((1.5, -0.6), (1.6, 0.1), (1.0, -0.65)):
            L((a - 0.05, b), (a + 0.05, b)); L((a, b - 0.05), (a, b + 0.05))
        return 1.65
    raise ValueError(which)


SPAN = {"hunting": 2.1, "fishing": 2.0, "farming": 2.1, "sailing": 2.0, "rocketry": 2.1,
        "comet": 1.4, "rafting": 1.3, "flying": 1.3, "shipbuilding": 1.65, "mining": 1.45, "windmill": 1.2,
        "watermill": 1.45, "nuclear": 1.25, "fusion": 1.3, "antimatter": 1.4, "dyson": 1.3, "computer": 1.45,
        "solar": 1.4, "station": 1.35, "colony": 1.45, "starship": 1.65,
        "spiral": 1.0, "sun": 0.9, "fish": 0.9, "bird": 0.8, "star": 0.6, "hand": 0.75, "stag": 0.95}


def scatter(free, items, seed, w=12):
    """Cave paintings: each item drawn alone, mirrored and turned at random, and set down at random
    where `free` allows and nothing else stands. `items` = [(name, size px)]. Returns a paint map:
    0 bare stone, 1 red ochre, 2 white clay, 3 charcoal (as on painted cave walls, ochre most)."""
    r = np.random.default_rng(seed)
    out = np.zeros((H, W), np.uint8)
    taken = ~free
    for name, size in items:
        span = SPAN[name]
        cw, ch = int(span * size + 5 * w + size * 0.3), int(1.6 * size + 5 * w)
        tile = Image.new("L", (cw, ch), 0)
        glyphs(ImageDraw.Draw(tile), w * 2.5 + size * 0.1, ch * 0.62, size, name, max(6, int(size * 0.075)))
        if r.random() < 0.5:
            tile = tile.transpose(Image.FLIP_LEFT_RIGHT)
        tile = tile.rotate(float(r.uniform(-32, 32)), resample=Image.BILINEAR, expand=True)
        t = np.array(tile) > 110
        th, tw = t.shape
        pad = ndimage.binary_dilation(t, iterations=int(6 + size * 0.06))
        paint = int(r.choice([1, 1, 1, 1, 2, 3]))
        for _ in range(2500):
            x, y = int(r.integers(0, W - tw)), int(r.integers(0, H - th))
            if not (taken[y:y + th, x:x + tw] & pad).any():
                out[y:y + th, x:x + tw][t] = paint
                taken[y:y + th, x:x + tw] |= pad
                break
    return out


def poly_slab(points, wobble, seed):
    """A stone cut to a polygon: its signed inset (px) with a hand-broken edge."""
    img = Image.new("L", (W, H), 0)
    ImageDraw.Draw(img).polygon(points, fill=255)
    m = np.array(img) > 127
    inset = ndimage.distance_transform_edt(m) - ndimage.distance_transform_edt(~m)
    edge = (fbm(140, 4, seed) - 0.5) * wobble + (fbm(30, 3, seed + 1) - 0.5) * wobble * 0.35
    return inset + edge


def carved_border(points, inset, d0, d1, seed, scroll_edges=None, frieze_edges=None, frieze_items=None, frieze_skip=None):
    """The hand-carved frame inside a polygon stone, between insets d0 and d1: two raised rims and,
    between them on a sunk floor, a running scroll (the 'running dog': spirals that turn alternately,
    linked) carved along every edge. Returns (height, band mask)."""
    band = (inset > d0) & (inset < d1)
    mid, half = (d0 + d1) / 2, (d1 - d0) / 2
    hgt = np.where(band, -0.05, 0.0)
    hgt += 0.07 * np.exp(-((inset - d0 - 6) / 5.0) ** 2) + 0.07 * np.exp(-((inset - d1 + 6) / 5.0) ** 2)
    img = Image.new("L", (W, H), 0)
    dr = ImageDraw.Draw(img)
    cx = sum(p[0] for p in points) / len(points)
    cy = sum(p[1] for p in points) / len(points)
    rad = half * 0.62
    stroke = max(5, int(half * 0.22))
    fimg = Image.new("L", (W, H), 0)
    fdr = ImageDraw.Draw(fimg)
    fs = half * 1.42                      # a scene's size, so it fills the band's height
    fw = max(6, int(fs * 0.085))
    for ei, ((ax, ay), (bx, by)) in enumerate(zip(points, points[1:] + points[:1])):
        L = math.hypot(bx - ax, by - ay)
        if frieze_edges and ei in frieze_edges:
            # the scenes etched along a level edge, each once, spread evenly over each run
            probe = int(ay + mid), int((ax + bx) / 2)
            yb = ay + (mid if inset[min(probe[0], H - 1), probe[1]] > 0 else -mid)
            for (r0, r1, seq) in frieze_items[ei]:
                sizes = [fs * (0.95 if SPAN[n] < 1.7 else 0.78) for n in seq]
                widths = [SPAN[n] * z for n, z in zip(seq, sizes)]
                gap = (r1 - r0 - sum(widths)) / (len(seq) + 1)
                x = r0 + gap
                for n, z, wd in zip(seq, sizes, widths):
                    glyphs(fdr, x, yb + z * 0.18, z, n, fw)
                    PLACED.append({"name": n, "x": x + wd / 2, "y": yb})
                    x += wd + gap
            continue
        if scroll_edges is not None and ei not in scroll_edges:
            continue
        ux, uy = (bx - ax) / L, (by - ay) / L
        nx, ny = -uy, ux
        tx_, ty_ = int((ax + bx) / 2 + nx * mid), int((ay + by) / 2 + ny * mid)
        if not (0 <= tx_ < W and 0 <= ty_ < H and inset[ty_, tx_] > 0):
            nx, ny = -nx, -ny
        step = rad * 2.6
        n = int((L - 2 * mid) / step)
        if n < 1:
            continue
        start = (L - n * step) / 2 + step / 2
        for k in range(n):
            t = start + k * step
            px, py = ax + ux * t + nx * mid, ay + uy * t + ny * mid
            sgn = 1 if k % 2 == 0 else -1
            pts = []
            for j in range(40):
                a = j / 39 * 1.6 * math.tau
                rr = rad * (1 - j / 39 * 0.85)
                ca, sa = math.cos(a * sgn), math.sin(a * sgn)
                # the spiral in the band's own frame (along the edge, across it)
                pts.append((px + (ux * ca + nx * sa) * rr, py + (uy * ca + ny * sa) * rr))
            dr.line(pts, fill=255, width=stroke, joint="curve")
            # the link to the next spiral, a wave along the band
            if k < n - 1:
                qx, qy = px + ux * step, py + uy * step
                link = [(px + ux * rad * 0.9 + nx * rad * 0.2 * sgn, py + uy * rad * 0.9 + ny * rad * 0.2 * sgn),
                        ((px + qx) / 2 + nx * rad * 0.7 * sgn, (py + qy) / 2 + ny * rad * 0.7 * sgn),
                        (qx - ux * rad * 0.9 - nx * rad * 0.2 * sgn, qy - uy * rad * 0.9 - ny * rad * 0.2 * sgn)]
                dr.line(link, fill=255, width=stroke, joint="curve")
    scroll = (np.array(img) > 127) & band
    hgt += raised(scroll, 0.09, 3.0, 0)
    etched = (np.array(fimg) > 110) & band
    peck = np.random.default_rng(seed).random((H, W)) * 0.4 + 0.6
    hgt += np.where(etched, -0.06 * peck, 0)
    return hgt, band, etched


def frieze(band, scenes, s, w, divider=True):
    """A band of scenes from x0 to x1 at mid-line y, spaced evenly, dots between."""
    x0, x1, y = band
    img = Image.new("L", (W, H), 0)
    dr = ImageDraw.Draw(img)
    widths = [{"hunting": 2.1, "fishing": 2.0, "farming": 2.1, "sailing": 2.0, "rocketry": 2.1}[k] * s for k in scenes]
    gap = (x1 - x0 - sum(widths)) / max(len(scenes) - 1, 1)
    x = x0
    for k, wd in zip(scenes, widths):
        glyphs(dr, x, y, s, k, w)
        if divider and k != scenes[-1]:
            # a dotted trail from one scene to the next: the story goes on
            n = max(2, int((gap - s * 0.3) / (w * 4.2)))
            for j in range(n):
                cx = x + wd + s * 0.15 + (gap - s * 0.3) * (j + 0.5) / n
                dr.ellipse([cx - w * 0.8, y - w * 0.8, cx + w * 0.8, y + w * 0.8], fill=255)
        x += wd + gap
    return np.array(img) > 127


def main(out):
    os.makedirs(out, exist_ok=True)
    rough = fbm(220, 6, 11)
    grain = fbm(18, 3, 12)
    pits = fbm(9, 2, 13)

    # ---- one slab (owner: "merge the outline shapes for pale blue dot together instead of two
    # different slabs"): a banner with angled ends, a lobe hanging from its middle for DOT, a
    # hand-carved frame all round, the scenes etched along its level edges, rays behind the words
    tx0, ty0, tx1, ty1 = TABLET
    dx0, dy0, dx1, dy1 = DOTSTONE
    tmid, dmid = (ty0 + ty1) / 2, (dy0 + dy1 + 10) / 2
    t_pts = [(tx0 - 70, tmid), (tx0 + 110, ty0), (tx1 - 110, ty0), (tx1 + 70, tmid), (tx1 - 110, ty1),
             (dx1 + 60, ty1), (dx1 + 130, dmid), (dx1, dy1 + 10), (dx0, dy1 + 10), (dx0 - 130, dmid), (dx0 - 60, ty1),
             (tx0 + 110, ty1)]
    t_in = poly_slab(t_pts, 34, 21)
    t_mask = t_in > 0
    h = np.where(t_mask, 0.42 + bevel(t_in, 60, 0.28) + (rough - 0.5) * 0.08 + (grain - 0.5) * 0.02, 0.0)
    ecx0, _, er0 = EMBLEM
    L0, R0 = tx0 + 110 + 120, tx1 - 110 - 120
    runs = {
        1: [(L0, ecx0 - er0 - 30, ["comet", "hunting", "fishing", "farming"]),
            (ecx0 + er0 + 30, R0, ["rafting", "sailing", "mining", "shipbuilding"])],
        4: [(dx1 + 60 + 60, tx1 - 110 - 70, ["windmill", "watermill", "flying"])],
        7: [(dx0 + 70, dx1 - 70, ["nuclear", "rocketry", "computer", "solar", "fusion", "station", "colony"])],
        10: [(tx0 + 110 + 70, dx0 - 60 - 60, ["dyson", "antimatter", "starship"])],
    }
    t_border, t_band, t_etch = carved_border(t_pts, t_in, 30, 150, 22,
                                              scroll_edges={0, 2, 3, 5, 6, 8, 9, 11},
                                              frieze_edges=set(runs), frieze_items=runs)
    h += np.where(t_mask, t_border, 0)
    inner = t_in - 154          # the field inside the frame
    # the letters stand well proud (owner: "make text pop out more so theres more dramatic shadows")
    letters, _ = text_mask("PALE BLUE", 370, (tx0 + tx1) / 2, tmid + 20, spacing=0.03)
    letters &= t_mask
    h += raised(letters, 0.55, 9.0, 0)
    mcx, mcy, mr = MOON
    dl, _ = text_mask("D", 380, mcx - MOON_SOCKET - 150, dmid + 8)
    tl, _ = text_mask("T", 380, mcx + MOON_SOCKET + 140, dmid + 8)
    dot_letters = (dl | tl) & t_mask
    h += raised(dot_letters, 0.55, 9.0, 0)
    rr = np.hypot(XX - mcx, YY - mcy)
    lip = np.exp(-((rr - MOON_SOCKET) / 14.0) ** 2) * 0.10
    cup = np.where(rr < MOON_SOCKET, -0.22 * np.sqrt(np.clip(1 - (rr / MOON_SOCKET) ** 2, 0, 1)) - 0.06, 0)
    h += np.where(t_mask, lip + cup, 0)
    d_mask = np.zeros_like(t_mask)
    d_in = np.zeros_like(t_in)
    d_etch = np.zeros_like(t_mask)

    # ---- the emblem: the menu's rock ring around the crystal ball, a ring of carved beads
    ecx, ecy, er = EMBLEM
    re = np.hypot(XX - ecx, YY - ecy)
    e_edge = er + (fbm(90, 4, 41) - 0.5) * 18
    e_mask = re < e_edge
    e_in = e_edge - re
    he = 0.95 + bevel(e_in, 60, 0.30) + (fbm(160, 6, 42) - 0.5) * 0.10 + (grain - 0.5) * 0.03
    face_r = er * 0.80
    he += np.where(re < face_r, -0.10, 0) + np.exp(-((re - face_r) / 8.0) ** 2) * -0.07
    beads = Image.new("L", (W, H), 0)
    bd = ImageDraw.Draw(beads)
    for k in range(48):
        a = k * math.tau / 48
        bx, by, br = ecx + math.cos(a) * er * 0.895, ecy + math.sin(a) * er * 0.895, er * 0.034
        bd.ellipse([bx - br, by - br, bx + br, by + br], fill=255)
    bead_mask = np.array(beads) > 127
    he += raised(bead_mask, 0.07, 6.0, 0)
    ticks = Image.new("L", (W, H), 0)
    td = ImageDraw.Draw(ticks)
    for k in range(12):
        a = k * math.pi / 6
        long = k % 3 == 0
        r0, r1 = er * (0.66 if long else 0.69), er * 0.76
        td.line([(ecx + math.cos(a) * r0, ecy + math.sin(a) * r0), (ecx + math.cos(a) * r1, ecy + math.sin(a) * r1)], fill=255, width=20 if long else 13)
    tick_mask = (np.array(ticks) > 127)
    he += groove(tick_mask, 0.10, 7.0)
    sock = MARBLE_R + 26
    he += np.exp(-((re - sock) / 12.0) ** 2) * 0.06
    he += np.where(re < sock, -0.30 * np.sqrt(np.clip(1 - (re / sock) ** 2, 0, 1)) - 0.08, 0)
    he = np.where(e_mask, he, 0.0)
    h = np.where(e_mask, np.maximum(he, h), h)

    # the rays: carved from the ball's centre across the tablet's field, behind the letters
    rays = Image.new("L", (W, H), 0)
    rd = ImageDraw.Draw(rays)
    for k in [k for k in range(-9, 10) if k != 0]:
        a = math.pi / 2 + k * math.radians(8.5)
        rd.line([(ecx + math.cos(a) * (er + 30), ecy + math.sin(a) * (er + 30)), (ecx + math.cos(a) * 3800, ecy + math.sin(a) * 3800)], fill=255, width=7)
    ray_mask = (np.array(rays) > 127) & t_mask & (inner > 10) & ~ndimage.binary_dilation(letters | dot_letters, iterations=24) & ~e_mask & (rr > MOON_SOCKET + 30)
    h += groove(ray_mask, 0.035, 4.0)

    tether = Image.new("L", (W, H), 0)
    tdr = ImageDraw.Draw(tether)
    tdr.line([(ecx, ecy + er - 20), (ecx, mcy - MOON_SOCKET - 6)], fill=255, width=9)
    for yy_ in (ecy + er + 120, ecy + er + 330, mcy - MOON_SOCKET - 110):
        tdr.rectangle([ecx - 20, yy_ - 13, ecx + 20, yy_ + 13], fill=255)      # the climbers
    tether_mask = (np.array(tether) > 127) & t_mask & ~e_mask & ~letters
    h += groove(tether_mask, 0.06, 5.0)
    mask = t_mask | d_mask | e_mask
    # ---- the cave scenes, etched into the frame on purpose (owner: "purposefully etched into the
    # decor/border would be awesome"), red ochre rubbed into the cuts
    glyph_mask = (t_etch & t_mask & ~e_mask) | (d_etch & d_mask) | tether_mask
    paint = glyph_mask.astype(np.uint8)
    # cracks: a few thin random walks, cut shallow
    crack = Image.new("L", (W, H), 0)
    cd = ImageDraw.Draw(crack)
    for _ in range(14):
        x, y = rng.uniform(0, W), rng.uniform(0, H)
        a = rng.uniform(0, math.tau)
        pts = [(x, y)]
        for _ in range(int(rng.integers(8, 22))):
            a += rng.normal(0, 0.5)
            x += math.cos(a) * 14
            y += math.sin(a) * 14
            pts.append((x, y))
        cd.line(pts, fill=255, width=3)
    crack_mask = (np.array(crack) > 127) & mask & ~letters & ~dot_letters
    h += groove(crack_mask, 0.04, 2.0)
    # pitting
    h += np.where(mask, -np.clip(pits - 0.86, 0, None) * 0.3, 0)
    h = np.where(mask, h, 0.0)

    # what the materials need: carved (letters), glyphs, cavity (for dirt in the cuts), lichen
    cavity = np.clip((ndimage.gaussian_filter(h, 6) - h) * 8, 0, 1)
    # no dark ring at the letters' feet (owner: "why is there a dark edge around the text so much?"):
    # their cast shadow is the edge
    cavity *= ~ndimage.binary_dilation(letters | dot_letters, iterations=40)
    lichen = np.clip((fbm(60, 4, 51) - 0.70) * 5, 0, 1) * np.clip((rough - 0.45) * 4, 0, 1) * (~(letters | dot_letters))
    np.savez_compressed(os.path.join(out, "field.npz"), h=h.astype(np.float32), mask=mask,
                        letters=(letters | dot_letters), glyphs=glyph_mask, paint=paint, ticks=tick_mask & e_mask, cavity=cavity.astype(np.float32),
                        lichen=lichen.astype(np.float32))
    # a quick hillshade
    gy, gx = np.gradient(h * 220)
    shade = np.clip(0.55 + (-gx * 0.7 - gy * 0.7) / np.sqrt(1 + gx * gx + gy * gy) * 0.6, 0, 1)
    img = (shade * 255 * mask + 30 * ~mask).astype(np.uint8)
    Image.fromarray(img).resize((W // 2, H // 2), Image.LANCZOS).save(os.path.join(out, "preview.png"))
    import json
    PLACED.append({"name": "elevator", "x": EMBLEM[0], "y": EMBLEM[1] + EMBLEM[2] + 220})
    with open(os.path.join(out, "glyphs.json"), "w") as fh:
        json.dump(PLACED, fh)
    print("field written", h.min(), h.max())


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "logo-out")
