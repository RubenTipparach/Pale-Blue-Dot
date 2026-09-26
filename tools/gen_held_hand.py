#!/usr/bin/env python3
"""The hand that holds the tools, posed from Blender's Rigify human metarig.

Rigify ships a human skeleton with real hand bones: four palm bones, three
per finger and three for the thumb, at human proportions. This script loads
it, curls the right hand into a power grip round a handle of a given radius,
and writes the posed bones out as stubby hexagonal prisms in a frame on the
handle (`tools/held_hand.json`), which `gen_held_tools.py` places on each tool.

    <python with bpy> tools/gen_held_hand.py                 # write held_hand.json
    <python with bpy> tools/gen_held_hand.py --render DIR    # and views of the fist

Needs Blender's `bpy` with the bundled Rigify add-on.
"""

import json
import math
import os
import sys

import bpy  # first: it puts Blender's own modules on the path
import addon_utils
from mathutils import Matrix, Vector

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "tools", "held_hand.json")

# The handle the fist closes round, metres. Thinner than any tool's grip on
# purpose: `gen_held_tools.py` scales the hand so this lands on each tool's
# grip, and closing a human hand round a thin handle then scaling it up is
# what makes the hand in view big and stubby rather than realistically small.
HANDLE_R = 0.011
FINGERS = ("f_index", "f_middle", "f_ring", "f_pinky")
# Finger thickness, metres: the prisms' corner radius. A third thicker than a
# real finger, which is what makes the hand stubby rather than realistic.
FINGER_R = {"f_index": 0.0118, "f_middle": 0.0124, "f_ring": 0.0117, "f_pinky": 0.0102, "thumb": 0.0135}


def load_rig():
    bpy.ops.wm.read_factory_settings(use_empty=True)
    addon_utils.enable("rigify", default_set=True)
    bpy.ops.object.armature_human_metarig_add()
    rig = bpy.context.active_object
    bpy.ops.object.mode_set(mode="POSE")
    return rig


def bend(rig, name, angle, axis="X"):
    pb = rig.pose.bones[name]
    pb.rotation_mode = "XYZ"
    e = [0.0, 0.0, 0.0]
    e["XYZ".index(axis)] = angle
    pb.rotation_euler = e


def pose(rig, joints, thumb):
    """`joints`: the bend at each finger's three joints, knuckle to tip.
    `thumb`: the thumb's base turned about its X and Z, then its two outer
    joints."""
    for f in FINGERS:
        for i, a in enumerate(joints, start=1):
            bend(rig, f"{f}.0{i}.R", a)
    pb = rig.pose.bones["thumb.01.R"]
    pb.rotation_mode = "XYZ"
    pb.rotation_euler = (thumb[0], 0.0, thumb[1])
    bend(rig, "thumb.02.R", thumb[2])
    bend(rig, "thumb.03.R", thumb[3])
    bpy.context.view_layer.update()


def seg(rig, name):
    pb = rig.pose.bones[name]
    m = rig.matrix_world
    return m @ pb.head, m @ pb.tail


def circumcentre(a, b, c):
    ab, ac = b - a, c - a
    n = ab.cross(ac)
    if n.length < 1e-9:
        return (a + c) / 2
    return a + (n.cross(ab) * ac.length_squared + ac.cross(n) * ab.length_squared) / (2 * n.length_squared)


def fit_handle(rig):
    """The handle's axis through the fist: the centres of the four fingers'
    curls, and the mean clearance of the finger bones from that axis."""
    centres = []
    for f in FINGERS:
        k, _ = seg(rig, f"{f}.01.R")
        m, _ = seg(rig, f"{f}.02.R")
        _, t = seg(rig, f"{f}.03.R")
        centres.append(circumcentre(k, m, t))
    axis = (centres[0] - centres[-1]).normalized()  # pinky to index: toward the tool's head
    origin = sum(centres, Vector()) / len(centres)
    gaps = []
    for f in FINGERS:
        for i in (2, 3):
            h, t = seg(rig, f"{f}.0{i}.R")
            mid = (h + t) / 2
            off = mid - origin
            radial = off - axis * off.dot(axis)
            gaps.append(radial.length - FINGER_R[f])
    return origin, axis, sum(gaps) / len(gaps)


def radial(p, origin, axis):
    off = p - origin
    return (off - axis * off.dot(axis)).length


def finger_cost(rig):
    """How far the fingers are from hugging the handle: every joint and
    segment middle of every finger should sit one finger's radius off a
    handle of HANDLE_R, the knuckle included, so the palm is on the handle."""
    origin, axis, _ = fit_handle(rig)
    cost = 0.0
    for f in FINGERS:
        want = HANDLE_R + FINGER_R[f]
        for i in (1, 2, 3):
            h, t = seg(rig, f"{f}.0{i}.R")
            for p in (h, (h + t) / 2, t):
                cost += (radial(p, origin, axis) - want) ** 2
    return cost


