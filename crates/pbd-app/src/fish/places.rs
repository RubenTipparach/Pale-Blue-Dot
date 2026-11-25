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
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::OnceLock;

/// A tile's side, m (on the cube projection, so up to about 1.7 times this
/// near a cube corner).
const TILE_M: f32 = 12.0;
/// The reef's lattice, m (the mockup's 0.62, opened a little for the cost;
/// then half as dense, 0.7 times the square root of two: owner, 2026-11-25,
/// "Lower its density by 2x").
const REEF_STEP_M: f32 = 0.99;

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
    /// The big anemone with its clownfish (design 11).
    Hero,
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
        Kind::Anemone | Kind::Hero => &[0xfffefe, 0xfaaf5c, 0xc1f1d8, 0xdd576d],
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
        k.tile("polyps");
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
            k.tile("knobs");
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
            k.tile("polyps");
        }
        g.insert(Kind::Branch, k.build());
        // Finger, knobbly like a cauliflower.
        let mut k = Kit::default();
        k.tile("polyps");
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
        k.tile("brain");
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
        k.tile("polyps");
        k.add(cyl(0.07, 0.11, 0.48, 5), at([0.0, 0.24, 0.0]), 0xb4b4b4);
        k.tile("plate");
        k.add_with(cyl(1.0, 0.9, 0.08, 10), at([0.0, 0.5, 0.0]), |c, _| {
            if c.y > 0.5 { 0xffffff } else { 0xa8a8a8 }
        });
        k.add(ocyl(1.03, 1.0, 0.07, 10), at([0.0, 0.535, 0.0]), 0xdcdcdc);
        k.add(ring(0.86, 1.03, 10), at([0.0, 0.57, 0.0]), 0xdedede);
        g.insert(Kind::Table, k.build());
        // Soft coral tufts.
        let mut k = Kit::default();
        k.tile("stripes");
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
        k.tile("pores");
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
        k.tile("pores");
        k.add_with(cyl(0.32, 0.22, 0.5, 8), at([0.0, 0.25, 0.0]), |_, n| {
            if n % 2 == 1 { 0xffffff } else { 0xdcdcdc }
        });
        k.add(disc(0.24, 8), at([0.0, 0.512, 0.0]), 0x1c1c1c);
        g.insert(Kind::Barrel, k.build());
        // A sea fan: a lattice fan on a stem.
        let mut k = Kit::default();
        k.tile("lattice");
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
        // 56 tentacles: 488 triangles, inside the owner's 500 a piece
        // (2026-11-25; it was 220 and 1800).
        g.insert(Kind::Hero, anemone(56, 0.55, 0.5));
        // Algae turf.
        let mut k = Kit::default();
        k.tile("veins");
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
        k.tile("veins");
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
        k.tile("veins");
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
        k.tile("grit");
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
        k.tile("ridges");
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
        k.tile("basalt");
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
        k.tile("mat");
        k.add(disc(0.9, 7), at([0.0, 0.02, 0.0]), 0xffffff);
        g.insert(Kind::Mat, k.build());
        let mut k = Kit::default();
        k.tile("ridges");
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
    k.tile("pores");
    k.add(
        cyl(r, r * 1.15, r * 0.5, 8),
        at([0.0, r * 0.25, 0.0]),
        0x9e3ba2,
    );
    k.add(disc(r * 0.5, 8), at([0.0, r * 0.5 + 0.004, 0.0]), 0x7d2dbd);
    k.tile("stripes");
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
                if r < 0.03 && deep(d) {
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
        // The skyline: big table corals layered at their heights.
        for t in 0..3u64 {
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
    // The big anemone, one reef tile in three, with its clownfish (drawn
    // by `ambient`).
    if any(Environment::Reef) && hash_unit(seed ^ 60, tile_k) < 0.35 {
        let (fu, fv) = (
            0.2 + 0.6 * hash_unit(seed ^ 61, tile_k),
            0.2 + 0.6 * hash_unit(seed ^ 62, tile_k),
        );
        if env_at(fu, fv) == Some(Environment::Reef) {
            let d = from_face(key.0, u0 + fu * unit, v0 + fv * unit);
            let mut p = piece(
                d,
                Kind::Hero,
                1.0,
                1.0,
                hash_unit(seed ^ 63, tile_k) * TAU,
                tile_k,
            );
            p.colour = [1.0; 3];
            out.push(p);
        }
    }
    // Kelp: M25's density (design B20), to the surface, on the kelp's rock.
    if any(Environment::Kelp) {
        let n = (TILE_M / KELP_STEP_M) as i32;
        for i in 0..n {
            for j in 0..n {
                let k = tile_k.wrapping_mul(7_919) ^ ((i as u64) << 16) ^ j as u64 ^ 0x4e1b;
                if hash_unit(seed ^ 30, k) > KELP_KEEP {
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
                    let mut rock = piece(
                        d,
                        Kind::Boulder,
                        0.6 + 0.5 * hash_unit(seed ^ 35, k),
                        0.8,
                        0.0,
                        k ^ 3,
                    );
                    // A kelp bed's rocks are rock, not brain coral (B16).
                    let pick = (hash_unit(seed ^ 36, k) * KELP_ROCK.len() as f32) as usize;
                    let l = lin(KELP_ROCK[pick.min(KELP_ROCK.len() - 1)]);
                    rock.colour = [l[0], l[1], l[2]];
                    out.push(rock);
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
    // Floes on polar water: hex slabs of one to four cells. Not on a world
    // whose floes are cells of sea ice (generator version 14,
    // `ocean-environments` B24): the cells are its floes.
    if any(Environment::UnderIce) && cfg.sea_cells.is_none() {
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

/// The water sea life is seen in: the daylight it lets down per metre and
/// its deep colour, both from the water settings (`ocean-environments` B23).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnderWater {
    pub downwell_per_m: [f32; 3],
    pub deep: [f32; 3],
}

impl UnderWater {
    pub fn of(water: Option<&crate::config::WaterSettings>) -> Self {
        let defaults;
        let water = match water {
            Some(water) => water,
            None => {
                defaults = crate::config::WaterSettings::default();
                &defaults
            }
        };
        Self {
            downwell_per_m: water.downwell_per_m,
            deep: water.deep_color,
        }
    }

    /// A creature's or a sea piece's colour `depth_m` under the surface of
    /// the water it is in, as the seabed at that depth is tinted
    /// ([`pbd_core::ocean::under_water_albedo`]): dimmed by the daylight
    /// left and greyed toward the deep, never quite black.
    pub fn albedo(&self, colour: [f32; 3], depth_m: f32) -> [f32; 3] {
        pbd_core::ocean::under_water_albedo(colour, self.downwell_per_m, self.deep, depth_m)
    }
}

// ---- the tile's mesh (main thread) -------------------------------------------

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
    out: &mut Verts,
) {
    for (p, c) in shape.positions.iter().zip(&shape.colours) {
        let m = *p * scale;
        out.positions
            .push((origin + axes.0 * m.x + axes.1 * m.y + axes.2 * m.z).to_array());
        out.colours
            .push([c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], 1.0]);
    }
    out.uvs.extend_from_slice(&shape.uvs);
}

/// A mesh's vertices being built: where, what colour, where in the atlas.
#[derive(Default)]
struct Verts {
    positions: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
}

/// The rocks a kelp bed stands on: the rock's palette, not the coral's.
const KELP_ROCK: [u32; 3] = [0x434137, 0x725244, 0xa09f91];

/// M25's kelp card (`docs/mockups/ocean.html`, "Kelp forest";
/// `tools/art/ocean_art.py` `kelp_card`), byte for byte: a frond, its stipe,
/// blades and gas floats, 16 x 64 with hard alpha (design B20).
const KELP_CARD_PNG: &[u8] = include_bytes!("../../../../assets/textures/water_life/kelp_card.png");
/// The card's width and the height of one card up a stipe, m (M25's
/// `PlaneGeometry(1.2, 4)`).
const KELP_CARD_W: f32 = 1.2;
const KELP_CARD_H: f32 = 4.0;
/// How far the canopy's card lies off the stipe, m (M25's 1.4).
const KELP_CANOPY_OFF_M: f32 = 1.4;
/// Kelp stands on a lattice of this, m, and on this share of its points
/// (M25: a stand on 58% of the 2.833 m cells, 0.083 a square metre).
const KELP_STEP_M: f32 = 3.0;
const KELP_KEEP: f32 = 0.75;

/// One kelp stand from the bed to the surface as M25 draws it (design B20,
/// owner 2026-11-16: "kelp forest in the mockup was like just a couple
/// polygons"): two crossed cards of the frond up the stipe, a card for every
/// 4 m of water stretched to reach the surface, and one card lying flat on
/// the surface beside it as the canopy. `4 * ceil(height / 4) + 2`
/// triangles; the card carries the frond's own colours, the vertex colour
/// the depth's light.
fn kelp_stand(origin: Vec3, axes: (Vec3, Vec3, Vec3), height: f32, k: f32, out: &mut Verts) {
    let (east, up, south) = axes;
    let mut quad = |corners: [Vec3; 4]| {
        // Corners: bottom left, bottom right, top right, top left; the
        // card's top row at the top.
        let uv = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
        for i in [0, 1, 2, 0, 2, 3] {
            out.positions.push((origin + corners[i]).to_array());
            out.colours.push([1.0; 4]);
            out.uvs.push(uv[i]);
        }
    };
    let n = (height / KELP_CARD_H).ceil().max(1.0);
    let step = height / n;
    let turn = k * 3.0;
    for j in 0..2 {
        let a = turn + j as f32 * FRAC_PI_2;
        let across = (east * a.cos() + south * a.sin()) * (KELP_CARD_W * 0.5);
        for i in 0..n as usize {
            let (y0, y1) = (up * (i as f32 * step), up * ((i + 1) as f32 * step));
            quad([y0 - across, y0 + across, y1 + across, y1 - across]);
        }
    }
    // The canopy: the card lying flat on the surface, off to one side,
    // turned its own way.
    let centre = east * KELP_CANOPY_OFF_M + up * (height - 0.15);
    let c = (k * 17.0).sin() * PI;
    let length = (east * c.cos() + south * c.sin()) * (KELP_CARD_H * 0.5);
    let across = (east * -c.sin() + south * c.cos()) * (KELP_CARD_W * 0.5);
    quad([
        centre - length - across,
        centre - length + across,
        centre + length + across,
        centre + length - across,
    ]);
}

fn floe(origin: Vec3, axes: (Vec3, Vec3, Vec3), r: f32, above: f32, out: &mut Verts) {
    let mut k = Kit::default();
    k.tile("ice");
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
    lay(&s, origin, axes, Vec3::ONE, [1.0; 3], out);
}

/// A part of a tile drawn afresh each frame near the eye: a cloud of reef
/// fish, the clownfish over the big anemone, a smoker's plume, a seal
/// hauled out on a floe. Positions are the tile's (its anchor taken off).
#[derive(Clone, Copy, Debug)]
enum Ambient {
    Anthias {
        at: Vec3,
        up: Vec3,
        seed: u32,
    },
    Clowns {
        at: Vec3,
        up: Vec3,
        seed: u32,
    },
    Smoke {
        at: Vec3,
        up: Vec3,
        seed: u32,
    },
    Seal {
        at: Vec3,
        up: Vec3,
        yaw: f32,
        /// Its floe's radius and how high its top stands, m.
        reach: f32,
        tall: f32,
        state: SealState,
    },
}

/// Where a floe's seal is in its slide (`ocean-environments` B18): each
/// step keyed to the moment it began, so the seal is seen going and coming,
/// never there or gone in one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
enum SealState {
    /// Lying out on its floe.
    Out,
    /// Slid off at `since` (the tile clock, s), away along `away`, and
    /// swimming off under the surface; `near_s` is when the eye was last
    /// within reach.
    Off { since: f32, away: Vec3, near_s: f32 },
    /// Coming back from `from_m` off at `since`, to climb out.
    Back { since: f32, away: Vec3, from_m: f32 },
}

/// Within this of the eye a floe's seal slides off, m.
const SEAL_SHY_M: f32 = 9.0;
/// How long the seal stays off after the eye has gone, s (the seals'
/// `UNDER_S`).
const SEAL_OFF_S: f32 = pbd_core::fauna::UNDER_S;
/// How long its slide off the floe, or its climb back, takes, s.
const SEAL_SLIDE_S: f32 = 1.2;
/// How fast it swims off, m/s, and how far, m.
const SEAL_SWIM_MPS: f32 = 1.8;
const SEAL_SWIM_M: f32 = 14.0;
/// How deep it swims under the surface, m.
const SEAL_UNDER_M: f32 = 0.45;

/// Where a floe's seal is drawn and which way it faces at `clock`, in its
/// tile's frame (`at` its place on the floe), stepping its state on: it
/// slides off when the eye comes within [`SEAL_SHY_M`], swims away under
/// the surface, and comes back and climbs out once the eye has been gone
/// [`SEAL_OFF_S`]. Never skipped.
#[allow(clippy::too_many_arguments)]
fn seal_at(
    state: &mut SealState,
    at: Vec3,
    up: Vec3,
    forward: Vec3,
    reach: f32,
    tall: f32,
    eye: Vec3,
    clock: f32,
) -> (Vec3, Vec3) {
    let ease = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    let near = at.distance(eye) < SEAL_SHY_M;
    // Off the edge and under: `k` 0 on the floe to 1 in the water at its
    // edge; then `d` metres on.
    let place = |away: Vec3, k: f32, d: f32| {
        let k = ease(k);
        at + away * (reach * 1.1 * k + d) - up * ((tall + SEAL_UNDER_M) * k)
    };
    match *state {
        SealState::Out => {
            if near {
                let off = at - eye;
                let away = (off - up * off.dot(up)).normalize_or(forward);
                *state = SealState::Off {
                    since: clock,
                    away,
                    near_s: clock,
                };
            }
            (at, forward)
        }
        SealState::Off {
            since,
            away,
            near_s,
        } => {
            let s = clock - since;
            let d = ((s - SEAL_SLIDE_S).max(0.0) * SEAL_SWIM_MPS).min(SEAL_SWIM_M);
            let here = place(away, s / SEAL_SLIDE_S, d);
            let near_s = if here.distance(eye) < SEAL_SHY_M || near {
                clock
            } else {
                near_s
            };
            *state = if clock - near_s >= SEAL_OFF_S {
                SealState::Back {
                    since: clock,
                    away,
                    from_m: d,
                }
            } else {
                SealState::Off {
                    since,
                    away,
                    near_s,
                }
            };
            (here, away)
        }
        SealState::Back {
            since,
            away,
            from_m,
        } => {
            let s = clock - since;
            let swim_s = from_m / SEAL_SWIM_MPS;
            let d = (from_m - s * SEAL_SWIM_MPS).max(0.0);
            let k = 1.0 - ((s - swim_s) / SEAL_SLIDE_S).clamp(0.0, 1.0);
            let here = place(away, k, d);
            if near {
                // Startled again on the way back: off it goes from here.
                *state = SealState::Off {
                    since: clock - SEAL_SLIDE_S * k - d / SEAL_SWIM_MPS,
                    away,
                    near_s: clock,
                };
            } else if k <= 0.0 {
                *state = SealState::Out;
            }
            (here, -away)
        }
    }
}

/// The swaying pieces of a tile, kept at rest so each frame can bend them:
/// a vertex's height over its piece's foot (what it sways by), the piece's
/// across axis and phase.
#[derive(Default)]
struct Sway {
    rest: Vec<Vec3>,
    colours: Vec<[f32; 4]>,
    uvs: Vec<[f32; 2]>,
    height: Vec<f32>,
    axis: Vec<Vec3>,
    phase: Vec<f32>,
}

#[derive(Default)]
struct Built {
    pieces: Verts,
    /// The floes, in a mesh of their own (a floe built into the pieces was
    /// drowned by the flood and the water sheet cut across its top in
    /// streaks; owner, 2026-11-15).
    floes: Verts,
    glow: Verts,
    sway: Sway,
    /// The kelp stands: M25's cards, in the cut-out kelp material (B20).
    kelp: Sway,
    ambient: Vec<Ambient>,
}

/// `PBD_PASS_OFF=kelp`: no kelp stand is laid (its rocks stay), a pricing
/// instrument (`ocean-environments` B20) beside `shower` and `splash`.
fn kelp_off() -> bool {
    static OFF: OnceLock<bool> = OnceLock::new();
    *OFF.get_or_init(|| {
        std::env::var("PBD_PASS_OFF").is_ok_and(|list| list.split(',').any(|p| p.trim() == "kelp"))
    })
}

/// Whether a kind bends in the surge.
fn sways(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Soft | Kind::Anemone | Kind::Hero | Kind::Turf | Kind::Grass | Kind::Weed
    )
}

#[allow(clippy::too_many_arguments)]
fn lay_swaying(
    shape: &Shape,
    origin: Vec3,
    axes: (Vec3, Vec3, Vec3),
    scale: Vec3,
    tint: [f32; 3],
    phase: f32,
    sway: &mut Sway,
) {
    for (p, c) in shape.positions.iter().zip(&shape.colours) {
        let m = *p * scale;
        sway.rest
            .push(origin + axes.0 * m.x + axes.1 * m.y + axes.2 * m.z);
        sway.colours
            .push([c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], 1.0]);
        sway.height.push(m.y.max(0.0));
        sway.axis.push(axes.0);
        sway.phase.push(phase);
    }
    sway.uvs.extend_from_slice(&shape.uvs);
}

/// How deep a floe's ice goes under the water, m (the slab's `above + 0.6`
/// less what stands above).
const FLOE_DRAFT_M: f32 = 0.6;

/// Whether a floe at `dir` of radius `r` floats: the ground under its
/// middle and its six corners stands at least its draft under the sea's
/// mean (the sea has no tide, `no-tides`). A floe the environment
/// function puts on a beach or a shore terrace is not laid at all, where it
/// stood in the sand tipped and stacked (`ocean-environments` B18).
fn floe_floats(
    dir: Vec3,
    axes: (Vec3, Vec3, Vec3),
    r: f32,
    bed: &impl Fn(Vec3) -> f32,
    sea_radius: f32,
) -> bool {
    let lowest = sea_radius - FLOE_DRAFT_M;
    let centre = dir * sea_radius;
    (0..7).all(|k| {
        let at = if k == 0 {
            centre
        } else {
            let a = (k - 1) as f32 * TAU / 6.0;
            centre + (axes.0 * a.cos() + axes.2 * a.sin()) * r
        };
        bed(at.normalize_or(dir)) <= lowest
    })
}

fn build_tile(
    pieces: &[Piece],
    packed: bool,
    anchor: Vec3,
    bed: impl Fn(Vec3) -> f32,
    sea_radius: f32,
    water: UnderWater,
    seed: u32,
) -> Built {
    let lib = library();
    let mut out = Built::default();
    let mut reef = Vec::new();
    for (i, p) in pieces.iter().enumerate() {
        let axes = frame_at(p.dir, p.yaw);
        if p.floe {
            if !floe_floats(p.dir, axes, p.reach, &bed, sea_radius) {
                continue;
            }
            let origin = p.dir * sea_radius - anchor;
            floe(origin, axes, p.reach, p.tall, &mut out.floes);
            // A seal hauled out on one floe in nine.
            if hh(i as i64, seed as i64) < 0.11 {
                out.ambient.push(Ambient::Seal {
                    at: origin + p.dir * p.tall,
                    up: p.dir,
                    yaw: p.yaw,
                    reach: p.reach,
                    tall: p.tall,
                    state: SealState::Out,
                });
            }
            continue;
        }
        let ground = bed(p.dir);
        let origin = p.dir * (ground + p.over) - anchor;
        match p.kind {
            None => {
                // Kelp, to the surface, swaying.
                if kelp_off() {
                    continue;
                }
                let height = (sea_radius - ground - 0.3).max(1.0);
                let mut kelp = Verts::default();
                kelp_stand(origin, axes, height, i as f32 * 0.37, &mut kelp);
                out.kelp.uvs.extend_from_slice(&kelp.uvs);
                for (q, c) in kelp.positions.into_iter().zip(kelp.colours) {
                    let q = Vec3::from_array(q);
                    out.kelp.rest.push(q);
                    // Each vertex by its own depth: the frond's foot darker
                    // than its top, as the bed beside it.
                    let [r, g, b] =
                        water.albedo([c[0], c[1], c[2]], sea_radius - (q + anchor).length());
                    out.kelp.colours.push([r, g, b, 1.0]);
                    out.kelp.height.push((q - origin).dot(p.dir).max(0.0));
                    out.kelp.axis.push(axes.0);
                    out.kelp.phase.push(i as f32 * 0.37);
                }
            }
            Some(kind) => {
                let Some(shape) = lib.get(&kind) else {
                    continue;
                };
                let scale = Vec3::new(p.scale, p.scale * p.tall, p.scale);
                let tint = water.albedo(p.colour, sea_radius - ground);
                if sways(kind) {
                    lay_swaying(
                        shape,
                        origin,
                        axes,
                        scale,
                        tint,
                        i as f32 * 0.61,
                        &mut out.sway,
                    );
                } else {
                    lay(shape, origin, axes, scale, tint, &mut out.pieces);
                }
                if kind == Kind::Hero {
                    out.ambient.push(Ambient::Clowns {
                        at: origin + p.dir * 0.8,
                        up: p.dir,
                        seed: seed ^ i as u32,
                    });
                }
                if !matches!(kind, Kind::Chimney | Kind::Mat | Kind::Mussels) {
                    reef.push((origin, p.dir));
                }
                if kind == Kind::Chimney {
                    // The mouth's glow, drawn unlit, and its plume.
                    let top = 6.5 * p.tall;
                    let mut k = Kit::default();
                    k.add(bx(0.6, 0.25, 0.6), at([0.0, top + 0.12, 0.0]), 0xff7a2a);
                    k.add(cyl(0.2, 0.35, 0.3, 6), at([0.0, top - 0.1, 0.0]), 0xfe6420);
                    let glow = k.build();
                    lay(&glow, origin, axes, Vec3::ONE, [1.0; 3], &mut out.glow);
                    out.ambient.push(Ambient::Smoke {
                        at: origin + p.dir * (top + 0.25),
                        up: p.dir,
                        seed: seed ^ i as u32,
                    });
                }
            }
        }
    }
    // A cloud of anthias over one packed reef tile in two.
    if packed && hh(seed as i64, 71) < 0.5 {
        let (at, up) = reef[reef.len() / 2];
        out.ambient.push(Ambient::Anthias {
            at: at + up * 1.6,
            up,
            seed,
        });
    }
    out
}

/// A tile's ambient life and plumes at a moment, round the eye (`eye` in
/// the tile's frame): lit fish and seals, and the unlit smoke. `seen` is the
/// tint the water gives a creature at a point of the tile's frame
/// ([`UnderWater::albedo`] by its depth, `ocean-environments` B23).
#[allow(clippy::too_many_arguments)]
fn ambient(
    list: &mut [Ambient],
    t: f32,
    clock: f32,
    eye: Vec3,
    seen: &dyn Fn(Vec3) -> [f32; 3],
    lit: &mut Verts,
    smoke: (&mut Vec<[f32; 3]>, &mut Vec<[f32; 4]>),
) {
    use super::bodies::{Look, Place, emit, model_for};
    use pbd_core::fauna::Pose;
    let (lp, lc, lu) = (&mut lit.positions, &mut lit.colours, &mut lit.uvs);
    let (sp, sc) = smoke;
    let fish = |model: &str,
                at: Vec3,
                along: Vec3,
                up: Vec3,
                len: f32,
                phase: f32,
                id: u32,
                lp: &mut Vec<[f32; 3]>,
                lc: &mut Vec<[f32; 4]>,
                lu: &mut Vec<[f32; 2]>| {
        let body = model_for(model, id);
        let forward = (along - up * along.dot(up)).normalize_or(up.any_orthonormal_vector());
        let side = forward.cross(up);
        let place = Place {
            origin: at,
            forward,
            up: side.cross(forward),
            scale: len / body.length,
        };
        emit(
            body,
            &Look {
                t,
                phase,
                pose: Pose::Moving,
            },
            &place,
            seen(at),
            lp,
            lc,
            lu,
        );
    };
    for a in list.iter_mut() {
        match *a {
            Ambient::Anthias { at, up, seed } => {
                let (e, n) = frame_tangents(up);
                for k in 0..40u32 {
                    let r = |s: i64| hh(i64::from(k), i64::from(seed) ^ s);
                    let radius = 1.5 + 3.5 * r(1);
                    let speed = (0.25 + 0.2 * r(2)) * if r(3) < 0.5 { 1.0 } else { -1.0 };
                    let a0 = r(4) * TAU + t * speed;
                    let h = (r(5) - 0.3) * 2.0 + 0.3 * (t * 0.7 + r(6) * 6.0).sin();
                    let pos = at + (e * a0.cos() * radius + n * a0.sin() * radius * 0.7) + up * h;
                    let along = (-e * a0.sin() + n * a0.cos() * 0.7) * speed.signum();
                    fish(
                        "anthias",
                        pos,
                        along,
                        up,
                        0.11 + 0.04 * r(7),
                        r(8) * TAU,
                        k,
                        lp,
                        lc,
                        lu,
                    );
                }
            }
            Ambient::Clowns { at, up, seed } => {
                let (e, n) = frame_tangents(up);
                for k in 0..2u32 {
                    let a0 = t * 0.3 + k as f32 * PI + hh(i64::from(seed), 3) * TAU;
                    let pos = at
                        + (e * a0.cos() + n * a0.sin()) * 0.45
                        + up * (0.06 * (t * 1.3 + k as f32).sin());
                    let along = -e * a0.sin() + n * a0.cos();
                    fish(
                        "darter",
                        pos,
                        along,
                        up,
                        0.13,
                        k as f32 * 2.0,
                        k,
                        lp,
                        lc,
                        lu,
                    );
                }
            }
            Ambient::Seal {
                at,
                up,
                yaw,
                reach,
                tall,
                ref mut state,
            } => {
                // On its floe.
                let (e, n) = frame_tangents(up);
                let forward = e * yaw.cos() + n * yaw.sin();
                // It slides off and swims away once the eye is within its
                // nine metres, and comes back later: drawn every frame.
                let (origin, forward) = seal_at(state, at, up, forward, reach, tall, eye, clock);
                let body = model_for("seal", 0);
                let place = Place {
                    origin,
                    forward,
                    up,
                    scale: 1.6 / body.length,
                };
                emit(
                    body,
                    &Look {
                        t,
                        phase: yaw,
                        pose: Pose::HauledOut,
                    },
                    &place,
                    seen(origin),
                    lp,
                    lc,
                    lu,
                );
            }
            Ambient::Smoke { at, up, seed } => {
                // A shimmering plume: puffs rising, widening and swirling,
                // hot orange at the mouth and smoke-black above.
                let (e, n) = frame_tangents(up);
                let puff = icosahedron(1.0);
                for k in 0..36u32 {
                    let r = |s: i64| hh(i64::from(k), i64::from(seed) ^ s);
                    let life = ((t * 0.12 + r(1)) % 1.0 + 1.0) % 1.0;
                    let rise = life * 11.0;
                    let swirl = t * (0.8 + r(2)) + r(3) * TAU;
                    let spread = 0.15 + life * 1.1;
                    let pos = at
                        + up * rise
                        + (e * swirl.cos() + n * swirl.sin()) * spread * (0.4 + 0.6 * r(4));
                    let size = 0.2 + life * 0.7;
                    let heat = (1.0 - life * 5.0).max(0.0);
                    let c = [
                        0.05 + 0.6 * heat,
                        0.045 + 0.22 * heat,
                        0.04 + 0.05 * heat,
                        1.0,
                    ];
                    for q in &puff {
                        sp.push((pos + *q * size * Vec3::new(1.0, 0.8, 1.0)).to_array());
                        sc.push(c);
                    }
                }
            }
        }
    }
}

fn frame_tangents(up: Vec3) -> (Vec3, Vec3) {
    let e = Vec3::Y.cross(up).normalize_or(Vec3::X);
    (e, up.cross(e))
}

// ---- the plugin --------------------------------------------------------------

/// The tile's fade (`detail-fade`'s screen-door mask, through the field's
/// `Faded`). A fade is a moment in time, never a place (CLAUDE.md, priority
/// 1, owner 2026-11-16; `ocean-environments` B24): a tile fades in over
/// `LAND_S` when it lands, starts fading out over `LAND_S` when its middle
/// passes `FADE_OUT_M` from the eye, and back in when it comes within
/// `FADE_IN_M` again, so a still eye sees no tile half faded once `LAND_S`
/// has passed. A tile is dropped only once it is wholly faded, so nothing
/// appears or vanishes in one frame.
const FADE_IN_M: f32 = 38.0;
const FADE_OUT_M: f32 = 42.0;
const WANT_M: f32 = 52.0;
/// The owner's budget (2026-11-25, "each reef budget: 500 triangles, number
/// of reefs 1000 or less"): a tile within `DETAIL_M` of the eye keeps up to
/// `NEAR_PER_M2` reef pieces a square metre of it, one farther its largest
/// at `FAR_PER_M2`; kelp and floes are not reef pieces and are kept. By area,
/// not a count a tile: near a cube face's edge or corner the tiles are small
/// and many. A whole reach stays inside `MAX_PIECES` (a test walks it, a
/// cube corner among the places).
const DETAIL_M: f32 = 12.0;
const NEAR_PER_M2: f32 = 0.6;
const FAR_PER_M2: f32 = 0.04;
const MAX_PIECES: usize = 1000;
const MAX_PIECE_TRIS: usize = 500;

/// A tile's level of detail: 0 near, 1 far.
fn level_at(metres: f32) -> u8 {
    u8::from(metres > DETAIL_M)
}

/// The reef pieces a tile keeps at `level`: the largest first, up to its
/// allowance; kelp and floes all.
fn trim(mut pieces: Vec<Piece>, level: u8, area_m2: f32) -> Vec<Piece> {
    let keep = allowance(level, area_m2);
    let (mut reef, other): (Vec<Piece>, Vec<Piece>) =
        pieces.drain(..).partition(|p| p.kind.is_some() && !p.floe);
    reef.sort_by(|a, b| (b.scale * b.tall.max(1e-3)).total_cmp(&(a.scale * a.tall.max(1e-3))));
    reef.truncate(keep);
    reef.extend(other);
    reef
}

/// The reef pieces a tile of `area_m2` keeps at `level`.
fn allowance(level: u8, area_m2: f32) -> usize {
    let per = if level == 0 { NEAR_PER_M2 } else { FAR_PER_M2 };
    (area_m2 * per).round() as usize
}

/// A tile's area on the sphere, m².
fn tile_area_m2(key: Key, radius: f32) -> f32 {
    let unit = TILE_M / radius;
    let (u0, v0) = (key.1 as f32 * unit, key.2 as f32 * unit);
    let c0 = from_face(key.0, u0, v0).normalize();
    let c1 = from_face(key.0, u0 + unit, v0).normalize();
    let c2 = from_face(key.0, u0, v0 + unit).normalize();
    (c1 - c0).cross(c2 - c0).length() * radius * radius
}

/// The tiles wanted round the eye and each one's level: every tile a ring
/// of points round the eye falls in, so a cube face's edge is crossed as any
/// other.
fn wanted_tiles(eye_dir: Vec3, radius: f32) -> HashMap<Key, u8> {
    let metres = |key: Key| tile_centre(key, radius).distance(eye_dir) * radius;
    let mut wanted = HashMap::new();
    let east = Vec3::Y.cross(eye_dir).normalize_or(Vec3::X);
    let north = eye_dir.cross(east);
    let step = TILE_M * 0.5;
    let reach = (WANT_M / step).ceil() as i32;
    for a in -reach..=reach {
        for b in -reach..=reach {
            let off = Vec2::new(a as f32, b as f32) * step;
            if off.length() > WANT_M + TILE_M {
                continue;
            }
            let d = (eye_dir + (east * off.x + north * off.y) / radius).normalize();
            let key = key_of(d, radius);
            let m = metres(key);
            if m <= WANT_M {
                wanted.insert(key, level_at(m));
            }
        }
    }
    wanted
}
const LAND_S: f32 = 0.8;
/// Within this of the eye a tile sways, m; the sway eases out over the
/// last `SWAY_EASE_M` (`ocean-environments` B18: it snapped to rest). Its
/// ambient life is drawn while the tile is, and fades with it.
/// The sway itself runs in `sway_vertex.wgsl`, which keeps these two.
#[cfg_attr(not(test), allow(dead_code))]
const NEAR_M: f32 = 30.0;
#[cfg_attr(not(test), allow(dead_code))]
const SWAY_EASE_M: f32 = 4.0;

/// Where a tile's fade goes from where it is, 0 whole to 1 gone: `metres`
/// its middle from the eye, `wanted` whether it is in reach, `fade` now, and
/// `dt` seconds on. It runs at one rate toward whole or gone, each a timed
/// fade over `LAND_S`; the distance only says which way, with a margin
/// (`FADE_IN_M` to `FADE_OUT_M`) in which it keeps the way it was going.
fn tile_fade(fade: f32, metres: f32, wanted: bool, dt: f32) -> f32 {
    let out = if !wanted || metres > FADE_OUT_M {
        true
    } else if metres < FADE_IN_M {
        false
    } else {
        // In the margin: on to the end it was nearer.
        fade >= 0.5
    };
    let step = dt.max(0.0) / LAND_S;
    if out {
        (fade + step).min(1.0)
    } else {
        (fade - step).max(0.0)
    }
}

struct Drawn {
    root: Entity,
    glow: Option<Entity>,
    life: (Entity, Handle<Mesh>),
    smoke: Option<(Entity, Handle<Mesh>)>,
    anchor: Vec3,
    /// Its fade, 0 whole to 1 gone ([`tile_fade`]); it lands at 1.
    fade: f32,
    ambient: Vec<Ambient>,
    /// The tile's triangles and the kelp's share of them (`SEA_PLACES`).
    tris: usize,
    kelp_tris: usize,
}

#[derive(Resource, Default)]
struct Tiles {
    /// Each tile at its level of detail: one fading out while its other
    /// level fades in is two entries.
    drawn: HashMap<(Key, u8), Drawn>,
    planning: HashMap<Key, Task<Vec<Piece>>>,
    ready: Vec<(Key, Vec<Piece>)>,
    /// Whether the world has sea places: a world switching its generator
    /// drops the tiles.
    version: Option<bool>,
    /// When the `SEA_PLACES` line was last logged, s.
    logged_s: f32,
}

/// The unlit material a vent's glow is drawn with.
#[derive(Resource)]
pub struct GlowMaterial(pub Handle<StandardMaterial>);

/// The kelp's material (design B20): M25's card, cut out at half alpha,
/// both faces, lit by the field as the reef's pieces are.
#[derive(Resource)]
pub struct KelpMaterial(pub Handle<StandardMaterial>);

fn kelp_desc(card: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(card),
        perceptual_roughness: 1.0,
        reflectance: 0.3,
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        double_sided: true,
        ..default()
    }
}

fn kelp_card() -> Image {
    let pixels = image::load_from_memory(KELP_CARD_PNG)
        .expect("assets/textures/water_life/kelp_card.png")
        .to_rgba8();
    let mut image = Image::from_dynamic(
        image::DynamicImage::ImageRgba8(pixels),
        true,
        RenderAssetUsages::default(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    image
}

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
    mut images: ResMut<Assets<Image>>,
) {
    // The kelp's material, with a tiny card held hidden under a field-lit
    // root so the loading screen warms its pipeline (B20).
    let kelp = materials.add(kelp_desc(images.add(kelp_card())));
    let mut card = Verts::default();
    kelp_stand(
        Vec3::ZERO,
        (Vec3::X, Vec3::Y, Vec3::Z),
        0.01,
        0.0,
        &mut card,
    );
    commands.spawn((
        Name::new("Kelp (warm)"),
        crate::field_light::LitByField,
        crate::field_light::LitLikeTerrain,
        Mesh3d(meshes.add(mesh_of(card, true))),
        MeshMaterial3d(kelp.clone()),
        Transform::default(),
        Visibility::Hidden,
    ));
    commands.insert_resource(KelpMaterial(kelp));
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

/// A mesh of a part of a tile; `textured` for the pieces and the life (the
/// water life's atlas), not for the unlit glow.
/// A swaying vertex's surge (`sway_vertex.wgsl`): xyz its axis times its
/// reach (`min(0.07 h, 0.7)`), w its phase with the height's share
/// (`phase + 0.6 h`). Written once when the tile is built; the GPU moves it.
pub const ATTRIBUTE_SWAY: bevy::mesh::MeshVertexAttribute = bevy::mesh::MeshVertexAttribute::new(
    "Reef_Sway",
    0x5eed_5ea_0001,
    bevy::render::render_resource::VertexFormat::Float32x4,
);

/// The sway attribute of a tile's swaying pieces, one row a vertex.
fn sway_rows(still: &Sway) -> Vec<[f32; 4]> {
    (0..still.rest.len())
        .map(|i| {
            let h = still.height[i];
            let reach = (0.07 * h).min(0.7);
            let a = still.axis[i] * reach;
            [a.x, a.y, a.z, still.phase[i] + h * 0.6]
        })
        .collect()
}

/// A swaying mesh: its rest pose, flat normals from it, and its sway.
fn swaying_mesh(still: &Sway) -> Mesh {
    let verts = Verts {
        positions: still.rest.iter().map(|p| p.to_array()).collect(),
        colours: still.colours.clone(),
        uvs: still.uvs.clone(),
    };
    let mut mesh = mesh_of(verts, true);
    mesh.insert_attribute(ATTRIBUTE_SWAY, sway_rows(still));
    mesh
}

fn mesh_of(verts: Verts, textured: bool) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, verts.colours);
    if textured {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, verts.uvs);
    }
    mesh.compute_flat_normals();
    mesh
}

