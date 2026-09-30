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
//! Only the home village is built so far; the other kinds follow in slice 4.

use crate::field_light::{LitByField, LitLikeTerrain, SkyShare};
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
use pbd_core::planet_gen::{self, TerrainConfig};
use pbd_core::records::Author;
use pbd_core::settlement::chart::{Chart, Patch};
use pbd_core::settlement::ground::{self, Ground, TownGround};
use pbd_core::settlement::pieces::{BuildingSolids, MeshBuf, Meshes};
use pbd_core::settlement::record::{self, Stored, Town};
use pbd_core::settlement::{Kits, Template};
use pbd_core::sites::{Site, SiteKind};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// How far round a site its cells are fetched, metres: the village's grid
/// reaches 82 m from its centre, and the margin rings a few more.
pub const PATCH_M: f32 = 130.0;

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
}

/// Metres one repeat of each town texture covers, from its manifest.
pub fn load_repeats() -> BTreeMap<String, f32> {
    let path = asset("textures/settlement/manifest.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let manifest: Manifest =
        ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    manifest
        .textures
        .into_iter()
        .map(|t| (t.name, t.repeat_m))
        .collect()
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
/// layout's east along the cell's side nearest east, and the terrace from
/// the natural ground (`record::lay`). What this returns is stored, and the
/// town is built from the store.
pub fn lay_out(site: &Site, template: &Template, config: &TerrainConfig) -> Result<Town, String> {
    let patch = patch_round(site.direction, config.radius_m, PATCH_M);
    let at = patch.nearest(site.direction).ok_or("an empty patch")?;
    let (_, east) = pbd_core::geo::north_east(patch.cells[at].direction);
    let d0 = patch.side_toward(at, east);
    record::lay(template, site.id, &patch, at, d0, natural(config))
}

/// Build a town from its definition: its ground and its pieces, cut on the
/// cells it was laid on. `repeat_m` is how far a texture repeats.
pub fn build(
    site: &Site,
    town: &Town,
    kits: &Kits,
    repeat_m: &dyn Fn(&str) -> f32,
    config: &TerrainConfig,
) -> Result<Laid, String> {
    let patch = patch_round(site.direction, config.radius_m, PATCH_M);
    let built = record::build(
        town,
        &patch,
        kits,
        repeat_m,
        config.radius_m,
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
pub fn ensure(
    save: &mut WorldSave,
    site: &Site,
    template: &Template,
    config: &TerrainConfig,
) -> Result<(Town, u64), String> {
    match stored(save, site.id) {
        Stored::Town(town) => Ok((town, 0)),
        Stored::Damaged(why) => Err(why),
        Stored::None => {
            let town = lay_out(site, template, config)?;
            let seq = store(save, &town).ok_or("the save refused the town")?;
            Ok((town, seq))
        }
    }
}

/// What the towns are built from: the kits, the templates and the textures.
#[derive(Resource)]
pub struct TownAssets {
    pub kits: Kits,
    pub village: Template,
    pub repeats: BTreeMap<String, f32>,
}

/// A town standing in the world.
#[derive(Clone, Debug)]
pub struct Standing {
    pub site: u32,
    pub name: String,
    pub anchor: Vec3,
    pub terrace_m: f32,
    pub entity: Entity,
}

/// The towns standing now, and the site list they were built for.
#[derive(Resource, Default)]
pub struct Towns {
    for_sites: Option<Vec<u32>>,
    /// A town laid and queued to the save, shown once the writer's mark
    /// passes its settlement's line (decision 8: written before it is
    /// shown).
    writing: Option<(Site, u64)>,
    pub standing: Vec<Standing>,
}

/// Put the walker back on the ground once the towns are built: a capture
/// that starts in a town (`--at`) stands on its terrace, not on the ground
/// that was there before it.
#[derive(Resource)]
pub struct RespawnInTown;

/// The root of a town's meshes.
#[derive(Component)]
pub struct TownRoot;

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
    let Some((site, seq)) = world.resource::<Towns>().writing.clone() else {
        return;
    };
    let Some(save) = world.get_resource::<WorldSave>() else {
        return;
    };
    if let Some(why) = save.failure() {
        error!("{} was not saved, so it is not shown: {why}", site.name);
        world.resource_mut::<Towns>().writing = None;
        return;
    }
    if save.committed() < seq {
        return;
    }
    world.resource_mut::<Towns>().writing = None;
    match stored(world.resource::<WorldSave>(), site.id) {
        Stored::Town(town) => {
            info!("{} is in the save", site.name);
            stand(world, &site, &town);
        }
        other => error!(
            "{} was written and does not read back: {other:?}",
            site.name
        ),
    }
}

/// A new site list: take down the old towns, and read or lay the home
/// village.
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
    {
        let mut towns = world.resource_mut::<Towns>();
        towns.for_sites = Some(ids);
        towns.writing = None;
        towns.standing.clear();
    }
    world.insert_resource(Structures::default());
    world.insert_resource(crate::planet::shadow::TownCasters::default());
    ground::install(None);
    let Some(home) = sites
        .iter()
        .find(|s| s.home && s.kind == SiteKind::Village)
        .cloned()
    else {
        return;
    };
    if !world.contains_resource::<WorldSave>() {
        warn!("{} is not built: there is no save to keep it in", home.name);
        return;
    }
    let config = *crate::planet::terrain_config();
    let made = world.resource_scope(|world, mut save: Mut<WorldSave>| {
        let template = &world.resource::<TownAssets>().village;
        ensure(&mut save, &home, template, &config)
    });
    match made {
        // Read or stored with nothing to wait for (a save with no disk
        // behind it): built from the records all the same.
        Ok((_, 0)) => match stored(world.resource::<WorldSave>(), home.id) {
            Stored::Town(town) => stand(world, &home, &town),
            other => error!("{} does not read from the save: {other:?}", home.name),
        },
        Ok((_, seq)) => {
            info!("{} laid out; shown once it is in the save", home.name);
            world.resource_mut::<Towns>().writing = Some((home, seq));
        }
        Err(why) => error!("{} is not built: {why}", home.name),
    }
}

/// Build a town from its record into the world: install its ground, rebuild
/// the planet round the player, spawn its meshes and give the walker its
/// solids.
fn stand(world: &mut World, site: &Site, town: &Town) {
    let started = std::time::Instant::now();
    let config = *crate::planet::terrain_config();
    let laid = {
        let assets = world.resource::<TownAssets>();
        let repeats = &assets.repeats;
        let repeat = |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0);
        build(site, town, &assets.kits, &repeat, &config)
    };
    let laid = match laid {
        Ok(laid) => laid,
        Err(why) => {
            warn!("{} could not be built: {why}", site.name);
            return;
        }
    };
    let (footprint, margin) = laid.ground.counts();
    ground::install(Some(Ground::new(config, vec![laid.ground.clone()])));
    // Each door as its save holds it: open where the player left it open,
    // shut where there is no record (slice 2b).
    let mut solids = laid.solids.clone();
    door_states(
        &mut solids,
        site.id,
        world.get_resource::<WorldSave>().map(|s| &s.records),
        world.contains_resource::<OpenDoors>(),
    );
    world.insert_resource(Structures(solids.clone()));
    let near = world
        .query_filtered::<&avian3d::prelude::Position, With<crate::walking::Walker>>()
        .iter(world)
        .next()
        .map(|p| p.0.normalize_or(Vec3::Y))
        .unwrap_or(site.direction);
    crate::planet::rebuild_planet(world, near);
    let entity = spawn_town(world, &laid);
    spawn_doors(world, entity, site.id, &solids);
    let triangles: usize = laid.meshes.values().map(|m| m.positions.len() / 3).sum();
    info!(
        "{} built: {} buildings, {triangles} triangles in {} textures, a terrace at {} m over {footprint} cells eased over {margin}, in {:.2} s",
        laid.name,
        town.buildings.len(),
        laid.meshes.len(),
        laid.terrace_m,
        started.elapsed().as_secs_f32()
    );
    world.resource_mut::<Towns>().standing.push(Standing {
        site: laid.site,
        name: laid.name.clone(),
        anchor: site.direction,
        terrace_m: laid.terrace_m,
        entity,
    });
    if world.contains_resource::<RespawnInTown>() {
        crate::walking::respawn(world);
    }
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
            let b = structures.0.get(door.building)?;
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
    let leaf = &mut structures.0[door.building].doors[door.door];
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
    assets: Res<TownAssets>,
    mut doors: Query<(&mut TownDoor, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(structures) = structures else {
        return;
    };
    let step = SWING_RAD_S * time.delta_secs().max(1.0 / 60.0);
    for (mut door, mesh) in &mut doors {
        let Some(b) = structures.0.get(door.building) else {
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
    let parts: Vec<(Mesh, Handle<Image>)> = {
        let assets = world.resource::<AssetServer>();
        laid.meshes
            .iter()
            .map(|(name, buf)| (to_mesh(buf), texture(assets, name)))
            .collect()
    };
    // Each building's rooms, apart: they take the room's own share of the
    // sky, which its doors decide (`sun-shadows` decision 7).
    let rooms: Vec<(usize, Mesh, Handle<Image>)> = {
        let assets = world.resource::<AssetServer>();
        laid.rooms
            .iter()
            .enumerate()
            .flat_map(|(b, meshes)| {
                meshes
                    .iter()
                    .map(move |(name, buf)| (b, to_mesh(buf), texture(assets, name)))
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
        .map(|(mesh, image)| (None, mesh, image))
        .chain(
            rooms
                .into_iter()
                .map(|(b, mesh, image)| (Some(b), mesh, image)),
        );
    for (room, mesh, image) in pieces {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: Some(image),
                perceptual_roughness: 0.93,
                ..default()
            });
        let mut child = world.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()));
        if let Some(building) = room {
            child.insert((TownRoom { building }, SkyShare(ROOM_SKY_SHUT)));
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
/// and not its doors, which swing.
pub fn casting(laid: &Laid) -> Vec<[f32; 3]> {
    laid.meshes
        .values()
        .chain(laid.rooms.iter().flat_map(|m| m.values()))
        .flat_map(|m| m.positions.iter().copied())
        .collect()
}

/// The share of the sky a room takes with a door of its building open, and
/// with all of them shut (`sun-shadows` decision 7): round the day a room is
/// about a quarter as bright as the street, as the mockup's rooms are.
pub const ROOM_SKY_OPEN: f32 = 0.3;
pub const ROOM_SKY_SHUT: f32 = 0.2;

/// The rooms' share of the sky, open and shut, where a launch sets it
/// (`--room-sky`): for tuning against the captures without a rebuild.
#[derive(Resource, Clone, Copy, Debug)]
pub struct RoomSky {
    pub open: f32,
    pub shut: f32,
}

impl Default for RoomSky {
    fn default() -> Self {
        Self {
            open: ROOM_SKY_OPEN,
            shut: ROOM_SKY_SHUT,
        }
    }
}

/// A mesh of a building's rooms: its index in the walker's [`Structures`].
#[derive(Component, Clone, Copy, Debug)]
pub struct TownRoom {
    pub building: usize,
}

/// A room takes more of the sky while a door of its building stands open.
pub fn rooms_follow_doors(
    structures: Option<Res<Structures>>,
    sky: Option<Res<RoomSky>>,
    mut rooms: Query<(&TownRoom, &mut SkyShare)>,
) {
    let Some(structures) = structures else {
        return;
    };
    let sky = sky.map(|s| *s).unwrap_or_default();
    for (room, mut share) in &mut rooms {
        let open = structures
            .0
            .get(room.building)
            .is_some_and(|b| b.doors.iter().any(|d| d.open));
        let want = SkyShare(if open { sky.open } else { sky.shut });
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
            kits: load_kits(),
            village: load_template("village"),
            repeats: load_repeats(),
        })
        .init_resource::<Towns>()
        .add_systems(
            Update,
            (
                build_towns,
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
