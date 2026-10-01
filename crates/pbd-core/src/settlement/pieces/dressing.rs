//! A harbour's dressing (`cities-in-the-world` task 4.2c): the things it
//! stands about its quay, beach and piers, and its shipyard. Each is cut in
//! its own frame at its point, flat on what is under it, to the mockup's own
//! sizes. They are placed from the template and the stored chart, as the
//! lamps are, and never saved.

use super::super::chart::{Chart, Patch};
use super::super::{Dress, Goods, Pier, Shipyard, Template, sea};
use super::{BuildingSolids, Frame, Meshes, Sink, Solid, Surface, ccw};
use glam::{Mat2, Vec2, Vec3};
use std::f32::consts::{PI, TAU};

/// How far over the mockup's own height a thing may stand. A sea template
/// rounds its ground to whole layers, so the mockup's 0.75 m beach is 1 m in
/// the game, and what stands on it rises with it.
const STAND_OVER_M: f32 = 0.5;

/// The mockup's beach under its shipyard, metres: the shipyard's parts are
/// cut from it.
const SHIPYARD_FOOT_M: f32 = 0.5;

/// The shipyard's hull: length, beam, depth and sheer (the mockup's `HL`,
/// `HB`, `HD`, `HS`), planked to `SHIPYARD_PLANKED` of its depth.
const SHIPYARD_HULL: Hull = Hull {
    l: 9.6,
    b: 3.3,
    d: 1.9,
    sheer: 0.6,
};
const SHIPYARD_PLANKED: f32 = 0.45;
/// The keel's top over the shipyard's foot: its blocks.
const KEEL_BLOCK_M: f32 = 0.35;

/// A hull lofted from U-shaped sections (the mockup's `hullSection`):
/// length along x, beam across z, the keel `d` under the gunwale amidships,
/// the gunwale rising by `sheer` at the ends.
#[derive(Clone, Copy, Debug)]
pub struct Hull {
    pub l: f32,
    pub b: f32,
    pub d: f32,
    pub sheer: f32,
}

impl Hull {
    /// The mockup's small boats (its `BOATS`), and their planks' texture.
    pub fn of(kind: &str) -> (Self, f32, &'static str) {
        match kind {
            "sail" => (
                Self {
                    l: 6.6,
                    b: 2.2,
                    d: 0.9,
                    sheer: 0.3,
                },
                0.45,
                "boards",
            ),
            "canoe" => (
                Self {
                    l: 5.0,
                    b: 0.85,
                    d: 0.42,
                    sheer: 0.12,
                },
                0.18,
                "driftwood",
            ),
            _ => (
                Self {
                    l: 4.2,
                    b: 1.45,
                    d: 0.6,
                    sheer: 0.22,
                },
                0.25,
                "boards",
            ),
        }
    }

    /// The section a share `t` along the hull: where it is along the hull,
    /// its half beam, its depth and its gunwale's rise.
    pub(super) fn section(&self, t: f32) -> (f32, f32, f32, f32) {
        let e = (2.0 * t - 1.0).abs();
        let f = (1.0 - e.powf(2.4)).max(0.0);
        (
            (t - 0.5) * self.l,
            self.b / 2.0 * f.powf(0.55),
            self.d * (0.35 + 0.65 * f.powf(0.4)),
            self.sheer * e * e,
        )
    }

    /// A point of the section at `t`, `th` round it from one gunwale
    /// (`-PI / 2`) through the keel to the other.
    fn point(&self, t: f32, th: f32) -> Vec3 {
        let (x, b, d, top) = self.section(t);
        Vec3::new(x, top - d * th.cos(), b * th.sin())
    }
}

/// Where a thing of the mockup lands: a frame flat on the planet at its
/// point, and the map from the mockup's metres about that point into the
/// frame's plan, which carries the chart's turn and its stretch.
pub(super) struct Place {
    pub(super) frame: Frame,
    x: f32,
    z: f32,
    m: Mat2,
}

