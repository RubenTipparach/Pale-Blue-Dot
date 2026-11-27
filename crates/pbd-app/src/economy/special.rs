//! The special town jobs in the game (`townsfolk` design 28 and 29): the
//! lost villager, who stands out of town until found and then follows the
//! player home, and the repair, a wall of a town building the world breaks
//! and the player mends with planks.
//!
//! The rules are `pbd_core::economy::follow` and the settlement record's
//! `broken` walls; this module places, stores and counts them. Each job's
//! state is a stored record by the job's id (`lost_villager`,
//! `town_repair`), so a reload finds it where it was.

use super::{Economy, EconomyConfig};
use crate::saves::WorldSave;
use crate::towns::Towns;
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_core::economy::follow::{self, LOST_KIND, LOST_SCHEMA, Lost};
use pbd_core::economy::jobs::{Job, JobKind, mix, scoped};
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Slots};
use pbd_core::missions::Kind;
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::{Author, Proposal, Record};
use pbd_core::roads::Water;
use pbd_core::settlement::record::{self as town_record, BUILDING_RECORD, RECORD_SCHEMA};
use pbd_core::townsfolk::PersonId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A repair job's broken wall, stored by the job's id.
pub const REPAIR_KIND: &str = "town_repair";
pub const REPAIR_SCHEMA: u32 = 1;
/// Planks a mended wall takes.
pub const MEND_PLANKS: u16 = 2;
/// How near a broken wall's middle the player must be to mend it, metres.
pub const MEND_REACH_M: f32 = 4.0;
/// A lost villager's place is written at most this often while they walk,
/// seconds, and when they come to rest.
pub const LOST_WRITE_S: f32 = 1.0;

/// A repair job's wall: the town, the building (its place in the town's
/// buildings) and the wall (`[c, r, d, storey]`), and whether it is mended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repair {
    pub site: u32,
    pub building: u32,
    pub wall: [i32; 4],
    pub mended: bool,
}

/// A job's lost villager as stored, if there is one.
pub fn lost_of(save: &WorldSave, job: u64) -> Option<Lost> {
    save.records
        .get(LOST_KIND, job)
        .and_then(|r| r.read::<Option<Lost>>().flatten().or_else(|| r.read()))
}

/// A job's broken wall as stored, if there is one.
pub fn repair_of(save: &WorldSave, job: u64) -> Option<Repair> {
    save.records.get(REPAIR_KIND, job).and_then(|r| r.read())
}

/// The taken jobs of a kind not yet done.
fn open_jobs(eco: &Economy, kind: JobKind) -> Vec<Job> {
    eco.jobs
        .taken
        .iter()
        .filter(|j| j.kind == kind && !eco.jobs.is_done(j.id))
        .cloned()
        .collect()
}

// ---- The lost villager -------------------------------------------------------

/// Where each taken lost villager job's villager stands, as directions from
/// the planet's centre, by job: what the map and the compass mark
/// (`town-missions-tree` design 10).
pub fn lost_directions(eco: &Economy, save: &WorldSave) -> Vec<Vec3> {
    open_jobs(eco, JobKind::Lost)
        .iter()
        .filter_map(|j| lost_of(save, j.id))
        .filter_map(|l| DVec3::from_array(l.at).try_normalize())
        .map(|d| d.as_vec3())
        .collect()
}

/// Whether a cell's ground will do to stand a lost villager on (design 28):
/// dry, no town's, holding no player's edit, within a layer of each of its
/// neighbours. Its ground's top when it will.
pub fn open_ground(save: &WorldSave, config: &TerrainConfig, direction: Vec3) -> Option<i32> {
    let built = pbd_core::roads::built(&save.records);
    let ground = crate::roads::ground_of(config, &save.edits, &save.yields, &built);
    let key = pbd_core::roads::nearest_key(direction)?;
    let g = ground(key)?;
    if g.water != Water::Dry || g.town || g.player {
        return None;
    }
    pbd_core::roads::neighbours(key)
        .into_iter()
        .all(|n| ground(n).is_some_and(|o| (o.top_m - g.top_m).abs() <= 1))
        .then_some(g.top_m)
}