/// A measurement instrument (`ocean-environments` B20): the main thread's
/// time in [`tiles`], summed until the `SEA_PLACES` line reads it.
static TILES_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static TILES_CALLS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct TilesTimer(std::time::Instant);

/// A measurement instrument (`memory-leak`, the reef): the share of
/// [`tiles`] spent laying the ambient life and plumes, summed until
/// `SEA_PLACES` reads it.
static AMBIENT_NS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct AmbientTimer(std::time::Instant);

impl Drop for AmbientTimer {
    fn drop(&mut self) {
        AMBIENT_NS.fetch_add(
            self.0.elapsed().as_nanos() as u64,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}

impl Drop for TilesTimer {
    fn drop(&mut self) {
        use std::sync::atomic::Ordering::Relaxed;
        TILES_NS.fetch_add(self.0.elapsed().as_nanos() as u64, Relaxed);
        TILES_CALLS.fetch_add(1, Relaxed);
    }
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
    kelp_material: Option<Res<KelpMaterial>>,
    water: Option<Res<crate::config::WaterSettings>>,
    time: Res<Time>,
    sun: Option<Res<crate::sky::Sun>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut roots: Query<(&mut Transform, &mut crate::field_light::Faded)>,
    mut shown: Query<&mut Visibility>,
) {
    let _timer = TilesTimer(std::time::Instant::now());
    let cfg = *crate::planet::terrain_config();
    let has_places = cfg.ocean.is_some();
    if state.version != Some(has_places) {
        for (_, drawn) in state.drawn.drain() {
            commands.entity(drawn.root).despawn();
        }
        state.planning.clear();
        state.ready.clear();
        state.version = Some(has_places);
    }
    let (Some(camera), Some(sea), Some(life), Some(glow), Some(kelp_material)) =
        (cameras.iter().next(), sea, life, glow, kelp_material)
    else {
        return;
    };
    if !has_places {
        return;
    }
    let now = time.elapsed_secs();
    let dt = time.delta_secs().min(0.1);
    let under = UnderWater::of(water.as_deref());
    let radius = cfg.radius_m;
    let eye = (camera.translation().as_dvec3() - frame.center).as_vec3();
    let eye_dir = eye.normalize_or(Vec3::Y);
    let metres = |key: Key| tile_centre(key, radius).distance(eye_dir) * radius;
    let wanted = wanted_tiles(eye_dir, radius);
    let is_wanted = |key: Key, level: u8| wanted.get(&key) == Some(&level);
    // Fade every tile by its distance and its landing; drop one once it is
    // out of reach and wholly faded.
    let seconds = sun
        .as_deref()
        .map_or(now as f64, |s| s.clock.seconds)
        .rem_euclid(3600.0) as f32;
    let mut gone = Vec::new();
    for (&(key, level), drawn) in state.drawn.iter_mut() {
        let m = metres(key);
        let fade = tile_fade(drawn.fade, m, is_wanted(key, level), dt);
        drawn.fade = fade;
        if !is_wanted(key, level) && fade >= 1.0 {
            gone.push((key, level));
            continue;
        }
        if let Ok((mut transform, mut faded)) = roots.get_mut(drawn.root) {
            transform.translation = (frame.center + drawn.anchor.as_dvec3()).as_vec3();
            if faded.0 != fade {
                faded.0 = fade;
            }
        }
        // The unlit glow and smoke cannot take the mask: they go at half.
        for entity in [drawn.glow, drawn.smoke.as_ref().map(|s| s.0)]
            .into_iter()
            .flatten()
        {
            if let Ok(mut v) = shown.get_mut(entity) {
                let want = if fade < 0.5 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if *v != want {
                    *v = want;
                }
            }
        }
    }
    // The ambient life and the plumes, while each tile is drawn: they fade
    // with it, never cut at a distance. Each tile's are its own, so they are
    // laid on every core at once (`memory-leak` 5.3: on the main thread they
    // were 34-38 ms of a reef's frame); the meshes are written after.
    {
        let _ambient_timer = AmbientTimer(std::time::Instant::now());
        let surface = sea.radius;
        let under = &under;
        let mut jobs: Vec<&mut Drawn> = state
            .drawn
            .values_mut()
            .filter(|d| !d.ambient.is_empty())
            .collect();
        let pool = bevy::tasks::ComputeTaskPool::get();
        let per = jobs.len().div_ceil(pool.thread_num().max(1)).max(1);
        let laid = pool.scope(|scope| {
            for chunk in jobs.chunks_mut(per) {
                scope.spawn(async move {
                    chunk
                        .iter_mut()
                        .map(|drawn| {
                            let (mut lit, mut smoke) = (Verts::default(), Verts::default());
                            if drawn.fade < 1.0 {
                                let anchor = drawn.anchor;
                                let seen = |at: Vec3| {
                                    under.albedo([1.0; 3], surface - (at + anchor).length())
                                };
                                ambient(
                                    &mut drawn.ambient,
                                    seconds,
                                    now,
                                    eye - anchor,
                                    &seen,
                                    &mut lit,
                                    (&mut smoke.positions, &mut smoke.colours),
                                );
                            }
                            (
                                drawn.life.1.clone(),
                                lit,
                                drawn.smoke.as_ref().map(|s| s.1.clone()),
                                smoke,
                            )
                        })
                        .collect::<Vec<_>>()
                });
            }
        });
        for (life, lit, smoke_mesh, smoke) in laid.into_iter().flatten() {
            if let Some(mesh) = meshes.get_mut(&life) {
                write_mesh(mesh, lit, true);
            }
            if let Some(handle) = smoke_mesh
                && let Some(mesh) = meshes.get_mut(&handle)
            {
                write_mesh(mesh, smoke, false);
            }
        }
    }
    for key in gone {
        if let Some(drawn) = state.drawn.remove(&key) {
            commands.entity(drawn.root).despawn();
        }
    }
    // A measurement instrument (`ocean-environments` B20): what the sea's
    // places draw and rewrite, once a second.
    if now - state.logged_s >= 1.0 {
        state.logged_s = now;
        let (tris, kelp) = state
            .drawn
            .values()
            .fold((0, 0), |(t, k), d| (t + d.tris, k + d.kelp_tris));
        let (tiles_ms, ambient_ms) = {
            use std::sync::atomic::Ordering::Relaxed;
            let calls = TILES_CALLS.swap(0, Relaxed).max(1) as f64;
            let ns = TILES_NS.swap(0, Relaxed) as f64;
            let amb = AMBIENT_NS.swap(0, Relaxed) as f64;
            (ns / 1e6 / calls, amb / 1e6 / calls)
        };
        info!(
            "SEA_PLACES tiles={} tris={tris} kelp_tris={kelp} tiles_ms={tiles_ms:.3} ambient_ms={ambient_ms:.3}",
            state.drawn.len(),
        );
    }
    state.planning.retain(|k, _| wanted.contains_key(k));
    // Plan what is new, off the main thread.
    let pool = AsyncComputeTaskPool::get();
    for (key, level) in &wanted {
        if state.drawn.contains_key(&(*key, *level))
            || state.planning.contains_key(key)
            || state.ready.iter().any(|(k, _)| k == key)
        {
            continue;
        }
        let key = *key;
        state
            .planning
            .insert(key, pool.spawn(async move { plan(&cfg, key) }));
    }
    let mut finished = Vec::new();
    let keys: Vec<Key> = state.planning.keys().copied().collect();
    for key in keys {
        if let Some(task) = state.planning.get_mut(&key)
            && let Some(pieces) = block_on(future::poll_once(task))
        {
            finished.push((key, pieces));
        }
    }
    for (key, pieces) in finished {
        state.planning.remove(&key);
        state.ready.push((key, pieces));
    }
    // Build one tile a frame, the nearest first.
    state
        .ready
        .sort_by(|a, b| metres(b.0).total_cmp(&metres(a.0)));
    let Some((key, pieces)) = state.ready.pop() else {
        return;
    };
    let Some(&level) = wanted.get(&key) else {
        return;
    };
    if state.drawn.contains_key(&(key, level)) {
        return;
    }
    // A packed tile has its anthias whatever the budget leaves of its reef.
    let packed = pieces
        .iter()
        .filter(|p| {
            p.kind
                .is_some_and(|k| !matches!(k, Kind::Chimney | Kind::Mat | Kind::Mussels))
                && !p.floe
        })
        .count()
        > 80;
    let pieces = trim(pieces, level, tile_area_m2(key, radius));
    let anchor = tile_centre(key, radius) * sea.radius;
    let contact = contact.as_deref();
    let bed = |d: Vec3| crate::vehicles::ground_under(contact, (d * sea.radius).as_dvec3()) as f32;
    let seed = (key.1 as u32).wrapping_mul(73_856_093)
        ^ (key.2 as u32).wrapping_mul(19_349_663)
        ^ u32::from(key.0);
    let built = build_tile(&pieces, packed, anchor, bed, sea.radius, under, seed);
    for a in &built.ambient {
        let (Ambient::Anthias { at, .. }
        | Ambient::Clowns { at, .. }
        | Ambient::Smoke { at, .. }
        | Ambient::Seal { at, .. }) = a;
        let (lat, lon) = pbd_core::geo::lat_lon(*at + anchor).degrees();
        debug!(
            "sea ambient {} at --at {lat:.4} {lon:.4}",
            match a {
                Ambient::Anthias { .. } => "anthias",
                Ambient::Clowns { .. } => "clownfish",
                Ambient::Smoke { .. } => "smoke",
                Ambient::Seal { .. } => "seal",
            }
        );
    }
    let tris = (built.pieces.positions.len()
        + built.floes.positions.len()
        + built.glow.positions.len()
        + built.sway.rest.len()
        + built.kelp.rest.len())
        / 3;
    let kelp_tris = built.kelp.rest.len() / 3;
    let at = (frame.center + anchor.as_dvec3()).as_vec3();
    let root = commands
        .spawn((
            Name::new("Sea places tile"),
            crate::field_light::LitByField,
            // Lit as the seabed beside it is (`ocean-environments` B23).
            crate::field_light::LitLikeTerrain,
            crate::field_light::Faded(1.0),
            Transform::from_translation(at),
            Visibility::default(),
        ))
        .id();
    let child = |commands: &mut Commands,
                 mesh: Handle<Mesh>,
                 material: Handle<StandardMaterial>,
                 name: &'static str| {
        let id = commands
            .spawn((
                Name::new(name),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::default(),
                bevy::camera::visibility::NoFrustumCulling,
            ))
            .id();
        commands.entity(root).add_child(id);
        id
    };
    if !built.pieces.positions.is_empty() {
        child(
            &mut commands,
            meshes.add(mesh_of(built.pieces, true)),
            life.0.clone(),
            "Sea pieces",
        );
    }
    if !built.floes.positions.is_empty() {
        child(
            &mut commands,
            meshes.add(mesh_of(built.floes, true)),
            life.0.clone(),
            "Ice floes",
        );
    }
    let glow_id = (!built.glow.positions.is_empty()).then(|| {
        child(
            &mut commands,
            meshes.add(mesh_of(built.glow, false)),
            glow.0.clone(),
            "Vent glow",
        )
    });
    // The swaying pieces and the kelp carry their sway; the GPU moves them
    // (`sway_vertex.wgsl`), so the meshes are never written again.
    if !built.sway.rest.is_empty() {
        child(
            &mut commands,
            meshes.add(swaying_mesh(&built.sway)),
            life.0.clone(),
            "Sea pieces (swaying)",
        );
    }
    // The kelp: M25's cards, cut out, frustum culled (a tile behind the eye
    // draws none of it).
    if !built.kelp.rest.is_empty() {
        let handle = meshes.add(swaying_mesh(&built.kelp));
        let id = commands
            .spawn((
                Name::new("Kelp"),
                Mesh3d(handle.clone()),
                MeshMaterial3d(kelp_material.0.clone()),
                Transform::default(),
                Visibility::default(),
            ))
            .id();
        commands.entity(root).add_child(id);
    }
    // The ambient meshes start with one hidden triangle, so the field gives
    // them their material at once and they never need it again.
    let blank = |textured: bool| {
        let mut m = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        write_mesh(&mut m, Verts::default(), textured);
        m
    };
    let life_handle = meshes.add(blank(true));
    let life_id = child(
        &mut commands,
        life_handle.clone(),
        life.0.clone(),
        "Sea life (ambient)",
    );
    let smoke = built
        .ambient
        .iter()
        .any(|a| matches!(a, Ambient::Smoke { .. }))
        .then(|| {
            let handle = meshes.add(blank(false));
            (
                child(&mut commands, handle.clone(), glow.0.clone(), "Vent smoke"),
                handle,
            )
        });
    state.drawn.insert(
        (key, level),
        Drawn {
            root,
            glow: glow_id,
            life: (life_id, life_handle),
            smoke,
            anchor,
            fade: 1.0,
            ambient: built.ambient,
            tris,
            kelp_tris,
        },
    );
}

fn write_mesh(mesh: &mut Mesh, verts: Verts, textured: bool) {
    let verts = if verts.positions.is_empty() {
        Verts {
            positions: vec![[0.0; 3]; 3],
            colours: vec![[0.0; 4]; 3],
            uvs: vec![[0.0; 2]; 3],
        }
    } else {
        verts
    };
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, verts.colours);
    if textured {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, verts.uvs);
    }
    mesh.remove_indices();
    mesh.compute_flat_normals();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Design B23: sea life is tinted by the terrain's own rule for the
    /// seabed. The rule lives once in the core
    /// (`pbd_core::ocean::under_water_albedo`); this holds the terrain
    /// shader's copy of it to the same expression, so the two cannot drift.
    #[test]
    fn sea_life_takes_the_seabeds_under_water_tint() {
        let shader = include_str!("../../../../assets/shaders/planet_surface.wgsl");
        let flat: String = shader.split_whitespace().collect();
        assert!(
            flat.contains(
                "color=mix(color*attenuation,params.water_deep.rgb*face_light*attenuation,\
                 (1.-dot(attenuation,vec3(1./3.)))*0.5);"
            ),
            "the seabed's under-water tint changed: change under_water_albedo with it"
        );
        assert!(flat.contains("letattenuation=exp(-params.water_absorption.rgb*water_depth);"));
        // And the app reads it from the settings the terrain is given.
        let settings = crate::config::WaterSettings::default();
        let water = UnderWater::of(Some(&settings));
        assert_eq!(water.downwell_per_m, settings.downwell_per_m);
        assert_eq!(water.deep, settings.deep_color);
        assert_eq!(water, UnderWater::of(None));
        let shallow = water.albedo([0.85, 0.63, 0.23], 1.0);
        let deeper = water.albedo([0.85, 0.63, 0.23], 8.0);
        assert!((0..3).all(|i| deeper[i] < shallow[i]));
        assert_eq!(water.albedo([0.85, 0.63, 0.23], -2.0), [0.85, 0.63, 0.23]);
    }

    /// Design B18: a floe is laid only where its ice clears the ground, so
    /// none stands on a beach or a terrace; and the floe seal
    /// is drawn on every frame of its slide off, its swim and its climb back,
    /// never moving further in a frame than it could swim.
    #[test]
    fn floes_float_and_their_seals_slide_off_and_come_back() {
        let radius = 1000.0;
        let dir = Vec3::Y;
        let axes = frame_at(dir, 0.3);
        let flat = |depth: f32| move |_: Vec3| radius - depth;
        // 3 m of water clears 0.6 m of ice; 0.4 m does not.
        assert!(floe_floats(dir, axes, 1.3, &flat(3.0), radius));
        assert!(!floe_floats(dir, axes, 1.3, &flat(0.4), radius));
        // Dry sand above the sea: none.
        assert!(!floe_floats(dir, axes, 1.3, &flat(-0.6), radius));
        // A terrace under one corner only (the bed takes a direction): none.
        let step = |p: Vec3| {
            if p.x > 0.001 {
                radius + 0.5
            } else {
                radius - 3.0
            }
        };
        assert!(!floe_floats(dir, axes, 1.3, &step, radius));
        let pieces = [Piece {
            dir,
            kind: None,
            scale: 1.0,
            tall: 0.4,
            yaw: 0.3,
            colour: [1.0; 3],
            over: 0.0,
            reach: 1.3,
            floe: true,
        }];
        let built = build_tile(
            &pieces,
            true,
            dir * radius,
            flat(-0.6),
            radius,
            UnderWater::of(None),
            1,
        );
        assert!(built.floes.positions.is_empty(), "no floe on the beach");
        let built = build_tile(
            &pieces,
            true,
            dir * radius,
            flat(4.0),
            radius,
            UnderWater::of(None),
            1,
        );
        assert!(!built.floes.positions.is_empty(), "a floe on the sea");

        // The seal: the eye walks in from 30 m to 2 m, waits, then leaves.
        let at = Vec3::new(0.0, 0.4, 0.0);
        let (up, forward) = (Vec3::Y, Vec3::X);
        let mut state = SealState::Out;
        let dt = 1.0 / 60.0;
        let mut last: Option<Vec3> = None;
        let mut slid = false;
        let mut back_out = false;
        for frame in 0..(60 * 70) {
            let clock = frame as f32 * dt;
            let metres = if clock < 14.0 {
                (30.0 - clock * 2.0).max(2.0)
            } else {
                40.0
            };
            let eye = at + Vec3::Z * metres + up * 1.6;
            let (here, facing) = seal_at(&mut state, at, up, forward, 1.3, 0.4, eye, clock);
            assert!(here.is_finite() && facing.is_finite());
            if let Some(last) = last {
                assert!(
                    here.distance(last) <= SEAL_SWIM_MPS * dt * 2.5 + 0.06,
                    "it jumped {} m at {clock} s",
                    here.distance(last)
                );
            }
            slid |= here.y < 0.0;
            if slid && state == SealState::Out {
                back_out = true;
            }
            last = Some(here);
        }
        assert!(slid, "it went into the water");
        assert!(back_out, "and climbed back out after the eye had gone");
        assert_eq!(last, Some(at), "on its floe again");
    }

    /// Design B20 (owner, 2026-11-16: "kelp forest in the mockup was like
    /// just a couple polygons"): a stand is M25's, two crossed cards a 4 m
    /// of water and one flat on the surface, `4 * ceil(h / 4) + 2`
    /// triangles; it rises to the surface; its canopy lies flat there; every
    /// UV is on the card.
    #[test]
    fn a_kelp_stand_is_the_mockup_s_few_cards() {
        let axes = (Vec3::X, Vec3::Y, Vec3::Z);
        for (height, tris) in [(6.0, 10), (12.0, 14), (18.0, 22), (40.0, 42)] {
            let mut kelp = Verts::default();
            kelp_stand(Vec3::ZERO, axes, height, 0.7, &mut kelp);
            assert_eq!(kelp.positions.len(), tris * 3, "{height} m");
            let top = kelp.positions.iter().map(|p| p[1]).fold(0.0f32, f32::max);
            assert!((top - height).abs() < 1e-3, "to the surface: {top}");
            let canopy = &kelp.positions[kelp.positions.len() - 6..];
            assert!(canopy.iter().all(|p| (p[1] - (height - 0.15)).abs() < 1e-3));
            assert!(
                kelp.uvs
                    .iter()
                    .all(|uv| (0.0..=1.0).contains(&uv[0]) && (0.0..=1.0).contains(&uv[1]))
            );
        }
    }

    /// Design B20: kelp stands at M25's density, about 0.083 a square metre.
    #[test]
    fn kelp_stands_at_the_mockup_s_density() {
        let per_m2 = (TILE_M / KELP_STEP_M).floor().powi(2) * KELP_KEEP / (TILE_M * TILE_M);
        let mockup = 0.58 / (3f32.sqrt() / 2.0 * 2.833 * 2.833);
        assert!((per_m2 - mockup).abs() < 0.01, "{per_m2} against {mockup}");
    }

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

    /// A tile never appears or vanishes in one frame (CLAUDE.md, "No
    /// pop-in"), and a fade is a moment in time, never a place (owner,
    /// 2026-11-16): it lands wholly faded and comes in over `LAND_S`; past
    /// `FADE_OUT_M` it fades out over `LAND_S`, before it is out of reach,
    /// and is dropped only once wholly faded (`tiles`); a still eye at any
    /// distance sees it whole or gone once `LAND_S` has passed; and the
    /// frame-to-frame change is small.
    #[test]
    fn a_tile_fades_in_when_it_lands_and_out_before_it_is_dropped() {
        let dt = 1.0 / 60.0;
        let run = |mut fade: f32, metres: f32, wanted: bool, seconds: f32| {
            for _ in 0..(seconds / dt).round() as usize {
                let next = tile_fade(fade, metres, wanted, dt);
                assert!((next - fade).abs() <= dt / LAND_S + 1e-5);
                fade = next;
            }
            fade
        };
        assert_eq!(
            run(1.0, 10.0, true, LAND_S + 0.05),
            0.0,
            "whole once landed"
        );
        assert_eq!(
            run(0.0, 45.0, true, LAND_S + 0.05),
            1.0,
            "gone past the edge"
        );
        assert_eq!(run(0.0, 10.0, false, LAND_S + 0.05), 1.0, "out of reach");
        const { assert!(FADE_OUT_M < WANT_M && FADE_IN_M < FADE_OUT_M) };
        // A still eye, every distance in reach: whole or gone, never half.
        for cm in (0..=(WANT_M as i32 * 100)).step_by(37) {
            let m = cm as f32 / 100.0;
            for start in [0.0, 0.3, 0.7, 1.0] {
                let f = run(start, m, true, LAND_S + 0.05);
                assert!(f == 0.0 || f == 1.0, "{f} at {m} m from {start}");
            }
        }
        // In the margin it holds the way it was going.
        assert_eq!(run(0.0, 40.0, true, 5.0), 0.0);
        assert_eq!(run(1.0, 40.0, true, 5.0), 1.0);
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

    fn inside(uv: [f32; 2], r: [f32; 4]) -> bool {
        uv[0] > r[0] && uv[0] < r[0] + r[2] && uv[1] > r[1] && uv[1] < r[1] + r[3]
    }

    /// Every piece of the reef, the kelp and the floes has a UV a vertex,
    /// each inside one of the atlas's reef tiles (`ocean-environments` B14),
    /// so nearest sampling never reads a fish's skin or the gutter.
    #[test]
    fn every_piece_is_laid_into_a_reef_tile() {
        use super::super::bodies::{atlas, rect_uv};
        let tiles: Vec<[f32; 4]> = atlas().tiles.values().map(|r| rect_uv(*r)).collect();
        let check = |what: &str, positions: usize, uvs: &[[f32; 2]]| {
            assert_eq!(positions, uvs.len(), "{what}: a UV a vertex");
            for uv in uvs {
                assert!(
                    tiles.iter().any(|r| inside(*uv, *r)),
                    "{what}: {uv:?} in no tile"
                );
            }
        };
        for (kind, shape) in library() {
            assert!(!shape.positions.is_empty(), "{kind:?} has faces");
            check(&format!("{kind:?}"), shape.positions.len(), &shape.uvs);
        }
        let axes = (Vec3::X, Vec3::Y, Vec3::Z);
        let mut ice = Verts::default();
        floe(Vec3::ZERO, axes, 2.0, 0.4, &mut ice);
        check("floe", ice.positions.len(), &ice.uvs);
    }

    /// Every fish variant is M9's body: faces, a UV a vertex, each in its own
    /// skin, its own fin tile or the white texel (the eyes).
    #[test]
    fn every_fish_variant_wears_its_own_skin() {
        use super::super::bodies::{VARIANTS, atlas, model_for, rect_uv, white_uv};
        let plans = &atlas().fish;
        for id in [
            "minnow",
            "silverfin",
            "perch",
            "ray",
            "eel",
            "reef",
            "deepback",
            "serpent",
            "darter",
            "anthias",
        ] {
            for v in 0..VARIANTS {
                let plan = plans
                    .iter()
                    .find(|p| p.id == id && p.variant == v)
                    .unwrap_or_else(|| panic!("{id} variant {v} in the atlas"));
                let body = model_for(id, v + VARIANTS * 3);
                let (skin, fin) = (rect_uv(plan.skin), rect_uv(plan.fin_tile));
                assert!(body.triangles() > 100, "{id}: {} faces", body.triangles());
                assert!(body.triangles() < 500, "{id}: {} faces", body.triangles());
                assert_eq!(body.base.uvs.len(), body.base.positions.len());
                for uv in &body.base.uvs {
                    assert!(
                        inside(*uv, skin) || inside(*uv, fin) || *uv == white_uv(),
                        "{id}: {uv:?}"
                    );
                }
            }
        }
        // The other creatures are modelled in M9's manner (B15): skinned,
        // in the fish's budget, every UV in a tile of the atlas.
        let tiles: Vec<[f32; 4]> = atlas().tiles.values().map(|r| rect_uv(*r)).collect();
        for (id, n) in [
            ("shore_crab", 0),
            ("lobster", 0),
            ("giant_clam", 0),
            ("reef_turtle", 0),
            ("sea_otter", 0),
            ("seal", 0),
            ("reed_dabbler", 0),
            ("reed_dabbler", 1),
            ("pond_diver", 0),
            ("octopus", 0),
            ("tube_worm", 0),
        ] {
            let body = model_for(id, n);
            let faces = body.triangles();
            eprintln!("{id} {n}: {faces} triangles");
            assert!(faces > 100 && faces < 500, "{id}: {faces} faces");
            let all = std::iter::once(&body.base).chain(body.parts.iter().map(|p| &p.shape));
            let mut skinned = 0;
            for shape in all {
                assert_eq!(shape.uvs.len(), shape.positions.len(), "{id}");
                for uv in &shape.uvs {
                    assert!(tiles.iter().any(|r| inside(*uv, *r)), "{id}: {uv:?}");
                    skinned += usize::from(*uv != white_uv());
                }
            }
            assert!(skinned * 2 > body.triangles() * 3, "{id} wears a skin");
        }
    }

    /// Writes the game's sea pieces and every water creature's body, at rest,
    /// to `docs/mockups/fish/water_life_models.js` for M9's "Reef and sea
    /// floor" tab, so the mockup draws the game's own models in the game's
    /// own atlas (`ocean-environments` B14). Run on purpose:
    /// `cargo test -p pbd-app --profile fast export_water_life_models -- --ignored`.
    #[test]
    #[ignore]
    fn export_water_life_models() {
        use super::super::bodies::{Look, MODELS, Place, emit, model_for};
        use std::fmt::Write;
        let num = |v: f32| {
            let r = (v * 1000.0).round() / 1000.0;
            if r == 0.0 {
                "0".to_string()
            } else {
                format!("{r}")
            }
        };
        let list =
            |vals: &mut dyn Iterator<Item = f32>| vals.map(&num).collect::<Vec<_>>().join(",");
        let entry = |id: &str, name: &str, group: &str, v: &Verts, palette: &[u32]| {
            let mut e = String::new();
            write!(
                e,
                "{{\"id\":\"{id}\",\"name\":\"{name}\",\"group\":\"{group}\",\"pos\":[{}],\"uv\":[{}],\"col\":[{}],\"palette\":[{}]}}",
                list(&mut v.positions.iter().flatten().copied()),
                list(&mut v.uvs.iter().flatten().copied()),
                list(&mut v.colours.iter().flat_map(|c| [c[0], c[1], c[2]])),
                list(&mut palette.iter().flat_map(|h| {
                    let c = lin(*h);
                    [c[0], c[1], c[2]]
                })),
            )
            .unwrap();
            e
        };
        let axes = (Vec3::X, Vec3::Y, Vec3::Z);
        let mut out = Vec::new();
        let lib = library();
        let kinds = [
            (Kind::Branch, "branch", "Branching coral"),
            (Kind::Finger, "finger", "Finger coral"),
            (Kind::Boulder, "boulder", "Boulder and brain coral"),
            (Kind::Table, "table", "Table coral"),
            (Kind::Soft, "soft", "Soft coral"),
            (Kind::Tube, "tube", "Tube sponge"),
            (Kind::Barrel, "barrel", "Barrel sponge"),
            (Kind::Fan, "fan", "Sea fan"),
            (Kind::Anemone, "anemone", "Anemone"),
            (Kind::Hero, "hero", "Giant anemone"),
            (Kind::Turf, "turf", "Algae turf"),
            (Kind::Grass, "grass", "Sea grass"),
            (Kind::Weed, "weed", "Seaweed"),
            (Kind::Rubble, "rubble", "Coral rubble"),
            (Kind::Shell, "shell", "Shells"),
            (Kind::Chimney, "chimney", "Black smoker"),
            (Kind::Mat, "mat", "Bacterial mat"),
            (Kind::Mussels, "mussels", "Mussel bed"),
        ];
        for (kind, id, name) in kinds {
            let mut v = Verts::default();
            lay(&lib[&kind], Vec3::ZERO, axes, Vec3::ONE, [1.0; 3], &mut v);
            let group = if kind == Kind::Chimney || kind == Kind::Mat || kind == Kind::Mussels {
                "vent"
            } else {
                "reef"
            };
            out.push(entry(id, name, group, &v, palette(kind)));
        }
        let mut kelp = Verts::default();
        kelp_stand(Vec3::ZERO, axes, 5.0, 0.0, &mut kelp);
        out.push(entry("kelp", "Kelp", "kelp", &kelp, &[0xffffff]));
        let mut ice = Verts::default();
        floe(Vec3::ZERO, axes, 1.4, 0.4, &mut ice);
        out.push(entry("floe", "Ice floe", "ice", &ice, &[0xffffff]));
        for model in MODELS {
            let id = match *model {
                "dabbler_drake" => "reed_dabbler",
                "dabbler_hen" => "reed_dabbler",
                other => other,
            };
            let body = model_for(id, if *model == "dabbler_hen" { 1 } else { 0 });
            let mut v = Verts::default();
            emit(
                body,
                &Look {
                    t: 0.0,
                    phase: 0.0,
                    pose: pbd_core::fauna::Pose::Moving,
                },
                &Place {
                    origin: Vec3::ZERO,
                    forward: Vec3::X,
                    up: Vec3::Y,
                    scale: 1.0 / body.length,
                },
                [1.0; 3],
                &mut v.positions,
                &mut v.colours,
                &mut v.uvs,
            );
            out.push(entry(model, model, "creature", &v, &[0xffffff]));
        }
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/mockups/fish/water_life_models.js");
        std::fs::write(
            &path,
            format!(
                "// Written by places::tests::export_water_life_models; do not edit.\nwindow.WATER_LIFE_MODELS = {{\"models\":[\n{}\n]}};\n",
                out.join(",\n")
            ),
        )
        .unwrap();
    }
}