impl Place {
    /// A frame at the mockup's `(x, z)`, its floor `at_m` over the radius.
    pub(super) fn new(
        chart: &Chart,
        patch: &Patch,
        cell_m: f32,
        (x, z): (f32, f32),
        radius_m: f32,
        at_m: f32,
    ) -> Option<Self> {
        let p = sea::point(chart, patch, x, z, cell_m)?;
        let px = sea::point(chart, patch, x + 1.0, z, cell_m)?;
        let pz = sea::point(chart, patch, x, z + 1.0, cell_m)?;
        let along = px - p;
        let xd = (along - p * along.dot(p)).normalize();
        let frame = Frame {
            origin: p * (radius_m + at_m),
            x: xd,
            y: p,
            z: xd.cross(p),
        };
        let o = frame.plane(p);
        let m = Mat2::from_cols(frame.plane(px) - o, frame.plane(pz) - o);
        Some(Self { frame, x, z, m })
    }

    /// A frame taken as it is, the mockup's metres about `(0, 0)` its own
    /// plan's: for a piece cut off any chart (a test's deck).
    pub(super) fn flat(frame: Frame) -> Self {
        Self {
            frame,
            x: 0.0,
            z: 0.0,
            m: Mat2::IDENTITY,
        }
    }

    /// The mockup's point `(x, z)` in the frame's plan.
    pub(super) fn plan(&self, x: f32, z: f32) -> Vec2 {
        self.m * Vec2::new(x - self.x, z - self.z)
    }

    /// The mockup's point `(x, z)`, `y` over the frame's floor.
    pub(super) fn at(&self, x: f32, y: f32, z: f32) -> Vec3 {
        let p = self.plan(x, z);
        Vec3::new(p.x, y, p.y)
    }

    /// The mockup's point under a frame point: what [`Place::plan`] took it
    /// from.
    pub(super) fn mockup(&self, p: Vec3) -> Vec2 {
        self.m.inverse() * Vec2::new(p.x, p.z) + Vec2::new(self.x, self.z)
    }

    /// The mockup's heading `ang` in the frame.
    pub(super) fn turn(&self, ang: f32) -> f32 {
        let v = self.m * Vec2::new(ang.cos(), ang.sin());
        v.y.atan2(v.x)
    }
}

/// Whether the mockup's `(x, z)` is on a pier's deck.
fn on_pier(p: &Pier, x: f32, z: f32) -> bool {
    let (dx, dz) = (p.to[0] - p.from[0], p.to[2] - p.from[2]);
    let len = dx.hypot(dz);
    if len < 1e-3 {
        return false;
    }
    let (rx, rz) = (x - p.from[0], z - p.from[2]);
    let u = (rx * dx + rz * dz) / len;
    let v = (rz * dx - rx * dz) / len;
    (-0.05..=len + 0.05).contains(&u) && v.abs() <= p.width_m / 2.0
}

/// What a thing at the mockup's `(x, y, z)` stands on in the game, metres
/// over the radius: the highest of a pier's deck under it and the ground
/// that is no more than [`STAND_OVER_M`] over the mockup's own height, or
/// the ground where nothing is. `ground` is the ground's height at a
/// direction, the town's where it is laid.
#[allow(clippy::too_many_arguments)]
pub fn stand(
    template: &Template,
    chart: &Chart,
    patch: &Patch,
    terrace_m: f32,
    ground: &dyn Fn(Vec3) -> f32,
    x: f32,
    y: f32,
    z: f32,
) -> Option<f32> {
    let d = sea::point(chart, patch, x, z, template.grid.cell_m)?;
    let limit = terrace_m + y + STAND_OVER_M;
    let floor = ground(d);
    let best = template
        .piers
        .iter()
        .filter(|p| on_pier(p, x, z))
        .map(|p| terrace_m + p.from[1])
        .chain([floor])
        .filter(|&h| h <= limit)
        .fold(None, |best: Option<f32>, h| {
            Some(best.map_or(h, |b| b.max(h)))
        });
    Some(best.unwrap_or(floor))
}

/// A ring of `n` points `r` round `c`, counter-clockwise in plan.
fn ring(c: Vec2, r: f32, n: usize) -> Vec<Vec2> {
    (0..n)
        .map(|k| {
            let a = k as f32 / n as f32 * TAU;
            c + Vec2::new(a.cos(), a.sin()) * r
        })
        .collect()
}

