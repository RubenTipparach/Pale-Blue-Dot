//! Nonrotating translating reference frames. Frame velocities are expressed in
//! the global inertial axes. Render/physics positions become f32 only after this
//! f64 subtraction. Accelerating/rotating frames require additional inertial terms
//! and are intentionally not silently approximated here.

use glam::DVec3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KinematicState {
    pub position: DVec3,
    pub velocity: DVec3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocalFrame {
    pub origin: DVec3,
    pub velocity: DVec3,
}

impl LocalFrame {
    pub fn to_local(self, global: KinematicState) -> KinematicState {
        KinematicState {
            position: global.position - self.origin,
            velocity: global.velocity - self.velocity,
        }
    }

    pub fn to_global(self, local: KinematicState) -> KinematicState {
        KinematicState {
            position: local.position + self.origin,
            velocity: local.velocity + self.velocity,
        }
    }

    /// A rebase changes representation, never global momentum or position.
    pub fn rebase(self, local: KinematicState, destination: Self) -> KinematicState {
        destination.to_local(self.to_global(local))
    }

    pub fn advanced(self, dt: f64) -> Self {
        Self {
            origin: self.origin + self.velocity * dt,
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebase_at_astronomical_distance_preserves_position_and_velocity() {
        let old = LocalFrame {
            origin: DVec3::new(1e12, -2e12, 3e12),
            velocity: DVec3::new(800.0, 120.0, 40.0),
        };
        let new = LocalFrame {
            origin: old.origin + DVec3::new(512.0, -128.0, 32.0),
            velocity: DVec3::new(120.0, 25.0, -9.0),
        };
        let local = KinematicState {
            position: DVec3::new(0.125, 3.75, -13.0),
            velocity: DVec3::new(18.0, -2.0, 7.0),
        };
        let original = old.to_global(local);
        let rebased = old.rebase(local, new);
        assert_eq!(new.to_global(rebased), original);
        assert_eq!(new.rebase(rebased, old), local);
        assert_eq!(original.position.x - old.origin.x, 0.125);
    }

    #[test]
    fn moving_frame_and_local_motion_reconstruct_inertial_motion() {
        let frame = LocalFrame {
            origin: DVec3::splat(1e9),
            velocity: DVec3::X * 500.0,
        };
        let state = KinematicState {
            position: DVec3::Y * 20.0,
            velocity: DVec3::Z * 3.0,
        };
        let global = frame.to_global(state);
        let next = frame.advanced(2.0).to_global(KinematicState {
            position: state.position + state.velocity * 2.0,
            ..state
        });
        assert_eq!(next.position, global.position + global.velocity * 2.0);
    }
}
