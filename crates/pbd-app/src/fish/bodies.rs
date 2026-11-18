//! The water life's bodies: the M25 mockup's low-poly creatures
//! (`docs/mockups/ocean/creatures3d.js`) in Rust (`ocean-environments`
//! design B8, decision 11).
//!
//! A body is built once per model from the mockup's kit: rings along the
//! body, boxes, prisms, ellipsoids, flat polygons and an icosahedron, each
//! triangle one colour, read off the creature's sprite (sRGB hex, made linear
//! for the vertex colour). It is one unit long, head along +X and back along
//! +Y. Parts that move (a duck's head, wings and feet, a ray's wings, a
//! turtle's flippers, an octopus's arms, a clam's upper shell, a worm's
//! plume) hang from pivots and are turned each frame by [`emit`], which lays
//! the posed creature into a mesh, as the herd's rig is (`husbandry/rig.rs`).
//!
//! A swimmer's body wave (`fish-behaviour` design 8) is the vertex shader's
//! (`swim.wgsl`): [`emit_swimmer`] lays it rigid and writes, per vertex, the
//! attributes the shader bends it by. [`emit`], for meshes drawn without
//! them (the sea's places), bends it on the CPU by the same wave.

use bevy::mesh::MeshVertexAttribute;
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;
use pbd_core::fauna::Pose;
use pbd_core::fauna::swim::{Wave, WaveKind, WaveProfile};
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::OnceLock;

/// A shape: a triangle list, one colour a vertex (the same three times a
/// face) and its place in the water life's atlas (`ocean-environments` B14).
#[derive(Clone, Default, Debug)]
pub struct Shape {
    pub positions: Vec<Vec3>,
    pub colours: Vec<[f32; 4]>,
    pub uvs: Vec<[f32; 2]>,
}

/// A moving part on its pivot.
#[derive(Clone, Debug)]
pub struct Part {
    pub name: &'static str,
    pub at: Vec3,
    pub yaw: f32,
    pub shape: Shape,
}

/// How a body is animated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Fish,
    Ray,
    Crab,
    Lobster,
    Clam,
    Turtle,
    Otter,
    Seal,
    Bird,
    Octopus,
    TubeWorm,
}

/// How a swimmer's body waves (`fish-behaviour` design 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sway {
    None,
    Fish,
    Eel,
    /// M9's flat plan: the wings flap by their distance from the spine, and
    /// the whip bends behind them.
    Ray,
}

#[derive(Clone, Debug)]
pub struct Body {
    pub kind: Kind,
    pub sway: Sway,
    pub base: Shape,
    pub parts: Vec<Part>,
    /// The model's extent along its length axis (X; Y for a worm), model
    /// units: a species' `length_m` over this is its scale.
    pub length: f32,
    /// Where its nose is along +X, model units: the front of the model,
    /// where a fish's mouth is.
    pub nose: f32,
    /// A swimmer's spine, nose to tail, model units: its body wave's `u`
    /// runs 0 to 1 along it (a ray's whip beyond 1). And how far its widest
    /// point stands off the spine, model units (a ray's wing span).
    pub spine: f32,
    pub span: f32,
    /// Its row of the body-wave table the shader reads ([`wave_rows`]).
    pub wave_row: u32,
}

impl Body {
    pub fn triangles(&self) -> usize {
        (self.base.positions.len()
            + self
                .parts
                .iter()
                .map(|p| p.shape.positions.len())
                .sum::<usize>())
            / 3
    }
}

// ---- the kit ---------------------------------------------------------------

/// sRGB hex to a linear vertex colour.
pub(crate) fn lin(hex: u32) -> [f32; 4] {
    let c = Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8).to_linear();
    [c.red, c.green, c.blue, 1.0]
}

/// The mockup's `hh`: a hash of two integers to 0..1.
pub fn hh(a: i64, b: i64) -> f32 {
    let mut h = (a
        .wrapping_mul(374_761_393)
        .wrapping_add(b.wrapping_mul(668_265_263))) as i32;
    h = (h ^ ((h as u32) >> 13) as i32).wrapping_mul(1_274_126_177);
    ((h ^ ((h as u32) >> 16) as i32) as u32) as f32 / 4_294_967_296.0
}

/// A transform as the mockup writes one: position, Euler XYZ, scale.
pub(crate) fn tf(p: [f32; 3], r: [f32; 3], s: [f32; 3]) -> Mat4 {
    Mat4::from_scale_rotation_translation(
        Vec3::from(s),
        Quat::from_euler(EulerRot::XYZ, r[0], r[1], r[2]),
        Vec3::from(p),
    )
}

pub(crate) fn at(p: [f32; 3]) -> Mat4 {
    Mat4::from_translation(Vec3::from(p))
}

pub(crate) fn atr(p: [f32; 3], r: [f32; 3]) -> Mat4 {
    tf(p, r, [1.0; 3])
}

/// A shape being built. Each part added is laid into `tile` (a rect of the
/// atlas in UV) by a box projection: every face by the axis its normal is
/// nearest, at one scale for the whole part (its longest side across the
/// tile), so no face crosses the tile's edge. With no tile, the white
/// texel.
#[derive(Default)]
pub(crate) struct Kit(pub(crate) Shape, pub(crate) Option<[f32; 4]>);

impl Kit {
    /// Lay the parts added from now on into the atlas's tile `name`.
    pub(crate) fn tile(&mut self, name: &str) {
        self.1 = Some(tile(name));
    }

    /// Add a triangle list, coloured per face by `colour(centre, face)`.
    pub(crate) fn add_with(
        &mut self,
        tris: Vec<Vec3>,
        m: Mat4,
        colour: impl Fn(Vec3, usize) -> u32,
    ) {
        let placed: Vec<Vec3> = tris.iter().map(|p| m.transform_point3(*p)).collect();
        let (mut lo, mut hi) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
        for p in &placed {
            lo = lo.min(*p);
            hi = hi.max(*p);
        }
        let k = 1.0 / (hi - lo).max_element().max(1e-5);
        for (n, t) in placed.chunks_exact(3).enumerate() {
            let c = lin(colour((t[0] + t[1] + t[2]) / 3.0, n));
            let normal = (t[1] - t[0]).cross(t[2] - t[0]).abs();
            for p in t {
                let q = (*p - lo) * k;
                let uv = match self.1 {
                    None => white_uv(),
                    Some(r) if normal.y >= normal.x && normal.y >= normal.z => in_rect(r, q.x, q.z),
                    Some(r) if normal.x >= normal.z => in_rect(r, q.z, 1.0 - q.y),
                    Some(r) => in_rect(r, q.x, 1.0 - q.y),
                };
                self.0.positions.push(*p);
                self.0.colours.push(c);
                self.0.uvs.push(uv);
            }
        }
    }

