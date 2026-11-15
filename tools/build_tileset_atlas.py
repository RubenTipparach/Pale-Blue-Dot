"""Bake every biome tileset into the one atlas the planet shader binds.

A tileset is a 4x4 grid of tiles in a 1254 px sheet, and the shader has always
read exactly 32x32 texels from each tile: `pixel_tile` quantises its UV to a
32-step grid and point-samples. So a sheet's other 98% is pixels nothing can
see, and sixteen tilesets baked at the sampled resolution are a 512 px texture
rather than sixteen bindings.

The sample point is the shader's own, `(tile + 0.025 + pixel * 0.95) * 0.25`
into the source sheet, so what this writes is what the single-sheet build drew,
texel for texel. Slot order is the sorted file name, which both this tool and
`planet_terrain.rs` derive rather than store, and `--check` holds the committed
atlas to the sources.

No third-party dependencies: stdlib and zlib, like the other generators here.
"""

import statistics
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TILESETS = ROOT / "assets" / "tilesets"
OUT = TILESETS / "atlas.png"

# The shader's own grid: tiles per tileset on a side, texels per tile on a
# side, and tilesets per side of the atlas.
TILES = 4
TEXELS = 32
SETS = 4
SIZE = SETS * TILES * TEXELS


def read_png(path):
    """Decode an 8-bit RGB or RGBA PNG to (width, height, rows of RGB tuples)."""
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"{path}: not a PNG")
    pos, width, height, depth, colour, idat = 8, 0, 0, 0, 0, bytearray()
    while pos < len(data):
        (length,) = struct.unpack(">I", data[pos : pos + 4])
        kind = data[pos + 4 : pos + 8]
        body = data[pos + 8 : pos + 8 + length]
        if kind == b"IHDR":
            width, height, depth, colour = struct.unpack(">IIBB", body[:10])
        elif kind == b"IDAT":
            idat += body
        pos += 12 + length
    if depth != 8 or colour not in (2, 6):
        raise SystemExit(f"{path}: want 8-bit RGB or RGBA, got depth {depth} colour {colour}")
    channels = 3 if colour == 2 else 4
    raw = zlib.decompress(bytes(idat))
    stride = width * channels
    rows, previous, at = [], bytearray(stride), 0
    for _ in range(height):
        filter_kind = raw[at]
        line = bytearray(raw[at + 1 : at + 1 + stride])
        at += 1 + stride
        for i in range(stride):
            a = line[i - channels] if i >= channels else 0
            b = previous[i]
            c = previous[i - channels] if i >= channels else 0
            if filter_kind == 1:
                line[i] = (line[i] + a) & 0xFF
            elif filter_kind == 2:
                line[i] = (line[i] + b) & 0xFF
            elif filter_kind == 3:
                line[i] = (line[i] + (a + b) // 2) & 0xFF
            elif filter_kind == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pick = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pick) & 0xFF
            elif filter_kind != 0:
                raise SystemExit(f"{path}: unknown row filter {filter_kind}")
        rows.append(
            [tuple(line[x * channels : x * channels + 3]) for x in range(width)]
        )
        previous = line
    return width, height, rows


def encode(pixels):
    raw = bytearray()
    for row in pixels:
        raw.append(0)
        for r, g, b in row:
            raw += bytes((r, g, b))
    size = len(pixels)

    def chunk(kind, body):
        return (
            struct.pack(">I", len(body))
            + kind
            + body
            + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
        )

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def slots():
    """The tilesets in slot order: sorted file name, which the Rust side derives
    the same way rather than storing a manifest that could fall out of step."""
    names = sorted(p.stem for p in TILESETS.glob("*.png") if p.stem != "atlas")
    if len(names) > SETS * SETS:
        raise SystemExit(f"{len(names)} tilesets, but the atlas holds {SETS * SETS}")
    return names


