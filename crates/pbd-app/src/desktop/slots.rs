//! The ten slots, drawn, and the squares the pack's grid is built from.
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
use pbd_core::inventory::{Item, SLOTS};
use pbd_core::terrain::Material;

pub use pbd_app::hotbar::Hotbar;

/// Sheets across the atlas, and tiles across a sheet.
pub const ATLAS_SHEETS: f32 = 4.0;
pub const ATLAS_TILES: f32 = 4.0;

/// The tiles a block is drawn with, as the ground draws it
/// (`planet_surface.wgsl`, after Tenebris's `face_tile`): its top, its sides
/// and its underside, all on one sheet. A grassy block wears the sheet's
/// ground on top, the ground-over-earth transition on its sides and earth
/// underneath; every other block is one tile all round (`inventory-grid`
/// decision 7, survey I4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockArt {
    pub sheet: u32,
    pub top: Vec2,
    pub side: Vec2,
    pub under: Vec2,
}

/// A block's art, or `None` for air and for a light, which is its own
/// picture (`hotbar::light_icon`, `lamps-and-lanterns` task 5.3).
pub fn block_art(material: Material) -> Option<BlockArt> {
    use pbd_app::planet::{snow_slot, tileset_slot};
    use pbd_core::planet_gen::Biome;
    // Which SHEET a block comes from, which is the biome it is found in: sand
    // is a beach's, snow is the tundra's, and the rest are the home meadow's.
    // The ground asks the same two functions for the same answer. The tiles
    // are each sheet's layout: #0 its ground, #1 that ground over the earth,
    // #2 the earth, #3 the stone.
    let home = |biome| tileset_slot(biome);
    let one = |sheet: u32, x: f32, y: f32| BlockArt {
        sheet,
        top: Vec2::new(x, y),
        side: Vec2::new(x, y),
        under: Vec2::new(x, y),
    };
    let grassy = |sheet: u32| BlockArt {
        sheet,
        top: Vec2::new(0., 0.),
        side: Vec2::new(1., 0.),
        under: Vec2::new(2., 0.),
    };
    Some(match material {
        Material::Air => return None,
        // Grass and dry grass are one render code: the ground draws them alike.
        Material::Grass | Material::DryGrass => grassy(home(Biome::Fields)),
        Material::JungleGrass => grassy(home(Biome::Jungle)),
        Material::Soil | Material::Dirt => one(home(Biome::Fields), 2., 0.),
        Material::Sand => one(home(Biome::Beach), 0., 0.),
        Material::Stone => one(home(Biome::Fields), 3., 0.),
        Material::Rock => one(home(Biome::Mountains), 3., 0.),
        Material::Snow => one(snow_slot(), 0., 0.),
        Material::Ore => one(home(Biome::Fields), 0., 1.),
        Material::Water => one(home(Biome::Ocean), 2., 2.),
        Material::Torch
        | Material::LanternPost
        | Material::LanternWall
        | Material::LanternHanging
        | Material::Brazier
        | Material::Candle => return None,
    })
}

/// Which tile a block's slot shows, and the colour laid over it: its SIDE,
/// as a block is seen standing in the world, for grass the sward over earth
/// (the owner, survey I4: "should use the side of the block, there is a grad
/// transtion to dirt block"). Untinted, since the ground draws its tiles in
/// their own colours.
pub fn thumbnail(material: Material) -> Option<(u32, Vec2, Color)> {
    let art = block_art(material)?;
    Some((art.sheet, art.side, Color::WHITE))
}

/// Every item picture that is not a block: a tool's and a fish's own 16x16
/// icon, loaded once, beside the atlas the blocks are cropped from. The
/// field guide and the tool slot draw from here too, so an item has one
/// picture everywhere it appears.
#[derive(Resource, Clone)]
pub struct ItemIcons {
    pub atlas: Handle<Image>,
    /// By roster index.
    pub fish: Vec<Handle<Image>>,
    /// In `Tool::ALL` order.
    pub tools: Vec<Handle<Image>>,
    /// In `Material::LAMPS` order.
    pub lights: Vec<Handle<Image>>,
}

impl ItemIcons {
    pub fn tool(&self, tool: pbd_core::inventory::Tool) -> Handle<Image> {
        self.tools[tool.index()].clone()
    }
}

/// Load the icons. `PostStartup`, beside the slot row, for the atlas's sake.
pub fn load_icons(
    mut commands: Commands,
    assets: Res<AssetServer>,
    atlas: Res<pbd_app::planet::PlanetArt>,
    fauna: Res<pbd_app::config::FaunaConfig>,
    body: Res<pbd_app::fish::Body>,
) {
    let fish = fauna
        .0
        .roster(&body.0)
        .iter()
        .map(|species| assets.load(pbd_app::fish::species_icon(species)))
        .collect();
    let tools = pbd_core::inventory::Tool::ALL
        .iter()
        .map(|tool| assets.load(pbd_app::fish::tool_icon(*tool)))
        .collect();
    let lights = Material::LAMPS
        .iter()
        .filter_map(|&light| pbd_app::hotbar::light_icon(light))
        .map(|path| assets.load(path))
        .collect();
    commands.insert_resource(ItemIcons {
        atlas: atlas.0.clone(),
        fish,
        tools,
        lights,
    });
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

pub fn border_of(selected: bool) -> Color {
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
                cell(row, index, &atlas, ());
            }
        });
}

