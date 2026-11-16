//! The water creatures that are not M9's fish, modelled again in M9's manner
//! (`ocean-environments` design B15, the owner's "proper models instead of
//! placeholders"): skinned tubes and tapered limbs in place of boxes, each
//! wearing a 64 x 32 skin of the water life's atlas that M9's `paintSkin`
//! drew on the creature's own colours (`tools/water_life_atlas.py`,
//! `CREATURE_SKINS`). A body keeps the parts and pivots its posing names
//! ([`super::animate`]), so every pose and gait is as it was.
//!
//! Budgets are the fish's: each body is pinned under 500 triangles by a test.

use super::{Kit, Part, Shape, at, atr, bx, hh, icosahedron, in_rect, lin, shape, tile, white_uv};
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// A skin's rect by its name in the atlas's manifest.
fn skin(name: &str) -> [f32; 4] {
    tile(name)
}

/// The skin row of an angle round a body (0 the top): M9's skin is the back
/// at its first row and the belly at its middle, the two flanks mirrored.
fn row_of(th: f32) -> f32 {
    let a = (th + PI).rem_euclid(TAU) - PI;
    a.abs() / TAU
}

/// A point in a skin, or the white texel for a part drawn in its vertex
/// colour alone (a skin of all zeros: a bill, a collar, a crest).
fn uv_in(skin: [f32; 4], s: f32, t: f32) -> [f32; 2] {
    if skin[2] == 0.0 {
        white_uv()
    } else {
        in_rect(skin, s, t)
    }
}

/// No skin: the vertex colour on the white texel.
const PLAIN: [f32; 4] = [0.0; 4];

/// One face into a shape, dropped if it has no area.
fn tri(s: &mut Shape, p: [Vec3; 3], uv: [[f32; 2]; 3], colour: [f32; 4]) {
    if (p[1] - p[0]).cross(p[2] - p[0]).length_squared() < 1e-14 {
        return;
    }
    for i in 0..3 {
        s.positions.push(p[i]);
        s.colours.push(colour);
        s.uvs.push(uv[i]);
    }
}

/// A grid of points (rows along, columns round) as faces, with its UVs.
fn grid(s: &mut Shape, pts: &[Vec<(Vec3, [f32; 2])>], m: Mat4, colour: [f32; 4]) {
    for i in 0..pts.len().saturating_sub(1) {
        for j in 0..pts[i].len().saturating_sub(1) {
            let (a, a1) = (pts[i][j], pts[i][j + 1]);
            let (b, b1) = (pts[i + 1][j], pts[i + 1][j + 1]);
            let t = |q: Vec3| m.transform_point3(q);
            tri(s, [t(a.0), t(b.0), t(a1.0)], [a.1, b.1, a1.1], colour);
            tri(s, [t(b.0), t(b1.0), t(a1.0)], [b.1, b1.1, a1.1], colour);
        }
    }
}

/// A skinned body of rings along X: `[x, half height, half width, y]`, `nv`
/// faces over the angles `turn` (0 the top, a full turn closed), the skin
/// along it by ring and round it by angle. `belly_up` turns the skin over
/// (an otter on its back).
#[allow(clippy::too_many_arguments)]
fn tube(
    s: &mut Shape,
    rings: &[[f32; 4]],
    nv: usize,
    turn: (f32, f32),
    m: Mat4,
    skin: [f32; 4],
    belly_up: bool,
    colour: u32,
) {
    let n = rings.len().max(2) - 1;
    let pts: Vec<Vec<(Vec3, [f32; 2])>> = rings
        .iter()
        .enumerate()
        .map(|(i, r)| {
            (0..=nv)
                .map(|j| {
                    let th = turn.0 + (turn.1 - turn.0) * j as f32 / nv as f32;
                    let v = row_of(th + if belly_up { PI } else { 0.0 });
                    (
                        Vec3::new(r[0], r[3] + r[1] * th.cos(), r[2] * th.sin()),
                        uv_in(skin, i as f32 / n as f32, v),
                    )
                })
                .collect()
        })
        .collect();
    grid(s, &pts, m, lin(colour));
}

