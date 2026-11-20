#!/usr/bin/env python3
"""The beaver, the river otter and the sea otter of M25's "Beavers and otters" tab (docs/mockups/ocean.html#critters,
openspec/changes/ocean-environments design B28, version 3): each one furred body sculpted as a signed distance
field, meshed, cut to a low-poly budget, skinned to the page's own rig, painted as pixel-art fur, with fur tufts.

The owner, 2026-11-20, on version 2's beaver mid-gnaw: "Some weird geometry poking out from middle of them. Also need
textures for both. Some fur tuffs would be nice too", then "See mining mike how we did that for the wolf". So this is
Mining Mike's Ice Wolf method (RubenTipparach/godot-sandbox at 1739e89: the `organic-creature` and `painted-bake`
skills; `ice_wolf_sculpt_model.py`, `ice_wolf_bake.py`, `gen_ice_wolf_sculpt.py`), adapted to Pale Blue Dot:

- **Bone, muscle, fur as one field.** Round cones along the rig's bones, ellipsoids for the muscle (haunches,
  shoulders, cheeks), the head's parts, blended with wide smooth unions; the fur a thin volume over the skin, thick at
  the nape, the cheeks and the back, broken into clumps along the way it grows. Never a raw primitive as a visible
  form: the legs, paws, ears, teeth and tail are grown out of the one skin, so nothing can poke out of the body or
  float beside it (version 2's boxes on pivots did both).
- **Meshed** with surface nets, pulled onto the field, decimated to a low-poly budget (`fast_simplification`), projected
  back onto the field.
- **Skinned** to the page's rig (critters.js `buildRig`: body, hind and chest segments, neck, jaw, four two-bone legs
  with feet, a three-part tail; the joints are critters.js `SPEC`'s, copied in `SPECS` below and checked by the page),
  four bones a vertex, gated by part (a leg's bones pull only their leg and the skin round its top), then smoothed.
- **Painted bake.** Charts flattened at one density (`PPM` texels a metre, square texels), every texel sent back to the
  sculpt's surface and painted from the animal's fur groups in Sweaft-64 tones (the page's palette): a darker back,
  a lighter belly, face and throat, fur strokes along the flow, a light from the upper left in a step or two, darker in
  the creases. The beaver's tail is bare, scaled and cross-hatched; the sea otter's head and throat are pale. No
  outline and no hard contrast: the page lights it like the ground. Nearest sampled, so it reads as pixel art.
- **Fur tufts** (`fur_locks`): a hundred or two low-poly tufts (a cupped row of three, a second row of three, then the
  point: six triangles), rooted under the skin, laid along the flow and drooping at the tip, scattered by area where
  the fur is long (nape, cheeks, back, the base of the tail), each with its root's colour and bone weights. Each
  carries a second, wet shape (narrow, clumped, spiked) the page blends to while the animal is wet.

Writes docs/mockups/ocean/critters_mesh.js (window.CRITTER_MESH) and docs/mockups/ocean/fur-<animal>.png, and the
aspen bark tile docs/mockups/ocean/tex-aspen.png.

    pip install numpy pillow fast_simplification
    python3 tools/art/critter_sculpt.py                # all three and the bark (a minute or two)
    python3 tools/art/critter_sculpt.py --sheet S      # also a contact sheet of the maps at S (keep it out of the repo)
"""
import base64
import json
import math
import os
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import sdf_kit as k  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(ROOT, "docs", "mockups", "ocean")
smin, smax, ellipsoid, round_cone, smoothstep, mirror, seg_dist = (k.smin, k.smax, k.ellipsoid, k.round_cone,
                                                                    k.smoothstep, k.mirror, k.seg_dist)
PPM = 150          # texels a metre: about 7 mm a texel, so the fur reads as pixels up close
SEED = 7


def hexc(h):
    h = h.lstrip("#")
    return np.array([int(h[i:i + 2], 16) for i in (0, 2, 4)], float)


# ---------------------------------------------------------------------------------------------------- the rigs
# critters.js SPEC's joints (the page checks the mesh's rest joints against its own rig and logs any difference).
# Body space: x forward, y up, z to the left side (legs[0] and legs[2] at +z); the body pivot at (0, H, 0).
SPECS = {
    "beaver": dict(H=0.17, hind=[(0, .155, .155), (-.12, .17, .175), (-.24, .135, .145), (-.32, .05, .06)],
                   chest=[(0, .155, .155), (.12, .135, .135), (.22, .1, .1), (.28, .075, .08)], neck=(.27, .035),
                   jaw=(.16, -.045, 0),
                   legs=[dict(at=(.16, -.08, .085), a=.07, b=.07, rt=.03, front=True, foot=(.05, .04)),
                         dict(at=(.16, -.08, -.085), a=.07, b=.07, rt=.03, front=True, foot=(.05, .04)),
                         dict(at=(-.2, -.07, .1), a=.09, b=.085, rt=.045, front=False, foot=(.11, .075)),
                         dict(at=(-.2, -.07, -.1), a=.09, b=.085, rt=.045, front=False, foot=(.11, .075))],
                   tailAt=(-.31, -.01), tail=[dict(L=.07, r=.05), dict(L=.06, r=.045), dict(L=.27, paddle=(.14, .026))],
                   tris=2400),
    "otter": dict(H=0.14, hind=[(0, .08, .085), (-.12, .085, .09), (-.24, .07, .075), (-.31, .04, .045)],
                  chest=[(0, .08, .085), (.12, .075, .075), (.2, .06, .062), (.25, .046, .05)], neck=(.245, .02),
                  jaw=(.12, -.03, 0),
                  legs=[dict(at=(.17, -.045, .055), a=.055, b=.05, rt=.022, front=True, foot=(.045, .035)),
                        dict(at=(.17, -.045, -.055), a=.055, b=.05, rt=.022, front=True, foot=(.045, .035)),
                        dict(at=(-.22, -.04, .062), a=.065, b=.06, rt=.028, front=False, foot=(.07, .045)),
                        dict(at=(-.22, -.04, -.062), a=.065, b=.06, rt=.028, front=False, foot=(.07, .045))],
                  tailAt=(-.3, 0.0), tail=[dict(L=.13, r=.042), dict(L=.12, r=.03), dict(L=.11, r=.018)], tris=2000),
    "sea": dict(H=0.0, hind=[(0, .12, .13), (-.15, .125, .135), (-.3, .095, .1), (-.38, .045, .05)],
                chest=[(0, .12, .13), (.14, .11, .115), (.24, .085, .09), (.3, .062, .066)], neck=(.29, 0.0),
                jaw=(.14, -.04, 0),
                legs=[dict(at=(.2, -.06, .085), a=.065, b=.06, rt=.026, front=True, foot=(.045, .04)),
                      dict(at=(.2, -.06, -.085), a=.065, b=.06, rt=.026, front=True, foot=(.045, .04)),
                      dict(at=(-.3, -.05, .08), a=.07, b=.06, rt=.032, front=False, foot=(.13, .085)),
                      dict(at=(-.3, -.05, -.08), a=.07, b=.06, rt=.032, front=False, foot=(.13, .085))],
                tailAt=(-.37, 0.0), tail=[dict(L=.09, paddle=(.075, .03)), dict(L=.09, paddle=(.07, .026)),
                                          dict(L=.09, paddle=(.055, .022))], tris=2400),
}


