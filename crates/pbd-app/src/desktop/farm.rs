//! Farming in play (`farming-and-smelting` decisions 2 to 5): the use key's
//! verbs on the ground, planting from the right button, and the crops and
//! forges drawn.
//!
//! The rules are the core's (`pbd_core::farming`, `pbd_core::station`). What
//! is here is which cell the eye is on, the durable writes, and the meshes.
//!
//! The use key (`E`, which opens doors in towns) on what the eye ray meets:
//! - a station block opens its panel (`super::forge`);
//! - a ripe crop is harvested; an unripe one says what it is and how long;
//! - with the shovel in hand, a grass, dirt, soil or sand top is tilled, and
//!   a grassy one turns up a seed.

use super::digging::{self, Aim, Edited, Hands, apply_edit};
use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::config::{FarmConfig, Recipes};
use pbd_app::drops::Drops;
use pbd_app::field_light::{Faded, LitByField, LitLikeTerrain};
use pbd_app::hotbar::{Gained, How, Planted};
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_core::column;
use pbd_core::farming::{
    self, Climate, Crop, PLOT_KIND, PLOT_SCHEMA, Picked, Plot, Stage, WILD_KIND, WILD_SCHEMA,
};
use pbd_core::inventory::{Item, Tool};
use pbd_core::records::Record as StoredRecord;
use pbd_core::station::{self, STATION_KIND, StationState};
use pbd_core::terrain::Material;
use std::collections::HashMap;

/// A line under the crosshair: what the last use did or why it did not.
#[derive(Resource, Default)]
pub struct FarmNote {
    pub text: String,
    /// Wall seconds (`Time::elapsed`) it goes at.
    until_s: f32,
}

impl FarmNote {
    pub fn say(&mut self, now_s: f32, text: impl Into<String>) {
        self.text = text.into().to_uppercase();
        self.until_s = now_s + 3.0;
        info!("farm: {}", self.text);
    }
}

/// The station whose panel is open: its cell and layer.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct StationAt(pub Option<(u32, u16)>);

/// The note's text node.
#[derive(Component)]
pub struct NoteText;

/// A cell's direction from the planet's centre, by its record.
fn direction_of(fine: &PlanetFine, record: usize) -> Option<Vec3> {
    let cell = fine.set.finest_records().get(record)?;
    Vec3::from_slice(&cell.direction_height[..3]).try_normalize()
}

/// The climate of a cell whose ground is at `layer`.
pub fn climate_at(fine: &PlanetFine, cell: u32, layer: usize) -> Option<Climate> {
    let record = digging::record_of(fine, cell)?;
    let direction = direction_of(fine, record)?;
    let height = column::layer_altitude(layer) + 1.0;
    let biome =
        pbd_core::planet_gen::biome_at(pbd_app::planet::terrain_config(), direction, height);
    Climate::of(biome)
}

/// A cell's plot record, read.
pub fn plot_at(save: &WorldSave, cell: u32) -> Option<Plot> {
    save.records.get(PLOT_KIND, u64::from(cell))?.read()
}

/// The crop growing on `cell`, if its plot holds one and its ground is still
/// tilled soil with air over it (decision 4).
pub fn live_crop(fine: &PlanetFine, save: &WorldSave, cell: u32) -> Option<(Plot, Crop)> {
    let plot = plot_at(save, cell)?;
    let crop = plot.crop()?;
    let layer = usize::from(plot.layer);
    let ground = digging::material_of(fine, save, (cell, layer))?;
    let over = digging::material_of(fine, save, (cell, layer + 1))?;
    (ground == Material::Tilled && over == Material::Air).then_some((plot, crop))
}

fn plot_record(cell: u32, plot: &Plot) -> StoredRecord {
    StoredRecord::of(PLOT_KIND, u64::from(cell), PLOT_SCHEMA, plot)
}

/// Plant the seed in the selected slot on the tilled soil at `cell`/`layer`
/// (decision 3). Refused, with a note, where the ground is not bare tilled
/// soil or the climate does not suit.
pub fn plant(
    world: &mut Edited,
    cell: u32,
    layer: usize,
    seed: pbd_core::goods::Good,
    note: &mut FarmNote,
    wall_s: f32,
    planted: &mut MessageWriter<Planted>,
) -> bool {
    let Some(crop) = Crop::of_seed(seed) else {
        return false;
    };
    let ground = digging::material_of(world.fine, world.save, (cell, layer));
    let over = digging::material_of(world.fine, world.save, (cell, layer + 1));
    if ground != Some(Material::Tilled) || over != Some(Material::Air) {
        note.say(wall_s, "Plant on tilled soil: E with the shovel tills");
        return false;
    }
    if live_crop(world.fine, world.save, cell).is_some() {
        note.say(wall_s, "Something is already growing here");
        return false;
    }
    let climate = climate_at(world.fine, cell, layer);
    if !crop.grows_in(climate) {
        note.say(
            wall_s,
            format!(
                "{} does not grow in a {} climate",
                crop.row().name,
                climate.map_or("barren", Climate::name)
            ),
        );
        return false;
    }
    let mut moved = world.slots.0.clone();
    if moved.take(moved.selected(), 1) != 1 {
        return false;
    }
    let plot = Plot::sown(layer as u16, crop, world.now_s);
    if !world
        .save
        .store_with_hands(plot_record(cell, &plot), PLOT_SCHEMA, &moved)
    {
        error!("plant BLOCKED: the save did not accept it");
        return false;
    }
    world.slots.0 = moved;
    planted.write(Planted { crop });
    note.say(wall_s, format!("Planted {}", crop.row().name));
    true
}

