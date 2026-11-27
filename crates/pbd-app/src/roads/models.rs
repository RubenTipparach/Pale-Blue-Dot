//! The shapes of the paving work (`townsfolk` design 39a and 41): a survey
//! stake and the two gravel stations, each built as lofted forms (rings
//! joined into one surface, tapered and bevelled where parts meet), never
//! boxes or cylinders set on a surface. Y up, the ground's top at y = 0,
//! metres. Plain colours in the forge's and workbench's pipeline, lit as
//! the terrain is by the entity they hang from (`LitByField`,
//! `LitLikeTerrain`).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::TAU;

/// What a part is made of: one material each, drawn in the towns' own pixel
/// textures (`assets/textures/settlement`), sampled nearest and laid at the
/// manifest's scale, as a town's pieces are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Stuff {
    /// Split timber: posts, rails, a peg.
    Wood,
    /// Iron fittings: a crank, an axle, hoops.
    Iron,
    /// Dressed stone: the grinder's rollers.
    Stone,
    /// A strip of dyed cloth.
    Cloth,
    /// Sawn planks: the hopper, the apron, the trough.
    Plank,
    /// A drum's staves.
    Barrel,
    /// A raw hide (`survival-mode` design 17): the tanning rack's.
    Hide,
}

impl Stuff {
    /// Its texture in `textures/settlement/`, and the metres one repeat of
    /// it covers (the manifest's `repeat_m`, or for a texture drawn to fit
    /// one piece, the size it is laid at here).
    pub fn texture(self) -> (&'static str, f32) {
        match self {
            Stuff::Wood => ("timber", 1.0),
            Stuff::Iron => ("iron", 1.0),
            Stuff::Stone => ("stone", 2.0),
            Stuff::Cloth => ("cloth", 0.25),
            Stuff::Plank => ("plank", 2.0),
            Stuff::Barrel => ("barrel", 1.1),
            Stuff::Hide => ("hide", 0.6),
        }
    }

    pub const ALL: [Stuff; 7] = [
        Stuff::Wood,
        Stuff::Iron,
        Stuff::Stone,
        Stuff::Cloth,
        Stuff::Plank,
        Stuff::Barrel,
        Stuff::Hide,
    ];
}

/// A mesh being built: vertices with normals and texture coordinates in
/// metres over the surface (the material scales them to its texture), and
/// triangles.
#[derive(Default, Clone, Debug)]
pub struct Shape {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}

impl Shape {
    /// Rings of points (each closed, all the same count) joined in order
    /// into one soft-shaded surface, its texture running round each ring by
    /// arc length and along the loft by distance, so the grain follows the
    /// piece; capped at the first and last ring when asked.
    pub fn loft(&mut self, rings: &[Vec<Vec3>], cap_start: bool, cap_end: bool) {
        let n = rings[0].len();
        let w = n + 1;
        let base = self.positions.len() as u32;
        let mut along = 0.0;
        let mut prev_mid: Option<Vec3> = None;
        for ring in rings {
            debug_assert_eq!(ring.len(), n);
            let mid = ring.iter().copied().sum::<Vec3>() / n as f32;
            if let Some(p) = prev_mid {
                along += p.distance(mid);
            }
            prev_mid = Some(mid);
            let mut around = 0.0;
            for k in 0..=n {
                if k > 0 {
                    around += ring[k - 1].distance(ring[k % n]);
                }
                self.positions.push(ring[k % n]);
                self.uvs.push(Vec2::new(around, along));
            }
        }
        let mut normals = vec![Vec3::ZERO; rings.len() * w];
        let at = |r: usize, k: usize| r * w + k;
        for r in 0..rings.len() - 1 {
            for k in 0..n {
                let (a, b, c, d) = (at(r, k), at(r, k + 1), at(r + 1, k + 1), at(r + 1, k));
                let pa = self.positions[base as usize + a];
                let pb = self.positions[base as usize + b];
                let pc = self.positions[base as usize + c];
                let pd = self.positions[base as usize + d];
                let face = (pb - pa).cross(pd - pa) + (pc - pb).cross(pc - pd);
                for i in [a, b, c, d] {
                    normals[i] += face;
                }
                self.indices
                    .extend([a, b, c, a, c, d].into_iter().map(|i| base + i as u32));
            }
        }
        // The seam's two copies of a point share one normal.
        for r in 0..rings.len() {
            let sum = normals[at(r, 0)] + normals[at(r, n)];
            normals[at(r, 0)] = sum;
            normals[at(r, n)] = sum;
        }
        self.normals
            .extend(normals.into_iter().map(|v| v.normalize_or(Vec3::Y)));
        if cap_start {
            self.cap(&rings[0], true);
        }
        if cap_end {
            self.cap(&rings[rings.len() - 1], false);
        }
    }

