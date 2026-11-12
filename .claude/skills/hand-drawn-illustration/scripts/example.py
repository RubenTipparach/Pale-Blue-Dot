"""Two worked examples in the friend's manner: a health heart and an armour
shield, the subjects they drew themselves (references/process/heart_health.png,
shield_plating.png), so the result can be judged against the original.

    python3 example.py OUT_DIR

Read it as the pattern for a new drawing: one function per icon, shapes back to
front, each laid down with `drawn()` (block, hatch, shadow and light crescents,
dabs, rim, tone edge), details on top with `line()` and `dot()`.
"""

import math
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from crayon import Icon, scale  # noqa: E402


def heart_points(cx, cy, s, n=80):
    pts = []
    for i in range(n):
        t = 2 * math.pi * i / n
        x = 16 * math.sin(t) ** 3
        y = -(13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t) - math.cos(4 * t))
        pts.append((cx + x * s, cy + y * s))
    return pts


def along(poly, n):
    """n points evenly spaced round a closed polygon's edge."""
    segs = [(poly[i], poly[(i + 1) % len(poly)]) for i in range(len(poly))]
    lens = [math.hypot(b[0] - a[0], b[1] - a[1]) for a, b in segs]
    total, out = sum(lens), []
    for k in range(n):
        d = total * k / n
        for (a, b), L in zip(segs, lens):
            if d <= L:
                t = d / L
                out.append((a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t))
                break
            d -= L
    return out


def health_heart(ic):
    # the little cross first, behind the heart's top
    cross = [(240, 30), (272, 30), (272, 80), (310, 80), (310, 108), (272, 108), (272, 170),
             (240, 170), (240, 108), (202, 108), (202, 80), (240, 80)]
    ic.drawn(cross, (40, 210, 60), hatch=0.6, highlight=0, shade=0, edge=(15, 110, 30), edge_w=6)
    # a saturated green, a glossy pale rim, an edge in its own dark green
    ic.drawn(heart_points(256, 305, 13), (60, 235, 90), rim_color=(170, 255, 180), edge=(20, 140, 50))


def armour_shield(ic):
    outer = [(256, 36), (424, 88), (414, 300), (256, 476), (98, 300), (88, 88)]
    ic.drawn(outer, (125, 145, 180))                                   # the steel rim
    band = scale(outer, 0.84, (256, 250))
    ic.drawn(band, (40, 46, 60), highlight=0, shade=0.12, edge=(20, 22, 30), edge_w=5)  # dark band
    for x, y in along(scale(outer, 0.76, (256, 250)), 18):          # rivets round the band
        ic.dot(x, y, 5, (150, 160, 175), ink=(15, 16, 22))
    face = scale(outer, 0.62, (256, 245))
    ic.drawn(face, (64, 98, 150), edge=(25, 35, 60), edge_w=5)        # the face
    ic.line([(176, 160), (256, 222), (338, 160)], (250, 205, 40), 8)   # one contrasting accent line


if __name__ == "__main__":
    out = sys.argv[1] if len(sys.argv) > 1 else "."
    os.makedirs(out, exist_ok=True)
    for name, fn, seed in (("health_heart", health_heart, 7), ("armour_shield", armour_shield, 9)):
        ic = Icon(seed)
        fn(ic)
        ic.save(os.path.join(out, name + ".png"))
        print("wrote", os.path.join(out, name + ".png"))