/// Clear a cell's crop after its soil was dug out from under it.
pub fn uproot(save: &mut WorldSave, slots: &pbd_core::inventory::Slots, cell: u32) {
    if let Some(plot) = plot_at(save, cell).filter(|p| p.crop.is_some()) {
        let bare = Plot::bare(plot.layer);
        save.store_with_hands(plot_record(cell, &bare), PLOT_SCHEMA, slots);
        info!("uprooted the crop on cell {cell}");
    }
}

/// The wild plant on `cell`, if one grows there and has not been picked
/// (decision 14): a derived plant on a grassy top with air over it.
pub fn wild_at(
    fine: &PlanetFine,
    save: &WorldSave,
    farm: &FarmConfig,
    cell: u32,
    layer: usize,
) -> Option<Crop> {
    let top = digging::material_of(fine, save, (cell, layer))?;
    let over = digging::material_of(fine, save, (cell, layer + 1))?;
    if !farming::sprouts_seed(top) || over != Material::Air {
        return None;
    }
    if save.records.get(WILD_KIND, u64::from(cell)).is_some() {
        return None;
    }
    farm.0
        .wild_crop(save.identity.seed, cell, climate_at(fine, cell, layer))
}

/// Pick a wild plant: its seeds and one of its produce, and a record that it
/// is picked, in one line.
#[allow(clippy::too_many_arguments)]
fn pick(
    save: &mut WorldSave,
    slots: &mut super::slots::Hotbar,
    farm: &FarmConfig,
    cell: u32,
    crop: Crop,
    now_s: f64,
    note: &mut FarmNote,
    wall_s: f32,
    gained: &mut MessageWriter<Gained>,
) {
    let row = crop.row();
    let seeds = farm.0.wild_seeds;
    let mut moved = slots.0.clone();
    if moved.give(Item::Good(row.seed), seeds) + moved.give(Item::Good(row.produce), 1) > 0 {
        note.say(wall_s, "No room in the pack");
        return;
    }
    let record = StoredRecord::of(
        WILD_KIND,
        u64::from(cell),
        WILD_SCHEMA,
        &Picked { picked_s: now_s },
    );
    if !save.store_with_hands(record, WILD_SCHEMA, &moved) {
        error!("pick BLOCKED: the save did not accept it");
        return;
    }
    slots.0 = moved;
    gained.write(Gained {
        item: Item::Good(row.seed),
        count: seeds,
        how: How::Picked,
    });
    gained.write(Gained {
        item: Item::Good(row.produce),
        count: 1,
        how: How::Picked,
    });
    note.say(wall_s, format!("Picked wild {}: +{seeds} seed", row.name));
}

/// Whether a block may be placed into `cell`/`layer`: not into the air a
/// growing crop stands in.
pub fn crop_in_the_way(fine: &PlanetFine, save: &WorldSave, cell: u32, layer: usize) -> bool {
    live_crop(fine, save, cell).is_some_and(|(plot, _)| usize::from(plot.layer) + 1 == layer)
}

/// A station's record, read; an empty queue where it has none.
pub fn station_state(
    save: &WorldSave,
    cell: u32,
    layer: u16,
    row: &station::StationRow,
) -> StationState {
    save.records
        .get(STATION_KIND, station::record_id(cell, layer))
        .and_then(|r| r.read::<StationState>())
        .filter(|s| s.station == row.code)
        .unwrap_or_else(|| StationState::new(row))
}

