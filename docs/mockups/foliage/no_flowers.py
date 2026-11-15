"""Take the painted daisies out of the fields sheet's grass tiles in the mockup's atlas copy.

Owner, 2026-11-15: "I would also remove the flowers from the ground texture as well, we have actual flowers
already". Mockup copy only (foliage/game/atlas.png); the game's assets/tilesets atlas is unchanged. A petal is a
near-white pixel, a centre a warm yellow one (red above green); each, and its bright neighbours, takes the median
colour of the unmarked grass in its 5 x 5 neighbourhood. Run from docs/mockups: python3 foliage/no_flowers.py
"""
from PIL import Image
import statistics

PATH = "foliage/game/atlas.png"
SHEET = 4                      # fields
TILES = [(0, 0), (1, 0)]       # the grass top and the sod side's grass lip


def main():
    im = Image.open(PATH).convert("RGB")
    px = im.load()
    sx, sy = (SHEET % 4) * 128, (SHEET // 4) * 128
    changed = 0
    for tx, ty in TILES:
        x0, y0 = sx + tx * 32, sy + ty * 32
        flower = lambda r, g, b: min(r, g, b) > 185 or (r > g + 4 and r > 170)
        mark = {(x, y) for y in range(32) for x in range(32) if flower(*px[x0 + x, y0 + y])}
        # grow by one pixel into neighbours brighter than the grass around them
        grow = set()
        for (x, y) in mark:
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < 32 and 0 <= ny < 32 and (nx, ny) not in mark and sum(px[x0 + nx, y0 + ny]) > 420:
                    grow.add((nx, ny))
        mark |= grow
        for (x, y) in mark:
            near = [px[x0 + i, y0 + j] for j in range(y - 2, y + 3) for i in range(x - 2, x + 3)
                    if 0 <= i < 32 and 0 <= j < 32 and (i, j) not in mark]
            if near:
                px[x0 + x, y0 + y] = tuple(int(statistics.median(c[k] for c in near)) for k in range(3))
                changed += 1
    im.save(PATH)
    print("pixels replaced:", changed)


if __name__ == "__main__":
    main()
