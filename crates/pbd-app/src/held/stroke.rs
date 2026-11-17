//! The dig and chop stroke (`openspec/changes/held-swing`, decisions 5-9),
//! ported from the approved Tools in Hand mockup (M36 v8,
//! `docs/mockups/tool-motion.html`: `timings`, `keys`, `strokeAt`,
//! `strokePose`, `shovelKeys`, `shovelAt`, `Schedule`).
//!
//! - One steady beat (decision 5): a block and the pause after it,
//!   L = T + between_s, take n = max(1, round(L / stroke_target_s)) strokes
//!   of one period P = L / n; the k-th impact of cycle c lands at
//!   c L + T - (n - k) P, the last on the break.
//! - The pickaxe and the axe swing from the shoulder (decision 6) in four
//!   stages, windup, anticipation, hit and recoil (decision 7), keyed poses
//!   blended by curves, with the head lagging the fist and overlapping it at
//!   the hit (decision 8).
//! - The shovel tips down, slides into the dirt, tips about the blade's
//!   point and pulls up (decision 9).
//!
//! Drawn only: the dig's timer is read, never fed. Eye space, metres: x
//! right, y up, -z ahead. Pitch is about eye x, tip up and back positive.

use super::HeldPose;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// The stroke of the pickaxe and the axe, and the parts the shovel shares
/// (its stage times, the beat, the weights), in `held.stroke`. Times s,
/// angles rad, lengths m, eye space.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct StrokeSettings {
    /// Windup: the tool raised up and back, the arm raised about the
    /// shoulder and gathered.
    pub windup_s: f32,
    pub windup_rad: f32,
    pub raise_rad: f32,
    pub coil_m: f32,
    /// Anticipation: a little further back, and a pause at the top.
    pub anticipate_s: f32,
    pub anticipate_rad: f32,
    /// The hit: the sweep to full reach (the tool's pitch forward, the arm
    /// down past rest, stretched, and swung across), eased in with t cubed.
    pub strike_s: f32,
    pub strike_rad: f32,
    pub reach_rad: f32,
    pub extend_m: f32,
    pub sweep_rad: f32,
    /// A held frame on contact.
    pub impact_hold_s: f32,
    /// The recoil: the tool bounces back off the block, the arm follows
    /// through, then eases to rest over `recover_s`.
    pub recoil_s: f32,
    pub recoil_rad: f32,
    pub follow_rad: f32,
    pub recover_s: f32,
    /// The head lags the fist, s, and overlaps it at the hit, rad.
    pub lag_s: f32,
    pub overlap_rad: f32,
    /// The beat's target period, s.
    pub stroke_target_s: f32,
    /// A camera kick on the impact, rad. The game does not kick the
    /// walking camera (the player's look is the raw mouse), so it is 0, the
    /// approved value; any other value is refused.
    pub camera_kick_rad: f32,
    /// Scales the raise, the recoil and the shovel's lever and lift.
    pub exaggerate: f32,
    /// The virtual shoulder the arm turns about, eye space, m.
    pub shoulder: [f32; 3],
    /// Heavier tools wind up slower and hold the impact longer: each
    /// tool's weight, and how much slower each tier below steel is.
    pub pickaxe_weight: f32,
    pub axe_weight: f32,
    pub shovel_weight: f32,
    pub tier_slow: f32,
    /// How much longer the sleeve is drawn than the rig's, tool units, so
    /// the arm runs off the screen at full reach.
    pub sleeve_extra: f32,
}

impl Default for StrokeSettings {
    fn default() -> Self {
        Self {
            windup_s: 0.10,
            windup_rad: 0.35,
            raise_rad: 0.42,
            coil_m: 0.03,
            anticipate_s: 0.07,
            anticipate_rad: 0.08,
            strike_s: 0.06,
            strike_rad: 0.55,
            reach_rad: 0.05,
            extend_m: 0.12,
            sweep_rad: 0.25,
            impact_hold_s: 0.05,
            recoil_s: 0.06,
            recoil_rad: 0.15,
            follow_rad: 0.08,
            recover_s: 0.16,
            lag_s: 0.03,
            overlap_rad: 0.06,
            stroke_target_s: 0.45,
            camera_kick_rad: 0.0,
            exaggerate: 1.0,
            shoulder: [0.2, -0.25, 0.05],
            pickaxe_weight: 1.15,
            axe_weight: 1.1,
            shovel_weight: 0.85,
            tier_slow: 0.04,
            sleeve_extra: 6.0,
        }
    }
}

impl StrokeSettings {
    pub fn validate(&self) -> Result<(), String> {
        let signed = [
            self.reach_rad,
            self.sweep_rad,
            self.shoulder[0],
            self.shoulder[1],
            self.shoulder[2],
        ];
        let sizes = [
            self.windup_s,
            self.windup_rad,
            self.raise_rad,
            self.coil_m,
            self.anticipate_s,
            self.anticipate_rad,
            self.strike_s,
            self.strike_rad,
            self.extend_m,
            self.impact_hold_s,
            self.recoil_s,
            self.recoil_rad,
            self.follow_rad,
            self.recover_s,
            self.lag_s,
            self.overlap_rad,
            self.stroke_target_s,
            self.camera_kick_rad,
            self.exaggerate,
            self.pickaxe_weight,
            self.axe_weight,
            self.shovel_weight,
            self.tier_slow,
            self.sleeve_extra,
        ];
        if signed.iter().any(|v| !v.is_finite()) || sizes.iter().any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(
                "held.stroke: every value is finite, and every size, time and weight non-negative"
                    .into(),
            );
        }
        if self.strike_s <= 0.0 || self.recoil_s <= 0.0 || self.recover_s <= 0.0 {
            return Err("held.stroke: strike_s, recoil_s and recover_s are positive".into());
        }
        if self.stroke_target_s <= 0.0 {
            return Err("held.stroke: stroke_target_s is positive".into());
        }
        if self.camera_kick_rad != 0.0 {
            return Err(
                "held.stroke: camera_kick_rad is 0; the game does not kick the camera".into(),
            );
        }
        Ok(())
    }

    /// A tool's weight at its tier (`Tier::digit`, stone 0 to steel 3).
    pub fn weight(&self, tool: StrokeTool, tier: u8) -> f32 {
        let w = match tool {
            StrokeTool::Pickaxe => self.pickaxe_weight,
            StrokeTool::Axe => self.axe_weight,
            StrokeTool::Shovel => self.shovel_weight,
            StrokeTool::Hands => 1.0,
        };
        w * (1.0 + self.tier_slow * (3.0 - f32::from(tier.min(3))))
    }
}

