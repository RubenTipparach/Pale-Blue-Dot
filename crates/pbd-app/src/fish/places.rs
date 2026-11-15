//! The sea's places drawn (`ocean-environments` design 5, 11 and B10): the
//! living reef, kelp, black smokers and their beds, and floes on polar
//! water, as pieces standing on the cells, on a generator-version-7 world
//! only.
//!
//! Pieces are derived like trees and never saved. Where they stand is
//! `pbd_core::ocean::places::environment`, the one function the generator,
//! the spawner and the map read. Round the eye the world is cut into tiles,
//! keyed on a cube projection of the sphere so a tile is the same tile
//! whoever comes near it: a tile's pieces are worked out off the main thread
//! (a jittered lattice, every point asking the environment and a hash what
//! stands there), then built on the main thread into one vertex-coloured
//! mesh, each piece set on the cell top the walker stands on
//! (`vehicles::ground_under`). The reef is the mockup's (`reefGeos`): grey
//! pieces tinted per instance from their kind's palette, packed every
//! 0.7 m, clustered by kind and grown on one another.

use super::bodies::{Kit, Shape, at, atr, bx, cyl, ell, hh, icosahedron, lin, shape, translate};
use crate::planet::PlanetContact;
use crate::walking::WalkingCamera;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use pbd_core::fauna::{Environment, hash2};
use pbd_core::ocean::places::environment;
use pbd_core::planet_gen::TerrainConfig;
use std::collections::{HashMap, HashSet};
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::OnceLock;

/// A tile's side, m (on the cube projection, so up to about 1.7 times this
/// near a cube corner).
const TILE_M: f32 = 12.0;
/// Tiles whose middle is within this of the eye are drawn, m.
const RADIUS_M: f32 = 46.0;
/// The reef's lattice, m (the mockup's 0.62, opened a little for the cost).
const REEF_STEP_M: f32 = 0.7;

/// What stands in one place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Kind {
    Branch,
    Finger,
    Boulder,
    Table,
    Soft,
    Tube,
    Barrel,
    Fan,
    Anemone,
    Turf,
    Grass,
    Weed,
    Rubble,
    Shell,
    Chimney,
    Mat,
    Mussels,
}

/// One piece: where (a direction), how big, turned how, its colour, and
/// whether it floats on the sea (a floe) or stands on the bed and how high
/// over it (a coral grown on a boulder).
#[derive(Clone, Copy, Debug)]
struct Piece {
    dir: Vec3,
    kind: Option<Kind>,
    scale: f32,
    tall: f32,
    yaw: f32,
    colour: [f32; 3],
    over: f32,
    /// Kelp: its height to the surface, m. Floes: its radius, m.
    reach: f32,
    floe: bool,
}

/// A tile on the cube projection: the face (0..6) and the cell on it.
type Key = (u8, i32, i32);

fn face_of(d: Vec3) -> (u8, f32, f32) {
    let a = d.abs();
    if a.x >= a.y && a.x >= a.z {
        (if d.x > 0.0 { 0 } else { 1 }, d.y / a.x, d.z / a.x)
    } else if a.y >= a.z {
        (if d.y > 0.0 { 2 } else { 3 }, d.x / a.y, d.z / a.y)
    } else {
        (if d.z > 0.0 { 4 } else { 5 }, d.x / a.z, d.y / a.z)
    }
}

fn from_face(face: u8, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, u, v),
        1 => Vec3::new(-1.0, u, v),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, v),
        4 => Vec3::new(u, v, 1.0),
        _ => Vec3::new(u, v, -1.0),
    }
    .normalize()
}

fn key_of(d: Vec3, radius: f32) -> Key {
    let (face, u, v) = face_of(d);
    let unit = TILE_M / radius;
    (face, (u / unit).floor() as i32, (v / unit).floor() as i32)
}

fn tile_centre(key: Key, radius: f32) -> Vec3 {
    let unit = TILE_M / radius;
    from_face(
        key.0,
        (key.1 as f32 + 0.5) * unit,
        (key.2 as f32 + 0.5) * unit,
    )
}

fn hash_unit(seed: u64, k: u64) -> f32 {
    (hash2(seed, k) >> 40) as f32 / (1u64 << 24) as f32
}

/// A smooth value noise on a lattice of `cell_m`, 0..1, for the reef's
/// zones (which kind grows where).
fn zone_noise(seed: u64, d: Vec3, radius: f32, cell_m: f32) -> f32 {
    let p = d * radius / cell_m;
    let base = p.floor();
    let f = p - base;
    let s = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let corner = |dx: f32, dy: f32, dz: f32| {
        let c = base + Vec3::new(dx, dy, dz);
        let k = (c.x as i64 as u64).wrapping_mul(0x9E37_79B9)
            ^ (c.y as i64 as u64)
                .wrapping_mul(0x85EB_CA6B)
                .rotate_left(21)
            ^ (c.z as i64 as u64)
                .wrapping_mul(0xC2B2_AE35)
                .rotate_left(42);
        hash_unit(seed, k)
    };
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let x00 = lerp(corner(0., 0., 0.), corner(1., 0., 0.), s.x);
    let x10 = lerp(corner(0., 1., 0.), corner(1., 1., 0.), s.x);
    let x01 = lerp(corner(0., 0., 1.), corner(1., 0., 1.), s.x);
    let x11 = lerp(corner(0., 1., 1.), corner(1., 1., 1.), s.x);
    lerp(lerp(x00, x10, s.y), lerp(x01, x11, s.y), s.z)
}