/// One slot's square: a bordered cell holding an icon and a count, marked
/// with its index so [`update`] paints it. The hotbar row and the pack's grid
/// are built from these, with `extra` on the square (the pack's are buttons).
pub fn cell(
    row: &mut ChildSpawnerCommands,
    index: usize,
    atlas: &Handle<Image>,
    extra: impl Bundle,
) {
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
        extra,
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

/// Paint one icon with a stack's picture: a fish's, a tool's or a light's
/// own, whole and untinted, or a block's tile cropped from the atlas at its
/// tint. Nothing, or art not loaded yet, paints it clear.
pub fn paint_icon(
    node: &mut ImageNode,
    stack: Option<pbd_core::inventory::Stack>,
    items: &ItemIcons,
    images: &Assets<Image>,
) {
    let own = stack.and_then(|stack| match stack.item {
        Item::Fish(species) => items.fish.get(species as usize).cloned(),
        Item::Tool(tool) => Some(items.tool(tool)),
        Item::Block(material) => Material::LAMPS
            .iter()
            .position(|&light| light == material)
            .and_then(|index| items.lights.get(index).cloned()),
    });
    if let Some(image) = own {
        node.image = image;
        node.rect = None;
        node.color = Color::WHITE;
        return;
    }
    node.image = items.atlas.clone();
    let art = stack.and_then(|stack| match stack.item {
        Item::Block(material) => thumbnail(material),
        Item::Tool(_) | Item::Fish(_) => None,
    });
    let Some((slot, tile, tint)) = art else {
        node.color = Color::NONE;
        return;
    };
    // The rect is in the image's own pixels, so it needs the loaded size.
    // Until the atlas has loaded there is nothing to crop to, and a
    // full-image icon would be sixteen tiles at once.
    let Some(size) = images.get(&node.image).map(|image| image.size_f32()) else {
        node.color = Color::NONE;
        return;
    };
    let sheet = size / ATLAS_SHEETS;
    let step = sheet / ATLAS_TILES;
    let origin = Vec2::new(
        (slot % ATLAS_SHEETS as u32) as f32 * sheet.x,
        (slot / ATLAS_SHEETS as u32) as f32 * sheet.y,
    );
    let min = origin + Vec2::new(tile.x * step.x, tile.y * step.y);
    // Half a texel, which is all the bake leaves to guard: a tile is exactly
    // 32 texels in the atlas with nothing bleeding into it, where the source
    // sheets had soft edges to keep clear of.
    let inset = step / 64.;
    node.rect = Some(Rect::from_corners(min + inset, min + step - inset));
    node.color = tint;
}

/// Repaint the row from the store. Everything a slot shows is derived here, so
/// the store stays the one place that knows what is carried.
#[allow(clippy::type_complexity)]
pub fn update(
    slots: Res<Hotbar>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    mut cells: Query<(&SlotCell, &mut BorderColor, &mut BackgroundColor)>,
    icons_query: Query<(&SlotIcon, &mut ImageNode)>,
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
    let Some(items) = icons else {
        return;
    };
    let ready = images.contains(&items.atlas)
        && items
            .fish
            .iter()
            .chain(&items.tools)
            .chain(&items.lights)
            .all(|h| images.contains(h));
    if !slots.is_changed() && *art_ready {
        return;
    }
    *art_ready = ready;
    let mut icons = icons_query;
    for (cell, mut border, mut background) in &mut cells {
        let selected = cell.0 == slots.selected();
        *border = BorderColor::all(border_of(selected));
        background.0 = fill_of(selected);
    }
    for (icon, mut node) in &mut icons {
        paint_icon(&mut node, slots.get(icon.0), &items, &images);
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
    mut picker: super::equipment::PickerWheel,
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
    // While G is held the picker has the wheel, and the slots and the zoom
    // do not see it.
    if picker.take(step) {
        return;
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

    /// A grassy block shows its side in a slot, the sward over earth, and
    /// wears its top, side and underside on a drop, as the ground draws it
    /// (survey I4); earth and stone are one tile all round; and a slot is not
    /// tinted, since the ground draws its tiles in their own colours.
    #[test]
    fn a_block_is_shown_as_the_ground_draws_it() {
        let grass = block_art(Material::Grass).unwrap();
        assert_eq!(grass.top, Vec2::new(0., 0.));
        assert_eq!(grass.side, Vec2::new(1., 0.), "the transition tile");
        assert_eq!(grass.under, Vec2::new(2., 0.), "earth underneath");
        assert_eq!(thumbnail(Material::Grass).unwrap().1, grass.side);
        assert_eq!(
            block_art(Material::DryGrass),
            Some(grass),
            "one render code"
        );
        let stone = block_art(Material::Stone).unwrap();
        assert_eq!((stone.top, stone.side), (stone.under, stone.under));
        assert_eq!(thumbnail(Material::Dirt).unwrap().2, Color::WHITE);
    }
}
