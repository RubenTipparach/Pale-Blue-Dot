//! The carts' and the horse's meshes (`land-transport` design 10 and 11),
//! built in code as shaped forms in the game's blocky low-poly look, never
//! boxes left standing on a surface (CLAUDE.md, "No obvious primitives"):
//!
//! - a timber is a lofted section with its edges chamfered, and where two
//!   meet they overlap into each other, so the joint reads as a joint;
//! - a wheel is a hub swept along its axle, spokes that taper out of the hub
//!   into the rim, and a rim swept round the wheel as one ring with its iron
//!   tyre on its outer face;
//! - the horse is one lofted body; its neck, head, legs and tail are swept
//!   out of it from rings that start inside it, so each grows from the body
//!   and tapers away, with no seam standing proud.
//!
//! Model space is the craft's: +X right, +Y up, -Z forward, the origin on the
//! ground midway between the wheels. Everything is flat-shaded with vertex
//! colours (`husbandry::rig::Builder`), and the meshes are drawn lit as the
//! terrain is (`LitByField`, `LitLikeTerrain` on the root).

use crate::husbandry::rig::Builder;
use bevy::prelude::*;
use pbd_core::vehicle::cart::{CartKind, CartSpec};
use std::f32::consts::{PI, TAU};

/// A chamfered square: a timber's section, its corners cut at a quarter.
const TIMBER: [(f32, f32); 8] = [
    (1.0, 0.55),
    (0.55, 1.0),
    (-0.55, 1.0),
    (-1.0, 0.55),
    (-1.0, -0.55),
    (-0.55, -1.0),
    (0.55, -1.0),
    (1.0, -0.55),
];

/// Eight sides round: a rounded form's section (a body, a leg, a hub).
fn round(n: usize) -> Vec<(f32, f32)> {
    (0..n)
        .map(|k| {
            let a = TAU * (k as f32 + 0.5) / n as f32;
            (a.cos(), a.sin())
        })
        .collect()
}

/// sRGB colour to the linear vertex colour the builder writes.
fn lin(c: [f32; 3]) -> [f32; 4] {
    let [r, g, b, _] = Color::srgb(c[0], c[1], c[2]).to_linear().to_f32_array();
    [r, g, b, 1.0]
}

/// One flat triangle, wound so its normal faces away from `inside`.
fn tri(out: &mut Builder, a: Vec3, b: Vec3, c: Vec3, inside: Vec3, colour: [f32; 4]) {
    let mut n = (b - a).cross(c - a);
    if n.length_squared() < 1e-14 {
        return;
    }
    let (b, c) = if n.dot((a + b + c) / 3.0 - inside) < 0.0 {
        n = -n;
        (c, b)
    } else {
        (b, c)
    };
    let n = n.normalize();
    let base = out.positions.len() as u32;
    for p in [a, b, c] {
        out.positions.push(p.to_array());
        out.normals.push(n.to_array());
        out.colours.push(colour);
        out.uvs.push([0.0, 0.0]);
    }
    out.indices.extend_from_slice(&[base, base + 1, base + 2]);
}

/// A ring of a loft: a centre, two axes across it, its half sizes along
/// them, and a colour for the faces from it to the next ring.
#[derive(Clone, Copy)]
pub struct Ring {
    pub centre: Vec3,
    pub u: Vec3,
    pub v: Vec3,
    pub ru: f32,
    pub rv: f32,
}

impl Ring {
    fn points(&self, profile: &[(f32, f32)]) -> Vec<Vec3> {
        profile
            .iter()
            .map(|&(x, y)| self.centre + self.u * (x * self.ru) + self.v * (y * self.rv))
            .collect()
    }
}

