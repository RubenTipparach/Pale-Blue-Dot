//! Marking and paving in the game (`townsfolk` design 39 to 41, 43 and 44): survey
//! stakes driven with the place button, the cells between them marked with
//! a lime band the terrain draws, and each marked cell paved with gravel by
//! the interact key with the shovel in hand.
//!
//! Every stake, take-back and paved cell is one durable line with the pack
//! (`WorldSave::store_with_hands`). The marks and the player's road cells
//! are installed for the terrain ([`install`]) and brought into the
//! standing fine set at once ([`follow_marks`]), so a cell's band appears,
//! and its tufts go, on the frame it is written.

use super::{RoadNet, RoadsSettings, RoadsStanding};
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::edits::Edits;
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Slots};
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::YieldSet;
use pbd_core::roads::paving;
use pbd_core::roads::planner::{self, Clash, How, RoadPlannerConfig};
use pbd_core::roads::{self, BuiltCell, Ground, RoadsConfig, Surface};
use std::collections::{BTreeMap, HashSet};

/// Something done to a road, for the missions (`town-missions-tree` design
/// 4: the Roads branch's `stake`, `mark` and `pave`). Raised on the frame
/// its durable write is accepted. The town tree reads these.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoadWork {
    /// A survey stake driven.
    Stake,
    /// Cells newly marked, by a stake joining its plan's last.
    Mark(u32),
    /// A marked cell paved.
    Pave,
}

static NO_ROAD_CELLS: BTreeMap<u32, BuiltCell> = BTreeMap::new();

/// What the planner asks of a cell (design 42b): the world's road ground,
/// the player's own road cells walked as ground.
pub fn stake_ground<'a>(
    config: &'a TerrainConfig,
    edits: &'a Edits,
    yields: &'a YieldSet,
) -> impl Fn(u32) -> Option<Ground> + Sync + 'a {
    let base = super::ground_of(config, edits, yields, &NO_ROAD_CELLS);
    move |key| {
        let mut g = base(key)?;
        // A paved cell of the player's is a road, not their work.
        if g.player && edits.for_cell(key).is_empty() && !yields.has_cell(key) {
            g.player = false;
        }
        Some(g)
    }
}

/// The road planner's tunables (`assets/config/road_planner.ron`).
pub fn load_planner() -> RoadPlannerConfig {
    let path = crate::paths::assets_dir().join("config/road_planner.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let cfg: RoadPlannerConfig =
        ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    cfg.validate()
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    cfg
}

/// The road planner's tunables, for its systems.
#[derive(Resource, Clone)]
pub struct PlannerSettings(pub std::sync::Arc<RoadPlannerConfig>);

impl Default for PlannerSettings {
    fn default() -> Self {
        Self(std::sync::Arc::new(load_planner()))
    }
}

/// Why an edit was not saved: a clash on the road, the pack, or the save.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditRefused {
    Clash(Clash),
    NoStakes,
    PackFull,
    Unsaved,
}

impl EditRefused {
    /// The plain line the player reads, where there is one.
    pub fn line(self) -> Option<pbd_core::locale::Localized> {
        match self {
            EditRefused::Clash(c) => Some(c.line()),
            EditRefused::NoStakes => Some(pbd_core::locale::tr("roads.plan.no_stakes")),
            EditRefused::PackFull | EditRefused::Unsaved => None,
        }
    }
}

