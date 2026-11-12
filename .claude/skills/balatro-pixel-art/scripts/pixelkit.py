"""pixelkit: true pixel art drawn in code, at native size, locked to a palette (Balatro's joker look).

Every shape is a boolean mask on the sprite's own grid (no anti-aliasing, ever), filled with a 2 to 5
tone ramp taken from the palette and shaded from the top left; parts drawn over other parts get a dark
seam where they overlap; one pass at the end draws the 1 px outline round the whole silhouette. Glow
(plumes, muzzle flashes, sparks) is drawn last and is never outlined.

    from pixelkit import Sprite, RAMPS, save, sheet
    sp = Sprite(56, 40, flip=False)          # flip=True mirrors every shape (a ship facing left)
    hull = sp.ellipse(4, 10, 50, 24)         # masks are numpy bool arrays: | & ~ combine them
    sp.part(hull, RAMPS['sand'], 'cyl_h')    # shaded fill; seams where it overlaps earlier parts
    sp.outline()                             # 1 px ink round everything that is not glow
    sp.part(sp.taper(...), RAMPS['flame'], 'inset', glow=True)
    save(sp.image(), 'out.png')              # fails if a pixel is off palette or half transparent

PIL and numpy only. The palette is Sweaft-64 by Sweaft (lospec.com/palette-list/sweaft-64); pass
another list of hex colours to `use_palette` for another game.
"""

import numpy as np
from PIL import Image, ImageDraw
from PIL.PngImagePlugin import PngInfo

SWEAFT_64 = [
    '#010e1a', '#011c34', '#1f3b52', '#3e586e', '#5d778d', '#92a3b2', '#bfcad7', '#fffefe',
    '#a09f91', '#434137', '#2c2c25', '#161612', '#010000', '#1c020d', '#4a051e', '#841c1d',
    '#d4462b', '#fe6420', '#fb902f', '#faaf5c', '#003227', '#05582d', '#297537', '#559642',
    '#9bb675', '#0c2f44', '#204f63', '#53998b', '#84e3b5', '#c1f1d8', '#191954', '#15578a',
    '#1096c3', '#74effe', '#d2f8ff', '#230c44', '#4f1d83', '#7d2dbd', '#9e3ba2', '#c34988',
    '#dd576d', '#2b1f35', '#3e3551', '#69608b', '#170916', '#2f1525', '#522b2f', '#7d4636',
    '#a8633e', '#e69d69', '#eab798', '#41322c', '#725244', '#ab8465', '#cfaa81', '#561c27',
    '#733126', '#8d4526', '#c56d25', '#2e0d06', '#681712', '#a61e1f', '#d12d41', '#fa3a64',
]


def rgb(h):
    """'#rrggbb' to an (r, g, b) tuple."""
    return tuple(int(h[i:i + 2], 16) for i in (1, 3, 5))


PALETTE = [rgb(h) for h in SWEAFT_64]
_IN_PALETTE = set(PALETTE)


def use_palette(hexes):
    """Lock the kit to another palette (a list of '#rrggbb')."""
    PALETTE[:] = [rgb(h) for h in hexes]
    _IN_PALETTE.clear()
    _IN_PALETTE.update(PALETTE)


def palette_from_file(path):
    """Every '#rrggbb' in a file (a JS or CSS palette module), or, for a lospec .hex file, one bare
    'rrggbb' a line, in order."""
    import re
    with open(path) as f:
        text = f.read()
    found = re.findall(r'#[0-9a-fA-F]{6}\b', text)
    if not found:
        found = ['#' + h for h in re.findall(r'^\s*([0-9a-fA-F]{6})\s*$', text, re.M)]
    return [h.lower() for h in found]


