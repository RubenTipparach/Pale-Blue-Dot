//! How a fish is drawn moving (`fish-behaviour`, M37): its heading turned
//! by yaw and pitch at a limited rate, its bank into a turn, its body wave
//! set by its speed, and the choreography of the fish the line is working
//! (the pecks, the take, the bolt and the fight), all read off the school's
//! steps and the line's phase, timer and run.
//!
//! Draw side only: nothing in the fishing rules or the school's step reads
//! it, except that a bolting fish is handed back to its school where it was
//! drawn ([`LineFish::update`] returns it). Everything is local metres
//! (`world-frames`): relative to the [`SimOrigin`], with "up" taken through
//! it; a pose holding a position moves with a rebase ([`Pose::rebase`],
//! [`LineFish::rebase`]).
//!
//! The algorithms and numbers are the mockup's Proposed side
//! (`docs/mockups/fish-behaviour/fish-behaviour.js` `Body`, `Choreo`,
//! `fight`; `body-wave.js` `Swimmer.swim`); its sliders are the
//! [`BehaviourSettings`] in `fauna.ron`'s `behaviour` block.

use super::{Mover, Rng, School, Species};
use crate::fishing::{Line, Phase};
use crate::world_frame::SimOrigin;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f32::consts::{PI, TAU};

const DEG: f32 = PI / 180.0;

/// `fauna.ron`'s `behaviour` block: how a fish is drawn moving.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BehaviourSettings {
    pub pose: PoseSettings,
    pub lure: LureSettings,
    pub fight: FightSettings,
    /// The body wave of a carangiform fish (the perch), an anguilliform one
    /// (the eel and the serpent) and a ray's wings.
    pub fish_wave: WaveProfile,
    pub eel_wave: WaveProfile,
    pub ray_wave: WaveProfile,
}

impl Default for BehaviourSettings {
    fn default() -> Self {
        Self {
            pose: PoseSettings::default(),
            lure: LureSettings::default(),
            fight: FightSettings::default(),
            fish_wave: WaveProfile::FISH,
            eel_wave: WaveProfile::EEL,
            ray_wave: WaveProfile::RAY,
        }
    }
}

/// Every drawn fish's heading and bank (design 4).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct PoseSettings {
    /// The fastest a heading turns, deg/s.
    pub turn_deg_s: f32,
    /// Its easing toward the way the fish swims, s.
    pub heading_ease_s: f32,
    /// The steepest a fish points up or down, deg.
    pub pitch_max_deg: f32,
    /// Bank into a turn at 90 deg/s, deg, and the most, deg.
    pub bank_at_90_deg: f32,
    pub bank_max_deg: f32,
    /// Its easing, s.
    pub roll_ease_s: f32,
}

impl Default for PoseSettings {
    fn default() -> Self {
        Self {
            turn_deg_s: 260.0,
            heading_ease_s: 0.12,
            pitch_max_deg: 35.0,
            bank_at_90_deg: 14.0,
            bank_max_deg: 28.0,
            roll_ease_s: 0.18,
        }
    }
}

/// The fish at the lure (design 2).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct LureSettings {
    /// How far off the lure its mouth hovers, m.
    pub standoff_m: f32,
    /// Its drift while it hovers, m, over this period, s.
    pub hover_drift_m: f32,
    pub drift_period_s: f32,
    /// A peck: the dart in, s, the back off, s, the head shake, deg at Hz,
    /// and the tug on the lure, m.
    pub dart_s: f32,
    pub back_off_s: f32,
    pub shake_deg: f32,
    pub shake_hz: f32,
    pub tug_m: f32,
    /// The chance it turns away and circles between pecks, 0..1, and the
    /// circle's radius, m.
    pub circle_chance: f32,
    pub circle_m: f32,
    /// The take: the dart onto the lure, s, the turn, deg/s, the run's
    /// speed, m/s, and dive, deg, and how far it drags the drawn lure, m.
    pub take_dart_s: f32,
    pub take_turn_deg_s: f32,
    pub take_mps: f32,
    pub take_dive_deg: f32,
    pub take_reach_m: f32,
    /// The bolt from a strike too early or a missed bite: times the
    /// species' top speed, easing out over this long, s.
    pub bolt_speed: f32,
    pub bolt_s: f32,
}

impl Default for LureSettings {
    fn default() -> Self {
        Self {
            standoff_m: 0.15,
            hover_drift_m: 0.03,
            drift_period_s: 2.6,
            dart_s: 0.14,
            back_off_s: 0.35,
            shake_deg: 12.0,
            shake_hz: 7.0,
            tug_m: 0.02,
            circle_chance: 0.6,
            circle_m: 0.2,
            take_dart_s: 0.08,
            take_turn_deg_s: 520.0,
            take_mps: 1.4,
            take_dive_deg: 22.0,
            take_reach_m: 0.3,
            bolt_speed: 1.6,
            bolt_s: 1.2,
        }
    }
}

/// The fish on the line (design 3, 3.1).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FightSettings {
    /// Its turn: the most, deg/s, the easing, s, and the turn rate's own
    /// most change, deg/s^2.
    pub turn_deg_s: f32,
    pub ease_s: f32,
    pub accel_deg_s2: f32,
    /// How quickly the head shake and the flash come in and go, s.
    pub fade_s: f32,
    /// Reeled in: the head shake, deg at Hz, and the roll that flashes the
    /// flank, deg.
    pub shake_deg: f32,
    pub shake_hz: f32,
    pub flash_deg: f32,
    /// Running: how far it dives, deg.
    pub run_dive_deg: f32,
}

impl Default for FightSettings {
    fn default() -> Self {
        Self {
            turn_deg_s: 300.0,
            ease_s: 0.22,
            accel_deg_s2: 1200.0,
            fade_s: 0.35,
            shake_deg: 18.0,
            shake_hz: 3.5,
            flash_deg: 30.0,
            run_dive_deg: 14.0,
        }
    }
}

