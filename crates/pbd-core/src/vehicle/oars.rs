//! Rowing the raft with two oars on a rowing frame (`river-craft` design 9,
//! M34 River Craft's Rowing tab, `RC.Oarsman` in
//! `docs/mockups/river-craft/rowing.js`).
//!
//! One stroke clock runs for both oars while a row key is held, through five
//! stages: reach, catch, drive, release, recovery. [`oar_pose`] is the one
//! schedule: the force reads the blade it gives, and the drawing (the oars,
//! the rower's arms, the first-person hands) reads the same pose from the
//! state. The push is the blade's drag through the water, never a push off
//! the bed: a blade over a bed shallower than its stroke rests on it and
//! pulls only with its share still in the water (design 9.2).
//!
//! Frames: everything here is the craft's reference frame, metres, +x right,
//! +y up, -z the bow. An oar's side is -1 (left, index 0) or +1 (right,
//! index 1). The sweep `theta` is + toward the bow.

use super::foil::FoilForce;
use super::{Context, Input, SEA_DENSITY};
use glam::DVec3;
use serde::{Deserialize, Serialize};

const D: f64 = std::f64::consts::PI / 180.0;

/// The oars on their pins (design 9.7, `oars:`), the game's frame: `pin` is
/// the right oar's (mirrored for the left).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OarSpec {
    /// The right pin, m: out (+x), over the reference frame's waterline,
    /// along (-z the bow).
    pub pin: [f32; 3],
    /// Handle end to blade tip, m.
    pub length_m: f32,
    /// Handle end to the pin, m (the hand grips 0.12 m in from the end).
    pub inboard_m: f32,
    pub blade_m2: f32,
    pub blade_cd: f32,
    /// Sweep at the catch and the release, degrees, + toward the bow.
    pub catch_deg: f32,
    pub release_deg: f32,
    /// The reach's coil before the catch and the release's carry past it.
    pub coil_deg: f32,
    pub follow_deg: f32,
    /// The blade's centre under the water through the drive and over it
    /// through the recovery, m.
    pub blade_depth_m: f32,
    pub blade_clear_m: f32,
    pub feather_deg: f32,
    /// A or D with W: how much shorter the inner oar's sweep is.
    pub turn_share: f32,
    /// A blade over a shallow bed rests this far above it, m.
    pub bed_clear_m: f32,
}

/// The stroke's stages, s, and one knob for the drawing's exaggeration
/// (design 9.7, `stroke:`).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StrokeSpec {
    pub reach_s: f32,
    pub catch_s: f32,
    pub drive_s: f32,
    pub release_s: f32,
    pub recovery_s: f32,
    /// Scales the coil, the follow-through, the lean and the head's follow.
    pub exaggerate: f32,
}

/// The seated rower (design 9.7, `rower:`), the game's frame.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RowerSpec {
    /// The hip on the bench, m.
    pub seat: [f32; 3],
    /// Hip to the shoulders' line, and each shoulder out from the middle.
    pub shoulder_m: f32,
    pub shoulder_half_m: f32,
    pub upper_arm_m: f32,
    pub forearm_m: f32,
    /// The body's lean at the catch and at the release, degrees, + toward
    /// the bow.
    pub lean_catch_deg: f32,
    pub lean_release_deg: f32,
    /// Share of the drive the body swings ahead of the arms.
    pub body_lead: f32,
    /// Share of the shoulders' move the first-person eye follows.
    pub head_follow: f32,
    /// Metres a hand-model unit: the first-person hand on the handle.
    pub hand_scale_m: f32,
}

/// The raft's oars (design 9): present on a kind rowed with oars, absent on
/// one paddled on the Loon's stroke.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RowingSpec {
    pub oars: OarSpec,
    pub stroke: StrokeSpec,
    pub rower: RowerSpec,
}

