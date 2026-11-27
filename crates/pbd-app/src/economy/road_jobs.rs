//! Jobs on the roads in the game (`townsfolk` design 46): the rockfall the
//! world drops on a town's road when the job is taken, counted cell by cell
//! as the player digs it clear, and the trader who walks the road to the
//! next town while the player keeps up.
//!
//! The rules are `pbd_core::economy::road_jobs`; this module places, stores
//! and counts them through the same taken jobs, `Did` and hand-in every
//! town job goes through.

use super::Economy;
use crate::saves::WorldSave;
use crate::towns::Towns;
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_core::economy::follow;
use pbd_core::economy::jobs::{Job, JobKind, scoped};
use pbd_core::economy::road_jobs::{
    self, ESCORT_KIND, ESCORT_SCHEMA, Escort, ROCKFALL_KIND, ROCKFALL_SCHEMA, Rockfall,
};
use pbd_core::edits::Edit;
use pbd_core::missions::Kind;
use pbd_core::records::{Author, Proposal, Record, Refusal};
use pbd_core::terrain::Material;
use pbd_core::townsfolk::PersonId;
use std::collections::{BTreeMap, HashSet};

/// The author the world drops a rockfall as.
pub fn rockfall_author(job: u64) -> Author {
    Author::World {
        process: "rockfall".into(),
        kind: pbd_core::economy::jobs::JOBS_KIND.into(),
        id: job,
    }
}

/// A job's rockfall as stored.
pub fn rockfall_of(save: &WorldSave, job: u64) -> Option<Rockfall> {
    save.records
        .get(ROCKFALL_KIND, job)
        .and_then(|r| r.read::<Option<Rockfall>>().flatten())
}

/// A job's trader as stored; `None` once home or before they are placed.
pub fn escort_of(save: &WorldSave, job: u64) -> Option<Escort> {
    save.records
        .get(ESCORT_KIND, job)
        .and_then(|r| r.read::<Option<Escort>>().flatten())
}

/// Whether the rock at `(cell, layer)` is air again.
fn is_air(save: &WorldSave, cell: u32, layer: u16) -> bool {
    save.edits
        .for_cell(cell)
        .iter()
        .rev()
        .find(|(l, _)| *l == layer)
        .is_some_and(|(_, m)| *m == Material::Air)
}

