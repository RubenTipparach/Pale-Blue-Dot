//! A town's smithy (`townsfolk` design 26a), the towns mockup's
//! `smithyShed`: an open shed two rows by three cells, six timber posts
//! under a shingled gable roof with timber gables, a forge at its far end and
//! an anvil by the smith. One piece, cut on the forge cell's terrace in a
//! frame flat on the planet there, placed from the template and the stored
//! chart as the dressing is and never saved.
//!
//! Nothing is a box stood on a box: the forge is one stepped stone mass (a
//! plinth, a base whose top is sunk round its hearth bed, a back wall that
//! carries a hood tapering into its chimney), the anvil one lofted iron
//! shape smooth across its curves (a waisted foot, a face and a horn) on a
//! stump, the fire one flame form on the hearth bed in its own pixel sheet,
//! the posts tapered and smooth round. The forge's light is not here: it
//! is a `Brazier` lamp in the voxel field at the forge cell's middle
//! ([`super::super::record::lamps_of`]), so it lights everything near. The forge stands centred on that cell, its
//! base taller than the brazier's drawn basket and fire, so only this
//! piece's own flame shows, over the hearth bed.

use super::super::Smithy;
use super::super::chart::{Chart, Patch};
use super::super::sea;
use super::dressing::{Place, finish};
use super::{BuildingSolids, Sink, Solid, ccw};
use glam::{Vec2, Vec3};

/// The shed's posts: their radius at the foot and the head, and their
/// height to the eaves, metres (the mockup's `solidCyl(..., 0.15, 3.2)`).
pub const POST_FOOT_M: f32 = 0.15;
pub const POST_HEAD_M: f32 = 0.12;
pub const EAVE_M: f32 = 3.2;
/// The roof's pitch (rise over run) and its overhang past the posts.
pub const PITCH: f32 = 0.7;
pub const OVERHANG_M: f32 = 0.3;
/// The forge: its base along and across, its height (over the brazier's
/// drawn fire, which tops out at 1.04 m), the hearth bed sunk in its top,
/// and the hood's foot.
pub const FORGE_ALONG_M: f32 = 1.6;
pub const FORGE_ACROSS_M: f32 = 1.3;
pub const FORGE_TOP_M: f32 = 1.1;
pub const BED_SINK_M: f32 = 0.05;
pub const HOOD_FOOT_M: f32 = 1.65;
/// The forge's fire (design 26a, "the fire and the anvil redone"): one
/// flame form on the hearth bed, drawn in [`FIRE`]'s sheet of
/// [`FIRE_FRAMES`] frames. An outer crown on an ellipse just inside the bed
/// (its half axes along and across, metres), its rim cut into tongues of
/// these heights, and a taller core of fewer tongues inside it.
pub const FIRE: &str = "forge_fire";
pub const FIRE_FRAMES: u32 = 8;
const FIRE_OUTER: (f32, f32) = (0.40, 0.28);
const FIRE_OUTER_TONGUES: [f32; 9] = [0.26, 0.33, 0.22, 0.30, 0.24, 0.34, 0.21, 0.28, 0.25];
const FIRE_CORE: (f32, f32) = (0.2, 0.14);
const FIRE_CORE_TONGUES: [f32; 4] = [0.40, 0.46, 0.36, 0.43];
/// The anvil's iron, its worn face, and the stump it stands on: the stump's
/// height (the anvil's face keeps the old 0.72 m) and its radius.
pub const ANVIL: &str = "anvil";
pub const ANVIL_FACE: &str = "anvilface";
pub const ANVIL_TOP_M: f32 = 0.72;
pub const STUMP_M: f32 = 0.3;
const STUMP_R_M: f32 = 0.24;

