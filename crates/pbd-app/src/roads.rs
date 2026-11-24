//! Roads in the game (`townsfolk` design 10, 10a and 11): the network the
//! world lays between towns, planned once and stored, and every road cell
//! stood near the viewer as a piece the walker meets.
//!
//! The plan is made in the loading thread (`loading::prepare`) after the
//! towns' ground is installed, so a world saved before roads gains them on
//! its next open, in place ([`prepare`]). Each road cell is cut by
//! `pbd_core::roads::piece` in chunks within [`STAND_M`] of the viewer,
//! drawn in the towns' textures and lit as the terrain is (`LitByField`,
//! `LitLikeTerrain`, baked per vertex as a town's outside is), faded in and
//! out as a town is. A cell the player has edited is never laid, however
//! late the edit came.

use crate::field_light::{BakedField, Faded, LitByField, LitLikeTerrain, UNBAKED, UNBAKED_BEFORE};
use crate::planet::PlanetRenderFrame;
use crate::saves::WorldSave;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::edits::Edits;
use pbd_core::missions::Kind;
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::{Author, Proposal, YieldSet};
use pbd_core::roads::{self, Ground, Road, RoadCell, RoadsConfig, Settlement, Surface, Water};
use pbd_core::settlement::ground;
use pbd_core::settlement::pieces::{BuildingSolids, MeshBuf, Meshes};
use pbd_core::sites::{Site, SiteKind};
use pbd_core::terrain::Material;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Metres from the viewer within which a road's chunk stands.
pub const STAND_M: f32 = 420.0;
/// Metres past which a standing chunk is dropped.
pub const DROP_M: f32 = 520.0;
/// Road cells to a chunk: one mesh per texture each.
pub const CHUNK_CELLS: usize = 24;

/// `assets/config/roads.ron`, checked.
pub fn load_config() -> RoadsConfig {
    let path = crate::paths::assets_dir().join("config/roads.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let cfg: RoadsConfig =
        ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    cfg.validate()
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    cfg
}

/// The roads' tunables, for the walker and the planner.
#[derive(Resource, Clone)]
pub struct RoadsSettings(pub Arc<RoadsConfig>);

impl Default for RoadsSettings {
    fn default() -> Self {
        Self(Arc::new(load_config()))
    }
}

/// One cell to lay: the road cell, what it is made of, and its neighbours
/// in its road (a bridge's rails run along the other sides).
#[derive(Clone, Debug, PartialEq)]
pub struct Laid {
    pub cell: RoadCell,
    pub surface: Surface,
    pub neighbours: Vec<u32>,
}

/// A run of road cells stood and dropped together.
#[derive(Clone, Debug)]
pub struct Chunk {
    pub cells: Vec<usize>,
    /// Its middle, a unit direction.
    pub centre: Vec3,
}

/// The open world's roads.
#[derive(Resource, Clone, Debug, Default)]
pub struct RoadNet {
    pub cells: Vec<Laid>,
    pub chunks: Vec<Chunk>,
    /// The save these were read from (`WorldSave::serial`).
    pub for_serial: u64,
}

impl RoadNet {
    /// The road cell nearest `direction` within `within_m` metres along the
    /// ground, for a road job (`townsfolk` task 8.5: a rockfall to clear, a
    /// trader to walk with): by distance, then by key.
    pub fn near(&self, direction: Vec3, within_m: f32, radius_m: f32) -> Option<&Laid> {
        self.cells
            .iter()
            .filter_map(|l| {
                let m = pbd_core::sites::arc_m(direction, roads::centre(l.cell.key)?, radius_m);
                (m <= within_m).then_some((m, l))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cell.key.cmp(&b.1.cell.key)))
            .map(|(_, l)| l)
    }

    /// Every road's cells, a cell laid once (the first road to reach it),
    /// and the player's own road cells over them; chunked along each road.
    pub fn of(roads: &[Road], built: &BTreeMap<u32, roads::BuiltCell>, serial: u64) -> Self {
        let mut seen = BTreeSet::new();
        let mut cells = Vec::new();
        let mut chunks = Vec::new();
        for road in roads {
            let mut chunk: Vec<usize> = Vec::new();
            for (k, cell) in road.cells.iter().enumerate() {
                if !seen.insert(cell.key) || built.contains_key(&cell.key) {
                    continue;
                }
                let neighbours = [k.checked_sub(1), Some(k + 1)]
                    .into_iter()
                    .flatten()
                    .filter_map(|j| road.cells.get(j).map(|c| c.key))
                    .collect();
                chunk.push(cells.len());
                cells.push(Laid {
                    cell: *cell,
                    surface: road.surface,
                    neighbours,
                });
                if chunk.len() == CHUNK_CELLS {
                    chunks.push(std::mem::take(&mut chunk));
                }
            }
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
        }
        let mut mine = Vec::new();
        for (&key, b) in built {
            mine.push(cells.len());
            // Every road cell beside it: a bridge's rails run only along its
            // open sides, so a wide bridge is railed outside (design 42m).
            let neighbours = if b.deck_m.is_some() {
                roads::neighbours(key)
                    .into_iter()
                    .filter(|n| built.contains_key(n) || seen.contains(n))
                    .collect()
            } else {
                Vec::new()
            };
            cells.push(Laid {
                cell: roads::built_cell(key, b),
                surface: b.surface,
                neighbours,
            });
            if mine.len() == CHUNK_CELLS {
                chunks.push(std::mem::take(&mut mine));
            }
        }
        if !mine.is_empty() {
            chunks.push(mine);
        }
        let chunks = chunks
            .into_iter()
            .map(|c| {
                let sum: Vec3 = c
                    .iter()
                    .filter_map(|&i| roads::centre(cells[i].cell.key))
                    .sum();
                Chunk {
                    centre: sum.normalize_or(Vec3::Y),
                    cells: c,
                }
            })
            .collect();
        Self {
            cells,
            chunks,
            for_serial: serial,
        }
    }
}

/// What a road needs to know of a cell of the world `save` holds: its
/// ground's top, its water, whether a town stands on it and whether the
/// player has changed it.
pub fn ground_of<'a>(
    config: &'a TerrainConfig,
    edits: &'a Edits,
    yields: &'a YieldSet,
    built: &'a BTreeMap<u32, roads::BuiltCell>,
) -> impl Fn(u32) -> Option<Ground> + Sync + 'a {
    move |key| {
        let d = roads::centre(key)?;
        let top = pbd_core::column::surface_m(config, d);
        let town = ground::cleared(config, d);
        let player =
            !edits.for_cell(key).is_empty() || yields.has_cell(key) || built.contains_key(&key);
        let water = if top < config.sea_level_m {
            Water::Sea
        } else if pbd_core::map::river_water(config, d) {
            let column = pbd_core::column::generate_solid(config, d);
            let from = pbd_core::column::layer_at(top)?;
            let depth = (from..pbd_core::column::LAYERS)
                .take_while(|&i| {
                    matches!(
                        column.material(i),
                        Material::Water | pbd_core::flowing_water!()
                    )
                })
                .count() as i32;
            if depth == 0 {
                Water::Dry
            } else {
                Water::River {
                    depth_m: depth,
                    surface_m: top as i32 + depth,
                }
            }
        } else {
            Water::Dry
        };
        Some(Ground {
            top_m: top as i32,
            water,
            town,
            player,
        })
    }
}