def joints(s):
    """Rest joints in mesh space (the rig's root at the ground, every rotation zero): name -> (point, parent)."""
    B0 = np.array([0.0, s["H"], 0.0])
    J = {"body": (B0, None), "hind": (B0, "body"), "chest": (B0, "body")}
    N = B0 + [s["neck"][0], s["neck"][1], 0]
    J["neck"] = (N, "chest")
    J["jaw"] = (N + np.array(s["jaw"]), "neck")
    for i, L in enumerate(s["legs"]):
        hip = B0 + np.array(L["at"])
        J["hip%d" % i] = (hip, "chest" if L["front"] else "hind")
        J["knee%d" % i] = (hip - [0, L["a"], 0], "hip%d" % i)
        J["foot%d" % i] = (hip - [0, L["a"] + L["b"], 0], "knee%d" % i)
    p = B0 + [s["tailAt"][0], s["tailAt"][1], 0]
    for i, T in enumerate(s["tail"]):
        J["tail%d" % i] = (p.copy(), "hind" if i == 0 else "tail%d" % (i - 1))
        p = p - [T["L"], 0, 0]
    return J


BONES = ["body", "hind", "chest", "neck", "jaw"] + ["%s%d" % (n, i) for i in range(4) for n in ("hip", "knee", "foot")] \
    + ["tail0", "tail1", "tail2"]
GROUPS = ("body", "head", "leg0", "leg1", "leg2", "leg3", "tail")


# ---------------------------------------------------------------------------------------------------- shapes
def xloft(P, st, pe=2.2):
    """A loft along x: each x-slice a superellipse (half height ry, half width rz) about (cy), between stations
    (x, ry, rz, cy). Mining Mike's zloft turned to run along x."""
    st = np.asarray(sorted(st), float)
    x = P[:, 0]
    xs = st[:, 0]
    ry = np.interp(x, xs, st[:, 1])
    rz = np.interp(x, xs, st[:, 2])
    cy = np.interp(x, xs, st[:, 3])
    v = (P[:, 1] - cy) / ry
    u = np.abs(P[:, 2]) / rz
    n = (u ** pe + np.abs(v) ** pe) ** (1.0 / pe)
    d2 = (n - 1.0) * np.minimum(ry, rz)
    dx = np.maximum(xs[0] - x, x - xs[-1])
    return np.sqrt(np.maximum(d2, 0) ** 2 + np.maximum(dx, 0) ** 2) + np.minimum(np.maximum(d2, dx), 0)


def ell_m(P, c, r):
    """An ellipsoid on both sides (mirrored in z)."""
    M = np.stack([P[:, 0], P[:, 1], np.abs(P[:, 2])], 1)
    return ellipsoid(M, c, r)


def rc_m(P, a, b, r1, r2):
    M = np.stack([P[:, 0], P[:, 1], np.abs(P[:, 2])], 1)
    return round_cone(M, a, b, r1, r2)