# Ramps run dark to light; every entry is a Sweaft-64 colour. Three to five tones a material is the
# Balatro budget: a shadow, a body, a light, sometimes a deep shadow and a glint.
RAMPS = {
    'ink': ['#010000', '#161612', '#2c2c25'],
    'metal': ['#1f3b52', '#3e586e', '#5d778d', '#92a3b2', '#bfcad7'],
    'steel': ['#2c2c25', '#434137', '#a09f91', '#bfcad7'],
    'soot': ['#161612', '#2c2c25', '#434137', '#a09f91'],
    'sand': ['#725244', '#ab8465', '#cfaa81', '#eab798'],
    'olive': ['#003227', '#05582d', '#297537', '#9bb675'],
    'khaki': ['#41322c', '#434137', '#a09f91', '#cfaa81'],
    'brass': ['#733126', '#c56d25', '#fb902f', '#faaf5c'],
    'gold': ['#8d4526', '#c56d25', '#fb902f', '#faaf5c', '#fffefe'],
    'rust': ['#2e0d06', '#733126', '#a8633e', '#e69d69'],
    'wood': ['#41322c', '#725244', '#ab8465', '#cfaa81'],
    'bone': ['#a09f91', '#cfaa81', '#eab798', '#fffefe'],
    'paper': ['#92a3b2', '#bfcad7', '#fffefe'],
    'glass': ['#0c2f44', '#204f63', '#53998b', '#84e3b5', '#c1f1d8'],
    'amber': ['#681712', '#c56d25', '#fb902f', '#faaf5c'],
    'cyan': ['#0c2f44', '#15578a', '#1096c3', '#74effe', '#d2f8ff'],
    'violet': ['#230c44', '#4f1d83', '#7d2dbd', '#9e3ba2', '#c34988'],
    'red': ['#4a051e', '#a61e1f', '#d12d41', '#fa3a64'],
    'green': ['#003227', '#297537', '#559642', '#9bb675'],
    'blue': ['#191954', '#15578a', '#1096c3', '#74effe'],
    'flame': ['#a61e1f', '#fe6420', '#fb902f', '#faaf5c', '#fffefe'],
    'night': ['#010e1a', '#011c34', '#1f3b52', '#3e586e'],
}

INK = '#161612'

# A 4x4 ordered (Bayer) matrix, 0..15: the only gradient pixel art allows.
BAYER4 = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]])


def _shift(m, dx, dy):
    """out[y, x] = m[y + dy, x + dx], False off the edge."""
    h, w = m.shape
    out = np.zeros_like(m)
    ys, yd = (slice(dy, h), slice(0, h - dy)) if dy >= 0 else (slice(0, h + dy), slice(-dy, h))
    xs, xd = (slice(dx, w), slice(0, w - dx)) if dx >= 0 else (slice(0, w + dx), slice(-dx, w))
    out[yd, xd] = m[ys, xs]
    return out


def dilate(m, diagonal=False):
    out = m | _shift(m, 1, 0) | _shift(m, -1, 0) | _shift(m, 0, 1) | _shift(m, 0, -1)
    if diagonal:
        out |= _shift(m, 1, 1) | _shift(m, -1, 1) | _shift(m, 1, -1) | _shift(m, -1, -1)
    return out


def erode(m):
    return m & _shift(m, 1, 0) & _shift(m, -1, 0) & _shift(m, 0, 1) & _shift(m, 0, -1)