/// Where a road can be looked at, for a capture's `--at` and `--yaw`: its
/// middle, its first climb and its first bridge, each with the bearing
/// along the road (degrees right of east, as `--yaw` turns).
fn log_road(road: &Road) {
    // (latitude, longitude, yaw) of cell `k`, or nought where there is none.
    let spot = |k: Option<usize>| -> [f32; 3] {
        let at = || -> Option<[f32; 3]> {
            let k = k?;
            let c = roads::centre(road.cells[k].key)?;
            let (next, back) = match road.cells.get(k + 1) {
                Some(n) => (n, false),
                None => (road.cells.get(k.checked_sub(1)?)?, true),
            };
            let mut along = roads::centre(next.key)? - c;
            if back {
                along = -along;
            }
            let (north, east) = pbd_core::geo::north_east(c);
            // `--yaw` turns from the walker's default heading, which faces the
            // other way from this bearing (measured on the capture: 0 faced west).
            let bearing = (-along.dot(north)).atan2(along.dot(east)).to_degrees();
            let yaw = (bearing + 360.0).rem_euclid(360.0) - 180.0;
            let (lat, lon) = pbd_core::geo::lat_lon(c).degrees();
            Some([lat, lon, yaw])
        };
        at().unwrap_or_default()
    };
    let first = |f: &dyn Fn(&RoadCell) -> bool| road.cells.iter().position(f);
    let m = spot(Some(road.cells.len() / 2));
    let c = spot(first(&|c| c.up.is_some()));
    let bridge = first(&|c| matches!(c.kind, roads::Kind::Bridge { .. }));
    let b = spot(bridge);
    // The bank before it, at the deck's height: where to stand to walk on.
    let bank = spot(bridge.and_then(|k| k.checked_sub(1)));
    info!(
        "road {} to {} ({:?}, {} cells): middle --at {:.5} {:.5} --yaw {:.1}; first climb --at {:.5} {:.5} --yaw {:.1}; first bridge --at {:.5} {:.5} --yaw {:.1}; its bank --at {:.5} {:.5} --yaw {:.1}",
        road.from_site,
        road.to_site,
        road.surface,
        road.cells.len(),
        m[0],
        m[1],
        m[2],
        c[0],
        c[1],
        c[2],
        b[0],
        b[1],
        b[2],
        bank[0],
        bank[1],
        bank[2],
    );
}

/// How many threads the plan routes on: the machine's, less one for the
/// loading screen.
pub fn threads() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get().saturating_sub(1).max(1))
}

/// The settlements a world's roads join: the towns standing, by site, but
/// a cave town, whose anchor is under its mountain.
pub fn settlements(sites: &[Site]) -> Vec<Settlement> {
    let mut out: Vec<Settlement> = sites
        .iter()
        .filter(|s| s.kind != SiteKind::Cave)
        .map(|s| Settlement {
            site: s.id,
            direction: s.direction,
            large: s.kind == SiteKind::Walled,
        })
        .collect();
    out.sort_by_key(|s| s.site);
    out
}