/// A body wave (design 8.3): its beat from the fish's speed, its amplitude
/// by what it is doing, its shape along the body and its bend into a turn.
/// Lengths are body lengths.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WaveProfile {
    /// The beat: at rest, Hz, more per body length a second, Hz, sculling
    /// at a hover, Hz, and the most, Hz.
    pub beat_rest_hz: f32,
    pub beat_per_len_hz: f32,
    pub hover_hz: f32,
    pub beat_max_hz: f32,
    /// The tail's amplitude cruising, bursting, hovering and gliding, body
    /// lengths, and its easing, s.
    pub amp_cruise: f32,
    pub amp_burst: f32,
    pub amp_hover: f32,
    pub amp_glide: f32,
    pub amp_ease_s: f32,
    /// The envelope: its share at the nose, its exponent, and the waves
    /// along the body.
    pub head: f32,
    pub expo: f32,
    pub waves: f32,
    /// The head's recoil against the tail, times the tail, reaching this
    /// far back from the nose, body lengths.
    pub recoil: f32,
    pub recoil_reach: f32,
    /// The bend into a turn at 90 deg/s and the most, body lengths, and its
    /// easing, s.
    pub bend_at_90: f32,
    pub bend_max: f32,
    pub bend_ease_s: f32,
    /// Slowing faster than this is a glide, body lengths/s^2; slower than
    /// this a hover, body lengths/s.
    pub glide_len_s2: f32,
    pub hover_len_s: f32,
    /// A ray's wings: the span exponent, the waves along a chord, undulate
    /// (0) to flap (1), and the whip's amplitude, body lengths.
    pub wing_expo: f32,
    pub wing_waves: f32,
    pub flap: f32,
    pub whip: f32,
}

impl WaveProfile {
    /// The perch's (carangiform), M37's defaults.
    pub const FISH: Self = Self {
        beat_rest_hz: 0.5,
        beat_per_len_hz: 1.0,
        hover_hz: 1.2,
        beat_max_hz: 8.0,
        amp_cruise: 0.075,
        amp_burst: 0.11,
        amp_hover: 0.025,
        amp_glide: 0.004,
        amp_ease_s: 0.18,
        head: 0.04,
        expo: 2.0,
        waves: 0.95,
        recoil: 0.35,
        recoil_reach: 0.4,
        bend_at_90: 0.06,
        bend_max: 0.14,
        bend_ease_s: 0.15,
        glide_len_s2: 0.35,
        hover_len_s: 0.4,
        wing_expo: 1.6,
        wing_waves: 0.7,
        flap: 0.0,
        whip: 0.05,
    };
    /// The eel's (anguilliform).
    pub const EEL: Self = Self {
        beat_rest_hz: 0.4,
        beat_per_len_hz: 1.0,
        hover_hz: 0.7,
        beat_max_hz: 4.0,
        amp_cruise: 0.07,
        amp_burst: 0.09,
        amp_hover: 0.03,
        amp_glide: 0.01,
        amp_ease_s: 0.25,
        head: 0.3,
        expo: 0.8,
        waves: 1.25,
        recoil: 0.0,
        recoil_reach: 0.3,
        bend_at_90: 0.1,
        bend_max: 0.22,
        bend_ease_s: 0.2,
        glide_len_s2: 0.3,
        hover_len_s: 0.3,
        wing_expo: 1.6,
        wing_waves: 0.7,
        flap: 0.0,
        whip: 0.05,
    };
    /// A ray's (rajiform): the amplitudes are the wing tip's.
    pub const RAY: Self = Self {
        beat_rest_hz: 0.4,
        beat_per_len_hz: 0.6,
        hover_hz: 0.6,
        beat_max_hz: 3.0,
        amp_cruise: 0.08,
        amp_burst: 0.12,
        amp_hover: 0.03,
        amp_glide: 0.006,
        amp_ease_s: 0.25,
        head: 0.0,
        expo: 1.0,
        waves: 1.0,
        recoil: 0.0,
        recoil_reach: 0.3,
        bend_at_90: 0.04,
        bend_max: 0.1,
        bend_ease_s: 0.2,
        glide_len_s2: 0.3,
        hover_len_s: 0.3,
        wing_expo: 1.6,
        wing_waves: 0.7,
        flap: 0.0,
        whip: 0.05,
    };

    fn values(&self) -> [f32; 23] {
        [
            self.beat_rest_hz,
            self.beat_per_len_hz,
            self.hover_hz,
            self.beat_max_hz,
            self.amp_cruise,
            self.amp_burst,
            self.amp_hover,
            self.amp_glide,
            self.amp_ease_s,
            self.head,
            self.expo,
            self.waves,
            self.recoil,
            self.recoil_reach,
            self.bend_at_90,
            self.bend_max,
            self.bend_ease_s,
            self.glide_len_s2,
            self.hover_len_s,
            self.wing_expo,
            self.wing_waves,
            self.flap,
            self.whip,
        ]
    }
}

impl Default for WaveProfile {
    fn default() -> Self {
        Self::FISH
    }
}

impl BehaviourSettings {
    pub fn validate(&self) -> Result<(), String> {
        let finite = |name: &str, v: &[f32]| {
            if v.iter().all(|x| x.is_finite() && *x >= 0.0) {
                Ok(())
            } else {
                Err(format!(
                    "behaviour.{name}: every value must be finite and not negative"
                ))
            }
        };
        let positive = |name: &str, v: &[f32]| {
            if v.iter().all(|x| x.is_finite() && *x > 0.0) {
                Ok(())
            } else {
                Err(format!(
                    "behaviour.{name}: these must be finite and positive"
                ))
            }
        };
        let p = &self.pose;
        positive(
            "pose",
            &[
                p.turn_deg_s,
                p.heading_ease_s,
                p.pitch_max_deg,
                p.roll_ease_s,
            ],
        )?;
        finite("pose", &[p.bank_at_90_deg, p.bank_max_deg])?;
        if p.pitch_max_deg >= 90.0 {
            return Err("behaviour.pose.pitch_max_deg must be under 90".into());
        }
        let l = &self.lure;
        positive(
            "lure",
            &[
                l.drift_period_s,
                l.dart_s,
                l.back_off_s,
                l.shake_hz,
                l.circle_m,
                l.take_dart_s,
                l.take_turn_deg_s,
                l.bolt_speed,
                l.bolt_s,
            ],
        )?;
        finite(
            "lure",
            &[
                l.standoff_m,
                l.hover_drift_m,
                l.shake_deg,
                l.tug_m,
                l.take_mps,
                l.take_dive_deg,
                l.take_reach_m,
            ],
        )?;
        if !(0.0..=1.0).contains(&l.circle_chance) {
            return Err("behaviour.lure.circle_chance is 0..1".into());
        }
        let f = &self.fight;
        positive(
            "fight",
            &[f.turn_deg_s, f.ease_s, f.accel_deg_s2, f.fade_s, f.shake_hz],
        )?;
        finite("fight", &[f.shake_deg, f.flash_deg, f.run_dive_deg])?;
        for (name, w) in [
            ("fish_wave", &self.fish_wave),
            ("eel_wave", &self.eel_wave),
            ("ray_wave", &self.ray_wave),
        ] {
            finite(name, &w.values())?;
            positive(
                name,
                &[
                    w.beat_max_hz,
                    w.amp_ease_s,
                    w.expo,
                    w.waves,
                    w.recoil_reach,
                    w.bend_ease_s,
                    w.wing_expo,
                ],
            )?;
            if w.flap > 1.0 || w.head > 1.0 {
                return Err(format!("behaviour.{name}: flap and head are 0..1"));
            }
        }
        Ok(())
    }