/// The use key.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn use_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    aim: Res<Aim>,
    tools: Res<pbd_app::fish::ToolSlot>,
    farm: Res<FarmConfig>,
    mut station_at: ResMut<StationAt>,
    mut note: ResMut<FarmNote>,
    time: Res<Time>,
    (mut gained, mut did): (MessageWriter<Gained>, MessageWriter<pbd_app::missions::Did>),
    (mut fine, mut contact, mut save, mut slots, mut drops, sun): (
        ResMut<PlanetFine>,
        ResMut<PlanetContact>,
        ResMut<WorldSave>,
        ResMut<super::slots::Hotbar>,
        ResMut<Drops>,
        Res<pbd_app::sky::Sun>,
    ),
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    // E shuts a station's panel as it opened it.
    if matches!(*screen, Screen::Forge | Screen::Kitchen) {
        *screen = Screen::Playing;
        return;
    }
    if *screen != Screen::Playing {
        return;
    }
    if !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    let Some(target) = aim.target else {
        return;
    };
    let wall_s = time.elapsed_secs();
    let now_s = sun.clock.seconds;
    let (cell, layer) = (target.dig.cell, target.dig.layer);
    let Some(material) = digging::material_of(&fine, &save, (cell, layer)) else {
        return;
    };
    if let Some(row) = station::station_of(material) {
        station_at.0 = Some((cell, layer as u16));
        // The kitchen's stations have their own panel (`cooking` design S4).
        *screen = super::kitchen::screen_for(row);
        info!("opened the {} at cell {cell} layer {layer}", row.name);
        return;
    }
    if let Some((plot, crop)) =
        live_crop(&fine, &save, cell).filter(|(p, _)| usize::from(p.layer) == layer)
    {
        harvest(
            &mut save,
            &mut slots,
            &farm,
            cell,
            plot,
            crop,
            now_s,
            &mut note,
            wall_s,
            &mut gained,
        );
        return;
    }
    if let Some(crop) = wild_at(&fine, &save, &farm, cell, layer) {
        pick(
            &mut save,
            &mut slots,
            &farm,
            cell,
            crop,
            now_s,
            &mut note,
            wall_s,
            &mut gained,
        );
        return;
    }
    if tools.0.held() != Some(Tool::Shovel) || !farming::tillable(material) {
        return;
    }
    if digging::material_of(&fine, &save, (cell, layer + 1)) != Some(Material::Air) {
        note.say(wall_s, "Nothing tills under a block");
        return;
    }
    let seed = farming::sprouts_seed(material)
        .then(|| farming::seed_turned_up(cell, climate_at(&fine, cell, layer)))
        .flatten();
    let tilled = apply_edit(
        &mut Edited {
            fine: &mut fine,
            contact: &mut contact,
            save: &mut save,
            slots: &mut slots,
            drops: &mut drops,
            now_s,
        },
        Hands::Till(seed.map(Item::Good)),
        cell,
        layer,
        Material::Tilled,
    );
    if tilled.is_some() {
        // What the starter missions count: a cell tilled.
        did.write(pbd_app::missions::Did::block(
            pbd_core::missions::Kind::Till,
            material,
        ));
        if let Some(seed) = seed {
            gained.write(Gained {
                item: Item::Good(seed),
                count: 1,
                how: How::Found,
            });
        }
        match seed {
            Some(seed) => note.say(wall_s, format!("Tilled, and turned up a {}", seed.name())),
            None => note.say(wall_s, "Tilled"),
        }
    }
}

/// Harvest a crop, or say how long it has left.
#[allow(clippy::too_many_arguments)]
fn harvest(
    save: &mut WorldSave,
    slots: &mut super::slots::Hotbar,
    farm: &FarmConfig,
    cell: u32,
    plot: Plot,
    crop: Crop,
    now_s: f64,
    note: &mut FarmNote,
    wall_s: f32,
    gained: &mut MessageWriter<Gained>,
) {
    let row = crop.row();
    if farm.0.stage(crop, plot.planted_s, now_s) != Stage::Ripe {
        let left_h = farm.0.left_s(crop, plot.planted_s, now_s)
            / f64::from(pbd_core::daylight::DAY_S)
            * 24.0;
        note.say(
            wall_s,
            format!("{}: ripe in {:.0} game hours", row.name, left_h.ceil()),
        );
        return;
    }
    let tuning = farm.0.of(crop);
    let mut moved = slots.0.clone();
    let left = moved.give(Item::Good(row.produce), tuning.produce)
        + moved.give(Item::Good(row.seed), tuning.seeds);
    if left > 0 {
        note.say(wall_s, "No room in the pack for the harvest");
        return;
    }
    let bare = Plot::bare(plot.layer);
    if !save.store_with_hands(plot_record(cell, &bare), PLOT_SCHEMA, &moved) {
        error!("harvest BLOCKED: the save did not accept it");
        return;
    }
    slots.0 = moved;
    gained.write(Gained {
        item: Item::Good(row.produce),
        count: tuning.produce,
        how: How::Harvested,
    });
    gained.write(Gained {
        item: Item::Good(row.seed),
        count: tuning.seeds,
        how: How::Harvested,
    });
    note.say(
        wall_s,
        format!("+{} {}, +{} seed", tuning.produce, row.name, tuning.seeds),
    );
}

/// The note under the crosshair.
pub fn spawn_note(mut commands: Commands) {
    commands.spawn((
        pbd_app::ui_kit::KitText::new("", pbd_app::ui_kit::LABEL).outline(pbd_app::ui_kit::INK),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(56),
            margin: UiRect::left(px(-120)),
            width: px(240),
            justify_content: JustifyContent::Center,
            ..default()
        },
        NoteText,
    ));
}

pub fn show_note(
    mut note: ResMut<FarmNote>,
    time: Res<Time>,
    mut text: Query<&mut pbd_app::ui_kit::KitText, With<NoteText>>,
) {
    if !note.text.is_empty() && time.elapsed_secs() > note.until_s {
        note.text.clear();
    }
    for mut words in &mut text {
        if words.text != note.text {
            words.text.clone_from(&note.text);
        }
    }
}

// ---------------------------------------------------------------- drawing