/// Write the plans an edit changed, with the pack (stakes given or taken,
/// and with the removal setting "return" the gravel of every paved cell
/// the edit drops), in one line: the other plans' records first, the last
/// with the pack (design 42h). Gives back the cells it took up.
pub fn commit(
    save: &mut WorldSave,
    slots: &mut Slots,
    edited: planner::Edited,
    config: &TerrainConfig,
    pcfg: &RoadPlannerConfig,
) -> Result<Vec<u32>, EditRefused> {
    let before = planner::plans(&save.records);
    let stake = Item::Good(Good::SURVEY_STAKES);
    let mut hands = slots.clone();
    if edited.stakes < 0 && !hands.take_item(stake, (-edited.stakes) as u16) {
        return Err(EditRefused::NoStakes);
    }
    if edited.stakes > 0 && hands.give(stake, edited.stakes as u16) != 0 {
        return Err(EditRefused::PackFull);
    }
    // Paved cells the edit drops, under "return": taken up at once.
    let mut taken = Vec::new();
    if pcfg.removal == planner::Removal::Return {
        let ground = stake_ground(config, &save.edits, &save.yields);
        let built = roads::built(&save.records);
        let was: HashSet<u32> =
            planner::for_removal(&planner::every_cell(&before, &ground, pcfg, 1.0), &built)
                .into_iter()
                .collect();
        let now = planner::for_removal(
            &planner::every_cell(&edited.plans, &ground, pcfg, 1.0),
            &built,
        );
        let mut gravel = 0.0f32;
        for key in now.into_iter().filter(|k| !was.contains(k)) {
            if built
                .get(&key)
                .is_some_and(|c| c.surface == Surface::Gravel)
            {
                gravel += pcfg.refund_share;
            }
            taken.push(key);
        }
        let gravel = gravel.floor() as u16;
        if gravel > 0 && hands.give(Item::Good(Good::GRAVEL), gravel) != 0 {
            return Err(EditRefused::PackFull);
        }
    }
    let ids: Vec<u64> = before
        .keys()
        .chain(edited.plans.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter(|id| before.get(id) != edited.plans.get(id))
        .collect();
    let mut records: Vec<_> = ids
        .iter()
        .map(|id| planner::plan_record(*id, edited.plans.get(id)))
        .chain(taken.iter().map(|&k| super::taken_up_record(k)))
        .collect();
    let Some(last) = records.pop() else {
        return Ok(taken);
    };
    if !records.is_empty()
        && save
            .store(&pbd_core::records::Author::Player(0), records)
            .is_none()
    {
        return Err(EditRefused::Unsaved);
    }
    let schema = if last.kind == paving::PLAN_RECORD {
        planner::ROADS_SCHEMA
    } else {
        roads::RECORD_SCHEMA
    };
    if !save.store_with_hands(last, schema, &hands) {
        return Err(EditRefused::Unsaved);
    }
    save.note_record_kinds(&[(paving::PLAN_RECORD, planner::ROADS_SCHEMA)]);
    *slots = hands;
    Ok(taken)
}

/// The cell whose top `point` is on.
pub fn key_of(point: Vec3) -> Option<u32> {
    roads::nearest_key(point.normalize_or(Vec3::Y))
}

/// The road from `start` to `end` added to the plans (design 42k): staked
/// every `max_segment` cells, a stake taken for each new one, written with
/// the pack. Gives back the road's stakes and its cells.
#[allow(clippy::too_many_arguments)]
pub fn end_road(
    save: &mut WorldSave,
    slots: &mut Slots,
    start: u32,
    end: u32,
    how: How,
    tree: &planner::TreeFn,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
    pcfg: &RoadPlannerConfig,
) -> Result<(usize, usize), EditRefused> {
    let plans = planner::plans(&save.records);
    let next = paving::next_plan_id(&save.records);
    let edited = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        planner::add_road(
            &plans,
            next,
            start,
            end,
            how,
            &ground,
            tree,
            pcfg,
            rcfg.ford_depth_m,
        )
        .map_err(EditRefused::Clash)?
    };
    let staked = (-edited.stakes).max(0) as usize;
    let cells = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        let road = edited
            .plans
            .values()
            .flat_map(|p| p.roads.last())
            .find(|r| r.stakes.first() == Some(&start) && r.stakes.last() == Some(&end));
        road.map_or(0, |r| {
            planner::lay(r, &ground, &|_| false, pcfg, rcfg.ford_depth_m).len()
        })
    };
    commit(save, slots, edited, config, pcfg)?;
    info!("road staked from cell {start} to {end}: {staked} stakes, {cells} cells");
    Ok((staked, cells))
}

