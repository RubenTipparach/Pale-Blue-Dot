//! The jungle's pieces (`cities-in-the-world` slice 4i): its kapoks, the
//! plank platforms round them, and the rope bridges between, sagging as the
//! mockup's do. Each is cut in its own frame from the template, as the
//! harbour's piers are, and never saved.

use super::super::chart::{Chart, Patch};
use super::super::{Kapok, Part, Platform, RopeBridge};
use super::dressing::{Place, both, cylinder};
use super::harbour::corners;
use super::{BuildingSolids, Frame, LIFT_M, Meshes, SLAB_M, Sink, Solid, Surface, ccw, rail};
use glam::{Vec2, Vec3};

/// A rope bridge's span, metres: the mockup's rope runs `L / 1.4`. Each is
/// cut flat in its own frame, its walk a ramp from the height at its start
/// to the height at its end.
const ROPE_SPAN_M: f32 = 1.4;
/// The mockup's slats: one every this many metres, 4 cm apart, 8 cm thick.
const SLAT_M: f32 = 0.34;
const SLAT_GAP_M: f32 = 0.04;
const SLAT_DEPTH_M: f32 = 0.08;
/// How far a span's walk reaches past its joints, metres, so the walker
/// finds no seam between two frames.
const SPAN_LAP_M: f32 = 0.05;
/// A bridge walk's depth under its top, for the walker.
const WALK_DEPTH_M: f32 = 0.12;
/// The ropes' heights over the walk, their depth, and how far out of the
/// walk's edge they run, metres: the mockup's `railSeg`.
const ROPE_HEIGHTS_M: [f32; 2] = [0.95, 0.5];
const ROPE_DEPTH_M: f32 = 0.07;
const ROPE_OUT_M: f32 = 0.05;
/// A post: its radius and height, the mockup's `post`.
const POST_R_M: f32 = 0.05;
const POST_H_M: f32 = 1.02;
/// A beam belongs to the kapok whose trunk it is nearest, within this.
const BEAM_REACH_M: f32 = 5.0;
/// How far a platform's floor reaches past its cells, metres. A piece cut
/// in a frame at another height (the pole tower's newel, from the ground)
/// meets it at the cell's edge seen from its own frame, a few centimetres
/// off at 8 m on a small body: the floor laps that, as a pier's stretches
/// lap their joints. Its outer edges are railed, so the lap is never
/// walked off.
const PLATFORM_LAP_M: f32 = 0.06;

/// A piece cut with no rooms, no doors and no lights.
fn piece(sink: Sink, reach_m: f32) -> BuildingSolids {
    BuildingSolids {
        frame: sink.frame,
        reach_m,
        solids: sink.solids,
        roof_plan: Vec::new(),
        surfaces: sink.surfaces,
        doors: Vec::new(),
        rooms: Meshes::new(),
        lights: Vec::new(),
        top_m: 0.0,
    }
}

/// A box the mockup drew, at its place in `place`'s frame, its foot `y0`
/// over the frame's floor; a solid too where the part is one. Returns how
/// far from the frame's origin it reaches in plan.
fn part(sink: &mut Sink, place: &Place, p: &Part, y0: f32) -> f32 {
    let c = place.plan(p.at[0], p.at[2]);
    let size = Vec3::from(p.size);
    let turn = place.turn(p.angle);
    sink.plain_box(&p.material, c.x, y0, c.y, size, turn);
    if let Some(h) = p.solid_m {
        sink.solid_box(c.x, y0, c.y, Vec3::new(size.x, h, size.z), turn);
    }
    c.length() + size.x.hypot(size.z) / 2.0
}

/// A kapok (the mockup's `kapok`): its trunk, a prism of twelve sides the
/// walker goes round, and the parts the mockup drew about it, the fins
/// solid to their own height. The beams under a platform round it are cut
/// with it: each of `beams` whose middle is within [`BEAM_REACH_M`] of its
/// trunk. `zero_m` is the template's 0 m over the radius.
#[allow(clippy::too_many_arguments)]
pub fn kapok(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    kapok: &Kapok,
    beams: &[Part],
    cell_m: f32,
    radius_m: f32,
    zero_m: f32,
) -> Option<BuildingSolids> {
    let place = Place::new(
        chart,
        patch,
        cell_m,
        (kapok.x, kapok.z),
        radius_m,
        zero_m + kapok.y,
    )?;
    let mut sink = Sink::new(meshes, repeat_m, place.frame);
    cylinder(
        &mut sink,
        "kapok",
        "kapok",
        place.plan(kapok.x, kapok.z),
        0.0,
        kapok.radius_m,
        kapok.height_m,
        12,
    );
    let mut far = kapok.radius_m;
    let near = |b: &&Part| (b.at[0] - kapok.x).hypot(b.at[2] - kapok.z) < BEAM_REACH_M;
    for p in kapok.parts.iter().chain(beams.iter().filter(near)) {
        far = far.max(part(&mut sink, &place, p, p.at[1] - kapok.y));
    }
    Some(piece(sink, far + 1.0))
}