/// The palettes, the mockup's `REEF_KINDS`.
fn palette(kind: Kind) -> &'static [u32] {
    match kind {
        Kind::Branch => &[
            0xeab798, 0xcfaa81, 0xab8465, 0xbfcad7, 0xdd576d, 0xc34988, 0x9e3ba2, 0x84e3b5,
            0xfffefe,
        ],
        Kind::Finger => &[0xdd576d, 0xeab798, 0xc34988, 0xcfaa81, 0x9bb675, 0xe69d69],
        Kind::Boulder => &[
            0x9bb675, 0x92a3b2, 0x559642, 0xa09f91, 0xeab798, 0x5d778d, 0x53998b, 0xc34988,
        ],
        Kind::Table => &[0x53998b, 0x204f63, 0x297537, 0x3e586e, 0x53998b],
        Kind::Soft => &[0xfaaf5c, 0xfb902f, 0xfaaf5c, 0xdd576d, 0xc1f1d8],
        Kind::Tube => &[0x7d2dbd, 0xc56d25, 0xfb902f, 0x9e3ba2, 0xd12d41],
        Kind::Barrel => &[0xa8633e, 0x733126, 0x725244, 0xc56d25],
        Kind::Fan => &[0xd12d41, 0xfaaf5c, 0x9e3ba2, 0xfb902f, 0xdd576d],
        Kind::Anemone => &[0xfffefe, 0xfaaf5c, 0xc1f1d8, 0xdd576d],
        Kind::Turf => &[0x559642, 0x9bb675, 0x297537, 0x84e3b5, 0x05582d, 0xc56d25],
        Kind::Grass => &[0x559642, 0x297537, 0x9bb675],
        Kind::Weed => &[0x8d4526, 0x559642, 0x297537, 0xc56d25],
        Kind::Rubble => &[
            0xa09f91, 0xcfaa81, 0xeab798, 0x92a3b2, 0xdd576d, 0x5d778d, 0xfffefe,
        ],
        Kind::Shell => &[0xfffefe, 0xeab798, 0xdd576d, 0xfaaf5c, 0xcfaa81],
        Kind::Chimney => &[0x2c2c25, 0x434137, 0x161612],
        Kind::Mat => &[0xfffefe, 0xd2f8ff, 0xbfcad7],
        Kind::Mussels => &[0x1f3b52, 0x3e586e, 0x2b1f35],
    }
}

// ---- the pieces' shapes, the mockup's `reefGeos` --------------------------------

/// An open cylinder: the side only, no caps.
fn ocyl(rt: f32, rb: f32, h: f32, n: usize) -> Vec<Vec3> {
    let mut out = Vec::new();
    for x in 0..n {
        let (t0, t1) = (x as f32 / n as f32 * TAU, (x + 1) as f32 / n as f32 * TAU);
        let p = |r: f32, y: f32, t: f32| Vec3::new(r * t.sin(), y, r * t.cos());
        let (a, b) = (p(rt, h / 2.0, t0), p(rb, -h / 2.0, t0));
        let (c, d) = (p(rb, -h / 2.0, t1), p(rt, h / 2.0, t1));
        out.extend([a, b, d, b, c, d]);
    }
    out
}

/// A flat disc facing up.
fn disc(r: f32, n: usize) -> Vec<Vec3> {
    let mut out = Vec::new();
    for i in 0..n {
        let (a, b) = (i as f32 / n as f32 * TAU, (i + 1) as f32 / n as f32 * TAU);
        out.extend([
            Vec3::ZERO,
            Vec3::new(r * a.cos(), 0.0, r * a.sin()),
            Vec3::new(r * b.cos(), 0.0, r * b.sin()),
        ]);
    }
    out
}

fn ring(r0: f32, r1: f32, n: usize) -> Vec<Vec3> {
    let mut out = Vec::new();
    for i in 0..n {
        let (a, b) = (i as f32 / n as f32 * TAU, (i + 1) as f32 / n as f32 * TAU);
        let p = |r: f32, t: f32| Vec3::new(r * t.cos(), 0.0, r * t.sin());
        out.extend([p(r0, a), p(r1, a), p(r1, b), p(r0, a), p(r1, b), p(r0, b)]);
    }
    out
}

fn octahedron(r: f32) -> Vec<Vec3> {
    let v = [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z].map(|p| p * r);
    let f = [
        [0, 2, 4],
        [0, 4, 3],
        [0, 3, 5],
        [0, 5, 2],
        [1, 2, 5],
        [1, 5, 3],
        [1, 3, 4],
        [1, 4, 2],
    ];
    f.iter().flat_map(|t| t.map(|i| v[i])).collect()
}

/// An icosahedron subdivided once and pushed back onto its sphere.
fn icosphere1(r: f32) -> Vec<Vec3> {
    let base = icosahedron(1.0);
    let mut out = Vec::new();
    for t in base.chunks_exact(3) {
        let (a, b, c) = (t[0], t[1], t[2]);
        let (ab, bc, ca) = (
            (a + b).normalize(),
            (b + c).normalize(),
            (c + a).normalize(),
        );
        for tri in [[a, ab, ca], [ab, b, bc], [ca, bc, c], [ab, bc, ca]] {
            out.extend(tri.map(|p| p * r));
        }
    }
    out
}

fn scaled(tris: Vec<Vec3>, s: Vec3) -> Vec<Vec3> {
    tris.into_iter().map(|p| p * s).collect()
}

/// A colour that is white above a height and `below` under it (a pale tip).
fn up(k: f32, below: u32) -> impl Fn(Vec3, usize) -> u32 {
    move |c, _| if c.y > k { 0xffffff } else { below }
}