    pub(crate) fn add(&mut self, tris: Vec<Vec3>, m: Mat4, hex: u32) {
        self.add_with(tris, m, |_, _| hex);
    }

    pub(crate) fn build(self) -> Shape {
        self.0
    }
}

/// three.js's `SphereGeometry(1, ws, hs)` scaled to an ellipsoid, in its own
/// face order. `theta_len` below pi is a cap (a jelly's bell).
pub(crate) fn ell(rx: f32, ry: f32, rz: f32, ws: usize, hs: usize) -> Vec<Vec3> {
    sphere(ws, hs, PI)
        .into_iter()
        .map(|p| p * Vec3::new(rx, ry, rz))
        .collect()
}

pub(crate) fn sphere(ws: usize, hs: usize, theta_len: f32) -> Vec<Vec3> {
    let mut grid = Vec::new();
    for iy in 0..=hs {
        let v = iy as f32 / hs as f32;
        let mut row = Vec::new();
        for ix in 0..=ws {
            let u = ix as f32 / ws as f32;
            let (phi, theta) = (u * TAU, v * theta_len);
            row.push(Vec3::new(
                -phi.cos() * theta.sin(),
                theta.cos(),
                phi.sin() * theta.sin(),
            ));
        }
        grid.push(row);
    }
    let mut out = Vec::new();
    for iy in 0..hs {
        for ix in 0..ws {
            let (a, b, c, d) = (
                grid[iy][ix + 1],
                grid[iy][ix],
                grid[iy + 1][ix],
                grid[iy + 1][ix + 1],
            );
            if iy != 0 {
                out.extend([a, b, d]);
            }
            if iy != hs - 1 || theta_len < PI {
                out.extend([b, c, d]);
            }
        }
    }
    out
}

/// A box centred on the origin.
pub(crate) fn bx(w: f32, h: f32, d: f32) -> Vec<Vec3> {
    let (x, y, z) = (w / 2.0, h / 2.0, d / 2.0);
    let v = |sx: f32, sy: f32, sz: f32| Vec3::new(sx * x, sy * y, sz * z);
    let quad = |a: Vec3, b: Vec3, c: Vec3, d: Vec3| [a, b, d, b, c, d];
    let mut out = Vec::with_capacity(36);
    // +x, -x, +y, -y, +z, -z, as three.js orders them.
    out.extend(quad(
        v(1., 1., 1.),
        v(1., -1., 1.),
        v(1., -1., -1.),
        v(1., 1., -1.),
    ));
    out.extend(quad(
        v(-1., 1., -1.),
        v(-1., -1., -1.),
        v(-1., -1., 1.),
        v(-1., 1., 1.),
    ));
    out.extend(quad(
        v(-1., 1., -1.),
        v(-1., 1., 1.),
        v(1., 1., 1.),
        v(1., 1., -1.),
    ));
    out.extend(quad(
        v(-1., -1., 1.),
        v(-1., -1., -1.),
        v(1., -1., -1.),
        v(1., -1., 1.),
    ));
    out.extend(quad(
        v(-1., 1., 1.),
        v(-1., -1., 1.),
        v(1., -1., 1.),
        v(1., 1., 1.),
    ));
    out.extend(quad(
        v(1., 1., -1.),
        v(1., -1., -1.),
        v(-1., -1., -1.),
        v(-1., 1., -1.),
    ));
    out
}

/// three.js's `CylinderGeometry(rt, rb, h, n, 1)`: the side, then the caps.
pub(crate) fn cyl(rt: f32, rb: f32, h: f32, n: usize) -> Vec<Vec3> {
    let ring = |r: f32, y: f32| -> Vec<Vec3> {
        (0..=n)
            .map(|x| {
                let t = x as f32 / n as f32 * TAU;
                Vec3::new(r * t.sin(), y, r * t.cos())
            })
            .collect()
    };
    let (top, bottom) = (ring(rt, h / 2.0), ring(rb, -h / 2.0));
    let mut out = Vec::new();
    for x in 0..n {
        let (a, b, c, d) = (top[x], bottom[x], bottom[x + 1], top[x + 1]);
        if rt > 0.0 {
            out.extend([a, b, d]);
        }
        if rb > 0.0 {
            out.extend([b, c, d]);
        }
    }
    if rt > 0.0 {
        let c = Vec3::new(0.0, h / 2.0, 0.0);
        for x in 0..n {
            out.extend([top[x], top[x + 1], c]);
        }
    }
    if rb > 0.0 {
        let c = Vec3::new(0.0, -h / 2.0, 0.0);
        for x in 0..n {
            out.extend([bottom[x + 1], bottom[x], c]);
        }
    }
    out
}

/// A flat polygon in the XY plane, ear-clipped (three.js's `ShapeGeometry`).
pub(crate) fn shape(points: &[[f32; 2]]) -> Vec<Vec3> {
    let pts: Vec<Vec2> = points.iter().map(|p| Vec2::new(p[0], p[1])).collect();
    let area: f32 = (0..pts.len())
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    let mut idx: Vec<usize> = (0..pts.len()).collect();
    if area < 0.0 {
        idx.reverse();
    }
    let cross = |o: Vec2, a: Vec2, b: Vec2| (a - o).perp_dot(b - o);
    let inside = |p: Vec2, a: Vec2, b: Vec2, c: Vec2| {
        cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0
    };
    let mut out = Vec::new();
    let mut guard = 0;
    while idx.len() > 3 && guard < 1000 {
        guard += 1;
        let n = idx.len();
        let mut clipped = false;
        for i in 0..n {
            let (ia, ib, ic) = (idx[(i + n - 1) % n], idx[i], idx[(i + 1) % n]);
            let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
            if cross(a, b, c) <= 1e-9 {
                continue;
            }
            let blocked = idx
                .iter()
                .any(|&j| j != ia && j != ib && j != ic && inside(pts[j], a, b, c));
            if blocked {
                continue;
            }
            out.extend([a, b, c].map(|p| p.extend(0.0)));
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            // A degenerate polygon: fan what is left.
            break;
        }
    }
    for i in 1..idx.len().saturating_sub(1) {
        out.extend([pts[idx[0]], pts[idx[i]], pts[idx[i + 1]]].map(|p| p.extend(0.0)));
    }
    out
}

pub(crate) fn icosahedron(r: f32) -> Vec<Vec3> {
    let t = (1.0 + 5f32.sqrt()) / 2.0;
    let v = [
        [-1., t, 0.],
        [1., t, 0.],
        [-1., -t, 0.],
        [1., -t, 0.],
        [0., -1., t],
        [0., 1., t],
        [0., -1., -t],
        [0., 1., -t],
        [t, 0., -1.],
        [t, 0., 1.],
        [-t, 0., -1.],
        [-t, 0., 1.],
    ]
    .map(|p| Vec3::from(p).normalize() * r);
    let f = [
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7,
        1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9,
        8, 1,
    ];
    f.iter().map(|&i| v[i]).collect()
}