/// A tapered limb through points with their radii, `sides` faces round,
/// skinned along it and round it.
fn limb(s: &mut Shape, pts: &[(Vec3, f32)], sides: usize, skin: [f32; 4], colour: u32) {
    let n = pts.len();
    let rings: Vec<Vec<(Vec3, [f32; 2])>> = (0..n)
        .map(|i| {
            let prev = pts[i.saturating_sub(1)].0;
            let next = pts[(i + 1).min(n - 1)].0;
            let t = (next - prev).normalize_or(Vec3::X);
            let side = t.cross(Vec3::Y).normalize_or(t.any_orthonormal_vector());
            let up = side.cross(t);
            (0..=sides)
                .map(|j| {
                    let th = TAU * j as f32 / sides as f32;
                    (
                        pts[i].0 + (up * th.cos() + side * th.sin()) * pts[i].1,
                        uv_in(skin, i as f32 / (n - 1) as f32, row_of(th)),
                    )
                })
                .collect()
        })
        .collect();
    grid(s, &rings, Mat4::IDENTITY, lin(colour));
}

/// An ellipsoid as a closed tube, `nv` round and `nu` along.
fn blob(s: &mut Shape, r: Vec3, m: Mat4, nu: usize, nv: usize, skin: [f32; 4], colour: u32) {
    let rings: Vec<[f32; 4]> = (0..=nu)
        .map(|i| {
            let a = PI * i as f32 / nu as f32;
            [r.x * a.cos(), r.y * a.sin(), r.z * a.sin(), 0.0]
        })
        .collect();
    tube(s, &rings, nv, (0.0, TAU), m, skin, false, colour);
}

/// A flat polygon in the XY plane through `m`, laid into a skin by the
/// kit's box projection.
fn flat(k: &mut Kit, pts: &[[f32; 2]], m: Mat4, skin: [f32; 4], colour: u32) {
    let keep = k.1;
    k.1 = (skin[2] != 0.0).then_some(skin);
    k.add(shape(pts), m, colour);
    k.1 = keep;
}

/// A box in one colour on the white texel (an eye, a bill's nail).
fn block(s: &mut Shape, size: Vec3, at: Vec3, colour: u32) {
    let w = white_uv();
    for t in bx(size.x, size.y, size.z).chunks_exact(3) {
        tri(s, [t[0] + at, t[1] + at, t[2] + at], [w; 3], lin(colour));
    }
}

/// A pale eye with its pupil, looking out along `out`.
fn eye(s: &mut Shape, at: Vec3, r: f32, out: Vec3) {
    block(s, Vec3::splat(r * 1.6), at, 0xf4f0e2);
    block(s, Vec3::splat(r * 0.9), at + out * r * 0.5, 0x0c0a10);
}

// ---- crawlers ------------------------------------------------------------------

/// The shore crab: a wide oval carapace with a toothed front, eight jointed
/// walking legs, two claws with a palm and two fingers, eyes on stalks.
pub(super) fn crab() -> Shape {
    let (shell, leg) = (skin("crab"), skin("crab_leg"));
    let mut k = Kit::default();
    let s = &mut k.0;
    let y = 0.16;
    tube(
        s,
        &[
            [0.27, 0.0, 0.0, y],
            [0.25, 0.06, 0.26, y],
            [0.15, 0.1, 0.37, y],
            [0.0, 0.11, 0.4, y],
            [-0.12, 0.1, 0.36, y],
            [-0.21, 0.07, 0.26, y],
            [-0.25, 0.0, 0.0, y],
        ],
        8,
        (0.0, TAU),
        Mat4::IDENTITY,
        shell,
        false,
        0xffffff,
    );
    for sd in [1.0f32, -1.0] {
        // Teeth along the front edge of the shell.
        for t in 0..2 {
            let z = sd * (0.2 + t as f32 * 0.09);
            let base = Vec3::new(0.24 - t as f32 * 0.05, y + 0.02, z);
            limb(
                s,
                &[
                    (base, 0.025),
                    (base + Vec3::new(0.06, 0.0, sd * 0.03), 0.002),
                ],
                4,
                shell,
                0xffffff,
            );
        }
        // Four walking legs a side, each up and out from the body and down
        // to a point.
        for i in 0..4 {
            let x = 0.1 - i as f32 * 0.1;
            let spread = (i as f32 - 1.5) * 0.18;
            let out = Vec3::new(spread.sin() * -0.6, 0.0, sd * spread.cos()).normalize();
            let hip = Vec3::new(x, y - 0.02, sd * 0.3);
            let knee = hip + out * 0.16 + Vec3::Y * 0.06;
            let tip = knee + out * 0.1 - Vec3::Y * 0.2;
            limb(
                s,
                &[(hip, 0.032), (knee, 0.026), (tip, 0.006)],
                4,
                leg,
                0xffffff,
            );
        }
        // The claw: an arm forward, a palm and its two fingers.
        let shoulder = Vec3::new(0.18, y - 0.02, sd * 0.22);
        let wrist = Vec3::new(0.34, y + 0.02, sd * 0.3);
        limb(s, &[(shoulder, 0.035), (wrist, 0.03)], 5, leg, 0xffffff);
        let palm = wrist + Vec3::new(0.07, 0.0, -sd * 0.02);
        blob(
            s,
            Vec3::new(0.08, 0.055, 0.05),
            Mat4::from_rotation_translation(Quat::from_rotation_y(sd * 0.3), palm),
            4,
            6,
            shell,
            0xffffff,
        );
        let finger = palm + Vec3::new(0.07, 0.0, -sd * 0.02);
        limb(
            s,
            &[
                (finger + Vec3::Y * 0.025, 0.022),
                (finger + Vec3::new(0.08, 0.0, -sd * 0.05), 0.003),
            ],
            4,
            leg,
            0x5a1010,
        );
        limb(
            s,
            &[
                (finger - Vec3::Y * 0.02, 0.02),
                (finger + Vec3::new(0.07, -0.02, -sd * 0.05), 0.003),
            ],
            4,
            leg,
            0x5a1010,
        );
        // Eyes on stalks.
        let stalk = Vec3::new(0.24, y + 0.08, sd * 0.07);
        limb(
            s,
            &[(stalk, 0.012), (stalk + Vec3::Y * 0.08, 0.01)],
            3,
            leg,
            0xffffff,
        );
        eye(s, stalk + Vec3::Y * 0.1, 0.022, Vec3::X);
    }
    k.build()
}