def thumb_cost(rig):
    """The thumb wraps the handle and rests on the index finger's middle
    joint, without passing through the handle or the fingers."""
    origin, axis, _ = fit_handle(rig)
    tip = seg(rig, "thumb.03.R")[1]
    ih, it = seg(rig, "f_index.02.R")
    target = (ih + it) / 2
    # Onto the index finger from outside the fist, not inside it.
    out = target - origin
    out = out - axis * out.dot(axis)
    target = target + out.normalized() * (FINGER_R["f_index"] + FINGER_R["thumb"] * 0.8)
    cost = (tip - target).length ** 2 * 4
    for i in (1, 2, 3):
        h, t = seg(rig, f"thumb.0{i}.R")
        for p in (h, (h + t) / 2, t):
            clear = radial(p, origin, axis) - (HANDLE_R + FINGER_R["thumb"] * 0.9)
            if clear < 0:
                cost += clear * clear * 40
    return cost


def search(rig, fn, start, spans, rounds=5, steps=5):
    """Coordinate-wise grid refinement of `fn` over angles, radians."""
    best = list(start)
    for rd in range(rounds):
        for j in range(len(best)):
            span = spans[j] / (2 ** rd)
            trials = [best[j] + span * (k / (steps - 1) - 0.5) for k in range(steps)]
            scores = []
            for v in trials:
                cand = best[:]
                cand[j] = v
                fn(cand)
                scores.append((cand_cost(rig, fn.cost), v))
            best[j] = min(scores)[1]
    fn(best)
    return best


def cand_cost(rig, cost):
    return cost(rig)


def solve(rig):
    """The finger joints that close the fist round the handle, then the
    thumb that wraps it."""
    thumb = [0.0, 0.0, 0.3, 0.3]
    def set_fingers(j): pose(rig, j, thumb)
    set_fingers.cost = finger_cost
    joints = search(rig, set_fingers, [math.radians(60), math.radians(70), math.radians(50)],
                    [math.radians(90)] * 3, rounds=6, steps=7)
    def set_thumb(t): pose(rig, joints, t)
    set_thumb.cost = thumb_cost
    thumb = search(rig, set_thumb, [0.3, -0.4, 0.4, 0.4], [math.radians(140)] * 4, rounds=6, steps=9)
    return joints, thumb


def parts(rig):
    origin, axis, gap = fit_handle(rig)
    axis_world = axis
    hand_h, _ = seg(rig, "hand.R")
    fore_h, _ = seg(rig, "forearm.R")
    elbow = (fore_h - hand_h)
    f = (elbow - axis * elbow.dot(axis)).normalized()
    k = axis.cross(f)
    to_frame = Matrix((axis, f, k))  # rows: world -> (x, f, k)

    def local(p):
        return to_frame @ (p - origin)

    out = []

    def prism(a, b, r, c, extend=0.0, across=None, w=None):
        """A hexagonal prism from a to b. With `across` and `w` it is
        flattened: `w` wide along `across`, `r` thick across that."""
        a, b = local(a), local(b)
        d = (b - a)
        length = d.length + extend
        part = {"axis": [round(v, 5) for v in d.normalized()], "at": [round(v, 5) for v in (a + b) / 2],
                "r": round(r, 5), "len": round(length, 5), "c": c}
        if across is not None:
            across = to_frame @ across  # world direction into the hand's frame, as a and b are
            u = across - d.normalized() * across.dot(d.normalized())
            part["across"] = [round(v, 5) for v in u.normalized()]
            part["w"] = round(w, 5)
        out.append(part)

    # The back of the hand: one flat block from the wrist to the knuckles, as
    # wide as the four palm bones. Four separate bones read as four sticks.
    wrist_pt = seg(rig, "hand.R")[0]
    tails = [seg(rig, f"palm.0{i}.R")[1] for i in range(1, 5)]
    knuckles = sum(tails, Vector()) / 4
    width = (tails[0] - tails[3]).length / 2 + FINGER_R["f_middle"]
    prism(wrist_pt, knuckles, 0.017, "skin", 0.012, across=axis_world, w=width)
    # The fingers: three segments each, knuckles darker so the joints read.
    for f_name in FINGERS:
        for i in (1, 2, 3):
            h, t = seg(rig, f"{f_name}.0{i}.R")
            prism(h, t, FINGER_R[f_name] * (1.0 if i < 3 else 0.9), "skin" if i != 2 else "knuckle", 0.004)
    for i in (1, 2, 3):
        h, t = seg(rig, f"thumb.0{i}.R")
        prism(h, t, FINGER_R["thumb"] * (1.1 if i == 1 else 0.95 if i == 2 else 0.85), "skin" if i != 2 else "knuckle", 0.004)
    # The heel of the hand, then the wrist, cuff and sleeve running straight
    # toward the elbow (+f), square to the handle.
    h, t = seg(rig, "hand.R")
    elbow_dir = to_frame.transposed() @ Vector((0.0, 1.0, 0.0))
    wrist = h + elbow_dir * 0.035
    prism(h, wrist, 0.025, "skin")
    cuff = wrist + elbow_dir * 0.018
    prism(wrist, cuff, 0.033, "cuff")
    prism(cuff, cuff + elbow_dir * 0.3, 0.031, "sleeve")
    return out, gap


