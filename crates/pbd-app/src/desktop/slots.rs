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
use pbd_app::ui_kit::Localized;
use pbd_app::ui_kit::{self, KitText, Set, Tooltip, UiKit};
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
    use pbd_app::planet::tileset_slot;
    // Which SHEET a block comes from: its home (`pbd_core::column::home_biome`:
    // sand a beach's, snow the tundra's, jungle sward the jungle's, rock and
    // the forge's bricks the mountains', the rest the home meadow's). The
    // ground draws a PLACED block from the same table
    // (`placed-blocks-keep-material`), so the slot and the block put down are
    // one picture wherever it lands. The tiles are each sheet's layout: #0 its
    // ground, #1 that ground over the earth, #2 the earth, #3 the stone.
    let home = pbd_core::column::home_biome(material)?;
    // The sea's cells are drawn from a sheet of their own (B24).
    let sheet = pbd_core::column::own_sheet(material)
        .map_or_else(|| tileset_slot(home), pbd_app::planet::sheet_index);
    let one = |x: f32, y: f32| BlockArt {
        sheet,
        top: Vec2::new(x, y),
        side: Vec2::new(x, y),
        under: Vec2::new(x, y),
    };
    let grassy = BlockArt {
        sheet,
        top: Vec2::new(0., 0.),
        side: Vec2::new(1., 0.),
        under: Vec2::new(2., 0.),
    };
    Some(match material {
        // Grass and dry grass are one render code: the ground draws them alike.
        Material::Grass | Material::DryGrass | Material::JungleGrass => grassy,
        Material::Soil | Material::Dirt => one(2., 0.),
        Material::Sand | Material::DesertSand | Material::Snow => one(0., 0.),
        Material::Stone | Material::Rock => one(3., 0.),
        Material::Ore => one(0., 1.),
        Material::Water | pbd_core::flowing_water!() => one(2., 2.),
        // The fields sheet's own tilled soil over its earth, as the ground
        // draws it (`farming-and-smelting` decision 2).
        Material::Tilled => BlockArt {
            sheet,
            top: Vec2::new(2., 3.),
            side: Vec2::new(2., 0.),
            under: Vec2::new(2., 0.),
        },
        // Slate bricks all round (decision 8).
        Material::Forge => one(1., 3.),
        // Cut oak boards (`tool-crafting` decision 5); its slot draws M3's
        // own picture, so this is only the floating drop's face.
        Material::Workbench | Material::Carpenter | Material::DraftingTable => one(1., 3.),
        // The gravel stations (`townsfolk` design 41): their blocks are the
        // workbench's timber; their slots draw their own pictures.
        Material::Grinder | Material::Mixer => one(1., 3.),
        // Survival's stations: their slots draw their own pictures.
        Material::TanningRack | Material::Still => one(1., 3.),
        // A chest (`storage-chests` design 1): the workbench's boards; its
        // slot draws its own picture.
        Material::Chest => one(1., 3.),
        // The sea's cells (`ocean-environments` B24), each its sheet's tile
        // all round, as the ground draws it.
        Material::ReefRock | Material::Basalt => one(3., 0.),
        Material::SeaIce => one(2., 1.),
        Material::MusselBed => one(3., 1.),
        // A beaver's sticks (B25): the meadow sheet's bark.
        Material::Sticks => one(2., 1.),
        Material::Air
        | Material::Torch
        | Material::LanternPost
        | Material::LanternWall
        | Material::LanternHanging
        | Material::Brazier
        | Material::Candle
        | Material::Campfire => return None,
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
    /// Bare hands, in the tool slot and the picker.
    pub hand: Handle<Image>,
    /// In `Material::LAMPS` order.
    pub lights: Vec<Handle<Image>>,
    /// The roster's names, by roster index, for a fish's tooltip.
    pub fish_names: Vec<String>,
    /// Every good's icon, in goods-table order (`pbd_core::goods`).
    pub goods: Vec<Handle<Image>>,
    /// The forge's own picture, M3's, for its block in a slot.
    pub forge: Handle<Image>,
    /// The workbench's, M3's (`tool-crafting`).
    pub workbench: Handle<Image>,
    /// The carpenter's bench's (`furniture` design 1).
    pub carpenter: Handle<Image>,
    /// The drafting table's (`building-planner` design 3).
    pub drafting: Handle<Image>,
    /// The grinder's and the mixer's (`townsfolk` design 41).
    pub grinder: Handle<Image>,
    pub mixer: Handle<Image>,
    /// The tanning rack's and the still's (`survival-mode` design 17, 18).
    pub tanning_rack: Handle<Image>,
    pub still: Handle<Image>,
    /// A chest's (`storage-chests` design 1).
    pub chest: Handle<Image>,
    /// Each tool at stone, bronze and iron, in `Tool::ALL` order; steel
    /// draws as iron until it is made. The rod's three are the rod.
    pub tiered: Vec<[Handle<Image>; 3]>,
}

impl ItemIcons {
    pub fn tool(&self, tool: pbd_core::inventory::Tool) -> Handle<Image> {
        self.tools[tool.index()].clone()
    }

    /// What the tool slot shows for what is in hand: a tool, or bare hands.
    pub fn in_hand(&self, held: Option<pbd_core::inventory::Tool>) -> Handle<Image> {
        held.map_or_else(|| self.hand.clone(), |tool| self.tool(tool))
    }

    /// A tool's picture at its tier (`tool-crafting`).
    pub fn tool_at(
        &self,
        tool: pbd_core::inventory::Tool,
        tier: pbd_core::inventory::Tier,
    ) -> Handle<Image> {
        let k = usize::from(tier.digit()).min(2);
        self.tiered
            .get(tool.index())
            .map_or_else(|| self.tool(tool), |row| row[k].clone())
    }

    /// What the tool slot shows for the tool in hand at its tier.
    pub fn in_hand_at(&self, equipment: &pbd_core::inventory::Equipment) -> Handle<Image> {
        match equipment.held() {
            Some(tool) => equipment
                .tier(tool)
                .map_or_else(|| self.tool(tool), |tier| self.tool_at(tool, tier)),
            None => self.hand.clone(),
        }
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
    let roster = fauna.0.roster(&body.0);
    let fish = roster
        .iter()
        .map(|species| assets.load(pbd_app::fish::species_icon(species)))
        .collect();
    let fish_names = roster
        .iter()
        .map(|species| species.name().into_string())
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
        hand: assets.load(pbd_app::fish::HAND_ICON),
        lights,
        fish_names,
        goods: pbd_core::goods::Good::all()
            .map(|good| assets.load(good.icon()))
            .collect(),
        forge: assets.load("items/stations/forge.png"),
        workbench: assets.load("items/stations/workbench.png"),
        carpenter: assets.load("items/stations/carpenter.png"),
        drafting: assets.load("items/stations/drafting-table.png"),
        grinder: assets.load("items/stations/grinder.png"),
        mixer: assets.load("items/stations/mixer.png"),
        tanning_rack: assets.load("items/stations/tanning-rack.png"),
        still: assets.load("items/stations/still.png"),
        chest: assets.load("items/stations/chest.png"),
        tiered: pbd_core::inventory::Tool::ALL
            .iter()
            .map(|tool| {
                ["stone", "bronze", "iron"].map(|tier| match tool {
                    pbd_core::inventory::Tool::Rod => assets.load(pbd_app::fish::tool_icon(*tool)),
                    _ => assets.load(format!("items/tools/{tier}-{}.png", tool.code())),
                })
            })
            .collect(),
    });
}