/// The shovel's stroke (decision 9), in `held.shovel`. Its stage times are
/// the stroke's, with `scoop_s` in place of `recoil_s`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShovelSettings {
    /// The entry: how far below level the spade lies, degrees, with the
    /// view looking down `DIG_PITCH`.
    pub entry_deg: f32,
    /// The slide forward into the dirt, m, sinking as far, m.
    pub slide_m: f32,
    pub slide_drop_m: f32,
    /// The tip about the blade's point in the dirt, rad.
    pub lever_rad: f32,
    /// The pull up and back, m.
    pub lift_m: f32,
    pub pull_m: f32,
    /// The scoop (the tip and the pull up), s.
    pub scoop_s: f32,
    /// How far toward the middle the fist comes at the hit, m (0, a
    /// straight wrist).
    pub across_m: f32,
    /// How much of the spade's turn the arm follows (1, a straight wrist).
    pub arm_follow: f32,
    /// Where the fist sits at the hit: below and ahead of its rest, m.
    pub fist_drop_m: f32,
    pub fist_fwd_m: f32,
}

impl Default for ShovelSettings {
    fn default() -> Self {
        Self {
            entry_deg: 6.0,
            slide_m: 0.28,
            slide_drop_m: 0.03,
            lever_rad: 0.5,
            lift_m: 0.14,
            pull_m: 0.05,
            scoop_s: 0.26,
            across_m: 0.0,
            arm_follow: 1.0,
            fist_drop_m: 0.04,
            fist_fwd_m: 0.12,
        }
    }
}

impl ShovelSettings {
    pub fn validate(&self) -> Result<(), String> {
        let all = [
            self.entry_deg,
            self.slide_m,
            self.slide_drop_m,
            self.lever_rad,
            self.lift_m,
            self.pull_m,
            self.scoop_s,
            self.across_m,
            self.arm_follow,
            self.fist_drop_m,
            self.fist_fwd_m,
        ];
        if all.iter().any(|v| !v.is_finite()) {
            return Err("held.shovel: every value is finite".into());
        }
        let sizes = [
            self.slide_m,
            self.slide_drop_m,
            self.lever_rad,
            self.lift_m,
            self.pull_m,
            self.across_m,
        ];
        if sizes.iter().any(|v| *v < 0.0) || self.scoop_s <= 0.0 {
            return Err("held.shovel: sizes are non-negative and scoop_s positive".into());
        }
        if !(0.0..=1.0).contains(&self.arm_follow) {
            return Err("held.shovel: arm_follow is 0..1".into());
        }
        Ok(())
    }
}

/// The view's pitch the shovel's keys are set for, rad: the mockup's Dig
/// tab looks down this far, and the entry is laid out in eye space from it.
pub const DIG_PITCH: f32 = -0.56;

/// Which stroke a tool in hand makes. Bare hands keep the beat (it times
/// the dig sounds) but draw nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrokeTool {
    Pickaxe,
    Axe,
    Shovel,
    Hands,
}

/// Where in a stroke the tool is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Rest,
    Windup,
    Anticipation,
    Hit,
    Recoil,
    /// The shovel's recoil: the tip about the blade, then the pull up.
    Tip,
    PullUp,
}

/// A stroke's stage times for a tool of weight `w` that must fit the
/// period `period`: the windup, the anticipation and the recover shorten in
/// proportion when it would not fit (down to a tenth); the hit, its hold
/// and the recoil never do.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Timings {
    pub windup: f32,
    pub anticipate: f32,
    pub strike: f32,
    pub hold: f32,
    pub recoil: f32,
    pub recover: f32,
}

impl Timings {
    pub fn new(
        s: &StrokeSettings,
        v: &ShovelSettings,
        tool: StrokeTool,
        w: f32,
        period: f32,
    ) -> Self {
        let (mut wu, mut an, mut rv) = (s.windup_s * w, s.anticipate_s * w, s.recover_s);
        let st = s.strike_s;
        let ho = s.impact_hold_s * w;
        let rc = if tool == StrokeTool::Shovel {
            v.scoop_s
        } else {
            s.recoil_s
        };
        let (fixed, soft) = (st + ho + rc, wu + an + rv);
        if period > 0.0 && fixed + soft > period {
            let k = ((period - fixed) / soft).max(0.1);
            wu *= k;
            an *= k;
            rv *= k;
        }
        Self {
            windup: wu,
            anticipate: an,
            strike: st,
            hold: ho,
            recoil: rc,
            recover: rv,
        }
    }

    /// From the stroke's start to its impact, s.
    pub fn impact(&self) -> f32 {
        self.windup + self.anticipate + self.strike
    }

    pub fn len(&self) -> f32 {
        self.impact() + self.hold + self.recoil + self.recover
    }
}

/// The steady beat (decision 5): `n` strokes of one period over each block
/// and its pause.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Beat {
    /// The block's break time, s.
    pub break_s: f32,
    /// A block and its pause, s.
    pub cycle_s: f32,
    pub n: u32,
    pub period: f32,
}