/// A frustum of four sides over a rectangle in the mockup's plan: centred on
/// `(x, z)`, `w0` along x and `d0` along z at `y0`, `w1` by `d1` at `y1`,
/// shifted `(sx, sz)` at the top; its sides in `side`, its top in `top`
/// (none, open), hard at its edges.
#[allow(clippy::too_many_arguments)]
fn frustum(
    sink: &mut Sink,
    place: &Place,
    side: &str,
    top: Option<&str>,
    (x, z): (f32, f32),
    (w0, d0, y0): (f32, f32, f32),
    (w1, d1, y1): (f32, f32, f32),
    (sx, sz): (f32, f32),
) {
    let ring = |w: f32, d: f32, y: f32, ox: f32, oz: f32| {
        [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
            .map(|(a, b)| place.at(x + ox + a * w / 2.0, y, z + oz + b * d / 2.0))
    };
    let lo = ring(w0, d0, y0, 0.0, 0.0);
    let hi = ring(w1, d1, y1, sx, sz);
    let middle = place.at(x + sx / 2.0, (y0 + y1) / 2.0, z + sz / 2.0);
    for k in 0..4 {
        let pts = [lo[k], lo[(k + 1) % 4], hi[(k + 1) % 4], hi[k]];
        let c = (pts[0] + pts[1] + pts[2] + pts[3]) / 4.0;
        let out = Vec3::new(c.x - middle.x, 0.0, c.z - middle.z);
        sink.face(side, &pts, out, None);
    }
    if let Some(top) = top {
        sink.face(top, &hi, Vec3::Y, None);
    }
}

/// A tapered post of eight sides at the mockup's `(x, z)`, its normals smooth
/// round it and hard only at its cap; a solid the walker goes round.
fn post(sink: &mut Sink, place: &Place, (x, z): (f32, f32), h: f32) {
    const SIDES: usize = 8;
    let c = place.plan(x, z);
    let dir = |k: usize| {
        let a = k as f32 / SIDES as f32 * std::f32::consts::TAU;
        Vec2::new(a.cos(), a.sin())
    };
    let slope = (POST_FOOT_M - POST_HEAD_M) / h;
    let normal = |d: Vec2| Vec3::new(d.x, slope, d.y).normalize();
    let rep = (sink.repeat_m)("timber").max(0.1);
    for k in 0..SIDES {
        let (d0, d1) = (dir(k), dir(k + 1));
        let p = |d: Vec2, r: f32, y: f32| Vec3::new(c.x + d.x * r, y, c.y + d.y * r);
        let pts = [
            p(d0, POST_FOOT_M, 0.0),
            p(d1, POST_FOOT_M, 0.0),
            p(d1, POST_HEAD_M, h),
            p(d0, POST_HEAD_M, h),
        ];
        let (u0, u1) = (k as f32 / SIDES as f32, (k + 1) as f32 / SIDES as f32);
        let uvs = [
            Vec2::new(u0 * 0.5, 0.0),
            Vec2::new(u1 * 0.5, 0.0),
            Vec2::new(u1 * 0.5, -h / rep),
            Vec2::new(u0 * 0.5, -h / rep),
        ];
        let ns = [normal(d0), normal(d1), normal(d1), normal(d0)];
        sink.smooth_face("timber", &pts, &ns, Some(&uvs));
    }
    let cap: Vec<Vec3> = (0..SIDES)
        .map(|k| {
            let d = dir(k);
            Vec3::new(c.x + d.x * POST_HEAD_M, h, c.y + d.y * POST_HEAD_M)
        })
        .collect();
    sink.face("timber", &cap, Vec3::Y, None);
    let s = POST_FOOT_M;
    sink.solids.push(Solid {
        outline: ccw(vec![
            c + Vec2::new(-s, -s),
            c + Vec2::new(s, -s),
            c + Vec2::new(s, s),
            c + Vec2::new(-s, s),
        ]),
        y0: 0.0,
        y1: h,
    });
}

/// The shed's roof: two shingled slopes from the eaves (with their
/// overhang) to a ridge along x, a timber underside, and timber gables.
fn roof(sink: &mut Sink, place: &Place, (x0, x1): (f32, f32), (z0, z1): (f32, f32)) {
    let zm = (z0 + z1) / 2.0;
    let half = (z1 - z0) / 2.0;
    let ridge = EAVE_M + PITCH * half;
    let low = EAVE_M - PITCH * OVERHANG_M;
    let (xa, xb) = (x0 - OVERHANG_M, x1 + OVERHANG_M);
    for (edge, out) in [(z0 - OVERHANG_M, -1.0f32), (z1 + OVERHANG_M, 1.0)] {
        let pts = [
            place.at(xa, low, edge),
            place.at(xb, low, edge),
            place.at(xb, ridge, zm),
            place.at(xa, ridge, zm),
        ];
        // Out and up from the slope; the underside down and in.
        let up = Vec3::new(0.0, 1.0, 0.0) + place.at(xa, 0.0, zm + out) - place.at(xa, 0.0, zm);
        sink.face("shingle", &pts, up, None);
        let lift = Vec3::Y * 0.12;
        let under = pts.map(|p| p - lift);
        sink.face("timber", &under, -up, None);
        // The eave's edge between them.
        sink.face(
            "timber",
            &[pts[0], pts[1], under[1], under[0]],
            up.with_y(0.0),
            None,
        );
    }
    for (x, out) in [(x0, -1.0f32), (x1, 1.0)] {
        let tri = [
            place.at(x, EAVE_M, z0),
            place.at(x, EAVE_M, z1),
            place.at(x, ridge - 0.1, zm),
        ];
        let n = place.at(x + out, 0.0, zm) - place.at(x, 0.0, zm);
        sink.face("timber", &tri, n, None);
        sink.face("timber", &tri, -n, None);
    }
}

/// The forge at the mockup's `(x, z)`, its back toward `+x`: a plinth, a
/// stone base sunk round its hearth bed, a back wall up to the hood, and the
/// hood tapering from over the hearth into a chimney through the roof to
/// `chimney_top`. Its flame over the bed. A solid to the walker.
fn forge(sink: &mut Sink, place: &Place, (x, z): (f32, f32), chimney_top: f32) {
    let (w, d) = (FORGE_ALONG_M, FORGE_ACROSS_M);
    // The plinth, a hand wider, to a shin's height.
    frustum(
        sink,
        place,
        "stone",
        Some("stone"),
        (x, z),
        (w + 0.12, d + 0.12, 0.0),
        (w + 0.06, d + 0.06, 0.18),
        (0.0, 0.0),
    );
    // The base, drawn in a little to its lip, its top a rim round the bed.
    frustum(
        sink,
        place,
        "stone",
        None,
        (x, z),
        (w, d, 0.18),
        (w - 0.04, d - 0.04, FORGE_TOP_M),
        (0.0, 0.0),
    );
    let (bw, bd) = (1.1, 0.8);
    let top = FORGE_TOP_M;
    let rim = |a: f32, b: f32| place.at(x + a, top, z + b);
    let (hw, hd) = ((w - 0.04) / 2.0, (d - 0.04) / 2.0);
    let (iw, id) = (bw / 2.0, bd / 2.0);
    for (o, i) in [
        ([(-hw, -hd), (hw, -hd)], [(-iw, -id), (iw, -id)]),
        ([(hw, -hd), (hw, hd)], [(iw, -id), (iw, id)]),
        ([(hw, hd), (-hw, hd)], [(iw, id), (-iw, id)]),
        ([(-hw, hd), (-hw, -hd)], [(-iw, id), (-iw, -id)]),
    ] {
        sink.face(
            "stone",
            &[
                rim(o[0].0, o[0].1),
                rim(o[1].0, o[1].1),
                rim(i[1].0, i[1].1),
                rim(i[0].0, i[0].1),
            ],
            Vec3::Y,
            None,
        );
    }
    // The bed, sunk, and its walls down from the rim, facing in.
    let bed = top - BED_SINK_M;
    let corners = [(-iw, -id), (iw, -id), (iw, id), (-iw, id)];
    for k in 0..4 {
        let (a, b) = (corners[k], corners[(k + 1) % 4]);
        let pts = [
            place.at(x + a.0, top, z + a.1),
            place.at(x + b.0, top, z + b.1),
            place.at(x + b.0, bed, z + b.1),
            place.at(x + a.0, bed, z + a.1),
        ];
        let inward = place.at(x, top, z) - (pts[0] + pts[1]) * 0.5;
        sink.face("stone", &pts, inward.with_y(0.0), None);
    }
    let floor =
        [(-iw, -id), (iw, -id), (iw, id), (-iw, id)].map(|(a, b)| place.at(x + a, bed, z + b));
    sink.face("dirt", &floor, Vec3::Y, None);
    // The back wall, from the base's back up under the hood.
    frustum(
        sink,
        place,
        "stone",
        Some("stone"),
        (x + w / 2.0 - 0.12, z),
        (0.24, d - 0.04, top),
        (0.24, d - 0.04, HOOD_FOOT_M),
        (0.0, 0.0),
    );
    // The hood, its mouth over the hearth, drawing in and back to the
    // chimney, which goes up through the roof.
    let flue = (0.55, 0.5);
    let back = w / 2.0 - 0.12 - flue.0 / 2.0;
    frustum(
        sink,
        place,
        "stone",
        None,
        (x + 0.1, z),
        (w - 0.2, d - 0.04, HOOD_FOOT_M),
        (flue.0, flue.1, EAVE_M),
        (back - 0.1, 0.0),
    );
    // The hood's mouth, seen from the hearth: its soot-dark stone.
    let mouth = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].map(|(a, b)| {
        place.at(
            x + 0.1 + a * (w - 0.2) / 2.0,
            HOOD_FOOT_M,
            z + b * (d - 0.04) / 2.0,
        )
    });
    sink.face("stone", &mouth, -Vec3::Y, None);
    // The chimney, up through the roof.
    frustum(
        sink,
        place,
        "stone",
        Some("stone"),
        (x + back, z),
        (flue.0, flue.1, EAVE_M),
        (flue.0 - 0.05, flue.1 - 0.05, chimney_top),
        (0.0, 0.0),
    );
    // The fire: one flame form fitted to the bed, not cones stood on it (a
    // cone a tongue read as primitives in the capture).
    fire(sink, place, (x - 0.05, z), bed + 0.01);
    let c = place.plan(x, z);
    sink.solid_box(
        c.x,
        0.0,
        c.y,
        Vec3::new(w + 0.12, top, d + 0.12),
        place.turn(0.0),
    );
    let wall = place.plan(x + w / 2.0 - 0.12, z);
    sink.solid_box(
        wall.x,
        top,
        wall.y,
        Vec3::new(0.24, chimney_top - top, d),
        place.turn(0.0),
    );
}