impl Default for RowingSpec {
    /// M34's version 4 numbers (design 9.7, 9.10), in the game's frame.
    fn default() -> Self {
        Self {
            oars: OarSpec {
                pin: [1.0, 0.62, 0.0],
                length_m: 3.0,
                inboard_m: 0.95,
                blade_m2: 0.14,
                blade_cd: 1.3,
                catch_deg: 22.0,
                release_deg: -16.0,
                coil_deg: 3.0,
                follow_deg: 3.0,
                blade_depth_m: 0.12,
                blade_clear_m: 0.22,
                feather_deg: 80.0,
                turn_share: 0.35,
                bed_clear_m: 0.1,
            },
            stroke: StrokeSpec {
                reach_s: 0.16,
                catch_s: 0.12,
                drive_s: 0.62,
                release_s: 0.14,
                recovery_s: 0.66,
                exaggerate: 1.0,
            },
            rower: RowerSpec {
                seat: [0.0, 0.62, 0.52],
                shoulder_m: 0.5,
                shoulder_half_m: 0.27,
                upper_arm_m: 0.3,
                forearm_m: 0.37,
                lean_catch_deg: -6.0,
                lean_release_deg: 22.0,
                body_lead: 0.25,
                head_follow: 0.6,
                hand_scale_m: 0.03,
            },
        }
    }
}

impl RowingSpec {
    /// Every time and length positive, the release behind the catch, and the
    /// blade's droop able to reach the water (design 9.7).
    pub fn validate(&self, name: &str) -> Result<(), String> {
        let (o, s, r) = (&self.oars, &self.stroke, &self.rower);
        let positive = [
            o.length_m,
            o.inboard_m,
            o.blade_m2,
            o.blade_cd,
            s.reach_s,
            s.catch_s,
            s.drive_s,
            s.release_s,
            s.recovery_s,
            r.shoulder_m,
            r.upper_arm_m,
            r.forearm_m,
            r.hand_scale_m,
        ];
        if !positive.iter().all(|v| v.is_finite() && *v > 0.0) {
            return Err(format!("{name}.rowing: times and lengths are positive"));
        }
        let rest = [
            o.coil_deg,
            o.follow_deg,
            o.blade_depth_m,
            o.blade_clear_m,
            o.feather_deg,
            o.turn_share,
            o.bed_clear_m,
            s.exaggerate,
            r.body_lead,
            r.head_follow,
        ];
        if !rest.iter().all(|v| v.is_finite() && *v >= 0.0) || o.turn_share > 1.0 {
            return Err(format!("{name}.rowing: a stroke number is out of range"));
        }
        if o.release_deg >= o.catch_deg {
            return Err(format!("{name}.rowing: release_deg < catch_deg"));
        }
        if o.inboard_m + 0.3 >= o.length_m {
            return Err(format!("{name}.rowing: the oar is longer than inboard"));
        }
        let reach = (o.length_m - o.inboard_m - 0.3) * 0.95;
        if o.pin[1] + o.blade_depth_m > reach {
            return Err(format!("{name}.rowing: the blade cannot reach the water"));
        }
        Ok(())
    }

    /// One stroke, s.
    pub fn period(&self) -> f64 {
        let s = &self.stroke;
        (s.reach_s + s.catch_s + s.drive_s + s.release_s + s.recovery_s) as f64
    }

    /// The body's lean for a body share (0 the catch's, 1 the release's),
    /// rad, + toward the bow.
    pub fn lean(&self, body: f64) -> f64 {
        let r = &self.rower;
        self.stroke.exaggerate as f64
            * lerp(r.lean_catch_deg as f64, r.lean_release_deg as f64, body)
            * D
    }

    /// A shoulder for a lean, the game's frame.
    pub fn shoulder(&self, lean: f64, side: f64) -> DVec3 {
        let r = &self.rower;
        let seat = v3(r.seat);
        DVec3::new(
            side * r.shoulder_half_m as f64,
            seat.y + lean.cos() * r.shoulder_m as f64,
            seat.z - lean.sin() * r.shoulder_m as f64,
        )
    }

    /// The rower's eye for a lean: the head rides `head_follow` of the
    /// shoulders' swing, a little ahead of the hip (design 9.6).
    pub fn eye(&self, lean: f64) -> DVec3 {
        let r = &self.rower;
        let seat = v3(r.seat);
        let base = DVec3::new(0.0, seat.y + 0.74, seat.z);
        let full = DVec3::new(0.0, seat.y + lean.cos() * 0.74, seat.z - lean.sin() * 0.74);
        base.lerp(full, (r.head_follow * self.stroke.exaggerate) as f64) - DVec3::Z * 0.1
    }