    /// The wave profile of a body that sways as `kind`.
    pub fn wave(&self, kind: WaveKind) -> &WaveProfile {
        match kind {
            WaveKind::Fish => &self.fish_wave,
            WaveKind::Eel => &self.eel_wave,
            WaveKind::Ray => &self.ray_wave,
        }
    }
}

/// Which body wave a body has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaveKind {
    Fish,
    Eel,
    Ray,
}

// ---- the mockup's fixed terms (not sliders) ---------------------------------

/// How long the turn rate a bank and a bend read is smoothed over, s.
const YAW_RATE_EASE_S: f32 = 0.08;
/// How long a fish's change of speed is smoothed over, s.
const ACCEL_EASE_S: f32 = 0.15;
/// How far under the lure's level a hovering fish keeps, m.
const HOVER_UNDER_M: f32 = 0.03;
/// The nibble's dip: the last this long of a nibble's timer
/// (`fishing::NIBBLE_DIP_S`), when the mouth is on the lure.
const DIP_S: f32 = crate::fishing::NIBBLE_DIP_S;
/// A peck's mouth stops this short of the lure, m.
const PECK_GAP_M: f32 = 0.01;
/// A peck's lunge onto the lure and its turn to face it: how fast it closes,
/// 1/s, and its turn, deg/s, eased over s.
const PECK_CLOSE_PER_S: f32 = 30.0;
const PECK_TURN_DEG_S: f32 = 900.0;
const PECK_TURN_EASE_S: f32 = 0.03;
const DART_TURN_EASE_S: f32 = 0.04;
/// Between pecks: the least gap a circle needs, s, its rise, m, and its
/// squash across the line to the lure.
const CIRCLE_GAP_S: f32 = 0.45;
const CIRCLE_RISE_M: f32 = 0.08;
const CIRCLE_ACROSS: f32 = 0.9;
/// How quickly a hovering fish settles back on its stand-off point, 1/s.
const SETTLE_PER_S: f32 = 3.0;
/// The take's run eases out at this rate, 1/s; its turn is eased over s.
const TAKE_EASE_PER_S: f32 = 1.5;
const TAKE_TURN_EASE_S: f32 = 0.05;
/// The take runs off the line to the angler by this much, rad, either side.
const TAKE_ASIDE_RAD: f32 = 0.6;
/// A bolt dives this steeply (its fall over its run), and eases out at this
/// rate, 1/s.
const BOLT_DIVE: f32 = 0.25;
const BOLT_EASE_PER_S: f32 = 1.2;
/// A lure the take pulled aside eases back onto the line at this rate, 1/s.
const LURE_BACK_PER_S: f32 = 3.0;
/// Reeled in, the hook lifts the head this much (rise over run).
const REELED_LIFT: f32 = 0.12;
/// The speed a fish on the line is drawn swimming at, m/s, at the least:
/// reeled, and running.
const REELED_MPS: f32 = 0.4;
const RUNNING_MPS: f32 = 1.2;

/// The two tangents at `up` a heading's yaw is measured in: east and north,
/// stable away from the poles' axis (as `school::tangents`).
fn basis(up: Vec3) -> (Vec3, Vec3) {
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    (east, up.cross(east))
}

/// Yaw (about up, from east toward north) and pitch (from level) of a unit
/// direction.
fn yaw_pitch(d: Vec3, up: Vec3) -> (f32, f32) {
    let (east, north) = basis(up);
    (
        d.dot(north).atan2(d.dot(east)),
        d.dot(up).clamp(-1.0, 1.0).asin(),
    )
}

fn from_yaw_pitch(yaw: f32, pitch: f32, up: Vec3) -> Vec3 {
    let (east, north) = basis(up);
    (east * yaw.cos() + north * yaw.sin()) * pitch.cos() + up * pitch.sin()
}

fn wrap(a: f32) -> f32 {
    a.sin().atan2(a.cos())
}

fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn flat(v: Vec3, up: Vec3) -> Vec3 {
    v - up * v.dot(up)
}

/// The body wave this frame: its phase (rad), amplitude and bend (body
/// lengths). Zero amplitude and bend draws the body as laid.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wave {
    pub phase: f32,
    pub amp: f32,
    pub bend: f32,
}

/// One drawn fish (design 5's `FishPose`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// Its middle (the model's origin), local.
    pub pos: Vec3,
    /// Which way it faces, unit.
    pub forward: Vec3,
    /// Its bank into a turn, and the flash of a hooked fish, rad.
    pub roll: f32,
    pub flash: f32,
    /// A head shake, rad, turning the body about its mouth.
    pub shake: f32,
    yaw_vel: f32,
    pitch_vel: f32,
    /// How fast it is turning, smoothed, rad/s (positive to the left).
    pub yaw_rate: f32,
    /// Its speed, m/s, and how fast that is changing, smoothed, m/s^2.
    pub speed: f32,
    prev_speed: f32,
    pub accel: f32,
    pub wave: Wave,
    fight_env: f32,
    shake_phase: f32,
}

impl Pose {
    /// A fish first drawn at `pos` swimming at `velocity`: facing along it,
    /// level; its wave starts at `phase` (rad), cruising.
    pub fn new(pos: Vec3, velocity: Vec3, up: Vec3, phase: f32, profile: &WaveProfile) -> Self {
        let (east, _) = basis(up);
        let speed = velocity.length();
        Self {
            pos,
            forward: flat(velocity, up).normalize_or(east),
            roll: 0.0,
            flash: 0.0,
            shake: 0.0,
            yaw_vel: 0.0,
            pitch_vel: 0.0,
            yaw_rate: 0.0,
            speed: if speed.is_finite() { speed } else { 0.0 },
            prev_speed: if speed.is_finite() { speed } else { 0.0 },
            accel: 0.0,
            wave: Wave {
                phase,
                amp: profile.amp_cruise,
                bend: 0.0,
            },
            fight_env: 0.0,
            shake_phase: 0.0,
        }
    }

    /// Move by `-shift`: the origin was rebased.
    pub fn rebase(&mut self, shift: Vec3) {
        self.pos -= shift;
    }

