//! Fire in the game (`wildfire` tasks 4.1 to 4.3): the app's half of
//! `pbd_core::fire`.
//!
//! - **The server's tick.** In a Survival world, once a world second, every
//!   live blaze whose origin is on the loaded tier takes one tick
//!   (`fire::blaze::run`), each through `WorldSave::propose` (refused whole
//!   if it touches the player's). An Explorer world never runs one.
//! - **The ground.** [`TierGround`] answers the rule from the fine tier's
//!   records: a cell's tree, cactus, grass and bush by the draw's own rules,
//!   its water and snow, its top; off the tier a cell is a break (design 13).
//!   The weather is the atmosphere's at the blaze's origin, once a tick.
//! - **Torches in the rain.** Every placed torch on the tier under the open
//!   sky soaks in the rain the player would see there, and goes out after
//!   `placed_out_s` as a `lamp_out` record of the world's.
//! - **What it changes in the field.** Burning cells light the column tier
//!   as lamps and an out torch does not ([`ColumnTier::set_fire`]); each
//!   burnt cell's record word carries its scar (no tufts, no generated tree,
//!   ash), brought round when the scar changes and on every new fine set.
//!
//! [`ColumnTier::set_fire`]: crate::planet::column::ColumnTier::set_fire

pub mod fx;

use crate::planet::{GpuCell, PLANET_RADIUS, PlanetContact, PlanetFine, terrain_config};
use crate::saves::WorldSave;
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_core::fire::blaze::{self, Air, Ground, Land};
use pbd_core::fire::{self, Burnt, FireConfig, GroundLook, Plant, TreeLook, torch};
use pbd_core::records::Proposal;
use pbd_core::scatter::{self, Kind, Odds};
use pbd_core::terrain::Material;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The rules, from `assets/config/fire.ron`.
#[derive(Resource, Clone, Debug)]
pub struct FireRules(pub FireConfig);

impl Default for FireRules {
    fn default() -> Self {
        Self(FireConfig::shipped())
    }
}

/// The bits of a finest record's material word a scar sets (design 13):
/// the road bit (no tufts or clutter), the cleared bit (no generated tree)
/// and the ash bit the ground's cap greys by.
pub const ROAD: u32 = 1 << crate::planet::terrain::ROAD_BIT;
pub const CLEARED: u32 = 1 << crate::planet::terrain::CLEARED_BIT;
/// `planet_surface.wgsl`'s `ASH_BIT`.
pub const ASH_BIT: u32 = 31;
pub const ASH: u32 = 1 << ASH_BIT;
pub const MARK_BITS: u32 = ROAD | CLEARED | ASH;

/// A burning cell where it burns, for the flames, their light and harm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flame {
    pub key: u32,
    /// The top of its ground in the planet's frame, metres (f64).
    pub at: DVec3,
    pub plant: Plant,
    pub crown: bool,
    /// Its time left, 0..1 of a fresh cell's.
    pub left: f32,
}

/// A burnt tree still standing charred, or growing back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snag {
    pub key: u32,
    pub at: DVec3,
    pub tree: pbd_core::tree_kit::Pick,
    pub look: TreeLook,
}

/// What burns and what has burnt on the tier now: what the flames, the
/// snags and the harm read. Rebuilt when the save or the tier changes.
#[derive(Resource, Clone, Debug, Default)]
pub struct FireNow {
    pub flames: Vec<Flame>,
    pub snags: Vec<Snag>,
    /// The world's wind at the fire, m/s, planet frame (the smoke leans).
    pub wind: Vec3,
    /// Small flames a torch made on a plant too green to burn (design 6):
    /// the cell, where, and the world second each dies. Saved nowhere.
    pub damp: Vec<(u32, DVec3, f64)>,
    /// Bumped when `flames` or `snags` change.
    pub version: u64,
}