/// A smooth surface lofted through `rings` (each closed, all with the same
/// count of points, in the frame), each ring about its `axis` point: its
/// normals smooth round it and along it, turned away from the axis, and each
/// quad wound to face along them. Textured by the metre in `material`, round
/// each ring and along the loft. A ring shrunk to a point (a horn's tip)
/// takes the normals of the ring before it.
fn loft(sink: &mut Sink, material: &str, rings: &[Vec<Vec3>], axis: &[Vec3]) {
    let count = rings.len();
    if count < 2 || axis.len() != count {
        return;
    }
    let n = rings[0].len();
    let rep = (sink.repeat_m)(material).max(0.1);
    let mut normals = vec![vec![Vec3::ZERO; n]; count];
    for i in 0..count {
        for k in 0..n {
            let round = rings[i][(k + 1) % n] - rings[i][(k + n - 1) % n];
            let along = rings[(i + 1).min(count - 1)][k] - rings[i.saturating_sub(1)][k];
            let mut v = round.cross(along).normalize_or_zero();
            if v.dot(rings[i][k] - axis[i]) < 0.0 {
                v = -v;
            }
            normals[i][k] = v;
        }
    }
    for i in 1..count {
        for k in 0..n {
            if normals[i][k] == Vec3::ZERO {
                normals[i][k] = normals[i - 1][k];
            }
        }
    }
    // Round each ring by its own length, along the loft by the rings' mean
    // spacing.
    let round: Vec<Vec<f32>> = rings
        .iter()
        .map(|r| {
            let mut s = vec![0.0];
            for k in 0..n {
                s.push(s[k] + r[k].distance(r[(k + 1) % n]));
            }
            s
        })
        .collect();
    let mut along = vec![0.0f32];
    for i in 1..count {
        let step = (0..n)
            .map(|k| rings[i][k].distance(rings[i - 1][k]))
            .sum::<f32>()
            / n as f32;
        along.push(along[i - 1] + step);
    }
    for i in 0..count - 1 {
        for k in 0..n {
            let k1 = (k + 1) % n;
            let mut pts = [rings[i][k], rings[i][k1], rings[i + 1][k1], rings[i + 1][k]];
            let mut ns = [
                normals[i][k],
                normals[i][k1],
                normals[i + 1][k1],
                normals[i + 1][k],
            ];
            let mut uvs = [
                Vec2::new(round[i][k] / rep, -along[i] / rep),
                Vec2::new(round[i][k + 1] / rep, -along[i] / rep),
                Vec2::new(round[i + 1][k + 1] / rep, -along[i + 1] / rep),
                Vec2::new(round[i + 1][k] / rep, -along[i + 1] / rep),
            ];
            let geometric = (pts[1] - pts[0]).cross(pts[3] - pts[0]);
            if geometric.dot(ns[0] + ns[1] + ns[2] + ns[3]) < 0.0 {
                pts.reverse();
                ns.reverse();
                uvs.reverse();
            }
            sink.smooth_face(material, &pts, &ns, Some(&uvs));
        }
    }
}

