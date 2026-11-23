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

/// One face: its texture and its points, counter-clockwise from outside.
#[derive(Clone, Debug, PartialEq)]
pub struct Poly {
    pub texture: &'static str,
    pub points: Vec<Vec3>,
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

#[derive(Default)]
struct Kit {
    polys: Vec<Poly>,
}

impl Kit {
    /// A face, turned to face away from `inside`.
    fn face(&mut self, texture: &'static str, mut points: Vec<Vec3>, inside: Vec3) {
        if points.len() < 3 {
            return;
        }
        let poly = Poly {
            texture,
            points: points.clone(),
        };
        let centre = points.iter().copied().sum::<Vec3>() / points.len() as f32;
        if poly.normal().dot(centre - inside) < 0.0 {
            points.reverse();
        }
        self.polys.push(Poly { texture, points });
    }

    /// A bevelled box: its bottom's middle at `at`, `size` across, up and
    /// deep, every edge taken off by `bevel`. The top in `top`, the rest in
    /// `side`.
    fn block(
        &mut self,
        top: &'static str,
        side: &'static str,
        at: Vec3,
        size: Vec3,
        bevel: f32,
    ) {
        let h = size / 2.0;
        let c = at + Vec3::Y * h.y;
        let b = bevel.min(h.min_element() * 0.9);
        let i = h - Vec3::splat(b);
        let pick = |n: Vec3| if n.y > 0.5 { top } else { side };
        let axes = [Vec3::X, Vec3::Y, Vec3::Z];
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
                self.face(pick(n), pts, c);
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
                    self.face(
                        pick(n),
                        vec![e1 - along, e1 + along, e2 + along, e2 - along],
                        c,
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
                    self.face(pick(n), pts, c);
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
            for k in 0..sides {
                let k2 = (k + 1) % sides;
                let (a, b, c, d) = (rs[j][k], rs[j][k2], rs[j + 1][k2], rs[j + 1][k]);
                if rings[j].1 < 1e-4 {
                    self.face(texture, vec![a, c, d], mid);
                } else if rings[j + 1].1 < 1e-4 {
                    self.face(texture, vec![a, b, c], mid);
                } else {
                    self.face(texture, vec![a, b, c, d], mid);
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
    fn run_x(&mut self, side: &'static str, ends: &'static str, prof: &[Vec2], x0: f32, x1: f32) {
        let n = prof.len();
        let c2 = prof.iter().copied().sum::<Vec2>() / n as f32;
        let mid = Vec3::new((x0 + x1) / 2.0, c2.y, c2.x);
        for k in 0..n {
            let (a, b) = (prof[k], prof[(k + 1) % n]);
            self.face(
                side,
                vec![
                    Vec3::new(x0, a.y, a.x),
                    Vec3::new(x1, a.y, a.x),
                    Vec3::new(x1, b.y, b.x),
                    Vec3::new(x0, b.y, b.x),
                ],
                mid,
            );
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
                k.leg(Vec3::new(x * (hx - 0.04), 0.0, z * (hz - 0.05)), seat, 0.018, 0.026);
            }
            // The back legs run up past the seat into the back, one piece.
            for x in [-1.0f32, 1.0] {
                k.loft(
                    FRAME,
                    Vec3::new(x * (hx - 0.04), 0.0, -(hz - 0.07)),
                    Vec3::new(0.0, 1.0, -0.04),
                    &[(0.0, 0.016), (0.02, 0.019), (seat, 0.028), (0.93, 0.022), (0.95, 0.012)],
                    8,
                    std::f32::consts::FRAC_PI_8,
                );
            }
            k.block(TOP, FRAME, Vec3::new(0.0, seat, 0.01), Vec3::new(s.x, 0.05, s.y - 0.02), 0.018);
            // The rolled front of the seat.
            k.loft(TOP, Vec3::new(-hx + 0.01, seat + 0.025, hz - 0.01), Vec3::X, &[(0.0, 0.026), (s.x - 0.02, 0.026)], 8, 0.0);
            // The back's rails, curved by a lean between the posts.
            for (y, h) in [(0.62, 0.06), (0.84, 0.09)] {
                k.block(FRAME, FRAME, Vec3::new(0.0, y, -(hz - 0.07) - y * 0.04), Vec3::new(s.x - 0.07, h, 0.026), 0.009);
            }
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.12, hz - 0.05), Vec3::new(s.x - 0.08, 0.03, 0.022), 0.008);
        }
        Kind::Stool => {
            let seat = 0.45;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                // Splayed: the foot out from under the seat.
                let top = Vec3::new(x * 0.1, seat, z * 0.1);
                let foot = Vec3::new(x * 0.16, 0.0, z * 0.16);
                k.loft(FRAME, foot, top - foot, &[(0.0, 0.016), (0.02, 0.018), ((top - foot).length() + 0.01, 0.026)], 8, 0.3);
            }
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.16, 0.0), Vec3::new(0.27, 0.022, 0.022), 0.007);
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.2, 0.0), Vec3::new(0.022, 0.022, 0.25), 0.007);
            // A round top with a rolled rim.
            k.loft(TOP, Vec3::new(0.0, seat, 0.0), Vec3::Y, &[(0.0, 0.17), (0.01, 0.19), (0.04, 0.2), (0.05, 0.185), (0.052, 0.0)], 12, 0.0);
        }
        Kind::Bench => {
            let seat = 0.4;
            // Slab ends, wide at the foot and let into the seat.
            for x in [-1.0f32, 1.0] {
                let at = Vec3::new(x * (hx - 0.16), 0.0, 0.0);
                k.loft(FRAME, at, Vec3::Y, &[(0.0, 0.2), (0.03, 0.2), (0.1, 0.15), (seat + 0.01, 0.17)], 4, std::f32::consts::FRAC_PI_4);
            }
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.17, 0.0), Vec3::new(s.x - 0.36, 0.05, 0.05), 0.012);
            k.block(TOP, FRAME, Vec3::new(0.0, seat, 0.0), Vec3::new(s.x, 0.06, s.y), 0.02);
        }
        Kind::Table => {
            let top = 0.71;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                // A turned leg: a foot, a swell, a neck and a block top.
                k.loft(
                    FRAME,
                    Vec3::new(x * (hx - 0.1), 0.0, z * (hz - 0.1)),
                    Vec3::Y,
                    &[(0.0, 0.026), (0.03, 0.03), (0.09, 0.024), (0.3, 0.038), (0.5, 0.03), (0.6, 0.033), (top - 0.08, 0.04), (top + 0.005, 0.04)],
                    8,
                    std::f32::consts::FRAC_PI_8,
                );
            }
            // The apron under the top, and the top over it.
            k.block(FRAME, FRAME, Vec3::new(0.0, top - 0.1, 0.0), Vec3::new(s.x - 0.18, 0.1, s.y - 0.18), 0.012);
            k.block(TOP, FRAME, Vec3::new(0.0, top, 0.0), Vec3::new(s.x, 0.05, s.y), 0.02);
        }
        Kind::Bed => {
            let rail = 0.22;
            // Corner posts, the head's taller.
            for (z, h) in [(-hz + 0.04, 0.95), (hz - 0.04, 0.6)] {
                for x in [-1.0f32, 1.0] {
                    k.loft(FRAME, Vec3::new(x * (hx - 0.04), 0.0, z), Vec3::Y, &[(0.0, 0.03), (0.03, 0.036), (h - 0.08, 0.038), (h - 0.03, 0.03), (h, 0.0)], 8, std::f32::consts::FRAC_PI_8);
                }
            }
            // The rails and the head and foot boards between the posts.
            for x in [-1.0f32, 1.0] {
                k.block(FRAME, FRAME, Vec3::new(x * (hx - 0.04), 0.12, 0.0), Vec3::new(0.05, 0.16, s.y - 0.12), 0.012);
            }
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.12, -hz + 0.04), Vec3::new(s.x - 0.1, 0.75, 0.05), 0.018);
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.12, hz - 0.04), Vec3::new(s.x - 0.1, 0.4, 0.05), 0.018);
            // The mattress, soft at every edge, and the blanket over its foot
            // end, standing proud of it and falling over its sides.
            k.block(SHEET, SHEET, Vec3::new(0.0, rail, 0.0), Vec3::new(s.x - 0.1, 0.24, s.y - 0.12), 0.07);
            k.block(BLANKET, BLANKET, Vec3::new(0.0, rail + 0.06, 0.24), Vec3::new(s.x - 0.06, 0.205, 1.28), 0.07);
            // The pillow, plump.
            k.block(SHEET, SHEET, Vec3::new(0.0, rail + 0.22, -hz + 0.27), Vec3::new(0.62, 0.12, 0.34), 0.055);
        }
        Kind::Drawers => {
            let h = 0.86;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                k.loft(FRAME, Vec3::new(x * (hx - 0.06), 0.0, z * (hz - 0.06)), Vec3::Y, &[(0.0, 0.026), (0.03, 0.036), (0.06, 0.03)], 8, 0.0);
            }
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.06, -0.01), Vec3::new(s.x - 0.04, h - 0.06, s.y - 0.04), 0.016);
            k.block(TOP, FRAME, Vec3::new(0.0, h, 0.0), Vec3::new(s.x, 0.04, s.y), 0.014);
            // Three drawer fronts, each a raised panel with a turned knob.
            for j in 0..3 {
                let y = 0.1 + j as f32 * 0.255;
                k.block(TOP, TOP, Vec3::new(0.0, y, hz - 0.03), Vec3::new(s.x - 0.1, 0.22, 0.03), 0.01);
                for x in [-0.22f32, 0.22] {
                    k.loft(FRAME, Vec3::new(x, y + 0.11, hz - 0.016), Vec3::Z, &[(0.0, 0.014), (0.022, 0.02), (0.032, 0.014), (0.036, 0.0)], 8, 0.0);
                }
            }
        }
        Kind::Closet => {
            let h = 1.95;
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.0, 0.0), Vec3::new(s.x - 0.02, 0.1, s.y - 0.02), 0.02);
            k.block(FRAME, FRAME, Vec3::new(0.0, 0.1, -0.01), Vec3::new(s.x - 0.06, h - 0.18, s.y - 0.06), 0.018);
            // The cornice, stepped out over the carcass.
            k.block(FRAME, FRAME, Vec3::new(0.0, h - 0.1, 0.0), Vec3::new(s.x - 0.02, 0.05, s.y - 0.02), 0.015);
            k.block(TOP, FRAME, Vec3::new(0.0, h - 0.05, 0.0), Vec3::new(s.x, 0.05, s.y), 0.02);
            // Two doors, raised panels, and their iron ring pulls.
            for x in [-1.0f32, 1.0] {
                k.block(TOP, TOP, Vec3::new(x * 0.245, 0.18, hz - 0.04), Vec3::new(0.46, h - 0.36, 0.03), 0.012);
                k.loft(IRON, Vec3::new(x * 0.05, 1.0, hz - 0.025), Vec3::Z, &[(0.0, 0.012), (0.03, 0.016), (0.04, 0.0)], 6, 0.0);
            }
        }
        Kind::Chest | Kind::BandedChest => {
            let body = if kind == Kind::Chest { 0.38 } else { 0.42 };
            let (edge, rise) = (0.04, s.y * 0.2);
            k.block(BOARDS, FRAME, Vec3::new(0.0, 0.0, 0.0), Vec3::new(s.x, body, s.y), 0.02);
            k.run_x(TOP, FRAME, &crown(s.y, body, edge, rise, 0.0), -hx + 0.005, hx - 0.005);
            // The bands wrap the body and the lid's curve: two of timber on
            // the plain chest, four of iron and iron corners on the banded.
            let (band, xs): (&'static str, Vec<f32>) = if kind == Kind::Chest {
                (FRAME, vec![-hx + 0.16, hx - 0.16])
            } else {
                (IRON, vec![-hx + 0.08, -0.15, 0.15, hx - 0.08])
            };
            for x in xs {
                k.block(band, band, Vec3::new(x, -0.002, 0.0), Vec3::new(0.06, body, s.y + 0.016), 0.006);
                k.run_x(band, band, &crown(s.y, body, edge, rise, 0.008), x - 0.03, x + 0.03);
            }
            // The lock: a plate on the front where the lid meets it, and its hasp.
            k.block(IRON, IRON, Vec3::new(0.0, body - 0.13, hz), Vec3::new(0.09, 0.12, 0.02), 0.006);
            k.block(IRON, IRON, Vec3::new(0.0, body - 0.02, hz + 0.004), Vec3::new(0.04, 0.09, 0.018), 0.006);
            if kind == Kind::BandedChest {
                for x in [-1.0f32, 1.0] {
                    k.loft(IRON, Vec3::new(x * hx, body * 0.6, 0.0), Vec3::new(x, 0.0, 0.0), &[(0.0, 0.035), (0.02, 0.035), (0.03, 0.0)], 6, 0.0);
                }
            }
        }
        Kind::Crate => {
            k.block(BOARDS, BOARDS, Vec3::new(0.0, 0.0, 0.0), Vec3::new(s.x - 0.03, s.y - 0.03, s.y - 0.03), 0.015);
            // Its frame: battens along every edge, set into the boards, and a
            // brace across the front and back.
            let e = s.x / 2.0 - 0.025;
            for (x, z) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                k.block(FRAME, FRAME, Vec3::new(x * e, 0.0, z * e), Vec3::new(0.06, s.x, 0.06), 0.012);
            }
            for y in [0.0f32, s.x - 0.06] {
                for z in [-1.0f32, 1.0] {
                    k.block(FRAME, FRAME, Vec3::new(0.0, y, z * e), Vec3::new(s.x - 0.1, 0.06, 0.055), 0.012);
                    k.block(FRAME, FRAME, Vec3::new(z * e, y, 0.0), Vec3::new(0.055, 0.06, s.x - 0.1), 0.012);
                }
            }
        }
    }
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
        })
        .collect()
}
