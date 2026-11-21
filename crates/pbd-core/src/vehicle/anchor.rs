//! One anchor per boat (`river-craft` design 4, M34's Anchor tab, version 6).
//!
//! Dropped, the anchor falls from the bow to the bed and the rode pays out
//! only until it touches (the owner, 2026-11-19: "by default anchor rope
//! should stop extending as soon as it touches ground"); a longer rope set
//! in the boat's menu (`Craft::rope_m`) is then veered out. It sets on the
//! bed in `set_s`, sliding a little toward the boat under a hard pull until
//! it has; set, it holds the boat within its swing on the rode, whatever
//! rows or flows (the owner: "anchor in mockup should stop you from
//! moving"). Weighing hauls the rode in to straight up and down and lifts
//! the anchor aboard: the only way it lets go.
//!
//! The rode's pull is the craft's mooring line (`Craft::moor`): while the
//! anchor is on the bed, `Craft::mooring` is the line to it, and that is
//! what a save keeps. The anchor's work (falling, setting, hauling) is not
//! saved: a mooring read back from a save is an anchor set and holding.

use crate::locale::{Localized, tr};
use glam::DVec3;
use serde::{Deserialize, Serialize};

/// How a kind's anchor works, in `vehicles.ron` (`anchor:`): metres,
/// seconds, newtons.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorSpec {
    /// The most rode the boat carries, m: the rope setting's top.
    pub rode_max_m: f32,
    /// How fast it falls, and how fast the rode is veered, hauled and the
    /// anchor lifted aboard, m/s.
    pub drop_mps: f32,
    pub veer_mps: f32,
    pub haul_mps: f32,
    pub lift_mps: f32,
    /// From touching the bed to holding, s.
    pub set_s: f32,
    /// While it sets, a pull over this slides it toward the boat, N.
    pub slide_n: f32,
}

impl Default for AnchorSpec {
    /// M34's (design 4, 10.4): 40 m of rode, so the 35 m depth limit still
    /// leaves scope; a rock bed's setting time and slide.
    fn default() -> Self {
        Self {
            rode_max_m: 40.0,
            drop_mps: 2.4,
            veer_mps: 1.5,
            haul_mps: 1.0,
            lift_mps: 1.2,
            set_s: 1.3,
            slide_n: 900.0,
        }
    }
}

impl AnchorSpec {
    pub fn validate(&self, name: &str) -> Result<(), String> {
        let all = [
            self.rode_max_m,
            self.drop_mps,
            self.veer_mps,
            self.haul_mps,
            self.lift_mps,
            self.set_s,
            self.slide_n,
        ];
        if all.iter().all(|v| v.is_finite() && *v > 0.0) {
            Ok(())
        } else {
            Err(format!("{name}.anchor: every number is positive"))
        }
    }
}

/// Where the anchor is in its work.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Phase {
    /// Falling from the bow: how far under where it was let go, m.
    Dropping { fallen_m: f64 },
    /// On the bed: how far set, 0..1, and whether it is being weighed.
    Down { set: f64, weighing: bool },
    /// Broken out and coming aboard.
    Up,
}

/// The anchor while it is out of the boat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub phase: Phase,
    /// Where it is, planet frame.
    pub at: DVec3,
    /// Rode out, m.
    pub rode: f64,
    /// The rode it is veered or hauled to, m.
    pub target: f64,
    /// The pull on it, smoothed, N.
    pub pull: f64,
}

/// The anchor's state in plain words, for the panel and the boat's menu:
/// `vehicle.anchor.<state>` in the locale file ("Holding").
pub fn word(anchor: Option<&Anchor>) -> Localized {
    tr(&format!("vehicle.anchor.{}", state(anchor)))
}

/// The same mid-sentence, `vehicle.anchor_lower.<state>` ("holding").
pub fn word_lower(anchor: Option<&Anchor>) -> Localized {
    tr(&format!("vehicle.anchor_lower.{}", state(anchor)))
}

/// The anchor's state's id, the key of its words.
pub fn state(anchor: Option<&Anchor>) -> &'static str {
    let Some(a) = anchor else {
        return "stowed";
    };
    match a.phase {
        Phase::Dropping { .. } => "dropping",
        Phase::Up => "coming_aboard",
        Phase::Down { weighing: true, .. } => "weighing",
        Phase::Down { set, .. } if set < 1.0 => "setting",
        Phase::Down { .. } if a.rode < a.target - 0.05 => "paying_out",
        Phase::Down { .. } if a.rode > a.target + 0.05 => "hauling_in",
        Phase::Down { .. } => "holding",
    }
}

impl Anchor {
    /// Let go from the bow (planet frame).
    pub fn drop_from(bow: DVec3) -> Self {
        Self {
            phase: Phase::Dropping { fallen_m: 0.3 },
            at: bow - bow.normalize() * 0.3,
            rode: 0.5,
            target: 0.5,
            pull: 0.0,
        }
    }

