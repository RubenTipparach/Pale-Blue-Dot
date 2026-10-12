//! Great-circle velocity/altitude guidance and shortest-path quaternion control.
//! All outputs are requests to ShipController; this module never integrates poses.

use bevy::prelude::*;

use super::FlightViewConfig;
use crate::planet::PLANET_RADIUS;

pub(super) fn tour_command(
    position: Vec3,
    velocity: Vec3,
    plane_normal: Vec3,
    config: FlightViewConfig,
) -> (Vec3, Quat) {
    let radius = position.length().max(1.0);
    let up = position / radius;
    let tangent = plane_normal.cross(up).normalize_or(Vec3::X);
    let radial_speed = velocity.dot(up);
    let tangential_speed = velocity.dot(tangent);
    let height_error = radius - (PLANET_RADIUS + config.tour_altitude);
    // Feed-forward centripetal acceleration plus a damped altitude correction.
    let radial = -tangential_speed.powi(2) / radius - height_error * 0.35 - radial_speed * 1.8;
    let radial = radial.clamp(-config.acceleration * 0.96, config.acceleration * 0.96);
    let tangent_budget = (config.acceleration.powi(2) - radial.powi(2))
        .max(0.0)
        .sqrt();
    let tangential =
        ((config.cruise_speed - tangential_speed) * 0.9).clamp(-tangent_budget, tangent_budget);
    let plane_error = position.dot(plane_normal);
    let plane_correction = -plane_error * 0.3 - velocity.dot(plane_normal) * 1.5;
    let acceleration = (up * radial + tangent * tangential + plane_normal * plane_correction)
        .clamp_length_max(config.acceleration);
    let view_direction = tangent * config.view_pitch_down.cos() - up * config.view_pitch_down.sin();
    let rotation = Transform::IDENTITY.looking_to(view_direction, up).rotation;
    (acceleration, rotation)
}

pub(super) fn attitude_acceleration(
    current: Quat,
    target: Quat,
    angular_velocity: Vec3,
    maximum_speed: f32,
    maximum_acceleration: f32,
) -> Vec3 {
    let mut error = (target * current.inverse()).normalize();
    if error.w < 0.0 {
        error = -error;
    }
    let desired_velocity = (error.to_scaled_axis() * 4.0).clamp_length_max(maximum_speed);
    ((desired_velocity - angular_velocity) * 7.0).clamp_length_max(maximum_acceleration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn great_circle_guidance_supplies_centripetal_acceleration() {
        let config = FlightViewConfig::default();
        let radius = PLANET_RADIUS + config.tour_altitude;
        let (acceleration, orientation) = tour_command(
            Vec3::Y * radius,
            Vec3::X * config.cruise_speed,
            Vec3::NEG_Z,
            config,
        );
        let centripetal = -config.cruise_speed.powi(2) / radius;
        assert!((acceleration.y - centripetal).abs() < 0.001);
        assert!(acceleration.x.abs() < 0.001);
        assert!((orientation.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn equivalent_quaternion_signs_request_the_same_bounded_rotation() {
        let current = Quat::from_rotation_x(1.7);
        let target = Quat::from_rotation_y(2.9) * Quat::from_rotation_z(0.7);
        let a = attitude_acceleration(current, target, Vec3::ZERO, 1.5, 3.0);
        let b = attitude_acceleration(current, -target, Vec3::ZERO, 1.5, 3.0);
        assert!((a - b).length() < 1e-5);
        assert!(a.length() <= 3.0001);
    }
}