impl Beat {
    pub fn new(s: &StrokeSettings, break_s: f32, between_s: f32) -> Self {
        let break_s = break_s.max(0.0);
        let cycle_s = (break_s + between_s.max(0.0)).max(1e-3);
        let n = ((cycle_s / s.stroke_target_s + 1e-9).round() as u32).max(1);
        Self {
            break_s,
            cycle_s,
            n,
            period: cycle_s / n as f32,
        }
    }

    /// When the k-th impact (1..n) of cycle c lands, s from the hold's
    /// start.
    pub fn impact(&self, cycle: i64, k: u32) -> f32 {
        cycle as f32 * self.cycle_s + self.break_s - (self.n - k) as f32 * self.period
    }

    /// The impacts in (t0, t1], each with whether it is a block's last (the
    /// break).
    pub fn impacts(&self, t0: f32, t1: f32) -> Vec<(f32, bool)> {
        let mut out = Vec::new();
        let c0 = (t0 / self.cycle_s).floor().max(0.0) as i64;
        let c1 = (t1 / self.cycle_s).floor() as i64 + 1;
        for c in c0..=c1 {
            for k in 1..=self.n {
                let t = self.impact(c, k);
                if t > t0 && t <= t1 {
                    out.push((t, k == self.n));
                }
            }
        }
        out
    }

    /// When each stroke round `t` started, in order (the mockup's `starts`
    /// filtered to those begun by `t`).
    fn starts(&self, t: f32, impact: f32) -> Vec<f32> {
        let c = (t / self.cycle_s).floor() as i64;
        let mut out = Vec::new();
        for cc in (c - 1).max(0)..=c + 1 {
            for k in 1..=self.n {
                let at = self.impact(cc, k) - impact;
                if at <= t {
                    out.push(at);
                }
            }
        }
        out
    }
}

/// The pickaxe's and the axe's pose in a stroke: the tool's forward pitch
/// about the fist `a`, the arm's angle about the shoulder `phi` (raised
/// positive), its stretch `ext` (m), the head's overlap past the fist `ov`
/// and the arm's swing across `yaw`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ArmPose {
    pub a: f32,
    pub phi: f32,
    pub ext: f32,
    pub ov: f32,
    pub yaw: f32,
}

impl ArmPose {
    fn lerp(self, to: Self, t: f32) -> Self {
        let l = |a: f32, b: f32| a + (b - a) * t;
        Self {
            a: l(self.a, to.a),
            phi: l(self.phi, to.phi),
            ext: l(self.ext, to.ext),
            ov: l(self.ov, to.ov),
            yaw: l(self.yaw, to.yaw),
        }
    }
}

/// The shovel's pose: its pitch and the shift of the fist, eye space.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpadePose {
    pub pitch: f32,
    pub shift: Vec3,
}

impl SpadePose {
    fn lerp(self, to: Self, t: f32) -> Self {
        Self {
            pitch: self.pitch + (to.pitch - self.pitch) * t,
            shift: self.shift.lerp(to.shift, t),
        }
    }
}

/// A pose in a stroke and where in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokePose {
    Arm(ArmPose),
    Spade(SpadePose),
}

impl StrokePose {
    fn rest(tool: StrokeTool) -> Self {
        if tool == StrokeTool::Shovel {
            Self::Spade(SpadePose::default())
        } else {
            Self::Arm(ArmPose::default())
        }
    }

    /// Part of the way to rest.
    pub fn toward_rest(self, t: f32) -> Self {
        match self {
            Self::Arm(p) => Self::Arm(p.lerp(ArmPose::default(), t)),
            Self::Spade(p) => Self::Spade(p.lerp(SpadePose::default(), t)),
        }
    }
}

/// What a stroke is at a moment: the pose, the stage, and whether the
/// shovel carries a clump of dirt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeAt {
    pub pose: StrokePose,
    pub stage: Stage,
    pub scoop: bool,
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// Out past the end and back: the recoil's overshoot.
fn back_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let (c1, c3) = (1.4, 2.4);
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

/// The four keyed poses of the pick's and the axe's chop.
#[derive(Clone, Copy, Debug)]
pub struct ArmKeys {
    pub top: ArmPose,
    pub coil: ArmPose,
    pub hit: ArmPose,
    pub recoil: ArmPose,
}

pub fn arm_keys(s: &StrokeSettings) -> ArmKeys {
    let ex = s.exaggerate;
    ArmKeys {
        top: ArmPose {
            a: -s.windup_rad * ex,
            phi: s.raise_rad * ex,
            ext: -s.coil_m,
            ov: 0.0,
            yaw: -0.08 * ex,
        },
        coil: ArmPose {
            a: -(s.windup_rad + s.anticipate_rad) * ex,
            phi: (s.raise_rad + s.anticipate_rad * 0.5) * ex,
            ext: -s.coil_m * 1.25,
            ov: 0.0,
            yaw: -0.1 * ex,
        },
        hit: ArmPose {
            a: s.strike_rad,
            phi: -s.reach_rad,
            ext: s.extend_m,
            ov: s.overlap_rad,
            yaw: s.sweep_rad,
        },
        recoil: ArmPose {
            a: s.strike_rad - s.recoil_rad * ex,
            phi: -(s.reach_rad + s.follow_rad),
            ext: s.extend_m * 0.7,
            ov: 0.0,
            yaw: s.sweep_rad * 0.9,
        },
    }
}

/// The shovel's keyed poses: the windup's top, the coil before the slide,
/// the hit in the dirt, the lever about the blade's point and the lift.
#[derive(Clone, Copy, Debug)]
pub struct SpadeKeys {
    pub top: SpadePose,
    pub coil: SpadePose,
    pub hit: SpadePose,
    pub lever: SpadePose,
    pub lift: SpadePose,
    /// Where the blade's point is at the hit, eye space: the lever turns
    /// about it.
    pub point: Vec3,
    /// The lever's full turn, rad.
    pub lever_rad: f32,
}