impl FireNow {
    /// The nearest flame to a planet-frame point, and its distance, m.
    pub fn nearest(&self, at: DVec3) -> Option<(&Flame, f64)> {
        self.flames
            .iter()
            .map(|f| (f, f.at.distance(at)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}

/// The world's ground for the rule, read from the fine tier (design 13).
pub struct TierGround<'a> {
    pub set: &'a crate::planet::lod::FineSet,
    pub odds: Odds,
    pub air: Air,
    /// The scar bits this module set on a cell's word, by key, and the bits
    /// the word had before: what the cell's own word says.
    pub marked: &'a BTreeMap<u32, u32>,
}

impl TierGround<'_> {
    fn record(&self, key: u32) -> Option<&GpuCell> {
        let i = self.set.finest_index_of(key)?;
        self.set.finest_records().get(i)
    }

    /// A record's own word, with the scar's bits taken back off.
    fn word(&self, key: u32, record: &GpuCell) -> u32 {
        let w = record.metadata[1];
        match self.marked.get(&key) {
            Some(orig) => (w & !MARK_BITS) | orig,
            None => w,
        }
    }
}

/// What grows on a record, by the draw's own rules: a tree (its species),
/// a cactus, a bush on grass, grass, or nothing.
pub fn plant_of(record: &GpuCell, word: u32, odds: &Odds) -> Plant {
    let id = record.spare[0];
    if pbd_core::flora::grows_tree(&terrain_config().trees, id, word)
        && let Some(pick) = pbd_core::tree_kit::pick_word(id, word)
    {
        return Plant::of_tree(pick.species);
    }
    if pbd_core::tree_kit::grows_cactus(id, word) {
        return Plant::Cactus;
    }
    let material = word & 0xff;
    if !scatter::grassy(material) {
        return Plant::None;
    }
    let biome = ((word >> 8) & 0xff).min(7) as usize;
    if scatter::grows(
        Kind::Bush,
        record.metadata[3],
        material,
        odds.chance(Kind::Bush, biome),
    ) {
        Plant::Shrub
    } else {
        Plant::Grass
    }
}

impl Ground for TierGround<'_> {
    fn land(&self, key: u32) -> Option<Land> {
        let record = self.record(key)?;
        let word = self.word(key, record);
        let material = word & 0xff;
        let flooded = (word >> pbd_core::flora::FLOODED_BIT) & 1 == 1;
        Some(Land {
            plant: plant_of(record, word, &self.odds),
            top: record.direction_height[3].floor() as i32,
            water: flooded || material == crate::planet::terrain::WATER,
            snow: material == 6,
            pen: false,
        })
    }

    fn air(&self, _key: u32) -> Air {
        self.air
    }

    fn radius_m(&self) -> f32 {
        PLANET_RADIUS
    }
}

/// The weather at a direction as the rule reads it.
pub fn air_at(atmosphere: &pbd_core::atmosphere::Atmosphere, direction: Vec3) -> Air {
    let s = atmosphere.sample(direction);
    Air {
        ground_wet: s.ground_wet,
        humidity: s.humidity,
        air_c: s.air_c,
        rain_mm_h: s.rain_rate * 3600.0,
        wind: s.wind,
        snow: s.snow,
    }
}

/// The scar's bits on a burnt cell now (design 8 and 13).
pub fn scar_bits(cfg: &FireConfig, key: u32, burnt: &Burnt, now_s: f64) -> u32 {
    let mut bits = 0;
    if fire::scar::ground_look(cfg, key, burnt, now_s) != GroundLook::Grass {
        bits |= ROAD | ASH;
    }
    if burnt.plant.is_tree() && fire::scar::tree_look(cfg, burnt, now_s) != TreeLook::Whole {
        bits |= CLEARED;
    }
    bits
}

/// Bring a set's records round to the scar bits `want` (by key): the bits
/// on, and every cell this module marked before and not wanted now put back
/// to its own word. Returns how many records changed.
pub fn apply_scars(
    set: &mut crate::planet::lod::FineSet,
    want: &BTreeMap<u32, u32>,
    marked: &mut BTreeMap<u32, u32>,
) -> usize {
    let mut changed = 0;
    let keys: Vec<u32> = want.keys().chain(marked.keys()).copied().collect();
    for key in keys {
        let Some(i) = set.finest_index_of(key) else {
            continue;
        };
        let w = set.levels[3][i].metadata[1];
        // The word's own bits: known from before, or this word's, which a
        // set built from the generator carries unmarked.
        let orig = *marked.entry(key).or_insert(w & (ROAD | CLEARED));
        let next = (w & !MARK_BITS) | orig | want.get(&key).copied().unwrap_or(0);
        if next != w {
            set.levels[3][i].metadata[1] = next;
            changed += 1;
        }
    }
    changed
}

/// The bush odds the draw rolls by, from `scatter.ron`.
pub fn odds_of(scatter: Option<&crate::config::ScatterSettings>) -> Odds {
    match scatter {
        Some(s) => crate::clutter::odds(s),
        None => crate::clutter::odds(&crate::config::ScatterSettings::default()),
    }
}