fn library() -> &'static HashMap<Kind, Shape> {
    static LIB: OnceLock<HashMap<Kind, Shape>> = OnceLock::new();
    LIB.get_or_init(|| {
        let mut g = HashMap::new();
        // Branching: a clump of fingers on a low dome.
        let mut k = Kit::default();
        k.add(ell(0.3, 0.14, 0.3, 7, 3), at([0.0, 0.04, 0.0]), 0xc8c8c8);
        for i in 0..20 {
            let a = i as f32 * 2.4;
            let rr = 0.04 + 0.27 * ((i as f32 + 0.5) / 20.0).sqrt();
            let (x, z) = (a.cos() * rr, a.sin() * rr);
            let h = 0.2 + 0.22 * hh(i, 1);
            k.add_with(
                translate(ocyl(0.02, 0.034, h, 4), [0.0, h / 2.0, 0.0]),
                atr([x, 0.08, z], [z * 1.5, 0.0, -x * 1.5]),
                up(0.1 + h * 0.75, 0xe4e4e4),
            );
            if i % 3 == 0 {
                k.add(
                    translate(ocyl(0.016, 0.024, h * 0.6, 4), [0.0, h * 0.3, 0.0]),
                    atr(
                        [x * 1.1, 0.08 + h * 0.5, z * 1.1],
                        [z * 3.0 + 0.4, 0.0, -x * 3.0],
                    ),
                    0xe8e8e8,
                );
            }
        }
        g.insert(Kind::Branch, k.build());
        // Finger, knobbly like a cauliflower.
        let mut k = Kit::default();
        for i in 0..12 {
            let a = i as f32 * 2.2;
            let rr = if i == 0 { 0.0 } else { 0.1 + 0.14 * hh(i, 3) };
            let y = 0.12 + (1.0 - rr / 0.25) * 0.12;
            k.add_with(
                octahedron(0.09 + 0.04 * hh(i, 4)),
                at([a.cos() * rr, y, a.sin() * rr]),
                |_, n| if n % 3 != 0 { 0xffffff } else { 0xd8d8d8 },
            );
        }
        k.add(ell(0.24, 0.12, 0.24, 6, 3), at([0.0, 0.06, 0.0]), 0xc8c8c8);
        g.insert(Kind::Finger, k.build());
        // Boulder and brain: faces in two tones, the grooves.
        let mut k = Kit::default();
        k.add_with(
            scaled(icosphere1(0.5), Vec3::new(1.0, 0.58, 1.0)),
            at([0.0, 0.16, 0.0]),
            |_, n| {
                if hh(n as i64, 11) < 0.45 {
                    0xd0d0d0
                } else {
                    0xffffff
                }
            },
        );
        g.insert(Kind::Boulder, k.build());
        // Table: a plate with a hard rim on a short stalk.
        let mut k = Kit::default();
        k.add(cyl(0.07, 0.11, 0.48, 5), at([0.0, 0.24, 0.0]), 0xb4b4b4);
        k.add_with(cyl(1.0, 0.9, 0.08, 10), at([0.0, 0.5, 0.0]), |c, _| {
            if c.y > 0.5 { 0xffffff } else { 0xa8a8a8 }
        });
        k.add(ocyl(1.03, 1.0, 0.07, 10), at([0.0, 0.535, 0.0]), 0xdcdcdc);
        k.add(ring(0.86, 1.03, 10), at([0.0, 0.57, 0.0]), 0xdedede);
        g.insert(Kind::Table, k.build());
        // Soft coral tufts.
        let mut k = Kit::default();
        for i in 0..16 {
            let a = i as f32 * 2.4;
            let t = 0.25 + 0.6 * hh(i, 5);
            let h = 0.2 + 0.25 * hh(i, 6);
            k.add_with(
                translate(ocyl(0.0, 0.03, h, 4), [0.0, h / 2.0, 0.0]),
                atr([0.0; 3], [a.sin() * t, 0.0, -a.cos() * t]),
                up(h * 0.6, 0xd0d0d0),
            );
        }
        g.insert(Kind::Soft, k.build());
        // Tube sponges, dark mouths.
        let mut k = Kit::default();
        for i in 0..5 {
            let a = i as f32 * 1.9;
            let rr = if i == 0 { 0.0 } else { 0.12 };
            let r = 0.06 + 0.04 * hh(i, 7);
            let h = 0.3 + 0.4 * hh(i, 8);
            k.add(
                cyl(r, r * 0.85, h, 6),
                at([a.cos() * rr, h / 2.0, a.sin() * rr]),
                0xffffff,
            );
            k.add(
                disc(r * 0.75, 6),
                at([a.cos() * rr, h + 0.012, a.sin() * rr]),
                0x202020,
            );
        }
        g.insert(Kind::Tube, k.build());
        // A barrel sponge.
        let mut k = Kit::default();
        k.add_with(cyl(0.32, 0.22, 0.5, 8), at([0.0, 0.25, 0.0]), |_, n| {
            if n % 2 == 1 { 0xffffff } else { 0xdcdcdc }
        });
        k.add(disc(0.24, 8), at([0.0, 0.512, 0.0]), 0x1c1c1c);
        g.insert(Kind::Barrel, k.build());
        // A sea fan: a lattice fan on a stem.
        let mut k = Kit::default();
        let mut pts = vec![[0.0, 0.0]];
        for i in 0..=8 {
            let a = 0.35 + i as f32 * (PI - 0.7) / 8.0;
            let rr = 0.5 + 0.08 * hh(i, 9);
            pts.push([a.cos() * rr, a.sin() * rr]);
        }
        k.add_with(shape(&pts), at([0.0, 0.12, 0.0]), |_, n| {
            if hh(n as i64, 13) < 0.35 {
                0x8c8c8c
            } else {
                0xffffff
            }
        });
        k.add(cyl(0.02, 0.03, 0.14, 4), at([0.0, 0.07, 0.0]), 0xaaaaaa);
        g.insert(Kind::Fan, k.build());
        // An anemone: a purple foot, waving tentacles.
        g.insert(Kind::Anemone, anemone(20, 0.13, 0.22));
        // Algae turf.
        let mut k = Kit::default();
        for i in 0..8 {
            let a = i as f32 * 2.3;
            let h = 0.1 + 0.16 * hh(i, 15);
            k.add_with(
                shape(&[[-0.02, 0.0], [0.02, 0.0], [0.0, h]]),
                atr([a.cos() * 0.08, 0.0, a.sin() * 0.08], [0.0, a, 0.0]),
                move |c, _| if c.y > h * 0.5 { 0xffffff } else { 0xd0d0d0 },
            );
        }
        g.insert(Kind::Turf, k.build());
        // Sea grass.
        let mut k = Kit::default();
        for i in 0..7 {
            let a = i as f32 * 2.1;
            let h = 0.3 + 0.3 * hh(i, 16);
            k.add(
                shape(&[[-0.025, 0.0], [0.025, 0.0], [0.012, h * 0.7], [0.0, h]]),
                atr(
                    [a.cos() * 0.06, 0.0, a.sin() * 0.06],
                    [0.1 * a.sin(), a, 0.1 * a.cos()],
                ),
                0xffffff,
            );
        }
        g.insert(Kind::Grass, k.build());
        // Seaweed.
        let mut k = Kit::default();
        for j in 0..3 {
            let a = j as f32 * 2.1;
            let tall = 1.6 + j as f32 * 0.5;
            for i in 0..8 {
                let (y0, y1) = (i as f32 * tall / 8.0, (i + 1) as f32 * tall / 8.0);
                let w = 0.08 + 0.04 * (i as f32).sin();
                k.add(
                    shape(&[[-w, y0], [w, y0], [w * 0.9, y1], [-w * 0.9, y1]]),
                    atr(
                        [a.cos() * 0.1, 0.0, a.sin() * 0.1],
                        [0.0, a + i as f32 * 0.25, 0.0],
                    ),
                    if i % 2 == 1 { 0xffffff } else { 0xe0e0e0 },
                );
            }
        }
        g.insert(Kind::Weed, k.build());
        // Rubble.
        let mut k = Kit::default();
        k.add_with(
            scaled(icosahedron(0.15), Vec3::new(1.0, 0.5, 1.0)),
            at([0.0, 0.04, 0.0]),
            |_, n| if n % 2 == 1 { 0xffffff } else { 0xd4d4d4 },
        );
        k.add(
            scaled(icosahedron(0.09), Vec3::new(1.0, 0.6, 1.3)),
            at([0.17, 0.03, 0.05]),
            0xd8d8d8,
        );
        g.insert(Kind::Rubble, k.build());
        // Shells.
        let mut k = Kit::default();
        k.add_with(
            super::bodies::rotate_z(cyl(0.0, 0.05, 0.11, 5), FRAC_PI_2),
            at([0.0, 0.035, 0.0]),
            |_, n| if n % 2 == 1 { 0xffffff } else { 0xc8c8c8 },
        );
        k.add(
            ell(0.06, 0.025, 0.05, 6, 2),
            at([0.13, 0.015, 0.06]),
            0xe8e8e8,
        );
        g.insert(Kind::Shell, k.build());
        // A black smoker: narrowing prisms of basalt (the mouth's glow is
        // its own mesh, `glow`).
        let mut k = Kit::default();
        let mut y = 0.0;
        for i in 0..5 {
            let r = 1.2 - i as f32 * 0.17;
            let h = 1.3;
            k.add_with(
                cyl(r * 0.92, r, h, 6),
                at([0.0, y + h / 2.0, 0.0]),
                |_, n| {
                    if n % 3 == 0 { 0x9a9a9a } else { 0xffffff }
                },
            );
            y += h;
        }
        g.insert(Kind::Chimney, k.build());
        // A bacterial mat and a mussel bed round a vent.
        let mut k = Kit::default();
        k.add(disc(0.9, 7), at([0.0, 0.02, 0.0]), 0xffffff);
        g.insert(Kind::Mat, k.build());
        let mut k = Kit::default();
        for i in 0..9 {
            let a = i as f32 * 2.4;
            let rr = 0.08 + 0.3 * ((i as f32 + 0.5) / 9.0).sqrt();
            k.add_with(
                ell(0.12, 0.05, 0.06, 6, 3),
                atr([a.cos() * rr, 0.04, a.sin() * rr], [0.0, a, 0.3]),
                |c, _| if c.y > 0.06 { 0xffffff } else { 0xb0b0b0 },
            );
        }
        g.insert(Kind::Mussels, k.build());
        g
    })
}

