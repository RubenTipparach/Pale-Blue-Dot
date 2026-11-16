"""Render the Pale Blue Dot logo in Blender (bpy 4.5, Cycles, headless).

Reads the stone's height field from `logo_height.py` (field.npz), builds it as one mesh coloured by
its masks (slate, the cut letters, the cave paintings in red ochre, lichen, dirt in the cuts), sets
the planet marble (a glassy sphere carrying a screenshot of the planet from the game, `--view
orbit`) in the emblem's socket and the moon in DOT's, lights it from the upper left, and renders
face-on with an orthographic camera on a transparent film.

    <bpy venv>/bin/python tools/art/logo/logo_render.py <field dir> <planet.png> <out.png> [samples]
"""

import math
import os
import sys

import bpy
import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import logo_height as LH  # noqa: E402  (the layout: where the marble and the moon stand)

field_dir, planet_png, out_png = sys.argv[1:4]
samples = int(sys.argv[4]) if len(sys.argv) > 4 else 64
STEP = 2                                   # px per mesh vertex
U = 100.0                                  # px per Blender unit

f = np.load(os.path.join(field_dir, "field.npz"))
h, mask = f["h"], f["mask"]
letters, glyphs, cavity, lichen, ticks = f["letters"], f["glyphs"], f["cavity"], f["lichen"], f["ticks"]
Hh, Ww = h.shape

bpy.ops.wm.read_factory_settings(use_empty=True)
scene = bpy.context.scene

# ---------------- the stone mesh ----------------
hs, ms = h[::STEP, ::STEP], mask[::STEP, ::STEP]
gh, gw = hs.shape
idx = -np.ones((gh, gw), np.int64)
keep = ms
idx[keep] = np.arange(keep.sum())
ys, xs = np.nonzero(keep)
verts = np.stack([xs * STEP / U, -(ys * STEP) / U, hs[keep]], 1).astype(np.float32)
q = keep[:-1, :-1] & keep[1:, :-1] & keep[1:, 1:] & keep[:-1, 1:]
qy, qx = np.nonzero(q)
faces = np.stack([idx[qy, qx], idx[qy + 1, qx], idx[qy + 1, qx + 1], idx[qy, qx + 1]], 1)

mesh = bpy.data.meshes.new("stone")
mesh.vertices.add(len(verts))
mesh.vertices.foreach_set("co", verts.ravel())
mesh.loops.add(faces.size)
mesh.loops.foreach_set("vertex_index", faces.ravel().astype(np.int32))
mesh.polygons.add(len(faces))
mesh.polygons.foreach_set("loop_start", (np.arange(len(faces)) * 4).astype(np.int32))
mesh.polygons.foreach_set("loop_total", np.full(len(faces), 4, np.int32))
mesh.update()
mesh.validate()
mesh.polygons.foreach_set("use_smooth", np.ones(len(faces), bool))

# vertex colours from the masks
rng = np.random.default_rng(3)
# a dark weathered patina on slate (the varnish of the owner's petroglyph wall), and the pale fresh
# stone a peck exposes
slate = np.array([0.06, 0.08, 0.105])
slate_hi = np.array([0.15, 0.18, 0.22])
pecked = np.array([0.50, 0.17, 0.07])        # red ochre rubbed into the frieze's cuts
lich = np.array([0.24, 0.27, 0.22])
var = (LH.fbm(120, 5, 61)[::STEP, ::STEP])[keep][:, None]
col = slate * (1 - var) + slate_hi * var
cav = cavity[::STEP, ::STEP][keep][:, None]
col = col * (1 - 0.55 * np.clip(cav * 1.6, 0, 1))
li = lichen[::STEP, ::STEP][keep][:, None]
col = col * (1 - 0.4 * li) + lich * 0.4 * li
gl = glyphs[::STEP, ::STEP][keep][:, None].astype(np.float32)
speck = rng.random((len(gl), 1)) * 0.35 + 0.65
col = col * (1 - 0.92 * gl) + pecked * speck * 0.92 * gl
le = letters[::STEP, ::STEP][keep][:, None].astype(np.float32)
# the raised letters: the stone's fresher face, a little lighter than the patina round them
col = col * (1 - le) + (np.array([0.32, 0.37, 0.42]) * (0.85 + 0.3 * var)) * le
# the emblem's tick marks in the menu emblem's mint
tk = ticks[::STEP, ::STEP][keep][:, None].astype(np.float32)
col = col * (1 - tk) + np.array([0.20, 0.62, 0.48]) * tk
rgba = np.concatenate([col, np.ones((len(col), 1))], 1).astype(np.float32)
attr = mesh.color_attributes.new("Col", "FLOAT_COLOR", "POINT")
attr.data.foreach_set("color", rgba.ravel())