/// Whether a lost villager may stand at a planet point: not on deep water
/// (more than a layer over its bed) or the sea.
pub fn dry(config: &TerrainConfig, point: DVec3) -> bool {
    let d = point.as_vec3().normalize_or(Vec3::Y);
    let Some(key) = pbd_core::roads::nearest_key(d) else {
        return false;
    };
    let none = (
        pbd_core::edits::Edits::new(),
        pbd_core::records::YieldSet::new(),
        BTreeMap::new(),
    );
    let ground = crate::roads::ground_of(config, &none.0, &none.1, &none.2);
    ground(key).is_some_and(|g| match g.water {
        Water::Dry => true,
        Water::River { depth_m, .. } => depth_m <= 1,
        Water::Sea => false,
    })
}

/// A point on the ground at a direction: its generated top (towns' ground
/// included), in the body's frame, f64.
pub fn on_ground(config: &TerrainConfig, direction: DVec3) -> DVec3 {
    let d = direction.normalize();
    let top = pbd_core::column::surface_m(config, d.as_vec3());
    d * (f64::from(config.radius_m) + f64::from(top))
}

/// A lost villager placed for a job: who (by the town's people's roles and
/// its keepers) and where (rolled out from the town's `anchor`).
pub fn place_lost(
    job: &Job,
    roles: &[&str],
    cfg: &EconomyConfig,
    anchor: Vec3,
    save: &WorldSave,
    config: &TerrainConfig,
) -> Option<Lost> {
    let keepers = cfg
        .jobs
        .keepers(roles)
        .map(|k| vec![k.trader, k.giver])
        .unwrap_or_default();
    let person = follow::who(job.id, roles.len(), &keepers)?;
    let d = follow::spot(job.id, anchor, config.radius_m, |d| {
        open_ground(save, config, d).is_some()
    })?;
    let at = on_ground(config, d.as_dvec3());
    Some(Lost {
        site: job.site,
        at: at.to_array(),
        following: false,
        person,
    })
}

/// A lost villager's record.
pub fn lost_record(job: u64, lost: &Lost) -> Record {
    Record::of(LOST_KIND, job, LOST_SCHEMA, lost)
}

/// A lost villager brought home: their record with no one in it.
pub fn home_record(job: u64) -> Record {
    Record::of(LOST_KIND, job, LOST_SCHEMA, &Option::<Lost>::None)
}

/// What a lost villager's step did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Walked {
    /// Stood where they were.
    Still,
    /// Walked this far, metres.
    Moved(f64),
    /// Reached their town's wall: the job's villager is home.
    Home,
}

/// One step of a found villager (design 28): behind the player at their
/// pace, waiting where `dry` says no, home where `home` says so; their feet
/// stood on the ground by `ground`.
pub fn walk(
    lost: &mut Lost,
    player: DVec3,
    player_mps: f64,
    dt: f64,
    dry: impl Fn(DVec3) -> bool,
    ground: impl Fn(DVec3) -> DVec3,
    home: impl Fn(DVec3) -> bool,
) -> Walked {
    if !lost.following {
        return Walked::Still;
    }
    let from = lost.at();
    let next = follow::step(from, player, player_mps, dt, &dry);
    if next == from {
        return if home(from) {
            Walked::Home
        } else {
            Walked::Still
        };
    }
    let next = ground(next);
    lost.at = next.to_array();
    if home(next) {
        Walked::Home
    } else {
        Walked::Moved((next - from).length())
    }
}

/// The talker is a taken job's lost villager, not yet following: they are
/// found, and follow from now on; their record is written. Whether they
/// were.
pub fn find_lost(save: &mut WorldSave, eco: &Economy, site: u32, index: u32) -> bool {
    for job in open_jobs(eco, JobKind::Lost) {
        let Some(mut lost) = lost_of(save, job.id) else {
            continue;
        };
        if lost.site != site || lost.person != index || lost.following {
            continue;
        }
        lost.following = true;
        if save
            .store(&Author::Player(0), vec![lost_record(job.id, &lost)])
            .is_some()
        {
            info!("lost villager of job {} found", job.id);
            return true;
        }
    }
    false
}

/// Where the walk stands between frames.
#[derive(Default)]
pub struct LostWalk {
    player: Option<DVec3>,
    since_write: BTreeMap<u64, (f32, bool)>,
}