    /// Turn toward `dir` by yaw and pitch, each eased over `tau` and
    /// limited to `rate_deg_s` (pitch to 0.6 of it), the pitch held inside
    /// `pitch_max`: a half turn goes round level, never through nose-down
    /// (the mockup's `turnToward`).
    pub fn turn_toward(
        &mut self,
        up: Vec3,
        dir: Vec3,
        rate_deg_s: f32,
        tau: f32,
        pitch_max_deg: f32,
        dt: f32,
    ) {
        if dir.length_squared() < 1e-10 || dt.is_nan() || dt <= 0.0 || !dir.is_finite() {
            return;
        }
        let d = dir.normalize();
        let (yaw0, pit0) = yaw_pitch(self.forward, up);
        let (yaw_t, pit_t) = yaw_pitch(d, up);
        let lim = pitch_max_deg * DEG;
        let pit_t = pit_t.clamp(-lim, lim);
        let k = 1.0 - (-dt / tau.max(1e-3)).exp();
        let rate = rate_deg_s * DEG * dt;
        let mut dy = wrap(yaw_t - yaw0);
        if flat(d, up).length() < 1e-6 {
            dy = 0.0;
        }
        let sy = dy.signum() * (dy.abs() * k).min(rate);
        let dp = pit_t - pit0;
        let sp = dp.signum() * (dp.abs() * k).min(rate * 0.6);
        let pit = (pit0 + sp).clamp(-lim, lim);
        self.forward = from_yaw_pitch(yaw0 + sy, pit, up);
        self.yaw_vel = sy / dt;
        self.pitch_vel = (pit - pit0) / dt;
        self.yaw_rate += (sy / dt - self.yaw_rate) * (dt / YAW_RATE_EASE_S).min(1.0);
    }

    /// The same turn, with the turn RATE eased too: it builds and dies at
    /// no more than `accel_deg_s2`, and is never more than can still be
    /// braked to a stop in the angle left (the mockup's `turnEased`,
    /// design 3.1). The rate a turn ends with carries into the next.
    #[allow(clippy::too_many_arguments)]
    pub fn turn_eased(
        &mut self,
        up: Vec3,
        dir: Vec3,
        rate_deg_s: f32,
        accel_deg_s2: f32,
        tau: f32,
        pitch_max_deg: f32,
        dt: f32,
    ) {
        if dir.length_squared() < 1e-10 || dt.is_nan() || dt <= 0.0 || !dir.is_finite() {
            return;
        }
        let d = dir.normalize();
        let (yaw0, pit0) = yaw_pitch(self.forward, up);
        let (yaw_t, pit_t) = yaw_pitch(d, up);
        let lim = pitch_max_deg * DEG;
        let pit_t = pit_t.clamp(-lim, lim);
        let mut dy = wrap(yaw_t - yaw0);
        if flat(d, up).length() < 1e-6 {
            dy = 0.0;
        }
        let acc = accel_deg_s2 * DEG;
        let t = tau.max(1e-3);
        let want = |e: f32, cap: f32| {
            e.signum() * (e.abs() / t).min((2.0 * acc * e.abs()).sqrt()).min(cap)
        };
        self.yaw_vel += (want(dy, rate_deg_s * DEG) - self.yaw_vel).clamp(-acc * dt, acc * dt);
        self.pitch_vel += (want(pit_t - pit0, rate_deg_s * DEG * 0.6) - self.pitch_vel)
            .clamp(-acc * dt, acc * dt);
        let raw = pit0 + self.pitch_vel * dt;
        let pit = raw.clamp(-lim, lim);
        if pit != raw {
            self.pitch_vel = (pit - pit0) / dt;
        }
        self.forward = from_yaw_pitch(yaw0 + self.yaw_vel * dt, pit, up);
        self.yaw_rate += (self.yaw_vel - self.yaw_rate) * (dt / YAW_RATE_EASE_S).min(1.0);
    }

    /// The frame's bank and body wave, from its speed, the speed's change
    /// and its turn (design 4 and 8.3; the mockup's `beat` and
    /// `Swimmer.swim`). `length_m` is the fish's length and `speeds` its
    /// species' cruising and top speed, m/s.
    pub fn settle(
        &mut self,
        dt: f32,
        pose: &PoseSettings,
        w: &WaveProfile,
        length_m: f32,
        speeds: (f32, f32),
    ) {
        if dt.is_nan() || dt <= 0.0 {
            return;
        }
        let speed = if self.speed.is_finite() {
            self.speed
        } else {
            0.0
        };
        self.accel +=
            ((speed - self.prev_speed) / dt.max(1e-4) - self.accel) * (dt / ACCEL_EASE_S).min(1.0);
        self.prev_speed = speed;
        let len = length_m.max(0.02);
        let (v_bl, a_bl) = (speed / len, self.accel / len);
        let (cruise_bl, burst_bl) = (speeds.0 / len, speeds.1 / len);
        let beat = (w.beat_rest_hz + w.beat_per_len_hz * v_bl).min(w.beat_max_hz);
        let (amp, hz) = if v_bl < w.hover_len_s {
            (w.amp_hover, w.hover_hz)
        } else if a_bl < -w.glide_len_s2 {
            (w.amp_glide, beat)
        } else {
            let k = ((v_bl - cruise_bl) / (burst_bl - cruise_bl).max(1e-3)).clamp(0.0, 1.0);
            (w.amp_cruise + (w.amp_burst - w.amp_cruise) * k, beat)
        };
        self.wave.amp += (amp - self.wave.amp) * (1.0 - (-dt / w.amp_ease_s.max(1e-3)).exp());
        self.wave.phase = (self.wave.phase + TAU * hz * dt).rem_euclid(TAU);
        // A left turn (positive yaw) bows the body to the left.
        let yaw_deg_s = self.yaw_rate / DEG;
        let bend = (-w.bend_at_90 * yaw_deg_s / 90.0).clamp(-w.bend_max, w.bend_max);
        self.wave.bend += (bend - self.wave.bend) * (1.0 - (-dt / w.bend_ease_s.max(1e-3)).exp());
        let bank = (-yaw_deg_s / 90.0 * pose.bank_at_90_deg)
            .clamp(-pose.bank_max_deg, pose.bank_max_deg)
            * DEG;
        self.roll += (bank - self.roll) * (dt / pose.roll_ease_s.max(1e-3)).min(1.0);
        if !(self.wave.amp.is_finite() && self.wave.bend.is_finite() && self.roll.is_finite()) {
            self.wave = Wave::default();
            self.roll = 0.0;
        }
    }

