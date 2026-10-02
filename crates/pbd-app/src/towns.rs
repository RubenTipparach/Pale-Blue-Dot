//! The towns in the world (`cities-in-the-world`, slice 1): a settlement
//! built into the ground at its site and drawn with the mockup's own pieces
//! and pixels.
//!
//! Once a world's sites are on disk, the home town is read from the world's
//! save, or, the first time, laid from its template onto the finest cells
//! round its anchor (`pbd_core::settlement::chart`) and stored
//! (`settlement::record`, slice 3a). It is built from the record only once
//! the record is on disk, and always from the record: the ground under it
//! is terraced (`settlement::ground`, installed where every height is read,
//! and the planet rebuilt round the player), and its buildings are cut from
//! their cells' real corners (`settlement::pieces`). The town is one entity
//! with a mesh per texture, lit by the field as a drop or a craft is.
//!
//! Every village, walled town and harbour site stands (slices 4a, 4b and
//! 4d): each is laid and stored on the
//! world's first open, its ground installed with every other town's, and its
//! pieces cut and faded in only within [`STAND_M`] of the viewer. The other
//! kinds follow, a kind at a time.

use crate::field_light::{Faded, LitByField, LitLikeTerrain, RoomLights, SkyShare};
use crate::planet::PlanetRenderFrame;
use crate::planet::lattice::Lattice;
use crate::saves::WorldSave;
use crate::sites::WorldSites;
use crate::walking::{EYE_HEIGHT, HALF_HEIGHT, Structures, Walker, WalkingState};
use bevy::asset::RenderAssetUsages;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use pbd_core::planet_gen::{self, TerrainConfig};
use pbd_core::records::Author;
use pbd_core::settlement::chart::{Chart, Patch};
use pbd_core::settlement::ground::{self, Ground, TownGround};
use pbd_core::settlement::pieces::{BuildingSolids, MeshBuf, Meshes, RoomLight};
use pbd_core::settlement::record::{self, Stored, Town};
use pbd_core::settlement::sea;
use pbd_core::settlement::{Kits, Template};
use pbd_core::sites::{Site, SiteKind};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

/// How far round a site its cells are fetched, metres: the village's grid
/// reaches 82 m from its centre, and the margin rings a few more.
pub const PATCH_M: f32 = 130.0;

/// How far round a site of `kind` its cells are fetched, metres: a
/// harbour's further by the most its anchor may be shifted toward its sea
/// (slice 4d).
pub fn patch_m(kind: SiteKind) -> f32 {
    match kind {
        SiteKind::Harbour => PATCH_M + sea::SHIFT_RINGS as f32 * 3.0,
        _ => PATCH_M,
    }
}

fn asset(path: &str) -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets")).join(path)
}

