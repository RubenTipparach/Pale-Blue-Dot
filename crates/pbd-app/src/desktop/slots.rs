//! The ten slots, drawn.
//!
//! A slot carries its item's own THUMBNAIL rather than its name, which is the
//! inherited UI rule: an inventory presented as a list of names is the failure
//! that rule exists to prevent.
//!
//! The thumbnails need no new art. The terrain shader already samples
//! `tilesets/atlas.png` - every biome's tileset baked into one texture, four
//! sheets across and four down, each a 4x4 grid - and already maps every
//! material to a sheet, a tile in it and a base colour over it; a slot is the
//! same tile at the same tint, so a block's icon IS the texture the ground is
//! drawn with. That is the grass blades' own trick - they sample the ground
//! tile they stand on - asked for a second time.

use bevy::prelude::*;
use pbd_core::inventory::{Item, SLOTS, Slots};
use pbd_core::terrain::Material;

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
        for (material, count) in [
            (Material::Grass, 64),
            (Material::Dirt, 64),
            (Material::Stone, 48),
            (Material::Sand, 32),
            (Material::Snow, 16),
            (Material::Rock, 12),
            (Material::Ore, 3),
            // Something to see with. A kit that could dig into the dark and
            // not light it was a kit that could only dig in daylight.
            (Material::Torch, 16),
        ] {
            slots.give(Item::Block(material), count);
        }
        Self(slots)
    }
}

/// Sheets across the atlas, and tiles across a sheet.
const ATLAS_SHEETS: f32 = 4.0;
const ATLAS_TILES: f32 = 4.0;

/// Which tile of the atlas a material draws with, and the colour the terrain
/// shader lays over it. Both halves come from `planet_surface.wgsl`'s own
/// table, so a slot and the ground cannot disagree about what dirt looks like.
///
/// Several materials share a tile, which is honest: they share it on the ground
/// too, and what tells grass from swamp grass there is the tint, here as well.
pub fn thumbnail(material: Material) -> Option<(u32, Vec2, Color)> {
    use pbd_app::planet::{snow_slot, tileset_slot};
    use pbd_core::planet_gen::Biome;
    // Which SHEET a block comes from, which is the biome it is found in: sand
    // is a beach's, snow is the tundra's, and the rest are the home meadow's.
    // The ground asks the same two functions for the same answer.
    let home = |biome| tileset_slot(biome);
    let (slot, tile, rgb): (u32, (f32, f32), (f32, f32, f32)) = match material {
        Material::Air => return None,
        Material::Grass | Material::DryGrass => {
            (home(Biome::Fields), (0., 0.), (0.12, 0.32, 0.075))
        }
        Material::JungleGrass => (home(Biome::Jungle), (0., 0.), (0.07, 0.25, 0.105)),
        // Earth has its own picture at last, which is the tile a wall shows
        // under the sod rather than the beach it used to borrow.
        Material::Soil | Material::Dirt => (home(Biome::Fields), (2., 0.), (0.61, 0.48, 0.25)),
        Material::Sand => (home(Biome::Beach), (0., 0.), (0.72, 0.62, 0.42)),
        Material::Stone => (home(Biome::Fields), (3., 0.), (0.31, 0.34, 0.33)),
        Material::Rock => (home(Biome::Mountains), (3., 0.), (0.37, 0.33, 0.29)),
        Material::Snow => (snow_slot(), (0., 0.), (0.80, 0.90, 0.91)),
        Material::Ore => (home(Biome::Fields), (0., 1.), (0.72, 0.62, 0.34)),
        Material::Water => (home(Biome::Ocean), (2., 2.), (0.13, 0.40, 0.56)),
        // A torch: the wood tile, lit. The grain is what a torch is made of
        // and the tint is the flame on it, which at a 44 px slot reads as a
        // burning brand. It is a STAND-IN for art a torch has not been drawn
        // yet - said here rather than left for a reader to notice, because
        // this repository's rule is that an item ships with a visual and a
        // borrowed tile is the weakest version of keeping it.
        Material::Torch => (home(Biome::Fields), (2., 1.), (1.0, 0.62, 0.22)),
    };
    // The shader's albedo is a fraction of full brightness because the ground
    // is then LIT by a sun, and a slot is lit by nothing. Lifting it by a
    // GAMMA raises the dark materials without flattening the bright ones,
    // which is what keeps snow whiter than stone and stone paler than soil.
    //
    // Normalising each material to its own brightest channel was the first
    // attempt and it is the wrong shape: it throws away exactly the relative
    // brightness that tells the materials apart, so snow came out the same
    // grey as stone and sand came out brick red. Preserve the order, lift the
    // floor.
    let lift = |c: f32| c.clamp(0.0, 1.0).powf(0.6);
    Some((
        slot,
        Vec2::new(tile.0, tile.1),
        Color::srgb(lift(rgb.0), lift(rgb.1), lift(rgb.2)),
    ))
}

