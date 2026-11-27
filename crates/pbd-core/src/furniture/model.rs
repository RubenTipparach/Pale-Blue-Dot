//! Each piece of furniture as one shaped form (`furniture` design 2; owner,
//! 2026-11-22: "avoid obvious primitives"): bevelled solids whose every edge
//! is taken off, legs that taper from a shoulder into the top they hold, a
//! seat with a rolled front, a mattress and pillow that bulge, a chest lid
//! that crowns with its bands wrapped round its curve. In the town atlas's
//! own textures, so a piece takes the room's material and light.
//!
//! A piece's frame: x across it (along the wall it backs on), y up, z toward
//! its front, the origin on the floor under its middle. Every polygon is
//! convex and wound counter-clockwise seen from outside.

use super::Kind;
use glam::{Vec2, Vec3};

/// One face: its texture, its points counter-clockwise from outside, and a
/// normal at each point: the face's own at a real board edge, shared across
/// a curve (a bevel's roll, a turned leg, a crowned lid, a mattress's bulge),
/// so a piece is shaded smooth and not faceted (owner, 2026-11-23: "I didn't
/// like flat shaded crap").
#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    pub texture: &'static str,
    pub points: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    /// Texture coordinates when the texture is mapped once over the face
    /// (image space: u right, v down, 0 to 1), as a drawer front's or a
    /// chest's painted bands are; `None` is the town's world mapping, at the
    /// manifest's scale, so texels match the walls round it.
    pub uvs: Option<Vec<Vec2>>,
}

impl Poly {
    /// Its outward normal.
    pub fn normal(&self) -> Vec3 {
        let p = &self.points;
        let mut n = Vec3::ZERO;
        for i in 0..p.len() {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            n += Vec3::new(
                (a.y - b.y) * (a.z + b.z),
                (a.z - b.z) * (a.x + b.x),
                (a.x - b.x) * (a.y + b.y),
            );
        }
        n.normalize_or_zero()
    }
}

/// The textures a piece is made of.
pub const FRAME: &str = "timber";
pub const TOP: &str = "plank";
pub const BOARDS: &str = "boards";
pub const SHEET: &str = "linen";
pub const BLANKET: &str = "wool";
pub const IRON: &str = "iron";
/// Painted once over a face (`tools/art/furniture_textures.py`): a chest of
/// drawers' front with its three drawers and pulls, a closet's two doors and
/// ring pulls, and chests' boards with their bands and lock.
pub const DRAWER_FRONT: &str = "drawerfront";
pub const CLOSET_FRONT: &str = "closetfront";
pub const CHEST_SIDE: &str = "chestside";
pub const CHEST_FRONT: &str = "chestfront";
pub const BANDED_SIDE: &str = "bandedside";
pub const BANDED_FRONT: &str = "bandedfront";

#[derive(Default)]
struct Kit {
    polys: Vec<Poly>,
}

impl Kit {
    /// A flat face, turned to face away from `inside`.
    fn face(&mut self, texture: &'static str, points: Vec<Vec3>, inside: Vec3) {
        self.face_n(texture, points, inside, &|_, n| n);
    }

    /// A face turned to face away from `inside`, its normal at each point
    /// `normal(point, face normal)`.
    fn face_n(
        &mut self,
        texture: &'static str,
        mut points: Vec<Vec3>,
        inside: Vec3,
        normal: &dyn Fn(Vec3, Vec3) -> Vec3,
    ) {
        if points.len() < 3 {
            return;
        }
        let mut poly = Poly {
            texture,
            points: points.clone(),
            normals: Vec::new(),
            uvs: None,
        };
        let centre = points.iter().copied().sum::<Vec3>() / points.len() as f32;
        if poly.normal().dot(centre - inside) < 0.0 {
            points.reverse();
        }
        poly.points = points;
        let n = poly.normal();
        poly.normals = poly
            .points
            .iter()
            .map(|&p| {
                let m = normal(p, n);
                if m.dot(n) > 0.05 { m.normalize() } else { n }
            })
            .collect();
        self.polys.push(poly);
    }