/// `assets/config/kits.ron`, checked.
pub fn load_kits() -> Kits {
    let path = asset("config/kits.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let kits: Kits = ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    kits.validate()
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    kits
}

/// A settlement template, `assets/settlements/v1/<name>.json`.
pub fn load_template(name: &str) -> Template {
    let path = asset(&format!("settlements/v1/{name}.json"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[derive(Deserialize)]
struct Manifest {
    textures: Vec<ManifestTexture>,
}

#[derive(Deserialize)]
struct ManifestTexture {
    name: String,
    #[allow(dead_code)]
    width: u32,
    #[allow(dead_code)]
    height: u32,
    repeat_m: f32,
    /// Cut out where it is clear, as the mockup's net is (task 4.2c).
    #[serde(default)]
    cut: bool,
}

fn load_manifest() -> Manifest {
    let path = asset("textures/settlement/manifest.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Metres one repeat of each town texture covers, from its manifest.
pub fn load_repeats() -> BTreeMap<String, f32> {
    load_manifest()
        .textures
        .into_iter()
        .map(|t| (t.name, t.repeat_m))
        .collect()
}

/// The town textures cut out where they are clear (task 4.2c), from the
/// manifest: drawn alpha-masked, and casting no shadow, since the shadow
/// pass casts whole triangles.
pub fn cut_textures() -> &'static BTreeSet<String> {
    static CUT: std::sync::OnceLock<BTreeSet<String>> = std::sync::OnceLock::new();
    CUT.get_or_init(|| {
        load_manifest()
            .textures
            .into_iter()
            .filter(|t| t.cut)
            .map(|t| t.name)
            .collect()
    })
}

/// A town built from its record, before it is in the world.
pub struct Laid {
    pub site: u32,
    pub name: String,
    pub chart: Chart,
    pub patch: Patch,
    /// The terrace, metres over the planet's radius.
    pub terrace_m: f32,
    pub ground: TownGround,
    pub meshes: Meshes,
    /// Each building's solids, for the walker (slice 2a).
    pub solids: Vec<BuildingSolids>,
    /// Each building's inside faces, by texture (`sun-shadows` decision 7).
    pub rooms: Vec<Meshes>,
    /// What burns in each building's rooms (decision 7a).
    pub lights: Vec<Vec<RoomLight>>,
    /// Its dressing things standing, and those left off its chart (task
    /// 4.2c).
    pub dressing: (usize, usize),
    /// Its cog's ship among its solids, where the template moors it
    /// (`sail-the-cog` design 6). The town stands it empty: the cog is a
    /// craft (step 3, part 3).
    pub cog: Option<usize>,
}

/// The drawn sea's surface, metres over the radius: the layers' sea level
/// less the depth the water pass draws its sheet under it (`water.ron`'s
/// `depth_offset_m`), which is what boats float on.
pub fn sheet_m(config: &TerrainConfig, water: &crate::config::WaterSettings) -> f32 {
    config.sea_level_m - water.depth_offset_m
}

/// [`sheet_m`] in a world, from its water settings or the shipped ones.
fn sheet_in(world: &World, config: &TerrainConfig) -> f32 {
    match world.get_resource::<crate::config::WaterSettings>() {
        Some(water) => sheet_m(config, water),
        None => sheet_m(config, &crate::config::WaterSettings::default()),
    }
}

/// The patch of finest cells round a direction.
pub fn patch_round(direction: Vec3, radius_m: f32, reach_m: f32) -> Patch {
    let locals = Lattice::default().cells_in_band(11, direction, 0.0, reach_m / radius_m);
    Patch {
        keys: locals
            .iter()
            .map(|l| pbd_core::cell_key::key(l.point.into()).unwrap_or(u32::MAX))
            .collect(),
        cells: locals.into_iter().map(|l| l.cell).collect(),
    }
}

/// The natural ground's layer at a direction, metres over the radius.
fn natural(config: &TerrainConfig) -> impl Fn(Vec3) -> f32 + '_ {
    |d: Vec3| planet_gen::surface_altitude(config, d).floor()
}

/// Lay a template at a site, once: its anchor on the site's cell, the
/// layout's east along the cell's side nearest east turned by the site's
/// seed (`record::turn`; the home village does not turn), and the terrace
/// from the natural ground (`record::lay`). What this returns is stored, and
/// the town is built from the store.
pub fn lay_out(site: &Site, template: &Template, config: &TerrainConfig) -> Result<Town, String> {
    let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
    lay_on(site, template, config, &patch)
}

fn lay_on(
    site: &Site,
    template: &Template,
    config: &TerrainConfig,
    patch: &Patch,
) -> Result<Town, String> {
    let at = patch.nearest(site.direction).ok_or("an empty patch")?;
    let (_, east) = pbd_core::geo::north_east(patch.cells[at].direction);
    let sides = patch.cells[at].corners.len();
    let turn = if site.home { 0 } else { record::turn(site.id) };
    let d0 = (patch.side_toward(at, east) + turn) % sides;
    if template.sea {
        // A harbour faces its sea (slice 4d): its turn and an anchor shift
        // are chosen from the ground, ties going to the site's own turn.
        let anchor = record::template_anchor(template);
        let sea_m = config.sea_level_m;
        let (cell, side, share) =
            sea::placement(template, patch, at, d0, anchor, natural(config), sea_m)?;
        info!(
            "{}: the harbour lies with {:.0}% of its cells agreeing about the sea",
            site.name,
            share * 100.0
        );
        return record::lay_at_sea(template, site.id, patch, cell, side, natural(config), sea_m);
    }
    record::lay(template, site.id, patch, at, d0, natural(config))
}

/// Build a town from its definition: its ground and its pieces, cut on the
/// cells it was laid on, and the masonry of `template`, the template it was
/// laid from (slice 4c). `repeat_m` is how far a texture repeats, and
/// `sheet_m` the drawn sea's surface over the radius ([`sheet_m`]).
pub fn build(
    site: &Site,
    town: &Town,
    template: Option<&Template>,
    kits: &Kits,
    repeat_m: &dyn Fn(&str) -> f32,
    config: &TerrainConfig,
    sheet_m: f32,
) -> Result<Laid, String> {
    let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
    let built = record::build_town(
        town,
        template,
        &patch,
        kits,
        repeat_m,
        config.radius_m,
        sheet_m,
        natural(config),
    )?;
    Ok(Laid {
        site: site.id,
        name: site.name.clone(),
        chart: built.chart,
        patch,
        terrace_m: town.terrace as f32,
        ground: built.ground,
        meshes: built.meshes,
        solids: built.solids,
        rooms: built.rooms,
        lights: built.lights,
        dressing: (built.dressing, built.dressing_skipped),
        cog: built.cog,
    })
}

/// A site's town as the world's save holds it.
pub fn stored(save: &WorldSave, site: u32) -> Stored {
    record::from_records(&save.records, site)
}

/// Queue a freshly laid town into the save, as the world's creation, and
/// name its record kinds in the identity. The sequence of its last line
/// (the settlement's), or `None` when the save has failed.
pub fn store(save: &mut WorldSave, town: &Town) -> Option<u64> {
    let seq = save.store(&Author::Creation, record::to_records(town))?;
    save.note_record_kinds(&[
        (record::BUILDING_RECORD, record::RECORD_SCHEMA),
        (record::SETTLEMENT_RECORD, record::RECORD_SCHEMA),
    ]);
    Some(seq)
}

/// A site's town: the one its save holds, or one laid now from `template`
/// and queued to the save, with the sequence to wait for (0 when nothing was
/// written). A damaged record is an error, and nothing is written over it.
///
/// `None` is a site left unsettled (task 4.4): one already stored so, or one
/// whose ground holds a player's edit, which is stored so now and never
/// laid, so the town never buries the player's work.
pub fn ensure(
    save: &mut WorldSave,
    site: &Site,
    template: &Template,
    config: &TerrainConfig,
) -> Result<(Option<Town>, u64), String> {
    match stored(save, site.id) {
        Stored::Town(town) => Ok((Some(town), 0)),
        Stored::Unsettled => Ok((None, 0)),
        Stored::Damaged(why) => Err(why),
        Stored::None => {
            let patch = patch_round(site.direction, config.radius_m, patch_m(site.kind));
            let town = lay_on(site, template, config, &patch)?;
            let (_, ground) = record::ground_of(&town, &patch, config.radius_m, natural(config))?;
            if ground.touches(&save.edits) {
                let seq = save
                    .store(
                        &Author::Creation,
                        vec![record::unsettled_record(
                            site.id,
                            "the player had changed its ground",
                        )],
                    )
                    .ok_or("the save refused the unsettled site")?;
                save.note_record_kinds(&[(record::UNSETTLED_RECORD, record::RECORD_SCHEMA)]);
                return Ok((None, seq));
            }
            let seq = store(save, &town).ok_or("the save refused the town")?;
            Ok((Some(town), seq))
        }
    }
}

/// Metres from the viewer within which a town stands (slice 4a): at
/// 1.2 km a house is a few pixels.
pub const STAND_M: f32 = 1200.0;
/// Metres past which a standing town is dropped. The gap keeps one from
/// standing and dropping on every step across the line.
pub const DROP_M: f32 = 1500.0;
/// Seconds a town takes to fade in or out (priority 1: no pop-in).
pub const FADE_S: f32 = 1.0;

/// What the towns are built from: the kits, the templates and the textures.
#[derive(Resource)]
pub struct TownAssets {
    pub kits: Arc<Kits>,
    pub village: Template,
    /// The walled town (slice 4b).
    pub walled: Template,
    /// The harbour (slice 4d).
    pub harbour: Template,
    /// The desert town (slice 4e).
    pub desert: Template,
    /// The tundra camp (slice 4g).
    pub tundra: Template,
    pub repeats: Arc<BTreeMap<String, f32>>,
}

impl TownAssets {
    /// The template a kind of site is laid from, where one is shipped: the
    /// village's, the walled town's, the harbour's, the desert's and the
    /// tundra's so far (slice 4 adds a kind at a time).
    pub fn template_for(&self, kind: SiteKind) -> Option<&Template> {
        match kind {
            SiteKind::Village => Some(&self.village),
            SiteKind::Walled => Some(&self.walled),
            SiteKind::Harbour => Some(&self.harbour),
            SiteKind::Desert => Some(&self.desert),
            SiteKind::Tundra => Some(&self.tundra),
            _ => None,
        }
    }

    /// The template a stored town names, by its scene.
    pub fn template_named(&self, scene: &str) -> Option<&Template> {
        [
            &self.village,
            &self.walled,
            &self.harbour,
            &self.desert,
            &self.tundra,
        ]
        .into_iter()
        .find(|t| t.scene == scene)
    }

    /// How far each texture repeats, metres, as the cutter asks it.
    pub fn repeat(&self) -> impl Fn(&str) -> f32 + Clone + Send + Sync + 'static {
        let repeats = self.repeats.clone();
        move |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0)
    }
}

/// A town the world holds, read back from its save (slice 4a). Its ground is
/// installed wherever the viewer is; its pieces stand only in range.
#[derive(Clone, Debug)]
pub struct Held {
    pub site: Site,
    pub town: Town,
}

impl Held {
    /// Its anchor at its terrace, in the planet's frame.
    pub fn point(&self, radius_m: f32) -> Vec3 {
        self.site.direction * (radius_m + self.town.terrace as f32)
    }
}

/// A town standing in the world.
#[derive(Clone, Debug)]
pub struct Standing {
    pub site: u32,
    pub name: String,
    pub anchor: Vec3,
    pub terrace_m: f32,
    pub entity: Entity,
    /// Its buildings' place in the walker's [`Structures`].
    first: usize,
    count: usize,
    /// How much of it is drawn, 0..1, and whether it is going.
    shown: f32,
    leaving: bool,
}

/// Every town the world holds, the ones standing now, and the site list they
/// were built for.
#[derive(Resource, Default)]
pub struct Towns {
    for_sites: Option<Vec<u32>>,
    /// Towns laid and queued to the save: none is shown until the writer's
    /// mark passes this line, the last one's (decision 8: written before it
    /// is shown).
    writing: Option<u64>,
    /// The sites laid and waiting on that mark.
    waiting: Vec<Site>,
    /// Every town the world holds, by site id.
    pub held: Vec<Held>,
    /// The sites left unsettled (task 4.4), by id.
    pub unsettled: Vec<u32>,
    pub standing: Vec<Standing>,
    /// Towns being cut on the pool, by site (decision 5: published whole).
    cutting: Vec<(u32, Task<Result<Laid, String>>)>,
}

impl Towns {
    /// The towns a world holds and the sites it left unsettled, standing
    /// nowhere yet: what the map reads (and its tests make).
    pub fn holding(held: Vec<Held>, unsettled: Vec<u32>) -> Self {
        Self {
            held,
            unsettled,
            ..default()
        }
    }

    /// Whether a site's town is built: laid, stored and read back, so it
    /// stands when the viewer comes near (the map's highlight).
    pub fn built(&self, site: u32) -> bool {
        self.held.iter().any(|h| h.site.id == site) || self.waiting.iter().any(|s| s.id == site)
    }

    /// One town standing with `count` buildings at the head of the walker's
    /// [`Structures`], for a test that puts them there itself.
    #[cfg(test)]
    pub fn standing_alone(site: u32, count: usize) -> Self {
        Self {
            standing: vec![Standing {
                site,
                name: String::new(),
                anchor: Vec3::Y,
                terrace_m: 0.0,
                entity: Entity::PLACEHOLDER,
                first: 0,
                count,
                shown: 1.0,
                leaving: false,
            }],
            ..default()
        }
    }

    /// Where the `n`th building of a site's town is in the walker's
    /// [`Structures`], while the town stands.
    pub fn index(&self, site: u32, n: usize) -> Option<usize> {
        self.standing
            .iter()
            .find(|s| s.site == site)
            .filter(|s| n < s.count)
            .map(|s| s.first + n)
    }
}

/// Put the walker back on the ground once the towns are built: a capture
/// that starts in a town (`--at`) stands on its terrace, not on the ground
/// that was there before it.
#[derive(Resource)]
pub struct RespawnInTown;

/// The root of a town's meshes.
#[derive(Component)]
pub struct TownRoot;

/// A town's flame: unlit, so it is not faded with the town's pieces but
/// shown once the town is more than half there.
#[derive(Component)]
pub struct TownFlame;

/// Build the towns once a world's sites are on disk: an exclusive system,
/// because it installs the ground and rebuilds the planet round the player.
///
/// A town the save holds is built from its records. One it does not hold is
/// laid from the template, queued to the save, and built from the records
/// once they are on disk. A town whose record is damaged is not built, and
/// nothing is written over it.
pub fn build_towns(world: &mut World) {
    let Some(sites) = world
        .get_resource::<WorldSites>()
        .and_then(|w| w.ready())
        .map(<[Site]>::to_vec)
    else {
        return;
    };
    let ids: Vec<u32> = sites.iter().map(|s| s.id).collect();
    if world.resource::<Towns>().for_sites.as_ref() != Some(&ids) {
        start_towns(world, &sites, ids);
        return;
    }
    let Some(seq) = world.resource::<Towns>().writing else {
        return;
    };
    let Some(save) = world.get_resource::<WorldSave>() else {
        return;
    };
    if let Some(why) = save.failure() {
        error!("the towns were not saved, so none is shown: {why}");
        let mut towns = world.resource_mut::<Towns>();
        towns.writing = None;
        towns.waiting.clear();
        towns.held.clear();
        return;
    }
    if save.committed() < seq {
        return;
    }
    let waiting = std::mem::take(&mut world.resource_mut::<Towns>().waiting);
    world.resource_mut::<Towns>().writing = None;
    let mut read = Vec::new();
    {
        let save = world.resource::<WorldSave>();
        for site in waiting {
            match stored(save, site.id) {
                Stored::Town(town) => read.push(Held { site, town }),
                other => error!(
                    "{} was written and does not read back: {other:?}",
                    site.name
                ),
            }
        }
    }
    info!("{} towns are in the save", read.len());
    world.resource_mut::<Towns>().held.extend(read);
    settle(world);
}

/// A new site list: take down the old towns, and read or lay every site's
/// town that has a template.
fn start_towns(world: &mut World, sites: &[Site], ids: Vec<u32>) {
    let old: Vec<Entity> = world
        .resource::<Towns>()
        .standing
        .iter()
        .map(|t| t.entity)
        .collect();
    for entity in old {
        if let Ok(e) = world.get_entity_mut(entity) {
            e.despawn();
        }
    }
    *world.resource_mut::<Towns>() = Towns {
        for_sites: Some(ids),
        ..default()
    };
    world.insert_resource(Structures::default());
    world.insert_resource(crate::planet::shadow::TownCasters::default());
    ground::install(None);
    let mut sites: Vec<Site> = {
        let assets = world.resource::<TownAssets>();
        sites
            .iter()
            .filter(|s| assets.template_for(s.kind).is_some())
            .cloned()
            .collect()
    };
    if sites.is_empty() {
        return;
    }
    sites.sort_by_key(|s| s.id);
    if !world.contains_resource::<WorldSave>() {
        warn!("the towns are not built: there is no save to keep them in");
        return;
    }
    let config = *crate::planet::terrain_config();
    let started = std::time::Instant::now();
    let (mut held, mut waiting, mut wait, mut laid, mut unsettled) =
        (Vec::new(), Vec::new(), 0u64, 0usize, Vec::new());
    world.resource_scope(|world, mut save: Mut<WorldSave>| {
        let assets = world.resource::<TownAssets>();
        for site in sites {
            let template = assets.template_for(site.kind).expect("chosen for one");
            if stored(&save, site.id) == Stored::None {
                laid += 1;
            }
            match ensure(&mut save, &site, template, &config) {
                Ok((Some(_), seq)) if seq > 0 => {
                    wait = wait.max(seq);
                    waiting.push(site);
                }
                // Read, or stored with nothing to wait for (a save with no
                // disk behind it): built from the records all the same.
                Ok((Some(_), _)) => match stored(&save, site.id) {
                    Stored::Town(town) => held.push(Held { site, town }),
                    other => error!("{} does not read from the save: {other:?}", site.name),
                },
                Ok((None, seq)) => {
                    wait = wait.max(seq);
                    unsettled.push(site.id);
                    info!(
                        "{} is unsettled: the player's work is on its ground",
                        site.name
                    );
                }
                Err(why) => error!("{} is not built: {why}", site.name),
            }
        }
    });
    info!(
        "towns: {} read, {laid} of them laid out now, {} waiting on the disk, {} unsettled, in {:.2} s",
        held.len(),
        waiting.len(),
        unsettled.len(),
        started.elapsed().as_secs_f32()
    );
    let mut towns = world.resource_mut::<Towns>();
    towns.held = held;
    towns.unsettled = unsettled;
    if wait > 0 {
        info!("the towns laid out now are shown once they are in the save");
        towns.writing = Some(wait);
        towns.waiting = waiting;
        return;
    }
    settle(world);
}

/// Where the viewer is in the planet's frame: the camera, or the walker
/// where no camera is active yet.
fn viewer(world: &mut World) -> Option<Vec3> {
    let centre = world.get_resource::<PlanetRenderFrame>().map(|f| f.center);
    let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    if let Some(centre) = centre
        && let Some((_, at)) = cameras.iter(world).find(|(c, _)| c.is_active)
    {
        return Some((at.translation().as_dvec3() - centre).as_vec3());
    }
    let mut walkers =
        world.query_filtered::<&avian3d::prelude::Position, With<crate::walking::Walker>>();
    walkers.iter(world).next().map(|p| p.0)
}

/// Every held town's ground, installed at once, and the planet rebuilt round
/// the player once; then whatever is in range stands, whole, as the ground
/// does.
fn settle(world: &mut World) {
    let started = std::time::Instant::now();
    let config = *crate::planet::terrain_config();
    let mut held = std::mem::take(&mut world.resource_mut::<Towns>().held);
    held.sort_by_key(|h| h.site.id);
    let mut grounds = Vec::new();
    let mut kept = Vec::new();
    for h in held {
        let patch = patch_round(h.site.direction, config.radius_m, patch_m(h.site.kind));
        let template = world
            .resource::<TownAssets>()
            .template_named(&h.town.template)
            .cloned();
        match record::ground_of(&h.town, &patch, config.radius_m, natural(&config)) {
            Ok((chart, g)) => {
                // Its street lamps and lanterns (task 5.2), from its template.
                let lamps = template
                    .as_ref()
                    .map(|t| record::lamps_of(&h.town, t, &chart, &patch))
                    .unwrap_or_default();
                let g = g.with_lamps(lamps);
                // Where the town stands: a harbour lies up to its shift from
                // its site's marker (slice 4d).
                let (lat, lon) = pbd_core::geo::lat_lon(g.anchor()).degrees();
                info!(
                    "{}: a {:?} at --at {lat:.5} {lon:.5}, a terrace at {} m, {} lamps",
                    h.site.name,
                    h.site.kind,
                    h.town.terrace,
                    g.lamps().len()
                );
                grounds.push(g);
                kept.push(h);
            }
            Err(why) => error!("{} could not be built: {why}", h.site.name),
        }
    }
    if grounds.is_empty() {
        return;
    }
    let count = grounds.len();
    ground::install(Some(Ground::new(config, grounds)));
    world.resource_mut::<Towns>().held = kept.clone();
    let near = viewer(world)
        .map(|p| p.normalize_or(Vec3::Y))
        .unwrap_or(kept[0].site.direction);
    crate::planet::rebuild_planet(world, near);
    info!(
        "{count} towns' ground installed and the planet rebuilt in {:.2} s",
        started.elapsed().as_secs_f32()
    );
    let at = near * (config.radius_m + 2.0);
    let (kits, repeat) = {
        let assets = world.resource::<TownAssets>();
        (assets.kits.clone(), assets.repeat())
    };
    let sheet = sheet_in(world, &config);
    for h in kept {
        if (h.point(config.radius_m) - at).length() > STAND_M {
            continue;
        }
        let template = world
            .resource::<TownAssets>()
            .template_named(&h.town.template)
            .cloned();
        match build(
            &h.site,
            &h.town,
            template.as_ref(),
            &kits,
            &repeat,
            &config,
            sheet,
        ) {
            Ok(laid) => stand(world, &h, laid, 1.0),
            Err(why) => warn!("{} could not be built: {why}", h.site.name),
        }
    }
    if world.contains_resource::<RespawnInTown>() {
        crate::walking::respawn(world);
    }
}

/// Whether a town `d` metres from the viewer that is not standing is stood.
pub fn wanted(d: f32) -> bool {
    d < STAND_M
}

/// Whether a standing town `d` metres from the viewer is going: past
/// [`DROP_M`] it goes, and once going it turns back only within [`STAND_M`].
pub fn leaving(d: f32, was: bool) -> bool {
    d > DROP_M || (was && d > STAND_M)
}

/// How much of a town is drawn after `dt` seconds more of its fade, toward
/// whole or toward gone: [`FADE_S`] end to end, and never a jump.
pub fn faded(shown: f32, leaving: bool, dt: f32) -> f32 {
    let step = dt.max(0.0) / FADE_S;
    if leaving {
        (shown - step).max(0.0)
    } else {
        (shown + step).min(1.0)
    }
}

/// Stand the towns that come within [`STAND_M`] of the viewer, cut on the
/// pool, and drop the ones past [`DROP_M`], each faded over [`FADE_S`]
/// (slice 4a).
pub fn stand_in_range(world: &mut World) {
    {
        let towns = world.resource::<Towns>();
        if towns.writing.is_some() || towns.held.is_empty() {
            return;
        }
    }
    let Some(at) = viewer(world) else {
        return;
    };
    let config = *crate::planet::terrain_config();
    let radius = config.radius_m;
    // Cuts that are done stand, unless the viewer has gone on past.
    let mut done = Vec::new();
    world
        .resource_mut::<Towns>()
        .cutting
        .retain_mut(|(site, task)| match block_on(future::poll_once(task)) {
            Some(result) => {
                done.push((*site, result));
                false
            }
            None => true,
        });
    for (site, result) in done {
        let Some(h) = world
            .resource::<Towns>()
            .held
            .iter()
            .find(|h| h.site.id == site)
            .cloned()
        else {
            continue;
        };
        match result {
            Ok(laid) if (h.point(radius) - at).length() <= DROP_M => stand(world, &h, laid, 0.0),
            Ok(_) => {}
            Err(why) => warn!("{} could not be built: {why}", h.site.name),
        }
    }
    // Every template a town can be laid from: the harbour's too, or a
    // harbour coming into range is cut with none of its sea's pieces.
    let (kits, repeat, templates) = {
        let assets = world.resource::<TownAssets>();
        (
            assets.kits.clone(),
            assets.repeat(),
            [
                assets.village.clone(),
                assets.walled.clone(),
                assets.harbour.clone(),
                assets.desert.clone(),
                assets.tundra.clone(),
            ],
        )
    };
    let sheet = sheet_in(world, &config);
    let dt = world
        .get_resource::<Time>()
        .map_or(1.0 / 60.0, |t| t.delta_secs());
    let mut gone = Vec::new();
    let mut fades = Vec::new();
    {
        let mut towns = world.resource_mut::<Towns>();
        let towns = &mut *towns;
        for h in &towns.held {
            let d = (h.point(radius) - at).length();
            if let Some(s) = towns.standing.iter_mut().find(|s| s.site == h.site.id) {
                s.leaving = leaving(d, s.leaving);
            } else if wanted(d) && !towns.cutting.iter().any(|(site, _)| *site == h.site.id) {
                let id = h.site.id;
                let template = templates
                    .iter()
                    .find(|t| t.scene == h.town.template)
                    .cloned();
                let (h, kits, repeat) = (h.clone(), kits.clone(), repeat.clone());
                let task = AsyncComputeTaskPool::get().spawn(async move {
                    build(
                        &h.site,
                        &h.town,
                        template.as_ref(),
                        &kits,
                        &repeat,
                        &config,
                        sheet,
                    )
                });
                towns.cutting.push((id, task));
            }
        }
        for s in &mut towns.standing {
            let before = s.shown;
            s.shown = faded(s.shown, s.leaving, dt);
            if s.shown != before {
                fades.push((s.entity, s.shown));
            }
            if s.leaving && s.shown == 0.0 {
                gone.push(s.site);
            }
        }
    }
    for (entity, shown) in fades {
        show(world, entity, shown);
    }
    for site in gone {
        drop_town(world, site);
    }
}

/// Draw a town's root `shown` of the way in: its pieces through the mask,
/// its flames once it is more than half there.
fn show(world: &mut World, root: Entity, shown: f32) {
    if let Ok(mut e) = world.get_entity_mut(root) {
        e.insert(Faded(1.0 - shown));
    }
    let flames: Vec<Entity> = world
        .get::<Children>(root)
        .map(|c| c.iter().collect::<Vec<Entity>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|&c| world.get::<TownFlame>(c).is_some())
        .collect();
    let visibility = if shown > 0.5 {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for flame in flames {
        if let Some(mut v) = world.get_mut::<Visibility>(flame) {
            *v = visibility;
        }
    }
}

/// Take a town out of the world: its meshes, its buildings from the
/// walker's [`Structures`], and its shadow.
fn drop_town(world: &mut World, site: u32) {
    let Some(k) = world
        .resource::<Towns>()
        .standing
        .iter()
        .position(|s| s.site == site)
    else {
        return;
    };
    let gone = world.resource_mut::<Towns>().standing.remove(k);
    if let Ok(e) = world.get_entity_mut(gone.entity) {
        e.despawn();
    }
    if let Some(mut structures) = world.get_resource_mut::<Structures>() {
        let end = (gone.first + gone.count).min(structures.0.len());
        structures.0.drain(gone.first.min(end)..end);
    }
    for s in &mut world.resource_mut::<Towns>().standing {
        if s.first > gone.first {
            s.first -= gone.count;
        }
    }
    if let Some(mut casters) = world.get_resource_mut::<crate::planet::shadow::TownCasters>() {
        casters.0.retain(|(s, _)| *s != site);
    }
    info!("{} is out of range", gone.name);
}

/// Stand a town cut from its record: its meshes, its doors as the save holds
/// them, and its buildings for the walker, drawn `shown` of the way in.
fn stand(world: &mut World, held: &Held, laid: Laid, shown: f32) {
    let site = &held.site;
    let (footprint, margin) = laid.ground.counts();
    // Each door as its save holds it: open where the player left it open,
    // shut where there is no record (slice 2b).
    let mut solids = laid.solids.clone();
    // Its cog is a craft (`sail-the-cog` step 3, part 3), made with its
    // boats: the ship's place among its pieces is kept, empty, so the
    // town's numbering holds, and its gangplank stays the town's.
    if let Some(n) = laid.cog {
        solids[n] = BuildingSolids {
            frame: solids[n].frame,
            reach_m: 0.0,
            solids: Vec::new(),
            roof_plan: Vec::new(),
            surfaces: Vec::new(),
            doors: Vec::new(),
            top_m: 0.0,
            rooms: Meshes::new(),
            lights: Vec::new(),
        };
    }
    door_states(
        &mut solids,
        site.id,
        world.get_resource::<WorldSave>().map(|s| &s.records),
        world.contains_resource::<OpenDoors>(),
    );
    let first = {
        let mut structures = world.get_resource_or_insert_with(Structures::default);
        let first = structures.0.len();
        structures.0.extend(solids.iter().cloned());
        first
    };
    let entity = spawn_town(world, &laid);
    spawn_doors(world, entity, site.id, &solids);
    let triangles: usize = laid.meshes.values().map(|m| m.positions.len() / 3).sum();
    let (things, off) = laid.dressing;
    let mut dressing = match (things, off) {
        (0, 0) => String::new(),
        (n, 0) => format!(", {n} dressing things"),
        (n, off) => format!(", {n} dressing things ({off} off its chart)"),
    };
    if laid.cog.is_some() {
        dressing.push_str(", a berth for its cog");
    }
    info!(
        "{} stands: {} buildings{dressing}, {triangles} triangles in {} textures, a terrace at {} m over {footprint} cells eased over {margin}",
        laid.name,
        held.town.buildings.len(),
        laid.meshes.len(),
        laid.terrace_m,
    );
    world.resource_mut::<Towns>().standing.push(Standing {
        site: laid.site,
        name: laid.name.clone(),
        anchor: site.direction,
        terrace_m: laid.terrace_m,
        entity,
        first,
        count: solids.len(),
        shown,
        leaving: false,
    });
    show(world, entity, shown);
}

/// How far each town texture repeats, metres, as the cutter asks it: the
/// towns' table where they are loaded, else their manifest's.
pub(crate) fn repeat_in(world: &World) -> impl Fn(&str) -> f32 + use<> {
    let repeats = world
        .get_resource::<TownAssets>()
        .map(|a| a.repeats.clone())
        .unwrap_or_else(|| Arc::new(load_repeats()));
    move |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0)
}

/// A cut's faces drawn under `parent` in the towns' textures, in the
/// parent's frame, a flame unlit in [`FLAME_RGB`] as a town's is: the cog a
/// craft carries. Each mesh drawn, by its texture's name.
pub(crate) fn spawn_meshes(
    world: &mut World,
    parent: Entity,
    meshes: &Meshes,
) -> Vec<(String, Entity)> {
    let parts: Vec<(String, Mesh, Option<Handle<Image>>)> = {
        let assets = world.resource::<AssetServer>();
        meshes
            .iter()
            .map(|(name, buf)| {
                let image = (name != FLAME).then(|| texture(assets, name));
                (name.clone(), to_mesh(buf), image)
            })
            .collect()
    };
    let mut out = Vec::new();
    for (name, mesh, image) in parts {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(match image {
                Some(image) => StandardMaterial {
                    base_color_texture: Some(image),
                    perceptual_roughness: 0.93,
                    ..default()
                },
                None => StandardMaterial {
                    base_color: Color::linear_rgb(FLAME_RGB[0], FLAME_RGB[1], FLAME_RGB[2]),
                    unlit: true,
                    cull_mode: None,
                    ..default()
                },
            });
        let part = world
            .spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()))
            .id();
        world.entity_mut(parent).add_child(part);
        out.push((name, part));
    }
    out
}