/// Marks the slot at this index, so the update can find it without a lookup.
#[derive(Component)]
pub struct SlotCell(pub usize);
/// The thumbnail inside a slot.
#[derive(Component)]
pub struct SlotIcon(pub usize);
/// The stack count badge.
#[derive(Component)]
pub struct SlotCount(pub usize);

const SLOT: f32 = 44.0;
const GAP: f32 = 4.0;

fn border_of(selected: bool) -> Color {
    if selected {
        Color::srgb(0.92, 0.97, 0.95)
    } else {
        Color::srgba(0.55, 0.75, 0.74, 0.5)
    }
}

fn fill_of(selected: bool) -> Color {
    if selected {
        Color::srgba(0.09, 0.16, 0.18, 0.92)
    } else {
        Color::srgba(0.02, 0.06, 0.08, 0.72)
    }
}

/// Build the row. One parent, ten children, each a bordered square holding an
/// icon and a count.
///
/// `PostStartup` rather than `Startup`, because `PlanetArt` is inserted BY a
/// startup system and a command is applied at the end of the schedule that
/// queued it: asking for it alongside is asking for a resource that does not
/// exist yet. The atlas handle is taken from there rather than loaded again, so
/// the slot art and the ground art are one asset with one sampler - loading it
/// a second time would make the nearest-point sampling depend on which load
/// happened to win.
pub fn spawn(
    mut commands: Commands,
    atlas: Res<pbd_app::planet::PlanetArt>,
    existing: Query<(), With<SlotCell>>,
) {
    if !existing.is_empty() {
        return;
    }
    let atlas = atlas.0.clone();
    let commands = &mut commands;
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: px(22),
            left: percent(50),
            margin: UiRect::left(px(-(SLOTS as f32 * (SLOT + GAP) - GAP) / 2.0)),
            column_gap: px(GAP),
            ..default()
        })
        .with_children(|row| {
            for index in 0..SLOTS {
                row.spawn((
                    Node {
                        width: px(SLOT),
                        height: px(SLOT),
                        border: UiRect::all(px(1)),
                        padding: UiRect::all(px(4)),
                        ..default()
                    },
                    BorderColor::all(border_of(index == 0)),
                    BackgroundColor(fill_of(index == 0)),
                    SlotCell(index),
                ))
                .with_children(|cell| {
                    cell.spawn((
                        ImageNode {
                            image: atlas.clone(),
                            color: Color::NONE,
                            ..default()
                        },
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            ..default()
                        },
                        SlotIcon(index),
                    ));
                    cell.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.97, 0.95)),
                        TextShadow::default(),
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: px(1),
                            right: px(3),
                            ..default()
                        },
                        SlotCount(index),
                    ));
                });
            }
        });
}

/// Repaint the row from the store. Everything a slot shows is derived here, so
/// the store stays the one place that knows what is carried.
#[allow(clippy::type_complexity)]
pub fn update(
    slots: Res<Hotbar>,
    images: Res<Assets<Image>>,
    mut cells: Query<(&SlotCell, &mut BorderColor, &mut BackgroundColor)>,
    mut icons: Query<(&SlotIcon, &mut ImageNode)>,
    mut counts: Query<(&SlotCount, &mut Text)>,
    mut art_ready: Local<bool>,
) {
    // Repaint when the store moves, and ALSO until the art has arrived once.
    //
    // Gating only on the store was a real bug and an invisible one: the atlas
    // loads asynchronously, so on the first frame there is no image to crop a
    // thumbnail out of, every icon was set transparent, and the store then
    // never changed again - so they were never revisited and the row drew ten
    // empty squares for ever. The stack counts were right the whole time,
    // because a number needs no asset, which is exactly what made it look like
    // a texture problem rather than a scheduling one.
    let ready = !icons.is_empty()
        && icons
            .iter()
            .next()
            .is_some_and(|(_, node)| images.contains(&node.image));
    if !slots.is_changed() && *art_ready {
        return;
    }
    *art_ready = ready;
    for (cell, mut border, mut background) in &mut cells {
        let selected = cell.0 == slots.selected();
        *border = BorderColor::all(border_of(selected));
        background.0 = fill_of(selected);
    }
    for (icon, mut node) in &mut icons {
        let art = slots.get(icon.0).and_then(|stack| match stack.item {
            Item::Block(material) => thumbnail(material),
            // A tool has no block texture. None of them is constructed yet, so
            // this cannot be hit; when the first one lands it needs an icon of
            // its own in the same change, which is the rule about no item
            // shipping without a visual.
            Item::Tool(_) => None,
        });
        match art {
            Some((slot, tile, tint)) => {
                // The rect is in the image's own pixels, so it needs the loaded
                // size. Until the atlas has loaded there is nothing to crop to,
                // and a full-image icon would be sixteen tiles at once.
                let Some(size) = images.get(&node.image).map(|image| image.size_f32()) else {
                    node.color = Color::NONE;
                    continue;
                };
                let sheet = size / ATLAS_SHEETS;
                let step = sheet / ATLAS_TILES;
                let origin = Vec2::new(
                    (slot % ATLAS_SHEETS as u32) as f32 * sheet.x,
                    (slot / ATLAS_SHEETS as u32) as f32 * sheet.y,
                );
                let min = origin + Vec2::new(tile.x * step.x, tile.y * step.y);
                // Half a texel, which is all the bake leaves to guard: a tile
                // is exactly 32 texels in the atlas with nothing bleeding into
                // it, where the source sheets had soft edges to keep clear of.
                let inset = step / 64.;
                node.rect = Some(Rect::from_corners(min + inset, min + step - inset));
                node.color = tint;
            }
            None => node.color = Color::NONE,
        }
    }
    for (count, mut text) in &mut counts {
        let label = match slots.get(count.0) {
            Some(stack) if stack.count > 1 => stack.count.to_string(),
            _ => String::new(),
        };
        if text.0 != label {
            text.0 = label;
        }
    }
}