    /// Paint the upright main faces among the polys from `from` on with a
    /// texture mapped once over each face (`texture(normal)`, or none to
    /// leave a face as it is): a drawer front, a closet's doors, a chest's
    /// sides with their bands. u runs left to right seen from outside, v
    /// from the face's top down.
    fn map_faces(&mut self, from: usize, texture: &dyn Fn(Vec3) -> Option<&'static str>) {
        for poly in &mut self.polys[from..] {
            let n = poly.normal();
            if n.y.abs() > 0.1 || n.length() < 0.5 {
                continue;
            }
            // A main face: flat across all its points.
            if poly.normals.iter().any(|m| m.dot(n) < 0.999) {
                continue;
            }
            let Some(t) = texture(n) else { continue };
            let right = Vec3::Y.cross(n).normalize();
            let us: Vec<f32> = poly.points.iter().map(|p| p.dot(right)).collect();
            let (u0, u1) = us
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &u| (a.min(u), b.max(u)));
            let (y0, y1) = poly
                .points
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
            poly.texture = t;
            poly.uvs = Some(
                poly.points
                    .iter()
                    .zip(&us)
                    .map(|(p, u)| {
                        Vec2::new(
                            (u - u0) / (u1 - u0).max(1e-4),
                            (y1 - p.y) / (y1 - y0).max(1e-4),
                        )
                    })
                    .collect(),
            );
        }
    }

    /// A bevelled box: its bottom's middle at `at`, `size` across, up and
    /// deep, every edge taken off by `bevel`. The top in `top`, the rest in
    /// `side`.
    fn block(&mut self, top: &'static str, side: &'static str, at: Vec3, size: Vec3, bevel: f32) {
        let h = size / 2.0;
        let c = at + Vec3::Y * h.y;
        let b = bevel.min(h.min_element() * 0.9);
        let i = h - Vec3::splat(b);
        let pick = |n: Vec3| if n.y > 0.5 { top } else { side };
        let axes = [Vec3::X, Vec3::Y, Vec3::Z];
        // A rounded box's normals: out from the inner box, so its faces stay
        // flat and its bevels roll from one face into the next.
        let (lo, hi) = (c - i, c + i);
        let round = move |p: Vec3, n: Vec3| {
            let d = p - p.clamp(lo, hi);
            if d.length_squared() < 1e-10 {
                n
            } else {
                d.normalize()
            }
        };
        // The six faces.
        for a in 0..3 {
            for s in [-1.0f32, 1.0] {
                let n = axes[a] * s;
                let (u, v) = (axes[(a + 1) % 3], axes[(a + 2) % 3]);
                let (hu, hv) = (i.dot(u), i.dot(v));
                let o = c + n * h.dot(axes[a]);
                let pts = vec![
                    o - u * hu - v * hv,
                    o + u * hu - v * hv,
                    o + u * hu + v * hv,
                    o - u * hu + v * hv,
                ];
                self.face_n(pick(n), pts, c, &round);
            }
        }
        // The twelve edges.
        for a in 0..3 {
            let (p, q) = (axes[(a + 1) % 3], axes[(a + 2) % 3]);
            let along = axes[a] * i.dot(axes[a]);
            for sp in [-1.0f32, 1.0] {
                for sq in [-1.0f32, 1.0] {
                    let e1 = c + p * (sp * h.dot(p)) + q * (sq * i.dot(q));
                    let e2 = c + p * (sp * i.dot(p)) + q * (sq * h.dot(q));
                    let n = (p * sp + q * sq).normalize();
                    self.face_n(
                        pick(n),
                        vec![e1 - along, e1 + along, e2 + along, e2 - along],
                        c,
                        &round,
                    );
                }
            }
        }
        // The eight corners.
        for sx in [-1.0f32, 1.0] {
            for sy in [-1.0f32, 1.0] {
                for sz in [-1.0f32, 1.0] {
                    let s = Vec3::new(sx, sy, sz);
                    let pts = vec![
                        c + Vec3::new(h.x, i.y, i.z) * s,
                        c + Vec3::new(i.x, h.y, i.z) * s,
                        c + Vec3::new(i.x, i.y, h.z) * s,
                    ];
                    let n = s.normalize();
                    self.face_n(pick(n), pts, c, &round);
                }
            }
        }
    }

    /// A lathe or a dressed post: rings `(t, radius)` along `axis` from
    /// `base`, `sides` round, the first turned by `rot`; capped at its ends.
    fn loft(
        &mut self,
        texture: &'static str,
        base: Vec3,
        axis: Vec3,
        rings: &[(f32, f32)],
        sides: usize,
        rot: f32,
    ) {
        let u = axis.normalize();
        let v = u.any_orthonormal_vector();
        let w = u.cross(v);
        let ring = |t: f32, r: f32| -> Vec<Vec3> {
            (0..sides)
                .map(|k| {
                    let a = rot + k as f32 / sides as f32 * std::f32::consts::TAU;
                    base + u * t + (v * a.cos() + w * a.sin()) * r
                })
                .collect()
        };
        let rs: Vec<Vec<Vec3>> = rings.iter().map(|&(t, r)| ring(t, r)).collect();
        for j in 0..rs.len().saturating_sub(1) {
            let mid = base + u * ((rings[j].0 + rings[j + 1].0) / 2.0);
            // Round (six sides or more): each point's normal is out from the
            // axis, tipped by the band's taper; a slab of four is flat.
            let (dt, dr) = (rings[j + 1].0 - rings[j].0, rings[j + 1].1 - rings[j].1);
            let smooth = move |p: Vec3, n: Vec3| {
                if sides < 6 {
                    return n;
                }
                let q = p - base;
                let radial = (q - u * q.dot(u)).normalize_or_zero();
                if radial == Vec3::ZERO {
                    return n;
                }
                (radial * dt - u * dr).normalize_or(n)
            };
            for k in 0..sides {
                let k2 = (k + 1) % sides;
                let (a, b, c, d) = (rs[j][k], rs[j][k2], rs[j + 1][k2], rs[j + 1][k]);
                if rings[j].1 < 1e-4 {
                    self.face_n(texture, vec![a, c, d], mid, &smooth);
                } else if rings[j + 1].1 < 1e-4 {
                    self.face_n(texture, vec![a, b, c], mid, &smooth);
                } else {
                    self.face_n(texture, vec![a, b, c, d], mid, &smooth);
                }
            }
        }
        let centre = base + u * ((rings[0].0 + rings[rings.len() - 1].0) / 2.0);
        if let (Some(first), Some(last)) = (rings.first(), rings.last()) {
            if first.1 > 1e-4 {
                self.face(texture, rs[0].clone(), centre);
            }
            if last.1 > 1e-4 {
                self.face(texture, rs[rs.len() - 1].clone(), centre);
            }
        }
    }

    /// A leg: tapered from a narrow foot to a wide shoulder, its top let a
    /// little into what it holds, octagonal (a dressed post, no box corner).
    fn leg(&mut self, at: Vec3, height: f32, foot: f32, shoulder: f32) {
        self.loft(
            FRAME,
            at,
            Vec3::Y,
            &[
                (0.0, foot * 0.85),
                (0.02, foot),
                (height * 0.7, shoulder * 0.9),
                (height + 0.01, shoulder),
            ],
            8,
            std::f32::consts::FRAC_PI_8,
        );
    }

    /// A convex profile in the (z, y) plane, `prof`, run across x from `x0`
    /// to `x1`: a chest's lid and its bands.
    /// Its sides' texture is mapped once over the whole curve when `mapped`:
    /// u along x, v along the profile from its first point.
    fn run_x_mapped(
        &mut self,
        side: &'static str,
        ends: &'static str,
        prof: &[Vec2],
        x0: f32,
        x1: f32,
        mapped: bool,
    ) {
        let from = self.polys.len();
        let n = prof.len();
        let c2 = prof.iter().copied().sum::<Vec2>() / n as f32;
        let mid = Vec3::new((x0 + x1) / 2.0, c2.y, c2.x);
        // Each edge's outward normal in (z, y); a point between two edges
        // less than 40 degrees apart shares their mean (the lid's curve), a
        // sharper corner keeps each edge's own (its foot).
        let out: Vec<Vec2> = (0..n)
            .map(|k| {
                let e = prof[(k + 1) % n] - prof[k];
                let o = Vec2::new(e.y, -e.x).normalize_or_zero();
                if o.dot((prof[k] + prof[(k + 1) % n]) * 0.5 - c2) < 0.0 {
                    -o
                } else {
                    o
                }
            })
            .collect();
        let at = |k: usize, own: usize| -> Vec3 {
            let (p, q) = (out[(k + n - 1) % n], out[k % n]);
            let m = if p.dot(q) > 0.766 {
                (p + q).normalize()
            } else {
                out[own]
            };
            Vec3::new(0.0, m.y, m.x)
        };
        for k in 0..n {
            let (a, b) = (prof[k], prof[(k + 1) % n]);
            let (na, nb) = (at(k, k), at((k + 1) % n, k));
            let smooth = move |p: Vec3, _n: Vec3| {
                if (p.y - a.y).abs() < 1e-5 && (p.z - a.x).abs() < 1e-5 {
                    na
                } else {
                    nb
                }
            };
            self.face_n(
                side,
                vec![
                    Vec3::new(x0, a.y, a.x),
                    Vec3::new(x1, a.y, a.x),
                    Vec3::new(x1, b.y, b.x),
                    Vec3::new(x0, b.y, b.x),
                ],
                mid,
                &smooth,
            );
        }
        if mapped {
            // Arc length to each profile point, over the curve's whole run.
            let mut along = vec![0.0f32; n];
            for k in 1..n {
                along[k] = along[k - 1] + prof[k].distance(prof[k - 1]);
            }
            let total = along[n - 1].max(1e-4);
            let at_point = |p: Vec3| {
                (0..n)
                    .min_by(|&a, &b| {
                        let d = |k: usize| (prof[k] - Vec2::new(p.z, p.y)).length_squared();
                        d(a).total_cmp(&d(b))
                    })
                    .map_or(0.0, |k| along[k] / total)
            };
            for poly in &mut self.polys[from..] {
                poly.uvs = Some(
                    poly.points
                        .iter()
                        .map(|p| Vec2::new((p.x - x0) / (x1 - x0), at_point(*p)))
                        .collect(),
                );
            }
        }
        for x in [x0, x1] {
            let pts = prof.iter().map(|p| Vec3::new(x, p.y, p.x)).collect();
            self.face(ends, pts, mid);
        }
    }
}