/// Open every door when the towns are built, as the player would, and write
/// nothing to the save: the capture flag `--open-doors`.
#[derive(Resource)]
pub struct OpenDoors;

/// A door's leaf in the world (slice 2b): its building and door in the
/// walker's [`Structures`], its record, and how far it has swung (0 shut, a
/// quarter turn open).
#[derive(Component, Clone, Copy, Debug)]
pub struct TownDoor {
    /// Its town's site, and its building's number there.
    pub site: u32,
    pub building: usize,
    pub door: usize,
    pub record: u64,
    pub angle: f32,
}

/// How far a door swings open, and how fast, radians a second.
const OPEN_ANGLE: f32 = std::f32::consts::FRAC_PI_2;
const SWING_RAD_S: f32 = 5.0;
/// How far from the eye E reaches for a doorway's middle, metres.
pub const DOOR_REACH_M: f32 = 2.2;

/// Each of a town's doors as its save holds it (slice 2b): open where the
/// player left it open, shut where there is no record. `open_all` is the
/// capture flag `--open-doors`, which writes nothing.
pub fn door_states(
    solids: &mut [BuildingSolids],
    site: u32,
    records: Option<&pbd_core::records::Records>,
    open_all: bool,
) {
    for (n, b) in solids.iter_mut().enumerate() {
        let building = record::building_id(site, n as u32);
        for d in &mut b.doors {
            d.open = open_all
                || records
                    .is_some_and(|r| record::door_open(r, record::door_id(building, d.index)));
        }
    }
}