impl SpadeKeys {
    /// Part of the lever's turn about the blade's point.
    pub fn lever_at(&self, k: f32, fist: Vec3) -> SpadePose {
        let r = self.lever_rad * k;
        let f = Quat::from_rotation_x(r) * (fist + self.hit.shift - self.point) + self.point;
        SpadePose {
            pitch: self.hit.pitch + r,
            shift: f - fist,
        }
    }
}

/// The shovel's keys (`shovelKeys`), from the shovel's fist and tip at rest,
/// eye space.
pub fn spade_keys(s: &StrokeSettings, v: &ShovelSettings, fist: Vec3, tip: Vec3) -> SpadeKeys {
    let ex = s.exaggerate;
    let tip_r = tip - fist;
    // The handle's elevation at rest, and the entry's, eye space.
    let a0 = tip_r.y.atan2(-tip_r.z);
    let entry_eye = -(v.entry_deg.to_radians() + DIG_PITCH);
    let entry = entry_eye - a0;
    let tip_at = |pitch: f32, shift: Vec3| Quat::from_rotation_x(pitch) * tip_r + fist + shift;
    // The fist is placed and the blade follows: below, ahead and across.
    let hit = SpadePose {
        pitch: entry,
        shift: Vec3::new(-v.across_m, -v.fist_drop_m, -v.fist_fwd_m),
    };
    // Back toward the player and up off the dirt by the slide.
    let coil = SpadePose {
        pitch: entry,
        shift: hit.shift + Vec3::new(0.0, v.slide_drop_m, v.slide_m),
    };
    let top = SpadePose {
        pitch: entry * 0.85,
        shift: coil.shift + Vec3::new(0.0, 0.03 * ex, 0.02),
    };
    let mut keys = SpadeKeys {
        top,
        coil,
        hit,
        lever: hit,
        lift: hit,
        point: tip_at(hit.pitch, hit.shift),
        lever_rad: v.lever_rad * ex,
    };
    keys.lever = keys.lever_at(1.0, fist);
    keys.lift = SpadePose {
        pitch: keys.lever.pitch,
        shift: keys.lever.shift + Vec3::new(0.0, v.lift_m * ex, v.pull_m),
    };
    keys
}

/// Everything a stroke needs to pose a tool: the settings, the tool, its
/// stage times, and the shovel's keys and fist.
#[derive(Clone, Copy, Debug)]
pub struct Stroker {
    pub tool: StrokeTool,
    pub times: Timings,
    pub arm: ArmKeys,
    pub spade: SpadeKeys,
    pub fist: Vec3,
}

/// The tool a stroke is made with: which, its weight at its tier, and its
/// fist and tip at rest, eye space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gear {
    pub tool: StrokeTool,
    pub weight: f32,
    pub fist: Vec3,
    pub tip: Vec3,
}

impl Stroker {
    pub fn new(s: &StrokeSettings, v: &ShovelSettings, gear: Gear, period: f32) -> Self {
        Self {
            tool: gear.tool,
            times: Timings::new(s, v, gear.tool, gear.weight, period),
            arm: arm_keys(s),
            spade: spade_keys(s, v, gear.fist, gear.tip),
            fist: gear.fist,
        }
    }

    /// The pose `e` seconds into a stroke begun from `from`.
    pub fn at(&self, e: f32, from: StrokePose) -> StrokeAt {
        if self.tool == StrokeTool::Shovel {
            return self.spade_at(e);
        }
        let from = match from {
            StrokePose::Arm(p) => p,
            StrokePose::Spade(_) => ArmPose::default(),
        };
        let (k, t) = (&self.arm, &self.times);
        let arm = |p: ArmPose, stage| StrokeAt {
            pose: StrokePose::Arm(p),
            stage,
            scoop: false,
        };
        if e < 0.0 {
            return arm(from, Stage::Rest);
        }
        let mut e = e;
        if e < t.windup {
            return arm(from.lerp(k.top, ease_out(e / t.windup)), Stage::Windup);
        }
        e -= t.windup;
        if e < t.anticipate {
            return arm(
                k.top.lerp(k.coil, smooth(e / (t.anticipate * 0.7))),
                Stage::Anticipation,
            );
        }
        e -= t.anticipate;
        if e < t.strike {
            let u = e / t.strike;
            let u3 = u * u * u;
            let mut p = k.coil.lerp(k.hit, u3);
            p.ov = k.hit.ov * u3;
            return arm(p, Stage::Hit);
        }
        e -= t.strike;
        if e < t.hold {
            return arm(k.hit, Stage::Hit);
        }
        e -= t.hold;
        if e < t.recoil {
            let u = e / t.recoil;
            let mut p = k.hit.lerp(k.recoil, back_out(u));
            p.ov = k.hit.ov * (1.0 - ease_out(u));
            return arm(p, Stage::Recoil);
        }
        e -= t.recoil;
        if e < t.recover {
            return arm(
                k.recoil.lerp(ArmPose::default(), smooth(e / t.recover)),
                Stage::Recoil,
            );
        }
        arm(ArmPose::default(), Stage::Rest)
    }

