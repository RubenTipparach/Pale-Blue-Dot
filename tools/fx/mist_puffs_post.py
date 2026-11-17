"""Cut the Blender mist render to the game's 32-texel cells (`water-flow` 6.5).

Reads `lit.png` and `normal.png` from `mist_puffs_blender.py` (a 4 x 4 sheet,
any cell size) and writes a 128 x 128 sheet of 32 x 32 cells, sampled nearest
in use like the splash dot it replaces:

- `albedo.png`: rgb the puff's shade as a five-tone ramp lit from above (the
  game's light multiplies it), alpha its cover, faded over its outer three
  texels so a puff reads as mist, not a cut-out. No outline (CLAUDE.md "No
  outlines in the world").
- `normal.png`: the camera-space normal, 0.5 + 0.5 n, for lighting the puffs
  by the sun as Crab Breeze's `smoke_lit` is.

    python3 tools/fx/mist_puffs_post.py <render dir> <out dir>
"""

import sys
from pathlib import Path

import numpy as np
from PIL import Image

GRID, TEX = 4, 32
# The ramp, darkest to lightest: cool greys to white, as the game's foam
# colour (0.95, 0.97, 1.0) under light from above.
RAMP = np.array(
    [[150, 160, 176], [180, 190, 204], [206, 214, 226], [228, 234, 242], [248, 250, 255]],
    dtype=np.float64,
)
EDGE = [0.35, 0.65, 0.88]   # alpha a texel 1, 2, 3 in from the edge


def cells(img):
    a = np.asarray(img, dtype=np.float64) / 255.0
    n = a.shape[0] // GRID
    return [[a[r * n:(r + 1) * n, c * n:(c + 1) * n] for c in range(GRID)] for r in range(GRID)]


def down(cell):
    """Premultiplied box average to TEX x TEX."""
    n = cell.shape[0]
    k = n // TEX
    cell = cell[: k * TEX, : k * TEX]
    rgb, a = cell[..., :3], cell[..., 3:4]
    pre = (rgb * a).reshape(TEX, k, TEX, k, 3).mean(axis=(1, 3))
    al = a.reshape(TEX, k, TEX, k, 1).mean(axis=(1, 3))
    return np.where(al > 1e-4, pre / np.maximum(al, 1e-4), 0.0), al[..., 0]


def inset(mask):
    """How many texels in from the edge each covered texel is (1 at the edge)."""
    d = np.zeros(mask.shape, dtype=int)
    cur = mask.copy()
    step = 0
    while cur.any() and step < 8:
        step += 1
        d[cur] = step
        p = np.pad(cur, 1)
        cur = cur & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
    return d


def main():
    src, out = Path(sys.argv[1]), Path(sys.argv[2])
    out.mkdir(parents=True, exist_ok=True)
    lit = cells(Image.open(src / "lit.png").convert("RGBA"))
    nrm = cells(Image.open(src / "normal.png").convert("RGBA"))
    albedo = np.zeros((GRID * TEX, GRID * TEX, 4), dtype=np.uint8)
    normal = np.zeros((GRID * TEX, GRID * TEX, 4), dtype=np.uint8)
    for r in range(GRID):
        for c in range(GRID):
            rgb, cover = down(lit[r][c])
            nrgb, _ = down(nrm[r][c])
            mask = cover > 0.5
            d = inset(mask)
            alpha = np.where(mask, 1.0, 0.0)
            for i, e in enumerate(EDGE):
                alpha = np.where(d == i + 1, e, alpha)
            # The shade: the puff's normal against light from above (up, a
            # little left and toward the eye), over a sky fill, snapped to
            # the ramp. The render's own shading is too even at 32 texels.
            n = nrgb * 2.0 - 1.0
            n /= np.maximum(np.linalg.norm(n, axis=-1, keepdims=True), 1e-6)
            sun = np.array([-0.35, 0.8, 0.5])
            sun /= np.linalg.norm(sun)
            t = np.clip(0.02 + 0.98 * np.clip(n @ sun, 0.0, 1.0) ** 1.4, 0.0, 1.0)
            # The lumps' creases: where the render is darker than its normal says.
            lum = rgb @ np.array([0.2126, 0.7152, 0.0722])
            if mask.any():
                t = np.clip(t - 0.35 * np.clip(np.median(lum[mask]) - lum, 0.0, 1.0), 0.0, 1.0)
            tone = RAMP[np.clip((t * len(RAMP)).astype(int), 0, len(RAMP) - 1)]
            ys, xs = slice(r * TEX, (r + 1) * TEX), slice(c * TEX, (c + 1) * TEX)
            albedo[ys, xs, :3] = np.where(mask[..., None], tone, 0).astype(np.uint8)
            albedo[ys, xs, 3] = np.round(alpha * 255).astype(np.uint8)
            normal[ys, xs, :3] = np.where(mask[..., None], np.round(nrgb * 255), 128).astype(np.uint8)
            normal[ys, xs, 3] = np.where(mask, 255, 0).astype(np.uint8)
    Image.fromarray(albedo).save(out / "albedo.png")
    Image.fromarray(normal).save(out / "normal.png")


if __name__ == "__main__":
    main()
