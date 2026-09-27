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

/// What a new world is dealt before the grants. Only a new world reads it: a
/// saved world carries its own slots, whatever the base was when it began.
///
/// Snow, rock and ore were in it until the lights needed their slots
/// (`lamps-and-lanterns` decision 12, the owner in survey L2: "dont need
/// those blocks in the inventory really"). Digging gives them back.
const KIT_BASE: [(Material, u16); 4] = [
    (Material::Grass, 64),
    (Material::Dirt, 64),
    (Material::Stone, 48),
    (Material::Sand, 32),
];

/// What each kit version added, in order; version N is entry N - 1. A new
/// world is dealt the base and all of these; a saved world is dealt the ones
/// past the version its log records. Append, never reorder: a saved version
/// names a prefix of this table.
const KIT_GRANTS: [&[(Material, u16)]; 2] = [
    // Something to see with. A kit that could dig into the dark and not light
    // it was a kit that could only dig in daylight.
    &[(Material::Torch, 16)],
    // The city's lights (`lamps-and-lanterns` task 5.4).
    &[
        (Material::LanternPost, 8),
        (Material::LanternWall, 8),
        (Material::LanternHanging, 8),
        (Material::Brazier, 4),
        (Material::Candle, 16),
    ],
];

/// A light's own icon, as an asset path, or `None` for a block that is not
/// a light. Each light is its own picture, drawn by `tools/gen_item_icons.py`
/// (`lamps-and-lanterns` task 5.3), rather than a tinted tile of a material
/// it is not.
pub fn light_icon(material: Material) -> Option<&'static str> {
    Some(match material {
        Material::Torch => "items/lights/torch.png",
        Material::LanternPost => "items/lights/lantern_post.png",
        Material::LanternWall => "items/lights/lantern_wall.png",
        Material::LanternHanging => "items/lights/lantern_hanging.png",
        Material::Brazier => "items/lights/brazier.png",
        Material::Candle => "items/lights/candle.png",
        _ => return None,
    })
}

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
        let dealt = grant_since(&mut slots, 0);
        debug_assert!(dealt.no_room.is_empty(), "a new world's kit fits");
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
            let grant = grant_since(&mut slots, dealt);
            if save.deal_kit(KIT_VERSION, &slots) {
                info!(
                    "starting kit v{dealt} -> v{KIT_VERSION}: dealt {:?}",
                    grant.given
                );
                if !grant.no_room.is_empty() {
                    // There is nowhere else to put them until the pack
                    // (`inventory-grid`), and the deal is by version, so
                    // these are not dealt later either.
                    warn!(
                        "starting kit v{KIT_VERSION}: no room in the hotbar for {:?}",
                        grant.no_room
                    );
                }
            }
        }
        Self(slots)
    }
}

/// What a deal gave, and what it had no room for.
#[derive(Debug, Default, PartialEq)]
struct Grant {
    given: Vec<(Material, u16)>,
    no_room: Vec<(Material, u16)>,
}

