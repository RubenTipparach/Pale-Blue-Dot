"""Render the waterfall mist puff sheet in Blender, headless (`water-flow` 6.5).

The owner, 2026-11-16: "design better mist instead of just a gradient circle
puff, use blender/krita or whatever, look at crab breeze smoke puffs as a
model". Crab Breeze's `smoke_puffs` / `smoke_lit` are a 4 x 4 sheet of lumpy
puffs, shaded and with normals so the game can light them. This is the same
idea built from geometry: each puff is a cluster of metaballs, roughened by a
displacement, lit from above, rendered face-on.

Rows are four puffs (variants); columns are four frames of one puff's life:
tight and dense as it leaves the foot, then swelling, then breaking up into
wisps. Two renders: `lit.png` (white puffs under a sun from above and a dim
sky) and `normal.png` (the camera-space normal as colour). `mist_puffs_post.py`
cuts them to the game's 32-texel cells.

    Blender -b -P tools/fx/mist_puffs_blender.py -- <out dir>
"""

import math
import random
import sys

import bpy
from mathutils import Vector

OUT = sys.argv[sys.argv.index("--") + 1] if "--" in sys.argv else "/tmp/mist_puffs"
CELL = 2.4          # metres a sheet cell spans in the render
GRID = 4
PX = 160            # render pixels a cell

scene = bpy.context.scene
for o in list(bpy.data.objects):
    bpy.data.objects.remove(o, do_unlink=True)


def puff(variant, frame, x, y):
    """One puff: a metaball cluster at (x, y), the variant its seed, the frame its age 0..3."""
    rnd = random.Random(1000 + variant * 17)
    mb = bpy.data.metaballs.new(f"puff{variant}{frame}")
    mb.resolution = 0.035
    mb.render_resolution = 0.03
    age = frame / (GRID - 1)
    # Lumps: the same seeds every frame, so one puff swells and breaks apart.
    n = 14 + variant % 3 * 3
    lumps = []
    for i in range(n):
        a = rnd.uniform(0, math.tau)
        r = rnd.uniform(0.0, 0.62) ** 0.8
        h = rnd.uniform(-0.38, 0.42)
        s = rnd.uniform(0.30, 0.52)
        drift = Vector((math.cos(a), 0.6 + h, math.sin(a))) * rnd.uniform(0.05, 0.22)
        lumps.append((Vector((math.cos(a) * r, h, math.sin(a) * r * 0.6)), s, drift, rnd.random()))
    for centre, s, drift, k in lumps:
        # With age the puff swells, rises a little and thins: the smallest
        # lumps shrink away first, so it frays rather than falling apart.
        fade = 1.0 - age * (0.25 + 0.7 * k)
        if fade <= 0.08:
            continue
        el = mb.elements.new()
        p = centre * (1.0 + 0.30 * age) + drift * age
        el.co = (p.x, p.z, p.y)
        el.radius = s * (1.0 + 0.25 * age) * (0.55 + 0.45 * fade)
        el.stiffness = 1.6 + 0.8 * fade
    mb.threshold = 0.55 + 0.35 * age
    ob = bpy.data.objects.new(f"puff{variant}{frame}", mb)
    ob.location = (x, 0.0, y)
    scene.collection.objects.link(ob)
    return ob


for v in range(GRID):
    for f in range(GRID):
        x = (f - (GRID - 1) / 2) * CELL
        y = ((GRID - 1) / 2 - v) * CELL
        puff(v, f, x, y)

# Convert the metaballs to meshes and roughen them: cauliflower billows.
bpy.ops.object.select_all(action="DESELECT")
for o in [o for o in scene.objects if o.type == "META"]:
    o.select_set(True)