    /// A flat fan over a ring, facing back along the loft at the start and
    /// on along it at the end, its texture laid flat across it.
    fn cap(&mut self, ring: &[Vec3], start: bool) {
        let mid = ring.iter().copied().sum::<Vec3>() / ring.len() as f32;
        let n = ring.len();
        let mut normal = Vec3::ZERO;
        for k in 0..n {
            normal += (ring[k] - mid).cross(ring[(k + 1) % n] - mid);
        }
        let normal = if start { -normal } else { normal }.normalize_or(Vec3::Y);
        let e1 = (ring[0] - mid).normalize_or(Vec3::X);
        let e2 = normal.cross(e1);
        let flat = |p: Vec3| Vec2::new((p - mid).dot(e1), (p - mid).dot(e2));
        let base = self.positions.len() as u32;
        self.positions.push(mid);
        self.normals.push(normal);
        self.uvs.push(Vec2::ZERO);
        for &p in ring {
            self.positions.push(p);
            self.normals.push(normal);
            self.uvs.push(flat(p));
        }
        for k in 0..n as u32 {
            let (a, b) = (base + 1 + k, base + 1 + (k + 1) % n as u32);
            if start {
                self.indices.extend([base, b, a]);
            } else {
                self.indices.extend([base, a, b]);
            }
        }
    }

    /// The same surface seen from both sides: a strip of cloth.
    pub fn both_sides(&mut self) {
        let count = self.positions.len() as u32;
        let positions = self.positions.clone();
        let uvs = self.uvs.clone();
        let normals: Vec<Vec3> = self.normals.iter().map(|n| -*n).collect();
        let flipped: Vec<u32> = self
            .indices
            .chunks(3)
            .flat_map(|t| [t[0] + count, t[2] + count, t[1] + count])
            .collect();
        self.positions.extend(positions);
        self.uvs.extend(uvs);
        self.normals.extend(normals);
        self.indices.extend(flipped);
    }

    /// Another shape's triangles added to this one.
    pub fn append(&mut self, other: Shape) {
        let offset = self.positions.len() as u32;
        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.uvs.extend(other.uvs);
        self.indices
            .extend(other.indices.into_iter().map(|i| i + offset));
    }

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
}