stone = bpy.data.objects.new("stone", mesh)
scene.collection.objects.link(stone)
mat = bpy.data.materials.new("stone")
mat.use_nodes = True
nt = mat.node_tree
bsdf = nt.nodes["Principled BSDF"]
vc = nt.nodes.new("ShaderNodeVertexColor")
vc.layer_name = "Col"
nt.links.new(vc.outputs["Color"], bsdf.inputs["Base Color"])
bsdf.inputs["Roughness"].default_value = 0.82
# fine grain in the shading, below the mesh's resolution
tex = nt.nodes.new("ShaderNodeTexNoise")
tex.inputs["Scale"].default_value = 260.0
tex.inputs["Detail"].default_value = 8.0
bump = nt.nodes.new("ShaderNodeBump")
bump.inputs["Strength"].default_value = 0.18
bump.inputs["Distance"].default_value = 0.01
nt.links.new(tex.outputs["Fac"], bump.inputs["Height"])
nt.links.new(bump.outputs["Normal"], bsdf.inputs["Normal"])
stone.data.materials.append(mat)


def socket_floor(cx, cy):
    return float(h[int(cy), int(cx)])


# ---------------- the crystal ball ----------------
# (owner: "no clouds on planet for that crystal ball shot, I want the planet to be surrounded by
# space, a crystal ball with reflections around it"): inside, black space with stars and the planet
# (the game's own --view orbit shot, PBD_NO_CLOUDS=1) a little over half the ball across; outside,
# a clear glass coat that holds the lights' reflections and a cool rim.
from PIL import Image as PImage, ImageDraw as PDraw  # noqa: E402

ecx, ecy, _ = LH.EMBLEM
R = LH.MARBLE_R / U
BALL = 2048
src = PImage.open(planet_png).convert("RGB")
PCX, PCY, PR = float(os.environ.get("PLANET_CX", src.width / 2)), float(os.environ.get("PLANET_CY", src.height / 2)), float(os.environ.get("PLANET_R", src.width * 0.37))
glow = PR * 1.16                                  # the atmosphere's glow beyond the limb
crop = src.crop((int(PCX - glow), int(PCY - glow), int(PCX + glow), int(PCY + glow)))
inner = int(BALL * 0.70 * glow / PR / 2) * 2      # the planet's limb at 70% of the ball's width
crop = crop.resize((inner, inner), PImage.LANCZOS)
ball = PImage.new("RGB", (BALL, BALL), (2, 3, 6))
sr = np.random.default_rng(12)
dr = PDraw.Draw(ball)
for _ in range(900):
    x, y = sr.uniform(0, BALL, 2)
    v = int(sr.uniform(90, 255) * (1 if sr.random() < 0.85 else 1.0))
    rr = sr.choice([1, 1, 1, 2, 2, 3])
    dr.ellipse([x - rr, y - rr, x + rr, y + rr], fill=(v, v, min(255, v + 15)))