/// A post or a barrel: `n` sides `r` round `c` from `y0`, `h` tall, its
/// side in `side` and its top in `top`, a solid the walker goes round. As
/// the mockup's `cyl`, a texture mapped once wraps once round it, full
/// height; any other repeats.
#[allow(clippy::too_many_arguments)]
pub(super) fn cylinder(
    sink: &mut Sink,
    side: &str,
    top: &str,
    c: Vec2,
    y0: f32,
    r: f32,
    h: f32,
    n: usize,
) {
    let pts = ring(c, r, n);
    let rep = (sink.repeat_m)(side);
    let around = TAU * r;
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let (u0, u1) = if rep <= 0.0 {
            (i as f32 / n as f32, (i + 1) as f32 / n as f32)
        } else {
            let s = around / rep;
            (i as f32 / n as f32 * s, (i + 1) as f32 / n as f32 * s)
        };
        let uv = move |p: Vec3| {
            let u = if Vec2::new(p.x, p.z).distance_squared(a) < 1e-8 {
                u0
            } else {
                u1
            };
            let v = if rep <= 0.0 {
                (p.y - y0) / h
            } else {
                p.y / rep
            };
            Vec2::new(u, v)
        };
        let mid = (a + b) * 0.5 - c;
        sink.face(
            side,
            &[
                Vec3::new(a.x, y0, a.y),
                Vec3::new(b.x, y0, b.y),
                Vec3::new(b.x, y0 + h, b.y),
                Vec3::new(a.x, y0 + h, a.y),
            ],
            Vec3::new(mid.x, 0.0, mid.y),
            Some(&uv),
        );
    }
    let cap: Vec<Vec3> = pts.iter().map(|p| Vec3::new(p.x, y0 + h, p.y)).collect();
    sink.face(top, &cap, Vec3::Y, None);
    sink.solids.push(Solid {
        outline: ccw(pts),
        y0,
        y1: y0 + h,
    });
}

/// A square bar `w` thick from `a` to `b`, any way up: a rib.
fn bar(sink: &mut Sink, material: &str, a: Vec3, b: Vec3, w: f32) {
    let u = (b - a).normalize_or_zero();
    if u == Vec3::ZERO {
        return;
    }
    let side = if u.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
    let v = u.cross(side).normalize() * (w / 2.0);
    let n = u.cross(v).normalize() * (w / 2.0);
    for (s, t) in [(v, n), (n, -v), (-v, -n), (-n, v)] {
        sink.face(
            material,
            &[a + s + t, b + s + t, b + s - t, a + s - t],
            s,
            None,
        );
    }
}

/// A two-sided face: drawn from both sides, as the mockup's cloth and nets
/// are.
pub(super) fn both(sink: &mut Sink, material: &str, pts: &[Vec3], uv: &dyn Fn(Vec3) -> Vec2) {
    let n = (pts[1] - pts[0]).cross(pts[2] - pts[0]);
    sink.face(material, pts, n, Some(uv));
    sink.face(material, pts, -n, Some(uv));
}

/// A hull lofted on its sections (the mockup's `hullGeometry`), both sides
/// of its planks drawn, to `part` of the way round each section from the
/// keel. `to` takes a point in the hull's own frame (x along it, y up, z
/// across) into the sink's.
pub(super) fn hull_skin(
    sink: &mut Sink,
    material: &str,
    hull: Hull,
    part: f32,
    to: &dyn Fn(Vec3) -> Vec3,
) {
    const S: usize = 18;
    const K: usize = 8;
    let at = |i: usize, k: usize| -> (Vec3, Vec2) {
        let t = i as f32 / S as f32;
        let th = (k as f32 / (2 * K) as f32 - 0.5) * PI * part;
        (
            to(hull.point(t, th)),
            Vec2::new((t - 0.5) * hull.l / 1.5, k as f32 / (2 * K) as f32 * 2.5),
        )
    };
    for i in 0..S {
        for k in 0..2 * K {
            let corners = [at(i, k), at(i + 1, k), at(i, k + 1), at(i + 1, k + 1)];
            for [p, q, r] in [
                [corners[0], corners[1], corners[2]],
                [corners[2], corners[1], corners[3]],
            ] {
                let pts = [p.0, q.0, r.0];
                let uv = |x: Vec3| {
                    [p, q, r]
                        .iter()
                        .min_by(|a, b| a.0.distance_squared(x).total_cmp(&b.0.distance_squared(x)))
                        .map_or(Vec2::ZERO, |c| c.1)
                };
                both(sink, material, &pts, &uv);
            }
        }
    }
}