/// The world's edits a plan's cuttings make (`townsfolk` design 32): each
/// cut cell's layers from its floor to its ground's top dug to air.
pub fn cut_edits(roads: &[Road]) -> Vec<pbd_core::edits::Edit> {
    roads
        .iter()
        .flat_map(|r| &r.cells)
        .flat_map(|cell| {
            roads::cut_layers(cell)
                .into_iter()
                .map(|layer| pbd_core::edits::Edit {
                    cell: cell.key,
                    layer,
                    material: Material::Air,
                })
        })
        .collect()
}

/// The author a world's roads are stored by (design 11).
pub fn author() -> Author {
    Author::World {
        process: "roads".into(),
        kind: roads::ROAD_NET_RECORD.into(),
        id: 0,
    }
}

/// A world's roads (design 11): the ones its save holds, or, the first time
/// a build that knows roads opens it, planned now from its towns and stored
/// as the world's, through the yield check. Called with the towns' ground
/// installed, so a road meets a town's terrace as it is. The sequence to
/// wait for comes back with the net (0 when nothing was written).
pub fn prepare(
    save: &mut WorldSave,
    towns: &[Site],
    config: &TerrainConfig,
    cfg: &RoadsConfig,
) -> (RoadNet, u64) {
    let mut seq = 0;
    if !roads::planned(&save.records) && !towns.is_empty() {
        let started = std::time::Instant::now();
        let built = roads::built(&save.records);
        let planned = {
            let ground = ground_of(config, &save.edits, &save.yields, &built);
            roads::plan(
                &settlements(towns),
                &ground,
                cfg,
                config.radius_m,
                threads(),
            )
        };
        for road in &planned {
            log_road(road);
        }
        let links = roads::links(&settlements(towns), cfg, config.radius_m).len();
        let cells: usize = planned.iter().map(|r| r.cells.len()).sum();
        let mut records: Vec<_> = planned.iter().map(roads::to_record).collect();
        let schema = records
            .iter()
            .map(|r| r.schema)
            .max()
            .unwrap_or(roads::RECORD_SCHEMA);
        records.push(roads::net_record(planned.len()));
        // A cutting's layers, dug by the world (`townsfolk` design 32).
        let edits = cut_edits(&planned);
        let (cut, filled) = planned
            .iter()
            .flat_map(|r| &r.cells)
            .fold((0, 0), |(c, f), cell| {
                (
                    c + usize::from(cell.cut_fill < 0),
                    f + usize::from(cell.cut_fill > 0),
                )
            });
        match save.propose(&author(), Proposal { edits, records }) {
            Ok(Some(s)) => {
                seq = s;
                save.note_record_kinds(&[
                    (roads::ROAD_RECORD, schema),
                    (roads::ROAD_NET_RECORD, roads::RECORD_SCHEMA),
                ]);
                info!(
                    "roads planned (version {}): {} of {links} links, {cells} cells ({cut} cut, {filled} filled), in {:.2} s",
                    roads::NET_VERSION,
                    planned.len(),
                    started.elapsed().as_secs_f32()
                );
            }
            Ok(None) => error!("the roads were not saved: the save has failed"),
            Err(why) => warn!("the roads were refused: {why:?}"),
        }
    }
    let stored = roads::stored(&save.records);
    let built = roads::built(&save.records);
    let net = RoadNet::of(&stored, &built, save.serial());
    // The lamps beside them near the towns (design 34), derived each load.
    let lamps = {
        let ground = ground_of(config, &save.edits, &save.yields, &built);
        roads::lamps(&stored, &settlements(towns), cfg, config.radius_m, &ground)
    };
    // No tree grows on a road: installed before the planet is built.
    roads::install_cleared(Some(
        roads::Cleared::new(net.cells.iter().map(|l| l.cell.key), config.radius_m)
            .with_lamps(lamps.iter().copied()),
    ));
    // The player's marks and road cells, for the terrain (`townsfolk`
    // design 39a and 43), before the planet is built too.
    paving::install(save, config.radius_m);
    info!(
        "{} roads read, {} cells to lay in {} chunks, {} lamps beside them",
        stored.len(),
        net.cells.len(),
        net.chunks.len(),
        lamps.len()
    );
    (net, seq)
}

// ---- Standing ------------------------------------------------------------------

/// The road pieces standing, for the walker: each cell's floor and rails,
/// and how fast it is walked.
#[derive(Resource, Default, Clone)]
pub struct RoadPieces {
    pub solids: Vec<BuildingSolids>,
    pub speed: Vec<f32>,
}

impl RoadPieces {
    /// How much faster than the ground a planet point's feet go: the road
    /// whose floor is under it within `hold_m`, or 1.
    pub fn speed_at(&self, point: Vec3, hold_m: f32) -> f32 {
        for (b, &s) in self.solids.iter().zip(&self.speed) {
            let p = b.frame.local(point);
            if Vec2::new(p.x, p.z).length() > b.reach_m {
                continue;
            }
            if let (Some(floor), _) = b.stand_in(p, hold_m)
                && p.y - floor < hold_m
            {
                return s;
            }
        }
        1.0
    }
}

