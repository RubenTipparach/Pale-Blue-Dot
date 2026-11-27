//! The weapons' models (`survival-mode` design 16, task 4.10): the stone,
//! bronze and iron spears, the bow, the three arrows, the hunting axes and
//! the swords, built as the approved weapons mockup builds them
//! (`docs/mockups/survival/weapons.js`, owner's survey G10).
//!
//! - **One fitted form a part**, never a box stuck on: a shaft and a head
//!   are lathed, a bow's limb and an axe's handle are tubes swept along a
//!   curve and tapered, a blade, a guard and an axe's head are outlines
//!   extruded with a rounded bevel. Every part is welded and its normals
//!   averaged, so it is smooth-shaded round its curves.
//! - **Textured**: each part wears one of the mockup's own 16 or 32 px
//!   textures (`tools/art/weapon_textures.py`, in the settlement set with
//!   their manifest rows), sampled nearest, mapped as the mockup maps it. A
//!   binding or a grip wrap is painted in, never modelled.
//! - Metres, the butt or the pommel at the origin and the weapon along +Y,
//!   its flat side facing +Z.
//!
//! What is built here is drawn in the hand (`held::weapon`) and lying in
//! the world as a drop (`drops`). Drawn only: no rule reads it.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::goods::Good;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// The weapons with a model, as goods.
pub const WEAPONS: [Good; 14] = [
    Good::STONE_SPEAR,
    Good::BRONZE_SPEAR,
    Good::IRON_SPEAR,
    Good::BOW,
    Good::STONE_ARROW,
    Good::BONE_ARROW,
    Good::BRONZE_ARROW,
    Good::BRONZE_HUNTING_AXE,
    Good::IRON_HUNTING_AXE,
    Good::BRONZE_SWORD,
    Good::IRON_SWORD,
    // The straw (design 15) is held as a tool too: a thin reed.
    Good::STRAW,
    // Held up to show, as the weapons are: what is worn, before a worn
    // slot draws it on the body.
    Good::STRAW_HAT,
    Good::WOOL_CLOAK,
];

/// One textured part: its mesh in metres and the settlement texture it
/// wears (`textures/settlement/<name>.png`), and whether it is seen from
/// both sides (a feather's vane).
#[derive(Clone, Debug, Default)]
pub struct Part {
    pub texture: &'static str,
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
    pub two_sided: bool,
}

impl Part {
    fn new(texture: &'static str) -> Self {
        Self {
            texture,
            ..default()
        }
    }

    /// Weld points that coincide and average the faces' normals into them,
    /// as the mockup's `weld` and `computeVertexNormals` do: smooth across
    /// the whole part. A seam whose two sides take different texture
    /// coordinates keeps both points but shares their normal.
    fn smooth(&mut self) {
        let key = |p: Vec3| {
            (
                (p.x * 1e4).round() as i64,
                (p.y * 1e4).round() as i64,
                (p.z * 1e4).round() as i64,
            )
        };
        let mut sums: std::collections::HashMap<(i64, i64, i64), Vec3> = Default::default();
        for t in self.indices.chunks(3) {
            let [a, b, c] = [t[0], t[1], t[2]].map(|i| self.positions[i as usize]);
            let face = (b - a).cross(c - a);
            for p in [a, b, c] {
                *sums.entry(key(p)).or_default() += face;
            }
        }
        self.normals = self
            .positions
            .iter()
            .map(|p| {
                sums.get(&key(*p))
                    .copied()
                    .unwrap_or(Vec3::Y)
                    .normalize_or(Vec3::Y)
            })
            .collect();
    }

    fn append(&mut self, other: Part) {
        let base = self.positions.len() as u32;
        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.uvs.extend(other.uvs);
        self.indices.extend(other.indices.iter().map(|i| i + base));
    }

    /// Scale every point (and its normal, inversely) by `s` about the
    /// origin, as three's `mesh.scale` does to the drawn part.
    fn scaled(mut self, s: Vec3) -> Self {
        for p in &mut self.positions {
            *p *= s;
        }
        for n in &mut self.normals {
            *n = (*n / s).normalize_or(Vec3::Y);
        }
        self
    }

    fn moved(mut self, by: Vec3, turn: Quat) -> Self {
        for p in &mut self.positions {
            *p = turn * *p + by;
        }
        for n in &mut self.normals {
            *n = turn * *n;
        }
        self
    }