/// A hull's outline in its own plan, as the mockup's boat solids are: a
/// hexagon a little inside its gunwale.
fn hull_outline(hull: Hull) -> [Vec2; 6] {
    let (hl, hw) = (hull.l / 2.0 * 0.97, hull.b / 2.0 * 0.95);
    [
        Vec2::new(hl, 0.0),
        Vec2::new(hl * 0.45, hw),
        Vec2::new(-hl * 0.45, hw),
        Vec2::new(-hl, 0.0),
        Vec2::new(-hl * 0.45, -hw),
        Vec2::new(hl * 0.45, -hw),
    ]
}

/// The solids and surfaces cut into `sink`, as a piece of the town.
pub(super) fn finish(sink: Sink, frame: Frame, reach_m: f32) -> BuildingSolids {
    BuildingSolids {
        frame,
        reach_m,
        solids: sink.solids,
        roof_plan: Vec::new(),
        surfaces: sink.surfaces,
        doors: Vec::new(),
        top_m: 0.0,
        rooms: Meshes::new(),
        lights: Vec::new(),
    }
}

/// A market stall (the mockup's `marketStall`): four posts, the back two
/// taller, a plank counter, an awning of `cloth` sloping to the front, and
/// its goods on the counter.
fn stall(sink: &mut Sink, place: &Place, (x, z): (f32, f32), cloth: &str, goods: &[Goods]) {
    for (a, b) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let h = if b > 0.0 { 2.4 } else { 2.1 };
        let c = place.plan(x + a * 0.9, z + b * 0.55);
        cylinder(sink, "timber", "timber", c, 0.0, 0.06, h, 6);
    }
    let c = place.plan(x, z - 0.3);
    let size = Vec3::new(1.9, 0.9, 0.7);
    sink.plain_box("plank", c.x, 0.0, c.y, size, place.turn(0.0));
    sink.solid_box(c.x, 0.0, c.y, size, place.turn(0.0));
    let awning = [
        place.at(x - 1.05, 2.45, z + 0.7),
        place.at(x + 1.05, 2.45, z + 0.7),
        place.at(x + 1.05, 2.05, z - 0.8),
        place.at(x - 1.05, 2.05, z - 0.8),
    ];
    let uv = |p: Vec3| {
        let q = place.mockup(p);
        Vec2::new((q.x - x + 1.05) / 2.1, (q.y - z + 0.8) / 1.5)
    };
    both(sink, cloth, &awning, &uv);
    for g in goods {
        let c = place.plan(g.x, g.z);
        sink.plain_box(
            &g.material,
            c.x,
            0.9,
            c.y,
            Vec3::new(0.22, 0.14, 0.22),
            place.turn(0.0),
        );
    }
}

/// Nets hung to dry (the mockup's `netRack`): three posts, a bar, and the
/// net hung both sides of them, see-through; a thin wall to the walker.
fn net_rack(sink: &mut Sink, place: &Place, (x, z): (f32, f32), ang: f32) {
    let (ca, sa) = (ang.cos(), ang.sin());
    for k in [-1.0, 0.0, 1.0] {
        let c = place.plan(x + ca * k * 1.5, z + sa * k * 1.5);
        cylinder(sink, "timber", "timber", c, 0.0, 0.07, 2.2, 6);
    }
    let c = place.plan(x, z);
    let turn = place.turn(ang);
    sink.plain_box("timber", c.x, 2.1, c.y, Vec3::new(3.1, 0.07, 0.07), turn);
    let (a, b) = ((x - ca * 1.5, z - sa * 1.5), (x + ca * 1.5, z + sa * 1.5));
    let net = [
        place.at(a.0, 0.35, a.1),
        place.at(b.0, 0.35, b.1),
        place.at(b.0, 2.05, b.1),
        place.at(a.0, 2.05, a.1),
    ];
    let uv = |p: Vec3| {
        let q = place.mockup(p);
        Vec2::new((q.x - x) * ca + (q.y - z) * sa, p.y)
    };
    both(sink, "net", &net, &uv);
    sink.solid_box(c.x, 0.0, c.y, Vec3::new(3.0, 2.1, 0.1), turn);
}