/// A chunk standing: its entity, its pieces' place in [`RoadPieces`] is
/// rebuilt whenever one comes or goes.
struct StandingChunk {
    chunk: usize,
    entity: Entity,
    solids: Vec<BuildingSolids>,
    speed: Vec<f32>,
    shown: f32,
    leaving: bool,
    /// The edits' revision it was cut at.
    revision: u64,
}

/// What stands.
#[derive(Resource, Default)]
pub struct RoadsStanding {
    chunks: Vec<StandingChunk>,
    /// The player has laid or taken up a road cell: the chunks from this
    /// one on (theirs) are stood again at once, drawn whole.
    pub redraw_from: Option<usize>,
}

/// A chunk's road root: moved with the planet's render frame.
#[derive(Component)]
pub struct RoadRoot;

fn to_mesh(buf: &MeshBuf) -> Mesh {
    let count = buf.positions.len() as u32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, buf.uvs.clone())
    // Baked by the field as a town's outside is, from the same layout, so
    // no new pipeline is made (`baked-light-ease` 7.4).
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_1, vec![UNBAKED; buf.positions.len()])
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![UNBAKED_BEFORE; buf.positions.len()],
    )
    .with_inserted_indices(Indices::U32((0..count).collect()))
}

/// Cut a chunk's cells, but any the player has touched (`player`: the
/// save's yield set, so a road's own cutting, the world's edit, does not
/// hide it): its meshes by texture, and its pieces with their speeds.
pub fn cut_chunk(
    net: &RoadNet,
    chunk: usize,
    player: &dyn Fn(u32) -> bool,
    cfg: &RoadsConfig,
    radius_m: f32,
    repeat_m: &dyn Fn(&str) -> f32,
) -> (Meshes, Vec<BuildingSolids>, Vec<f32>) {
    let mut meshes = Meshes::new();
    let mut solids = Vec::new();
    let mut speed = Vec::new();
    for &i in &net.chunks[chunk].cells {
        let laid = &net.cells[i];
        if player(laid.cell.key) {
            continue;
        }
        let Some(piece) = roads::piece::cut(
            &laid.cell,
            laid.surface,
            &laid.neighbours,
            radius_m,
            repeat_m,
        ) else {
            continue;
        };
        for (name, buf) in piece.meshes {
            let into = meshes.entry(name).or_default();
            into.positions.extend(buf.positions);
            into.normals.extend(buf.normals);
            into.uvs.extend(buf.uvs);
        }
        solids.push(piece.solids);
        speed.push(cfg.speed(laid.surface));
    }
    (meshes, solids, speed)
}

fn spawn_chunk(world: &mut World, meshes: &Meshes) -> Entity {
    let centre = world
        .get_resource::<PlanetRenderFrame>()
        .map_or(Vec3::ZERO, |f| f.center.as_vec3());
    let parts: Vec<(Mesh, Handle<Image>)> = {
        let assets = world.resource::<AssetServer>();
        meshes
            .iter()
            .filter(|(_, b)| !b.positions.is_empty())
            .map(|(name, buf)| (to_mesh(buf), crate::towns::texture(assets, name)))
            .collect()
    };
    let root = world
        .spawn((
            Name::new("Road"),
            RoadRoot,
            LitByField,
            LitLikeTerrain,
            Faded(1.0),
            Transform::from_translation(centre),
            Visibility::default(),
        ))
        .id();
    for (mesh, image) in parts {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = crate::towns::town_material(world, Some(image), false);
        let child = world
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
                BakedField::default(),
            ))
            .id();
        world.entity_mut(root).add_child(child);
    }
    root
}

/// The viewer in the planet's frame: the camera, or the walker.
pub(crate) fn viewer(world: &mut World) -> Option<Vec3> {
    let centre = world.get_resource::<PlanetRenderFrame>().map(|f| f.center);
    let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    if let Some(centre) = centre
        && let Some((_, at)) = cameras.iter(world).find(|(c, _)| c.is_active)
    {
        return Some((at.translation().as_dvec3() - centre).as_vec3());
    }
    let o = crate::sim_origin::origin_of(world);
    let mut walkers =
        world.query_filtered::<&avian3d::prelude::Position, With<crate::walking::Walker>>();
    walkers.iter(world).next().map(|p| o.query_point(p.0))
}

