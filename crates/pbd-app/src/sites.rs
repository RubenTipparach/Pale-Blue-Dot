//! The planet's city sites in the game (`city-sites` group 3): each world
//! keeps its own list, in its save.
//!
//! The rules are `pbd_core::sites`, read from `assets/config/sites.ron`. A
//! world's list is made ONCE, when the world is first opened by a build with
//! sites (a new world, or an older world's first open), from the world's own
//! generator and the spawn direction. It is written into the save as records by the
//! world's creation, and read from the save ever after, so a retune of the
//! rules never moves a town in a world that has been played.
//!
//! The list is shown only once it is on disk: the map draws no site until the
//! writer's mark has passed the list's last line.

use crate::flight_view::FlightViewConfig;
use crate::planet::{generator_version, terrain_epoch};
use crate::saves::WorldSave;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::Author;
use pbd_core::sites::{self, Site, SiteList, SitesConfig};
use std::path::PathBuf;

/// `assets/config/sites.ron`.
pub fn rules_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/config/sites.ron"
    ))
}

/// The rules, read and checked. A file that does not parse or validate is a
/// startup error naming the file, as every config is (`config.rs`), never a
/// world with silently different towns.
pub fn load_rules() -> SitesConfig {
    let path = rules_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let rules: SitesConfig =
        ron::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    rules
        .validate()
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    rules
}

/// The rules a world's list is made by, when it has none yet.
#[derive(Resource, Clone)]
pub struct SitesRules(pub SitesConfig);

/// The world a list belongs to: a different slot, seed or generator is a
/// different list.
#[derive(Clone, Debug, PartialEq, Eq)]
struct WorldKey {
    slot: Option<String>,
    seed: u64,
    generator: u32,
}

#[derive(Default)]
enum Survey {
    #[default]
    Idle,
    /// Being made on the pool, for the planet of this epoch.
    Running {
        task: Task<SiteList>,
        epoch: u64,
    },
    /// Queued to the save; shown once the mark passes `seq`.
    Writing {
        seq: u64,
        sites: Vec<Site>,
    },
    Ready,
    /// The world's generator is not the planet's, or the save refused the
    /// write; nothing is shown, and the log says why.
    Failed,
}

/// This world's sites, once they are on disk.
#[derive(Resource, Default)]
pub struct WorldSites {
    sites: Vec<Site>,
    survey: Survey,
    world: Option<WorldKey>,
}

impl WorldSites {
    /// The world's sites once its save holds them; `None` while the list is
    /// being made or written (the legend says "surveying").
    pub fn ready(&self) -> Option<&[Site]> {
        matches!(self.survey, Survey::Ready).then_some(self.sites.as_slice())
    }

    /// A world whose list is on disk already: for the map's tests, which
    /// need no save behind them.
    pub fn ready_with(sites: Vec<Site>) -> Self {
        Self {
            sites,
            survey: Survey::Ready,
            world: None,
        }
    }

    /// Whether the list is being made or written.
    pub fn surveying(&self) -> bool {
        matches!(self.survey, Survey::Running { .. } | Survey::Writing { .. })
    }
}

/// The list a world's save holds, or `None` when it holds none yet.
pub fn stored(save: &WorldSave) -> Option<Vec<Site>> {
    sites::from_records(&save.records)
}

/// Queue a freshly made list into the save, as the world's creation, and
/// name its record kinds in the identity. The sequence of its last line, or
/// `None` when the save has failed.
pub fn store(
    save: &mut WorldSave,
    list: &SiteList,
    rules: &SitesConfig,
    generator: u32,
) -> Option<u64> {
    let seq = save.store(
        &Author::Creation,
        sites::to_records(list, rules.version, generator),
    )?;
    save.note_record_kinds(&[
        (sites::SITE_RECORD, sites::RECORD_SCHEMA),
        (sites::LIST_RECORD, sites::RECORD_SCHEMA),
    ]);
    Some(seq)
}

/// A world's list: the one its save holds, or one made now by `rules` and
/// queued to the save. The synchronous path, for a capture that pins its
/// frames and for the tests; the game makes the list on the pool.
pub fn ensure(
    save: &mut WorldSave,
    rules: &SitesConfig,
    terrain: &TerrainConfig,
    start: Vec3,
    threads: usize,
) -> Option<(Vec<Site>, u64)> {
    if let Some(list) = stored(save) {
        return Some((list, 0));
    }
    let list = sites::generate(rules, terrain, start, threads);
    let seq = store(save, &list, rules, save.identity.generator)?;
    Some((list.sites, seq))
}

