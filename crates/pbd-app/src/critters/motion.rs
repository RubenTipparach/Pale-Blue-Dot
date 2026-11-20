//! The tunables of the critters' motion (`ocean-environments` design B28,
//! B29): `assets/config/creature_motion.ron`, the block M25's Beavers and
//! otters tab prints under its sliders, so a block tuned there pastes in
//! unchanged. `Default` is the one source of the defaults; a test holds the
//! shipped file to it. Drawn only: nothing in the game reads these back.

use serde::{Deserialize, Serialize};

/// Every block of the file, in the page's order.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CreatureMotion {
    pub beaver: Beaver,
    pub river_otter: RiverOtter,
    pub sea_otter: SeaOtter,
    pub shared: Shared,
    /// Scales every raise, roll, arch and recoil together.
    pub exaggerate: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Beaver {
    pub walk: Walk,
    pub gnaw: Gnaw,
    pub fell: Fell,
    pub swim: Swim,
    pub slap: Slap,
    pub shake: Shake,
    pub forelimb: Forelimb,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct RiverOtter {
    pub bound: Bound,
    pub slide: Slide,
    pub porpoise: Porpoise,
    pub eat: Eat,
    pub look: Look,
    pub forelimb: Forelimb,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SeaOtter {
    pub crack: Crack,
    pub roll: Roll,
    pub dive: Dive,
    pub sleep: Sleep,
    pub forelimb: Forelimb,
}

#[derive(Clone, Debug, PartialEq, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Shared {
    pub forelegs: Forelegs,
    pub upright: Upright,
    pub paddle: Paddle,
    pub fur: Fur,
}

/// The beaver's waddle: a lateral walk, its phase on the distance moved.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Walk {
    pub stride_m: f32,
    /// The share of the cycle a foot is down.
    pub stance: f32,
    pub speed_mps: f32,
    pub foot_lift_m: f32,
    pub waddle_roll_rad: f32,
    pub hip_bob_m: f32,
    pub head_lag_s: f32,
    pub tail_lag_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Gnaw {
    /// One bite's four stages: windup 0.30, anticipation 0.20, hit 0.12 and
    /// recoil 0.38 of it.
    pub beat_s: f32,
    pub pull_rad: f32,
    pub chips_per_bite: u32,
    pub bites_to_fell: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Fell {
    pub creak_s: f32,
    pub lean_back_rad: f32,
    pub fall_s: f32,
    pub bounce_rad: f32,
    pub settle_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Swim {
    pub speed_mps: f32,
    pub scull_hz: f32,
    pub scull_rad: f32,
    pub head_above_m: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Slap {
    pub windup_s: f32,
    pub anticipate_s: f32,
    pub strike_s: f32,
    pub recoil_s: f32,
    pub raise_rad: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Shake {
    pub hz: f32,
    pub time_s: f32,
    pub roll_rad: f32,
    pub dry_s: f32,
}

/// The forelimb's limits, degrees (version 5): the elbow's flex from
/// straight, the wrist curled down and under and bent back from in line
/// with the forearm, and the hands drawn in toward each other.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Forelimb {
    pub elbow_max_deg: f32,
    pub wrist_flex_deg: f32,
    pub wrist_ext_deg: f32,
    pub hands_in_deg: f32,
}

/// The river otter's bound: the hind pair together, the fore pair half a
/// cycle later.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Bound {
    pub stride_m: f32,
    pub stance: f32,
    pub speed_mps: f32,
    pub arch_rad: f32,
    pub lift_m: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Slide {
    pub accel_mps2: f32,
    pub max_mps: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Porpoise {
    pub arc_m: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Eat {
    pub beat_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Look {
    pub hold_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Crack {
    pub windup_s: f32,
    pub anticipate_s: f32,
    pub strike_s: f32,
    pub recoil_s: f32,
    pub raise_m: f32,
    pub cracks: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Roll {
    pub roll_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Dive {
    pub dive_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Sleep {
    pub kelp_wrap_s: f32,
}

/// Every rear-up and sit-down on one schedule (version 4), and the swimming
/// forepaws' reach.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Forelegs {
    pub anticipate_s: f32,
    pub rise_s: f32,
    pub lower_s: f32,
    pub overshoot: f32,
    pub swing_curl_rad: f32,
    pub paddle_rad: f32,
}

/// Where an upright animal's hands hang (version 5).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Upright {
    pub reach: f32,
    pub drop_deg: f32,
    pub curl_deg: f32,
}

/// The forepaws' stroke when swimming (version 5).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Paddle {
    pub paddle_idle: f32,
    pub paddle_phase_rad: f32,
    pub paddle_tuck: f32,
    pub rate_idle: f32,
}

/// The fur tufts' spring (version 4). Read and validated; the game's tufts
/// ride their bones in their dry shape until the spring is built (B29).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Fur {
    pub spring_hz: f32,
    pub damping: f32,
    pub sway: f32,
    pub max_m: f32,
    pub wet_spike: f32,
    pub shake: f32,
}

impl Default for Walk {
    fn default() -> Self {
        Self {
            stride_m: 0.36,
            stance: 0.62,
            speed_mps: 0.5,
            foot_lift_m: 0.05,
            waddle_roll_rad: 0.1,
            hip_bob_m: 0.02,
            head_lag_s: 0.1,
            tail_lag_s: 0.18,
        }
    }
}

impl Default for Gnaw {
    fn default() -> Self {
        Self {
            beat_s: 0.42,
            pull_rad: 0.35,
            chips_per_bite: 4,
            bites_to_fell: 12,
        }
    }
}

impl Default for Fell {
    fn default() -> Self {
        Self {
            creak_s: 1.0,
            lean_back_rad: 0.05,
            fall_s: 1.7,
            bounce_rad: 0.1,
            settle_s: 0.9,
        }
    }
}

impl Default for Swim {
    fn default() -> Self {
        Self {
            speed_mps: 0.75,
            scull_hz: 1.3,
            scull_rad: 0.35,
            head_above_m: 0.07,
        }
    }
}

impl Default for Slap {
    fn default() -> Self {
        Self {
            windup_s: 0.3,
            anticipate_s: 0.22,
            strike_s: 0.08,
            recoil_s: 0.35,
            raise_rad: 1.15,
        }
    }
}

impl Default for Shake {
    fn default() -> Self {
        Self {
            hz: 7.0,
            time_s: 1.0,
            roll_rad: 0.45,
            dry_s: 20.0,
        }
    }
}

impl Forelimb {
    const fn new(elbow: f32, flex: f32, ext: f32, hands_in: f32) -> Self {
        Self {
            elbow_max_deg: elbow,
            wrist_flex_deg: flex,
            wrist_ext_deg: ext,
            hands_in_deg: hands_in,
        }
    }
}

/// The beaver's (the struct's own default; each animal's block sets its own).
impl Default for Forelimb {
    fn default() -> Self {
        Self::new(80.0, 60.0, 100.0, 16.0)
    }
}

impl Default for Beaver {
    fn default() -> Self {
        Self {
            walk: Walk::default(),
            gnaw: Gnaw::default(),
            fell: Fell::default(),
            swim: Swim::default(),
            slap: Slap::default(),
            shake: Shake::default(),
            forelimb: Forelimb::new(80.0, 60.0, 100.0, 16.0),
        }
    }
}

impl Default for Bound {
    fn default() -> Self {
        Self {
            stride_m: 0.85,
            stance: 0.38,
            speed_mps: 1.5,
            arch_rad: 0.42,
            lift_m: 0.07,
        }
    }
}

impl Default for Slide {
    fn default() -> Self {
        Self {
            accel_mps2: 3.0,
            max_mps: 4.5,
        }
    }
}

impl Default for Porpoise {
    fn default() -> Self {
        Self { arc_m: 0.45 }
    }
}

impl Default for Eat {
    fn default() -> Self {
        Self { beat_s: 0.36 }
    }
}

impl Default for Look {
    fn default() -> Self {
        Self { hold_s: 0.9 }
    }
}

impl Default for RiverOtter {
    fn default() -> Self {
        Self {
            bound: Bound::default(),
            slide: Slide::default(),
            porpoise: Porpoise::default(),
            eat: Eat::default(),
            look: Look::default(),
            forelimb: Forelimb::new(100.0, 70.0, 100.0, 14.0),
        }
    }
}

impl Default for Crack {
    fn default() -> Self {
        Self {
            windup_s: 0.32,
            anticipate_s: 0.16,
            strike_s: 0.07,
            recoil_s: 0.26,
            raise_m: 0.16,
            cracks: 4,
        }
    }
}

impl Default for Roll {
    fn default() -> Self {
        Self { roll_s: 0.7 }
    }
}

impl Default for Dive {
    fn default() -> Self {
        Self { dive_s: 5.0 }
    }
}

impl Default for Sleep {
    fn default() -> Self {
        Self { kelp_wrap_s: 2.4 }
    }
}

impl Default for SeaOtter {
    fn default() -> Self {
        Self {
            crack: Crack::default(),
            roll: Roll::default(),
            dive: Dive::default(),
            sleep: Sleep::default(),
            forelimb: Forelimb::new(85.0, 70.0, 95.0, 12.0),
        }
    }
}

impl Default for Forelegs {
    fn default() -> Self {
        Self {
            anticipate_s: 0.18,
            rise_s: 0.42,
            lower_s: 0.55,
            overshoot: 0.12,
            swing_curl_rad: 0.8,
            paddle_rad: 0.7,
        }
    }
}

impl Default for Upright {
    fn default() -> Self {
        Self {
            reach: 0.86,
            drop_deg: 62.0,
            curl_deg: 25.0,
        }
    }
}

impl Default for Paddle {
    fn default() -> Self {
        Self {
            paddle_idle: 0.35,
            paddle_phase_rad: 1.57,
            paddle_tuck: 0.85,
            rate_idle: 0.45,
        }
    }
}

impl Default for Fur {
    fn default() -> Self {
        Self {
            spring_hz: 3.6,
            damping: 0.75,
            sway: 0.4,
            max_m: 0.012,
            wet_spike: 0.4,
            shake: 0.45,
        }
    }
}

/// `values` all finite and at least `lo` (and at most `hi`).
fn within(name: &str, values: &[f32], lo: f32, hi: f32) -> Result<(), String> {
    match values
        .iter()
        .find(|v| !(v.is_finite() && **v >= lo && **v <= hi))
    {
        Some(v) => Err(format!("{name}: {v} is outside {lo}..={hi}")),
        None => Ok(()),
    }
}

/// `values` all finite and above nought.
fn positive(name: &str, values: &[f32]) -> Result<(), String> {
    match values.iter().find(|v| !(v.is_finite() && **v > 0.0)) {
        Some(v) => Err(format!("{name}: {v} must be above nought")),
        None => Ok(()),
    }
}

impl Forelimb {
    /// The page's sliders' ranges: an elbow folded past 120 degrees, or a
    /// wrist past 120, is the fold version 5 was made to stop.
    fn validate(&self, name: &str) -> Result<(), String> {
        within(name, &[self.elbow_max_deg], 10.0, 120.0)?;
        within(name, &[self.wrist_flex_deg], 0.0, 90.0)?;
        within(name, &[self.wrist_ext_deg], 60.0, 120.0)?;
        within(name, &[self.hands_in_deg], 0.0, 30.0)
    }
}

impl CreatureMotion {
    /// What a legal block is: every time and length above nought, every
    /// share within 0..=1, the limits within the page's slider ranges.
    pub fn validate(&self) -> Result<(), String> {
        let b = &self.beaver;
        positive(
            "beaver.walk",
            &[
                b.walk.stride_m,
                b.walk.speed_mps,
                b.walk.head_lag_s,
                b.walk.tail_lag_s,
            ],
        )?;
        within("beaver.walk.stance", &[b.walk.stance], 0.05, 0.95)?;
        within(
            "beaver.walk",
            &[b.walk.foot_lift_m, b.walk.waddle_roll_rad, b.walk.hip_bob_m],
            0.0,
            1.0,
        )?;
        positive("beaver.gnaw", &[b.gnaw.beat_s, b.gnaw.pull_rad])?;
        if b.gnaw.bites_to_fell == 0 {
            return Err("beaver.gnaw.bites_to_fell must be at least one".into());
        }
        positive(
            "beaver.fell",
            &[b.fell.creak_s, b.fell.fall_s, b.fell.settle_s],
        )?;
        within(
            "beaver.fell",
            &[b.fell.lean_back_rad, b.fell.bounce_rad],
            0.0,
            1.0,
        )?;
        positive("beaver.swim", &[b.swim.speed_mps, b.swim.scull_hz])?;
        within(
            "beaver.swim",
            &[b.swim.scull_rad, b.swim.head_above_m],
            0.0,
            2.0,
        )?;
        positive(
            "beaver.slap",
            &[
                b.slap.windup_s,
                b.slap.strike_s,
                b.slap.recoil_s,
                b.slap.raise_rad,
            ],
        )?;
        within("beaver.slap.anticipate_s", &[b.slap.anticipate_s], 0.0, 5.0)?;
        positive(
            "beaver.shake",
            &[b.shake.hz, b.shake.time_s, b.shake.roll_rad, b.shake.dry_s],
        )?;
        b.forelimb.validate("beaver.forelimb")?;
        let o = &self.river_otter;
        positive("river_otter.bound", &[o.bound.stride_m, o.bound.speed_mps])?;
        within("river_otter.bound.stance", &[o.bound.stance], 0.05, 0.95)?;
        within(
            "river_otter.bound",
            &[o.bound.arch_rad, o.bound.lift_m],
            0.0,
            2.0,
        )?;
        positive(
            "river_otter",
            &[
                o.slide.accel_mps2,
                o.slide.max_mps,
                o.porpoise.arc_m,
                o.eat.beat_s,
                o.look.hold_s,
            ],
        )?;
        o.forelimb.validate("river_otter.forelimb")?;
        let s = &self.sea_otter;
        positive(
            "sea_otter.crack",
            &[
                s.crack.windup_s,
                s.crack.strike_s,
                s.crack.recoil_s,
                s.crack.raise_m,
            ],
        )?;
        within(
            "sea_otter.crack.anticipate_s",
            &[s.crack.anticipate_s],
            0.0,
            5.0,
        )?;
        if s.crack.cracks == 0 {
            return Err("sea_otter.crack.cracks must be at least one".into());
        }
        positive(
            "sea_otter",
            &[s.roll.roll_s, s.dive.dive_s, s.sleep.kelp_wrap_s],
        )?;
        s.forelimb.validate("sea_otter.forelimb")?;
        let f = &self.shared.forelegs;
        positive("shared.forelegs", &[f.rise_s, f.lower_s])?;
        within(
            "shared.forelegs",
            &[f.anticipate_s, f.overshoot, f.swing_curl_rad, f.paddle_rad],
            0.0,
            2.0,
        )?;
        let u = &self.shared.upright;
        within("shared.upright.reach", &[u.reach], 0.1, 1.0)?;
        within("shared.upright", &[u.drop_deg, u.curl_deg], 0.0, 90.0)?;
        let p = &self.shared.paddle;
        within(
            "shared.paddle",
            &[p.paddle_idle, p.paddle_tuck, p.rate_idle],
            0.0,
            1.0,
        )?;
        within(
            "shared.paddle.paddle_phase_rad",
            &[p.paddle_phase_rad],
            0.0,
            std::f32::consts::TAU,
        )?;
        let fur = &self.shared.fur;
        positive("shared.fur", &[fur.spring_hz, fur.max_m])?;
        within(
            "shared.fur",
            &[fur.damping, fur.sway, fur.wet_spike, fur.shake],
            0.0,
            2.0,
        )?;
        within("exaggerate", &[self.exaggerate], 0.05, 4.0)
    }
}

impl Default for CreatureMotion {
    fn default() -> Self {
        Self {
            beaver: Beaver::default(),
            river_otter: RiverOtter::default(),
            sea_otter: SeaOtter::default(),
            shared: Shared::default(),
            exaggerate: 1.0,
        }
    }
}