/// The road cells a rockfall stood on that the player has dug clear and
/// changed nothing else on: drawn and stood on as road again.
pub fn dug_clear(save: &WorldSave) -> HashSet<u32> {
    save.records
        .of_kind(ROCKFALL_KIND)
        .filter_map(|r| r.read::<Option<Rockfall>>().flatten())
        .flat_map(|f| f.cells)
        .filter(|&(c, l)| {
            is_air(save, c, l) && save.edits.for_cell(c).iter().all(|(layer, _)| *layer == l)
        })
        .map(|(c, _)| c)
        .collect()
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

/// The road from `site` toward `to` (any road of the town's for `None`),
/// its cells outward from the town.
fn road_from(
    save: &WorldSave,
    site: u32,
    to: Option<u32>,
) -> Option<Vec<pbd_core::roads::RoadCell>> {
    pbd_core::roads::stored(&save.records)
        .into_iter()
        .filter(|r| to.is_none_or(|t| r.from_site == t || r.to_site == t))
        .find_map(|r| road_jobs::outward(&r, site).filter(|c| !c.is_empty()))
}

/// Drop a job's rockfall (design 46): stone in the first air layer over
/// each cell, proposed by the world through the yield check (a cell the
/// player touched is left out and the next tried), and its record. The
/// edits laid, for the terrain.
pub fn drop_rockfall(save: &mut WorldSave, job: &Job) -> Option<Vec<Edit>> {
    let cells = road_from(save, job.site, None)?;
    let want = job.wanted().max(1) as usize;
    let mut skip: HashSet<u32> = HashSet::new();
    for _ in 0..8 {
        let usable: Vec<_> = cells
            .iter()
            .filter(|c| !skip.contains(&c.key))
            .copied()
            .collect();
        let fall = road_jobs::rockfall_cells(&usable, want, pbd_core::column::layer_at);
        if fall.is_empty() {
            return None;
        }
        let edits: Vec<Edit> = fall
            .iter()
            .map(|&(cell, layer)| Edit {
                cell,
                layer,
                material: Material::Stone,
            })
            .collect();
        let record = Rockfall {
            site: job.site,
            cells: fall,
            cleared: 0,
        };
        let proposal = Proposal {
            edits: edits.clone(),
            records: vec![Record::of(
                ROCKFALL_KIND,
                job.id,
                ROCKFALL_SCHEMA,
                &Some(record),
            )],
        };
        match save.propose(&rockfall_author(job.id), proposal) {
            Ok(Some(_)) => {
                save.note_record_kinds(&[(ROCKFALL_KIND, ROCKFALL_SCHEMA)]);
                return Some(edits);
            }
            Ok(None) => return None,
            Err(Refusal::Cell(c)) => {
                skip.insert(c);
            }
            Err(why) => {
                info!("job {}: no rockfall: {why:?}", job.id);
                return None;
            }
        }
    }
    None
}

/// Write edits the world made into the standing fine set, as any edit goes
/// (the beavers' way): the column, its neighbours, relit.
pub fn into_tier(
    laid: &[Edit],
    save: &WorldSave,
    fine: &mut crate::planet::PlanetFine,
    contact: &mut crate::planet::PlanetContact,
) {
    let mut set = (*fine.set).clone();
    let mut wrote = 0;
    for edit in laid {
        let Some(record) = set
            .finest_records()
            .iter()
            .position(|r| r.key() == edit.cell)
        else {
            continue;
        };
        if set.write_layer(record, usize::from(edit.layer), edit.material, &save.edits) {
            wrote += 1;
        }
    }
    if wrote == 0 {
        return;
    }
    set.columns.relight();
    set.mark_sea();
    let set = std::sync::Arc::new(set);
    contact.set_fine(&set);
    fine.set = set;
    fine.version += 1;
}

/// Drop each taken rockfall job's rock once, and count every cell dug
/// clear since (`Clear`, `<site>:road`), its record's count written so a
/// reload never counts a cell twice.
pub fn rockfalls(
    eco: Res<Economy>,
    save: Option<ResMut<WorldSave>>,
    fine: Option<ResMut<crate::planet::PlanetFine>>,
    contact: Option<ResMut<crate::planet::PlanetContact>>,
    mut did: MessageWriter<crate::missions::Did>,
) {
    let Some(mut save) = save else {
        return;
    };
    let (mut fine, mut contact) = (fine, contact);
    for job in open_jobs(&eco, JobKind::Rockfall) {
        let Some(mut fall) = rockfall_of(&save, job.id) else {
            if save.records.get(ROCKFALL_KIND, job.id).is_some() {
                continue;
            }
            if let Some(laid) = drop_rockfall(&mut save, &job) {
                info!("job {}: rock fell on {} road cells", job.id, laid.len());
                if let (Some(f), Some(c)) = (fine.as_deref_mut(), contact.as_deref_mut()) {
                    into_tier(&laid, &save, f, c);
                }
            }
            continue;
        };
        let now = road_jobs::cleared(&fall, |c, l| is_air(&save, c, l));
        if now <= fall.cleared {
            continue;
        }
        did.write(crate::missions::Did::new(
            Kind::Clear,
            scoped(job.site, "road"),
            now - fall.cleared,
        ));
        fall.cleared = now;
        let record = Record::of(ROCKFALL_KIND, job.id, ROCKFALL_SCHEMA, &Some(fall));
        if save.store(&Author::Player(0), vec![record]).is_some() {
            info!("job {}: {now} road cells dug clear", job.id);
        }
    }
}

/// A job's trader placed (design 46): a person of the town (not its
/// trader or job giver), at the first cell of the road toward the job's
/// town.
pub fn place_trader(
    save: &WorldSave,
    job: &Job,
    roles: &[&str],
    cfg: &super::EconomyConfig,
) -> Option<Escort> {
    let to = job.to?;
    let route: Vec<u32> = road_from(save, job.site, Some(to))?
        .iter()
        .map(|c| c.key)
        .collect();
    let keepers = cfg
        .jobs
        .keepers(roles)
        .map(|k| vec![k.trader, k.giver])
        .unwrap_or_default();
    let person = follow::who(job.id ^ 0x7472_6164, roles.len(), &keepers)?;
    Some(Escort {
        site: job.site,
        to,
        person,
        route,
        along: 0.0,
        started: false,
    })
}

/// Where a trader stands `along` their road: between two cells' centres on
/// the ground, in the body's frame.
pub fn trader_point(
    escort: &Escort,
    config: &pbd_core::planet_gen::TerrainConfig,
) -> Option<DVec3> {
    let n = escort.route.len();
    if n == 0 {
        return None;
    }
    let k = (escort.along.floor() as usize).min(n - 1);
    let t = (escort.along - k as f64).clamp(0.0, 1.0) as f32;
    let a = pbd_core::roads::centre(escort.route[k])?;
    let b = pbd_core::roads::centre(escort.route[(k + 1).min(n - 1)])?;
    Some(super::special::on_ground(
        config,
        a.lerp(b, t).normalize().as_dvec3(),
    ))
}

/// Where each taken escort job's trader stands, as directions from the
/// planet's centre, by job: what the map and the compass mark (design 42o).
/// None once the trader is home and the record taken back.
pub fn escort_directions(eco: &Economy, save: &WorldSave) -> Vec<Vec3> {
    let config = *crate::planet::terrain_config();
    open_jobs(eco, JobKind::Escort)
        .iter()
        .filter_map(|j| escort_of(save, j.id))
        .filter_map(|e| trader_point(&e, &config))
        .filter_map(|p| p.try_normalize())
        .map(|d| d.as_vec3())
        .collect()
}

/// The walk between frames.
#[derive(Default)]
pub struct LeadWalk {
    player: Option<DVec3>,
    since_write: BTreeMap<u64, (f32, bool)>,
}

/// Place each taken escort job's trader, lead the player along the road
/// while they keep up, and count the trader into the other town.
#[allow(clippy::too_many_arguments)]
pub fn escorts(
    eco: Res<Economy>,
    cfg: Res<super::EconomyConfig>,
    save: Option<ResMut<WorldSave>>,
    folk: Option<ResMut<crate::townsfolk::Townsfolk>>,
    towns: Res<Towns>,
    walkers: Query<&avian3d::prelude::Position, With<crate::walking::Walker>>,
    sim: Option<Res<crate::sim_origin::SimFrame>>,
    time: Res<Time>,
    mut did: MessageWriter<crate::missions::Did>,
    mut walk: Local<LeadWalk>,
) {
    let (Some(mut save), Some(mut folk)) = (save, folk) else {
        return;
    };
    let jobs = open_jobs(&eco, JobKind::Escort);
    if jobs.is_empty() {
        walk.player = None;
        return;
    }
    let config = *crate::planet::terrain_config();
    let dt = time.delta_secs().clamp(0.0, 0.1);
    let player = sim
        .as_ref()
        .and_then(|s| walkers.iter().next().map(|p| s.0.to_world(p.0).pos));
    let player_mps = match (player, walk.player) {
        (Some(now), Some(was)) if dt > 0.0 => (now - was).length() / f64::from(dt),
        _ => 0.0,
    };
    walk.player = player;
    for job in jobs {
        let Some(mut escort) = escort_of(&save, job.id) else {
            if save.records.get(ESCORT_KIND, job.id).is_some()
                || !towns.held.iter().any(|h| h.site.id == job.site)
            {
                continue;
            }
            let roles: Vec<String> = folk.people(job.site).map(|p| p.role.clone()).collect();
            let roles: Vec<&str> = roles.iter().map(String::as_str).collect();
            if roles.is_empty() {
                continue;
            }
            if let Some(e) = place_trader(&save, &job, &roles, &cfg)
                && save
                    .store(&Author::Player(0), vec![escort_record(job.id, Some(&e))])
                    .is_some()
            {
                save.note_record_kinds(&[(ESCORT_KIND, ESCORT_SCHEMA)]);
                info!(
                    "job {}: trader {} waits on the road to {}",
                    job.id, e.person, e.to
                );
            }
            continue;
        };
        let id = PersonId {
            site: escort.site,
            index: escort.person,
        };
        let Some(here) = trader_point(&escort, &config) else {
            continue;
        };
        let mut moved = false;
        if let Some(player) = player {
            let cell_m = f64::from(pbd_core::roads::planner::cell_angle(escort.route[0]))
                * f64::from(config.radius_m);
            let (along, started, m) = road_jobs::lead(
                escort.along,
                escort.started,
                escort.route.len(),
                cell_m,
                (player - here).length(),
                player_mps,
                f64::from(dt),
            );
            let set_off = started && !escort.started;
            (escort.along, escort.started, moved) = (along, started, m);
            let there = trader_point(&escort, &config).unwrap_or(here);
            let home = pbd_core::settlement::ground::cleared(
                &config,
                there.as_vec3().normalize_or(Vec3::Y),
            ) && escort.along > 1.0;
            if home || escort.along + 1e-6 >= (escort.route.len().saturating_sub(1)) as f64 {
                did.write(crate::missions::Did::new(
                    Kind::Escort,
                    scoped(escort.to, "trader"),
                    1,
                ));
                if save
                    .store(&Author::Player(0), vec![escort_record(job.id, None)])
                    .is_some()
                {
                    info!("job {}: the trader reached town {}", job.id, escort.to);
                }
                folk.release(id);
                walk.since_write.remove(&job.id);
                continue;
            }
            // Written as they set off, when they stop after walking, and at
            // most once a second while they walk.
            let (since, walked) = walk.since_write.entry(job.id).or_insert((0.0, false));
            *since += dt;
            *walked |= moved;
            if set_off || (*walked && (!moved || *since >= 1.0)) {
                *since = 0.0;
                *walked = false;
                save.store(
                    &Author::Player(0),
                    vec![escort_record(job.id, Some(&escort))],
                );
            }
        }
        let at = trader_point(&escort, &config).unwrap_or(here);
        let next = escort.route.get(escort.along as usize + 1).copied();
        let forward = match (moved, player, next.and_then(pbd_core::roads::centre)) {
            (true, _, Some(c)) => (c.as_dvec3() * at.length() - at).try_normalize(),
            (false, Some(p), _) => (p - at).try_normalize(),
            _ => None,
        }
        .unwrap_or(DVec3::X);
        folk.drive(id, at, forward, moved);
    }
}

/// A trader's record; `None` once they are home.
pub fn escort_record(job: u64, escort: Option<&Escort>) -> Record {
    Record::of(ESCORT_KIND, job, ESCORT_SCHEMA, &escort)
}