/// A triangle soup as a Bevy mesh.
fn to_mesh(buf: &MeshBuf) -> Mesh {
    let count = buf.positions.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, buf.uvs.clone())
    .with_inserted_indices(Indices::U32((0..count).collect()))
}

/// A door leaf drawn swung `angle` from shut.
fn leaf_mesh(assets: &TownAssets, b: &BuildingSolids, door: usize, angle: f32) -> Mesh {
    let repeat = |m: &str| {
        assets
            .repeats
            .get(m)
            .copied()
            .filter(|r| *r > 0.0)
            .unwrap_or(2.0)
    };
    let meshes = b.doors[door].mesh(b.frame, angle, &repeat);
    to_mesh(meshes.values().next().expect("a leaf is timber"))
}

/// Each door of a town's buildings as its own entity under the town's root.
fn spawn_doors(world: &mut World, root: Entity, site: u32, solids: &[BuildingSolids]) {
    let image = texture(world.resource::<AssetServer>(), "timber");
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color_texture: Some(image),
            perceptual_roughness: 0.93,
            ..default()
        });
    for (n, b) in solids.iter().enumerate() {
        for (k, d) in b.doors.iter().enumerate() {
            let angle = if d.open { OPEN_ANGLE } else { 0.0 };
            let mesh = leaf_mesh(world.resource::<TownAssets>(), b, k, angle);
            let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
            let door = TownDoor {
                site,
                building: n,
                door: k,
                record: record::door_id(record::building_id(site, n as u32), d.index),
                angle,
            };
            let child = world
                .spawn((
                    Name::new("Door"),
                    Mesh3d(mesh),
                    MeshMaterial3d(material.clone()),
                    Transform::default(),
                    door,
                ))
                .id();
            world.entity_mut(root).add_child(child);
        }
    }
}

