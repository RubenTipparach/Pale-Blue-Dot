//! The player's own figure in a craft's seat (`player-character` design 8,
//! task 4.1): the townsfolk's figure in the player's look, seated where the
//! craft says, its hands aimed at the controls. Drawn only from the chase
//! view, so first person stays hands only (design 4).
//!
//! The rower: hips on the raft's bench, facing the bow, legs out to the
//! beam, leaning with the stroke, each hand on its oar's grip.

use super::view::VehicleView;
use super::{Aboard, Vehicle};
use crate::character::PlayerLook;
use crate::townsfolk::{Posture, Ride, Townsfolk};
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::vehicle::Craft;

/// The visitor number the player's own seated figure stands as: no host
/// gives it to a player, and the Character page's preview is another.
pub const RIDER: u32 = u32::MAX - 2;

/// The rower's legs, rad: out to the beam (M40's `legs(-1.25, 0.14)`).
const ROWER_LEGS: f32 = -1.25;
const ROWER_SPREAD: f32 = 0.14;

/// A vector from the craft's space (x right, y up, z aft) into a figure's
/// that faces the bow (x across, y up, z forward): the bow is the figure's
/// forward, so x and z turn round.
fn bow_facing(v: DVec3) -> Vec3 {
    Vec3::new(-v.x as f32, v.y as f32, -v.z as f32)
}

/// The rower's seat and pose on `craft`, if it is a rowing craft: the hips
/// in the planet's frame, the way the figure faces, and the pose.
pub fn rower(craft: &Craft) -> Option<(DVec3, DVec3, Ride)> {
    let (spec, state) = craft.rowing()?;
    let seat = DVec3::from(spec.rower.seat.map(f64::from));
    let orient = craft.body.orientation;
    let hips = craft.reference_position() + orient * seat;
    let forward = orient * DVec3::NEG_Z;
    let hand = |i: usize| bow_facing(state.oars[i].geom.grip - seat);
    // The rower faces the bow, so the left hand (the figure's own -x) takes
    // the starboard oar (the craft's +x).
    let ride = Ride {
        legs: [ROWER_LEGS; 2],
        spread: ROWER_SPREAD,
        lean: state.lean(spec) as f32,
        hands: [Some(hand(1)), Some(hand(0))],
    };
    Some((hips, forward, ride))
}

/// How another player sits in a seat their game names (`player-character`
/// design 8): the mockup's seats, the hands where the controls are. Their
/// own game does not send the craft's live controls, so the hands are held
/// at rest on them. `None` standing.
pub fn seat_pose(seat: pbd_core::net::protocol::Seat) -> Option<Ride> {
    use pbd_core::net::protocol::Seat;
    let both = |x: f32, y: f32, z: f32| [Some(Vec3::new(-x, y, z)), Some(Vec3::new(x, y, z))];
    Some(match seat {
        Seat::Standing => return None,
        // At the oars, the grips in front at the catch.
        Seat::Rower => Ride {
            legs: [ROWER_LEGS; 2],
            spread: ROWER_SPREAD,
            lean: 0.1,
            hands: both(0.24, 0.25, 0.42),
        },
        // On a cart's bench, the reins in the lap.
        Seat::Bench => Ride {
            legs: [-1.42; 2],
            spread: 0.06,
            lean: 0.08,
            hands: both(0.15, 0.22, 0.4),
        },
        // Astride, the reins over the neck (M40's `legs(-0.32, 0.78)`).
        Seat::Mount => Ride {
            legs: [-0.32; 2],
            spread: 0.78,
            lean: 0.05,
            hands: both(0.12, 0.3, 0.42),
        },
    })
}

/// Seat the player's figure in the craft they are aboard while the view is
/// from outside; stand it down otherwise.
pub fn seat(
    aboard: Res<Aboard>,
    view: Res<VehicleView>,
    crafts: Query<&Vehicle>,
    look: Option<Res<PlayerLook>>,
    folk: Option<ResMut<Townsfolk>>,
    mut seated: Local<bool>,
) {
    let (Some(mut folk), Some(look)) = (folk, look) else {
        return;
    };
    let place = aboard
        .0
        .filter(|_| !view.seat)
        .and_then(|e| crafts.get(e).ok())
        .filter(|v| v.craft.occupied)
        .and_then(|v| rower(&v.craft));
    let Some((hips, forward, ride)) = place else {
        if std::mem::take(&mut *seated) {
            folk.unvisit(RIDER);
        }
        return;
    };
    folk.visit(RIDER, hips, forward, false, Some(look.0), look.0.shirt);
    let id = crate::townsfolk::player_id(RIDER);
    folk.rest(id, hips, forward, Posture::Ride(ride));
    *seated = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Another player seated anywhere but standing is drawn in a pose.
    #[test]
    fn every_seat_but_standing_has_a_pose() {
        use pbd_core::net::protocol::Seat;
        for seat in Seat::ALL {
            assert_eq!(seat_pose(seat).is_some(), seat != Seat::Standing);
        }
    }

    /// The craft's right is the bow-facing figure's left, and the bow is
    /// its forward.
    #[test]
    fn the_bow_is_the_rowers_forward() {
        assert_eq!(bow_facing(DVec3::NEG_Z), Vec3::Z);
        assert_eq!(bow_facing(DVec3::X), Vec3::NEG_X);
        assert_eq!(bow_facing(DVec3::Y), Vec3::Y);
    }
}
