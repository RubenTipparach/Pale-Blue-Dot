"""Write a 32 x 32 tile into a 1254 px tileset sheet at the shader's sample grid.

The game reads 32 x 32 texels out of each tile of a 4 x 4 sheet: the sample
point of texel `p` in tile `t` is `(t + 0.025 + ((p + 0.5) / 32) * 0.95) / 4`
of the sheet (`tools/build_tileset_atlas.py`, `pixel_tile` in
`planet_surface.wgsl`). This paints every source pixel of tile `(tx, ty)` with
the texel whose sample it is nearest to, so the rebaked atlas holds the tile
texel for texel. Rebake with `tools/build_tileset_atlas.py` afterwards.

    python tools/paint_sheet_tile.py <tile.png> <sheet name> <tx> <ty> [--from x y]

`--from x y` takes the 32 x 32 block at texel (x, y) of the tile image, so a
candidate strip (`docs/mockups/sand/sand_tiles.png`) can be read directly.
Used for `sand-tiles` (beach and desert sand) and `mountain-ground` (the alpine
turf in the mountains sheet's (3,2)).
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
TILESETS = ROOT / "assets" / "tilesets"
TEXELS = 32
TILES = 4


def paint(tile, sheet_name, tx, ty):
    path = TILESETS / f"{sheet_name}.png"
    sheet = Image.open(path)
    mode = sheet.mode
    a = np.array(sheet.convert("RGBA"))
    size = a.shape[0]
    t = np.array(tile.convert("RGBA"))
    assert t.shape[:2] == (TEXELS, TEXELS), t.shape
    x0, x1 = tx * size // TILES, (tx + 1) * size // TILES
    y0, y1 = ty * size // TILES, (ty + 1) * size // TILES

    def texel(s, t0):
        # Invert the sample: which texel's sample point is nearest this pixel.
        u = (s + 0.5) / size * TILES - t0
        p = np.floor(((u - 0.025) / 0.95) * TEXELS)
        return np.clip(p, 0, TEXELS - 1).astype(int)

    xs = texel(np.arange(x0, x1), tx)
    ys = texel(np.arange(y0, y1), ty)
    a[y0:y1, x0:x1] = t[np.ix_(ys, xs)]
    Image.fromarray(a).convert(mode).save(path)
    # Read back at the shader's own sample points: the tile, texel for texel.
    check = np.array(Image.open(path).convert("RGBA"))
    for py in range(TEXELS):
        for px in range(TEXELS):
            u = (tx + 0.025 + ((px + 0.5) / TEXELS) * 0.95) / TILES
            v = (ty + 0.025 + ((py + 0.5) / TEXELS) * 0.95) / TILES
            sx = min(int(u * size), size - 1)
            sy = min(int(v * size), size - 1)
            assert (check[sy, sx, :3] == t[py, px, :3]).all(), (sheet_name, px, py)
    print(f"painted {sheet_name} ({tx},{ty})")


def main():
    args = sys.argv[1:]
    offset = (0, 0)
    if "--from" in args:
        i = args.index("--from")
        offset = (int(args[i + 1]), int(args[i + 2]))
        del args[i : i + 3]
    src, sheet_name, tx, ty = args[0], args[1], int(args[2]), int(args[3])
    img = Image.open(src).convert("RGBA")
    ox, oy = offset
    paint(img.crop((ox, oy, ox + TEXELS, oy + TEXELS)), sheet_name, tx, ty)


if __name__ == "__main__":
    main()