/// A frame flat at the middle of some cells, its floor `floor_m` over the
/// radius, `x` along the first cell's edge 0.
fn frame_over(
    patch: &Patch,
    chart: &Chart,
    cells: &[[i32; 2]],
    radius_m: f32,
    floor_m: f32,
) -> Result<Frame, String> {
    let mut sum = Vec3::ZERO;
    for &[c, r] in cells {
        let at = chart
            .cells
            .get(&(c, r))
            .ok_or_else(|| format!("a platform's cell ({c}, {r}) is not charted"))?;
        sum += patch.cells[at.cell].direction;
    }
    let [c0, r0] = cells.first().ok_or("a platform with no cells")?;
    let at = &chart.cells[&(*c0, *r0)];
    let first = &patch.cells[at.cell];
    let y = sum.normalize();
    let along = (first.corners[at.d0] + first.corners[(at.d0 + 1) % first.corners.len()]) * 0.5
        - first.direction;
    let x = (along - y * along.dot(y)).normalize();
    Ok(Frame {
        origin: y * (radius_m + floor_m),
        x,
        y,
        z: x.cross(y),
    })
}

/// A cell's outline grown by `by` metres past each of its edges.
fn grown(hex: &[Vec2; 6], by: f32) -> Vec<Vec2> {
    let c = hex.iter().fold(Vec2::ZERO, |s, p| s + *p) / 6.0;
    let inner = (0..6)
        .map(|d| ((hex[d] + hex[(d + 1) % 6]) * 0.5).distance(c))
        .sum::<f32>()
        / 6.0;
    let k = 1.0 + by / inner.max(0.1);
    hex.iter().map(|p| c + (*p - c) * k).collect()
}

/// A platform (the mockup's `deck`): a plank slab on each of its cells, cut
/// from the cells' real corners with a floor for the walker, as a stilt
/// house's deck is, and a rail on each edge the template rails.
pub fn platform(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    platform: &Platform,
    radius_m: f32,
    zero_m: f32,
) -> Result<BuildingSolids, String> {
    let frame = frame_over(patch, chart, &platform.cells, radius_m, zero_m + platform.y)?;
    let mut sink = Sink::new(meshes, repeat_m, frame);
    let mut far = 0.0f32;
    for &[c, r] in &platform.cells {
        let hex = corners(patch, chart, &frame, c, r)
            .ok_or_else(|| format!("a platform's cell ({c}, {r}) is not charted"))?;
        far = hex.iter().map(|p| p.length()).fold(far, f32::max);
        sink.prism("plank", "timber", &hex, -SLAB_M, LIFT_M, Some("plank"));
        sink.surfaces.push(Surface::Floor {
            outline: ccw(grown(&hex, PLATFORM_LAP_M)),
            top: LIFT_M,
            bottom: -SLAB_M,
        });
    }
    for &[c, r, d] in &platform.rails {
        let hex = corners(patch, chart, &frame, c, r)
            .ok_or_else(|| format!("a platform's rail ({c}, {r}) is not charted"))?;
        let (a, b) = (hex[d as usize % 6], hex[(d as usize + 1) % 6]);
        let e = b - a;
        rail(&mut sink, (a + b) * 0.5, LIFT_M, e.length(), e.y.atan2(e.x));
    }
    Ok(piece(sink, far + 1.0))
}