fn anemone(n: usize, r: f32, l: f32) -> Shape {
    let mut k = Kit::default();
    k.add(
        cyl(r, r * 1.15, r * 0.5, 8),
        at([0.0, r * 0.25, 0.0]),
        0x9e3ba2,
    );
    k.add(disc(r * 0.5, 8), at([0.0, r * 0.5 + 0.004, 0.0]), 0x7d2dbd);
    for i in 0..n {
        let a = i as f32 * 2.39996;
        let rr = r * ((i as f32 + 0.5) / n as f32).sqrt() * 0.95;
        let h = l * (0.7 + 0.5 * hh(i as i64, 14));
        let t = 0.3 + rr / r * 0.9;
        k.add_with(
            translate(
                ocyl(0.006 * l / 0.25, 0.016 * l / 0.25, h, 4),
                [0.0, h / 2.0, 0.0],
            ),
            atr(
                [a.cos() * rr, r * 0.45, a.sin() * rr],
                [a.sin() * t, 0.0, -a.cos() * t],
            ),
            move |c, _| {
                if c.y > r * 0.5 + h * 0.75 {
                    0xc1f1d8
                } else {
                    0x9bb675
                }
            },
        );
    }
    k.build()
}

// ---- where the pieces go (off the main thread) -----------------------------

