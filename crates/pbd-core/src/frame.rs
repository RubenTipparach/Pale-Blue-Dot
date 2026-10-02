//! Reference frames that translate and turn. Frame velocities are expressed
//! in the global inertial axes. Render/physics positions become f32 only after
//! this f64 subtraction. A turning frame composes with the `ω × r` term
//! (`sail-the-cog` task 1.1); the fictitious forces of an accelerating or
//! turning frame are not modelled: a walker on a deck is carried by it, as a
//! floor in a game carries it (`sail-the-cog`, a non-goal).

use glam::{DQuat, DVec3};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KinematicState {
    pub position: DVec3,
    pub velocity: DVec3,
}

/// A frame: an origin moving at `velocity`, its axes turned by
/// `orientation` and turning at `angular_velocity` (radians a second about
/// the global axis it points along). A frame that does not turn has the
/// identity orientation and no angular velocity, and composes exactly as a
/// translation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalFrame {
    pub origin: DVec3,
    pub velocity: DVec3,
    pub orientation: DQuat,
    pub angular_velocity: DVec3,
}

impl Default for LocalFrame {
    fn default() -> Self {
        Self {
            origin: DVec3::ZERO,
            velocity: DVec3::ZERO,
            orientation: DQuat::IDENTITY,
            angular_velocity: DVec3::ZERO,
        }
    }
}

impl LocalFrame {
    /// A frame that only translates.
    pub fn moving(origin: DVec3, velocity: DVec3) -> Self {
        Self {
            origin,
            velocity,
            ..Self::default()
        }
    }

    pub fn to_local(self, global: KinematicState) -> KinematicState {
        let r = global.position - self.origin;
        let back = self.orientation.inverse();
        KinematicState {
            position: back * r,
            velocity: back * (global.velocity - self.velocity - self.angular_velocity.cross(r)),
        }
    }

    pub fn to_global(self, local: KinematicState) -> KinematicState {
        let r = self.orientation * local.position;
        KinematicState {
            position: r + self.origin,
            velocity: self.orientation * local.velocity
                + self.velocity
                + self.angular_velocity.cross(r),
        }
    }

    /// A rebase changes representation, never global momentum or position.
    pub fn rebase(self, local: KinematicState, destination: Self) -> KinematicState {
        destination.to_local(self.to_global(local))
    }

    /// The frame `dt` later, moving and turning as it is.
    pub fn advanced(self, dt: f64) -> Self {
        let turn = DQuat::from_scaled_axis(self.angular_velocity * dt);
        Self {
            origin: self.origin + self.velocity * dt,
            orientation: (turn * self.orientation).normalize(),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rebase_at_astronomical_distance_preserves_position_and_velocity() {
        let old = LocalFrame::moving(
            DVec3::new(1e12, -2e12, 3e12),
            DVec3::new(800.0, 120.0, 40.0),
        );
        let new = LocalFrame::moving(
            old.origin + DVec3::new(512.0, -128.0, 32.0),
            DVec3::new(120.0, 25.0, -9.0),
        );
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
        let frame = LocalFrame::moving(DVec3::splat(1e9), DVec3::X * 500.0);
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

    /// `sail-the-cog` task 1.1: a point 8 m from the origin of a frame
    /// turning at 0.3 rad/s moves at the frame's velocity, its own turned,
    /// and `ω × r`, which is what differencing its world position over a
    /// short step finds.
    #[test]
    fn a_point_on_a_turning_deck_moves_with_the_turn() {
        let frame = LocalFrame {
            origin: DVec3::new(2.0e6, -1.0e6, 3.5e6),
            velocity: DVec3::new(4.0, 0.5, -2.0),
            orientation: DQuat::from_rotation_y(0.7) * DQuat::from_rotation_x(0.2),
            angular_velocity: DVec3::new(0.05, 0.3, -0.02).normalize() * 0.3,
        };
        let local = KinematicState {
            position: DVec3::new(8.0, 0.0, 0.0),
            velocity: DVec3::new(0.0, 0.0, 1.2),
        };
        let now = frame.to_global(local);
        let r = frame.orientation * local.position;
        assert!(
            (now.velocity
                - (frame.velocity
                    + frame.orientation * local.velocity
                    + frame.angular_velocity.cross(r)))
            .length()
                < 1e-12
        );
        assert!(
            frame.angular_velocity.cross(r).length() > 2.0,
            "ω × r is in it"
        );
        let dt = 1e-4;
        let later = frame.advanced(dt).to_global(KinematicState {
            position: local.position + local.velocity * dt,
            ..local
        });
        let differenced = (later.position - now.position) / dt;
        assert!(
            (differenced - now.velocity).length() < 1e-3,
            "{differenced} against {}",
            now.velocity
        );
    }

    /// `sail-the-cog` task 1.1: a local position composed to the world and
    /// back through a turned, turning frame far from the origin returns to
    /// within a millimetre.
    #[test]
    fn a_position_composed_out_and_back_returns() {
        let frame = LocalFrame {
            origin: DVec3::new(1.0e9, 2.5e8, -7.0e8),
            velocity: DVec3::new(30.0, -4.0, 9.0),
            orientation: DQuat::from_euler(glam::EulerRot::YXZ, 1.1, -0.4, 0.25),
            angular_velocity: DVec3::new(0.0, 0.3, 0.1),
        };
        let local = KinematicState {
            position: DVec3::new(-6.5, 1.9, 2.25),
            velocity: DVec3::new(1.0, 0.0, -0.5),
        };
        let back = frame.to_local(frame.to_global(local));
        assert!((back.position - local.position).length() < 1e-3);
        assert!((back.velocity - local.velocity).length() < 1e-6);
    }
}
