#!/usr/bin/env python3
"""Crop every frame of a capture sequence (`PBD_CAPTURE_RUN`) to one box and
lay them in a row, labelled by frame, so a motion reads frame by frame
(`townsfolk` task 2.2). Nearest-neighbour scaling keeps the pixels.

Usage:
  tools/crop_sequence.py OUT.jpg DIR X0 Y0 X1 Y1 [--scale 2] [--every 1]
"""
import argparse
import glob
import os

from PIL import Image, ImageDraw


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("dir")
    ap.add_argument("box", nargs=4, type=int)
    ap.add_argument("--scale", type=int, default=2)
    ap.add_argument("--every", type=int, default=1)
    ap.add_argument("--cols", type=int, default=6)
    a = ap.parse_args()
    frames = sorted(glob.glob(os.path.join(a.dir, "f*.png")))[:: a.every]
    if not frames:
        raise SystemExit(f"no frames in {a.dir}")
    x0, y0, x1, y1 = a.box
    w, h = (x1 - x0) * a.scale, (y1 - y0) * a.scale
    label = 18
    cols = min(a.cols, len(frames))
    rows = (len(frames) + cols - 1) // cols
    sheet = Image.new("RGB", (cols * w, rows * (h + label)), (24, 24, 28))
    draw = ImageDraw.Draw(sheet)
    for k, path in enumerate(frames):
        crop = Image.open(path).convert("RGB").crop((x0, y0, x1, y1))
        crop = crop.resize((w, h), Image.NEAREST)
        cx, cy = (k % cols) * w, (k // cols) * (h + label)
        sheet.paste(crop, (cx, cy + label))
        draw.text((cx + 4, cy + 3), os.path.basename(path)[:-4], fill=(230, 230, 230))
    sheet.save(a.out, quality=86)
    print(f"{a.out}: {len(frames)} frames, {sheet.size[0]}x{sheet.size[1]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
