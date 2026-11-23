//! The herd's bodies as fitted forms (`animal-husbandry` design 16,
//! `land-transport` designs 11 and 12): one family of models.
//!
//! Every animal is a lofted, smooth-shaded body interpolated through a few
//! authored rings; its neck, head, legs and tail are swept out of it from
//! rings that start inside it and taper away, so each part grows from the
//! body (CLAUDE.md, "No obvious primitives"). It wears a tile of the `coat`
//! strip tinted by its genome's colours; a pattern, a muzzle and the eyes
//! are tints painted onto the body's own points, never parts stuck on.
//!
//! Model space is the carts': +X right, +Y up, -Z forward, the feet on the
//! ground at the origin. `husbandry::rig::build` turns it into rig space.

use super::loft::{Ring, Skin, Tile, curve, smooth};
use super::rig::Builder;
use bevy::prelude::*;
use pbd_core::husbandry::Sex;

/// How an animal stands this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stance {
    /// The gait's phase, rad: ground distance over its stride.
    pub phase: f32,
    /// How far its legs swing, 0 standing to 1 a full walk.
    pub walk: f32,
    /// Its head down to graze or peck, rad.
    pub head_down: f32,
    /// Its head turned, rad left of ahead.
    pub glance: f32,
    /// A bird's head forward and back with its steps, m at size 1.
    pub bob: f32,
    /// Its tail's lift (a bird) or swish (a beast), rad.
    pub tail: f32,
    /// Its weight shifted side to side, rad of roll.
    pub shift: f32,
    /// 0 standing to 1 lying down or roosting.
    pub down: f32,
    /// A rider on its back (`land-transport` design 14): a blanket painted
    /// on the coat where the rider sits.
    pub saddled: bool,
}

/// An animal's colours, sRGB: its body, its pale accent (a belly, a muzzle,
/// a belt, a comb), its legs and points, and its head.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coat {
    pub body: [f32; 3],
    pub pale: [f32; 3],
    pub legs: [f32; 3],
    pub head: [f32; 3],
    /// A pattern from the genome: 0 none.
    pub pattern: usize,
}

impl Default for Coat {
    /// A bay horse.
    fn default() -> Self {
        Self {
            body: [0.45, 0.28, 0.16],
            pale: [0.62, 0.46, 0.32],
            legs: [0.16, 0.11, 0.08],
            head: [0.40, 0.25, 0.15],
            pattern: 0,
        }
    }
}

/// The four-legged bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beast {
    Horse,
    Plodder,
    Woolback,
}

impl Beast {
    /// Its height at the withers at size 1, m.
    pub fn withers_m(self) -> f32 {
        match self {
            Beast::Horse => 1.55,
            Beast::Plodder => 1.35,
            Beast::Woolback => 0.8,
        }
    }

    /// The form a species id is drawn by.
    pub fn of_id(id: &str) -> Option<Beast> {
        match id {
            "horse" => Some(Beast::Horse),
            "plodder" => Some(Beast::Plodder),
            "woolback" => Some(Beast::Woolback),
            _ => None,
        }
    }
}

/// A beast's proportions, m at size 1.
struct Form {
    /// Body rings from rump to chest: z, y, half width, half height.
    body: &'static [(f32, f32, f32, f32)],
    body_tile: Tile,
    /// The neck's root (y, z), its direction (up, forward) and rings
    /// (share along, half width, half height); the poll at `poll`.
    neck_root: (f32, f32),
    neck_dir: (f32, f32),
    neck: &'static [(f32, f32, f32)],
    poll: f32,
    neck_tile: Tile,
    /// The head's direction from the poll and its rings; the muzzle from
    /// `muzzle` along it.
    head_dir: (f32, f32),
    head: &'static [(f32, f32, f32)],
    muzzle: f32,
    /// Ear length; an ear stands up (a horse) or out (a plodder, a sheep).
    ear: f32,
    ears_out: bool,
    /// Horn length, 0 for none.
    horn: f32,
    /// The tail's root (y, z), its direction (down, back) and rings
    /// (share along its length, radius); a long-haired switch from `switch`.
    tail_root: (f32, f32),
    tail_dir: (f32, f32),
    tail_len: f32,
    tail: &'static [(f32, f32)],
    switch: f32,
    /// Legs: half the stance's width, the fore and hind legs' z, where they
    /// leave the body, the knee and hock heights, and their thickness.
    leg_x: f32,
    fore_z: f32,
    hind_z: f32,
    leg_top: f32,
    fore_joint: f32,
    hind_joint: f32,
    leg_w: f32,
    /// A mane along the neck's crest.
    mane: bool,
}