/// E opens or shuts the door in reach: the nearest doorway within
/// [`DOOR_REACH_M`] of the eye and in front of it. The change is the
/// player's, and it is written to the save before the leaf moves; a save
/// that refuses it leaves the door as it was.
pub fn use_doors(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    walking: Option<Res<WalkingState>>,
    walker: Query<&avian3d::prelude::Position, With<Walker>>,
    structures: Option<ResMut<Structures>>,
    towns: Res<Towns>,
    save: Option<ResMut<WorldSave>>,
    doors: Query<&TownDoor>,
) {
    // A camera that does not walk (a capture's view from above) opens no
    // door.
    let (Some(keys), Some(walking)) = (keys, walking) else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyE) || !walking.active {
        return;
    }
    if !(walking.captured || walking.scripted) {
        return;
    }
    let (Some(mut structures), Ok(position)) = (structures, walker.single()) else {
        return;
    };
    let up = position.0.normalize_or(Vec3::Y);
    let eye = position.0 + up * (EYE_HEIGHT - HALF_HEIGHT);
    let (heading, pitch) = walking.view();
    let look = heading * pitch.cos() + up * pitch.sin();
    let nearest = doors
        .iter()
        .filter_map(|door| {
            let b = structures.0.get(towns.index(door.site, door.building)?)?;
            let d = b.doors.get(door.door)?;
            let middle = b.frame.world(Vec3::new(d.middle.x, d.y0 + 1.1, d.middle.y));
            let to = middle - eye;
            let dist = to.length();
            (dist <= DOOR_REACH_M && to.dot(look) >= 0.3 * dist).then_some((dist, *door))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let Some((_, door)) = nearest else {
        return;
    };
    let Some(index) = towns.index(door.site, door.building) else {
        return;
    };
    let leaf = &mut structures.0[index].doors[door.door];
    let open = !leaf.open;
    if let Some(mut save) = save {
        if save
            .store(
                &Author::Player(0),
                vec![record::door_record(door.record, open)],
            )
            .is_none()
        {
            warn!("the save refused the door; it stays as it was");
            return;
        }
        save.note_record_kinds(&[(record::DOOR_RECORD, record::RECORD_SCHEMA)]);
    }
    leaf.open = open;
}

/// Swing each door's leaf toward its state, redrawing it as it goes.
pub fn swing_doors(
    time: Res<Time>,
    structures: Option<Res<Structures>>,
    towns: Res<Towns>,
    assets: Res<TownAssets>,
    mut doors: Query<(&mut TownDoor, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(structures) = structures else {
        return;
    };
    let step = SWING_RAD_S * time.delta_secs().max(1.0 / 60.0);
    for (mut door, mesh) in &mut doors {
        let Some(b) = towns
            .index(door.site, door.building)
            .and_then(|i| structures.0.get(i))
        else {
            continue;
        };
        let Some(leaf) = b.doors.get(door.door) else {
            continue;
        };
        let target = if leaf.open { OPEN_ANGLE } else { 0.0 };
        if door.angle == target {
            continue;
        }
        door.angle = if door.angle < target {
            (door.angle + step).min(target)
        } else {
            (door.angle - step).max(target)
        };
        if let Some(m) = meshes.get_mut(&mesh.0) {
            *m = leaf_mesh(&assets, b, door.door, door.angle);
        }
    }
}

/// A texture of the town's, sampled nearest and repeated.
fn texture(assets: &AssetServer, name: &str) -> Handle<Image> {
    assets.load_with_settings(
        format!("textures/settlement/{name}.png"),
        |s: &mut ImageLoaderSettings| {
            s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                address_mode_w: ImageAddressMode::Repeat,
                mag_filter: ImageFilterMode::Nearest,
                min_filter: ImageFilterMode::Linear,
                mipmap_filter: ImageFilterMode::Linear,
                ..default()
            });
        },
    )
}