class Critter:
    def __init__(self, name):
        self.name = name
        s = self.s = SPECS[name]
        self.J = joints(s)
        self.B0 = self.J["body"][0]
        self.N = self.J["neck"][0]
        lo = np.array([-0.8, -0.3, -0.3])
        hi = np.array([0.6, 0.45, 0.3])
        self.LO, self.HI = lo + [0, s["H"], 0], hi + [0, s["H"], 0]
        self.CELL = 0.0045 if name != "otter" else 0.0036

    # ---- the parts, each its own field (mesh space)
    def torso(self, P):
        s = self.s
        st = [(x, ry, rz, 0.0) for x, ry, rz in s["hind"]] + [(x, ry, rz, 0.0) for x, ry, rz in s["chest"][1:]]
        Q = P - self.B0
        d = xloft(Q, st)
        # Round the rump and the chest's front (the loft's ends are flat).
        x0, ry0, rz0 = s["hind"][-2]
        d = smin(d, ellipsoid(Q, (x0 - 0.02, 0, 0), (ry0 * 0.9, ry0 * 0.92, rz0 * 0.92)), 0.03)
        # The belly hangs a little (more on the beaver), the back stays straight.
        r0 = s["hind"][1][1]
        d = smin(d, ellipsoid(Q, (-0.06, -0.18 * r0, 0), (r0 * 1.15, r0 * 0.82, s["hind"][1][2] * 0.95)), 0.04)
        # The neck: a column from the chest up into the back of the skull, so a turned head never opens a gap.
        n = np.array([s["neck"][0], s["neck"][1], 0.0])
        rr = s["chest"][2][1]
        d = smin(d, round_cone(Q, (s["chest"][2][0] - 0.03, 0, 0), n + [0.01, 0.005, 0], rr * 0.95, rr * 0.75), 0.03)
        return d

    def head(self, P):
        Q = P - self.N
        n = self.name
        if n == "beaver":
            d = ellipsoid(Q, (0.025, 0.012, 0), (0.078, 0.068, 0.074))
            d = smin(d, ellipsoid(Q, (0.105, -0.006, 0), (0.07, 0.05, 0.054)), 0.03)
            d = smin(d, ell_m(Q, (0.065, -0.022, 0.042), (0.045, 0.04, 0.03)), 0.02)
            d = smin(d, ellipsoid(Q, (0.161, 0.014, 0), (0.019, 0.016, 0.024)), 0.012)
            d = smin(d, rc_m(Q, (0.152, -0.012, 0.0075), (0.157, -0.05, 0.0075), 0.0075, 0.0062), 0.006)
            d = smin(d, ell_m(Q, (0.0, 0.058, 0.056), (0.017, 0.02, 0.011)), 0.012)
            d = smin(d, ell_m(Q, (0.072, 0.04, 0.054), (0.014, 0.012, 0.008)), 0.008)
        elif n == "otter":
            d = ellipsoid(Q, (0.025, 0.006, 0), (0.06, 0.045, 0.06))
            d = smin(d, ellipsoid(Q, (0.088, -0.006, 0), (0.045, 0.031, 0.04)), 0.022)
            d = smin(d, ell_m(Q, (0.098, -0.012, 0.022), (0.027, 0.021, 0.02)), 0.012)
            d = smin(d, ellipsoid(Q, (0.124, 0.006, 0), (0.012, 0.011, 0.018)), 0.008)
            d = smin(d, ell_m(Q, (-0.008, 0.036, 0.044), (0.01, 0.013, 0.008)), 0.008)
            d = smin(d, ell_m(Q, (0.064, 0.026, 0.04), (0.011, 0.009, 0.006)), 0.006)
        else:
            d = ellipsoid(Q, (0.03, 0.01, 0), (0.08, 0.072, 0.08))
            d = smin(d, ellipsoid(Q, (0.108, -0.01, 0), (0.05, 0.045, 0.055)), 0.03)
            d = smin(d, ell_m(Q, (0.124, -0.016, 0.026), (0.03, 0.026, 0.026)), 0.014)
            d = smin(d, ellipsoid(Q, (0.152, 0.008, 0), (0.014, 0.013, 0.02)), 0.009)
            d = smin(d, ell_m(Q, (-0.002, 0.06, 0.064), (0.011, 0.013, 0.008)), 0.009)
            d = smin(d, ell_m(Q, (0.082, 0.044, 0.058), (0.013, 0.011, 0.007)), 0.007)
        return d

    def leg(self, P, i):
        L = self.s["legs"][i]
        hip, knee, foot = (self.J[n % i][0] for n in ("hip%d", "knee%d", "foot%d"))
        rt, f0, f1 = L["rt"], L["foot"][0], L["foot"][1]
        d = round_cone(P, hip, knee, rt * 1.25, rt * 0.95)
        d = smin(d, round_cone(P, knee, foot + [0, 0.006, 0], rt * 0.85, rt * 0.62), 0.012)
        # Muscle: the haunch or the shoulder over the hip, the forearm or the shank's swell.
        if L["front"]:
            d = smin(d, ellipsoid(P, hip - [0.0, L["a"] * 0.3, 0], (rt * 1.35, L["a"] * 0.7, rt * 1.15)), 0.02)
        else:
            d = smin(d, ellipsoid(P, hip + [0.005, -L["a"] * 0.15, 0], (L["a"] * 0.62, L["a"] * 0.78, rt * 1.35)), 0.03)
        d = smin(d, ellipsoid(P, (knee + foot) / 2 + [0.004 * (1 if L["front"] else -1), 0, 0],
                              (rt * 0.8, L["b"] * 0.48, rt * 0.75)), 0.012)
        # The paw or the webbed foot: a pad along the foot bone (+x), toes or webbed digits at its front.
        sz = np.sign(L["at"][2])
        pad_c = foot + [f0 * 0.3, -0.006, 0]
        th = 0.008 if L["front"] else 0.009
        d = smin(d, ellipsoid(P, pad_c, (f0 * 0.55, th + 0.004, f1 * 0.45)), 0.012)
        nt = 4 if L["front"] else 5
        for j in range(nt):
            a = (j / (nt - 1) - 0.5) * (1.2 if L["front"] else 1.5)
            tip = foot + [f0 * 0.3 + math.cos(a) * f0 * 0.6, -0.007, math.sin(a) * f1 * 0.55]
            d = smin(d, round_cone(P, pad_c + [f0 * 0.15, 0, 0], tip, th * 0.9, th * 0.55),
                     0.004 if L["front"] else 0.012)
        del sz
        return d

    def tail(self, P):
        s = self.s
        d = None
        r_prev = None
        for i, T in enumerate(s["tail"]):
            a = self.J["tail%d" % i][0]
            b = a - [T["L"], 0, 0]
            if "paddle" in T:
                w, th = T["paddle"]
                if self.name == "beaver":
                    # The beaver's paddle: a broad flat oval, thicker along its middle, its root narrow.
                    c = (a + b) / 2 - [0.01, 0, 0]
                    Q = P - c
                    u = np.abs(Q[:, 0]) / (T["L"] * 0.52)
                    v = np.abs(Q[:, 2]) / (w * 0.5)
                    rr = (u ** 2.4 + v ** 2.4) ** (1 / 2.4)
                    flat = (rr - 1.0) * min(T["L"] * 0.5, w * 0.5)
                    half = th * 0.5 * (1.1 - 0.5 * np.clip(rr, 0, 1) ** 2)
                    seg = np.maximum(flat, np.abs(Q[:, 1]) - half)
                else:
                    seg = ellipsoid(P, (a + b) / 2, (T["L"] * 0.62, th * 0.5, w * 0.5))
                    seg = smin(seg, round_cone(P, a, b, th * 0.5, th * 0.45), 0.01)
            else:
                r1 = T["r"]
                r2 = s["tail"][i + 1]["r"] if i + 1 < len(s["tail"]) and "r" in s["tail"][i + 1] else r1 * 0.35
                seg = round_cone(P, a, b, r1, r2)
            d = seg if d is None else smin(d, seg, 0.02 if self.name != "beaver" or i < 2 else 0.035)
            r_prev = T.get("r", r_prev)
        return d

    def groups(self, P):
        G = {"body": self.torso(P), "head": self.head(P), "tail": self.tail(P)}
        for i in range(4):
            G["leg%d" % i] = self.leg(P, i)
        return G

    def skin(self, G):
        d = smin(G["body"], G["head"], 0.035)
        d = smin(d, np.minimum(G["leg0"], G["leg1"]), 0.03)
        d = smin(d, np.minimum(G["leg2"], G["leg3"]), 0.04)
        d = smin(d, G["tail"], 0.03 if self.name != "beaver" else 0.02)
        return d

    # ---- the fur: how long it is, the way it lies, its volume over the skin
    def fur_len(self, P):
        """How long the fur stands at P (m): the nape, the cheeks, the back and the base of the tail long; the face's
        front, the legs and the feet short; the beaver's paddle bare."""
        Q = P - self.B0
        H = P - self.N
        n = self.name
        base = {"beaver": 0.014, "otter": 0.009, "sea": 0.014}[n]
        L = np.full(len(P), base)
        top = smoothstep(0.2, 0.75, self._up(P))
        nape = np.exp(-(((H[:, 0] + 0.03) / 0.07) ** 2) - ((H[:, 1] - 0.05) / 0.06) ** 2 - (H[:, 2] / 0.07) ** 2)
        cheek = np.exp(-(((H[:, 0] - 0.04) / 0.04) ** 2) - ((H[:, 1] + 0.02) / 0.035) ** 2
                       - ((np.abs(H[:, 2]) - 0.06 * self._hs()) / 0.03) ** 2)
        back = top * np.exp(-((Q[:, 0] + 0.06) / 0.2) ** 2)
        tb = self.J["tail0"][0]
        tailbase = np.exp(-((P[:, 0] - tb[0] - 0.01) / 0.06) ** 2 - ((P[:, 1] - tb[1] - 0.02) / 0.06) ** 2)
        L = L + {"beaver": 0.03, "otter": 0.018, "sea": 0.028}[n] * nape + 0.02 * cheek * (0.7 if n == "otter" else 1) \
            + {"beaver": 0.016, "otter": 0.008, "sea": 0.012}[n] * back + 0.014 * tailbase * (1 if n != "sea" else 0.6)
        face = smoothstep(0.05, 0.1, H[:, 0]) * smoothstep(0.2, 0.0, np.linalg.norm(H, axis=1) - 0.12)
        L = L * (1 - 0.7 * face)
        legs = smoothstep(self.B0[1] - 0.06, self.B0[1] - 0.15, P[:, 1]) * (1 - smoothstep(-0.02, 0.06, Q[:, 0] - self.s["tailAt"][0]) * 0)
        L = L * (1 - 0.75 * legs)
        if n == "beaver":
            paddle = smoothstep(self.J["tail1"][0][0] + 0.01, self.J["tail1"][0][0] - 0.03, P[:, 0])
            L = L * (1 - paddle)
        return L

    def _hs(self):
        return {"beaver": 1.0, "otter": 0.72, "sea": 1.1}[self.name]

    def _up(self, P):
        """0 under the body, 1 along the spine (by height in the torso's section)."""
        Q = P - self.B0
        st = [(x, ry) for x, ry, rz in self.s["hind"]] + [(x, ry) for x, ry, rz in self.s["chest"][1:]]
        st = np.asarray(sorted(st))
        ry = np.interp(Q[:, 0], st[:, 0], st[:, 1])
        return np.clip(Q[:, 1] / np.maximum(ry, 1e-3), -1, 1)

    def flow(self, P):
        """The way the fur lies: back along the body and off the face, down the sides and the legs, along the tail."""
        Q = P - self.B0
        back = np.tile([-1.0, 0.0, 0.0], (len(P), 1))
        down = np.tile([0.0, -1.0, 0.0], (len(P), 1))
        side = smoothstep(0.2, 0.9, np.abs(Q[:, 2]) / np.maximum(np.abs(Q[:, 1]) + np.abs(Q[:, 2]), 1e-6))[:, None] * 0.45
        f = back * (1 - side) + down * side
        leg = smoothstep(self.B0[1] - 0.05, self.B0[1] - 0.13, P[:, 1])[:, None] * 0.9
        f = f * (1 - leg) + down * leg
        f /= np.maximum(np.linalg.norm(f, axis=1), 1e-9)[:, None]
        return f

    def strands(self, P, F, across=90.0, along=22.0, seed=11):
        a = (P * F).sum(1)[:, None]
        Q = (P - F * a) * across + F * a * along
        return 0.6 * k.vnoise(Q, 1.0, seed) + 0.4 * k.vnoise(Q, 2.1, seed + 1)

    def sdf(self, P):
        G = self.groups(P)
        d = self.skin(G)
        L = self.fur_len(P)
        F = self.flow(P)
        s = self.strands(P, F)
        th = 0.12 * L + 0.0015
        clump = 0.55 + 0.6 * s
        return d - th * clump

    def chunked(self, fn=None):
        fn = fn or self.sdf
        return lambda P: np.concatenate([fn(P[s:s + 200000]) for s in range(0, len(P), 200000)]) if len(P) else np.zeros(0)