    /// The blade's centre from the pin, m.
    pub fn blade_reach(&self) -> f64 {
        (self.oars.length_m - self.oars.inboard_m - 0.3) as f64
    }
}

/// The stroke's five stages (design 9.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Reach,
    Catch,
    Drive,
    Release,
    Recovery,
}

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Reach => "Reach",
            Stage::Catch => "Catch",
            Stage::Drive => "Drive",
            Stage::Release => "Release",
            Stage::Recovery => "Recovery",
        }
    }
}

/// An oar's pose at a time into a stroke.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OarPose {
    /// The sweep, rad, + toward the bow.
    pub theta: f64,
    /// 0 the blade clear .. 1 the blade buried.
    pub dip: f64,
    /// 0 square .. 1 flat.
    pub feather: f64,
    /// 0 the catch's lean .. 1 the release's.
    pub body: f64,
    pub stage: Option<Stage>,
}

/// Where an oar is, the game's frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OarGeom {
    pub pin: DVec3,
    /// Outboard along the loom.
    pub dir: DVec3,
    pub handle: DVec3,
    /// Where the hand closes on the handle.
    pub grip: DVec3,
    /// The blade's centre.
    pub blade: DVec3,
    /// The blade's feather about the loom, rad.
    pub feather: f64,
    /// The blade rests on a shallow bed.
    pub on_bed: bool,
}

fn c01(u: f64) -> f64 {
    u.clamp(0.0, 1.0)
}
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}
fn smooth(u: f64) -> f64 {
    let u = c01(u);
    u * u * (3.0 - 2.0 * u)
}
fn half_cos(u: f64) -> f64 {
    (1.0 - (std::f64::consts::PI * c01(u)).cos()) / 2.0
}
fn ease_out(u: f64) -> f64 {
    1.0 - (1.0 - c01(u)).powi(3)
}
fn ease_in(u: f64) -> f64 {
    c01(u).powi(3)
}
fn v3(a: [f32; 3]) -> DVec3 {
    DVec3::from(a.map(f64::from))
}

/// The schedule (design 9.3): the oar's pose `t` seconds into a stroke,
/// ahead (`dir` 1) or backing (-1), its sweep scaled about its middle by
/// `range` (an inner oar on a turn). The catch's lift and the release's press
/// each overlap the sweep a little, so the hands go round one rounded loop.
pub fn oar_pose(spec: &RowingSpec, t: f64, dir: f64, range: f64) -> OarPose {
    let (s, a) = (&spec.stroke, &spec.oars);
    let e = s.exaggerate as f64;
    let (tc, tr) = (a.catch_deg as f64 * D, a.release_deg as f64 * D);
    let mid = (tc + tr) / 2.0;
    let c = mid + (tc - mid) * range;
    let rl = mid + (tr - mid) * range;
    let coil = a.coil_deg as f64 * D * e;
    let foll = a.follow_deg as f64 * D * e;
    let lead = (spec.rower.body_lead as f64).clamp(0.0, 0.9);
    let e1 = s.reach_s as f64;
    let e2 = e1 + s.catch_s as f64;
    let e3 = e2 + s.drive_s as f64;
    let e4 = e3 + s.release_s as f64;
    let mut p = if t < e1 {
        let k = t / s.reach_s as f64;
        OarPose {
            theta: c + coil * ease_out(k),
            dip: 0.0,
            feather: 1.0 - smooth(k),
            body: -0.08 * e * ease_out(k),
            stage: Some(Stage::Reach),
        }
    } else if t < e2 {
        let k = (t - e1) / s.catch_s as f64;
        OarPose {
            theta: c + coil * (1.0 - smooth(k)) - (c - rl) * 0.04 * k * k,
            dip: ease_in(k) * 0.75 + 0.25 * k,
            feather: 0.0,
            body: -0.08 * e * (1.0 - k),
            stage: Some(Stage::Catch),
        }
    } else if t < e3 {
        let k = (t - e2) / s.drive_s as f64;
        OarPose {
            theta: lerp(c - (c - rl) * 0.04, rl, half_cos(k)),
            dip: 1.0,
            feather: 0.0,
            body: half_cos(k / (1.0 - lead)),
            stage: Some(Stage::Drive),
        }
    } else if t < e4 {
        let k = (t - e3) / s.release_s as f64;
        OarPose {
            theta: rl - foll * ease_out(k),
            dip: (1.0 - k) * (1.0 - k),
            feather: ease_out(k),
            body: 1.0,
            stage: Some(Stage::Release),
        }
    } else {
        let k = ((t - e4) / s.recovery_s as f64).min(1.0);
        OarPose {
            theta: lerp(rl - foll, c, half_cos(k)),
            dip: 0.0,
            feather: 1.0,
            body: 1.0 - half_cos(k),
            stage: Some(Stage::Recovery),
        }
    };
    if dir < 0.0 {
        p.theta = c + rl - p.theta;
        p.body = 1.0 - p.body;
    }
    p
}

