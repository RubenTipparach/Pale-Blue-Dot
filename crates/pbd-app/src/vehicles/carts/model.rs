//! The carts' meshes (`land-transport` designs 10 to 12), built in code as
//! shaped forms in the game's blocky low-poly look, never boxes left
//! standing on a surface (CLAUDE.md, "No obvious primitives"):
//!
//! - a timber is a lofted section with its edges chamfered, and where two
//!   meet they overlap into each other, so the joint reads as a joint; its
//!   chamfers are real creases, so its normals are hard;
//! - a wheel is a hub swept along its axle, round and smooth-shaded, spokes
//!   that taper out of the hub into the rim, and a rim swept round the wheel
//!   as one ring with its iron tyre on its outer face.
//!
//! Every piece is textured (design 12): wood in the settlement `timber`
//! texture, its grain laid along each piece by metres at the manifest's
//! `repeat_m`, iron in `iron`. A cart is two meshes, [`WOOD`] and
//! [`IRON_PART`]. The animal in the shafts is the herd's own form
//! (`husbandry::forms`).
//!
//! Model space is the craft's: +X right, +Y up, -Z forward, the origin on the
//! ground midway between the wheels. The meshes are drawn lit as the terrain
//! is (`LitByField`, `LitLikeTerrain` on the root).

use crate::husbandry::loft::{self, Ring, Skin, TIMBER};
use crate::husbandry::rig::Builder;
use bevy::prelude::*;
use pbd_core::vehicle::cart::{CartKind, CartSpec};
use std::f32::consts::TAU;

/// The cart's wood mesh, in `timber`.
pub const WOOD: usize = 0;
/// The cart's iron mesh, in `iron`.
pub const IRON_PART: usize = 1;
/// The settlement textures of [`WOOD`] and [`IRON_PART`].
pub const TEXTURES: [&str; 2] = ["timber", "iron"];

/// One metre of timber and of iron per repeat, as the manifest has them.
const REPEAT_M: f32 = 1.0;

/// A smooth form through `rings` (a handle, a spoke, a hub, a band).
fn round_wood(out: &mut [Builder], rings: &[Ring], sides: usize, part: usize, colour: [f32; 3]) {
    loft::smooth(
        &mut out[part],
        rings,
        sides,
        Skin::Metres(REPEAT_M),
        &|_, _, _| colour,
    );
}

/// A chamfered timber from `a` to `b`, `w` wide and `h` deep (half sizes),
/// its `up` the timber's top. Its ends are cut square with their chamfer.
fn timber(out: &mut [Builder], a: Vec3, b: Vec3, w: f32, h: f32, up: Vec3, colour: [f32; 3]) {
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
    loft::hard(out, REPEAT_M, &[ring(a), ring(b)], &TIMBER, false, &|_| {
        (WOOD, colour)
    });
}

/// Tints of the textures: oak a shade under the timber's own brown, pine
/// paler, the bearers and shafts darker; iron as it is.
const OAK: [f32; 3] = [0.9, 0.86, 0.8];
const OAK_DARK: [f32; 3] = [0.72, 0.66, 0.6];
const PINE: [f32; 3] = [1.0, 0.97, 0.9];
const IRON: [f32; 3] = [1.0, 1.0, 1.0];

/// A wheel at its centre `at`, its axle along X, `radius` to the tyre's
/// face, `spokes` of them, turned by `turn` rad about the axle.
pub fn wheel(out: &mut [Builder], at: Vec3, radius: f32, spokes: usize, turn: f32) {
    let width = radius * 0.16;
    let rim = radius * 0.11;
    let segments = 24;
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
    loft::hard(out, REPEAT_M, &rings, &TIMBER, true, &|n| {
        let radial = Vec3::new(0.0, n.y, n.z).normalize_or(Vec3::Y);
        if n.dot(radial) > 0.6 {
            (IRON_PART, IRON)
        } else {
            (WOOD, OAK)
        }
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
    round_wood(out, &hub_rings, 12, WOOD, OAK_DARK);
    // An iron band round each end of the hub, fitted to it.
    for end in [-1.0f32, 1.0] {
        let band: Vec<Ring> = [(0.95, 0.97), (1.5, 0.6)]
            .into_iter()
            .map(|(x, r)| Ring {
                centre: at + Vec3::X * (end * x * width),
                u: Vec3::Y,
                v: Vec3::Z,
                ru: hub * r,
                rv: hub * r,
            })
            .collect();
        round_wood(out, &band, 12, IRON_PART, IRON);
    }
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
        round_wood(
            out,
            &[
                ring(hub * 0.6, radius * 0.05),
                ring(radius - rim * 1.2, radius * 0.03),
            ],
            8,
            WOOD,
            PINE,
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
pub fn cart(out: &mut [Builder], kind: CartKind, spec: &CartSpec, pose: CartPose) {
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
                round_wood(out, &loft::curve(&rings, 3), 8, WOOD, OAK_DARK);
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
                loft::hard(
                    out,
                    REPEAT_M,
                    &loft::curve(&rings, 2),
                    &TIMBER,
                    false,
                    &|_| (WOOD, OAK_DARK),
                );
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

/// The animal in a cart's shafts, drawn by its species' own form.
pub fn puller(
    out: &mut Builder,
    species: Option<&pbd_core::husbandry::LandSpecies>,
    genome: &pbd_core::husbandry::Genome,
    stance: &crate::husbandry::forms::Stance,
) {
    use crate::husbandry::forms;
    let kind = species
        .and_then(|s| forms::Beast::of_id(&s.id))
        .unwrap_or(forms::Beast::Horse);
    let (coat, size) = match species {
        Some(sp) => (
            crate::husbandry::rig::coat(sp, genome),
            genome.size_factor() * sp.height_m / kind.withers_m(),
        ),
        None => (forms::Coat::default(), 1.0),
    };
    forms::beast(out, kind, &coat, stance, size);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mesh is whole: as many normals and colours as points, every
    /// face a triangle on points that are there, nothing not finite, and
    /// each model about the size the design gives it.
    #[test]
    fn the_carts_and_the_horse_are_whole_textured_meshes_of_their_size() {
        let specs = pbd_core::vehicle::cart::CartSpecs::default();
        for kind in CartKind::ALL {
            let mut b = [Builder::default(), Builder::default()];
            cart(&mut b, kind, specs.of(kind), CartPose::default());
            for part in &b {
                check(part);
                assert!(part.textured, "every part is textured");
                assert_eq!(part.uvs.len(), part.positions.len());
            }
            let (lo, hi) = bounds(&b[WOOD]);
            let length = hi.z - lo.z;
            match kind {
                CartKind::HandCart => assert!((1.8..2.6).contains(&length), "{length}"),
                CartKind::HorseCart => assert!((3.5..5.5).contains(&length), "{length}"),
            }
            assert!(lo.y > -0.01, "nothing under the ground: {}", lo.y);
        }
        let mut b = Builder::default();
        puller(
            &mut b,
            None,
            &pbd_core::husbandry::Genome::default(),
            &crate::husbandry::forms::Stance {
                walk: 1.0,
                phase: 1.0,
                ..default()
            },
        );
        check(&b);
        let (lo, hi) = bounds(&b);
        assert!((1.5..2.3).contains(&(hi.y - lo.y)), "{lo} {hi}");
        assert!((2.0..3.4).contains(&(hi.z - lo.z)), "{lo} {hi}");
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