# ---------------------------------------------------------------------------------------------------- meshing
def field_grid(c, fn, band):
    """The field on the fine grid: evaluated on a grid four times coarser, interpolated, and evaluated exactly in a
    band round the surface (Mining Mike's ice_wolf_sculpt_model.field_grid)."""
    n = np.ceil((c.HI - c.LO) / c.CELL).astype(int) + 1
    fx, fy, fz = (c.LO[i] + c.CELL * np.arange(n[i]) for i in range(3))
    cc = c.CELL * 4
    nc = np.ceil((c.HI - c.LO) / cc).astype(int) + 2
    gx, gy, gz = (c.LO[i] + cc * np.arange(nc[i]) for i in range(3))
    C = np.stack(np.meshgrid(gx, gy, gz, indexing="ij"), -1).reshape(-1, 3)
    Fc = fn(C).reshape(nc)
    F = np.empty(n)
    uy = (fy - c.LO[1]) / cc
    iy = np.minimum(uy.astype(int), nc[1] - 2)
    ty = uy - iy
    uz = (fz - c.LO[2]) / cc
    iz = np.minimum(uz.astype(int), nc[2] - 2)
    tz = uz - iz
    for i in range(n[0]):
        ux = (fx[i] - c.LO[0]) / cc
        ix = min(int(ux), nc[0] - 2)
        tx = ux - ix
        a = Fc[ix][:, iz] * (1 - tz) + Fc[ix][:, iz + 1] * tz
        b = Fc[ix + 1][:, iz] * (1 - tz) + Fc[ix + 1][:, iz + 1] * tz
        s = a * (1 - tx) + b * tx
        F[i] = s[iy] * (1 - ty)[:, None] + s[iy + 1] * ty[:, None]
    idx = np.nonzero(np.abs(F) < band)
    P = np.stack([fx[idx[0]], fy[idx[1]], fz[idx[2]]], 1)
    F[idx] = fn(P)
    return F


def mesh_field(c, F, fn):
    V, Q = k.surface_nets(F)
    P = c.LO + V * c.CELL
    for _ in range(3):
        d = fn(P)
        g = k.gradient(P, fn, 0.001)
        step = d[:, None] * g / np.maximum((g * g).sum(1), 1e-6)[:, None]
        nn = np.linalg.norm(step, axis=1)
        step *= np.minimum(1.0, 0.6 * c.CELL / np.maximum(nn, 1e-9))[:, None]
        P = P - step
    a, b, cc, d = Q.T
    short = np.linalg.norm(P[a] - P[cc], axis=1) <= np.linalg.norm(P[b] - P[d], axis=1)
    T = np.concatenate([np.where(short[:, None], np.stack([a, b, cc], 1), np.stack([a, b, d], 1)),
                        np.where(short[:, None], np.stack([a, cc, d], 1), np.stack([b, cc, d], 1))])
    N = k.gradient(P, fn, 0.001)
    fnrm = np.cross(P[T[:, 1]] - P[T[:, 0]], P[T[:, 2]] - P[T[:, 0]])
    flip = (fnrm * N[T].sum(1)).sum(1) < 0
    T[flip] = T[flip][:, ::-1]
    return P, T[np.linalg.norm(fnrm, axis=1) > 1e-12]