    fn uv_scaled(mut self, k: Vec2) -> Self {
        for uv in &mut self.uvs {
            *uv *= k;
        }
        self
    }

    /// The mesh, lit by the world (a drop): positions, normals, UVs.
    pub fn mesh(&self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.positions
                .iter()
                .map(|p| p.to_array())
                .collect::<Vec<_>>(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            self.normals
                .iter()
                .map(|p| p.to_array())
                .collect::<Vec<_>>(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            self.uvs.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
        )
        .with_inserted_indices(Indices::U32(self.indices.clone()))
    }

    /// The mesh for the hand, drawn unlit with the field laid over it: the
    /// same, with a light fixed in its own frame baked into vertex colours
    /// (the held torch's `shaded`), so it reads round.
    pub fn held_mesh(&self) -> Mesh {
        let l = Vec3::new(0.45, 0.8, 0.4).normalize();
        let colours: Vec<[f32; 4]> = self
            .normals
            .iter()
            .map(|n| {
                let k = 0.55 + 0.45 * n.dot(l).max(0.0);
                [k, k, k, 1.0]
            })
            .collect();
        self.mesh()
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    }
}

/// A lathe about +Y (three's `LatheGeometry`): `points` are (radius, y)
/// from the bottom up, turned through `segments`; u round, v up the
/// points.
fn lathe(texture: &'static str, points: &[(f32, f32)], segments: usize) -> Part {
    let mut part = Part::new(texture);
    let rows = points.len();
    for i in 0..=segments {
        let phi = i as f32 / segments as f32 * TAU;
        let (s, c) = phi.sin_cos();
        for (j, &(r, y)) in points.iter().enumerate() {
            part.positions.push(Vec3::new(r * s, y, r * c));
            part.uvs.push(Vec2::new(
                i as f32 / segments as f32,
                j as f32 / (rows - 1) as f32,
            ));
        }
    }
    for i in 0..segments {
        for j in 0..rows - 1 {
            let a = (i * rows + j) as u32;
            let b = ((i + 1) * rows + j) as u32;
            let (c, d) = (b + 1, a + 1);
            part.indices.extend([a, b, d, b, c, d]);
        }
    }
    part.smooth();
    part
}

/// A Catmull-Rom curve through `points` (three's centripetal default is
/// not needed at these sizes: uniform), sampled at `t` in 0..1 by arc
/// length closely enough for a tube's rings.
fn catmull(points: &[Vec3], t: f32) -> Vec3 {
    let n = points.len();
    let f = t.clamp(0.0, 1.0) * (n - 1) as f32;
    let i = (f.floor() as usize).min(n - 2);
    let u = f - i as f32;
    let p0 = points[i.saturating_sub(1)];
    let p1 = points[i];
    let p2 = points[i + 1];
    let p3 = points[(i + 2).min(n - 1)];
    let (u2, u3) = (u * u, u * u * u);
    0.5 * ((2.0 * p1)
        + (-p0 + p2) * u
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u2
        + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * u3)
}

/// The mockup's `taperTube`: a tube of radius `r` swept along `points`
/// (`rings` + 1 rings of 10 sides), each ring scaled by `f(t)` and squeezed
/// by `k` across Z (the flat of a bow's limb, an axe's handle); u along,
/// v round, as three's `TubeGeometry`.
fn taper_tube(
    texture: &'static str,
    points: &[Vec3],
    rings: usize,
    r: f32,
    f: impl Fn(f32) -> f32,
    k: f32,
) -> Part {
    const SIDES: usize = 10;
    let mut part = Part::new(texture);
    let at = |t: f32| catmull(points, t);
    let mut normal = Vec3::Z;
    for i in 0..=rings {
        let t = i as f32 / rings as f32;
        let c = at(t);
        let ahead = (at((t + 1e-3).min(1.0)) - at((t - 1e-3).max(0.0))).normalize_or(Vec3::Y);
        // A frame carried along the curve: the last normal made square to
        // the tangent.
        normal = (normal - ahead * normal.dot(ahead)).normalize_or(Vec3::X);
        let binormal = ahead.cross(normal);
        let s = f(t);
        for j in 0..=SIDES {
            let a = j as f32 / SIDES as f32 * TAU;
            let mut off = (normal * a.cos() + binormal * a.sin()) * (r * s);
            off.z *= k;
            part.positions.push(c + off);
            part.uvs
                .push(Vec2::new(i as f32 / rings as f32, j as f32 / SIDES as f32));
        }
    }
    let w = (SIDES + 1) as u32;
    for i in 0..rings as u32 {
        for j in 0..SIDES as u32 {
            let a = i * w + j;
            let b = (i + 1) * w + j;
            // Wound so the faces look out from the curve.
            part.indices.extend([a, a + 1, b, b, a + 1, b + 1]);
        }
    }
    part.smooth();
    part
}

/// An outline drawn as three's `Shape` is: moves, lines and quadratic
/// curves, each curve sampled in 18 steps.
struct Outline(Vec<Vec2>);

impl Outline {
    fn at(x: f32, y: f32) -> Self {
        Self(vec![Vec2::new(x, y)])
    }
    fn line(mut self, x: f32, y: f32) -> Self {
        self.0.push(Vec2::new(x, y));
        self
    }
    fn quad(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        let p0 = *self.0.last().unwrap();
        let (c, p) = (Vec2::new(cx, cy), Vec2::new(x, y));
        for k in 1..=18 {
            let t = k as f32 / 18.0;
            self.0
                .push(p0 * (1.0 - t) * (1.0 - t) + c * 2.0 * t * (1.0 - t) + p * t * t);
        }
        self
    }
    /// The points once round, the closing repeat dropped, counter-clockwise.
    fn points(&self) -> Vec<Vec2> {
        let mut pts = self.0.clone();
        if pts.len() > 1 && pts[0].distance(*pts.last().unwrap()) < 1e-6 {
            pts.pop();
        }
        pts.dedup_by(|a, b| a.distance(*b) < 1e-6);
        let area: f32 = (0..pts.len())
            .map(|i| pts[i].perp_dot(pts[(i + 1) % pts.len()]))
            .sum();
        if area < 0.0 {
            pts.reverse();
        }
        pts
    }
}

/// Ear-clip a simple counter-clockwise polygon into triangles.
fn triangulate(pts: &[Vec2]) -> Vec<[usize; 3]> {
    let mut idx: Vec<usize> = (0..pts.len()).collect();
    let mut out = Vec::new();
    let inside = |p: Vec2, a: Vec2, b: Vec2, c: Vec2| {
        let d1 = (b - a).perp_dot(p - a);
        let d2 = (c - b).perp_dot(p - b);
        let d3 = (a - c).perp_dot(p - c);
        d1 > 0.0 && d2 > 0.0 && d3 > 0.0
    };
    let mut guard = 0;
    while idx.len() > 3 && guard < 10_000 {
        guard += 1;
        let n = idx.len();
        let mut clipped = false;
        for i in 0..n {
            let (ia, ib, ic) = (idx[(i + n - 1) % n], idx[i], idx[(i + 1) % n]);
            let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
            if (b - a).perp_dot(c - b) <= 1e-9 {
                continue;
            }
            if idx
                .iter()
                .any(|&j| j != ia && j != ib && j != ic && inside(pts[j], a, b, c))
            {
                continue;
            }
            out.push([ia, ib, ic]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0], idx[1], idx[2]]);
    }
    out
}