const HORSE: Form = Form {
    body: &[
        (0.95, 1.20, 0.17, 0.20),
        (0.82, 1.22, 0.27, 0.29),
        (0.55, 1.20, 0.31, 0.34),
        (0.15, 1.17, 0.32, 0.37),
        (-0.30, 1.19, 0.30, 0.36),
        (-0.62, 1.24, 0.25, 0.33),
        (-0.80, 1.30, 0.18, 0.26),
    ],
    body_tile: Tile::Hair,
    neck_root: (1.32, -0.62),
    neck_dir: (0.62, -0.5),
    neck: &[
        (0.0, 0.2, 0.27),
        (0.35, 0.16, 0.22),
        (0.7, 0.12, 0.17),
        (0.86, 0.1, 0.15),
    ],
    poll: 0.8,
    neck_tile: Tile::Hair,
    head_dir: (-0.62, -0.78),
    head: &[
        (-0.04, 0.10, 0.13),
        (0.18, 0.11, 0.12),
        (0.38, 0.08, 0.09),
        (0.5, 0.075, 0.07),
        (0.54, 0.055, 0.05),
    ],
    muzzle: 0.38,
    ear: 0.13,
    ears_out: false,
    horn: 0.0,
    tail_root: (1.36, 0.92),
    tail_dir: (-0.85, 0.32),
    tail_len: 0.8,
    tail: &[(0.0, 0.06), (0.3, 0.075), (0.65, 0.065), (1.0, 0.025)],
    switch: 0.0,
    leg_x: 0.17,
    fore_z: -0.55,
    hind_z: 0.62,
    leg_top: 1.12,
    fore_joint: 0.5,
    hind_joint: 0.58,
    leg_w: 1.0,
    mane: true,
};

const PLODDER: Form = Form {
    body: &[
        (0.86, 0.98, 0.17, 0.20),
        (0.74, 1.00, 0.31, 0.32),
        (0.45, 0.97, 0.37, 0.38),
        (0.05, 0.93, 0.39, 0.43),
        (-0.35, 0.95, 0.37, 0.41),
        (-0.62, 1.02, 0.31, 0.35),
        (-0.77, 1.07, 0.21, 0.27),
    ],
    body_tile: Tile::Hair,
    neck_root: (1.05, -0.62),
    neck_dir: (0.3, -0.9),
    neck: &[(0.0, 0.22, 0.28), (0.3, 0.19, 0.25), (0.55, 0.16, 0.21)],
    poll: 0.5,
    neck_tile: Tile::Hair,
    head_dir: (-0.78, -0.62),
    head: &[
        (-0.04, 0.13, 0.15),
        (0.18, 0.135, 0.13),
        (0.36, 0.115, 0.1),
        (0.47, 0.105, 0.08),
        (0.51, 0.08, 0.055),
    ],
    muzzle: 0.36,
    ear: 0.1,
    ears_out: true,
    horn: 0.17,
    tail_root: (1.07, 0.85),
    tail_dir: (-0.95, 0.18),
    tail_len: 0.82,
    tail: &[
        (0.0, 0.045),
        (0.4, 0.03),
        (0.75, 0.028),
        (0.85, 0.055),
        (1.0, 0.02),
    ],
    switch: 0.78,
    leg_x: 0.21,
    fore_z: -0.5,
    hind_z: 0.6,
    leg_top: 0.82,
    fore_joint: 0.42,
    hind_joint: 0.47,
    leg_w: 1.2,
    mane: false,
};

const WOOLBACK: Form = Form {
    body: &[
        (0.5, 0.57, 0.13, 0.15),
        (0.42, 0.59, 0.25, 0.25),
        (0.2, 0.58, 0.3, 0.28),
        (-0.05, 0.57, 0.31, 0.29),
        (-0.28, 0.59, 0.28, 0.27),
        (-0.42, 0.63, 0.21, 0.22),
        (-0.5, 0.66, 0.12, 0.15),
    ],
    body_tile: Tile::Fleece,
    neck_root: (0.68, -0.42),
    neck_dir: (0.6, -0.8),
    neck: &[(0.0, 0.13, 0.15), (0.16, 0.1, 0.12), (0.26, 0.085, 0.095)],
    poll: 0.24,
    neck_tile: Tile::Fleece,
    head_dir: (-0.55, -0.83),
    head: &[
        (-0.03, 0.075, 0.085),
        (0.1, 0.075, 0.075),
        (0.2, 0.057, 0.055),
        (0.27, 0.045, 0.04),
        (0.29, 0.03, 0.028),
    ],
    muzzle: 0.22,
    ear: 0.08,
    ears_out: true,
    horn: 0.0,
    tail_root: (0.66, 0.48),
    tail_dir: (-0.7, 0.7),
    tail_len: 0.14,
    tail: &[(0.0, 0.06), (0.5, 0.05), (1.0, 0.025)],
    switch: 0.0,
    leg_x: 0.13,
    fore_z: -0.3,
    hind_z: 0.33,
    leg_top: 0.42,
    fore_joint: 0.22,
    hind_joint: 0.25,
    leg_w: 0.42,
    mane: false,
};