def keep_largest(P, T):
    E = np.concatenate([T[:, [0, 1]], T[:, [1, 2]]])
    lab = k.components(len(P), E)
    keep = lab == np.bincount(lab).argmax()
    remap = -np.ones(len(P), int)
    remap[keep] = np.arange(keep.sum())
    T2 = remap[T]
    return P[keep], T2[(T2 >= 0).all(1)]


def project(P, fn, iters=3):
    for _ in range(iters):
        d = fn(P)
        g = k.gradient(P, fn, 0.001)
        P = P - d[:, None] * g / np.maximum((g * g).sum(1), 1e-6)[:, None]
    N = k.gradient(P, fn, 0.001)
    return P, N / np.maximum(np.linalg.norm(N, axis=1), 1e-9)[:, None]


def build_mesh(c):
    fn = c.chunked()
    F = field_grid(c, fn, c.CELL * 3)
    P, T = mesh_field(c, F, fn)
    P, T = keep_largest(P, T)
    hi = len(T)
    import fast_simplification as fs
    P2, T2 = fs.simplify(P.astype(np.float32), T.astype(np.int32), target_count=c.s["tris"])
    P2, T2 = P2.astype(np.float64), T2.astype(np.int64)
    P2, N2 = project(P2, fn, 2)
    return P2, N2, T2, hi


# ---------------------------------------------------------------------------------------------------- skinning
def skin_weights(c, P, T):
    """Four bones a vertex (Mining Mike's skin_weights): each bone pulls the skin of its own part, by the inverse
    fourth power of the distance to its segment; a leg's bones only their leg and the skin round its top, the tail's
    only the tail, the hind and chest segments the body between them; then smoothed over the mesh."""
    G = c.groups(P)
    stack = np.stack([G[g] for g in GROUPS])
    dmin = stack.min(0)
    nearest = np.array(GROUPS)[stack.argmin(0)]
    aff = {g: np.exp(-np.maximum(G[g] - dmin, 0) / 0.012) for g in GROUPS}
    J = c.J
    seg = {"hind": (J["body"][0], J["body"][0] + [-0.3, 0, 0]), "chest": (J["body"][0], J["neck"][0]),
           "neck": (J["neck"][0], J["neck"][0] + [0.14, 0, 0])}
    grp = {"hind": "body", "chest": "body", "neck": "head"}
    for i, L in enumerate(c.s["legs"]):
        seg["hip%d" % i] = (J["hip%d" % i][0], J["knee%d" % i][0])
        seg["knee%d" % i] = (J["knee%d" % i][0], J["foot%d" % i][0])
        seg["foot%d" % i] = (J["foot%d" % i][0], J["foot%d" % i][0] + [L["foot"][0], 0, 0])
        for n in ("hip", "knee", "foot"):
            grp["%s%d" % (n, i)] = "leg%d" % i
    for i, Tl in enumerate(c.s["tail"]):
        seg["tail%d" % i] = (J["tail%d" % i][0], J["tail%d" % i][0] - [Tl["L"], 0, 0])
        grp["tail%d" % i] = "tail"
    W = np.zeros((len(P), len(BONES)))
    for bi, name in enumerate(BONES):
        if name not in seg:
            continue
        g = grp[name]
        a = aff[g].copy()
        if g.startswith("leg"):
            i = int(g[3])
            hip = J["hip%d" % i][0]
            a = a * ((nearest == g) | (np.linalg.norm(P - hip, axis=1) < c.s["legs"][i]["a"] * 0.8))
        elif g == "tail":
            a = a * ((nearest == "tail") | (P[:, 0] < J["tail0"][0][0] - 0.02))
        eps = 0.01 if g != "body" else 0.03
        W[:, bi] = a / (seg_dist(P, *seg[name]) + eps) ** 4
    W /= np.maximum(W.sum(1), 1e-30)[:, None]
    E = np.concatenate([T[:, [0, 1]], T[:, [1, 2]], T[:, [2, 0]]])
    deg = np.bincount(E[:, 0], minlength=len(P)) + np.bincount(E[:, 1], minlength=len(P))
    for _ in range(4):
        acc = np.zeros_like(W)
        np.add.at(acc, E[:, 0], W[E[:, 1]])
        np.add.at(acc, E[:, 1], W[E[:, 0]])
        W = 0.5 * W + 0.5 * acc / np.maximum(deg, 1)[:, None]
    top = np.argsort(-W, axis=1)[:, :4]
    tw = np.take_along_axis(W, top, 1)
    tw /= np.maximum(tw.sum(1), 1e-30)[:, None]
    return top.astype(np.uint8), tw, nearest


def u8w(W):
    q = np.clip(np.round(W * 255), 0, 255).astype(np.int32)
    q[:, 0] += 255 - q.sum(1)
    return q.astype(np.uint8)


# ---------------------------------------------------------------------------------------------------- unwrap and bake
def unwrap(P, T, part, ppm, pad=2):
    """Mining Mike's tyrant_bake.unwrap at a fixed density: charts grown within 35 degrees of their seed (so a chart
    stretches at most 1.22 times), each flattened along its normal, turned to its principal axes and packed at `ppm`
    texels a metre into the smallest square atlas that holds them: every texel the same size, square."""
    fn = np.cross(P[T[:, 1]] - P[T[:, 0]], P[T[:, 2]] - P[T[:, 0]])
    area = np.linalg.norm(fn, axis=1)
    fn /= np.maximum(area, 1e-12)[:, None]
    chart, axes = k.grow_charts(fn, area, part[T[:, 0]], k.face_pairs(T), cone=35.0, merge=60.0, min_faces=8)
    nc = chart.max() + 1
    uv = np.zeros((len(T), 3, 2))
    order = np.argsort(chart, kind="stable")
    bounds = np.searchsorted(chart[order], np.arange(nc + 1))
    boxes = []
    for ci in range(nc):
        fs = order[bounds[ci]:bounds[ci + 1]]
        d = axes[ci]
        up = np.array([0, 1.0, 0]) if abs(d[1]) < 0.9 else np.array([0, 0, 1.0])
        e1 = np.cross(up, d)
        e1 /= np.linalg.norm(e1)
        e2 = np.cross(d, e1)
        pts = P[T[fs]]
        q = np.stack([pts @ e1, pts @ e2], -1)
        flat = q.reshape(-1, 2)
        mu = flat.mean(0)
        if len(flat) > 3:
            w, V = np.linalg.eigh(np.cov((flat - mu).T))
            R = V[:, ::-1]
            if np.linalg.det(R) < 0:
                R[:, 1] *= -1
            q = (q - mu) @ R
        q -= q.reshape(-1, 2).min(0)
        uv[fs] = q
        boxes.append(q.reshape(-1, 2).max(0))
    boxes = np.array(boxes)
    for size in (64, 128, 256, 512):
        pos = k.skyline_pack(np.ceil(boxes * ppm) + 2 * pad, size)
        if pos is not None:
            break
    tx = uv * ppm + (pos[chart] + pad)[:, None, :]
    return np.stack([tx[..., 0] / size, 1.0 - tx[..., 1] / size], -1), chart, size


