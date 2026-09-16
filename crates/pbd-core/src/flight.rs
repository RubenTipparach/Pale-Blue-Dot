//! Bounded flight controls, not orbital rocket mechanics.
//! This module outputs accelerations only. Avian owns integration and collisions.

use glam::{DQuat, DVec3};

#[derive(Clone, Copy, Debug)]
pub struct GravityWell {
    pub center: DVec3,
    pub radius: f64,
    pub surface_acceleration: f64,
}

impl GravityWell {
    /// Inverse-square exterior falloff; a uniform-density interior makes the
    /// acceleration continuous at the surface and zero at the centre.
    pub fn acceleration_at(self, position: DVec3) -> DVec3 {
        assert!(self.radius.is_finite() && self.radius > 0.0);
        assert!(self.surface_acceleration.is_finite() && self.surface_acceleration >= 0.0);
        let offset = self.center - position;
        let distance = offset.length();
        if distance < 1e-9 {
            return DVec3::ZERO;
        }
        let falloff = if distance >= self.radius {
            (self.radius / distance).powi(2)
        } else {
            distance / self.radius
        };
        offset / distance * self.surface_acceleration * falloff
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FlightLimits {
    /// Desired frame-relative mode speed; changing this does not snap momentum.
    pub speed: f64,
    pub acceleration: f64,
    pub angular_speed: f64,
    pub angular_acceleration: f64,
    pub linear_damping: f64,
    pub angular_damping: f64,
}

impl Default for FlightLimits {
    fn default() -> Self {
        Self {
            speed: 120.0,
            acceleration: 20.0,
            angular_speed: 1.5,
            angular_acceleration: 3.0,
            linear_damping: 1.0,
            angular_damping: 4.0,
        }
    }
}

impl FlightLimits {
    pub fn validate(self) -> Result<(), &'static str> {
        let positive = [
            self.speed,
            self.acceleration,
            self.angular_speed,
            self.angular_acceleration,
        ];
        let nonnegative = [self.linear_damping, self.angular_damping];
        if !positive.iter().all(|v| v.is_finite() && *v > 0.0)
            || !nonnegative.iter().all(|v| v.is_finite() && *v >= 0.0)
        {
            return Err("flight limits must be finite and positive; damping may be zero");
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FlightInput {
    /// Body-local axes, normalised to a unit ball (diagonals do not gain thrust).
    pub thrust: DVec3,
    pub rotation: DVec3,
    pub inertial_dampeners: bool,
    pub rotational_dampeners: bool,
}

impl Default for FlightInput {
    fn default() -> Self {
        Self {
            thrust: DVec3::ZERO,
            rotation: DVec3::ZERO,
            inertial_dampeners: true,
            rotational_dampeners: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FlightMotion {
    /// Quaternion orientation of the ship in the local nonrotating frame.
    pub orientation: DQuat,
    pub velocity: DVec3,
    pub angular_velocity: DVec3,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct FlightAcceleration {
    pub linear: DVec3,
    pub angular: DVec3,
}

/// Project a reachable candidate velocity onto the desired speed sphere.
/// Starting inside that sphere preserves BOTH acceleration and speed bounds.
/// Starting outside (new mode, collision, f32 rounding) preserves steering when
/// the speed sphere is reachable. Otherwise full bounded braking is required.
fn bounded_acceleration(
    velocity: DVec3,
    requested: DVec3,
    max_acceleration: f64,
    max_speed: f64,
    dt: f64,
) -> DVec3 {
    let speed = velocity.length();
    let maximum_step = max_acceleration * dt;
    if speed > max_speed + maximum_step {
        // A real mode change can put the speed target outside this tick's
        // reachable velocities. Brake without snapping the existing momentum.
        return -velocity / speed * max_acceleration;
    }
    let candidate = velocity + requested.clamp_length_max(max_acceleration) * dt;
    let mut target = candidate.clamp_length_max(max_speed);
    if speed > max_speed && target.distance_squared(velocity) > maximum_step * maximum_step {
        // The two velocity balls intersect, but projecting onto the speed ball
        // alone would exceed the acceleration budget. Their intersection circle
        // retains as much requested steering as possible while meeting both.
        let axis = velocity / speed;
        let along = ((speed * speed + max_speed * max_speed - maximum_step * maximum_step)
            / (2.0 * speed))
            .clamp(-max_speed, max_speed);
        let lateral_radius = (max_speed * max_speed - along * along).max(0.0).sqrt();
        let lateral =
            (target - axis * target.dot(axis)).normalize_or(axis.any_orthonormal_vector());
        target = axis * along + lateral * lateral_radius;
    }
    ((target - velocity) / dt).clamp_length_max(max_acceleration)
}

/// Gravity, thrust and damping share ONE commanded acceleration budget. This is
/// intentionally assisted flight: sufficiently strong gravity is limited too.
/// Contacts and emergency safety clamps are solver impulses outside that budget.
pub fn control_acceleration(
    motion: FlightMotion,
    input: FlightInput,
    gravity: DVec3,
    limits: FlightLimits,
    dt: f64,
) -> FlightAcceleration {
    assert!(dt.is_finite() && dt > 0.0);
    assert!(limits.validate().is_ok());
    assert!(motion.velocity.is_finite() && motion.angular_velocity.is_finite());
    assert!(motion.orientation.is_finite() && motion.orientation.length_squared() > 1e-12);
    assert!(input.thrust.is_finite() && input.rotation.is_finite() && gravity.is_finite());
    let orientation = motion.orientation.normalize();
    let linear_damping = if input.inertial_dampeners {
        motion.velocity * limits.linear_damping
    } else {
        DVec3::ZERO
    };
    let angular_damping = if input.rotational_dampeners {
        motion.angular_velocity * limits.angular_damping
    } else {
        DVec3::ZERO
    };
    let linear_request = orientation * input.thrust.clamp_length_max(1.0) * limits.acceleration
        + gravity
        - linear_damping;
    let angular_request =
        orientation * input.rotation.clamp_length_max(1.0) * limits.angular_acceleration
            - angular_damping;
    FlightAcceleration {
        linear: bounded_acceleration(
            motion.velocity,
            linear_request,
            limits.acceleration,
            limits.speed,
            dt,
        ),
        angular: bounded_acceleration(
            motion.angular_velocity,
            angular_request,
            limits.angular_acceleration,
            limits.angular_speed,
            dt,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravity_falls_to_quarter_at_twice_radius_and_is_finite_inside() {
        let well = GravityWell {
            center: DVec3::ZERO,
            radius: 4_000.0,
            surface_acceleration: 9.0,
        };
        assert_eq!(well.acceleration_at(DVec3::Y * 4_000.0), DVec3::NEG_Y * 9.0);
        assert_eq!(
            well.acceleration_at(DVec3::Y * 8_000.0),
            DVec3::NEG_Y * 2.25
        );
        assert_eq!(well.acceleration_at(DVec3::ZERO), DVec3::ZERO);
        assert_eq!(well.acceleration_at(DVec3::Y * 2_000.0), DVec3::NEG_Y * 4.5);
    }

    #[test]
    fn sustained_diagonal_thrust_and_gravity_respect_all_four_limits() {
        let limits = FlightLimits::default();
        let input = FlightInput {
            thrust: DVec3::splat(1.0),
            rotation: DVec3::splat(1.0),
            inertial_dampeners: false,
            rotational_dampeners: false,
        };
        let mut motion = FlightMotion {
            orientation: DQuat::from_rotation_y(0.7),
            ..Default::default()
        };
        let dt = 1.0 / 60.0;
        for _ in 0..12_000 {
            let command = control_acceleration(motion, input, DVec3::NEG_Y * 30.0, limits, dt);
            assert!(command.linear.length() <= limits.acceleration + 1e-9);
            assert!(command.angular.length() <= limits.angular_acceleration + 1e-9);
            // This test integrates only velocity to verify the controller contract.
            // Production position/orientation integration happens ONLY in Avian.
            motion.velocity += command.linear * dt;
            motion.angular_velocity += command.angular * dt;
            assert!(motion.velocity.length() <= limits.speed + 1e-9);
            assert!(motion.angular_velocity.length() <= limits.angular_speed + 1e-9);
        }
    }

    #[test]
    fn mode_change_brakes_without_velocity_snap_or_acceleration_spike() {
        let limits = FlightLimits {
            speed: 15.0,
            ..Default::default()
        };
        let mut motion = FlightMotion {
            velocity: DVec3::X * 600.0,
            ..Default::default()
        };
        let dt = 1.0 / 60.0;
        let command = control_acceleration(motion, FlightInput::default(), DVec3::ZERO, limits, dt);
        assert_eq!(command.linear, DVec3::NEG_X * 20.0);
        motion.velocity += command.linear * dt;
        assert!(motion.velocity.x > 599.0);
        for _ in 0..1_800 {
            let command =
                control_acceleration(motion, FlightInput::default(), DVec3::ZERO, limits, dt);
            motion.velocity += command.linear * dt;
        }
        assert!(motion.velocity.length() < limits.speed);
    }

    #[test]
    fn f32_rounding_at_the_speed_limit_does_not_disable_centripetal_steering() {
        // A valid local f32 velocity can exceed the f64 limit by a few micrometres/s.
        let velocity = (DVec3::new(0.3, 0.8, 0.7).normalize() * 600.0)
            .as_vec3()
            .as_dvec3();
        assert!(velocity.length() > 600.0);
        let turn = velocity.normalize().any_orthonormal_vector();
        let acceleration = bounded_acceleration(velocity, turn * 72.0, 80.0, 600.0, 1.0 / 60.0);
        assert!(
            acceleration.dot(turn) > 71.9,
            "steering was dropped: {acceleration:?}"
        );
        assert!(acceleration.length() <= 80.0 + 1e-9);
        assert!((velocity + acceleration / 60.0).length() <= 600.0 + 1e-9);
    }

    #[test]
    fn reachable_overspeed_keeps_steering_inside_both_velocity_balls() {
        let velocity = DVec3::X * 600.5;
        let acceleration = bounded_acceleration(velocity, DVec3::Y * 80.0, 80.0, 600.0, 1.0 / 60.0);
        assert!(acceleration.x < 0.0 && acceleration.y > 70.0);
        assert!(acceleration.length() <= 80.0 + 1e-9);
        assert!((velocity + acceleration / 60.0).length() <= 600.0 + 1e-9);
    }

    #[test]
    fn dampeners_remove_translation_and_rotation_and_can_be_disabled() {
        let limits = FlightLimits::default();
        let initial = FlightMotion {
            velocity: DVec3::X * 10.0,
            angular_velocity: DVec3::Y,
            ..Default::default()
        };
        let off = FlightInput {
            inertial_dampeners: false,
            rotational_dampeners: false,
            ..Default::default()
        };
        let command = control_acceleration(initial, off, DVec3::ZERO, limits, 0.01);
        assert_eq!(command.linear, DVec3::ZERO);
        assert_eq!(command.angular, DVec3::ZERO);
        let mut motion = initial;
        for _ in 0..1_000 {
            let command =
                control_acceleration(motion, FlightInput::default(), DVec3::ZERO, limits, 0.01);
            motion.velocity += command.linear * 0.01;
            motion.angular_velocity += command.angular * 0.01;
        }
        assert!(motion.velocity.length() < 0.001);
        assert!(motion.angular_velocity.length() < 0.001);
    }

    #[test]
    fn orientation_rotates_local_thrust_into_frame_axes() {
        let motion = FlightMotion {
            orientation: DQuat::from_rotation_y(std::f64::consts::FRAC_PI_2),
            ..Default::default()
        };
        let input = FlightInput {
            thrust: DVec3::Z,
            ..Default::default()
        };
        let command =
            control_acceleration(motion, input, DVec3::ZERO, FlightLimits::default(), 0.01);
        assert!((command.linear - DVec3::X * 20.0).length() < 1e-10);
    }
}