class Sprite:
    """One sprite at native size. Shapes are given facing right in pixel coordinates (inclusive);
    with flip=True every mask and plot is mirrored, but shading is worked out after the mirror, so the
    light still comes from the top left."""

    def __init__(self, w, h, flip=False):
        self.w, self.h, self.flip = w, h, flip
        self.rgb = np.zeros((h, w, 3), np.uint8)
        self.alpha = np.zeros((h, w), bool)
        self.glow = np.zeros((h, w), bool)

    # ---- masks (never anti-aliased) ------------------------------------------------------------
    def _mask(self, draw):
        im = Image.new('1', (self.w, self.h), 0)
        draw(ImageDraw.Draw(im))
        m = np.array(im, bool)
        return m[:, ::-1].copy() if self.flip else m

    def empty(self):
        return np.zeros((self.h, self.w), bool)

    def poly(self, pts):
        pts = [(round(x), round(y)) for x, y in pts]
        return self._mask(lambda d: d.polygon(pts, fill=1, outline=1))

    def rect(self, x0, y0, x1, y1):
        return self._mask(lambda d: d.rectangle([round(x0), round(y0), round(x1), round(y1)], fill=1))

    def ellipse(self, x0, y0, x1, y1):
        return self._mask(lambda d: d.ellipse([round(x0), round(y0), round(x1), round(y1)], fill=1))

    def circle(self, cx, cy, r):
        return self.ellipse(cx - r, cy - r, cx + r, cy + r)

    def line(self, pts, width=1):
        pts = [(round(x), round(y)) for x, y in pts]
        return self._mask(lambda d: d.line(pts, fill=1, width=width))

    def taper(self, x0, y0, x1, y1, w0, w1=0):
        """A tapered stroke from (x0, y0), w0 wide, to (x1, y1), w1 wide: plumes, booms, blades."""
        dx, dy = x1 - x0, y1 - y0
        n = max((dx * dx + dy * dy) ** 0.5, 1e-6)
        nx, ny = -dy / n, dx / n
        return self.poly([(x0 + nx * w0 / 2, y0 + ny * w0 / 2), (x1 + nx * w1 / 2, y1 + ny * w1 / 2),
                          (x1 - nx * w1 / 2, y1 - ny * w1 / 2), (x0 - nx * w0 / 2, y0 - ny * w0 / 2)])

    def star(self, cx, cy, r_out, r_in, points=5, turn=-90):
        import math
        pts = []
        for i in range(points * 2):
            a = math.radians(turn + i * 180 / points)
            r = r_out if i % 2 == 0 else r_in
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
        return self.poly(pts)

    # ---- painting ------------------------------------------------------------------------------
    def _put(self, m, color, glow=False):
        self.rgb[m] = rgb(color)
        self.alpha[m] = True
        self.glow[m] = glow

    def fill(self, m, color, glow=False):
        """Flat colour, no seam."""
        self._put(m, color, glow)

    def plot(self, x, y, color, glow=False):
        """Single pixels: highlight glints, rivets, eyes. Mirrored with the sprite."""
        x = self.w - 1 - x if self.flip else x
        if 0 <= x < self.w and 0 <= y < self.h:
            m = self.empty()
            m[y, x] = True
            self._put(m, color, glow)

    def plots(self, pts, color, glow=False):
        for x, y in pts:
            self.plot(x, y, color, glow)

    def tones(self, m, ramp, mode='rim', base=None, depth=1):
        """Tone index per pixel of mask m (dark 0 .. light n-1). Light comes from the top left.

        rim     the body tone; the top and left edge one lighter, the bottom and right `depth` px darker
        cyl_h   a cylinder lying along x: bands by height in each column, light at the top
        cyl_v   a cylinder standing up: bands by place in each row, light at the left
        sphere  round: lighter towards the top left, darker to the bottom right
        inset   by distance from the edge: the edge darkest, the core lightest (plumes, gems, glows)
        flat    the body tone
        """
        n = len(ramp)
        base = max(n - 2, 0) if base is None else base
        t = np.full(m.shape, base, int)
        if mode == 'rim':
            shadow = np.zeros_like(m)
            for d in range(1, depth + 1):
                shadow |= ~_shift(m, 0, d) | ~_shift(m, d, 0)
            t[shadow] = max(base - 1, 0)
            lit = ~_shift(m, 0, -1) | ~_shift(m, -1, 0)
            bottom = ~_shift(m, 0, 1)
            t[m & lit & ~bottom] = min(base + 1, n - 1)
        elif mode in ('cyl_h', 'cyl_v'):
            mm = m if mode == 'cyl_h' else m.T
            tt = t if mode == 'cyl_h' else t.T
            cuts = [(0.22, base + 1), (0.62, base), (0.86, base - 1), (1.01, base - 2)]
            for col in range(mm.shape[1]):
                ys = np.nonzero(mm[:, col])[0]
                if not len(ys):
                    continue
                top, bot = ys.min(), ys.max()
                span = max(bot - top, 1)
                for y in ys:
                    f = (y - top) / span
                    for cut, tone in cuts:
                        if f < cut:
                            tt[y, col] = min(max(tone, 0), n - 1)
                            break
        elif mode == 'sphere':
            ys, xs = np.nonzero(m)
            if len(ys):
                cy, cx = (ys.min() + ys.max()) / 2, (xs.min() + xs.max()) / 2
                ry, rx = max((ys.max() - ys.min()) / 2, 1), max((xs.max() - xs.min()) / 2, 1)
                v = -((xs - cx) / rx * 0.6 + (ys - cy) / ry * 0.8)
                tone = np.where(v > 0.55, base + 1, np.where(v > -0.25, base, np.where(v > -0.75, base - 1, base - 2)))
                t[ys, xs] = np.clip(tone, 0, n - 1)
        elif mode == 'inset':
            cur = m.copy()
            level = 0
            t[:] = 0
            while cur.any():
                t[cur] = min(level, n - 1)
                cur = erode(cur)
                level += 1
        return t

    def part(self, m, ramp, mode='rim', base=None, depth=1, seam=True, glow=False, seam_color=None):
        """A shaded part. Where it lies over earlier parts a 1 px seam in the ramp's darkest tone (or
        seam_color) separates them, as Balatro's drawings outline every piece."""
        if seam and not glow:
            ring = dilate(m) & ~m & self.alpha & ~self.glow
            self._put(ring, seam_color or ramp[0])
        t = self.tones(m, ramp, mode, base, depth)
        for i, c in enumerate(ramp):
            self._put(m & (t == i), c, glow)
        return m

    def dither(self, m, c1, c2, amount=0.5, glow=False):
        """Ordered dither: `amount` of m's pixels in c2, the rest c1, in a 4x4 Bayer pattern."""
        ys, xs = np.mgrid[0:self.h, 0:self.w]
        pick = BAYER4[ys % 4, xs % 4] < amount * 16
        self._put(m & ~pick, c1, glow)
        self._put(m & pick, c2, glow)

    def outline(self, color=INK, diagonal=False):
        """The 1 px outline round the silhouette (glow excluded), only on empty pixels."""
        solid = self.alpha & ~self.glow
        ring = dilate(solid, diagonal) & ~self.alpha
        self._put(ring, color)

    # ---- out -----------------------------------------------------------------------------------
    def image(self):
        a = np.where(self.alpha, 255, 0).astype(np.uint8)
        return Image.fromarray(np.dstack([self.rgb, a]), 'RGBA')