/// Whether bringing `set` round to `want` would change any record.
pub fn scars_differ(
    set: &crate::planet::lod::FineSet,
    want: &BTreeMap<u32, u32>,
    marked: &BTreeMap<u32, u32>,
) -> bool {
    want.keys().chain(marked.keys()).any(|&key| {
        let Some(i) = set.finest_index_of(key) else {
            return false;
        };
        let w = set.finest_records()[i].metadata[1];
        let orig = marked.get(&key).copied().unwrap_or(w & (ROAD | CLEARED));
        (w & !MARK_BITS) | orig | want.get(&key).copied().unwrap_or(0) != w
    })
}

/// The world second now, and its year fraction.
fn clock(sun: &crate::sky::Sun) -> (f64, f64) {
    (sun.clock.seconds, sun.clock.year_fraction())
}

/// The scar marks this module put on the tier's words, and the originals.
#[derive(Resource, Default)]
pub struct Scars {
    pub marked: BTreeMap<u32, u32>,
    /// What [`show`] last built from: the fire records' stamp, the fine
    /// set it saw, and the world second.
    seen: (u64, u64, f64),
}

/// MEASUREMENT (capture instrument, `wildfire` 4.2's shots):
/// `PBD_FIRE=<scene>` makes a capture's world a Survival one in a dry late
/// summer (the weather the rule reads is held at two dry weeks, warm, with
/// no wind) and sets a scene once the tier is in:
/// - `front[:<ticks>]`: a blaze lit by a torch on the driest plant in
///   view, 5 to 20 m ahead (24 for a scar), and run up to `<ticks>` world
///   seconds at once (60 unset), stopped while it still burns, so the shot
///   shows its flames;
/// - `scar:<days>`: the same blaze run until it is out, `<days>` world days
///   before the clock, so the shot shows its scar;
/// - `torch-out`: every placed torch put out by the rain.
///
/// Unset, nothing is forced.
pub fn capture_scene() -> Option<String> {
    std::env::var("PBD_FIRE").ok()
}

/// The air the rule reads under [`capture_scene`]: two dry weeks of late
/// summer, no wind. `None` when no scene is set.
fn forced_air() -> Option<Air> {
    capture_scene()?;
    Some(Air {
        ground_wet: 0.12,
        humidity: 0.32,
        air_c: 29.0,
        rain_mm_h: 0.0,
        wind: Vec3::ZERO,
        snow: false,
    })
}

/// Late summer at a direction's own hemisphere.
fn late_summer_at(direction: Vec3) -> f64 {
    if direction.y >= 0.0 { 0.10 } else { 0.60 }
}