#[cfg(test)]
mod sway_tests {
    use super::*;

    /// MEASUREMENT (`memory-leak` 5.4): each reef piece's triangles, against
    /// the owner's budget of 500 a piece.
    #[test]
    #[ignore]
    fn print_piece_triangles() {
        let mut rows: Vec<_> = library()
            .iter()
            .map(|(k, s)| (s.positions.len() / 3, format!("{k:?}")))
            .collect();
        rows.sort();
        for (t, k) in rows {
            println!("PIECE {k} {t}");
        }
    }

    /// The GPU sway is the CPU's old `fill_bent`: the vertex moves along its
    /// axis by `sin(clock * 1.1 + phase + 0.6 h) * min(0.07 h, 0.7) * bend`.
    #[test]
    fn a_swaying_vertex_carries_its_reach_and_phase() {
        let still = Sway {
            rest: vec![Vec3::ZERO, Vec3::Y, Vec3::X * 2.0],
            colours: vec![[1.0; 4]; 3],
            uvs: vec![[0.0; 2]; 3],
            height: vec![0.0, 2.0, 20.0],
            axis: vec![Vec3::X, Vec3::Z, Vec3::Y],
            phase: vec![0.5, 1.0, 2.0],
        };
        let rows = sway_rows(&still);
        assert_eq!(rows[0], [0.0, 0.0, 0.0, 0.5]);
        assert!((rows[1][2] - 0.14).abs() < 1e-6 && (rows[1][3] - 2.2).abs() < 1e-6);
        assert!(
            (rows[2][1] - 0.7).abs() < 1e-6,
            "the reach is capped at 0.7 m"
        );
        let clock = 7.25f32;
        for i in 0..3 {
            let h = still.height[i];
            let old = still.axis[i]
                * (clock * 1.1 + still.phase[i] + h * 0.6).sin()
                * (0.07 * h).min(0.7);
            let r = rows[i];
            let new = Vec3::new(r[0], r[1], r[2]) * (clock * 1.1 + r[3]).sin();
            assert!((old - new).length() < 1e-4, "{i}");
        }
        let mesh = swaying_mesh(&still);
        assert!(mesh.attribute(ATTRIBUTE_SWAY).is_some());
    }