/// Erase the road under cell `key` (design 42k): its stakes back where no
/// other road holds them. `false` where no road runs over it.
pub fn erase_road(
    save: &mut WorldSave,
    slots: &mut Slots,
    key: u32,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
    pcfg: &RoadPlannerConfig,
) -> Result<bool, EditRefused> {
    let plans = planner::plans(&save.records);
    let found = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        planner::road_at(&plans, key, &ground, pcfg, rcfg.ford_depth_m)
    };
    let Some((id, r)) = found else {
        return Ok(false);
    };
    commit(save, slots, planner::erase(&plans, id, r), config, pcfg)?;
    info!("road {r} of plan {id} erased");
    Ok(true)
}

/// Move the stake at `from` to `to` (design 42f), every road holding it
/// following.
#[allow(clippy::too_many_arguments)]
pub fn move_stake(
    save: &mut WorldSave,
    slots: &mut Slots,
    from: u32,
    to: u32,
    tree: &planner::TreeFn,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
    pcfg: &RoadPlannerConfig,
) -> Result<(), EditRefused> {
    let plans = planner::plans(&save.records);
    let edited = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        planner::move_stake(&plans, from, to, &ground, tree, pcfg, rcfg.ford_depth_m)
            .map_err(EditRefused::Clash)?
    };
    commit(save, slots, edited, config, pcfg)?;
    info!("survey stake moved from cell {from} to {to}");
    Ok(())
}

/// Take back the stake in cell `key` into the pack (design 42f): its road
/// shortened, or its neighbours joined. `Ok(false)` where no stake stands.
pub fn take_stake(
    save: &mut WorldSave,
    slots: &mut Slots,
    key: u32,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
    pcfg: &RoadPlannerConfig,
) -> Result<bool, EditRefused> {
    let plans = planner::plans(&save.records);
    if planner::stake_at(&plans, key).is_none() {
        return Ok(false);
    }
    let edited = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        planner::take_back(&plans, key, &ground, &|_| false, pcfg, rcfg.ford_depth_m)
            .map_err(EditRefused::Clash)?
    };
    commit(save, slots, edited, config, pcfg)?;
    info!("survey stake at cell {key} taken back");
    Ok(true)
}

/// Insert a stake at `key` on the leg it lies on (design 42f, 42m), a
/// stake from the pack. `Ok(false)` where no leg runs over the cell.
pub fn insert_stake(
    save: &mut WorldSave,
    slots: &mut Slots,
    key: u32,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
    pcfg: &RoadPlannerConfig,
) -> Result<bool, EditRefused> {
    let plans = planner::plans(&save.records);
    let edited = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        match planner::insert_stake(&plans, key, &ground, &|_| false, pcfg, rcfg.ford_depth_m) {
            None => return Ok(false),
            Some(e) => e.map_err(EditRefused::Clash)?,
        }
    };
    commit(save, slots, edited, config, pcfg)?;
    info!("survey stake inserted at cell {key}");
    Ok(true)
}

/// Whether a street lantern stands on cell `key` by the player's hand: a
/// road's lamp post built (design 42m).
pub fn lamp_standing(save: &WorldSave, key: u32) -> bool {
    save.edits
        .for_cell(key)
        .iter()
        .any(|(_, m)| *m == pbd_core::terrain::Material::LanternPost)
}

/// The road's lamp posts beside cell `key` not yet standing: `(post cell,
/// the layer it stands in)`, the first air layer over its ground.
pub fn lamps_owed(
    save: &WorldSave,
    key: u32,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
) -> Vec<(u32, usize)> {
    let pcfg = load_planner_cached();
    let plans = planner::plans(&save.records);
    let ground = stake_ground(config, &save.edits, &save.yields);
    let mut out = Vec::new();
    for road in plans.values().flat_map(|p| &p.roads).filter(|r| r.lamps) {
        let laid = planner::lay(road, &ground, &|_| false, &pcfg, rcfg.ford_depth_m);
        if !laid.iter().any(|c| c.key == key) {
            continue;
        }
        for (beside, post) in planner::lamp_posts(road, &laid, &ground, &pcfg) {
            if beside != key || lamp_standing(save, post) || out.iter().any(|(p, _)| *p == post) {
                continue;
            }
            let Some(top) = roads::centre(post).map(|c| pbd_core::column::surface_m(config, c))
            else {
                continue;
            };
            if let Some(layer) = pbd_core::column::layer_at(top.floor() + 0.5) {
                out.push((post, layer));
            }
        }
    }
    out
}