/// A body of rings along X: `[x, half height, half width, y offset]`.
pub(crate) fn lathe(prof: &[[f32; 4]], nv: usize, rot: f32) -> Vec<Vec3> {
    let rings: Vec<Vec<Vec3>> = prof
        .iter()
        .map(|r| {
            (0..nv)
                .map(|j| {
                    let th = TAU * j as f32 / nv as f32 + rot;
                    Vec3::new(r[0], r[3] + r[1] * th.cos(), r[2] * th.sin())
                })
                .collect()
        })
        .collect();
    let mut out = Vec::new();
    for i in 0..rings.len().saturating_sub(1) {
        for j in 0..nv {
            let (a, b) = (rings[i][j], rings[i][(j + 1) % nv]);
            let (c, d) = (rings[i + 1][j], rings[i + 1][(j + 1) % nv]);
            out.extend([a, c, b, b, c, d]);
        }
    }
    out
}

pub(crate) fn rotate_z(tris: Vec<Vec3>, a: f32) -> Vec<Vec3> {
    let q = Quat::from_rotation_z(a);
    tris.into_iter().map(|p| q * p).collect()
}

pub(crate) fn translate(tris: Vec<Vec3>, by: [f32; 3]) -> Vec<Vec3> {
    tris.into_iter().map(|p| p + Vec3::from(by)).collect()
}

// ---- the water life's atlas (`ocean-environments` B14) ----------------------

/// The manifest `tools/water_life_atlas.py` writes beside the atlas: every
/// rect, and each fish variant's genome and body plan (M9's numbers, read
/// here rather than copied).
#[derive(serde::Deserialize, Debug)]
pub struct Atlas {
    pub size: (u32, u32),
    pub tiles: HashMap<String, (u32, u32, u32, u32)>,
    pub fish: Vec<FishPlan>,
}

/// One fish variant: M9's `genome(species, seed)` and its species' plan.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct FishPlan {
    pub id: String,
    pub variant: u32,
    pub seed: u32,
    pub skin: (u32, u32, u32, u32),
    pub fin_tile: (u32, u32, u32, u32),
    pub profile: String,
    pub tail: String,
    pub d: f32,
    pub w: f32,
    pub u_max: f32,
    pub ped: f32,
    pub body: f32,
    pub eel: bool,
    pub flat: bool,
    pub depth: f32,
    pub width: f32,
    pub fin: f32,
    pub length_k: f32,
    pub dorsal: Vec<(f32, f32, f32, String)>,
    pub anal: Vec<(f32, f32, f32)>,
    pub pec: f32,
    pub eye: f32,
    pub crest: bool,
}

/// The atlas's path under `assets/`.
pub const ATLAS_PNG: &str = "textures/water_life/atlas.png";
/// Variants a fish species has in the atlas; a creature wears `id % this`.
pub const VARIANTS: u32 = 4;

pub fn atlas() -> &'static Atlas {
    static ATLAS: OnceLock<Atlas> = OnceLock::new();
    ATLAS.get_or_init(|| {
        ron::from_str(include_str!(
            "../../../../assets/textures/water_life/atlas.ron"
        ))
        .expect("assets/textures/water_life/atlas.ron")
    })
}

/// A rect of the atlas in UV: x, y, width, height, each 0..1.
pub fn rect_uv(r: (u32, u32, u32, u32)) -> [f32; 4] {
    let (w, h) = atlas().size;
    [
        r.0 as f32 / w as f32,
        r.1 as f32 / h as f32,
        r.2 as f32 / w as f32,
        r.3 as f32 / h as f32,
    ]
}

/// A reef tile's rect in UV, by its name in the manifest.
pub fn tile(name: &str) -> [f32; 4] {
    rect_uv(
        *atlas()
            .tiles
            .get(name)
            .unwrap_or_else(|| panic!("water-life tile {name}")),
    )
}

/// A point `(s, t)` (0..1 each, t down) inside a rect, held a quarter texel
/// in from its edges so nearest sampling never reads the gutter.
pub fn in_rect(r: [f32; 4], s: f32, t: f32) -> [f32; 2] {
    let (w, h) = atlas().size;
    let (ix, iy) = (0.25 / w as f32, 0.25 / h as f32);
    [
        r[0] + ix + s.clamp(0.0, 1.0) * (r[2] - 2.0 * ix),
        r[1] + iy + t.clamp(0.0, 1.0) * (r[3] - 2.0 * iy),
    ]
}

/// The UV of an untextured face: the middle of the white tile.
pub fn white_uv() -> [f32; 2] {
    static WHITE: OnceLock<[f32; 2]> = OnceLock::new();
    *WHITE.get_or_init(|| in_rect(tile("white"), 0.5, 0.5))
}

// ---- fish (M9's buildFish, `docs/mockups/fish.html`) -----------------------

/// M9's `profH`: the body's depth along it, 0 at the nose to the stalk.
fn prof_h(u: f32, p: &FishPlan) -> f32 {
    let mut h = if u <= p.u_max {
        let t = u / p.u_max;
        (2.0 * t - t * t).max(0.0).sqrt()
    } else {
        let t = (u - p.u_max) / (1.0 - p.u_max);
        p.ped + (1.0 - p.ped) * 0.5 * (1.0 + (PI * t).cos())
    };
    if p.eel {
        h *= (1.0 - u.powi(3)).max(0.04);
    }
    h
}

/// M9's `rayW`: a diamond in plan.
fn ray_w(u: f32) -> f32 {
    if u < 0.4 {
        (u / 0.4).powf(0.95)
    } else {
        ((1.0 - u) / 0.6).powf(1.25) * 0.97 + 0.03
    }
}

/// Texels of the fin tile a model unit (M9's 8 px tile repeated nine times).
const FIN_TEXELS_PER_UNIT: f32 = 72.0;

struct FishKit {
    shape: Shape,
}

impl FishKit {
    fn tri(&mut self, p: [Vec3; 3], uv: [[f32; 2]; 3], colour: [f32; 4]) {
        // A face with no area (the nose's point, a cap) is dropped: it has
        // no normal.
        if (p[1] - p[0]).cross(p[2] - p[0]).length_squared() < 1e-14 {
            return;
        }
        for i in 0..3 {
            self.shape.positions.push(p[i]);
            self.shape.colours.push(colour);
            self.shape.uvs.push(uv[i]);
        }
    }