# a faint nebula, violet and blue, as in the owner's reference
from scipy import ndimage as _nd  # noqa: E402
neb = _nd.gaussian_filter(sr.random((BALL // 16, BALL // 16)), 2.5)
neb = np.clip((_nd.zoom(neb, 16, order=1) - neb.mean()) * 9, 0, 1)[..., None]
nb = np.asarray(ball, np.float32) / 255 + neb * np.array([0.10, 0.05, 0.20]) + neb ** 2 * np.array([0.02, 0.06, 0.16])
ball = PImage.fromarray((np.clip(nb, 0, 1) * 255).astype(np.uint8))
# the planet over the stars, its glow fading out (screen-blend so the black around it adds nothing)
pc = np.asarray(crop, np.float32) / 255
yy, xx = np.mgrid[0:inner, 0:inner]
d = np.hypot(xx - inner / 2, yy - inner / 2) / (inner / 2)
alpha = np.clip((1 - d) / 0.04, 0, 1)[..., None]
base = np.asarray(ball, np.float32) / 255
o = (BALL - inner) // 2
patch = base[o:o + inner, o:o + inner]
base[o:o + inner, o:o + inner] = 1 - (1 - patch) * (1 - pc * alpha)
# a blue halo round the planet, wider than its own atmosphere
yb, xb = np.mgrid[0:BALL, 0:BALL]
db = np.hypot(xb - BALL / 2, yb - BALL / 2) / (inner / 2 * PR / glow)
halo = np.exp(-np.clip(db - 1.0, 0, None) * 6.0) * (db > 0.97)
base = 1 - (1 - base) * (1 - halo[..., None] * np.array([0.22, 0.45, 1.0]) * 0.75)
ballp = os.path.join(os.path.dirname(os.path.abspath(out_png)), "ball.png")
PImage.fromarray((np.clip(base, 0, 1) * 255).astype(np.uint8)).save(ballp)

bpy.ops.mesh.primitive_uv_sphere_add(segments=128, ring_count=64, radius=R,
                                     location=(ecx / U, -ecy / U, socket_floor(ecx, ecy) + R * 0.55))
marble = bpy.context.object
bpy.ops.object.shade_smooth()
mm = bpy.data.materials.new("crystal")
mm.use_nodes = True
mt = mm.node_tree
mb = mt.nodes["Principled BSDF"]
img = mt.nodes.new("ShaderNodeTexImage")
img.image = bpy.data.images.load(ballp)
img.extension = "EXTEND"
coord = mt.nodes.new("ShaderNodeTexCoord")
mapn = mt.nodes.new("ShaderNodeMapping")
# object x, y in -R..R onto the image's 0..1, seen face-on; a slight lens: the middle magnified
mapn.inputs["Scale"].default_value = (0.5 / R, 0.5 / R, 1)
mapn.inputs["Location"].default_value = (0.5, 0.5, 0)
mt.links.new(coord.outputs["Object"], mapn.inputs["Vector"])
mt.links.new(mapn.outputs["Vector"], img.inputs["Vector"])
mb.inputs["Base Color"].default_value = (0.0, 0.0, 0.0, 1)
mt.links.new(img.outputs["Color"], mb.inputs["Emission Color"])
mb.inputs["Emission Strength"].default_value = 1.15
mb.inputs["Roughness"].default_value = 0.05
mb.inputs["Specular IOR Level"].default_value = 0.9
mb.inputs["Coat Weight"].default_value = 1.0
mb.inputs["Coat Roughness"].default_value = 0.09
mb.inputs["Coat IOR"].default_value = 1.6
# a cool rim, as thick glass shows at its edge
lw = mt.nodes.new("ShaderNodeLayerWeight")
lw.inputs["Blend"].default_value = 0.10
rim = mt.nodes.new("ShaderNodeEmission")
rim.inputs["Color"].default_value = (0.70, 0.85, 1.0, 1)
rim.inputs["Strength"].default_value = 3.2
add = mt.nodes.new("ShaderNodeAddShader")
mix = mt.nodes.new("ShaderNodeMixShader")
mt.links.new(lw.outputs["Fresnel"], mix.inputs["Fac"])
outn = mt.nodes["Material Output"]
mt.links.new(mb.outputs["BSDF"], add.inputs[0])
mt.links.new(rim.outputs["Emission"], add.inputs[1])
mt.links.new(mb.outputs["BSDF"], mix.inputs[1])
mt.links.new(add.outputs["Shader"], mix.inputs[2])
mt.links.new(mix.outputs["Shader"], outn.inputs["Surface"])
marble.data.materials.append(mm)
# the ball's own light on the ring round it
glowl = bpy.data.lights.new("ballglow", "POINT")
glowl.energy = 260.0
glowl.color = (0.35, 0.6, 1.0)
glowl.shadow_soft_size = R * 0.9
gl_o = bpy.data.objects.new("ballglow", glowl)
gl_o.location = (ecx / U, -ecy / U, socket_floor(ecx, ecy) + R * 1.05)
scene.collection.objects.link(gl_o)
marble.visible_shadow = False
# point lights for the glass to catch as glints (owner: "the multiple point light source idea was
# better"): small and bright, seen only in reflections
for name, loc, energy, col in (("glint1", (-3.2, 3.0, 4.0), 1400.0, (1.0, 0.97, 0.92)),
                               ("glint2", (3.6, 1.2, 3.4), 900.0, (1.0, 0.85, 0.65)),
                               ("glint3", (1.6, -3.4, 3.0), 500.0, (0.7, 0.85, 1.0)),
                               ("glint4", (-2.6, -2.2, 2.6), 350.0, (0.75, 0.8, 1.0))):
    lt = bpy.data.lights.new(name, "POINT")
    lt.energy = energy
    lt.color = col
    lt.shadow_soft_size = 0.06
    lo = bpy.data.objects.new(name, lt)
    lo.location = (ecx / U + loc[0], -ecy / U + loc[1], socket_floor(ecx, ecy) + loc[2])
    scene.collection.objects.link(lo)
    lo.visible_camera = False
    lo.visible_diffuse = False
    lo.visible_transmission = False
    lo.visible_volume_scatter = False

# ---------------- the moon ----------------
# (owner: "moon needs way more cratering"): three sizes of crater, each a bowl with a raised rim
# (Voronoi distance to the nearest centre), dark lunar seas from a low noise, floors a shade darker
mcx, mcy, mr = LH.MOON
Rm = mr / U
bpy.ops.mesh.primitive_uv_sphere_add(segments=128, ring_count=64, radius=Rm,
                                     location=(mcx / U, -mcy / U, socket_floor(mcx, mcy) + Rm * 0.6))
moon = bpy.context.object
bpy.ops.object.shade_smooth()
mo = bpy.data.materials.new("moon")
mo.use_nodes = True
ot = mo.node_tree
ob = ot.nodes["Principled BSDF"]
ob.inputs["Roughness"].default_value = 0.95
lk = ot.links


def node(kind, **inputs):
    n = ot.nodes.new(kind)
    for k, v in inputs.items():
        n.inputs[k].default_value = v
    return n


def math_(op, a, b=None, clamp=False):
    n = ot.nodes.new("ShaderNodeMath")
    n.operation = op
    n.use_clamp = clamp
    for i, x in enumerate((a, b)):
        if x is None:
            continue
        if isinstance(x, (int, float)):
            n.inputs[i].default_value = x
        else:
            lk.new(x, n.inputs[i])
    return n.outputs[0]


coordm = ot.nodes.new("ShaderNodeTexCoord")
height = None
floor = None
for scale, wgt, seed in ((3.5, 1.0, 1.0), (8.0, 0.55, 7.0), (19.0, 0.3, 13.0), (40.0, 0.15, 21.0)):
    v = node("ShaderNodeTexVoronoi", Scale=scale, Randomness=0.9)
    v.voronoi_dimensions = "3D"
    lk.new(coordm.outputs["Object"], v.inputs["Vector"])
    d = v.outputs["Distance"]
    bowl = math_("MULTIPLY", math_("MINIMUM", math_("DIVIDE", d, 0.32), 1.0), 1.0)
    rim = math_("MAXIMUM", math_("SUBTRACT", 1.0, math_("DIVIDE", math_("ABSOLUTE", math_("SUBTRACT", d, 0.34)), 0.07)), 0.0)
    hk = math_("MULTIPLY", math_("ADD", bowl, math_("MULTIPLY", rim, 0.6)), wgt)
    fk = math_("MULTIPLY", math_("SUBTRACT", 1.0, bowl), wgt)
    height = hk if height is None else math_("ADD", height, hk)
    floor = fk if floor is None else math_("ADD", floor, fk)
mbump = node("ShaderNodeBump", Strength=1.0, Distance=0.02)
lk.new(height, mbump.inputs["Height"])
lk.new(mbump.outputs["Normal"], ob.inputs["Normal"])
maria = node("ShaderNodeTexNoise", Scale=1.6, Detail=4.0)
lk.new(coordm.outputs["Object"], maria.inputs["Vector"])
tone = math_("SUBTRACT", math_("MULTIPLY", maria.outputs["Fac"], 1.0), math_("MULTIPLY", floor, 0.18))
ramp = ot.nodes.new("ShaderNodeValToRGB")
ramp.color_ramp.elements[0].position = 0.35
ramp.color_ramp.elements[0].color = (0.07, 0.07, 0.075, 1)
ramp.color_ramp.elements[1].position = 0.62
ramp.color_ramp.elements[1].color = (0.46, 0.45, 0.43, 1)
lk.new(tone, ramp.inputs["Fac"])
lk.new(ramp.outputs["Color"], ob.inputs["Base Color"])
moon.data.materials.append(mo)

# ---------------- light, camera, render ----------------
sun = bpy.data.lights.new("sun", "SUN")
sun.energy = 4.2
sun.angle = math.radians(2.5)
sun.color = (1.0, 0.95, 0.88)
so = bpy.data.objects.new("sun", sun)
so.rotation_euler = (math.radians(62), 0, math.radians(-135))   # from the upper left, low: long shadows
scene.collection.objects.link(so)
fill = bpy.data.lights.new("fill", "SUN")
fill.energy = 0.6
fill.color = (0.70, 0.80, 1.0)
fo = bpy.data.objects.new("fill", fill)
fo.rotation_euler = (math.radians(30), 0, math.radians(60))
scene.collection.objects.link(fo)
world = bpy.data.worlds.new("w")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs["Color"].default_value = (0.05, 0.06, 0.08, 1)
world.node_tree.nodes["Background"].inputs["Strength"].default_value = 1.0
scene.world = world

cam = bpy.data.cameras.new("cam")
cam.type = "ORTHO"
cam.ortho_scale = Ww / U
co = bpy.data.objects.new("cam", cam)
co.location = (Ww / U / 2, -Hh / U / 2, 20)
scene.collection.objects.link(co)
scene.camera = co

scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = samples
scene.cycles.use_denoising = True
scene.render.resolution_x = Ww
scene.render.resolution_y = Hh
scene.render.film_transparent = True
scene.render.image_settings.file_format = "PNG"
scene.render.image_settings.color_mode = "RGBA"
scene.view_settings.view_transform = "AgX"
scene.view_settings.look = "AgX - Medium High Contrast"
scene.render.filepath = os.path.abspath(out_png)
bpy.ops.render.render(write_still=True)
print("rendered", out_png)