/// Place each taken lost-villager job's villager, stand their figure where
/// they are, walk the found ones behind the player, and count them home.
#[allow(clippy::too_many_arguments)]
pub fn lost_villagers(
    mut eco: ResMut<Economy>,
    cfg: Res<EconomyConfig>,
    save: Option<ResMut<WorldSave>>,
    towns: Res<Towns>,
    folk: Option<ResMut<crate::townsfolk::Townsfolk>>,
    walkers: Query<&avian3d::prelude::Position, With<crate::walking::Walker>>,
    sim: Option<Res<crate::sim_origin::SimFrame>>,
    time: Res<Time>,
    mut did: MessageWriter<crate::missions::Did>,
    mut walk_state: Local<LostWalk>,
) {
    let (Some(mut save), Some(mut folk)) = (save, folk) else {
        return;
    };
    // An abandoned job's villager goes back to their post.
    if !eco.let_go.is_empty() {
        for id in std::mem::take(&mut eco.let_go) {
            folk.release(id);
        }
    }
    let jobs = open_jobs(&eco, JobKind::Lost);
    if jobs.is_empty() {
        walk_state.player = None;
        return;
    }
    let config = *crate::planet::terrain_config();
    let dt = time.delta_secs().clamp(0.0, 0.1);
    let player = sim
        .as_ref()
        .and_then(|s| walkers.iter().next().map(|p| s.0.to_world(p.0).pos));
    let player_mps = match (player, walk_state.player) {
        (Some(now), Some(was)) if dt > 0.0 => (now - was).length() / f64::from(dt),
        _ => 0.0,
    };
    walk_state.player = player;
    for job in jobs {
        let roles: Vec<String> = folk.people(job.site).map(|p| p.role.clone()).collect();
        if roles.is_empty() {
            continue;
        }
        let Some(mut lost) = lost_of(&save, job.id) else {
            let Some(held) = towns.held.iter().find(|h| h.site.id == job.site) else {
                continue;
            };
            let roles: Vec<&str> = roles.iter().map(String::as_str).collect();
            if let Some(lost) = place_lost(&job, &roles, &cfg, held.site.direction, &save, &config)
                && save
                    .store(&Author::Player(0), vec![lost_record(job.id, &lost)])
                    .is_some()
            {
                save.note_record_kinds(&[(LOST_KIND, LOST_SCHEMA)]);
                info!(
                    "job {}: {} of {} lost at {:?}",
                    job.id, lost.person, held.site.name, lost.at
                );
            }
            continue;
        };
        let id = PersonId {
            site: job.site,
            index: lost.person,
        };
        let mut moved = false;
        if let Some(player) = player {
            let ground = |p: DVec3| on_ground(&config, p);
            let home = |p: DVec3| {
                pbd_core::settlement::ground::cleared(&config, p.as_vec3().normalize_or(Vec3::Y))
            };
            match walk(
                &mut lost,
                player,
                player_mps,
                f64::from(dt),
                |p| dry(&config, p),
                ground,
                home,
            ) {
                Walked::Home => {
                    did.write(crate::missions::Did::new(
                        Kind::Escort,
                        scoped(job.site, "villager"),
                        1,
                    ));
                    if save
                        .store(&Author::Player(0), vec![home_record(job.id)])
                        .is_some()
                    {
                        info!("job {}: the lost villager is home", job.id);
                    }
                    folk.release(id);
                    walk_state.since_write.remove(&job.id);
                    continue;
                }
                Walked::Moved(_) => moved = true,
                Walked::Still => {}
            }
        }
        let forward = player
            .map(|p| p - lost.at())
            .and_then(|v| v.try_normalize())
            .unwrap_or(DVec3::X);
        folk.drive(id, lost.at(), forward, moved);
        // Written when they come to rest after walking, and at most once a
        // second while they walk, so a reload finds them where they were.
        let (since, walked) = walk_state.since_write.entry(job.id).or_insert((0.0, false));
        *since += dt;
        *walked |= moved;
        if *walked && (!moved || *since >= LOST_WRITE_S) {
            *since = 0.0;
            *walked = false;
            save.store(&Author::Player(0), vec![lost_record(job.id, &lost)]);
        }
    }
}

// ---- Repair ------------------------------------------------------------------

/// The author a broken wall is the world's by (design 29).
pub fn repair_author(job: u64) -> Author {
    Author::World {
        process: "repair".into(),
        kind: pbd_core::economy::jobs::JOBS_KIND.into(),
        id: job,
    }
}