/// A rope bridge (the mockup's `bridge` with sag), cut in spans of about
/// [`ROPE_SPAN_M`], each flat in its own frame at its middle: its walk a
/// ramp from the bridge's height at the span's start to its height at the
/// end, reaching a little past both, the mockup's slats on it, and two
/// ropes either side from post to post, each span's a wall solid to a metre
/// over the walk. The first and last spans' ropes end on the corners of the
/// platform edge the bridge comes in on, with a post at the far end. A
/// lantern's hanger is cut in the span it hangs from.
#[allow(clippy::too_many_arguments)]
pub fn rope_bridge(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    bridge: &RopeBridge,
    cell_m: f32,
    radius_m: f32,
    zero_m: f32,
) -> Option<Vec<BuildingSolids>> {
    let len = bridge.length();
    if len < 0.1 {
        return Some(Vec::new());
    }
    let (ux, uz) = (
        (bridge.to[0] - bridge.from[0]) / len,
        (bridge.to[2] - bridge.from[2]) / len,
    );
    let (nx, nz) = (-uz, ux);
    let ang = uz.atan2(ux);
    let w = bridge.width_m;
    let spans = ((len / ROPE_SPAN_M).round() as usize).max(2);
    let slats = ((len / SLAT_M).round() as usize).max(2);
    // The mockup's `corner`: of the end edge's two corners, the one on the
    // `s` side of the walk.
    let corner = |end: usize, s: f32| -> (f32, f32) {
        let [p0, p1] = bridge.ends[end];
        let c = if end == 0 { bridge.from } else { bridge.to };
        if ((p0[0] - c[0]) * nx + (p0[1] - c[2]) * nz) * s > 0.0 {
            (p0[0], p0[1])
        } else {
            (p1[0], p1[1])
        }
    };
    let side = |t: f32, s: f32| {
        let (x, z) = bridge.plan(t);
        let off = s * (w / 2.0 + ROPE_OUT_M);
        (x + nx * off, z + nz * off)
    };
    // A lantern's share along the bridge.
    let along = |l: &[f32; 3]| ((l[0] - bridge.from[0]) * ux + (l[2] - bridge.from[2]) * uz) / len;
    let mut out = Vec::with_capacity(spans);
    for k in 0..spans {
        let (t0, t1) = (k as f32 / spans as f32, (k + 1) as f32 / spans as f32);
        let mid = bridge.plan((t0 + t1) / 2.0);
        let place = Place::new(chart, patch, cell_m, mid, radius_m, zero_m)?;
        let mut sink = Sink::new(meshes, repeat_m, place.frame);
        let mut far = 0.0f32;
        // The walk, lapping its joints.
        let lap = SPAN_LAP_M / len;
        let (a, b) = (bridge.plan(t0 - lap), bridge.plan(t1 + lap));
        let (pa, pb) = (place.plan(a.0, a.1), place.plan(b.0, b.1));
        let dir = (pb - pa).normalize();
        sink.surfaces.push(Surface::Ramp {
            foot: pa,
            dir,
            len: pa.distance(pb),
            half_width: w / 2.0,
            from: bridge.height(t0 - lap),
            to: bridge.height(t1 + lap),
            depth: WALK_DEPTH_M,
        });
        far = far.max(pa.length()).max(pb.length());
        // Its slats: those whose middles are in this span.
        for j in 0..slats {
            let t = (j as f32 + 0.5) / slats as f32;
            if t < t0 || t >= t1 {
                continue;
            }
            let (x, z) = bridge.plan(t);
            let c = place.plan(x, z);
            sink.plain_box(
                "plank",
                c.x,
                bridge.height(t) - SLAT_DEPTH_M,
                c.y,
                Vec3::new(len / slats as f32 - SLAT_GAP_M, SLAT_DEPTH_M, w),
                place.turn(ang),
            );
        }
        // Its ropes, either side, from its post to the next.
        for s in [-1.0f32, 1.0] {
            let p0 = if k == 0 { corner(0, s) } else { side(t0, s) };
            let p1 = if k + 1 == spans {
                corner(1, s)
            } else {
                side(t1, s)
            };
            let (q0, q1) = (place.plan(p0.0, p0.1), place.plan(p1.0, p1.1));
            let (y0, y1) = (bridge.height(t0), bridge.height(t1));
            for hgt in ROPE_HEIGHTS_M {
                let pts = [
                    Vec3::new(q0.x, y0 + hgt, q0.y),
                    Vec3::new(q1.x, y1 + hgt, q1.y),
                    Vec3::new(q1.x, y1 + hgt + ROPE_DEPTH_M, q1.y),
                    Vec3::new(q0.x, y0 + hgt + ROPE_DEPTH_M, q0.y),
                ];
                let run = q0.distance(q1);
                let uv = move |p: Vec3| {
                    let u = Vec2::new(p.x, p.z).distance(q0) / run.max(1e-3);
                    Vec2::new(u * run, (p.y - y0 - hgt) * 10.0)
                };
                both(&mut sink, "rope", &pts, &uv);
            }
            cylinder(&mut sink, "timber", "timber", q0, y0, POST_R_M, POST_H_M, 5);
            if k + 1 == spans {
                cylinder(&mut sink, "timber", "timber", q1, y1, POST_R_M, POST_H_M, 5);
            }
            // The rope is a wall the walker is held by, as the mockup's
            // rail is: 10 cm thick, from under the walk to a metre over it.
            let e = (q1 - q0).normalize_or_zero();
            let n = Vec2::new(-e.y, e.x) * 0.05;
            sink.solids.push(Solid {
                outline: ccw(vec![q0 - n, q1 - n, q1 + n, q0 + n]),
                y0: y0.min(y1) - 0.2,
                y1: y0.max(y1) + 1.0,
            });
            far = far.max(q0.length()).max(q1.length());
        }
        // The hangers of the lanterns that hang in this span.
        for l in bridge
            .lanterns
            .iter()
            .filter(|l| (t0..t1).contains(&along(l)))
        {
            let c = place.plan(l[0], l[2]);
            sink.plain_box(
                "rope",
                c.x,
                l[1] + 0.16,
                c.y,
                Vec3::new(0.02, 0.16, 0.02),
                0.0,
            );
        }
        out.push(piece(sink, far + 1.0));
    }
    Some(out)
}