/// Fish hung to dry (the mockup's `fishRack`): two posts, a bar and seven
/// fish; a wall to the walker from the fish up.
fn fish_rack(sink: &mut Sink, place: &Place, (x, z): (f32, f32), ang: f32) {
    let (ca, sa) = (ang.cos(), ang.sin());
    for k in [-1.0, 1.0] {
        let c = place.plan(x + ca * k * 1.2, z + sa * k * 1.2);
        cylinder(sink, "timber", "timber", c, 0.0, 0.06, 1.9, 5);
    }
    let c = place.plan(x, z);
    let turn = place.turn(ang);
    sink.plain_box("timber", c.x, 1.85, c.y, Vec3::new(2.5, 0.06, 0.06), turn);
    for k in 0..7 {
        let t = (k as f32 - 3.0) * 0.32;
        let f = place.plan(x + ca * t, z + sa * t);
        sink.plain_box("fish", f.x, 1.25, f.y, Vec3::new(0.08, 0.5, 0.16), turn);
    }
    sink.solid_box(c.x, 0.9, c.y, Vec3::new(2.5, 1.0, 0.2), turn);
}

/// A box the walker goes round: a pot, a crate.
fn solid_box(sink: &mut Sink, material: &str, c: Vec2, y0: f32, size: Vec3, turn: f32) {
    sink.plain_box(material, c.x, y0, c.y, size, turn);
    sink.solid_box(c.x, y0, c.y, size, turn);
}

/// A small boat on land (the mockup's `boat` beached or still): keel up on
/// two trestles where `beached`, else on its keel, a rowboat with its
/// thwarts and oars.
///
/// Two departures from the mockup, both on the beached boats. The mockup
/// stands one a hull's depth over its trestles, and runs its trestles along
/// its keel. Here its gunwale rests on them, and they run across it.
fn boat(sink: &mut Sink, place: &Place, (x, z): (f32, f32), kind: &str, ang: f32, beached: bool) {
    let (hull, _, wood) = Hull::of(kind);
    let (ca, sa) = (ang.cos(), ang.sin());
    // The trestles stand at 0.28 of the length either side of the middle,
    // where the turned-over gunwale is this far under its middle.
    let trestle_m = 0.45;
    let at_trestle = hull.sheer * 0.56 * 0.56;
    let gunwale = if beached {
        trestle_m + at_trestle
    } else {
        hull.d
    };
    let to = |p: Vec3| -> Vec3 {
        let p = if beached {
            Vec3::new(p.x, -p.y, -p.z)
        } else {
            p
        };
        place.at(
            x + p.x * ca - p.z * sa,
            gunwale + p.y,
            z + p.x * sa + p.z * ca,
        )
    };
    hull_skin(sink, wood, hull, 1.0, &to);
    let turn = place.turn(ang);
    if beached {
        for k in [-1.0, 1.0] {
            let along = k * hull.l * 0.28;
            let c = place.plan(x + ca * along, z + sa * along);
            sink.plain_box(
                "timber",
                c.x,
                0.0,
                c.y,
                Vec3::new(0.2, trestle_m, hull.b * 1.1),
                turn,
            );
        }
    } else if kind == "rowboat" {
        let part = |sink: &mut Sink, size: Vec3, p: Vec3| {
            let c = to(Vec3::new(p.x, 0.0, p.z));
            sink.plain_box(wood, c.x, gunwale + p.y - size.y / 2.0, c.z, size, turn);
        };
        for tx in [-0.7, 0.7] {
            part(
                sink,
                Vec3::new(0.22, 0.05, hull.b * 0.82),
                Vec3::new(tx, -hull.d * 0.35, 0.0),
            );
        }
        for s in [-0.2, 0.2] {
            part(
                sink,
                Vec3::new(2.6, 0.04, 0.08),
                Vec3::new(0.1, -hull.d * 0.3, s),
            );
        }
    }
    let outline: Vec<Vec2> = hull_outline(hull)
        .iter()
        .map(|p| place.plan(x + p.x * ca - p.y * sa, z + p.x * sa + p.y * ca))
        .collect();
    let top = if beached { gunwale + hull.d } else { hull.d };
    sink.solids.push(Solid {
        outline: ccw(outline),
        y0: -0.2,
        y1: top,
    });
}

