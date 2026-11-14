"""The stone-and-wood UI panels modelled and rendered in Blender, then pixelated
(openspec/changes/stone-and-wood-ui, the "rendered" variant beside tools/art/ui_art.py's drawn one).

The method is the panel-textures / blender-organic-panels one, adapted (see those skills' "In Pale
Blue Dot" notes): each panel is ONE height field z = h(x, y) computed in numpy with masks (wood,
stone, iron, groove), written into a grid mesh whose colour attribute carries the material, lit
from the upper left by a sun and a weak sky, and rendered face-on orthographic in Cycles on a
transparent film. The render is then area-averaged in linear light down to the panel's native
pixel size, every pixel snapped to the nearest Sweaft-64 colour, alpha cut at one half and a 1 px
INK outline added, so it can stand beside the drawn kit at the same whole-number scale.

Two panels, at the drawn kit's sizes and slice margins:
  frame  48 x 48, slices 14: an oak moulding (bead, flat, groove, flat, fillet) with mitred corners
         and iron corner plates with rivets, round a dressed-stone face sunk behind a lip;
  rock   48 x 48, slices 14: a rough rock slab, fbm relief and chipped edges, with a dressed face.
Grain and strata run ALONG each edge (they vary only across it), so a stretched side is unchanged.

Needs the bpy module (Blender 4.5 as a Python wheel, Python 3.11) with numpy and Pillow:
    python3.11 -m venv /tmp/bpy && /tmp/bpy/bin/pip install "bpy==4.5.*" pillow numpy
    /tmp/bpy/bin/python tools/art/ui_panels_blend.py [--samples 24] [--px 10]
Writes docs/mockups/ui/blender-<name>.png (native, pixelated) and
docs/mockups/ui/blender-<name>-render.png (the smooth render, reference only).
In a cloud container this runs on the CPU (no GPU there); on the owner's desktop use the GPU.
"""

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


def grid():
    """Sample positions in native pixels, one per vertex, RES + 1 a side."""
    s = np.linspace(0, N, RES + 1)
    return np.meshgrid(s, s)


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


def rock_field():
    x, y = grid()
    d = np.minimum.reduce([x, y, N - x, N - y])
    rough = fbm(x, y, 7, 5, 0.12)
    rough2 = fbm(x, y, 11, 3, 0.5)
    rock = [hexlin(c) for c in ("#2b1f35", "#3e3551", "#69608b", "#92a3b2")]
    # Chipped silhouette: stronger chips toward the corners.
    corner = np.minimum(np.minimum(x, N - x), np.minimum(y, N - y))
    near_corner = np.clip(1 - np.maximum(np.minimum(x, N - x), np.minimum(y, N - y)) / 14.0, 0, 1)
    edge_cut = 0.6 + 2.6 * near_corner * fbm(x, y, 3, 3, 0.3)
    alpha = (d > edge_cut).astype(float)
    h = 2.0 + rough * 2.2 + rough2 * 0.5 - np.clip(2.5 - (d - edge_cut), 0, None) * 0.8
    v = np.clip((rough - 0.5) * 1.6 + 0.5, 0, 1)
    col = rock[1] * (1 - v[..., None]) + rock[2] * v[..., None]
    # Strata along each side (vary only across it).
    strata = np.sin(d * 3.1) > 0.92
    col[strata & (d < 9)] = rock[0]
    lip = (d >= 9.0) & (d < 12.0)
    h = np.where(lip, 0.9 - (d - 9.0) * 0.3, h)
    col[lip] = rock[1]
    face = d >= 12.0
    h = np.where(face, 0.0, h)
    col[face] = hexlin("#2c2c25")
    del corner
    return h, col, alpha


def build_mesh(name, h, col, alpha):
    n = RES + 1
    xs = np.linspace(-N / 2, N / 2, n)
    X, Y = np.meshgrid(xs, -xs)
    verts = np.stack([X.ravel(), Y.ravel(), h.ravel()], 1)
    keep = alpha.ravel() > 0.5
    idx = np.arange(n * n).reshape(n, n)
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
    rgba = np.concatenate([col.reshape(-1, 3), np.ones((n * n, 1))], 1)
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


def scene():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    sc = bpy.context.scene
    sc.render.engine = "CYCLES"
    sc.cycles.device = "CPU"
    sc.cycles.samples = SAMPLES
    sc.cycles.seed = 1
    sc.cycles.use_denoising = False
    sc.cycles.max_bounces = 2
    sc.render.resolution_x = sc.render.resolution_y = RES
    sc.render.film_transparent = True
    sc.view_settings.view_transform = "Standard"
    sc.render.image_settings.file_format = "PNG"
    sc.render.image_settings.color_mode = "RGBA"
    cam = bpy.data.cameras.new("cam")
    cam.type = "ORTHO"
    cam.ortho_scale = N
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


def main():
    os.makedirs(OUT, exist_ok=True)
    for name, field in (("frame", frame_field), ("rock", rock_field)):
        sc = scene()
        build_mesh(name, *field())
        render = os.path.join(OUT, f"blender-{name}-render.png")
        sc.render.filepath = render
        bpy.ops.render.render(write_still=True)
        pixelate(render, os.path.join(OUT, f"blender-{name}.png"))
        # Keep the reference render small: half size is plenty to judge the modelling.
        Image.open(render).resize((RES // 2, RES // 2), Image.LANCZOS).save(render, optimize=True)
        print("rendered", name, render)


if __name__ == "__main__":
    main()