/// Sweep `profile` through `rings`, capping both ends (or joining the last
/// to the first, `closed`). `colour` picks each face's colour from its
/// outward normal, so a part's top (a mane) or outer face (a tyre) takes its
/// own colour from the one form.
fn loft(
    out: &mut Builder,
    rings: &[Ring],
    profile: &[(f32, f32)],
    closed: bool,
    colour: &dyn Fn(Vec3) -> [f32; 3],
) {
    if rings.len() < 2 {
        return;
    }
    let pts: Vec<Vec<Vec3>> = rings.iter().map(|r| r.points(profile)).collect();
    let n = profile.len();
    let spans = if closed { rings.len() } else { rings.len() - 1 };
    for i in 0..spans {
        let j = (i + 1) % rings.len();
        let inside = (rings[i].centre + rings[j].centre) * 0.5;
        for k in 0..n {
            let l = (k + 1) % n;
            let (a, b, c, d) = (pts[i][k], pts[i][l], pts[j][l], pts[j][k]);
            let normal = ((a + b + c + d) * 0.25 - inside).normalize_or(Vec3::Y);
            let col = lin(colour(normal));
            tri(out, a, b, c, inside, col);
            tri(out, a, c, d, inside, col);
        }
    }
    if !closed {
        for (ring, p, toward) in [
            (rings[0], &pts[0], rings[1].centre),
            (
                rings[rings.len() - 1],
                &pts[rings.len() - 1],
                rings[rings.len() - 2].centre,
            ),
        ] {
            let normal = (ring.centre - toward).normalize_or(Vec3::Y);
            let col = lin(colour(normal));
            let inside = ring.centre * 2.0 - (ring.centre + normal);
            for k in 0..n {
                tri(out, ring.centre, p[k], p[(k + 1) % n], inside, col);
            }
        }
    }
}

/// A chamfered timber from `a` to `b`, `w` wide and `h` deep (half sizes),
/// its `up` the timber's top. Its ends are cut square with their chamfer.
fn timber(out: &mut Builder, a: Vec3, b: Vec3, w: f32, h: f32, up: Vec3, colour: [f32; 3]) {
    let along = (b - a).normalize_or(Vec3::Z);
    let side = up.cross(along).normalize_or(Vec3::X);
    let top = along.cross(side).normalize_or(Vec3::Y);
    let ring = |c| Ring {
        centre: c,
        u: side,
        v: top,
        ru: w,
        rv: h,
    };
    loft(out, &[ring(a), ring(b)], &TIMBER, false, &|_| colour);
}

/// Wood and iron.
const OAK: [f32; 3] = [0.52, 0.37, 0.22];
const OAK_DARK: [f32; 3] = [0.38, 0.26, 0.15];
const PINE: [f32; 3] = [0.64, 0.49, 0.31];
const IRON: [f32; 3] = [0.22, 0.21, 0.2];

/// A wheel at its centre `at`, its axle along X, `radius` to the tyre's
/// face, `spokes` of them, turned by `turn` rad about the axle.
pub fn wheel(out: &mut Builder, at: Vec3, radius: f32, spokes: usize, turn: f32) {
    let width = radius * 0.16;
    let rim = radius * 0.11;
    let segments = 16;
    // The rim: one ring swept round, its outer face the iron tyre.
    let rings: Vec<Ring> = (0..segments)
        .map(|k| {
            let a = turn + TAU * k as f32 / segments as f32;
            let radial = Vec3::new(0.0, a.cos(), a.sin());
            Ring {
                centre: at + radial * (radius - rim),
                u: radial,
                v: Vec3::X,
                ru: rim,
                rv: width,
            }
        })
        .collect();
    loft(out, &rings, &TIMBER, true, &|n| {
        let radial = Vec3::new(0.0, n.y, n.z).normalize_or(Vec3::Y);
        if n.dot(radial) > 0.6 { IRON } else { OAK }
    });
    // The hub: swept along the axle, swelling in its middle.
    let hub = radius * 0.17;
    let hub_rings: Vec<Ring> = [(-1.45, 0.55), (-0.9, 0.95), (0.9, 0.95), (1.45, 0.55)]
        .into_iter()
        .map(|(x, r)| Ring {
            centre: at + Vec3::X * (x * width),
            u: Vec3::Y,
            v: Vec3::Z,
            ru: hub * r,
            rv: hub * r,
        })
        .collect();
    loft(out, &hub_rings, &round(8), false, &|n| {
        if n.x.abs() > 0.8 { IRON } else { OAK_DARK }
    });
    // The spokes: from inside the hub, tapering out into the rim.
    for s in 0..spokes {
        let a = turn + TAU * (s as f32 + 0.5) / spokes as f32;
        let radial = Vec3::new(0.0, a.cos(), a.sin());
        let across = Vec3::X.cross(radial);
        let ring = |d: f32, r: f32| Ring {
            centre: at + radial * d,
            u: across,
            v: Vec3::X,
            ru: r,
            rv: r * 0.7,
        };
        loft(
            out,
            &[
                ring(hub * 0.6, radius * 0.05),
                ring(radius - rim * 1.2, radius * 0.03),
            ],
            &round(6),
            false,
            &|_| PINE,
        );
    }
}

