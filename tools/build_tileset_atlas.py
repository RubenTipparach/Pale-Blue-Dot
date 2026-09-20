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


def main():
    baked = encode(bake())
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