/// A small deterministic roll in [0, 1) from a seed and a salt.
fn roll(seed: u32, salt: u32) -> f32 {
    let mut x = seed ^ salt.wrapping_mul(0x9e37_79b9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    (x & 0xffff) as f32 / 65536.0
}

/// A ring of `sides` points round `centre` in the plane across `axis`, of
/// radii `rx` (along `across`) and `ry` (along `axis x across`), turned by
/// `turn` of a side.
fn ring(
    centre: Vec3,
    axis: Vec3,
    across: Vec3,
    rx: f32,
    ry: f32,
    sides: usize,
    turn: f32,
) -> Vec<Vec3> {
    let axis = axis.normalize();
    let u = (across - axis * across.dot(axis)).normalize();
    let v = axis.cross(u);
    (0..sides)
        .map(|k| {
            let a = (k as f32 + turn) / sides as f32 * TAU;
            centre + u * (a.cos() * rx) + v * (a.sin() * ry)
        })
        .collect()
}

/// A timber post or rail from `a` to `b`, `r0` thick at `a` and `r1` at `b`,
/// six-sided and bevelled at both ends, its faces slightly uneven.
fn timber(shape: &mut Shape, a: Vec3, b: Vec3, r0: f32, r1: f32, seed: u32) {
    let axis = (b - a).normalize();
    let across = if axis.y.abs() > 0.9 { Vec3::X } else { Vec3::Y };
    let len = a.distance(b);
    let bevel = (r0.min(r1) * 0.6).min(len * 0.2);
    let stops = [
        (0.0, 0.7 * r0),
        (bevel, r0),
        (len - bevel, r1),
        (len, 0.7 * r1),
    ];
    let rings: Vec<Vec<Vec3>> = stops
        .iter()
        .enumerate()
        .map(|(i, &(t, r))| {
            let mut pts = ring(a + axis * t, axis, across, r, r, 6, 0.5);
            for (k, p) in pts.iter_mut().enumerate() {
                let c = a + axis * t;
                *p = c + (*p - c) * (0.92 + 0.12 * roll(seed, (i * 7 + k) as u32));
            }
            pts
        })
        .collect();
    shape.loft(&rings, true, true);
}

/// A drum or roller along `axis` through `centre`: radius `profile(t)` over
/// `t` from -1 to 1 across its `half` length, chamfered at the ends.
fn drum(
    shape: &mut Shape,
    centre: Vec3,
    axis: Vec3,
    half: f32,
    sides: usize,
    profile: impl Fn(f32) -> f32,
) {
    let stops = [-1.0, -0.94, -0.6, -0.2, 0.2, 0.6, 0.94, 1.0];
    let rings: Vec<Vec<Vec3>> = stops
        .iter()
        .map(|&t: &f32| {
            let end = if t.abs() > 0.99 { 0.86 } else { 1.0 };
            let r = profile(t) * end;
            ring(centre + axis * (t * half), axis, Vec3::Y, r, r, sides, 0.0)
        })
        .collect();
    shape.loft(&rings, true, true);
}

/// An open, thick-walled vessel: square rings rising from `y0` (half sizes
/// `h0`) to `y1` (`h1`), walls `wall` thick, a floor at the bottom and a
/// rim at the top. A hopper or a trough.
fn vessel(shape: &mut Shape, centre: Vec3, y0: f32, y1: f32, h0: Vec2, h1: Vec2, wall: f32) {
    let rect = |y: f32, h: Vec2| -> Vec<Vec3> {
        // Four sides with their corners cut, so the walls meet on a bevel.
        let c = (h.x.min(h.y) * 0.18).min(0.05);
        [
            (h.x - c, -h.y),
            (h.x, -h.y + c),
            (h.x, h.y - c),
            (h.x - c, h.y),
            (-h.x + c, h.y),
            (-h.x, h.y - c),
            (-h.x, -h.y + c),
            (-h.x + c, -h.y),
        ]
        .into_iter()
        .map(|(x, z)| centre + Vec3::new(x, y, z))
        .collect()
    };
    let inset = Vec2::splat(wall);
    // Outside, up and over the rim, down the inside to the floor.
    let rings = vec![
        rect(y0, h0),
        rect(y1, h1),
        rect(y1 + wall * 0.4, h1 - inset * 0.5),
        rect(y1, h1 - inset),
        rect(y0 + wall, h0 - inset),
    ];
    shape.loft(&rings, true, true);
}

/// A street lantern's ghost in the road planner (`townsfolk` design 42o),
/// its foot at the origin, `y` up: a six-sided timber post bevelled at its
/// foot, flaring at its collar into the lantern's housing and narrowing to
/// its roof and finial, lofted as one piece at the size the world draws the
/// street lantern (a post to 1.5 m, a lantern 0.3 m across to 1.86 m).
pub fn ghost_lamp() -> Shape {
    // (height, half width) up the lamp.
    const STOPS: [(f32, f32); 11] = [
        (0.0, 0.05),
        (0.05, 0.07),
        (1.38, 0.055),
        (1.46, 0.07),
        (1.5, 0.14),
        (1.53, 0.15),
        (1.7, 0.15),
        (1.75, 0.165),
        (1.8, 0.11),
        (1.86, 0.04),
        (1.9, 0.015),
    ];
    let rings: Vec<Vec<Vec3>> = STOPS
        .iter()
        .map(|&(y, r)| ring(Vec3::Y * y, Vec3::Y, Vec3::X, r, r, 6, 0.5))
        .collect();
    let mut shape = Shape::default();
    shape.loft(&rings, true, true);
    shape
}

/// A survey stake (design 39a): one tapered peg of split timber, its foot
/// sunk under the ground's top, its head split and frayed by the mallet,
/// 0.9 m out of the ground. `seed` varies its grain and fray.
pub fn stake(seed: u32) -> Vec<(Stuff, Shape)> {
    let mut wood = Shape::default();
    let sides = 7;
    let lean = Vec3::new(roll(seed, 1) - 0.5, 0.0, roll(seed, 2) - 0.5) * 0.06;
    let stops: [(f32, f32); 7] = [
        (-0.30, 0.004),
        (-0.12, 0.026),
        (0.0, 0.038),
        (0.55, 0.034),
        (0.82, 0.031),
        (0.86, 0.038),
        (0.90, 0.030),
    ];
    let rings: Vec<Vec<Vec3>> = stops
        .iter()
        .enumerate()
        .map(|(i, &(y, r))| {
            let c = Vec3::Y * y + lean * (y.max(0.0) / 0.9);
            let mut pts = ring(c, Vec3::Y, Vec3::X, r, r * 0.8, sides, roll(seed, 3));
            for (k, p) in pts.iter_mut().enumerate() {
                let jitter = 0.88 + 0.22 * roll(seed, (10 + i * sides + k) as u32);
                *p = c + (*p - c) * jitter;
                // The head's top is split and frayed: each point its own
                // height.
                if i == stops.len() - 1 {
                    p.y += (roll(seed, (90 + k) as u32) - 0.4) * 0.05;
                }
            }
            pts
        })
        .collect();
    wood.loft(&rings, true, true);
    // A strip of dyed cloth tied under the head: a band round the peg and a
    // tail blowing off it, both sides drawn.
    let mut cloth = Shape::default();
    let tie = 0.74;
    let c = Vec3::Y * tie + lean * (tie / 0.9);
    let band = [
        ring(
            c - Vec3::Y * 0.03,
            Vec3::Y,
            Vec3::X,
            0.041,
            0.034,
            sides,
            0.0,
        ),
        ring(
            c + Vec3::Y * 0.03,
            Vec3::Y,
            Vec3::X,
            0.040,
            0.033,
            sides,
            0.0,
        ),
    ];
    cloth.loft(&band, false, false);
    let mut tail = Shape::default();
    let out = Vec3::new(roll(seed, 4) - 0.5, 0.0, roll(seed, 5) - 0.5).normalize_or(Vec3::X);
    let side = out.cross(Vec3::Y).normalize();
    let points: Vec<Vec3> = (0..5)
        .map(|i| {
            let t = i as f32 / 4.0;
            c + out * (0.04 + 0.16 * t) - Vec3::Y * (0.11 * t * t) + side * (0.02 * (t * 5.0).sin())
        })
        .collect();
    let rings: Vec<Vec<Vec3>> = points
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let w = 0.026 * (1.0 - 0.45 * i as f32 / 4.0);
            vec![p - Vec3::Y * w, p + Vec3::Y * w]
        })
        .collect();
    // A two-point "ring" lofts into a single strip of quads (and its
    // closing back face, which is the same strip turned: drawn once each
    // way below).
    let base = tail.positions.len() as u32;
    let mut run = 0.0;
    for (i, r) in rings.iter().enumerate() {
        if i > 0 {
            run += points[i].distance(points[i - 1]);
        }
        tail.positions.extend(r.iter().copied());
        tail.normals.extend([side, side]);
        let w = r[0].distance(r[1]);
        tail.uvs.extend([Vec2::new(run, 0.0), Vec2::new(run, w)]);
    }
    for i in 0..rings.len() as u32 - 1 {
        let (a, b, c2, d) = (
            base + i * 2,
            base + i * 2 + 1,
            base + i * 2 + 3,
            base + i * 2 + 2,
        );
        tail.indices.extend([a, b, c2, a, c2, d]);
    }
    tail.both_sides();
    cloth.append(tail);
    vec![(Stuff::Wood, wood), (Stuff::Cloth, cloth)]
}