/// The forge's fire, centred on the mockup's `(x, z)` with its root at
/// `y0`: a bed of coals, and one flame form standing on it, an outer crown
/// of tongues and a taller core inside it, all sharing that base.
fn fire(sink: &mut Sink, place: &Place, (x, z): (f32, f32), y0: f32) {
    let coals: Vec<Vec3> = (0..16)
        .map(|k| {
            let a = k as f32 / 16.0 * std::f32::consts::TAU;
            place.at(
                x + FIRE_OUTER.0 * 1.05 * a.cos(),
                y0 - 0.004,
                z + FIRE_OUTER.1 * 1.05 * a.sin(),
            )
        })
        .collect();
    // The sheet's bottom row, its coals, whatever frame is shown.
    sink.face(
        FIRE,
        &coals,
        Vec3::Y,
        Some(&|p: Vec3| Vec2::new(p.x * 0.8, -0.985)),
    );
    crown(
        sink,
        place,
        (x, z),
        y0,
        FIRE_OUTER,
        &FIRE_OUTER_TONGUES,
        0.55,
        0,
    );
    crown(
        sink,
        place,
        (x + 0.03, z),
        y0,
        FIRE_CORE,
        &FIRE_CORE_TONGUES,
        0.5,
        5,
    );
}

/// A crown of flame on an ellipse of half axes `(ra, rb)` round `(x, z)` at
/// `y0`: one band round it whose rim rises into a tongue of each height in
/// `tongues`, the band drawing in by `taper` of its radius at the tallest
/// tongue's tip, so the tongues lean together. Each tongue is mapped once
/// over a frame of the sheet, starting at `first` and stepping on, so the
/// sheet's own ragged edge cuts the tongue and no two side by side flicker
/// alike. The rim between two tongues is cut just outside the sheet's edge.
#[allow(clippy::too_many_arguments)]
fn crown(
    sink: &mut Sink,
    place: &Place,
    (x, z): (f32, f32),
    y0: f32,
    (ra, rb): (f32, f32),
    tongues: &[f32],
    taper: f32,
    first: u32,
) {
    const ACROSS: usize = 6;
    const UP: usize = 4;
    let count = tongues.len() as f32;
    let top = tongues.iter().fold(0.0f32, |a, &b| a.max(b));
    let middle = place.at(x, y0, z);
    for (j, &h) in tongues.iter().enumerate() {
        let frame = (first + j as u32 * 3) % FIRE_FRAMES;
        let at = |m: usize, r: usize| {
            let p = m as f32 / ACROSS as f32;
            let a = (j as f32 + p) / count * std::f32::consts::TAU;
            let column = h * (1.0 - 0.6 * (2.0 * p - 1.0).abs().powf(1.18));
            let y = column * r as f32 / UP as f32;
            let s = 1.0 - taper * y / top;
            let point = place.at(x + ra * s * a.cos(), y0 + y, z + rb * s * a.sin());
            let out = (point - middle).with_y(0.0).normalize_or_zero() + Vec3::Y * 0.3;
            let uv = Vec2::new(
                (frame as f32 + 0.03 + 0.94 * p) / FIRE_FRAMES as f32,
                (1.0 - y / h).clamp(0.0, 0.99),
            );
            (point, out.normalize_or_zero(), uv)
        };
        for m in 0..ACROSS {
            for r in 0..UP {
                let q = [at(m, r), at(m + 1, r), at(m + 1, r + 1), at(m, r + 1)];
                sink.smooth_face(
                    FIRE,
                    &q.map(|c| c.0),
                    &q.map(|c| c.1),
                    Some(&q.map(|c| c.2)),
                );
            }
        }
    }
}