/// The hotbar row, which build mode hides while its palette stands in it
/// (`player-building` design B5).
#[derive(Component)]
pub struct HotbarRow;

/// Marks the slot at this index, so the update can find it without a lookup.
#[derive(Component)]
pub struct SlotCell(pub usize);
/// The thumbnail inside a slot.
#[derive(Component)]
pub struct SlotIcon(pub usize);
/// The stack count badge.
#[derive(Component)]
pub struct SlotCount(pub usize);

/// A slot, native pixels: the dressed-stone well of `slot-idle` (24 x 24)
/// with its 16 px icon floor at 4, 4, the item PNGs' own size.
pub const SLOT: f32 = 24.0;
pub const ICON_AT: f32 = 4.0;
pub const ICON: f32 = 16.0;
/// The hotbar row stands this far off the bottom of the window.
pub const HOTBAR_BOTTOM: f32 = 10.0;

/// The stone slot piece for a slot that is or is not the selected one: the
/// selected wears the rune (design 5).
pub fn slot_piece(selected: bool) -> &'static str {
    if selected {
        "slot-selected"
    } else {
        "slot-idle"
    }
}

/// Build the row. One parent, ten children, each a stone slot holding an
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
    kit: Res<UiKit>,
    atlas: Res<pbd_app::planet::PlanetArt>,
    existing: Query<(), With<SlotCell>>,
) {
    if !existing.is_empty() {
        return;
    }
    let atlas = atlas.0.clone();
    let commands = &mut commands;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(HOTBAR_BOTTOM),
                left: percent(50),
                margin: UiRect::left(px(-(SLOTS as f32 * SLOT) / 2.0)),
                ..default()
            },
            super::menu::InPlayHud,
            HotbarRow,
        ))
        .with_children(|row| {
            for index in 0..SLOTS {
                // Hovered while the pack has the pointer, for its tooltip.
                cell(row, &kit, index, &atlas, Interaction::default());
            }
        });
}

