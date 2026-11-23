//! Riding a kept plodder or horse (`land-transport` design 14).
//!
//! A ride is kinematic, as a cart is: a body-local `f64` point on the
//! ground and a heading, eased toward the asked speed and stood on the
//! ground the app's query gives. An animal is not wheeled, so the roads'
//! wheel factors do not apply; a rise of more than `max_step_m` between two
//! steps refuses, as a wall does.

use super::cart::Drive;
use glam::DVec3;
use serde::{Deserialize, Serialize};

/// How one species is ridden.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RideSpec {
    /// Ridden at a walk, m/s.
    pub walk_mps: f32,
    /// Ridden at a run (Shift), m/s.
    pub run_mps: f32,
    /// How fast it comes up to speed and slows, m/s^2.
    pub accel_mps2: f32,
    /// How fast it turns, rad/s.
    pub turn_rad_s: f32,
    /// The most the ground may rise between two steps, m.
    pub max_step_m: f32,
    /// The rider's eye over the ground where it stands, m.
    pub eye_m: f32,
}

impl RideSpec {
    fn validate(&self, at: &str) -> Result<(), String> {
        let all = [
            self.walk_mps,
            self.run_mps,
            self.accel_mps2,
            self.turn_rad_s,
            self.max_step_m,
            self.eye_m,
        ];
        if all.iter().any(|v| !(v.is_finite() && *v > 0.0)) {
            return Err(format!("{at}: every number is positive"));
        }
        if self.run_mps < self.walk_mps {
            return Err(format!("{at}: a run is no slower than a walk"));
        }
        Ok(())
    }
}

/// The ridden species' specs (`vehicles.ron`, `carts.riding`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RideSpecs {
    pub plodder: RideSpec,
    pub horse: RideSpec,
}

impl Default for RideSpecs {
    /// Design 14's numbers: a plodder walks 3 and runs 6 m/s, a horse walks
    /// 4 and runs 12.
    fn default() -> Self {
        Self {
            plodder: RideSpec {
                walk_mps: 3.0,
                run_mps: 6.0,
                accel_mps2: 3.0,
                turn_rad_s: 1.6,
                max_step_m: 1.1,
                eye_m: 2.15,
            },
            horse: RideSpec {
                walk_mps: 4.0,
                run_mps: 12.0,
                accel_mps2: 3.0,
                turn_rad_s: 1.6,
                max_step_m: 1.1,
                eye_m: 2.35,
            },
        }
    }
}

impl RideSpecs {
    /// The spec of the species `id`, if it can be ridden.
    pub fn of(&self, id: &str) -> Option<&RideSpec> {
        match id {
            "plodder" => Some(&self.plodder),
            "horse" => Some(&self.horse),
            _ => None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.plodder.validate("carts.riding.plodder")?;
        self.horse.validate("carts.riding.horse")
    }
}

/// An animal being ridden.
#[derive(Clone, Debug, PartialEq)]
pub struct Ride {
    /// The animal's record id.
    pub animal: u64,
    /// Its feet, body-local, m.
    pub at: DVec3,
    /// The unit tangent it faces.
    pub heading: DVec3,
    /// Its speed along its heading, m/s.
    pub speed: f64,
    /// How far it has gone, m: what its gait is worked from.
    pub walked_m: f64,
}

fn tangent(v: DVec3, up: DVec3, fallback: DVec3) -> DVec3 {
    let t = v - up * v.dot(up);
    t.try_normalize().unwrap_or_else(|| {
        let f = fallback - up * fallback.dot(up);
        f.try_normalize().unwrap_or(up.any_orthonormal_vector())
    })
}

impl Ride {
    pub fn new(animal: u64, at: DVec3, facing: DVec3) -> Self {
        let up = at.normalize_or(DVec3::Y);
        Self {
            animal,
            at,
            heading: tangent(facing, up, DVec3::X),
            speed: 0.0,
            walked_m: 0.0,
        }
    }

