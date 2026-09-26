#!/usr/bin/env python3
"""The held tools and the hand holding them: one source for the mockup and
for Blender.

Each tool is a shape in tool units (x along the handle from the grip, y up the
drawing, z out of it) filled with hexes on a pointy-top grid whose row 0 lies on
y = 0, so a handle is whole rows and straight at any density. Each hex carries a
colour and a depth. The held pose puts the grip on an anchor in eye space
(x right, y up, -z ahead), runs the handle along a direction, and turns the tool
about its handle so its working end points where it should. The hand is hex
prisms in metres, laid out in a frame built on the grip.

    python3 tools/gen_held_tools.py                 # write the data into the mockup
    python3 tools/gen_held_tools.py --check         # fail if the mockup's data is stale
    <python with bpy> tools/gen_held_tools.py --render DIR   # Blender views of it

No third-party dependencies for the data; `--render` needs Blender's `bpy`.
"""

import json
import math
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MOCKUP = os.path.join(ROOT, "docs", "mockups", "held-tools.html")
BEGIN, END = "/*HELD-DATA*/", "/*END-HELD-DATA*/"
S3 = math.sqrt(3.0)
DENSITIES = (1, 2, 4)
SIZES = (1, 2)

COL = {
    "wood": "#8a5a2b", "woodDark": "#72471f", "grip": "#5e3a18", "wrap": "#7a4d22",
    "iron": "#c9d3d6", "ironDark": "#7d8a8f", "edge": "#eef3f4",
    "cork": "#b9a27a", "corkDark": "#9c865e", "reel": "#9aa6ab", "red": "#e35d4a",
}
SKINS = ["#e3b089", "#c98f64", "#8f5d3d", "#5c3b27"]
SLEEVE, CUFF = "#4d6a8c", "#3a526e"


# ---- small vector helpers (tuples) ------------------------------------------
def add(a, b): return tuple(x + y for x, y in zip(a, b))
def sub(a, b): return tuple(x - y for x, y in zip(a, b))
def mul(a, k): return tuple(x * k for x in a)
def dot(a, b): return sum(x * y for x, y in zip(a, b))
def cross(a, b): return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])
def norm(a):
    n = math.sqrt(dot(a, a))
    return mul(a, 1.0 / n)


def mat_cols(c0, c1, c2):
    """A 3x3 matrix as rows, from three columns."""
    return [[c0[i], c1[i], c2[i]] for i in range(3)]


