//! The towns in the world (`cities-in-the-world`, slice 1): a settlement
//! built into the ground at its site and drawn with the mockup's own pieces
//! and pixels.
//!
//! Once a world's sites are on disk, the home town's template is laid onto
//! the finest cells round its anchor (`pbd_core::settlement::chart`), the
//! ground under it is terraced (`settlement::ground`, installed where every
//! height is read, and the planet rebuilt round the player), and its
//! buildings are cut from their cells' real corners
//! (`settlement::pieces`). The town is one entity with a mesh per texture,
//! lit by the field as a drop or a craft is.
//!
//! Slice 1 builds the home village only, from its template each time. No
//! build with towns ships before settlements are stored records (the
//! design's decision 9).

use crate::field_light::LitByField;
use crate::planet::PlanetRenderFrame;
use crate::planet::lattice::Lattice;
use crate::sites::WorldSites;
use crate::walking::Structures;
use bevy::asset::RenderAssetUsages;
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::planet_gen::{self, TerrainConfig};
use pbd_core::settlement::chart::{Chart, Patch, chart};
use pbd_core::settlement::ground::{self, Ground, TownGround};
use pbd_core::settlement::pieces::{BuildingSolids, Meshes, cut_building};
use pbd_core::settlement::{Kits, Template, neighbour};
use pbd_core::sites::{Site, SiteKind};
use pbd_core::terrain::Material;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// How far round a site its cells are fetched, metres: the village's grid
/// reaches 82 m from its centre, and the margin rings a few more.
pub const PATCH_M: f32 = 130.0;
/// Rings of layout cells round the built ones that are terraced with them,
/// so the yards between the houses are level too.
pub const YARD_RINGS: usize = 2;

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

/// A town laid out and cut, before it is in the world.
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
}