def check(img, name='sprite'):
    """Raise if a pixel is half transparent or an opaque pixel is off the palette."""
    px = np.array(img.convert('RGBA'))
    a = px[..., 3]
    bad_a = int(((a != 0) & (a != 255)).sum())
    cols = {tuple(c) for c in px[a == 255][:, :3]}
    off = [c for c in cols if c not in _IN_PALETTE]
    if bad_a or off:
        raise ValueError(f'{name}: {bad_a} half-transparent pixels, off-palette colours {off[:6]}')


def save(img, path, source='pixelkit'):
    check(img, path)
    meta = PngInfo()
    meta.add_text('source', source)
    img.save(path, pnginfo=meta)


def upscale(img, s):
    return img.resize((img.width * s, img.height * s), Image.NEAREST)


def sheet(items, scale=4, cols=8, pad=6, bg='#1f3b52', label=True, cell=None):
    """A contact sheet: (label, image) pairs, each blown up `scale` times nearest neighbour."""
    cw = cell[0] if cell else max(im.width for _, im in items) * scale
    ch = cell[1] if cell else max(im.height for _, im in items) * scale
    lh = 12 if label else 0
    rows = (len(items) + cols - 1) // cols
    out = Image.new('RGB', (cols * (cw + pad) + pad, rows * (ch + lh + pad) + pad), rgb(bg))
    d = ImageDraw.Draw(out)
    for i, (name, im) in enumerate(items):
        x = pad + (i % cols) * (cw + pad)
        y = pad + (i // cols) * (ch + lh + pad)
        big = upscale(im, scale)
        out.paste(big, (x + (cw - big.width) // 2, y + (ch - big.height) // 2), big if big.mode == 'RGBA' else None)
        if label:
            d.text((x, y + ch + 1), name, fill=(220, 220, 220))
    return out