/// Build the lamp posts owed beside paved cell `key` (design 42m): each the
/// town's street lantern, placed as the player's block, a log and a street
/// lantern taken from the pack in the same line. The edits it made, for the
/// terrain; refused where the pack is short.
pub fn build_lamps(
    save: &mut WorldSave,
    slots: &mut Slots,
    key: u32,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
) -> Result<Vec<pbd_core::edits::Edit>, PaveRefused> {
    let lantern = Item::Block(pbd_core::terrain::Material::LanternPost);
    let mut made = Vec::new();
    for (post, layer) in lamps_owed(save, key, config, rcfg) {
        let mut hands = slots.clone();
        if !hands.take_item(Item::Good(Good::LOG), 1) || !hands.take_item(lantern, 1) {
            return if made.is_empty() {
                Err(PaveRefused::NoLamp)
            } else {
                Ok(made)
            };
        }
        let edit = pbd_core::edits::Edit {
            cell: post,
            layer: layer as u16,
            material: pbd_core::terrain::Material::LanternPost,
        };
        if !save.accept(edit, None, &hands) {
            return Err(PaveRefused::Unsaved);
        }
        *slots = hands;
        info!("road lamp built on cell {post} beside {key}");
        made.push(edit);
    }
    Ok(made)
}

/// Why a cell was not paved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaveRefused {
    NotMarked,
    NoGravel,
    /// A bridge cell with no planks or log in the pack.
    NoPlanks,
    /// A lamp post with no log or street lantern in the pack.
    NoLamp,
    Unsaved,
}

impl PaveRefused {
    /// The plain line the player reads; none where the aim is simply
    /// elsewhere.
    pub fn line(self) -> Option<pbd_core::locale::Localized> {
        match self {
            PaveRefused::NotMarked | PaveRefused::Unsaved => None,
            PaveRefused::NoGravel => Some(pbd_core::locale::tr("roads.pave.needs_gravel")),
            PaveRefused::NoPlanks => Some(pbd_core::locale::tr("roads.pave.needs_planks")),
            PaveRefused::NoLamp => Some(pbd_core::locale::tr("roads.lamp.needs")),
        }
    }
}

/// Pave the marked cell `key` (design 40, 42c): its slope the world road's
/// over its road's order, one gravel for a gravel road or a ford and none
/// for dirt, written as a `road-cell` of the player's (schema 2) with the
/// pack in one line.
pub fn pave(
    save: &mut WorldSave,
    slots: &mut Slots,
    key: u32,
    config: &TerrainConfig,
    rcfg: &RoadsConfig,
    pcfg: &RoadPlannerConfig,
) -> Result<BuiltCell, PaveRefused> {
    let plans = planner::plans(&save.records);
    if roads::built(&save.records).contains_key(&key) {
        return Err(PaveRefused::NotMarked);
    }
    let cell = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        let cells = planner::every_cell(&plans, &ground, pcfg, rcfg.ford_depth_m);
        let (id, r, _) = cells.get(&key).ok_or(PaveRefused::NotMarked)?;
        planner::paved_cell(
            *id,
            &plans[id].roads[*r],
            key,
            &ground,
            pcfg,
            rcfg.ford_depth_m,
        )
        .ok_or(PaveRefused::NotMarked)?
    };
    let mut hands = slots.clone();
    if cell.deck_m.is_some() {
        // A bridge cell: its deck's planks and a log for its posts (42m).
        let planks = (pcfg.bridge_planks as f32 * pcfg.cost_scale).ceil() as u16;
        if !hands.take_item(Item::Good(Good::PLANKS), planks)
            || !hands.take_item(Item::Good(Good::LOG), 1)
        {
            return Err(PaveRefused::NoPlanks);
        }
    } else {
        let cost = if cell.surface == Surface::Gravel {
            pcfg.cost_scale.ceil() as u16
        } else {
            0
        };
        if cost > 0 && !hands.take_item(Item::Good(Good::GRAVEL), cost) {
            return Err(PaveRefused::NoGravel);
        }
    }
    if !save.store_with_hands(
        paving::paved_record(key, &cell),
        paving::PAVED_SCHEMA,
        &hands,
    ) {
        return Err(PaveRefused::Unsaved);
    }
    *slots = hands;
    info!(
        "road cell {key} paved in {:?} on plan {:?}",
        cell.surface, cell.plan
    );
    Ok(cell)
}

