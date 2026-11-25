//! Farming in play (`farming-and-smelting` decisions 2 to 5): the interact
//! key's verbs on the ground, planting from the right button, and the crops
//! and forges drawn.
//!
//! The rules are the core's (`pbd_core::farming`, `pbd_core::station`). What
//! is here is which cell the eye is on, the durable writes, and the meshes.
//!
//! The interact key (`F`, `pbd_app::interact`) on what the eye ray meets
//! ([`farm_act`], one decision for the key and its prompt):
//! - a station block opens its panel (`super::forge`);
//! - a ripe crop is harvested; an unripe one says what it is and how long;
//! - a wild plant is picked;
//! - with the shovel in hand, a grass, dirt, soil or sand top is tilled, and
//!   a grassy one turns up a seed.
//!
//! A crop or a wild plant is found by its own shape ([`plant_on_ray`]), not
//! only by the cell under it, so aiming at its leaves finds it.

use super::digging::{self, Aim, Edited, Hands, apply_edit};
use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::config::{FarmConfig, Recipes};
use pbd_app::drops::Drops;
use pbd_app::field_light::{Faded, LitByField, LitLikeTerrain};
use pbd_app::hotbar::{Gained, How, Planted};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::Localized;
use pbd_core::column;
use pbd_core::farming::{
    self, Climate, Crop, PLOT_KIND, PLOT_SCHEMA, Picked, Plot, Stage, WILD_KIND, WILD_SCHEMA,
};
use pbd_core::inventory::{Item, Tool};
use pbd_core::locale::{tr, tr_with};
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

    /// Say it for `seconds` rather than the farm's three.
    pub fn say_for(&mut self, now_s: f32, text: impl Into<String>, seconds: f32) {
        self.text = text.into().to_uppercase();
        self.until_s = now_s + seconds;
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
        note.say(wall_s, tr("farm.seeds_need_tilled"));
        return false;
    }
    if live_crop(world.fine, world.save, cell).is_some() {
        note.say(wall_s, tr("farm.already_growing"));
        return false;
    }
    let climate = climate_at(world.fine, cell, layer);
    if !crop.grows_in(climate) {
        note.say(
            wall_s,
            tr_with(
                "farm.wrong_climate",
                &[
                    ("crop", &crop.row().name()),
                    (
                        "climate",
                        &climate.map_or_else(|| tr("farming.climate.barren"), Climate::name),
                    ),
                ],
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
    note.say(
        wall_s,
        tr_with("farm.planted", &[("crop", &crop.row().name())]),
    );
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
    now_s: f64,
) -> Option<Crop> {
    let top = digging::material_of(fine, save, (cell, layer))?;
    let over = digging::material_of(fine, save, (cell, layer + 1))?;
    if !farming::sprouts_seed(top) || over != Material::Air {
        return None;
    }
    if wild_picked(save, cell, now_s) {
        return None;
    }
    // No wild plant shares a tree's cell (`farming-and-smelting`, "No wild
    // plant in a tree's cell").
    if digging::record_of(fine, cell).is_some_and(|r| grows_tree(&fine.set.finest_records()[r])) {
        return None;
    }
    farm.0
        .wild_crop(save.identity.seed, cell, climate_at(fine, cell, layer))
}

/// Whether the wild plant on `cell` stands picked at world second `now_s`:
/// picked, and not yet grown back (`playtest-fixes-2026-11-25` 3). It
/// grows back as a new one comes up: the bush fades in.
pub fn wild_picked(save: &WorldSave, cell: u32, now_s: f64) -> bool {
    save.records
        .get(WILD_KIND, u64::from(cell))
        .is_some_and(|r| r.read::<Picked>().is_none_or(|p| !p.grown_back(now_s)))
}

/// Whether a cell's record grows a tree, by the foliage pass's own rule.
fn grows_tree(record: &pbd_app::planet::GpuCell) -> bool {
    pbd_core::flora::grows_tree(
        &pbd_app::planet::terrain_config().trees,
        record.spare[0],
        record.metadata[1],
    )
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
        note.say(wall_s, tr("farm.no_room"));
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
    note.say(
        wall_s,
        tr_with(
            "farm.picked_wild",
            &[("crop", &row.name()), ("seeds", &seeds)],
        ),
    );
}

/// Take a piece of the ground clutter by hand (`gather-by-hand`): its goods
/// into the hands and the cell's `clutter` record, in one line.
#[allow(clippy::too_many_arguments)]
pub fn gather(
    save: &mut WorldSave,
    slots: &mut super::slots::Hotbar,
    key: u32,
    kind: pbd_core::scatter::Kind,
    day: f64,
    note: &mut FarmNote,
    wall_s: f32,
    gained: &mut MessageWriter<Gained>,
) -> bool {
    use pbd_core::scatter::{CLUTTER_KIND, CLUTTER_SCHEMA};
    let (code, count) = kind.gives();
    let Some(good) = pbd_core::goods::Good::by_code(code) else {
        return false;
    };
    let mut taken = pbd_app::clutter::taken_at(save, key);
    if !taken.take(kind, day) {
        return false;
    }
    let mut moved = slots.0.clone();
    if moved.give(Item::Good(good), count) > 0 {
        note.say(wall_s, tr("farm.no_room"));
        return false;
    }
    let record = StoredRecord::of(CLUTTER_KIND, u64::from(key), CLUTTER_SCHEMA, &taken);
    if !save.store_with_hands(record, CLUTTER_SCHEMA, &moved) {
        error!("gather BLOCKED: the save did not accept it");
        return false;
    }
    slots.0 = moved;
    gained.write(Gained {
        item: Item::Good(good),
        count,
        how: How::Gathered,
    });
    info!("gathered {kind:?} at cell {key}: +{count} {code}");
    true
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

/// The radius and height of a sown crop's three clumps, m: what the eye
/// ray finds it by.
const CROP_SHAPE_M: (f32, f32) = (0.9, 1.1);
/// The same for a wild plant's one bush.
const WILD_SHAPE_M: (f32, f32) = (0.55, 0.95);

/// Where along the ray `eye + look * t` (look a unit vector) it first meets
/// an upright cylinder standing on `base` along `up`, through its side or
/// its top, within `reach`.
pub fn ray_meets_upright(
    eye: Vec3,
    look: Vec3,
    base: Vec3,
    up: Vec3,
    (radius, height): (f32, f32),
    reach: f32,
) -> Option<f32> {
    let rel = eye - base;
    let across = |t: f32| {
        let p = rel + look * t;
        (p - up * p.dot(up)).length()
    };
    let mut best: Option<f32> = None;
    let mut take = |t: f32| {
        if (0.0..=reach).contains(&t) && best.is_none_or(|b| t < b) {
            best = Some(t);
        }
    };
    // The side.
    let d_across = look - up * look.dot(up);
    let p_across = rel - up * rel.dot(up);
    let a = d_across.length_squared();
    if a > 1e-6 {
        let b = 2.0 * p_across.dot(d_across);
        let c = p_across.length_squared() - radius * radius;
        let disc = b * b - 4.0 * a * c;
        if disc >= 0.0 {
            for t in [
                (-b - disc.sqrt()) / (2.0 * a),
                (-b + disc.sqrt()) / (2.0 * a),
            ] {
                let h = (rel + look * t).dot(up);
                if (0.0..=height).contains(&h) {
                    take(t);
                }
            }
        }
    }
    // The top, looked down onto.
    let down = look.dot(up);
    if down.abs() > 1e-6 {
        let t = (height - rel.dot(up)) / down;
        if across(t) <= radius {
            take(t);
        }
    }
    best
}

/// A crop or wild plant the eye ray meets by its own shape.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlantHit {
    pub cell: u32,
    /// The layer its ground is.
    pub layer: usize,
    /// How far along the ray, m.
    pub distance: f32,
}

/// The nearest sown crop or wild plant the eye ray meets within reach, by
/// its own shape (the clumps or the bush), so aiming at its leaves finds it
/// rather than the ground behind.
pub fn plant_on_ray(
    fine: &PlanetFine,
    save: &WorldSave,
    farm: &FarmConfig,
    eye: Vec3,
    look: Vec3,
    now_s: f64,
) -> Option<PlantHit> {
    let reach = pbd_core::aim::REACH_M;
    let look = look.normalize_or_zero();
    let mut best: Option<PlantHit> = None;
    let mut take = |hit: PlantHit| {
        if best.is_none_or(|b| hit.distance < b.distance) {
            best = Some(hit);
        }
    };
    let toward = eye.normalize_or(Vec3::Y);
    let near = ((reach + 2.0) / PLANET_RADIUS).cos();
    let mut sown = std::collections::HashSet::new();
    for record in save.records.of_kind(PLOT_KIND) {
        let cell = record.id as u32;
        let Some((plot, _)) = live_crop(fine, save, cell) else {
            continue;
        };
        sown.insert(cell);
        let Some(index) = digging::record_of(fine, cell) else {
            continue;
        };
        let Some(up) = direction_of(fine, index) else {
            continue;
        };
        if up.dot(toward) < near {
            continue;
        }
        let layer = usize::from(plot.layer);
        let base = up * (PLANET_RADIUS + column::layer_altitude(layer) + 1.0);
        if let Some(distance) = ray_meets_upright(eye, look, base, up, CROP_SHAPE_M, reach) {
            take(PlantHit {
                cell,
                layer,
                distance,
            });
        }
    }
    for (index, record) in fine.set.finest_records().iter().enumerate() {
        let up = Vec3::from_slice(&record.direction_height[..3]);
        if up.dot(toward) < near * up.length() {
            continue;
        }
        let cell = record.key();
        if sown.contains(&cell) {
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
        let up = up.normalize_or(Vec3::Y);
        let base = up * (PLANET_RADIUS + column::layer_altitude(top) + 1.0);
        let Some(distance) = ray_meets_upright(eye, look, base, up, WILD_SHAPE_M, reach) else {
            continue;
        };
        if wild_at(fine, save, farm, cell, top, now_s).is_some() {
            take(PlantHit {
                cell,
                layer: top,
                distance,
            });
        }
    }
    best
}

/// What the interact key does on the farm: one decision, read by the key
/// ([`use_key`]) and by its prompt ([`offer`]).
#[derive(Clone, Debug)]
pub enum FarmAct {
    /// Open a station's panel.
    Open {
        cell: u32,
        layer: usize,
        row: &'static station::StationRow,
    },
    /// A sown crop: harvest it if ripe, else say how long.
    Crop {
        cell: u32,
        plot: Plot,
        crop: Crop,
        ripe: bool,
    },
    /// Pick a wild plant.
    Wild { cell: u32, crop: Crop },
    /// Take a piece of the ground clutter (`gather-by-hand`).
    Gather {
        key: u32,
        kind: pbd_core::scatter::Kind,
    },
    /// Till the top with the shovel.
    Till {
        cell: u32,
        layer: usize,
        material: Material,
    },
}

impl FarmAct {
    /// Its row of `interact::ACTS` and what it is done to.
    pub fn offer(&self) -> Offer {
        match *self {
            FarmAct::Open { row, .. } => Offer::new(Act::Open, row.name()),
            FarmAct::Crop { crop, ripe, .. } => Offer::new(
                if ripe { Act::Harvest } else { Act::Check },
                crop.row().name(),
            ),
            FarmAct::Wild { crop, .. } => Offer::new(Act::PickWild, crop.row().name()),
            FarmAct::Gather { kind, .. } => {
                let (code, _) = kind.gives();
                Offer::new(
                    Act::Gather,
                    pbd_core::goods::Good::by_code(code)
                        .map_or_else(|| Localized::verbatim(code), |g| g.name()),
                )
            }
            FarmAct::Till { material, .. } => Offer::new(
                Act::Till,
                match material {
                    Material::Grass | Material::DryGrass | Material::JungleGrass => {
                        tr("block.grass.name")
                    }
                    Material::Dirt => tr("block.dirt.name"),
                    Material::Sand | Material::DesertSand => tr("block.sand.name"),
                    _ => tr("block.soil.name"),
                },
            ),
        }
    }
}

/// What F does on the farm with the eye on `aim`, a plant met by its shape
/// `plant`, a piece of ground clutter `clutter`, and `shovel` in hand: a
/// station the ray meets first, then the plant (a crop, then a wild one) the
/// ray meets before the ground, then the clutter it meets before the ground
/// (`gather-by-hand`), then a crop or wild plant on the cell the ray lands
/// in, then tilling it.
#[allow(clippy::too_many_arguments)]
pub fn farm_act(
    fine: &PlanetFine,
    save: &WorldSave,
    farm: &FarmConfig,
    aim: Option<&pbd_core::aim::Target>,
    plant: Option<PlantHit>,
    clutter: Option<pbd_app::clutter::ClutterHit>,
    shovel: bool,
    now_s: f64,
    axe_logs: u16,
) -> Option<FarmAct> {
    let crop_on = |cell: u32, layer: usize| {
        live_crop(fine, save, cell)
            .filter(|(p, _)| usize::from(p.layer) == layer)
            .map(|(plot, crop)| FarmAct::Crop {
                cell,
                ripe: farm.0.stage(crop, plot.planted_s, now_s) == Stage::Ripe,
                plot,
                crop,
            })
    };
    let wild_on = |cell: u32, layer: usize| {
        wild_at(fine, save, farm, cell, layer, now_s).map(|crop| FarmAct::Wild { cell, crop })
    };
    let on_ground = aim.and_then(|target| {
        let (cell, layer) = (target.dig.cell, target.dig.layer);
        let material = digging::material_of(fine, save, (cell, layer))?;
        // A campfire is a light, walked through and passed by the eye ray,
        // so the ray meets the ground it stands on: F there opens it
        // (`step-videos` 5.2, found by the showcase's bot, which could not
        // open one any other way).
        let over = digging::material_of(fine, save, (cell, layer + 1))
            .filter(|m| m.is_lamp() && station::station_of(*m).is_some());
        let (layer, material) = match over {
            Some(lit) => (layer + 1, lit),
            None => (layer, material),
        };
        Some((cell, layer, material, target.distance))
    });
    // A plant nearer than the block the ray meets is what the eye is on.
    let plant = plant.filter(|p| on_ground.is_none_or(|(_, _, _, d)| p.distance <= d + 0.05));
    if let Some((cell, layer, material, d)) = on_ground
        && let Some(row) = station::station_of(material)
        && plant.is_none_or(|p| d <= p.distance)
    {
        return Some(FarmAct::Open { cell, layer, row });
    }
    if let Some(p) = plant
        && clutter.is_none_or(|c| p.distance <= c.distance)
        && let Some(act) = crop_on(p.cell, p.layer).or_else(|| wild_on(p.cell, p.layer))
    {
        return Some(act);
    }
    // A piece stands on the ground the ray goes on to meet: it is what the
    // eye is on when it lies no further than a hand's breadth past it.
    if let Some(c) = clutter
        && on_ground.is_none_or(|(_, _, _, d)| c.distance <= d + 0.3)
    {
        return Some(FarmAct::Gather {
            key: c.key,
            kind: c.kind,
        });
    }
    if let Some(p) = plant
        && let Some(act) = crop_on(p.cell, p.layer).or_else(|| wild_on(p.cell, p.layer))
    {
        return Some(act);
    }
    let (cell, layer, material, _) = on_ground?;
    if let Some(act) = crop_on(cell, layer).or_else(|| wild_on(cell, layer)) {
        return Some(act);
    }
    (shovel
        && farming::tillable(material)
        && digging::material_of(fine, save, (cell, layer + 1)) == Some(Material::Air)
        && !digging::under_standing_tree(fine, save, axe_logs, (cell, layer)))
    .then_some(FarmAct::Till {
        cell,
        layer,
        material,
    })
}

/// The eye and its look, from the active 3D camera.
/// Planet-local: render space less the planet's centre
/// (`movement-smoothness` design 6.1).
fn eye_ray(
    cameras: &Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<&pbd_app::planet::PlanetRenderFrame>,
) -> Option<(Vec3, Vec3)> {
    cameras.iter().find(|(_, c)| c.is_active).map(|(t, _)| {
        (
            pbd_app::render_origin::local_of(frame, t.translation()),
            t.forward().as_vec3(),
        )
    })
}

/// The logs a tree gives the axe before it falls, from the gathering's
/// settings (absent in a still capture).
pub(super) fn axe_logs(gather: Option<&pbd_app::config::GatherConfig>) -> u16 {
    gather.map_or_else(
        || pbd_core::gather::GatherSettings::default().axe_logs,
        |g| g.0.axe_logs,
    )
}

/// Offer the farm's act to the interact key.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn offer(
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    aim: Res<Aim>,
    tools: Res<pbd_app::fish::ToolSlot>,
    farm: Res<FarmConfig>,
    (fine, save, sun, time, scatter, gathering): (
        Res<PlanetFine>,
        Res<WorldSave>,
        Res<pbd_app::sky::Sun>,
        Res<Time>,
        Option<Res<pbd_app::config::ScatterSettings>>,
        Option<Res<pbd_app::config::GatherConfig>>,
    ),
    mut interact: ResMut<Interact>,
    mut last: Local<(
        pbd_app::interact::Recheck,
        Option<PlantHit>,
        Option<pbd_app::clutter::ClutterHit>,
        u64,
    )>,
) {
    if !walking.is_some_and(|w| w.active) {
        return;
    }
    let Some((eye, look)) = eye_ray(&cameras, frame.as_deref()) else {
        return;
    };
    let (recheck, plant, clutter, records) = &mut *last;
    // A take changes what stands without moving the eye or the fine set.
    let stored = save.records.len() as u64;
    if recheck.due(eye, look, fine.version, time.delta_secs()) || *records != stored {
        *records = stored;
        *plant = plant_on_ray(&fine, &save, &farm, eye, look, sun.clock.seconds);
        *clutter = scatter.as_deref().and_then(|s| {
            pbd_app::clutter::clutter_on_ray(
                &fine,
                &save,
                &pbd_app::clutter::odds(s),
                pbd_app::clutter::day_of(sun.clock.seconds),
                eye,
                look,
            )
        });
    }
    let shovel = tools.0.held() == Some(Tool::Shovel);
    if let Some(act) = farm_act(
        &fine,
        &save,
        &farm,
        aim.target.as_ref(),
        *plant,
        *clutter,
        shovel,
        sun.clock.seconds,
        axe_logs(gathering.as_deref()),
    ) {
        interact.offer(act.offer());
    }
}

/// The interact key on the farm: does what [`farm_act`] says.
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
    (cameras, interact, scatter, gathering, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<Interact>>,
        Option<Res<pbd_app::config::ScatterSettings>>,
        Option<Res<pbd_app::config::GatherConfig>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    (mut gained, mut did): (MessageWriter<Gained>, MessageWriter<pbd_app::missions::Did>),
    (mut fine, mut contact, mut save, mut slots, mut drops, sun): (
        ResMut<PlanetFine>,
        ResMut<PlanetContact>,
        ResMut<WorldSave>,
        ResMut<super::slots::Hotbar>,
        ResMut<Drops>,
        Res<pbd_app::sky::Sun>,
    ),
    (mut open_chest, walkers): (
        Option<ResMut<super::pack::OpenChest>>,
        Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    ),
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    // F shuts a station's panel as it opened it, and a chest's.
    if matches!(*screen, Screen::Forge | Screen::Kitchen)
        || (*screen == Screen::Pack && open_chest.as_deref().is_some_and(|open| open.0.is_some()))
    {
        *screen = Screen::Playing;
        return;
    }
    if *screen != Screen::Playing {
        return;
    }
    if !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    if !pbd_app::interact::may(interact.as_deref(), Reader::Farm) {
        return;
    }
    let wall_s = time.elapsed_secs();
    let now_s = sun.clock.seconds;
    let plant = eye_ray(&cameras, frame.as_deref())
        .and_then(|(eye, look)| plant_on_ray(&fine, &save, &farm, eye, look, now_s));
    let day = pbd_app::clutter::day_of(now_s);
    let clutter = eye_ray(&cameras, frame.as_deref())
        .zip(scatter.as_deref())
        .and_then(|((eye, look), s)| {
            pbd_app::clutter::clutter_on_ray(
                &fine,
                &save,
                &pbd_app::clutter::odds(s),
                day,
                eye,
                look,
            )
        });
    let shovel = tools.0.held() == Some(Tool::Shovel);
    let act = farm_act(
        &fine,
        &save,
        &farm,
        aim.target.as_ref(),
        plant,
        clutter,
        shovel,
        now_s,
        axe_logs(gathering.as_deref()),
    );
    let Some(act) = act else {
        // Tillable ground under a block or a tree: say why F does nothing.
        if shovel
            && let Some(target) = aim.target
            && digging::material_of(&fine, &save, (target.dig.cell, target.dig.layer))
                .is_some_and(farming::tillable)
        {
            let block = (target.dig.cell, target.dig.layer);
            if digging::under_standing_tree(&fine, &save, axe_logs(gathering.as_deref()), block) {
                note.say(wall_s, tr("farm.tree_first"));
            } else {
                note.say(wall_s, tr("farm.till_under_block"));
            }
        }
        return;
    };
    let (cell, layer, material) = match act {
        // A chest opens beside the pack (`storage-chests` design 3), within
        // the hold's reach of the walker.
        FarmAct::Open { cell, layer, row } if row.material == Some(Material::Chest) => {
            let centre = frame.as_deref().copied().unwrap_or_default();
            let near = walkers
                .iter()
                .next()
                .map(|p| p.0.as_dvec3() - centre.physics_center())
                .and_then(|walker| super::pack::chest_distance(&fine, walker, cell, layer as u16))
                .is_some_and(|d| d <= super::pack::HOLD_REACH_M);
            if let (true, Some(open)) = (near, open_chest.as_deref_mut()) {
                open.0 = Some((cell, layer as u16));
                *screen = Screen::Pack;
                info!("CHEST_OPEN cell {cell} layer {layer}");
            } else {
                note.say(wall_s, tr("farm.too_far"));
            }
            return;
        }
        FarmAct::Open { cell, layer, row } => {
            station_at.0 = Some((cell, layer as u16));
            // The kitchen's stations have their own panel (`cooking` design S4).
            *screen = super::kitchen::screen_for(row);
            info!("opened the {} at cell {cell} layer {layer}", row.name());
            return;
        }
        FarmAct::Crop {
            cell, plot, crop, ..
        } => {
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
        FarmAct::Wild { cell, crop } => {
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
        FarmAct::Gather { key, kind } => {
            gather(
                &mut save,
                &mut slots,
                key,
                kind,
                day,
                &mut note,
                wall_s,
                &mut gained,
            );
            return;
        }
        FarmAct::Till {
            cell,
            layer,
            material,
        } => (cell, layer, material),
    };
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
        // A seed turned up is the toast's to say (`item-toasts` decision 6).
        note.say(wall_s, tr("farm.tilled"));
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
            tr_with(
                "farm.ripe_in",
                &[
                    ("crop", &row.name()),
                    ("hours", &format!("{:.0}", left_h.ceil())),
                ],
            ),
        );
        return;
    }
    let tuning = farm.0.of(crop);
    let mut moved = slots.0.clone();
    let left = moved.give(Item::Good(row.produce), tuning.produce)
        + moved.give(Item::Good(row.seed), tuning.seeds);
    if left > 0 {
        note.say(wall_s, tr("farm.no_room_harvest"));
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
        tr_with(
            "farm.harvested",
            &[
                ("produce", &tuning.produce),
                ("crop", &row.name()),
                ("seeds", &tuning.seeds),
            ],
        ),
    );
}

/// The note under the crosshair.
pub fn spawn_note(mut commands: Commands) {
    commands.spawn((
        pbd_app::ui_kit::KitText::new(Localized::empty(), pbd_app::ui_kit::LABEL)
            .outline(pbd_app::ui_kit::INK),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(60),
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
    road: Option<ResMut<pbd_app::roads::RoadSay>>,
) {
    // A road's refusal (`townsfolk` design 39 and 40) shows as the note.
    if let Some(mut road) = road
        && let Some(line) = road.0.take()
    {
        note.say(time.elapsed_secs(), line);
    }
    if !note.text.is_empty() && time.elapsed_secs() > note.until_s {
        note.text.clear();
    }
    for mut words in &mut text {
        if words.text != note.text {
            words.text = Localized::verbatim(note.text.clone());
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
    /// The carpenter's bench's board, saw and square (`furniture` design
    /// 1), in the oak and the iron.
    carpenter_wood: Handle<Mesh>,
    carpenter_iron: Handle<Mesh>,
    plank_tex: Handle<StandardMaterial>,
    iron_tex: Handle<StandardMaterial>,
    /// The drafting table's board and its sheet (`building-planner` design
    /// 3), in the plank and the linen.
    drafting_wood: Handle<Mesh>,
    drafting_sheet: Handle<Mesh>,
    linen_tex: Handle<StandardMaterial>,
}

/// A mesh of the faces of `polys` in textures `keep`, flat-shaded.
fn polys_mesh(polys: &[pbd_core::furniture::model::Poly], keep: &[&str], repeat_m: f32) -> Mesh {
    let (mut positions, mut normals, mut uvs) = (Vec::new(), Vec::new(), Vec::new());
    for poly in polys.iter().filter(|p| keep.contains(&p.texture)) {
        let n = poly.normal();
        // The town's world mapping at the texture's own scale, so its texels
        // match the walls'; each point its own normal, shaded smooth.
        let uv = |p: Vec3| -> [f32; 2] {
            let s = 1.0 / repeat_m;
            if n.y.abs() > 0.7 {
                return [p.x * s, -p.z * s];
            }
            let l = n.x.hypot(n.z).max(1e-6);
            [(p.x * -n.z + p.z * n.x) / l * s, -p.y * s]
        };
        for i in 1..poly.points.len().saturating_sub(1) {
            for k in [0, i, i + 1] {
                let p = poly.points[k];
                positions.push(p.to_array());
                normals.push(poly.normals[k].to_array());
                uvs.push(uv(p));
            }
        }
    }
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
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
        carpenter_wood: meshes.add(polys_mesh(
            &pbd_core::furniture::model::carpenter_top(),
            &[
                pbd_core::furniture::model::FRAME,
                pbd_core::furniture::model::TOP,
            ],
            2.0,
        )),
        carpenter_iron: meshes.add(polys_mesh(
            &pbd_core::furniture::model::carpenter_top(),
            &[pbd_core::furniture::model::IRON],
            1.0,
        )),
        // The carpenter's bench's board and tools in the town's own plank
        // and iron, sampled nearest (owner: "Make sure things have textures").
        plank_tex: materials.add(StandardMaterial {
            base_color_texture: Some(pbd_app::towns::texture(&assets, "plank")),
            perceptual_roughness: 0.92,
            ..default()
        }),
        drafting_wood: meshes.add(polys_mesh(
            &pbd_core::furniture::model::drafting_top(),
            &[
                pbd_core::furniture::model::FRAME,
                pbd_core::furniture::model::TOP,
            ],
            2.0,
        )),
        drafting_sheet: meshes.add(polys_mesh(
            &pbd_core::furniture::model::drafting_top(),
            &[pbd_core::furniture::model::SHEET],
            1.0,
        )),
        linen_tex: materials.add(StandardMaterial {
            base_color_texture: Some(pbd_app::towns::texture(&assets, "linen")),
            perceptual_roughness: 0.95,
            ..default()
        }),
        iron_tex: materials.add(StandardMaterial {
            base_color_texture: Some(pbd_app::towns::texture(&assets, "iron")),
            perceptual_roughness: 0.6,
            ..default()
        }),
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
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    mut crops: Query<(Entity, &mut CropView, &mut Faded), Without<ForgeView>>,
    mut forges: Query<(Entity, &mut ForgeView, &mut Faded, &Children), Without<CropView>>,
    mut flames: Query<&mut Visibility, With<Flame>>,
    mut keys: Local<Keys>,
    mut since: Local<f32>,
    paving: Option<Res<pbd_app::roads::paving::PavingLooks>>,
) {
    let Some(looks) = looks else { return };
    let Some(paving) = paving else { return };
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
        .map(|(t, _)| pbd_app::render_origin::local_of(frame.as_deref(), t.translation()))
    else {
        return;
    };
    // Spawned in render space from the planet-local stand.
    let drawn = |mut t: Transform| {
        t.translation = pbd_app::render_origin::drawn_of(frame.as_deref(), t.translation);
        t
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
        if wanted.contains_key(&cell) || wild_picked(&save, cell, now_s) || grows_tree(record) {
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
        let Some(transform) = standing(&fine, index, usize::from(layer), turn).map(drawn) else {
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
    // The forges, workbenches and chests: every such block an edit placed,
    // near the eye. A campfire is a light, drawn by the column pass
    // (`cooking` S2).
    let mut wanted: HashMap<(u32, u16), (usize, bool, Material)> = HashMap::new();
    for edit in save.edits.all() {
        if !matches!(
            edit.material,
            Material::Forge
                | Material::Workbench
                | Material::Carpenter
                | Material::DraftingTable
                | Material::Grinder
                | Material::Mixer
                | Material::Chest
        ) {
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
        // The drafting table queues no jobs (`building-planner` design 3).
        let working = station::station_of(edit.material).is_some_and(|row| {
            station_state(&save, edit.cell, edit.layer, row).working(&recipes.0, now_s)
        });
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
        let Some(transform) = standing(&fine, index, usize::from(layer), 0.0).map(drawn) else {
            continue;
        };
        if material == Material::Chest {
            // A chest (`storage-chests` design 1): an iron strap across the
            // lid and a dark iron hasp at its edge, on the wood block; lit
            // as the ground is, with no outline.
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
                .with_children(|chest| {
                    chest.spawn((
                        Mesh3d(looks.board.clone()),
                        MeshMaterial3d(looks.iron.clone()),
                        Transform::from_xyz(0.0, 0.01, 0.0).with_scale(Vec3::new(1.6, 0.5, 0.5)),
                    ));
                    chest.spawn((
                        Mesh3d(looks.vise.clone()),
                        MeshMaterial3d(looks.iron.clone()),
                        Transform::from_xyz(1.12, 0.08, 0.0).with_scale(Vec3::new(0.8, 0.7, 1.0)),
                    ));
                });
            continue;
        }
        if material == Material::DraftingTable {
            // Its board tipped to the front with a sheet on it
            // (`building-planner` design 3), in the plank and the linen.
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
                .with_children(|table| {
                    table.spawn((
                        Mesh3d(looks.drafting_wood.clone()),
                        MeshMaterial3d(looks.plank_tex.clone()),
                    ));
                    table.spawn((
                        Mesh3d(looks.drafting_sheet.clone()),
                        MeshMaterial3d(looks.linen_tex.clone()),
                    ));
                });
            continue;
        }
        if material == Material::Carpenter {
            // A board laid on it, a saw across the board and a square at
            // its end (`furniture` design 1).
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
                    bench.spawn((
                        Mesh3d(looks.carpenter_wood.clone()),
                        MeshMaterial3d(looks.plank_tex.clone()),
                    ));
                    bench.spawn((
                        Mesh3d(looks.carpenter_iron.clone()),
                        MeshMaterial3d(looks.iron_tex.clone()),
                    ));
                });
            continue;
        }
        if matches!(material, Material::Grinder | Material::Mixer) {
            // The gravel stations (`townsfolk` design 41): shaped forms of
            // timber, iron and stone, lit as the ground is.
            let parts = if material == Material::Grinder {
                &paving.grinder
            } else {
                &paving.mixer
            };
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
                .with_children(|station| {
                    for (stuff, mesh) in parts {
                        station.spawn((
                            Mesh3d(mesh.clone()),
                            MeshMaterial3d(paving.material(*stuff)),
                        ));
                    }
                });
            continue;
        }
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

/// `--pick-wild-ago <days>` (playtest-fixes design 3): a capture instrument.
/// Once the ground round the player is in, every wild plant within 20 m is
/// recorded as picked that many game days ago, the record a pick writes, so
/// a still shows one picked a day ago gone and one picked over two days ago
/// grown back. The capture's save is memory-only.
#[allow(clippy::type_complexity)]
pub fn scripted_pick_wild(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    refresh: Res<pbd_app::planet::LodRefresh>,
    (fine, mut save, farm, sun): (
        Res<PlanetFine>,
        ResMut<WorldSave>,
        Res<FarmConfig>,
        Res<pbd_app::sky::Sun>,
    ),
    mut frames: Local<u32>,
) {
    let Some(days) = launch.pick_wild_ago else {
        return;
    };
    if *frames == u32::MAX || refresh.in_flight_s().is_some() {
        return;
    }
    *frames += 1;
    if *frames < 20 {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    *frames = u32::MAX;
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let up = eye.normalize_or(Vec3::Y);
    let now_s = sun.clock.seconds;
    let picked_s = now_s - days * f64::from(pbd_core::daylight::DAY_S);
    let mut picked = 0;
    for record in fine.set.finest_records() {
        let at = Vec3::from_slice(&record.direction_height[..3]).normalize_or(Vec3::Y);
        if at.angle_between(up) * PLANET_RADIUS > 20.0 {
            continue;
        }
        let cell = record.key();
        let Some(index) = digging::record_of(&fine, cell) else {
            continue;
        };
        let Some(top) = fine.set.columns.column(index).and_then(|c| c.surface()) else {
            continue;
        };
        if wild_at(&fine, &save, &farm, cell, top, now_s).is_none() {
            continue;
        }
        let line = StoredRecord::of(
            WILD_KIND,
            u64::from(cell),
            WILD_SCHEMA,
            &Picked { picked_s },
        );
        if save
            .store(&pbd_core::records::Author::Player(0), vec![line])
            .is_some()
        {
            picked += 1;
        }
    }
    info!("--pick-wild-ago: {picked} wild plants picked {days} game days ago");
}

/// `--gather-around <metres>` (`gather-by-hand`): a measurement instrument
/// for the headless captures, which have no hands. Once the ground round the
/// player is in, every piece of clutter within `metres` is taken through
/// [`gather`], as F takes one, dated a little before now so a still shows it
/// sunk away rather than mid-fade. The capture's save is memory-only.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted_gather(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    refresh: Res<pbd_app::planet::LodRefresh>,
    (fine, mut save, mut slots, sun, scatter): (
        Res<PlanetFine>,
        ResMut<WorldSave>,
        ResMut<super::slots::Hotbar>,
        Res<pbd_app::sky::Sun>,
        Res<pbd_app::config::ScatterSettings>,
    ),
    (mut note, time, mut gained): (ResMut<FarmNote>, Res<Time>, MessageWriter<Gained>),
    mut frames: Local<u32>,
) {
    let Some(metres) = launch.gather_around else {
        return;
    };
    if *frames == u32::MAX || refresh.in_flight_s().is_some() {
        return;
    }
    // A few frames for the fine set round the player to land.
    *frames += 1;
    if *frames < 20 {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    *frames = u32::MAX;
    let up = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation())
        .normalize_or(Vec3::Y);
    let odds = pbd_app::clutter::odds(&scatter);
    // A hundredth of a day ago: long past the sink, far short of regrowth.
    let day = pbd_app::clutter::day_of(sun.clock.seconds) - 0.01;
    let wall_s = time.elapsed_secs();
    let mut taken = 0;
    for record in fine.set.finest_records() {
        let at = Vec3::from_slice(&record.direction_height[..3]).normalize_or(Vec3::Y);
        if at.angle_between(up) * PLANET_RADIUS > metres {
            continue;
        }
        for piece in pbd_app::clutter::pieces_of(record, &odds) {
            taken += usize::from(gather(
                &mut save,
                &mut slots,
                record.key(),
                piece.kind,
                day,
                &mut note,
                wall_s,
                &mut gained,
            ));
        }
    }
    info!("scripted gather: took {taken} pieces within {metres} m");
}

/// `--farm` and `--forge` (`farming-and-smelting` task 5.1): a measurement
/// instrument for the headless captures. A capture has no hands, so this
/// does through the same durable paths what a player would: till, sow at
/// staggered world times (so the rows show every stage), turn up a seed,
/// stand a lantern by the field; place a forge and queue two jobs, the first
/// part done. The capture's save is memory-only, so nothing is kept.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
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
    (net, standing, roads_cfg, planner): (
        Option<ResMut<pbd_app::roads::RoadNet>>,
        Option<ResMut<pbd_app::roads::RoadsStanding>>,
        Option<Res<pbd_app::roads::RoadsSettings>>,
        Option<ResMut<pbd_app::roads::planner::Planner>>,
    ),
) {
    if *done
        || launch.capture.is_none()
        || (launch.farm.is_none() && launch.forge.is_none() && launch.paving.is_none())
    {
        return;
    }
    if refresh.in_flight_s().is_some() {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
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
    let first = launch
        .farm
        .or(launch.forge)
        .or(launch.paving)
        .unwrap_or(6.0);
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
                let share = if launch.farm_ripe { 1.2 } else { share };
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
                    crop.row().name(),
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
            if let Some(crop) = wild_at(&fine, &save, &farm, record.key(), top, now_s) {
                info!(
                    "scripted farm: a wild {} {d:.1} m away, {:.0} degrees right of the view",
                    crop.row().name(),
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
                if let Some(crop) = wild_at(&fine, &save, &farm, record.key(), top, now_s) {
                    nearest = Some((d, record.key(), crop));
                }
            }
            match nearest {
                Some((d, cell, crop)) => {
                    info!(
                        "scripted farm: picking the wild {} {d:.1} m away",
                        crop.row().name()
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
    if let Some(metres) = launch.paving {
        use pbd_app::roads::paving;
        // The grinder to the left and the mixer to the right (not for the
        // slope and the bridge, whose roads are the shot).
        let shot_road = launch
            .road_plan
            .as_deref()
            .is_some_and(|s| s.starts_with("slope") || s.starts_with("bridge"));
        let stations = !shot_road;
        for (across, material) in [(-3.2, Material::Grinder), (3.2, Material::Mixer)] {
            if !stations {
                continue;
            }
            if let Some((cell, top)) = spot(&fine, &contact, metres, across) {
                apply_edit(world!(), Hands::Empty, cell, top + 1, material);
                info!("scripted paving: a {material:?} at cell {cell}");
            }
        }
        // A road staked from past them along the view by its two ends, as
        // the planner's two clicks do (design 42k), its nearer cells paved.
        let config = *pbd_app::planet::terrain_config();
        let cfg = roads_cfg.map_or_else(pbd_app::roads::load_config, |r| (*r.0).clone());
        let pcfg = paving::load_planner();
        let key_at = |fine: &PlanetFine, contact: &PlanetContact, ahead: f32, across: f32| {
            spot(fine, contact, ahead, across).map(|(cell, _)| cell)
        };
        slots.give(Item::Good(pbd_core::goods::Good::SURVEY_STAKES), 16);
        slots.give(Item::Good(pbd_core::goods::Good::GRAVEL), 12);
        let scene = launch.road_plan.clone();
        let wide = scene.as_deref() == Some("wide");
        // `lamps`: a three-wide dirt road with lamps, staked, paved and its
        // lamp posts built through the paving's own calls (design 42m).
        let lamps = scene.as_deref() == Some("lamps");
        let how = pbd_core::roads::planner::How {
            width: if wide || lamps { 3 } else { 1 },
            surface: if lamps {
                pbd_core::roads::Surface::Dirt
            } else {
                pbd_core::roads::Surface::Gravel
            },
            lamps,
        };
        if lamps {
            slots.give(Item::Good(pbd_core::goods::Good::LOG), 16);
            slots.give(Item::Block(Material::LanternPost), 16);
        }
        let road = |save: &mut WorldSave, slots: &mut super::slots::Hotbar, a: u32, b: u32| {
            match paving::end_road(save, slots, a, b, how, &|_| false, &config, &cfg, &pcfg) {
                Ok((stakes, cells)) => {
                    info!("scripted road plan: {stakes} stakes, {cells} cells");
                }
                Err(why) => warn!("scripted road plan: refused: {why:?}"),
            }
        };
        // `slope` and `bridge` (design 42m, the captures the handoff owes):
        // the first road of a fan along the view that the planner lays with
        // no red and that climbs (two wide where the ground takes it, else
        // one) or carries a bridge; staked and paved whole through the
        // paving's own calls.
        // `slope:A:B:W` and `bridge:A:B:W` stake that road (cells A to B,
        // width W) as a first run logged it, for a second run from the
        // view it logged.
        if shot_road {
            use pbd_core::roads::planner as core;
            let name = scene.clone().unwrap_or_default();
            let bridge = name.starts_with("bridge");
            let given: Vec<u32> = name
                .split(':')
                .skip(1)
                .filter_map(|v| v.parse().ok())
                .collect();
            slots.give(Item::Good(pbd_core::goods::Good::GRAVEL), 120);
            slots.give(Item::Good(pbd_core::goods::Good::PLANKS), 120);
            slots.give(Item::Good(pbd_core::goods::Good::LOG), 30);
            // Clear of the world's roads, so the two are not laid together.
            let world: std::collections::BTreeSet<u32> = net
                .as_deref()
                .map(|n| {
                    n.cells
                        .iter()
                        .flat_map(|l| {
                            std::iter::once(l.cell.key)
                                .chain(pbd_core::roads::neighbours(l.cell.key))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let mut picked = match given[..] {
                [a, b, w] => Some((
                    a,
                    b,
                    core::How {
                        width: w as u8,
                        surface: pbd_core::roads::Surface::Gravel,
                        lamps: false,
                    },
                    0,
                )),
                _ => None,
            };
            'fan: for width in [2u8, 1] {
                if picked.is_some() {
                    break;
                }
                // Up the view from spots across it; for a bridge, from a
                // grid of spots and turned every 30 degrees, to find water a
                // plank bridge spans.
                let mut legs: Vec<((f32, f32), (f32, f32))> = Vec::new();
                for across in [0.0f32, 4.0, -4.0, 8.0, -8.0, 12.0, -12.0, 16.0, -16.0] {
                    for (near, far) in [(3.0f32, 30.0f32), (3.0, 20.0), (8.0, 40.0)] {
                        legs.push(((near, across), (far, across)));
                    }
                }
                if bridge {
                    for near in [3.0f32, 10.0, 18.0, 26.0] {
                        for across in [0.0f32, 8.0, -8.0, 16.0, -16.0, 24.0, -24.0] {
                            for turn in 0..12 {
                                let t = (turn as f32 * 30.0).to_radians();
                                for length in [14.0f32, 22.0] {
                                    legs.push((
                                        (near, across),
                                        (near + t.cos() * length, across + t.sin() * length),
                                    ));
                                }
                            }
                        }
                    }
                }
                for ((near, across), (far, across_far)) in legs {
                    {
                        let (Some(a), Some(b)) = (
                            key_at(&fine, &contact, metres + near, across),
                            key_at(&fine, &contact, metres + far, across_far),
                        ) else {
                            continue;
                        };
                        // Straight along a row, where a wide road climbs.
                        let rows = (far - near).hypot(across_far - across) / 2.833;
                        let b = core::along_row(a, b, rows.round() as i64).unwrap_or(b);
                        let how = core::How {
                            width,
                            surface: pbd_core::roads::Surface::Gravel,
                            lamps: false,
                        };
                        let road =
                            core::PlanRoad::new(core::stake_line(a, b, pcfg.max_segment), how);
                        // As the planner asks: a standing tree refuses a cell.
                        let tree =
                            |k: u32| pbd_app::roads::planner::tree_on(&fine, &contact, &save, 4, k);
                        let laid = {
                            let ground = paving::stake_ground(&config, &save.edits, &save.yields);
                            core::lay(&road, &ground, &tree, &pcfg, cfg.ford_depth_m)
                        };
                        let wanted = if bridge {
                            laid.iter().any(|c| c.deck.is_some())
                        } else {
                            laid.iter()
                                .filter(|c| c.lane == 0 && c.up.is_some())
                                .count()
                                >= 2
                        };
                        let apart = laid.iter().all(|c| !world.contains(&c.key));
                        if core::first_clash(&laid).is_none() && wanted && apart {
                            picked = Some((a, b, how, laid.len()));
                            break 'fan;
                        }
                    }
                }
            }
            match picked {
                Some((a, b, how, n)) => {
                    info!(
                        "scripted road plan: {} {n} cells, width {}",
                        scene.as_deref().unwrap_or(""),
                        how.width
                    );
                    // Where to stand for the shot: on its first stake's cell,
                    // 1.5 m back from the stake, facing along it (`--yaw` as
                    // the roads' log gives it).
                    if let (Some(ca), Some(cb)) =
                        (pbd_core::roads::centre(a), pbd_core::roads::centre(b))
                    {
                        let along = (cb - ca - ca * (cb - ca).dot(ca)).normalize_or_zero();
                        let kind = if bridge { "bridge" } else { "slope" };
                        // On foot on its first cell, and from over the
                        // ground 14 m back (`--view column`).
                        for (what, back_m) in [("view", 1.5f32), ("overview", 14.0)] {
                            let back = (ca - along * (back_m / config.radius_m)).normalize();
                            let (north, east) = pbd_core::geo::north_east(back);
                            let bearing = (-along.dot(north)).atan2(along.dot(east)).to_degrees();
                            let yaw = (bearing + 360.0).rem_euclid(360.0) - 180.0;
                            let (lat, lon) = pbd_core::geo::lat_lon(back).degrees();
                            info!(
                                "scripted road {what}: --at {lat:.5} {lon:.5} --yaw {yaw:.1} --road-plan {kind}:{a}:{b}:{}",
                                how.width
                            );
                        }
                    }
                    match paving::end_road(
                        &mut save,
                        &mut slots,
                        a,
                        b,
                        how,
                        &|_| false,
                        &config,
                        &cfg,
                        &pcfg,
                    ) {
                        Ok(_) => {
                            let plans = pbd_core::roads::planner::plans(&save.records);
                            let keys: Vec<u32> = plans
                                .values()
                                .flat_map(|p| &p.roads)
                                .flat_map(|r| {
                                    let ground =
                                        paving::stake_ground(&config, &save.edits, &save.yields);
                                    core::lay(r, &ground, &|_| false, &pcfg, cfg.ford_depth_m)
                                })
                                .map(|c| c.key)
                                .collect();
                            for key in keys {
                                if let Err(why) =
                                    paving::pave(&mut save, &mut slots, key, &config, &cfg, &pcfg)
                                {
                                    warn!("scripted paving: cell {key} not paved: {why:?}");
                                }
                            }
                        }
                        Err(why) => warn!("scripted road plan: refused: {why:?}"),
                    }
                }
                None => warn!("scripted road plan: no {:?} road in the fan", scene),
            }
            // An empty hand, so the view is the walker's own.
            if let Some(i) = (0..10).find(|&i| slots.get(i).is_none()) {
                slots.select(i);
            }
            if let (Some(mut net), Some(mut standing)) = (net, standing) {
                paving::restand(&save, &mut net, &mut standing);
            }
            return;
        }
        let (start, end) = (
            key_at(&fine, &contact, metres + 3.0, 0.0),
            key_at(&fine, &contact, metres + 34.0, 0.0),
        );
        if let (Some(a), Some(b)) = (start, end) {
            // `wide` stakes nothing: its road is the open one below, so its
            // ghost shows which lanes the ground allows.
            if lamps {
                // Straight along a row of the lattice, where a wide road
                // may climb (design 42m).
                let b = pbd_core::roads::planner::along_row(a, b, 11).unwrap_or(b);
                road(&mut save, &mut slots, a, b);
                if pbd_core::roads::planner::plans(&save.records).is_empty() {
                    // Ground no wide road takes here: one wide, lamps on.
                    let one = pbd_core::roads::planner::How { width: 1, ..how };
                    if let Err(why) = paving::end_road(
                        &mut save,
                        &mut slots,
                        a,
                        b,
                        one,
                        &|_| false,
                        &config,
                        &cfg,
                        &pcfg,
                    ) {
                        warn!("scripted road plan: refused one wide too: {why:?}");
                    }
                }
            } else if !wide {
                road(&mut save, &mut slots, a, b);
            }
            let plans = pbd_core::roads::planner::plans(&save.records);
            let first = plans.values().flat_map(|p| p.roads.first()).next().cloned();
            match scene.as_deref() {
                None => {
                    if let Some(first) = &first {
                        for &key in first.centre().iter().take(6) {
                            if let Err(why) =
                                paving::pave(&mut save, &mut slots, key, &config, &cfg, &pcfg)
                            {
                                warn!("scripted paving: cell {key} not paved: {why:?}");
                            }
                        }
                    }
                }
                Some("lamps") => {
                    if let Some(first) = &first {
                        let ground = paving::stake_ground(&config, &save.edits, &save.yields);
                        let keys: Vec<u32> = pbd_core::roads::planner::lay(
                            first,
                            &ground,
                            &|_| false,
                            &pcfg,
                            cfg.ford_depth_m,
                        )
                        .iter()
                        .map(|c| c.key)
                        .collect();
                        drop(ground);
                        for key in keys {
                            if let Err(why) =
                                paving::pave(&mut save, &mut slots, key, &config, &cfg, &pcfg)
                            {
                                warn!("scripted paving: cell {key} not paved: {why:?}");
                                continue;
                            }
                            match paving::build_lamps(&mut save, &mut slots, key, &config, &cfg) {
                                Ok(made) => pbd_app::economy::road_jobs::into_tier(
                                    &made,
                                    &save,
                                    &mut fine,
                                    &mut contact,
                                ),
                                Err(why) => warn!("scripted lamps: {why:?}"),
                            }
                        }
                    }
                }
                Some("stake") | Some("erased") => {
                    let mid = first.as_ref().map(|r| r.centre()[r.centre().len() / 2]);
                    if let (Some(mid), Some(side)) =
                        (mid, key_at(&fine, &contact, metres + 22.0, 16.0))
                    {
                        road(&mut save, &mut slots, mid, side);
                        if scene.as_deref() == Some("erased")
                            && let Err(why) = paving::erase_road(
                                &mut save, &mut slots, side, &config, &cfg, &pcfg,
                            )
                        {
                            warn!("scripted eraser: refused: {why:?}");
                        }
                    }
                }
                _ => {}
            }
            // Stakes in hand, and for `two` an open start, so the ghost
            // follows the aim from it.
            let held = (0..10).find(|&i| {
                slots.get(i).map(|s| s.item)
                    == Some(Item::Good(pbd_core::goods::Good::SURVEY_STAKES))
            });
            if scene.is_some()
                && let Some(i) = held
            {
                slots.select(i);
            }
            match (planner, scene.as_deref()) {
                (Some(mut planner), Some("two")) => {
                    planner.start = key_at(&fine, &contact, metres + 1.0, -3.0);
                }
                // A three-wide road open from far up the view: its ghost
                // runs back to the aim.
                (Some(mut planner), Some("wide")) => {
                    planner.how = how;
                    planner.start = Some(b);
                }
                _ => {}
            }
        }
        if let (Some(mut net), Some(mut standing)) = (net, standing) {
            paving::restand(&save, &mut net, &mut standing);
        }
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

    /// A ray meets an upright cylinder through its side and, looked down
    /// onto, through its top; past it or beside it, not at all.
    #[test]
    fn a_ray_meets_a_plant_by_its_side_and_its_top() {
        let (base, up) = (Vec3::ZERO, Vec3::Y);
        let shape = (0.5, 1.0);
        let side = ray_meets_upright(Vec3::new(-3.0, 0.5, 0.0), Vec3::X, base, up, shape, 5.0);
        assert!((side.unwrap() - 2.5).abs() < 1e-4);
        let top = ray_meets_upright(Vec3::new(0.0, 3.0, 0.0), -Vec3::Y, base, up, shape, 5.0);
        assert!((top.unwrap() - 2.0).abs() < 1e-4);
        assert!(
            ray_meets_upright(Vec3::new(-3.0, 1.5, 0.0), Vec3::X, base, up, shape, 5.0).is_none()
        );
        assert!(
            ray_meets_upright(Vec3::new(-3.0, 0.5, 0.0), Vec3::X, base, up, shape, 2.0).is_none()
        );
    }

    /// A grassy cell and where to look at it from: its cell, its top layer,
    /// its up, the base of a plant on it, and an eye 1.6 m off.
    type Spot = (u32, usize, Vec3, Vec3, Vec3);

    /// Grassy, level cells round the fields spawn, each with a place to
    /// stand 1.6 m off it at the same height: (cell, top layer, up, the
    /// plant's base, where the eye stands).
    fn grassy_spots(fine: &PlanetFine, anchor: Vec3) -> Vec<Spot> {
        let mut spots: Vec<(f32, Spot)> = fine
            .set
            .finest_records()
            .iter()
            .enumerate()
            .filter_map(|(index, record)| {
                let column = fine.set.columns.column(index)?;
                let top = column.surface()?;
                if !farming::sprouts_seed(column.material(top))
                    || column.material(top + 1) != Material::Air
                {
                    return None;
                }
                let up = Vec3::from_slice(&record.direction_height[..3]).normalize();
                let ground = PLANET_RADIUS + column::layer_altitude(top) + 1.0;
                let base = up * ground;
                let east = Vec3::Y.cross(up).normalize();
                let stand = (base + east * 1.6).normalize();
                let height = pbd_app::planet::surface_height(stand);
                ((PLANET_RADIUS + height - ground).abs() < 0.3).then(|| {
                    let eye = stand * (PLANET_RADIUS + height + pbd_app::walking::EYE_HEIGHT);
                    (up.distance(anchor), (record.key(), top, up, base, eye))
                })
            })
            .collect();
        spots.sort_by(|a, b| a.0.total_cmp(&b.0));
        spots.into_iter().map(|(_, spot)| spot).collect()
    }

    /// Interact on F (owner, 2026-11-16: "I should be able to harvest
    /// plants by pressing F on them"): looking at a ripe crop's leaves, or a
    /// wild plant's, through the real input path - the farm's offer, the
    /// prompt's resolve, F, `use_key` - names it in the prompt, and F
    /// harvests the crop and picks the plant into the pack and the save.
    /// The plant is found by its own shape: the block the eye ray meets is
    /// the ground beyond it.
    #[test]
    fn f_harvests_a_ripe_crop_and_picks_a_wild_plant() {
        use bevy::time::TimeUpdateStrategy;
        use pbd_app::interact::{InteractSet, resolve};
        use std::time::Duration;
        let anchor = super::super::spawn_direction(&super::super::Launch::parse(&[
            "--walk".into(),
            "--spawn".into(),
            "fields".into(),
        ]));
        let (fine, contact) = pbd_app::planet::test_ground(anchor);
        let farm = FarmConfig::default();
        let mut save = WorldSave::memory_only();
        let spots = grassy_spots(&fine, anchor);
        let wild = *spots
            .iter()
            .find(|(cell, top, ..)| wild_at(&fine, &save, &farm, *cell, *top, 0.0).is_some())
            .expect("a wild plant near the fields spawn");
        let sown = *spots
            .iter()
            .find(|(cell, top, ..)| {
                *cell != wild.0 && wild_at(&fine, &save, &farm, *cell, *top, 0.0).is_none()
            })
            .expect("a grassy cell to sow");
        let hotbar = super::super::slots::Hotbar::restore(&mut save);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / 60.0,
            )))
            .insert_resource(fine)
            .insert_resource(contact)
            .insert_resource(save)
            .insert_resource(hotbar)
            .insert_resource(farm)
            .insert_resource(pbd_app::walking::WalkingReadout {
                active: true,
                captured: true,
                ..default()
            })
            .insert_resource(pbd_app::sky::Sun::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Aim>()
            .init_resource::<Screen>()
            .init_resource::<pbd_app::fish::ToolSlot>()
            .init_resource::<Drops>()
            .init_resource::<FarmNote>()
            .init_resource::<StationAt>()
            .init_resource::<Interact>()
            .add_message::<Gained>()
            .add_message::<pbd_app::missions::Did>()
            .configure_sets(
                PostUpdate,
                (InteractSet::Offer, InteractSet::Resolve).chain(),
            )
            .add_systems(Update, use_key)
            .add_systems(
                PostUpdate,
                (
                    offer.in_set(InteractSet::Offer),
                    resolve.in_set(InteractSet::Resolve),
                ),
            );
        // Till and sow the crop's cell, sown long enough ago to be ripe.
        {
            let world = app.world_mut();
            let now_s = world.resource::<pbd_app::sky::Sun>().clock.seconds;
            world.resource_scope(|world, mut fine: Mut<PlanetFine>| {
                world.resource_scope(|world, mut contact: Mut<PlanetContact>| {
                    world.resource_scope(|world, mut save: Mut<WorldSave>| {
                        world.resource_scope(
                            |world, mut slots: Mut<super::super::slots::Hotbar>| {
                                let mut drops = world.resource_mut::<Drops>();
                                let tilled = apply_edit(
                                    &mut Edited {
                                        fine: &mut fine,
                                        contact: &mut contact,
                                        save: &mut save,
                                        slots: &mut slots,
                                        drops: &mut drops,
                                        now_s,
                                    },
                                    Hands::Till(None),
                                    sown.0,
                                    sown.1,
                                    Material::Tilled,
                                );
                                assert!(tilled.is_some(), "the cell is tilled");
                                let plot = Plot::sown(sown.1 as u16, Crop::WHEAT, now_s - 1.0e8);
                                assert!(save.store_with_hands(
                                    plot_record(sown.0, &plot),
                                    PLOT_SCHEMA,
                                    &slots.0
                                ));
                            },
                        );
                    });
                });
            });
        }
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Camera::default(), Transform::default()))
            .id();
        // Stand 1.6 m off a plant and look at its leaves, 0.9 m up: the ray goes on to the ground two metres past it, outside its cell.
        let look_at = |app: &mut App, (_, _, up, base, eye): Spot| {
            let at = Transform::from_translation(eye).looking_at(base + up * 0.9, up);
            app.world_mut()
                .entity_mut(camera)
                .insert((at, GlobalTransform::from(at)));
            let look = at.forward().as_vec3();
            let target = {
                let world = app.world();
                let (fine, contact) = (
                    world.resource::<PlanetFine>(),
                    world.resource::<PlanetContact>(),
                );
                pbd_core::aim::march(eye, look, |p| digging::sample_at(fine, contact, p))
            };
            app.world_mut().resource_mut::<Aim>().target = target;
            for _ in 0..3 {
                app.update();
            }
            target
        };
        let press_f = |app: &mut App| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyF);
            app.update();
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyF);
            keys.clear();
        };
        let carried = |app: &App, good: pbd_core::goods::Good| {
            app.world()
                .resource::<super::super::slots::Hotbar>()
                .0
                .count(Item::Good(good))
        };

        // The ripe wheat.
        let target = look_at(&mut app, sown);
        let world = app.world();
        let plant = {
            let (eye, look) = {
                let t = world.get::<GlobalTransform>(camera).unwrap();
                (t.translation(), t.forward().as_vec3())
            };
            plant_on_ray(
                world.resource::<PlanetFine>(),
                world.resource::<WorldSave>(),
                world.resource::<FarmConfig>(),
                eye,
                look,
                0.0,
            )
            .expect("the ray meets the wheat")
        };
        assert_eq!(plant.cell, sown.0);
        assert!(
            target.is_none_or(|t| t.dig.cell != sown.0 && t.distance > plant.distance),
            "the block the ray meets is beyond the plant: {target:?}"
        );
        assert_eq!(
            world.resource::<Interact>().now.as_ref().map(Offer::words),
            Some(Localized::verbatim("Harvest wheat"))
        );
        press_f(&mut app);
        let save = app.world().resource::<WorldSave>();
        assert_eq!(
            plot_at(save, sown.0).and_then(|p| p.crop),
            None,
            "the plot is saved bare"
        );
        assert!(
            carried(&app, Crop::WHEAT.row().produce) > 0,
            "wheat in the pack"
        );

        // The wild plant.
        look_at(&mut app, wild);
        let crop = {
            let world = app.world();
            farm_act(
                world.resource::<PlanetFine>(),
                world.resource::<WorldSave>(),
                world.resource::<FarmConfig>(),
                None,
                Some(PlantHit {
                    cell: wild.0,
                    layer: wild.1,
                    distance: 1.0,
                }),
                None,
                false,
                0.0,
                4,
            )
        };
        let Some(FarmAct::Wild { crop, .. }) = crop else {
            panic!("a wild plant: {crop:?}");
        };
        assert_eq!(
            app.world()
                .resource::<Interact>()
                .now
                .as_ref()
                .map(Offer::words),
            Some(tr_with(
                "interact.act.pick_wild",
                &[("thing", &crop.row().name())]
            ))
        );
        let seeds = carried(&app, crop.row().seed);
        press_f(&mut app);
        let save = app.world().resource::<WorldSave>();
        assert!(
            save.records.get(WILD_KIND, u64::from(wild.0)).is_some(),
            "the picked plant is in the save"
        );
        assert!(
            carried(&app, crop.row().seed) > seeds,
            "its seeds in the pack"
        );
        // Nothing left to do there: the prompt goes after its hold.
        for _ in 0..30 {
            app.update();
        }
        assert_eq!(app.world().resource::<Interact>().now, None);
    }

    /// F on the ground clutter (`gather-by-hand`; owner, 2026-11-16: "if I
    /// see like a shrub or something I should be able press F to collect
    /// it. same with a rock"): standing by a piece the draw puts down and
    /// looking at it, through the real input path, the prompt offers it and
    /// F takes its goods into the pack, a `clutter` record into the save,
    /// and the piece off the ground, so it is not offered again.
    #[test]
    fn f_gathers_a_stone_or_fibre_from_the_ground_clutter() {
        use bevy::time::TimeUpdateStrategy;
        use pbd_app::interact::{InteractSet, resolve};
        use std::time::Duration;
        let anchor = super::super::spawn_direction(&super::super::Launch::parse(&[
            "--walk".into(),
            "--spawn".into(),
            "fields".into(),
        ]));
        let (fine, contact) = pbd_app::planet::test_ground(anchor);
        let scatter = pbd_app::config::ScatterSettings::default();
        let odds = pbd_app::clutter::odds(&scatter);
        // The nearest pebble and the nearest fibre-giver on dry, level
        // ground, each with a place to stand 1.4 m off it.
        let mut found: Vec<(f32, u32, pbd_core::scatter::Piece, Vec3)> = Vec::new();
        for record in fine.set.finest_records() {
            let up = Vec3::from_slice(&record.direction_height[..3]).normalize();
            let d = up.distance(anchor) * PLANET_RADIUS;
            if d > 120.0 {
                continue;
            }
            for piece in pbd_app::clutter::pieces_of(record, &odds) {
                let east = Vec3::Y.cross(up).normalize();
                let stand = (piece.seat + east * 1.4).normalize();
                let height = pbd_app::planet::surface_height(stand);
                if height < 0.5 || (PLANET_RADIUS + height - piece.seat.length()).abs() > 0.3 {
                    continue;
                }
                let eye = stand * (PLANET_RADIUS + height + pbd_app::walking::EYE_HEIGHT);
                found.push((d, record.key(), piece, eye));
            }
        }
        found.sort_by(|a, b| a.0.total_cmp(&b.0));
        let pebble = *found
            .iter()
            .find(|f| f.2.kind == pbd_core::scatter::Kind::Pebble)
            .expect("a pebble near the fields spawn");
        let fibre = *found
            .iter()
            .find(|f| f.2.kind != pbd_core::scatter::Kind::Pebble)
            .expect("a bush, shrub or fern near the fields spawn");
        let mut save = WorldSave::memory_only();
        let hotbar = super::super::slots::Hotbar::restore(&mut save);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / 60.0,
            )))
            .insert_resource(fine)
            .insert_resource(contact)
            .insert_resource(save)
            .insert_resource(hotbar)
            .insert_resource(FarmConfig::default())
            .insert_resource(scatter)
            .insert_resource(pbd_app::walking::WalkingReadout {
                active: true,
                captured: true,
                ..default()
            })
            .insert_resource(pbd_app::sky::Sun::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Aim>()
            .init_resource::<Screen>()
            .init_resource::<pbd_app::fish::ToolSlot>()
            .init_resource::<Drops>()
            .init_resource::<FarmNote>()
            .init_resource::<StationAt>()
            .init_resource::<Interact>()
            .add_message::<Gained>()
            .add_message::<pbd_app::missions::Did>()
            .configure_sets(
                PostUpdate,
                (InteractSet::Offer, InteractSet::Resolve).chain(),
            )
            .add_systems(Update, use_key)
            .add_systems(
                PostUpdate,
                (
                    offer.in_set(InteractSet::Offer),
                    resolve.in_set(InteractSet::Resolve),
                ),
            );
        let camera = app
            .world_mut()
            .spawn((Camera3d::default(), Camera::default(), Transform::default()))
            .id();
        let carried = |app: &App, code: &str| {
            app.world()
                .resource::<super::super::slots::Hotbar>()
                .0
                .count(Item::Good(pbd_core::goods::Good::by_code(code).unwrap()))
        };
        for (_, key, piece, eye) in [pebble, fibre] {
            let up = piece.seat.normalize();
            let at = Transform::from_translation(eye).looking_at(piece.centre, up);
            app.world_mut()
                .entity_mut(camera)
                .insert((at, GlobalTransform::from(at)));
            let look = at.forward().as_vec3();
            let target = {
                let world = app.world();
                let (fine, contact) = (
                    world.resource::<PlanetFine>(),
                    world.resource::<PlanetContact>(),
                );
                pbd_core::aim::march(eye, look, |p| digging::sample_at(fine, contact, p))
            };
            app.world_mut().resource_mut::<Aim>().target = target;
            for _ in 0..3 {
                app.update();
            }
            let (code, count) = piece.kind.gives();
            let name = pbd_core::goods::Good::by_code(code).unwrap().name();
            assert_eq!(
                app.world()
                    .resource::<Interact>()
                    .now
                    .as_ref()
                    .map(Offer::words),
                Some(tr_with("interact.act.gather", &[("thing", &name)])),
                "{:?} at cell {key}",
                piece.kind
            );
            let before = carried(&app, code);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyF);
            app.update();
            {
                let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
                keys.release(KeyCode::KeyF);
                keys.clear();
            }
            assert_eq!(
                carried(&app, code),
                before + u32::from(count),
                "{code} in the pack"
            );
            let world = app.world();
            let save = world.resource::<WorldSave>();
            let day = pbd_app::clutter::day_of(world.resource::<pbd_app::sky::Sun>().clock.seconds);
            assert!(
                pbd_app::clutter::taken_at(save, key).gone(piece.kind, day),
                "the take is in the save"
            );
            let again = pbd_app::clutter::clutter_on_ray(
                world.resource::<PlanetFine>(),
                save,
                &odds,
                day,
                eye,
                look,
            );
            assert!(
                again.is_none_or(|h| !(h.key == key && h.kind == piece.kind)),
                "a taken piece is not offered again"
            );
        }
    }
}