    /// A flat fin in the XY plane, through `m`, in the fin tile.
    fn fin(&mut self, pts: &[[f32; 2]], m: Mat4, tile: [f32; 4]) {
        let (mut lo, mut hi) = (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY));
        for p in pts {
            lo = lo.min(Vec2::from(*p));
            hi = hi.max(Vec2::from(*p));
        }
        let span = (hi - lo).max_element().max(1e-4) * FIN_TEXELS_PER_UNIT;
        let k = (31.5 / span).min(1.0) * FIN_TEXELS_PER_UNIT / 32.0;
        let tris = shape(pts);
        for t in tris.chunks_exact(3) {
            let uv = [t[0], t[1], t[2]].map(|p| in_rect(tile, (p.x - lo.x) * k, (hi.y - p.y) * k));
            let p = [t[0], t[1], t[2]].map(|p| m.transform_point3(p));
            self.tri(p, uv, [1.0; 4]);
        }
    }

    /// A box in one colour on the white texel (M9's block eye).
    fn block(&mut self, size: Vec3, at: Vec3, hex: u32) {
        let w = white_uv();
        let c = lin(hex);
        let tris = bx(size.x, size.y, size.z);
        for t in tris.chunks_exact(3) {
            self.tri([t[0] + at, t[1] + at, t[2] + at], [w; 3], c);
        }
    }
}

/// A fish of length one, head along +X and back along +Y, from its M9 plan:
/// a six-sided body in rings, its skin by length and the angle round it,
/// the tail, dorsal and anal fins, an eel's ribbons and a serpent's crest,
/// the pectorals, a ray's whip, and the block eyes.
fn m9_fish(p: &FishPlan) -> Shape {
    let mut k = FishKit {
        shape: Shape::default(),
    };
    let skin = rect_uv(p.skin);
    let fin_tile = rect_uv(p.fin_tile);
    let d = p.d * p.depth;
    let wd = p.w * p.width;
    let bl = p.body;
    let x0 = 0.5;
    let nu = if p.eel { 22 } else { 12 };
    let nv = 6;
    let h_at = |u: f32| d * prof_h(u, p);
    let w_at = |u: f32| {
        if p.flat {
            wd * ray_w(u)
        } else {
            wd * prof_h(u, p)
        }
    };
    let x_at = |u: f32| x0 - u * bl;
    // The body's rings, as M9 lays them.
    let mut ring: Vec<Vec<(Vec3, [f32; 2])>> = Vec::new();
    for i in 0..=nu + 1 {
        let u = (i as f32 / nu as f32).min(1.0);
        let cap = i == nu + 1;
        let xx = x0 - u * bl - if cap { 0.004 } else { 0.0 };
        let (h, w) = if cap { (0.0, 0.0) } else { (h_at(u), w_at(u)) };
        ring.push(
            (0..=nv)
                .map(|j| {
                    let th = TAU * j as f32 / nv as f32 + PI / nv as f32;
                    let belly = if th.cos() < 0.0 { 0.94 } else { 1.0 };
                    (
                        Vec3::new(xx, h * th.cos() * belly, w * th.sin()),
                        in_rect(skin, u, j as f32 / nv as f32),
                    )
                })
                .collect(),
        );
    }
    for i in 0..=nu {
        for j in 0..nv {
            let (a, a1) = (ring[i][j], ring[i][j + 1]);
            let (b, b1) = (ring[i + 1][j], ring[i + 1][j + 1]);
            k.tri([a.0, b.0, a1.0], [a.1, b.1, a1.1], [1.0; 4]);
            k.tri([b.0, b1.0, a1.0], [b.1, b1.1, a1.1], [1.0; 4]);
        }
    }
    let f = p.fin;
    let xt = x_at(1.0);
    let hp = h_at(1.0).max(0.012);
    let tl = (1.0 - bl + 0.04) * f;
    let tail: Vec<[f32; 2]> = match p.tail.as_str() {
        "forked" => vec![
            [xt + 0.01, hp],
            [xt - tl, hp + tl * 0.75],
            [xt - tl * 0.55, 0.0],
            [xt - tl, -hp - tl * 0.75],
            [xt + 0.01, -hp],
        ],
        "deepfork" => vec![
            [xt + 0.01, hp],
            [xt - tl * 1.05, hp + tl * 0.95],
            [xt - tl * 0.85, hp + tl * 0.7],
            [xt - tl * 0.45, 0.0],
            [xt - tl * 0.85, -hp - tl * 0.7],
            [xt - tl * 1.05, -hp - tl * 0.95],
            [xt + 0.01, -hp],
        ],
        "lunate" => vec![
            [xt + 0.01, hp],
            [xt - tl * 1.05, hp + tl * 1.05],
            [xt - tl * 0.6, hp * 0.5],
            [xt - tl * 0.5, 0.0],
            [xt - tl * 0.6, -hp * 0.5],
            [xt - tl * 1.05, -hp - tl * 1.05],
            [xt + 0.01, -hp],
        ],
        "truncate" => vec![
            [xt + 0.01, hp],
            [xt - tl, hp + tl * 0.6],
            [xt - tl * 1.03, 0.0],
            [xt - tl, -hp - tl * 0.6],
            [xt + 0.01, -hp],
        ],
        "rounded" => {
            let mut pts = vec![[xt + 0.01, hp]];
            for i in 0..=12 {
                let a = -FRAC_PI_2 + PI * i as f32 / 12.0;
                pts.push([
                    xt - tl * 0.3 - tl * 0.75 * a.cos(),
                    -(hp + tl * 0.55) * a.sin(),
                ]);
            }
            pts.push([xt + 0.01, -hp]);
            pts
        }
        _ => Vec::new(),
    };
    if !tail.is_empty() {
        k.fin(&tail, Mat4::IDENTITY, fin_tile);
    }
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    for (u0, u1, hh_, kind) in &p.dorsal {
        let h = hh_ * f;
        let base: Vec<[f32; 2]> = (0..=10)
            .map(|i| {
                let u = lerp(*u0, *u1, i as f32 / 10.0);
                [x_at(u), h_at(u) * 0.86]
            })
            .collect();
        let rev = |from: usize, to: usize| base[from..to].iter().rev().copied();
        let pts: Vec<[f32; 2]> = match kind.as_str() {
            "spiny" => {
                let mut pts = vec![base[0]];
                for (i, b) in base.iter().enumerate() {
                    let up = if i % 2 == 1 { 0.55 } else { 0.95 };
                    pts.push([b[0], b[1] + h * up * (1.0 - 0.3 * i as f32 / 10.0)]);
                }
                pts.extend(base.iter().rev().copied());
                pts
            }
            "tall" => [
                base[0],
                [base[1][0] + h * 0.15, base[1][1] + h * 1.3],
                [base[10][0] - h * 0.5, base[10][1] + h * 0.6],
                base[10],
            ]
            .into_iter()
            .chain(rev(1, 10))
            .collect(),
            _ => [
                base[0],
                [base[3][0] + h * 0.1, base[3][1] + h],
                [base[10][0] - h * 0.35, base[10][1] + h * 0.25],
                base[10],
            ]
            .into_iter()
            .chain(rev(1, 10))
            .collect(),
        };
        k.fin(&dedup(pts), Mat4::IDENTITY, fin_tile);
    }
    for (u0, u1, hh_) in &p.anal {
        let h = hh_ * f;
        let base: Vec<[f32; 2]> = (0..=8)
            .map(|i| {
                let u = lerp(*u0, *u1, i as f32 / 8.0);
                [x_at(u), -h_at(u) * 0.84]
            })
            .collect();
        let pts: Vec<[f32; 2]> = [
            base[0],
            [base[2][0] - h * 0.3, base[2][1] - h],
            [base[8][0] - h * 0.3, base[8][1] - h * 0.3],
            base[8],
        ]
        .into_iter()
        .chain(base[1..8].iter().rev().copied())
        .collect();
        k.fin(&pts, Mat4::IDENTITY, fin_tile);
    }
    if p.eel {
        // Two thin ribbons along the back and the belly, meeting at the tip.
        let n = 24;
        let mut up = Vec::new();
        let mut dn = Vec::new();
        let mut in_up = Vec::new();
        let mut in_dn = Vec::new();
        for i in 0..=n {
            let u = lerp(0.35, 1.0, i as f32 / n as f32);
            let h = h_at(u);
            let fw = (0.003 + 0.007 * (u - 0.35) / 0.65) * f;
            up.push([x_at(u), h + fw]);
            dn.push([x_at(u), -h - fw]);
            in_up.push([x_at(u), h * 0.85]);
            in_dn.push([x_at(u), -h * 0.85]);
        }
        let tip = [x_at(1.0) - 0.03, 0.0];
        let top: Vec<[f32; 2]> = up
            .iter()
            .copied()
            .chain([tip])
            .chain(in_up.iter().rev().copied())
            .collect();
        let bottom: Vec<[f32; 2]> = in_dn
            .iter()
            .copied()
            .chain([tip])
            .chain(dn.iter().rev().copied())
            .collect();
        k.fin(&dedup(top), Mat4::IDENTITY, fin_tile);
        k.fin(&dedup(bottom), Mat4::IDENTITY, fin_tile);
        if p.crest {
            let mut pts = Vec::new();
            for i in 0..=18 {
                let u = lerp(0.03, 0.55, i as f32 / 18.0);
                let h = h_at(u);
                let spike = if i % 2 == 1 { 0.05 } else { 0.016 };
                pts.push([
                    x_at(u) + 0.004,
                    h * 0.8 + spike * f * (1.0 - i as f32 / 22.0),
                ]);
            }
            for i in (0..=18).rev() {
                let u = lerp(0.03, 0.55, i as f32 / 18.0);
                pts.push([x_at(u), h_at(u) * 0.6]);
            }
            k.fin(&pts, Mat4::IDENTITY, fin_tile);
        }
    }
    if p.pec > 0.0 {
        let u = if p.eel { 0.07 } else { 0.24 };
        let s = p.pec * f;
        let pts = [
            [0.0, 0.0],
            [-s * 1.3, s * 0.15],
            [-s * 1.6, -s * 0.35],
            [-s * 0.9, -s * 0.5],
            [0.0, -s * 0.18],
        ];
        for side in [1.0f32, -1.0] {
            let m = Mat4::from_rotation_translation(
                Quat::from_rotation_y(side * 0.5),
                Vec3::new(x_at(u), -h_at(u) * 0.25, side * w_at(u) * 0.9),
            );
            k.fin(&pts, m, fin_tile);
        }
    }
    if p.flat {
        // The whip: a thin cone in eight rings, so the tail can bend.
        let (x1, len) = (x_at(1.0), 0.62);
        let rings: Vec<[f32; 4]> = (0..=8)
            .map(|i| {
                let t = i as f32 / 8.0;
                let r = 0.016 + (0.004 - 0.016) * t;
                [x1 + 0.02 - t * len, r, r, 0.01]
            })
            .collect();
        let tris = lathe(&rings, 6, PI / 6.0);
        for t in tris.chunks_exact(3) {
            let uv = [t[0], t[1], t[2]].map(|q| in_rect(fin_tile, (x1 + 0.02 - q.x) / len, 0.5));
            k.tri([t[0], t[1], t[2]], uv, [1.0; 4]);
        }
    }
    let eu = if p.eel {
        0.045
    } else if p.flat {
        0.22
    } else {
        0.08
    };
    let er = p.eye;
    for side in [1.0f32, -1.0] {
        let ey = if p.flat {
            h_at(eu) * 0.9
        } else {
            h_at(0.2f32.min(p.u_max)) * 0.32
        };
        let ez = if p.flat {
            side * w_at(eu) * 0.35
        } else {
            side * (w_at(eu) * 0.9 + er * 0.25)
        };
        let e = Vec3::new(x_at(eu), ey, ez);
        k.block(Vec3::new(er * 1.8, er * 1.8, er * 1.2), e, 0xf4f0e2);
        let pupil = e + Vec3::new(
            er * 0.12,
            if p.flat { er * 0.55 } else { 0.0 },
            side * er * 0.55,
        );
        k.block(Vec3::new(er, er, er * 0.4), pupil, 0x0c0a10);
    }
    k.shape
}