/// The capture instrument's system ([`capture_scene`]).
#[allow(clippy::too_many_arguments)]
pub fn instrument(
    rules: Res<FireRules>,
    save: Option<ResMut<WorldSave>>,
    fine: Option<Res<PlanetFine>>,
    air: Option<Res<crate::atmosphere::Air>>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut done: Local<u32>,
) {
    let Some(scene) = capture_scene() else {
        return;
    };
    let missing = [
        save.is_none(),
        fine.is_none(),
        air.is_none(),
        sun.is_none(),
        frame.is_none(),
    ];
    let (Some(mut save), Some(fine), Some(air), Some(sun), Some(frame)) =
        (save, fine, air, sun, frame)
    else {
        if *done == 0 {
            info!("FIRE_CAPTURE {scene}: waiting on {missing:?}");
            *done = u32::MAX;
        }
        return;
    };
    if *done == u32::MAX {
        *done = 0;
    }
    if *done == 0 {
        save.identity.mode = Some(pbd_core::survival::WorldMode::Survival);
    }
    *done += 1;
    // A few frames in, once the camera stands where the shot is taken.
    if *done != 20 {
        return;
    }
    let cfg = &rules.0;
    if scene == "torch-out" {
        let torches: Vec<(u32, u16)> = save
            .edits
            .all()
            .into_iter()
            .filter(|e| e.material == Material::Torch)
            .map(|e| (e.cell, e.layer))
            .collect();
        for (cell, layer) in torches {
            let proposal = Proposal {
                edits: Vec::new(),
                records: vec![torch::out_record(&save.records, cell, layer)],
            };
            save.note_record_kinds(&torch::KINDS);
            let _ = save.propose(&torch::author(cell), proposal);
            info!("FIRE_CAPTURE torch {cell}/{layer} put out");
        }
        return;
    }
    let Some((_, eye)) = cameras.iter().find(|(c, _)| c.is_active) else {
        warn!("FIRE_CAPTURE {scene}: no camera");
        return;
    };
    let at = eye.translation().as_dvec3() - frame.center;
    let up = at.normalize_or(DVec3::Y);
    let mut ahead = eye.forward().as_vec3().as_dvec3();
    ahead -= up * ahead.dot(up);
    let target = (at + ahead.normalize_or(DVec3::X) * 14.0)
        .normalize()
        .as_vec3();
    let year = late_summer_at(target);
    let marked = BTreeMap::new();
    let odds = odds_of(None);
    // The dry plant nearest the point ahead.
    let ground = TierGround {
        set: &fine.set,
        odds,
        air: forced_air().unwrap_or_default(),
        marked: &marked,
    };
    let world = blaze::World {
        cfg,
        seed: save.identity.seed,
        ground: &ground,
        records: &save.records,
        yields: &save.yields,
        now_s: sun.clock.seconds,
        year_fraction: year,
    };
    let near: Vec<u32> = fine
        .set
        .finest_records()
        .iter()
        // In view: 5 to 20 m ahead (24 for a scar), within 25 degrees of
        // the look.
        .filter(|r| {
            let d = Vec3::from_slice(&r.direction_height[..3])
                .normalize_or(Vec3::Y)
                .as_dvec3();
            let mut off = d * (f64::from(PLANET_RADIUS) + f64::from(r.direction_height[3])) - at;
            off -= up * off.dot(up);
            let along = off.dot(ahead.normalize_or(DVec3::X));
            let far = if scene.starts_with("scar") {
                24.0
            } else {
                20.0
            };
            (5.0..far).contains(&along) && off.length() * 0.906 <= along
        })
        .map(|r| r.key())
        .filter(|&k| ground.land(k).is_some_and(|l| l.plant.burns()))
        .collect();
    // The driest plant near the point ahead.
    let best = near
        .iter()
        .map(|&k| (world.plant_dry(k), k))
        .max_by(|a, b| a.0.total_cmp(&b.0));
    info!(
        "FIRE_CAPTURE {scene}: {} burnable cells near, the driest {best:?}",
        near.len()
    );
    let Some((_, cell)) = best.filter(|(d, _)| *d >= cfg.dryness.dry_min) else {
        warn!("FIRE_CAPTURE no dry plant ahead");
        return;
    };
    let (ticks, back_days) = match scene.split_once(':') {
        Some(("front", n)) => (n.parse().unwrap_or(60), 0.0),
        Some(("scar", d)) => (900, d.parse::<f64>().unwrap_or(1.0)),
        _ if scene == "front" => (60, 0.0),
        _ => {
            warn!("PBD_FIRE {scene}: no such scene");
            return;
        }
    };
    let day = f64::from(pbd_core::daylight::DAY_S);
    let start = sun.clock.seconds - back_days * day - if back_days > 0.0 { 1000.0 } else { 0.0 };
    let lit = light(
        cfg, &mut save, &fine.set, &air.now, &marked, &odds, cell, start, year,
    );
    info!("FIRE_CAPTURE {scene}: {lit:?} at cell {cell}");
    let mut ran = 0;
    let mut peak = 0;
    for k in 0..ticks {
        if run_once(
            cfg,
            &mut save,
            &fine.set,
            &air.now,
            &marked,
            &odds,
            start + 1.0 + f64::from(k),
            year,
        ) == 0
        {
            break;
        }
        ran += 1;
        // A front's shot is taken while it burns: stopped once it is past
        // its height and before it is out.
        let burning: usize = blaze::live(&save.records)
            .iter()
            .map(|(_, b)| b.burning.len())
            .sum();
        peak = peak.max(burning);
        if back_days == 0.0 && ran >= 8 && burning > 0 && (burning < peak || burning <= 2) {
            break;
        }
    }
    info!(
        "FIRE_CAPTURE ran {ran} ticks: {} burning, {} burnt",
        blaze::live(&save.records)
            .iter()
            .map(|(_, b)| b.burning.len())
            .sum::<usize>(),
        save.records.of_kind(blaze::BURNT_KIND).count()
    );
}

