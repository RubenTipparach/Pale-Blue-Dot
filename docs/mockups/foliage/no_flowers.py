"""Take the painted flowers out of the grassy ground tiles in the mockup's atlas copy.

Owner, 2026-11-15: "I would also remove the flowers from the ground texture as well, we have actual flowers
already". Mockup copy only (foliage/game/atlas.png); the game's assets/tilesets atlas is unchanged.

Each tile has its own rule for what a painted flower pixel is:
  fields (sheet 4) grass top and sod lip: daisies, a near-white petal or a warm yellow centre;
  swamp (sheet 12) moss top: the pale yellow sprouts that read as tiny flowers at a distance;
  tundra (sheet 13) top, the taiga's ground: the orange flower dots and their darker rims.
Each marked pixel, and its neighbours that match a looser form of the same rule, takes the median colour of the
unmarked ground in its 5 x 5 neighbourhood. The rules no longer match once a tile is clean, so a second run
changes nothing. The jungle's yellow fern sprigs are leaves, not flowers, and stay.
Run from docs/mockups: python3 foliage/no_flowers.py
"""
from PIL import Image
import statistics

PATH = "foliage/game/atlas.png"

# (sheet, (column, row)): (strict rule, loose rule for the one-pixel grow)
RULES = {
    (4, (0, 0)): (lambda r, g, b: min(r, g, b) > 185 or (r > g + 4 and r > 170), lambda r, g, b: r + g + b > 420),
    (4, (1, 0)): (lambda r, g, b: min(r, g, b) > 185 or (r > g + 4 and r > 170), lambda r, g, b: r + g + b > 420),
    (12, (0, 0)): (lambda r, g, b: r > 125 and r >= g - 12, lambda r, g, b: r > 110 and r >= g - 16),
    (13, (0, 0)): (lambda r, g, b: r > g + 20 and b < 140, lambda r, g, b: r > g + 8 and b < 150),
}


def clean(px, x0, y0, strict, loose):
    mark = {(x, y) for y in range(32) for x in range(32) if strict(*px[x0 + x, y0 + y])}
    grow = set()
    for (x, y) in mark:
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            nx, ny = x + dx, y + dy
            if 0 <= nx < 32 and 0 <= ny < 32 and (nx, ny) not in mark and loose(*px[x0 + nx, y0 + ny]):
                grow.add((nx, ny))
    mark |= grow
    changed = 0
    for (x, y) in sorted(mark):
        near = [px[x0 + i, y0 + j] for j in range(y - 2, y + 3) for i in range(x - 2, x + 3)
                if 0 <= i < 32 and 0 <= j < 32 and (i, j) not in mark]
        if near:
            px[x0 + x, y0 + y] = tuple(int(statistics.median(c[k] for c in near)) for k in range(3))
            changed += 1
    return changed


def main():
    im = Image.open(PATH).convert("RGB")
    px = im.load()
    for (sheet, (tx, ty)), (strict, loose) in RULES.items():
        x0, y0 = (sheet % 4) * 128 + tx * 32, (sheet // 4) * 128 + ty * 32
        print("sheet", sheet, "tile", (tx, ty), "pixels replaced:", clean(px, x0, y0, strict, loose))
    im.save(PATH)


if __name__ == "__main__":
    main()