/// Number row picks a slot, the wheel steps it.
pub fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    menu: Option<Res<pbd_app::controls::MenuOpen>>,
    mut slots: ResMut<Hotbar>,
) {
    // A menu holds the keyboard, so the number row does not change what you
    // are holding while you read the bindings. The wheel is still DRAINED
    // below whatever happens here, which is the reader's own old lesson: a
    // reader that skips its messages delivers the whole backlog next time.
    let held = menu.is_some_and(|open| open.0);
    const ROW: [KeyCode; SLOTS] = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ];
    for (index, key) in ROW.iter().enumerate() {
        if !held && keys.just_pressed(*key) {
            slots.select(index);
        }
    }
    // The wheel already drives the camera's zoom. Only one of them may have it
    // on a frame, or a player changing slots would dolly the camera at the same
    // time: it picks slots on foot, where the hotbar is what a wheel is for,
    // and stays the zoom in flight. The events are drained either way, because
    // a reader that skips its messages delivers the whole backlog the next time
    // it does read - which is the bug the camera drag already had once.
    let walking = walking.is_some_and(|readout| readout.active);
    let mut step = 0;
    for message in wheel.read() {
        step += if message.y > 0.0 {
            -1
        } else if message.y < 0.0 {
            1
        } else {
            0
        };
    }
    if walking && step != 0 && !held {
        slots.step(step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every material the terrain can put on screen has a thumbnail, so a new
    /// material cannot ship as a blank slot. Air is the one exclusion and it is
    /// explicit: it is the absence of a block, not a block you could hold.
    #[test]
    fn every_drawable_material_has_a_thumbnail() {
        for material in [
            Material::Stone,
            Material::Soil,
            Material::Grass,
            Material::Water,
            Material::Ore,
            Material::Sand,
            Material::DryGrass,
            Material::JungleGrass,
            Material::Snow,
            Material::Rock,
            Material::Dirt,
        ] {
            let art = thumbnail(material);
            assert!(art.is_some(), "{material:?} would draw a blank slot");
            let (slot, tile, _) = art.unwrap();
            assert!(
                (0.0..ATLAS_TILES).contains(&tile.x) && (0.0..ATLAS_TILES).contains(&tile.y),
                "{material:?} points outside its {ATLAS_TILES}x{ATLAS_TILES} sheet"
            );
            assert!(
                slot < (ATLAS_SHEETS * ATLAS_SHEETS) as u32,
                "{material:?} points at sheet {slot}, outside the atlas"
            );
        }
        assert!(thumbnail(Material::Air).is_none(), "air is not a block");
    }

    /// Snow is brighter than stone and stone brighter than soil, which is the
    /// whole reason the lift is a gamma rather than a per-material normalise:
    /// normalising threw this ordering away and made snow look like stone.
    #[test]
    fn the_thumbnails_keep_their_relative_brightness() {
        let value = |material| {
            let (_, _, colour) = thumbnail(material).unwrap();
            let rgb = colour.to_srgba();
            rgb.red + rgb.green + rgb.blue
        };
        assert!(value(Material::Snow) > value(Material::Stone));
        assert!(value(Material::Stone) > value(Material::Grass));
        assert!(value(Material::Sand) > value(Material::JungleGrass));
    }
}