/// Stand the chunks within [`STAND_M`] of the viewer and drop those past
/// [`DROP_M`], fading each over the towns' fade time; re-cut a standing one
/// when the player's edits have moved. An exclusive system, as the towns'.
pub fn stand_roads(world: &mut World) {
    let Some(net) = world.get_resource::<RoadNet>().cloned() else {
        return;
    };
    let Some(save) = world.get_resource::<WorldSave>() else {
        return;
    };
    if net.for_serial != save.serial() {
        return;
    }
    let revision = save.edits.revision();
    let Some(at) = viewer(world) else {
        return;
    };
    let radius = crate::planet::terrain_config().radius_m;
    let dt = world.resource::<Time>().delta_secs();
    let cfg = world
        .get_resource_or_insert_with(RoadsSettings::default)
        .0
        .clone();
    let repeat = crate::towns::repeat_in(world);
    let (mut standing, redraw_from) = {
        let mut s = world.get_resource_or_insert_with(RoadsStanding::default);
        (std::mem::take(&mut s.chunks), s.redraw_from.take())
    };
    let mut changed = false;
    if let Some(from) = redraw_from {
        standing.retain(|s| {
            let again = s.chunk >= from;
            if again && let Ok(e) = world.get_entity_mut(s.entity) {
                e.despawn();
            }
            !again
        });
        changed = true;
    }
    let dist = |c: &Chunk| (c.centre * at.length()).distance(at);
    // leave, fade, re-cut
    for s in &mut standing {
        let d = dist(&net.chunks[s.chunk]);
        if !s.leaving && d > DROP_M {
            s.leaving = true;
        }
        s.shown = crate::towns::faded(s.shown, s.leaving, dt);
        if let Ok(mut e) = world.get_entity_mut(s.entity) {
            e.insert(Faded(1.0 - s.shown));
        }
    }
    standing.retain(|s| {
        let gone = s.leaving && s.shown <= 0.0;
        if gone {
            if let Ok(e) = world.get_entity_mut(s.entity) {
                e.despawn();
            }
            changed = true;
        }
        !gone
    });
    let yields = world.resource::<WorldSave>().yields.clone();
    // A rockfall dug off a road is the road again (`townsfolk` design 46).
    let dug_clear = crate::economy::road_jobs::dug_clear(world.resource::<WorldSave>());
    let player = |key: u32| yields.has_cell(key) && !dug_clear.contains(&key);
    for s in &mut standing {
        if s.revision == revision {
            continue;
        }
        s.revision = revision;
        let (meshes, solids, speed) = cut_chunk(&net, s.chunk, &player, &cfg, radius, &repeat);
        if solids.len() == s.solids.len() {
            continue;
        }
        if let Ok(e) = world.get_entity_mut(s.entity) {
            e.despawn();
        }
        s.entity = spawn_chunk(world, &meshes);
        world.entity_mut(s.entity).insert(Faded(1.0 - s.shown));
        s.solids = solids;
        s.speed = speed;
        changed = true;
    }
    let held: BTreeSet<usize> = standing.iter().map(|s| s.chunk).collect();
    for (k, c) in net.chunks.iter().enumerate() {
        if held.contains(&k) || dist(c) > STAND_M {
            continue;
        }
        let (meshes, solids, speed) = cut_chunk(&net, k, &player, &cfg, radius, &repeat);
        let entity = spawn_chunk(world, &meshes);
        standing.push(StandingChunk {
            chunk: k,
            entity,
            solids,
            speed,
            shown: if redraw_from.is_some_and(|f| k >= f) {
                1.0
            } else {
                0.0
            },
            leaving: false,
            revision,
        });
        changed = true;
    }
    if changed {
        let mut pieces = RoadPieces::default();
        for s in &standing {
            pieces.solids.extend(s.solids.iter().cloned());
            pieces.speed.extend(s.speed.iter().copied());
        }
        world.insert_resource(pieces);
    }
    world.resource_mut::<RoadsStanding>().chunks = standing;
}

/// Take every chunk down: a world switched for another.
pub fn replace(world: &mut World, net: Option<RoadNet>) {
    let old = std::mem::take(
        &mut world
            .get_resource_or_insert_with(RoadsStanding::default)
            .chunks,
    );
    for s in old {
        if let Ok(e) = world.get_entity_mut(s.entity) {
            e.despawn();
        }
    }
    world.insert_resource(RoadPieces::default());
    match net {
        Some(net) => world.insert_resource(net),
        None => {
            world.remove_resource::<RoadNet>();
        }
    }
}

/// A world opened without the loading thread (its sites and towns made
/// while it runs): its roads read or planned once its towns are settled.
pub fn keep_roads(world: &mut World) {
    let Some(serial) = world.get_resource::<WorldSave>().map(WorldSave::serial) else {
        return;
    };
    if world
        .get_resource::<RoadNet>()
        .is_some_and(|n| n.for_serial == serial)
    {
        return;
    }
    let Some(towns) = world.get_resource::<crate::towns::Towns>() else {
        return;
    };
    if towns.held.is_empty() || !towns.settled() {
        return;
    }
    let sites: Vec<Site> = towns.held.iter().map(|h| h.site.clone()).collect();
    let config = *crate::planet::terrain_config();
    let cfg = world
        .get_resource_or_insert_with(RoadsSettings::default)
        .0
        .clone();
    let (net, _) = world
        .resource_scope(|_, mut save: Mut<WorldSave>| prepare(&mut save, &sites, &config, &cfg));
    replace(world, Some(net));
}