    /// How it is drawn: its forward and up (banked, flashing, its head
    /// shaken), and its model's origin, local, so that a shake turns it
    /// about its mouth `mouth_m` ahead of its middle (the mockup's `frame`).
    pub fn frame(&self, up: Vec3, mouth_m: f32) -> (Vec3, Vec3, Vec3) {
        let (east, _) = basis(up);
        let side0 = self.forward.cross(up).normalize_or(east);
        let top0 = side0.cross(self.forward).normalize_or(up);
        let r = self.roll + self.flash;
        let top = (top0 * r.cos() + side0 * r.sin()).normalize_or(up);
        let forward = if self.shake != 0.0 {
            (Quat::from_axis_angle(top, self.shake) * self.forward).normalize_or(self.forward)
        } else {
            self.forward
        };
        let origin = self.pos + self.forward * mouth_m - forward * mouth_m;
        (forward, top, origin)
    }

    /// Where its mouth is drawn, local.
    pub fn mouth(&self, up: Vec3, mouth_m: f32) -> Vec3 {
        let (forward, _, origin) = self.frame(up, mouth_m);
        origin + forward * mouth_m
    }

    /// A free-swimming fish this frame (the mockup's `draw` for a fish of
    /// the school): drawn at `pos`, between its school's steps, turned
    /// toward `velocity` (its vertical part halved).
    pub fn swim(&mut self, pos: Vec3, velocity: Vec3, up: Vec3, s: &PoseSettings, dt: f32) {
        self.pos = pos;
        self.speed = velocity.length();
        self.shake = 0.0;
        self.flash = 0.0;
        self.fight_env = 0.0;
        let want = velocity - up * velocity.dot(up) * 0.5;
        self.turn_toward(
            up,
            want,
            s.turn_deg_s,
            s.heading_ease_s,
            s.pitch_max_deg,
            dt,
        );
    }
}

/// What the line is doing this frame, as the fish it works sees it.
#[derive(Clone, Copy, Debug)]
pub struct LineNow {
    pub phase: Phase,
    /// The line's own clock ([`crate::fishing::Line::timer`]) and run.
    pub timer: f32,
    pub run: f32,
    /// Whether the line is still working this fish.
    pub working: bool,
    /// The lure as drawn under the drawn float, before what the fish does to
    /// it; and where the angler stands. Local.
    pub lure: Vec3,
    pub feet: Vec3,
}

/// The fish's own measures: its length, how far its mouth is ahead of its
/// middle, m, and its species' cruising and top speeds, m/s.
#[derive(Clone, Copy, Debug)]
pub struct Build {
    pub length_m: f32,
    pub mouth_m: f32,
    pub speeds: (f32, f32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Mode {
    Nibble,
    Bite,
    Hooked,
    Bolt,
}

/// The fish the line is working, from its first nibble until its bolt ends
/// or it is landed (the mockup's `Choreo`, design 2 and 3).
#[derive(Clone, Debug, PartialEq)]
pub struct LineFish {
    /// Its school's seed and its id.
    pub key: (u64, u32),
    mode: Mode,
    /// The side of the lure it came up on, unit, along the water.
    side: Vec3,
    t: f32,
    last_timer: f32,
    back_from: Option<Vec3>,
    dart_from: Option<Vec3>,
    circle: bool,
    loop_s: f32,
    loop_dir: f32,
    from: Vec3,
    run_dir: Vec3,
    hold: Option<Vec3>,
    bolt_dir: Vec3,
    /// How far the fish has the lure pulled from where the line holds it,
    /// local; drawn only.
    pub lure_offset: Vec3,
    rng: Rng,
}

/// A bolting fish's end: where it is and how fast it swims, to hand back
/// to its school.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Handback {
    pub pos: Vec3,
    pub velocity: Vec3,
}

impl LineFish {
    /// The line's first nibble on fish `key`, drawn at `pose`.
    pub fn begin(o: &SimOrigin, key: (u64, u32), pose: &Pose, lure: Vec3) -> Self {
        let up = o.up_at(lure);
        let (east, _) = basis(up);
        Self {
            key,
            mode: Mode::Nibble,
            side: flat(pose.pos - lure, up).normalize_or(east),
            t: 0.0,
            last_timer: f32::INFINITY,
            back_from: None,
            dart_from: None,
            circle: false,
            loop_s: 0.0,
            loop_dir: 1.0,
            from: pose.pos,
            run_dir: east,
            hold: None,
            bolt_dir: east,
            lure_offset: Vec3::ZERO,
            rng: Rng::new(key.0 ^ u64::from(key.1).wrapping_mul(0x51ed) ^ 0x51ed),
        }
    }

    /// Whether it is bolting: its school has let it go and it is no longer
    /// the line's.
    pub fn bolting(&self) -> bool {
        self.mode == Mode::Bolt
    }

    /// Move by `-shift`: the origin was rebased.
    pub fn rebase(&mut self, shift: Vec3) {
        for p in [&mut self.back_from, &mut self.dart_from, &mut self.hold]
            .into_iter()
            .flatten()
        {
            *p -= shift;
        }
        self.from -= shift;
    }

    /// The stand-off point: its middle, on the side it came from, its mouth
    /// `standoff_m` off the lure, a little under it.
    fn station(&self, up: Vec3, lure: Vec3, s: &LureSettings, b: &Build) -> Vec3 {
        lure + self.side * (s.standoff_m + b.mouth_m) - up * HOVER_UNDER_M
    }

