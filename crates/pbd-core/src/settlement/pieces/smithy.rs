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
/// The fire (design 26b): the bed of coals' grid over the hearth bed, its
/// lowest rise at the bed's walls and its highest in the heap, metres.
const COAL_GRID: (usize, usize) = (10, 7);
const COAL_EDGE_M: f32 = 0.012;
const COAL_HEAP_M: f32 = 0.07;
/// The flame body: three ribbons crossed at 60 degrees through the fire's
/// middle, each `rows` rows, its width at the root and the top, and its
/// height and lean (along and across the forge, at the top), metres.
pub const FLAME_ROWS: usize = 4;
const FLAME_ROOT_W_M: f32 = 0.56;
const FLAME_TOP_W_M: f32 = 0.3;
const FLAME_RIBBONS: [(f32, f32, f32); 3] =
    [(0.55, 0.05, 0.02), (0.47, -0.03, 0.05), (0.5, 0.02, -0.05)];
/// The pieces' names for the fire's coals and its flame body, drawn unlit.
pub const EMBERS: &str = "embers";
pub const FORGEFIRE: &str = "forgefire";
/// The anvil's forged iron and its polished face.
pub const ANVIL: &str = "anvil";
pub const ANVIL_FACE: &str = "anvilface";

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
    // The fire (design 26b): a bed of coals over the hearth bed, and one
    // flame body standing in it.
    coals(sink, place, (x, z), (iw, id), bed);
    flame(sink, place, (x - 0.1, z), bed + COAL_EDGE_M + 0.02);
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

/// A fixed hash of a grid point to `[0, 1)`, the same every load.
fn hash01(i: usize, j: usize) -> f32 {
    let mut h = (i as u32).wrapping_mul(0x9e37_79b9) ^ (j as u32).wrapping_mul(0x85eb_ca6b);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    (h & 0xffff) as f32 / 65536.0
}

/// The bed of coals filling the hearth bed (design 26b): one low, lumpy
/// surface over a grid, in `embers`, its edge just over the bed where it
/// meets the bed's walls and heaped toward the fire's middle.
fn coals(sink: &mut Sink, place: &Place, (x, z): (f32, f32), (iw, id): (f32, f32), bed: f32) {
    let (nx, nz) = COAL_GRID;
    let height = |i: usize, j: usize| {
        let (a, b) = (
            i as f32 / nx as f32 * 2.0 - 1.0,
            j as f32 / nz as f32 * 2.0 - 1.0,
        );
        if i == 0 || j == 0 || i == nx || j == nz {
            return bed + COAL_EDGE_M;
        }
        // Heaped toward the fire's middle (0.1 m toward -x), lumpy over it.
        let heap = (1.0 - ((a + 0.18) * (a + 0.18) * 0.8 + b * b)).max(0.0);
        bed + COAL_EDGE_M + 0.012 + COAL_HEAP_M * heap * (0.45 + 0.55 * hash01(i, j))
    };
    let at = |i: usize, j: usize| {
        let a = -iw + 2.0 * iw * i as f32 / nx as f32;
        let b = -id + 2.0 * id * j as f32 / nz as f32;
        place.at(x + a, height(i, j), z + b)
    };
    for i in 0..nx {
        for j in 0..nz {
            let q = [at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)];
            // Split along alternate diagonals so the lumps do not run in rows.
            let (t0, t1) = if (i + j) % 2 == 0 {
                ([q[0], q[1], q[2]], [q[0], q[2], q[3]])
            } else {
                ([q[0], q[1], q[3]], [q[1], q[2], q[3]])
            };
            sink.face(EMBERS, &t0, Vec3::Y, None);
            sink.face(EMBERS, &t1, Vec3::Y, None);
        }
    }
}