/// The outline moved out by `by` along each corner's outward normal.
fn offset(pts: &[Vec2], by: f32) -> Vec<Vec2> {
    let n = pts.len();
    (0..n)
        .map(|i| {
            let (a, b, c) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
            let e1 = (b - a).normalize_or(Vec2::X);
            let e2 = (c - b).normalize_or(Vec2::X);
            // Outward of a counter-clockwise outline is the edge turned
            // clockwise.
            let n1 = Vec2::new(e1.y, -e1.x);
            let n2 = Vec2::new(e2.y, -e2.x);
            let m = (n1 + n2).normalize_or(n1);
            let miter = (1.0 / m.dot(n1).max(0.35)).min(2.5);
            b + m * by * miter
        })
        .collect()
}

/// The mockup's `extrude`: an outline extruded `depth` through Z (centred
/// on it) with a rounded bevel `bevel` deep and `size` out in `segs` steps
/// each side, welded and smooth; its texture laid flat across X and Y
/// times `rep`, as three's world UVs are.
fn extrude(
    texture: &'static str,
    outline: Outline,
    depth: f32,
    bevel: f32,
    size: f32,
    segs: usize,
    rep: f32,
) -> Part {
    let base = outline.points();
    let n = base.len();
    let mut part = Part::new(texture);
    // Rings from the front face round the bevel, down the side and round
    // the back bevel: (offset out, z).
    let mut stops: Vec<(f32, f32)> = Vec::new();
    for k in 0..=segs {
        let a = k as f32 / segs as f32 * FRAC_PI_2;
        stops.push((size * a.sin(), depth * 0.5 + bevel * a.cos()));
    }
    for k in (0..=segs).rev() {
        let a = k as f32 / segs as f32 * FRAC_PI_2;
        stops.push((size * a.sin(), -depth * 0.5 - bevel * a.cos()));
    }
    let rings: Vec<Vec<Vec2>> = stops.iter().map(|&(o, _)| offset(&base, o)).collect();
    for (r, &(_, z)) in rings.iter().zip(&stops) {
        for p in r {
            part.positions.push(Vec3::new(p.x, p.y, z));
            part.uvs.push(*p * rep);
        }
    }
    for r in 0..stops.len() as u32 - 1 {
        for i in 0..n as u32 {
            let j = (i + 1) % n as u32;
            let (a, b) = (r * n as u32 + i, r * n as u32 + j);
            let (c, d) = ((r + 1) * n as u32 + j, (r + 1) * n as u32 + i);
            part.indices.extend([a, d, b, b, d, c]);
        }
    }
    // The two faces share the first and last rings' points.
    let tris = triangulate(&base);
    let last = (stops.len() - 1) as u32 * n as u32;
    for t in &tris {
        part.indices.extend([t[0] as u32, t[1] as u32, t[2] as u32]);
        part.indices
            .extend([last + t[0] as u32, last + t[2] as u32, last + t[1] as u32]);
    }
    part.smooth();
    part
}