bpy.context.view_layer.objects.active = [o for o in scene.objects if o.type == "META"][0]
bpy.ops.object.convert(target="MESH")
tex = bpy.data.textures.new("billow", type="CLOUDS")
tex.noise_scale = 0.11
tex.noise_depth = 2
tex.noise_basis = "VORONOI_F1"
for o in [o for o in scene.objects if o.type == "MESH"]:
    m = o.modifiers.new("billow", "DISPLACE")
    m.texture = tex
    m.strength = 0.14
    m.mid_level = 0.5
    s = o.modifiers.new("smooth", "SMOOTH")
    s.factor = 0.5
    s.iterations = 2
    o.data.shape_keys  # noqa: B018 - keep the mesh data alive

# Materials: a white diffuse for the lit pass, the normal as colour for the other.
lit = bpy.data.materials.new("lit")
lit.use_nodes = True
bsdf = lit.node_tree.nodes["Principled BSDF"]
bsdf.inputs["Base Color"].default_value = (0.92, 0.95, 1.0, 1.0)
bsdf.inputs["Roughness"].default_value = 1.0
nrm = bpy.data.materials.new("normal")
nrm.use_nodes = True
nt = nrm.node_tree
for n in list(nt.nodes):
    nt.nodes.remove(n)
geo = nt.nodes.new("ShaderNodeNewGeometry")
xf = nt.nodes.new("ShaderNodeVectorTransform")
xf.vector_type = "NORMAL"
xf.convert_from = "WORLD"
xf.convert_to = "CAMERA"
mapr = nt.nodes.new("ShaderNodeVectorMath")
mapr.operation = "MULTIPLY_ADD"
mapr.inputs[1].default_value = (0.5, 0.5, -0.5)  # camera space looks down -z: toward the eye is blue
mapr.inputs[2].default_value = (0.5, 0.5, 0.5)
emit = nt.nodes.new("ShaderNodeEmission")
out = nt.nodes.new("ShaderNodeOutputMaterial")
nt.links.new(geo.outputs["Normal"], xf.inputs["Vector"])
nt.links.new(xf.outputs["Vector"], mapr.inputs[0])
nt.links.new(mapr.outputs["Vector"], emit.inputs["Color"])
nt.links.new(emit.outputs["Emission"], out.inputs["Surface"])

# Camera: orthographic, face-on, looking along +Y.
cam_data = bpy.data.cameras.new("cam")
cam_data.type = "ORTHO"
cam_data.ortho_scale = CELL * GRID
cam = bpy.data.objects.new("cam", cam_data)
cam.location = (0.0, -20.0, 0.0)
cam.rotation_euler = (math.radians(90), 0.0, 0.0)
scene.collection.objects.link(cam)
scene.camera = cam

# Light: a sun from above, a little to the left and toward the camera; a dim
# blue sky fill so the undersides are not black.
sun_data = bpy.data.lights.new("sun", "SUN")
sun_data.energy = 4.5
sun_data.angle = math.radians(12)
sun = bpy.data.objects.new("sun", sun_data)
sun.rotation_euler = (math.radians(-35), math.radians(-20), 0.0)
scene.collection.objects.link(sun)
world = bpy.data.worlds.new("w")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (0.42, 0.48, 0.58, 1.0)
world.node_tree.nodes["Background"].inputs[1].default_value = 0.4
scene.world = world

scene.render.engine = "CYCLES"
scene.cycles.device = "CPU"
scene.cycles.samples = 48
scene.cycles.use_denoising = True
scene.render.film_transparent = True
scene.render.resolution_x = PX * GRID
scene.render.resolution_y = PX * GRID
scene.render.image_settings.file_format = "PNG"
scene.render.image_settings.color_mode = "RGBA"
scene.render.image_settings.color_depth = "16"
scene.view_settings.view_transform = "Standard"


def render(material, name):
    for o in [o for o in scene.objects if o.type == "MESH"]:
        o.data.materials.clear()
        o.data.materials.append(material)
    scene.render.filepath = f"{OUT}/{name}.png"
    bpy.ops.render.render(write_still=True)


render(lit, "lit")
scene.cycles.samples = 8
scene.cycles.use_denoising = False
render(nrm, "normal")
