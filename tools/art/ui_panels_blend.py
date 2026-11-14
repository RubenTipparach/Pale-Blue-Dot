"""The stone-and-wood UI kit modelled and rendered in Blender (openspec/changes/stone-and-wood-ui,
design.md section 7): the "Rendered" and "Smooth" looks beside tools/art/ui_art.py's drawn one.

The base rock (approved by the owner, 2026-11-14, "Ok good continue with the mockups") is the first
render's rock_field() - its relief, colours, silhouette, light and camera - with two changes only:
the strata rings are removed, and each in-play panel's rock is rendered at that panel's own size
(ROCK_PANELS), so its rim and lip shadow never tile. Every other piece (stone plaques, slots, tabs,
bars, scroll, slider, checkboxes, the compass strip, frames, the oak set for the timber menus) is one
height field with material masks at the drawn piece's size, rendered under the same light with
rock_field's surface on its stone. No outline in either look.

Outputs, in docs/mockups/ui/:
  rendered/<piece>.png, rendered/oak/<piece>.png   pixelated to the drawn sizes and Sweaft-64 ramps
  rendered/hires/..., rendered/hires/oak/...       the 8x renders, the Smooth look
  rendered/rock/rock-WxH.png, rendered/hires/rock/  the rock per panel (pixelated; 4x smooth)
  rendered/slices.json                             sizes, rock_panels, contrast in both looks, timings
  blender-frame*.png, blender-rock*.png            the first two renders (--legacy), kept as the record

Needs the bpy module (Blender 4.5 as a Python wheel, Python 3.11) with numpy and Pillow:
    python3.11 -m venv /tmp/bpy && /tmp/bpy/bin/pip install "bpy==4.5.*" pillow numpy
    /tmp/bpy/bin/python tools/art/ui_panels_blend.py [--samples 24]          # everything (~4 min, 4 CPU cores)
    /tmp/bpy/bin/python tools/art/ui_panels_blend.py --only rock --rocks pack  # one rock panel
    /tmp/bpy/bin/python tools/art/ui_panels_blend.py --only button-idle,oak/frame
In a cloud container this runs on the CPU (no GPU there); on the owner's desktop use the GPU.
"""

