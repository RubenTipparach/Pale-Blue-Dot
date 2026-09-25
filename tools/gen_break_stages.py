#!/usr/bin/env python3
"""Draw the ten crack stages shown over a block being broken, Minecraft's
destroy_stage_0..9, as 32x32 RGBA PNGs under assets/textures/break/.

One set of jagged fractures is walked out from an impact point near the middle
of the square, branching as it goes, and every crack pixel is given the time
the crack reached it. Stage k draws the first (k+1)/10 of those pixels in time
order, so each stage contains every crack pixel of the stage before it and
the block reads as one block cracking apart, not ten different pictures. A
crack pixel is near black; some carry a lighter pixel below and to the right,
the chipped edge that makes Minecraft's cracks read as cut into the surface
rather than drawn on it. Tenebris's generator (tools/gen-break-stages.py in
tenebris-rs) is the same idea over six stages; the drawing here is new.

32 px because a terrain atlas tile is 32 px: the overlay uses the terrain's
own face UVs and nearest sampling, so one crack pixel is one block pixel.

    python3 tools/gen_break_stages.py            # write the stages
    python3 tools/gen_break_stages.py --check    # fail if any differs

Deterministic, stdlib only (zlib), so --check holds on any machine.
"""

import math
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "assets" / "textures" / "break"
S = 32        # pixels a side, the atlas tile size
STAGES = 10   # Minecraft's count

CRACK = (18, 14, 11, 225)     # near black, a little warm
CHIP = (255, 250, 240, 70)    # the lit edge beside a crack

_seed = [0x5EED_B10C]


def rnd():
    """A deterministic LCG in [0, 1)."""
    _seed[0] = (_seed[0] * 1103515245 + 12345) & 0x7FFFFFFF
    return _seed[0] / 0x80000000


def line(x0, y0, x1, y1):
    """The pixels of a 1 px line, Bresenham, first to last."""
    x0, y0, x1, y1 = (int(round(v)) for v in (x0, y0, x1, y1))
    dx, dy = abs(x1 - x0), -abs(y1 - y0)
    sx, sy = (1 if x0 < x1 else -1), (1 if y0 < y1 else -1)
    err = dx + dy
    out = []
    while True:
        out.append((x0, y0))
        if x0 == x1 and y0 == y1:
            return out
        e2 = 2 * err
        if e2 >= dy:
            err += dy
            x0 += sx
        if e2 <= dx:
            err += dx
            y0 += sy


def fractures():
    """Every crack pixel and the time the crack reaches it."""
    times = {}

    def walk(x, y, angle, steps, t, depth):
        for _ in range(steps):
            angle += (rnd() - 0.5) * 1.1
            length = 1.6 + rnd() * 1.8
            nx = min(max(x + math.cos(angle) * length, 0.0), S - 1.0)
            ny = min(max(y + math.sin(angle) * length, 0.0), S - 1.0)
            for px, py in line(x, y, nx, ny)[1:]:
                t += 1.0
                if (px, py) not in times or times[(px, py)] > t:
                    times[(px, py)] = t
            x, y = nx, ny
            # A branch leaves at an angle and is shorter, and starts when the
            # crack it leaves from reaches that point.
            if depth < 2 and rnd() < 0.3:
                side = 1 if rnd() < 0.5 else -1
                walk(x, y, angle + side * (0.7 + rnd() * 0.6),
                     3 + int(rnd() * 4), t, depth + 1)
            if x in (0.0, S - 1.0) or y in (0.0, S - 1.0):
                return

    cx, cy = S * 0.5 + (rnd() - 0.5) * 3, S * 0.5 + (rnd() - 0.5) * 3
    times[(int(cx), int(cy))] = 0.0
    arms = 6
    for arm in range(arms):
        angle = arm * 2 * math.pi / arms + (rnd() - 0.5) * 0.6
        walk(cx, cy, angle, 8 + int(rnd() * 6), rnd() * 3.0, 0)
    return times


def stages():
    times = fractures()
    order = sorted(times, key=lambda p: (times[p], p[1], p[0]))
    # Which crack pixels carry a chipped edge: decided once, so a stage adds
    # chips and never moves one.
    chipped = {p for p in order if rnd() < 0.35}
    grids = []
    for k in range(STAGES):
        shown = order[: max(1, round(len(order) * (k + 1) / STAGES))]
        grid = [[(0, 0, 0, 0)] * S for _ in range(S)]
        cracks = set(shown)
        for (x, y) in shown:
            if (x, y) in chipped:
                cx, cy = x + 1, y + 1
                if cx < S and cy < S and (cx, cy) not in cracks:
                    grid[cy][cx] = CHIP
        for (x, y) in shown:
            grid[y][x] = CRACK
        grids.append(grid)
    return grids


def png(grid):
    raw = b"".join(b"\x00" + b"".join(bytes(px) for px in row) for row in grid)

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", S, S, 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
            + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b""))


def main():
    check = "--check" in sys.argv[1:]
    grids = stages()
    # Each stage holds every crack pixel of the one before: checked here as
    # well as in the Rust test, since this is where it is made true.
    for k in range(1, STAGES):
        for y in range(S):
            for x in range(S):
                if grids[k - 1][y][x] == CRACK:
                    assert grids[k][y][x] == CRACK, f"stage {k} lost ({x},{y})"
    stale = []
    for k, grid in enumerate(grids):
        path = ROOT / f"stage_{k}.png"
        data = png(grid)
        if check:
            if not path.exists() or path.read_bytes() != data:
                stale.append(str(path))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    if stale:
        raise SystemExit("stale stages (run tools/gen_break_stages.py): " + ", ".join(stale))
    counts = [sum(px == CRACK for row in g for px in row) for g in grids]
    print(("checked " if check else "wrote ") + f"{STAGES} stages, crack pixels {counts}")


if __name__ == "__main__":
    main()