/// Have the world break a wall of one of `town`'s buildings for a repair
/// job (design 29), through the yield check: from a building rolled from
/// the job's id on, the first with a wall to break whose record the player
/// has not changed. The wall chosen, or `None` where there is none.
pub fn break_wall(save: &mut WorldSave, job: &Job, town: &mut town_record::Town) -> Option<Repair> {
    let n = town.buildings.len();
    if n == 0 {
        return None;
    }
    let start = (mix(job.id ^ 0x7265_7061) % n as u64) as usize;
    for k in 0..n {
        let b = (start + k) % n;
        let id = town_record::building_id(town.site, b as u32);
        // A building the player has changed is theirs to keep as it is.
        if save.yields.has_record(BUILDING_RECORD, id) {
            continue;
        }
        let walls = town.buildings[b].breakable();
        if walls.is_empty() {
            continue;
        }
        let wall = walls[(mix(job.id ^ b as u64) % walls.len() as u64) as usize];
        let mut building = town.buildings[b].clone();
        building.broken.push(wall);
        let repair = Repair {
            site: town.site,
            building: b as u32,
            wall,
            mended: false,
        };
        let proposal = Proposal {
            edits: Vec::new(),
            records: vec![
                Record::of(BUILDING_RECORD, id, RECORD_SCHEMA, &building),
                Record::of(REPAIR_KIND, job.id, REPAIR_SCHEMA, &repair),
            ],
        };
        match save.propose(&repair_author(job.id), proposal) {
            Ok(Some(_)) => {
                save.note_record_kinds(&[(REPAIR_KIND, REPAIR_SCHEMA)]);
                town.buildings[b] = building;
                return Some(repair);
            }
            Ok(None) => return None,
            Err(why) => info!("job {}: building {b} not broken: {why:?}", job.id),
        }
    }
    None
}

/// Mend a repair job's wall with planks from `slots` (design 29): the
/// building record without the wall, the player's, with the pack in the
/// same line; then the job's record mended. Whether it was.
pub fn mend(
    save: &mut WorldSave,
    slots: &mut Slots,
    job: u64,
    town: &mut town_record::Town,
) -> bool {
    let Some(mut repair) = repair_of(save, job).filter(|r| !r.mended) else {
        return false;
    };
    let Some(mut building) = town.buildings.get(repair.building as usize).cloned() else {
        return false;
    };
    let mut hands = slots.clone();
    if !hands.take_item(Item::Good(Good::PLANKS), MEND_PLANKS) {
        return false;
    }
    building.broken.retain(|w| *w != repair.wall);
    let record = Record::of(
        BUILDING_RECORD,
        town_record::building_id(town.site, repair.building),
        RECORD_SCHEMA,
        &building,
    );
    if !save.store_with_hands(record, RECORD_SCHEMA, &hands) {
        return false;
    }
    repair.mended = true;
    save.store(
        &Author::Player(0),
        vec![Record::of(REPAIR_KIND, job, REPAIR_SCHEMA, &repair)],
    );
    town.buildings[repair.building as usize] = building;
    *slots = hands;
    true
}

/// Where a broken wall's middle stands, in the planet's frame (f32, a
/// query), from its town's chart.
pub fn gap_point(towns: &Towns, repair: &Repair, radius_m: f32) -> Option<Vec3> {
    let held = towns.held.iter().find(|h| h.site.id == repair.site)?;
    let chart = towns.charts.get(&repair.site)?;
    let (chart, patch) = (&chart.0, &chart.1);
    let [c, r, d, s] = repair.wall;
    let cell = &patch.cells[chart.cell(c, r)?];
    let side = chart.side(c, r, d as usize)?;
    let n = cell.corners.len();
    let mid = (cell.corners[side % n] + cell.corners[(side + 1) % n]).normalize();
    let building = held.town.buildings.get(repair.building as usize)?;
    let alt = held.town.terrace as f32
        + building.floor as f32
        + s as f32 * pbd_core::settlement::STOREY_M;
    Some(mid * (radius_m + alt + 1.2))
}