fn follow_frame(frame: Res<PlanetRenderFrame>, mut roots: Query<&mut Transform, With<RoadRoot>>) {
    let centre = frame.center.as_vec3();
    for mut t in &mut roots {
        if t.translation != centre {
            t.translation = centre;
        }
    }
}

// ---- Roads the player lays (`townsfolk` design 31) ---------------------------

/// Why a road cell was not laid or taken up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayRefused {
    /// The aim is not on a cell's top.
    NotATop,
    /// Water, the sea, or a town's footprint.
    NotGround,
    /// A road is there already.
    Road,
    /// No gravel in the selected slot.
    NoGravel,
    /// No room in the pack for the gravel taken up.
    PackFull,
    /// The save refused the write.
    Unsaved,
}

/// What a press did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lay {
    Laid(u32, Surface),
    TakenUp(u32),
}

/// A road cell taken up: its record with no cell in it, which
/// [`roads::built`] reads as none.
pub fn taken_up_record(key: u32) -> pbd_core::records::Record {
    pbd_core::records::Record::of(
        roads::ROAD_CELL_RECORD,
        u64::from(key),
        roads::RECORD_SCHEMA,
        &Option::<roads::BuiltCell>::None,
    )
}

/// Lay a road cell of `surface` on the cell whose top `point` (body-local,
/// planet-centred metres) is on, rising toward the neighbour faced first
/// along `facing` that stands a layer up; or, with `surface` `None`, take
/// up the player's own road cell there. Gravel is taken from (and given
/// back to) `slots`; the record and the pack are written in one line.
pub fn lay(
    save: &mut WorldSave,
    slots: &mut pbd_core::inventory::Slots,
    net: &RoadNet,
    config: &TerrainConfig,
    point: Vec3,
    facing: Vec3,
    surface: Option<Surface>,
) -> Result<Lay, LayRefused> {
    use pbd_core::goods::Good;
    use pbd_core::inventory::Item;
    let d = point.normalize_or(Vec3::Y);
    let alt = point.length() - config.radius_m;
    let base_m = alt.round() as i32;
    if (alt - base_m as f32).abs() > 0.08 {
        return Err(LayRefused::NotATop);
    }
    let key = roads::nearest_key(d).ok_or(LayRefused::NotATop)?;
    let built = roads::built(&save.records);
    let gravel = Item::Good(Good::GRAVEL);
    let mut hands = slots.clone();
    let Some(surface) = surface else {
        let Some(cell) = built.get(&key) else {
            return Err(LayRefused::Road);
        };
        if cell.surface == Surface::Gravel && hands.give(gravel, 1) != 0 {
            return Err(LayRefused::PackFull);
        }
        if !save.store_with_hands(taken_up_record(key), roads::RECORD_SCHEMA, &hands) {
            return Err(LayRefused::Unsaved);
        }
        *slots = hands;
        info!("road cell {key} taken up");
        return Ok(Lay::TakenUp(key));
    };
    if built.contains_key(&key) || net.cells.iter().any(|l| l.cell.key == key) {
        return Err(LayRefused::Road);
    }
    let centre = roads::centre(key).ok_or(LayRefused::NotATop)?;
    if ground::cleared(config, centre) || base_m < config.sea_level_m.ceil() as i32 {
        return Err(LayRefused::NotGround);
    }
    let column = pbd_core::column::generate_solid(config, centre);
    let over = |m: i32| pbd_core::column::layer_at(m as f32 + 0.5).map(|i| column.material(i));
    // Dry: no water standing over the generated ground at this height.
    if over(base_m).is_some_and(|m| m.is_water()) {
        return Err(LayRefused::NotGround);
    }
    if surface == Surface::Gravel && !hands.take_item(gravel, 1) {
        return Err(LayRefused::NoGravel);
    }
    // The slope: toward the neighbour faced first that stands a layer up,
    // its road's top or its ground's.
    let height = |n: u32| -> i32 {
        if let Some(b) = built.get(&n) {
            return b.base_m;
        }
        if let Some(l) = net.cells.iter().find(|l| l.cell.key == n) {
            return l.cell.low_m();
        }
        roads::centre(n).map_or(i32::MIN, |c| {
            pbd_core::column::surface_m(config, c).floor() as i32
        })
    };
    let mut toward: Vec<(f32, u32, i32)> = roads::neighbours(key)
        .into_iter()
        .filter_map(|n| Some((-(roads::centre(n)? - centre).dot(facing), n, height(n))))
        .collect();
    toward.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let toward: Vec<(u32, i32)> = toward.into_iter().map(|(_, n, h)| (n, h)).collect();
    let cell = roads::BuiltCell {
        surface,
        base_m,
        up: roads::built_up(base_m, &toward),
        from: None,
        raised: false,
        plan: None,
        ford: false,
        bed: Default::default(),
        deck_m: None,
    };
    if !save.store_with_hands(
        roads::built_record(key, &cell),
        roads::RECORD_SCHEMA,
        &hands,
    ) {
        return Err(LayRefused::Unsaved);
    }
    save.note_record_kinds(&[(roads::ROAD_CELL_RECORD, roads::RECORD_SCHEMA)]);
    *slots = hands;
    info!("road cell {key} laid in {surface:?} at {base_m} m");
    Ok(Lay::Laid(key, surface))
}