fn spawn_town(world: &mut World, laid: &Laid) -> Entity {
    let centre = world
        .get_resource::<PlanetRenderFrame>()
        .map_or(Vec3::ZERO, |f| f.center.as_vec3());
    let parts: Vec<(Mesh, Handle<Image>, bool)> = {
        let assets = world.resource::<AssetServer>();
        let cut = cut_textures();
        laid.meshes
            .iter()
            .map(|(name, buf)| (to_mesh(buf), texture(assets, name), cut.contains(name)))
            .collect()
    };
    // Each building's rooms, apart: they take the room's own share of the
    // sky, which its doors decide (`sun-shadows` decision 7).
    let rooms: Vec<(usize, Mesh, Option<Handle<Image>>)> = {
        let assets = world.resource::<AssetServer>();
        laid.rooms
            .iter()
            .enumerate()
            .flat_map(|(b, meshes)| {
                meshes.iter().map(move |(name, buf)| {
                    // A flame is drawn unlit, warm, and is no texture
                    // (decision 7a).
                    let image = (name != FLAME).then(|| texture(assets, name));
                    (b, to_mesh(buf), image)
                })
            })
            .collect()
    };
    let root = world
        .spawn((
            Name::new(format!("Town: {}", laid.name)),
            TownRoot,
            LitByField,
            LitLikeTerrain,
            Transform::from_translation(centre),
            Visibility::default(),
        ))
        .id();
    let pieces = parts
        .into_iter()
        .map(|(mesh, image, cut)| (None, mesh, Some(image), cut))
        .chain(
            rooms
                .into_iter()
                .map(|(b, mesh, image)| (Some(b), mesh, image, false)),
        );
    for (room, mesh, image, cut) in pieces {
        let flame = image.is_none();
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(match image {
                Some(image) => StandardMaterial {
                    base_color_texture: Some(image),
                    perceptual_roughness: 0.93,
                    // A net is see-through between its cords (task 4.2c).
                    alpha_mode: if cut {
                        AlphaMode::Mask(0.5)
                    } else {
                        AlphaMode::Opaque
                    },
                    ..default()
                },
                None => StandardMaterial {
                    base_color: Color::linear_rgb(FLAME_RGB[0], FLAME_RGB[1], FLAME_RGB[2]),
                    unlit: true,
                    cull_mode: None,
                    ..default()
                },
            });
        let mut child = world.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()));
        if flame {
            child.insert(TownFlame);
        }
        if let Some(building) = room.filter(|_| !flame) {
            child.insert(RoomLights(
                laid.lights.get(building).cloned().unwrap_or_default(),
            ));
            child.insert((
                TownRoom {
                    site: laid.site,
                    building,
                },
                SkyShare {
                    sky: ROOM_SKY_SHUT,
                    bounce: ROOM_BOUNCE,
                },
            ));
        }
        let child = child.id();
        world.entity_mut(root).add_child(child);
    }
    let casting = casting(laid);
    let mut casters =
        world.get_resource_or_insert_with(crate::planet::shadow::TownCasters::default);
    casters.0.retain(|(site, _)| *site != laid.site);
    casters.0.push((laid.site, std::sync::Arc::new(casting)));
    root
}