# Ramps dark to light, every tone Sweaft-64 (the page's palette, balatro-pixel-art's pixelkit.SWEAFT_64).
RAMPS = {
    "beaver": {"coat": ["#2f1525", "#522b2f", "#7d4636", "#a8633e", "#ab8465"],
               "foot": ["#2f1525", "#41322c", "#522b2f"], "tail": ["#170916", "#2b1f35", "#3e3551", "#69608b"],
               "ink": ["#161612"], "tooth": ["#c56d25", "#fb902f"]},
    "otter": {"coat": ["#2e0d06", "#41322c", "#725244", "#ab8465", "#cfaa81"], "foot": ["#2e0d06", "#41322c", "#725244"],
              "ink": ["#161612"]},
    "sea": {"coat": ["#2f1525", "#522b2f", "#725244", "#ab8465"], "pale": ["#725244", "#ab8465", "#cfaa81", "#eab798"],
            "foot": ["#2f1525", "#41322c", "#522b2f"], "ink": ["#161612"]},
}


def paint(c, P, N, ao):
    """Colour per texel at sculpt points P (normals N): the ramp each region uses and where in it, then rounded to a
    tone, so every texel is one palette colour."""
    n = c.name
    R = RAMPS[n]
    H = P - c.N
    Q = P - c.B0
    G = c.groups(P)
    stack = np.stack([G[g] for g in GROUPS])
    part = np.array(GROUPS)[stack.argmin(0)]
    F = c.flow(P)
    s = c.strands(P, F, across=105.0, along=13.0, seed=31)
    s2 = c.strands(P, F, across=60.0, along=14.0, seed=41)
    light = np.clip(N @ (np.array([-0.35, 0.85, 0.4]) / np.linalg.norm([-0.35, 0.85, 0.4])), -1, 1)
    up = c._up(P)
    # The coat: a darker back, lighter flanks, the belly lightest; strokes along the fur (fine strands over broader
    # clumps); a step of painted light; darker in the creases. The legs keep the flanks' tone, not the belly's.
    body_ish = (part == "body") | (part == "head")
    belly = smoothstep(-0.1, -0.65, up) * body_ish
    legd = np.zeros(len(P))
    for i in range(4):
        legd = np.maximum(legd, (part == "leg%d" % i) * smoothstep(c.B0[1] - 0.03, c.B0[1] - 0.09, P[:, 1]))
    base = {"beaver": 1.75, "otter": 1.8, "sea": 1.8}[n]
    t = base + 1.2 * belly - 0.5 * smoothstep(0.45, 0.95, up) - 0.4 * legd + 0.35 * light - 1.3 * (1 - ao)
    t = t + 0.5 * (s2 - 0.5)
    # Sparse strokes, long along the fur and a texel across: a dark parting here and there, a lit tip on the top side.
    t = t - 1.0 * (s > 0.66) + 1.0 * ((s < 0.24) & (light > 0.3))
    # The face: a lighter muzzle and cheeks (the beaver's face stays its coat, a little lighter).
    muzzle = (part == "head") * smoothstep(0.06, 0.11, H[:, 0])
    throat = smoothstep(0.02, -0.04, H[:, 1]) * smoothstep(-0.12, -0.02, H[:, 0]) * smoothstep(0.02, 0.0, up) * \
        smoothstep(0.0, 0.05, Q[:, 0] - c.s["chest"][1][0])
    if n == "beaver":
        t = t + 0.6 * muzzle
    elif n == "otter":
        t = t + 1.6 * muzzle * smoothstep(0.03, -0.02, H[:, 1] - 0.005) + 1.7 * throat
        t = t + 1.4 * (part == "head") * smoothstep(-0.005, -0.025, H[:, 1]) * smoothstep(0.0, 0.05, H[:, 0])
    idx = np.clip(np.round(t), 0, len(R["coat"]) - 1).astype(int)
    col = np.array([hexc(h) for h in R["coat"]])[idx]
    if n == "sea":
        # Pale head and throat, fading down the neck.
        edge = c.s["chest"][2][0] - 0.03 + 0.05 * (s2 - 0.5) + 0.012 * (s - 0.5)
        pale = (part == "head") | ((Q[:, 0] > edge) & (part == "body"))
        pale = pale | ((throat > 0.3) & (Q[:, 0] > c.s["chest"][1][0] + 0.06 * (s2 - 0.5)))
        tp = 1.6 + 0.9 * (s - 0.5) + 0.45 * light - 1.0 * (1 - ao) + 0.5 * muzzle
        ip = np.clip(np.round(tp), 0, 3).astype(int)
        col = np.where(pale[:, None], np.array([hexc(h) for h in R["pale"]])[ip], col)
    # Hands and feet: the dark, bare-ish skin of the paws, the legs' fur darkening toward them.
    footy = np.zeros(len(P))
    for i in range(4):
        fj = c.J["foot%d" % i][0]
        footy = np.maximum(footy, (part == "leg%d" % i) * smoothstep(fj[1] + 0.035, fj[1] + 0.01, P[:, 1]))
    tf = 1.0 + 0.6 * (s - 0.5) + 0.6 * light - (1 - ao)
    fi = np.clip(np.round(tf), 0, len(R["foot"]) - 1).astype(int)
    col = np.where((footy > 0.5)[:, None], np.array([hexc(h) for h in R["foot"]])[fi], col)
    if n == "beaver":
        # The paddle: bare, leathery, scaled in a cross-hatch, a little lighter along its middle.
        j1 = c.J["tail1"][0][0]
        tail = (part == "tail") & (P[:, 0] < j1 - 0.035)
        u, w = P[:, 0] * 75.0, P[:, 2] * 75.0
        hatch = (np.minimum(np.abs(((u + w) % 1.0) - 0.5), np.abs(((u - w) % 1.0) - 0.5)) > 0.36)
        tt = 2.0 - 1.0 * hatch + 0.5 * light - 0.8 * (1 - ao) + 0.3 * (k.vnoise(P * 60, 1.0, 5) - 0.5)
        ti = np.clip(np.round(tt), 0, 2).astype(int)
        col = np.where(tail[:, None], np.array([hexc(h) for h in R["tail"]])[ti], col)
        # The orange incisors.
        Mh = np.stack([H[:, 0], H[:, 1], np.abs(H[:, 2])], 1)
        tooth = (seg_dist(Mh, (0.152, -0.016, 0.0075), (0.157, -0.06, 0.0075)) < 0.0095) & (H[:, 0] > 0.15)
        tcol = np.array([hexc(h) for h in R["tooth"]])[(light > 0.2).astype(int)]
        col = np.where(tooth[:, None], tcol, col)
    # Eyes and nose: the ink tone.
    eye_c = {"beaver": (0.072, 0.04, 0.054), "otter": (0.064, 0.026, 0.04), "sea": (0.082, 0.044, 0.058)}[n]
    eye_r = {"beaver": 0.0125, "otter": 0.0105, "sea": 0.013}[n]
    Mh = np.stack([H[:, 0], H[:, 1], np.abs(H[:, 2])], 1)
    eye = (np.linalg.norm((Mh - eye_c) * [1.0, 1.25, 0.8], axis=1) < eye_r) & (part == "head")
    nose_c = {"beaver": (0.165, 0.016, 0), "otter": (0.128, 0.006, 0), "sea": (0.156, 0.008, 0)}[n]
    nose = (np.linalg.norm((H - nose_c) * [0.8, 1.0, 1.0], axis=1) < {"beaver": 0.016, "otter": 0.012, "sea": 0.014}[n]) \
        & (H[:, 0] > nose_c[0] - 0.01)
    col = np.where((eye | nose)[:, None], hexc(R["ink"][0]), col)
    return col