    /// An anchor read back from a mooring: on the bed, set and holding.
    pub fn holding(at: DVec3, rode: f64) -> Self {
        Self {
            phase: Phase::Down {
                set: 1.0,
                weighing: false,
            },
            at,
            rode,
            target: rode,
            pull: 0.0,
        }
    }

    /// Set and holding: what the panel calls "Anchored".
    pub fn holds(&self) -> bool {
        matches!(
            self.phase,
            Phase::Down {
                set,
                weighing: false
            } if set >= 1.0
        )
    }

    /// The rode a rope setting asks for with the bow `plumb_m` over the
    /// anchor: never shorter than straight up and down, never longer than
    /// the boat carries. `None` is "to the bed": straight up and down.
    pub fn wanted(rope_m: Option<f64>, plumb_m: f64, max_m: f64) -> f64 {
        let plumb = plumb_m.max(0.2) + 0.3;
        match rope_m {
            None => plumb,
            Some(rope) => rope.clamp(plumb, max_m.max(plumb)),
        }
    }

    /// One substep, `h` seconds: the fall, the setting, the veering and
    /// hauling, the lift. `bow` is the bow (planet frame), `ground` the bed
    /// under a point, `rope_m` the boat's rope setting. Returns true when it
    /// is back aboard.
    pub fn step(
        &mut self,
        spec: &AnchorSpec,
        h: f64,
        bow: DVec3,
        ground: &dyn Fn(DVec3) -> f64,
        rope_m: Option<f64>,
    ) -> bool {
        let max = spec.rode_max_m as f64;
        match self.phase {
            Phase::Dropping { fallen_m } => {
                let fallen = fallen_m + spec.drop_mps as f64 * h;
                let up = self.at.normalize();
                let bed = ground(self.at);
                let from = self.at.length() + (fallen - fallen_m);
                let r = from - fallen;
                self.rode = self.rode.max(self.at.distance(bow) + 0.3).min(max);
                if r <= bed {
                    // On the bed the rode stops paying out, unless a longer
                    // rope is set.
                    self.at = up * bed;
                    self.rode = self.rode.max(self.at.distance(bow) + 0.3).min(max);
                    self.target = match rope_m {
                        None => self.rode,
                        Some(_) => self.rode.max(Self::wanted(rope_m, self.plumb(bow), max)),
                    };
                    self.phase = Phase::Down {
                        set: 0.0,
                        weighing: false,
                    };
                } else {
                    self.at = up * r;
                    self.phase = Phase::Dropping { fallen_m: fallen };
                }
            }
            Phase::Down { set, weighing } => {
                if self.rode < self.target {
                    self.rode = (self.rode + spec.veer_mps as f64 * h).min(self.target);
                } else if self.rode > self.target {
                    self.rode = (self.rode - spec.haul_mps as f64 * h).max(self.target);
                }
                let set = (set + h / spec.set_s as f64).min(1.0);
                // Setting: a hard pull slides it toward the boat a little.
                if set < 1.0 && self.pull > spec.slide_n as f64 {
                    let up = self.at.normalize();
                    let toward = bow - self.at;
                    let toward = (toward - up * toward.dot(up)).normalize_or_zero();
                    let v = ((self.pull - spec.slide_n as f64) / 320.0).min(0.6);
                    let moved = self.at + toward * v * h;
                    self.at = moved.normalize() * ground(moved);
                }
                if weighing {
                    let plumb = self.plumb(bow);
                    self.rode = (self.rode - spec.haul_mps as f64 * h).max(plumb + 0.2);
                    self.target = self.rode;
                    if self.rode <= plumb + 0.45 {
                        self.phase = Phase::Up;
                        return false;
                    }
                }
                self.phase = Phase::Down { set, weighing };
            }
            Phase::Up => {
                let k = (3.0 * h).min(1.0);
                let up = bow.normalize();
                let lifted = self.at + up * spec.lift_mps as f64 * h;
                let flat = (bow - lifted) - up * (bow - lifted).dot(up);
                self.at = lifted + flat * k;
                self.rode = self.at.distance(bow).max(0.3);
                if (bow - self.at).dot(up) <= 0.3 {
                    return true;
                }
            }
        }
        false
    }

    /// How far the bow stands over the anchor, m.
    pub fn plumb(&self, bow: DVec3) -> f64 {
        bow.length() - self.at.length()
    }

    /// The rode as the boat's line while it holds anything: on the bed.
    pub fn line(&self) -> Option<super::Mooring> {
        matches!(self.phase, Phase::Down { .. }).then_some(super::Mooring {
            at: self.at,
            length: self.rode,
            anchored: true,
        })
    }

    /// Set the rope: veered or hauled toward what it asks, if on the bed.
    pub fn set_rope(&mut self, rope_m: Option<f64>, bow: DVec3, max_m: f64) {
        if let Phase::Down {
            weighing: false, ..
        } = self.phase
        {
            self.target = Self::wanted(rope_m, self.plumb(bow), max_m);
        }
    }
}