/// A crowned lid's profile: across `depth`, from `y0` up `rise` over its
/// flat sides `edge` high, as a polygon in (z, y).
fn crown(depth: f32, y0: f32, edge: f32, rise: f32, grow: f32) -> Vec<Vec2> {
    let hz = depth / 2.0 + grow;
    let mut pts = vec![Vec2::new(-hz, y0 - grow), Vec2::new(hz, y0 - grow)];
    let steps = 7;
    for k in 0..=steps {
        let a = k as f32 / steps as f32 * std::f32::consts::PI;
        let z = hz * a.cos();
        let y = y0 + edge + (rise + grow) * a.sin();
        pts.push(Vec2::new(z, y));
    }
    pts
}

/// A piece's polygons, in its own frame.
pub fn polys(kind: Kind) -> Vec<Poly> {
    let mut k = Kit::default();
    let s = kind.size_m();
    let (hx, hz) = (s.x / 2.0, s.y / 2.0);
    match kind {
        Kind::Chair => {
            let seat = 0.42;
            for (x, z) in [(-1.0, 1.0), (1.0, 1.0)] {
                k.leg(
                    Vec3::new(x * (hx - 0.04), 0.0, z * (hz - 0.05)),
                    seat,
                    0.018,
                    0.026,
                );
            }
            // The back legs run up past the seat into the back, one piece.
            for x in [-1.0f32, 1.0] {
                k.loft(
                    FRAME,
                    Vec3::new(x * (hx - 0.04), 0.0, -(hz - 0.07)),
                    Vec3::new(0.0, 1.0, -0.04),
                    &[
                        (0.0, 0.016),
                        (0.02, 0.019),
                        (seat, 0.028),
                        (0.93, 0.022),
                        (0.95, 0.012),
                    ],
                    8,
                    std::f32::consts::FRAC_PI_8,
                );
            }
            k.block(
                TOP,
                FRAME,
                Vec3::new(0.0, seat, 0.01),
                Vec3::new(s.x, 0.05, s.y - 0.02),
                0.018,
            );
            // The rolled front of the seat.
            k.loft(
                TOP,
                Vec3::new(-hx + 0.01, seat + 0.025, hz - 0.01),
                Vec3::X,
                &[(0.0, 0.026), (s.x - 0.02, 0.026)],
                8,
                0.0,
            );
            // The back's rails, curved by a lean between the posts.
            for (y, h) in [(0.62, 0.06), (0.84, 0.09)] {
                k.block(
                    FRAME,
                    FRAME,
                    Vec3::new(0.0, y, -(hz - 0.07) - y * 0.04),
                    Vec3::new(s.x - 0.07, h, 0.026),
                    0.009,
                );
            }
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.12, hz - 0.05),
                Vec3::new(s.x - 0.08, 0.03, 0.022),
                0.008,
            );
        }
        Kind::Stool => {
            let seat = 0.45;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                // Splayed: the foot out from under the seat.
                let top = Vec3::new(x * 0.1, seat, z * 0.1);
                let foot = Vec3::new(x * 0.16, 0.0, z * 0.16);
                k.loft(
                    FRAME,
                    foot,
                    top - foot,
                    &[
                        (0.0, 0.016),
                        (0.02, 0.018),
                        ((top - foot).length() + 0.01, 0.026),
                    ],
                    8,
                    0.3,
                );
            }
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.16, 0.0),
                Vec3::new(0.27, 0.022, 0.022),
                0.007,
            );
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.2, 0.0),
                Vec3::new(0.022, 0.022, 0.25),
                0.007,
            );
            // A round top with a rolled rim.
            k.loft(
                TOP,
                Vec3::new(0.0, seat, 0.0),
                Vec3::Y,
                &[
                    (0.0, 0.17),
                    (0.01, 0.19),
                    (0.04, 0.2),
                    (0.05, 0.185),
                    (0.052, 0.0),
                ],
                12,
                0.0,
            );
        }
        Kind::Bench => {
            let seat = 0.4;
            // Slab ends, wide at the foot and let into the seat.
            for x in [-1.0f32, 1.0] {
                let at = Vec3::new(x * (hx - 0.16), 0.0, 0.0);
                k.loft(
                    FRAME,
                    at,
                    Vec3::Y,
                    &[(0.0, 0.2), (0.03, 0.2), (0.1, 0.15), (seat + 0.01, 0.17)],
                    4,
                    std::f32::consts::FRAC_PI_4,
                );
            }
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.17, 0.0),
                Vec3::new(s.x - 0.36, 0.05, 0.05),
                0.012,
            );
            k.block(
                TOP,
                FRAME,
                Vec3::new(0.0, seat, 0.0),
                Vec3::new(s.x, 0.06, s.y),
                0.02,
            );
        }
        Kind::Table => {
            let top = 0.71;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                // A turned leg: a foot, a swell, a neck and a block top.
                k.loft(
                    FRAME,
                    Vec3::new(x * (hx - 0.1), 0.0, z * (hz - 0.1)),
                    Vec3::Y,
                    &[
                        (0.0, 0.026),
                        (0.03, 0.03),
                        (0.09, 0.024),
                        (0.3, 0.038),
                        (0.5, 0.03),
                        (0.6, 0.033),
                        (top - 0.08, 0.04),
                        (top + 0.005, 0.04),
                    ],
                    8,
                    std::f32::consts::FRAC_PI_8,
                );
            }
            // The apron under the top, and the top over it.
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, top - 0.1, 0.0),
                Vec3::new(s.x - 0.18, 0.1, s.y - 0.18),
                0.012,
            );
            k.block(
                TOP,
                FRAME,
                Vec3::new(0.0, top, 0.0),
                Vec3::new(s.x, 0.05, s.y),
                0.02,
            );
        }
        Kind::Bed => {
            let rail = 0.22;
            // Corner posts, the head's taller.
            for (z, h) in [(-hz + 0.04, 0.95), (hz - 0.04, 0.6)] {
                for x in [-1.0f32, 1.0] {
                    k.loft(
                        FRAME,
                        Vec3::new(x * (hx - 0.04), 0.0, z),
                        Vec3::Y,
                        &[
                            (0.0, 0.03),
                            (0.03, 0.036),
                            (h - 0.08, 0.038),
                            (h - 0.03, 0.03),
                            (h, 0.0),
                        ],
                        8,
                        std::f32::consts::FRAC_PI_8,
                    );
                }
            }
            // The rails and the head and foot boards between the posts.
            for x in [-1.0f32, 1.0] {
                k.block(
                    FRAME,
                    FRAME,
                    Vec3::new(x * (hx - 0.04), 0.12, 0.0),
                    Vec3::new(0.05, 0.16, s.y - 0.12),
                    0.012,
                );
            }
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.12, -hz + 0.04),
                Vec3::new(s.x - 0.1, 0.75, 0.05),
                0.018,
            );
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.12, hz - 0.04),
                Vec3::new(s.x - 0.1, 0.4, 0.05),
                0.018,
            );
            // The mattress, soft at every edge, and the blanket over its foot
            // end, standing proud of it and falling over its sides.
            k.block(
                SHEET,
                SHEET,
                Vec3::new(0.0, rail, 0.0),
                Vec3::new(s.x - 0.1, 0.24, s.y - 0.12),
                0.07,
            );
            k.block(
                BLANKET,
                BLANKET,
                Vec3::new(0.0, rail + 0.06, 0.24),
                Vec3::new(s.x - 0.06, 0.205, 1.28),
                0.07,
            );
            // The pillow, plump.
            k.block(
                SHEET,
                SHEET,
                Vec3::new(0.0, rail + 0.22, -hz + 0.27),
                Vec3::new(0.62, 0.12, 0.34),
                0.055,
            );
        }
        Kind::Drawers => {
            let h = 0.86;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                k.loft(
                    FRAME,
                    Vec3::new(x * (hx - 0.06), 0.0, z * (hz - 0.06)),
                    Vec3::Y,
                    &[(0.0, 0.026), (0.03, 0.036), (0.06, 0.03)],
                    8,
                    0.0,
                );
            }
            let from = k.polys.len();
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.06, -0.01),
                Vec3::new(s.x - 0.04, h - 0.06, s.y - 0.04),
                0.016,
            );
            // The three drawers and their pulls are painted on its front.
            k.map_faces(from, &|n| (n.z > 0.9).then_some(DRAWER_FRONT));
            k.block(
                TOP,
                FRAME,
                Vec3::new(0.0, h, 0.0),
                Vec3::new(s.x, 0.04, s.y),
                0.014,
            );
        }
        Kind::Closet => {
            let h = 1.95;
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(s.x - 0.02, 0.1, s.y - 0.02),
                0.02,
            );
            // The carcass, its two doors and their ring pulls painted on its
            // front.
            let from = k.polys.len();
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, 0.1, -0.01),
                Vec3::new(s.x - 0.06, h - 0.18, s.y - 0.06),
                0.018,
            );
            k.map_faces(from, &|n| (n.z > 0.9).then_some(CLOSET_FRONT));
            // The cornice, stepped out over the carcass.
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, h - 0.1, 0.0),
                Vec3::new(s.x - 0.02, 0.05, s.y - 0.02),
                0.015,
            );
            k.block(
                TOP,
                FRAME,
                Vec3::new(0.0, h - 0.05, 0.0),
                Vec3::new(s.x, 0.05, s.y),
                0.02,
            );
        }
        Kind::Chest | Kind::BandedChest => {
            let body = if kind == Kind::Chest { 0.38 } else { 0.42 };
            let (edge, rise) = (0.04, s.y * 0.2);
            // Its bands, its lock and its corners are painted on its boards,
            // mapped once round each side and once over the lid, so they run
            // on round the curve.
            let (side, front) = if kind == Kind::Chest {
                (CHEST_SIDE, CHEST_FRONT)
            } else {
                (BANDED_SIDE, BANDED_FRONT)
            };
            let from = k.polys.len();
            k.block(
                BOARDS,
                FRAME,
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(s.x, body, s.y),
                0.02,
            );
            k.map_faces(from, &|n| Some(if n.z > 0.9 { front } else { side }));
            k.run_x_mapped(
                side,
                FRAME,
                &crown(s.y, body, edge, rise, 0.0),
                -hx + 0.005,
                hx - 0.005,
                true,
            );
        }
        Kind::Crate => {
            k.block(
                BOARDS,
                BOARDS,
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(s.x - 0.03, s.y - 0.03, s.y - 0.03),
                0.015,
            );
            // Its frame: battens along every edge, set into the boards, and a
            // brace across the front and back.
            let e = s.x / 2.0 - 0.025;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                k.block(
                    FRAME,
                    FRAME,
                    Vec3::new(x * e, 0.0, z * e),
                    Vec3::new(0.06, s.x, 0.06),
                    0.012,
                );
            }
            for y in [0.0f32, s.x - 0.06] {
                for z in [-1.0f32, 1.0] {
                    k.block(
                        FRAME,
                        FRAME,
                        Vec3::new(0.0, y, z * e),
                        Vec3::new(s.x - 0.1, 0.06, 0.055),
                        0.012,
                    );
                    k.block(
                        FRAME,
                        FRAME,
                        Vec3::new(z * e, y, 0.0),
                        Vec3::new(0.055, 0.06, s.x - 0.1),
                        0.012,
                    );
                }
            }
        }
    }
    k.polys
}

