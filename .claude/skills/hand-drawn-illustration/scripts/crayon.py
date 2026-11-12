"""A crayon and marker drawing kit for the upgrade icons (PIL only, no numpy).

The game's own upgrade icons (resources/sprites/upgrade icons/) are drawn by hand:
128 x 128, a transparent ground, no frame, flat colour laid in streaky marker
strokes with a darker and a lighter shade on top, a wobbly dark ink outline, and
sparks, dots and speed lines round the subject. This kit paints in that manner so
the Prototype Armory's icons sit beside them: every shape is a polygon that is
wobbled, filled with streaks and outlined in ink.

Work is done on a 512 canvas (S) and scaled down to 128 at the end, the way a
drawing made large and shrunk keeps its line weight.

    ic = Icon(seed=3)
    body = circle(256, 256, 150)
    ic.fill(body, (90, 160, 60), angle=30)
    ic.shade(body, (60, 110, 40), (1, 1))       # darker toward the lower right
    ic.shade(body, (150, 210, 110), (-1, -1), 0.45)  # a highlight up left
    ic.outline(body)
    ic.save("x.png")
"""

import math
import random
from PIL import Image, ImageDraw, ImageFilter

S = 512
OUT = 128
INK = (24, 20, 28)


# ---------- geometry ---------------------------------------------------------------

def circle(cx, cy, r, n=56, start=0.0):
    return [(cx + r * math.cos(start + 2 * math.pi * i / n), cy + r * math.sin(start + 2 * math.pi * i / n)) for i in range(n)]


def ellipse(cx, cy, rx, ry, n=56, rot=0.0):
    pts = [(rx * math.cos(2 * math.pi * i / n), ry * math.sin(2 * math.pi * i / n)) for i in range(n)]
    return translate(rotate(pts, rot), cx, cy)


def arc(cx, cy, r, a0, a1, n=40):
    return [(cx + r * math.cos(a0 + (a1 - a0) * i / (n - 1)), cy + r * math.sin(a0 + (a1 - a0) * i / (n - 1))) for i in range(n)]


def ring(cx, cy, r_out, r_in, n=56):
    """A ring as one polygon (out round, a hairline cut, back round inside)."""
    a = circle(cx, cy, r_out, n)
    b = circle(cx, cy, r_in, n)
    return a + [a[0], b[0]] + b[::-1] + [b[0]]


def rect(x0, y0, x1, y1):
    return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]


def roundrect(x0, y0, x1, y1, r, n=8):
    pts = []
    for cx, cy, a0 in ((x1 - r, y0 + r, -math.pi / 2), (x1 - r, y1 - r, 0), (x0 + r, y1 - r, math.pi / 2), (x0 + r, y0 + r, math.pi)):
        for i in range(n + 1):
            a = a0 + (math.pi / 2) * i / n
            pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def star(cx, cy, r_out, r_in, points=5, rot=-math.pi / 2, jag=0.0, seed=0):
    rnd = random.Random(seed)
    pts = []
    for i in range(points * 2):
        r = r_out if i % 2 == 0 else r_in
        if jag:
            r *= 1.0 + rnd.uniform(-jag, jag)
        a = rot + math.pi * i / points
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def rotate(pts, ang, c=(0.0, 0.0)):
    ca, sa = math.cos(ang), math.sin(ang)
    return [(c[0] + (x - c[0]) * ca - (y - c[1]) * sa, c[1] + (x - c[0]) * sa + (y - c[1]) * ca) for x, y in pts]


def translate(pts, dx, dy):
    return [(x + dx, y + dy) for x, y in pts]


def scale(pts, k, c=(256.0, 256.0)):
    return [(c[0] + (x - c[0]) * k, c[1] + (y - c[1]) * k) for x, y in pts]


def centroid(pts):
    n = len(pts)
    return (sum(p[0] for p in pts) / n, sum(p[1] for p in pts) / n)


def mix(a, b, t):
    return tuple(int(round(a[i] + (b[i] - a[i]) * t)) for i in range(3))


def darker(c, t=0.35):
    return mix(c, (10, 8, 20), t)


def lighter(c, t=0.4):
    return mix(c, (255, 252, 235), t)


def tone(c, t=0.55):
    """The edge colour the hand-drawn icons use: a deep tone of the fill's own
    hue, not black (the shield's navy rim, the fireball's burnt orange)."""
    return mix(c, (6, 4, 16), t)


# ---------- the drawing --------------------------------------------------------------

