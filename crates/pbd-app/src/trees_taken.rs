//! What players have taken from the trees near the camera, as the tree draw
//! reads it (`flora-v2` design 12.5): the trees felled (a `tree` record whose
//! wood is all given) and the trees picked bare (a `fruit` record younger than
//! the regrowth), each as the tree's id (the record's `spare.x`, which the
//! draw rolls on), nearest first, in two short lists of the planet uniform.
//!
//! The records are the save's; this only says which of them stand near enough
//! to be drawn. A felled tree beyond the list's reach is drawn again, which a
//! test of the list's size and the draw's reach keeps out of sight: the
//! nearest [`FELLED`] felled trees cover a clearing the size of the near ring.

use crate::planet::{PLANET_RADIUS, PlanetFine, PlanetRenderFrame};
use crate::saves::WorldSave;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use pbd_core::daylight::DAY_S;
use pbd_core::flora::{FRUIT_RECORD, FruitPicked};
use pbd_core::gather::{TREE_KIND, TreeTaken};
use std::collections::HashMap;

/// How many felled trees, and picked trees, the uniform carries.
pub const FELLED: usize = 64;
pub const PICKED: usize = 32;

/// The tree ids the draw leaves out, nearest the camera first; zero is an
/// empty slot.
#[derive(Resource, Clone, Default, ExtractResource, PartialEq, Debug)]
pub struct TreesTaken {
    pub felled: Vec<u32>,
    pub picked: Vec<u32>,
}

impl TreesTaken {
    /// The lists as the uniform packs them: four ids a lane.
    pub fn packed(&self) -> ([UVec4; FELLED / 4], [UVec4; PICKED / 4]) {
        fn pack<const N: usize>(ids: &[u32]) -> [UVec4; N] {
            let mut out = [UVec4::ZERO; N];
            for (i, id) in ids.iter().take(N * 4).enumerate() {
                out[i / 4][i % 4] = *id;
            }
            out
        }
        (pack(&self.felled), pack(&self.picked))
    }
}

/// The logs a tree gives before it is felled: the gathering's axe count.
fn felled(taken: &TreeTaken, axe_logs: u16) -> bool {
    taken.logs >= axe_logs
}

/// Rebuild the lists twice a second: the records a save holds for trees,
/// matched to the resident fine set's cells, nearest the camera first.
#[allow(clippy::too_many_arguments)]
pub(crate) fn refresh(
    time: Res<Time>,
    mut wait: Local<f32>,
    save: Option<Res<WorldSave>>,
    fine: Option<Res<PlanetFine>>,
    frame: Res<PlanetRenderFrame>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    sun: Option<Res<crate::sky::Sun>>,
    gather: Option<Res<crate::config::GatherConfig>>,
    mut taken: ResMut<TreesTaken>,
) {
    *wait -= time.delta_secs();
    if *wait > 0.0 {
        return;
    }
    *wait = 0.5;
    let (Some(save), Some(fine)) = (save, fine) else {
        return;
    };
    let axe_logs = gather.map_or(4, |g| g.0.axe_logs);
    let day = sun.map_or(0.0, |s| s.clock.seconds / f64::from(DAY_S));
    let mut want: HashMap<u32, bool> = HashMap::new();
    for r in save.records.of_kind(TREE_KIND) {
        if r.read::<TreeTaken>().is_some_and(|t| felled(&t, axe_logs)) {
            want.insert(r.id as u32, true);
        }
    }
    for r in save.records.of_kind(FRUIT_RECORD) {
        if r.read::<FruitPicked>().is_some_and(|p| p.bare_at(day)) {
            want.entry(r.id as u32).or_insert(false);
        }
    }
    if want.is_empty() {
        if !taken.felled.is_empty() || !taken.picked.is_empty() {
            *taken = TreesTaken::default();
        }
        return;
    }
    let eye = cameras
        .iter()
        .next()
        .map(|t| (t.translation().as_dvec3() - frame.center).as_vec3())
        .unwrap_or(Vec3::Y * PLANET_RADIUS);
    let up = eye.normalize_or(Vec3::Y);
    let (mut fell, mut pick) = (Vec::new(), Vec::new());
    for r in fine.set.finest_records() {
        let Some(&is_felled) = want.get(&r.key()) else {
            continue;
        };
        let d = Vec3::from_slice(&r.direction_height[..3]).normalize_or(Vec3::Y);
        let near = 1.0 - d.dot(up);
        if is_felled {
            fell.push((near, r.spare[0]));
        } else {
            pick.push((near, r.spare[0]));
        }
    }
    fell.sort_by(|a, b| a.0.total_cmp(&b.0));
    pick.sort_by(|a, b| a.0.total_cmp(&b.0));
    let next = TreesTaken {
        felled: fell.into_iter().take(FELLED).map(|(_, id)| id).collect(),
        picked: pick.into_iter().take(PICKED).map(|(_, id)| id).collect(),
    };
    if *taken != next {
        *taken = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lists pack four ids a lane, nearest first, the rest zero; and a
    /// tree is felled once the axe has taken all its logs.
    #[test]
    fn the_lists_pack_nearest_first_and_a_spent_tree_is_felled() {
        let taken = TreesTaken {
            felled: vec![11, 22, 33, 44, 55],
            picked: vec![7],
        };
        let (f, p) = taken.packed();
        assert_eq!(f[0], UVec4::new(11, 22, 33, 44));
        assert_eq!(f[1], UVec4::new(55, 0, 0, 0));
        assert!(f[2..].iter().all(|v| *v == UVec4::ZERO));
        assert_eq!(p[0], UVec4::new(7, 0, 0, 0));
        assert!(felled(&TreeTaken { logs: 4 }, 4));
        assert!(
            !felled(&TreeTaken { logs: 1 }, 4),
            "a branch taken, still standing"
        );
    }
}