def render(rig, out_dir, handle_r):
    os.makedirs(out_dir, exist_ok=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    scene.cycles.samples = 24
    scene.cycles.use_denoising = False
    world = bpy.data.worlds.new("w")
    world.use_nodes = True
    world.node_tree.nodes["Background"].inputs[0].default_value = (0.35, 0.42, 0.5, 1)
    scene.world = world
    scene.view_settings.view_transform = "Standard"
    rig.hide_render = True
    data = json.load(open(OUT))
    skin = {"skin": (0.76, 0.43, 0.25, 1), "knuckle": (0.62, 0.34, 0.2, 1), "cuff": (0.05, 0.09, 0.16, 1),
            "sleeve": (0.07, 0.14, 0.26, 1), "wood": (0.25, 0.1, 0.03, 1)}
    mats = {}
    for k, v in skin.items():
        m = bpy.data.materials.new(k)
        m.use_nodes = True
        m.node_tree.nodes["Principled BSDF"].inputs["Base Color"].default_value = v
        mats[k] = m
    for i, p in enumerate(data["parts"] + [{"axis": [1, 0, 0], "at": [0, 0, 0], "r": handle_r, "len": 0.24, "c": "wood"}]):
        bpy.ops.mesh.primitive_cylinder_add(vertices=6, radius=p["r"], depth=p["len"])
        ob = bpy.context.active_object
        z = Vector(p["axis"]).normalized()
        xv = Vector(p.get("across") or z.orthogonal()).normalized()
        ob.matrix_world = Matrix.LocRotScale(Vector(p["at"]), Matrix((xv, z.cross(xv), z)).transposed().to_quaternion(),
                                             Vector((p.get("w", p["r"]) / p["r"], 1, 1)))
        ob.data.materials.append(mats[p["c"]])
    sun = bpy.data.objects.new("sun", bpy.data.lights.new("sun", "SUN"))
    sun.data.energy = 3.5
    sun.rotation_euler = (0.6, 0.4, 0.3)
    scene.collection.objects.link(sun)
    cam = bpy.data.objects.new("cam", bpy.data.cameras.new("cam"))
    cam.data.lens = 60
    scene.collection.objects.link(cam)
    scene.camera = cam
    scene.render.resolution_x = scene.render.resolution_y = 500
    # The frame: x along the handle, f toward the elbow, k across.
    views = {"back": (0, -0.05, 0.28), "front": (0, -0.05, -0.28), "knuckles": (0.05, -0.28, 0.06),
             "thumb-end": (0.28, -0.03, 0.05), "pinky-end": (-0.28, 0.0, 0.05), "three-quarter": (0.18, -0.16, 0.18)}
    for name, eye in views.items():
        cam.location = eye
        cam.rotation_mode = "QUATERNION"
        cam.rotation_quaternion = (Vector((0, 0.02, 0)) - Vector(eye)).to_track_quat("-Z", "Y")
        scene.render.filepath = os.path.join(out_dir, f"{name}.png")
        bpy.ops.render.render(write_still=True)


if __name__ == "__main__":
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else sys.argv[1:]
    rig = load_rig()
    joints, thumb = solve(rig)
    hand, gap = parts(rig)
    json.dump({"source": "Blender Rigify human metarig, right hand, posed by tools/gen_held_hand.py",
               "handle_r": HANDLE_R,
               "finger_joints_deg": [round(math.degrees(a), 1) for a in joints],
               "thumb_deg": [round(math.degrees(a), 1) for a in thumb],
               "frame": "x along the handle toward the tool's head, f toward the elbow, k = x cross f; metres",
               "parts": hand}, open(OUT, "w"), indent=1)
    print(f"finger joints {[round(math.degrees(a)) for a in joints]} deg, thumb {[round(math.degrees(a)) for a in thumb]} deg, "
          f"finger cost {finger_cost(rig) * 1e6:.1f} mm2, thumb cost {thumb_cost(rig) * 1e6:.1f} mm2, {len(hand)} parts")
    if "--render" in argv:
        render(rig, os.path.abspath(argv[argv.index("--render") + 1]), HANDLE_R)