/// An oar at rest: blades forward at the catch's sweep, clear and flat.
pub fn rest_pose(spec: &RowingSpec) -> OarPose {
    OarPose {
        theta: spec.oars.catch_deg as f64 * D,
        dip: 0.0,
        feather: 1.0,
        body: 0.0,
        stage: None,
    }
}

/// The oar from a pose (`oarGeom`): `surface` is the water's height at the
/// pin in the craft's frame (0 on still water at the reference waterline),
/// `floor` the lowest the blade may go there (the bed plus `bed_clear_m`).
pub fn oar_geom(
    spec: &RowingSpec,
    side: f64,
    pose: &OarPose,
    surface: f64,
    floor: Option<f64>,
) -> OarGeom {
    let a = &spec.oars;
    let rb = spec.blade_reach();
    let mut yb = surface + lerp(a.blade_clear_m as f64, -(a.blade_depth_m as f64), pose.dip);
    let on_bed = floor.is_some_and(|f| yb < f);
    if let Some(f) = floor.filter(|_| on_bed) {
        yb = f;
    }
    let pin = DVec3::new(side * a.pin[0] as f64, a.pin[1] as f64, a.pin[2] as f64);
    let phi = ((pin.y - yb) / rb).clamp(-0.95, 0.95).asin();
    let (st, ct) = pose.theta.sin_cos();
    let (sp, cp) = phi.sin_cos();
    let dir = DVec3::new(side * ct * cp, -sp, -st * cp);
    OarGeom {
        pin,
        dir,
        handle: pin - dir * a.inboard_m as f64,
        grip: pin - dir * (a.inboard_m as f64 - 0.12),
        blade: pin + dir * rb,
        feather: pose.feather * a.feather_deg as f64 * D,
        on_bed,
    }
}

/// The arm's elbow (`elbowOf`): two bones from the shoulder `s` to the hand
/// `h`, bent toward `pole`.
pub fn elbow(s: DVec3, h: DVec3, upper: f64, fore: f64, pole: DVec3) -> DVec3 {
    let d = h - s;
    let l = d.length().clamp(1e-4, upper + fore - 1e-4);
    let d = d.normalize_or(DVec3::NEG_Y);
    let x = (upper * upper - fore * fore + l * l) / (2.0 * l);
    let h = (upper * upper - x * x).max(0.0).sqrt();
    let pv = pole - d * pole.dot(d);
    let pv = if pv.length_squared() < 1e-8 {
        DVec3::NEG_Y
    } else {
        pv.normalize()
    };
    s + d * x + pv * h
}

/// The way an arm bends: out and down from its side, a little aft.
pub fn elbow_pole(side: f64) -> DVec3 {
    DVec3::new(side * 0.9, -1.0, 0.2)
}

/// Which oars the keys ask for (design 9.4): W both, S back both, A the
/// right oar alone (turn left), D the left, Q and E the left or right on its
/// own; A or D with W row both, the turn's inner oar shorter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Want {
    /// Left, right.
    pub rows: [bool; 2],
    pub dir: f64,
    pub range: [f64; 2],
}

impl Want {
    pub fn of(input: &Input, spec: &RowingSpec) -> Self {
        let w = input.forward > 0.0;
        let s = input.forward < 0.0;
        let a = input.steer > 0.0;
        let d = input.steer < 0.0;
        let q = input.rudder > 0.0;
        let e = input.rudder < 0.0;
        let turn = 1.0 - spec.oars.turn_share as f64;
        Self {
            rows: [w || s || d || q, w || s || a || e],
            dir: if s && !w { -1.0 } else { 1.0 },
            range: [
                if w && a { turn } else { 1.0 },
                if w && d { turn } else { 1.0 },
            ],
        }
    }