def vertex_ao(fn, P, N):
    occ = np.zeros(len(P))
    for h, wt in ((0.006, 0.25), (0.014, 0.3), (0.03, 0.25), (0.06, 0.2)):
        occ += wt * np.clip((h - fn(P + N * h)) / h, 0, 1)
    return np.clip(1.0 - 1.3 * occ, 0, 1)


def bake(c, P, N, T, nearest):
    fn = c.chunked()
    gi = np.array([GROUPS.index(g) for g in nearest])
    corner_uv, chart, size = unwrap(P, T, gi, PPM)
    src, T2, uv = k.split_vertices(T, chart, corner_uv)
    ys, xs, ts, bc = k.raster(T2, uv, size)
    tri = T2[ts]
    Pt = (P[src][tri] * bc[..., None]).sum(1)
    Nt = (N[src][tri] * bc[..., None]).sum(1)
    Nt /= np.maximum(np.linalg.norm(Nt, axis=1), 1e-9)[:, None]
    Pp, Np = project(Pt, fn, 2)
    far = np.linalg.norm(Pp - Pt, axis=1) > 0.012
    Pp[far], Np[far] = Pt[far], Nt[far]
    ao = vertex_ao(fn, Pp, Np)
    col = paint(c, Pp, Np, ao)
    filled = np.zeros((size, size), bool)
    filled[ys, xs] = True
    A = np.zeros((size, size, 3))
    A[ys, xs] = col
    # Gutters take the nearest texel's colour unchanged (no blend: every texel stays a palette tone).
    for _ in range(6):
        grow = np.zeros_like(filled)
        for dy, dx in ((-1, 0), (1, 0), (0, -1), (0, 1)):
            f = np.roll(np.roll(filled, dy, 0), dx, 1) & ~filled & ~grow
            A[f] = np.roll(np.roll(A, dy, 0), dx, 1)[f]
            grow |= f
        filled |= grow
    A[~filled] = A[filled].mean(0).round()
    return {"src": src, "T": T2, "uv": uv, "img": A.astype(np.uint8), "size": size,
            "coverage": float((np.zeros((size, size), bool) | filled).mean())}


# ---------------------------------------------------------------------------------------------------- fur tufts
def fur_locks(c, P, N, T, uv, bi, bw, count):
    """Mining Mike's fur_locks (gen_ice_wolf_sculpt.py), scaled to these animals: chunky low-poly tufts where the fur
    is long, each a cupped row of three, a second row of three and the point (six triangles), rooted a little under
    the skin, laid along the fur's flow and drooping toward the body at the tip, with its root's colour and bones.
    Each has a dry shape and a wet one (narrower, clumped to a spike, standing a little more out of the coat)."""
    rng = np.random.default_rng(SEED + len(c.name))
    L = c.fur_len(P)
    wgt = smoothstep(0.016, 0.03, L) if c.name != "otter" else smoothstep(0.011, 0.02, L)
    a = np.linalg.norm(np.cross(P[T[:, 1]] - P[T[:, 0]], P[T[:, 2]] - P[T[:, 0]]), axis=1) * 0.5
    pr = a * wgt[T].mean(1)
    pr /= pr.sum()
    tri = rng.choice(len(T), size=count, p=pr)
    r1, r2 = rng.random(count), rng.random(count)
    sq = np.sqrt(r1)
    bc = np.stack([1 - sq, sq * (1 - r2), sq * r2], 1)
    V = T[tri]
    root = (P[V] * bc[..., None]).sum(1)
    n = (N[V] * bc[..., None]).sum(1)
    n /= np.linalg.norm(n, axis=1)[:, None]
    cb = c.flow(root)
    cb -= n * (cb * n).sum(1)[:, None]
    cb /= np.maximum(np.linalg.norm(cb, axis=1), 1e-6)[:, None]
    ruv = (uv[V] * bc[..., None]).sum(1)
    Ls = c.fur_len(root)
    near = V[np.arange(count), np.argmax(bc, 1)]
    length = 1.5 * Ls * rng.uniform(0.8, 1.2, count)
    width = np.clip(0.38 * length, 0.005, 0.016) * rng.uniform(0.85, 1.15, count)
    twist = rng.uniform(-0.35, 0.35, count)
    neck_i = BONES.index("neck")
    out = {k_: [] for k_ in ("pos", "wpos", "nrm", "uv", "lk", "si", "sw", "idx")}
    down = np.array([0.0, -1.0, 0.0])
    for i in range(count):
        ni, ci = n[i], cb[i]
        sv = np.cross(ci, ni)
        sv /= max(np.linalg.norm(sv), 1e-6)
        ci = ci * math.cos(twist[i]) + sv * math.sin(twist[i])
        sv = np.cross(ci, ni)
        sv /= max(np.linalg.norm(sv), 1e-6)
        l, wd = length[i], width[i]
        d_dry = ci * 0.82 + ni * 0.5
        d_dry /= np.linalg.norm(d_dry)
        d_wet = ci * 0.72 + ni * 0.7
        d_wet /= np.linalg.norm(d_wet)

        def at(t, d0, droop, ll):
            return root[i] - ni * 0.003 + d0 * ll * t + (-ni * droop + down * droop * 0.6) * ll * t * t
        head = float(bi[near[i]][0] == neck_i or (bi[near[i]][1] == neck_i and bw[near[i]][1] > 100))
        ph = (i * 0.618034) % 1.0
        base = len(out["pos"])
        for t in (0.0, 0.55):
            cd, cw = at(t, d_dry, 0.33, l), at(t, d_wet, 0.08, l * 1.12)
            hd, hw = wd * (1 - t) ** 0.7, wd * 0.38 * (1 - t) ** 0.9
            for sgn, cup in ((-1.0, 0.0), (0.0, 0.4), (1.0, 0.0)):
                out["pos"].append(cd + sv * hd * sgn + ni * hd * cup)
                out["wpos"].append(cw + sv * hw * sgn + ni * hw * cup * 0.5)
                nn = ni + sv * sgn * 0.45
                out["nrm"].append(nn / np.linalg.norm(nn))
                out["lk"].append((t, sgn, ph, head))
        out["pos"].append(at(1.0, d_dry, 0.33, l))
        out["wpos"].append(at(1.0, d_wet, 0.08, l * 1.12))
        out["nrm"].append(ni)
        out["lk"].append((1.0, 0.0, ph, head))
        for _ in range(7):
            out["uv"].append(ruv[i])
            out["si"].append(bi[near[i]])
            out["sw"].append(bw[near[i]])
        l0, m0, r0, l1, m1, r1_, tip = range(base, base + 7)
        out["idx"] += [(l0, m0, m1), (l0, m1, l1), (m0, r0, r1_), (m0, r1_, m1), (l1, m1, tip), (m1, r1_, tip)]
    return {k_: np.asarray(v) for k_, v in out.items()}


