#!/usr/bin/env python3
"""Draw the pixel textures of the lights held in hand (`glowing-mushrooms`
design 14, Part C) as RGBA PNGs under assets/textures/held_light/: the
torch's haft and head, its flame sheet, a lantern's iron and its panes lit and
dark, the glowing mushroom's stalk, and the flame's soft glow.

They are the Foliage Detail Playground's (docs/mockups/foliage.html, the Glow
tab, version 3, `heldTextures` and `flameSheet`), ported line for line with
the same seeded random (`rngS`), so the game draws what the mockup showed and
the owner approved. Every colour is from the mockup's Sweaft-64 ramps (`SW`).

The flame sheet is the mockup's eight 12 x 20 frames in three rows: the
whole flame for the outer card, then only its orange and hotter texels for
the middle card, then only its yellow and white heart for the inner one (the
mockup's FLAME_FS discards by the texel's green, 0.36 and 0.55), so the
three cards stack from a red rim to a white heart.

    python3 tools/gen_held_lights.py            # write the textures
    python3 tools/gen_held_lights.py --check    # fail if any differs

Deterministic, stdlib only, so --check holds on any machine.
"""

import math
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "assets" / "textures" / "held_light"

SW = {
    "char": ["#161612", "#2c2c25", "#434137"],
    "wood": ["#41322c", "#725244", "#7d4636", "#a8633e"],
    "rag": ["#522b2f", "#7d4636", "#a8633e", "#cfaa81"],
    "cord": ["#2e0d06", "#41322c", "#725244"],
    "flame": ["#a61e1f", "#d4462b", "#fe6420", "#fb902f", "#faaf5c", "#fffefe"],
    "iron": ["#161612", "#2c2c25", "#434137", "#5d778d"],
}


def rng(seed):
    """The mockup's `rngS`: a 32-bit LCG, in 0..1."""
    state = [seed & 0xFFFFFFFF]

    def next_value():
        state[0] = (state[0] * 1664525 + 1013904223) & 0xFFFFFFFF
        return state[0] / 4294967296

    return next_value


def rgba(hex_colour, alpha=255):
    h = hex_colour.lstrip("#")
    return bytes(int(h[i:i + 2], 16) for i in (0, 2, 4)) + bytes([alpha])


class Canvas:
    def __init__(self, w, h):
        self.w, self.h = w, h
        self.px = [[b"\x00\x00\x00\x00"] * w for _ in range(h)]

    def put(self, x, y, colour):
        self.px[y][x] = rgba(colour) if isinstance(colour, str) else colour

    def png(self):
        raw = b"".join(b"\x00" + b"".join(row) for row in self.px)

        def chunk(kind, data):
            body = kind + data
            return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

        header = struct.pack(">IIBBBBB", self.w, self.h, 8, 6, 0, 0, 0)
        return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
                + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def haft():
    """8 x 32 round and along a 4 cm stick: bark strands, darker grain runs,
    the odd pale fleck, a knot."""
    r, w, h, c, wood = rng(17), 8, 32, Canvas(8, 32), SW["wood"]
    for x in range(w):
        run = 0
        for y in range(h):
            if run <= 0 and r() < 0.09:
                run = 2 + int(r() * 6)
            col = wood[2] if x % 3 == 0 else wood[1]
            if run > 0:
                col = wood[0]
                run -= 1
            elif r() < 0.06:
                col = wood[3]
            c.put(x, y, col)
    for x, y, k in [(5, 21, 0), (6, 21, 0), (5, 22, 0), (6, 22, 3), (4, 22, 2)]:
        c.put(x, y, wood[k])
    return c


