//! Lofted forms: a section swept through rings (`land-transport` design 12,
//! `animal-husbandry` design 16).
//!
//! Two kinds, by what the surface is:
//!
//! - a **smooth** loft shares its points round each ring and along the
//!   loft, its normals averaged from the faces about them, so the light
//!   rolls round a barrel, a neck, a leg or a hub. Its ends are cut flat,
//!   with hard normals: a hoof's sole, a timber's end grain;
//! - a **hard** loft keeps a normal per face: a chamfered timber, whose
//!   edges are real creases.
//!
//! Every loft carries UVs. On the animals' `coat` strip a smooth loft lays
//! its tile once round the section and repeats it along the part at square
//! texels; on a settlement texture it lays the texture by metres at the
//! manifest's `repeat_m`, so grain runs along every piece.

use super::rig::Builder;
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Tiles of the `coat` strip (`tools/art/coat_texture.py`), in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Hair = 0,
    Mane = 1,
    Hoof = 2,
    Feather = 3,
    Fleece = 4,
    Skin = 5,
}

/// Tiles in the strip.
pub const TILES: f32 = 6.0;

/// How a loft lays its texture.
#[derive(Clone, Copy, Debug)]
pub enum Skin {
    /// A tile of the coat strip, once round, repeated along at square
    /// texels.
    Coat(Tile),
    /// A settlement texture, by metres at its `repeat_m`.
    Metres(f32),
}

/// A ring of a loft: a centre, two axes across it and its half sizes along
/// them.
#[derive(Clone, Copy, Debug)]
pub struct Ring {
    pub centre: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub ru: f32,
    pub rv: f32,
}

impl Ring {
    pub fn at(&self, x: f32, y: f32) -> Vec3 {
        self.centre + self.u * (x * self.ru) + self.v * (y * self.rv)
    }
}

/// `n` sides round, starting on the ring's `u` axis and going toward `v`.
pub fn round(n: usize) -> Vec<(f32, f32)> {
    (0..n)
        .map(|k| {
            let a = TAU * k as f32 / n as f32;
            (a.cos(), a.sin())
        })
        .collect()
}

/// A chamfered square: a timber's section, its corners cut at a quarter.
pub const TIMBER: [(f32, f32); 8] = [
    (1.0, 0.55),
    (0.55, 1.0),
    (-0.55, 1.0),
    (-1.0, 0.55),
    (-1.0, -0.55),
    (-0.55, -1.0),
    (0.55, -1.0),
    (1.0, -0.55),
];

/// sRGB colour to the linear vertex colour the builder writes.
pub fn lin(c: [f32; 3]) -> [f32; 4] {
    let [r, g, b, _] = Color::srgb(c[0], c[1], c[2]).to_linear().to_f32_array();
    [r, g, b, 1.0]
}

/// The rings through `rings` on a Catmull-Rom curve, `per` between each
/// pair, so a back, a neck or a belly bends smoothly between the authored
/// rings. The axes are interpolated and squared to the curve's run.
pub fn curve(rings: &[Ring], per: usize) -> Vec<Ring> {
    if rings.len() < 3 || per < 2 {
        return rings.to_vec();
    }
    let n = rings.len();
    let get = |i: isize| rings[i.clamp(0, n as isize - 1) as usize];
    let cr = |a: f32, b: f32, c: f32, d: f32, t: f32| {
        let t2 = t * t;
        let t3 = t2 * t;
        0.5 * ((2.0 * b)
            + (-a + c) * t
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
            + (-a + 3.0 * b - 3.0 * c + d) * t3)
    };
    let crv = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, t: f32| {
        Vec3::new(
            cr(a.x, b.x, c.x, d.x, t),
            cr(a.y, b.y, c.y, d.y, t),
            cr(a.z, b.z, c.z, d.z, t),
        )
    };
    let mut out = Vec::new();
    for i in 0..n - 1 {
        let (a, b, c, d) = (
            get(i as isize - 1),
            get(i as isize),
            get(i as isize + 1),
            get(i as isize + 2),
        );
        for s in 0..per {
            let t = s as f32 / per as f32;
            out.push(Ring {
                centre: crv(a.centre, b.centre, c.centre, d.centre, t),
                u: b.u.lerp(c.u, t).normalize_or(b.u),
                v: b.v.lerp(c.v, t).normalize_or(b.v),
                ru: cr(a.ru, b.ru, c.ru, d.ru, t).max(b.ru.min(c.ru) * 0.5),
                rv: cr(a.rv, b.rv, c.rv, d.rv, t).max(b.rv.min(c.rv) * 0.5),
            });
        }
    }
    out.push(rings[n - 1]);
    out
}