/// A polygon with no point repeated next to itself (M9's `spiny` fin starts
/// on its first base point twice).
fn dedup(pts: Vec<[f32; 2]>) -> Vec<[f32; 2]> {
    let mut out: Vec<[f32; 2]> = Vec::with_capacity(pts.len());
    for p in pts {
        if out
            .last()
            .is_none_or(|q| (q[0] - p[0]).abs() + (q[1] - p[1]).abs() > 1e-6)
        {
            out.push(p);
        }
    }
    while out.len() > 3
        && (out[0][0] - out[out.len() - 1][0]).abs() + (out[0][1] - out[out.len() - 1][1]).abs()
            < 1e-6
    {
        out.pop();
    }
    out
}

/// The fish a plan builds, with its sway: an eel's wave, a ray's flap, or
/// the tail's beat.
fn m9_body(p: &FishPlan) -> Body {
    let sway = if p.eel {
        Sway::Eel
    } else if p.flat {
        Sway::Ray
    } else {
        Sway::Fish
    };
    let kind = if p.flat { Kind::Ray } else { Kind::Fish };
    let mut body = Body::new(kind, sway, m9_fish(p), Vec::new());
    // M9 builds a fish of length one and scales it by the species' length.
    body.length = 1.0;
    // A ray's disc is the length; its whip runs on past it (`u` over 1).
    if p.flat {
        body.spine = 1.0;
    }
    body
}