def head():
    """16 x 14, a rag wound round the top of the stick and soaked in pitch:
    the charred crust with live embers, scorched cloth, the wraps in
    diagonal bands with dark seams and pitch stains, the cord binding it."""
    r = rng(29)
    stains = set()
    for _ in range(7):
        sx, sy = int(r() * 16), 4 + int(r() * 7)
        stains.add((sx, sy))
        stains.add(((sx + 1) % 16, sy))
        if r() < 0.5:
            stains.add((sx, sy + 1))
    c, ch, rag, cord, fl = Canvas(16, 14), SW["char"], SW["rag"], SW["cord"], SW["flame"]
    for x in range(16):
        c.put(x, 0, fl[2] if r() < 0.16 else (ch[1] if r() < 0.3 else ch[0]))
        c.put(x, 1, fl[0] if r() < 0.06 else (ch[1] if r() < 0.35 else ch[0]))
        c.put(x, 2, ch[1] if x & 1 else ch[2])
        c.put(x, 3, rag[0] if r() < 0.5 else ch[1])
        for y in range(4, 12):
            bp, band = (y + x // 2) % 3, (y + x // 2) // 3
            col = rag[0] if bp == 0 else (rag[1] if band % 2 else rag[2])
            if bp == 1 and band % 2 == 0 and r() < 0.12:
                col = rag[3]
            if (x, y) in stains:
                col = ch[1]
            c.put(x, y, col)
        c.put(x, 12, cord[1] if x % 2 else cord[2])
        c.put(x, 13, cord[0] if (x + 1) % 2 else cord[1])
    return c


def pane():
    """A lantern's pane, lit: 8 x 10, the glass glowing round a small flame."""
    c, fl = Canvas(8, 10), SW["flame"]
    for x in range(8):
        for y in range(10):
            c.put(x, y, fl[3])
    for y in range(2, 8):
        for x in range(2, 6):
            c.put(x, y, fl[4])
    for x, y, k in [(3, 3, 5), (4, 4, 5), (3, 4, 5), (3, 5, 5), (4, 5, 5), (3, 6, 4), (4, 6, 4)]:
        c.put(x, y, fl[k])
    for x in range(8):
        c.put(x, 0, fl[2])
    return c


def pane_day():
    """...and dark by day: glass over a dead wick."""
    c = Canvas(8, 10)
    for x in range(8):
        for y in range(10):
            c.put(x, y, "#53998b" if (x + y) % 5 == 0 else "#204f63")
    for x, y, col in [(3, 6, SW["char"][1]), (4, 6, SW["char"][1]),
                      (3, 7, SW["cord"][1]), (4, 7, SW["cord"][1])]:
        c.put(x, y, col)
    return c


def stalk():
    """The mushroom's stalk: the plant's pale green, a darker streak."""
    r, c = rng(53), Canvas(6, 16)
    for x in range(6):
        for y in range(16):
            c.put(x, y, "#559642" if x == 2 and y > 3 else ("#c1f1d8" if r() < 0.08 else "#9bb675"))
    return c


def iron():
    """Iron: dark, a lighter speck of wear."""
    r, c, ir = rng(41), Canvas(8, 8), SW["iron"]
    for x in range(8):
        for y in range(8):
            c.put(x, y, ir[2] if r() < 0.1 else (ir[1] if r() < 0.5 else ir[0]))
    return c


FRAMES, FW, FH = 8, 12, 20


def flame_texel(f, x, py):
    """The mockup's `flameSheet` at one texel: a ramp index, or None."""
    ph = f / FRAMES * math.pi * 2
    t = (FH - 1 - py) / (FH - 1)
    wid = (0.7 + 1.3 * t if t < 0.22 else max(0.0, 1 - (t - 0.22) / 0.8) ** 0.85) * FW * 0.46
    cx = FW / 2 + math.sin(ph + t * 4.2) * t * 1.1 + math.sin(2 * ph + t * 7.0 + 1.0) * t * 0.6
    n = 0.5 + 0.25 * math.sin(x * 1.7 + py * 0.9 - ph * 2) + 0.25 * math.sin(x * 0.6 - py * 1.3 + ph * 3 + 2)
    v = 1 - abs(x + 0.5 - cx) / max(0.3, wid)
    v = v * (1 - 0.3 * t) - (n - 0.5) * 0.35 * t + (0.1 if t < 0.12 else 0)
    if t > 0.86 and abs(x + 0.5 - cx - math.sin(ph * 2) * 1.2) < 0.9 and math.sin(ph * 3 + 1) > 0.2:
        v = max(v, 0.15)
    if v > 0.74:
        return 5
    if v > 0.56:
        return 4
    if v > 0.4:
        return 3
    if v > 0.24:
        return 2
    if v > 0.12:
        return 0 if t > 0.45 else 1
    return None


def flame():
    """Eight frames across, three rows down: the outer card's whole flame,
    the middle card's orange and hotter, the inner card's yellow and white."""
    c, fl = Canvas(FW * FRAMES, FH * 3), SW["flame"]
    keep = [0.0, 0.36, 0.55]
    for row in range(3):
        for f in range(FRAMES):
            for py in range(FH):
                for x in range(FW):
                    k = flame_texel(f, x, py)
                    if k is None:
                        continue
                    green = int(fl[k][3:5], 16) / 255
                    if green < keep[row]:
                        continue
                    c.put(f * FW + x, row * FH + py, fl[k])
    return c


def glow():
    """The flame's soft glow, 16 x 16: (1 - r)^2 in its alpha, warm."""
    c = Canvas(16, 16)
    for y in range(16):
        for x in range(16):
            d = math.hypot((x + 0.5) / 8 - 1, (y + 0.5) / 8 - 1)
            a = max(0.0, 1 - d) ** 2
            c.put(x, y, bytes([255, 140, 56, int(round(a * 255))]))
    return c


TEXTURES = {
    "haft": haft, "head": head, "pane": pane, "pane_day": pane_day,
    "stalk": stalk, "iron": iron, "flame": flame, "glow": glow,
}


def main():
    check = "--check" in sys.argv[1:]
    stale = []
    for name, draw in TEXTURES.items():
        path = ROOT / f"{name}.png"
        data = draw().png()
        if check:
            if not path.exists() or path.read_bytes() != data:
                stale.append(str(path))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    if stale:
        raise SystemExit("stale textures (run tools/gen_held_lights.py): " + ", ".join(stale))
    print(("checked " if check else "wrote ") + f"{len(TEXTURES)} textures")


if __name__ == "__main__":
    main()