/// The anvil at the mockup's `(x, z)`, along x, on its stump: one iron form,
/// smooth across its curves, lofted from a spread foot in to a waist and out
/// to a flat face (a hard crease only at the face's edge), its horn lofted
/// round off the face's `-x` end from the face's own section, tapering to a
/// point, its heel square. A solid to the walker.
fn anvil(sink: &mut Sink, place: &Place, (x, z): (f32, f32)) {
    const N: usize = 16;
    let cx = x + 0.04;
    stump(sink, place, (cx, z));
    // The body's sections up from the stump: height, half along, half
    // across, and how square (2 round, higher squarer).
    let body = [
        (STUMP_M, 0.2, 0.17, 4.0),
        (STUMP_M + 0.04, 0.19, 0.155, 4.0),
        (0.42, 0.12, 0.09, 3.0),
        (0.5, 0.14, 0.095, 3.0),
        (0.58, 0.24, 0.11, 4.0),
        (0.64, 0.27, 0.12, 5.0),
        (ANVIL_TOP_M, 0.27, 0.12, 5.0),
    ];
    let section = |y: f32, a: f32, b: f32, sq: f32| -> Vec<Vec3> {
        let e = 2.0 / sq;
        (0..N)
            .map(|k| {
                let t = k as f32 / N as f32 * std::f32::consts::TAU;
                let (c, s) = (t.cos(), t.sin());
                place.at(
                    cx + a * c.signum() * c.abs().powf(e),
                    y,
                    z + b * s.signum() * s.abs().powf(e),
                )
            })
            .collect()
    };
    let rings: Vec<Vec<Vec3>> = body
        .iter()
        .map(|&(y, a, b, sq)| section(y, a, b, sq))
        .collect();
    let axis: Vec<Vec3> = body.iter().map(|&(y, ..)| place.at(cx, y, z)).collect();
    loft(sink, ANVIL, &rings, &axis);
    sink.face(ANVIL_FACE, &rings[rings.len() - 1], Vec3::Y, None);
    // The horn: round sections along -x, from inside the face's end out to
    // its point, its top a hair under the face so the two never fight.
    let horn = [
        (-0.20, 0.66, 0.055, 0.1),
        (-0.29, 0.658, 0.05, 0.085),
        (-0.38, 0.652, 0.04, 0.064),
        (-0.47, 0.641, 0.027, 0.042),
        (-0.54, 0.631, 0.014, 0.022),
        (-0.59, 0.625, 0.0, 0.0),
    ];
    let rings: Vec<Vec<Vec3>> = horn
        .iter()
        .map(|&(dx, yc, hy, hz)| {
            (0..N)
                .map(|k| {
                    let t = k as f32 / N as f32 * std::f32::consts::TAU;
                    place.at(cx + dx, yc + hy * t.sin(), z + hz * t.cos())
                })
                .collect()
        })
        .collect();
    let axis: Vec<Vec3> = horn
        .iter()
        .map(|&(dx, yc, ..)| place.at(cx + dx, yc, z))
        .collect();
    loft(sink, ANVIL, &rings, &axis);
    let c = place.plan(x, z);
    sink.solid_box(c.x, 0.0, c.y, Vec3::new(0.62, 0.72, 0.42), place.turn(0.0));
}