mod creatures;

// ---- the species ------------------------------------------------------------

impl Body {
    fn new(kind: Kind, sway: Sway, base: Shape, parts: Vec<Part>) -> Self {
        let mut body = Self {
            kind,
            sway,
            base,
            parts,
            length: 1.0,
            nose: 0.0,
            spine: 1.0,
            span: 0.0,
            wave_row: 0,
        };
        let axis = |p: Vec3| if kind == Kind::TubeWorm { p.y } else { p.x };
        let mut lo = f32::INFINITY;
        let mut hi = f32::NEG_INFINITY;
        let all = body.base.positions.iter().copied().chain(
            body.parts
                .iter()
                .flat_map(|part| part.shape.positions.iter().map(move |p| *p + part.at)),
        );
        for p in all {
            lo = lo.min(axis(p));
            hi = hi.max(axis(p));
        }
        body.length = (hi - lo).max(1e-3);
        body.nose = body
            .base
            .positions
            .iter()
            .map(|p| p.x)
            .fold(f32::NEG_INFINITY, f32::max)
            .max(0.0);
        let tail = body
            .base
            .positions
            .iter()
            .map(|p| p.x)
            .fold(f32::INFINITY, f32::min);
        body.spine = (body.nose - tail).max(1e-3);
        body.span = body
            .base
            .positions
            .iter()
            .map(|p| p.z.abs())
            .fold(0.0, f32::max)
            .max(1e-3);
        body
    }

    /// Its body wave, if it is a swimmer.
    pub fn wave_kind(&self) -> Option<WaveKind> {
        match self.sway {
            Sway::None => None,
            Sway::Fish => Some(WaveKind::Fish),
            Sway::Eel => Some(WaveKind::Eel),
            Sway::Ray => Some(WaveKind::Ray),
        }
    }
}

/// A model by name that is not one of M9's fish (those are built from the
/// atlas's plans, [`model_for`]).
fn build(model: &str) -> Body {
    match model {
        "shore_crab" => Body::new(Kind::Crab, Sway::None, creatures::crab(), Vec::new()),
        "lobster" => Body::new(Kind::Lobster, Sway::None, creatures::lobster(), Vec::new()),
        "giant_clam" => {
            let (lower, upper) = creatures::clam();
            let lid = Part {
                name: "lid",
                at: Vec3::new(0.0, 0.2, 0.36),
                yaw: 0.0,
                shape: upper,
            };
            Body::new(Kind::Clam, Sway::None, lower, vec![lid])
        }
        "reef_turtle" => {
            let (base, parts) = creatures::turtle();
            Body::new(Kind::Turtle, Sway::None, base, parts)
        }
        "sea_otter" => Body::new(Kind::Otter, Sway::None, creatures::otter(), Vec::new()),
        "seal" => Body::new(Kind::Seal, Sway::Fish, creatures::seal(), Vec::new()),
        "octopus" => {
            let (base, parts) = creatures::octopus();
            Body::new(Kind::Octopus, Sway::None, base, parts)
        }
        "tube_worm" => {
            let (base, parts) = creatures::tube_worm();
            Body::new(Kind::TubeWorm, Sway::None, base, parts)
        }
        "dabbler_drake" => bird(&creatures::Duck {
            body: "drake",
            head: "drake_head",
            bill: 0xfaaf5c,
            leg: 0xfb902f,
            ring: Some(0xfffefe),
            spec: Some(0x1096c3),
            crest: None,
            tail: 0x2c2c25,
        }),
        "dabbler_hen" => bird(&creatures::Duck {
            body: "hen",
            head: "hen_head",
            bill: 0xfb902f,
            leg: 0xfb902f,
            ring: None,
            spec: Some(0x1096c3),
            crest: None,
            tail: 0x725244,
        }),
        "pond_diver" => bird(&creatures::Duck {
            body: "diver",
            head: "diver_head",
            bill: 0x92a3b2,
            leg: 0x92a3b2,
            ring: None,
            spec: Some(0xfffefe),
            crest: Some(0x161612),
            tail: 0x161612,
        }),
        other => panic!("no body for {other}"),
    }
}

fn bird(p: &creatures::Duck) -> Body {
    let (base, parts) = creatures::duck(p);
    Body::new(Kind::Bird, Sway::None, base, parts)
}

/// Every model the table builds, by name: the species ids, and the dabbler's
/// hen beside its drake.
pub const MODELS: &[&str] = &[
    "minnow",
    "silverfin",
    "perch",
    "ray",
    "eel",
    "reef",
    "deepback",
    "serpent",
    "shore_crab",
    "lobster",
    "giant_clam",
    "reef_turtle",
    "sea_otter",
    "dabbler_drake",
    "dabbler_hen",
    "pond_diver",
    "octopus",
    "tube_worm",
    "seal",
    "darter",
    "anthias",
];

/// The model a species' creature `id` wears: a fish its M9 variant
/// `id % VARIANTS` (`ocean-environments` B14), the hen for every second
/// reed dabbler, and the minnow's for a species with none.
pub fn model_for(species: &str, id: u32) -> &'static Body {
    let fish = fish_bodies();
    let variant = id % VARIANTS;
    if let Some(body) = fish.get(&(species.to_string(), variant)) {
        return body;
    }
    let others = other_bodies();
    let name = match species {
        "reed_dabbler" if id % 2 == 1 => "dabbler_hen",
        "reed_dabbler" => "dabbler_drake",
        other => other,
    };
    others
        .get(name)
        .unwrap_or_else(|| &fish[&("minnow".to_string(), variant)])
}

/// Every M9 fish's body by species and variant, each given its row of the
/// body-wave table in the plans' order.
fn fish_bodies() -> &'static HashMap<(String, u32), Body> {
    static FISH: OnceLock<HashMap<(String, u32), Body>> = OnceLock::new();
    FISH.get_or_init(|| {
        atlas()
            .fish
            .iter()
            .enumerate()
            .map(|(row, plan)| {
                let mut body = m9_body(plan);
                body.wave_row = row as u32;
                ((plan.id.clone(), plan.variant), body)
            })
            .collect()
    })
}

/// Every other model's body, its rows after the fish's in `MODELS` order.
fn other_bodies() -> &'static HashMap<&'static str, Body> {
    static OTHERS: OnceLock<HashMap<&'static str, Body>> = OnceLock::new();
    OTHERS.get_or_init(|| {
        let fish = fish_bodies();
        let first = atlas().fish.len();
        MODELS
            .iter()
            .filter(|m| !fish.contains_key(&(m.to_string(), 0)))
            .enumerate()
            .map(|(k, m)| {
                let mut body = build(m);
                body.wave_row = (first + k) as u32;
                (*m, body)
            })
            .collect()
    })
}

