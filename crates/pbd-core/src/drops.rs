//! Floating drops: a dug block hovers where it was cut until the player comes
//! for it (`inventory-grid` decision 4, survey I3).
//!
//! The numbers are Tenebris's (`tenebris-client/src/drops.rs`, read at the
//! pinned commit): the scatter, the bob and spin, the magnet and the pickup
//! radius, and the 300 s life. What differs is durability. Tenebris keeps its
//! drops in memory and loses them at a quit; here a drop is the player's
//! property lying in the world, so the save records it (`saves/format.rs`)
//! and a load puts it back with the time it had left.
//!
//! The rules are here rather than beside the system that runs them because
//! where a drop goes and who picks it up are what a future multiplayer has to
//! agree about. Drawing one is the app's business.

use crate::inventory::{Item, Slots};
use glam::Vec3;
use std::f32::consts::TAU;

/// How far a drop is pushed sideways from the cell's centre, m, so a pile
/// from one spot does not stack in one place.
pub const SCATTER_M: f32 = 0.22;
/// A block's drop is a hex prism of its material: the corner radius, m.
pub const PRISM_RADIUS_M: f32 = 0.16;
/// And its half height, m.
pub const PRISM_HALF_HEIGHT_M: f32 = 0.10;
/// A light drops as its icon on a card that faces the camera: half its side,
/// m.
pub const ICON_HALF_M: f32 = 0.14;
/// How far a drop bobs above and below where it floats, m.
pub const BOB_M: f32 = 0.05;
/// How fast it bobs, rad/s.
pub const BOB_RATE: f32 = 2.2;
/// How fast it turns about the local up, rad/s.
pub const SPIN_RATE: f32 = 1.5;
/// Where on the player a drop is pulled to: the chest, this far above the
/// feet, m.
pub const CHEST_M: f32 = 0.8;
/// Inside this distance of the chest a drop is pulled in, m.
pub const MAGNET_M: f32 = 2.6;
/// Inside this distance it is picked up, m.
pub const PICKUP_M: f32 = 1.2;
/// The magnet's speed at its edge, m/s. It grows by the same again for every
/// metre closer: `MAGNET_SPEED_MPS * (1 + MAGNET_M - d)`.
pub const MAGNET_SPEED_MPS: f32 = 7.0;
/// How long a drop lasts, s of world time. World time passes only while the
/// world is played, so a drop left at a quit has the same time left at the
/// next load.
pub const LIFE_S: f64 = 300.0;

/// One drop floating in the world.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemDrop {
    /// Its name in the save. A world numbers its drops in the order they are
    /// made, and a pickup names the drop it took from.
    pub id: u64,
    pub item: Item,
    /// How many are on it. A drop that is partly picked up keeps the rest.
    pub count: u16,
    /// Where it floats, planet-local metres.
    pub position: Vec3,
    /// The world time it was made, s since midnight of day 0.
    pub made_s: f64,
}

impl ItemDrop {
    /// Its phase in the bob and the spin, so a pile of drops does not move
    /// in step. Tenebris uses `1.7 n` for the n-th drop alive; the id is the
    /// n here, and is stable across a load.
    pub fn phase(&self) -> f32 {
        ((self.id % 1024) as f32 * 1.7) % TAU
    }

    /// Whether it has outlived its time at world time `now_s`.
    pub fn expired(&self, now_s: f64) -> bool {
        now_s - self.made_s > LIFE_S
    }

    /// How far it sits above its floating point, m, at time `t` s.
    pub fn bob(&self, t: f32) -> f32 {
        (t * BOB_RATE + self.phase()).sin() * BOB_M
    }

    /// How far it has turned about its up, rad, at time `t` s.
    pub fn spin(&self, t: f32) -> f32 {
        t * SPIN_RATE + self.phase()
    }
}

/// Where a drop from a cell starts: the cell's centre, pushed sideways by
/// [`SCATTER_M`] in a direction its id picks. Tenebris keys the direction off
/// the same counter as the phase, so two drops from one cut part.
pub fn scattered(centre: Vec3, id: u64) -> Vec3 {
    let up = centre.normalize_or(Vec3::Y);
    let reference = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let east = reference.cross(up).normalize();
    let north = up.cross(east);
    let angle = ((id % 1024) as f32 * 1.7) % TAU;
    centre + (east * angle.cos() + north * angle.sin()) * SCATTER_M
}

/// What one frame did to a drop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moved {
    /// Out of the magnet's reach, or already where the magnet put it.
    Stayed,
    /// Pulled toward the player.
    Pulled,
    /// Picked up, this many. What did not fit is still on the drop, and
    /// `count` says how many are left.
    Picked { taken: u16 },
}