/// The elm stump the anvil stands on, round `(x, z)`: a little uneven round,
/// flaring at its foot into the floor, its sides bark and smooth, its top
/// sawn end grain.
fn stump(sink: &mut Sink, place: &Place, (x, z): (f32, f32)) {
    const WOBBLE: [f32; 12] = [
        1.0, 0.96, 1.03, 0.98, 1.02, 0.95, 1.01, 1.04, 0.97, 1.0, 0.94, 1.02,
    ];
    let ring = |y: f32, r: f32| -> Vec<Vec3> {
        WOBBLE
            .iter()
            .enumerate()
            .map(|(k, w)| {
                let a = k as f32 / WOBBLE.len() as f32 * std::f32::consts::TAU;
                place.at(x + r * w * a.cos(), y, z + r * w * a.sin())
            })
            .collect()
    };
    let levels = [
        (-0.05, STUMP_R_M * 1.12),
        (0.05, STUMP_R_M * 1.03),
        (STUMP_M, STUMP_R_M * 0.97),
    ];
    let rings: Vec<Vec<Vec3>> = levels.iter().map(|&(y, r)| ring(y, r)).collect();
    let axis: Vec<Vec3> = levels.iter().map(|&(y, _)| place.at(x, y, z)).collect();
    loft(sink, "bark", &rings, &axis);
    sink.face("timber", &rings[2], Vec3::Y, None);
}