/// One row of the body-wave table per body, in row order: its wave and its
/// span over its spine (a ray's wing in body lengths). `swim.wgsl` reads a
/// row by the index each vertex carries; the numbers of each wave come from
/// `fauna.ron` ([`wave_table`]).
pub fn wave_rows() -> Vec<(Option<WaveKind>, f32)> {
    let mut rows: Vec<(u32, Option<WaveKind>, f32)> = fish_bodies()
        .values()
        .chain(other_bodies().values())
        .map(|b| (b.wave_row, b.wave_kind(), b.span / b.spine))
        .collect();
    rows.sort_by_key(|r| r.0);
    rows.into_iter().map(|(_, k, s)| (k, s)).collect()
}

/// A body-wave table row as `swim.wgsl`'s `SwimRow` reads it: waves,
/// exponent, head, recoil; recoil reach, kind (0 a swimmer, 1 a ray), span,
/// wing exponent; wing waves, flap, whip, nothing.
pub fn wave_row(kind: Option<WaveKind>, span: f32, w: &WaveProfile) -> [[f32; 4]; 3] {
    let ray = if kind == Some(WaveKind::Ray) {
        1.0
    } else {
        0.0
    };
    [
        [w.waves, w.expo, w.head, w.recoil],
        [w.recoil_reach, ray, span, w.wing_expo],
        [w.wing_waves, w.flap, w.whip, 0.0],
    ]
}

/// The whole table, from the waves in `fauna.ron`.
pub fn wave_table(s: &pbd_core::fauna::BehaviourSettings) -> Vec<[[f32; 4]; 3]> {
    wave_rows()
        .into_iter()
        .map(|(kind, span)| {
            let w = kind.map_or(&s.fish_wave, |k| s.wave(k));
            wave_row(kind, span, w)
        })
        .collect()
}

/// Whether a species has a model of its own.
pub fn has_model(species: &str) -> bool {
    species == "reed_dabbler" || MODELS.contains(&species)
}

// ---- posing ------------------------------------------------------------------

/// What a creature is doing this frame, for its drawing.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    /// The world clock, s, wrapped so an f32 keeps its precision.
    pub t: f32,
    /// The creature's own phase, radians.
    pub phase: f32,
    pub pose: Pose,
}

/// A part's turn and stretch this frame (the mockup's `animate`).
fn animate(kind: Kind, name: &str, index: usize, yaw: f32, look: &Look) -> (Quat, Vec3) {
    let (t, ph) = (look.t, look.phase);
    let e = |x: f32, y: f32, z: f32| Quat::from_euler(EulerRot::XYZ, x, y, z);
    let one = Vec3::ONE;
    match (kind, name) {
        (Kind::Bird, "wingR") => (e(0.25, -1.42, 0.0), one),
        (Kind::Bird, "wingL") => (e(-0.25, 1.42, 0.0), one),
        (Kind::Bird, "head") => match look.pose {
            Pose::Asleep => (e(0.0, 2.6, 0.9), one),
            Pose::TippedUp => (e(0.0, (t * 0.7 + ph).sin() * 0.25, -0.6), one),
            _ => (e(0.0, (t * 0.7 + ph).sin() * 0.25, 0.0), one),
        },
        (Kind::Bird, "footR" | "footL") => {
            let pad = if look.pose == Pose::Asleep {
                0.0
            } else {
                (t * 5.0 + ph).sin() * 0.6
            };
            let pad = if name == "footR" { pad } else { -pad };
            (e(0.0, 0.0, pad), one)
        }
        (Kind::Ray, _) => {
            let f = (t * 1.6 + ph).sin() * 0.35;
            (e(if name == "wingR" { f } else { -f }, 0.0, 0.0), one)
        }
        (Kind::Turtle, _) => {
            let f = (t * 1.4 + ph).sin();
            let (x, y) = match name {
                "fr" => (f * 0.5, 0.2 * f),
                "fl" => (-f * 0.5, 0.2 * f),
                "br" => (-f * 0.25, 0.0),
                _ => (f * 0.25, 0.0),
            };
            (e(x, y, 0.0), one)
        }
        (Kind::Octopus, _) => {
            let i = index as f32;
            let calm = if look.pose == Pose::Inking { 0.2 } else { 1.0 };
            let z = -0.25 + (t * 2.0 + i * 0.9 + ph).sin() * 0.18 * calm;
            let x = (t * 1.3 + i).sin() * 0.15;
            (e(x, yaw, z), one)
        }
        (Kind::Clam, _) => {
            let open = if look.pose == Pose::Shut {
                0.0
            } else {
                0.6 + 0.4 * (t * 0.25 + ph).sin()
            };
            (e(open * 0.9, 0.0, 0.0), one)
        }
        (Kind::TubeWorm, _) => {
            if look.pose == Pose::Shut {
                (Quat::IDENTITY, Vec3::new(0.5, 0.15, 0.5))
            } else {
                let sway = (t * 0.9 + ph).sin() * 0.08;
                (e(sway, 0.0, sway * 0.6), one)
            }
        }
        _ => (Quat::from_rotation_y(yaw), one),
    }
}

/// The body wave's per-vertex attributes (`swim.wgsl`): the creature's side
/// and up, scaled to metres a body length (its spine), with `u` (0 nose to 1
/// tail) and its row of the table; its wave's phase, amplitude and bend, and
/// the vertex's distance off the spine, body lengths.
pub const ATTRIBUTE_SWIM_AXES: MeshVertexAttribute =
    MeshVertexAttribute::new("Swim_Axes", 0x5eed_f15b_0001, VertexFormat::Float32x4);
pub const ATTRIBUTE_SWIM_WAVE: MeshVertexAttribute =
    MeshVertexAttribute::new("Swim_Wave", 0x5eed_f15b_0002, VertexFormat::Float32x4);
pub const ATTRIBUTE_SWIM_UP: MeshVertexAttribute =
    MeshVertexAttribute::new("Swim_Up", 0x5eed_f15b_0003, VertexFormat::Float32x4);