/// Once a world second, in a Survival world: every live blaze on the tier
/// takes a tick, through the save.
#[allow(clippy::too_many_arguments)]
pub fn tick(
    rules: Res<FireRules>,
    save: Option<ResMut<WorldSave>>,
    fine: Option<Res<PlanetFine>>,
    air: Option<Res<crate::atmosphere::Air>>,
    sun: Option<Res<crate::sky::Sun>>,
    scatter: Option<Res<crate::config::ScatterSettings>>,
    scars: Res<Scars>,
    mut last: Local<Option<u64>>,
) {
    let (Some(mut save), Some(fine), Some(air), Some(sun)) = (save, fine, air, sun) else {
        return;
    };
    if !save.mode().is_survival() {
        return;
    }
    let (now_s, year) = clock(&sun);
    let second = now_s.max(0.0).floor() as u64;
    let from = last.unwrap_or(second.saturating_sub(1));
    *last = Some(second);
    if second <= from {
        return;
    }
    // At most a few seconds caught up in one frame: a long hitch is not a
    // reason to run a minute of fire at once.
    let ticks = (second - from).min(4);
    let odds = odds_of(scatter.as_deref());
    for k in 0..ticks {
        run_once(
            &rules.0,
            &mut save,
            &fine.set,
            &air.now,
            &scars.marked,
            &odds,
            (from + k + 1) as f64,
            year,
        );
    }
}

/// One tick of every live blaze on the tier. Returns how many it ran.
#[allow(clippy::too_many_arguments)]
pub fn run_once(
    cfg: &FireConfig,
    save: &mut WorldSave,
    set: &crate::planet::lod::FineSet,
    atmosphere: &pbd_core::atmosphere::Atmosphere,
    marked: &BTreeMap<u32, u32>,
    odds: &Odds,
    now_s: f64,
    year_fraction: f64,
) -> usize {
    let live = blaze::live(&save.records);
    let ids: Vec<u64> = live
        .iter()
        .filter(|(_, b)| set.finest_index_of(b.origin).is_some())
        .map(|(id, _)| *id)
        .collect();
    let Some(first) = live.iter().find(|(id, _)| ids.contains(id)) else {
        return 0;
    };
    let toward = pbd_core::roads::centre(first.1.origin).unwrap_or(Vec3::Y);
    let air = forced_air().unwrap_or_else(|| air_at(atmosphere, toward));
    let year_fraction = if forced_air().is_some() {
        late_summer_at(toward)
    } else {
        year_fraction
    };
    let ground = TierGround {
        set,
        odds: *odds,
        air,
        marked,
    };
    let world = blaze::World {
        cfg,
        seed: save.identity.seed,
        ground: &ground,
        records: &save.records,
        yields: &save.yields,
        now_s,
        year_fraction,
    };
    let proposals = blaze::run(&world, &ids);
    let ran = proposals.len();
    for (author, proposal) in proposals {
        if let Err(why) = save.propose(&author, proposal) {
            warn!("FIRE {} refused: {why:?}", author.token());
        }
    }
    ran
}

/// Light a blaze at `cell` by player `player`: the player's ignition record
/// and the world's blaze. What happened: a blaze, a damp flame or nothing.
#[allow(clippy::too_many_arguments)]
pub fn light(
    cfg: &FireConfig,
    save: &mut WorldSave,
    set: &crate::planet::lod::FineSet,
    atmosphere: &pbd_core::atmosphere::Atmosphere,
    marked: &BTreeMap<u32, u32>,
    odds: &Odds,
    cell: u32,
    now_s: f64,
    year_fraction: f64,
) -> blaze::Lighting {
    if !save.mode().is_survival() {
        return blaze::Lighting::Nothing;
    }
    let toward = pbd_core::roads::centre(cell).unwrap_or(Vec3::Y);
    let air = forced_air().unwrap_or_else(|| air_at(atmosphere, toward));
    let year_fraction = if forced_air().is_some() {
        late_summer_at(toward)
    } else {
        year_fraction
    };
    let ground = TierGround {
        set,
        odds: *odds,
        air,
        marked,
    };
    let world = blaze::World {
        cfg,
        seed: save.identity.seed,
        ground: &ground,
        records: &save.records,
        yields: &save.yields,
        now_s,
        year_fraction,
    };
    let lighting = blaze::ignite(&world, cell, blaze::LitBy::Player(0));
    if let blaze::Lighting::Blaze(b) = &lighting {
        let id = blaze::next_id(&save.records);
        save.note_record_kinds(&fire::record_kinds());
        let by = pbd_core::records::Author::Player(0);
        if save
            .store(&by, vec![blaze::ignition_record(id, cell, now_s)])
            .is_none()
        {
            return blaze::Lighting::Nothing;
        }
        let proposal = Proposal {
            edits: Vec::new(),
            records: vec![b.record(id)],
        };
        if let Err(why) = save.propose(&blaze::author(id), proposal) {
            warn!("FIRE blaze {id} refused: {why:?}");
            return blaze::Lighting::Nothing;
        }
        info!("FIRE blaze {id} lit at cell {cell}");
    }
    lighting
}