    fn spade_at(&self, e: f32) -> StrokeAt {
        let (k, t) = (&self.spade, &self.times);
        let spade = |p: SpadePose, stage, scoop| StrokeAt {
            pose: StrokePose::Spade(p),
            stage,
            scoop,
        };
        let rest = SpadePose::default();
        if e < 0.0 {
            return spade(rest, Stage::Rest, false);
        }
        let mut e = e;
        if e < t.windup {
            return spade(
                rest.lerp(k.top, ease_out(e / t.windup)),
                Stage::Windup,
                false,
            );
        }
        e -= t.windup;
        if e < t.anticipate {
            return spade(
                k.top.lerp(k.coil, smooth(e / (t.anticipate * 0.7))),
                Stage::Anticipation,
                false,
            );
        }
        e -= t.anticipate;
        if e < t.strike {
            let u = e / t.strike;
            return spade(k.coil.lerp(k.hit, u * u * u), Stage::Hit, false);
        }
        e -= t.strike;
        if e < t.hold {
            return spade(k.hit, Stage::Hit, false);
        }
        e -= t.hold;
        if e < t.recoil {
            // Angle up about the blade in the ground, then lift.
            let u = e / t.recoil;
            if u < 0.45 {
                return spade(k.lever_at(smooth(u / 0.45), self.fist), Stage::Tip, true);
            }
            return spade(
                k.lever.lerp(k.lift, ease_out((u - 0.45) / 0.55)),
                Stage::PullUp,
                true,
            );
        }
        e -= t.recoil;
        if e < t.recover {
            let r = e / t.recover;
            // The clump tips off as the spade returns.
            return spade(k.lift.lerp(rest, smooth(r)), Stage::PullUp, r < 0.45);
        }
        spade(rest, Stage::Rest, false)
    }

    /// The pose at `t` on the beat: the last stroke begun by then, begun
    /// from wherever the one before it had the tool (the mockup's
    /// `poseAt`).
    pub fn on_beat(&self, beat: &Beat, t: f32) -> StrokeAt {
        let starts = beat.starts(t, self.times.impact());
        let rest = StrokePose::rest(self.tool);
        let Some(&last) = starts.last() else {
            return StrokeAt {
                pose: rest,
                stage: Stage::Rest,
                scoop: false,
            };
        };
        let mut from = rest;
        let first = starts.len().saturating_sub(4);
        for i in first..starts.len() - 1 {
            from = self.at(starts[i + 1] - starts[i], from).pose;
        }
        self.at(t - last, from)
    }
}

/// The pose as drawn (`strokePose`): for the pick and the axe, the fist on
/// its arc about the shoulder from `s`, the tool's pitch from the lagged
/// arm `lag` less its own forward pitch and overlap, and the arm's swing
/// across; for the shovel, its pitch and shift.
pub fn drawn(s: &StrokeSettings, fist0: Vec3, now: StrokePose, lag: StrokePose) -> HeldPose {
    match now {
        StrokePose::Spade(p) => HeldPose {
            pitch: p.pitch,
            shift: p.shift,
            ..default()
        },
        StrokePose::Arm(p) => {
            let lag = match lag {
                StrokePose::Arm(l) => l,
                StrokePose::Spade(_) => p,
            };
            let shoulder = Vec3::from(s.shoulder);
            let arm = fist0 - shoulder;
            let reach = arm.length().max(1e-4);
            let fist = Quat::from_rotation_y(p.yaw)
                * (Quat::from_rotation_x(p.phi) * (arm * ((reach + p.ext) / reach)))
                + shoulder;
            HeldPose {
                pitch: lag.phi - p.a - p.ov,
                roll: 0.0,
                yaw: lag.yaw,
                shift: fist - fist0,
            }
        }
    }
}

/// What the dig's rule says this frame, as the stroke reads it: whether a
/// hold is working (a block being broken, or the pause after one), the
/// block's break time and how far into it and its pause the hold is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DigBeat {
    pub working: bool,
    pub break_s: f32,
    pub between_s: f32,
    /// 0..break_s while breaking, then on through the pause.
    pub local_s: f32,
}

/// What the stroke drew this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StrokeFrame {
    pub pose: HeldPose,
    pub stage: Option<Stage>,
    pub scoop: bool,
    /// Impacts landed this frame: each is a block's last (the break) or
    /// not.
    pub impacts: Vec<bool>,
}

/// The stroke's own clock, kept by the hold system: which cycle of the hold
/// it is in, and the moves from wherever the tool was (letting go, a change
/// of block, tool or beat) eased over `recover_s` so nothing pops.
#[derive(Clone, Debug, Default)]
pub struct StrokeClock {
    cycle: i64,
    last: Option<(DigBeat, StrokeTool)>,
    /// The beat's time last frame, s.
    t: f32,
    /// The pose and its lag last frame, to ease from.
    now: Option<(StrokePose, StrokePose)>,
    /// Letting go: easing back to rest from a pose, s since.
    release: Option<(StrokePose, StrokePose, f32)>,
    /// Taking up again from where the tool was drawn: the pose, s since.
    carry: Option<(HeldPose, f32)>,
    drawn: HeldPose,
}

impl StrokeClock {
    /// One frame of the stroke.
    pub fn step(
        &mut self,
        s: &StrokeSettings,
        v: &ShovelSettings,
        gear: Gear,
        dig: DigBeat,
        dt: f32,
    ) -> StrokeFrame {
        let dt = dt.clamp(0.0, 0.1);
        let mut frame = StrokeFrame::default();
        let rest = StrokePose::rest(gear.tool);
        let (now, lag) = if dig.working {
            self.release = None;
            let beat = Beat::new(s, dig.break_s, dig.between_s);
            let stroker = Stroker::new(s, v, gear, beat.period);
            let t0;
            match self.last {
                Some((was, tool))
                    if tool == gear.tool
                        && (was.break_s - dig.break_s).abs() < 1e-4
                        && (was.between_s - dig.between_s).abs() < 1e-4 =>
                {
                    if dig.local_s + 1e-4 < was.local_s {
                        if was.local_s >= was.break_s - 1e-3 {
                            // The pause ran out and the next block began: on
                            // round the beat.
                            self.cycle += 1;
                            t0 = self.t;
                        } else {
                            // Started again mid-block (aimed elsewhere).
                            self.restart();
                            t0 = -1e-6;
                        }
                    } else {
                        t0 = self.t;
                    }
                }
                _ => {
                    self.restart();
                    t0 = -1e-6;
                }
            }
            let t = self.cycle as f32 * beat.cycle_s + dig.local_s;
            frame.impacts = beat
                .impacts(t0, t)
                .into_iter()
                .map(|(_, last)| last)
                .collect();
            self.t = t;
            let at = stroker.on_beat(&beat, t);
            let lagged = stroker.on_beat(&beat, (t - s.lag_s).max(0.0));
            frame.stage = Some(at.stage);
            frame.scoop = at.scoop;
            self.last = Some((dig, gear.tool));
            (at.pose, lagged.pose)
        } else {
            if self.last.take().is_some()
                && let Some((now, lag)) = self.now
            {
                self.release = Some((now, lag, 0.0));
            }
            match &mut self.release {
                Some((now, lag, since)) => {
                    *since += dt;
                    let k = smooth(*since / s.recover_s);
                    let pair = (now.toward_rest(k), lag.toward_rest(k));
                    if k >= 1.0 {
                        self.release = None;
                    }
                    pair
                }
                None => (rest, rest),
            }
        };
        self.now = Some((now, lag));
        let mut pose = drawn(s, gear.fist, now, lag);
        if let Some((from, since)) = &mut self.carry {
            *since += dt;
            let k = smooth(*since / s.recover_s);
            pose = from.lerp(pose, k);
            if k >= 1.0 {
                self.carry = None;
            }
        }
        self.drawn = pose;
        frame.pose = pose;
        frame
    }

