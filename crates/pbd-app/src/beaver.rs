//! Beavers in the game (`ocean-environments` design B25): the app's half of
//! `pbd_core::beaver`.
//!
//! - **Founding.** Every few seconds, near the player on a world with
//!   rivers (generator version 8 and up), a still reach on Fields ground at
//!   40 to 60 degrees of latitude with the generator's trees beside it is
//!   looked for. The first time one is found with no colony within
//!   `COLONY_APART_M`, a colony record is stored by the world's creation,
//!   naming that site and the next two up the same river.
//! - **The process.** Once an in-game hour, `pbd_core::beaver::catch_up`
//!   gives every colony's proposals since it last acted; each goes through
//!   `WorldSave::propose` (refused whole if it touches the player's), and a
//!   taken dam or lodge cell is written into the tier as any edit is, so the
//!   flowing-water rule meets it as a wall.
//!
//! Where a site is, and its cells and trees, are read off the ground here;
//! the process is the core's, a pure function of the record.

use crate::planet::{PLANET_RADIUS, PlanetContact, PlanetFine, PlanetRenderFrame, terrain_config};
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::beaver::{self, Colony, Rules, Site};
use pbd_core::planet_gen::{self, Biome};
use pbd_core::records::Author;
use std::sync::Arc;

/// How often a site is looked for, s.
const FOUND_EVERY_S: f32 = 5.0;
/// How far round the player a site is looked for, m.
const FOUND_REACH_M: f32 = 120.0;
/// No colony is founded within this of another's first site, m.
const COLONY_APART_M: f32 = 400.0;
/// The latitudes beavers live at, degrees (decision 4's beaver stream).
const LATITUDE_DEG: (f32, f32) = (40.0, 60.0);
/// The trees a colony fells stand within this of its lip, m.
const TREE_REACH_M: f32 = 20.0;
/// The cell width every body shares, m (CLAUDE.md, "Hex size is fixed").
const CELL_M: f32 = 2.833;
/// The sites a colony knows: its own and the next ones up the river.
const SITES: usize = 3;

pub struct BeaverPlugin;

impl Plugin for BeaverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Founding>()
            .add_systems(Update, (found, run).chain().run_if(crate::world_open));
    }
}

#[derive(Resource)]
struct Founding {
    since_s: f32,
}

impl Default for Founding {
    /// The first look is made as soon as the world is open.
    fn default() -> Self {
        Self {
            since_s: FOUND_EVERY_S,
        }
    }
}

