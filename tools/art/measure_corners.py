"""Measure a 9-slice frame's corners off a capture (`stone-and-wood-ui`).

The design's check (section 11, step 1): at UI scale 2 a frame's 14 px
corner is drawn 28 screen px, pixel exact. This finds the frame's top-left
and bottom-right corner blocks, as the game bakes them at scale 2
(`cargo test --profile fast -p pbd-app --lib -- --ignored
dump_the_frames_baked_at_scale_two` writes them to `target/kit-dump/`), in a
capture, and reports for each block size N around 28:

* how many of the block's opaque texels the capture draws exactly (each
  channel within `--tolerance`, for a PNG round trip and lavapipe's
  rounding), at the best place in the capture;
* where that place is, so the panel's drawn width is the distance between
  the two corners plus N.

A corner drawn at its native size matches all its opaque texels at N = 28 and
not beyond, where the stretched side begins. Usage:

    python3 tools/art/measure_corners.py <capture.png> <baked frame@2.png> \
        [--margin 14] [--scale 2] [--tolerance 2]
"""

import argparse
import sys

import numpy as np
from PIL import Image


def best_place(screen, block, opaque, tolerance):
    """The capture position where `block`'s opaque texels match best, and
    the share that match there."""
    h, w = block.shape[:2]
    sh, sw = screen.shape[:2]
    ys, xs = np.nonzero(opaque)
    if len(ys) == 0:
        return None, 0.0
    # Candidates: where the block's first opaque texel matches.
    y0, x0 = ys[0], xs[0]
    first = block[y0, x0]
    near = np.all(np.abs(screen.astype(int) - first.astype(int)) <= tolerance, axis=2)
    best, best_share = None, -1.0
    for cy, cx in zip(*np.nonzero(near)):
        top, left = cy - y0, cx - x0
        if top < 0 or left < 0 or top + h > sh or left + w > sw:
            continue
        drawn = screen[top + ys, left + xs].astype(int)
        want = block[ys, xs].astype(int)
        share = float(np.mean(np.all(np.abs(drawn - want) <= tolerance, axis=1)))
        if share > best_share:
            best, best_share = (left, top), share
            if share == 1.0:
                break
    return best, best_share


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("capture")
    ap.add_argument("baked")
    ap.add_argument("--margin", type=int, default=14, help="native px")
    ap.add_argument("--scale", type=int, default=2)
    ap.add_argument("--tolerance", type=int, default=2)
    args = ap.parse_args()
    screen = np.asarray(Image.open(args.capture).convert("RGB"))
    baked = np.asarray(Image.open(args.baked).convert("RGBA"))
    corner = args.margin * args.scale
    bh, bw = baked.shape[:2]
    print(f"capture {args.capture} {screen.shape[1]}x{screen.shape[0]}")
    print(f"baked {args.baked} {bw}x{bh}, corner {args.margin} native px x {args.scale} = {corner} px")
    found = {}
    for n in (corner - 2, corner, corner + 2, corner + 4):
        for name, rows, cols in (
            ("top-left", slice(0, n), slice(0, n)),
            ("bottom-right", slice(bh - n, bh), slice(bw - n, bw)),
        ):
            block = baked[rows, cols]
            rgb, opaque = block[..., :3], block[..., 3] == 255
            at, share = best_place(screen, rgb, opaque, args.tolerance)
            print(f"  N={n:2d} {name:12s} opaque {int(opaque.sum()):4d}  "
                  f"match {share * 100:6.2f} %  at {at}")
            found[(n, name)] = (at, share)
    tl, tl_share = found[(corner, "top-left")]
    br, br_share = found[(corner, "bottom-right")]
    if tl and br:
        print(f"panel drawn {br[0] + corner - tl[0]} x {br[1] + corner - tl[1]} px "
              f"from ({tl[0]}, {tl[1]})")
    exact = tl_share == 1.0 and br_share == 1.0
    beyond = max(found[(corner + 2, "top-left")][1], found[(corner + 2, "bottom-right")][1])
    print(f"corners {corner} px pixel exact: {exact}; at {corner + 2} px the best match is "
          f"{beyond * 100:.2f} % (the stretched side)")
    return 0 if exact else 1


if __name__ == "__main__":
    sys.exit(main())