/// The smithy of `smithy`, on its forge cell's terrace `floor_m` over the
/// radius. `None` where its cells are not charted.
pub fn smithy(
    sink_meshes: &mut super::Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    smithy: &Smithy,
    cell_m: f32,
    radius_m: f32,
    floor_m: f32,
) -> Option<BuildingSolids> {
    let (fc, fr) = smithy.forge_cell(cell_m);
    let forge_at = sea::centre(fc, fr, cell_m);
    let place = Place::new(chart, patch, cell_m, forge_at, radius_m, floor_m)?;
    let frame = place.frame;
    let mut sink = Sink::new(sink_meshes, repeat_m, frame);
    let r_m = cell_m / 3f32.sqrt();
    let centres: Vec<(f32, f32)> = smithy
        .cells
        .iter()
        .map(|&[c, r]| sea::centre(c, r, cell_m))
        .collect();
    let fold = |f: fn(f32, f32) -> f32, k: usize, init: f32| {
        centres
            .iter()
            .map(|p| if k == 0 { p.0 } else { p.1 })
            .fold(init, f)
    };
    let x0 = fold(f32::min, 0, f32::MAX) - cell_m / 2.0;
    let x1 = fold(f32::max, 0, f32::MIN) + cell_m / 2.0;
    let z0 = fold(f32::min, 1, f32::MAX) - r_m;
    let z1 = fold(f32::max, 1, f32::MIN) + r_m;
    let xm = (x0 + x1) / 2.0;
    for p in [(x0, z0), (x1, z0), (x0, z1), (x1, z1), (xm, z0), (xm, z1)] {
        post(&mut sink, &place, p, EAVE_M);
    }
    roof(&mut sink, &place, (x0, x1), (z0, z1));
    let ridge = EAVE_M + PITCH * (z1 - z0) / 2.0;
    forge(&mut sink, &place, forge_at, ridge + 0.5);
    anvil(&mut sink, &place, sea::centre(fc - 1, fr, cell_m));
    let reach = place.plan(x0, z0).length().max(place.plan(x1, z1).length()) + 1.0;
    Some(finish(sink, frame, reach))
}