/// The lobster: a carapace with its rostrum, a segmented tail to a fan of
/// five plates, two big claws, long feelers and eight walking legs.
pub(super) fn lobster() -> Shape {
    let (body, tail, leg) = (skin("lobster"), skin("lobster_tail"), skin("crab_leg"));
    let mut k = Kit::default();
    let y = 0.1;
    tube(
        &mut k.0,
        &[
            [0.5, 0.0, 0.0, y + 0.02],
            [0.43, 0.03, 0.03, y + 0.02],
            [0.36, 0.07, 0.075, y],
            [0.2, 0.09, 0.09, y],
            [0.05, 0.09, 0.085, y],
            [-0.02, 0.075, 0.075, y],
        ],
        8,
        (0.0, TAU),
        Mat4::IDENTITY,
        body,
        false,
        0xffffff,
    );
    // The tail: ridged segments narrowing back.
    let mut rings = Vec::new();
    for i in 0..5 {
        let x = -0.02 - i as f32 * 0.08;
        let r = 0.075 - i as f32 * 0.008;
        rings.push([x, r * 0.85, r, y]);
        rings.push([x - 0.035, r * 0.95, r * 1.06, y]);
    }
    rings.push([-0.45, 0.03, 0.035, y]);
    tube(
        &mut k.0,
        &rings,
        6,
        (0.0, TAU),
        Mat4::IDENTITY,
        tail,
        false,
        0xffffff,
    );
    // The fan.
    for f in 0..5 {
        let a = (f as f32 - 2.0) * 0.38;
        flat(
            &mut k,
            &[[0.0, -0.025], [-0.13, -0.05], [-0.15, 0.05], [0.0, 0.025]],
            Mat4::from_rotation_translation(
                Quat::from_rotation_y(a) * Quat::from_rotation_x(FRAC_PI_2),
                Vec3::new(-0.44, y, 0.0),
            ),
            tail,
            0xffffff,
        );
    }
    let s = &mut k.0;
    for sd in [1.0f32, -1.0] {
        // The claw: an arm, a big palm and two fingers.
        let shoulder = Vec3::new(0.36, y - 0.02, sd * 0.06);
        let elbow = Vec3::new(0.48, y, sd * 0.16);
        limb(s, &[(shoulder, 0.025), (elbow, 0.03)], 5, leg, 0xffffff);
        let palm = elbow + Vec3::new(0.12, 0.0, sd * 0.04);
        blob(
            s,
            Vec3::new(0.13, 0.05, 0.065),
            Mat4::from_rotation_translation(Quat::from_rotation_y(sd * -0.3), palm),
            4,
            6,
            body,
            0xffffff,
        );
        let finger = palm + Vec3::new(0.12, 0.0, sd * 0.04);
        limb(
            s,
            &[
                (finger + Vec3::Y * 0.02, 0.025),
                (finger + Vec3::new(0.11, 0.0, sd * 0.02), 0.004),
            ],
            4,
            body,
            0xffffff,
        );
        limb(
            s,
            &[
                (finger - Vec3::Y * 0.02, 0.02),
                (finger + Vec3::new(0.09, -0.02, sd * 0.02), 0.004),
            ],
            4,
            body,
            0xffffff,
        );
        // Feelers, long and thin.
        let root = Vec3::new(0.44, y + 0.05, sd * 0.03);
        limb(
            s,
            &[
                (root, 0.01),
                (root + Vec3::new(0.3, 0.06, sd * 0.1), 0.006),
                (root + Vec3::new(0.7, 0.0, sd * 0.25), 0.002),
            ],
            3,
            leg,
            0x5a2010,
        );
        // Walking legs, three a side behind the claw's.
        for i in 0..3 {
            let hip = Vec3::new(0.26 - i as f32 * 0.09, y - 0.05, sd * 0.07);
            let knee = hip + Vec3::new(0.0, 0.03, sd * 0.1);
            let tip = knee + Vec3::new(0.02, -0.1, sd * 0.06);
            limb(
                s,
                &[(hip, 0.014), (knee, 0.012), (tip, 0.004)],
                3,
                leg,
                0xffffff,
            );
        }
        eye(s, Vec3::new(0.41, y + 0.06, sd * 0.04), 0.018, Vec3::X);
    }
    k.build()
}