/// How far from the eye crops and forges are drawn, m.
const SHOWN_M: f32 = 64.0;
/// How long a crop takes to fade in or out, s.
const FADE_S: f32 = 0.6;

/// The crops' and forges' meshes and materials, made once.
#[derive(Resource)]
pub struct FarmLooks {
    /// Three clumps of crossed quads, base at the origin, Y up.
    clumps: Handle<Mesh>,
    /// A wild plant: one bush of crossed quads.
    wild: Handle<Mesh>,
    /// By crop, then stage.
    stages: Vec<[Handle<StandardMaterial>; 4]>,
    chimney: Handle<Mesh>,
    iron: Handle<StandardMaterial>,
    flame: Handle<Mesh>,
    fire: Handle<StandardMaterial>,
    /// The workbench's plank top and its vise (`tool-crafting` decision 5):
    /// plain boxes in the forge's chimney's pipeline.
    board: Handle<Mesh>,
    vise: Handle<Mesh>,
    oak: Handle<StandardMaterial>,
}

/// The path of a crop's stage sprite (decision 5).
pub fn stage_sprite(crop: Crop, stage: Stage) -> String {
    format!(
        "items/crops/{}_{}.png",
        crop.row().code.replace('-', "_"),
        stage.index()
    )
}

/// Three clumps, each a pair of crossed quads `size` metres square, on a
/// ring of `ring` metres. Normals point up, as the ground clutter's do, so a
/// clump is lit like the ground under it rather than like a wall.
fn clumps_mesh(size: f32, ring: f32) -> Mesh {
    let (mut positions, mut normals, mut uvs, mut indices) = (
        Vec::<[f32; 3]>::new(),
        Vec::new(),
        Vec::new(),
        Vec::<u32>::new(),
    );
    for k in 0..3 {
        let a = std::f32::consts::FRAC_PI_2 + k as f32 * std::f32::consts::TAU / 3.0;
        let centre = Vec3::new(a.cos() * ring, 0.0, a.sin() * ring);
        for q in 0..2 {
            let turn = a + q as f32 * std::f32::consts::FRAC_PI_2 + 0.4;
            let across = Vec3::new(turn.cos(), 0.0, turn.sin()) * (size * 0.5);
            let base = positions.len() as u32;
            for (corner, uv) in [
                (centre - across, [0.0, 1.0]),
                (centre + across, [1.0, 1.0]),
                (centre + across + Vec3::Y * size, [1.0, 0.0]),
                (centre - across + Vec3::Y * size, [0.0, 0.0]),
            ] {
                positions.push(corner.to_array());
                normals.push([0.0, 1.0, 0.0]);
                uvs.push(uv);
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

/// A sprite cut out by an alpha test, both sides, never blended (M8).
fn cutout(texture: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color_texture: Some(texture),
        alpha_mode: AlphaMode::Mask(0.5),
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 0.95,
        reflectance: 0.1,
        ..default()
    }
}

pub fn make_looks(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stages = Crop::all()
        .map(|crop| {
            [Stage::Sown, Stage::Sprout, Stage::Growing, Stage::Ripe]
                .map(|stage| materials.add(cutout(assets.load(stage_sprite(crop, stage)))))
        })
        .collect();
    let looks = FarmLooks {
        clumps: meshes.add(clumps_mesh(1.1, 0.62)),
        wild: meshes.add(clumps_mesh(0.95, 0.12)),
        stages,
        chimney: meshes.add(Cuboid::new(0.34, 0.9, 0.34)),
        iron: materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.15, 0.14),
            perceptual_roughness: 0.8,
            ..default()
        }),
        flame: meshes.add(clumps_mesh(0.45, 0.0)),
        board: meshes.add(Cuboid::new(1.5, 0.08, 0.32)),
        vise: meshes.add(Cuboid::new(0.22, 0.22, 0.3)),
        oak: materials.add(StandardMaterial {
            base_color: Color::srgb(0.55, 0.38, 0.21),
            perceptual_roughness: 0.9,
            ..default()
        }),
        // The flame draws its own light, unlit, as a lamp's flame does.
        fire: materials.add(StandardMaterial {
            base_color_texture: Some(assets.load("items/crops/flame.png")),
            alpha_mode: AlphaMode::Mask(0.5),
            cull_mode: None,
            double_sided: true,
            unlit: true,
            ..default()
        }),
    };
    // One of each kind of mesh stands hidden under the ground from the first
    // frame, so the loading screen compiles their pipelines (CLAUDE.md, "a
    // loading screen on boot") rather than the first crop planted.
    commands
        .spawn((
            LitByField,
            LitLikeTerrain,
            Transform::default(),
            Visibility::Hidden,
            Prewarm,
        ))
        .with_children(|root| {
            root.spawn((
                Mesh3d(looks.clumps.clone()),
                MeshMaterial3d(looks.stages[0][0].clone()),
            ));
            root.spawn((
                Mesh3d(looks.chimney.clone()),
                MeshMaterial3d(looks.iron.clone()),
            ));
            root.spawn((
                Mesh3d(looks.flame.clone()),
                MeshMaterial3d(looks.fire.clone()),
            ));
        });
    commands.insert_resource(looks);
}

