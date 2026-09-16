//! Designer-authored circular on-rails ephemerides, preserving Tenebris's
//! analytical planet/moon model. Ships have NO orbit elements or patched conics.
//! Positions and velocities are sampled from absolute time, never accumulated.

use crate::frame::KinematicState;
use glam::{DQuat, DVec3};
use std::f64::consts::TAU;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RailsBodyKind {
    Planet,
    Moon,
    Station,
}

#[derive(Clone, Copy, Debug)]
pub struct CircularOrbit {
    radius: f64,
    period: f64,
    phase: f64,
    plane: DQuat,
}

impl CircularOrbit {
    pub fn new(radius: f64, period: f64, phase: f64, plane: DQuat) -> Result<Self, &'static str> {
        if !radius.is_finite()
            || radius <= 0.0
            || !period.is_finite()
            || period <= 0.0
            || !phase.is_finite()
            || !plane.is_finite()
            || plane.length_squared() < 1e-12
        {
            return Err("orbit radius/period must be positive finite and orientation nonzero");
        }
        Ok(Self {
            radius,
            period,
            phase: phase.rem_euclid(TAU),
            plane: plane.normalize(),
        })
    }

    pub fn sample(self, seconds: f64) -> KinematicState {
        assert!(seconds.is_finite());
        let angle =
            (seconds.rem_euclid(self.period) / self.period * TAU + self.phase).rem_euclid(TAU);
        let (s, c) = angle.sin_cos();
        KinematicState {
            position: self.plane * DVec3::new(c, 0.0, s) * self.radius,
            velocity: self.plane * DVec3::new(-s, 0.0, c) * (self.radius * TAU / self.period),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RailsBody {
    pub kind: RailsBodyKind,
    /// Parent index MUST precede this body; None means fixed system origin.
    pub parent: Option<usize>,
    pub orbit: CircularOrbit,
}

#[derive(Clone, Debug)]
pub struct Ephemeris {
    bodies: Vec<RailsBody>,
}

impl Ephemeris {
    pub fn new(bodies: Vec<RailsBody>) -> Result<Self, &'static str> {
        for (index, body) in bodies.iter().enumerate() {
            if body.parent.is_some_and(|parent| parent >= index) {
                return Err("ephemeris parents must precede children; cycles are invalid");
            }
        }
        Ok(Self { bodies })
    }

    pub fn bodies(&self) -> &[RailsBody] {
        &self.bodies
    }

    pub fn sample(&self, seconds: f64) -> Vec<KinematicState> {
        let mut states: Vec<KinematicState> = Vec::with_capacity(self.bodies.len());
        self.sample_into(seconds, &mut states);
        states
    }

    /// Reuse the caller's allocation during fixed-tick sampling.
    pub fn sample_into(&self, seconds: f64, states: &mut Vec<KinematicState>) {
        states.clear();
        states.reserve(self.bodies.len());
        for body in &self.bodies {
            let mut state = body.orbit.sample(seconds);
            if let Some(parent) = body.parent {
                state.position += states[parent].position;
                state.velocity += states[parent].velocity;
            }
            states.push(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circular_orbit_radius_period_and_tangent_velocity_do_not_drift() {
        let orbit = CircularOrbit::new(30_000.0, 600.0, 0.4, DQuat::from_rotation_x(0.3)).unwrap();
        for t in [0.0, 1.0, 301.0, 10_000_000.0, -200.0] {
            let state = orbit.sample(t);
            assert!((state.position.length() - 30_000.0).abs() < 1e-8);
            assert!(state.position.dot(state.velocity).abs() < 1e-8);
            assert!((state.position - orbit.sample(t + 600.0).position).length() < 1e-8);
            let numerical_velocity =
                (orbit.sample(t + 0.001).position - orbit.sample(t - 0.001).position) / 0.002;
            assert!((numerical_velocity - state.velocity).length() < 1e-3);
        }
    }

    #[test]
    fn child_inherits_parent_position_and_velocity() {
        let planet = CircularOrbit::new(1e8, 100_000.0, 0.0, DQuat::IDENTITY).unwrap();
        let moon = CircularOrbit::new(20_000.0, 500.0, 0.0, DQuat::IDENTITY).unwrap();
        let ephemeris = Ephemeris::new(vec![
            RailsBody {
                kind: RailsBodyKind::Planet,
                parent: None,
                orbit: planet,
            },
            RailsBody {
                kind: RailsBodyKind::Moon,
                parent: Some(0),
                orbit: moon,
            },
        ])
        .unwrap();
        let states = ephemeris.sample(37.0);
        assert!(
            (states[1].position - states[0].position - moon.sample(37.0).position).length() < 1e-7
        );
        assert!(
            (states[1].velocity - states[0].velocity - moon.sample(37.0).velocity).length() < 1e-10
        );
        assert!(
            Ephemeris::new(vec![RailsBody {
                kind: RailsBodyKind::Station,
                parent: Some(0),
                orbit: moon
            }])
            .is_err()
        );
    }
}
