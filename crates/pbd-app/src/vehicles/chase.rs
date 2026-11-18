//! Immediate chase placement in the body's frame, including terrain clearance.

use super::view::VehicleView;
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::vehicle::Craft;

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
    // A metre over the craft's reference point: every boat's chase view
    // aims there now the Tern's masthead is gone (`retire-tern`).
    let target = craft.reference_position() + (up * 1.0).as_dvec3();
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
}