/// What a town casts into the sun's cascades (`sun-shadows` task 5.1): every
/// triangle of its outside and its rooms, a floor shading the room under it,
/// and not its doors, which swing, nor what is cut out (a net), which would
/// cast as a solid sheet.
pub fn casting(laid: &Laid) -> Vec<[f32; 3]> {
    let cut = cut_textures();
    laid.meshes
        .iter()
        .filter(|(name, _)| !cut.contains(*name))
        .map(|(_, buf)| buf)
        .chain(laid.rooms.iter().flat_map(|m| {
            m.iter()
                .filter(|(name, _)| name.as_str() != FLAME)
                .map(|(_, buf)| buf)
        }))
        .flat_map(|m| m.positions.iter().copied())
        .collect()
}

/// The pieces' name for a flame (`pbd_core::settlement::pieces`), which is
/// drawn unlit in [`FLAME_RGB`] and casts nothing.
pub const FLAME: &str = "flame";
/// A flame's colour, linear: between the terrain's lantern fire at its root
/// and at its tip (`planet_surface.wgsl`), kept under the tonemapper's
/// shoulder, which took a brighter or paler flame to a cream cone rather
/// than a fire.
pub const FLAME_RGB: [f32; 3] = [0.92, 0.46, 0.12];

/// The share of the sky a room takes with a door of its building open, and
/// with all of them shut (`sun-shadows` decision 7): round the day a room is
/// about a quarter as bright as the street, as the mockup's rooms are.
pub const ROOM_SKY_OPEN: f32 = 0.3;
pub const ROOM_SKY_SHUT: f32 = 0.2;
/// How much of the sun bounces round a room, times `field_lit.wgsl`'s
/// `ROOM_BOUNCE`.
pub const ROOM_BOUNCE: f32 = 0.0;