/// What stands on the carpenter's bench (`furniture` design 1), over its
/// block's top (y = 0 at the top, x along the bench, z across it): a board
/// laid on it, a saw across the board, its blade's back straight and its
/// handle a turned grip, and a square at the board's end.
pub fn carpenter_top() -> Vec<Poly> {
    let mut k = Kit::default();
    k.block(
        TOP,
        FRAME,
        Vec3::new(-0.15, 0.0, 0.05),
        Vec3::new(1.4, 0.05, 0.36),
        0.012,
    );
    // The saw: a blade tapering from heel to toe, and its grip.
    let (y, z) = (0.05, -0.02);
    k.face(
        IRON,
        vec![
            Vec3::new(-0.45, y + 0.004, z - 0.09),
            Vec3::new(0.2, y + 0.004, z - 0.05),
            Vec3::new(0.2, y + 0.004, z + 0.0),
            Vec3::new(-0.45, y + 0.004, z + 0.04),
        ],
        Vec3::new(0.0, y - 1.0, 0.0),
    );
    k.face(
        IRON,
        vec![
            Vec3::new(-0.45, y + 0.001, z - 0.09),
            Vec3::new(0.2, y + 0.001, z - 0.05),
            Vec3::new(0.2, y + 0.001, z + 0.0),
            Vec3::new(-0.45, y + 0.001, z + 0.04),
        ],
        Vec3::new(0.0, y + 1.0, 0.0),
    );
    k.loft(
        FRAME,
        Vec3::new(-0.6, y + 0.025, z - 0.02),
        Vec3::X,
        &[(0.0, 0.016), (0.02, 0.026), (0.13, 0.024), (0.15, 0.012)],
        8,
        0.0,
    );
    // The square: a stock and its blade, at right angles, flat on the board.
    k.block(
        FRAME,
        FRAME,
        Vec3::new(0.38, y, 0.1),
        Vec3::new(0.05, 0.02, 0.22),
        0.006,
    );
    k.block(
        IRON,
        IRON,
        Vec3::new(0.27, y, 0.19),
        Vec3::new(0.22, 0.008, 0.035),
        0.003,
    );
    k.polys
}