/// `PBD_BEAVER_AGE_DAYS=<days>`, a capture instrument (design B25): a colony
/// founded now is founded that many days ago, so the process catches it up
/// at once and one capture shows what it made. Unset, a colony starts now.
fn founded_days_ago() -> u64 {
    std::env::var("PBD_BEAVER_AGE_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// The world hour now: a twenty-fourth of the world's day
/// (`pbd_core::daylight::DAY_S`), not an hour of the clock on the wall.
fn hour_now(sun: Option<&crate::sky::Sun>) -> Option<u64> {
    let hour_s = f64::from(pbd_core::daylight::DAY_S) / 24.0;
    Some((sun?.clock.seconds / hour_s).floor().max(0.0) as u64)
}

#[allow(clippy::too_many_arguments)]
fn found(
    time: Res<Time>,
    mut founding: ResMut<Founding>,
    save: Option<ResMut<WorldSave>>,
    contact: Option<Res<PlanetContact>>,
    fine: Option<Res<PlanetFine>>,
    sun: Option<Res<crate::sky::Sun>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    frame: Res<PlanetRenderFrame>,
) {
    founding.since_s += time.delta_secs();
    if founding.since_s < FOUND_EVERY_S {
        return;
    }
    founding.since_s = 0.0;
    let (Some(mut save), Some(contact), Some(fine), Some(hour)) =
        (save, contact, fine, hour_now(sun.as_deref()))
    else {
        return;
    };
    let cfg = terrain_config();
    if cfg.rivers.is_none() {
        return;
    }
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    // A planet direction to query the ground at, never a stored position.
    let eye = (camera.translation().as_dvec3() - frame.center)
        .as_vec3()
        .normalize_or(Vec3::Y);
    if !beaver_latitude(eye) {
        return;
    }
    let apart = COLONY_APART_M / PLANET_RADIUS;
    let near_one = save
        .records
        .of_kind(beaver::KIND)
        .filter_map(|r| r.read::<Colony>())
        .any(|c| {
            c.sites
                .first()
                .is_some_and(|s| Vec3::from(s.at).angle_between(eye) < apart)
        });
    if near_one {
        return;
    }
    let Some(sites) = sites_near(&contact, &fine, eye) else {
        passed_over(eye, "no site in reach", 0);
        return;
    };
    let id = save
        .records
        .of_kind(beaver::KIND)
        .map(|r| r.id + 1)
        .max()
        .unwrap_or(1);
    let colony = Colony::new(sites, 4, hour.saturating_sub(founded_days_ago() * 24));
    let at = colony.sites[0].at;
    if save
        .store(&Author::Creation, vec![colony.record(id)])
        .is_some()
    {
        save.note_record_kinds(&[(beaver::KIND, beaver::SCHEMA)]);
        let first = &colony.sites[0];
        info!(
            "BEAVER colony {id} founded at {:.4} {:.4} with {} sites: lip {:?}, bed {} bank {}, \
             {} trees",
            at[1].asin().to_degrees(),
            at[2].atan2(at[0]).to_degrees(),
            colony.sites.len(),
            first.lip,
            first.bed,
            first.bank,
            first.trees.len()
        );
    }
}

/// Once an hour: every colony's proposals since it last acted, through the
/// save, and the cells taken into the tier.
fn run(
    save: Option<ResMut<WorldSave>>,
    fine: Option<ResMut<PlanetFine>>,
    contact: Option<ResMut<PlanetContact>>,
    sun: Option<Res<crate::sky::Sun>>,
    gather: Option<Res<crate::config::GatherConfig>>,
    mut last: Local<Option<u64>>,
) {
    let (Some(mut save), Some(mut fine), Some(mut contact), Some(hour)) =
        (save, fine, contact, hour_now(sun.as_deref()))
    else {
        return;
    };
    // Nothing to run until there is a colony; then once an hour.
    if *last == Some(hour) || save.records.of_kind(beaver::KIND).next().is_none() {
        return;
    }
    *last = Some(hour);
    let axe_logs = gather.map_or(4, |g| g.0.axe_logs);
    let proposals = beaver::catch_up(
        &save.records,
        &save.yields,
        hour,
        &Rules::default(),
        axe_logs,
    );
    let mut laid = Vec::new();
    for (author, proposal) in proposals {
        let edits = proposal.edits.clone();
        match save.propose(&author, proposal) {
            Ok(_) => laid.extend(edits),
            Err(why) => warn!("BEAVER {} refused: {why:?}", author.token()),
        }
    }
    if laid.is_empty() {
        return;
    }
    // Into the tier as any edit goes (`FineSet::write_layer`): the column,
    // its record and its neighbours', the water queued round it, relit.
    let mut set = (*fine.set).clone();
    let mut wrote = 0;
    for edit in &laid {
        let Some(record) = set.finest_index_of(edit.cell) else {
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
    let set = Arc::new(set);
    contact.set_fine(&set);
    fine.set = set;
    fine.version += 1;
    info!("BEAVER laid {wrote} cells at hour {hour}");
}

/// Whether a direction is at a beaver's latitude.
fn beaver_latitude(d: Vec3) -> bool {
    let lat = d.y.clamp(-1.0, 1.0).asin().to_degrees().abs();
    (LATITUDE_DEG.0..=LATITUDE_DEG.1).contains(&lat)
}

/// A point `metres` from `d` along the tangent `along`.
fn step(d: Vec3, along: Vec3, metres: f32) -> Vec3 {
    (d + along * (metres / PLANET_RADIUS)).normalize_or(d)
}

/// A colony's sites round `eye`: the lip of a still reach within reach of
/// the player and the next lips up its river, or `None`.
fn sites_near(contact: &PlanetContact, fine: &PlanetFine, eye: Vec3) -> Option<Vec<Site>> {
    let east = Vec3::Y.cross(eye).normalize_or(Vec3::X);
    let north = eye.cross(east);
    let rings = (FOUND_REACH_M / CELL_M) as i32;
    let mut tried = Vec::new();
    for ring in 0..=rings {
        let metres = ring as f32 * CELL_M;
        let around = (ring * 6).max(1);
        for k in 0..around {
            let a = k as f32 / around as f32 * std::f32::consts::TAU;
            let d = step(eye, east * a.cos() + north * a.sin(), metres);
            let Some(lip) = lip_from(d) else {
                continue;
            };
            // A lip is met from many points of its reach; each is tried once,
            // and a few at most a search (a site reads every fine record).
            if tried
                .iter()
                .any(|t: &Vec3| t.angle_between(lip) < CELL_M / PLANET_RADIUS)
            {
                continue;
            }
            if tried.len() >= 4 {
                return None;
            }
            tried.push(lip);
            let mut sites = Vec::new();
            let mut from = lip;
            while sites.len() < SITES {
                let Some(site) = site_at(contact, fine, from) else {
                    break;
                };
                sites.push(site);
                let Some(next) = next_lip_up(from) else {
                    break;
                };
                from = next;
            }
            if !sites.is_empty() {
                return Some(sites);
            }
        }
    }
    None
}

/// Whether `d` is in a river's channel on beaver ground: Fields, at a
/// beaver's latitude.
fn beaver_water(d: Vec3) -> Option<pbd_core::rivers::RiverHere> {
    let cfg = terrain_config();
    let here = pbd_core::rivers::at(cfg, d)?;
    if here.across_m > here.half_width_m * 0.6 || !beaver_latitude(d) {
        return None;
    }
    let bank = step(
        d,
        here.upstream.cross(d).normalize_or(Vec3::X),
        here.half_width_m + 4.0,
    );
    let ground = planet_gen::surface_altitude(cfg, bank);
    (planet_gen::biome_at(cfg, bank, ground) == Biome::Fields).then_some(here)
}

/// The lip down the reach `d` is in: the last point of the reach before its
/// water falls to the next, walked downstream in half cells, within 30 m.
fn lip_from(d: Vec3) -> Option<Vec3> {
    let here = beaver_water(d)?;
    let mut at = d;
    let mut upstream = here.upstream;
    for _ in 0..((30.0 / (CELL_M * 0.5)) as usize) {
        let next = step(at, -upstream, CELL_M * 0.5);
        let there = pbd_core::rivers::at(terrain_config(), next)?;
        if there.level < here.level {
            return Some(at);
        }
        if there.level > here.level || there.across_m > there.half_width_m {
            return None;
        }
        at = next;
        upstream = there.upstream;
    }
    None
}

/// The next lip up the river from a lip: walk upstream, over the step into
/// the reach above, and on to that reach's own lip, which is the first
/// point of it; within 120 m.
fn next_lip_up(lip: Vec3) -> Option<Vec3> {
    let cfg = terrain_config();
    let here = pbd_core::rivers::at(cfg, lip)?;
    let mut at = lip;
    let mut upstream = here.upstream;
    for _ in 0..((120.0 / (CELL_M * 0.5)) as usize) {
        at = step(at, upstream, CELL_M * 0.5);
        let there = pbd_core::rivers::at(cfg, at)?;
        upstream = there.upstream;
        if there.level > here.level {
            // Into the reach above: its lip is where its water steps down.
            return beaver_water(at).map(|_| at);
        }
    }
    None
}

/// Says once in a while why a lip was passed over, so a world with no
/// colony can be read from its log; at most a few lines a run.
fn passed_over(lip: Vec3, why: &str, n: u16) {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SAID: AtomicUsize = AtomicUsize::new(0);
    if SAID.fetch_add(1, Ordering::Relaxed) < 8 {
        info!(
            "BEAVER lip at {:.4} {:.4} passed over: {why} ({n})",
            lip.y.clamp(-1.0, 1.0).asin().to_degrees(),
            lip.z.atan2(lip.x).to_degrees()
        );
    }
}

/// A site at a lip: the cells across the stream there, the bed and the
/// bank, the reach's cells beside the bank, and the trees within reach.
fn site_at(contact: &PlanetContact, fine: &PlanetFine, lip: Vec3) -> Option<Site> {
    let here = beaver_water(lip)?;
    let across = here.upstream.cross(lip).normalize_or(Vec3::X);
    let key = |d: Vec3| contact.finest_key(d);
    let records = fine.set.finest_records();
    let column_of = |d: Vec3| {
        let index = contact.finest_cell(d)?;
        fine.set.columns.column(index).cloned()
    };
    // The bed and the water's top at the middle of the stream.
    let middle = column_of(lip)?;
    let bed = middle.surface()?;
    let water_top = pbd_core::column::layer_at(here.level as f32 - 0.5)
        .map(|l| l as u16)
        .unwrap_or(bed as u16);
    let bed = bed as u16;
    let bank = water_top.max(bed) + 1;
    // A beaver dams a stream it can build up from the bed to the water's
    // top: no more than the dam's four layers of water at the lip.
    if bank - bed > Rules::default().dam_layers + 1 {
        passed_over(lip, "too deep", bank - bed - 1);
        return None;
    }
    // Across the channel, every cell whose top is no higher than the bed's
    // layer above: the dam spans the water.
    let reach = (here.half_width_m / CELL_M).ceil() as i32 + 1;
    let mut lip_cells: Vec<u32> = Vec::new();
    for k in -reach..=reach {
        let d = step(lip, across, k as f32 * CELL_M * 0.8);
        let (Some(cell), Some(column)) = (key(d), column_of(d)) else {
            continue;
        };
        if column.surface().is_some_and(|s| s as u16 <= bed + 1) && !lip_cells.contains(&cell) {
            lip_cells.push(cell);
        }
    }
    if lip_cells.is_empty() {
        passed_over(lip, "no channel cells", 0);
        return None;
    }
    // The lodge: the reach's cells by the bank, a few metres up from the lip.
    let mut pond: Vec<u32> = Vec::new();
    for metres in [5.0, 8.0, 11.0] {
        let d = step(
            step(lip, here.upstream, metres),
            across,
            here.half_width_m * 0.6,
        );
        if let Some(cell) = key(d)
            && !pond.contains(&cell)
            && !lip_cells.contains(&cell)
        {
            pond.push(cell);
        }
    }
    // The generator's trees within reach, nearest first, by their cell's key.
    let reach = TREE_REACH_M / PLANET_RADIUS;
    let table = &terrain_config().trees;
    let mut trees: Vec<(f32, u64)> = records
        .iter()
        .filter_map(|r| {
            let d = Vec3::from_slice(&r.direction_height[..3]).normalize_or(Vec3::Y);
            let off = d.angle_between(lip);
            (off < reach && pbd_core::flora::grows_tree(table, r.spare[0], r.metadata[1]))
                .then_some((off, u64::from(r.key())))
        })
        .collect();
    trees.sort_by(|a, b| a.0.total_cmp(&b.0));
    if trees.is_empty() {
        passed_over(lip, "no trees", 0);
        return None;
    }
    Some(Site {
        at: lip.to_array(),
        lip: lip_cells,
        pond,
        bed,
        bank,
        trees: trees.into_iter().take(8).map(|(_, id)| id).collect(),
    })
}