/// A plain line for the player from the roads (a refused stake, "Needs
/// gravel"): the desktop shows it as the farm's note.
#[derive(Resource, Default)]
pub struct RoadSay(pub Option<String>);

/// The player's hands on the roads (`townsfolk` design 39 to 42l):
/// - survey stakes in the selected slot make the hand the road planner
///   ([`planner`]), which has the place button then;
/// - the interact key with the shovel in hand paves the marked cell aimed
///   at with one gravel, and held, the next marked cell in reach on the
///   shovel's beat;
/// - the place button with the shovel and an empty slot takes up the
///   player's own road cell, its gravel back. The place button no longer
///   lays a road.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn road_hands(
    (buttons, keys, aimed, walking, menu): (
        Option<Res<ButtonInput<MouseButton>>>,
        Option<ResMut<ButtonInput<KeyCode>>>,
        Option<Res<crate::husbandry::AimedGround>>,
        Option<Res<crate::walking::WalkingReadout>>,
        Option<Res<crate::controls::MenuOpen>>,
    ),
    (tools, hotbar, save, net): (
        Option<Res<crate::fish::ToolSlot>>,
        Option<ResMut<crate::hotbar::Hotbar>>,
        Option<ResMut<WorldSave>>,
        Option<ResMut<RoadNet>>,
    ),
    (settings, pcfg, mut standing, mut say, mut work): (
        Res<RoadsSettings>,
        Res<paving::PlannerSettings>,
        ResMut<RoadsStanding>,
        ResMut<RoadSay>,
        MessageWriter<paving::RoadWork>,
    ),
    mut did: MessageWriter<crate::missions::Did>,
    (mut fine, mut contact): (
        Option<ResMut<crate::planet::PlanetFine>>,
        Option<ResMut<crate::planet::PlanetContact>>,
    ),
    time: Res<Time>,
    mut beat: Local<f32>,
) {
    let (
        Some(buttons),
        Some(mut keys),
        Some(aimed),
        Some(mut hotbar),
        Some(mut save),
        Some(mut net),
    ) = (buttons, keys, aimed, hotbar, save, net)
    else {
        return;
    };
    let stake = pbd_core::inventory::Item::Good(pbd_core::goods::Good::SURVEY_STAKES);
    let held = hotbar.0.held().map(|s| s.item);
    if menu.is_some_and(|m| m.0) || !walking.is_some_and(|w| w.active && w.captured) {
        *beat = 0.0;
        return;
    }
    let Some(point) = aimed.0 else {
        *beat = 0.0;
        return;
    };
    let shovel = tools.is_some_and(|t| t.0.held() == Some(pbd_core::inventory::Tool::Shovel));
    let config = *crate::planet::terrain_config();
    let cfg = settings.0.clone();
    let key = roads::nearest_key(point.normalize_or(Vec3::Y));
    if buttons.just_pressed(MouseButton::Right) {
        if held == Some(stake) {
            // The road planner's.
            return;
        }
        let mine = key.is_some_and(|k| roads::built(&save.records).contains_key(&k));
        if held.is_none() && shovel && mine {
            match lay(
                &mut save,
                &mut hotbar.0,
                &net,
                &config,
                point,
                Vec3::Z,
                None,
            ) {
                Ok(_) => paving::restand(&save, &mut net, &mut standing),
                Err(why) => info!("ROAD refused: {why:?}"),
            }
        }
        return;
    }
    if !keys.pressed(KeyCode::KeyF) {
        *beat = 0.0;
        return;
    }
    let pressed = keys.just_pressed(KeyCode::KeyF);
    let aimed_marked = key.filter(|&k| paving::marked_near(&save, point, 1.5).contains(&k));
    // The use key with the shovel on a paved cell whose lamp post waits
    // for its log and lantern builds it (design 42m).
    let mut lamps = |save: &mut WorldSave,
                     slots: &mut pbd_core::inventory::Slots,
                     at: u32,
                     say: &mut RoadSay| {
        match paving::build_lamps(save, slots, at, &config, &cfg) {
            Ok(made) => {
                if let (Some(f), Some(c)) = (fine.as_deref_mut(), contact.as_deref_mut()) {
                    crate::economy::road_jobs::into_tier(&made, save, f, c);
                }
            }
            Err(why) => {
                if let Some(line) = why.line() {
                    say.0 = Some(line.into());
                }
            }
        }
    };
    if pressed && shovel && aimed_marked.is_none() {
        if let Some(k) = key
            && roads::built(&save.records).contains_key(&k)
            && !paving::lamps_owed(&save, k, &config, &cfg).is_empty()
        {
            keys.clear_just_pressed(KeyCode::KeyF);
            lamps(&mut save, &mut hotbar.0, k, &mut say);
        }
        return;
    }
    if pressed {
        if aimed_marked.is_none() {
            return;
        }
        keys.clear_just_pressed(KeyCode::KeyF);
        if !shovel {
            say.0 = Some(pbd_core::locale::tr("roads.pave.needs_shovel").into());
            return;
        }
        *beat = 0.0;
    } else {
        if !shovel {
            return;
        }
        *beat += time.delta_secs();
        if *beat < cfg.pave_beat_s {
            return;
        }
        *beat -= cfg.pave_beat_s;
    }
    if !shovel {
        return;
    }
    // The aimed cell first, else the nearest marked cell in reach.
    let Some(target) = aimed_marked.or_else(|| {
        paving::marked_near(&save, point, PAVE_REACH_M)
            .first()
            .copied()
    }) else {
        return;
    };
    match paving::pave(&mut save, &mut hotbar.0, target, &config, &cfg, &pcfg.0) {
        Ok(_) => {
            work.write(paving::RoadWork::Pave);
            did.write(crate::missions::Did::new(Kind::Pave, "gravel", 1));
            // A paved cell beside a town's footprint joins the road to it
            // (`town-missions-tree` design 2, "Join a town").
            let beside_town = roads::neighbours(target)
                .into_iter()
                .any(|n| roads::centre(n).is_some_and(|c| ground::cleared(&config, c)));
            if beside_town {
                did.write(crate::missions::Did::new(Kind::Pave, "joined", 1));
            }
            if !paving::lamps_owed(&save, target, &config, &cfg).is_empty() {
                lamps(&mut save, &mut hotbar.0, target, &mut say);
            }
            paving::restand(&save, &mut net, &mut standing);
        }
        Err(why) => {
            info!("PAVE refused: {why:?}");
            if let Some(line) = why.line() {
                say.0 = Some(line.into());
            }
        }
    }
}