/// What stands on the drafting table (`building-planner` design 3), over
/// its block's top at y 0: a drawing board tipped toward the front, its low
/// edge on a lip that keeps a sheet from sliding, and the sheet itself, one
/// form each with a bevel at the board's edge.
pub fn drafting_top() -> Vec<Poly> {
    let mut k = Kit::default();
    // The board: a slab, its back raised 0.22 m, 0.84 m wide, 0.72 deep.
    let (w, d, t) = (0.42f32, 0.36f32, 0.035f32);
    let rise = 0.22f32;
    let y = |z: f32| 0.04 + rise * (0.5 - z / (2.0 * d));
    let top = |x: f32, z: f32| Vec3::new(x, y(z) + t, z);
    let low = |x: f32, z: f32| Vec3::new(x, y(z), z);
    let inside = Vec3::new(0.0, y(0.0) + t * 0.5, 0.0);
    let (x0, x1, z0, z1) = (-w, w, -d, d);
    k.face(
        TOP,
        vec![top(x0, z0), top(x1, z0), top(x1, z1), top(x0, z1)],
        inside,
    );
    k.face(
        TOP,
        vec![low(x0, z0), low(x1, z0), low(x1, z1), low(x0, z1)],
        inside,
    );
    k.face(
        FRAME,
        vec![low(x0, z0), low(x1, z0), top(x1, z0), top(x0, z0)],
        inside,
    );
    k.face(
        FRAME,
        vec![low(x0, z1), low(x1, z1), top(x1, z1), top(x0, z1)],
        inside,
    );
    k.face(
        FRAME,
        vec![low(x0, z0), low(x0, z1), top(x0, z1), top(x0, z0)],
        inside,
    );
    k.face(
        FRAME,
        vec![low(x1, z0), low(x1, z1), top(x1, z1), top(x1, z0)],
        inside,
    );
    // The back leg the board rests on: a block across under its high edge.
    k.block(
        FRAME,
        FRAME,
        Vec3::new(0.0, 0.0, -d + 0.06),
        Vec3::new(2.0 * w - 0.1, y(-d + 0.06), 0.07),
        0.01,
    );
    // The lip along the low edge.
    k.block(
        FRAME,
        FRAME,
        Vec3::new(0.0, y(d - 0.02) + t, d - 0.02),
        Vec3::new(2.0 * w, 0.03, 0.035),
        0.006,
    );
    // The sheet, a hair over the board, short of its edges.
    let sheet = |x: f32, z: f32| Vec3::new(x, y(z) + t + 0.004, z);
    let (sx, sz) = (w - 0.08, d - 0.09);
    k.face(
        SHEET,
        vec![
            sheet(-sx, -sz),
            sheet(sx, -sz),
            sheet(sx, sz),
            sheet(-sx, sz),
        ],
        Vec3::new(0.0, -1.0, 0.0),
    );
    k.polys
}