    /// One frame. `clamp` keeps a point in the water. Returns the fish to
    /// hand back to its school when its bolt is over.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        o: &SimOrigin,
        pose: &mut Pose,
        line: &LineNow,
        b: &Build,
        s: &BehaviourSettings,
        dt: f32,
        clamp: impl Fn(Vec3) -> Vec3,
    ) -> Option<Handback> {
        if dt.is_nan() || dt <= 0.0 {
            return None;
        }
        let lure = line.lure;
        let up = o.up_at(lure);
        let ls = &s.lure;
        let ps = &s.pose;
        self.t += dt;
        let kept = self.lure_offset * (-dt * LURE_BACK_PER_S).exp();
        self.lure_offset = Vec3::ZERO;
        pose.shake = 0.0;
        pose.flash = 0.0;
        // What the line did since last frame.
        let free = !line.working;
        match self.mode {
            Mode::Nibble | Mode::Bite | Mode::Hooked if free => {
                self.mode = Mode::Bolt;
                self.t = 0.0;
                self.bolt_dir = (flat(pose.pos - lure, up).normalize_or(self.side)
                    - up * BOLT_DIVE)
                    .normalize();
            }
            Mode::Nibble if line.phase == Phase::Bite => {
                self.mode = Mode::Bite;
                self.t = 0.0;
                self.from = pose.pos;
                self.hold = None;
                let aside = if self.rng.unit() < 0.5 { -1.0 } else { 1.0 } * TAKE_ASIDE_RAD;
                let away = flat(lure - line.feet, up).normalize_or(self.side);
                let away = Quat::from_axis_angle(up, aside) * away;
                self.run_dir = (away - up * (ls.take_dive_deg * DEG).tan()).normalize();
            }
            Mode::Nibble | Mode::Bite if line.phase == Phase::Hooked => {
                self.mode = Mode::Hooked;
            }
            _ => {}
        }
        match self.mode {
            Mode::Nibble => self.nibble(up, pose, line, b, s, dt),
            Mode::Bite => self.take(up, pose, lure, b, ls, ps, dt),
            Mode::Hooked => {
                // A lure the take pulled aside eases back onto the line.
                self.lure_offset = kept;
                fight(up, pose, line, lure + kept, b, s, dt);
            }
            Mode::Bolt => {
                let speed = b.speeds.1 * ls.bolt_speed * (-self.t * BOLT_EASE_PER_S).exp();
                pose.turn_toward(
                    up,
                    self.bolt_dir,
                    ls.take_turn_deg_s,
                    TAKE_TURN_EASE_S,
                    ps.pitch_max_deg,
                    dt,
                );
                pose.speed = speed;
                pose.pos = clamp(pose.pos + pose.forward * speed * dt);
                if self.t > ls.bolt_s {
                    return Some(Handback {
                        pos: pose.pos,
                        velocity: pose.forward * speed.max(b.speeds.0),
                    });
                }
            }
        }
        None
    }

    /// Each nibble: hover off the lure, dart in so the mouth meets it while
    /// the float dips (the last `DIP_S` of the nibble's timer), shake and
    /// tug, back off; maybe turn away and circle back.
    fn nibble(
        &mut self,
        up: Vec3,
        pose: &mut Pose,
        line: &LineNow,
        b: &Build,
        s: &BehaviourSettings,
        dt: f32,
    ) {
        let ls = &s.lure;
        let ps = &s.pose;
        let lure = line.lure;
        // The line steps every frame, so its timer is this frame's.
        let rem = line.timer;
        if line.timer > self.last_timer + 1e-3 {
            // A new nibble.
            self.t = 0.0;
            self.back_from = Some(pose.pos);
            let gap = line.timer - ls.dart_s - DIP_S - DIP_S;
            self.circle = gap > CIRCLE_GAP_S && self.rng.unit() < ls.circle_chance;
            self.loop_s = gap;
            self.loop_dir = if self.rng.unit() < 0.5 { -1.0 } else { 1.0 };
        }
        self.last_timer = line.timer;
        let (east, _) = basis(up);
        let across = up.cross(self.side).normalize_or(east);
        let w = self.t * TAU / ls.drift_period_s;
        let st = self.station(up, lure, ls, b)
            + across * w.sin() * ls.hover_drift_m
            + up * (w * 0.7 + 1.0).sin() * ls.hover_drift_m * 0.5;
        let face = (lure - st).normalize_or(-self.side);
        let contact = lure - face * (b.mouth_m + PECK_GAP_M);
        let prev = pose.pos;
        if rem <= DIP_S {
            let k = 1.0 - rem / DIP_S;
            pose.pos = pose.pos.lerp(contact, 1.0 - (-dt * PECK_CLOSE_PER_S).exp());
            pose.turn_toward(
                up,
                face,
                PECK_TURN_DEG_S,
                PECK_TURN_EASE_S,
                ps.pitch_max_deg,
                dt,
            );
            pose.shake =
                ls.shake_deg * DEG * (TAU * ls.shake_hz * (DIP_S - rem)).sin() * (1.0 - 0.5 * k);
            self.lure_offset = -face * ls.tug_m * (PI * k.clamp(0.0, 1.0)).sin();
            self.dart_from = None;
        } else if rem <= DIP_S + ls.dart_s {
            let from = *self.dart_from.get_or_insert(pose.pos);
            let k = ease(1.0 - (rem - DIP_S) / ls.dart_s);
            pose.pos = from.lerp(contact, k);
            pose.turn_toward(
                up,
                face,
                PECK_TURN_DEG_S,
                DART_TURN_EASE_S,
                ps.pitch_max_deg,
                dt,
            );
        } else if let (Some(from), false, true) =
            (self.back_from, self.circle, self.t < ls.back_off_s)
        {
            pose.pos = from.lerp(st, ease(self.t / ls.back_off_s));
            pose.turn_toward(
                up,
                lure - pose.pos,
                ps.turn_deg_s,
                ps.heading_ease_s,
                ps.pitch_max_deg,
                dt,
            );
        } else if self.circle && self.t < self.loop_s {
            // Turn away and swim a small loop, back at the stand-off point
            // facing the lure.
            let u = ease(self.t / self.loop_s);
            let th = u * TAU * self.loop_dir;
            let side = face.cross(up).normalize_or(east);
            let c = st - face * ls.circle_m;
            let mut at =
                c + face * ls.circle_m * th.cos() + side * ls.circle_m * th.sin() * CIRCLE_ACROSS;
            at += up * (u * PI).sin() * CIRCLE_RISE_M;
            if let Some(from) = self.back_from {
                // It leaves from where the peck ended.
                at += (from - st) * (1.0 - ease(u / 0.3));
            }
            pose.pos = at;
            let tangent = at - prev;
            if tangent.length_squared() > 1e-10 {
                pose.turn_toward(
                    up,
                    tangent,
                    ps.turn_deg_s,
                    ps.heading_ease_s * 0.6,
                    ps.pitch_max_deg,
                    dt,
                );
            }
        } else {
            pose.pos = pose.pos.lerp(st, 1.0 - (-dt * SETTLE_PER_S).exp());
            pose.turn_toward(
                up,
                lure - pose.pos,
                ps.turn_deg_s,
                ps.heading_ease_s * 2.0,
                ps.pitch_max_deg,
                dt,
            );
        }
        pose.speed = prev.distance(pose.pos) / dt.max(1e-4);
    }