/// The rooms' share of the sky, open and shut, where a launch sets it
/// (`--room-sky`): for tuning against the captures without a rebuild.
#[derive(Resource, Clone, Copy, Debug)]
pub struct RoomSky {
    pub open: f32,
    pub shut: f32,
    pub bounce: f32,
}

impl Default for RoomSky {
    fn default() -> Self {
        Self {
            open: ROOM_SKY_OPEN,
            shut: ROOM_SKY_SHUT,
            bounce: ROOM_BOUNCE,
        }
    }
}

/// A mesh of a building's rooms: its index in the walker's [`Structures`].
#[derive(Component, Clone, Copy, Debug)]
pub struct TownRoom {
    /// Its town's site, and its building's number there.
    pub site: u32,
    pub building: usize,
}

/// A room takes more of the sky while a door of its building stands open.
pub fn rooms_follow_doors(
    structures: Option<Res<Structures>>,
    towns: Option<Res<Towns>>,
    sky: Option<Res<RoomSky>>,
    mut rooms: Query<(&TownRoom, &mut SkyShare)>,
) {
    let Some(structures) = structures else {
        return;
    };
    let sky = sky.map(|s| *s).unwrap_or_default();
    for (room, mut share) in &mut rooms {
        let open = towns
            .as_ref()
            .and_then(|t| t.index(room.site, room.building))
            .and_then(|i| structures.0.get(i))
            .is_some_and(|b| b.doors.iter().any(|d| d.open));
        let want = SkyShare {
            sky: if open { sky.open } else { sky.shut },
            bounce: sky.bounce,
        };
        if *share != want {
            *share = want;
        }
    }
}

/// Keep each town on the planet as its render frame moves.
fn follow_frame(frame: Res<PlanetRenderFrame>, mut roots: Query<&mut Transform, With<TownRoot>>) {
    let centre = frame.center.as_vec3();
    for mut t in &mut roots {
        if t.translation != centre {
            t.translation = centre;
        }
    }
}

/// The towns.
pub struct TownsPlugin;

impl Plugin for TownsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TownAssets {
            kits: Arc::new(load_kits()),
            village: load_template("village"),
            walled: load_template("town"),
            harbour: load_template("coast"),
            desert: load_template("desert"),
            tundra: load_template("tundra"),
            repeats: Arc::new(load_repeats()),
        })
        .init_resource::<Towns>()
        .add_systems(
            Update,
            (
                build_towns,
                stand_in_range,
                follow_frame,
                use_doors,
                swing_doors,
                rooms_follow_doors,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests;