/// The hidden set the loading screen warms from.
#[derive(Component)]
pub struct Prewarm;

/// One crop drawn on one cell.
#[derive(Component)]
pub struct CropView {
    cell: u32,
    /// A wild plant rather than a sown crop.
    wild: bool,
    crop: Crop,
    stage: Stage,
    layer: u16,
    leaving: bool,
}

/// One forge's chimney (and flame).
#[derive(Component)]
pub struct ForgeView {
    cell: u32,
    layer: u16,
    working: bool,
    leaving: bool,
}

#[derive(Component)]
pub struct Flame;

/// Where a cell's top is, and the turn that stands a thing up on it.
fn standing(fine: &PlanetFine, record: usize, layer: usize, turn: f32) -> Option<Transform> {
    let up = direction_of(fine, record)?;
    let at = up * (PLANET_RADIUS + column::layer_altitude(layer) + 1.0);
    let rotation = Quat::from_rotation_arc(Vec3::Y, up) * Quat::from_rotation_y(turn);
    Some(Transform::from_translation(at).with_rotation(rotation))
}

/// Which record each cell key is, rebuilt when the fine set moves.
#[derive(Default)]
pub struct Keys {
    version: u64,
    map: HashMap<u32, usize>,
}

impl Keys {
    fn refresh(&mut self, fine: &PlanetFine) {
        if self.version != fine.version || self.map.is_empty() {
            self.map = fine
                .set
                .finest_records()
                .iter()
                .enumerate()
                .map(|(i, r)| (r.key(), i))
                .collect();
            self.version = fine.version;
        }
    }
}