    fn any(&self) -> bool {
        self.rows[0] || self.rows[1]
    }
}

/// One oar's part of the stroke.
#[derive(Clone, Debug, PartialEq)]
pub struct OarState {
    pub rowing: bool,
    pub range: f64,
    /// Where it rests, and where it was last: a stroke starts from here.
    rest: OarPose,
    pub pose: OarPose,
    /// Where it is now, the craft's frame.
    pub geom: OarGeom,
    prev_blade: Option<DVec3>,
    /// The bed under the blade last step, the craft's frame, plus
    /// `bed_clear_m`.
    floor: Option<f64>,
    /// Catches and releases made, for the drawing's splash and swirls.
    pub catches: u32,
    pub releases: u32,
    /// The blade's force last step and where (planet frame).
    pub force: FoilForce,
}

/// The rowing frame's state: one clock for both oars.
#[derive(Clone, Debug, PartialEq)]
pub struct RowingState {
    /// Seconds into the stroke.
    pub t: f64,
    pub running: bool,
    pub dir: f64,
    /// Left, right.
    pub oars: [OarState; 2],
    /// The body's share, 0 the catch's lean .. 1 the release's.
    pub body: f64,
    body_v: f64,
    pub stage: Option<Stage>,
    /// An oar is resting on the bed through its drive.
    pub on_bed: bool,
    /// World times strokes began, the last few seconds.
    pub strokes: Vec<f64>,
}

impl Default for RowingState {
    fn default() -> Self {
        Self::new(&RowingSpec::default())
    }
}

impl RowingState {
    pub fn new(spec: &RowingSpec) -> Self {
        let rest = rest_pose(spec);
        let oar = |side: f64| OarState {
            rowing: false,
            range: 1.0,
            rest,
            pose: rest,
            geom: oar_geom(spec, side, &rest, 0.0, None),
            prev_blade: None,
            floor: None,
            catches: 0,
            releases: 0,
            force: FoilForce::default(),
        };
        Self {
            t: 0.0,
            running: false,
            dir: 1.0,
            oars: [oar(-1.0), oar(1.0)],
            body: 0.0,
            body_v: 0.0,
            stage: None,
            on_bed: false,
            strokes: Vec::new(),
        }
    }

    /// The body's lean now, rad.
    pub fn lean(&self, spec: &RowingSpec) -> f64 {
        spec.lean(self.body)
    }

    /// Advance the clock `h` seconds for what the keys ask: the stroke's
    /// schedule, each oar's pose, and the events. Whether an oar joins is
    /// decided only at the start of a stroke, so the two never fall out of
    /// step; an oar whose key comes up finishes its stroke and rests where
    /// it ended.
    pub fn advance(&mut self, spec: &RowingSpec, want: &Want, h: f64, now: f64) {
        let period = spec.period();
        let s = &spec.stroke;
        if !self.running && want.any() {
            self.running = true;
            self.t = 0.0;
            self.dir = want.dir;
            for i in 0..2 {
                self.oars[i].rowing = want.rows[i];
                self.oars[i].range = want.range[i];
            }
            self.strokes.push(now);
        }
        if self.running {
            let t0 = self.t;
            self.t += h;
            let e_catch = (s.reach_s + s.catch_s) as f64;
            let e_rel = e_catch + s.drive_s as f64 + s.release_s as f64 * 0.45;
            for oar in &mut self.oars {
                if !oar.rowing {
                    continue;
                }
                if t0 < e_catch && self.t >= e_catch && !oar.geom.on_bed {
                    oar.catches += 1;
                }
                if t0 < e_rel && self.t >= e_rel {
                    oar.releases += 1;
                }
            }
            if self.t >= period {
                self.t -= period;
                if want.any() {
                    self.dir = want.dir;
                    self.strokes.push(now);
                }
                for i in 0..2 {
                    let oar = &mut self.oars[i];
                    if oar.rowing && !want.rows[i] {
                        oar.rest = oar.pose;
                    }
                    oar.rowing = want.rows[i];
                    oar.range = want.range[i];
                }
                if !want.any() {
                    self.running = false;
                    self.t = 0.0;
                }
            }
        }
        self.strokes.retain(|t| now - t < 6.0);
        let target = rest_pose(spec);
        let mut leader = None;
        for oar in &mut self.oars {
            if oar.rowing && self.running {
                let mut pose = oar_pose(spec, self.t, self.dir, oar.range);
                if self.t < s.reach_s as f64 {
                    // Out of wherever it rested into the reach: no jump.
                    let w = smooth(self.t / s.reach_s as f64);
                    pose.theta = lerp(oar.rest.theta, pose.theta, w);
                    pose.dip = lerp(oar.rest.dip, pose.dip, w);
                    pose.feather = lerp(oar.rest.feather, pose.feather, w);
                }
                oar.rest = pose;
                oar.pose = pose;
                leader.get_or_insert(pose);
            } else {
                let k = 1.0 - (-h * 5.0).exp();
                let r = &mut oar.rest;
                r.theta += (target.theta - r.theta) * k;
                r.dip += (target.dip - r.dip) * k;
                r.feather += (target.feather - r.feather) * k;
                r.stage = None;
                r.body = 0.0;
                oar.pose = *r;
            }
        }
        match leader {
            Some(pose) => {
                self.body = pose.body;
                self.body_v = 0.0;
                self.stage = pose.stage;
            }
            None => {
                self.body_v += (-self.body * 60.0 - self.body_v * 14.0) * h;
                self.body += self.body_v * h;
                self.stage = None;
            }
        }
    }
}