/// The flame body (design 26b): three ribbons crossed at 60 degrees through
/// the fire's middle `(x, z)`, rooted at `root` among the coals, each a
/// strip of [`FLAME_ROWS`] rows narrowing to its top and leaning a little,
/// its tongues cut by the `forgefire` texture's alpha, mapped once over it.
fn flame(sink: &mut Sink, place: &Place, (x, z): (f32, f32), root: f32) {
    for (r, &(h, lean_x, lean_z)) in FLAME_RIBBONS.iter().enumerate() {
        let a = r as f32 * std::f32::consts::FRAC_PI_3 + 0.26;
        let d = Vec2::new(a.cos(), a.sin());
        // Every other ribbon is mapped the other way round, so the tongues
        // stand differently from every side.
        let flip = r % 2 == 1;
        let level = |k: usize| {
            let f = k as f32 / FLAME_ROWS as f32;
            let half = (FLAME_ROOT_W_M + (FLAME_TOP_W_M - FLAME_ROOT_W_M) * f) / 2.0;
            let lean = Vec2::new(lean_x, lean_z) * f.powf(1.5);
            let c = Vec2::new(x, z) + lean;
            let y = root + h * f;
            let v = 0.985 - 0.975 * f;
            let (u0, u1) = if flip { (1.0, 0.0) } else { (0.0, 1.0) };
            (
                place.at(c.x - d.x * half, y, c.y - d.y * half),
                place.at(c.x + d.x * half, y, c.y + d.y * half),
                Vec2::new(u0, v),
                Vec2::new(u1, v),
            )
        };
        for k in 0..FLAME_ROWS {
            let (l0, r0, ul0, ur0) = level(k);
            let (l1, r1, ul1, ur1) = level(k + 1);
            let n = (r0 - l0).cross(l1 - l0).normalize_or_zero();
            for (pts, uvs) in [
                ([l0, r0, r1], [ul0, ur0, ur1]),
                ([l0, r1, l1], [ul0, ur1, ul1]),
            ] {
                sink.smooth_face(FORGEFIRE, &pts, &[n; 3], Some(&uvs));
            }
        }
    }
}

/// One cross-section of the anvil, across it at `u` along it: its bottom
/// and top, its half-width, and how round it is (0 a bevelled block, 1 an
/// ellipse).
#[derive(Clone, Copy)]
struct Section {
    u: f32,
    yb: f32,
    yt: f32,
    w: f32,
    round: f32,
}

const fn sec(u: f32, yb: f32, yt: f32, w: f32, round: f32) -> Section {
    Section {
        u,
        yb,
        yt,
        w,
        round,
    }
}

/// The anvil's sections from the heel's end to the horn's root, then the
/// horn's tip (design 26b): a square heel, the face over the body, a step
/// down to the table, and the horn tapering to a rounded point that curves
/// down a little. Metres, `u` along the anvil (the horn toward `-u`).
const SECTIONS: [Section; 13] = [
    sec(0.34, 0.62, 0.74, 0.115, 0.0),
    sec(0.27, 0.585, 0.74, 0.12, 0.0),
    sec(0.17, 0.53, 0.74, 0.12, 0.0),
    sec(0.08, 0.52, 0.74, 0.12, 0.0),
    sec(0.0, 0.52, 0.74, 0.12, 0.0),
    sec(-0.08, 0.52, 0.74, 0.12, 0.0),
    sec(-0.165, 0.53, 0.74, 0.12, 0.0),
    sec(-0.18, 0.55, 0.705, 0.11, 0.1),
    sec(-0.25, 0.585, 0.7, 0.1, 0.3),
    sec(-0.33, 0.61, 0.69, 0.08, 0.6),
    sec(-0.41, 0.625, 0.68, 0.06, 0.85),
    sec(-0.48, 0.63, 0.666, 0.042, 1.0),
    sec(-0.54, 0.627, 0.651, 0.024, 1.0),
];
/// How far the anvil stands along x off its cell's middle, so its heel is
/// at the end of its walker's box and its horn reaches out past the other.
pub const ANVIL_SHIFT_M: f32 = -0.03;
/// The horn's rounded point.
const HORN_TIP: Vec3 = Vec3::new(-0.575, 0.632, 0.0);
/// The sections over the body (their bottoms are where the waist joins).
const BODY: std::ops::RangeInclusive<usize> = 2..=6;
/// The body below the face, from the join down to the foot: each level's
/// height, its half-length along and its half-width across, metres. The
/// waist, then the foot flared on four sides.
const LEVELS: [(f32, f32, f32); 6] = [
    (0.45, 0.13, 0.075),
    (0.36, 0.1, 0.062),
    (0.24, 0.13, 0.09),
    (0.13, 0.19, 0.145),
    (0.05, 0.235, 0.185),
    (0.0, 0.25, 0.2),
];
/// Points round a section: from its bottom on the `+z` side, up, over the
/// top and down to its bottom on the `-z` side.
const RING: usize = 12;
/// A section's points as a bevelled block, (across, up) in its half-sizes:
/// a flat top with a bevel to each side, a small bevel at the bottom.
const BLOCK: [(f32, f32); RING] = [
    (0.75, -1.0),
    (1.0, -0.85),
    (1.0, -0.3),
    (1.0, 0.3),
    (1.0, 0.8),
    (0.8, 1.0),
    (-0.8, 1.0),
    (-1.0, 0.8),
    (-1.0, 0.3),
    (-1.0, -0.3),
    (-1.0, -0.85),
    (-0.75, -1.0),
];