    /// The owner's budget: no reef piece over 500 triangles.
    #[test]
    fn every_reef_piece_is_inside_its_triangle_budget() {
        for (kind, shape) in library() {
            let tris = shape.positions.len() / 3;
            assert!(tris <= MAX_PIECE_TRIS, "{kind:?}: {tris}");
        }
    }

    /// The owner's budget: a whole reach draws no more than 1000 reef
    /// pieces, wherever the eye is (a cube face's middle, an edge, a corner).
    #[test]
    fn a_reach_draws_at_most_a_thousand_pieces() {
        let radius = crate::planet::terrain_config().radius_m;
        for eye in [
            Vec3::Y,
            Vec3::new(1.0, 1.0, 0.0).normalize(),
            Vec3::ONE.normalize(),
            Vec3::new(0.3, -0.8, 0.52).normalize(),
        ] {
            let wanted = wanted_tiles(eye, radius);
            let near = wanted.values().filter(|l| **l == 0).count();
            let most: usize = wanted
                .iter()
                .map(|(k, l)| allowance(*l, tile_area_m2(*k, radius)))
                .sum();
            println!(
                "REACH {eye}: {} tiles, {near} near, {most} pieces at most",
                wanted.len()
            );
            assert!(most <= MAX_PIECES, "{eye}: {most}");
            assert!(near >= 1, "the eye's own tile is near");
        }
    }

    /// The shader's distances are the ones the CPU used.
    #[test]
    fn the_shader_keeps_the_sway_distances() {
        let wgsl = include_str!("../../../../assets/shaders/sway_vertex.wgsl");
        assert!(wgsl.contains(&format!("const NEAR_M: f32 = {NEAR_M:.1};")));
        assert!(wgsl.contains(&format!("const SWAY_EASE_M: f32 = {SWAY_EASE_M:.1};")));
    }
}