/// How far from the aim a held interact key reaches for the next marked
/// cell to pave, m (design 40).
pub const PAVE_REACH_M: f32 = 4.0;

/// The dig press on a survey stake takes it back into the pack (design
/// 39), whatever is in hand, and is spent: the ground under it is not dug.
/// Runs before the digging.
#[allow(clippy::type_complexity)]
pub fn take_stakes(
    (buttons, aimed, walking, menu): (
        Option<ResMut<ButtonInput<MouseButton>>>,
        Option<Res<crate::husbandry::AimedGround>>,
        Option<Res<crate::walking::WalkingReadout>>,
        Option<Res<crate::controls::MenuOpen>>,
    ),
    (hotbar, save, settings, pcfg, mut say): (
        Option<ResMut<crate::hotbar::Hotbar>>,
        Option<ResMut<WorldSave>>,
        Res<RoadsSettings>,
        Res<paving::PlannerSettings>,
        ResMut<RoadSay>,
    ),
) {
    let (Some(mut buttons), Some(aimed), Some(mut hotbar), Some(mut save)) =
        (buttons, aimed, hotbar, save)
    else {
        return;
    };
    if !buttons.just_pressed(MouseButton::Left)
        || menu.is_some_and(|m| m.0)
        || !walking.is_some_and(|w| w.active && w.captured)
    {
        return;
    }
    let Some(key) = aimed
        .0
        .and_then(|p| roads::nearest_key(p.normalize_or(Vec3::Y)))
    else {
        return;
    };
    if !paving::stakes(&save).contains(&key) {
        return;
    }
    let config = *crate::planet::terrain_config();
    match paving::take_stake(&mut save, &mut hotbar.0, key, &config, &settings.0, &pcfg.0) {
        Ok(true) => buttons.reset(MouseButton::Left),
        Ok(false) => {}
        Err(why) => {
            // A refused take-back is spent too: the stake stays, not dug.
            buttons.reset(MouseButton::Left);
            info!("STAKE take-back refused: {why:?}");
            if let Some(line) = why.line() {
                say.0 = Some(line.into());
            }
        }
    }
}

/// The roads.
pub struct RoadsPlugin;

impl Plugin for RoadsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoadsSettings>()
            .init_resource::<RoadPieces>()
            .init_resource::<RoadsStanding>()
            .init_resource::<RoadSay>()
            .init_resource::<paving::MarksShown>()
            .init_resource::<paving::PlannerSettings>()
            .init_resource::<planner::Planner>()
            .init_resource::<planner::PlannerMenu>()
            .init_resource::<planner::PlannerAim>()
            .init_resource::<planner::Ghost>()
            .init_resource::<planner::PlannerLift>()
            .add_message::<paving::RoadWork>()
            .add_systems(
                Startup,
                (
                    paving::make_looks,
                    planner::make_ghost_looks,
                    planner::spawn_hud,
                ),
            )
            .add_systems(
                Update,
                (
                    keep_roads,
                    planner::ease_lift,
                    planner::planner_menu,
                    planner::planner_aim,
                    planner::planner_hands,
                    road_hands,
                    stand_roads,
                    paving::stand_stakes,
                    paving::follow_marks,
                    follow_frame,
                    planner::draw_ghost,
                    planner::follow_ghost,
                    planner::show_hud,
                )
                    .chain()
                    .after(crate::towns::build_towns),
            );
    }
}

pub mod models;
pub mod paving;
pub mod planner;

#[cfg(test)]
mod tests;