/// A piece's polygons stood in a plan: its middle at `at` (x, z), on a floor
/// at `y0`, its front along `front` (x, z).
pub fn placed(kind: Kind, at: Vec2, y0: f32, front: Vec2) -> Vec<Poly> {
    let z = front.normalize_or(Vec2::Y);
    let x = Vec2::new(z.y, -z.x);
    polys(kind)
        .into_iter()
        .map(|p| Poly {
            texture: p.texture,
            points: p
                .points
                .iter()
                .map(|q| {
                    let h = at + x * q.x + z * q.z;
                    Vec3::new(h.x, y0 + q.y, h.y)
                })
                .collect(),
            normals: p
                .normals
                .iter()
                .map(|n| {
                    let h = x * n.x + z * n.z;
                    Vec3::new(h.x, n.y, h.y)
                })
                .collect(),
            uvs: p.uvs,
        })
        .collect()
}

/// The chest a player sets down (`station-models` design 2): the towns'
/// chest, a quarter larger, so it reads at the size of what a cell holds.
/// On its cell's floor, in the furniture's frame.
pub fn station_chest() -> Vec<Poly> {
    const SCALE: f32 = 1.25;
    polys(Kind::Chest)
        .into_iter()
        .map(|p| Poly {
            points: p.points.iter().map(|q| *q * SCALE).collect(),
            ..p
        })
        .collect()
}