/// The mockup's spear: a tapering lathed shaft 1.74 m long with the
/// binding painted near its top (its texture twice round), and a knapped
/// leaf head lathed in six sides and flattened to 0.38 across Z.
pub fn spear(head: &'static str) -> Vec<Part> {
    let mut shaft_pts = vec![(0.0, 0.0)];
    shaft_pts.extend((0..=18).map(|i| (0.017 - 0.002 * i as f32 / 18.0, 1.72 * i as f32 / 18.0)));
    shaft_pts.push((0.0, 1.74));
    let shaft = lathe("weapon_shaft", &shaft_pts, 12).uv_scaled(Vec2::new(2.0, 1.0));
    let head = lathe(
        head,
        &[
            (0.0, 1.62),
            (0.016, 1.63),
            (0.026, 1.69),
            (0.037, 1.78),
            (0.032, 1.87),
            (0.018, 1.95),
            (0.004, 1.995),
            (0.0, 2.0),
        ],
        6,
    )
    .scaled(Vec3::new(1.0, 1.0, 0.38));
    vec![shaft, head]
}

/// The mockup's bow: a limb 1.5 m tall swept on a curve and tapered toward
/// its tips, flat across Z, its leather grip painted in the middle, and the
/// string from tip to tip.
pub fn bow() -> Vec<Part> {
    let pts: Vec<Vec3> = (0..=20)
        .map(|i| {
            let t = i as f32 / 20.0;
            let mut x = -0.13 * (PI * t).sin();
            if t < 0.08 {
                x += 0.04 * (0.08 - t) / 0.08;
            }
            if t > 0.92 {
                x += 0.04 * (t - 0.92) / 0.08;
            }
            Vec3::new(x, 1.5 * t, 0.0)
        })
        .collect();
    let limb = taper_tube(
        "weapon_bow",
        &pts,
        80,
        0.02,
        |t| 0.35 + 0.65 * (PI * t).sin().max(0.0).powf(0.6),
        0.55,
    );
    let a = pts[0] + Vec3::new(0.004, 0.012, 0.0);
    let b = pts[20] + Vec3::new(0.004, -0.012, 0.0);
    let string = taper_tube("weapon_feather", &[a, b], 4, 0.0022, |_| 1.0, 1.0);
    vec![limb, string]
}