/// How a cart is drawn this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct CartPose {
    /// The wheels' turn, rad.
    pub wheel_turn: f32,
    /// Its tilt about the axle, rad, nose down: a hand cart let go rests on
    /// its handles' ends.
    pub tilt: f32,
}

/// The tilt a hand cart rests at when let go: its handles' ends on the
/// ground.
pub fn rest_tilt(spec: &CartSpec) -> f32 {
    // Held level the grips stand `HAND_GRIP_M` up, `reach_m` ahead of the
    // axle; turned down about the axle until they touch the ground.
    -(HAND_GRIP_M / spec.reach_m).atan()
}

/// The height of a hand cart's grips held level, m.
pub const HAND_GRIP_M: f32 = 0.82;

/// A cart's body and wheels, in model space.
pub fn cart(out: &mut Builder, kind: CartKind, spec: &CartSpec, pose: CartPose) {
    let r = spec.wheel_radius_m;
    let axle = Vec3::new(0.0, r, 0.0);
    // Everything but the wheels turns about the axle by the tilt.
    let tilt = Quat::from_rotation_x(pose.tilt);
    let t = |p: Vec3| axle + tilt * (p - axle);
    let up = tilt * Vec3::Y;
    let track = spec.track_m;
    match kind {
        CartKind::HandCart => {
            let bed_y = r + 0.14;
            let (half_w, front, back) = (track - 0.12, -0.55, 0.75);
            // The floor: planks run along, their joints cut as grooves.
            let planks = 4;
            for p in 0..planks {
                let x = -half_w + (p as f32 + 0.5) * 2.0 * half_w / planks as f32;
                let w = half_w / planks as f32 * 0.96;
                timber(
                    out,
                    t(Vec3::new(x, bed_y, back)),
                    t(Vec3::new(x, bed_y, front)),
                    w,
                    0.03,
                    up,
                    if p % 2 == 0 { PINE } else { OAK },
                );
            }
            // Low sides and a tailboard, overlapping at the corners.
            for side in [-1.0f32, 1.0] {
                timber(
                    out,
                    t(Vec3::new(side * half_w, bed_y + 0.13, back + 0.03)),
                    t(Vec3::new(side * half_w, bed_y + 0.13, front - 0.03)),
                    0.03,
                    0.11,
                    up,
                    OAK,
                );
            }
            timber(
                out,
                t(Vec3::new(-half_w - 0.02, bed_y + 0.13, back)),
                t(Vec3::new(half_w + 0.02, bed_y + 0.13, back)),
                0.11,
                0.03,
                tilt * Vec3::Z,
                OAK,
            );
            // Two bearers under the floor run out forward into the handles,
            // rising to the grips: one timber each, so the handle grows out
            // of the bed.
            let grip = Vec3::new(0.0, HAND_GRIP_M, -spec.reach_m);
            for side in [-1.0f32, 1.0] {
                let x = side * (half_w - 0.06);
                let rear = Vec3::new(x, bed_y - 0.06, back - 0.05);
                let mid = Vec3::new(x, bed_y - 0.06, front - 0.05);
                let end = Vec3::new(side * 0.3, grip.y, grip.z);
                timber(out, t(rear), t(mid), 0.04, 0.04, up, OAK_DARK);
                let rings = [
                    Ring {
                        centre: t(mid + Vec3::Z * 0.05),
                        u: tilt * Vec3::X,
                        v: up,
                        ru: 0.04,
                        rv: 0.04,
                    },
                    Ring {
                        centre: t(mid.lerp(end, 0.5)),
                        u: tilt * Vec3::X,
                        v: up,
                        ru: 0.034,
                        rv: 0.034,
                    },
                    Ring {
                        centre: t(end),
                        u: tilt * Vec3::X,
                        v: up,
                        ru: 0.028,
                        rv: 0.028,
                    },
                ];
                loft(out, &rings, &round(6), false, &|_| OAK_DARK);
            }
            // The axle under it, into both hubs.
            timber(
                out,
                Vec3::new(-track - 0.05, r, 0.0),
                Vec3::new(track + 0.05, r, 0.0),
                0.05,
                0.05,
                Vec3::Y,
                OAK_DARK,
            );
            for side in [-1.0f32, 1.0] {
                wheel(out, Vec3::new(side * track, r, 0.0), r, 8, pose.wheel_turn);
            }
        }
        CartKind::HorseCart => {
            let bed_y = r + 0.18;
            let (half_w, front, back) = (track - 0.15, -0.95, 1.05);
            let planks = 6;
            for p in 0..planks {
                let x = -half_w + (p as f32 + 0.5) * 2.0 * half_w / planks as f32;
                timber(
                    out,
                    Vec3::new(x, bed_y, back),
                    Vec3::new(x, bed_y, front),
                    half_w / planks as f32 * 0.96,
                    0.035,
                    Vec3::Y,
                    if p % 2 == 0 { PINE } else { OAK },
                );
            }
            // Sides of two boards with a post at each corner and midway, the
            // boards let into the posts.
            for side in [-1.0f32, 1.0] {
                for (y, h) in [(0.16, 0.08), (0.34, 0.07)] {
                    timber(
                        out,
                        Vec3::new(side * half_w, bed_y + y, back + 0.04),
                        Vec3::new(side * half_w, bed_y + y, front - 0.04),
                        0.03,
                        h,
                        Vec3::Y,
                        OAK,
                    );
                }
                for z in [back, 0.05, front] {
                    timber(
                        out,
                        Vec3::new(side * half_w, bed_y - 0.02, z),
                        Vec3::new(side * half_w, bed_y + 0.46, z),
                        0.045,
                        0.045,
                        Vec3::Z,
                        OAK_DARK,
                    );
                }
            }
            for (y, h) in [(0.16, 0.08), (0.34, 0.07)] {
                timber(
                    out,
                    Vec3::new(-half_w - 0.03, bed_y + y, back),
                    Vec3::new(half_w + 0.03, bed_y + y, back),
                    h,
                    0.03,
                    Vec3::Z,
                    OAK,
                );
            }
            // The seat at the front: a bench on two sloped risers.
            let seat_y = bed_y + 0.55;
            let seat_z = front + 0.25;
            for side in [-1.0f32, 1.0] {
                timber(
                    out,
                    Vec3::new(side * (half_w - 0.08), bed_y, seat_z + 0.18),
                    Vec3::new(side * (half_w - 0.08), seat_y, seat_z),
                    0.04,
                    0.05,
                    Vec3::Z,
                    OAK_DARK,
                );
            }
            timber(
                out,
                Vec3::new(-half_w + 0.02, seat_y, seat_z),
                Vec3::new(half_w - 0.02, seat_y, seat_z),
                0.17,
                0.035,
                Vec3::NEG_Z,
                PINE,
            );
            // A footboard, sloped, in front of the seat.
            timber(
                out,
                Vec3::new(-half_w + 0.05, bed_y + 0.12, front - 0.12),
                Vec3::new(half_w - 0.05, bed_y + 0.12, front - 0.12),
                0.14,
                0.025,
                Vec3::new(0.0, 0.8, -0.6),
                OAK,
            );
            // The shafts: from under the bed out forward past the horse's
            // shoulders, a long taper with a curve up at their tips.
            let tip_z = -spec.reach_m - 0.35;
            for side in [-1.0f32, 1.0] {
                let x = side * (half_w - 0.08);
                let out_x = side * 0.42;
                let pts = [
                    Vec3::new(x, bed_y - 0.07, back - 0.1),
                    Vec3::new(x, bed_y - 0.07, front),
                    Vec3::new((x + out_x) * 0.5, bed_y - 0.02, front - 0.9),
                    Vec3::new(out_x, 1.0, tip_z + 0.35),
                    Vec3::new(out_x, 1.08, tip_z),
                ];
                let sizes = [0.055, 0.055, 0.048, 0.04, 0.032];
                let rings: Vec<Ring> = pts
                    .iter()
                    .zip(sizes)
                    .map(|(&c, s)| Ring {
                        centre: c,
                        u: Vec3::X,
                        v: Vec3::Y,
                        ru: s,
                        rv: s * 1.1,
                    })
                    .collect();
                loft(out, &rings, &TIMBER, false, &|_| OAK_DARK);
            }
            // The axle bed between the wheels.
            timber(
                out,
                Vec3::new(-track - 0.06, r, 0.0),
                Vec3::new(track + 0.06, r, 0.0),
                0.07,
                0.07,
                Vec3::Y,
                OAK_DARK,
            );
            for side in [-1.0f32, 1.0] {
                wheel(out, Vec3::new(side * track, r, 0.0), r, 10, pose.wheel_turn);
            }
        }
    }
}