/// Cut each of a town's dressing things in its own frame, on what is under
/// it. `ground` is the ground's height at a direction, metres over the
/// radius, the town's where it is laid. Returns the pieces, and how many
/// things were left out for standing off the chart: a harbour stored before
/// its dressing charted no cells for its slip.
#[allow(clippy::too_many_arguments)]
pub fn dressing(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    template: &Template,
    radius_m: f32,
    terrace_m: f32,
    ground: &dyn Fn(Vec3) -> f32,
) -> (Vec<BuildingSolids>, usize) {
    let cell_m = template.grid.cell_m;
    let mut out = Vec::with_capacity(template.dressing.len());
    let mut skipped = 0;
    for dress in &template.dressing {
        let ((x, z), y) = (dress.at(), dress.y());
        let Some(place) = stand(template, chart, patch, terrace_m, ground, x, y, z)
            .and_then(|at_m| Place::new(chart, patch, cell_m, (x, z), radius_m, at_m))
        else {
            skipped += 1;
            continue;
        };
        let frame = place.frame;
        let mut sink = Sink::new(meshes, repeat_m, frame);
        let o = Vec2::ZERO;
        let reach_m = match dress {
            Dress::Stall { cloth, goods, .. } => {
                stall(&mut sink, &place, (x, z), cloth, goods);
                2.5
            }
            Dress::NetRack { angle, .. } => {
                net_rack(&mut sink, &place, (x, z), *angle);
                2.5
            }
            Dress::FishRack { angle, .. } => {
                fish_rack(&mut sink, &place, (x, z), *angle);
                2.0
            }
            Dress::Pot { angle, lift_m, .. } => {
                let size = Vec3::new(0.5, 0.4, 0.5);
                solid_box(&mut sink, "wool", o, *lift_m, size, place.turn(*angle));
                1.0
            }
            Dress::Crate { angle, side_m, .. } => {
                let size = Vec3::splat(*side_m);
                solid_box(&mut sink, "timber", o, 0.0, size, place.turn(*angle));
                1.0
            }
            Dress::Barrel { .. } => {
                cylinder(&mut sink, "barrel", "timber", o, 0.0, 0.34, 0.9, 10);
                1.0
            }
            Dress::Bollard {
                radius_m, height_m, ..
            } => {
                cylinder(
                    &mut sink, "timber", "timber", o, 0.0, *radius_m, *height_m, 6,
                );
                1.0
            }
            Dress::Oar { .. } => {
                sink.plain_box("timber", 0.0, 0.0, 0.0, Vec3::new(0.07, 2.6, 0.07), 0.0);
                1.0
            }
            Dress::Boat {
                boat: kind,
                angle,
                beached,
                ..
            } => {
                boat(&mut sink, &place, (x, z), kind, *angle, *beached);
                Hull::of(kind).0.l / 2.0 + 1.0
            }
        };
        out.push(finish(sink, frame, reach_m));
    }
    if let Some(yard) = &template.shipyard {
        match shipyard(
            meshes, repeat_m, patch, chart, template, yard, radius_m, terrace_m, ground,
        ) {
            Ok(yard) => out.extend(yard),
            Err(_) => skipped += 1,
        }
    }
    (out, skipped)
}