/// The mockup's arrow: a lathed shaft 0.71 m, a small flattened head of
/// `head`'s texture, and three fletching vanes round its foot, seen from
/// both sides.
pub fn arrow(head: &'static str) -> Vec<Part> {
    let shaft = lathe(
        "weapon_wood",
        &[(0.0, 0.0), (0.0055, 0.005), (0.0055, 0.7), (0.0, 0.71)],
        8,
    );
    let tip = lathe(
        head,
        &[
            (0.0, 0.69),
            (0.007, 0.695),
            (0.013, 0.715),
            (0.009, 0.735),
            (0.0, 0.755),
        ],
        6,
    )
    .scaled(Vec3::new(1.0, 1.0, 0.4));
    let mut vanes = Part::new("weapon_feather");
    vanes.two_sided = true;
    let vane = Outline::at(0.0, 0.0)
        .quad(0.022, 0.03, 0.02, 0.1)
        .quad(0.01, 0.11, 0.0, 0.12)
        .line(0.0, 0.0)
        .points();
    for k in 0..3 {
        let turn = Quat::from_rotation_y(k as f32 * 2.094);
        let mut one = Part::new("weapon_feather");
        for p in &vane {
            one.positions.push(turn * Vec3::new(p.x, p.y + 0.04, 0.0));
            one.normals.push(turn * Vec3::Z);
            one.uvs.push(*p * 8.0);
        }
        for t in triangulate(&vane) {
            one.indices.extend([t[0] as u32, t[1] as u32, t[2] as u32]);
        }
        vanes.append(one);
    }
    vec![shaft, tip, vanes]
}

/// The mockup's hunting axe: a handle 0.6 m swept on a gentle curve, its
/// leather wrap painted down one edge, and a bearded head extruded with a
/// rounded bevel.
pub fn hunting_axe(head: &'static str) -> Vec<Part> {
    let curve = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.012, 0.2, 0.0),
        Vec3::new(-0.006, 0.42, 0.0),
        Vec3::new(0.0, 0.6, 0.0),
    ];
    let handle = taper_tube(
        "weapon_haft",
        &curve,
        30,
        0.017,
        |t| if t < 0.05 { 1.15 } else { 0.85 + 0.15 * t },
        0.75,
    );
    let outline = Outline::at(-0.03, 0.6)
        .line(-0.03, 0.51)
        .line(0.02, 0.5)
        .quad(0.07, 0.48, 0.1, 0.43)
        .quad(0.14, 0.47, 0.15, 0.55)
        .quad(0.155, 0.6, 0.14, 0.64)
        .quad(0.08, 0.62, 0.03, 0.62)
        .line(-0.03, 0.6);
    let head = extrude(head, outline, 0.016, 0.008, 0.005, 3, 3.0);
    vec![handle, head]
}

/// The mockup's sword: a lathed pommel and grip (the grip's leather wrap
/// painted), a curved guard and a tapering blade with its fuller painted
/// down the middle, both extruded with a rounded bevel.
pub fn sword(cast: &'static str, blade: &'static str) -> Vec<Part> {
    let pommel = lathe(
        cast,
        &[
            (0.0, 0.0),
            (0.016, 0.004),
            (0.024, 0.018),
            (0.02, 0.034),
            (0.012, 0.042),
            (0.011, 0.046),
            (0.0, 0.046),
        ],
        14,
    )
    .scaled(Vec3::new(1.0, 1.0, 0.7));
    let grip = lathe(
        "weapon_leather",
        &[
            (0.0, 0.04),
            (0.013, 0.042),
            (0.015, 0.1),
            (0.013, 0.165),
            (0.0, 0.17),
        ],
        12,
    );
    let guard = Outline::at(-0.095, 0.172)
        .quad(-0.05, 0.165, 0.0, 0.165)
        .quad(0.05, 0.165, 0.095, 0.172)
        .quad(0.1, 0.18, 0.093, 0.19)
        .quad(0.05, 0.185, 0.0, 0.188)
        .quad(-0.05, 0.185, -0.093, 0.19)
        .quad(-0.1, 0.18, -0.095, 0.172);
    let guard = extrude(cast, guard, 0.018, 0.006, 0.004, 3, 6.0);
    let edge = Outline::at(-0.023, 0.186)
        .line(0.023, 0.186)
        .quad(0.02, 0.5, 0.017, 0.74)
        .quad(0.012, 0.85, 0.0, 0.9)
        .quad(-0.012, 0.85, -0.017, 0.74)
        .quad(-0.02, 0.5, -0.023, 0.186);
    let mut blade = extrude(blade, edge, 0.003, 0.0035, 0.006, 3, 2.2);
    // The texture's 16 columns laid once across the blade's width (edge,
    // fuller, edge), so its bright edges and the fuller down the middle
    // read; laid by the metre, as the mockup lays it, the blade took one
    // column and read as flat colour.
    let half = 0.023 + 0.006;
    for (uv, p) in blade.uvs.iter_mut().zip(&blade.positions) {
        uv.x = ((p.x + half) / (2.0 * half)).clamp(0.0, 1.0);
    }
    vec![pommel, grip, guard, blade]
}