/// One frame of the magnet for one drop: pick it up into `slots` by the give
/// order if it is inside [`PICKUP_M`] of `chest`, or pull it toward `chest`
/// if it is inside [`MAGNET_M`]. A pull never overshoots the chest.
///
/// A drop inside the pickup radius that does not fit is tried again the next
/// frame, and stays where it is until something makes room: a full pack is
/// the owner's "floating blocks".
pub fn step(drop: &mut ItemDrop, chest: Vec3, dt: f32, slots: &mut Slots) -> Moved {
    let to_chest = chest - drop.position;
    let distance = to_chest.length();
    if distance < PICKUP_M {
        let left = slots.give(drop.item, drop.count);
        let taken = drop.count - left;
        drop.count = left;
        return if taken > 0 {
            Moved::Picked { taken }
        } else {
            Moved::Stayed
        };
    }
    if distance < MAGNET_M && distance > 1e-6 && dt > 0.0 {
        let speed = MAGNET_SPEED_MPS * (1.0 + (MAGNET_M - distance));
        let moved = (speed * dt).min(distance);
        drop.position += to_chest * (moved / distance);
        return Moved::Pulled;
    }
    Moved::Stayed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{CARRIED, Stack};
    use crate::terrain::Material;

    fn dirt(at: Vec3) -> ItemDrop {
        ItemDrop {
            id: 7,
            item: Item::Block(Material::Dirt),
            count: 1,
            position: at,
            made_s: 100.0,
        }
    }

    const CHEST: Vec3 = Vec3::new(0.0, 300.8, 0.0);

    /// Inside the magnet a drop moves toward the chest, by the magnet's speed
    /// for the frame, and outside it does not move at all.
    #[test]
    fn the_magnet_pulls_inside_its_reach_and_not_outside() {
        let mut slots = Slots::new();
        let mut near = dirt(CHEST + Vec3::X * 2.0);
        assert_eq!(step(&mut near, CHEST, 0.01, &mut slots), Moved::Pulled);
        let expected = MAGNET_SPEED_MPS * (1.0 + (MAGNET_M - 2.0)) * 0.01;
        assert!(((2.0 - (near.position - CHEST).length()) - expected).abs() < 1e-4);
        let mut far = dirt(CHEST + Vec3::X * 3.0);
        assert_eq!(step(&mut far, CHEST, 0.01, &mut slots), Moved::Stayed);
        assert_eq!(far.position, CHEST + Vec3::X * 3.0);
        assert_eq!(slots.iter().flatten().count(), 0, "nothing picked up");
    }

    /// A long frame does not throw a drop through the player and out the
    /// other side.
    #[test]
    fn a_pull_never_overshoots_the_chest() {
        let mut slots = Slots::new();
        let mut drop = dirt(CHEST + Vec3::Z * 1.5);
        step(&mut drop, CHEST, 10.0, &mut slots);
        assert!((drop.position - CHEST).length() < 1e-4);
    }

    /// Inside the pickup radius it goes in by the give order: onto the
    /// hotbar's matching stack first.
    #[test]
    fn a_drop_in_reach_is_picked_up_by_the_give_order() {
        let mut slots = Slots::new();
        slots.set(3, Some(Stack::new(Item::Block(Material::Dirt), 5)));
        let mut drop = dirt(CHEST + Vec3::X);
        drop.count = 2;
        assert_eq!(
            step(&mut drop, CHEST, 0.01, &mut slots),
            Moved::Picked { taken: 2 }
        );
        assert_eq!(drop.count, 0);
        assert_eq!(slots.get(3).unwrap().count, 7);
    }

    /// With everything full it floats, and is tried again when there is
    /// room: nothing is lost and nothing is taken that did not fit.
    #[test]
    fn what_does_not_fit_stays_on_the_drop() {
        let mut slots = Slots::new();
        for k in 0..CARRIED {
            slots.set(k, Some(Stack::new(Item::Fish(k as u16), 16)));
        }
        let mut drop = dirt(CHEST + Vec3::X);
        drop.count = 3;
        assert_eq!(step(&mut drop, CHEST, 0.01, &mut slots), Moved::Stayed);
        assert_eq!(drop.count, 3);
        assert_eq!(drop.position, CHEST + Vec3::X, "it floats where it was");
        slots.set(20, Some(Stack::new(Item::Block(Material::Dirt), 98)));
        assert_eq!(
            step(&mut drop, CHEST, 0.01, &mut slots),
            Moved::Picked { taken: 1 }
        );
        assert_eq!(drop.count, 2, "the rest stays on the drop");
    }

    /// A drop lasts 300 s of world time and not a moment more.
    #[test]
    fn a_drop_goes_after_three_hundred_seconds() {
        let drop = dirt(CHEST);
        assert!(!drop.expired(drop.made_s + LIFE_S));
        assert!(drop.expired(drop.made_s + LIFE_S + 0.01));
    }

    /// The scatter is sideways, the length Tenebris uses, and parts two
    /// drops from one cut.
    #[test]
    fn the_scatter_is_sideways_and_parts_neighbours() {
        let centre = Vec3::new(120.0, 250.0, -90.0);
        let a = scattered(centre, 1);
        let b = scattered(centre, 2);
        for p in [a, b] {
            assert!(((p - centre).length() - SCATTER_M).abs() < 1e-4);
            assert!(
                (p - centre).dot(centre.normalize()).abs() < 1e-3,
                "sideways"
            );
        }
        assert!((a - b).length() > 0.1);
        let pole = Vec3::new(0.0, 300.0, 0.0);
        assert!(scattered(pole, 5).is_finite());
    }
}