    /// The take: a quick dart onto the lure, then a hard turn and a run
    /// down and away; the drawn lure goes with the mouth.
    #[allow(clippy::too_many_arguments)]
    fn take(
        &mut self,
        up: Vec3,
        pose: &mut Pose,
        lure: Vec3,
        b: &Build,
        ls: &LureSettings,
        ps: &PoseSettings,
        dt: f32,
    ) {
        let prev = pose.pos;
        if self.t < ls.take_dart_s {
            let face = (lure - pose.pos).normalize_or(-self.side);
            let contact = lure - face * b.mouth_m;
            pose.pos = self.from.lerp(contact, ease(self.t / ls.take_dart_s));
            pose.turn_toward(
                up,
                face,
                PECK_TURN_DEG_S,
                PECK_TURN_EASE_S,
                ps.pitch_max_deg,
                dt,
            );
            self.hold = None;
        } else {
            pose.turn_toward(
                up,
                self.run_dir,
                ls.take_turn_deg_s,
                TAKE_TURN_EASE_S,
                ps.pitch_max_deg,
                dt,
            );
            let speed = ls.take_mps * (-(self.t - ls.take_dart_s) * TAKE_EASE_PER_S).exp();
            let mut hold = self.hold.unwrap_or(lure) + pose.forward * speed * dt;
            let off = hold - lure;
            if off.length() > ls.take_reach_m {
                hold = lure + off.normalize_or_zero() * ls.take_reach_m;
            }
            self.hold = Some(hold);
            self.lure_offset = hold - lure;
            pose.pos = hold - pose.forward * b.mouth_m;
        }
        pose.speed = prev.distance(pose.pos) / dt.max(1e-4);
    }
}

/// On the line: the mouth stays on the lure and the body swings round it.
/// Running, head away from the rod, diving a little; being reeled, the hook
/// turns the head toward the rod, and it shakes its head and rolls,
/// flashing. Nothing switches in one frame: the turn's rate is eased, the
/// shake and the flash fade in and out on one envelope, and their phase
/// runs on the frame's own clock (the mockup's `fight`, design 3.1).
fn fight(
    up: Vec3,
    pose: &mut Pose,
    line: &LineNow,
    lure: Vec3,
    b: &Build,
    s: &BehaviourSettings,
    dt: f32,
) {
    let f = &s.fight;
    let (east, _) = basis(up);
    let away = flat(lure - line.feet, up).normalize_or(east);
    let prev = pose.pos;
    let reeled = line.run.is_nan() || line.run <= 0.0;
    let want = if reeled {
        -away + up * REELED_LIFT
    } else {
        away - up * (f.run_dive_deg * DEG).tan()
    };
    pose.turn_eased(
        up,
        want,
        f.turn_deg_s,
        f.accel_deg_s2,
        f.ease_s,
        s.pose.pitch_max_deg,
        dt,
    );
    let target = if reeled { 1.0 } else { 0.0 };
    pose.fight_env += (target - pose.fight_env) * (1.0 - (-dt / f.fade_s.max(1e-3)).exp());
    pose.shake_phase = (pose.shake_phase + TAU * f.shake_hz * dt).rem_euclid(2.0 * TAU);
    pose.shake = pose.fight_env * f.shake_deg * DEG * pose.shake_phase.sin();
    pose.flash = pose.fight_env * f.flash_deg * DEG * (pose.shake_phase * 0.5 + 0.7).sin();
    pose.pos = lure - pose.forward * b.mouth_m;
    pose.speed =
        (prev.distance(pose.pos) / dt.max(1e-4)).max(if reeled { REELED_MPS } else { RUNNING_MPS });
}

/// Every drawn swimmer's pose and the line's fish, frame to frame (design
/// 5): poses by school seed and fish id, so a fish caught out of its school
/// does not move another's.
#[derive(Clone, Debug, Default)]
pub struct Drawing {
    pub poses: HashMap<(u64, u32), Pose>,
    pub line_fish: Option<LineFish>,
    seen: HashMap<(u64, u32), u64>,
    frame: u64,
}

/// What the frame's drawing needs of the line: the line, its lure as drawn
/// under the drawn float, and where the angler stands, local.
#[derive(Clone, Copy, Debug)]
pub struct LineView<'a> {
    pub line: &'a Line,
    pub lure: Vec3,
    pub feet: Vec3,
}

impl Drawing {
    /// Move every pose by `-shift`: the origin was rebased.
    pub fn rebase(&mut self, shift: Vec3) {
        for pose in self.poses.values_mut() {
            pose.rebase(shift);
        }
        if let Some(fish) = self.line_fish.as_mut() {
            fish.rebase(shift);
        }
    }

    /// How far the line's fish has the lure pulled, local; drawn only.
    pub fn lure_offset(&self) -> Vec3 {
        self.line_fish
            .as_ref()
            .map_or(Vec3::ZERO, |f| f.lure_offset)
    }