/// The layout cell the template is anchored by: the middle of what it
/// builds on.
pub fn template_anchor(template: &Template) -> (i32, i32) {
    let built = template.built_cells();
    let (mut lo, mut hi) = ([i32::MAX; 2], [i32::MIN; 2]);
    for [c, r] in &built {
        lo = [lo[0].min(*c), lo[1].min(*r)];
        hi = [hi[0].max(*c), hi[1].max(*r)];
    }
    ((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2)
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

/// Lay a template at a site: chart it, terrace its ground, cut its
/// buildings. `repeat_m` is how far a texture repeats.
pub fn lay_out(
    site: &Site,
    template: &Template,
    kits: &Kits,
    repeat_m: &dyn Fn(&str) -> f32,
    config: &TerrainConfig,
) -> Result<Laid, String> {
    let radius = config.radius_m;
    let patch = patch_round(site.direction, radius, PATCH_M);
    let at = patch.nearest(site.direction).ok_or("an empty patch")?;
    let (_, east) = pbd_core::geo::north_east(patch.cells[at].direction);
    let d0 = patch.side_toward(at, east);
    let wanted: BTreeSet<(i32, i32)> = template.ground.iter().map(|g| (g.c, g.r)).collect();
    let anchor = template_anchor(template);
    let chart = chart(&patch, anchor, at, d0, &wanted)?;
    // The footprint: what is built on, and the yards round it.
    let built: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    let mut footprint = built.clone();
    let mut ring = built.clone();
    for _ in 0..YARD_RINGS {
        let mut next = BTreeSet::new();
        for &(c, r) in &ring {
            for d in 0..6 {
                let n = neighbour(c, r, d);
                if chart.cells.contains_key(&n) && footprint.insert(n) {
                    next.insert(n);
                }
            }
        }
        ring = next;
    }
    let natural = |d: Vec3| planet_gen::surface_altitude(config, d).floor();
    let mut heights: Vec<f32> = built
        .iter()
        .filter_map(|&(c, r)| chart.cell(c, r))
        .map(|i| natural(patch.cells[i].direction))
        .collect();
    heights.sort_by(f32::total_cmp);
    let terrace_m = *heights.get(heights.len() / 2).ok_or("nothing built")?;
    let dirt: BTreeSet<(i32, i32)> = template
        .ground
        .iter()
        .filter(|g| g.top != "grass" && g.top != "sand")
        .map(|g| (g.c, g.r))
        .collect();
    let cells: Vec<(usize, Option<Material>)> = footprint
        .iter()
        .filter_map(|&(c, r)| {
            let top = dirt.contains(&(c, r)).then_some(Material::Dirt);
            chart.cell(c, r).map(|i| (i, top))
        })
        .collect();
    let ground = TownGround::new(&patch, radius, &cells, terrace_m, natural);
    // The mockup's datum: the level most of its village stands on.
    let mut levels: Vec<i32> = template
        .ground
        .iter()
        .filter(|g| built.contains(&(g.c, g.r)))
        .map(|g| g.h)
        .collect();
    levels.sort();
    let datum = levels.get(levels.len() / 2).copied().unwrap_or(0);
    let mut meshes = Meshes::new();
    let mut solids = Vec::new();
    for b in &template.buildings {
        let kit = kits
            .get(&b.kit)
            .ok_or_else(|| format!("{}: no kit {}", b.name, b.kit))?;
        solids.push(cut_building(
            &mut meshes,
            repeat_m,
            &patch,
            &chart,
            b,
            kit,
            radius,
            terrace_m + (b.base - datum) as f32,
        )?);
    }
    Ok(Laid {
        site: site.id,
        name: site.name.clone(),
        chart,
        patch,
        terrace_m,
        ground,
        meshes,
        solids,
    })
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
pub fn build_towns(world: &mut World) {
    let Some(sites) = world
        .get_resource::<WorldSites>()
        .and_then(|w| w.ready())
        .map(<[Site]>::to_vec)
    else {
        return;
    };
    let ids: Vec<u32> = sites.iter().map(|s| s.id).collect();
    if world.resource::<Towns>().for_sites.as_ref() == Some(&ids) {
        return;
    }
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
        towns.standing.clear();
    }
    world.insert_resource(Structures::default());
    let Some(home) = sites
        .iter()
        .find(|s| s.home && s.kind == SiteKind::Village)
        .cloned()
    else {
        ground::install(None);
        return;
    };
    let started = std::time::Instant::now();
    let config = *crate::planet::terrain_config();
    let laid = {
        let assets = world.resource::<TownAssets>();
        let repeats = &assets.repeats;
        let repeat = |m: &str| repeats.get(m).copied().filter(|r| *r > 0.0).unwrap_or(2.0);
        lay_out(&home, &assets.village, &assets.kits, &repeat, &config)
    };
    let laid = match laid {
        Ok(laid) => laid,
        Err(why) => {
            warn!("{} could not be built: {why}", home.name);
            return;
        }
    };
    let (footprint, margin) = laid.ground.counts();
    ground::install(Some(Ground::new(config, vec![laid.ground.clone()])));
    world.insert_resource(Structures(laid.solids.clone()));
    let near = world
        .query_filtered::<&avian3d::prelude::Position, With<crate::walking::Walker>>()
        .iter(world)
        .next()
        .map(|p| p.0.normalize_or(Vec3::Y))
        .unwrap_or(home.direction);
    crate::planet::rebuild_planet(world, near);
    let entity = spawn_town(world, &laid);
    let triangles: usize = laid.meshes.values().map(|m| m.positions.len() / 3).sum();
    info!(
        "{} built: {} buildings, {triangles} triangles in {} textures, a terrace at {} m over {footprint} cells eased over {margin}, in {:.2} s",
        laid.name,
        world.resource::<TownAssets>().village.buildings.len(),
        laid.meshes.len(),
        laid.terrace_m,
        started.elapsed().as_secs_f32()
    );
    world.resource_mut::<Towns>().standing.push(Standing {
        site: laid.site,
        name: laid.name.clone(),
        anchor: home.direction,
        terrace_m: laid.terrace_m,
        entity,
    });
    if world.contains_resource::<RespawnInTown>() {
        crate::walking::respawn(world);
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
            .map(|(name, buf)| {
                let count = buf.positions.len() as u32;
                let mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions.clone())
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals.clone())
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, buf.uvs.clone())
                .with_inserted_indices(Indices::U32((0..count).collect()));
                (mesh, texture(assets, name))
            })
            .collect()
    };
    let root = world
        .spawn((
            Name::new(format!("Town: {}", laid.name)),
            TownRoot,
            LitByField,
            Transform::from_translation(centre),
            Visibility::default(),
        ))
        .id();
    for (mesh, image) in parts {
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: Some(image),
                perceptual_roughness: 0.93,
                ..default()
            });
        let child = world
            .spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::default()))
            .id();
        world.entity_mut(root).add_child(child);
    }
    root
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
        .add_systems(Update, (build_towns, follow_frame).chain());
    }
}

#[cfg(test)]
mod tests;