/// The shipyard (the mockup's in `makeCoast`): a hull in frame on its keel
/// blocks with shores either side, a stack of planks, and the slip down
/// from the beach into the water, a ramp the walker goes down.
#[allow(clippy::too_many_arguments)]
fn shipyard(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    template: &Template,
    yard: &Shipyard,
    radius_m: f32,
    terrace_m: f32,
    ground: &dyn Fn(Vec3) -> f32,
) -> Result<Vec<BuildingSolids>, String> {
    let cell_m = template.grid.cell_m;
    let off = || "the shipyard is off the chart".to_string();
    let stand = |x: f32, y: f32, z: f32| stand(template, chart, patch, terrace_m, ground, x, y, z);
    let mut out = Vec::new();
    // The hull in frame.
    let (x, z) = (yard.x, yard.z);
    let at_m = stand(x, SHIPYARD_FOOT_M, z).ok_or_else(off)?;
    let place = Place::new(chart, patch, cell_m, (x, z), radius_m, at_m).ok_or_else(off)?;
    let mut sink = Sink::new(meshes, repeat_m, place.frame);
    let turn = place.turn(0.0);
    let c = place.plan(x, z);
    sink.plain_box(
        "timber",
        c.x,
        0.0,
        c.y,
        Vec3::new(9.8, KEEL_BLOCK_M, 0.4),
        turn,
    );
    for k in -3i32..=3 {
        for s in [-1.0, 1.0] {
            let p = place.plan(x + k as f32 * 1.3, z + s * 1.3);
            let h = 1.0 - k.abs() as f32 * 0.08;
            sink.plain_box("timber", p.x, 0.0, p.y, Vec3::new(0.12, h, 0.12), turn);
        }
    }
    let hull = SHIPYARD_HULL;
    let gunwale = KEEL_BLOCK_M + hull.d;
    let to = |p: Vec3| place.at(x + p.x, gunwale + p.y, z + p.z);
    hull_skin(&mut sink, "timber", hull, SHIPYARD_PLANKED, &to);
    for i in 1..12 {
        let t = i as f32 / 12.0;
        let pts: Vec<Vec3> = (0..=10)
            .map(|k| to(hull.point(t, (k as f32 / 10.0 - 0.5) * PI)))
            .collect();
        for w in pts.windows(2) {
            bar(&mut sink, "timber", w[0], w[1], 0.12);
        }
    }
    // Stem and stern posts.
    let post_h = hull.d + hull.sheer + 0.2;
    let post_y = hull.sheer - hull.d / 2.0 + 0.1 - post_h / 2.0;
    for s in [-1.0, 1.0] {
        let p = place.plan(x + s * hull.l / 2.0, z);
        sink.plain_box(
            "timber",
            p.x,
            gunwale + post_y,
            p.y,
            Vec3::new(0.2, post_h, 0.2),
            turn,
        );
    }
    sink.solid_box(
        c.x,
        0.0,
        c.y,
        Vec3::new(10.2, 3.4 - SHIPYARD_FOOT_M, 3.4),
        turn,
    );
    out.push(finish(sink, place.frame, 6.0));
    // The planks, stacked three high.
    let [px, py, pz] = yard.planks;
    let at_m = stand(px, py, pz).ok_or_else(off)?;
    let place = Place::new(chart, patch, cell_m, (px, pz), radius_m, at_m).ok_or_else(off)?;
    let mut sink = Sink::new(meshes, repeat_m, place.frame);
    let turn = place.turn(0.0);
    for k in 0..3 {
        let size = Vec3::new(3.2, 0.3, 0.5);
        sink.plain_box("timber", 0.0, k as f32 * 0.3, 0.0, size, turn);
    }
    sink.solid_box(0.0, 0.0, 0.0, Vec3::new(3.2, 0.9, 0.5), turn);
    out.push(finish(sink, place.frame, 2.0));
    out.push(slip(
        meshes, repeat_m, patch, chart, template, &yard.slip, radius_m, terrace_m, ground,
    )?);
    Ok(out)
}

/// The slip (the mockup's `bridge` with `ramp`): from just over the beach
/// at its head, 5 cm over the sand as the mockup's, down to its foot in the
/// water at the mockup's height there over the sea.
#[allow(clippy::too_many_arguments)]
fn slip(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    template: &Template,
    slip: &Pier,
    radius_m: f32,
    terrace_m: f32,
    ground: &dyn Fn(Vec3) -> f32,
) -> Result<BuildingSolids, String> {
    let (a, b) = (slip.from, slip.to);
    let head_m = stand(template, chart, patch, terrace_m, ground, a[0], a[1], a[2])
        .ok_or("the slip is off the chart")?
        + 0.05;
    let ends = (head_m, terrace_m + b[1]);
    let cell_m = template.grid.cell_m;
    ramp(
        meshes, repeat_m, patch, chart, cell_m, slip, ends, radius_m, "plank",
    )
    .ok_or_else(|| "the slip is off the chart".to_string())
}