    /// A new hold, or one broken off: the beat from nought, eased from
    /// wherever the tool is drawn.
    fn restart(&mut self) {
        self.cycle = 0;
        self.t = 0.0;
        if self.now.is_some() && self.drawn != HeldPose::default() {
            self.carry = Some((self.drawn, 0.0));
        }
        self.release = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::inventory::Tool;

    const DT: f32 = 1.0 / 60.0;
    const BETWEEN: f32 = 0.15;

    fn gear(tool: StrokeTool) -> Gear {
        let model = crate::held::models().get(match tool {
            StrokeTool::Shovel => Tool::Shovel,
            StrokeTool::Axe => Tool::Axe,
            _ => Tool::Pickaxe,
        });
        Gear {
            tool,
            weight: 1.0,
            fist: model.fist.into(),
            tip: model.tip.into(),
        }
    }

    fn near(a: f32, b: f32, by: f32) -> bool {
        (a - b).abs() <= by
    }

    /// Decision 5: n = max(1, round(L / stroke_target_s)) strokes of one
    /// period over each block and its pause, the last impact on the break,
    /// and the impacts one period apart from block to block: no sets with
    /// gaps between them. The stone axe on wood (0.6 s) strikes twice, the
    /// second impact when the block breaks (the spec's scenario).
    #[test]
    fn one_steady_beat_with_the_last_impact_on_the_break() {
        let s = StrokeSettings::default();
        let wood = Beat::new(&s, 0.6, BETWEEN);
        assert_eq!(wood.n, 2);
        let first: Vec<(f32, bool)> = wood.impacts(-1e-6, wood.cycle_s);
        assert_eq!(first.len(), 2, "{first:?}");
        assert!(near(first[1].0, 0.6, 1e-5) && first[1].1, "{first:?}");
        assert!(!first[0].1);
        for break_s in [0.1, 0.5, 0.6, 1.2, 1.6, 2.0, 4.0] {
            let beat = Beat::new(&s, break_s, BETWEEN);
            let l = break_s + BETWEEN;
            assert_eq!(beat.n, ((l / s.stroke_target_s).round() as u32).max(1));
            assert!(near(beat.period, l / beat.n as f32, 1e-6));
            let all = beat.impacts(-1e-6, 5.0 * l - 1e-4);
            assert_eq!(all.len() as u32, 5 * beat.n, "{break_s}");
            for pair in all.windows(2) {
                assert!(
                    near(pair[1].0 - pair[0].0, beat.period, 1e-4),
                    "{break_s}: {all:?}"
                );
            }
            for (c, last) in all.iter().filter(|(_, last)| *last).enumerate() {
                assert!(
                    near(last.0, c as f32 * l + break_s, 1e-4),
                    "{break_s}: on the break"
                );
            }
        }
    }

    /// Decision 7: windup, anticipation, hit (with its held frame) and
    /// recoil, in order at their times, the tool at the hit key on the
    /// impact; the shovel's recoil is the tip and the pull up, carrying
    /// dirt.
    #[test]
    fn each_stage_at_its_time() {
        let (s, v) = (StrokeSettings::default(), ShovelSettings::default());
        let pick = Stroker::new(&s, &v, gear(StrokeTool::Pickaxe), 10.0);
        let t = pick.times;
        assert!(near(t.windup, s.windup_s, 1e-6) && near(t.recover, s.recover_s, 1e-6));
        let rest = StrokePose::Arm(ArmPose::default());
        let stage = |e: f32| pick.at(e, rest).stage;
        let w = t.windup;
        let a = w + t.anticipate;
        let i = t.impact();
        let h = i + t.hold;
        let r = h + t.recoil;
        assert_eq!(stage(0.5 * w), Stage::Windup);
        assert_eq!(stage(w + 0.5 * t.anticipate), Stage::Anticipation);
        assert_eq!(stage(a + 0.5 * t.strike), Stage::Hit);
        assert_eq!(stage(i + 0.5 * t.hold), Stage::Hit);
        assert_eq!(stage(h + 0.5 * t.recoil), Stage::Recoil);
        assert_eq!(stage(r + 0.5 * t.recover), Stage::Recoil);
        assert_eq!(stage(t.len() + 0.01), Stage::Rest);
        let keys = arm_keys(&s);
        assert_eq!(pick.at(i + 1e-4, rest).pose, StrokePose::Arm(keys.hit));
        // The hit is fastest at the impact: the last tenth of the strike
        // moves the tool more than the first half.
        let pitch = |e: f32| match pick.at(e, rest).pose {
            StrokePose::Arm(p) => p.a,
            _ => unreachable!(),
        };
        assert!(pitch(i - 1e-4) - pitch(i - 0.1 * t.strike) > pitch(a + 0.5 * t.strike) - pitch(a));
        let spade = Stroker::new(&s, &v, gear(StrokeTool::Shovel), 10.0);
        let t = spade.times;
        assert!(near(t.recoil, v.scoop_s, 1e-6));
        let h = t.impact() + t.hold;
        let at = |e: f32| spade.at(e, StrokePose::Spade(SpadePose::default()));
        assert_eq!(at(0.5 * t.windup).stage, Stage::Windup);
        assert_eq!(at(t.impact() - 1e-4).stage, Stage::Hit);
        let tip = at(h + 0.2 * t.recoil);
        assert!(tip.stage == Stage::Tip && tip.scoop);
        let pull = at(h + 0.8 * t.recoil);
        assert!(pull.stage == Stage::PullUp && pull.scoop);
        assert!(
            !at(h + t.recoil + 0.9 * t.recover).scoop,
            "the clump tips off"
        );
        assert!(!at(0.5 * t.windup).scoop);
    }

    /// A stroke that would not fit its period shortens its windup, its
    /// anticipation and its recover in proportion; the hit, its hold and
    /// the recoil never shorten.
    #[test]
    fn the_hit_is_never_shortened() {
        let (s, v) = (StrokeSettings::default(), ShovelSettings::default());
        let w = 1.15;
        let full = Timings::new(&s, &v, StrokeTool::Pickaxe, w, 0.0);
        for period in [2.0, 0.6, 0.45, 0.35, 0.25, 0.05] {
            let t = Timings::new(&s, &v, StrokeTool::Pickaxe, w, period);
            assert_eq!(t.strike, s.strike_s);
            assert!(near(t.hold, s.impact_hold_s * w, 1e-6));
            assert_eq!(t.recoil, s.recoil_s);
            let k = t.windup / full.windup;
            assert!(near(t.anticipate / full.anticipate, k, 1e-5));
            assert!(near(t.recover / full.recover, k, 1e-5));
            if full.len() <= period {
                assert!(near(k, 1.0, 1e-6), "{period}: fits, unshortened");
            } else if k > 0.1 + 1e-4 {
                assert!(near(t.len(), period, 1e-4), "{period}: fits the period");
            } else {
                assert!(near(k, 0.1, 1e-5), "{period}: a tenth at least");
            }
        }
    }

    /// Decision 6: the fist moves on an arc about the shoulder; it rises up
    /// and back on the windup and reaches out at the hit, the arm longer
    /// by `extend_m`; at rest it is where it was drawn.
    #[test]
    fn the_arm_swings_from_the_shoulder() {
        let s = StrokeSettings::default();
        let fist = gear(StrokeTool::Pickaxe).fist;
        let shoulder = Vec3::from(s.shoulder);
        let keys = arm_keys(&s);
        let arm = |p: ArmPose| drawn(&s, fist, StrokePose::Arm(p), StrokePose::Arm(p));
        let rest = arm(ArmPose::default());
        assert!(rest.shift.length() < 1e-6 && rest.pitch.abs() < 1e-6);
        let hit = arm(keys.hit);
        let reach = (fist + hit.shift - shoulder).length();
        assert!(near(reach, (fist - shoulder).length() + s.extend_m, 1e-5));
        assert!(hit.shift.y < 0.0, "down past rest: {hit:?}");
        assert!(near(hit.yaw, s.sweep_rad, 1e-6), "swung across");
        let top = arm(keys.top);
        assert!(
            top.shift.y > 0.05 && top.shift.z > 0.0,
            "up and back: {top:?}"
        );
        assert!(top.pitch > 0.5, "the tool raised back: {top:?}");
        // The lag: the tool's pitch follows the arm a moment behind.
        let lagged = drawn(
            &s,
            fist,
            StrokePose::Arm(keys.hit),
            StrokePose::Arm(keys.top),
        );
        assert!(lagged.pitch > hit.pitch);
        assert_eq!(lagged.shift, hit.shift);
    }

    /// Decision 9: the spade tips down to the entry angle at the hit, slides
    /// forward along the ground into it, tips about the blade's point, and
    /// pulls up and back; the fist never moves across (a straight wrist).
    #[test]
    fn the_shovels_keyed_poses() {
        let (s, v) = (StrokeSettings::default(), ShovelSettings::default());
        let g = gear(StrokeTool::Shovel);
        let k = spade_keys(&s, &v, g.fist, g.tip);
        let tip_r = g.tip - g.fist;
        let handle = Quat::from_rotation_x(k.hit.pitch) * tip_r;
        let elevation = handle.y.atan2(-handle.z);
        assert!(near(
            elevation,
            -(v.entry_deg.to_radians() + DIG_PITCH),
            1e-5
        ));
        // In the world, with the view looking down DIG_PITCH, the handle
        // runs down to the ground ahead.
        assert!(
            elevation + DIG_PITCH < 0.0,
            "the tip points down at the ground ahead"
        );
        assert_eq!(
            k.hit.shift,
            Vec3::new(-v.across_m, -v.fist_drop_m, -v.fist_fwd_m)
        );
        assert_eq!(k.hit.shift.x, 0.0, "a straight wrist");
        let slide = k.coil.shift - k.hit.shift;
        assert!((slide - Vec3::new(0.0, v.slide_drop_m, v.slide_m)).length() < 1e-6);
        assert_eq!(k.coil.pitch, k.hit.pitch, "the slide keeps the angle");
        for f in [0.0, 0.3, 0.7, 1.0] {
            let p = k.lever_at(f, g.fist);
            let tip = Quat::from_rotation_x(p.pitch) * tip_r + g.fist + p.shift;
            assert!(
                tip.distance(k.point) < 1e-5,
                "the lever turns about the point"
            );
        }
        assert!(
            near(k.lever.pitch - k.hit.pitch, v.lever_rad, 1e-6),
            "face up"
        );
        let lift = k.lift.shift - k.lever.shift;
        assert!((lift - Vec3::new(0.0, v.lift_m, v.pull_m)).length() < 1e-6);
        let spade = Stroker::new(&s, &v, g, 10.0);
        let drawn_hit = drawn(
            &s,
            g.fist,
            spade
                .at(
                    spade.times.impact(),
                    StrokePose::Spade(SpadePose::default()),
                )
                .pose,
            StrokePose::Spade(SpadePose::default()),
        );
        assert_eq!(drawn_hit.pitch, k.hit.pitch);
        assert_eq!(drawn_hit.yaw, 0.0);
    }

    /// The dig's rule as the hold sees it: `blocks` blocks of `break_s`,
    /// each then the pause, the button held from frame 0.
    fn rule(frames: usize, break_s: f32) -> Vec<DigBeat> {
        let mut local = 0.0;
        (0..frames)
            .map(|_| {
                let beat = DigBeat {
                    working: true,
                    break_s,
                    between_s: BETWEEN,
                    local_s: local,
                };
                local += DT;
                if local >= break_s + BETWEEN {
                    local = 0.0;
                }
                beat
            })
            .collect()
    }

    fn change(a: HeldPose, b: HeldPose) -> (f32, f32) {
        (
            (a.pitch - b.pitch).abs() + (a.yaw - b.yaw).abs(),
            a.shift.distance(b.shift),
        )
    }

    /// Run the clock over the frames, the largest frame-to-frame change on
    /// the first frames after `watch`, and the impacts.
    fn run(
        clock: &mut StrokeClock,
        g: Gear,
        frames: &[DigBeat],
        last: &mut HeldPose,
    ) -> (f32, f32, Vec<bool>) {
        let (s, v) = (StrokeSettings::default(), ShovelSettings::default());
        let (mut turn, mut moved, mut impacts) = (0.0f32, 0.0f32, Vec::new());
        for (i, dig) in frames.iter().enumerate() {
            let f = clock.step(&s, &v, g, *dig, DT);
            if i < 3 {
                let (a, m) = change(*last, f.pose);
                turn = turn.max(a);
                moved = moved.max(m);
            }
            impacts.extend(f.impacts);
            *last = f.pose;
        }
        (turn, moved, impacts)
    }

    /// No pop on a state change: taking up the dig, letting go at the hit,
    /// taking up again mid-release, a block started again mid-way (aimed
    /// elsewhere), a tool changed mid-stroke, and the move from one block
    /// to the next: each moves the tool a few hundredths in its first
    /// frames, never a jump. Three stone blocks of 1.2 s land 3n impacts,
    /// a break on each block's last.
    #[test]
    fn no_pop_on_a_state_change() {
        let pick = gear(StrokeTool::Pickaxe);
        let mut clock = StrokeClock::default();
        let mut last = HeldPose::default();
        let beat = Beat::new(&StrokeSettings::default(), 1.2, BETWEEN);
        let frames = rule((3.0 * beat.cycle_s / DT).round() as usize - 1, 1.2);
        let (turn, moved, impacts) = run(&mut clock, pick, &frames, &mut last);
        assert!(turn < 0.05 && moved < 0.01, "taking up: {turn} {moved}");
        // The steady stroke's own fastest frame: a pop is a frame faster
        // than a third of that.
        let (mut fast_turn, mut fast_move) = (0.0f32, 0.0f32);
        {
            let (s, v) = (StrokeSettings::default(), ShovelSettings::default());
            let mut c = StrokeClock::default();
            let mut was = HeldPose::default();
            for dig in &frames {
                let f = c.step(&s, &v, pick, *dig, DT);
                let (a, m) = change(was, f.pose);
                fast_turn = fast_turn.max(a);
                fast_move = fast_move.max(m);
                was = f.pose;
            }
        }
        let calm = |turn: f32, moved: f32| turn < fast_turn / 3.0 && moved < fast_move / 3.0;
        assert_eq!(impacts.len() as u32, 3 * beat.n, "{impacts:?}");
        assert_eq!(impacts.iter().filter(|l| **l).count(), 3);
        // Let go just after an impact.
        let stop = [DigBeat::default(); 30];
        let at_hit = rule(((beat.break_s - 0.0) / DT) as usize + 2, 1.2);
        let mut clock = StrokeClock::default();
        let mut last = HeldPose::default();
        run(&mut clock, pick, &at_hit, &mut last);
        let (turn, moved, _) = run(&mut clock, pick, &stop[..5], &mut last);
        assert!(
            calm(turn, moved),
            "letting go: {turn} {moved} of {fast_turn} {fast_move}"
        );
        // Take up again mid-release.
        let (turn, moved, _) = run(&mut clock, pick, &rule(20, 1.2), &mut last);
        assert!(
            calm(turn, moved),
            "again mid-release: {turn} {moved} of {fast_turn} {fast_move}"
        );
        // Mid-block, the block starts again from nought.
        let mut half = rule(40, 1.2);
        half.extend(rule(20, 1.2));
        let (_, _, _) = run(&mut clock, pick, &half[..40], &mut last);
        let (turn, moved, _) = run(&mut clock, pick, &half[40..], &mut last);
        assert!(
            calm(turn, moved),
            "started again: {turn} {moved} of {fast_turn} {fast_move}"
        );
        // A tool changed mid-stroke.
        let shovel = gear(StrokeTool::Shovel);
        let (turn, moved, _) = run(&mut clock, shovel, &rule(20, 0.5), &mut last);
        assert!(
            calm(turn, moved),
            "another tool: {turn} {moved} of {fast_turn} {fast_move}"
        );
        // And after letting go the tool comes to rest.
        run(&mut clock, shovel, &stop, &mut last);
        assert!(
            last.pitch.abs() < 1e-5 && last.shift.length() < 1e-5,
            "{last:?}"
        );
    }
}