/// Where each taken, unfinished repair job's broken wall stands, as
/// directions from the planet's centre: what the compass and the map mark
/// (`job-waypoints`). Only while its town is held, since the gap is read
/// off the town's chart.
pub fn repair_directions(eco: &Economy, save: &WorldSave, towns: &Towns) -> Vec<Vec3> {
    open_jobs(eco, JobKind::Repair)
        .iter()
        .filter_map(|j| repair_of(save, j.id))
        .filter_map(|r| gap_point(towns, &r, crate::planet::PLANET_RADIUS))
        .filter_map(|p| p.try_normalize())
        .collect()
}

/// The repair jobs: break each taken job's wall once its town stands, and
/// stand the town again with the gap.
pub fn repairs(eco: Res<Economy>, save: Option<ResMut<WorldSave>>, mut towns: ResMut<Towns>) {
    let Some(mut save) = save else {
        return;
    };
    for job in open_jobs(&eco, JobKind::Repair) {
        if repair_of(&save, job.id).is_some() {
            continue;
        }
        if !towns.standing.iter().any(|s| s.site == job.site) {
            continue;
        }
        let Some(k) = towns.held.iter().position(|h| h.site.id == job.site) else {
            continue;
        };
        let mut town = towns.held[k].town.clone();
        match break_wall(&mut save, &job, &mut town) {
            Some(repair) => {
                info!("job {}: a wall broken at {:?}", job.id, repair.wall);
                towns.held[k].town = town;
                towns.recut.insert(job.site);
            }
            None => warn!("job {}: no wall to break in town {}", job.id, job.site),
        }
    }
}

// ---- Lamps at dusk (`town-missions-tree` design 8) -------------------------

/// How near a town lamp the eye must be to light it, metres.
pub const LAMP_REACH_M: f32 = 3.0;

/// The town lamp the eye is near, while a lamps job of its town is taken
/// and not done and the clock is at dusk: its town and its place in the
/// town's lamps.
pub fn lamp_in_reach(
    eco: &Economy,
    towns: &Towns,
    eye: Vec3,
    hour: f32,
    radius_m: f32,
) -> Option<(u32, usize)> {
    if !pbd_core::economy::jobs::is_dusk(hour) {
        return None;
    }
    let jobs = open_jobs(eco, JobKind::Lamps);
    if jobs.is_empty() {
        return None;
    }
    let ground = pbd_core::settlement::ground::installed()?;
    // The installed grounds are the held towns', in site-id order.
    towns
        .held
        .iter()
        .zip(ground.towns())
        .filter(|(h, _)| jobs.iter().any(|j| j.site == h.site.id))
        .find_map(|(h, g)| {
            g.lamps().iter().enumerate().find_map(|(i, l)| {
                let at = l.direction * (radius_m + l.altitude_m);
                (at.distance(eye) <= LAMP_REACH_M).then_some((h.site.id, i))
            })
        })
}

/// Offer LIGHT to the interact key near a town lamp at dusk, with its
/// town's lamps job taken.
pub fn offer_lamp(
    eco: Res<Economy>,
    towns: Res<Towns>,
    sun: Option<Res<crate::sky::Sun>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    interact: Option<ResMut<crate::interact::Interact>>,
    lit: Res<LitLamps>,
) {
    let (Some(sun), Some(mut interact)) = (sun, interact) else {
        return;
    };
    let Some(eye) = eye_point(&cameras, frame.as_deref()) else {
        return;
    };
    let radius = crate::planet::terrain_config().radius_m;
    let day = sun.clock.day();
    if let Some((site, i)) = lamp_in_reach(&eco, &towns, eye, sun.clock.hour(), radius)
        && !lit.0.contains(&(day, site, i))
    {
        interact.offer(crate::interact::Offer::new(
            crate::interact::Act::Light,
            crate::ui_kit::tr("jobs.lamp"),
        ));
    }
}

/// The lamps lit this run, by day, town and lamp: a lamp counts once an
/// evening.
#[derive(Resource, Default)]
pub struct LitLamps(pub std::collections::BTreeSet<(u64, u32, usize)>);