/// The torch in the player's hand (design 9): out in the rain, and how long
/// it has stood in it. Not saved: a torch taken up again is lit.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TorchOut {
    pub held: bool,
    pub soak: torch::Soak,
}

/// How long each placed torch has stood in the rain, by `(key, layer)`.
#[derive(Resource, Default)]
pub struct Soaking(pub BTreeMap<(u32, u16), torch::Soak>);

/// Once a second, in a Survival world: every placed torch on the tier
/// soaks in the rain it stands in, and goes out after `placed_out_s`.
pub fn soak_torches(
    rules: Res<FireRules>,
    save: Option<ResMut<WorldSave>>,
    fine: Option<Res<PlanetFine>>,
    air: Option<Res<crate::atmosphere::Air>>,
    time: Res<Time>,
    mut soaking: ResMut<Soaking>,
    mut every: Local<f32>,
) {
    let (Some(mut save), Some(fine), Some(air)) = (save, fine, air) else {
        return;
    };
    if !save.mode().is_survival() {
        soaking.0.clear();
        return;
    }
    *every += time.delta_secs();
    if *every < 1.0 {
        return;
    }
    let dt = std::mem::take(&mut *every);
    soak_once(&rules.0, &mut save, &fine.set, &air.now, &mut soaking, dt);
}

/// One soak of every placed torch on the tier by `dt` seconds.
pub fn soak_once(
    cfg: &FireConfig,
    save: &mut WorldSave,
    set: &crate::planet::lod::FineSet,
    atmosphere: &pbd_core::atmosphere::Atmosphere,
    soaking: &mut Soaking,
    dt: f32,
) -> usize {
    let torches: Vec<(u32, u16)> = save
        .edits
        .all()
        .into_iter()
        .filter(|e| e.material == Material::Torch)
        .map(|e| (e.cell, e.layer))
        .collect();
    soaking.0.retain(|k, _| torches.contains(k));
    let mut out = 0;
    for (cell, layer) in torches {
        if torch::is_out(&save.records, cell, layer) {
            continue;
        }
        let Some(record) = set.finest_index_of(cell) else {
            continue;
        };
        let Some(column) = set.columns.column(record) else {
            continue;
        };
        let sample = atmosphere.sample(pbd_core::roads::centre(cell).unwrap_or(Vec3::Y));
        let exposure = torch::Exposure {
            rain_seen: sample.rain_seen,
            snow: sample.snow,
            open_sky: torch::open_sky(cfg, layer, |l| {
                (l as usize) < pbd_core::column::LAYERS && column.solid(l as usize)
            }),
        };
        let soak = soaking.0.entry((cell, layer)).or_default();
        if soak.step(cfg, exposure, false, dt) {
            let proposal = Proposal {
                edits: Vec::new(),
                records: vec![torch::out_record(&save.records, cell, layer)],
            };
            save.note_record_kinds(&torch::KINDS);
            match save.propose(&torch::author(cell), proposal) {
                Ok(_) => {
                    out += 1;
                    info!("FIRE torch at {cell}/{layer} out in the rain");
                }
                Err(why) => warn!("FIRE torch out refused: {why:?}"),
            }
        }
    }
    out
}

/// Relight the placed torch at `cell`, `layer`: the player's record.
pub fn relight(save: &mut WorldSave, cell: u32, layer: u16) -> bool {
    if !torch::is_out(&save.records, cell, layer) {
        return false;
    }
    let record = torch::lit_record(&save.records, cell, layer);
    save.note_record_kinds(&torch::KINDS);
    save.store(&pbd_core::records::Author::Player(0), vec![record])
        .is_some()
}