/// The pieces of one tile, from the environment and hashes alone.
fn plan(cfg: &TerrainConfig, key: Key) -> Vec<Piece> {
    let radius = cfg.radius_m;
    let unit = TILE_M / radius;
    let (u0, v0) = (key.1 as f32 * unit, key.2 as f32 * unit);
    let seed = cfg.seed ^ 0x5ea_91ace;
    let tile_k = (u64::from(key.0) << 56) ^ ((key.1 as u32 as u64) << 28) ^ key.2 as u32 as u64;
    // The place on a coarse lattice (every 3 m), so the environment, which
    // runs the terrain's noise, is asked a few dozen times a tile.
    const COARSE: usize = 5;
    let mut envs = [[None; COARSE]; COARSE];
    for (i, row) in envs.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate() {
            let d = from_face(
                key.0,
                u0 + unit * i as f32 / (COARSE - 1) as f32,
                v0 + unit * j as f32 / (COARSE - 1) as f32,
            );
            *slot = environment(cfg, d);
        }
    }
    let env_at = |fu: f32, fv: f32| {
        let i = (fu * (COARSE - 1) as f32)
            .round()
            .clamp(0.0, (COARSE - 1) as f32) as usize;
        let j = (fv * (COARSE - 1) as f32)
            .round()
            .clamp(0.0, (COARSE - 1) as f32) as usize;
        envs[i][j]
    };
    let any = |e: Environment| envs.iter().flatten().any(|x| *x == Some(e));
    let mut out = Vec::new();
    let colour = |kind: Kind, k: u64| {
        let pal = palette(kind);
        let c = lin(pal[(hash_unit(seed ^ 7, k) * pal.len() as f32) as usize % pal.len()]);
        let v = 0.85 + 0.3 * hash_unit(seed ^ 3, k);
        [c[0] * v, c[1] * v, c[2] * v]
    };
    let piece = |dir: Vec3, kind: Kind, scale: f32, tall: f32, yaw: f32, k: u64| Piece {
        dir,
        kind: Some(kind),
        scale,
        tall,
        yaw,
        colour: colour(kind, k),
        over: 0.0,
        reach: 0.0,
        floe: false,
    };
    // Deep enough for a table coral to stand clear of the surface.
    let deep = |d: Vec3| cfg.sea_level_m - pbd_core::planet_gen::surface_altitude(cfg, d) >= 3.5;
    // The reef: a jittered lattice, every point something.
    if any(Environment::Reef) {
        let n = (TILE_M / REEF_STEP_M) as i32;
        for i in 0..n {
            for j in 0..n {
                let k = tile_k.wrapping_mul(1_000_003) ^ ((i as u64) << 16) ^ j as u64;
                let fu = (i as f32 + 0.5 + (hash_unit(seed, k) - 0.5) * 0.7) / n as f32;
                let fv = (j as f32 + 0.5 + (hash_unit(seed ^ 1, k) - 0.5) * 0.7) / n as f32;
                if env_at(fu, fv) != Some(Environment::Reef) {
                    continue;
                }
                let d = from_face(key.0, u0 + fu * unit, v0 + fv * unit);
                let r = hash_unit(seed ^ 4, k);
                let s = 0.7 + 0.6 * hash_unit(seed ^ 5, k);
                let yaw = hash_unit(seed ^ 6, k) * TAU;
                let zone = zone_noise(seed, d, radius, 3.1);
                // Table corals on the deeper rock only, as the mockup's.
                if r < 0.015 && deep(d) {
                    out.push(piece(
                        d,
                        Kind::Table,
                        0.7 + 0.6 * hash_unit(seed ^ 8, k),
                        0.6 + 0.6 * hash_unit(seed ^ 9, k),
                        yaw,
                        k,
                    ));
                    continue;
                }
                let kind = match zone {
                    z if z < 0.27 => Kind::Branch,
                    z if z < 0.4 => Kind::Finger,
                    z if z < 0.5 => Kind::Boulder,
                    z if z < 0.57 => Kind::Soft,
                    z if z < 0.63 => {
                        if r < 0.5 {
                            Kind::Tube
                        } else {
                            Kind::Barrel
                        }
                    }
                    z if z < 0.69 => Kind::Fan,
                    z if z < 0.73 => Kind::Anemone,
                    z if z < 0.84 => Kind::Branch,
                    _ => Kind::Turf,
                };
                let tall = if kind == Kind::Boulder {
                    0.7 + 0.8 * hash_unit(seed ^ 10, k)
                } else {
                    1.0
                };
                let size = if kind == Kind::Boulder { s * 0.85 } else { s };
                out.push(piece(d, kind, size, tall, yaw, k));
                // Corals grow on corals.
                if kind == Kind::Boulder && hash_unit(seed ^ 11, k) < 0.35 {
                    let on = if hash_unit(seed ^ 12, k) < 0.6 {
                        Kind::Branch
                    } else {
                        Kind::Finger
                    };
                    let mut p = piece(d, on, s * 0.6, 1.0, yaw, k ^ 0xb);
                    p.over = 0.42 * size * tall;
                    out.push(p);
                }
                // The gaps filled.
                if hash_unit(seed ^ 13, k) < 0.55 {
                    let fill = if hash_unit(seed ^ 14, k) < 0.6 {
                        Kind::Turf
                    } else {
                        Kind::Rubble
                    };
                    let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
                    let beside = (d + east * (0.3 / radius)).normalize();
                    out.push(piece(
                        beside,
                        fill,
                        0.6 + 0.6 * hash_unit(seed ^ 15, k),
                        1.0,
                        yaw,
                        k ^ 0xf,
                    ));
                }
            }
        }
        // The skyline: now and then a big table coral.
        for t in 0..1u64 {
            let k = tile_k ^ (0x7ab1e << 8) ^ t;
            let (fu, fv) = (hash_unit(seed ^ 21, k), hash_unit(seed ^ 22, k));
            let d = from_face(key.0, u0 + fu * unit, v0 + fv * unit);
            if env_at(fu, fv) != Some(Environment::Reef)
                || !deep(d)
                || hash_unit(seed ^ 26, k) > 0.6
            {
                continue;
            }
            out.push(piece(
                d,
                Kind::Table,
                1.1 + 0.7 * hash_unit(seed ^ 23, k),
                0.9 + 0.9 * hash_unit(seed ^ 25, k),
                hash_unit(seed ^ 24, k) * TAU,
                k,
            ));
        }
    }
    // Kelp: stands every 2 m, to the surface, on the kelp's rock.
    if any(Environment::Kelp) {
        let n = (TILE_M / 2.0) as i32;
        for i in 0..n {
            for j in 0..n {
                let k = tile_k.wrapping_mul(7_919) ^ ((i as u64) << 16) ^ j as u64 ^ 0x4e1b;
                if hash_unit(seed ^ 30, k) > 0.7 {
                    continue;
                }
                let fu = (i as f32 + 0.5 + (hash_unit(seed ^ 31, k) - 0.5) * 0.9) / n as f32;
                let fv = (j as f32 + 0.5 + (hash_unit(seed ^ 32, k) - 0.5) * 0.9) / n as f32;
                if env_at(fu, fv) != Some(Environment::Kelp) {
                    continue;
                }
                let d = from_face(key.0, u0 + fu * unit, v0 + fv * unit);
                let mut p = piece(d, Kind::Weed, 1.0, 1.0, hash_unit(seed ^ 33, k) * TAU, k);
                p.kind = None;
                p.reach = -1.0; // to the surface, set when the bed is known
                out.push(p);
                if hash_unit(seed ^ 34, k) < 0.5 {
                    out.push(piece(
                        d,
                        Kind::Boulder,
                        0.6 + 0.5 * hash_unit(seed ^ 35, k),
                        0.8,
                        0.0,
                        k ^ 3,
                    ));
                }
            }
        }
    }
    // Vents: a black smoker now and then, beds and mats round it.
    if any(Environment::Vent) {
        for t in 0..1u64 {
            let k = tile_k ^ (0xbe47 << 12) ^ t;
            if hash_unit(seed ^ 40, k) > 0.22 {
                continue;
            }
            let (fu, fv) = (hash_unit(seed ^ 41, k), hash_unit(seed ^ 42, k));
            if env_at(fu, fv) != Some(Environment::Vent) {
                continue;
            }
            let d = from_face(key.0, u0 + fu * unit, v0 + fv * unit);
            let hgt = 0.75 + 0.55 * hash_unit(seed ^ 43, k);
            out.push(piece(
                d,
                Kind::Chimney,
                1.0,
                hgt,
                hash_unit(seed ^ 44, k) * TAU,
                k,
            ));
            let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
            let north = d.cross(east);
            for m in 0..10u64 {
                let a = hash_unit(seed ^ 45, k ^ m) * TAU;
                let rr = 1.8 + 2.8 * hash_unit(seed ^ 46, k ^ m);
                let at = (d + (east * a.cos() + north * a.sin()) * (rr / radius)).normalize();
                let kind = if m % 2 == 0 { Kind::Mat } else { Kind::Mussels };
                out.push(piece(
                    at,
                    kind,
                    0.8 + 0.6 * hash_unit(seed ^ 47, k ^ m),
                    1.0,
                    a,
                    k ^ m,
                ));
            }
        }
    }
    // Floes on polar water: hex slabs of one to four cells.
    if any(Environment::UnderIce) {
        let n = (TILE_M / 2.833) as i32;
        for i in 0..n {
            for j in 0..n {
                let k = tile_k.wrapping_mul(104_729) ^ ((i as u64) << 16) ^ j as u64 ^ 0x1ce;
                let fu = (i as f32 + 0.5) / n as f32;
                let fv = (j as f32 + 0.5) / n as f32;
                if env_at(fu, fv) != Some(Environment::UnderIce)
                    || zone_noise(
                        seed ^ 0x1ce,
                        from_face(key.0, u0 + fu * unit, v0 + fv * unit),
                        radius,
                        9.0,
                    ) < 0.42
                {
                    continue;
                }
                let d = from_face(key.0, u0 + fu * unit, v0 + fv * unit);
                out.push(Piece {
                    dir: d,
                    kind: None,
                    scale: 1.0,
                    tall: 0.25 + 0.35 * hash_unit(seed ^ 50, k),
                    yaw: hash_unit(seed ^ 51, k) * TAU,
                    colour: [1.0; 3],
                    over: 0.0,
                    reach: 1.2 + 0.25 * hash_unit(seed ^ 52, k),
                    floe: true,
                });
            }
        }
    }
    out
}