/// Deal every grant past `dealt` into `slots`, into whatever room they have.
fn grant_since(slots: &mut Slots, dealt: u32) -> Grant {
    let mut grant = Grant::default();
    for &(material, count) in KIT_GRANTS.iter().skip(dealt as usize).copied().flatten() {
        let left = slots.give(Item::Block(material), count);
        if left < count {
            grant.given.push((material, count - left));
        }
        if left > 0 {
            grant.no_room.push((material, left));
        }
    }
    grant
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
    /// other. It fills the ten slots exactly, and holds every light but not
    /// the blocks digging gives back (decision 12).
    #[test]
    fn a_new_world_is_dealt_the_base_and_every_grant() {
        let kit = Hotbar::starting_kit();
        for (material, expected) in KIT_BASE.iter().chain(KIT_GRANTS.iter().copied().flatten()) {
            assert_eq!(count(&kit, *material), u32::from(*expected), "{material:?}");
        }
        assert_eq!(KIT_VERSION, 2);
        assert!(kit.iter().all(|slot| slot.is_some()), "ten slots, all full");
        for light in Material::LAMPS {
            assert!(count(&kit, light) > 0, "{light:?} is in the kit");
        }
        for dug in [Material::Snow, Material::Rock, Material::Ore] {
            assert_eq!(count(&kit, dug), 0, "{dug:?} is dug, not dealt");
        }
    }

    /// The kit as it was dealt at version 1: seven blocks and the torches.
    fn the_old_kit() -> Slots {
        let mut slots = Slots::new();
        for (material, count) in [
            (Material::Grass, 64),
            (Material::Dirt, 64),
            (Material::Stone, 48),
            (Material::Sand, 32),
            (Material::Snow, 16),
            (Material::Rock, 12),
            (Material::Ore, 3),
            (Material::Torch, 16),
        ] {
            slots.give(Item::Block(material), count);
        }
        slots
    }

    /// A save dealt the old kit has two slots free. The lights go into them
    /// in the grant's order, once, and the rest are named as having no room
    /// rather than dealt later (decision 12).
    #[test]
    fn an_old_save_is_dealt_the_lights_that_fit_once() {
        let mut save = WorldSave::memory_only();
        save.carried = Some(the_old_kit());
        save.kit = 1;
        let mut slots = the_old_kit();
        let grant = grant_since(&mut slots, 1);
        assert_eq!(
            grant.given,
            vec![(Material::LanternPost, 8), (Material::LanternWall, 8)]
        );
        assert_eq!(
            grant.no_room,
            vec![
                (Material::LanternHanging, 8),
                (Material::Brazier, 4),
                (Material::Candle, 16)
            ]
        );

        let restored = Hotbar::restore(&mut save);
        assert_eq!(restored.0, slots, "restore deals the same");
        assert_eq!(count(&restored, Material::Ore), 3, "and keeps its own");
        assert_eq!(save.kit, KIT_VERSION, "the save records the deal");
        let again = Hotbar::restore(&mut save);
        assert_eq!(again.0, slots, "not dealt twice");
    }

    /// A full hotbar is dealt nothing, and the deal is still recorded, so it
    /// is not tried again on every open.
    #[test]
    fn a_full_hotbar_is_dealt_nothing_and_says_so() {
        let mut full = the_old_kit();
        full.give(Item::Block(Material::DryGrass), 1);
        full.give(Item::Block(Material::JungleGrass), 1);
        let mut save = WorldSave::memory_only();
        save.carried = Some(full.clone());
        save.kit = 1;
        let mut slots = full.clone();
        let grant = grant_since(&mut slots, 1);
        assert!(grant.given.is_empty());
        assert_eq!(grant.no_room.len(), 5, "every light is named");
        let restored = Hotbar::restore(&mut save);
        assert_eq!(restored.0, full, "nothing moved");
        assert_eq!(save.kit, KIT_VERSION);
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

    /// Every light has its own icon on disk, a 16x16 picture on a transparent
    /// ground in the lights' one palette, and no block that is not a light
    /// claims one (`lamps-and-lanterns` task 5.3).
    #[test]
    fn every_light_has_its_own_icon_and_nothing_else_does() {
        use bevy::asset::RenderAssetUsages;
        use bevy::image::{CompressedImageFormats, Image, ImageSampler, ImageType};
        let assets = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        let mut seen = std::collections::HashSet::new();
        for light in Material::LAMPS {
            let path = light_icon(light).unwrap_or_else(|| panic!("{light:?} has no icon"));
            assert!(seen.insert(path), "{light:?} shares {path}");
            let bytes = std::fs::read(assets.join(path)).expect(path);
            let image = Image::from_buffer(
                &bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::nearest(),
                RenderAssetUsages::default(),
            )
            .expect(path);
            assert_eq!(image.size(), UVec2::new(16, 16), "{path}");
            let data = image.data.as_ref().expect(path);
            let alpha = |x: usize, y: usize| data[(y * 16 + x) * 4 + 3];
            for (x, y) in [(0, 15), (15, 0), (15, 15)] {
                assert_eq!(alpha(x, y), 0, "{path}: a transparent ground at ({x}, {y})");
            }
            let opaque = (0..256).filter(|&i| data[i * 4 + 3] == 255).count();
            assert!(opaque >= 20, "{path}: {opaque} pixels is not a picture");
            assert!(
                (0..256).all(|i| matches!(data[i * 4 + 3], 0 | 255)),
                "{path}: pixel art is opaque or clear, never half"
            );
        }
        for block in [
            Material::Stone,
            Material::Grass,
            Material::Air,
            Material::Water,
        ] {
            assert!(light_icon(block).is_none(), "{block:?} is not a light");
        }
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