/// The light a burning cell's flames give the field (0..15): a tree's a
/// brazier's, grass a campfire's.
pub fn flame_level(plant: Plant) -> u8 {
    if plant.is_tree() {
        Material::Brazier.emission()
    } else {
        Material::Campfire.emission()
    }
}

/// What the tier and [`FireNow`] should show now: the burning cells' light,
/// the out torches, the scar bits and the snags.
pub fn shown(
    cfg: &FireConfig,
    save: &WorldSave,
    set: &crate::planet::lod::FineSet,
    now_s: f64,
) -> (
    crate::planet::column::TierFire,
    BTreeMap<u32, u32>,
    Vec<Flame>,
    Vec<Snag>,
) {
    let point = |key: u32| -> Option<DVec3> {
        let i = set.finest_index_of(key)?;
        let r = &set.finest_records()[i];
        let d = Vec3::from_slice(&r.direction_height[..3])
            .normalize_or(Vec3::Y)
            .as_dvec3();
        Some(d * (f64::from(PLANET_RADIUS) + f64::from(r.direction_height[3])))
    };
    let mut lights = Vec::new();
    let mut flames = Vec::new();
    for (_, b) in blaze::live(&save.records) {
        for c in &b.burning {
            let Some(at) = point(c.cell) else {
                continue;
            };
            lights.push((c.cell, flame_level(c.plant)));
            let full = blaze_full_ds(cfg, c.plant);
            flames.push(Flame {
                key: c.cell,
                at,
                plant: c.plant,
                crown: c.crown,
                left: (f32::from(c.left_ds) / full).clamp(0.0, 1.0),
            });
        }
    }
    lights.sort_unstable();
    lights.dedup_by_key(|l| l.0);
    let mut want = BTreeMap::new();
    let mut snags = Vec::new();
    for r in save.records.of_kind(blaze::BURNT_KIND) {
        let key = r.id as u32;
        let Some(burnt) = r.read::<Burnt>() else {
            continue;
        };
        let Some(i) = set.finest_index_of(key) else {
            continue;
        };
        let bits = scar_bits(cfg, key, &burnt, now_s);
        if bits != 0 {
            want.insert(key, bits);
        }
        if burnt.plant.is_tree() {
            let look = fire::scar::tree_look(cfg, &burnt, now_s);
            let record = &set.finest_records()[i];
            // The tree's own word, without the cleared bit this module set.
            let word = record.metadata[1] & !CLEARED;
            if look != TreeLook::Whole
                && let (Some(at), Some(tree)) = (
                    point(key),
                    pbd_core::tree_kit::pick_word(record.spare[0], word),
                )
            {
                snags.push(Snag {
                    key,
                    at,
                    tree,
                    look,
                });
            }
        }
    }
    let fire = crate::planet::column::TierFire {
        burning: lights,
        out: torch::out_lamps(&save.records)
            .into_iter()
            .filter(|(c, _)| set.finest_index_of(*c).is_some())
            .collect(),
    };
    (fire, want, flames, snags)
}

fn blaze_full_ds(cfg: &FireConfig, plant: Plant) -> f32 {
    let f = &cfg.fuel;
    let s = match plant {
        Plant::Grass => f.grass.burn_s,
        Plant::Shrub => f.shrub.burn_s,
        Plant::Deciduous => f.deciduous.burn_s,
        Plant::Conifer => f.conifer.burn_s,
        Plant::Evergreen => f.evergreen.burn_s,
        Plant::None | Plant::Cactus => 1.0,
    };
    (s * 10.0).max(1.0)
}

/// What changes when the fire's records do: the world's opening, how many
/// records it holds, every blaze's ticks and every torch's counts.
fn fire_stamp(save: &WorldSave) -> u64 {
    let mut h = save.serial().wrapping_mul(1_000_003) ^ save.records.len() as u64;
    for kind in [blaze::BLAZE_KIND, torch::OUT_KIND, torch::LIT_KIND] {
        for r in save.records.of_kind(kind) {
            h = h.wrapping_mul(31).wrapping_add(r.id ^ r.body.len() as u64);
            h = h.wrapping_add(
                r.body
                    .bytes()
                    .fold(0u64, |a, b| a.wrapping_mul(131).wrapping_add(u64::from(b))),
            );
        }
    }
    h
}