/// What is left of a colour at `depth_m` under the sea: the daylight's
/// downwelling, as the seabed is lit (`ocean-environments` B2), never quite
/// black so a shape still reads against the murk.
pub fn dimmed(downwell_per_m: [f32; 3], depth_m: f32) -> [f32; 3] {
    pbd_core::ocean::light_left(downwell_per_m, depth_m).map(|l| l.max(0.04))
}

// ---- the tile's mesh (main thread) -------------------------------------------

#[derive(Default)]
struct Built {
    positions: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    glow_positions: Vec<[f32; 3]>,
    glow_colours: Vec<[f32; 4]>,
}

fn frame_at(dir: Vec3, yaw: f32) -> (Vec3, Vec3, Vec3) {
    let up = dir;
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    let x = east * yaw.cos() + north * yaw.sin();
    let z = x.cross(up);
    (x, up, z)
}

fn lay(
    shape: &Shape,
    origin: Vec3,
    axes: (Vec3, Vec3, Vec3),
    scale: Vec3,
    tint: [f32; 3],
    positions: &mut Vec<[f32; 3]>,
    colours: &mut Vec<[f32; 4]>,
) {
    for (p, c) in shape.positions.iter().zip(&shape.colours) {
        let m = *p * scale;
        positions.push((origin + axes.0 * m.x + axes.1 * m.y + axes.2 * m.z).to_array());
        colours.push([c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], 1.0]);
    }
}

fn kelp_stand(
    origin: Vec3,
    axes: (Vec3, Vec3, Vec3),
    height: f32,
    k: f32,
    positions: &mut Vec<[f32; 3]>,
    colours: &mut Vec<[f32; 4]>,
) {
    let cols = [0x8d4526, 0xc56d25, 0x559642, 0x297537].map(lin);
    let mut kit = Kit::default();
    for stipe in 0..3 {
        let a = stipe as f32 * 2.1 + k;
        let top = height * (0.85 + 0.15 * (stipe as f32 * 1.7 + k).sin().abs());
        let segs = (top / 0.7).ceil().max(2.0) as usize;
        for i in 0..segs {
            let (y0, y1) = (
                i as f32 * top / segs as f32,
                (i + 1) as f32 * top / segs as f32,
            );
            let w = 0.12 + 0.05 * (i as f32 * 0.9).sin();
            kit.add(
                shape(&[[-w, y0], [w, y0], [w * 0.8, y1], [-w * 0.8, y1]]),
                atr(
                    [a.cos() * 0.12, 0.0, a.sin() * 0.12],
                    [0.0, a + i as f32 * 0.6, 0.0],
                ),
                if i % 2 == 1 { 0xffffff } else { 0xe0e0e0 },
            );
        }
        // The canopy: blades lying out at the top.
        for b in 0..4 {
            let t = b as f32 * 1.57 + a;
            kit.add(
                shape(&[
                    [0.0, -0.08],
                    [1.4, -0.05],
                    [1.6, 0.0],
                    [1.4, 0.05],
                    [0.0, 0.08],
                ]),
                atr(
                    [a.cos() * 0.12, top - 0.1, a.sin() * 0.12],
                    [FRAC_PI_2, t, 0.0],
                ),
                0xf0f0f0,
            );
        }
    }
    let shape = kit.build();
    for (n, (p, c)) in shape.positions.iter().zip(&shape.colours).enumerate() {
        positions.push((origin + axes.0 * p.x + axes.1 * p.y + axes.2 * p.z).to_array());
        let tint = cols[(n / 3 / 6) % cols.len()];
        colours.push([c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], 1.0]);
    }
}