/// How the horse stands this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct HorsePose {
    /// The gait's phase, rad: ground distance over its stride (so the feet
    /// never slide), a full cycle of all four feet.
    pub phase: f32,
    /// How far its legs swing, 0 standing to 1 a full walk.
    pub walk: f32,
    /// Its head down to graze, rad.
    pub head_down: f32,
    /// Its tail's swish, rad.
    pub tail: f32,
}

/// The horse's colours: coat, muzzle and belly, legs and mane, hooves.
#[derive(Clone, Copy, Debug)]
pub struct HorseColours {
    pub coat: [f32; 3],
    pub pale: [f32; 3],
    pub dark: [f32; 3],
    pub hoof: [f32; 3],
}

impl Default for HorseColours {
    /// A bay: a brown coat with black points.
    fn default() -> Self {
        Self {
            coat: [0.45, 0.28, 0.16],
            pale: [0.62, 0.46, 0.32],
            dark: [0.16, 0.11, 0.08],
            hoof: [0.12, 0.1, 0.09],
        }
    }
}

/// The horse, its feet on the ground at the origin, facing -Z, scaled by
/// `size` (1 a grown horse: 1.55 m at the withers).
pub fn horse(out: &mut Builder, c: &HorseColours, pose: HorsePose, size: f32) {
    let s = size;
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let walk = pose.walk.clamp(0.0, 1.0);
    // The walk is a four-beat gait: the hind and fore feet of one side a
    // quarter cycle apart. The hips rise and fall twice a cycle.
    let bob = (pose.phase * 2.0).cos() * 0.025 * walk;
    let lift = bob * s;
    let coat = c.coat;
    let dark = c.dark;
    let pale = c.pale;
    // The body: barrel from rump to chest, deepest at the girth.
    let body: Vec<Ring> = [
        (0.95, 1.20, 0.17, 0.2),
        (0.82, 1.22, 0.27, 0.29),
        (0.55, 1.2, 0.31, 0.34),
        (0.15, 1.17, 0.32, 0.37),
        (-0.3, 1.19, 0.3, 0.36),
        (-0.62, 1.24, 0.25, 0.33),
        (-0.8, 1.3, 0.18, 0.26),
    ]
    .into_iter()
    .map(|(z, y, w, h)| Ring {
        centre: v(0.0, y, z) + Vec3::Y * lift,
        u: Vec3::X,
        v: Vec3::Y,
        ru: w * s,
        rv: h * s,
    })
    .collect();
    let belly = |n: Vec3| if n.y < -0.7 { pale } else { coat };
    loft(out, &body, &round(10), false, &belly);
    // The neck rises out of the chest to the poll, its crest the mane.
    let nod = (pose.phase * 2.0 + 0.6).sin() * 0.06 * walk;
    let head_pitch = pose.head_down + nod;
    let neck_root = v(0.0, 1.32, -0.62) + Vec3::Y * lift;
    let neck_dir = Quat::from_rotation_x(head_pitch) * Vec3::new(0.0, 0.62, -0.5).normalize();
    let neck: Vec<Ring> = [
        (0.0, 0.2, 0.27),
        (0.35, 0.16, 0.22),
        (0.7, 0.12, 0.17),
        (0.84, 0.1, 0.15),
    ]
    .into_iter()
    .map(|(t, w, h)| {
        let centre = neck_root + neck_dir * (t * s);
        let fwd = neck_dir.cross(Vec3::X).normalize();
        Ring {
            centre,
            u: Vec3::X,
            v: -fwd,
            ru: w * s,
            rv: h * s,
        }
    })
    .collect();
    let crest = neck_dir.cross(Vec3::X).normalize();
    loft(out, &neck, &round(8), false, &|n| {
        if n.dot(-crest) > 0.75 { dark } else { coat }
    });
    // The head: from the poll forward and down to the muzzle.
    let poll = neck_root + neck_dir * (0.8 * s);
    let head_dir = Quat::from_rotation_x(head_pitch) * Vec3::new(0.0, -0.62, -0.78).normalize();
    let head_up = head_dir.cross(Vec3::X).normalize();
    let head: Vec<Ring> = [
        (-0.04, 0.1, 0.13),
        (0.18, 0.11, 0.12),
        (0.38, 0.08, 0.09),
        (0.5, 0.075, 0.07),
    ]
    .into_iter()
    .map(|(t, w, h)| Ring {
        centre: poll + head_dir * (t * s),
        u: Vec3::X,
        v: -head_up,
        ru: w * s,
        rv: h * s,
    })
    .collect();
    let muzzle = poll + head_dir * (0.38 * s);
    loft(out, &head, &round(8), false, &|n| {
        let _ = n;
        coat
    });
    // The muzzle's soft end and the eyes, painted on the head's own faces
    // rather than stuck on: a darker loft just over the nose's end.
    loft(
        out,
        &[
            Ring {
                centre: muzzle,
                u: Vec3::X,
                v: -head_up,
                ru: 0.082 * s,
                rv: 0.092 * s,
            },
            Ring {
                centre: poll + head_dir * (0.53 * s),
                u: Vec3::X,
                v: -head_up,
                ru: 0.07 * s,
                rv: 0.065 * s,
            },
        ],
        &round(8),
        false,
        &|_| dark,
    );
    // Ears from the poll, tapering to points.
    for side in [-1.0f32, 1.0] {
        let base = poll + Vec3::X * (side * 0.06 * s) - head_up * (0.06 * s);
        let tip = base + (-head_up * 0.13 + Vec3::X * side * 0.03 - head_dir * 0.02) * s;
        loft(
            out,
            &[
                Ring {
                    centre: base,
                    u: Vec3::X,
                    v: head_dir,
                    ru: 0.03 * s,
                    rv: 0.02 * s,
                },
                Ring {
                    centre: tip,
                    u: Vec3::X,
                    v: head_dir,
                    ru: 0.006 * s,
                    rv: 0.005 * s,
                },
            ],
            &round(5),
            false,
            &|_| coat,
        );
    }
    // The tail from the top of the rump, hanging and swishing.
    let tail_root = v(0.0, 1.36, 0.92) + Vec3::Y * lift;
    let swish = Quat::from_rotation_y(pose.tail + (pose.phase).sin() * 0.08 * walk);
    let tail: Vec<Ring> = [(0.0, 0.06), (0.25, 0.075), (0.55, 0.065), (0.78, 0.03)]
        .into_iter()
        .map(|(t, r)| {
            let dir = swish * Vec3::new(0.0, -0.85, 0.32).normalize();
            Ring {
                centre: tail_root + dir * (t * s) + Vec3::Z * (0.1 * t * t * s),
                u: Vec3::X,
                v: Vec3::Z,
                ru: r * s,
                rv: r * s,
            }
        })
        .collect();
    loft(out, &tail, &round(6), false, &|_| dark);
    // Legs: each swept from inside the body down through the knee or hock,
    // the cannon and the fetlock to the hoof. A leg swings about its top
    // and lifts its hoof as it comes through.
    for (x, z, offset, fore) in [
        (-1.0f32, 0.62, 0.0, false),
        (-1.0, -0.55, 0.25, true),
        (1.0, 0.62, 0.5, false),
        (1.0, -0.55, 0.75, true),
    ] {
        let p = pose.phase + offset * TAU;
        let swing = p.sin() * 0.32 * walk;
        // The hoof lifts in the swing's forward half, on an arc.
        let lift_leg = (p.cos().max(0.0)) * 0.5 * walk;
        let top = v(x * 0.17, 1.12, z) + Vec3::Y * lift;
        let turn = Quat::from_rotation_x(swing);
        let joint_y = if fore { 0.5 } else { 0.58 };
        // The knee bends back on a foreleg, the hock forward on a hind.
        let bend = if fore { -lift_leg } else { lift_leg * 0.8 };
        let joint =
            top + turn * Vec3::new(0.0, (joint_y - 1.12) * s, if fore { 0.0 } else { 0.08 * s });
        let below = turn * Quat::from_rotation_x(bend);
        let fetlock = joint + below * Vec3::new(0.0, -(joint_y - 0.12) * s, 0.0);
        let hoof = fetlock + below * Vec3::new(0.0, -0.1 * s, -0.03 * s);
        let ring = |centre: Vec3, w: f32, h: f32| Ring {
            centre,
            u: Vec3::X,
            v: Vec3::Z,
            ru: w * s,
            rv: h * s,
        };
        let upper = [
            ring(top + Vec3::Y * (0.14 * s), 0.13, 0.22),
            ring(top.lerp(joint, 0.3), 0.1, if fore { 0.13 } else { 0.18 }),
            ring(top.lerp(joint, 0.7), 0.07, if fore { 0.09 } else { 0.11 }),
            ring(joint, 0.055, 0.07),
        ];
        loft(out, &upper, &round(8), false, &|_| coat);
        let lower = [
            ring(joint + Vec3::Y * (0.03 * s), 0.055, 0.065),
            ring(joint.lerp(fetlock, 0.5), 0.04, 0.05),
            ring(fetlock, 0.05, 0.055),
        ];
        loft(out, &lower, &round(8), false, &|_| dark);
        let foot = [ring(fetlock, 0.05, 0.055), ring(hoof, 0.065, 0.075)];
        loft(out, &foot, &round(8), false, &|_| c.hoof);
    }
    // The collar and its traces lie on the coat: a dark band round the
    // shoulders, fitted to the neck's root.
    let collar: Vec<Ring> = [0.02f32, 0.12]
        .into_iter()
        .map(|t| {
            let centre = neck_root + neck_dir * (t * s);
            Ring {
                centre,
                u: Vec3::X,
                v: -neck_dir.cross(Vec3::X).normalize(),
                ru: 0.215 * s,
                rv: 0.285 * s,
            }
        })
        .collect();
    let _ = PI;
    loft(out, &collar, &round(10), false, &|_| [0.3, 0.17, 0.09]);
}