fn uv_of(skin: Skin, around: f32, along: f32, perimeter: f32) -> [f32; 2] {
    match skin {
        // Inset half a texel each side, so nearest sampling never takes
        // the neighbouring tile's edge.
        Skin::Coat(tile) => [
            (tile as u8 as f32 * 32.0 + 0.5 + around.clamp(0.0, 1.0) * 31.0) / (32.0 * TILES),
            along / perimeter.max(1e-3),
        ],
        Skin::Metres(m) => [around * perimeter / m, along / m],
    }
}

/// A smooth loft of `sides` round through `rings`, its ends capped flat.
/// `colour` gives each point's sRGB colour from its smooth normal, its
/// share along the loft and its share round (0 on `u`, 0.25 on `v`).
pub fn smooth(
    out: &mut Builder,
    rings: &[Ring],
    sides: usize,
    skin: Skin,
    colour: &dyn Fn(Vec3, f32, f32) -> [f32; 3],
) {
    if rings.len() < 2 || sides < 3 {
        return;
    }
    out.textured = true;
    let profile = round(sides);
    let rows = rings.len();
    let cols = sides + 1;
    let pts: Vec<Vec<Vec3>> = rings
        .iter()
        .map(|r| {
            (0..cols)
                .map(|k| {
                    let (x, y) = profile[k % sides];
                    r.at(x, y)
                })
                .collect()
        })
        .collect();
    // Outward: from the loft's middle ring centre toward the surface.
    let mut normals = vec![vec![Vec3::ZERO; sides]; rows];
    let mut sign = 0.0f32;
    for i in 0..rows - 1 {
        for k in 0..sides {
            let (a, b, c, d) = (pts[i][k], pts[i][k + 1], pts[i + 1][k + 1], pts[i + 1][k]);
            let n = (b - a).cross(d - a) + (c - b).cross(d - b);
            if sign == 0.0 && n.length_squared() > 1e-12 {
                let mid = (rings[i].centre + rings[i + 1].centre) * 0.5;
                sign = if n.dot((a + b + c + d) * 0.25 - mid) >= 0.0 {
                    1.0
                } else {
                    -1.0
                };
            }
            for (ii, kk) in [(i, k), (i, k + 1), (i + 1, k + 1), (i + 1, k)] {
                normals[ii][kk % sides] += n;
            }
        }
    }
    let sign = if sign == 0.0 { 1.0 } else { sign };
    // Lengths along and the mean perimeter, for the texture.
    let mut along = vec![0.0f32; rows];
    for i in 1..rows {
        along[i] = along[i - 1] + rings[i].centre.distance(rings[i - 1].centre);
    }
    let total = along[rows - 1].max(1e-4);
    let perimeter = rings
        .iter()
        .map(|r| {
            let (a, b) = (r.ru.abs(), r.rv.abs());
            // Ramanujan's approximation.
            std::f32::consts::PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt())
        })
        .sum::<f32>()
        / rows as f32;
    let base = out.positions.len() as u32;
    for i in 0..rows {
        for k in 0..cols {
            let p = pts[i][k];
            let n = (normals[i][k % sides] * sign)
                .normalize_or((p - rings[i].centre).normalize_or(Vec3::Y));
            let share = k as f32 / sides as f32;
            out.positions.push(p.to_array());
            out.normals.push(n.to_array());
            out.colours.push(lin(colour(n, along[i] / total, share)));
            out.uvs.push(uv_of(skin, share, along[i], perimeter));
        }
    }
    for i in 0..rows - 1 {
        for k in 0..sides {
            let a = base + (i * cols + k) as u32;
            let b = a + 1;
            let d = base + ((i + 1) * cols + k) as u32;
            let c = d + 1;
            if sign > 0.0 {
                out.indices.extend_from_slice(&[a, b, c, a, c, d]);
            } else {
                out.indices.extend_from_slice(&[a, d, c, a, c, b]);
            }
        }
    }
    // The ends, flat.
    for (i, toward) in [(0usize, 1usize), (rows - 1, rows - 2)] {
        let r = rings[i];
        let n = (r.centre - rings[toward].centre).normalize_or(Vec3::Y);
        if r.ru.abs() * r.rv.abs() < 1e-8 {
            continue;
        }
        let c = lin(colour(n, along[i] / total, 0.75));
        let ring: Vec<Vec3> = pts[i][..sides].to_vec();
        for k in 0..sides {
            let (p, q) = (ring[k], ring[(k + 1) % sides]);
            push_tri(
                out,
                [r.centre, p, q],
                n,
                c,
                [
                    uv_of(skin, 0.5, along[i], perimeter),
                    uv_of(skin, k as f32 / sides as f32, along[i] + 0.01, perimeter),
                    uv_of(
                        skin,
                        (k + 1) as f32 / sides as f32,
                        along[i] + 0.01,
                        perimeter,
                    ),
                ],
            );
        }
    }
}