/// The grinder (design 41): a hopper over a pair of stone rollers in a
/// timber frame, a crank at its side and an apron the crushed stone falls
/// onto. On a cell's top, about 1.6 m tall.
pub fn grinder() -> Vec<(Stuff, Shape)> {
    let (mut wood, mut iron, mut stone) = (Shape::default(), Shape::default(), Shape::default());
    let mut planks = Shape::default();
    // The frame: four splayed posts and the rails between them.
    let foot = |x: f32, z: f32| Vec3::new(x * 0.58, 0.0, z * 0.46);
    let head = |x: f32, z: f32| Vec3::new(x * 0.5, 1.12, z * 0.38);
    let mut seed = 11;
    for (x, z) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        timber(
            &mut wood,
            foot(x, z) - Vec3::Y * 0.05,
            head(x, z),
            0.075,
            0.06,
            seed,
        );
        seed += 1;
    }
    let at = |x: f32, z: f32, y: f32| foot(x, z).lerp(head(x, z), y / 1.12);
    for y in [0.32, 1.02] {
        for (a, b) in [
            ((-1.0, -1.0), (-1.0, 1.0)),
            ((1.0, -1.0), (1.0, 1.0)),
            ((-1.0, -1.0), (1.0, -1.0)),
            ((-1.0, 1.0), (1.0, 1.0)),
        ] {
            let (p, q) = (at(a.0, a.1, y), at(b.0, b.1, y));
            let inset = (q - p).normalize() * 0.03;
            timber(&mut wood, p - inset, q + inset, 0.045, 0.045, seed);
            seed += 1;
        }
    }
    // The hopper, resting on the top rails.
    vessel(
        &mut planks,
        Vec3::new(0.0, 0.0, 0.0),
        1.02,
        1.62,
        Vec2::new(0.2, 0.16),
        Vec2::new(0.5, 0.42),
        0.05,
    );
    // The two rollers under it, dressed stone, bellied a little.
    for z in [-0.13, 0.13] {
        drum(&mut stone, Vec3::new(0.0, 0.8, z), Vec3::X, 0.4, 14, |t| {
            0.125 + 0.01 * (1.0 - t * t)
        });
    }
    // The axle through the front roller and the crank on it.
    let axle = Vec3::new(0.0, 0.8, 0.13);
    timber(
        &mut iron,
        axle - Vec3::X * 0.62,
        axle + Vec3::X * 0.72,
        0.022,
        0.022,
        40,
    );
    let elbow = axle + Vec3::X * 0.72 - Vec3::Y * 0.26;
    timber(&mut iron, axle + Vec3::X * 0.70, elbow, 0.024, 0.02, 41);
    timber(&mut wood, elbow, elbow + Vec3::X * 0.2, 0.028, 0.03, 42);
    // The apron: a plank sloping out under the rollers to the front.
    let apron = |w: &mut Shape| {
        let (top, low) = (Vec3::new(0.0, 0.56, 0.18), Vec3::new(0.0, 0.16, 0.78));
        let across = Vec3::X * 0.3;
        let up = (low - top).cross(Vec3::X).normalize() * 0.02;
        let rings = vec![
            vec![
                top - across - up,
                top + across - up,
                top + across + up,
                top - across + up,
            ],
            vec![
                low - across * 1.1 - up,
                low + across * 1.1 - up,
                low + across * 1.1 + up,
                low - across * 1.1 + up,
            ],
        ];
        w.loft(&rings, true, true);
    };
    apron(&mut planks);
    vec![
        (Stuff::Wood, wood),
        (Stuff::Iron, iron),
        (Stuff::Stone, stone),
        (Stuff::Plank, planks),
    ]
}