import json
import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
OUT = os.path.join(ROOT, "docs", "mockups", "ui")
sys.path.insert(0, os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art", "scripts"))
from pixelkit import SWEAFT_64, rgb, save  # noqa: E402

sys.path.insert(0, HERE)
import ui_art  # noqa: E402

KIT_COLOURS = sorted({c for r in (ui_art.STONE, ui_art.ROCK, ui_art.WOOD, ui_art.IRON) for c in r}
                     | {ui_art.SEAM, ui_art.FACE, ui_art.WELL})
assert all(c in SWEAFT_64 for c in KIT_COLOURS)

import bpy  # noqa: E402
from PIL import Image  # noqa: E402

N = 48  # native size, px


def arg(name, default):
    return type(default)(sys.argv[sys.argv.index(name) + 1]) if name in sys.argv else default


SAMPLES = arg("--samples", 24)
PX = arg("--px", 10)  # render pixels per native pixel
RES = N * PX


def srgb_to_lin(c):
    c = np.asarray(c, float) / 255.0
    return np.where(c <= 0.04045, c / 12.92, ((c + 0.055) / 1.055) ** 2.4)


def hexlin(h):
    return srgb_to_lin(rgb(h))


def fbm(x, y, seed, octaves=4, scale=1.0):
    """Value-noise fbm on arrays x, y (smooth, deterministic)."""
    rng = np.random.default_rng(seed)
    total = np.zeros_like(x, float)
    amp, freq = 1.0, scale
    for _ in range(octaves):
        g = rng.random((64, 64))
        xi, yi = x * freq, y * freq
        x0, y0 = np.floor(xi).astype(int), np.floor(yi).astype(int)
        fx, fy = xi - x0, yi - y0
        fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
        a = g[y0 % 64, x0 % 64]
        b = g[y0 % 64, (x0 + 1) % 64]
        c = g[(y0 + 1) % 64, x0 % 64]
        d = g[(y0 + 1) % 64, (x0 + 1) % 64]
        total += amp * ((a * (1 - fx) + b * fx) * (1 - fy) + (c * (1 - fx) + d * fx) * fy)
        amp *= 0.5
        freq *= 2.0
    return total


def grid(w=None, h=None):
    """Sample positions in native pixels, one per vertex, RES + 1 a side (w x h for a sized panel)."""
    if w is None:
        s = np.linspace(0, N, RES + 1)
        return np.meshgrid(s, s)
    return np.meshgrid(np.linspace(0, w, w * PX + 1), np.linspace(0, h, h * PX + 1))


def frame_field():
    x, y = grid()
    d = np.minimum.reduce([x, y, N - x, N - y])  # distance from the outer edge: mitres for free
    # Which side is nearest: the grain runs along it, varying only across it (d).
    h = np.zeros_like(d)
    col = np.zeros(d.shape + (3,))
    wood = [hexlin(c) for c in ("#41322c", "#725244", "#ab8465", "#cfaa81")]
    stone = [hexlin(c) for c in ("#2c2c25", "#434137", "#a09f91")]
    iron = hexlin("#434137")
    # Moulding profile, 0.6..8 px: bead, flat, groove, flat, fillet (heights in px).
    t = d
    bead = np.clip(1 - ((t - 2.0) / 1.4) ** 2, 0, None) * 1.2
    h = np.where(t < 8.0, 2.6 + bead, 0)
    h = np.where((t > 3.6) & (t < 4.6), h - 0.9, h)  # the carved groove
    h = np.where((t >= 6.4) & (t < 8.0), h - (t - 6.4) * 0.9, h)  # the fillet falls to the seam
    grain = np.sin(t * 9.0 + 2.0 * np.sin(t * 2.3)) * 0.5 + 0.5  # lines along each side
    wcol = wood[1] * (1 - grain[..., None]) * 0.5 + wood[2] * (0.5 + 0.5 * (1 - grain[..., None]))
    wcol = np.where(grain[..., None] > 0.85, wood[0], wcol)
    col[:] = wcol
    # Seam, then the stone lip sloping down to the face.
    seam = (t >= 8.0) & (t < 9.0)
    h = np.where(seam, 0.3, h)
    col[seam] = hexlin("#170916")
    lip = (t >= 9.0) & (t < 13.0)
    h = np.where(lip, 0.8 - (t - 9.0) * 0.2, h)
    col[lip] = stone[1]
    face = t >= 13.0
    h = np.where(face, 0.0, h)
    col[face] = stone[0]
    # The mitre: a fine groove on the diagonals of the moulding.
    diag = (np.abs(np.abs(x - N / 2) - np.abs(y - N / 2)) < 0.35) & (t < 8.0)
    h = np.where(diag, h - 0.6, h)
    col[diag] = hexlin("#170916")
    # Iron corner plates (6 x 6 L brackets) and rivets: corners only, never the stretching edges.
    for cx, cy in ((0, 0), (N, 0), (0, N), (N, N)):
        ax, ay = np.abs(x - cx), np.abs(y - cy)
        plate = ((ax < 7) & (ay < 2.4)) | ((ay < 7) & (ax < 2.4))
        plate &= (ax > 0.7) & (ay > 0.7)
        h = np.where(plate, np.maximum(h, 3.6), h)
        col[plate] = iron
        for rx, ry in ((5.0, 1.6), (1.6, 5.0)):
            r = np.hypot(ax - rx, ay - ry)
            dome = r < 0.9
            h = np.where(dome, 3.6 + np.sqrt(np.clip(0.81 - r ** 2, 0, None)) * 0.8, h)
            col[dome] = hexlin("#a09f91")
    alpha = (d > 0.55).astype(float)
    return h, col, alpha


def rock_field(W=None, H=None, strata_rings=True):
    """The rough rock. With W and H it is the approved base rock (owner, 2026-11-14, the parent
    session's sample, "Ok good continue with the mockups"): this same field rendered at the panel's
    own size, so no edge, rim or lip shadow ever tiles, and with the strata rings removed (the owner:
    "this version without the lines"). Nothing else differs from the first render."""
    W = W or N
    H = H or N
    x, y = grid(W, H) if (W, H) != (N, N) or not strata_rings else grid()
    d = np.minimum.reduce([x, y, W - x, H - y])
    rough = fbm(x, y, 7, 5, 0.12)
    rough2 = fbm(x, y, 11, 3, 0.5)
    # The kit's own rock ramp, slate blue since the owner's "make the stone more blue" (2026-11-14).
    rock = [hexlin(c) for c in ui_art.ROCK[1:]]
    # Chipped silhouette: stronger chips toward the corners.
    corner = None
    near_corner = np.clip(1 - np.maximum(np.minimum(x, W - x), np.minimum(y, H - y)) / 14.0, 0, 1)
    edge_cut = 0.6 + 2.6 * near_corner * fbm(x, y, 3, 3, 0.3)
    alpha = (d > edge_cut).astype(float)
    h = 2.0 + rough * 2.2 + rough2 * 0.5 - np.clip(2.5 - (d - edge_cut), 0, None) * 0.8
    v = np.clip((rough - 0.5) * 1.6 + 0.5, 0, 1)
    col = rock[1] * (1 - v[..., None]) + rock[2] * v[..., None]
    # Strata along each side (vary only across it).
    if strata_rings:  # the first render's rings at fixed distances: straight lines along the rim
        strata = np.sin(d * 3.1) > 0.92
        col[strata & (d < 9)] = rock[0]
    lip = (d >= 9.0) & (d < 12.0)
    h = np.where(lip, 0.9 - (d - 9.0) * 0.3, h)
    col[lip] = rock[1]
    face = d >= 12.0
    h = np.where(face, 0.0, h)
    col[face] = hexlin(ui_art.FACE)
    del corner
    return h, col, alpha


def build_mesh(name, h, col, alpha, W=None, H=None):
    W = W or N
    H = H or N
    nx, ny = h.shape[1], h.shape[0]
    X, Y = np.meshgrid(np.linspace(-W / 2, W / 2, nx), np.linspace(H / 2, -H / 2, ny))
    verts = np.stack([X.ravel(), Y.ravel(), h.ravel()], 1)
    keep = alpha.ravel() > 0.5
    idx = np.arange(nx * ny).reshape(ny, nx)
    a, b = idx[:-1, :-1].ravel(), idx[:-1, 1:].ravel()
    c, d = idx[1:, 1:].ravel(), idx[1:, :-1].ravel()
    ok = keep[a] & keep[b] & keep[c] & keep[d]
    faces = np.stack([a, d, c, b], 1)[ok]
    me = bpy.data.meshes.new(name)
    me.vertices.add(len(verts))
    me.vertices.foreach_set("co", verts.astype(np.float32).ravel())
    me.loops.add(len(faces) * 4)
    me.loops.foreach_set("vertex_index", faces.astype(np.int32).ravel())
    me.polygons.add(len(faces))
    me.polygons.foreach_set("loop_start", (np.arange(len(faces)) * 4).astype(np.int32))
    me.polygons.foreach_set("loop_total", np.full(len(faces), 4, np.int32))
    me.update()
    me.validate()
    attr = me.color_attributes.new("albedo", "FLOAT_COLOR", "POINT")
    rgba = np.concatenate([col.reshape(-1, 3), np.ones((nx * ny, 1))], 1)
    attr.data.foreach_set("color", rgba.astype(np.float32).ravel())
    me.shade_smooth()
    ob = bpy.data.objects.new(name, me)
    bpy.context.scene.collection.objects.link(ob)
    mat = bpy.data.materials.new(name + "_mat")
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    at = nt.nodes.new("ShaderNodeAttribute")
    at.attribute_name = "albedo"
    nt.links.new(at.outputs["Color"], bsdf.inputs["Base Color"])
    bsdf.inputs["Roughness"].default_value = 0.7
    ob.data.materials.append(mat)
    return ob


def scene(W=None, H=None):
    W = W or N
    H = H or N
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = SAMPLES
    sc.cycles.seed = 1
    sc.cycles.use_denoising = False
    sc.cycles.max_bounces = 2
    sc.render.resolution_x, sc.render.resolution_y = W * PX, H * PX
    sc.render.film_transparent = True
    sc.view_settings.view_transform = "Standard"
    sc.render.image_settings.file_format = "PNG"
    sc.render.image_settings.color_mode = "RGBA"
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = max(W, H)
    co = bpy.data.objects.new("cam", cam)
    co.location = (0, 0, 50)
    sc.collection.objects.link(co)
    sc.camera = co
    sun = bpy.data.lights.new("sun", "SUN")
    sun.energy = 3.2
    sun.angle = math.radians(12)
    so = bpy.data.objects.new("sun", sun)
    # From the upper left of the screen, 50 degrees up.
    so.rotation_euler = (math.radians(-40), math.radians(-28), math.radians(0))
    sc.collection.objects.link(so)
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.35
    sc.world = world
    return sc


def pixelate(path_render, path_out):
    im = np.asarray(Image.open(path_render).convert("RGBA"), float) / 255.0
    rgb_lin = srgb_to_lin(im[..., :3] * 255.0)
    a = im[..., 3]
    k = PX
    pre = rgb_lin * a[..., None]
    pre = pre.reshape(N, k, N, k, 3).mean((1, 3))
    am = a.reshape(N, k, N, k).mean((1, 3))
    lin = pre / np.maximum(am[..., None], 1e-6)
    s = np.where(lin <= 0.0031308, lin * 12.92, 1.055 * np.power(lin, 1 / 2.4) - 0.055) * 255.0
    # Snap to the kit's material ramps only (the panel-textures "reduce the palette" step): the whole
    # of Sweaft-64 lets fbm noise pick blues and greens, which reads as speckle, not stone.
    pal = np.array([rgb(h) for h in KIT_COLOURS], float)
    dist = ((s[:, :, None, :] - pal[None, None]) ** 2 * np.array([0.3, 0.59, 0.11])).sum(-1)
    q = pal[dist.argmin(-1)].astype(np.uint8)
    solid = am >= 0.5
    out = np.zeros((N, N, 4), np.uint8)
    out[solid, :3] = q[solid]
    out[solid, 3] = 255
    ink = np.array(rgb("#010000"), np.uint8)
    for y in range(N):
        for x in range(N):
            edge = x in (0, N - 1) or y in (0, N - 1)
            if solid[y, x] and (edge or not (solid[y - 1, x] and solid[y + 1, x] and solid[y, x - 1]
                                             and solid[y, x + 1])):
                out[y, x, :3] = ink
    save(Image.fromarray(out, "RGBA"), path_out, source="tools/art/ui_panels_blend.py")


def legacy_main():
    """The first two renders (M29 before the whole kit): frame and rock, into docs/mockups/ui/blender-*.png.
    Kept as the record of Q1's first comparison. Their rock rim carried strata = sin(d * 3.1) > 0.92, rings
    of constant distance from the edge, which read as straight stitched lines along the rim (owner,
    2026-11-14: "how come the rock one has lines in it?" ... "the 3d rendered one"); the approved base
    rock (rock_field with W, H) drops the rings and renders each panel at its own size."""
    os.makedirs(OUT, exist_ok=True)
    only = arg("--only", "")
    for name, field in (("frame", frame_field), ("rock", rock_field)):
        if only and name != only:
            continue
        sc = scene()
        build_mesh(name, *field())
        render = os.path.join(OUT, f"blender-{name}-render.png")
        sc.render.filepath = render
        bpy.ops.render.render(write_still=True)
        pixelate(render, os.path.join(OUT, f"blender-{name}.png"))
        # Keep the reference render small: half size is plenty to judge the modelling.
        Image.open(render).resize((RES // 2, RES // 2), Image.LANCZOS).save(render, optimize=True)
        print("rendered", name, render)




# =============================================================================================
# The whole kit, rendered (owner, 2026-11-14: "show me the game with that 3d rendered look", then
# "yea the 3d rendered look as a whole kit"). Every piece of the drawn kit (ui_art.PIECES) is
# modelled here as one height field with material masks, at the drawn piece's size and layout, so
# its 9-slice margins (slices.json) are the drawn ones and the rendered piece is drop-in.
#
# Rendered at KPX = 8 render pixels per native pixel (an exact multiple, so the smooth renders keep
# the slices too: a margin of 14 is 112 there). Two outputs per piece:
#   docs/mockups/ui/rendered/<name>.png        pixelated: area-averaged in linear light, each pixel
#                                              snapped to its own material's Sweaft-64 ramp, the
#                                              drawn piece's silhouette and INK outline kept;
#   docs/mockups/ui/rendered/hires/<name>.png  smooth: the render itself at 8x (the "Smooth" look).
# Stretching sides are made plain: each row of a top or bottom band and each column of a side band
# is set to its own commonest colour, and the centre to one colour (the panel-textures rule). The
# rough rock's sides tile on a 20 px period, so its noise is periodic in 20 px instead.
# =============================================================================================
KPX = 8
RELIEF = 1.7
# The whole kit is rendered under the first rock render's light and material, the approved base
# rock's (owner, 2026-11-14): a sun at Euler (-40, -28, 0) degrees, 3.2, a 0.35 sky, Blender's default
# specular, roughness 0.7, no normalising. Every other stone piece carries that material.
ROCK_PX = 4  # render pixels per native pixel for the sized rock panels (the approved sample's)
RENDERED = os.path.join(OUT, "rendered")
HIRES = os.path.join(RENDERED, "hires")

INKC = "#010000"
RAMPS = {
    # every ramp dark to light, every colour a Sweaft-64 entry
    "stone": ["#010e1a", "#011c34", "#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "rock": ["#010e1a", "#011c34", "#1f3b52", "#3e586e", "#5d778d", "#92a3b2", "#bfcad7"],
    "well": ["#010e1a", "#011c34", "#1f3b52"],
    "oak": ["#170916", "#2f1525", "#41322c", "#725244", "#ab8465", "#cfaa81", "#eab798"],
    "iron": ui_art.IRON,
    "brass": ui_art.BRASS,
    "rune": ui_art.RUNE,
    "seam": ["#010000", "#170916"],
    "ink": [INKC],
    "health": ui_art.HEALTH,
    "stamina": ui_art.STAMINA,
    "breath": ui_art.BREATH,
    "progress": ui_art.RUNE,
    "dot": ["#0c2f44", "#15578a", "#1096c3", "#74effe", "#d2f8ff"],
    "uncommon": ui_art.RARITY["uncommon"],
    "rare": ui_art.RARITY["rare"],
    "epic": ui_art.RARITY["epic"],
    "legendary": ui_art.RARITY["legendary"],
}
MAT_IDS = {n: i for i, n in enumerate(RAMPS)}
for _r in RAMPS.values():
    for _c in _r:
        assert _c in SWEAFT_64, _c
S = ui_art  # the drawn kit's tokens


def vnoise(x, y, seed, cells, period=None):
    """Value noise with `cells` lattice cells per native pixel... per unit; periodic in `period`
    native px when given (the lattice wraps), so a field tiles exactly."""
    rng = np.random.default_rng(seed)
    if period:
        n = max(1, int(round(period * cells)))
        g = rng.random((n, n))
        xi, yi = x * n / period, y * n / period
    else:
        n = 97
        g = rng.random((n, n))
        xi, yi = x * cells, y * cells
    x0, y0 = np.floor(xi).astype(int), np.floor(yi).astype(int)
    fx, fy = xi - x0, yi - y0
    fx, fy = fx * fx * (3 - 2 * fx), fy * fy * (3 - 2 * fy)
    a, b = g[y0 % n, x0 % n], g[y0 % n, (x0 + 1) % n]
    c, d = g[(y0 + 1) % n, x0 % n], g[(y0 + 1) % n, (x0 + 1) % n]
    return (a * (1 - fx) + b * fx) * (1 - fy) + (c * (1 - fx) + d * fx) * fy


def fbm2(x, y, seed, cells=0.25, octaves=4, period=None):
    tot, amp, norm = 0.0, 1.0, 0.0
    for o in range(octaves):
        tot = tot + amp * vnoise(x, y, seed + 31 * o, cells * 2 ** o, period)
        norm += amp
        amp *= 0.5
    return tot / norm


def voronoi(x, y, seed, cell=4.0, period=None):
    """F1 and F2 distances to scattered points (isotropic cells: chips, pits, crack networks). Periodic
    when `period` is a whole number of cells."""
    rng = np.random.default_rng(seed)
    if period:
        n = max(1, int(round(period / cell)))
        cell = period / n
    else:
        n = 64
    pts = rng.random((n, n, 2))
    gx, gy = np.floor(x / cell).astype(int), np.floor(y / cell).astype(int)
    f1 = np.full(x.shape, 9e9)
    f2 = np.full(x.shape, 9e9)
    for oy in (-1, 0, 1):
        for ox in (-1, 0, 1):
            cx, cy = gx + ox, gy + oy
            p = pts[cy % n, cx % n]
            px_, py_ = (cx + p[..., 0]) * cell, (cy + p[..., 1]) * cell
            dd = np.hypot(x - px_, y - py_)
            f2 = np.where(dd < f1, f1, np.minimum(f2, dd))
            f1 = np.minimum(f1, dd)
    return f1, f2


class Field:
    """One piece's height field and materials, sampled at render-pixel centres in native px units
    (x right, y down; pixel (i, j) of the drawn piece covers [i, i+1) x [j, j+1))."""

    def __init__(self, w, h, slices=None, sides="stretch"):
        self.w, self.h = w, h
        sx = (np.arange(w * KPX) + 0.5) / KPX
        sy = (np.arange(h * KPX) + 0.5) / KPX
        self.X, self.Y = np.meshgrid(sx, sy)
        self.H = np.zeros_like(self.X)
        self.mat = np.full(self.X.shape, MAT_IDS["stone"], np.int32)
        self.alb = np.zeros(self.X.shape + (3,))
        self.alb[:] = hexlin(S.STONE[2])
        self.mre = np.zeros(self.X.shape + (3,))
        self.mre[..., 1] = 0.7
        if isinstance(slices, int):
            slices = [slices] * 4
        self.slices, self.sides = slices, sides
        # Corner weight: 1 deep in a corner slice, 0 in the bands and centre (ramped over 2 px so the
        # corner's detail fades into the band's plain run instead of stopping on the slice line).
        if slices:
            t, r, b, l = slices
            cx = np.maximum(l - self.X, self.X - (w - r))
            cy = np.maximum(t - self.Y, self.Y - (h - b))
            self.cw = np.clip(np.minimum(cx, cy) / 2.0, 0, 1)
            self.band_x = (cy > 0) & (cx <= 0)  # top/bottom bands: may vary with y only
            self.band_y = (cx > 0) & (cy <= 0)  # side bands: with x only
        else:
            self.cw = np.ones_like(self.X)
            self.band_x = self.band_y = np.zeros(self.X.shape, bool)

    # ---- geometry helpers (rect coordinates are the drawn kit's inclusive pixel indices) ----
    def inset(self, x0, y0, x1, y1):
        """Signed distance inside the rectangle of pixels x0..x1, y0..y1 (negative outside)."""
        return np.minimum.reduce([self.X - x0, x1 + 1 - self.X, self.Y - y0, y1 + 1 - self.Y])

    def ring_d(self, x0, y0, x1, y1):
        """Distance to the centre line of the one-pixel ring x0..x1, y0..y1."""
        qx = np.abs(self.X - (x0 + x1 + 1) / 2) - (x1 - x0) / 2
        qy = np.abs(self.Y - (y0 + y1 + 1) / 2) - (y1 - y0) / 2
        out = np.hypot(np.maximum(qx, 0), np.maximum(qy, 0)) + np.minimum(np.maximum(qx, qy), 0)
        return np.abs(out)

    def paint(self, m, mat, tone, metal=0.0, rough=0.7, emit=0.0, jitter=0.0, seed=0):
        col = hexlin(tone)
        if jitter:
            n = self.tex(seed, 0.6) - 0.5
            col = col[None, None, :] * (1 + jitter * 2 * n)[..., None]
            self.alb[m] = col[m]
        else:
            self.alb[m] = col
        self.mat[m] = MAT_IDS[mat]
        self.mre[m] = (metal, rough, emit)

    def tex(self, seed, cells=0.35, period=None):
        """Surface texture in 0..1: isotropic fbm + Voronoi pits in the corners; in the stretching
        bands a texture that varies only across the band; none in the centre. Periodic pieces
        (the rock) use one periodic field everywhere."""
        n2 = fbm2(self.X, self.Y, seed, cells, 4, period)
        if self.sides == "tile" or not self.slices:
            return n2
        n1x = fbm2(np.zeros_like(self.X), self.Y, seed + 7, cells, 3)
        n1y = fbm2(self.X, np.zeros_like(self.Y), seed + 9, cells, 3)
        base = np.where(self.band_x, n1x, np.where(self.band_y, n1y, 0.5))
        return base * (1 - self.cw) + n2 * self.cw

    def prof(self, d, bevel):
        """A rounded edge profile 0..1 over `bevel` px inside a shape (quarter-round)."""
        t = np.clip(d / bevel, 0, 1)
        return np.sqrt(1 - (1 - t) ** 2)

    def slab(self, x0, y0, x1, y1, height, bevel, mat, tone, base=0.0, **kw):
        d = self.inset(x0, y0, x1, y1)
        m = d > 0
        self.H = np.where(m, base + height * self.prof(d, bevel), self.H)
        self.paint(m, mat, tone, **kw)
        return d

    def sink(self, x0, y0, x1, y1, depth, bevel, mat=None, tone=None, floor_from=None, **kw):
        d = self.inset(x0, y0, x1, y1)
        m = d > 0
        self.H = np.where(m, self.H - depth * self.prof(d, bevel), self.H)
        if mat:
            fm = m if floor_from is None else (d > floor_from)
            self.paint(fm, mat, tone, **kw)
        return d

    def groove(self, m_dist, width, depth, mat=None, tone=None, **kw):
        g = np.clip(1 - m_dist / width, 0, 1)
        self.H = self.H - depth * np.sqrt(g)
        if mat:
            self.paint(g > 0.45, mat, tone, **kw)

    def dome(self, cx, cy, r, height, mat, tone, base=None, **kw):
        rr = np.hypot(self.X - cx, self.Y - cy)
        m = rr < r
        b = self.H if base is None else base
        top = b + height * np.sqrt(np.clip(1 - (rr / r) ** 2, 0, 1))
        self.H = np.where(m, np.maximum(self.H, top), self.H)
        self.paint(m, mat, tone, **kw)

    def rough(self, m, amp, seed, cells=0.12, period=None):
        """The base rock's surface on dressed stone: the same two fbm layers as rock_field (0.12 and
        0.5 a pixel, relief 2.2 : 0.5) and the same light/dark blend in the albedo, scaled to the
        piece; full in the corners, varying only across a stretching band, none in the centre."""
        n1 = self.tex(seed, cells, period)
        n2 = self.tex(seed + 1, 0.5, period)
        self.H = np.where(m, self.H + 2.5 * amp * ((n1 - 0.5) + 0.23 * (n2 - 0.5)), self.H)
        v = np.clip((n1 - 0.5) * 1.6 + 0.5, 0, 1)
        k = 0.86 + 0.28 * v
        self.alb = np.where(m[..., None], self.alb * k[..., None], self.alb)

def fold(F):
    """Coordinates folded into the top-left quadrant, for four-fold symmetric corner fittings."""
    return np.minimum(F.X, F.w - F.X), np.minimum(F.Y, F.h - F.Y)


def iron_kw():
    return dict(metal=0.85, rough=0.42)


def stud(F, cx, cy, r=1.05, height=0.9, mat="iron", tone=None):
    tone = tone or (S.IRON[3] if mat == "iron" else S.BRASS[2])
    F.dome(cx, cy, r, height, mat, tone, **iron_kw())


# ---- the pieces ------------------------------------------------------------------------------
def moulding_profile(d, groove_at=3.5):
    """Dressed-stone moulding across its width d (px from the outer edge): a rolled arris, a flat,
    a cut groove, a flat, a fillet falling to the seam."""
    h = 2.4 + 0.7 * np.clip(1 - ((d - 1.4) / 1.3) ** 2, 0, None)
    h = h - 0.9 * np.clip(1 - np.abs(d - groove_at) / 0.7, 0, 1)
    h = np.where(d > 5.4, h - (d - 5.4) * 1.0, h)
    return h


def r_frame():
    F = Field(48, 48, 14)
    d = F.inset(1, 1, 46, 46)
    F.H = moulding_profile(d)
    F.paint(d > 0, "stone", S.STONE_MID, jitter=0.12, seed=3)
    F.groove(F.ring_d(4, 4, 43, 43), 0.6, 0.0, "stone", S.STONE[1])
    F.rough((d > 0) & (d < 7.5), 0.35, 5)
    fx, fy = fold(F)
    mitre = (np.abs(fx - fy) < 0.45) & (d < 7.6)
    F.H = np.where(mitre, F.H - 0.6, F.H)
    F.paint(mitre, "stone", S.STONE[1])
    for j in (12.5, 35.5):  # quoin joints, inside the corner slices on both axes
        for m in ((np.abs(F.X - j) < 0.4) & (d < 7.6) & ((F.Y < 8) | (F.Y > 40)),
                  (np.abs(F.Y - j) < 0.4) & (d < 7.6) & ((F.X < 8) | (F.X > 40))):
            F.H = np.where(m, F.H - 0.5, F.H)
            F.paint(m, "stone", S.STONE[1])
    seam = (d > 7) & (d <= 8)
    F.H = np.where(seam, 0.6, F.H)
    lip = F.inset(13, 13, 34, 34)
    inl = (d > 8) & (lip <= 0)
    F.H = np.where(inl, np.clip(-lip, 0, 4) / 4 * 1.1, F.H)
    F.paint(inl, "stone", S.STONE[1])
    F.H = np.where(lip > 0, 0.0, F.H)
    F.paint(lip > 0, "stone", S.FACE)
    # Iron corner brackets (an L, 2 px thick, 6 long) with a forged rivet on each arm.
    br = ((fx > 1) & (fx < 7) & (fy > 1) & (fy < 3)) | ((fy > 1) & (fy < 7) & (fx > 1) & (fx < 3))
    F.H = np.where(br, 3.6 + 0.25 * np.clip(np.minimum(np.minimum(fx - 1, 7 - fx), 1), 0, 1), F.H)
    F.paint(br, "iron", S.IRON[3], **iron_kw())
    for cx, cy in ((6.0, 2.0), (2.0, 6.0)):
        for X0 in (cx, 48 - cx):
            for Y0 in (cy, 48 - cy):
                stud(F, X0, Y0, 0.95, 0.8)
    return F


def r_lintel():
    F = Field(40, 16, [5, 9, 5, 9])
    F.slab(1, 1, 38, 14, 2.0, 1.8, "oak", S.WOOD[3], rough=0.7)
    # Grain along the board: varies with y only through the bands and centre; knots and the end
    # notches in the corners only.
    gy = fbm2(np.zeros_like(F.X), F.Y, 61, 1.2, 3)
    lines = (np.abs(F.Y - 3.5) < 0.5) | (np.abs(F.Y - 12.5) < 0.5)
    F.H = F.H + 0.18 * (gy - 0.5) - 0.25 * lines
    F.alb = np.where(lines[..., None], hexlin(S.WOOD[2]), F.alb * (0.9 + 0.2 * gy)[..., None])
    knot = np.hypot((F.X - 4.0) * 0.8, F.Y - 11.5) < 1.1
    F.alb = np.where((knot & (F.cw > 0.5))[..., None], hexlin(S.WOOD[2]), F.alb)
    for x0 in (4.5, 35.5):
        m = (np.abs(F.X - x0) < 0.5) & (F.Y > 4) & (F.Y < 12)
        F.H = np.where(m, F.H - 0.8, F.H)
        F.paint(m, "seam", S.SEAM)
    for cx in (7.0, 33.0):
        stud(F, cx, 8.0, 1.0, 1.0)
    return F


def r_slot(state="idle", rarity="common"):
    F = Field(24, 24)
    body = {"idle": S.STONE[2], "hover": S.STONE[3], "selected": S.STONE[2], "disabled": S.STONE[1]}[state]
    F.slab(1, 1, 22, 22, 2.0, 1.4, "stone", body, jitter=0.1, seed=4)
    F.rough(F.inset(1, 1, 22, 22) > 0, 0.3, 8)
    wd = F.inset(3, 3, 20, 20)
    m = wd > 0
    F.H = np.where(m, F.H - 2.0 * F.prof(wd, 1.0), F.H)
    F.paint(m & (wd <= 1), "stone", S.STONE[1])
    F.paint(wd > 1, "well", S.WELL)
    if state == "selected":
        rd = F.inset(1, 1, 22, 22)
        rm = (rd > 0) & (rd < 2)
        F.H = np.where(rm, F.H - 0.2, F.H)
        F.paint(rm, "rune", S.RUNE[2], emit=0.9, rough=0.5)
    if state == "disabled":
        dust = fbm2(F.X, F.Y, 71, 0.9, 2)
        fl = wd > 1
        F.H = np.where(fl, F.H + 0.5 * (dust - 0.5), F.H)
        F.alb = np.where((fl & (dust > 0.55))[..., None], hexlin(S.STONE[1]), F.alb)
    gem = ui_art.RARITY.get(rarity)
    if gem:
        F.paint(m & (wd <= 1), rarity, gem[1], rough=0.35)
        for cx, cy in ((2, 2), (22, 2), (2, 22), (22, 22)):
            F.dome(cx, cy, 1.15, 0.9, rarity, gem[1], rough=0.25)
    return F


def r_button(state):
    F = Field(48, 22, 8)
    body = S.PLAQUE[state][0]
    top = 2 if state == "pressed" else 1
    d = F.inset(1, top, 46, 20)
    m = d > 0
    if state == "pressed":
        # Sunk a pixel: the rim stands above the face, so the light turns round.
        F.H = np.where(m, 1.0 - 0.7 * F.prof(d, 2.0), 0)
    elif state == "disabled":
        F.H = np.where(m, 0.6 * F.prof(d, 2.0), 0)
    else:
        F.H = np.where(m, 1.7 * F.prof(d, 2.2), 0)
    F.paint(m, "stone", body, jitter=0.08, seed=11)
    F.rough(m, 0.3 if state != "disabled" else 0.15, 12)
    F.groove(F.ring_d(4, top + 3, 43, 17), 0.7, 0.8 if state != "disabled" else 0.4)
    if state != "disabled":
        mat = "brass" if state == "hover" else "iron"
        for cx, cy in ((4, top + 3), (44, top + 3), (4, 17), (44, 17)):
            stud(F, cx, cy, 0.95, 0.8, mat)
    if state == "focus":
        rm = (F.ring_d(1, 1, 46, 20) < 0.5) & ~((np.abs(F.X - 1.5) < 1) & (np.abs(F.Y - 1.5) < 1))
        F.paint(rm, "rune", S.RUNE[2], emit=0.9, rough=0.5)
    return F


def r_tab(active):
    F = Field(40, 18, 7)
    y0 = 0 if active else 2
    body = S.STONE[2] if active else S.STONE[1]
    side = np.minimum.reduce([F.X - 1, 39 - F.X, F.Y - (y0 + 1)])
    if not active:
        side = np.minimum(side, 17 - F.Y)
    m = side > 0
    F.H = np.where(m, 1.5 * F.prof(side, 1.6), 0)
    F.paint(m, "stone", body, jitter=0.08, seed=13)
    F.rough(m, 0.25, 14)
    if active:
        cap = (F.Y >= y0 + 1) & (F.Y < y0 + 4) & m
        cy = (F.Y - (y0 + 2.5)) / 1.5
        F.H = np.where(cap, 1.9 + 0.5 * np.sqrt(np.clip(1 - cy ** 2, 0, 1)), F.H)
        gy = fbm2(np.zeros_like(F.X), F.Y, 81, 1.5, 2)
        F.paint(cap, "oak", S.WOOD[3], rough=0.7)
        F.alb = np.where(cap[..., None], F.alb * (0.85 + 0.3 * gy)[..., None], F.alb)
        sm = (F.Y >= y0 + 4) & (F.Y < y0 + 5) & m
        F.H = np.where(sm, 1.1, F.H)
        F.paint(sm, "seam", S.SEAM)
        for cx in (4.0, 36.0):
            stud(F, cx, y0 + 3.0, 0.9, 0.6)
    else:
        F.groove(np.abs(F.Y - (y0 + 3.5)) + np.where((F.X > 4) & (F.X < 36), 0, 9), 0.6, 0.45, "stone", S.STONE[0])
        foot = (F.Y >= 16) & m
        F.H = np.where(foot, F.H - 0.6, F.H)
        F.paint(foot, "stone", S.STONE[0])
    return F


def r_groove(w, h, sl):
    F = Field(w, h, sl)
    d = F.inset(1, 1, w - 2, h - 2)
    F.H = np.where(d > 0, 1.2 - 1.3 * F.prof(d, 1.0), 1.2)
    F.paint(d > 0, "well", S.WELL)
    F.paint((d > 0) & (d < 0.9), "stone", S.STONE[1])
    return F


def r_scroll_thumb():
    F = Field(8, 16, 4)
    F.slab(1, 1, 6, 14, 1.5, 1.2, "stone", S.STONE_MID, jitter=0.08, seed=15)
    for yy in (2, 13):
        m = (F.Y >= yy) & (F.Y < yy + 1) & (F.X > 1) & (F.X < 7)
        F.H = np.where(m, F.H + 0.35, F.H)
        F.paint(m, "iron", S.IRON[3] if yy == 2 else S.IRON[2], **iron_kw())
    return F


def r_slider_knob():
    F = Field(9, 13)
    F.slab(1, 1, 7, 11, 1.6, 1.2, "stone", S.STONE_MID, jitter=0.08, seed=17)
    cap = F.inset(1, 1, 7, 3)
    F.H = np.where(cap > 0, 2.0 + 0.5 * F.prof(cap, 0.8), F.H)
    F.paint(cap > 0, "iron", S.IRON[3], **iron_kw())
    for yy in (5, 7, 9):
        F.groove(np.hypot(np.maximum(np.abs(F.X - 4.5) - 1.0, 0), F.Y - (yy + 0.5)), 0.55, 0.5, "stone", S.STONE[1])
    return F


def r_check(on):
    F = Field(12, 12)
    F.slab(1, 1, 10, 10, 1.6, 1.0, "stone", S.STONE[2], jitter=0.08, seed=19)
    wd = F.inset(2, 2, 9, 9)
    F.H = np.where(wd > 0, F.H - 1.6 * F.prof(wd, 1.0), F.H)
    F.paint(wd > 1, "well", S.WELL)
    if on:
        pts = [(3, 6), (4, 7), (5, 8), (6, 7), (7, 6), (8, 5), (8, 4), (8, 3)]
        dd = np.full(F.X.shape, 9.0)
        for (ax, ay), (bx, by) in zip(pts[:-1], pts[1:]):
            ax, ay, bx, by = ax + 0.5, ay + 0.5, bx + 0.5, by + 0.5
            vx, vy = bx - ax, by - ay
            t = np.clip(((F.X - ax) * vx + (F.Y - ay) * vy) / (vx * vx + vy * vy), 0, 1)
            dd = np.minimum(dd, np.hypot(F.X - ax - t * vx, F.Y - ay - t * vy))
        m = dd < 0.75
        F.H = np.where(m, F.H + 0.4 * np.sqrt(np.clip(1 - dd / 0.75, 0, 1)), F.H)
        F.paint(m, "rune", S.RUNE[2], emit=1.0, rough=0.4)
    return F


def r_tooltip():
    F = Field(24, 18, 6)
    d = F.inset(1, 1, 22, 16)
    F.H = np.where(d > 0, 1.6 * F.prof(d, 1.4), 0)
    F.paint(d > 0, "stone", S.STONE_MID, jitter=0.08, seed=21)
    F.rough(d > 0, 0.25, 22)
    lip = F.inset(5, 5, 18, 12)
    inl = (d > 2) & (lip <= 0)
    F.H = np.where(inl, np.clip(-lip, 0, 2) / 2 * 1.0, F.H)
    F.paint(inl, "stone", S.STONE[1])
    F.H = np.where(lip > 0, 0.0, F.H)
    F.paint(lip > 0, "stone", S.FACE)
    for cx in (3.0, 21.0):
        for cy in (3.0, 15.0):
            stud(F, cx, cy, 0.85, 0.6)
    return F


def r_bar():
    F = Field(40, 10, 4)
    F.slab(1, 1, 38, 8, 1.5, 1.0, "stone", S.STONE[2], jitter=0.08, seed=23)
    F.rough(F.inset(1, 1, 38, 8) > 0, 0.25, 24)
    ch = F.inset(4, 2, 35, 7)
    F.H = np.where(ch > 0, F.H - 1.5 * F.prof(ch, 1.0), F.H)
    F.paint(ch > 1, "well", S.WELL)
    F.paint((ch > 0) & (ch <= 1), "stone", S.STONE[1])
    stud(F, 3.0, 5.0, 1.0, 0.8)
    stud(F, 37.0, 5.0, 1.0, 0.8)
    return F


def r_fill(ramp_name):
    F = Field(2, 4)
    cy = (F.Y - 2.0) / 2.0
    F.H = 1.2 * np.sqrt(np.clip(1 - cy ** 2, 0, 1))
    F.paint(np.ones(F.X.shape, bool), ramp_name, RAMPS[ramp_name][2], rough=0.3, emit=0.15)
    return F


def r_compass():
    F = Field(80, 18, [9, 10, 9, 10])
    d = F.inset(1, 1, 78, 16)
    F.H = np.where(d > 0, 1.8 * F.prof(d, 1.6), 0)
    F.paint(d > 0, "stone", S.STONE[3], jitter=0.06, seed=25)
    F.rough(d > 0, 0.25, 26)
    F.groove(np.abs(F.Y - 12.8) + np.where((F.X > 10) & (F.X < 70), 0, 9), 0.9, 0.7, "stone", S.STONE_MID)
    for x0 in (3, 72):
        band = (F.X >= x0) & (F.X < x0 + 2) & (F.Y > 2) & (F.Y < 16)
        F.H = np.where(band, 2.1, F.H)
        F.paint(band, "iron", S.IRON[3], **iron_kw())
        sep = (np.abs(F.X - (x0 + 4.5)) < 0.45) & (F.Y > 2) & (F.Y < 16)
        F.H = np.where(sep, F.H - 0.5, F.H)
        F.paint(sep, "stone", S.STONE[2])
        F.dome(x0 + 2.5, 8.5, 1.4, 1.0, "brass", S.BRASS[2], **iron_kw())
    return F


def r_pointer():
    F = Field(7, 6)
    ridge = 1.6 - 0.45 * np.abs(F.X - 3.5) - 0.15 * F.Y
    F.H = np.clip(ridge, 0.2, None)
    F.paint(np.ones(F.X.shape, bool), "oak", S.WOOD[3], rough=0.65)
    F.dome(3.5, 4.5, 0.9, 0.7, "iron", S.IRON[4], **iron_kw())
    return F


def r_town():
    F = Field(5, 5)
    F.H[:] = 0
    F.dome(2.5, 2.5, 2.6, 1.6, "brass", S.BRASS[2], base=0.0, **iron_kw())
    return F


def r_map_frame():
    F = Field(64, 64, 22)
    d = F.inset(1, 1, 62, 62)
    h = 2.2 + 0.7 * np.clip(1 - ((d - 1.0) / 1.0) ** 2, 0, None)
    h = h - 0.9 * np.clip(1 - np.abs(d - 2.5) / 0.6, 0, 1)
    h = np.where(d > 4.2, h - (d - 4.2) * 0.8, h)
    F.H = h
    F.paint(d > 0, "stone", S.STONE_MID, jitter=0.1, seed=27)
    F.rough((d > 0) & (d < 6.5), 0.3, 28)
    ink = (d > 6) & (d <= 7)
    F.H = np.where(ink, 0.5, F.H)
    win = F.inset(11, 11, 52, 52)
    lip = (d > 7) & (win <= 0)
    F.H = np.where(lip, np.clip(-win, 0, 3) / 3 * 0.9, F.H)
    F.paint(lip, "stone", S.STONE[1])
    F.H = np.where(win > 0, -0.5, F.H)
    F.paint(win > 0, "well", S.WELL)
    # Rough-rock corner blocks with the planet's ring cut in and a rune set in the middle.
    for ox, oy in ((0, 0), (46, 0), (0, 46), (46, 46)):
        bd = F.inset(ox + 1, oy + 1, ox + 16, oy + 16)
        bm = bd > 0
        lx, ly = F.X - ox, F.Y - oy
        rn = fbm2(lx, ly, 91 + ox + oy, 0.4, 4)
        F.H = np.where(bm, 2.8 + 1.2 * F.prof(bd, 1.6) + 0.9 * (rn - 0.5), F.H)
        F.paint(bm, "rock", S.ROCK[2], jitter=0.25, seed=93)
        r = np.hypot(lx - 9.0, ly - 9.0)
        rg = bm & (np.abs(r - 5.5) < 0.55)
        F.H = np.where(rg, F.H - 0.7, F.H)
        F.paint(rg, "rock", S.ROCK[0])
        F.dome(ox + 9.0, oy + 9.0, 1.15, 0.6, "rune", S.RUNE[2], emit=1.0, rough=0.4)
    return F


def r_portrait():
    F = Field(32, 32, 8)
    d = F.inset(1, 1, 30, 30)
    F.H = np.where(d > 0, 1.6 * F.prof(d, 1.5), 0)
    F.paint(d > 0, "stone", S.STONE_MID, jitter=0.08, seed=29)
    F.rough(d > 0, 0.25, 30)
    lip = F.inset(5, 5, 26, 26)
    inl = (d > 2) & (lip <= 0)
    F.H = np.where(inl, np.clip(-lip, 0, 2) / 2 * 1.0, F.H)
    F.paint(inl, "stone", S.STONE[1])
    F.H = np.where(lip > 0, -0.4, F.H)
    F.paint(lip > 0, "well", S.WELL)
    for cx in (3.0, 29.0):
        for cy in (3.0, 29.0):
            stud(F, cx, cy, 1.0, 0.8)
    return F


def r_emblem():
    F = Field(48, 48)
    r = np.hypot(F.X - 24.0, F.Y - 24.0)
    rn = fbm2(F.X, F.Y, 101, 0.3, 4)
    disc = r < 23.0
    F.H = np.where(disc, 2.2 + 1.0 * F.prof(23.0 - r, 2.0) + 0.6 * (rn - 0.5), 0)
    F.paint(disc, "rock", S.ROCK[2], jitter=0.25, seed=103)
    F.groove(np.abs(r - 19.0), 0.6, 0.9, "rock", S.ROCK[0])
    F.groove(np.abs(r - 10.5), 0.5, 0.7, "rock", S.ROCK[0])
    band = (r > 11) & (r < 18.4)
    F.H = np.where(band, F.H - 0.3 + 0.2 * (rn - 0.5), F.H)
    well = r < 10
    F.H = np.where(well, 0.6 - 0.6 * F.prof(10 - r, 1.0), F.H)
    F.paint(r < 9.2, "well", S.WELL)
    ang = np.arctan2(F.Y - 24.0, F.X - 24.0)
    for k in range(12):
        a = np.radians(k * 30 - 90)
        along = (F.X - 24.0) * np.cos(a) + (F.Y - 24.0) * np.sin(a)
        across = np.abs(-(F.X - 24.0) * np.sin(a) + (F.Y - 24.0) * np.cos(a))
        m = (along > 13.0) & (along < 16.2) & (across < 0.55)
        F.H = np.where(m, F.H - 0.5, F.H)
        if k % 3 == 0:
            F.paint(m, "rune", S.RUNE[1], emit=0.9, rough=0.4)
        else:
            F.paint(m, "rock", S.ROCK[0])
    del ang
    F.dome(24.0, 24.0, 6.2, 4.0, "dot", "#1096c3", base=0.0, rough=0.25, emit=0.1)
    return F


def r_icon(name):
    F = Field(11, 11)
    rows = ui_art.ICONS[name]
    mark = np.zeros((11, 11))
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            mark[y, x] = 1.0 if ch == "#" else 0.0
    up = np.kron(mark, np.ones((KPX, KPX)))
    k = KPX // 2
    pad = np.pad(up, k, mode="edge")
    cs = pad.cumsum(0).cumsum(1)
    cs = np.pad(cs, ((1, 0), (1, 0)))
    n = 2 * k
    sm = (cs[n:, n:] - cs[:-n, n:] - cs[n:, :-n] + cs[:-n, :-n]) / (n * n)
    sm = sm[: up.shape[0], : up.shape[1]]
    F.H = 1.3 * np.sqrt(np.clip(sm, 0, 1))
    F.paint(up > 0.5, "stone", S.STONE[4], rough=0.6)
    F.paint(up <= 0.5, "stone", S.STONE[0])
    return F


def kit_fields():
    """name -> builder, in the drawn kit's order (ui_art.build())."""
    b = {"frame": r_frame, "lintel": r_lintel}  # the rock itself: render_rocks(), at each panel's size
    for st in ("idle", "hover", "selected", "disabled"):
        b[f"slot-{st}"] = (lambda s=st: r_slot(s))
    for rr in ("uncommon", "rare", "epic", "legendary"):
        b[f"slot-{rr}"] = (lambda r_=rr: r_slot("idle", r_))
    for st in ("idle", "hover", "pressed", "disabled", "focus"):
        b[f"button-{st}"] = (lambda s=st: r_button(s))
    b["tab-open"] = lambda: r_tab(True)
    b["tab-shut"] = lambda: r_tab(False)
    b["scroll-track"] = lambda: r_groove(8, 24, 4)
    b["scroll-thumb"] = r_scroll_thumb
    b["slider-track"] = lambda: r_groove(32, 8, 3)
    b["slider-knob"] = r_slider_knob
    b["check-off"] = lambda: r_check(False)
    b["check-on"] = lambda: r_check(True)
    b["tooltip"] = r_tooltip
    b["bar"] = r_bar
    for f in ("health", "stamina", "breath", "progress"):
        b[f"fill-{f}"] = (lambda f_=f: r_fill(f_))
    b["compass"] = r_compass
    b["compass-pointer"] = r_pointer
    b["compass-town"] = r_town
    b["map-frame"] = r_map_frame
    b["portrait"] = r_portrait
    b["emblem"] = r_emblem
    for n in ui_art.ICONS:
        b[f"icon-{n}"] = (lambda n_=n: r_icon(n_))
    return b


# ---- the timber kit (main menu, pause, settings) ---------------------------------------------
# The owner, 2026-11-14: "main menu I think should use the timber stuff, but the game I like the rock
# stuff", then "pause menu = timber too". The out-of-play menus use the first draft's oak set
# (ui_art.py --oak, docs/mockups/ui/oak/), rendered here the same way into rendered/oak/ and
# rendered/hires/oak/. Grain runs ALONG each piece (it varies only across it), knots sit in corners.
def grain(F, across, seed, lo, hi, dark=None, lines=0.85):
    """Oak albedo from a 1-D grain profile across the board: rings between lo and hi tones, with dark
    late-wood lines where the profile peaks."""
    g = fbm2(np.zeros_like(across), across, seed, 1.3, 3)
    w = np.sin(across * 5.0 + 3.0 * g) * 0.5 + 0.5
    col = hexlin(lo)[None, None, :] * (1 - w[..., None]) + hexlin(hi)[None, None, :] * w[..., None]
    if dark:
        col = np.where((w > lines)[..., None], hexlin(dark)[None, None, :], col)
    return col, w


def knots(F, seed, tone):
    """A few knots, corners only (so the stretching bands stay plain)."""
    f1, _ = voronoi(F.X, F.Y, seed, 7.0)
    k = (f1 < 0.8) & (F.cw > 0.6)
    F.H = np.where(k, F.H - 0.25 * (1 - f1 / 0.8), F.H)
    F.alb = np.where(k[..., None], hexlin(tone), F.alb)


def r_oak_frame():
    F = Field(48, 48, 14)
    d = F.inset(1, 1, 46, 46)
    F.H = moulding_profile(d)
    m = d > 0
    F.paint(m, "oak", S.WOOD[3], rough=0.65)
    col, w = grain(F, d, 201, S.WOOD[2], S.WOOD[3], S.WOOD[1])
    F.alb = np.where((m & (d < 8))[..., None], col, F.alb)
    F.H = F.H + 0.12 * (w - 0.5) * (d < 8)
    F.groove(F.ring_d(4, 4, 43, 43), 0.6, 0.0, "seam", S.SEAM)
    knots(F, 203, S.WOOD[1])
    fx, fy = fold(F)
    mitre = (np.abs(fx - fy) < 0.45) & (d < 7.6)
    F.H = np.where(mitre, F.H - 0.6, F.H)
    F.paint(mitre, "seam", S.SEAM)
    seam = (d > 7) & (d <= 8)
    F.H = np.where(seam, 0.6, F.H)
    F.paint(seam, "seam", S.SEAM)
    lip = F.inset(13, 13, 34, 34)
    inl = (d > 8) & (lip <= 0)
    F.H = np.where(inl, np.clip(-lip, 0, 4) / 4 * 1.1, F.H)
    F.paint(inl, "stone", S.STONE[1])
    F.H = np.where(lip > 0, 0.0, F.H)
    F.paint(lip > 0, "stone", S.FACE)
    br = ((fx > 1) & (fx < 7) & (fy > 1) & (fy < 3)) | ((fy > 1) & (fy < 7) & (fx > 1) & (fx < 3))
    F.H = np.where(br, 3.6, F.H)
    F.paint(br, "iron", S.IRON[3], **iron_kw())
    for cx, cy in ((6.0, 2.0), (2.0, 6.0)):
        for X0 in (cx, 48 - cx):
            for Y0 in (cy, 48 - cy):
                stud(F, X0, Y0, 0.95, 0.8)
    return F


OAK_PLAQUE = {"idle": S.WOOD[3], "hover": S.WOOD[4], "pressed": S.WOOD[3], "disabled": S.WOOD[0],
              "focus": S.WOOD[3]}  # disabled: WOOD[0], one step darker than the drawn plaque's WOOD_DIM[2],
# because the smooth render's grain lifts it to 4.4:1 against LABEL_DIM (measured, 2026-11-14)


def r_oak_button(state):
    F = Field(48, 22, 8)
    top = 2 if state == "pressed" else 1
    d = F.inset(1, top, 46, 20)
    m = d > 0
    if state == "pressed":
        F.H = np.where(m, 1.0 - 0.7 * F.prof(d, 2.0), 0)
    elif state == "disabled":
        F.H = np.where(m, 0.6 * F.prof(d, 2.0), 0)
    else:
        F.H = np.where(m, 1.7 * F.prof(d, 2.2), 0)
    body = OAK_PLAQUE[state]
    F.paint(m, "oak", body, rough=0.65)
    lo = {"hover": S.WOOD[3], "disabled": S.SEAM}.get(state, S.WOOD[2])
    col, w = grain(F, F.Y, 211, lo, body)
    # the face where the label sits keeps the body tone; grain shows on the rim and the corners
    rim = m & ((d < 3.2) | (F.cw > 0.5))
    F.alb = np.where(rim[..., None], col, F.alb)
    F.H = F.H + 0.1 * (w - 0.5) * m
    knots(F, 213, S.WOOD[1] if state != "disabled" else S.WOOD[0])
    F.groove(F.ring_d(4, top + 3, 43, 17), 0.7, 0.8 if state != "disabled" else 0.4, "seam", S.SEAM)
    if state != "disabled":
        mat = "brass" if state == "hover" else "iron"
        for cx, cy in ((3, top + 2), (45, top + 2), (3, 18), (45, 18)):
            stud(F, cx, cy, 0.95, 0.8, mat)
    if state == "focus":
        rm = (F.ring_d(1, 1, 46, 20) < 0.5) & ~((np.abs(F.X - 1.5) < 1) & (np.abs(F.Y - 1.5) < 1))
        F.paint(rm, "rune", S.RUNE[2], emit=0.9, rough=0.5)
    return F


def r_oak_tab(active):
    F = Field(40, 18, 7)
    y0 = 0 if active else 2
    body = S.WOOD[3] if active else S.WOOD[2]
    side = np.minimum.reduce([F.X - 1, 39 - F.X, F.Y - (y0 + 1)])
    if not active:
        side = np.minimum(side, 16 - F.Y)
    m = side > 0
    F.H = np.where(m, 1.5 * F.prof(side, 1.6), 0)
    F.paint(m, "oak", body, rough=0.65)
    col, w = grain(F, F.Y, 221, S.WOOD[2] if active else S.WOOD[1], body)
    band = m & ((F.Y < y0 + 6) | (F.cw > 0.5))
    F.alb = np.where(band[..., None], col, F.alb)
    F.groove(np.abs(F.Y - (y0 + 3.5)) + np.where((F.X > 4) & (F.X < 36), 0, 9), 0.6, 0.6, "seam", S.SEAM)
    if not active:
        foot = (F.Y >= 16) & (F.X > 0) & (F.X < 40)
        F.H = np.where(foot, 0.2, F.H)
        F.paint(foot, "seam", S.SEAM)
    return F


def r_oak_scroll_thumb():
    F = Field(8, 16, 4)
    F.slab(1, 1, 6, 14, 1.5, 1.2, "oak", S.WOOD[3], rough=0.65)
    col, _ = grain(F, F.X, 231, S.WOOD[2], S.WOOD[3])
    F.alb = col
    for yy in (2, 13):
        m = (F.Y >= yy) & (F.Y < yy + 1) & (F.X > 1) & (F.X < 7)
        F.H = np.where(m, F.H + 0.35, F.H)
        F.paint(m, "iron", S.IRON[3] if yy == 2 else S.IRON[2], **iron_kw())
    return F


def r_oak_slider_knob():
    F = Field(9, 13)
    F.slab(1, 1, 7, 11, 1.6, 1.2, "oak", S.WOOD[3], rough=0.65)
    col, _ = grain(F, F.X, 241, S.WOOD[2], S.WOOD[3])
    F.alb = col
    cap = F.inset(1, 1, 7, 3)
    F.H = np.where(cap > 0, 2.0 + 0.5 * F.prof(cap, 0.8), F.H)
    F.paint(cap > 0, "iron", S.IRON[3], **iron_kw())
    for yy in (5, 7, 9):
        F.groove(np.hypot(np.maximum(np.abs(F.X - 4.5) - 1.0, 0), F.Y - (yy + 0.5)), 0.55, 0.5, "seam", S.SEAM)
    return F


def r_oak_tooltip():
    F = Field(24, 18, 6)
    d = F.inset(1, 1, 22, 16)
    F.H = np.where(d > 0, 1.6 * F.prof(d, 1.4), 0)
    F.paint(d > 0, "oak", S.WOOD[3], rough=0.65)
    col, _ = grain(F, d, 251, S.WOOD[2], S.WOOD[3])
    F.alb = np.where((d > 0)[..., None], col, F.alb)
    lip = F.inset(5, 5, 18, 12)
    inl = (d > 2) & (lip <= 0)
    F.H = np.where(inl, np.clip(-lip, 0, 2) / 2 * 1.0, F.H)
    F.paint(inl, "stone", S.STONE[1])
    F.paint(inl & (lip < -1.2), "seam", S.SEAM)
    F.H = np.where(lip > 0, 0.0, F.H)
    F.paint(lip > 0, "stone", S.FACE)
    for cx in (3.0, 21.0):
        for cy in (3.0, 15.0):
            stud(F, cx, cy, 0.85, 0.6)
    return F


def r_oak_portrait():
    F = Field(32, 32, 8)
    d = F.inset(1, 1, 30, 30)
    F.H = np.where(d > 0, 1.6 * F.prof(d, 1.5), 0)
    F.paint(d > 0, "oak", S.WOOD[3], rough=0.65)
    col, _ = grain(F, d, 261, S.WOOD[2], S.WOOD[3])
    F.alb = np.where((d > 0)[..., None], col, F.alb)
    lip = F.inset(5, 5, 26, 26)
    inl = (d > 2) & (lip <= 0)
    F.H = np.where(inl, np.clip(-lip, 0, 2) / 2 * 1.0, F.H)
    F.paint(inl, "seam", S.SEAM)
    F.H = np.where(lip > 0, -0.4, F.H)
    F.paint(lip > 0, "well", S.WELL)
    for cx in (3.0, 29.0):
        for cy in (3.0, 29.0):
            stud(F, cx, cy, 1.0, 0.8)
    return F


def r_oak_icon(name):
    """Cut into oak: the mark is a sunk dark cut, its lip lit (the drawn oak icons' look)."""
    F = r_icon(name)
    up = F.mat == MAT_IDS["stone"]
    mark = np.isclose(F.alb[..., 0], hexlin(S.STONE[4])[0])
    F.H = -F.H
    F.paint(mark, "seam", S.SEAM)
    F.paint(up & ~mark, "oak", S.WOOD[4], rough=0.65)
    return F


def oak_fields():
    b = {"frame": r_oak_frame}
    for st in ("idle", "hover", "pressed", "disabled", "focus"):
        b[f"button-{st}"] = (lambda s=st: r_oak_button(s))
    b["tab-open"] = lambda: r_oak_tab(True)
    b["tab-shut"] = lambda: r_oak_tab(False)
    b["scroll-thumb"] = r_oak_scroll_thumb
    b["slider-knob"] = r_oak_slider_knob
    b["tooltip"] = r_oak_tooltip
    b["portrait"] = r_oak_portrait
    for n in ui_art.ICONS:
        b[f"icon-{n}"] = (lambda n_=n: r_oak_icon(n_))
    return b


# ---- Blender ---------------------------------------------------------------------------------
def kit_scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"  # no GPU in the cloud container; on the owner's desktop use the GPU
    sc.cycles.samples = SAMPLES
    sc.cycles.seed = 1
    sc.cycles.use_denoising = False
    sc.cycles.max_bounces = 3
    sc.render.film_transparent = True
    sc.view_settings.view_transform = "Standard"
    sc.render.image_settings.file_format = "OPEN_EXR"
    sc.render.image_settings.color_mode = "RGBA"
    sc.render.image_settings.color_depth = "32"
    sc.render.resolution_percentage = 100
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    co = bpy.data.objects.new("cam", cam)
    sc.collection.objects.link(co)
    sc.camera = co
    sun = bpy.data.lights.new("sun", "SUN")
    sun.energy = 3.2
    sun.angle = math.radians(12)
    so = bpy.data.objects.new("sun", sun)
    # the base rock's sun: from the upper left of the screen, 50 degrees up
    so.rotation_euler = (math.radians(-40), math.radians(-28), 0.0)
    sc.collection.objects.link(so)
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.35
    sc.world = world
    mat = bpy.data.materials.new("kit")
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes["Principled BSDF"]
    a = nt.nodes.new("ShaderNodeAttribute")
    a.attribute_name = "albedo"
    e = nt.nodes.new("ShaderNodeAttribute")
    e.attribute_name = "mre"
    sep = nt.nodes.new("ShaderNodeSeparateColor")
    nt.links.new(a.outputs["Color"], bsdf.inputs["Base Color"])
    nt.links.new(a.outputs["Color"], bsdf.inputs["Emission Color"])
    nt.links.new(e.outputs["Color"], sep.inputs["Color"])
    nt.links.new(sep.outputs[0], bsdf.inputs["Metallic"])
    nt.links.new(sep.outputs[1], bsdf.inputs["Roughness"])
    nt.links.new(sep.outputs[2], bsdf.inputs["Emission Strength"])
    return sc, co, mat


def field_mesh(F, mat):
    h, w = F.X.shape
    # Relief is modelled in native px and exaggerated 1.7x for the render: at 1 px a bevel has to
    # turn a whole tone after the snap, which a true-scale relief under a soft sky does not.
    verts = np.stack([F.X.ravel(), -F.Y.ravel(), RELIEF * F.H.ravel()], 1)
    idx = np.arange(h * w).reshape(h, w)
    a, b = idx[:-1, :-1].ravel(), idx[:-1, 1:].ravel()
    c, d = idx[1:, 1:].ravel(), idx[1:, :-1].ravel()
    faces = np.stack([a, d, c, b], 1)
    me = bpy.data.meshes.new("piece")
    me.vertices.add(len(verts))
    me.vertices.foreach_set("co", verts.astype(np.float32).ravel())
    me.loops.add(len(faces) * 4)
    me.loops.foreach_set("vertex_index", faces.astype(np.int32).ravel())
    me.polygons.add(len(faces))
    me.polygons.foreach_set("loop_start", (np.arange(len(faces)) * 4).astype(np.int32))
    me.polygons.foreach_set("loop_total", np.full(len(faces), 4, np.int32))
    me.update()
    for name, arr in (("albedo", F.alb), ("mre", F.mre)):
        at = me.color_attributes.new(name, "FLOAT_COLOR", "POINT")
        rgba = np.concatenate([arr.reshape(-1, 3), np.ones((h * w, 1))], 1)
        at.data.foreach_set("color", rgba.astype(np.float32).ravel())
    me.shade_smooth()
    ob = bpy.data.objects.new("piece", me)
    bpy.context.scene.collection.objects.link(ob)
    ob.data.materials.append(mat)
    # A skirt plane under it so rims have something to cast shadow on and the edge is not a cliff
    return ob


def render_field(sc, co, mat, F, path):
    ob = field_mesh(F, mat)
    sc.render.resolution_x = F.w * KPX
    sc.render.resolution_y = F.h * KPX
    co.data.ortho_scale = max(F.w, F.h)
    co.location = (F.w / 2, -F.h / 2, 60)
    sc.render.filepath = path
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(path)
    px = np.array(img.pixels[:], np.float32).reshape(F.h * KPX, F.w * KPX, 4)[::-1]
    bpy.data.images.remove(img)
    bpy.data.objects.remove(ob)
    return px[..., :3]  # linear, scene-referred


def calibrate(sc, co, mat):
    """The base rock's light is not normalised; every piece renders under it as it is."""
    return 1.0


TMP = os.environ.get("UI_BLEND_TMP", "/tmp/ui_blend")


def lin2srgb(lin):
    lin = np.clip(lin, 0, 1)
    return np.where(lin <= 0.0031308, lin * 12.92, 1.055 * np.power(lin, 1 / 2.4) - 0.055)


DRAWN_REF = {}  # a rendered variant that takes its silhouette from another drawn piece


def drawn_piece(name):
    im = np.asarray(Image.open(os.path.join(OUT, DRAWN_REF.get(name, name) + ".png")).convert("RGBA"))
    return im


def pixelate_field(F, lin, name):
    """Area-average in linear light to native size; snap each pixel to its own material's ramp; keep
    the drawn silhouette and its INK; make the stretching sides plain."""
    w, h = F.w, F.h
    blk = lin.reshape(h, KPX, w, KPX, 3).mean((1, 3))
    s = lin2srgb(blk) * 255.0
    nm = len(RAMPS)
    onehot = np.zeros((h, w, nm))
    mm = F.mat.reshape(h, KPX, w, KPX)
    for i in range(nm):
        onehot[..., i] = (mm == i).sum((1, 3))
    maj = onehot.argmax(-1)
    names = list(RAMPS)
    out = np.zeros((h, w, 4), np.uint8)
    wts = np.array([0.3, 0.59, 0.11])
    for i in np.unique(maj):
        pal = np.array([rgb(c) for c in RAMPS[names[i]]], float)
        sel = maj == i
        dist = (((s[sel][:, None, :] - pal[None]) ** 2) * wts).sum(-1)
        out[sel, :3] = pal[dist.argmin(-1)].astype(np.uint8)
    drawn = drawn_piece(name)
    solid = drawn[..., 3] > 0
    ink = solid & np.all(drawn[..., :3] == rgb(INKC), -1)
    if getattr(F, "alpha", None) is not None:
        # a piece with its own modelled silhouette (the classic rock): alpha cut at one half, a 1 px
        # INK outline round it, as the first render was pixelated
        solid = F.alpha.reshape(h, KPX, w, KPX).mean((1, 3)) >= 0.5
        pad = np.pad(solid, 1)
        inner = pad[:-2, 1:-1] & pad[2:, 1:-1] & pad[1:-1, :-2] & pad[1:-1, 2:]
        ink = solid & ~inner
        ink[0, :] |= solid[0, :]; ink[-1, :] |= solid[-1, :]; ink[:, 0] |= solid[:, 0]; ink[:, -1] |= solid[:, -1]
    out[solid, 3] = 255
    out[~solid] = 0  # no INK outline: the render's own colours to its edge (owner: "the original didn't have a black outline")
    flat = plain_sides(F, out)
    return out, flat


def plain_sides(F, out):
    """Stretching bands and the centre made plain (the 9-slice rule); returns how many pixels it
    changed, reported so nothing is hidden."""
    if not F.slices:
        return 0
    t, r, b, l = F.slices
    h, w = out.shape[:2]
    before = out.copy()

    def mode(px):
        keys, counts = np.unique(px.reshape(-1, 4), axis=0, return_counts=True)
        return keys[counts.argmax()]

    # A tiling piece (the rock) keeps its periodic sides; only its centre is made one colour (the
    # rim's shadow falls a pixel or two past the slice line, which would tile into a grid).
    tile = F.sides == "tile"
    for y in ([] if tile else list(range(0, t)) + list(range(h - b, h))):
        seg = out[y, l:w - r]
        if seg.size:
            out[y, l:w - r] = mode(seg)
    for x in ([] if tile else list(range(0, l)) + list(range(w - r, w))):
        seg = out[t:h - b, x]
        if seg.size:
            out[t:h - b, x] = mode(seg)
    c = out[t:h - b, l:w - r]
    if c.size:
        out[t:h - b, l:w - r] = mode(c)
    return int((before != out).any(-1).sum())


def smooth_piece(F, lin, name):
    """The 8x render as the Smooth look: sRGB, the drawn silhouette scaled up and softened by a
    pixel at that scale, the INK outline kept as the drawn one (softened the same way)."""
    srgb = (lin2srgb(lin) * 255).round().astype(np.uint8)
    drawn = drawn_piece(name)
    a = np.kron((drawn[..., 3] > 0).astype(float), np.ones((KPX, KPX)))
    ink = np.kron(np.all(drawn[..., :3] == rgb(INKC), -1) & (drawn[..., 3] > 0), np.ones((KPX, KPX))).astype(float)
    if getattr(F, "alpha", None) is not None:  # its own silhouette, and no drawn outline (as the first render)
        a, ink = F.alpha.astype(float), np.zeros_like(F.alpha, float)

    def blur(m, k=3):
        p = np.pad(m, k // 2, mode="edge")
        cs = np.pad(p.cumsum(0).cumsum(1), ((1, 0), (1, 0)))
        return (cs[k:, k:] - cs[:-k, k:] - cs[k:, :-k] + cs[:-k, :-k]) / (k * k)

    a, ink = blur(a), blur(ink)
    rgbf = srgb.astype(float)  # no outline (as the base rock)
    if F.slices:  # the centre stretches (or, for the rock, tiles): one colour, its median
        t, r, b, l = [v * KPX for v in F.slices]
        H_, W_ = rgbf.shape[:2]
        c = rgbf[t:H_ - b, l:W_ - r]
        if c.size:
            rgbf[t:H_ - b, l:W_ - r] = np.median(c.reshape(-1, 3), 0)
        if F.sides != "tile":  # stretching bands: each row (top, bottom) or column (sides) one colour
            for ys in (slice(0, t), slice(H_ - b, H_)):
                seg = rgbf[ys, l:W_ - r]
                if seg.size:
                    rgbf[ys, l:W_ - r] = np.median(seg, 1)[:, None, :]
            for xs in (slice(0, l), slice(W_ - r, W_)):
                seg = rgbf[t:H_ - b, xs]
                if seg.size:
                    rgbf[t:H_ - b, xs] = np.median(seg, 0)[None, :, :]
    outp = np.dstack([rgbf, a * 255]).round().clip(0, 255).astype(np.uint8)
    return outp


def contrast_hex(a, b):
    return ui_art.contrast(a, b)


def lum_rgb(c):
    v = np.asarray(c, float) / 255
    v = np.where(v <= 0.04045, v / 12.92, ((v + 0.055) / 1.055) ** 2.4)
    return 0.2126 * v[..., 0] + 0.7152 * v[..., 1] + 0.0722 * v[..., 2]


def cratio(l1, l2):
    hi, lo = np.maximum(l1, l2), np.minimum(l1, l2)
    return (hi + 0.05) / (lo + 0.05)


# Where text sits on each piece (native px rects, inclusive) and which token is read there.
TEXT_AREAS = [
    ("rock", (14, 14, 33, 33), ["TEXT", "TEXT_DIM", "TITLE"]),
    ("frame", (14, 14, 33, 33), ["TEXT", "TITLE"]),
    ("tooltip", (6, 6, 17, 11), ["TEXT"]),
    ("slot-idle", (5, 5, 18, 18), ["TEXT"]),
    ("button-idle", (8, 8, 39, 13), ["LABEL"]),
    ("button-hover", (8, 8, 39, 13), ["LABEL"]),
    ("button-pressed", (8, 9, 39, 13), ["LABEL"]),
    ("button-disabled", (8, 8, 39, 13), ["LABEL_DIM"]),
    ("button-focus", (8, 8, 39, 13), ["LABEL"]),
    ("tab-open", (7, 7, 32, 10), ["LABEL"]),
    ("tab-shut", (7, 7, 32, 10), ["TAB_DIM"]),
    ("compass", (10, 3, 69, 10), ["CUT", "NORTH"]),
    ("lintel", (9, 5, 30, 10), ["OAK_LABEL"]),
    ("oak/frame", (14, 14, 33, 33), ["TEXT", "TITLE"]),
    ("oak/tooltip", (6, 6, 17, 11), ["TEXT"]),
    ("oak/button-idle", (8, 8, 39, 13), ["OAK_LABEL"]),
    ("oak/button-hover", (8, 8, 39, 13), ["OAK_LABEL"]),
    ("oak/button-pressed", (8, 9, 39, 13), ["OAK_LABEL"]),
    ("oak/button-disabled", (8, 8, 39, 13), ["LABEL_DIM"]),
    ("oak/button-focus", (8, 8, 39, 13), ["OAK_LABEL"]),
    ("oak/tab-open", (7, 7, 32, 10), ["OAK_LABEL"]),
    ("oak/tab-shut", (7, 7, 32, 10), ["LABEL"]),
]


def step_face(px, name):
    x0, y0, x1, y1 = text_area(name)
    darker = fix_direction(name) < 1
    out = px.copy()
    for ramp in ("stone",):
        pal = [tuple(rgb(c)) for c in RAMPS[ramp]]
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                c = tuple(int(v) for v in out[y, x, :3])
                if c in pal:
                    i = pal.index(c)
                    j = max(0, i - 1) if darker else min(len(pal) - 1, i + 1)
                    out[y, x, :3] = pal[j]
    return out


def text_area(name):
    for nm, rect, _ in TEXT_AREAS:
        if nm == name:
            return rect
    return None


def fix_direction(name):
    """Light text on a dark face: darken the face; dark text (the compass's cut letters, the lintel's
    and the oak plaques' titles) on a light face: lighten it."""
    for nm, _, toks in TEXT_AREAS:
        if nm == name:
            dark = any(t in ("CUT", "NORTH", "OAK_LABEL") for t in toks)
            return 1.15 if dark else 0.85
    return 1.0


def measure(name, px, scale):
    """Contrast of each token against the piece's text area: the area's commonest colour, and the
    worst colour that covers at least a tenth of it (the text can land on that too)."""
    for nm, (x0, y0, x1, y1), toks in TEXT_AREAS:
        if nm != name:
            continue
        area = px[y0 * scale:(y1 + 1) * scale, x0 * scale:(x1 + 1) * scale, :3].reshape(-1, 3)
        keys, counts = np.unique(area, axis=0, return_counts=True)
        if scale > 1:  # smooth: judge the area's mean and its 10th/90th percentile luminance
            L = lum_rgb(area)
            cands = [np.percentile(L, 10), np.percentile(L, 50), np.percentile(L, 90)]
        else:
            cands = [lum_rgb(k) for k, c in zip(keys, counts) if c >= 0.1 * counts.sum()]
        res = {}
        for t in toks:
            lt = lum_rgb(rgb(getattr(ui_art, t)))
            res[t] = round(float(min(cratio(lt, c) for c in cands)), 2)
        return res
    return None


# The rough-rock frame of every in-play panel, rendered at the panel's own native size so its rim,
# lip and the shadow the lip casts never tile (owner, 2026-11-14: "also shadow tiling can be better";
# the base rock was approved rendered this way). Name: (width, height) in native px, borders included.
# docs/mockups/ui/game-ui.js reads these from rendered/slices.json ("rock_panels") and sizes each panel
# exactly; the drawn look still draws the 48 px drawn slab as a 9-slice.
ROCK_PANELS = {
    "pack": (268, 152), "station": (230, 156), "chest": (372, 168), "talk": (270, 124), "trade": (308, 148),
    "loading": (150, 148), "sample": (192, 96), "kit-48": (48, 48), "kit-128": (128, 64), "kit-200": (200, 100),
}


ROCK_CONTRAST = {}


def render_rocks(only=""):
    """Each sized rock panel: the approved base rock (rock_field, no strata rings) under its own light
    (scene()), ROCK_PX render pixels a native pixel. Smooth: the render itself, its own alpha edge, no
    outline. Pixelated: area-averaged in linear light to native size and snapped to the rock ramp, the
    render's alpha cut at one half, no outline. The 48 x 48 one is also the kit's rock piece."""
    global PX
    import time
    os.makedirs(os.path.join(RENDERED, "rock"), exist_ok=True)
    os.makedirs(os.path.join(HIRES, "rock"), exist_ok=True)
    os.makedirs(TMP, exist_ok=True)
    keep_px, PX = PX, ROCK_PX
    times = {}
    try:
        for name, (w, h) in ROCK_PANELS.items():
            if only and name not in only.split(","):
                continue
            t0 = time.time()
            sc = scene(w, h)
            build_mesh("rock", *rock_field(w, h, strata_rings=False), W=w, H=h)
            raw = os.path.join(TMP, f"rock-{w}x{h}.png")
            sc.render.filepath = raw
            bpy.ops.render.render(write_still=True)
            im = np.asarray(Image.open(raw).convert("RGBA"))
            Image.fromarray(im, "RGBA").save(os.path.join(HIRES, "rock", f"rock-{w}x{h}.png"), optimize=True)
            a = im[..., 3] / 255.0
            lin = srgb_to_lin(im[..., :3].astype(float))
            pre = (lin * a[..., None]).reshape(h, PX, w, PX, 3).mean((1, 3))
            am = a.reshape(h, PX, w, PX).mean((1, 3))
            blk = lin2srgb(pre / np.maximum(am[..., None], 1e-6)) * 255
            pal = np.array([rgb(c) for c in RAMPS["rock"]], float)
            dist = (((blk[:, :, None, :] - pal[None, None]) ** 2) * np.array([0.3, 0.59, 0.11])).sum(-1)
            out = np.zeros((h, w, 4), np.uint8)
            out[..., :3] = pal[dist.argmin(-1)].astype(np.uint8)
            out[..., 3] = np.where(am >= 0.5, 255, 0)
            save(Image.fromarray(out, "RGBA"), os.path.join(RENDERED, "rock", f"rock-{w}x{h}.png"), source="tools/art/ui_panels_blend.py")
            if (w, h) == (48, 48):
                save(Image.fromarray(out, "RGBA"), os.path.join(RENDERED, "rock.png"), source="tools/art/ui_panels_blend.py")
                Image.fromarray(im, "RGBA").save(os.path.join(HIRES, "rock.png"), optimize=True)
            times[name] = round(time.time() - t0, 1)
            # text on the face: the centre's colour, pixelated and smooth, against the face tokens
            for key, img, k in (("px", out, 1), ("smooth", im, PX)):
                c = img[16 * k:(h - 16) * k, 16 * k:(w - 16) * k, :3].reshape(-1, 3)
                if len(c):
                    L = lum_rgb(c)
                    worst = {t: round(float(min(cratio(lum_rgb(rgb(getattr(ui_art, t))), q) for q in (np.percentile(L, 10), np.percentile(L, 90)))), 2) for t in ("TEXT", "TEXT_DIM", "TITLE")}
                    ROCK_CONTRAST.setdefault(name, {})[key] = worst
            print(f"rock {name:8s} {w}x{h}  {times[name]}s")
    finally:
        PX = keep_px
    return times


def render_kit(only=""):
    os.makedirs(HIRES, exist_ok=True)
    os.makedirs(TMP, exist_ok=True)
    import time
    sc, co, mat = kit_scene()
    k = calibrate(sc, co, mat)
    print(f"calibration: flat face x{k:.3f}")
    slices = json.load(open(os.path.join(OUT, "slices.json")))
    oak_slices = json.load(open(os.path.join(OUT, "oak", "slices.json")))
    for n, v in oak_slices["pieces"].items():
        slices["pieces"]["oak/" + n] = v
    os.makedirs(os.path.join(RENDERED, "oak"), exist_ok=True)
    os.makedirs(os.path.join(HIRES, "oak"), exist_ok=True)
    builders = dict(kit_fields())
    builders.update({"oak/" + n: b for n, b in oak_fields().items()})
    report = {"source": "tools/art/ui_panels_blend.py", "renderer": "Cycles (bpy 4.5), CPU, no GPU",
              "samples": SAMPLES, "render_px_per_native_px": KPX, "pieces": {}, "contrast": {},
              "contrast_smooth": {}, "plain_sides_changed_px": {}, "render_seconds": {}}
    meta_path = os.path.join(RENDERED, "slices.json")
    if only and os.path.exists(meta_path):
        report = json.load(open(meta_path))
    for name, build in builders.items():
        if only and name not in only.split(","):
            continue
        t0 = time.time()
        face_k = 1.0
        for attempt in range(10):
            F = build()
            area = text_area(name)
            if area and face_k != 1.0:
                # a text pair under 4.5:1: darken (or lighten) the face the text sits on, never drop the rule
                x0, y0, x1, y1 = area
                m = (F.X >= x0) & (F.X < x1 + 1) & (F.Y >= y0) & (F.Y < y1 + 1)
                F.alb[m] = F.alb[m] * face_k
            lin = render_field(sc, co, mat, F, os.path.join(TMP, name.replace("/", "-") + ".exr")) * k
            px, changed = pixelate_field(F, lin, name)
            sm_try = smooth_piece(F, lin, name)
            c_try = [measure(name, px, 1), measure(name, sm_try, KPX)]
            worst = min([v for c in c_try if c for v in c.values()] or [99])
            if worst >= 4.5:
                break
            if attempt >= 5 and c_try[0] and min(c_try[0].values()) < 4.5 and (not c_try[1] or min(c_try[1].values()) >= 4.5):
                # the pixelated face sits on the wrong side of a ramp step (the sky's sheen greys it);
                # move the face one step along its ramp, the pixel art's own way of changing its value
                px = step_face(px, name)
                c_try[0] = measure(name, px, 1)
                if min(c_try[0].values()) >= 4.5:
                    break
            face_k *= fix_direction(name)
        report.setdefault("face_adjust", {})[name] = round(face_k, 3)
        dt = time.time() - t0
        save(Image.fromarray(px, "RGBA"), os.path.join(RENDERED, f"{name}.png"), source="tools/art/ui_panels_blend.py")
        sm = sm_try
        Image.fromarray(sm, "RGBA").save(os.path.join(HIRES, f"{name}.png"), optimize=True)
        report["pieces"][name] = slices["pieces"][DRAWN_REF.get(name, name)]
        report["render_seconds"][name] = round(dt, 1)
        report["plain_sides_changed_px"][name] = changed
        c1 = measure(name, px, 1)
        if c1:
            report["contrast"][name] = c1
            report["contrast_smooth"][name] = measure(name, sm, KPX)
        print(f"{name:18s} {dt:5.1f}s  plain-sides {changed:4d} px  {c1 or ''}")
    # The selection rune on the well, and the icon inlay on a face, as in the drawn kit.
    if not only or "rock" in only:
        report["rock_render_seconds"] = render_rocks(arg("--rocks", ""))
    report["rock_panels"] = {k: list(v) for k, v in ROCK_PANELS.items()}
    if ROCK_CONTRAST:
        report["rock_contrast"] = ROCK_CONTRAST
    report["rock_px"] = ROCK_PX
    report["light"] = "the approved base rock's (Euler -40, -28, 0; 3.2; sky 0.35; roughness 0.7)"
    report["tokens"] = slices["tokens"]
    report["ramps"] = slices["ramps"]
    report["palette"] = "Sweaft-64"
    with open(meta_path, "w") as f:
        json.dump(report, f, indent=1)
    bad = [(n, t, v) for key in ("contrast", "contrast_smooth") for n, d in report[key].items() if d
           for t, v in d.items() if v < 4.5]
    for b_ in bad:
        print("CONTRAST BELOW 4.5:1", b_)
    return report, bad


def main():
    if "--legacy" in sys.argv:
        return legacy_main()

    only = arg("--only", "")
    _, bad = render_kit(only)
    if bad:
        sys.exit(f"contrast below 4.5:1: {bad}")


if __name__ == "__main__":
    main()