/// One flat triangle facing `n`.
fn push_tri(out: &mut Builder, p: [Vec3; 3], n: Vec3, colour: [f32; 4], uv: [[f32; 2]; 3]) {
    let face = (p[1] - p[0]).cross(p[2] - p[0]);
    let (order, uvs) = if face.dot(n) >= 0.0 {
        ([p[0], p[1], p[2]], [uv[0], uv[1], uv[2]])
    } else {
        ([p[0], p[2], p[1]], [uv[0], uv[2], uv[1]])
    };
    let base = out.positions.len() as u32;
    for (q, t) in order.into_iter().zip(uvs) {
        out.positions.push(q.to_array());
        out.normals.push(n.to_array());
        out.colours.push(colour);
        out.uvs.push(t);
    }
    out.indices.extend_from_slice(&[base, base + 1, base + 2]);
}

/// A hard loft of `profile` through `rings` (joined last to first when
/// `closed`), a normal per face, its ends capped. `pick` chooses each face's
/// builder among `outs` and its sRGB colour from its outward normal; every
/// builder lays its texture by metres at `repeat_m`.
pub fn hard(
    outs: &mut [Builder],
    repeat_m: f32,
    rings: &[Ring],
    profile: &[(f32, f32)],
    closed: bool,
    pick: &dyn Fn(Vec3) -> (usize, [f32; 3]),
) {
    if rings.len() < 2 || outs.is_empty() {
        return;
    }
    for o in outs.iter_mut() {
        o.textured = true;
    }
    let m = repeat_m.max(0.05);
    let pts: Vec<Vec<Vec3>> = rings
        .iter()
        .map(|r| profile.iter().map(|&(x, y)| r.at(x, y)).collect())
        .collect();
    let n = profile.len();
    // Each profile edge's start, as metres round the first ring.
    let mut round_at = vec![0.0f32; n + 1];
    for k in 0..n {
        round_at[k + 1] = round_at[k] + pts[0][k].distance(pts[0][(k + 1) % n]);
    }
    let spans = if closed { rings.len() } else { rings.len() - 1 };
    let mut along = 0.0f32;
    for i in 0..spans {
        let j = (i + 1) % rings.len();
        let length = rings[i].centre.distance(rings[j].centre);
        let inside = (rings[i].centre + rings[j].centre) * 0.5;
        for k in 0..n {
            let l = (k + 1) % n;
            let (a, b, c, d) = (pts[i][k], pts[i][l], pts[j][l], pts[j][k]);
            let normal = ((a + b + c + d) * 0.25 - inside).normalize_or(Vec3::Y);
            let (which, col) = pick(normal);
            let out = &mut outs[which.min(outs.len() - 1)];
            let (u0, u1) = (round_at[k] / m, round_at[k + 1] / m);
            let (v0, v1) = (along / m, (along + length) / m);
            let col = lin(col);
            push_tri(out, [a, b, c], normal, col, [[u0, v0], [u1, v0], [u1, v1]]);
            push_tri(out, [a, c, d], normal, col, [[u0, v0], [u1, v1], [u0, v1]]);
        }
        along += length;
    }
    if !closed {
        let last = rings.len() - 1;
        for (i, toward) in [(0usize, 1usize), (last, last - 1)] {
            let r = rings[i];
            let normal = (r.centre - rings[toward].centre).normalize_or(Vec3::Y);
            let (which, col) = pick(normal);
            let out = &mut outs[which.min(outs.len() - 1)];
            let col = lin(col);
            let uv = |p: Vec3| {
                let d = p - r.centre;
                [d.dot(r.u) / m, d.dot(r.v) / m]
            };
            for k in 0..n {
                let (p, q) = (pts[i][k], pts[i][(k + 1) % n]);
                push_tri(
                    out,
                    [r.centre, p, q],
                    normal,
                    col,
                    [uv(r.centre), uv(p), uv(q)],
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tube() -> Vec<Ring> {
        (0..4)
            .map(|i| Ring {
                centre: Vec3::Z * i as f32,
                u: Vec3::X,
                v: Vec3::Y,
                ru: 0.5,
                rv: 0.5,
            })
            .collect()
    }

    /// A smooth loft's normals point out and vary smoothly round it: two
    /// neighbours round a ring share their surface, so their normals are
    /// close; and every vertex has a UV inside its tile.
    #[test]
    fn a_smooth_loft_is_smooth_and_points_out() {
        let mut b = Builder::default();
        smooth(&mut b, &tube(), 12, Skin::Coat(Tile::Hair), &|_, _, _| {
            [1.0; 3]
        });
        assert!(b.textured);
        assert_eq!(b.positions.len(), b.uvs.len());
        // The side's first ring: 13 points (the seam doubled).
        for k in 0..12 {
            let p = Vec3::from(b.positions[13 + k]);
            let n = Vec3::from(b.normals[13 + k]);
            let radial = Vec3::new(p.x, p.y, 0.0).normalize();
            assert!(n.dot(radial) > 0.99, "{k}: {n} against {radial}");
        }
        for uv in &b.uvs[..13 * 4] {
            assert!((0.0..=32.0 / 192.0).contains(&uv[0]), "{uv:?}");
        }
        // The winding agrees with the normal.
        for t in b.indices.chunks(3).take(24) {
            let [a, c, d] = [t[0], t[1], t[2]].map(|i| Vec3::from(b.positions[i as usize]));
            let face = (c - a).cross(d - a);
            let n = Vec3::from(b.normals[t[0] as usize]);
            assert!(face.dot(n) > 0.0);
        }
    }

    /// A hard loft lays a settlement texture by metres along its run.
    #[test]
    fn a_hard_loft_lays_its_texture_by_metres() {
        let mut outs = [Builder::default()];
        hard(&mut outs, 1.0, &tube(), &TIMBER, false, &|_| (0, [1.0; 3]));
        let most_v = outs[0].uvs.iter().map(|uv| uv[1]).fold(0.0f32, f32::max);
        assert!((most_v - 3.0).abs() < 1e-4, "{most_v}");
    }

    /// The curve passes through every authored ring.
    #[test]
    fn a_curve_passes_through_its_rings() {
        let rings = tube();
        let c = curve(&rings, 3);
        assert_eq!(c.len(), 3 * 3 + 1);
        for (i, r) in rings.iter().enumerate() {
            assert!(c[i * 3].centre.distance(r.centre) < 1e-5);
        }
    }
}