/// A straw (design 15): a reed 0.24 m long, lathed thin, in the straw's
/// own colour (the bow's pale wood).
pub fn straw() -> Vec<Part> {
    let mut pts = vec![(0.0, 0.0)];
    pts.extend((0..=8).map(|i| (0.0045, 0.24 * i as f32 / 8.0)));
    pts.push((0.0, 0.24));
    vec![lathe("weapon_feather", &pts, 8)]
}

/// A straw hat (design 13): a brim and a crown lathed as one, in the
/// straw's colour.
pub fn straw_hat() -> Vec<Part> {
    vec![lathe(
        "weapon_feather",
        &[
            (0.0, 0.0),
            (0.2, 0.005),
            (0.22, 0.015),
            (0.2, 0.025),
            (0.11, 0.03),
            (0.1, 0.09),
            (0.07, 0.12),
            (0.0, 0.125),
        ],
        18,
    )]
}

/// A wool cloak folded (design 13): a soft roll lathed about its length,
/// in the wool's colour (the hide's ramp would read as leather).
pub fn wool_cloak() -> Vec<Part> {
    vec![
        lathe(
            "weapon_feather",
            &[
                (0.0, 0.0),
                (0.05, 0.01),
                (0.07, 0.05),
                (0.075, 0.2),
                (0.07, 0.35),
                (0.05, 0.39),
                (0.0, 0.4),
            ],
            12,
        )
        .scaled(Vec3::new(1.0, 1.0, 0.6)),
    ]
}

/// A weapon's model by its good, and where the hand holds it (m up its
/// length); `None` for a good with none.
pub fn model(good: Good) -> Option<(Vec<Part>, f32)> {
    Some(match good {
        g if g == Good::STONE_SPEAR => (spear("weapon_stone"), 0.8),
        g if g == Good::BRONZE_SPEAR => (spear("weapon_bronze"), 0.8),
        g if g == Good::IRON_SPEAR => (spear("weapon_iron"), 0.8),
        g if g == Good::BOW => (bow(), 0.75),
        g if g == Good::STONE_ARROW => (arrow("weapon_stone"), 0.3),
        g if g == Good::BONE_ARROW => (arrow("weapon_feather"), 0.3),
        g if g == Good::BRONZE_ARROW => (arrow("weapon_bronze"), 0.3),
        g if g == Good::BRONZE_HUNTING_AXE => (hunting_axe("weapon_bronze"), 0.12),
        g if g == Good::IRON_HUNTING_AXE => (hunting_axe("weapon_iron"), 0.12),
        g if g == Good::BRONZE_SWORD => (sword("weapon_bronze", "weapon_blade_bronze"), 0.1),
        g if g == Good::IRON_SWORD => (sword("weapon_iron", "weapon_blade_iron"), 0.1),
        g if g == Good::STRAW => (straw(), 0.08),
        g if g == Good::STRAW_HAT => (straw_hat(), 0.0),
        g if g == Good::WOOL_CLOAK => (wool_cloak(), 0.2),
        _ => return None,
    })
}

/// A model's length along Y, m: its highest point.
pub fn length(parts: &[Part]) -> f32 {
    parts
        .iter()
        .flat_map(|p| p.positions.iter())
        .map(|p| p.y)
        .fold(0.0, f32::max)
}

/// The settlement textures every weapon wears, for a test and the
/// loading screen's prewarm.
pub fn textures() -> Vec<&'static str> {
    let mut all: Vec<&'static str> = WEAPONS
        .iter()
        .filter_map(|g| model(*g))
        .flat_map(|(parts, _)| parts.into_iter().map(|p| p.texture))
        .collect();
    all.sort_unstable();
    all.dedup();
    all
}