/// The marked cells with their sides, and the paved cells marked for removal.
pub type Marks = (BTreeMap<u32, (Option<u32>, Option<u32>)>, Vec<u32>);

/// Every marked cell (a plan's cell not paved) with its sides, and the
/// paved cells marked for removal.
pub fn marks(save: &WorldSave) -> Marks {
    let config = crate::planet::terrain_config();
    let pcfg = load_planner_cached();
    let plans = planner::plans(&save.records);
    let built = roads::built(&save.records);
    let ground = stake_ground(config, &save.edits, &save.yields);
    let cells = planner::every_cell(&plans, &ground, &pcfg, 1.0);
    (
        planner::marked(&cells, &built),
        planner::for_removal(&cells, &built),
    )
}

/// The planner's tunables read once, for code with no system parameters.
pub fn load_planner_cached() -> std::sync::Arc<RoadPlannerConfig> {
    static CFG: std::sync::OnceLock<std::sync::Arc<RoadPlannerConfig>> = std::sync::OnceLock::new();
    CFG.get_or_init(|| std::sync::Arc::new(load_planner()))
        .clone()
}

/// The marked cells within `reach_m` of `point`, nearest first.
pub fn marked_near(save: &WorldSave, point: Vec3, reach_m: f32) -> Vec<u32> {
    let (marked, _) = marks(save);
    let d = point.normalize_or(Vec3::Y);
    let radius = point.length();
    let mut near: Vec<(f32, u32)> = marked
        .keys()
        .filter_map(|&k| {
            let m = roads::centre(k)?.distance(d) * radius;
            (m <= reach_m).then_some((m, k))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    near.into_iter().map(|(_, k)| k).collect()
}

/// Every stake standing, by cell key.
pub fn stakes(save: &WorldSave) -> Vec<u32> {
    planner::stakes(&planner::plans(&save.records))
        .into_iter()
        // A construction site's stakes at its corners (`building-planner` 5).
        .chain(crate::construction::site_marks(&save.records).1)
        .collect()
}

/// Install the marks and the player's road cells for the terrain
/// (design 39a and 43), and give back every key they name.
pub fn install(save: &WorldSave, radius_m: f32) -> HashSet<u32> {
    install_held(save, radius_m).0
}

/// [`install`], with the guard that keeps other installs waiting (see
/// `pbd_core::roads::paving::install_held`).
pub fn install_held(
    save: &WorldSave,
    radius_m: f32,
) -> (HashSet<u32>, std::sync::RwLockWriteGuard<'static, ()>) {
    let built = roads::built(&save.records);
    let (mut marks, _) = marks(save);
    // A construction site's lime line (`building-planner` design 5).
    marks.extend(crate::construction::site_marks(&save.records).0);
    let keys: HashSet<u32> = marks.keys().chain(built.keys()).copied().collect();
    let held = paving::install_held(Some(paving::Installed {
        marks: marks.into_iter().collect(),
        bare: roads::Cleared::new(keys.iter().copied(), radius_m),
    }));
    (keys, held)
}

/// The keys installed last, and whether the standing set has them yet.
#[derive(Resource, Default)]
pub struct MarksShown {
    /// Every key installed or installed before, so a mark taken back is
    /// cleared too.
    pub keys: HashSet<u32>,
    /// The save's records as last installed: (serial, count of plan and
    /// road-cell lines).
    pub installed_for: (u64, usize),
    /// The fine set's version last brought round.
    pub fine_version: u64,
}

/// Re-install when the plans or the player's road cells change, and bring
/// the standing fine set round to them at once (design 44).
pub fn follow_marks(
    save: Option<Res<WorldSave>>,
    fine: Option<ResMut<crate::planet::PlanetFine>>,
    contact: Option<ResMut<crate::planet::PlanetContact>>,
    mut shown: ResMut<MarksShown>,
) {
    let Some(save) = save else {
        return;
    };
    let stamp = (
        save.serial(),
        save.records.of_kind(paving::PLAN_RECORD).count()
            + save.records.of_kind(roads::ROAD_CELL_RECORD).count()
            + save
                .records
                .of_kind(paving::PLAN_RECORD)
                .map(|r| r.body.len())
                .sum::<usize>()
            // A site set out, changed or finished moves its lime line.
            + save
                .records
                .of_kind(pbd_core::building::site::SITE_RECORD)
                .map(|r| r.body.len())
                .sum::<usize>()
            + save
                .records
                .of_kind(pbd_core::building::PIECE_RECORD)
                .count(),
    );
    let fresh = stamp != shown.installed_for;
    if fresh {
        let keys = install(&save, crate::planet::terrain_config().radius_m);
        shown.keys.extend(keys);
        shown.installed_for = stamp;
    }
    let (Some(mut fine), Some(mut contact)) = (fine, contact) else {
        return;
    };
    if !fresh && fine.version == shown.fine_version {
        return;
    }
    if shown.keys.is_empty() {
        shown.fine_version = fine.version;
        return;
    }
    let mut set = (*fine.set).clone();
    let changed = set.apply_road_marks(&shown.keys);
    if changed > 0 {
        let set = std::sync::Arc::new(set);
        contact.set_fine(&set);
        fine.set = set;
        fine.version += 1;
        info!("ROAD_MARKS {changed} cells brought round to the marks");
    }
    shown.fine_version = fine.version;
}

/// The net again from the save, and the player's chunks stood again at
/// once (after a cell is paved or taken up).
pub fn restand(save: &WorldSave, net: &mut RoadNet, standing: &mut RoadsStanding) {
    let stored = roads::stored(&save.records);
    let built = roads::built(&save.records);
    let fresh = RoadNet::of(&stored, &built, save.serial());
    let first_mine = fresh
        .chunks
        .iter()
        .position(|c| {
            c.cells
                .iter()
                .any(|&i| built.contains_key(&fresh.cells[i].cell.key))
        })
        .unwrap_or(fresh.chunks.len());
    standing.redraw_from = Some(first_mine.min(net.chunks.len()));
    *net = fresh;
}

/// The roads' tunables, for a system.
pub fn settings(world_cfg: Option<&RoadsSettings>) -> std::sync::Arc<RoadsConfig> {
    world_cfg.map_or_else(
        || std::sync::Arc::new(super::load_config()),
        |s| s.0.clone(),
    )
}

// ---- The stakes standing -------------------------------------------------------

/// The pegs standing, by cell key, under one root that follows the planet's
/// render frame.
#[derive(Resource, Default)]
pub struct StakesStanding {
    root: Option<Entity>,
    pegs: BTreeMap<u32, Entity>,
}

/// A peg's meshes and materials, made once; one of each stands hidden from
/// the first frame so the loading screen compiles their pipelines.
#[derive(Resource)]
pub struct PavingLooks {
    /// By [`Stuff`](super::models::Stuff), in `Stuff::ALL` order.
    pub materials: Vec<Handle<StandardMaterial>>,
    pub stakes: Vec<Vec<(super::models::Stuff, Handle<Mesh>)>>,
    pub grinder: Vec<(super::models::Stuff, Handle<Mesh>)>,
    pub mixer: Vec<(super::models::Stuff, Handle<Mesh>)>,
}

impl PavingLooks {
    pub fn material(&self, stuff: super::models::Stuff) -> Handle<StandardMaterial> {
        let k = super::models::Stuff::ALL
            .iter()
            .position(|s| *s == stuff)
            .expect("every stuff");
        self.materials[k].clone()
    }
}

/// How many stake shapes there are to vary the pegs by.
const STAKE_LOOKS: u32 = 4;

pub fn make_looks(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The towns' own pixel textures, nearest-sampled and repeated, laid at
    // their metres a repeat: the models' coordinates are metres.
    let materials: Vec<Handle<StandardMaterial>> = super::models::Stuff::ALL
        .iter()
        .map(|stuff| {
            let (name, repeat_m) = stuff.texture();
            materials.add(StandardMaterial {
                base_color_texture: Some(crate::towns::texture(&assets, name)),
                uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(1.0 / repeat_m)),
                perceptual_roughness: 0.93,
                ..default()
            })
        })
        .collect();
    let mut add = |parts: Vec<(super::models::Stuff, super::models::Shape)>| {
        parts
            .into_iter()
            .map(|(s, shape)| (s, meshes.add(shape.mesh())))
            .collect::<Vec<_>>()
    };
    let looks = PavingLooks {
        materials,
        stakes: (0..STAKE_LOOKS)
            .map(|k| add(super::models::stake(k * 7 + 1)))
            .collect(),
        grinder: add(super::models::grinder()),
        mixer: add(super::models::mixer()),
    };
    commands
        .spawn((
            crate::field_light::LitByField,
            crate::field_light::LitLikeTerrain,
            Transform::default(),
            // Shown behind the loading screen, as the townsfolk's stand-in
            // is, so their pipelines are built before play; gone at READY.
            Visibility::Visible,
            crate::loading::PrewarmProxy,
            Name::new("Paving prewarm"),
        ))
        .with_children(|root| {
            for (stuff, mesh) in looks.stakes[0]
                .iter()
                .chain(&looks.grinder)
                .chain(&looks.mixer)
            {
                root.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(looks.material(*stuff))));
            }
        });
    commands.insert_resource(looks);
}