/// The anvil's surface: its points, its triangles and which of them are on
/// the polished face.
pub type AnvilSurface = (Vec<Vec3>, Vec<[u32; 3]>, Vec<bool>);

/// The anvil as one closed surface (design 26b), in its own metres (`x`
/// along it, the horn toward `-x`; `y` up from its foot; `z` across): its
/// points, its triangles wound to face out, and whether each is on the
/// polished face. Lofted: the sections from the heel to the horn's tip, and
/// from the body's underside down through the waist to the flared foot,
/// joined where the waist meets the body, every edge shared by two faces.
pub fn anvil_surface() -> AnvilSurface {
    let mut points: Vec<Vec3> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();
    let mut face: Vec<bool> = Vec::new();
    let quad = |tris: &mut Vec<[u32; 3]>, face: &mut Vec<bool>, q: [u32; 4], polished: bool| {
        tris.push([q[0], q[1], q[2]]);
        tris.push([q[0], q[2], q[3]]);
        face.extend([polished, polished]);
    };
    // The upper loft, heel to horn.
    let ring = |s: &Section| -> [Vec3; RING] {
        let (yc, h) = ((s.yb + s.yt) / 2.0, (s.yt - s.yb) / 2.0);
        std::array::from_fn(|k| {
            let th = (-75.0 + 30.0 * k as f32).to_radians();
            let (bz, by) = BLOCK[k];
            let z = bz + (th.cos() - bz) * s.round;
            let y = by + (th.sin() - by) * s.round;
            Vec3::new(s.u, yc + y * h, z * s.w)
        })
    };
    let at = |s: usize, k: usize| (s * RING + k) as u32;
    for s in &SECTIONS {
        points.extend(ring(s));
    }
    for s in 0..SECTIONS.len() - 1 {
        for k in 0..RING {
            let k1 = (k + 1) % RING;
            // The underside over the body is open: the waist joins there.
            if k1 == 0 && BODY.contains(&s) && BODY.contains(&(s + 1)) {
                continue;
            }
            // The polished top: the face, from the heel's end to the step.
            let polished = k == 5 && s < *BODY.end();
            quad(
                &mut tris,
                &mut face,
                [at(s, k), at(s, k1), at(s + 1, k1), at(s + 1, k)],
                polished,
            );
        }
    }
    // The heel's square end, and the horn's point.
    let heel = points.len() as u32;
    points.push(points[..RING].iter().copied().sum::<Vec3>() / RING as f32);
    let tip = points.len() as u32;
    points.push(HORN_TIP);
    let last = SECTIONS.len() - 1;
    for k in 0..RING {
        let k1 = (k + 1) % RING;
        tris.push([heel, at(0, k1), at(0, k)]);
        tris.push([at(last, k), at(last, k1), tip]);
        face.extend([false, false]);
    }
    // The join: the body's underside, round from the heel side along +z
    // and back along -z, and the levels below it, the same shape.
    let join: Vec<u32> = BODY
        .map(|s| at(s, 0))
        .chain(BODY.rev().map(|s| at(s, RING - 1)))
        .collect();
    let n = join.len();
    let (u_hi, u_lo) = (SECTIONS[*BODY.start()].u, SECTIONS[*BODY.end()].u);
    let (mid, half) = ((u_hi + u_lo) / 2.0, (u_hi - u_lo) / 2.0);
    let mut upper = join.clone();
    for &(y, along, across) in &LEVELS {
        let first = points.len() as u32;
        for &j in &join {
            let p = points[j as usize];
            let t = (p.x - mid) / half;
            points.push(Vec3::new(mid + t * along, y, p.z.signum() * across));
        }
        let lower: Vec<u32> = (0..n as u32).map(|i| first + i).collect();
        for i in 0..n {
            let i1 = (i + 1) % n;
            quad(
                &mut tris,
                &mut face,
                [upper[i], upper[i1], lower[i1], lower[i]],
                false,
            );
        }
        upper = lower;
    }
    // The foot's underside.
    let foot = points.len() as u32;
    points.push(Vec3::new(mid, 0.0, 0.0));
    for i in 0..n {
        tris.push([upper[i], upper[(i + 1) % n], foot]);
        face.push(false);
    }
    // Wound out: the enclosed volume is positive.
    if signed_volume(&points, &tris) < 0.0 {
        for t in &mut tris {
            t.swap(1, 2);
        }
    }
    (points, tris, face)
}