fn floe(
    origin: Vec3,
    axes: (Vec3, Vec3, Vec3),
    r: f32,
    above: f32,
    positions: &mut Vec<[f32; 3]>,
    colours: &mut Vec<[f32; 4]>,
) {
    let mut k = Kit::default();
    let h = above + 0.6;
    k.add_with(
        cyl(r, r * 0.92, h, 6),
        at([0.0, above - h / 2.0, 0.0]),
        |c, n| {
            if c.y > above - 0.01 {
                if n % 4 == 0 { 0xd2f8ff } else { 0xfffefe }
            } else if n % 3 == 0 {
                0x92a3b2
            } else {
                0xbfcad7
            }
        },
    );
    let s = k.build();
    lay(&s, origin, axes, Vec3::ONE, [1.0; 3], positions, colours);
}

fn build_tile(
    pieces: &[Piece],
    anchor: Vec3,
    bed: impl Fn(Vec3) -> f32,
    sea_radius: f32,
    downwell: [f32; 3],
) -> Built {
    let lib = library();
    let mut out = Built::default();
    for (i, p) in pieces.iter().enumerate() {
        let axes = frame_at(p.dir, p.yaw);
        if p.floe {
            let origin = p.dir * sea_radius - anchor;
            floe(
                origin,
                axes,
                p.reach,
                p.tall,
                &mut out.positions,
                &mut out.colours,
            );
            continue;
        }
        let ground = bed(p.dir);
        let origin = p.dir * (ground + p.over) - anchor;
        match p.kind {
            None => {
                // Kelp, to the surface.
                let height = (sea_radius - ground - 0.3).max(1.0);
                let from = out.colours.len();
                kelp_stand(
                    origin,
                    axes,
                    height,
                    i as f32 * 0.37,
                    &mut out.positions,
                    &mut out.colours,
                );
                let light = dimmed(downwell, height * 0.5);
                for c in &mut out.colours[from..] {
                    for (k, l) in light.iter().enumerate() {
                        c[k] *= l;
                    }
                }
            }
            Some(kind) => {
                let Some(shape) = lib.get(&kind) else {
                    continue;
                };
                let scale = Vec3::new(p.scale, p.scale * p.tall, p.scale);
                let light = dimmed(downwell, sea_radius - ground);
                let tint = [
                    p.colour[0] * light[0],
                    p.colour[1] * light[1],
                    p.colour[2] * light[2],
                ];
                lay(
                    shape,
                    origin,
                    axes,
                    scale,
                    tint,
                    &mut out.positions,
                    &mut out.colours,
                );
                if kind == Kind::Chimney {
                    // The mouth's glow, drawn unlit.
                    let top = 6.5 * p.tall;
                    let mut k = Kit::default();
                    k.add(bx(0.6, 0.25, 0.6), at([0.0, top + 0.12, 0.0]), 0xff7a2a);
                    k.add(cyl(0.2, 0.35, 0.3, 6), at([0.0, top - 0.1, 0.0]), 0xfe6420);
                    let glow = k.build();
                    lay(
                        &glow,
                        origin,
                        axes,
                        Vec3::ONE,
                        [1.0; 3],
                        &mut out.glow_positions,
                        &mut out.glow_colours,
                    );
                }
            }
        }
    }
    out
}

// ---- the plugin --------------------------------------------------------------

#[derive(Resource, Default)]
struct Tiles {
    drawn: HashMap<Key, (Entity, Option<Entity>, Vec3)>,
    planning: HashMap<Key, Task<Vec<Piece>>>,
    ready: Vec<(Key, Vec<Piece>)>,
    /// The terrain epoch the tiles were made for: a world switching its
    /// generator drops them.
    version: Option<bool>,
}

/// The unlit material a vent's glow is drawn with.
#[derive(Resource)]
pub struct GlowMaterial(pub Handle<StandardMaterial>);

pub struct SeaPlacesPlugin;

impl Plugin for SeaPlacesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tiles>()
            .add_systems(Startup, warm_glow)
            .add_systems(Update, tiles);
    }
}

fn glow_desc() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        cull_mode: None,
        ..default()
    }
}

/// The glow's material, with a tiny mesh held hidden so the loading screen
/// warms its pipeline.
fn warm_glow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let glow = materials.add(glow_desc());
    let mut warm = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    warm.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0], [0.01, 0.0, 0.0], [0.0, 0.01, 0.0]],
    );
    warm.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; 3]);
    warm.compute_flat_normals();
    commands.spawn((
        Name::new("Vent glow (warm)"),
        Mesh3d(meshes.add(warm)),
        MeshMaterial3d(glow.clone()),
        Transform::default(),
        Visibility::Hidden,
    ));
    commands.insert_resource(GlowMaterial(glow));
}

fn mesh_of(positions: Vec<[f32; 3]>, colours: Vec<[f32; 4]>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh.compute_flat_normals();
    mesh
}