/// Metres from the viewer within which a stake's peg stands.
pub const STAKE_STAND_M: f32 = 160.0;

/// Stand a peg at every stake near the viewer and take away those gone or
/// past reach; each peg is lit as the terrain under it is.
pub fn stand_stakes(world: &mut World) {
    let Some(save) = world.get_resource::<WorldSave>() else {
        return;
    };
    let keys = stakes(save);
    let config = *crate::planet::terrain_config();
    let Some(eye) = super::viewer(world) else {
        return;
    };
    if !world.contains_resource::<PavingLooks>() {
        return;
    }
    let mut standing =
        std::mem::take(&mut *world.get_resource_or_insert_with(StakesStanding::default));
    let root = match standing.root.filter(|&e| world.get_entity(e).is_ok()) {
        Some(e) => e,
        None => {
            let centre = world
                .get_resource::<crate::planet::PlanetRenderFrame>()
                .map_or(Vec3::ZERO, |f| f.center.as_vec3());
            world
                .spawn((
                    Name::new("Survey stakes"),
                    super::RoadRoot,
                    crate::field_light::LitByField,
                    crate::field_light::LitLikeTerrain,
                    Transform::from_translation(centre),
                    Visibility::default(),
                ))
                .id()
        }
    };
    standing.root = Some(root);
    let near: BTreeMap<u32, Vec3> = keys
        .into_iter()
        .filter_map(|k| {
            let c = roads::centre(k)?;
            let top = pbd_core::column::surface_m(&config, c).floor();
            let at = c * (config.radius_m + top);
            (at.distance(eye) <= STAKE_STAND_M).then_some((k, at))
        })
        .collect();
    standing.pegs.retain(|k, e| {
        let keep = near.contains_key(k);
        if !keep && let Ok(e) = world.get_entity_mut(*e) {
            e.despawn();
        }
        keep
    });
    for (key, at) in near {
        if standing.pegs.contains_key(&key) {
            continue;
        }
        let up = at.normalize_or(Vec3::Y);
        let spin = Quat::from_axis_angle(Vec3::Y, (key % 360) as f32 * 0.01745);
        let transform = Transform::from_translation(at)
            .with_rotation(Quat::from_rotation_arc(Vec3::Y, up) * spin);
        let parts = {
            let looks = world.resource::<PavingLooks>();
            looks.stakes[(key % STAKE_LOOKS) as usize]
                .iter()
                .map(|(s, m)| (m.clone(), looks.material(*s)))
                .collect::<Vec<_>>()
        };
        let peg = world
            .spawn((transform, Visibility::default(), Name::new("Survey stake")))
            .with_children(|peg| {
                for (mesh, material) in parts {
                    peg.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
                }
            })
            .id();
        world.entity_mut(root).add_child(peg);
        standing.pegs.insert(key, peg);
    }
    world.insert_resource(standing);
}
