"""The logo's emblem alone: the crystal ball in its stone ring, cut round from the render (owner,
2026-11-17: "the loading logo should just have the globe part with the circle stone. no need to put
pale blue dot there"). For the loading screen (`assets/ui/logo-emblem.png`, 512 px) and the promo
folder (full size).

    python3 tools/art/logo/logo_emblem.py <render.png> <promo dir> <assets/ui dir>
"""
import os
import sys

import numpy as np
from PIL import Image, ImageFilter

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from logo_height import EMBLEM  # noqa: E402

render, promo, ui = sys.argv[1:4]
im = Image.open(render).convert("RGBA")
cx, cy, r = EMBLEM
R = r + 9                                    # the ring's rough edge, and none of the tablet behind it
box = (cx - R, cy - R, cx + R, cy + R)
crop = im.crop(box)
yy, xx = np.mgrid[0:2 * R, 0:2 * R]
d = np.hypot(xx - R, yy - R)
# keep the ring's own alpha inside the circle; fade the edge over a few pixels so no tablet shows
circle = np.clip((R - d) / 6.0, 0, 1)
a = np.asarray(crop.getchannel("A"), np.float32) / 255 * circle
crop.putalpha(Image.fromarray((a * 255).astype(np.uint8)))
# a soft drop shadow, as the finished logo has
pad = 40
out = Image.new("RGBA", (2 * R + 2 * pad, 2 * R + 2 * pad), (0, 0, 0, 0))
sh = Image.new("RGBA", crop.size, (0, 0, 0, 255))
sh.putalpha(crop.getchannel("A").filter(ImageFilter.GaussianBlur(16)).point(lambda v: int(v * 0.7)))
out.alpha_composite(sh, (pad + 10, pad + 16))
out.alpha_composite(crop, (pad, pad))
out.save(os.path.join(promo, "pale-blue-dot-emblem.png"))
out.resize((512, 512), Image.LANCZOS).save(os.path.join(ui, "logo-emblem.png"))
print("emblem", out.size)
