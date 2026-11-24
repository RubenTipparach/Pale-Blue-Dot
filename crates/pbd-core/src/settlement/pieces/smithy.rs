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
//! shape (a waisted foot, a face and a horn), the posts tapered and smooth
//! round. The forge's light is not here: it is a `Brazier` lamp in the
//! voxel field at the forge cell's middle ([`super::super::record::lamps_of`]),
//! so it lights everything near. The forge stands centred on that cell, its
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
    let fire = place.plan(x - 0.1, z);
    sink.flame("flame", Vec3::new(fire.x, bed, fire.y), 0.5);
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

/// The anvil at the mockup's `(x, z)`, along x: a waisted foot, its face,
/// and a horn tapering off its `-x` end to a point. A solid to the walker.
fn anvil(sink: &mut Sink, place: &Place, (x, z): (f32, f32)) {
    // The foot, drawn in to its waist.
    frustum(
        sink,
        place,
        "iron",
        None,
        (x, z),
        (0.5, 0.42, 0.0),
        (0.3, 0.22, 0.48),
        (0.0, 0.0),
    );
    // The body, flaring out to the face.
    frustum(
        sink,
        place,
        "iron",
        Some("iron"),
        (x, z),
        (0.3, 0.22, 0.48),
        (0.62, 0.26, 0.72),
        (0.04, 0.0),
    );
    // The horn: from the face's end to a point, lofted down a little.
    let (hx, top) = (x + 0.04 - 0.31, 0.72);
    let root = [
        place.at(hx, top, z - 0.12),
        place.at(hx, top, z + 0.12),
        place.at(hx, top - 0.18, z + 0.08),
        place.at(hx, top - 0.18, z - 0.08),
    ];
    let tip = place.at(hx - 0.32, top - 0.06, z);
    let middle = place.at(hx - 0.1, top - 0.08, z);
    for k in 0..4 {
        let tri = [root[k], root[(k + 1) % 4], tip];
        let c = (tri[0] + tri[1] + tri[2]) / 3.0;
        sink.face("iron", &tri, c - middle, None);
    }
    let c = place.plan(x, z);
    sink.solid_box(c.x, 0.0, c.y, Vec3::new(0.62, 0.72, 0.42), place.turn(0.0));
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