/// The mixer (design 41): a tumbling drum of staves bound with iron hoops,
/// on a timber frame, a crank on its axle and a trough under it.
pub fn mixer() -> Vec<(Stuff, Shape)> {
    let (mut wood, mut iron) = (Shape::default(), Shape::default());
    let (mut staves, mut planks) = (Shape::default(), Shape::default());
    let axis_y = 1.02;
    // The drum, bellied like a barrel.
    drum(
        &mut staves,
        Vec3::new(0.0, axis_y, 0.0),
        Vec3::X,
        0.52,
        16,
        |t| 0.36 + 0.07 * (1.0 - t * t),
    );
    // Its hoops, a little proud of the staves.
    for x in [-0.36, 0.0, 0.36] {
        let t: f32 = x / 0.52;
        let r = 0.36 + 0.07 * (1.0 - t * t) + 0.012;
        let rings: Vec<Vec<Vec3>> = [-0.03, -0.022, 0.022, 0.03]
            .iter()
            .enumerate()
            .map(|(i, &dx)| {
                let rr = if i == 0 || i == 3 { r - 0.01 } else { r };
                ring(
                    Vec3::new(x + dx, axis_y, 0.0),
                    Vec3::X,
                    Vec3::Y,
                    rr,
                    rr,
                    16,
                    0.0,
                )
            })
            .collect();
        iron.loft(&rings, false, false);
    }
    // Two A-frames carry the axle.
    let mut seed = 60;
    for x in [-0.7, 0.7] {
        let top = Vec3::new(x, axis_y + 0.06, 0.0);
        for z in [-0.48, 0.48] {
            timber(&mut wood, Vec3::new(x, -0.05, z), top, 0.07, 0.055, seed);
            seed += 1;
        }
        timber(
            &mut wood,
            Vec3::new(x, 0.4, -0.3),
            Vec3::new(x, 0.4, 0.3),
            0.04,
            0.04,
            seed,
        );
        seed += 1;
    }
    let axle = Vec3::new(0.0, axis_y, 0.0);
    timber(
        &mut iron,
        axle - Vec3::X * 0.82,
        axle + Vec3::X * 0.9,
        0.025,
        0.025,
        70,
    );
    let elbow = axle + Vec3::X * 0.88 - Vec3::Y * 0.3;
    timber(&mut iron, axle + Vec3::X * 0.86, elbow, 0.026, 0.022, 71);
    timber(&mut wood, elbow, elbow + Vec3::X * 0.2, 0.03, 0.032, 72);
    // The trough the gravel tips into.
    vessel(
        &mut planks,
        Vec3::ZERO,
        0.02,
        0.4,
        Vec2::new(0.44, 0.24),
        Vec2::new(0.52, 0.32),
        0.045,
    );
    vec![
        (Stuff::Wood, wood),
        (Stuff::Iron, iron),
        (Stuff::Barrel, staves),
        (Stuff::Plank, planks),
    ]
}