    /// Up where it stands.
    pub fn up(&self) -> DVec3 {
        self.at.normalize_or(DVec3::Y)
    }

    /// Ridden for `dt` seconds: the speed eased toward the ask (a walk, or
    /// a run with `run`), turned, then moved along the ground `ground`
    /// gives (a radius under a body-local point). A rise of more than
    /// `max_step_m` refuses, and it stands. Metres moved.
    pub fn step(
        &mut self,
        spec: &RideSpec,
        input: Drive,
        run: bool,
        dt: f64,
        ground: &dyn Fn(DVec3) -> f64,
    ) -> f64 {
        if !(dt.is_finite() && dt > 0.0) {
            return 0.0;
        }
        let top = f64::from(if run { spec.run_mps } else { spec.walk_mps });
        let forward = f64::from(input.forward.clamp(-1.0, 1.0));
        // Reined back at a third of a walk.
        let want = if forward >= 0.0 {
            forward * top
        } else {
            forward * f64::from(spec.walk_mps) / 3.0
        };
        let step = f64::from(spec.accel_mps2) * dt;
        self.speed += (want - self.speed).clamp(-step, step);
        let turn = f64::from(input.turn.clamp(-1.0, 1.0) * spec.turn_rad_s) * dt;
        if turn != 0.0 {
            let q = glam::DQuat::from_axis_angle(self.up(), turn);
            self.heading = tangent(q * self.heading, self.up(), self.heading);
        }
        let next = self.at + self.heading * (self.speed * dt);
        let (here, there) = (ground(self.at), ground(next));
        if !(here.is_finite() && there.is_finite()) || there - here > f64::from(spec.max_step_m) {
            self.speed = 0.0;
            return 0.0;
        }
        let up = next.normalize_or(DVec3::Y);
        let stood = up * there;
        let moved = (stood - self.at).length();
        self.at = stood;
        self.heading = tangent(self.heading, up, self.heading);
        self.walked_m += moved;
        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f64 = 4800.0;

    fn flat(_: DVec3) -> f64 {
        R
    }

    fn ride() -> Ride {
        Ride::new(7, DVec3::Y * R, DVec3::X)
    }

    /// A plodder walks and runs at its own speeds, slower than a horse.
    #[test]
    fn a_ride_reaches_its_speeds() {
        let specs = RideSpecs::default();
        let go = Drive {
            forward: 1.0,
            turn: 0.0,
        };
        for (spec, run, want) in [
            (specs.plodder, false, 3.0),
            (specs.plodder, true, 6.0),
            (specs.horse, true, 12.0),
        ] {
            let mut r = ride();
            for _ in 0..600 {
                r.step(&spec, go, run, 1.0 / 60.0, &flat);
            }
            assert!((r.speed - want).abs() < 1e-6, "{} against {want}", r.speed);
            assert!((r.at.length() - R).abs() < 1e-6, "it stays on the ground");
        }
        assert!(specs.plodder.run_mps < specs.horse.run_mps);
        assert!(specs.validate().is_ok());
    }

    /// A rise of more than a layer and a little refuses; it turns as asked.
    #[test]
    fn a_ride_refuses_a_wall_and_turns() {
        let spec = RideSpecs::default().plodder;
        let wall = |p: DVec3| if p.x > 1.0 { R + 2.0 } else { R };
        let mut r = ride();
        for _ in 0..600 {
            r.step(
                &spec,
                Drive {
                    forward: 1.0,
                    turn: 0.0,
                },
                true,
                1.0 / 60.0,
                &wall,
            );
        }
        assert!(r.at.x <= 1.0 + 1e-9, "stopped at the wall: {}", r.at.x);
        let mut t = ride();
        t.step(
            &spec,
            Drive {
                forward: 0.0,
                turn: 1.0,
            },
            false,
            1.0,
            &flat,
        );
        let turned = DVec3::X.angle_between(t.heading);
        assert!(
            (turned - f64::from(spec.turn_rad_s)).abs() < 1e-6,
            "{turned}"
        );
    }
}