/// The oars' forces for one substep: each blade's drag through the water,
/// at the blade, horizontal (design 9.4). Returns the larger blade force.
pub(super) fn row(
    body: &mut super::body::RigidBody,
    state: &mut RowingState,
    spec: &RowingSpec,
    input: Option<&Input>,
    com: DVec3,
    cx: &Context,
) -> FoilForce {
    let want = input.map_or_else(Want::default, |i| Want::of(i, spec));
    state.advance(spec, &want, cx.dt, cx.seconds);
    let up = cx.up;
    let a = &spec.oars;
    let mut strongest = FoilForce::default();
    state.on_bed = false;
    for (i, oar) in state.oars.iter_mut().enumerate() {
        let side = if i == 0 { -1.0 } else { 1.0 };
        // The water at the pin, in the craft's frame.
        let pin_local = DVec3::new(side * a.pin[0] as f64, a.pin[1] as f64, a.pin[2] as f64);
        let pin_world = body.point(pin_local - com);
        let surface = pin_local.y - cx.above_sea(pin_world);
        let geom = oar_geom(spec, side, &oar.pose, surface, oar.floor);
        let blade_world = body.point(geom.blade - com);
        // The bed under the blade for the next step: the blade comes to rest
        // on it and pushes nothing off it (design 9.2).
        let r = blade_world.length();
        let ground = (cx.env.ground)(blade_world);
        let bed = geom.blade.y - (r - ground);
        oar.floor = Some(bed + a.bed_clear_m as f64);
        if geom.on_bed && oar.pose.dip > 0.3 {
            state.on_bed = true;
        }
        let prev = oar.prev_blade.replace(geom.blade);
        oar.geom = geom;
        oar.force = FoilForce {
            at: blade_world,
            ..Default::default()
        };
        let Some(prev) = prev else {
            continue;
        };
        let depth = -cx.above_sea(blade_world);
        let water_share = c01((depth + 0.12) / 0.24);
        let bed_share = c01((geom.blade.y - bed) / 0.2);
        let share = water_share * bed_share;
        if share <= 0.0 {
            continue;
        }
        let stroke = body.axis((geom.blade - prev) / cx.dt);
        let (_, water) = cx.water(blade_world, depth.max(0.0));
        let rel = body.velocity_at(blade_world) + stroke - water;
        let rel = rel - up * rel.dot(up);
        let force = rel
            * (-0.5 * SEA_DENSITY * a.blade_cd as f64 * a.blade_m2 as f64 * share * rel.length());
        body.push(force, blade_world);
        oar.force.force = force;
        if force.length() > strongest.force.length() {
            strongest = oar.force;
        }
    }
    strongest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> RowingSpec {
        RowingSpec::default()
    }

    /// The schedule: which stage at which time, ahead and backing.
    #[test]
    fn the_stroke_runs_its_five_stages_in_order() {
        let s = spec();
        let st = s.stroke;
        let at = |t: f32| oar_pose(&s, t as f64, 1.0, 1.0).stage;
        assert_eq!(at(st.reach_s * 0.5), Some(Stage::Reach));
        assert_eq!(at(st.reach_s + st.catch_s * 0.5), Some(Stage::Catch));
        let drive = st.reach_s + st.catch_s;
        assert_eq!(at(drive + st.drive_s * 0.5), Some(Stage::Drive));
        let release = drive + st.drive_s;
        assert_eq!(at(release + st.release_s * 0.5), Some(Stage::Release));
        assert_eq!(at(release + st.release_s + 0.1), Some(Stage::Recovery));
        assert!((s.period() - 1.70).abs() < 1e-6, "35 strokes a minute");
        // The blade is buried through the drive and clear in the recovery;
        // the drive sweeps it aft, backing sweeps it forward.
        let mid = (drive + st.drive_s * 0.5) as f64;
        let p = oar_pose(&s, mid, 1.0, 1.0);
        assert_eq!(p.dip, 1.0);
        assert_eq!(p.feather, 0.0);
        let start = oar_pose(&s, drive as f64 + 0.01, 1.0, 1.0);
        let end = oar_pose(&s, release as f64 - 0.01, 1.0, 1.0);
        assert!(end.theta < start.theta, "the drive sweeps aft");
        let back_start = oar_pose(&s, drive as f64 + 0.01, -1.0, 1.0);
        let back_end = oar_pose(&s, release as f64 - 0.01, -1.0, 1.0);
        assert!(back_end.theta > back_start.theta, "backing sweeps forward");
        let rec = oar_pose(&s, s.period() - 0.2, 1.0, 1.0);
        assert_eq!(rec.dip, 0.0);
    }

    /// The pose never jumps: sampled finely through two strokes the sweep
    /// moves less than a step's worth at 240 Hz.
    #[test]
    fn the_pose_is_continuous_through_the_stroke() {
        let s = spec();
        let n = 2000;
        let mut last = oar_pose(&s, 0.0, 1.0, 1.0);
        for i in 1..n {
            let t = s.period() * i as f64 / n as f64;
            let p = oar_pose(&s, t, 1.0, 1.0);
            assert!((p.theta - last.theta).abs() < 0.01, "theta jumps at {t}");
            assert!((p.dip - last.dip).abs() < 0.05, "dip jumps at {t}");
            assert!(
                (p.feather - last.feather).abs() < 0.05,
                "feather jumps at {t}"
            );
            last = p;
        }
    }

    /// The catch event lands on the catch's end, once a stroke for each
    /// rowing oar.
    #[test]
    fn the_catch_lands_on_the_catchs_end() {
        let s = spec();
        let mut st = RowingState::new(&s);
        let want = Want {
            rows: [true, true],
            dir: 1.0,
            range: [1.0, 1.0],
        };
        let h = 1.0 / 240.0;
        let e_catch = (s.stroke.reach_s + s.stroke.catch_s) as f64;
        let mut t = 0.0;
        let mut caught_at = None;
        while t < s.period() * 2.0 {
            st.advance(&s, &want, h, t);
            t += h;
            if caught_at.is_none() && st.oars[0].catches == 1 {
                caught_at = Some(t);
            }
        }
        let at = caught_at.expect("a catch");
        assert!((at - e_catch).abs() <= h * 1.5, "caught at {at}, {e_catch}");
        assert_eq!(st.oars[0].catches, 2);
        assert_eq!(st.oars[1].catches, 2);
        assert_eq!(st.oars[0].releases, 2);
    }

    /// An oar asked for mid-stroke joins only at the next reach, and one let
    /// go finishes its stroke and rests without a jump.
    #[test]
    fn an_oar_joins_at_the_next_reach_and_rests_without_a_jump() {
        let s = spec();
        let mut st = RowingState::new(&s);
        let h = 1.0 / 240.0;
        let right = Want {
            rows: [false, true],
            dir: 1.0,
            range: [1.0, 1.0],
        };
        let both = Want {
            rows: [true, true],
            ..right
        };
        let mut t = 0.0;
        while t < 0.5 {
            st.advance(&s, &right, h, t);
            t += h;
        }
        st.advance(&s, &both, h, t);
        assert!(!st.oars[0].rowing, "the left waits for the next reach");
        while st.t > 0.1 {
            st.advance(&s, &both, h, t);
            t += h;
        }
        assert!(st.oars[0].rowing && st.oars[1].rowing);
        // Let go mid-drive: nothing snaps.
        while st.stage != Some(Stage::Drive) {
            st.advance(&s, &both, h, t);
            t += h;
        }
        let none = Want::default();
        let mut last = st.oars[1].pose;
        for _ in 0..(3.0 / h) as usize {
            st.advance(&s, &none, h, t);
            t += h;
            let p = st.oars[1].pose;
            assert!((p.theta - last.theta).abs() < 0.02, "the sweep snapped");
            assert!((p.dip - last.dip).abs() < 0.1, "the dip snapped");
            last = p;
        }
        assert!(!st.running);
        let rest = rest_pose(&s);
        assert!((last.theta - rest.theta).abs() < 0.01 && last.dip < 0.01);
    }

    /// The hands reach the handles: from each shoulder to its grip is never
    /// longer than the arm, through the whole stroke ahead and back, at full
    /// and turning range (design 9.10).
    #[test]
    fn the_arms_reach_the_handles_through_the_stroke() {
        let s = spec();
        let arm = (s.rower.upper_arm_m + s.rower.forearm_m) as f64;
        for dir in [1.0, -1.0] {
            for range in [1.0, 1.0 - s.oars.turn_share as f64] {
                for i in 0..200 {
                    let t = s.period() * i as f64 / 200.0;
                    let p = oar_pose(&s, t, dir, range);
                    for side in [-1.0, 1.0] {
                        let g = oar_geom(&s, side, &p, 0.0, None);
                        let sh = s.shoulder(s.lean(p.body), side);
                        let d = sh.distance(g.grip);
                        assert!(d <= arm + 0.02, "{d} m to the grip, arm {arm}");
                        // The blade goes under at the drive and clears in
                        // the recovery.
                        if p.stage == Some(Stage::Drive) {
                            assert!(g.blade.y < 0.0);
                        }
                        if p.stage == Some(Stage::Recovery) {
                            assert!(g.blade.y > 0.0);
                        }
                    }
                }
            }
        }
    }

    /// The left oar is the right one mirrored, and a blade on a shallow bed
    /// rests on it.
    #[test]
    fn the_oars_mirror_and_a_blade_rests_on_the_bed() {
        let s = spec();
        let p = oar_pose(&s, 0.5, 1.0, 1.0);
        let l = oar_geom(&s, -1.0, &p, 0.0, None);
        let r = oar_geom(&s, 1.0, &p, 0.0, None);
        assert!((l.blade.x + r.blade.x).abs() < 1e-9 && (l.blade.z - r.blade.z).abs() < 1e-9);
        assert!(r.blade.x > 1.5, "the blade is outboard");
        let shallow = oar_geom(&s, 1.0, &p, 0.0, Some(-0.02));
        assert!(shallow.on_bed && (shallow.blade.y + 0.02).abs() < 1e-6);
    }

    #[test]
    fn the_keys_pick_the_oars() {
        let s = spec();
        let w = |forward, steer, rudder| {
            Want::of(
                &Input {
                    forward,
                    steer,
                    rudder,
                    bail: false,
                },
                &s,
            )
        };
        assert_eq!(w(1.0, 0.0, 0.0).rows, [true, true]);
        assert_eq!(w(-1.0, 0.0, 0.0).dir, -1.0);
        // A turns left: the right oar alone.
        assert_eq!(w(0.0, 1.0, 0.0).rows, [false, true]);
        assert_eq!(w(0.0, -1.0, 0.0).rows, [true, false]);
        // Q the left oar, E the right.
        assert_eq!(w(0.0, 0.0, 1.0).rows, [true, false]);
        assert_eq!(w(0.0, 0.0, -1.0).rows, [false, true]);
        let turning = w(1.0, 1.0, 0.0);
        assert_eq!(turning.rows, [true, true]);
        assert!(turning.range[0] < 1.0 && turning.range[1] == 1.0);
    }
}