/// The giant clam: two fluted shells, the lower and the lid on its hinge,
/// and the wavy blue mantle between them.
pub(super) fn clam() -> (Shape, Shape) {
    let (shell, mantle) = (skin("clam_shell"), skin("clam_mantle"));
    // Folds across the long axis, as a giant clam's are.
    let fold = |x: f32| 1.0 + 0.09 * (x * TAU * 2.5).cos();
    let rings = |h: f32| -> Vec<[f32; 4]> {
        (0..=10)
            .map(|i| {
                let a = PI * i as f32 / 10.0;
                let x = 0.5 * a.cos();
                [x, h * a.sin() * fold(x), 0.36 * a.sin() * fold(x), 0.0]
            })
            .collect()
    };
    let mut k = Kit::default();
    // The lower shell: the bottom half of a fluted ellipsoid, its lip at 0.2.
    tube(
        &mut k.0,
        &rings(0.18),
        6,
        (FRAC_PI_2, PI + FRAC_PI_2),
        at([0.0, 0.2, 0.0]),
        shell,
        false,
        0xffffff,
    );
    // The mantle: a wavy sheet across the opening.
    let pts: Vec<Vec<(Vec3, [f32; 2])>> = (0..=10)
        .map(|i| {
            let x = -0.46 + 0.92 * i as f32 / 10.0;
            (0..=3)
                .map(|j| {
                    let z =
                        (-0.3 + 0.6 * j as f32 / 3.0) * (1.0 - (x / 0.5).powi(2)).max(0.05).sqrt();
                    let yy = 0.21 + 0.03 * (x * TAU * 2.5).cos() + 0.015 * (j as f32 * 2.0).sin();
                    (
                        Vec3::new(x, yy, z),
                        in_rect(mantle, i as f32 / 10.0, j as f32 / 6.0),
                    )
                })
                .collect()
        })
        .collect();
    grid(&mut k.0, &pts, Mat4::IDENTITY, [1.0; 4]);
    let lower = k.build();
    // The lid: the top half, its hinge at the pivot's z.
    let mut k = Kit::default();
    tube(
        &mut k.0,
        &rings(0.15),
        6,
        (-FRAC_PI_2, FRAC_PI_2),
        at([0.0, 0.0, -0.36]),
        shell,
        false,
        0xffffff,
    );
    (lower, k.build())
}