/// The workbench (`station-models` design 2), on its cell's floor: a thick
/// plank top on four dressed legs that taper into it, aprons under the top
/// and stretchers low down carrying a boards shelf, a wooden vise at the
/// right of the front with an iron screw and a timber bar through it, and a
/// mallet and a chisel on the top.
pub fn workbench() -> Vec<Poly> {
    let mut k = Kit::default();
    let (hx, hz) = (0.8f32, 0.35f32);
    let top_y = 0.79;
    let face = top_y + 0.09;
    k.block(
        TOP,
        FRAME,
        Vec3::new(0.0, top_y, 0.0),
        Vec3::new(hx * 2.0, 0.09, hz * 2.0),
        0.02,
    );
    // Legs, set in from the top's corners.
    let (lx, lz) = (hx - 0.12, hz - 0.09);
    for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        k.leg(Vec3::new(x * lx, 0.0, z * lz), top_y, 0.045, 0.06);
    }
    // Aprons under the top and stretchers low, front and back, then the ends.
    for (y, h) in [(top_y - 0.12, 0.12f32), (0.14, 0.06)] {
        for z in [-1.0f32, 1.0] {
            k.block(
                FRAME,
                FRAME,
                Vec3::new(0.0, y, z * lz),
                Vec3::new(lx * 2.0 - 0.06, h, 0.05),
                0.012,
            );
        }
        for x in [-1.0f32, 1.0] {
            k.block(
                FRAME,
                FRAME,
                Vec3::new(x * lx, y, 0.0),
                Vec3::new(0.05, h, lz * 2.0 - 0.06),
                0.012,
            );
        }
    }
    // The shelf, its boards laid on the stretchers.
    k.block(
        BOARDS,
        BOARDS,
        Vec3::new(0.0, 0.2, 0.0),
        Vec3::new(lx * 2.0 - 0.02, 0.03, lz * 2.0 + 0.02),
        0.008,
    );
    // The vise: a jaw against the front of the top, its screw through the
    // jaw into the bench, and the bar through the screw's head.
    let (vx, vy) = (hx - 0.26, face - 0.13);
    k.block(
        FRAME,
        FRAME,
        Vec3::new(vx, face - 0.28, hz + 0.035),
        Vec3::new(0.32, 0.28, 0.07),
        0.015,
    );
    k.loft(
        IRON,
        Vec3::new(vx, vy, hz - 0.02),
        Vec3::Z,
        &[(0.0, 0.017), (0.15, 0.017), (0.155, 0.026), (0.2, 0.026)],
        8,
        0.0,
    );
    k.loft(
        FRAME,
        Vec3::new(vx - 0.13, vy, hz + 0.155),
        Vec3::X,
        &[
            (0.0, 0.019),
            (0.025, 0.019),
            (0.03, 0.012),
            (0.23, 0.012),
            (0.235, 0.019),
            (0.26, 0.019),
        ],
        8,
        0.0,
    );
    // A mallet: its head on the top, its handle running down to it.
    let (mx, mz) = (-0.38, 0.06);
    k.block(
        FRAME,
        FRAME,
        Vec3::new(mx, face, mz),
        Vec3::new(0.1, 0.1, 0.18),
        0.018,
    );
    k.loft(
        FRAME,
        Vec3::new(mx + 0.05, face + 0.05, mz),
        Vec3::new(1.0, -0.13, 0.0),
        &[
            (0.0, 0.018),
            (0.04, 0.015),
            (0.24, 0.013),
            (0.27, 0.016),
            (0.28, 0.012),
        ],
        8,
        0.0,
    );
    // A chisel: an iron blade and a turned handle, flat on the top.
    let (cx, cz) = (0.02, -0.16);
    k.block(
        IRON,
        IRON,
        Vec3::new(cx, face, cz),
        Vec3::new(0.15, 0.008, 0.026),
        0.002,
    );
    k.loft(
        FRAME,
        Vec3::new(cx + 0.07, face + 0.016, cz),
        Vec3::X,
        &[(0.0, 0.011), (0.02, 0.016), (0.11, 0.015), (0.125, 0.009)],
        8,
        0.0,
    );
    k.polys
}