#[allow(clippy::too_many_arguments)]
fn tiles(
    mut commands: Commands,
    mut state: ResMut<Tiles>,
    cameras: Query<&GlobalTransform, With<WalkingCamera>>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    contact: Option<Res<PlanetContact>>,
    sea: Option<Res<crate::sea::Sea>>,
    life: Option<Res<super::draw::LifeMaterial>>,
    glow: Option<Res<GlowMaterial>>,
    water: Option<Res<crate::config::WaterSettings>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut placed: Query<&mut Transform>,
) {
    let cfg = *crate::planet::terrain_config();
    let has_places = cfg.ocean.is_some();
    if state.version != Some(has_places) {
        for (_, (body, light, _)) in state.drawn.drain() {
            commands.entity(body).despawn();
            if let Some(light) = light {
                commands.entity(light).despawn();
            }
        }
        state.planning.clear();
        state.ready.clear();
        state.version = Some(has_places);
    }
    let (Some(camera), Some(sea), Some(life), Some(glow)) =
        (cameras.iter().next(), sea, life, glow)
    else {
        return;
    };
    if !has_places {
        return;
    }
    let radius = cfg.radius_m;
    let eye = (camera.translation().as_dvec3() - frame.center).as_vec3();
    let eye_dir = eye.normalize_or(Vec3::Y);
    // The tiles wanted: every tile a ring of points round the eye falls in,
    // so a cube face's edge is crossed as any other.
    let mut wanted = HashSet::new();
    let east = Vec3::Y.cross(eye_dir).normalize_or(Vec3::X);
    let north = eye_dir.cross(east);
    let step = TILE_M * 0.5;
    let reach = (RADIUS_M / step).ceil() as i32;
    for a in -reach..=reach {
        for b in -reach..=reach {
            let off = Vec2::new(a as f32, b as f32) * step;
            if off.length() > RADIUS_M + TILE_M {
                continue;
            }
            let d = (eye_dir + (east * off.x + north * off.y) / radius).normalize();
            let key = key_of(d, radius);
            if tile_centre(key, radius).distance(eye_dir) * radius <= RADIUS_M + TILE_M * 0.75 {
                wanted.insert(key);
            }
        }
    }
    // Drop what has gone out of reach.
    let gone: Vec<Key> = state
        .drawn
        .keys()
        .filter(|k| !wanted.contains(k))
        .copied()
        .collect();
    for key in gone {
        if let Some((body, light, _)) = state.drawn.remove(&key) {
            commands.entity(body).despawn();
            if let Some(light) = light {
                commands.entity(light).despawn();
            }
        }
    }
    state.planning.retain(|k, _| wanted.contains(k));
    // Plan what is new, off the main thread.
    let pool = AsyncComputeTaskPool::get();
    for key in &wanted {
        if state.drawn.contains_key(key) || state.planning.contains_key(key) {
            continue;
        }
        if state.ready.iter().any(|(k, _)| k == key) {
            continue;
        }
        let key = *key;
        state
            .planning
            .insert(key, pool.spawn(async move { plan(&cfg, key) }));
    }
    let finished: Vec<(Key, Vec<Piece>)> = {
        let mut out = Vec::new();
        let keys: Vec<Key> = state.planning.keys().copied().collect();
        for key in keys {
            if let Some(task) = state.planning.get_mut(&key)
                && let Some(pieces) = block_on(future::poll_once(task))
            {
                out.push((key, pieces));
            }
        }
        out
    };
    for (key, pieces) in finished {
        state.planning.remove(&key);
        state.ready.push((key, pieces));
    }
    // Build one tile a frame, the nearest first.
    state.ready.sort_by(|a, b| {
        let da = tile_centre(a.0, radius).distance(eye_dir);
        let db = tile_centre(b.0, radius).distance(eye_dir);
        db.total_cmp(&da)
    });
    if let Some((key, pieces)) = state.ready.pop() {
        if wanted.contains(&key) {
            let anchor = tile_centre(key, radius) * sea.radius;
            let contact = contact.as_deref();
            let bed = |d: Vec3| {
                crate::vehicles::ground_under(contact, (d * sea.radius).as_dvec3()) as f32
            };
            let downwell = water
                .as_deref()
                .map_or(pbd_core::ocean::OCEAN_DOWNWELL_PER_M, |w| w.downwell_per_m);
            let built = build_tile(&pieces, anchor, bed, sea.radius, downwell);
            let at = (frame.center + anchor.as_dvec3()).as_vec3();
            let body = (!built.positions.is_empty()).then(|| {
                commands
                    .spawn((
                        Name::new("Sea places tile"),
                        crate::field_light::LitByField,
                        Mesh3d(meshes.add(mesh_of(built.positions, built.colours))),
                        MeshMaterial3d(life.0.clone()),
                        Transform::from_translation(at),
                        Visibility::default(),
                    ))
                    .id()
            });
            let light = (!built.glow_positions.is_empty()).then(|| {
                commands
                    .spawn((
                        Name::new("Vent glow"),
                        Mesh3d(meshes.add(mesh_of(built.glow_positions, built.glow_colours))),
                        MeshMaterial3d(glow.0.clone()),
                        Transform::from_translation(at),
                        Visibility::default(),
                    ))
                    .id()
            });
            match body {
                Some(body) => {
                    state.drawn.insert(key, (body, light, anchor));
                }
                None => {
                    // An empty tile: remembered by a bare entity, so it is
                    // not planned again while it stays in reach.
                    let empty = commands.spawn(Name::new("Sea places tile (empty)")).id();
                    state.drawn.insert(key, (empty, light, anchor));
                }
            }
        }
    }
    // Keep the tiles where they are as the render frame's centre moves.
    for (body, light, anchor) in state.drawn.values() {
        let at = (frame.center + anchor.as_dvec3()).as_vec3();
        for entity in [Some(*body), *light].into_iter().flatten() {
            if let Ok(mut transform) = placed.get_mut(entity) {
                transform.translation = at;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tile is the same tile whoever comes near it, and its plan is a
    /// function of the seed and the tile: the same pieces each time.
    #[test]
    fn a_tile_is_the_same_whoever_comes_near() {
        let cfg = TerrainConfig::for_version(7).unwrap();
        let d = pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: 12.985f32.to_radians(),
            lon: 95.043f32.to_radians(),
        });
        let key = key_of(d, cfg.radius_m);
        let centre = tile_centre(key, cfg.radius_m);
        assert_eq!(key_of(centre, cfg.radius_m), key);
        let a = plan(&cfg, key);
        let b = plan(&cfg, key);
        assert_eq!(a.len(), b.len());
        assert!(
            a.iter()
                .zip(&b)
                .all(|(p, q)| p.dir == q.dir && p.kind == q.kind)
        );
        assert!(
            a.iter().filter(|p| p.kind.is_some()).count() > 50,
            "a reef tile is packed: {} pieces",
            a.len()
        );
        // Every shape in the library has faces.
        for kind in [
            Kind::Branch,
            Kind::Table,
            Kind::Anemone,
            Kind::Chimney,
            Kind::Mussels,
        ] {
            assert!(!library()[&kind].positions.is_empty(), "{kind:?}");
        }
    }

    /// No pieces on a world made before the sea's places.
    #[test]
    fn an_older_world_has_no_pieces() {
        let cfg = TerrainConfig::for_version(6).unwrap();
        let d = pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: 12.985f32.to_radians(),
            lon: 95.043f32.to_radians(),
        });
        assert!(plan(&cfg, key_of(d, cfg.radius_m)).is_empty());
    }
}