def mat_mul(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def mat_t(a):
    return [[a[j][i] for j in range(3)] for i in range(3)]


def mat_vec(a, v):
    return tuple(sum(a[i][k] * v[k] for k in range(3)) for i in range(3))


def quat(m):
    """A unit quaternion (x, y, z, w) from a rotation matrix."""
    t = m[0][0] + m[1][1] + m[2][2]
    if t > 0:
        s = math.sqrt(t + 1.0) * 2
        w, x, y, z = 0.25 * s, (m[2][1] - m[1][2]) / s, (m[0][2] - m[2][0]) / s, (m[1][0] - m[0][1]) / s
    elif m[0][0] > m[1][1] and m[0][0] > m[2][2]:
        s = math.sqrt(1.0 + m[0][0] - m[1][1] - m[2][2]) * 2
        w, x, y, z = (m[2][1] - m[1][2]) / s, 0.25 * s, (m[0][1] + m[1][0]) / s, (m[0][2] + m[2][0]) / s
    elif m[1][1] > m[2][2]:
        s = math.sqrt(1.0 + m[1][1] - m[0][0] - m[2][2]) * 2
        w, x, y, z = (m[0][2] - m[2][0]) / s, (m[0][1] + m[1][0]) / s, 0.25 * s, (m[1][2] + m[2][1]) / s
    else:
        s = math.sqrt(1.0 + m[2][2] - m[0][0] - m[1][1]) * 2
        w, x, y, z = (m[1][0] - m[0][1]) / s, (m[0][2] + m[2][0]) / s, (m[1][2] + m[2][1]) / s, 0.25 * s
    return (x, y, z, w)


# ---- the shapes ---------------------------------------------------------------
def in_circle(x, y, cx, cy, r): return (x - cx) ** 2 + (y - cy) ** 2 <= r * r
def in_ellipse(x, y, cx, cy, rx, ry): return ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1
def clamp01(v): return max(0.0, min(1.0, v))


def rim(inside, x, y, w):
    """Near the edge of a region: some point `w` away is outside it."""
    return any(not inside(x + math.cos(k * math.pi / 4) * w, y + math.sin(k * math.pi / 4) * w) for k in range(8))


def round_depth(y, half, lo, hi):
    return lo + (hi - lo) * math.sqrt(clamp01(1 - (y / half) ** 2))


def handle(x, y, length, half, grip_len, r):
    if x < 0 or x > length or abs(y) > half:
        return None
    d = round_depth(y, half, 0.45, 0.95)
    if x < grip_len:
        return ("wrap" if (x % 0.9) < 0.2 else "grip", d + 0.08)
    # Grain on whole rows, so it runs straight along the handle.
    grain = abs(r) % 3 == 1 and math.fmod(math.floor(x * 1.7) * 7 + r * 3, 5) < 3
    return ("woodDark" if grain else "wood", d)


def pickaxe(x, y, r):
    def head(px, py): return in_circle(px, py, 7.5, 0, 4.7) and not in_circle(px, py, 6.4, 0, 4.6)
    if 10.3 <= x <= 11.7 and abs(y) <= 0.8:
        return ("ironDark", 1.15)
    if head(x, y):
        thin = 0.75 - 0.35 * clamp01((abs(y) - 2) / 2.6)
        if not in_circle(x, y, 7.5, 0, 4.45):
            return ("edge", thin * 0.8)
        if in_circle(x, y, 6.4, 0, 4.82):
            return ("ironDark", thin)
        return ("iron", thin)
    if -0.25 <= x < 0 and abs(y) <= 0.55:
        return ("grip", 1.0)
    return handle(x, y, 11, 0.45, 3, r)


def shovel(x, y, r):
    def blade(px, py):
        return in_ellipse(px, py, 12.6, 0, 2.8, 2.3) or (12.6 <= px <= 15.6 and abs(py) <= 2.3 * (15.6 - px) / 3.0)
    if blade(x, y):
        if abs(y) < 0.3 and x < 14.2:
            return ("ironDark", 0.75)
        if rim(blade, x, y, 0.3):
            return ("edge", 0.3)
        return ("iron", 0.4)
    if 9.2 <= x <= 10.8 and abs(y) <= 0.55 + (x - 9.2) * 0.3:
        return ("ironDark", 0.95)
    # A D-grip crossbar at the butt.
    if -0.9 <= x <= -0.1 and abs(y) <= 1.4:
        return ("wrap" if abs(y) > 1.1 else "grip", 0.9)
    return handle(x, y, 9.6, 0.42, 2.4, r)


def axe(x, y, r):
    def top(px): return 4.3 + 0.6 * (1 - ((px - 10.6) / 2.8) ** 2)
    def blade(px, py): return 1.0 <= py <= top(px) and 9.7 - (py - 1) * 0.48 <= px <= 11.5 + (py - 1) * 0.5
    if blade(x, y):
        d = 0.85 - 0.5 * clamp01((y - 1) / 3.6)
        if y > top(x) - 0.4:
            return ("edge", 0.3)
        if rim(blade, x, y, 0.25):
            return ("ironDark", d)
        return ("iron", d)
    if 9.4 <= x <= 11.8 and -1.0 <= y <= 1.0:
        return ("ironDark", 1.15)
    if 9.7 <= x <= 11.5 and -1.8 <= y < -1.0:
        return ("ironDark", 0.9)
    return handle(x, y, 12.4, 0.45, 3, r)


def rod(x, y, r):
    if 17.7 <= x <= 18.3 and abs(y) <= 0.22:
        return ("red", 0.4)
    # The reel hangs under the seat.
    if in_circle(x, y, 1.9, -1.45, 1.0):
        if in_circle(x, y, 1.9, -1.45, 0.35):
            return ("ironDark", 1.1)
        return ("ironDark" if rim(lambda px, py: in_circle(px, py, 1.9, -1.45, 1.0), x, y, 0.2) else "reel", 0.9)
    if 1.7 <= x <= 2.1 and -0.8 <= y < -0.4:
        return ("ironDark", 0.6)
    # Line guides: small rings on stems under the blank.
    for gx in (7, 11, 14.5, 16.8):
        r2 = (x - gx) ** 2 + (y + 0.78) ** 2
        if 0.14 ** 2 <= r2 <= 0.34 ** 2:
            return ("ironDark", 0.3)
        if abs(x - gx) <= 0.08 and -0.5 <= y <= -0.3:
            return ("ironDark", 0.3)
    if 1.2 <= x <= 2.6 and abs(y) <= 0.6:
        return ("ironDark", 0.95)
    if 0 <= x <= 3.8 and abs(y) <= 0.55:
        return ("corkDark" if (x % 0.6) < 0.12 else "cork", round_depth(y, 0.55, 0.5, 1.0))
    if 3.8 < x <= 4.1 and abs(y) <= 0.42:
        return ("ironDark", 0.8)
    half = 0.4 - 0.24 * clamp01(x / 18)
    if 4.1 < x <= 17.7 and abs(y) <= half:
        return ("grip", round_depth(y, half, 0.25, 0.75))
    return None


# Each tool: its shape, its length (grip to head along x), where the fist
# closes on it, and its held pose. `along` is the handle's direction from the
# grip in eye space and `length` its reach at 1x, metres. `work` is the working
# end, a direction in the tool's plane (tool space); the tool is turned about
# its handle so `work` points as nearly as it can at `toward` (eye space).
DOWN_AHEAD = norm((0.0, -1.0, -0.45))
TOOLS = [
    {"id": "pickaxe", "name": "Pickaxe", "note": "Stone, rock and ore", "shape": pickaxe, "len": 12.2, "fist": 1.5,
     "along": (-0.2, 0.45, -0.5), "length": 0.232, "work": (0, -1, 0), "toward": DOWN_AHEAD},
    {"id": "shovel", "name": "Shovel", "note": "Dirt, sand and snow", "shape": shovel, "len": 15.6, "fist": 1.2,
     "along": (-0.75, -0.2, -0.55), "length": 0.26, "work": (0, 0, 1), "toward": norm((0, 1, 0.35))},
    {"id": "axe", "name": "Axe", "note": "Wood", "shape": axe, "len": 12.4, "fist": 1.5,
     "along": (-0.2, 0.45, -0.5), "length": 0.232, "work": (0, 1, 0), "toward": DOWN_AHEAD},
    {"id": "rod", "name": "Rod", "note": "Fishing", "shape": rod, "len": 18.3, "fist": 3.0,
     "along": (0.0, 0.31, -0.47), "length": 0.563, "work": (0, -1, 0), "toward": (0.0, -1.0, 0.0)},
]
ANCHOR = {1: (0.26, -0.26, -0.48), 2: (0.34, -0.36, -0.5)}
X0, X1, Y1 = -2.2, 19.0, 6.2


def pos(q, r, s):
    """A hex's centre in tool units, y up."""
    return (s * (q + r / 2.0), -s * r * S3 / 2.0)


def generate(tool, density):
    s = 1.0 / density
    out = []
    r_max = math.ceil(Y1 / (s * S3 / 2))
    for r in range(-r_max, r_max + 1):
        for q in range(math.floor(X0 / s - r / 2) - 1, math.ceil(X1 / s - r / 2) + 2):
            x, y = pos(q, r, s)
            h = tool["shape"](x, y, r)
            if h:
                out.append([q, r, h[0], round(h[1], 2)])
    return out


def pose(tool, size):
    """Rotation (tool to eye), scale (metres per tool unit) and grip."""
    b1 = norm(tool["along"])
    toward = tool["toward"]
    bw = norm(sub(toward, mul(b1, dot(toward, b1))))
    a1, aw = (1.0, 0.0, 0.0), tuple(float(v) for v in tool["work"])
    a = mat_cols(a1, aw, cross(a1, aw))
    b = mat_cols(b1, bw, cross(b1, bw))
    rot = mat_mul(b, mat_t(a))
    return rot, tool["length"] * size / tool["len"], ANCHOR[size]


# ---- the hand -----------------------------------------------------------------
# In metres, in a frame on the grip: x along the handle toward the head, f
# toward the elbow (square to the handle), k = the third axis, turned to face
# the eye. Each part is a hexagonal prism: its axis in that frame, its centre,
# its corner radius and length, and a colour role.
FOREARM_EYE = norm((0.45, -0.75, 0.5))
# How much of the forearm's lie along the handle is taken out: all of it makes
# the arm square to the handle (a hammer grip, which from the eye runs the arm
# out sideways), none lets the arm continue the handle's line. Part of it is a
# cocked wrist, with the arm coming up from the bottom right of the view.
FOREARM_SQUARE = 0.6
HAND = [
    # The fist: a fat prism around the handle, carrying the back of the hand.
    {"axis": "x", "at": (0.0, 0.006, 0.0), "r": 0.040, "len": 0.074, "c": "skin"},
    # Four knuckles along the fist's far edge, on the side toward the eye.
    *[{"axis": "k", "at": (-0.027 + i * 0.018, -0.026, 0.020), "r": 0.0135, "len": 0.034, "c": "knuckle"} for i in range(4)],
    # The fingers' middle joints, wrapped round the far side of the handle.
    *[{"axis": "k", "at": (-0.027 + i * 0.018, -0.040, -0.004), "r": 0.0125, "len": 0.044, "c": "skin"} for i in range(4)],
    # The thumb: its base on the back of the hand, its tip along the handle.
    {"axis": "x", "at": (0.036, 0.004, 0.032), "r": 0.0155, "len": 0.030, "c": "skin"},
    {"axis": "x", "at": (0.060, -0.010, 0.026), "r": 0.0125, "len": 0.028, "c": "knuckle"},
    # The wrist, the cuff and the sleeve, toward the elbow.
    {"axis": "f", "at": (0.0, 0.050, 0.0), "r": 0.030, "len": 0.044, "c": "skin"},
    {"axis": "f", "at": (0.0, 0.078, 0.0), "r": 0.0385, "len": 0.020, "c": "cuff"},
    {"axis": "f", "at": (0.0, 0.318, 0.0), "r": 0.036, "len": 0.46, "c": "sleeve"},
]


def hand_parts(tool, size):
    """The hand's prisms in the tool's own units, for this tool and size."""
    rot, scale, _ = pose(tool, size)
    x = (1.0, 0.0, 0.0)
    d = mat_vec(mat_t(rot), FOREARM_EYE)
    f = norm(sub(d, mul(x, FOREARM_SQUARE * dot(d, x))))
    k = norm(cross(x, f))
    if mat_vec(rot, k)[2] < 0:
        k = mul(k, -1)
    axes = {"x": x, "f": f, "k": k}
    grip = (tool["fist"], 0.0, 0.0)
    out = []
    for p in HAND:
        at = add(grip, mul(add(add(mul(x, p["at"][0]), mul(f, p["at"][1])), mul(k, p["at"][2])), 1.0 / scale))
        out.append({"axis": [round(v, 4) for v in axes[p["axis"]]], "at": [round(v, 4) for v in at],
                    "r": round(p["r"] / scale, 4), "len": round(p["len"] / scale, 4), "c": p["c"]})
    return out


def data():
    tools = []
    for t in TOOLS:
        poses = {}
        for size in SIZES:
            rot, scale, grip = pose(t, size)
            poses[str(size)] = {"q": [round(v, 6) for v in quat(rot)], "scale": round(scale, 6), "grip": list(grip),
                                "hand": hand_parts(t, size)}
        tools.append({"id": t["id"], "name": t["name"], "note": t["note"], "len": t["len"],
                      "hexes": {str(d): generate(t, d) for d in DENSITIES}, "poses": poses})
    return {"colours": COL, "skins": SKINS, "sleeve": SLEEVE, "cuff": CUFF, "tools": tools}


def write_mockup(check):
    with open(MOCKUP, encoding="utf-8") as f:
        page = f.read()
    a, b = page.index(BEGIN) + len(BEGIN), page.index(END)
    blob = json.dumps(data(), separators=(",", ":"))
    fresh = page[:a] + blob + page[b:]
    if check:
        if fresh != page:
            sys.exit("the mockup's held-tool data is stale: run tools/gen_held_tools.py")
        print("held-tool data is current")
        return
    with open(MOCKUP, "w", encoding="utf-8") as f:
        f.write(fresh)
    print(f"wrote {len(blob)} bytes of held-tool data into {os.path.relpath(MOCKUP, ROOT)}")


# ---- Blender -------------------------------------------------------------------
NEIGHBOUR = [(1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1), (1, 0)]  # edge k faces 60 + 60k degrees, y up


def hex_mesh(hexes, s):
    """Vertices and faces (with a colour role each) of a tool's hexes."""
    R = s / S3
    cells = {(q, r): (c, d) for q, r, c, d in hexes}
    verts, faces = [], []

    def face(points, role):
        base = len(verts)
        verts.extend(points)
        faces.append((list(range(base, base + len(points))), role))

    for (q, r), (c, d) in cells.items():
        cx, cy = pos(q, r, s)
        half = d / 2
        corner = [(cx + math.cos(math.radians(30 + 60 * k)) * R, cy + math.sin(math.radians(30 + 60 * k)) * R) for k in range(6)]
        face([(x, y, half) for x, y in corner], c)
        face([(x, y, -half) for x, y in reversed(corner)], c)
        for k in range(6):
            nb = cells.get((q + NEIGHBOUR[k][0], r + NEIGHBOUR[k][1]))
            n_half = nb[1] / 2 if nb else -1
            if n_half >= half:
                continue
            (ax, ay), (bx, by) = corner[k], corner[(k + 1) % 6]
            spans = [(-half, half)] if n_half < 0 else [(n_half, half), (-half, -n_half)]
            for z0, z1 in spans:
                face([(ax, ay, z0), (bx, by, z0), (bx, by, z1), (ax, ay, z1)], c)
    return verts, faces


def render(out_dir):
    import bpy
    from mathutils import Matrix, Quaternion, Vector

    os.makedirs(out_dir, exist_ok=True)
    blob = data()
    skin = SKINS[0]
    roles = dict(COL, skin=skin, knuckle="#c4916d", cuff=CUFF, sleeve=SLEEVE)

    def srgb(h):
        v = [int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]
        return [c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4 for c in v] + [1.0]

    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 24
    scene.cycles.use_denoising = False
    scene.render.film_transparent = False
    world = bpy.data.worlds.new("sky")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.42, 0.55, 0.70, 1)
    world.node_tree.nodes["Background"].inputs[1].default_value = 0.8
    scene.world = world
    scene.view_settings.view_transform = "Standard"

    mats = {}

    def mat(role):
        if role not in mats:
            m = bpy.data.materials.new(role)
            m.use_nodes = True
            bsdf = m.node_tree.nodes["Principled BSDF"]
            bsdf.inputs["Base Color"].default_value = srgb(roles[role])
            bsdf.inputs["Roughness"].default_value = 0.8
            mats[role] = m
        return mats[role]

    sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
    sun.data.energy = 3.2
    sun.rotation_euler = (math.radians(-55), math.radians(25), 0)  # from above, a little behind
    scene.collection.objects.link(sun)

    ground = bpy.data.meshes.new("ground")
    ground.from_pydata([(-200, -1.7, 200), (200, -1.7, 200), (200, -1.7, -200), (-200, -1.7, -200)], [], [(0, 1, 2, 3)])
    g = bpy.data.objects.new("ground", ground)
    gm = bpy.data.materials.new("grass")
    gm.use_nodes = True
    gm.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = srgb("#4a6f31")
    ground.materials.append(gm)
    scene.collection.objects.link(g)

    def make_object(name, verts, faces):
        me = bpy.data.meshes.new(name)
        me.from_pydata(verts, [], [f for f, _ in faces])
        order = sorted({role for _, role in faces})
        for role in order:
            me.materials.append(mat(role))
        for poly, (_, role) in zip(me.polygons, faces):
            poly.material_index = order.index(role)
        me.validate()
        ob = bpy.data.objects.new(name, me)
        scene.collection.objects.link(ob)
        return ob

    def prism(p):
        axis = Vector(p["axis"]).normalized()
        q = Vector((0, 0, 1)).rotation_difference(axis)
        verts, faces = [], []
        for zi, z in enumerate((-p["len"] / 2, p["len"] / 2)):
            for k in range(6):
                a = math.radians(60 * k)
                v = q @ Vector((math.cos(a) * p["r"], math.sin(a) * p["r"], z)) + Vector(p["at"])
                verts.append(tuple(v))
        faces.append((list(range(0, 6))[::-1], p["c"]))
        faces.append((list(range(6, 12)), p["c"]))
        for k in range(6):
            k1 = (k + 1) % 6
            faces.append(([k, k1, 6 + k1, 6 + k], p["c"]))
        return verts, faces

    def held(tool, size, density=4):
        t = next(x for x in blob["tools"] if x["id"] == tool)
        pz = t["poses"][str(size)]
        verts, faces = hex_mesh(t["hexes"][str(density)], 1.0 / density)
        for p in pz["hand"]:
            v, f = prism(p)
            base = len(verts)
            verts += v
            faces += [([i + base for i in idx], role) for idx, role in f]
        ob = make_object(tool, verts, faces)
        x, y, z, w = pz["q"]
        ob.rotation_mode = "QUATERNION"
        ob.rotation_quaternion = Quaternion((w, x, y, z))
        ob.scale = (pz["scale"],) * 3
        ob.location = pz["grip"]
        return ob, t, pz

    cam = bpy.data.objects.new("eye", bpy.data.cameras.new("eye"))
    cam.data.sensor_fit = "VERTICAL"
    cam.data.angle_y = math.radians(70)
    cam.data.clip_start = 0.01
    scene.collection.objects.link(cam)
    scene.camera = cam

    def shoot(path, w=960, h=600):
        scene.render.resolution_x, scene.render.resolution_y = w, h
        scene.render.filepath = path
        bpy.ops.render.render(write_still=True)
        print("rendered", os.path.relpath(path, ROOT) if path.startswith(ROOT) else path)

    def look(at, eye):
        cam.location = eye
        direction = Vector(at) - Vector(eye)
        cam.rotation_mode = "QUATERNION"
        cam.rotation_quaternion = direction.to_track_quat("-Z", "Y")

    for t in blob["tools"]:
        ob, _, pz = held(t["id"], 2)
        # First person: the eye at the origin, looking down -z.
        cam.location = (0, 0, 0)
        cam.rotation_mode = "QUATERNION"
        cam.rotation_quaternion = Quaternion()
        shoot(os.path.join(out_dir, f"fp-{t['id']}.png"))
        if t["id"] == "pickaxe":
            # Round the hand: the fist's centre in eye space, from four sides.
            fist = Vector(pz["grip"]) + (ob.rotation_quaternion @ (Vector((next(x for x in TOOLS if x["id"] == "pickaxe")["fist"], 0, 0)) * pz["scale"]))
            for name, off in {"front": (0, 0, 0.32), "right": (0.32, 0.02, 0), "top": (0, 0.32, 0.02),
                              "left": (-0.3, 0.05, -0.08), "three-quarter": (0.2, 0.16, 0.2)}.items():
                look(fist, tuple(Vector(fist) + Vector(off)))
                cam.data.angle_y = math.radians(40)
                shoot(os.path.join(out_dir, f"hand-{name}.png"), 600, 600)
            cam.data.angle_y = math.radians(70)
        bpy.data.objects.remove(ob)


if __name__ == "__main__":
    if "--render" in sys.argv:
        render(os.path.abspath(sys.argv[sys.argv.index("--render") + 1]))
    else:
        write_mockup("--check" in sys.argv)