/// A lathe about the upright through `centre`: (radius, height) stops from
/// the foot up, `sides` round, one smooth piece capped at both ends.
fn upright(shape: &mut Shape, centre: Vec3, stops: &[(f32, f32)], sides: usize) {
    let rings: Vec<Vec<Vec3>> = stops
        .iter()
        .map(|&(r, y)| ring(centre + Vec3::Y * y, Vec3::Y, Vec3::X, r, r, sides, 0.0))
        .collect();
    shape.loft(&rings, true, true);
}

/// The tanning rack (`survival-mode` design 17): two splayed uprights and
/// a back prop, a rail top and bottom, and a raw hide laced between them,
/// one irregular sheet bellied a little by the stretch, its edge ragged.
/// The lacing is in the hide's texture, never a part stuck on.
pub fn tanning_rack() -> Vec<(Stuff, Shape)> {
    let (mut wood, mut hide) = (Shape::default(), Shape::default());
    let lean = Vec3::new(0.0, 0.0, -0.18);
    let mut seed = 81;
    for x in [-0.62, 0.62] {
        let foot = Vec3::new(x * 1.08, -0.05, 0.12);
        let head = Vec3::new(x, 1.62, 0.0) + lean;
        timber(&mut wood, foot, head, 0.06, 0.05, seed);
        seed += 1;
    }
    // The back prop, from the top rail's middle to the ground behind.
    timber(
        &mut wood,
        Vec3::new(0.0, 1.45, -0.2),
        Vec3::new(0.0, -0.05, -0.85),
        0.05,
        0.055,
        seed,
    );
    seed += 1;
    for y in [0.3, 1.48] {
        let at = |x: f32| Vec3::new(x, y, 0.12 * (1.0 - y / 1.62)) + lean * (y / 1.62);
        timber(&mut wood, at(-0.72), at(0.72), 0.04, 0.04, seed);
        seed += 1;
    }
    // The hide: rows from the bottom rail to the top, each across the
    // frame's width with a ragged edge, bellied toward the front.
    let rows = 9;
    let cols = 9;
    let mut grid = Vec::new();
    for r in 0..=rows {
        let v = r as f32 / rows as f32;
        let y = 0.38 + v * (1.4 - 0.38);
        let half = 0.5 + 0.06 * (v * std::f32::consts::PI).sin() - 0.05 * roll(91, r as u32);
        for c in 0..=cols {
            let u = c as f32 / cols as f32 * 2.0 - 1.0;
            let x = u * half * (0.92 + 0.08 * roll(93, (r * 17 + c) as u32));
            let belly = 0.05 * (1.0 - u * u) * (v * std::f32::consts::PI).sin();
            let z = 0.12 * (1.0 - y / 1.62) + belly;
            grid.push(Vec3::new(x, y, z) + lean * (y / 1.62));
        }
    }
    let w = cols + 1;
    let base = hide.positions.len() as u32;
    for (i, p) in grid.iter().enumerate() {
        let (r, c) = (i / w, i % w);
        let dx = grid[r * w + (c + 1).min(cols)] - grid[r * w + c.saturating_sub(1)];
        let dy = grid[(r + 1).min(rows) * w + c] - grid[r.saturating_sub(1) * w + c];
        hide.positions.push(*p);
        hide.normals.push(dx.cross(dy).normalize_or(Vec3::Z));
        hide.uvs.push(Vec2::new(p.x, p.y));
    }
    for r in 0..rows as u32 {
        for c in 0..cols as u32 {
            let a = base + r * w as u32 + c;
            let (b, d) = (a + 1, a + w as u32);
            hide.indices.extend([a, b, d + 1, a, d + 1, d]);
        }
    }
    hide.both_sides();
    vec![(Stuff::Wood, wood), (Stuff::Hide, hide)]
}