/// The colours a kept horse's genome gives: its coat over the bay's points.
pub fn colours_of(
    species: &pbd_core::husbandry::LandSpecies,
    genome: &pbd_core::husbandry::Genome,
) -> HorseColours {
    let (c, _) = crate::husbandry::rig::colours(species, genome);
    HorseColours {
        coat: c[0],
        pale: c[1],
        dark: c[2],
        hoof: [0.12, 0.1, 0.09],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mesh is whole: as many normals and colours as points, every
    /// face a triangle on points that are there, nothing not finite, and
    /// each model about the size the design gives it.
    #[test]
    fn the_carts_and_the_horse_are_whole_meshes_of_their_size() {
        let specs = pbd_core::vehicle::cart::CartSpecs::default();
        for kind in CartKind::ALL {
            let mut b = Builder::default();
            cart(&mut b, kind, specs.of(kind), CartPose::default());
            check(&b);
            let (lo, hi) = bounds(&b);
            let length = hi.z - lo.z;
            match kind {
                CartKind::HandCart => assert!((1.8..2.6).contains(&length), "{length}"),
                CartKind::HorseCart => assert!((3.5..5.5).contains(&length), "{length}"),
            }
            assert!(lo.y > -0.01, "nothing under the ground: {}", lo.y);
        }
        let mut b = Builder::default();
        horse(
            &mut b,
            &HorseColours::default(),
            HorsePose {
                walk: 1.0,
                phase: 1.0,
                ..default()
            },
            1.0,
        );
        check(&b);
        let (lo, hi) = bounds(&b);
        assert!((1.5..2.2).contains(&(hi.y - lo.y)), "{lo} {hi}");
        assert!((2.0..3.2).contains(&(hi.z - lo.z)), "{lo} {hi}");
    }

    fn check(b: &Builder) {
        assert!(!b.positions.is_empty());
        assert_eq!(b.positions.len(), b.normals.len());
        assert_eq!(b.positions.len(), b.colours.len());
        assert_eq!(b.indices.len() % 3, 0);
        assert!(b.indices.iter().all(|&i| (i as usize) < b.positions.len()));
        assert!(b.positions.iter().flatten().all(|v| v.is_finite()));
    }

    fn bounds(b: &Builder) -> (Vec3, Vec3) {
        b.positions
            .iter()
            .fold((Vec3::MAX, Vec3::MIN), |(lo, hi), p| {
                let p = Vec3::from(*p);
                (lo.min(p), hi.max(p))
            })
    }
}