/// The reef turtle: a domed shell and its flat plastron, a beaked head,
/// paddle flippers on their pivots and a short tail.
pub(super) fn turtle() -> (Shape, Vec<Part>) {
    let (shell, skin_) = (skin("turtle_shell"), skin("turtle_skin"));
    let mut k = Kit::default();
    let dome: Vec<[f32; 4]> = (0..=8)
        .map(|i| {
            let a = PI * i as f32 / 8.0;
            [0.38 * a.cos(), 0.15 * a.sin(), 0.31 * a.sin(), 0.0]
        })
        .collect();
    tube(
        &mut k.0,
        &dome,
        6,
        (-FRAC_PI_2, FRAC_PI_2),
        at([0.0, 0.0, 0.0]),
        shell,
        false,
        0xffffff,
    );
    let plastron: Vec<[f32; 4]> = dome
        .iter()
        .map(|r| [r[0], r[1] * 0.25, r[2], r[3]])
        .collect();
    tube(
        &mut k.0,
        &plastron,
        4,
        (FRAC_PI_2, PI + FRAC_PI_2),
        Mat4::IDENTITY,
        shell,
        false,
        0xffffff,
    );
    // The head on its neck, a beak at the front.
    tube(
        &mut k.0,
        &[
            [0.3, 0.05, 0.06, 0.0],
            [0.4, 0.06, 0.06, 0.02],
            [0.48, 0.065, 0.065, 0.03],
            [0.54, 0.04, 0.045, 0.02],
            [0.57, 0.0, 0.0, 0.01],
        ],
        6,
        (0.0, TAU),
        Mat4::IDENTITY,
        skin_,
        false,
        0xffffff,
    );
    for sd in [1.0f32, -1.0] {
        eye(
            &mut k.0,
            Vec3::new(0.5, 0.06, sd * 0.05),
            0.016,
            Vec3::Z * sd,
        );
    }
    limb(
        &mut k.0,
        &[
            (Vec3::new(-0.34, 0.0, 0.0), 0.04),
            (Vec3::new(-0.46, -0.01, 0.0), 0.004),
        ],
        4,
        skin_,
        0xffffff,
    );
    let flipper = |front: bool, sd: f32| {
        let mut k = Kit::default();
        let pts: Vec<[f32; 2]> = if front {
            vec![
                [0.05, 0.0],
                [0.0, 0.06],
                [-0.12, 0.3],
                [-0.2, 0.38],
                [-0.12, 0.2],
                [-0.05, 0.0],
            ]
        } else {
            vec![
                [0.04, 0.0],
                [0.02, 0.08],
                [-0.06, 0.16],
                [-0.12, 0.1],
                [-0.06, 0.0],
            ]
        };
        let pts: Vec<[f32; 2]> = pts.into_iter().map(|p| [p[0], p[1] * sd]).collect();
        flat(
            &mut k,
            &pts,
            Mat4::from_rotation_x(FRAC_PI_2),
            skin_,
            0xffffff,
        );
        k.build()
    };
    let parts = vec![
        Part {
            name: "fr",
            at: Vec3::new(0.18, 0.0, 0.24),
            yaw: 0.0,
            shape: flipper(true, 1.0),
        },
        Part {
            name: "fl",
            at: Vec3::new(0.18, 0.0, -0.24),
            yaw: 0.0,
            shape: flipper(true, -1.0),
        },
        Part {
            name: "br",
            at: Vec3::new(-0.24, 0.0, 0.2),
            yaw: 0.0,
            shape: flipper(false, 1.0),
        },
        Part {
            name: "bl",
            at: Vec3::new(-0.24, 0.0, -0.2),
            yaw: 0.0,
            shape: flipper(false, -1.0),
        },
    ];
    (k.build(), parts)
}

// ---- the soft-bodied ---------------------------------------------------------