def bake():
    pixels = [[(0, 0, 0)] * SIZE for _ in range(SIZE)]
    for slot, name in enumerate(slots()):
        width, height, rows = read_png(TILESETS / f"{name}.png")
        if width != height:
            raise SystemExit(f"{name}: tilesets are square, got {width}x{height}")
        ox = (slot % SETS) * TILES * TEXELS
        oy = (slot // SETS) * TILES * TEXELS
        for ty in range(TILES):
            for tx in range(TILES):
                for py in range(TEXELS):
                    for px in range(TEXELS):
                        # The shader's own sample point, in sheet UV.
                        u = (tx + 0.025 + ((px + 0.5) / TEXELS) * 0.95) * 0.25
                        v = (ty + 0.025 + ((py + 0.5) / TEXELS) * 0.95) * 0.25
                        sx = min(int(u * width), width - 1)
                        sy = min(int(v * height), height - 1)
                        pixels[oy + ty * TEXELS + py][ox + tx * TEXELS + px] = rows[sy][sx]
    return pixels


# Painted flowers taken off the grassy ground tiles (owner, 2026-11-15: "I would
# also remove the flowers from the ground texture as well, we have actual
# flowers already"; `flora-v2` design 9.5). The rules are the M13 mockup's
# `docs/mockups/foliage/no_flowers.py`, run on the baked atlas so the source
# sheets stay as generated and `--check` still holds the atlas to them:
#   fields (slot 4) grass top and sod lip: daisies, a near-white petal or a warm
#   yellow centre; swamp (12) moss top: the pale yellow sprouts; tundra (13) top:
#   the orange flower dots. Each marked texel, and its neighbours that match a
#   looser form of the rule, takes the median colour of the unmarked ground in
#   its 5 x 5 neighbourhood. The result is byte-for-byte the mockup's own copy.
FLOWER_RULES = {
    (4, (0, 0)): (
        lambda r, g, b: min(r, g, b) > 185 or (r > g + 4 and r > 170),
        lambda r, g, b: r + g + b > 420,
    ),
    (4, (1, 0)): (
        lambda r, g, b: min(r, g, b) > 185 or (r > g + 4 and r > 170),
        lambda r, g, b: r + g + b > 420,
    ),
    (12, (0, 0)): (
        lambda r, g, b: r > 125 and r >= g - 12,
        lambda r, g, b: r > 110 and r >= g - 16,
    ),
    (13, (0, 0)): (
        lambda r, g, b: r > g + 20 and b < 140,
        lambda r, g, b: r > g + 8 and b < 150,
    ),
}


def clean_flowers(pixels):
    for (slot, (tx, ty)), (strict, loose) in FLOWER_RULES.items():
        x0 = (slot % SETS) * TILES * TEXELS + tx * TEXELS
        y0 = (slot // SETS) * TILES * TEXELS + ty * TEXELS

        def at(x, y):
            return pixels[y0 + y][x0 + x]

        mark = {
            (x, y) for y in range(TEXELS) for x in range(TEXELS) if strict(*at(x, y))
        }
        grow = set()
        for x, y in mark:
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if (
                    0 <= nx < TEXELS
                    and 0 <= ny < TEXELS
                    and (nx, ny) not in mark
                    and loose(*at(nx, ny))
                ):
                    grow.add((nx, ny))
        mark |= grow
        for x, y in sorted(mark):
            near = [
                at(i, j)
                for j in range(y - 2, y + 3)
                for i in range(x - 2, x + 3)
                if 0 <= i < TEXELS and 0 <= j < TEXELS and (i, j) not in mark
            ]
            if near:
                pixels[y0 + y][x0 + x] = tuple(
                    int(statistics.median(c[k] for c in near)) for k in range(3)
                )
    return pixels


def main():
    baked = encode(clean_flowers(bake()))
    if "--check" in sys.argv:
        if not OUT.exists():
            raise SystemExit(f"{OUT} is missing; run this tool without --check")
        if OUT.read_bytes() != baked:
            raise SystemExit(f"{OUT} has drifted from the tilesets; rerun this tool")
        print(f"{OUT.name} matches its {len(slots())} tilesets")
        return
    OUT.write_bytes(baked)
    print(f"wrote {OUT} ({SIZE}x{SIZE}) from {len(slots())} tilesets:")
    for slot, name in enumerate(slots()):
        print(f"  {slot:2d} {name}")


if __name__ == "__main__":
    main()