/// Bring the tier round to the fire: its light, the out torches and the
/// scars, when the save or the tier changes and every few seconds as a scar
/// regrows; and [`FireNow`] for the flames and the harm.
#[allow(clippy::too_many_arguments)]
pub fn show(
    rules: Res<FireRules>,
    save: Option<Res<WorldSave>>,
    fine: Option<ResMut<PlanetFine>>,
    contact: Option<ResMut<PlanetContact>>,
    sun: Option<Res<crate::sky::Sun>>,
    air: Option<Res<crate::atmosphere::Air>>,
    mut scars: ResMut<Scars>,
    mut now: ResMut<FireNow>,
) {
    let (Some(save), Some(mut fine), Some(sun)) = (save, fine, sun) else {
        return;
    };
    let (now_s, _) = clock(&sun);
    let (serial, version, at) = scars.seen;
    let stamp = fire_stamp(&save);
    // The set itself, not its version: the water's tick lands a set of its
    // own (built from the one before this module's marks) without bumping
    // the version when nothing of the water moved.
    let set_id = Arc::as_ptr(&fine.set) as u64;
    if serial == stamp && version == set_id && (now_s - at).abs() < 5.0 {
        return;
    }
    let (tier_fire, want, flames, snags) = shown(&rules.0, &save, &fine.set, now_s);
    if !want.is_empty() || !flames.is_empty() {
        debug!(
            "FIRE shown: {} flames, {} snags, {} scars, {} burnt records",
            flames.len(),
            snags.len(),
            want.len(),
            save.records.of_kind(blaze::BURNT_KIND).count()
        );
    }
    let wind = flames
        .first()
        .zip(air.as_ref())
        .map_or(Vec3::ZERO, |(f, air)| {
            air.now.sample(f.at.normalize().as_vec3()).wind
        });
    now.damp.retain(|d| d.2 > now_s);
    if flames != now.flames || snags != now.snags {
        now.flames = flames;
        now.snags = snags;
        now.version += 1;
    }
    now.wind = wind;
    let mut set = None;
    if *fine.set.columns.fire() != tier_fire {
        let mut s = (*fine.set).clone();
        s.columns.set_fire(tier_fire);
        set = Some(s);
    }
    if scars_differ(set.as_ref().unwrap_or(&*fine.set), &want, &scars.marked) {
        let mut s = set.take().unwrap_or_else(|| (*fine.set).clone());
        let changed = apply_scars(&mut s, &want, &mut scars.marked);
        info!(
            "FIRE scars: {} cells marked, {changed} records changed",
            want.len()
        );
        set = Some(s);
    }
    if let Some(set) = set {
        let set = Arc::new(set);
        if let Some(mut contact) = contact {
            contact.set_fine(&set);
        }
        fine.set = set;
        fine.version += 1;
    }
    scars.seen = (stamp, Arc::as_ptr(&fine.set) as u64, now_s);
}

/// The cell spacing every body shares, m (CLAUDE.md, "Hex size is fixed").
const CELL_M: f64 = 2.833;

/// Health a second a fire takes from a body at `feet` (design 10): in a
/// burning cell, or beside one.
pub fn harm_at(cfg: &FireConfig, now: &FireNow, feet: DVec3) -> f32 {
    let Some((_, d)) = now.nearest(feet) else {
        return 0.0;
    };
    fire::harm_per_s(cfg, d < CELL_M * 0.6, d < CELL_M * 1.5)
}

/// Which way an animal at `at` (planet frame) runs from the fire: away from
/// the nearest burning cell along the ground, if one is within `flee_m`.
pub fn flee_from(cfg: &FireConfig, now: &FireNow, at: DVec3) -> Option<DVec3> {
    let (f, d) = now.nearest(at)?;
    if d > f64::from(cfg.harm.flee_m) {
        return None;
    }
    let up = at.normalize_or(DVec3::Y);
    let mut away = at - f.at;
    away -= up * away.dot(up);
    Some(
        away.try_normalize()
            .unwrap_or_else(|| up.any_orthonormal_vector()),
    )
}

pub struct WildfirePlugin;

impl Plugin for WildfirePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FireRules>()
            .init_resource::<FireNow>()
            .init_resource::<Scars>()
            .init_resource::<Soaking>()
            .init_resource::<TorchOut>()
            .add_systems(
                Update,
                (instrument, tick, soak_torches, show)
                    .chain()
                    .run_if(crate::world_open),
            );
        fx::build(app);
    }
}

#[cfg(test)]
mod tests;