/// Keep the drawn crops and forges in step with the records and the clock,
/// and fade each in, out and between stages (decision 5).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn show(
    mut commands: Commands,
    looks: Option<Res<FarmLooks>>,
    fine: Res<PlanetFine>,
    save: Res<WorldSave>,
    farm: Res<FarmConfig>,
    recipes: Res<Recipes>,
    sun: Res<pbd_app::sky::Sun>,
    time: Res<Time>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut crops: Query<(Entity, &mut CropView, &mut Faded), Without<ForgeView>>,
    mut forges: Query<(Entity, &mut ForgeView, &mut Faded, &Children), Without<CropView>>,
    mut flames: Query<&mut Visibility, With<Flame>>,
    mut keys: Local<Keys>,
    mut since: Local<f32>,
) {
    let Some(looks) = looks else { return };
    let dt = time.delta_secs();
    let step = dt / FADE_S;
    // Fades run every frame.
    for (entity, view, mut faded) in &mut crops {
        let goal = if view.leaving { 1.0 } else { 0.0 };
        faded.0 = (faded.0 + (goal - faded.0).clamp(-step, step)).clamp(0.0, 1.0);
        if view.leaving && faded.0 >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    for (entity, view, mut faded, _) in &mut forges {
        let goal = if view.leaving { 1.0 } else { 0.0 };
        faded.0 = (faded.0 + (goal - faded.0).clamp(-step, step)).clamp(0.0, 1.0);
        if view.leaving && faded.0 >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    *since += dt;
    if *since < 0.25 {
        return;
    }
    *since = 0.0;
    let Some(eye) = cameras
        .iter()
        .find(|(_, c)| c.is_active)
        .map(|(t, _)| t.translation())
    else {
        return;
    };
    keys.refresh(&fine);
    let now_s = sun.clock.seconds;
    let near = |record: usize, layer: usize| -> Option<Transform> {
        let t = standing(&fine, record, layer, 0.0)?;
        (t.translation.distance(eye) < SHOWN_M).then_some(t)
    };
    // The crops that should stand.
    let mut wanted: HashMap<u32, (Crop, Stage, u16, usize, bool)> = HashMap::new();
    for record in save.records.of_kind(PLOT_KIND) {
        let Some(plot) = record.read::<Plot>() else {
            continue;
        };
        let Some(crop) = plot.crop() else { continue };
        let cell = record.id as u32;
        let Some(&index) = keys.map.get(&cell) else {
            continue;
        };
        let Some(column) = fine.set.columns.column(index) else {
            continue;
        };
        let layer = usize::from(plot.layer);
        if column.material(layer) != Material::Tilled || column.material(layer + 1) != Material::Air
        {
            continue;
        }
        if near(index, layer).is_none() {
            continue;
        }
        let stage = farm.0.stage(crop, plot.planted_s, now_s);
        wanted.insert(cell, (crop, stage, plot.layer, index, false));
    }
    // The wild plants (decision 14): derived, on the grassy tops near the
    // eye that have not been picked.
    let reach = (SHOWN_M / PLANET_RADIUS).cos();
    let toward = eye.normalize_or(Vec3::Y);
    for (index, record) in fine.set.finest_records().iter().enumerate() {
        let up = Vec3::from_slice(&record.direction_height[..3]);
        if up.dot(toward) < reach * up.length() {
            continue;
        }
        let Some(column) = fine.set.columns.column(index) else {
            continue;
        };
        let Some(top) = column.surface() else {
            continue;
        };
        if !farming::sprouts_seed(column.material(top)) || column.material(top + 1) != Material::Air
        {
            continue;
        }
        let cell = record.key();
        if wanted.contains_key(&cell) || save.records.get(WILD_KIND, u64::from(cell)).is_some() {
            continue;
        }
        let height = column::layer_altitude(top) + 1.0;
        let biome = pbd_core::planet_gen::biome_at(
            pbd_app::planet::terrain_config(),
            up.normalize_or(Vec3::Y),
            height,
        );
        if let Some(crop) = farm
            .0
            .wild_crop(save.identity.seed, cell, Climate::of(biome))
            && near(index, top).is_some()
        {
            wanted.insert(cell, (crop, Stage::Ripe, top as u16, index, true));
        }
    }
    for (_, mut view, _) in &mut crops {
        if view.leaving {
            continue;
        }
        match wanted.get(&view.cell) {
            Some(&(crop, stage, layer, _, wild))
                if (crop, stage, layer, wild) == (view.crop, view.stage, view.layer, view.wild) =>
            {
                wanted.remove(&view.cell);
            }
            _ => view.leaving = true,
        }
    }
    for (cell, (crop, stage, layer, index, wild)) in wanted {
        let turn = (cell % 360) as f32 * std::f32::consts::PI / 180.0;
        let Some(transform) = standing(&fine, index, usize::from(layer), turn) else {
            continue;
        };
        commands
            .spawn((
                transform,
                Visibility::default(),
                LitByField,
                LitLikeTerrain,
                Faded(1.0),
                CropView {
                    cell,
                    wild,
                    crop,
                    stage,
                    layer,
                    leaving: false,
                },
            ))
            .with_child((
                Mesh3d(if wild {
                    looks.wild.clone()
                } else {
                    looks.clumps.clone()
                }),
                MeshMaterial3d(looks.stages[crop.index()][stage.index()].clone()),
            ));
    }
    // The forges and workbenches: every such block an edit placed, near the
    // eye. A campfire is a light, drawn by the column pass (`cooking` S2).
    let mut wanted: HashMap<(u32, u16), (usize, bool, Material)> = HashMap::new();
    for edit in save.edits.all() {
        if !matches!(edit.material, Material::Forge | Material::Workbench) {
            continue;
        }
        let Some(&index) = keys.map.get(&edit.cell) else {
            continue;
        };
        let Some(column) = fine.set.columns.column(index) else {
            continue;
        };
        let layer = usize::from(edit.layer);
        if column.material(layer) != edit.material || near(index, layer).is_none() {
            continue;
        }
        let row = station::station_of(edit.material).expect("checked");
        let working = station_state(&save, edit.cell, edit.layer, row).working(&recipes.0, now_s);
        wanted.insert((edit.cell, edit.layer), (index, working, edit.material));
    }
    for (_, mut view, _, children) in &mut forges {
        if view.leaving {
            continue;
        }
        match wanted.remove(&(view.cell, view.layer)) {
            Some((_, working, _)) => {
                view.working = working;
                for child in children.iter() {
                    if let Ok(mut shown) = flames.get_mut(child) {
                        let want = if working {
                            Visibility::Inherited
                        } else {
                            Visibility::Hidden
                        };
                        if *shown != want {
                            *shown = want;
                        }
                    }
                }
            }
            None => view.leaving = true,
        }
    }
    for ((cell, layer), (index, working, material)) in wanted {
        let Some(transform) = standing(&fine, index, usize::from(layer), 0.0) else {
            continue;
        };
        if material == Material::Workbench {
            // Two planks laid across the top and an iron vise at one end,
            // after M3's workbench icon.
            commands
                .spawn((
                    transform,
                    Visibility::default(),
                    LitByField,
                    LitLikeTerrain,
                    Faded(1.0),
                    ForgeView {
                        cell,
                        layer,
                        working: false,
                        leaving: false,
                    },
                ))
                .with_children(|bench| {
                    for z in [-0.18, 0.18] {
                        bench.spawn((
                            Mesh3d(looks.board.clone()),
                            MeshMaterial3d(looks.oak.clone()),
                            Transform::from_xyz(0.0, 0.04, z),
                        ));
                    }
                    bench.spawn((
                        Mesh3d(looks.vise.clone()),
                        MeshMaterial3d(looks.iron.clone()),
                        Transform::from_xyz(0.6, 0.19, 0.0),
                    ));
                });
            continue;
        }
        // The chimney stands off the centre, toward one corner, as M3's
        // forge icon has it; the flame in its mouth while a job runs.
        let off = Vec3::new(0.55, 0.45, 0.25);
        commands
            .spawn((
                transform,
                Visibility::default(),
                LitByField,
                LitLikeTerrain,
                Faded(1.0),
                ForgeView {
                    cell,
                    layer,
                    working,
                    leaving: false,
                },
            ))
            .with_children(|forge| {
                forge.spawn((
                    Mesh3d(looks.chimney.clone()),
                    MeshMaterial3d(looks.iron.clone()),
                    Transform::from_translation(off),
                ));
                forge.spawn((
                    Mesh3d(looks.flame.clone()),
                    MeshMaterial3d(looks.fire.clone()),
                    Transform::from_translation(off + Vec3::Y * 0.42),
                    if working {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    Flame,
                ));
            });
    }
}

// ---------------------------------------------------------------- capture rig

/// `--farm` and `--forge` (`farming-and-smelting` task 5.1): a measurement
/// instrument for the headless captures. A capture has no hands, so this
/// does through the same durable paths what a player would: till, sow at
/// staggered world times (so the rows show every stage), turn up a seed,
/// stand a lantern by the field; place a forge and queue two jobs, the first
/// part done. The capture's save is memory-only, so nothing is kept.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    launch: Res<super::Launch>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    refresh: Res<pbd_app::planet::LodRefresh>,
    (mut fine, mut contact, mut save, mut slots, mut drops, sun): (
        ResMut<PlanetFine>,
        ResMut<PlanetContact>,
        ResMut<WorldSave>,
        ResMut<super::slots::Hotbar>,
        ResMut<Drops>,
        Res<pbd_app::sky::Sun>,
    ),
    (farm, recipes, mut station_at, mut screen, mut note, time, mut gained): (
        Res<FarmConfig>,
        Res<Recipes>,
        ResMut<StationAt>,
        ResMut<Screen>,
        ResMut<FarmNote>,
        Res<Time>,
        MessageWriter<Gained>,
    ),
    mut done: Local<bool>,
) {
    if *done || launch.capture.is_none() || (launch.farm.is_none() && launch.forge.is_none()) {
        return;
    }
    if refresh.in_flight_s().is_some() {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = transform.translation();
    let up = eye.normalize_or(Vec3::Y);
    let forward = transform.forward().as_vec3();
    let ahead = (forward - up * forward.dot(up)).normalize_or(Vec3::X);
    let right = ahead.cross(up).normalize_or(Vec3::Z);
    // The cell `metres` ahead and `across` to the right: its key and the
    // top solid layer.
    let spot = |fine: &PlanetFine,
                contact: &PlanetContact,
                metres: f32,
                across: f32|
     -> Option<(u32, usize)> {
        let offset = ahead * metres + right * across;
        let angle = offset.length() / PLANET_RADIUS;
        let direction = (up * angle.cos() + offset.normalize_or(ahead) * angle.sin()).normalize();
        let record = contact.finest_cell(direction)?;
        let key = fine.set.finest_records().get(record)?.key();
        let top = fine.set.columns.column(record)?.surface()?;
        Some((key, top))
    };
    let first = launch.farm.or(launch.forge).unwrap_or(6.0);
    if spot(&fine, &contact, first, 0.0).is_none() {
        return;
    }
    *done = true;
    let now_s = sun.clock.seconds;
    let wall_s = time.elapsed_secs();
    let day = f64::from(pbd_core::daylight::DAY_S);
    macro_rules! world {
        () => {
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut save,
                slots: &mut slots,
                drops: &mut drops,
                now_s,
            }
        };
    }
    if let Some(metres) = launch.farm {
        const STEP: f32 = 2.9;
        let mut used = Vec::new();
        let mut ripe = Vec::new();
        for (row, crop) in [Crop::WHEAT, Crop::TURNIP].into_iter().enumerate() {
            for (col, share) in [0.1, 0.4, 0.8, 1.2].into_iter().enumerate() {
                let mut across = (col as f32 - 1.5) * STEP;
                let mut found = None;
                for _ in 0..12 {
                    match spot(&fine, &contact, metres + row as f32 * STEP, across) {
                        Some((cell, top)) if !used.contains(&cell) => {
                            found = Some((cell, top));
                            break;
                        }
                        _ => across += 0.5,
                    }
                }
                let Some((cell, top)) = found else { continue };
                used.push(cell);
                let material = digging::material_of(&fine, &save, (cell, top));
                if material.is_some_and(farming::tillable) {
                    apply_edit(world!(), Hands::Till(None), cell, top, Material::Tilled);
                }
                if digging::material_of(&fine, &save, (cell, top)) != Some(Material::Tilled) {
                    warn!("scripted farm: cell {cell} would not till ({material:?})");
                    continue;
                }
                let planted = now_s - share * f64::from(farm.0.of(crop).days) * day;
                let plot = Plot::sown(top as u16, crop, planted);
                save.store_with_hands(plot_record(cell, &plot), PLOT_SCHEMA, &slots.0);
                if share >= 1.0 {
                    ripe.push((cell, plot, crop));
                }
                info!(
                    "scripted farm: {} at cell {cell}, {:.0}% grown",
                    crop.row().name,
                    share * 100.0
                );
            }
        }
        // A grassy cell tilled before the field, so its seed floats up.
        for across in [0.0, 1.0, -1.0, 2.0] {
            if let Some((cell, top)) = spot(&fine, &contact, metres - 2.4, across)
                && !used.contains(&cell)
                && digging::material_of(&fine, &save, (cell, top))
                    .is_some_and(farming::sprouts_seed)
            {
                let seed = farming::seed_turned_up(cell, climate_at(&fine, cell, top))
                    .unwrap_or(pbd_core::goods::Good::WHEAT_SEED);
                apply_edit(
                    world!(),
                    Hands::Till(Some(Item::Good(seed))),
                    cell,
                    top,
                    Material::Tilled,
                );
                info!(
                    "scripted farm: tilled grass at cell {cell}, turned up a {}",
                    seed.name()
                );
                break;
            }
        }
        // Where the wild plants round the camera are, so a shot can be
        // turned onto one (`--yaw` is the bearing plus the camera's own).
        for (index, record) in fine.set.finest_records().iter().enumerate() {
            let Some(dir) = direction_of(&fine, index) else {
                continue;
            };
            let off = dir * PLANET_RADIUS - up * PLANET_RADIUS;
            let (fwd, side) = (off.dot(ahead), off.dot(right));
            let d = fwd.hypot(side);
            if d > 25.0 {
                continue;
            }
            let Some(top) = fine.set.columns.column(index).and_then(|c| c.surface()) else {
                continue;
            };
            if let Some(crop) = wild_at(&fine, &save, &farm, record.key(), top) {
                info!(
                    "scripted farm: a wild {} {d:.1} m away, {:.0} degrees right of the view",
                    crop.row().name,
                    side.atan2(fwd).to_degrees()
                );
            }
        }
        // A street lantern at the field's end, for the night shot.
        if let Some((cell, top)) = spot(&fine, &contact, metres + 0.5 * STEP, 2.0 * STEP) {
            apply_edit(world!(), Hands::Empty, cell, top + 1, Material::LanternPost);
        }
        if launch.farm_harvest {
            for (cell, plot, crop) in ripe {
                harvest(
                    &mut save,
                    &mut slots,
                    &farm,
                    cell,
                    plot,
                    crop,
                    now_s,
                    &mut note,
                    wall_s,
                    &mut gained,
                );
            }
            // And the nearest wild plant within a short walk.
            let mut nearest: Option<(f32, u32, Crop)> = None;
            for (index, record) in fine.set.finest_records().iter().enumerate() {
                let Some(dir) = direction_of(&fine, index) else {
                    continue;
                };
                let d = dir.angle_between(up) * PLANET_RADIUS;
                if d > 30.0 || nearest.is_some_and(|n| n.0 <= d) {
                    continue;
                }
                let Some(top) = fine.set.columns.column(index).and_then(|c| c.surface()) else {
                    continue;
                };
                if let Some(crop) = wild_at(&fine, &save, &farm, record.key(), top) {
                    nearest = Some((d, record.key(), crop));
                }
            }
            match nearest {
                Some((d, cell, crop)) => {
                    info!(
                        "scripted farm: picking the wild {} {d:.1} m away",
                        crop.row().name
                    );
                    pick(
                        &mut save,
                        &mut slots,
                        &farm,
                        cell,
                        crop,
                        now_s,
                        &mut note,
                        wall_s,
                        &mut gained,
                    );
                }
                None => warn!("scripted farm: no wild plant within 30 m"),
            }
        }
    }
    if let Some(metres) = launch.forge {
        let Some((cell, top)) = spot(&fine, &contact, metres, 0.0) else {
            return;
        };
        let layer = top + 1;
        apply_edit(world!(), Hands::Empty, cell, layer, Material::Forge);
        slots.give(Item::Block(Material::Ore), 12);
        slots.give(Item::Good(pbd_core::goods::Good::COAL), 8);
        let at = (cell, layer as u16);
        // The first job three seconds in, the second waiting behind it.
        for at_s in [now_s - 3.0, now_s] {
            super::forge::queue(
                &mut save,
                &mut slots,
                &recipes,
                at,
                "mixed-ore",
                at_s,
                &mut note,
                wall_s,
            );
        }
        if launch.forge_panel {
            station_at.0 = Some(at);
            *screen = Screen::Forge;
        }
        info!("scripted forge at cell {cell} layer {layer}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every crop's four stage sprites are on disk, 32 px square, and the
    /// forge's flame too.
    #[test]
    fn every_stage_has_its_sprite() {
        let assets = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        for crop in Crop::all() {
            for stage in [Stage::Sown, Stage::Sprout, Stage::Growing, Stage::Ripe] {
                let path = assets.join(stage_sprite(crop, stage));
                assert!(path.exists(), "{}", path.display());
            }
        }
        assert!(assets.join("items/crops/flame.png").exists());
    }

    /// Every good's icon is on disk.
    #[test]
    fn every_good_has_its_icon() {
        let assets = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        for good in pbd_core::goods::Good::all() {
            assert!(assets.join(good.icon()).exists(), "{}", good.icon());
        }
    }

    /// The clumps mesh is three pairs of quads with up normals.
    #[test]
    fn a_crop_is_three_crossed_clumps() {
        let mesh = clumps_mesh(1.0, 0.5);
        assert_eq!(mesh.count_vertices(), 3 * 2 * 4);
    }
}