/// The interact key at a town lamp at dusk lights it for the job: one lamp
/// counted, once an evening.
#[allow(clippy::too_many_arguments)]
pub fn use_lamp(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    eco: Res<Economy>,
    towns: Res<Towns>,
    sun: Option<Res<crate::sky::Sun>>,
    interact: Option<Res<crate::interact::Interact>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    mut lit: ResMut<LitLamps>,
    mut did: MessageWriter<crate::missions::Did>,
) {
    let (Some(keys), Some(sun)) = (keys, sun) else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyF)
        || !crate::interact::may(interact.as_deref(), crate::interact::Reader::Lamps)
    {
        return;
    }
    let Some(eye) = eye_point(&cameras, frame.as_deref()) else {
        return;
    };
    let radius = crate::planet::terrain_config().radius_m;
    let day = sun.clock.day();
    let Some((site, i)) = lamp_in_reach(&eco, &towns, eye, sun.clock.hour(), radius) else {
        return;
    };
    if lit.0.insert((day, site, i)) {
        info!("lamp {i} of town {site} lit");
        did.write(crate::missions::Did::new(
            Kind::Light,
            scoped(site, "lamp"),
            1,
        ));
    }
}

/// The broken wall the eye is near, with planks in the pack: its job.
pub fn wall_in_reach(
    eco: &Economy,
    save: &WorldSave,
    towns: &Towns,
    eye: Vec3,
    radius_m: f32,
) -> Option<u64> {
    open_jobs(eco, JobKind::Repair).into_iter().find_map(|job| {
        let repair = repair_of(save, job.id).filter(|r| !r.mended)?;
        let at = gap_point(towns, &repair, radius_m)?;
        (at.distance(eye) <= MEND_REACH_M).then_some(job.id)
    })
}

/// The eye in the planet's frame, f32 (a query).
fn eye_point(
    cameras: &Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<&crate::planet::PlanetRenderFrame>,
) -> Option<Vec3> {
    let (g, _) = cameras.iter().find(|(_, c)| c.is_active)?;
    let centre = frame?.center;
    Some((g.translation().as_dvec3() - centre).as_vec3())
}

/// Offer MEND to the interact key near a broken wall, with planks in the pack.
pub fn offer_mend(
    eco: Res<Economy>,
    save: Option<Res<WorldSave>>,
    towns: Res<Towns>,
    hotbar: Option<Res<crate::hotbar::Hotbar>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    interact: Option<ResMut<crate::interact::Interact>>,
) {
    let (Some(save), Some(hotbar), Some(mut interact)) = (save, hotbar, interact) else {
        return;
    };
    if hotbar.0.count(Item::Good(Good::PLANKS)) < u32::from(MEND_PLANKS) {
        return;
    }
    let Some(eye) = eye_point(&cameras, frame.as_deref()) else {
        return;
    };
    let radius = crate::planet::terrain_config().radius_m;
    if wall_in_reach(&eco, &save, &towns, eye, radius).is_some() {
        interact.offer(crate::interact::Offer::new(
            crate::interact::Act::Mend,
            crate::ui_kit::tr("jobs.wall"),
        ));
    }
}

/// The interact key at a broken wall mends it: planks taken, the wall
/// back, the job counted.
#[allow(clippy::too_many_arguments)]
pub fn use_mend(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    eco: Res<Economy>,
    save: Option<ResMut<WorldSave>>,
    mut towns: ResMut<Towns>,
    hotbar: Option<ResMut<crate::hotbar::Hotbar>>,
    interact: Option<Res<crate::interact::Interact>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    mut did: MessageWriter<crate::missions::Did>,
) {
    let (Some(keys), Some(mut save), Some(mut hotbar)) = (keys, save, hotbar) else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyF)
        || !crate::interact::may(interact.as_deref(), crate::interact::Reader::Mend)
    {
        return;
    }
    let Some(eye) = eye_point(&cameras, frame.as_deref()) else {
        return;
    };
    let radius = crate::planet::terrain_config().radius_m;
    let Some(job) = wall_in_reach(&eco, &save, &towns, eye, radius) else {
        return;
    };
    let Some(repair) = repair_of(&save, job) else {
        return;
    };
    let Some(k) = towns.held.iter().position(|h| h.site.id == repair.site) else {
        return;
    };
    let mut town = towns.held[k].town.clone();
    if mend(&mut save, &mut hotbar.0, job, &mut town) {
        info!("job {job}: the wall is mended");
        did.write(crate::missions::Did::new(
            Kind::Repair,
            scoped(repair.site, "piece"),
            1,
        ));
        towns.held[k].town = town;
        towns.recut.insert(repair.site);
    }
}
