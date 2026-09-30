#!/usr/bin/env python3
"""Lay captures side by side in one labelled JPEG: a row a view, a column a
variant (before, after, the mockup), so a change is judged at a glance.

Usage:
  tools/contact_sheet.py OUT.jpg --cols 3 [--width 480] LABEL=PATH ...

Cells fill row by row. A PATH of `-` leaves the cell blank (a shot that was
not taken), and says so in the cell.
"""
import argparse
import sys

from PIL import Image, ImageDraw, ImageFont


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("out")
    ap.add_argument("cells", nargs="+", help="LABEL=PATH")
    ap.add_argument("--cols", type=int, default=3)
    ap.add_argument("--width", type=int, default=480)
    ap.add_argument("--quality", type=int, default=84)
    a = ap.parse_args()
    cells = []
    for c in a.cells:
        label, _, path = c.partition("=")
        cells.append((label, path))
    w = a.width
    h = w * 900 // 1440
    band = 22
    rows = (len(cells) + a.cols - 1) // a.cols
    sheet = Image.new("RGB", (a.cols * w, rows * (h + band)), (24, 24, 28))
    draw = ImageDraw.Draw(sheet)
    try:
        font = ImageFont.truetype("DejaVuSans.ttf", 14)
    except OSError:
        font = ImageFont.load_default()
    for k, (label, path) in enumerate(cells):
        x = (k % a.cols) * w
        y = (k // a.cols) * (h + band)
        if path != "-":
            img = Image.open(path).convert("RGB").resize((w, h), Image.LANCZOS)
            sheet.paste(img, (x, y + band))
        else:
            draw.text((x + 10, y + band + h // 2), "not taken", fill=(150, 150, 150), font=font)
        draw.text((x + 6, y + 3), label, fill=(235, 235, 235), font=font)
    sheet.save(a.out, quality=a.quality)
    print(f"{a.out}: {len(cells)} cells, {sheet.size[0]}x{sheet.size[1]}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