/// A ramp (the mockup's `bridge` with `ramp`): one sloped deck of
/// `material`, 10 cm thick, with cleats across it, from `pier.from` to
/// `pier.to` (the mockup's metres), its top at `ends`' heights over the
/// radius: a slip, a gangplank. Cut in one frame at its head, the walker
/// going up or down it on a [`Surface::Ramp`]. `None` where an end is off
/// the chart.
#[allow(clippy::too_many_arguments)]
pub(super) fn ramp(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    cell_m: f32,
    pier: &Pier,
    (head_m, foot_m): (f32, f32),
    radius_m: f32,
    material: &str,
) -> Option<BuildingSolids> {
    let (a, b) = (pier.from, pier.to);
    let mid = ((a[0] + b[0]) / 2.0, (a[2] + b[2]) / 2.0);
    let place = Place::new(chart, patch, cell_m, mid, radius_m, head_m)?;
    let mut sink = Sink::new(meshes, repeat_m, place.frame);
    let (pa, pb) = (place.plan(a[0], a[2]), place.plan(b[0], b[2]));
    let len = pa.distance(pb);
    let dir = (pb - pa) / len;
    let side = dir.perp() * (pier.width_m / 2.0);
    let drop = foot_m - head_m;
    let p = |t: f32, s: f32, dy: f32| {
        let q = pa + (pb - pa) * t + side * s;
        Vec3::new(q.x, drop * t + dy, q.y)
    };
    sink.face(
        material,
        &[
            p(0.0, -1.0, 0.0),
            p(1.0, -1.0, 0.0),
            p(1.0, 1.0, 0.0),
            p(0.0, 1.0, 0.0),
        ],
        Vec3::Y,
        None,
    );
    sink.face(
        material,
        &[
            p(0.0, -1.0, -0.1),
            p(1.0, -1.0, -0.1),
            p(1.0, 1.0, -0.1),
            p(0.0, 1.0, -0.1),
        ],
        -Vec3::Y,
        None,
    );
    for s in [-1.0f32, 1.0] {
        let out = side * s;
        sink.face(
            material,
            &[
                p(0.0, s, -0.1),
                p(1.0, s, -0.1),
                p(1.0, s, 0.0),
                p(0.0, s, 0.0),
            ],
            Vec3::new(out.x, 0.0, out.y),
            None,
        );
    }
    let ang = dir.y.atan2(dir.x);
    let cleats = ((len / 0.45).floor() as usize).max(1);
    for k in 1..cleats {
        let t = k as f32 / cleats as f32;
        let q = p(t, 0.0, 0.0);
        sink.plain_box(
            "timber",
            q.x,
            q.y - 0.01,
            q.z,
            Vec3::new(0.05, 0.05, pier.width_m * 0.9),
            ang,
        );
    }
    sink.surfaces.push(Surface::Ramp {
        foot: pa,
        dir,
        len,
        half_width: pier.width_m / 2.0,
        from: 0.0,
        to: drop,
        depth: 0.1,
    });
    let reach_m = len / 2.0 + pier.width_m;
    let frame = place.frame;
    Some(finish(sink, frame, reach_m))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    #[test]
    fn a_hulls_gunwale_rises_to_its_ends_and_its_keel_is_its_depth_down() {
        let (hull, ..) = Hull::of("rowboat");
        let mid_keel = hull.point(0.5, 0.0);
        assert!((mid_keel.y + hull.d).abs() < 1e-5, "{mid_keel}");
        let mid_gunwale = hull.point(0.5, FRAC_PI_2);
        assert!(mid_gunwale.y.abs() < 1e-5 && (mid_gunwale.z - hull.b / 2.0).abs() < 1e-5);
        let end = hull.point(0.0, FRAC_PI_2);
        assert!((end.y - hull.sheer).abs() < 1e-5 && end.z.abs() < 1e-5);
    }

    #[test]
    fn a_thing_on_a_pier_is_on_its_deck() {
        let pier = Pier {
            from: [0.0, 1.0, 0.0],
            to: [10.0, 1.0, 0.0],
            width_m: 2.0,
        };
        assert!(on_pier(&pier, 5.0, 0.9));
        assert!(!on_pier(&pier, 5.0, 1.1));
        assert!(!on_pier(&pier, 10.2, 0.0));
    }
}
