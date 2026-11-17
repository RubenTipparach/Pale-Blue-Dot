"""The logo's finish: a dark rim round its silhouette and a soft drop shadow (the owner's game-logo
references, 2026-11-16), then the deliverables: a transparent PNG at full size and 1024 px wide, and
a copy on the game's night blue.

    python tools/art/logo/logo_finish.py <render.png> <out dir>
"""
import os
import sys

import numpy as np
from PIL import Image, ImageFilter

src, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
im = Image.open(src).convert("RGBA")
a = im.getchannel("A")
rim = a.filter(ImageFilter.MaxFilter(15)).filter(ImageFilter.GaussianBlur(1.5))
shadow = a.filter(ImageFilter.MaxFilter(9)).filter(ImageFilter.GaussianBlur(22))
W, H = im.size
pad = 60
canvas = Image.new("RGBA", (W + 2 * pad, H + 2 * pad), (0, 0, 0, 0))
sh = Image.new("RGBA", im.size, (0, 0, 0, 255))
sh.putalpha(shadow.point(lambda v: int(v * 0.75)))
canvas.alpha_composite(sh, (pad + 18, pad + 26))
rimimg = Image.new("RGBA", im.size, (6, 9, 13, 255))
rimimg.putalpha(rim)
canvas.alpha_composite(rimimg, (pad, pad))
canvas.alpha_composite(im, (pad, pad))
bbox = canvas.getchannel("A").getbbox()
canvas = canvas.crop((max(0, bbox[0] - 30), max(0, bbox[1] - 30), min(canvas.width, bbox[2] + 30), min(canvas.height, bbox[3] + 30)))
canvas.save(os.path.join(out, "pale-blue-dot-logo.png"))
small = canvas.resize((1024, round(canvas.height * 1024 / canvas.width)), Image.LANCZOS)
small.save(os.path.join(out, "pale-blue-dot-logo-1024.png"))
bg = Image.new("RGBA", canvas.size, (8, 14, 24, 255))
bg.alpha_composite(canvas)
bg.convert("RGB").save(os.path.join(out, "pale-blue-dot-logo-on-night.jpg"), quality=92)
print("finished", canvas.size)
