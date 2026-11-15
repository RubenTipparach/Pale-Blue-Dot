# Two 24x24 bestiary portraits for M25's reef (anthias, damsel), side view head right, Sweaft-64 tones,
# a 1 px outline (161612), lit from the top left, hard alpha; the same framing as ocean/*.png.
from PIL import Image
import math, sys
OUT = sys.argv[1]
def hexc(h): return tuple(int(h[i:i+2], 16) for i in (0, 2, 4)) + (255,)
def fish(name, back, side, belly, fin, tailc, eye, depth, spots=None, stripe=None):
    W = 24; im = Image.new("RGBA", (W, W), (0, 0, 0, 0)); px = im.load()
    cx, cy, L = 13.5, 12.0, 7.6
    mask = [[0]*W for _ in range(W)]
    for y in range(W):
        for x in range(W):
            u = (x + 0.5 - cx) / L; v = (y + 0.5 - cy)
            if -1 <= u <= 1:
                hh = depth * (1 - u*u) ** 0.6 * (1.0 if u > -0.2 else 0.85)
                if abs(v) <= hh: mask[y][x] = 1
            # tail: a fork behind
            tu = (cx - L) - (x + 0.5)
            if 0 <= tu <= 4.5 and abs(v) <= 1 + tu * 0.9 and not (tu > 2.5 and abs(v) < (tu - 2.5) * 0.9): mask[y][x] = 2
            # dorsal and anal fins
            if -0.5 <= u <= 0.5 and -depth - 2.2 + abs(u) * 3 <= v < -depth * (1 - u*u) ** 0.6 + 0.5: mask[y][x] = mask[y][x] or 3
            if -0.6 <= u <= -0.1 and depth * (1 - u*u) ** 0.6 - 0.5 < v <= depth + 1.2 - abs(u + 0.35) * 4: mask[y][x] = mask[y][x] or 3
    for y in range(W):
        for x in range(W):
            m = mask[y][x]
            if not m: continue
            v = (y + 0.5 - cy)
            if m == 1:
                c = back if v < -depth * 0.35 else belly if v > depth * 0.35 else side
                if stripe and abs(v) < 0.8 and x > 7: c = stripe
                if spots and (x * 7 + y * 13) % 11 == 0 and v < 1: c = spots
            elif m == 2: c = tailc
            else: c = fin
            px[x, y] = hexc(c)
    # highlight top-left of the body, eye, outline
    for y in range(W):
        for x in range(W):
            if mask[y][x] == 1 and y > 0 and mask[y-1][x] != 1 and x < cx + 2: px[x, y] = hexc(side)
    ex, ey = int(cx + L * 0.62), int(cy - depth * 0.3)
    px[ex, ey] = hexc(eye); px[ex - 1, ey] = hexc("fffefe") if eye != "fffefe" else hexc("161612")
    out = im.copy(); op = out.load()
    for y in range(W):
        for x in range(W):
            if px[x, y][3] == 0 and any(0 <= x+dx < W and 0 <= y+dy < W and px[x+dx, y+dy][3] for dx, dy in ((1,0),(-1,0),(0,1),(0,-1))):
                op[x, y] = hexc("161612")
    out.save(f"{OUT}/{name}.png")
fish("anthias", "dd576d", "fb902f", "faaf5c", "fb902f", "dd576d", "161612", 3.4, stripe="faaf5c")
fish("damsel", "161612", "2c2c25", "434137", "161612", "2c2c25", "fffefe", 4.6, spots="fffefe")