/// The volume a closed surface encloses, positive when its faces wind out.
pub fn signed_volume(points: &[Vec3], tris: &[[u32; 3]]) -> f32 {
    tris.iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| points[i as usize]);
            a.dot(b.cross(c)) / 6.0
        })
        .sum()
}

/// The anvil at the mockup's `(x, z)`, along x, its horn toward `-x`: one
/// lofted surface ([`anvil_surface`]) in `anvil`, its face in `anvilface`,
/// shaded smooth across its curves and hard at its creases (the face's
/// edge, the heel's end, the step, the foot's corners). A solid to the
/// walker, the box it has always been.
fn anvil(sink: &mut Sink, place: &Place, (x, z): (f32, f32)) {
    let (local, mut tris, face) = anvil_surface();
    let points: Vec<Vec3> = local
        .iter()
        .map(|p| place.at(x + ANVIL_SHIFT_M + p.x, p.y, z + p.z))
        .collect();
    // The mockup's plan may be mirrored in the frame: wind out there too.
    if signed_volume(&points, &tris) < 0.0 {
        for t in &mut tris {
            t.swap(1, 2);
        }
    }
    let normal = |t: &[u32; 3]| {
        let [a, b, c] = t.map(|i| points[i as usize]);
        (b - a).cross(c - a)
    };
    let mut around: Vec<Vec<usize>> = vec![Vec::new(); points.len()];
    for (i, t) in tris.iter().enumerate() {
        for &v in t {
            around[v as usize].push(i);
        }
    }
    // Smooth across a crease gentler than this, and never across the
    // face's edge.
    let crease = 50f32.to_radians().cos();
    for (i, t) in tris.iter().enumerate() {
        let n = normal(t).normalize_or_zero();
        let normals = t.map(|v| {
            around[v as usize]
                .iter()
                .filter(|&&j| face[j] == face[i])
                .map(|&j| normal(&tris[j]))
                .filter(|m| m.normalize_or_zero().dot(n) >= crease)
                .sum::<Vec3>()
                .normalize_or(n)
        });
        let pts = t.map(|v| points[v as usize]);
        sink.smooth_face(
            if face[i] { ANVIL_FACE } else { ANVIL },
            &pts,
            &normals,
            None,
        );
    }
    let c = place.plan(x, z);
    sink.solid_box(c.x, 0.0, c.y, Vec3::new(0.62, 0.72, 0.42), place.turn(0.0));
}

/// The smithy of `smithy`, on its forge cell's terrace `floor_m` over the
/// radius. `None` where its cells are not charted.
#[allow(clippy::too_many_arguments)]
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