/// A part turned about its own Y and set at `at`: for laying a model down.
pub fn laid(parts: &[Part], at: Vec3, turn: Quat) -> Vec<Part> {
    parts.iter().cloned().map(|p| p.moved(at, turn)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every weapon has a model of textured parts, each whole and facing
    /// out: every triangle's winding agrees with its smooth normals, no
    /// point is lost, and it is the mockup's length.
    #[test]
    fn every_weapon_is_a_whole_textured_smooth_model_of_its_length() {
        let manifest = std::fs::read_to_string(format!(
            "{}/textures/settlement/manifest.ron",
            crate::paths::DEV_ASSETS
        ))
        .unwrap();
        for good in WEAPONS {
            let (parts, grip) = model(good).unwrap_or_else(|| panic!("{}", good.code()));
            assert!(!parts.is_empty());
            let len = length(&parts);
            assert!(
                grip >= 0.0 && grip < len,
                "{}: grip {grip} on {len}",
                good.code()
            );
            for part in &parts {
                assert!(
                    manifest.contains(&format!("\"{}\"", part.texture)),
                    "{}: {} has a manifest row",
                    good.code(),
                    part.texture
                );
                assert_eq!(part.positions.len(), part.normals.len());
                assert_eq!(part.positions.len(), part.uvs.len());
                assert!(
                    part.indices
                        .iter()
                        .all(|&i| (i as usize) < part.positions.len())
                );
                assert!(
                    part.positions.iter().all(|p| p.is_finite()),
                    "{} {}",
                    good.code(),
                    part.texture
                );
                assert!(part.normals.iter().all(|n| (n.length() - 1.0).abs() < 1e-3));
                if part.two_sided {
                    continue;
                }
                // Smooth: a point's normal is shared with its welded twins,
                // and most faces agree with the normals at their corners.
                let mut agree = 0;
                for t in part.indices.chunks(3) {
                    let [a, b, c] = [t[0], t[1], t[2]].map(|i| part.positions[i as usize]);
                    let face = (b - a).cross(c - a);
                    let n = part.normals[t[0] as usize]
                        + part.normals[t[1] as usize]
                        + part.normals[t[2] as usize];
                    if face.length() < 1e-12 || face.dot(n) > 0.0 {
                        agree += 1;
                    }
                }
                let tris = part.indices.len() / 3;
                assert!(
                    agree * 10 >= tris * 9,
                    "{} {}: {agree} of {tris} faces out",
                    good.code(),
                    part.texture
                );
            }
        }
        let close = |a: f32, b: f32| (a - b).abs() < 0.02;
        assert!(close(length(&spear("weapon_stone")), 2.0));
        assert!(close(length(&bow()), 1.5));
        assert!(close(length(&arrow("weapon_stone")), 0.755));
        assert!(close(length(&hunting_axe("weapon_bronze")), 0.64));
        assert!(close(
            length(&sword("weapon_bronze", "weapon_blade_bronze")),
            0.9 + 0.0035
        ));
    }

    /// A curved part is smooth: the shaft's normals turn round it point by
    /// point (no two neighbours round a ring share one), as a flat-shaded
    /// one's would not.
    #[test]
    fn a_shaft_is_smooth_shaded_round() {
        let shaft = &spear("weapon_stone")[0];
        let rows = 21;
        let a = shaft.normals[rows + 5];
        let b = shaft.normals[2 * rows + 5];
        assert!(a.dot(b) < 0.999 && a.dot(b) > 0.7, "{a} {b}");
    }

    /// Every outline triangulates whole: its triangles' area is its own.
    #[test]
    fn outlines_triangulate_whole() {
        let pts = Outline::at(-0.023, 0.186)
            .line(0.023, 0.186)
            .quad(0.02, 0.5, 0.017, 0.74)
            .quad(0.012, 0.85, 0.0, 0.9)
            .quad(-0.012, 0.85, -0.017, 0.74)
            .quad(-0.02, 0.5, -0.023, 0.186)
            .points();
        let area: f32 = (0..pts.len())
            .map(|i| pts[i].perp_dot(pts[(i + 1) % pts.len()]))
            .sum::<f32>()
            * 0.5;
        let tris: f32 = triangulate(&pts)
            .iter()
            .map(|t| (pts[t[1]] - pts[t[0]]).perp_dot(pts[t[2]] - pts[t[0]]) * 0.5)
            .sum();
        assert!((area - tris).abs() < 1e-6, "{area} {tris}");
    }
}