/// `swim.wgsl`'s bend of a model point `p` by `wave` (`fish-behaviour`
/// design 8.2), in model units: for meshes drawn without the shader's
/// attributes. The shader is the one the schools are drawn with; this is
/// the same maths, on the CPU.
fn bend(body: &Body, p: Vec3, wave: Wave, w: &WaveProfile) -> Vec3 {
    let Some(kind) = body.wave_kind() else {
        return p;
    };
    if wave.amp == 0.0 && wave.bend == 0.0 {
        return p;
    }
    let spine = body.spine;
    let u = (body.nose - p.x) / spine;
    let d = p.z / spine;
    if kind == WaveKind::Ray {
        let span = body.span / spine;
        let sp = (d.abs() / span).clamp(0.0, 1.0);
        let sg = if d < 0.0 { -1.0 } else { 1.0 };
        let k = TAU * w.wing_waves * (1.0 - w.flap);
        let a = k * u.clamp(0.0, 1.0) - wave.phase;
        let inner = 1.0 - (wave.bend * sg * 6.0).clamp(-0.6, 0.6);
        let g = wave.amp * inner * sp.max(1e-3).powf(w.wing_expo) * a.sin();
        let tail = (u - 1.0).max(0.0);
        let whip =
            w.whip * tail * (TAU * 1.2 * u - 0.7 * wave.phase).sin() + wave.bend * u * u * 0.5;
        return p + Vec3::new(0.0, g, whip) * spine;
    }
    let (f, df) = pbd_core::fauna::swim::lateral(u, wave, w);
    let slope = -df;
    let c = 1.0 / (1.0 + slope * slope).sqrt();
    let sn = slope * c;
    p + Vec3::new(-d * sn, 0.0, f + d * (c - 1.0)) * spine
}

/// The wave a creature drawn without the shader's attributes swims with:
/// cruising, its beat at two body lengths a second, from the world clock.
pub fn cruising(body: &Body, look: &Look) -> Wave {
    let w = match body.wave_kind() {
        Some(WaveKind::Eel) => WaveProfile::EEL,
        Some(WaveKind::Ray) => WaveProfile::RAY,
        _ => WaveProfile::FISH,
    };
    let hz = (w.beat_rest_hz + w.beat_per_len_hz * 2.0).min(w.beat_max_hz);
    Wave {
        phase: (look.phase + TAU * hz * look.t).rem_euclid(TAU),
        amp: w.amp_cruise,
        bend: 0.0,
    }
}

fn profile_of(body: &Body) -> WaveProfile {
    match body.wave_kind() {
        Some(WaveKind::Eel) => WaveProfile::EEL,
        Some(WaveKind::Ray) => WaveProfile::RAY,
        _ => WaveProfile::FISH,
    }
}

/// Where and how a creature stands: `origin` (its feet or middle, mesh
/// space) and its axes, model +X `forward`, +Y `up`; `scale` metres a model
/// unit.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub origin: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub scale: f32,
}

/// Lay one posed creature into a mesh being built. `tint` multiplies every
/// colour (an octopus flushing or fading to the rock).
pub fn emit(
    body: &Body,
    look: &Look,
    place: &Place,
    tint: [f32; 3],
    positions: &mut Vec<[f32; 3]>,
    colours: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
) {
    let wave = cruising(body, look);
    emit_with(body, look, place, tint, Some(wave), positions, colours, uvs);
}

/// Lay a creature, its base bent on the CPU by `wave` (none: as modelled).
#[allow(clippy::too_many_arguments)]
fn emit_with(
    body: &Body,
    look: &Look,
    place: &Place,
    tint: [f32; 3],
    wave: Option<Wave>,
    positions: &mut Vec<[f32; 3]>,
    colours: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
) {
    let side = place.forward.cross(place.up);
    // A model turned for the pose before it is placed: a dabbler upended,
    // nose down about its own across axis (the mockup's -1.7 rad).
    let pre = if body.kind == Kind::Bird && look.pose == Pose::TippedUp {
        Quat::from_rotation_z(-1.7)
    } else {
        Quat::IDENTITY
    };
    let lift = if body.kind == Kind::Bird && look.pose == Pose::TippedUp {
        0.04
    } else {
        0.0
    };
    let to_world = |m: Vec3| -> [f32; 3] {
        let m = pre * m;
        (place.origin + (place.forward * m.x + place.up * (m.y + lift) + side * m.z) * place.scale)
            .to_array()
    };
    let tinted = |c: [f32; 4]| [c[0] * tint[0], c[1] * tint[1], c[2] * tint[2], 1.0];
    let profile = profile_of(body);
    for (p, c) in body.base.positions.iter().zip(&body.base.colours) {
        let p = wave.map_or(*p, |w| bend(body, *p, w, &profile));
        positions.push(to_world(p));
        colours.push(tinted(*c));
    }
    uvs.extend_from_slice(&body.base.uvs);
    for (index, part) in body.parts.iter().enumerate() {
        let (turn, stretch) = animate(body.kind, part.name, index, part.yaw, look);
        for (p, c) in part.shape.positions.iter().zip(&part.shape.colours) {
            let m = part.at + turn * (*p * stretch);
            positions.push(to_world(m));
            colours.push(tinted(*c));
        }
        uvs.extend_from_slice(&part.shape.uvs);
    }
}

/// The body-wave attributes of a mesh being built, one of each per vertex.
#[derive(Default)]
pub struct SwimAttributes {
    pub axes: Vec<[f32; 4]>,
    pub wave: Vec<[f32; 4]>,
    pub up: Vec<[f32; 4]>,
}

/// Lay one creature into a school's mesh as [`emit`] does, but rigid, with
/// the attributes `swim.wgsl` bends it by: its `wave` (a swimmer's; `None`
/// for one with no body wave, whose attributes are zero and which the
/// shader leaves as laid).
#[allow(clippy::too_many_arguments)]
pub fn emit_swimmer(
    body: &Body,
    look: &Look,
    place: &Place,
    tint: [f32; 3],
    wave: Option<Wave>,
    positions: &mut Vec<[f32; 3]>,
    colours: &mut Vec<[f32; 4]>,
    uvs: &mut Vec<[f32; 2]>,
    swim: &mut SwimAttributes,
) {
    let first = positions.len();
    // The base laid rigid; its moving parts posed on the CPU.
    emit_with(body, look, place, tint, None, positions, colours, uvs);
    let laid = positions.len() - first;
    let base = body.base.positions.len();
    let side = place.forward.cross(place.up);
    let metres = place.scale * body.spine;
    match wave.filter(|_| body.wave_kind().is_some()) {
        Some(w) => {
            let axes = (side * metres).extend(0.0);
            let up = (place.up * metres).extend(body.wave_row as f32);
            for p in &body.base.positions {
                let u = (body.nose - p.x) / body.spine;
                swim.axes.push([axes.x, axes.y, axes.z, u]);
                swim.wave.push([w.phase, w.amp, w.bend, p.z / body.spine]);
                swim.up.push(up.to_array());
            }
            for _ in base..laid {
                swim.axes.push([0.0; 4]);
                swim.wave.push([0.0; 4]);
                swim.up.push([0.0; 4]);
            }
        }
        None => {
            for _ in 0..laid {
                swim.axes.push([0.0; 4]);
                swim.wave.push([0.0; 4]);
                swim.up.push([0.0; 4]);
            }
        }
    }
}