fn form(b: Beast) -> &'static Form {
    match b {
        Beast::Horse => &HORSE,
        Beast::Plodder => &PLODDER,
        Beast::Woolback => &WOOLBACK,
    }
}

fn dark(c: [f32; 3], k: f32) -> [f32; 3] {
    c.map(|x| x * k)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// A four-legged animal at its stance, scaled by `size` (1 the form's own).
pub fn beast(out: &mut Builder, kind: Beast, c: &Coat, pose: &Stance, size: f32) {
    let f = form(kind);
    let s = size;
    let walk = pose.walk.clamp(0.0, 1.0);
    let down = pose.down.clamp(0.0, 1.0);
    let k = f.leg_top / 1.12;
    // The hips rise and fall twice a cycle; lying, the body sinks onto its
    // folded legs.
    let bob = (pose.phase * 2.0).cos() * 0.025 * k * walk;
    let drop = f.leg_top * 0.72 * down;
    let lift = Vec3::Y * ((bob - drop) * s);
    // The weight shifted: a roll about the body's long axis at its middle.
    let roll_at = Vec3::Y * (f.leg_top * s) + lift;
    let roll = Quat::from_rotation_z(pose.shift);
    let rolled = |p: Vec3| roll_at + roll * (p - roll_at);
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let blanket = [0.42, 0.16, 0.12];
    // The body, rump to chest.
    let body: Vec<Ring> = f
        .body
        .iter()
        .map(|&(z, y, w, h)| Ring {
            centre: rolled(v(0.0, y, z) + lift),
            u: roll * Vec3::X,
            v: roll * Vec3::Y,
            ru: w * s,
            rv: h * s,
        })
        .collect();
    let pattern = c.pattern;
    smooth(
        out,
        &curve(&body, 3),
        16,
        Skin::Coat(f.body_tile),
        &|n, t, _| {
            let mut col = c.body;
            // A paler belly on a horse and a plodder.
            if kind != Beast::Woolback && n.y < -0.6 {
                col = mix(col, c.pale, 0.6);
            }
            // A belt round the middle.
            if pattern > 0 && (0.38..0.62).contains(&t) && kind != Beast::Horse {
                col = c.pale;
            }
            if pose.saddled && n.y > 0.35 && (0.42..0.72).contains(&t) {
                col = blanket;
            }
            col
        },
    );
    // The neck rises out of the chest to the poll.
    let nod = (pose.phase * 2.0 + 0.6).sin() * 0.06 * walk;
    let head_pitch = pose.head_down * (1.0 - 0.6 * down) + nod;
    // A positive turn about X lifts -Z toward +Y, so the head goes down by
    // the negative.
    let turn = Quat::from_rotation_y(pose.glance) * Quat::from_rotation_x(-head_pitch);
    let neck_root = rolled(v(0.0, f.neck_root.0, f.neck_root.1) + lift);
    let neck_dir = turn * Vec3::new(0.0, f.neck_dir.0, f.neck_dir.1).normalize();
    let side = (turn * Vec3::X).normalize();
    let crest = neck_dir.cross(side).normalize();
    let neck: Vec<Ring> = f
        .neck
        .iter()
        .map(|&(t, w, h)| Ring {
            centre: neck_root + neck_dir * (t * s),
            u: side,
            v: -crest,
            ru: w * s,
            rv: h * s,
        })
        .collect();
    let mane = f.mane;
    smooth(
        out,
        &curve(&neck, 3),
        14,
        Skin::Coat(f.neck_tile),
        &|n, _, _| {
            if mane && n.dot(-crest) > 0.55 {
                c.legs
            } else if kind == Beast::Woolback {
                c.body
            } else {
                mix(c.body, c.head, 0.5)
            }
        },
    );
    // The head, from the poll forward and down to the muzzle.
    let poll = neck_root + neck_dir * (f.poll * s);
    let head_dir = turn * Vec3::new(0.0, f.head_dir.0, f.head_dir.1).normalize();
    let head_up = head_dir.cross(side).normalize();
    let head: Vec<Ring> = f
        .head
        .iter()
        .map(|&(t, w, h)| Ring {
            centre: poll + head_dir * (t * s),
            u: side,
            v: -head_up,
            ru: w * s,
            rv: h * s,
        })
        .collect();
    let head_len = f.head.last().map_or(0.5, |h| h.0) + 0.04;
    let muzzle_t = (f.muzzle + 0.04) / head_len;
    let face_tile = if kind == Beast::Woolback {
        Tile::Skin
    } else {
        Tile::Hair
    };
    smooth(
        out,
        &curve(&head, 2),
        12,
        Skin::Coat(face_tile),
        &|n, t, _| {
            // The eyes: a dark spot each side, high on the head.
            let eye = n.dot(side).abs() > 0.8 && n.dot(head_up) < 0.1 && (0.22..0.4).contains(&t);
            if eye {
                [0.05, 0.04, 0.04]
            } else if t > muzzle_t {
                match kind {
                    Beast::Horse => mix(c.head, c.legs, 0.7),
                    Beast::Plodder => c.pale,
                    Beast::Woolback => dark(c.head, 0.8),
                }
            } else {
                c.head
            }
        },
    );
    // Ears from the poll, tapering to points; standing up or held out.
    for sgn in [-1.0f32, 1.0] {
        let base = poll + side * (sgn * 0.06 * s * k.max(0.6)) - head_up * (0.04 * s);
        let reach = if f.ears_out {
            side * sgn * 0.9 - head_up * 0.3 + head_dir * 0.1
        } else {
            -head_up * 0.95 + side * sgn * 0.2 - head_dir * 0.15
        }
        .normalize();
        let along = reach.cross(head_dir).normalize_or(side);
        let rings = [
            Ring {
                centre: base,
                u: along,
                v: head_dir,
                ru: f.ear * 0.24 * s,
                rv: f.ear * 0.14 * s,
            },
            Ring {
                centre: base + reach * (f.ear * 0.55 * s),
                u: along,
                v: head_dir,
                ru: f.ear * 0.22 * s,
                rv: f.ear * 0.09 * s,
            },
            Ring {
                centre: base + reach * (f.ear * s),
                u: along,
                v: head_dir,
                ru: f.ear * 0.04 * s,
                rv: f.ear * 0.03 * s,
            },
        ];
        smooth(out, &rings, 6, Skin::Coat(face_tile), &|_, _, _| c.head);
        // Horns: from inside the poll, out and curving up to a point.
        if f.horn > 0.0 {
            let root = poll + side * (sgn * 0.07 * s) - head_up * (0.07 * s);
            let pts = [
                root - side * (sgn * 0.03 * s),
                root + side * (sgn * f.horn * 0.55 * s) - head_up * (0.02 * s),
                root + side * (sgn * f.horn * 0.85 * s)
                    - head_up * (0.09 * s)
                    - head_dir * (0.03 * s),
                root + side * (sgn * f.horn * 0.95 * s)
                    - head_up * (0.17 * s)
                    - head_dir * (0.05 * s),
            ];
            let radii = [0.03, 0.026, 0.016, 0.004];
            let rings: Vec<Ring> = pts
                .iter()
                .zip(radii)
                .map(|(&p, r)| Ring {
                    centre: p,
                    u: head_dir,
                    v: head_up,
                    ru: r * s,
                    rv: r * s,
                })
                .collect();
            smooth(out, &rings, 6, Skin::Coat(Tile::Hoof), &|_, t, _| {
                mix([0.86, 0.8, 0.66], [0.3, 0.26, 0.22], t * t)
            });
        }
    }
    // The tail from the top of the rump, hanging and swishing.
    let tail_root = rolled(v(0.0, f.tail_root.0, f.tail_root.1) + lift);
    let swish = Quat::from_rotation_y(pose.tail + pose.phase.sin() * 0.08 * walk);
    let tail_dir = swish * Vec3::new(0.0, f.tail_dir.0, f.tail_dir.1).normalize();
    let tail: Vec<Ring> = f
        .tail
        .iter()
        .map(|&(t, r)| Ring {
            centre: tail_root
                + tail_dir * (t * f.tail_len * s)
                + Vec3::Z * (0.08 * t * t * f.tail_len * s),
            u: Vec3::X,
            v: Vec3::Z,
            ru: r * s,
            rv: r * s,
        })
        .collect();
    let switch = f.switch;
    let tail_tile = if kind == Beast::Woolback {
        Tile::Fleece
    } else {
        Tile::Mane
    };
    smooth(
        out,
        &tail,
        8,
        Skin::Coat(tail_tile),
        &|_, t, _| match kind {
            Beast::Horse => c.legs,
            Beast::Woolback => c.body,
            Beast::Plodder if t >= switch => dark(c.legs, 0.8),
            Beast::Plodder => c.body,
        },
    );
    // Legs: each swept from inside the body through the knee or hock, the
    // cannon and the fetlock to the hoof. A leg swings about its top and
    // lifts its hoof as it comes through; lying, it folds under.
    for (x, z, offset, fore) in [
        (-1.0f32, f.hind_z, 0.0, false),
        (-1.0, f.fore_z, 0.25, true),
        (1.0, f.hind_z, 0.5, false),
        (1.0, f.fore_z, 0.75, true),
    ] {
        let p = pose.phase + offset * std::f32::consts::TAU;
        let swing = p.sin() * 0.32 * walk + if fore { 1.2 } else { -1.1 } * down;
        let lift_leg = p.cos().max(0.0) * 0.5 * walk;
        let top = rolled(v(x * f.leg_x, f.leg_top, z) + lift);
        let swing_q = Quat::from_rotation_x(swing);
        let joint_y = if fore { f.fore_joint } else { f.hind_joint };
        let bend = if fore {
            -lift_leg - 2.4 * down
        } else {
            lift_leg * 0.8 + 2.2 * down
        };
        let joint = top
            + swing_q
                * Vec3::new(
                    0.0,
                    (joint_y - f.leg_top) * s,
                    if fore { 0.0 } else { 0.08 * k * s },
                );
        let below = swing_q * Quat::from_rotation_x(bend);
        let fetlock = joint + below * Vec3::new(0.0, -(joint_y - 0.12 * k) * s, 0.0);
        let hoof = fetlock + below * Vec3::new(0.0, -0.1 * k * s, -0.03 * k * s);
        let w = f.leg_w * k.max(0.55);
        let ring = |centre: Vec3, rw: f32, rh: f32| Ring {
            centre,
            u: Vec3::X,
            v: Vec3::Z,
            ru: rw * w * s,
            rv: rh * w * s,
        };
        let leg = [
            ring(top + Vec3::Y * (0.14 * k * s), 0.13, 0.2),
            ring(top.lerp(joint, 0.3), 0.1, if fore { 0.13 } else { 0.17 }),
            ring(top.lerp(joint, 0.7), 0.07, if fore { 0.09 } else { 0.11 }),
            ring(joint, 0.058, 0.07),
            ring(joint.lerp(fetlock, 0.5), 0.042, 0.05),
            ring(fetlock, 0.05, 0.055),
        ];
        let upper_colour = if kind == Beast::Woolback {
            c.legs
        } else {
            c.body
        };
        let leg_tile = if kind == Beast::Woolback {
            Tile::Skin
        } else {
            Tile::Hair
        };
        smooth(
            out,
            &curve(&leg, 2),
            10,
            Skin::Coat(leg_tile),
            &|_, t, _| {
                if kind == Beast::Woolback {
                    upper_colour
                } else {
                    mix(upper_colour, c.legs, ((t - 0.45) * 3.0).clamp(0.0, 1.0))
                }
            },
        );
        let foot = [ring(fetlock, 0.05, 0.055), ring(hoof, 0.065, 0.075)];
        smooth(out, &foot, 10, Skin::Coat(Tile::Hoof), &|_, _, _| {
            [0.2, 0.17, 0.15]
        });
    }
}

/// A bird (the meadow hen) at its stance, scaled by `size` (1 a grown hen:
/// 0.42 m tall).
pub fn bird(out: &mut Builder, c: &Coat, sex: Sex, pose: &Stance, size: f32) {
    let s = size;
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let down = pose.down.clamp(0.0, 1.0);
    // Roosting, the body settles onto the legs folded under it.
    let sink = Vec3::Y * (-0.1 * down * s);
    let roll_at = v(0.0, 0.2, 0.0) + sink;
    let roll = Quat::from_rotation_z(pose.shift);
    let rolled = |p: Vec3| roll_at + roll * (p - roll_at);
    let body: Vec<Ring> = [
        (0.17, 0.23, 0.035, 0.045),
        (0.13, 0.21, 0.085, 0.085),
        (0.06, 0.2, 0.115, 0.105),
        (-0.03, 0.2, 0.12, 0.11),
        (-0.1, 0.22, 0.1, 0.1),
        (-0.14, 0.26, 0.06, 0.07),
    ]
    .into_iter()
    .map(|(z, y, w, h)| Ring {
        centre: rolled(v(0.0, y, z) + sink),
        u: roll * Vec3::X,
        v: roll * Vec3::Y,
        ru: w * s,
        rv: h * s,
    })
    .collect();
    let speckle = c.pattern > 0;
    smooth(
        out,
        &curve(&body, 3),
        14,
        Skin::Coat(Tile::Feather),
        &|n, t, share| {
            // A speckled hen: darker flecks scattered over her sides.
            let fleck =
                speckle && ((t * 37.0).floor() as i32 + (share * 14.0).round() as i32) % 3 == 0;
            if fleck && n.y.abs() < 0.7 {
                dark(c.body, 0.62)
            } else {
                c.body
            }
        },
    );
    // Wings folded along her sides, growing from the shoulder.
    for sgn in [-1.0f32, 1.0] {
        let rings: Vec<Ring> = [
            (-0.08, 0.235, 0.012, 0.05),
            (0.0, 0.235, 0.02, 0.062),
            (0.09, 0.24, 0.016, 0.05),
            (0.15, 0.25, 0.006, 0.02),
        ]
        .into_iter()
        .map(|(z, y, w, h)| Ring {
            centre: rolled(v(sgn * 0.095, y, z) + sink),
            u: roll * Vec3::X,
            v: roll * Vec3::new(0.0, 1.0, 0.25).normalize(),
            ru: w * s,
            rv: h * s,
        })
        .collect();
        smooth(
            out,
            &curve(&rings, 2),
            8,
            Skin::Coat(Tile::Feather),
            &|_, _, _| dark(c.body, 0.88),
        );
    }
    // The tail: a wedge lifted from the rump, a cockerel's taller and dark.
    let (tail_colour, tail_size) = if sex == Sex::M {
        ([0.12, 0.25, 0.2], 1.4)
    } else {
        (c.body, 1.0)
    };
    let lift = Quat::from_rotation_x(-(0.6 + pose.tail));
    let tail_root = rolled(v(0.0, 0.24, 0.12) + sink);
    let tail: Vec<Ring> = [(0.0, 0.05, 0.03), (0.06, 0.055, 0.02), (0.13, 0.03, 0.008)]
        .into_iter()
        .map(|(t, w, h)| Ring {
            centre: tail_root + lift * Vec3::new(0.0, 0.0, t * tail_size * s),
            u: Vec3::X,
            v: lift * Vec3::Y,
            ru: w * s,
            rv: h * s,
        })
        .collect();
    smooth(out, &tail, 8, Skin::Coat(Tile::Feather), &|_, _, _| {
        tail_colour
    });
    // The neck and head: turned to glance, dipped to peck, bobbing.
    let pivot = rolled(v(0.0, 0.27, -0.1 + pose.bob) + sink);
    let neck_q =
        Quat::from_rotation_y(pose.glance) * Quat::from_rotation_x(-pose.head_down * (1.0 - down));
    let at = |p: Vec3| pivot + neck_q * (p * s);
    let fwd = neck_q * Vec3::NEG_Z;
    let side = neck_q * Vec3::X;
    let neck: Vec<Ring> = [
        (Vec3::new(0.0, -0.03, 0.02), 0.06, 0.065),
        (Vec3::new(0.0, 0.05, -0.025), 0.045, 0.045),
        (Vec3::new(0.0, 0.11, -0.045), 0.045, 0.048),
        (Vec3::new(0.0, 0.135, -0.075), 0.038, 0.036),
        (Vec3::new(0.0, 0.13, -0.1), 0.02, 0.018),
    ]
    .into_iter()
    .map(|(p, w, h)| Ring {
        centre: at(p),
        u: side,
        v: fwd,
        ru: w * s,
        rv: h * s,
    })
    .collect();
    smooth(
        out,
        &curve(&neck, 2),
        12,
        Skin::Coat(Tile::Feather),
        &|n, t, _| {
            let eye = n.dot(side).abs() > 0.85 && (0.6..0.8).contains(&t);
            if eye {
                [0.04, 0.035, 0.03]
            } else if t < 0.35 {
                mix(c.body, c.head, t / 0.35)
            } else {
                c.head
            }
        },
    );
    // The beak, tapering out of the face.
    let beak: Vec<Ring> = [
        (Vec3::new(0.0, 0.125, -0.09), 0.018, 0.015),
        (Vec3::new(0.0, 0.118, -0.115), 0.01, 0.008),
        (Vec3::new(0.0, 0.112, -0.13), 0.002, 0.002),
    ]
    .into_iter()
    .map(|(p, w, h)| Ring {
        centre: at(p),
        u: side,
        v: neck_q * Vec3::Y,
        ru: w * s,
        rv: h * s,
    })
    .collect();
    smooth(out, &beak, 6, Skin::Coat(Tile::Skin), &|_, _, _| c.legs);
    // The comb, one ridge grown along the crown, and the wattle under the
    // beak: a cockerel's larger.
    let comb = if sex == Sex::M { 1.7 } else { 1.0 };
    let crown: Vec<Ring> = [
        (Vec3::new(0.0, 0.145, -0.03), 0.006, 0.008),
        (Vec3::new(0.0, 0.16, -0.05), 0.009, 0.02 * comb),
        (Vec3::new(0.0, 0.165, -0.07), 0.009, 0.022 * comb),
        (Vec3::new(0.0, 0.155, -0.09), 0.006, 0.012 * comb),
    ]
    .into_iter()
    .map(|(p, w, h)| Ring {
        centre: at(p),
        u: side,
        v: neck_q * Vec3::Y,
        ru: w * s,
        rv: h * s,
    })
    .collect();
    smooth(
        out,
        &curve(&crown, 2),
        6,
        Skin::Coat(Tile::Skin),
        &|_, _, _| c.pale,
    );
    let wattle: Vec<Ring> = [
        (Vec3::new(0.0, 0.105, -0.085), 0.008, 0.008),
        (Vec3::new(0.0, 0.085, -0.09), 0.01, 0.012 * comb.sqrt()),
        (Vec3::new(0.0, 0.065, -0.088), 0.004, 0.004),
    ]
    .into_iter()
    .map(|(p, w, h)| Ring {
        centre: at(p),
        u: side,
        v: fwd,
        ru: w * s,
        rv: h * s,
    })
    .collect();
    smooth(out, &wattle, 6, Skin::Coat(Tile::Skin), &|_, _, _| c.pale);
    // Legs from inside the body, swinging opposite, with three toes; folded
    // away under her when she roosts.
    if down < 0.9 {
        let k = 1.0 - down;
        for (sgn, phase) in [(-1.0f32, 0.0), (1.0, std::f32::consts::PI)] {
            let swing = (pose.phase + phase).sin() * 0.45 * pose.walk.clamp(0.0, 1.0);
            let q = Quat::from_rotation_x(swing);
            let hip = v(sgn * 0.04, 0.17, 0.01) + sink;
            let hock = hip + q * v(0.0, -0.1 * k, 0.01);
            let foot = hip + q * v(0.0, -0.17 * k, -0.005);
            let leg = [
                Ring {
                    centre: hip,
                    u: Vec3::X,
                    v: Vec3::Z,
                    ru: 0.035 * s,
                    rv: 0.04 * s,
                },
                Ring {
                    centre: hip.lerp(hock, 0.6),
                    u: Vec3::X,
                    v: Vec3::Z,
                    ru: 0.02 * s,
                    rv: 0.025 * s,
                },
                Ring {
                    centre: hock,
                    u: Vec3::X,
                    v: Vec3::Z,
                    ru: 0.009 * s,
                    rv: 0.009 * s,
                },
                Ring {
                    centre: foot,
                    u: Vec3::X,
                    v: Vec3::Z,
                    ru: 0.008 * s,
                    rv: 0.008 * s,
                },
            ];
            smooth(out, &leg, 6, Skin::Coat(Tile::Skin), &|_, t, _| {
                if t < 0.4 { c.body } else { c.legs }
            });
            for toe in [-0.5f32, 0.0, 0.5] {
                let dir = (Quat::from_rotation_y(toe) * Vec3::NEG_Z).normalize();
                let rings = [
                    Ring {
                        centre: foot,
                        u: Vec3::X,
                        v: Vec3::Y,
                        ru: 0.008 * s,
                        rv: 0.006 * s,
                    },
                    Ring {
                        centre: foot + dir * (0.045 * s) - Vec3::Y * (0.003 * s),
                        u: Vec3::X,
                        v: Vec3::Y,
                        ru: 0.003 * s,
                        rv: 0.003 * s,
                    },
                ];
                smooth(out, &rings, 5, Skin::Coat(Tile::Skin), &|_, _, _| c.legs);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(b: &Builder) -> (Vec3, Vec3) {
        b.positions
            .iter()
            .fold((Vec3::MAX, Vec3::MIN), |(lo, hi), p| {
                let p = Vec3::from(*p);
                (lo.min(p), hi.max(p))
            })
    }

    fn whole(b: &Builder) {
        assert!(!b.positions.is_empty());
        assert!(b.textured);
        assert_eq!(b.positions.len(), b.normals.len());
        assert_eq!(b.positions.len(), b.colours.len());
        assert_eq!(b.positions.len(), b.uvs.len());
        assert_eq!(b.indices.len() % 3, 0);
        assert!(b.indices.iter().all(|&i| (i as usize) < b.positions.len()));
        assert!(b.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(
            b.normals
                .iter()
                .all(|n| (Vec3::from(*n).length() - 1.0).abs() < 1e-3)
        );
    }

    /// Each beast is a whole textured mesh about its height, standing on the
    /// ground, and lying down brings it lower.
    #[test]
    fn every_beast_is_whole_and_its_size() {
        for kind in [Beast::Horse, Beast::Plodder, Beast::Woolback] {
            let mut b = Builder::default();
            beast(&mut b, kind, &Coat::default(), &Stance::default(), 1.0);
            whole(&b);
            let (lo, hi) = bounds(&b);
            assert!(lo.y > -0.05 && lo.y < 0.05, "{kind:?} feet at {}", lo.y);
            let withers = kind.withers_m();
            assert!(
                (withers * 0.95..withers * 1.6).contains(&hi.y),
                "{kind:?} {}",
                hi.y
            );
            let mut lying = Builder::default();
            beast(
                &mut lying,
                kind,
                &Coat::default(),
                &Stance {
                    down: 1.0,
                    ..default()
                },
                1.0,
            );
            assert!(
                bounds(&lying).1.y < hi.y - withers * 0.3,
                "{kind:?} lies down"
            );
        }
        let mut b = Builder::default();
        bird(&mut b, &Coat::default(), Sex::M, &Stance::default(), 1.0);
        whole(&b);
        let (lo, hi) = bounds(&b);
        assert!(lo.y > -0.03 && (0.35..0.55).contains(&hi.y), "{lo} {hi}");
    }

    /// No faceting: most adjacent points of a body share their normal
    /// direction closely, where the old box rig's faces met at right angles.
    #[test]
    fn a_body_is_smooth_shaded() {
        let mut b = Builder::default();
        beast(
            &mut b,
            Beast::Plodder,
            &Coat::default(),
            &Stance::default(),
            1.0,
        );
        // The body loft's first two rows of 17 points.
        let row = 17;
        let mut close = 0;
        for k in 0..16 {
            let a = Vec3::from(b.normals[row + k]);
            let c = Vec3::from(b.normals[row + k + 1]);
            if a.dot(c) > 0.85 {
                close += 1;
            }
        }
        assert!(close >= 14, "{close} of 16 neighbours smooth");
    }

    /// Grazing puts the head down, not up: the muzzle (the point furthest
    /// forward) is lower with `head_down`.
    #[test]
    fn grazing_lowers_the_head() {
        let muzzle_y = |head_down: f32| {
            let mut b = Builder::default();
            beast(
                &mut b,
                Beast::Woolback,
                &Coat::default(),
                &Stance {
                    head_down,
                    ..default()
                },
                1.0,
            );
            let front = b
                .positions
                .iter()
                .map(|p| Vec3::from(*p))
                .min_by(|a, c| a.z.total_cmp(&c.z))
                .unwrap();
            front.y
        };
        assert!(muzzle_y(0.8) < muzzle_y(0.0) - 0.1);
    }

    /// The genome's colours tint the coat: a different body colour changes
    /// the body's vertex colours and nothing else about the mesh.
    #[test]
    fn a_coat_tints_its_texture() {
        let mut a = Builder::default();
        let mut b = Builder::default();
        beast(
            &mut a,
            Beast::Horse,
            &Coat::default(),
            &Stance::default(),
            1.0,
        );
        let other = Coat {
            body: [0.7, 0.68, 0.64],
            ..Coat::default()
        };
        beast(&mut b, Beast::Horse, &other, &Stance::default(), 1.0);
        assert_eq!(a.positions, b.positions);
        assert_eq!(a.uvs, b.uvs);
        assert_ne!(a.colours, b.colours);
    }
}