/// One slot: the stone well holding an icon and a count, marked with its
/// index so [`update`] paints it. The hotbar row and the pack's grid are
/// built from these, with `extra` on the slot (the pack's are buttons).
pub fn cell(
    row: &mut ChildSpawnerCommands,
    kit: &UiKit,
    index: usize,
    atlas: &Handle<Image>,
    extra: impl Bundle,
) {
    row.spawn((
        Node {
            width: px(SLOT),
            height: px(SLOT),
            flex_shrink: 0.0,
            ..default()
        },
        kit.whole(Set::Stone, slot_piece(index == 0)),
        SlotCell(index),
        Tooltip::default(),
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
                position_type: PositionType::Absolute,
                left: px(ICON_AT),
                top: px(ICON_AT),
                width: px(ICON),
                height: px(ICON),
                ..default()
            },
            SlotIcon(index),
        ));
        cell.spawn((
            // In the slot's bottom-right corner, over the rim rather than the
            // icon, outlined in ink so it reads on any tile.
            KitText::new(Localized::empty(), ui_kit::LABEL).outline(ui_kit::INK),
            Node {
                position_type: PositionType::Absolute,
                bottom: px(1),
                right: px(1),
                ..default()
            },
            SlotCount(index),
        ));
    });
}

/// An item's own picture, whole and untinted: a fish's, a tool's, a light's,
/// a good's, a station block's. `None` for a terrain block, which is drawn
/// from the ground's atlas. The one table for a slot and a drop
/// (`drop-and-trash` 5).
pub fn own_icon(item: Item, items: &ItemIcons) -> Option<Handle<Image>> {
    match item {
        Item::Fish(species) => items.fish.get(species as usize).cloned(),
        Item::Tool(tool) => Some(items.tool(tool)),
        Item::Block(Material::Forge) => Some(items.forge.clone()),
        Item::Block(Material::Workbench) => Some(items.workbench.clone()),
        Item::Block(Material::Carpenter) => Some(items.carpenter.clone()),
        Item::Block(Material::DraftingTable) => Some(items.drafting.clone()),
        Item::Block(Material::Grinder) => Some(items.grinder.clone()),
        Item::Block(Material::Mixer) => Some(items.mixer.clone()),
        Item::Block(Material::TanningRack) => Some(items.tanning_rack.clone()),
        Item::Block(Material::Still) => Some(items.still.clone()),
        Item::Block(Material::Chest) => Some(items.chest.clone()),
        Item::Block(material) => Material::LAMPS
            .iter()
            .position(|&light| light == material)
            .and_then(|index| items.lights.get(index).cloned()),
        Item::Good(good) => items.goods.get(good.index()).cloned(),
    }
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
    let own = stack.and_then(|stack| own_icon(stack.item, items));
    if let Some(image) = own {
        node.image = image;
        node.rect = None;
        node.color = Color::WHITE;
        return;
    }
    node.image = items.atlas.clone();
    let art = stack.and_then(|stack| match stack.item {
        Item::Block(material) => thumbnail(material),
        Item::Tool(_) | Item::Fish(_) | Item::Good(_) => None,
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

/// What a slot's tooltip says: nothing for an empty slot; a tool's name and
/// what it is for; a fish's species; a block's name and how many.
pub fn stack_tip(stack: Option<pbd_core::inventory::Stack>, items: &ItemIcons) -> Localized {
    let Some(stack) = stack else {
        return Localized::empty();
    };
    let name = match stack.item {
        Item::Tool(tool) => {
            return pbd_core::locale::tr_with(
                "slots.tool_tip",
                &[("tool", &tool.name()), ("purpose", &tool.purpose())],
            );
        }
        Item::Fish(species) => items
            .fish_names
            .get(species as usize)
            .map(|name| Localized::verbatim(name.clone()))
            .unwrap_or_else(|| stack.item.name()),
        Item::Block(_) | Item::Good(_) => stack.item.name(),
    };
    let name = pbd_core::locale::sentence(&name);
    if stack.count > 1 {
        pbd_core::locale::tr_with("slots.many", &[("name", &name), ("count", &stack.count)])
    } else {
        name
    }
}

/// Repaint the row from the store. Everything a slot shows is derived here, so
/// the store stays the one place that knows what is carried.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update(
    slots: Res<Hotbar>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    kit: Res<UiKit>,
    mut cells: Query<(&SlotCell, &mut ImageNode), Without<SlotIcon>>,
    icons_query: Query<(&SlotIcon, &mut ImageNode), Without<SlotCell>>,
    mut counts: Query<(&SlotCount, &mut KitText)>,
    mut tips: Query<(&SlotCell, &mut Tooltip)>,
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
    for (cell, mut image) in &mut cells {
        let want = kit.image(slot_piece(cell.0 == slots.selected()));
        if image.image != want {
            image.image = want;
        }
    }
    for (icon, mut node) in &mut icons {
        paint_icon(&mut node, slots.get(icon.0), &items, &images);
    }
    for (cell, mut tip) in &mut tips {
        let want = stack_tip(slots.get(cell.0), &items);
        if tip.0 != want {
            tip.0 = want;
        }
    }
    for (count, mut text) in &mut counts {
        let label = match slots.get(count.0) {
            Some(stack) if stack.count > 1 => Localized::verbatim(stack.count.to_string()),
            _ => Localized::empty(),
        };
        if text.text != label {
            text.text = label;
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
    road_menu: Option<ResMut<pbd_app::roads::planner::PlannerMenu>>,
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
    // While X is held with stakes in hand the road planner's menu has it.
    if let Some(mut menu) = road_menu
        && menu.open
    {
        menu.step += step;
        return;
    }
    if walking && step != 0 && !held {
        slots.step(step);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `placed-blocks-keep-material`: a block's slot and the block put down
    /// on the ground are one picture. The sheet the slot crops is the sheet
    /// the ground draws the placed block from, for every block.
    #[test]
    fn a_slot_and_a_placed_block_share_a_sheet() {
        for material in Material::ALL {
            let mut column = pbd_core::column::Column::bedrock();
            assert!(column.place(10, material));
            let ground = pbd_app::planet::column::placed_sheet(&column, 10);
            match block_art(material) {
                Some(art) => assert_eq!(art.sheet + 1, ground, "{material:?}"),
                None => assert_eq!(ground, 0, "{material:?} has no block picture"),
            }
        }
    }

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

    /// A slot's tooltip: nothing when empty, a block's name and count, a
    /// tool's name and purpose, a fish's species.
    #[test]
    fn a_slot_says_what_it_holds() {
        use pbd_core::inventory::{Stack, Tool};
        let items = ItemIcons {
            atlas: Handle::default(),
            fish: Vec::new(),
            tools: Vec::new(),
            hand: Handle::default(),
            lights: Vec::new(),
            fish_names: vec!["Perch".into(), "Char".into()],
            goods: Vec::new(),
            forge: Handle::default(),
            workbench: Handle::default(),
            carpenter: Handle::default(),
            drafting: Handle::default(),
            grinder: Handle::default(),
            mixer: Handle::default(),
            tanning_rack: Handle::default(),
            still: Handle::default(),
            chest: Handle::default(),
            tiered: Vec::new(),
        };
        assert_eq!(stack_tip(None, &items), "");
        let stone = Stack {
            item: Item::Block(Material::Stone),
            count: 12,
        };
        assert_eq!(stack_tip(Some(stone), &items), "Stone x 12");
        let one = Stack { count: 1, ..stone };
        assert_eq!(stack_tip(Some(one), &items), "Stone");
        let rod = Stack {
            item: Item::Tool(Tool::Rod),
            count: 1,
        };
        assert_eq!(
            stack_tip(Some(rod), &items),
            "Fishing rod: cast, hook, reel"
        );
        let char_ = Stack {
            item: Item::Fish(1),
            count: 3,
        };
        assert_eq!(stack_tip(Some(char_), &items), "Char x 3");
        assert_eq!(
            super::super::equipment::hand_tip(&pbd_core::inventory::Equipment::new_world()),
            "Bare hands: digs soft ground"
        );
    }
}
