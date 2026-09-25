//! What the player carries, as a Bevy resource: the ten slots and the
//! starting kit they are dealt.
//!
//! In the library rather than beside the slot row that draws it, because the
//! world puts things in it too - a catch lands here the frame it is landed -
//! and a store only the desktop binary could reach is one no plugin test
//! could reach either. Drawing it stays in `desktop/slots.rs`.

use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::inventory::{Item, Slots};
use pbd_core::terrain::Material;

/// The starting kit's version: the number of grants in [`KIT_GRANTS`]. A save
/// records the version it was dealt, so a kit that grows deals a saved world
/// what it missed, once.
pub const KIT_VERSION: u32 = KIT_GRANTS.len() as u32;

/// What the kit was before it was versioned: every save from then carries it.
const KIT_BASE: [(Material, u16); 7] = [
    (Material::Grass, 64),
    (Material::Dirt, 64),
    (Material::Stone, 48),
    (Material::Sand, 32),
    (Material::Snow, 16),
    (Material::Rock, 12),
    (Material::Ore, 3),
];

/// What each kit version added, in order; version N is entry N - 1. A new
/// world is dealt the base and all of these; a saved world is dealt the ones
/// past the version its log records. Append, never reorder: a saved version
/// names a prefix of this table.
const KIT_GRANTS: [(Material, u16); 1] = [
    // Something to see with. A kit that could dig into the dark and not light
    // it was a kit that could only dig in daylight.
    (Material::Torch, 16),
];

/// The player's slots as a Bevy resource.
///
/// A newtype rather than a `Resource` derive on the core type: the core owns
/// what a slot holds and depends on nothing but `std`, which is the rule that
/// keeps engine APIs out of it. Everything here derefs straight through.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct Hotbar(pub Slots);

impl Hotbar {
    /// What the player starts carrying.
    ///
    /// A kit rather than an empty row, and that is a preview decision worth
    /// naming: there is nothing to dig yet, so an empty hotbar would draw ten
    /// blank squares and prove nothing about the thumbnails. It goes when
    /// mining lands and the world can fill the slots itself.
    pub fn starting_kit() -> Self {
        let mut slots = Slots::new();
        for (material, count) in KIT_BASE {
            slots.give(Item::Block(material), count);
        }
        grant_since(&mut slots, 0);
        Self(slots)
    }

    /// The hotbar a world opens with: the kit for a new world, the saved
    /// hotbar for a saved one, dealt whatever the kit has gained since the
    /// save last saw it. A grant is recorded through the save's own durable
    /// path on the frame it is dealt, so it is dealt exactly once.
    pub fn restore(save: &mut WorldSave) -> Self {
        let Some(mut slots) = save.carried.clone() else {
            return Self::starting_kit();
        };
        let dealt = save.kit;
        if dealt < KIT_VERSION {
            let given = grant_since(&mut slots, dealt);
            if save.deal_kit(KIT_VERSION, &slots) {
                info!("starting kit v{dealt} -> v{KIT_VERSION}: dealt {given:?}");
            }
        }
        Self(slots)
    }
}

/// Deal every grant past `dealt` into `slots`; what was given.
fn grant_since(slots: &mut Slots, dealt: u32) -> Vec<(Material, u16)> {
    KIT_GRANTS
        .iter()
        .skip(dealt as usize)
        .map(|&(material, count)| {
            slots.give(Item::Block(material), count);
            (material, count)
        })
        .collect()
}

#[cfg(test)]
mod kit_tests {
    use super::*;

    fn count(slots: &Slots, material: Material) -> u32 {
        slots
            .iter()
            .flatten()
            .filter(|stack| stack.item == Item::Block(material))
            .map(|stack| u32::from(stack.count))
            .sum()
    }

    /// A new world is dealt the base and every grant: the kit is one table
    /// and the version is its length, so a grant cannot be in one and not the
    /// other.
    #[test]
    fn a_new_world_is_dealt_the_base_and_every_grant() {
        let kit = Hotbar::starting_kit();
        for (material, expected) in KIT_BASE.iter().chain(&KIT_GRANTS) {
            assert_eq!(count(&kit, *material), u32::from(*expected), "{material:?}");
        }
        assert_eq!(KIT_VERSION, 1);
    }

    /// The owner's report: a world saved before torches joined the kit had
    /// none and never would. It is dealt them once, the log says so, and a
    /// second open deals nothing.
    #[test]
    fn a_save_from_before_the_torches_is_dealt_them_once() {
        let mut save = WorldSave::memory_only();
        let mut old = Slots::new();
        old.give(Item::Block(Material::Grass), 12);
        old.give(Item::Block(Material::Dirt), 53);
        save.carried = Some(old.clone());
        assert_eq!(save.kit, 0);

        let restored = Hotbar::restore(&mut save);
        assert_eq!(count(&restored, Material::Torch), 16, "dealt the torches");
        assert_eq!(count(&restored, Material::Grass), 12, "and kept its own");
        assert_eq!(save.kit, KIT_VERSION, "the save records the deal");
        assert_eq!(
            save.carried.as_ref(),
            Some(&restored.0),
            "and the hotbar it was dealt into"
        );

        let again = Hotbar::restore(&mut save);
        assert_eq!(count(&again, Material::Torch), 16, "not dealt twice");
    }

    /// A save that spent its torches is not refilled: the deal is by version,
    /// never by what is missing.
    #[test]
    fn a_spent_grant_is_not_refilled() {
        let mut save = WorldSave::memory_only();
        save.carried = Some(Slots::new());
        save.kit = KIT_VERSION;
        let restored = Hotbar::restore(&mut save);
        assert_eq!(count(&restored, Material::Torch), 0);
    }
}
