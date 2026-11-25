//! The bare fist's frame and its aimed pose (`openspec/changes/held-swing`,
//! decision 10, "The fist, version 16"), ported from the approved Tools in
//! Hand mockup (M36 v16, `docs/mockups/tool-motion.html`: `fistFrame`,
//! `applyAimed`).
//!
//! The fist is posed as an arm, not a tool: the forearm aims from a virtual
//! elbow (`held.punch` `elbow_*`, eye space, low and to the right) through
//! the fist, the elbow following part of the fist's travel, so a long
//! forearm and sleeve come in from the lower right through the whole
//! stroke. At twist 0 the back of the hand faces `FistShape::back_eye`,
//! square to the forearm, and the knuckles aim forward into the view. The
//! wrist is straight: the arm is turned onto the back of the hand's line
//! about the wrist joint. Drawn only.

use super::HeldPose;
use super::stroke::PunchSettings;
use bevy::prelude::*;
use pbd_core::hand::{Bone, FistShape, Look};

/// The fist's own frame, in the grip's tool units after the grip's turn: A
/// down the knuckles' column (index to little finger), K the knuckles' way
/// (wrist to the back of the hand), B out of the back of the hand; J the
/// wrist joint, C the fist's centre (where the handle was), W the forearm's
/// way from the wrist (toward the elbow).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FistFrame {
    pub a: Vec3,
    pub k: Vec3,
    pub b: Vec3,
    pub j: Vec3,
    pub c: Vec3,
    pub w: Vec3,
}

fn at(b: &Bone) -> Vec3 {
    Vec3::from(b.at)
}

fn axis(b: &Bone) -> Vec3 {
    Vec3::from(b.axis).normalize()
}

impl FistFrame {
    /// From the fist's bones (`hand::fist_bones`) and the fist's centre in
    /// tool units (the pickaxe's fist, which the fist is built about), the
    /// grip's turn as the hand's builder makes it.
    pub fn new(bones: &[Bone], centre: Vec3, look: &Look) -> Self {
        let a = (at(&bones[10]) - at(&bones[1])).normalize();
        let turn = Quat::from_axis_angle(a, look.grip_turn_deg.to_radians());
        let pt = |p: Vec3| turn * (p - centre) + centre;
        let back = pt(at(&bones[0]));
        let wrist = &bones[16];
        let (e1, e2) = (
            at(wrist) + axis(wrist) * wrist.length / 2.0,
            at(wrist) - axis(wrist) * wrist.length / 2.0,
        );
        let near = if e1.distance(at(&bones[0])) < e2.distance(at(&bones[0])) {
            e1
        } else {
            e2
        };
        let j = pt(near);
        let w = (pt(at(&bones[17])) - j).normalize();
        let mut k = turn * axis(&bones[0]);
        if k.dot(back - j) < 0.0 {
            k = -k;
        }
        let k = (k - a * k.dot(a)).normalize();
        let tips = [3, 6, 9, 12]
            .iter()
            .map(|&i| pt(at(&bones[i])))
            .sum::<Vec3>()
            * 0.25;
        let b = back - tips;
        let b = (b - a * b.dot(a) - k * b.dot(k)).normalize();
        Self {
            a,
            k,
            b,
            j,
            c: centre,
            w,
        }
    }
}

/// Where the fist is drawn: the holder's transform under the camera and the
/// arm's turn about the wrist (tool units).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aimed {
    pub holder: Transform,
    pub arm: Transform,
    /// The fist's centre, eye space.
    pub fist: Vec3,
    /// The forearm's way, eye space, toward the knuckles.
    pub aim: Vec3,
}

