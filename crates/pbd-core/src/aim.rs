//! What the player is pointing at: the cell and layer a view ray first meets,
//! and the empty one in front of it.
//!
//! Ported in shape from `tenebris-client`'s `interact.rs`, which marches the
//! eye ray in fixed steps, resolves each sample to a (tile, depth) with a warm
//! hint, stops at the first solid block, and remembers the last sample that
//! was not solid as the place target. That last part is the whole of placing:
//! a block goes where the ray WAS, not where it stopped.
//!
//! A fixed step rather than a grid walk, because the lattice is not a grid. A
//! hex cell has five or six neighbours and no axis to step along, so there is
//! no DDA to run; what the march needs is only that a step cannot skip a cell,
//! and a quarter metre against a one-metre layer and a 2.8 m tile cannot.
//!
//! The lookup is the caller's: this crate knows nothing about tiers or LOD
//! sets, so the app passes a closure that answers what stands at a point.

use glam::Vec3;

/// How far a player can reach, metres. Tenebris's `INTERACT_MAX_DIST` at our
/// scale: far enough to dig the wall in front of you and the ground under
/// your feet, short enough that you cannot reshape a hillside from a distance.
pub const REACH_M: f32 = 5.0;

/// The march's step, metres. A quarter of a layer, so no step can cross a
/// whole cell in any direction.
pub const STEP_M: f32 = 0.25;

/// What stands at a sampled point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    /// The cell's stable ID.
    pub cell: u32,
    /// The layer the point is in.
    pub layer: usize,
    /// Whether that layer stops a ray. Water does not, as it does not stop a
    /// walker: this is `Column::solid`.
    pub solid: bool,
}

/// A cell and layer to dig, and the empty cell and layer to place into.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    /// The first solid layer the ray met.
    pub dig: Sample,
    /// The last non-solid sample before it, which is where a placed block
    /// goes. `None` when the ray started inside the solid it hit, which is a
    /// player with their eye in the rock: there is nowhere in front of it.
    pub place: Option<Sample>,
    /// How far along the ray the solid was met, metres.
    pub distance: f32,
}

/// March the eye ray and answer what it hits.
///
/// `at` answers what stands at a world point, or `None` where there is no
/// column - outside the tier, or off the set. A gap in the lookup does not end
/// the march: the reference keeps stepping so a ray that clips the corner of
/// an unloaded region still finds the wall past it, and it does NOT count the
/// gap as the empty cell to place into, because nothing is known about it.
pub fn march(
    eye: Vec3,
    direction: Vec3,
    mut at: impl FnMut(Vec3) -> Option<Sample>,
) -> Option<Target> {
    let direction = direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        return None;
    }
    let mut place: Option<Sample> = None;
    let mut travelled = 0.0;
    while travelled <= REACH_M {
        let point = eye + direction * travelled;
        if let Some(sample) = at(point) {
            if sample.solid {
                return Some(Target {
                    dig: sample,
                    place,
                    distance: travelled,
                });
            }
            place = Some(sample);
        }
        travelled += STEP_M;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat floor of solid layers under an empty sky, in one cell: the
    /// simplest world a march can be checked against.
    fn floor_at(ground: f32) -> impl FnMut(Vec3) -> Option<Sample> {
        move |point: Vec3| {
            let altitude = point.z;
            let layer = (altitude.floor() + 200.0) as usize;
            Some(Sample {
                cell: 1,
                layer,
                solid: altitude < ground,
            })
        }
    }

    #[test]
    fn a_ray_down_onto_the_ground_digs_the_top_layer_and_places_above_it() {
        let eye = Vec3::new(0.0, 0.0, 3.0);
        let target = march(eye, -Vec3::Z, floor_at(0.0)).expect("the ground is in reach");
        assert!(
            !target.dig.solid || target.dig.layer == 199,
            "{:?}",
            target.dig
        );
        assert_eq!(
            target.dig.layer, 199,
            "the metre under zero is the top solid"
        );
        let place = target.place.expect("the ray crossed air on the way down");
        assert_eq!(place.layer, 200, "and the metre above it takes the block");
        assert!(target.distance <= 3.25);
    }

    #[test]
    fn a_ray_into_the_sky_finds_nothing() {
        let eye = Vec3::new(0.0, 0.0, 3.0);
        assert!(march(eye, Vec3::Z, floor_at(0.0)).is_none());
    }

    #[test]
    fn a_ray_further_than_the_reach_finds_nothing() {
        // The ground is well past REACH_M below the eye.
        let eye = Vec3::new(0.0, 0.0, REACH_M + 2.0);
        assert!(march(eye, -Vec3::Z, floor_at(0.0)).is_none());
    }

    #[test]
    fn an_eye_inside_the_rock_has_nowhere_to_place() {
        let eye = Vec3::new(0.0, 0.0, -1.0);
        let target = march(eye, -Vec3::Z, floor_at(0.0)).expect("solid at once");
        assert!(target.place.is_none(), "nothing empty was crossed first");
        assert_eq!(target.distance, 0.0);
    }

    #[test]
    fn a_gap_in_the_lookup_is_not_somewhere_to_place() {
        // Air, then a hole in the lookup, then rock: the block must go in the
        // air it knew about, never in the gap it did not.
        let eye = Vec3::new(0.0, 0.0, 3.0);
        let target = march(eye, -Vec3::Z, |point| {
            let altitude = point.z;
            if (1.0..2.0).contains(&altitude) {
                return None;
            }
            Some(Sample {
                cell: 1,
                layer: (altitude.floor() + 200.0) as usize,
                solid: altitude < 0.0,
            })
        })
        .expect("the ground is in reach");
        assert_eq!(target.dig.layer, 199);
        assert_eq!(
            target.place.expect("the air below the gap").layer,
            200,
            "the last KNOWN empty layer, not the unknown one"
        );
    }
}
