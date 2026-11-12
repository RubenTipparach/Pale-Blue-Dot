"""Lay drawn icons beside the friend's own, big and at the game's sizes.

    python3 sheet.py OUT.png drawn1.png drawn2.png ... [--ref "inferno" --ref "plating"]

Each drawn PNG is shown blown up (nearest, so the strokes show), then at 64 and
48; the references (file names under resources/sprites/upgrade icons/, without
.png) close the sheet with "(theirs)" under them. Read the sheet and judge the
drawing against them, not on its own.
"""

import os
import sys

from PIL import Image, ImageDraw, ImageFont

HERE = os.path.dirname(os.path.abspath(__file__))
# The friend's icons: copies beside this skill (references/theirs), else godot-sandbox's own folder.
ICONS = os.path.join(HERE, "..", "references", "theirs")
if not os.path.isdir(ICONS):
    ICONS = os.path.abspath(os.path.join(HERE, "..", "..", "..", "..", "godot-sandbox", "resources", "sprites", "upgrade icons"))
DEFAULT_REFS = ["inferno", "plating", "death aura", "magnetic field", "power output", "eagle eye"]


def main(argv):
    out, rest = argv[0], argv[1:]
    refs, drawn = [], []
    i = 0
    while i < len(rest):
        if rest[i] == "--ref":
            refs.append(rest[i + 1])
            i += 2
        else:
            drawn.append(rest[i])
            i += 1
    refs = refs or DEFAULT_REFS
    items = [(os.path.splitext(os.path.basename(p))[0], p) for p in drawn]
    items += [("(theirs) " + r, os.path.join(ICONS, r + ".png")) for r in refs]
    try:
        font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 13)
    except OSError:
        font = ImageFont.load_default()
    cols, cw, ch = 6, 200, 236
    rows = (len(items) + cols - 1) // cols
    img = Image.new("RGBA", (cols * cw + 20, rows * ch + 20), (36, 40, 50, 255))
    d = ImageDraw.Draw(img)
    for k, (name, path) in enumerate(items):
        if not os.path.exists(path):
            continue
        im = Image.open(path).convert("RGBA")
        x, y = 10 + (k % cols) * cw, 10 + (k // cols) * ch
        img.alpha_composite(im.resize((160, 160), Image.NEAREST), (x + 20, y))
        img.alpha_composite(im.resize((64, 64), Image.LANCZOS), (x + 44, y + 160))
        img.alpha_composite(im.resize((48, 48), Image.LANCZOS), (x + 116, y + 168))
        label = name if len(name) < 26 else name[:25] + "."
        d.text((x + (cw - d.textlength(label, font=font)) / 2, y + 222), label, font=font, fill=(215, 220, 230))
    img.save(out)
    print("wrote", out)


if __name__ == "__main__":
    main(sys.argv[1:])