/// The fist posed by `pose` (its shift from a tool's fist `fist0`, the guard
/// included, and its turn: pitch the cock on the wrist, roll the twist
/// about the forearm, yaw the aim sideways), `scale` metres a tool unit (the
/// mockup's `applyAimed`).
pub fn aimed(
    frame: &FistFrame,
    p: &PunchSettings,
    shape: &FistShape,
    fist0: Vec3,
    scale: f32,
    pose: HeldPose,
) -> Aimed {
    let f = frame;
    let fist = fist0 + pose.shift;
    let guard = super::stroke::guard(p);
    let elbow = Vec3::new(p.elbow_x, p.elbow_y, p.elbow_z) + (pose.shift - guard) * p.elbow_follow;
    let d = Quat::from_rotation_y(pose.yaw) * (fist - elbow).normalize_or(Vec3::NEG_Z);
    let back_eye = Vec3::from(shape.back_eye);
    let be = (back_eye - d * back_eye.dot(d)).normalize_or(Vec3::X);
    let be = Quat::from_axis_angle(d, p.guard_twist_rad + pose.roll) * be;
    // A as the tool frame has it (B = s K x A, so A = s B x K): a proper
    // turn.
    let s = if f.b.dot(f.k.cross(f.a)) < 0.0 {
        -1.0
    } else {
        1.0
    };
    let ae = (be.cross(d) * s).normalize();
    let q0 =
        Quat::from_mat3(&(Mat3::from_cols(ae, d, be) * Mat3::from_cols(f.a, f.k, f.b).transpose()))
            .normalize();
    // The hand turned on the wrist: the bend about the back of the hand's
    // normal, the cock about the column. Both turn the hand only; the arm
    // is turned back by them, so the forearm stays on d.
    let tb = Quat::from_axis_angle(f.b, p.bend_rad);
    let ta = Quat::from_axis_angle(f.a, pose.pitch);
    let q = q0 * tb * ta;
    let straight = Quat::from_rotation_arc(f.w, -f.k);
    let arm_q = ta.inverse() * tb.inverse() * straight;
    // The wrist on the forearm's line, the fist's centre ahead of it.
    let jc = (f.c - f.j).dot(f.k) * scale;
    let wrist_eye = fist - d * jc;
    Aimed {
        holder: Transform {
            translation: wrist_eye - q * (f.j * scale),
            rotation: q,
            scale: Vec3::splat(scale),
        },
        arm: Transform {
            translation: f.j - arm_q * f.j,
            rotation: arm_q,
            scale: Vec3::ONE,
        },
        fist,
        aim: d,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::held::stroke::{Meets, Motion, StrokePose, drawn, punch_keys};
    use pbd_core::inventory::Tool;

    fn fist() -> (FistFrame, Vec3, f32) {
        let models = crate::held::models();
        let model = models.get(Tool::Pickaxe);
        let centre = model
            .rest()
            .compute_affine()
            .inverse()
            .transform_point3(model.fist.into());
        let bones = pbd_core::hand::fist_bones(&model.bones, &models.hand, &FistShape::V16);
        (
            FistFrame::new(&bones, centre, &models.hand),
            model.fist.into(),
            model.scale,
        )
    }

    /// The frame is orthonormal and the forearm runs off from the wrist
    /// away from the knuckles.
    #[test]
    fn the_fists_frame_is_square() {
        let (f, _, _) = fist();
        for v in [f.a, f.k, f.b, f.w] {
            assert!((v.length() - 1.0).abs() < 1e-4, "{f:?}");
        }
        assert!(
            f.a.dot(f.k).abs() < 1e-4 && f.a.dot(f.b).abs() < 1e-4 && f.k.dot(f.b).abs() < 1e-4
        );
        assert!(
            f.w.dot(f.k) < 0.0,
            "the forearm runs back from the knuckles: {f:?}"
        );
    }

    /// Version 16: at the hit the knuckles aim forward into the view, a
    /// little left and up (about (-0.6, 0.2, -0.77)); the back of the hand
    /// faces right and a little toward the eye; the fist lands below and
    /// left of the crosshair, the forearm in from the lower right, and the
    /// wrist is straight (the arm turned onto the back of the hand's line).
    #[test]
    fn the_fist_punches_forward_into_the_view() {
        let (f, fist0, scale) = fist();
        let m = Motion::default();
        let keys = punch_keys(&m, fist0, Meets::Breaks);
        let hit = drawn(
            &m,
            fist0,
            StrokePose::Fist(keys.hit),
            StrokePose::Fist(keys.hit),
        );
        let a = aimed(&f, &m.punch, &FistShape::V16, fist0, scale, hit);
        assert!(
            a.aim.z < -0.6 && a.aim.x < -0.3 && a.aim.y > 0.0,
            "{:?}",
            a.aim
        );
        // The back of the hand faces `back_eye` square to the forearm: to
        // the right, the fist driving away from the eye.
        let back = a.holder.rotation * f.b;
        let be = Vec3::from(FistShape::V16.back_eye);
        let want = (be - a.aim * be.dot(a.aim)).normalize();
        assert!(
            back.dot(want) > 0.9999,
            "the back of the hand: {back} {want}"
        );
        assert!(back.x > 0.5, "to the right: {back}");
        // Below and left of the crosshair.
        let ndc = Vec2::new(a.fist.x, a.fist.y) / -a.fist.z;
        assert!(ndc.x < 0.0 && ndc.y < 0.0, "{ndc}");
        // The wrist is straight: the arm's way, turned, is the knuckles'
        // back the other way.
        let forearm = a.holder.rotation * (a.arm.rotation * f.w);
        let knuckles = a.holder.rotation * f.k;
        assert!(forearm.dot(-knuckles) > 0.999, "{forearm} {knuckles}");
        // The wrist sits on the line from the elbow through the fist.
        let wrist = a.holder.transform_point(f.j);
        let off = (wrist - a.fist).cross(a.aim).length();
        assert!(off < 1e-4, "the wrist on the forearm's line: {off}");
        // At the guard the forearm comes from the lower right too.
        let rest = drawn(
            &m,
            fist0,
            StrokePose::Fist(HeldPose::default()),
            StrokePose::Fist(HeldPose::default()),
        );
        let g = aimed(&f, &m.punch, &FistShape::V16, fist0, scale, rest);
        let elbow_side = g.fist - g.aim * 0.3;
        assert!(elbow_side.x > g.fist.x && elbow_side.y < g.fist.y);
    }
}