/// The octopus: a mottled mantle leaning back over its head, eyes with a
/// bar pupil, and eight tapered arms on their pivots, pale beneath.
pub(super) fn octopus() -> (Shape, Vec<Part>) {
    let sk = skin("octopus");
    let mut k = Kit::default();
    blob(
        &mut k.0,
        Vec3::new(0.24, 0.2, 0.2),
        Mat4::from_rotation_translation(Quat::from_rotation_z(0.5), Vec3::new(-0.1, 0.27, 0.0)),
        7,
        8,
        sk,
        0xffffff,
    );
    blob(
        &mut k.0,
        Vec3::new(0.13, 0.11, 0.15),
        at([0.06, 0.12, 0.0]),
        5,
        8,
        sk,
        0xffffff,
    );
    for sd in [1.0f32, -1.0] {
        let e = Vec3::new(0.13, 0.19, sd * 0.1);
        block(&mut k.0, Vec3::new(0.06, 0.05, 0.05), e, 0xeab798);
        block(
            &mut k.0,
            Vec3::new(0.06, 0.012, 0.02),
            e + Vec3::new(0.01, 0.0, sd * 0.02),
            0x0c0a10,
        );
    }
    let arm = || {
        let mut s = Shape::default();
        let pts: Vec<(Vec3, f32)> = (0..5)
            .map(|i| {
                let t = i as f32 / 4.0;
                (
                    Vec3::new(t * 0.52, -0.03 * (t * PI).sin(), 0.0),
                    0.045 * (1.0 - t) + 0.006,
                )
            })
            .collect();
        limb(&mut s, &pts, 4, sk, 0xffffff);
        s
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

/// The plume worm: a ringed white tube one unit tall (+Y) and its crown of
/// banded red radioles, a part that draws in.
pub(super) fn tube_worm() -> (Shape, Vec<Part>) {
    let (tube_s, plume) = (skin("worm_tube"), skin("worm_plume"));
    let mut k = Kit::default();
    // The tube built along X, stood up on +Y.
    let rings: Vec<[f32; 4]> = (0..=6)
        .map(|i| {
            let t = i as f32 / 6.0;
            [
                t,
                0.065 - 0.012 * t + if i == 6 { 0.01 } else { 0.0 },
                0.065 - 0.012 * t,
                0.0,
            ]
        })
        .collect();
    tube(
        &mut k.0,
        &rings,
        8,
        (0.0, TAU),
        Mat4::from_rotation_z(FRAC_PI_2),
        tube_s,
        false,
        0xffffff,
    );
    let mut p = Kit::default();
    limb(
        &mut p.0,
        &[(Vec3::ZERO, 0.05), (Vec3::Y * 0.05, 0.06)],
        6,
        plume,
        0xffffff,
    );
    for r in 0..14 {
        let a = r as f32 * TAU / 14.0;
        let lean = 0.45 + 0.1 * hh(r, 3);
        flat(
            &mut p,
            &[
                [-0.012, 0.0],
                [0.012, 0.0],
                [0.02, 0.16],
                [0.0, 0.24],
                [-0.02, 0.16],
            ],
            Mat4::from_rotation_translation(
                Quat::from_rotation_y(a) * Quat::from_rotation_x(lean),
                Vec3::new(0.0, 0.04, 0.0) + Vec3::new(a.sin(), 0.0, a.cos()) * 0.035,
            ),
            plume,
            0xffffff,
        );
    }
    let parts = vec![Part {
        name: "plume",
        at: Vec3::new(0.0, 0.98, 0.0),
        yaw: 0.0,
        shape: p.build(),
    }];
    (k.build(), parts)
}

// ---- the furred ----------------------------------------------------------------

/// The seal: a smooth spotted body from its muzzle to its hind flippers,
/// fore flippers and a fan of hind flippers.
pub(super) fn seal() -> Shape {
    let sk = skin("seal");
    let mut k = Kit::default();
    let y = 0.13;
    tube(
        &mut k.0,
        &[
            [0.52, 0.0, 0.0, y - 0.01],
            [0.49, 0.045, 0.05, y - 0.01],
            [0.44, 0.08, 0.08, y],
            [0.36, 0.1, 0.1, y],
            [0.28, 0.1, 0.1, y - 0.005],
            [0.15, 0.13, 0.14, y],
            [0.0, 0.14, 0.15, y],
            [-0.15, 0.12, 0.13, y],
            [-0.28, 0.08, 0.08, y],
            [-0.38, 0.04, 0.045, y],
            [-0.41, 0.0, 0.0, y],
        ],
        8,
        (0.0, TAU),
        Mat4::IDENTITY,
        sk,
        false,
        0xffffff,
    );
    for sd in [1.0f32, -1.0] {
        eye(
            &mut k.0,
            Vec3::new(0.44, y + 0.05, sd * 0.06),
            0.02,
            Vec3::Z * sd,
        );
        flat(
            &mut k,
            &[[0.04, 0.0], [-0.06, 0.02], [-0.16, 0.06], [-0.12, 0.0]],
            Mat4::from_rotation_translation(
                Quat::from_rotation_y(sd * 0.5) * Quat::from_rotation_x(sd * 1.2),
                Vec3::new(0.18, y - 0.07, sd * 0.12),
            ),
            sk,
            0xffffff,
        );
        flat(
            &mut k,
            &[
                [0.0, 0.0],
                [-0.14, sd * 0.02],
                [-0.2, sd * 0.1],
                [-0.12, sd * 0.06],
            ],
            Mat4::from_rotation_translation(
                Quat::from_rotation_x(FRAC_PI_2),
                Vec3::new(-0.38, y, 0.0),
            ),
            sk,
            0xffffff,
        );
    }
    block(
        &mut k.0,
        Vec3::new(0.025, 0.02, 0.03),
        Vec3::new(0.52, y + 0.005, 0.0),
        0x161612,
    );
    k.build()
}

/// The sea otter rafting on its back: a long furred body belly up, its
/// round head with ears and a pale muzzle, paws on its chest round an
/// urchin, hind feet and a thick tail.
pub(super) fn otter() -> Shape {
    let sk = skin("otter");
    let mut k = Kit::default();
    let y = 0.03;
    tube(
        &mut k.0,
        &[
            [0.5, 0.0, 0.0, y],
            [0.48, 0.035, 0.04, y],
            [0.43, 0.065, 0.065, y],
            [0.36, 0.07, 0.07, y],
            [0.3, 0.055, 0.055, y],
            [0.2, 0.09, 0.1, y],
            [0.0, 0.1, 0.11, y],
            [-0.15, 0.09, 0.09, y],
            [-0.27, 0.06, 0.06, y],
            [-0.3, 0.0, 0.0, y],
        ],
        8,
        (0.0, TAU),
        Mat4::IDENTITY,
        sk,
        true,
        0xffffff,
    );
    limb(
        &mut k.0,
        &[
            (Vec3::new(-0.26, y, 0.0), 0.045),
            (Vec3::new(-0.42, y + 0.01, 0.0), 0.035),
            (Vec3::new(-0.56, y + 0.02, 0.0), 0.01),
        ],
        6,
        sk,
        0xffffff,
    );
    for sd in [1.0f32, -1.0] {
        eye(
            &mut k.0,
            Vec3::new(0.43, y + 0.05, sd * 0.04),
            0.014,
            Vec3::Y,
        );
        limb(
            &mut k.0,
            &[
                (Vec3::new(0.38, y + 0.04, sd * 0.06), 0.015),
                (Vec3::new(0.38, y + 0.07, sd * 0.07), 0.003),
            ],
            3,
            sk,
            0xffffff,
        );
        // Forepaws on the chest.
        limb(
            &mut k.0,
            &[
                (Vec3::new(0.24, y + 0.06, sd * 0.09), 0.025),
                (Vec3::new(0.24, y + 0.13, sd * 0.05), 0.02),
            ],
            4,
            sk,
            0xffffff,
        );
        // Hind feet, webbed.
        flat(
            &mut k,
            &[[0.0, 0.0], [-0.12, -0.03], [-0.14, 0.04], [-0.02, 0.03]],
            Mat4::from_rotation_translation(
                Quat::from_rotation_x(-0.4 * sd),
                Vec3::new(-0.22, y + 0.08, sd * 0.06),
            ),
            sk,
            0xffffff,
        );
    }
    block(
        &mut k.0,
        Vec3::new(0.025, 0.02, 0.03),
        Vec3::new(0.5, y + 0.02, 0.0),
        0x161612,
    );
    let urchin = icosahedron(0.05);
    let w = white_uv();
    for t in urchin.chunks_exact(3) {
        let o = Vec3::new(0.24, 0.17, 0.0);
        tri(
            &mut k.0,
            [t[0] + o, t[1] + o, t[2] + o],
            [w; 3],
            lin(0x7d2dbd),
        );
    }
    k.build()
}

// ---- birds ---------------------------------------------------------------------

/// A duck's look: its skins and the colours its bill, legs, collar, wing
/// patch and crest are drawn in.
pub(super) struct Duck {
    pub body: &'static str,
    pub head: &'static str,
    pub bill: u32,
    pub leg: u32,
    pub ring: Option<u32>,
    pub spec: Option<u32>,
    pub crest: Option<u32>,
    pub tail: u32,
}

/// A duck on the water: a boat-shaped body, the head on its neck (a part,
/// so it turns, dips and tucks), a flat bill, wings folded on the back with
/// their coloured patch, and paddling feet (parts).
pub(super) fn duck(p: &Duck) -> (Shape, Vec<Part>) {
    let (body, head) = (skin(p.body), skin(p.head));
    let by = 0.12;
    let mut k = Kit::default();
    tube(
        &mut k.0,
        &[
            [0.36, 0.0, 0.0, by],
            [0.33, 0.09, 0.09, by],
            [0.22, 0.14, 0.15, by],
            [0.05, 0.15, 0.16, by],
            [-0.12, 0.14, 0.14, by + 0.01],
            [-0.26, 0.1, 0.09, by + 0.03],
            [-0.36, 0.05, 0.05, by + 0.06],
            [-0.42, 0.0, 0.0, by + 0.08],
        ],
        8,
        (0.0, TAU),
        Mat4::IDENTITY,
        body,
        false,
        0xffffff,
    );
    flat(
        &mut k,
        &[[0.0, 0.03], [-0.14, 0.08], [-0.16, -0.02], [0.0, -0.03]],
        atr([-0.38, by + 0.07, 0.0], [FRAC_PI_2, 0.0, 0.3]),
        body,
        p.tail,
    );
    // The head and neck, pivoted at the neck's base.
    let mut h = Kit::default();
    let (nl, lean) = (0.14f32, -0.25f32);
    let top = Vec3::new((-lean).sin() * nl, lean.cos() * nl, 0.0);
    limb(
        &mut h.0,
        &[(Vec3::ZERO, 0.065), (top * 0.6, 0.05), (top, 0.055)],
        6,
        head,
        0xffffff,
    );
    blob(
        &mut h.0,
        Vec3::new(0.11, 0.09, 0.085),
        at([top.x + 0.03, top.y + 0.03, 0.0]),
        5,
        6,
        head,
        0xffffff,
    );
    // The bill: flat and broad, wider at its tip.
    let bx0 = top.x + 0.12;
    tube(
        &mut h.0,
        &[
            [bx0, 0.022, 0.03, top.y + 0.01],
            [bx0 + 0.06, 0.014, 0.034, top.y],
            [bx0 + 0.11, 0.01, 0.03, top.y - 0.006],
            [bx0 + 0.12, 0.0, 0.0, top.y - 0.008],
        ],
        4,
        (0.0, TAU),
        Mat4::IDENTITY,
        PLAIN,
        false,
        p.bill,
    );
    let w = white_uv();
    for sd in [1.0f32, -1.0] {
        eye(
            &mut h.0,
            Vec3::new(top.x + 0.07, top.y + 0.05, sd * 0.075),
            0.014,
            Vec3::Z * sd,
        );
    }
    if let Some(ring) = p.ring {
        limb(
            &mut h.0,
            &[(Vec3::Y * 0.02, 0.07), (Vec3::Y * 0.045, 0.066)],
            6,
            PLAIN,
            ring,
        );
    }
    if let Some(crest) = p.crest {
        flat(
            &mut h,
            &[[0.0, 0.0], [-0.18, 0.03], [-0.16, -0.02]],
            Mat4::from_translation(Vec3::new(top.x - 0.02, top.y + 0.1, 0.0)),
            PLAIN,
            crest,
        );
    }
    let wing = |sd: f32| {
        let mut wk = Kit::default();
        let (sp, ch) = (0.34f32, 0.2f32);
        flat(
            &mut wk,
            &[
                [0.04, 0.0],
                [-ch, 0.0],
                [-ch * 1.1, sp * 0.6],
                [-ch * 0.7, sp],
                [0.0, sp * 0.55],
            ],
            atr([0.0; 3], [sd * FRAC_PI_2, 0.0, 0.0]),
            body,
            0xffffff,
        );
        if let Some(spec) = p.spec {
            // The speculum, a patch across the trailing edge, a hair proud.
            flat(
                &mut wk,
                &[
                    [-ch * 0.55, 0.03],
                    [-ch * 1.0, 0.03],
                    [-ch * 1.03, sp * 0.28],
                    [-ch * 0.6, sp * 0.28],
                ],
                atr([0.0; 3], [sd * FRAC_PI_2, 0.0, 0.0]) * Mat4::from_translation(Vec3::Z * 0.004),
                PLAIN,
                spec,
            );
        }
        wk.build()
    };
    let foot = || {
        let mut f = Kit::default();
        limb(
            &mut f.0,
            &[(Vec3::ZERO, 0.012), (Vec3::new(0.0, -0.08, 0.0), 0.01)],
            4,
            PLAIN,
            p.leg,
        );
        let web = shape(&[[0.0, 0.0], [0.09, -0.035], [0.1, 0.035]]);
        for t in web.chunks_exact(3) {
            let m = |q: Vec3| Vec3::new(q.x, -0.08, q.y);
            tri(&mut f.0, [m(t[0]), m(t[1]), m(t[2])], [w; 3], lin(p.leg));
        }
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