/// The still (`survival-mode` design 15 and 18): a bellied stone pot on
/// three stones, its lid, and a cane spout from the lid down to a cup,
/// where the fresh water drips. Set beside a campfire, whose fire it
/// shares; it has none of its own.
pub fn still() -> Vec<(Stuff, Shape)> {
    let (mut stone, mut wood) = (Shape::default(), Shape::default());
    for k in 0..3 {
        let a = k as f32 / 3.0 * std::f32::consts::TAU + 0.4;
        upright(
            &mut stone,
            Vec3::new(a.cos() * 0.2, -0.02, a.sin() * 0.2),
            &[(0.07, 0.0), (0.09, 0.05), (0.07, 0.12), (0.0, 0.14)],
            7,
        );
    }
    // The pot, bellied and lipped, its lid on it.
    upright(
        &mut stone,
        Vec3::ZERO,
        &[
            (0.0, 0.12),
            (0.16, 0.13),
            (0.27, 0.24),
            (0.3, 0.38),
            (0.27, 0.52),
            (0.21, 0.6),
            (0.23, 0.63),
            (0.2, 0.66),
        ],
        16,
    );
    upright(
        &mut stone,
        Vec3::ZERO,
        &[
            (0.21, 0.64),
            (0.22, 0.67),
            (0.15, 0.73),
            (0.05, 0.76),
            (0.0, 0.77),
        ],
        16,
    );
    // The cane spout, out of the lid and down to the cup.
    timber(
        &mut wood,
        Vec3::new(0.06, 0.74, 0.0),
        Vec3::new(0.62, 0.42, 0.04),
        0.022,
        0.018,
        97,
    );
    upright(
        &mut stone,
        Vec3::new(0.66, 0.0, 0.05),
        &[
            (0.0, 0.0),
            (0.09, 0.01),
            (0.1, 0.12),
            (0.08, 0.14),
            (0.06, 0.06),
        ],
        10,
    );
    vec![(Stuff::Stone, stone), (Stuff::Wood, wood)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts_ok(parts: &[(Stuff, Shape)]) {
        for (_, s) in parts {
            assert_eq!(s.positions.len(), s.normals.len());
            assert_eq!(s.positions.len(), s.uvs.len());
            assert!(s.uvs.iter().all(|u| u.is_finite()));
            assert!(s.indices.len() % 3 == 0 && !s.indices.is_empty());
            assert!(s.indices.iter().all(|&i| (i as usize) < s.positions.len()));
            assert!(s.positions.iter().all(|p| p.is_finite()));
            assert!(s.normals.iter().all(|n| (n.length() - 1.0).abs() < 1e-3));
        }
    }

    /// Every shape is whole, within its cell (about 1.4 m from the middle),
    /// and stands on the ground's top.
    #[test]
    fn the_paving_shapes_are_whole_and_fit_their_cell() {
        for (name, parts) in [
            ("stake", stake(7)),
            ("grinder", grinder()),
            ("mixer", mixer()),
            ("tanning rack", tanning_rack()),
            ("still", still()),
        ] {
            parts_ok(&parts);
            let all: Vec<Vec3> = parts
                .iter()
                .flat_map(|(_, s)| s.positions.clone())
                .collect();
            let reach = all
                .iter()
                .map(|p| Vec2::new(p.x, p.z).length())
                .fold(0.0, f32::max);
            let top = all.iter().map(|p| p.y).fold(f32::MIN, f32::max);
            assert!(reach < 1.4, "{name} reaches {reach} m");
            assert!(top > 0.7 && top < 1.8, "{name} stands {top} m");
        }
        // The stake's foot is sunk; its head 0.9 m up.
        let peg = &stake(3)[0].1;
        let low = peg.positions.iter().map(|p| p.y).fold(f32::MAX, f32::min);
        assert!(low < -0.2, "sunk to {low}");
    }
}