/// Where a world's list puts its small town: the world's spawn direction,
/// the default every world starts from, as the map mockup's list was made
/// from, so a new world's list is the one the owner approved. Not the level
/// start a version-6 walker stands on, 438 m off, from which no small town
/// lies within reach (the design's finding 6). And never the player's pose,
/// which is what a saved world's flight view holds.
pub fn list_spawn() -> Vec3 {
    FlightViewConfig::default().spawn_direction.normalize()
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

/// Keep [`WorldSites`] to the open world: read its list, or make and store
/// one, and say when it is on disk.
pub fn keep_sites(
    mut sites: ResMut<WorldSites>,
    rules: Option<Res<SitesRules>>,
    save: Option<ResMut<WorldSave>>,
    sun: Option<Res<crate::sky::Sun>>,
) {
    let (Some(rules), Some(mut save)) = (rules, save) else {
        return;
    };
    let key = WorldKey {
        slot: save.slot().map(|slot| slot.id.clone()),
        seed: save.identity.seed,
        generator: save.identity.generator,
    };
    if sites.world.as_ref() != Some(&key) {
        sites.world = Some(key.clone());
        sites.sites.clear();
        sites.survey = Survey::Idle;
        if let Some(list) = stored(&save) {
            info!("{} sites read from the save", list.len());
            sites.sites = list;
            sites.survey = Survey::Ready;
            return;
        }
        if key.generator != generator_version() {
            warn!(
                "the world is generator {} and the planet {}; no sites are made",
                key.generator,
                generator_version()
            );
            sites.survey = Survey::Failed;
            return;
        }
        let start = list_spawn();
        let terrain = *crate::planet::terrain_config();
        let config = rules.0.clone();
        if sun.is_some_and(|sun| !sun.running) {
            // A capture: made in place, so every frame is the same frame.
            let list = sites::generate(&config, &terrain, start, threads());
            finish(&mut sites, &mut save, &rules.0, list);
        } else {
            let epoch = terrain_epoch();
            sites.survey = Survey::Running {
                task: AsyncComputeTaskPool::get().spawn(async move {
                    let started = std::time::Instant::now();
                    let list = sites::generate(&config, &terrain, start, threads());
                    info!(
                        "the world's sites surveyed: {} in {:.1} s",
                        list.sites.len(),
                        started.elapsed().as_secs_f32()
                    );
                    list
                }),
                epoch,
            };
        }
        return;
    }
    let stale = matches!(&sites.survey, Survey::Running { epoch, .. } if *epoch != terrain_epoch());
    if stale {
        // The planet changed under the survey: it is for another world, and
        // the next frame starts this one's again.
        sites.world = None;
        sites.survey = Survey::Idle;
        return;
    }
    let made = match &mut sites.survey {
        Survey::Running { task, .. } => block_on(future::poll_once(task)),
        _ => None,
    };
    if let Some(list) = made {
        finish(&mut sites, &mut save, &rules.0, list);
        return;
    }
    let committed = save.committed();
    if let Survey::Writing { seq, sites: list } = &mut sites.survey
        && committed >= *seq
    {
        let list = std::mem::take(list);
        info!("{} sites are in the save", list.len());
        sites.sites = list;
        sites.survey = Survey::Ready;
    }
}

fn finish(sites: &mut WorldSites, save: &mut WorldSave, rules: &SitesConfig, list: SiteList) {
    for (kind, missing) in &list.shortfall {
        info!("{missing} fewer {} than the rules ask for", kind.name());
    }
    let generator = save.identity.generator;
    sites.survey = match store(save, &list, rules, generator) {
        Some(seq) => Survey::Writing {
            seq,
            sites: list.sites,
        },
        None => {
            error!("the world's sites could not be saved; none are shown");
            Survey::Failed
        }
    };
}

/// The sites: the rules at startup, and each world's list kept.
pub struct SitesPlugin;

impl Plugin for SitesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SitesRules(load_rules()))
            .init_resource::<WorldSites>()
            .add_systems(Update, keep_sites);
    }
}

#[cfg(test)]
mod tests;