    /// One frame: every swimmer of every school turned and its wave stepped,
    /// and the line's fish moved by its choreography. `alpha` is how far the
    /// schools' clock has run from its last step; `build` gives a fish's
    /// measures and its wave, `None` for one not drawn as a swimmer; `clamp`
    /// keeps a point in the water. A bolt that ends is handed back to its
    /// school here.
    #[allow(clippy::too_many_arguments)]
    pub fn frame(
        &mut self,
        o: &SimOrigin,
        schools: &mut [School],
        roster: &[Species],
        s: &BehaviourSettings,
        view: Option<LineView>,
        alpha: f32,
        dt: f32,
        build: impl Fn(&School, usize, &Species) -> Option<(Build, WaveKind)>,
        clamp: impl Fn(Vec3) -> Vec3,
    ) {
        self.frame += 1;
        let frame = self.frame;
        // The fish the line is working now, by key.
        let working = view.and_then(|v| {
            let line = v.line;
            if !matches!(line.phase, Phase::Nibble | Phase::Bite | Phase::Hooked) {
                return None;
            }
            let t = line.target?;
            let school = schools.get(t.school)?;
            Some((school.seed, *school.ids.get(t.fish)?))
        });
        // A new fish at the lure: one still bolting is handed back first.
        if let Some(key) = working
            && self.line_fish.as_ref().is_none_or(|f| f.key != key)
        {
            if let Some(old) = self.line_fish.take()
                && let Some(pose) = self.poses.get(&old.key)
                && let Some(school) = schools.iter_mut().find(|sc| sc.seed == old.key.0)
            {
                school.place(old.key.1, pose.pos, pose.forward * pose.speed);
            }
            if let (Some(pose), Some(v)) = (self.poses.get(&key), view) {
                self.line_fish = Some(LineFish::begin(o, key, pose, v.lure));
            }
        }
        let mut line_fish_seen = false;
        let mut finished = false;
        for school in schools.iter_mut() {
            let Some(species) = roster.get(school.species as usize) else {
                continue;
            };
            if species.mover != Mover::School {
                continue;
            }
            let speeds = species.speed_mps;
            for i in 0..school.len() {
                let Some((b, kind)) = build(school, i, species) else {
                    continue;
                };
                let id = school.ids[i];
                let key = (school.seed, id);
                let profile = s.wave(kind);
                let pose = self.poses.entry(key).or_insert_with(|| {
                    let at = school.drawn(i, alpha);
                    let phase = (super::hash2(school.seed, u64::from(id)) >> 40) as f32
                        / (1u64 << 24) as f32
                        * TAU;
                    Pose::new(at, school.velocities[i], o.up_at(at), phase, profile)
                });
                self.seen.insert(key, frame);
                let mine = self.line_fish.as_mut().filter(|f| f.key == key);
                if let (Some(fish), Some(v)) = (mine, view) {
                    line_fish_seen = true;
                    let now = LineNow {
                        phase: v.line.phase,
                        timer: v.line.timer(),
                        run: v.line.run,
                        working: working == Some(key),
                        lure: v.lure,
                        feet: v.feet,
                    };
                    if let Some(back) = fish.update(o, pose, &now, &b, s, dt, &clamp) {
                        school.place(id, back.pos, back.velocity);
                        finished = true;
                    } else if fish.bolting() {
                        school.owned = Some(id);
                    }
                } else {
                    let at = school.drawn(i, alpha);
                    pose.swim(at, school.velocities[i], o.up_at(at), &s.pose, dt);
                }
                pose.settle(dt, &s.pose, profile, b.length_m, speeds);
            }
        }
        if (finished || !line_fish_seen)
            && let Some(fish) = self.line_fish.take()
            && let Some(school) = schools.iter_mut().find(|sc| sc.seed == fish.key.0)
            && school.owned == Some(fish.key.1)
        {
            school.owned = None;
        }
        let seen = &self.seen;
        self.poses.retain(|k, _| seen.get(k) == Some(&frame));
        self.seen.retain(|_, f| *f == frame);
    }
}

/// Where the float is drawn (`movement-smoothness` design 7.1): the line's
/// float plus an offset that takes up the one-frame jump the line makes
/// when a hooked fish starts or stops running (its swing switched on or
/// off) or is struck, and eases it out, critically damped. Draw only:
/// nothing in the line's rules reads it.
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawnFloat {
    /// Drawn minus simulated, local, m, and how fast it changes.
    pub offset: Vec3,
    velocity: Vec3,
    /// The line as last frame left it: its float, phase and run.
    last: Option<(Vec3, Phase, f32)>,
    /// Struck, and not yet swung: the line's first hooked step, a frame
    /// after the strike, puts the swing on.
    struck: bool,
}

/// How quickly a jump of the hooked float is eased out, 1/s: critically
/// damped, so 95% of it is gone in about 4.7 / this (a third of a second).
pub const FLOAT_EASE_PER_S: f32 = 14.0;

impl DrawnFloat {
    /// One frame: `float`, `phase` and `run` are the line's after this
    /// frame's step.
    pub fn update(&mut self, o: &SimOrigin, dt: f32, float: Vec3, phase: Phase, run: f32) {
        let hooked = phase == Phase::Hooked;
        let jumped = match self.last {
            Some((was, Phase::Hooked, was_run)) if hooked && (was_run != run || self.struck) => {
                self.struck = false;
                Some(was)
            }
            Some((was, Phase::Bite, _)) if hooked => {
                self.struck = true;
                Some(was)
            }
            _ => None,
        };
        self.last = Some((float, phase, run));
        if !hooked {
            *self = Self {
                last: self.last,
                ..Self::default()
            };
            return;
        }
        if let Some(was) = jumped {
            // Drawn where it was drawn last frame.
            self.offset += was - float;
        } else if dt > 0.0 && dt.is_finite() {
            // Critically damped: x = (x0 + (v0 + w x0) t) e^-wt.
            let w = FLOAT_EASE_PER_S;
            let e = (-w * dt).exp();
            let (x0, v0) = (self.offset, self.velocity);
            let c = v0 + x0 * w;
            self.offset = (x0 + c * dt) * e;
            self.velocity = (v0 - c * (w * dt)) * e;
        }
        // Along the water: no part of it toward the planet's centre.
        let up = o.up_at(float);
        self.offset -= up * self.offset.dot(up);
        self.velocity -= up * self.velocity.dot(up);
        if !(self.offset.is_finite() && self.velocity.is_finite()) {
            self.offset = Vec3::ZERO;
            self.velocity = Vec3::ZERO;
        }
    }

    /// The float as drawn: `float` moved by the offset and kept at its
    /// height over the centre.
    pub fn drawn(&self, o: &SimOrigin, float: Vec3) -> Vec3 {
        if self.offset == Vec3::ZERO {
            return float;
        }
        o.with_radius(float + self.offset, o.radius_at(float))
    }

    /// How far the drawn float is from the line's.
    pub fn shift(&self, o: &SimOrigin, float: Vec3) -> Vec3 {
        self.drawn(o, float) - float
    }

    /// The origin was rebased: the float it last saw moves with it.
    pub fn rebase(&mut self, shift: Vec3) {
        if let Some((float, _, _)) = self.last.as_mut() {
            *float -= shift;
        }
    }
}

/// The spine's lateral offset at `u` (0 nose, 1 tail) and its slope d/du,
/// body lengths: `swim.wgsl`'s `swim_lateral`, for the tests and the
/// trace. The shader is the one that draws.
pub fn lateral(u: f32, wave: Wave, w: &WaveProfile) -> (f32, f32) {
    let u = u.clamp(0.0, 1.0);
    let um = u.max(1e-3);
    let e = w.head + (1.0 - w.head) * um.powf(w.expo);
    let de = (1.0 - w.head) * w.expo * um.powf(w.expo - 1.0);
    let a = TAU * w.waves * u - wave.phase;
    let tail = (TAU * w.waves - wave.phase).sin();
    let r = (1.0 - u / w.recoil_reach).max(0.0);
    let q = (u - 0.35) / 0.65;
    let f = wave.amp * (e * a.sin() - w.recoil * r * r * tail) + wave.bend * q * q;
    let df = wave.amp
        * (de * a.sin() + e * TAU * w.waves * a.cos() + 2.0 * w.recoil * r / w.recoil_reach * tail)
        + wave.bend * 2.0 * q / 0.65;
    (f, df)
}

#[cfg(test)]
mod tests;
