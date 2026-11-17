"""The logo on deep space (and on nothing, transparent), for itch.io's cover (630 x 500) and a large copy five times that
(owner, 2026-11-16: "I need a version with space background for itch io, make a large version and
check that in too").

The space is drawn here: a dark blue-black field, a faint nebula in the game's violet and blue, and
stars of every size, the bright ones with a soft glow and a few with a four-point glint, so it sits
with the crystal ball's own starfield. The logo is the finished transparent one (rim and shadow).

    <venv with numpy, pillow, scipy>/bin/python tools/art/logo/logo_cover.py <logo.png> <out dir>
"""
import os
import sys

import numpy as np
from PIL import Image
from scipy import ndimage

logo_png, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
W, H = 3150, 2500                         # 5 x itch.io's 630 x 500
rng = np.random.default_rng(2026)

# the field: darker at the edges
yy, xx = np.mgrid[0:H, 0:W].astype(np.float32)
r = np.hypot((xx - W / 2) / W, (yy - H / 2) / H)
img = np.zeros((H, W, 3), np.float32) + np.array([0.012, 0.018, 0.035])
img *= (1.15 - r * 0.9)[..., None]


def cloud(scale, seed):
    g = np.random.default_rng(seed).random((H // scale + 2, W // scale + 2)).astype(np.float32)
    g = ndimage.gaussian_filter(g, 1.6)
    return ndimage.zoom(g, scale, order=3)[:H, :W]


# the nebula: two layers of soft cloud, violet and blue, brightest in a band across the field
n1 = cloud(160, 1) * 0.6 + cloud(60, 2) * 0.3 + cloud(20, 3) * 0.1
n2 = cloud(220, 4) * 0.7 + cloud(80, 5) * 0.3
band = np.exp(-((yy - H * 0.62 + (xx - W / 2) * 0.35) / (H * 0.28)) ** 2)
neb1 = np.clip((n1 - 0.47) * 3.2, 0, 1) ** 1.6 * band
neb2 = np.clip((n2 - 0.5) * 3.0, 0, 1) ** 1.8 * (0.4 + 0.6 * band)
img += neb1[..., None] * np.array([0.16, 0.06, 0.26]) + neb2[..., None] * np.array([0.03, 0.10, 0.24])

# stars: many faint points, fewer bright ones with a glow, a handful with glints
stars = np.zeros((H, W), np.float32)
n = 5200
xs, ys = rng.integers(0, W, n), rng.integers(0, H, n)
mag = rng.power(6, n)[::-1] if False else rng.random(n) ** 3.2
stars[ys, xs] = mag
glow = ndimage.gaussian_filter(stars * (stars > 0.35), 3.0) * 30 + ndimage.gaussian_filter(stars, 0.8) * 6
tint = np.where(rng.random((H, W)) < 0.5, 1.0, 0.0)
col = np.stack([0.85 + 0.15 * tint, 0.9 * np.ones_like(tint), 1.0 - 0.1 * tint], -1)
img += glow[..., None] * col * 0.6
# a few bright glints: a soft core and four thin rays
for _ in range(14):
    cx, cy = rng.integers(60, W - 60), rng.integers(60, H - 60)
    s = rng.uniform(0.5, 1.0)
    L = int(rng.uniform(40, 110))
    dx = np.abs(xx[cy - L:cy + L, cx - L:cx + L] - cx)
    dy = np.abs(yy[cy - L:cy + L, cx - L:cx + L] - cy)
    ray = np.exp(-dy / 1.2) * np.exp(-dx / (L * 0.35)) + np.exp(-dx / 1.2) * np.exp(-dy / (L * 0.35))
    core = np.exp(-(dx ** 2 + dy ** 2) / 18.0) * 1.4
    img[cy - L:cy + L, cx - L:cx + L] += ((ray * 0.6 + core) * s)[..., None] * np.array([0.9, 0.95, 1.0])

bg = Image.fromarray((np.clip(img, 0, 1) ** (1 / 1.1) * 255).astype(np.uint8)).convert("RGBA")

logo = Image.open(logo_png).convert("RGBA")
scale = min(W * 0.92 / logo.width, H * 0.86 / logo.height)
lw, lh = int(logo.width * scale), int(logo.height * scale)
logo = logo.resize((lw, lh), Image.LANCZOS)
bg.alpha_composite(logo, ((W - lw) // 2, (H - lh) // 2 + int(H * 0.01)))

# the same layout with nothing behind it (owner: "and need a transparent version")
clear = Image.new("RGBA", (W, H), (0, 0, 0, 0))
clear.alpha_composite(logo, ((W - lw) // 2, (H - lh) // 2 + int(H * 0.01)))
clear.save(os.path.join(out, "pale-blue-dot-cover-transparent.png"), optimize=True)
clear.resize((630, 500), Image.LANCZOS).save(os.path.join(out, "pale-blue-dot-cover-transparent-630x500.png"), optimize=True)

big = bg.convert("RGB")
big.save(os.path.join(out, "pale-blue-dot-cover-space.png"), optimize=True)
big.resize((630, 500), Image.LANCZOS).save(os.path.join(out, "pale-blue-dot-cover-space-630x500.png"), optimize=True)
print("cover", big.size)