class Icon:
    def __init__(self, seed=1):
        self.rnd = random.Random(seed)
        self.img = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        # Halos live on their own layer, under everything, and skip the edge
        # crisping in final(): crisped, a soft glow turns into a hard disc.
        self.halo = Image.new("RGBA", (S, S), (0, 0, 0, 0))

    # The hand's tremor: a smooth offset along the line's normal.
    def _wobble(self, pts, closed=True, amp=3.0, step=7.0):
        if len(pts) < 2:
            return pts
        seq = list(pts) + ([pts[0]] if closed else [])
        out = []
        ph = [self.rnd.uniform(0, 6.28) for _ in range(3)]
        fr = [self.rnd.uniform(0.010, 0.020), self.rnd.uniform(0.03, 0.05), self.rnd.uniform(0.07, 0.11)]
        dist = 0.0
        for i in range(len(seq) - 1):
            (x0, y0), (x1, y1) = seq[i], seq[i + 1]
            seg = math.hypot(x1 - x0, y1 - y0)
            n = max(1, int(seg / step))
            nx, ny = (-(y1 - y0) / seg, (x1 - x0) / seg) if seg > 0 else (0, 0)
            for k in range(n):
                t = k / n
                d = dist + seg * t
                w = amp * (0.6 * math.sin(d * fr[0] + ph[0]) + 0.3 * math.sin(d * fr[1] + ph[1]) + 0.15 * math.sin(d * fr[2] + ph[2]))
                out.append((x0 + (x1 - x0) * t + nx * w, y0 + (y1 - y0) * t + ny * w))
            dist += seg
        if not closed:
            out.append(seq[-1])
        return out

    def _mask(self, poly):
        m = Image.new("L", (S, S), 0)
        ImageDraw.Draw(m).polygon([(round(x), round(y)) for x, y in poly], fill=255)
        return m

    def _streaks(self, color, angle, width, var, bbox):
        """A layer of marker strokes across bbox at `angle` degrees, each a shade
        of `color` within `var`."""
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        d = ImageDraw.Draw(layer)
        x0, y0, x1, y1 = bbox
        cx, cy = (x0 + x1) / 2, (y0 + y1) / 2
        half = math.hypot(x1 - x0, y1 - y0) / 2 + width
        a = math.radians(angle)
        ux, uy = math.cos(a), math.sin(a)
        vx, vy = -uy, ux
        off = -half
        while off < half:
            t = self.rnd.uniform(-var, var)
            c = mix(color, (255, 255, 240), t) if t > 0 else mix(color, (0, 0, 10), -t)
            w = max(2, int(width * self.rnd.uniform(0.7, 1.25)))
            px, py = cx + vx * off, cy + vy * off
            line = [(px - ux * half, py - uy * half), (px + ux * half, py + uy * half)]
            line = self._wobble(line, closed=False, amp=width * 0.35, step=10)
            d.line(line, fill=c + (255,), width=w, joint="curve")
            off += width * self.rnd.uniform(0.55, 0.85)
        return layer

    def fill(self, poly, color, angle=None, width=9, var=0.10, wobble=2.5):
        poly = self._wobble(poly, amp=wobble)
        m = self._mask(poly)
        if angle is None:
            angle = self.rnd.uniform(20, 70)
        base = Image.new("RGBA", (S, S), color + (255,))
        self.img.paste(base, (0, 0), m)
        bbox = m.getbbox() or (0, 0, S, S)
        layer = self._streaks(color, angle, width, var, bbox)
        self._comp(layer, m)
        self._last = (poly, m)
        return poly

    def shade(self, poly, color, direction, amount=0.5, width=8, angle=None, var=0.06):
        """Streaks of `color` over the part of `poly` lying toward `direction`:
        `amount` 0 to 1 is how far across the shape it reaches."""
        m = self._mask(self._wobble(poly, amp=2.0))
        cx, cy = centroid(poly)
        dx, dy = direction
        dl = math.hypot(dx, dy) or 1.0
        dx, dy = dx / dl, dy / dl
        ext = max(math.hypot(x - cx, y - cy) for x, y in poly)
        cut = ext * (1.0 - 2.0 * amount)
        # a band from the cut line outwards, wobbled so the edge is drawn, not ruled
        px, py = -dy, dx
        mx, my = cx + dx * cut, cy + dy * cut
        far = 2 * S
        half = [(mx - px * far, my - py * far), (mx + px * far, my + py * far),
                (mx + px * far + dx * far, my + py * far + dy * far), (mx - px * far + dx * far, my - py * far + dy * far)]
        h = self._mask(self._wobble(half, amp=6, step=12))
        from PIL import ImageChops
        mm = ImageChops.multiply(m, h)
        if angle is None:
            angle = math.degrees(math.atan2(dy, dx)) + 90 + self.rnd.uniform(-15, 15)
        bbox = mm.getbbox()
        if not bbox:
            return
        layer = self._streaks(color, angle, width, var, bbox)
        self._comp(layer, mm)

    def _comp(self, layer, mask):
        a = layer.getchannel("A")
        from PIL import ImageChops
        layer.putalpha(ImageChops.multiply(a, mask))
        self.img.alpha_composite(layer)

    def outline(self, poly, color=INK, width=10, closed=True, amp=2.5, passes=2):
        d = ImageDraw.Draw(self.img)
        for p in range(passes):
            line = self._wobble(poly, closed=closed, amp=amp + p)
            if closed:
                line = line + [line[0]]
            d.line(line, fill=color + (255,), width=max(2, width - p * 3), joint="curve")
            for x, y in (line[0], line[-1]):
                r = max(1, (width - p * 3) // 2)
                d.ellipse((x - r, y - r, x + r, y + r), fill=color + (255,))

    def shape(self, poly, color, light=(-1, -1), dark_amt=0.45, light_amt=0.30, ink=INK, ink_w=10, angle=None, width=9):
        """The usual: a filled shape, shaded dark away from the light and lit
        toward it, inked round."""
        self.fill(poly, color, angle=angle, width=width)
        self.shade(poly, darker(color, 0.35), (-light[0], -light[1]), dark_amt)
        if light_amt > 0:
            self.shade(poly, lighter(color, 0.35), light, light_amt, width=6)
        if ink:
            self.outline(poly, ink, ink_w)

    # ---------- the friend's own method (see SKILL.md, "How a drawing is built") -----

    def flat(self, poly, color, wobble=2.5):
        """Stage 1: the silhouette blocked in one flat mid tone."""
        poly = self._wobble(poly, amp=wobble)
        m = self._mask(poly)
        self.img.paste(Image.new("RGBA", (S, S), color + (255,)), (0, 0), m)
        return poly

    def scribble(self, poly, color, var=0.16, width=6, length=(22, 64), angle=None,
                 spread=35, density=1.0, mask_poly=None):
        """Stage 3: loose hatching, short strokes laid at about `angle` (+/- spread
        degrees) in lighter and darker versions of `color`, clipped to the shape.
        Layer it over a flat() block, more than once with different angles."""
        m = self._mask(mask_poly or poly)
        bbox = m.getbbox()
        if not bbox:
            return
        if angle is None:
            angle = self.rnd.uniform(15, 75)
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        d = ImageDraw.Draw(layer)
        x0, y0, x1, y1 = bbox
        n = int((x1 - x0) * (y1 - y0) / (width * 26) * density)
        for _ in range(n):
            cx, cy = self.rnd.uniform(x0, x1), self.rnd.uniform(y0, y1)
            a = math.radians(angle + self.rnd.uniform(-spread, spread))
            L = self.rnd.uniform(*length) / 2
            t = self.rnd.uniform(-var, var)
            c = mix(color, (255, 255, 240), t) if t > 0 else mix(color, (0, 0, 10), -t)
            seg = [(cx - math.cos(a) * L, cy - math.sin(a) * L), (cx + math.cos(a) * L, cy + math.sin(a) * L)]
            d.line(self._wobble(seg, closed=False, amp=width * 0.4, step=8), fill=c + (235,),
                   width=max(2, int(width * self.rnd.uniform(0.6, 1.3))), joint="curve")
        self._comp(layer, m)

    def dabs(self, poly, color, direction=(-1, -1), n=7, size=16, reach=0.42):
        """Stage 4: highlight dabs, short fat light strokes on the lit side."""
        cx, cy = centroid(poly)
        ext = max(math.hypot(x - cx, y - cy) for x, y in poly)
        dl = math.hypot(*direction) or 1.0
        dx, dy = direction[0] / dl, direction[1] / dl
        m = self._mask(poly)
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        d = ImageDraw.Draw(layer)
        for _ in range(n):
            r = ext * self.rnd.uniform(reach * 0.5, reach * 1.3)
            side = self.rnd.uniform(-0.6, 0.6)
            px = cx + dx * r - dy * side * ext * 0.5
            py = cy + dy * r + dx * side * ext * 0.5
            a = math.atan2(dy, dx) + math.pi / 2 + self.rnd.uniform(-0.4, 0.4)
            L = size * self.rnd.uniform(0.8, 1.8)
            seg = [(px - math.cos(a) * L, py - math.sin(a) * L), (px + math.cos(a) * L, py + math.sin(a) * L)]
            d.line(self._wobble(seg, closed=False, amp=2, step=6), fill=color + (230,),
                   width=max(3, int(size * self.rnd.uniform(0.35, 0.6))), joint="curve")
        self._comp(layer, m)

    def rim(self, poly, color, width=7, inset=7):
        """A bright line just inside the edge (the heart's glossy rim)."""
        c = centroid(poly)
        ext = max(math.hypot(x - c[0], y - c[1]) for x, y in poly) or 1.0
        inner = scale(poly, 1.0 - inset / ext, c)
        m = self._mask(poly)
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        line = self._wobble(inner, amp=1.5)
        ImageDraw.Draw(layer).line(line + [line[0]], fill=color + (255,), width=width, joint="curve")
        self._comp(layer, m)

    def crescent(self, poly, direction, amount=0.18):
        """The part of `poly` lying toward `direction`, cut by the shape's own
        outline shifted the other way: a crescent that follows the form, the way
        a hand shades a heart or a shield (never a straight band across it)."""
        c = centroid(poly)
        ext = max(math.hypot(x - c[0], y - c[1]) for x, y in poly) or 1.0
        dl = math.hypot(*direction) or 1.0
        dx, dy = direction[0] / dl * ext * amount, direction[1] / dl * ext * amount
        from PIL import ImageChops
        m = self._mask(poly)
        shifted = self._mask(self._wobble(translate(poly, -dx, -dy), amp=4, step=10))
        return ImageChops.subtract(m, shifted)

    def scribble_in(self, mask, color, var=0.16, width=6, angle=None, density=1.0, length=(22, 64), spread=35):
        """scribble() clipped to a mask (from crescent()) rather than a polygon."""
        bbox = mask.getbbox()
        if not bbox:
            return
        if angle is None:
            angle = self.rnd.uniform(15, 75)
        layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
        d = ImageDraw.Draw(layer)
        x0, y0, x1, y1 = bbox
        n = int((x1 - x0) * (y1 - y0) / (width * 26) * density)
        for _ in range(n):
            cx, cy = self.rnd.uniform(x0, x1), self.rnd.uniform(y0, y1)
            a = math.radians(angle + self.rnd.uniform(-spread, spread))
            L = self.rnd.uniform(*length) / 2
            t = self.rnd.uniform(-var, var)
            c = mix(color, (255, 255, 240), t) if t > 0 else mix(color, (0, 0, 10), -t)
            seg = [(cx - math.cos(a) * L, cy - math.sin(a) * L), (cx + math.cos(a) * L, cy + math.sin(a) * L)]
            d.line(self._wobble(seg, closed=False, amp=width * 0.4, step=8), fill=c + (235,),
                   width=max(2, int(width * self.rnd.uniform(0.6, 1.3))), joint="curve")
        self._comp(layer, mask)

    def drawn(self, poly, color, light=(-1, -1), hatch=1.0, shade=0.22, highlight=0.16,
              rim_color=None, edge="tone", edge_w=8, angle=None):
        """One shape the way the friend draws it, start to finish (SKILL.md):
        a flat block in a saturated mid tone, bold hatching in its own lighter
        and darker tones, a darker scribbled crescent on the shadow side and a
        light one toward the light (both follow the outline), highlight dabs,
        an optional bright rim just inside the edge, and an edge in a deep tone
        of the same hue (edge=INK for black, None for none)."""
        poly = self.flat(poly, color)
        a = angle if angle is not None else self.rnd.uniform(20, 70)
        # Long strokes laid mostly one way, as a hand fills a shape; a lighter
        # cross pass on top. Short jittery strokes read as noise, not hatching.
        self.scribble(poly, color, var=0.26, angle=a, density=hatch * 0.8, length=(40, 110), spread=14)
        self.scribble(poly, color, var=0.18, angle=a + 60, density=hatch * 0.3, length=(30, 80), spread=18)
        if shade:
            self.scribble_in(self.crescent(poly, (-light[0], -light[1]), shade), darker(color, 0.35),
                             var=0.12, angle=a, density=1.4, length=(36, 90), spread=14)
        if highlight:
            self.scribble_in(self.crescent(poly, light, highlight), lighter(color, 0.45),
                             var=0.10, angle=a + 20, density=1.2, width=5, length=(30, 80), spread=14)
            self.dabs(poly, lighter(color, 0.6), light, n=4, size=12)
        if rim_color:
            self.rim(poly, rim_color)
        if edge:
            self.outline(poly, tone(color) if edge == "tone" else edge, edge_w)
        return poly

    def line(self, pts, color, width=8, amp=2.0, ink=None, ink_w=None):
        """A drawn line (a beam, a rope, a speed line), optionally inked under."""
        d = ImageDraw.Draw(self.img)
        w = self._wobble(pts, closed=False, amp=amp)
        if ink:
            d.line(w, fill=ink + (255,), width=width + (ink_w or 8), joint="curve")
        d.line(w, fill=color + (255,), width=width, joint="curve")
        r = width // 2
        for x, y in (w[0], w[-1]):
            d.ellipse((x - r, y - r, x + r, y + r), fill=color + (255,))

    def dot(self, x, y, r, color, ink=None):
        d = ImageDraw.Draw(self.img)
        if ink:
            d.ellipse((x - r - 4, y - r - 4, x + r + 4, y + r + 4), fill=ink + (255,))
        d.ellipse((x - r, y - r, x + r, y + r), fill=color + (255,))

    def sparkle(self, x, y, r, color=(255, 236, 120), ink=INK):
        """A four point twinkle, the kind round the mineral and cryo icons."""
        p = star(x, y, r, r * 0.28, 4, rot=-math.pi / 2)
        self.fill(p, color, width=5, wobble=1)
        if ink:
            self.outline(p, ink, 6, amp=1)

    def speed(self, x, y, length, angle_deg, color=(240, 240, 240), width=7, ink=None):
        a = math.radians(angle_deg)
        self.line([(x, y), (x + math.cos(a) * length, y + math.sin(a) * length)], color, width, amp=1.5, ink=ink, ink_w=6)

    def zap(self, pts, color=(255, 236, 90), width=9, jag=16, seed=0):
        """A lightning stroke along pts, jagged."""
        rnd = random.Random(seed)
        out = []
        for i in range(len(pts) - 1):
            (x0, y0), (x1, y1) = pts[i], pts[i + 1]
            n = max(2, int(math.hypot(x1 - x0, y1 - y0) / 28))
            for k in range(n):
                t = k / n
                out.append((x0 + (x1 - x0) * t + rnd.uniform(-jag, jag), y0 + (y1 - y0) * t + rnd.uniform(-jag, jag)))
        out.append(pts[-1])
        d = ImageDraw.Draw(self.img)
        d.line(out, fill=INK + (255,), width=width + 8, joint="curve")
        d.line(out, fill=color + (255,), width=width, joint="curve")
        d.line(out, fill=lighter(color, 0.6) + (255,), width=max(2, width // 3), joint="curve")

    def glow(self, poly_or_center, color, radius=30, strength=0.55):
        """A soft halo under the whole drawing (it is kept on its own layer)."""
        m = Image.new("L", (S, S), 0)
        dr = ImageDraw.Draw(m)
        if isinstance(poly_or_center, tuple):
            x, y, r = poly_or_center
            dr.ellipse((x - r, y - r, x + r, y + r), fill=int(255 * strength))
        else:
            dr.polygon(poly_or_center, fill=int(255 * strength))
        m = m.filter(ImageFilter.GaussianBlur(radius))
        layer = Image.new("RGBA", (S, S), color + (0,))
        layer.putalpha(m)
        self.halo.alpha_composite(layer)

    def final(self):
        im = self.img.resize((OUT, OUT), Image.LANCZOS)
        # Crisp the edge the way a drawn icon's is: a soft alpha fringe at 128
        # reads as blur, not as a hand-drawn line.
        a = im.getchannel("A").point(lambda v: 0 if v < 40 else (255 if v > 200 else v))
        im.putalpha(a)
        halo = self.halo.resize((OUT, OUT), Image.LANCZOS)
        # A halo cut off by the canvas reads as a square tile: fade it out over
        # the outer 18 pixels so it never reaches the edge.
        from PIL import ImageChops
        fade = Image.new("L", (OUT, OUT), 0)
        fd = ImageDraw.Draw(fade)
        band = 18
        for i in range(band):
            v = int(255 * (i / band) ** 1.5)
            fd.rectangle((i, i, OUT - 1 - i, OUT - 1 - i), outline=v)
        fd.rectangle((band, band, OUT - 1 - band, OUT - 1 - band), fill=255)
        halo.putalpha(ImageChops.multiply(halo.getchannel("A"), fade))
        halo.alpha_composite(im)
        return halo

    def save(self, path):
        from PIL.PngImagePlugin import PngInfo
        meta = PngInfo()
        # Drawn by this kit, not by hand: tools/armory_art/build.py makes it again.
        meta.add_text("source", "armory-crayon")
        self.final().save(path, pnginfo=meta)