# ---------------------------------------------------------------------------------------------------- output
def b64(a):
    return base64.b64encode(np.ascontiguousarray(a).tobytes()).decode("ascii")


def smooth_normals(V, F):
    nn = np.zeros_like(V)
    fnrm = np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]])
    for i in range(3):
        np.add.at(nn, F[:, i], fnrm)
    return nn / np.maximum(np.linalg.norm(nn, axis=1), 1e-12)[:, None]


def build(name):
    c = Critter(name)
    P, N, T, hi = build_mesh(c)
    bi, bw, nearest = skin_weights(c, P, T)
    b = bake(c, P, N, T, nearest)
    src, T2, uv = b["src"], b["T"], b["uv"]
    Ps, Ns = P[src], smooth_normals(P, T)[src]
    bis, bws = bi[src], u8w(bw[src])
    count = {"beaver": 220, "otter": 150, "sea": 230}[name]
    lk = fur_locks(c, Ps, Ns, T2, uv, bis, bws, count)
    Image.fromarray(b["img"]).save(os.path.join(OUT, "fur-%s.png" % name), optimize=True)
    rest = {n: [round(float(v), 5) for v in c.J[n][0]] for n in BONES}
    print("%s: sculpt %d tris -> %d, %d verts, atlas %d px (%.0f%% used), %d tufts" %
          (name, hi, len(T2), len(src), b["size"], 100 * b["coverage"], count), flush=True)
    return {"bones": BONES, "rest": rest, "tex": "fur-%s" % name, "size": b["size"], "ppm": PPM,
            "body": {"n": int(len(src)), "pos": b64(Ps.astype(np.float32)), "nrm": b64(Ns.astype(np.float32)),
                     "uv": b64(uv.astype(np.float32)), "si": b64(bis.astype(np.uint8)), "sw": b64(bws),
                     "idx": b64(T2.astype(np.uint16)), "tris": int(len(T2))},
            "tufts": {"n": int(len(lk["pos"])), "pos": b64(lk["pos"].astype(np.float32)),
                      "wpos": b64(lk["wpos"].astype(np.float32)), "nrm": b64(lk["nrm"].astype(np.float32)),
                      "uv": b64(lk["uv"].astype(np.float32)), "lk": b64(lk["lk"].astype(np.float32)),
                      "si": b64(lk["si"].astype(np.uint8)), "sw": b64(lk["sw"].astype(np.uint8)),
                      "idx": b64(lk["idx"].astype(np.uint16)), "count": count}}


def aspen_bark():
    """A 32 x 32 tile of aspen bark, half a metre round the trunk and half a metre up it (64 texels a metre): pale grey,
    faintly streaked, a few fine darker rings, and the small dark 'eyes' where twigs fell, a darker centre in a grey
    rim. Sweaft-64 tones, no outline, nothing near white or black (it is lit like the ground)."""
    pal = [hexc(h) for h in ("#a09f91", "#bfcad7", "#92a3b2", "#434137", "#2c2c25")]
    rng = np.random.default_rng(3)
    yy, xx = np.mgrid[0:32, 0:32]
    n = k.vnoise(np.stack([xx.ravel() * 0.7, yy.ravel() * 0.15, np.zeros(1024)], 1), 1.0, 4).reshape(32, 32)
    idx = np.where(n < 0.33, 0, 1)
    idx[(yy % 9 == 4) & (rng.random((32, 32)) < 0.45)] = 2
    img = np.array([pal[i] for i in idx.ravel()]).reshape(32, 32, 3)
    for cx, cy, w in ((6, 6, 2), (21, 15, 3), (12, 25, 2), (28, 2, 1)):
        for dx in range(-w, w + 1):
            for dy in (-1, 0):
                x, y = (cx + dx) % 32, (cy + dy) % 32
                img[y, x] = pal[4] if (dy == 0 and abs(dx) < w) else pal[3] if dy == 0 else pal[0]
    Image.fromarray(img.astype(np.uint8)).save(os.path.join(OUT, "tex-aspen.png"), optimize=True)


def main():
    data = {}
    for name in ("beaver", "otter", "sea"):
        data[name] = build(name)
    head = ("// Generated by tools/art/critter_sculpt.py: the beaver, river otter and sea otter of ocean.html#critters,\n"
            "// sculpted, skinned and painted (openspec/changes/ocean-environments design B28, version 3). Do not edit.\n")
    with open(os.path.join(OUT, "critters_mesh.js"), "w") as f:
        f.write(head + "window.CRITTER_MESH = " + json.dumps(data, separators=(",", ":")) + ";\n")
    aspen_bark()
    if "--sheet" in sys.argv:
        out = sys.argv[sys.argv.index("--sheet") + 1]
        ims = [Image.open(os.path.join(OUT, "fur-%s.png" % n)).resize((512, 512), Image.NEAREST) for n in data]
        ims.append(Image.open(os.path.join(OUT, "tex-aspen.png")).resize((512, 512), Image.NEAREST))
        sheet = Image.new("RGB", (512 * len(ims), 512))
        for i, im in enumerate(ims):
            sheet.paste(im.convert("RGB"), (512 * i, 0))
        sheet.save(out)


if __name__ == "__main__":
    main()
