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
//! plume) hang from pivots and are turned each frame by [`emit`], which also
//! sways a swimmer from its tail and lays the posed creature into a school's
//! mesh. All of it on the CPU, as the herd's rig is (`husbandry/rig.rs`).

use bevy::prelude::*;
use pbd_core::fauna::Pose;
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI, TAU};
use std::sync::OnceLock;

/// A shape: a triangle list and one colour a vertex (the same three times a
/// face).
#[derive(Clone, Default, Debug)]
pub struct Shape {
    pub positions: Vec<Vec3>,
    pub colours: Vec<[f32; 4]>,
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

/// How a swimmer sways (the mockup's vertex shader).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sway {
    None,
    Fish,
    Eel,
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

#[derive(Default)]
pub(crate) struct Kit(pub(crate) Shape);

impl Kit {
    /// Add a triangle list, coloured per face by `colour(centre, face)`.
    pub(crate) fn add_with(
        &mut self,
        tris: Vec<Vec3>,
        m: Mat4,
        colour: impl Fn(Vec3, usize) -> u32,
    ) {
        for (n, tri) in tris.chunks_exact(3).enumerate() {
            let t = [
                m.transform_point3(tri[0]),
                m.transform_point3(tri[1]),
                m.transform_point3(tri[2]),
            ];
            let c = lin(colour((t[0] + t[1] + t[2]) / 3.0, n));
            for p in t {
                self.0.positions.push(p);
                self.0.colours.push(c);
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

pub(crate) fn lathe6(prof: &[[f32; 3]]) -> Vec<Vec3> {
    let p: Vec<[f32; 4]> = prof.iter().map(|r| [r[0], r[1], r[2], 0.0]).collect();
    lathe(&p, 6, PI / 6.0)
}

pub(crate) fn rotate_z(tris: Vec<Vec3>, a: f32) -> Vec<Vec3> {
    let q = Quat::from_rotation_z(a);
    tris.into_iter().map(|p| q * p).collect()
}

pub(crate) fn translate(tris: Vec<Vec3>, by: [f32; 3]) -> Vec<Vec3> {
    tris.into_iter().map(|p| p + Vec3::from(by)).collect()
}

pub(crate) const EYE: u32 = 0x161612;

// ---- fish (after fish.html's buildFish) ------------------------------------

#[derive(Clone, Copy)]
enum Tail {
    Fork,
    Deep,
    Lunate,
    Trunc,
    Round,
}

#[derive(Clone)]
struct FishP {
    back: u32,
    side: u32,
    belly: u32,
    fin: u32,
    tail_c: Option<u32>,
    head: Option<u32>,
    h: f32,
    w: f32,
    tail: Tail,
    dorsal: Option<Vec<[f32; 3]>>,
    bands: Vec<(f32, f32, u32)>,
    lateral: Option<u32>,
    spots: Option<(f32, u32)>,
    eel: bool,
    peak: f32,
}

impl FishP {
    fn new(back: u32, side: u32, belly: u32, fin: u32, h: f32, w: f32, tail: Tail) -> Self {
        Self {
            back,
            side,
            belly,
            fin,
            tail_c: None,
            head: None,
            h,
            w,
            tail,
            dorsal: None,
            bands: Vec::new(),
            lateral: None,
            spots: None,
            eel: false,
            peak: 0.32,
        }
    }
}

fn fish_geo(p: &FishP) -> Shape {
    let mut k = Kit::default();
    let eel = p.eel;
    let nu = if eel { 16 } else { 9 };
    let bl = if eel { 0.98 } else { 0.8 };
    let (hh_, wd) = (p.h / 2.0, p.w / 2.0);
    let x_at = |u: f32| 0.5 - u * bl;
    let u_at = |x: f32| (0.5 - x) / bl;
    let peak = p.peak;
    let prof = |u: f32| -> f32 {
        if eel {
            if u < 0.07 {
                (u / 0.07 * FRAC_PI_2).sin().powf(0.8)
            } else {
                1.0 - 0.8 * ((u - 0.07) / 0.93).powf(1.6)
            }
        } else if u < peak {
            (u / peak * FRAC_PI_2).sin().powf(0.75)
        } else {
            0.17 + 0.83 * (1.0 - (u - peak) / (1.0 - peak)).max(0.0).powf(1.25)
        }
    };
    let mut rings = Vec::new();
    for i in 0..=nu {
        let u = i as f32 / nu as f32;
        let f = if i == 0 { 0.0 } else { prof(u) };
        rings.push([x_at(u), hh_ * f, wd * f, 0.0]);
    }
    rings.push([x_at(1.0) - 0.004, 0.0, 0.0, 0.0]);
    let body = lathe(&rings, 6, PI / 6.0);
    k.add_with(body, Mat4::IDENTITY, |c, n| {
        let u = u_at(c.x).clamp(0.0, 1.0);
        let hy = (hh_ * prof(u)).max(1e-4);
        let yn = c.y / hy;
        for &(u0, u1, colour) in &p.bands {
            if u >= u0 && u <= u1 {
                return colour;
            }
        }
        if let Some((tenths, colour)) = p.spots
            && hh(n as i64, 7) < tenths / 10.0
            && yn > -0.2
        {
            return colour;
        }
        if let Some(l) = p.lateral
            && yn.abs() < 0.22
            && u > 0.15
        {
            return l;
        }
        if let Some(h) = p.head
            && u < 0.22
        {
            return h;
        }
        if yn > 0.3 {
            p.back
        } else if yn < -0.35 {
            p.belly
        } else {
            p.side
        }
    });
    let fin = p.fin;
    let xt = x_at(1.0);
    let hp = (hh_ * prof(1.0)).max(0.012);
    let tl = if eel { 0.06 } else { xt + 0.52 };
    if !eel {
        let tail: Vec<[f32; 2]> = match p.tail {
            Tail::Fork => vec![
                [xt + 0.02, hp],
                [xt - tl, hp + tl * 0.8],
                [xt - tl * 0.55, 0.0],
                [xt - tl, -hp - tl * 0.8],
                [xt + 0.02, -hp],
            ],
            Tail::Deep => vec![
                [xt + 0.02, hp],
                [xt - tl * 1.05, hp + tl],
                [xt - tl * 0.8, hp + tl * 0.7],
                [xt - tl * 0.45, 0.0],
                [xt - tl * 0.8, -hp - tl * 0.7],
                [xt - tl * 1.05, -hp - tl],
                [xt + 0.02, -hp],
            ],
            Tail::Lunate => vec![
                [xt + 0.02, hp],
                [xt - tl, hp + tl * 1.05],
                [xt - tl * 0.55, hp * 0.4],
                [xt - tl * 0.5, 0.0],
                [xt - tl * 0.55, -hp * 0.4],
                [xt - tl, -hp - tl * 1.05],
                [xt + 0.02, -hp],
            ],
            Tail::Trunc => vec![
                [xt + 0.02, hp],
                [xt - tl, hp + tl * 0.55],
                [xt - tl * 1.04, 0.0],
                [xt - tl, -hp - tl * 0.55],
                [xt + 0.02, -hp],
            ],
            Tail::Round => vec![
                [xt + 0.02, hp],
                [xt - tl * 0.6, hp + tl * 0.45],
                [xt - tl, 0.0],
                [xt - tl * 0.6, -hp - tl * 0.45],
                [xt + 0.02, -hp],
            ],
        };
        k.add(shape(&tail), Mat4::IDENTITY, p.tail_c.unwrap_or(fin));
    }
    let fins = |list: &[[f32; 3]], sgn: f32, k: &mut Kit| {
        for &[u0, u1, hgt] in list {
            let base: Vec<[f32; 2]> = (0..=6)
                .map(|i| {
                    let u = u0 + (u1 - u0) * i as f32 / 6.0;
                    [x_at(u), sgn * hh_ * prof(u) * 0.86]
                })
                .collect();
            let mut poly = vec![base[0]];
            poly.push([base[1][0] + hgt * 0.1, base[1][1] + sgn * hgt]);
            poly.push([base[6][0] - hgt * 0.45, base[6][1] + sgn * hgt * 0.3]);
            poly.push(base[6]);
            for i in (1..6).rev() {
                poly.push(base[i]);
            }
            k.add(shape(&poly), Mat4::IDENTITY, fin);
        }
    };
    let dorsal = p.dorsal.clone().unwrap_or_else(|| {
        if eel {
            vec![[0.12, 0.97, 0.03]]
        } else {
            vec![[0.28, 0.62, hh_ * 0.9]]
        }
    });
    fins(&dorsal, 1.0, &mut k);
    let anal = if eel {
        vec![[0.4, 0.97, 0.025]]
    } else {
        vec![[0.6, 0.82, hh_ * 0.5]]
    };
    fins(&anal, -1.0, &mut k);
    if !eel {
        for s in [1.0f32, -1.0] {
            let pw = hh_ * 0.55;
            let g = shape(&[[0.0, 0.0], [-pw, 0.03], [-pw * 0.8, -pw * 0.45]]);
            k.add(
                g,
                atr(
                    [x_at(0.27), -hh_ * 0.25, s * wd * 0.75],
                    [s * 0.9, 0.0, 0.0],
                ),
                fin,
            );
        }
    }
    let eu = if eel { 0.04 } else { 0.13 };
    let (ex, ew) = (x_at(eu), wd * prof(eu));
    for s in [1.0f32, -1.0] {
        k.add(
            bx(0.045, 0.045, 0.03),
            at([ex, hh_ * 0.25, s * (ew + 0.008)]),
            EYE,
        );
    }
    k.build()
}

// ---- crawlers --------------------------------------------------------------

struct CrabP {
    top: u32,
    under: u32,
    leg: u32,
    claw: u32,
}

fn crab_geo(p: &CrabP) -> Shape {
    let mut k = Kit::default();
    let (top, under) = (p.top, p.under);
    k.add_with(ell(0.28, 0.12, 0.36, 8, 4), at([0.0, 0.16, 0.0]), |c, _| {
        if c.y > 0.05 { top } else { under }
    });
    for s in [1.0f32, -1.0] {
        for i in 0..4 {
            let x = 0.12 - i as f32 * 0.1;
            k.add(
                bx(0.04, 0.04, 0.3),
                atr(
                    [x, 0.12, s * 0.42],
                    [s * 0.55, (i as f32 - 1.5) * 0.25 * s, 0.0],
                ),
                p.leg,
            );
            k.add(
                bx(0.035, 0.18, 0.035),
                at([x - 0.01, 0.06, s * 0.56]),
                p.leg,
            );
        }
    }
    for s in [1.0f32, -1.0] {
        k.add(
            bx(0.2, 0.08, 0.08),
            atr([0.32, 0.17, s * 0.22], [0.0, s * 0.5, 0.0]),
            p.claw,
        );
        k.add(bx(0.06, 0.06, 0.06), at([0.22, 0.17, s * 0.22]), p.claw);
    }
    for s in [1.0f32, -1.0] {
        k.add(cyl(0.012, 0.012, 0.12, 4), at([0.24, 0.29, s * 0.07]), EYE);
        k.add(bx(0.04, 0.04, 0.04), at([0.24, 0.36, s * 0.07]), EYE);
    }
    k.build()
}

fn lobster_geo() -> Shape {
    let (top, under, band, claw, ant) = (0xa8633e, 0xe69d69, 0x733126, 0x733126, 0x2e0d06);
    let mut k = Kit::default();
    k.add_with(
        lathe6(&[
            [0.42, 0.0, 0.0],
            [0.36, 0.06, 0.07],
            [0.2, 0.09, 0.09],
            [0.05, 0.09, 0.085],
            [-0.02, 0.07, 0.07],
        ]),
        at([0.0, 0.1, 0.0]),
        |c, _| if c.y > 0.12 { top } else { under },
    );
    for i in 0..5 {
        let (x, r) = (-0.04 - i as f32 * 0.075, 0.065 - i as f32 * 0.007);
        k.add(
            lathe6(&[[x + 0.035, r, r * 1.1], [x - 0.035, r * 0.95, r * 1.05]]),
            at([0.0, 0.1, 0.0]),
            if i % 2 == 1 { top } else { band },
        );
    }
    k.add(
        shape(&[[0.0, 0.0], [-0.12, 0.09], [-0.15, 0.0], [-0.12, -0.09]]),
        atr([-0.41, 0.09, 0.0], [FRAC_PI_2, 0.0, 0.0]),
        top,
    );
    for s in [1.0f32, -1.0] {
        k.add(
            bx(0.18, 0.03, 0.03),
            atr([0.42, 0.11, s * 0.1], [0.0, s * 0.5, 0.0]),
            top,
        );
        k.add(
            ell(0.13, 0.05, 0.065, 6, 3),
            atr([0.6, 0.11, s * 0.2], [0.0, s * 0.3, 0.0]),
            claw,
        );
        k.add(
            bx(0.7, 0.012, 0.012),
            atr([0.75, 0.16, s * 0.18], [0.0, s * 0.35, 0.12]),
            ant,
        );
        for i in 0..4 {
            k.add(
                bx(0.025, 0.12, 0.025),
                atr([0.25 - i as f32 * 0.07, 0.05, s * 0.1], [s * 0.6, 0.0, 0.0]),
                under,
            );
        }
    }
    for s in [1.0f32, -1.0] {
        k.add(bx(0.03, 0.03, 0.03), at([0.4, 0.16, s * 0.04]), EYE);
    }
    k.build()
}

/// A giant clam: the lower shell and the mantle, its upper shell a part on
/// the hinge (the mockup's `makeClam`, its colours read off `clam.png`).
fn clam_geo() -> (Shape, Shape) {
    let (shell, ridge, lip, mantle, spot) = (0xa09f91, 0xcfaa81, 0xeab798, 0x1096c3, 0x74effe);
    let mut k = Kit::default();
    k.add_with(ell(0.5, 0.16, 0.36, 8, 4), at([0.0, 0.12, 0.0]), |c, n| {
        if c.y > 0.2 {
            lip
        } else if n % 3 == 0 {
            ridge
        } else {
            shell
        }
    });
    k.add_with(bx(0.85, 0.06, 0.5), at([0.0, 0.25, 0.0]), |_, n| {
        if hh(n as i64, 5) < 0.3 { spot } else { mantle }
    });
    let lower = k.build();
    let mut k = Kit::default();
    k.add_with(ell(0.5, 0.14, 0.36, 8, 4), at([0.0, 0.0, -0.36]), |c, n| {
        if c.y < -0.06 {
            lip
        } else if n % 3 == 0 {
            ridge
        } else {
            shell
        }
    });
    (lower, k.build())
}

fn turtle_geo() -> (Shape, Vec<Part>) {
    let (shell, scute, under, skin) = (0x725244, 0xab8465, 0xcfaa81, 0x297537);
    let mut k = Kit::default();
    k.add_with(ell(0.36, 0.13, 0.3, 8, 4), at([0.0, 0.02, 0.0]), |c, n| {
        if c.y < 0.02 {
            under
        } else if n % 5 == 0 {
            scute
        } else {
            shell
        }
    });
    k.add(ell(0.36, 0.03, 0.3, 8, 2), at([0.0, -0.01, 0.0]), under);
    k.add(ell(0.1, 0.07, 0.07, 6, 4), at([0.42, 0.02, 0.0]), skin);
    for s in [1.0f32, -1.0] {
        k.add(bx(0.03, 0.03, 0.02), at([0.46, 0.05, s * 0.055]), EYE);
    }
    k.add(
        rotate_z(cyl(0.0, 0.04, 0.12, 4), FRAC_PI_2),
        at([-0.38, 0.0, 0.0]),
        skin,
    );
    let flipper = |front: bool, s: f32| {
        let mut k = Kit::default();
        let (w, d) = if front { (0.14, 0.36) } else { (0.1, 0.16) };
        let g = translate(bx(w, 0.02, d), [0.0, 0.0, d / 2.0])
            .into_iter()
            .map(|p| p * Vec3::new(1.0, 1.0, s))
            .collect();
        k.add(g, Mat4::IDENTITY, skin);
        k.build()
    };
    let parts = vec![
        Part {
            name: "fr",
            at: Vec3::new(0.18, 0.0, 0.22),
            yaw: 0.0,
            shape: flipper(true, 1.0),
        },
        Part {
            name: "fl",
            at: Vec3::new(0.18, 0.0, -0.22),
            yaw: 0.0,
            shape: flipper(true, -1.0),
        },
        Part {
            name: "br",
            at: Vec3::new(-0.22, 0.0, 0.22),
            yaw: 0.0,
            shape: flipper(false, 1.0),
        },
        Part {
            name: "bl",
            at: Vec3::new(-0.22, 0.0, -0.22),
            yaw: 0.0,
            shape: flipper(false, -1.0),
        },
    ];
    (k.build(), parts)
}

fn ray_geo() -> (Shape, Vec<Part>) {
    let (top, under, spot) = (0x8c7a58, 0xd8cfb0, 0xb9a57d);
    let mut k = Kit::default();
    k.add_with(
        lathe6(&[
            [0.42, 0.0, 0.0],
            [0.3, 0.04, 0.14],
            [0.05, 0.06, 0.2],
            [-0.15, 0.05, 0.14],
            [-0.3, 0.02, 0.05],
            [-0.33, 0.0, 0.0],
        ]),
        Mat4::IDENTITY,
        |c, _| if c.y > 0.0 { top } else { under },
    );
    k.add(
        rotate_z(cyl(0.004, 0.018, 0.5, 4), FRAC_PI_2),
        at([-0.55, 0.0, 0.0]),
        top,
    );
    for s in [1.0f32, -1.0] {
        k.add(bx(0.04, 0.03, 0.03), at([0.2, 0.05, s * 0.07]), EYE);
    }
    let wing = |s: f32| {
        let mut k = Kit::default();
        k.add_with(
            shape(&[[0.32, 0.0], [-0.25, 0.0], [-0.02, s * 0.5]]),
            atr([0.0; 3], [FRAC_PI_2, 0.0, 0.0]),
            |_, n| if hh(n as i64, 9) < 0.2 { spot } else { top },
        );
        k.build()
    };
    let parts = vec![
        Part {
            name: "wingR",
            at: Vec3::new(0.0, 0.0, 0.1),
            yaw: 0.0,
            shape: wing(1.0),
        },
        Part {
            name: "wingL",
            at: Vec3::new(0.0, 0.0, -0.1),
            yaw: 0.0,
            shape: wing(-1.0),
        },
    ];
    (k.build(), parts)
}

fn octopus_geo() -> (Shape, Vec<Part>) {
    let (body, spot, eye, sucker) = (0xd4462b, 0x841c1d, 0xeab798, 0xe69d69);
    let mut k = Kit::default();
    k.add_with(
        ell(0.22, 0.2, 0.2, 7, 5),
        atr([-0.12, 0.26, 0.0], [0.0, 0.0, 0.4]),
        |_, n| if hh(n as i64, 6) < 0.25 { spot } else { body },
    );
    k.add(ell(0.13, 0.11, 0.15, 7, 4), at([0.06, 0.12, 0.0]), body);
    for s in [1.0f32, -1.0] {
        k.add(bx(0.06, 0.05, 0.05), at([0.12, 0.19, s * 0.1]), eye);
        k.add(bx(0.07, 0.015, 0.015), at([0.13, 0.19, s * 0.125]), EYE);
    }
    let arm = || {
        let mut k = Kit::default();
        k.add_with(
            translate(
                rotate_z(cyl(0.012, 0.045, 0.5, 4), -FRAC_PI_2),
                [0.25, 0.0, 0.0],
            ),
            Mat4::IDENTITY,
            |c, _| if c.y < 0.0 { sucker } else { body },
        );
        k.build()
    };
    let parts = (0..8)
        .map(|i| Part {
            name: "arm",
            at: Vec3::new(0.05, 0.08, 0.0),
            yaw: i as f32 * PI / 4.0 + PI / 8.0,
            shape: arm(),
        })
        .collect();
    (k.build(), parts)
}

fn seal_geo() -> Shape {
    let (back, side, belly, spot) = (0x3e586e, 0x5d778d, 0x92a3b2, 0x1f3b52);
    let mut k = Kit::default();
    k.add_with(
        lathe6(&[
            [0.5, 0.0, 0.0],
            [0.44, 0.08, 0.08],
            [0.36, 0.1, 0.1],
            [0.3, 0.09, 0.09],
            [0.15, 0.13, 0.14],
            [-0.05, 0.14, 0.15],
            [-0.25, 0.1, 0.11],
            [-0.38, 0.05, 0.05],
            [-0.42, 0.0, 0.0],
        ]),
        at([0.0, 0.13, 0.0]),
        |c, n| {
            let y = c.y - 0.13;
            if y > 0.06 {
                if hh(n as i64, 4) < 0.2 { spot } else { back }
            } else if y < -0.05 {
                belly
            } else {
                side
            }
        },
    );
    for s in [1.0f32, -1.0] {
        k.add(bx(0.035, 0.035, 0.03), at([0.43, 0.2, s * 0.06]), EYE);
        k.add(
            bx(0.18, 0.025, 0.08),
            atr([0.15, 0.04, s * 0.14], [0.0, s * 0.6, 0.0]),
            back,
        );
        k.add(
            shape(&[[0.0, 0.0], [-0.16, 0.0], [-0.18, s * 0.12]]),
            atr([-0.38, 0.13, 0.0], [FRAC_PI_2, 0.0, 0.0]),
            back,
        );
    }
    k.add(bx(0.03, 0.02, 0.03), at([0.5, 0.15, 0.0]), EYE);
    k.build()
}

/// A sea otter rafting on its back, paws on its chest round an urchin.
fn otter_geo() -> Shape {
    let (back, belly) = (0x522b2f, 0xcfaa81);
    let mut k = Kit::default();
    k.add_with(
        lathe6(&[
            [0.5, 0.0, 0.0],
            [0.45, 0.06, 0.06],
            [0.36, 0.075, 0.075],
            [0.3, 0.065, 0.065],
            [0.12, 0.1, 0.11],
            [-0.12, 0.1, 0.1],
            [-0.26, 0.07, 0.07],
            [-0.3, 0.0, 0.0],
        ]),
        at([0.0, 0.03, 0.0]),
        |c, _| if c.y < 0.05 { back } else { belly },
    );
    k.add(
        rotate_z(cyl(0.01, 0.045, 0.3, 5), FRAC_PI_2),
        at([-0.42, 0.03, 0.0]),
        back,
    );
    for s in [1.0f32, -1.0] {
        k.add(bx(0.03, 0.03, 0.03), at([0.42, 0.075, s * 0.04]), EYE);
        k.add(bx(0.06, 0.03, 0.04), at([0.36, 0.0, s * 0.06]), back);
    }
    k.add(bx(0.03, 0.03, 0.04), at([0.5, 0.05, 0.0]), EYE);
    for s in [1.0f32, -1.0] {
        k.add(
            bx(0.12, 0.03, 0.03),
            atr([0.2, 0.13, s * 0.05], [0.0, s * 0.3, 0.3]),
            back,
        );
    }
    k.add(icosahedron(0.05), at([0.24, 0.16, 0.0]), 0x7d2dbd);
    for s in [1.0f32, -1.0] {
        k.add(
            bx(0.12, 0.03, 0.04),
            atr([-0.24, 0.08, s * 0.06], [0.0, 0.0, -0.4]),
            back,
        );
    }
    k.build()
}

/// A plume worm: a white tube standing up (+Y, one unit tall) and its red
/// plume, a part that draws in.
fn worm_geo() -> (Shape, Vec<Part>) {
    let (tube, tube2, rim) = (0xbfcad7, 0x92a3b2, 0xfffefe);
    let (plume, plume2, plume3) = (0xd12d41, 0xfa3a64, 0xa61e1f);
    let mut k = Kit::default();
    k.add_with(cyl(0.05, 0.065, 1.0, 6), at([0.0, 0.5, 0.0]), |c, n| {
        if c.y > 0.95 {
            rim
        } else if n % 4 == 0 {
            tube2
        } else {
            tube
        }
    });
    let mut p = Kit::default();
    p.add_with(cyl(0.13, 0.05, 0.22, 6), at([0.0, 0.11, 0.0]), |c, n| {
        if c.y > 0.2 {
            plume2
        } else if n % 3 == 0 {
            plume3
        } else {
            plume
        }
    });
    let parts = vec![Part {
        name: "plume",
        at: Vec3::new(0.0, 0.98, 0.0),
        yaw: 0.0,
        shape: p.build(),
    }];
    (k.build(), parts)
}

// ---- birds -------------------------------------------------------------------

#[derive(Clone, Default)]
struct BirdP {
    back: u32,
    side: u32,
    belly: u32,
    breast: Option<u32>,
    flank: Option<u32>,
    tail_c: u32,
    curl: Option<u32>,
    head_c: u32,
    ring: Option<u32>,
    neck_c: u32,
    bill_c: u32,
    wing_c: u32,
    spec: Option<u32>,
    leg_c: u32,
    crest: Option<u32>,
    cheek: Option<u32>,
}

fn bird_geo(p: &BirdP) -> (Shape, Vec<Part>) {
    let by = 0.12;
    let mut k = Kit::default();
    let prof = [
        [0.36, 0.0, 0.0],
        [0.32, 0.1, 0.1],
        [0.2, 0.15, 0.15],
        [0.0, 0.15, 0.16],
        [-0.2, 0.12, 0.12],
        [-0.34, 0.07, 0.06],
        [-0.4, 0.0, 0.0],
    ];
    let pr: Vec<[f32; 4]> = prof.iter().map(|r| [r[0], r[1], r[2], 0.0]).collect();
    k.add_with(lathe(&pr, 7, PI / 6.0), at([0.0, by, 0.0]), |c, _| {
        let (x, y, z) = (c.x, c.y - by, c.z);
        if let Some(b) = p.breast
            && x > 0.12
            && y > -0.08
        {
            return b;
        }
        if let Some(f) = p.flank
            && z.abs() > 0.09
            && y < 0.07
            && y > -0.08
            && x < 0.15
        {
            return f;
        }
        if y > 0.06 {
            p.back
        } else if y < -0.06 {
            p.belly
        } else {
            p.side
        }
    });
    k.add(
        shape(&[[0.0, 0.03], [-0.16, 0.08], [-0.16, -0.05], [0.0, -0.03]]),
        atr([-0.38, by + 0.03, 0.0], [FRAC_PI_2, 0.0, 0.25]),
        p.tail_c,
    );
    if let Some(curl) = p.curl {
        k.add(bx(0.04, 0.06, 0.02), at([-0.42, by + 0.12, 0.0]), curl);
    }
    // The head and neck, pivoted at the neck's base.
    let (nl, hr, lean) = (0.14f32, 0.1f32, -0.25f32);
    let mut h = Kit::default();
    h.add(
        cyl(0.05, 0.065, nl, 6),
        atr([0.0, nl / 2.0, 0.0], [0.0, 0.0, lean]),
        p.neck_c,
    );
    let (hx, hy) = ((-lean).sin() * nl, lean.cos() * nl);
    h.add(
        ell(hr * 1.1, hr * 0.9, hr * 0.85, 7, 4),
        at([hx + 0.02, hy + 0.02, 0.0]),
        p.head_c,
    );
    if let Some(ring) = p.ring {
        h.add(
            cyl(0.068, 0.068, 0.025, 6),
            at([hx * 0.2, nl * 0.2, 0.0]),
            ring,
        );
    }
    let bl = 0.12;
    h.add(
        bx(bl, 0.03, 0.06),
        atr(
            [hx + 0.02 + hr + bl / 2.0 - 0.02, hy, 0.0],
            [0.0, 0.0, -0.1],
        ),
        p.bill_c,
    );
    for s in [1.0f32, -1.0] {
        h.add(
            bx(0.025, 0.025, 0.02),
            at([hx + 0.02 + hr * 0.45, hy + 0.04, s * hr * 0.8]),
            EYE,
        );
    }
    if let Some(crest) = p.crest {
        h.add(
            bx(0.18, 0.015, 0.015),
            atr([hx - 0.1, hy + 0.05, 0.0], [0.0, 0.0, 0.2]),
            crest,
        );
    }
    if let Some(cheek) = p.cheek {
        for s in [1.0f32, -1.0] {
            h.add(
                bx(0.05, 0.04, 0.01),
                at([hx + 0.03, hy - 0.01, s * hr * 0.85]),
                cheek,
            );
        }
    }
    let wing = |s: f32| {
        let (sp, ch) = (0.34f32, 0.2f32);
        let mut w = Kit::default();
        w.add_with(
            shape(&[
                [0.04, 0.0],
                [-ch, 0.0],
                [-ch * 1.1, sp * 0.6],
                [-ch * 0.7, sp],
                [0.0, sp * 0.55],
            ]),
            atr([0.0; 3], [s * FRAC_PI_2, 0.0, 0.0]),
            |c, _| match p.spec {
                Some(spec) if c.x < -ch * 0.55 && c.z.abs() < sp * 0.5 => spec,
                _ => p.wing_c,
            },
        );
        w.build()
    };
    let foot = || {
        let mut f = Kit::default();
        f.add(
            translate(bx(0.02, 0.08, 0.02), [0.0, -0.04, 0.0]),
            Mat4::IDENTITY,
            p.leg_c,
        );
        f.add(
            translate(bx(0.08, 0.012, 0.06), [0.02, -0.08, 0.0]),
            Mat4::IDENTITY,
            p.leg_c,
        );
        f.build()
    };
    let parts = vec![
        Part {
            name: "head",
            at: Vec3::new(0.28, by + 0.06, 0.0),
            yaw: 0.0,
            shape: h.build(),
        },
        Part {
            name: "wingR",
            at: Vec3::new(0.12, by + 0.1, 0.12),
            yaw: 0.0,
            shape: wing(1.0),
        },
        Part {
            name: "wingL",
            at: Vec3::new(0.12, by + 0.1, -0.12),
            yaw: 0.0,
            shape: wing(-1.0),
        },
        Part {
            name: "footR",
            at: Vec3::new(-0.04, by - 0.05, 0.06),
            yaw: 0.0,
            shape: foot(),
        },
        Part {
            name: "footL",
            at: Vec3::new(-0.04, by - 0.05, -0.06),
            yaw: 0.0,
            shape: foot(),
        },
    ];
    (k.build(), parts)
}

// ---- the species ------------------------------------------------------------

fn fish_body(p: FishP) -> Body {
    let sway = if p.eel { Sway::Eel } else { Sway::Fish };
    Body::new(Kind::Fish, sway, fish_geo(&p), Vec::new())
}

impl Body {
    fn new(kind: Kind, sway: Sway, base: Shape, parts: Vec<Part>) -> Self {
        let mut body = Self {
            kind,
            sway,
            base,
            parts,
            length: 1.0,
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
        body
    }
}

fn build(model: &str) -> Body {
    use Tail::*;
    match model {
        // The reef's ambient fish (`places`): not species, never caught.
        "darter" => fish_body(FishP {
            tail_c: Some(0x8d4526),
            bands: vec![
                (0.13, 0.19, 0xfffefe),
                (0.42, 0.5, 0xfffefe),
                (0.74, 0.8, 0xfffefe),
            ],
            dorsal: Some(vec![[0.22, 0.7, 0.12]]),
            ..FishP::new(0xc56d25, 0xfb902f, 0xfaaf5c, 0xfb902f, 0.36, 0.14, Round)
        }),
        "anthias" => fish_body(FishP {
            tail_c: Some(0xdd576d),
            lateral: Some(0xfaaf5c),
            ..FishP::new(0xdd576d, 0xfb902f, 0xfaaf5c, 0xfb902f, 0.3, 0.1, Lunate)
        }),
        "minnow" => fish_body(FishP {
            lateral: Some(0x3c4c5c),
            ..FishP::new(0x54697e, 0x7896b4, 0xbcd0e4, 0x7896b4, 0.2, 0.1, Fork)
        }),
        "silverfin" => fish_body(FishP::new(
            0x9fb9c2, 0xcfe3ea, 0xf0f6f8, 0x9fb9c2, 0.22, 0.11, Deep,
        )),
        "perch" => fish_body(FishP {
            bands: vec![
                (0.22, 0.28, 0x6b4a18),
                (0.4, 0.46, 0x6b4a18),
                (0.58, 0.64, 0x6b4a18),
            ],
            dorsal: Some(vec![[0.22, 0.45, 0.13], [0.5, 0.7, 0.09]]),
            ..FishP::new(0x6b4a18, 0xd8a13a, 0xe8c878, 0xc8612b, 0.3, 0.13, Trunc)
        }),
        "eel" => fish_body(FishP {
            eel: true,
            ..FishP::new(0x4c562e, 0x788248, 0xa8b070, 0x4c562e, 0.07, 0.06, Fork)
        }),
        "serpent" => fish_body(FishP {
            eel: true,
            dorsal: Some(vec![[0.08, 0.98, 0.04]]),
            ..FishP::new(0x1e5042, 0x307866, 0xb6d0b2, 0x307866, 0.06, 0.05, Fork)
        }),
        "reef" => fish_body(FishP::new(
            0x246064, 0x3c9696, 0x96d6d6, 0x246064, 0.38, 0.1, Lunate,
        )),
        "deepback" => fish_body(FishP {
            dorsal: Some(vec![[0.25, 0.7, 0.16]]),
            ..FishP::new(0x4f6f8f, 0x6f8faf, 0x9fb6c9, 0x4f6f8f, 0.3, 0.14, Trunc)
        }),
        "ray" => {
            let (base, parts) = ray_geo();
            Body::new(Kind::Ray, Sway::None, base, parts)
        }
        "shore_crab" => Body::new(
            Kind::Crab,
            Sway::None,
            crab_geo(&CrabP {
                top: 0xa61e1f,
                under: 0xd4462b,
                leg: 0xfe6420,
                claw: 0x4a051e,
            }),
            Vec::new(),
        ),
        "lobster" => Body::new(Kind::Lobster, Sway::None, lobster_geo(), Vec::new()),
        "giant_clam" => {
            let (lower, upper) = clam_geo();
            let lid = Part {
                name: "lid",
                at: Vec3::new(0.0, 0.2, 0.36),
                yaw: 0.0,
                shape: upper,
            };
            Body::new(Kind::Clam, Sway::None, lower, vec![lid])
        }
        "reef_turtle" => {
            let (base, parts) = turtle_geo();
            Body::new(Kind::Turtle, Sway::None, base, parts)
        }
        "sea_otter" => Body::new(Kind::Otter, Sway::None, otter_geo(), Vec::new()),
        "seal" => Body::new(Kind::Seal, Sway::Fish, seal_geo(), Vec::new()),
        "octopus" => {
            let (base, parts) = octopus_geo();
            Body::new(Kind::Octopus, Sway::None, base, parts)
        }
        "tube_worm" => {
            let (base, parts) = worm_geo();
            Body::new(Kind::TubeWorm, Sway::None, base, parts)
        }
        "dabbler_drake" => bird(BirdP {
            back: 0x434137,
            side: 0xbfcad7,
            belly: 0xa09f91,
            breast: Some(0xab8465),
            flank: Some(0xbfcad7),
            tail_c: 0x2c2c25,
            curl: Some(0x2c2c25),
            head_c: 0x297537,
            ring: Some(0xfffefe),
            neck_c: 0x297537,
            bill_c: 0xfaaf5c,
            wing_c: 0x434137,
            spec: Some(0x1096c3),
            leg_c: 0xfb902f,
            ..default()
        }),
        "dabbler_hen" => bird(BirdP {
            back: 0x725244,
            side: 0xab8465,
            belly: 0xcfaa81,
            tail_c: 0x725244,
            head_c: 0xab8465,
            neck_c: 0xab8465,
            cheek: Some(0x2e0d06),
            bill_c: 0xfb902f,
            wing_c: 0x725244,
            spec: Some(0x1096c3),
            leg_c: 0xfb902f,
            ..default()
        }),
        "pond_diver" => bird(BirdP {
            back: 0x2c2c25,
            side: 0xbfcad7,
            belly: 0xfffefe,
            flank: Some(0xfffefe),
            tail_c: 0x161612,
            head_c: 0x434137,
            neck_c: 0x2c2c25,
            bill_c: 0x92a3b2,
            wing_c: 0x2c2c25,
            spec: Some(0xfffefe),
            leg_c: 0x92a3b2,
            crest: Some(0x161612),
            ..default()
        }),
        _ => fish_body(FishP {
            lateral: Some(0x3c4c5c),
            ..FishP::new(0x54697e, 0x7896b4, 0xbcd0e4, 0x7896b4, 0.2, 0.1, Fork)
        }),
    }
}

fn bird(p: BirdP) -> Body {
    let (base, parts) = bird_geo(&p);
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

/// The model a species' creature `id` wears: its own, the hen for every
/// second reed dabbler, and the minnow's for a species with none.
pub fn model_for(species: &str, id: u32) -> &'static Body {
    static BODIES: OnceLock<HashMap<&'static str, Body>> = OnceLock::new();
    let bodies = BODIES.get_or_init(|| MODELS.iter().map(|m| (*m, build(m))).collect());
    let name = match species {
        "reed_dabbler" if id % 2 == 1 => "dabbler_hen",
        "reed_dabbler" => "dabbler_drake",
        other => other,
    };
    bodies.get(name).unwrap_or_else(|| &bodies["minnow"])
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

/// The sway of a swimmer's vertex, model units across (+Z).
fn sway(kind: Sway, x: f32, look: &Look) -> f32 {
    match kind {
        Sway::None => 0.0,
        Sway::Fish => {
            let w = ((0.3 - x) / 0.8).clamp(0.0, 1.0);
            (look.t * 7.0 + look.phase - x * 6.0).sin() * 0.075 * w * w
        }
        Sway::Eel => {
            let w = ((0.55 - x) / 1.0).clamp(0.15, 1.0);
            (look.t * 3.5 + look.phase - x * 13.0).sin() * 0.06 * w
        }
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
    for (p, c) in body.base.positions.iter().zip(&body.base.colours) {
        let m = Vec3::new(p.x, p.y, p.z + sway(body.sway, p.x, look));
        positions.push(to_world(m));
        colours.push(tinted(*c));
    }
    for (index, part) in body.parts.iter().enumerate() {
        let (turn, stretch) = animate(body.kind, part.name, index, part.yaw, look);
        for (p, c) in part.shape.positions.iter().zip(&part.shape.colours) {
            let m = part.at + turn * (*p * stretch);
            positions.push(to_world(m));
            colours.push(tinted(*c));
        }
    }
}
