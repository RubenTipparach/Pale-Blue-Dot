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
use pbd_core::roads::paving::{self, Plan, Refused};
use pbd_core::roads::{self, BuiltCell, Ground, RoadsConfig};
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

/// The plan the stakes in hand are laying: the next stake joins its last.
/// Cleared when the stakes leave the selected slot (design 39).
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenPlan(pub Option<u64>);

static NO_ROAD_CELLS: BTreeMap<u32, BuiltCell> = BTreeMap::new();

/// What a stake's search asks of a cell (design 39): the world's road
/// ground, but a town's footprint is never crossed and the player's own
/// road cells are walked as ground.
pub fn stake_ground<'a>(
    config: &'a TerrainConfig,
    edits: &'a Edits,
    yields: &'a YieldSet,
) -> impl Fn(u32) -> Option<Ground> + Sync + 'a {
    let base = super::ground_of(config, edits, yields, &NO_ROAD_CELLS);
    move |key| {
        let mut g = base(key)?;
        if g.town {
            g.player = true;
        }
        Some(g)
    }
}

/// What a stake did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Staked {
    /// A stake driven: the plan it is on and the cells it newly marked.
    Driven { plan: u64, marked: usize },
    /// Aimed at a plan's last stake: the stakes in hand go on from it, and
    /// nothing is taken.
    Resumed(u64),
}

/// Drive a survey stake into the cell whose top `point` (body-local,
/// planet-centred metres) is on (design 39). One stake is taken from
/// `slots`, and the plan and the pack are written in one line.
pub fn drive_stake(
    save: &mut WorldSave,
    slots: &mut Slots,
    open: Option<u64>,
    point: Vec3,
    config: &TerrainConfig,
    cfg: &RoadsConfig,
) -> Result<Staked, Refused> {
    let key = roads::nearest_key(point.normalize_or(Vec3::Y)).ok_or(Refused::NotGround)?;
    let plans = paving::plans(&save.records);
    if let Some((&id, _)) = plans.iter().find(|(_, p)| p.stakes.last() == Some(&key)) {
        return Ok(Staked::Resumed(id));
    }
    let stake = Item::Good(Good::SURVEY_STAKES);
    let mut hands = slots.clone();
    if !hands.take_item(stake, 1) {
        return Err(Refused::NotGround);
    }
    let next = paving::next_plan_id(&save.records);
    let driven = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        paving::drive(&plans, next, open, key, &ground, cfg, config.radius_m)?
    };
    let record = paving::plan_record(driven.id, Some(&driven.plan));
    if !save.store_with_hands(record, paving::PLAN_SCHEMA, &hands) {
        return Err(Refused::NotGround);
    }
    *slots = hands;
    info!(
        "survey stake at cell {key}: plan {} now {} cells",
        driven.id,
        driven.plan.cells.len()
    );
    Ok(Staked::Driven {
        plan: driven.id,
        marked: driven.marked.len(),
    })
}

/// Take back the stake in cell `key` into the pack (design 39): the plan
/// either side rejoined, shortened or split. `false` where no stake stands
/// there or the pack has no room.
pub fn take_stake(
    save: &mut WorldSave,
    slots: &mut Slots,
    key: u32,
    config: &TerrainConfig,
    cfg: &RoadsConfig,
) -> bool {
    let plans = paving::plans(&save.records);
    let Some((&id, plan)) = plans.iter().find(|(_, p)| p.stakes.contains(&key)) else {
        return false;
    };
    let mut hands = slots.clone();
    if hands.give(Item::Good(Good::SURVEY_STAKES), 1) != 0 {
        return false;
    }
    let (first, second) = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        paving::take_back(plan, key, &ground, cfg, config.radius_m)
    };
    if let Some(second) = second {
        let next = paving::next_plan_id(&save.records);
        if !save.store_with_hands(
            paving::plan_record(next, Some(&second)),
            paving::PLAN_SCHEMA,
            slots,
        ) {
            return false;
        }
    }
    if !save.store_with_hands(
        paving::plan_record(id, first.as_ref()),
        paving::PLAN_SCHEMA,
        &hands,
    ) {
        return false;
    }
    *slots = hands;
    info!("survey stake at cell {key} taken back from plan {id}");
    true
}

/// Why a cell was not paved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaveRefused {
    NotMarked,
    NoGravel,
    Unsaved,
}

impl PaveRefused {
    /// The plain line the player reads; none where the aim is simply
    /// elsewhere.
    pub fn line(self) -> Option<pbd_core::locale::Localized> {
        match self {
            PaveRefused::NotMarked | PaveRefused::Unsaved => None,
            PaveRefused::NoGravel => Some(pbd_core::locale::tr("roads.pave.needs_gravel")),
        }
    }
}

/// Pave the marked cell `key` with one gravel (design 40): its slope the
/// world road's over its plan's order, written as a `road-cell` of the
/// player's (schema 2) with the pack in one line.
pub fn pave(
    save: &mut WorldSave,
    slots: &mut Slots,
    key: u32,
    config: &TerrainConfig,
) -> Result<BuiltCell, PaveRefused> {
    let plans = paving::plans(&save.records);
    let built = roads::built(&save.records);
    if built.contains_key(&key) {
        return Err(PaveRefused::NotMarked);
    }
    let (id, plan) = paving::plan_of(&plans, key).ok_or(PaveRefused::NotMarked)?;
    let mut hands = slots.clone();
    if !hands.take_item(Item::Good(Good::GRAVEL), 1) {
        return Err(PaveRefused::NoGravel);
    }
    let cell = {
        let ground = stake_ground(config, &save.edits, &save.yields);
        paving::paved_cell(id, plan, key, &ground).map_err(|_| PaveRefused::NotMarked)?
    };
    if !save.store_with_hands(
        paving::paved_record(key, &cell),
        paving::PAVED_SCHEMA,
        &hands,
    ) {
        return Err(PaveRefused::Unsaved);
    }
    *slots = hands;
    info!("road cell {key} paved in gravel on plan {id}");
    Ok(cell)
}

/// The marked cells within `reach_m` of `point`, nearest first.
pub fn marked_near(save: &WorldSave, point: Vec3, reach_m: f32) -> Vec<u32> {
    let plans = paving::plans(&save.records);
    let built = roads::built(&save.records);
    let marked = paving::marked(&plans, &built);
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
    paving::plans(&save.records)
        .values()
        .flat_map(|p| p.stakes.iter().copied())
        // A construction site's stakes at its corners (`building-planner` 5).
        .chain(crate::construction::site_marks(&save.records).1)
        .collect()
}

/// Install the marks and the player's road cells for the terrain
/// (design 39a and 43), and give back every key they name.
pub fn install(records: &pbd_core::records::Records, radius_m: f32) -> HashSet<u32> {
    let plans: BTreeMap<u64, Plan> = paving::plans(records);
    let built = roads::built(records);
    let mut marks = paving::marked(&plans, &built);
    // A construction site's lime line (`building-planner` design 5).
    marks.extend(crate::construction::site_marks(records).0);
    let keys: HashSet<u32> = marks.keys().chain(built.keys()).copied().collect();
    paving::install(Some(paving::Installed {
        marks: marks.into_iter().collect(),
        bare: roads::Cleared::new(keys.iter().copied(), radius_m),
    }));
    keys
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
        let keys = install(&save.records, crate::planet::terrain_config().radius_m);
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
