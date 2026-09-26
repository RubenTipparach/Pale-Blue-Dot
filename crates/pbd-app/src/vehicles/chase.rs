//! Immediate chase placement in the body's frame, including terrain clearance.

use super::view::VehicleView;
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::vehicle::{Craft, Kind};

const EYE_CLEARANCE_M: f64 = 0.25;
const BOOM_SAMPLE_M: f64 = 0.5;

/// Eye and target in body-local metres, and the local up direction.
pub(super) fn pose(
    craft: &Craft,
    view: &VehicleView,
    ground: impl Fn(DVec3) -> f64,
) -> (DVec3, DVec3, Vec3) {
    let up = craft.body.position.normalize_or(DVec3::Y).as_vec3();
    let bow = craft.body.orientation.as_quat() * Vec3::NEG_Z;
    let heading = (bow - up * bow.dot(up)).normalize_or(up.any_orthonormal_vector());
    let look = Quat::from_axis_angle(up, view.yaw) * heading;
    let right = look.cross(up).normalize_or(Vec3::X);
    let forward = Quat::from_axis_angle(right, view.pitch) * look;
    let aim_height = if craft.kind == Kind::Tern {
        let sail = &craft.specs().tern.sail;
        (sail.mast[1] + sail.head_height_m) * 0.5
    } else {
        1.0
    };
    let target = craft.reference_position() + (up * aim_height).as_dvec3();
    let desired = target + (-forward * view.distance + up * (view.distance * 0.12)).as_dvec3();
    (clear_boom(target, desired, ground), target, up)
}

/// Shorten an obstructed boom without modifying the requested zoom or easing look.
fn clear_boom(target: DVec3, desired: DVec3, ground: impl Fn(DVec3) -> f64) -> DVec3 {
    let up = target.normalize_or(DVec3::Y);
    let start = up * target.length().max(ground(target) + EYE_CLEARANCE_M);
    let steps = ((desired - start).length() / BOOM_SAMPLE_M).ceil().max(1.0) as usize;
    let mut clear = start;
    for step in 1..=steps {
        let at = start.lerp(desired, step as f64 / steps as f64);
        if at.length() < ground(at) + EYE_CLEARANCE_M {
            break;
        }
        clear = at;
    }
    clear
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::vehicle::{Hulls, spec::VehicleSpecs};
    use std::sync::Arc;

    #[test]
    fn a_shore_shortens_the_boom_and_clear_ground_restores_it() {
        let target = DVec3::new(0.0, 102.0, 0.0);
        let desired = DVec3::new(0.0, 103.0, 8.0);
        let shore = |at: DVec3| if at.z >= 3.0 { 106.0 } else { 100.0 };
        let eye = clear_boom(target, desired, shore);
        assert!(eye.z > 2.0 && eye.z < 3.0);
        assert!(eye.length() >= shore(eye) + EYE_CLEARANCE_M);
        assert_eq!(clear_boom(target, desired, |_| 100.0), desired);
        let buried_target = DVec3::Y * 99.0;
        assert!(clear_boom(buried_target, desired, |_| 100.0).length() >= 100.25);
    }

    #[test]
    fn the_default_tern_view_contains_the_masthead_and_hull() {
        let specs = Arc::new(VehicleSpecs::default());
        let craft = Craft::new(
            Kind::Tern,
            1,
            specs.clone(),
            Hulls::new(&specs),
            DVec3::Y * 4800.0,
            pbd_core::DQuat::IDENTITY,
        );
        let view = VehicleView {
            distance: super::super::view::default_distance(Kind::Tern),
            ..default()
        };
        let (eye, target, up) = pose(&craft, &view, |_| 4750.0);
        let camera = Transform::from_translation((eye - craft.reference_position()).as_vec3())
            .looking_at((target - craft.reference_position()).as_vec3(), up);
        let projection = PerspectiveProjection::default();
        let sail = &specs.tern.sail;
        let mut points = craft.hull().unwrap().loft(28, 10).0;
        points.push(Vec3::from(sail.mast) + Vec3::Y * sail.head_height_m);
        for point in points {
            let in_view = camera.rotation.inverse() * (point - camera.translation);
            assert!(in_view.z < -projection.near);
            let vertical = in_view.y.abs() / -in_view.z;
            assert!(
                vertical < (projection.fov * 0.5).tan(),
                "point {point:?} is cropped"
            );
        }
        assert_eq!(
            view.distance,
            super::super::view::default_distance(Kind::Tern)
        );
    }
}
