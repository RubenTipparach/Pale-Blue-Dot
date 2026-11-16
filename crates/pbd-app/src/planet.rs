//! A complete, closed hex/pentagon world drawn from persistent GPU topology.
//!
//! CPU columns are authoritative. A compute pass compacts visible column IDs
//! and writes an indirect draw; the vertex shader reconstructs caps, exposed
//! walls, and decorative trees. No expanded terrain vertex buffer is uploaded.
//! This is a surface-column prototype, not the editable volumetric chunk engine.

#[path = "planet_cloud_noise.rs"]
mod cloud_noise;
#[path = "planet_cloud_puffs.rs"]
mod cloud_puffs;
#[path = "planet_cloud_timer.rs"]
mod cloud_timer;
pub use cloud_timer::cloud_gpu_ms;
#[path = "planet_bounce.rs"]
pub mod bounce;
#[path = "planet_column.rs"]
pub mod column;
pub use column::mouth_of;
#[path = "planet_contact.rs"]
mod contact;
#[path = "planet_lattice.rs"]
pub(crate) mod lattice;
#[path = "planet_lod.rs"]
pub(crate) mod lod;
#[path = "planet_reflect.rs"]
mod reflect;
#[path = "planet_shadow.rs"]
pub mod shadow;
#[path = "planet_splash.rs"]
pub mod splash;
#[path = "planet_ssr.rs"]
mod ssr;
#[path = "planet_terrain.rs"]
pub(crate) mod terrain;
#[path = "planet_topology.rs"]
pub(crate) mod topology;
#[path = "planet_trees.rs"]
pub(crate) mod trees;
#[cfg(test)]
#[path = "planet_visibility_tests.rs"]
mod visibility_tests;
#[path = "planet_water.rs"]
mod water;
#[path = "planet_weather.rs"]
pub mod weather_maps;

pub use contact::{PlanetContact, SurfaceContact};
pub use lod::{BAND_M, BASE_LEVEL, FINEST_LEVEL, LodRefresh, NearField, PlanetFine, tile_width_m};
pub use terrain::{
    DIRT, ELEVATION_STEP, GRASS_SIDE, PLANET_RADIUS, SNOW_SIDE, WATER, generator_version,
    nearest_ground_near, river_channel, snow_slot, surface_code, surface_height, switch_generator,
    terrain_config, terrain_epoch, terrain_radius, tileset_slot,
};
pub use water::{EyeWater, EyeWaterState, RainShadows, emerge, submersion};

use bevy::math::{DMat4, DVec3};
use bevy::{
    core_pipeline::core_3d::CORE_3D_DEPTH_FORMAT,
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        diagnostic::RecordDiagnostics,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_phase::TrackedRenderPass,
        render_resource::{
            binding_types::{
                storage_buffer_read_only_sized, storage_buffer_sized, texture_2d, uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        sync_world::SyncToRenderWorld,
        texture::GpuImage,
        view::{ExtractedView, ViewTarget},
    },
};
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, num::NonZeroU64, sync::Arc};

// Trees live on the finest tier, so they reach exactly as far as it does.
// Above the band's radius plus the highest summit and a tree, no tree can be
// in range, and the foliage draw is disabled outright.
// Trees stand on every fine level out to the coarsest band's edge, fading
// out across its ring (`distance-lod-fade`).
const FOLIAGE_DRAW_DISTANCE: f32 = lod::BAND_M[0];
const FOLIAGE_DRAW_CUTOFF_ALTITUDE: f32 = FOLIAGE_DRAW_DISTANCE + 600.;

/// Blades the clutter vertex budget covers. An indirect draw has ONE vertex
/// count for every instance, so this is the upper bound a cell can roll and
/// `ScatterSettings::validate` refuses a config that asks for more: a file that
/// said twenty and drew eighteen would be a disagreement nobody could see.
pub const GRASS_BLADE_BUDGET: u32 = 18;
/// Vertices per clutter instance, which is the whole of what one cell can grow:
/// the blades, then a flower, a pebble, a bush and a dead shrub. A piece a cell
/// did not roll collapses to a degenerate triangle, the way a tree part a cell
/// does not carry already does.
///
/// The shipping shader is the only consumer of these two, because an indirect
/// draw reads its vertex count out of the buffer the compute pass wrote. They
/// exist here so a test can hold that shader to the arithmetic, which is why
/// they are test-only: a second copy compiled into the binary and read by
/// nobody would be exactly the drift they are meant to catch.
#[cfg(test)]
pub(crate) fn clutter_vertices() -> u32 {
    // ...and `flora-v2`'s mushroom (a stem and a cap prism) and fern (four
    // fronds).
    GRASS_BLADE_BUDGET * 2 * 6 + 18 + 54 + 108 + 36 + 2 * 54 + 4 * 6
}
/// Vertices per tree in the impostor draw (`flora-v2` design 10.5): the mid
/// ring's four crossed cards, then the far billboard. The near model is
/// drawn by the near draws (`trees::near_vertices`).
#[cfg(test)]
pub(crate) fn foliage_vertices() -> u32 {
    4 * 6 + 6
}
/// Where the clutter branch starts in the shared vertex shader: after the
/// terrain's 60 and the foliage.
#[cfg(test)]
pub(crate) fn clutter_first_vertex() -> u32 {
    60 + foliage_vertices()
}
/// Bytes of indirect draw arguments the compute pass publishes: five draws of
/// four words each - terrain, foliage, water, clutter, the column tier. The
/// shader's `array<DrawArgs,5>` is the same number, and it is the binding's
/// minimum size, which is what a fifth draw added to only one of the two fails
/// on: a pipeline whose layout still says four is refused at creation with
/// "shader global binding 3 is not available", which names the binding and not
/// the draw that outgrew it.
///
/// Then eight more, the near trees, one per species (`flora-v2` design
/// 10.5), whose vertex counts and first vertices the CPU writes once when
/// the buffer is made (`near_args`) and the cull only counts instances into.
pub(crate) const INDIRECT_BYTES: u64 = 13 * 16;
/// Where the near draws' arguments start in the indirect buffer.
pub(crate) const NEAR_ARGS_OFFSET: u64 = 5 * 16;

/// The near draws' arguments, as the indirect buffer holds them after the
/// first five: per species its vertex count, no instances yet, its first
/// vertex (the species over bit 16) and no first instance.
pub(crate) fn near_args() -> Vec<u32> {
    trees::near_vertices()
        .iter()
        .enumerate()
        .flat_map(|(s, n)| [*n, 0, trees::NEAR_FIRST_VERTEX + ((s as u32) << 16), 0])
        .collect()
}
/// Bytes of the near-tree list: a region of `NEAR_CAP` entries per species.
pub(crate) const NEAR_LIST_BYTES: u64 = 8 * trees::NEAR_CAP as u64 * 4;

/// Vertices per column instance: a cave ceiling fan and a cave floor fan per
/// run, then one flank quad per side per run per stretch of the NEIGHBOUR's
/// air, since a column of `MAX_RUNS` runs has one more air gap than it has
/// runs. Same arrangement as the clutter budget above, and for the same reason.
#[cfg(test)]
pub(crate) fn column_vertices() -> u32 {
    let runs = pbd_core::column::MAX_RUNS as u32;
    // ...and a lamp: a torch's post and head, or two boxes of 36.
    runs * (18 + 18) + 6 * runs * (runs + 1) * 6 + 72
}
/// Where the column branch starts: after the clutter.
#[cfg(test)]
pub(crate) fn column_first_vertex() -> u32 {
    clutter_first_vertex() + clutter_vertices()
}

/// The preview body's centre in the translating render/physics frame. System
/// positions are subtracted in f64 before any bounded GPU coordinate is cast.
#[derive(Resource, Clone, Copy, Default, ExtractResource)]
pub(crate) struct PlanetRenderFrame {
    pub(crate) center: DVec3,
}

pub(crate) fn update_planet_frame(
    scene: Res<crate::CelestialScene>,
    frame: Res<crate::PhysicsFrame>,
    mut render_frame: ResMut<PlanetRenderFrame>,
) {
    if let Some(body) = scene.states.first() {
        render_frame.center = body.position - frame.0.origin;
    }
}

impl PlanetRenderFrame {
    fn camera_and_clip(
        self,
        world_from_view: &GlobalTransform,
        clip_from_view: Mat4,
        clip_from_world: Option<Mat4>,
    ) -> (Vec3, Mat4) {
        let camera = (world_from_view.translation().as_dvec3() - self.center).as_vec3();
        let clip_from_body = if let Some(clip) = clip_from_world {
            (clip.as_dmat4() * DMat4::from_translation(self.center)).as_mat4()
        } else {
            // Equivalent to clip_from_world * world_from_body, but invert the
            // bounded camera transform so large translations never enter f32
            // matrix multiplication before cancelling one another.
            let mut body_from_view = world_from_view.to_matrix();
            body_from_view.w_axis = camera.extend(1.);
            clip_from_view * body_from_view.inverse()
        };
        (camera, clip_from_body)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuCell {
    pub direction_height: [f32; 4],
    // xyz is the shared corner ray; w is this edge's adjacent surface height,
    // negative under the sea.
    pub corners: [[f32; 4]; 6],
    // Degree in the low byte and level above it, explicit biome, baked sky
    // occlusion (0..65535), stable cell ID.
    pub metadata: [u32; 4],
    // The level below's cells this one belongs to: itself when centred on a
    // coarse vertex, the two whose edge it bisects otherwise. w carries the
    // fine floor of sides 0 and 1: the height at that edge's midpoint.
    pub owner_a: [f32; 4],
    pub owner_b: [f32; 4],
    // Fine floors of sides 2 to 5.
    pub floors: [f32; 4],
    // x: the tree this cell stands, the stable id of the finest cell at its
    // centre (`distance-lod-fade`): a coarse cell's tree is the fine tree
    // standing where it stood. An integer lane, never float bits, which a GPU
    // may flush as denormals. y: the cell's exact key (`pbd_core::cell_key`),
    // which an edit is saved and found by; `NO_KEY` on the base level. zw
    // spare.
    pub spare: [u32; 4],
}

/// The key lane of a record that has no key: a base-level cell, which no
/// edit is ever made on. Bit 31 is set, which no key has.
pub const NO_KEY: u32 = u32::MAX;

impl GpuCell {
    /// The cell's exact key (`pbd_core::cell_key`): what its edits are saved
    /// and found by. `metadata[3]` is the old hash, kept only as the seed the
    /// shaders roll clutter and texture from; it collides, so nothing is
    /// looked up by it.
    pub fn key(&self) -> u32 {
        self.spare[1]
    }

    pub fn degree(&self) -> usize {
        (self.metadata[0] & 0xff) as usize
    }
}

// Match the WGSL Cell storage ABI. These inspect the actual upload type; the
// standalone shader validator separately inspects the parsed shader layouts.
const _: [(); 192] = [(); size_of::<GpuCell>()];
const _: [(); 16] = [(); std::mem::offset_of!(GpuCell, corners)];
const _: [(); 112] = [(); std::mem::offset_of!(GpuCell, metadata)];
const _: [(); 128] = [(); std::mem::offset_of!(GpuCell, owner_a)];
const _: [(); 160] = [(); std::mem::offset_of!(GpuCell, floors)];

/// The base records, and which build of them: a switch of generator builds
/// new ones, and the render world rewrites its buffer when the generation
/// moves (`bigger-biomes` decision 8).
#[derive(Resource, Clone, ExtractResource)]
struct PlanetBase {
    cells: Arc<Vec<GpuCell>>,
    generation: u64,
}

#[derive(Resource, Clone, Default, ExtractResource)]
pub(crate) struct PlanetClock(f32);

/// How the plant decoration is drawn, from the player's settings
/// (`flora-v2` design 1): the pixel tufts in the grass slots, or the blades.
#[derive(Resource, Clone, Copy, Default, ExtractResource)]
pub struct FloraLook {
    pub tufts: bool,
}

/// The per-biome flora tables as the shaders read them (`flora-v2` design
/// 6): fern weight, mushroom weight, flower colour bits and deciduous per
/// biome; then the tuft ramps as `0xRRGGBB`, w one for red caps; then the
/// plants' tile means and, in w, the fruit the trees bear (design 9) in the
/// low half and the open world's tree table in the high half (design 12.1).
fn flora_tables() -> ([Vec4; 8], [UVec4; 8], [UVec4; 8]) {
    use pbd_core::flora::{BIOME_FLORA, BIOME_FRUIT, PLANT_MEANS, TUFT_RAMPS};
    let biome = BIOME_FLORA.map(|b| {
        Vec4::new(
            b.fern,
            b.mushroom,
            b.flowers as f32,
            if b.deciduous { 1.0 } else { 0.0 },
        )
    });
    let mut ramps = [UVec4::ZERO; 8];
    for (i, ramp) in TUFT_RAMPS.iter().enumerate() {
        ramps[i] = UVec4::new(ramp[0], ramp[1], ramp[2], BIOME_FLORA[i].red_caps as u32);
    }
    let mut means = [UVec4::ZERO; 8];
    for (i, m) in PLANT_MEANS.iter().enumerate() {
        let trees = terrain_config().trees.packed(i);
        means[i] = UVec4::new(m[0], m[1], m[2], BIOME_FRUIT[i] | trees);
    }
    (biome, ramps, means)
}

#[derive(Resource, Clone, ExtractResource)]
pub struct PlanetArt(pub Handle<Image>);

#[derive(Component, Clone, Copy, ExtractComponent)]
struct PlanetSurface;

/// Installs CPU generation, persistent storage buffers and actual GPU pipelines.
pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        // The detailed trees' kit builds on a thread of its own behind the
        // loading screen (`flora-v2` design 10.2).
        trees::prepare();
        app.add_plugins((crate::sea::SeaPlugin, shadow::SunShadowPlugin));
        water::install_eye_water(app);
        app.add_plugins((
            ExtractResourcePlugin::<PlanetBase>::default(),
            ExtractResourcePlugin::<lod::PlanetFine>::default(),
            ExtractResourcePlugin::<PlanetClock>::default(),
            // The sun moves now, so the render world needs this frame's, not
            // the one the pipeline was built with.
            ExtractResourcePlugin::<crate::sky::Sun>::default(),
            ExtractResourcePlugin::<PlanetArt>::default(),
            ExtractResourcePlugin::<water::EyeWaterState>::default(),
            ExtractResourcePlugin::<water::RainShadows>::default(),
            ExtractResourcePlugin::<FloraLook>::default(),
            ExtractResourcePlugin::<PlanetRenderFrame>::default(),
            ExtractComponentPlugin::<PlanetSurface>::default(),
            ExtractResourcePlugin::<bounce::BounceField>::default(),
            ExtractResourcePlugin::<bounce::LampBakeField>::default(),
            ExtractResourcePlugin::<lod::DuskRelight>::default(),
        ))
        .add_plugins(crate::night_lights::NightLightsPlugin)
        // What players took from the trees near the camera (`flora-v2`
        // design 12.5): felled trees and picked fruit the draw leaves out.
        .add_plugins(ExtractResourcePlugin::<crate::trees_taken::TreesTaken>::default())
        .init_resource::<crate::trees_taken::TreesTaken>()
        .add_systems(Update, crate::trees_taken::refresh)
        .init_resource::<bounce::BounceField>()
        .init_resource::<bounce::LampBakeField>()
        .init_resource::<lod::DuskRelight>()
        // After the landing, which publishes the set's own bakes
        // (`baked-light-ease` 7.2): a bake begun before it would be of the
        // set it replaced.
        .add_systems(
            Update,
            (bounce::update_bounce, bounce::update_lamp_bake).after(lod::refresh_lod),
        )
        .init_resource::<PlanetClock>()
        .init_resource::<water::RainShadows>()
        .init_resource::<FloraLook>()
        .init_resource::<PlanetRenderFrame>()
        .init_resource::<lod::LodRefresh>()
        .init_resource::<lod::NearField>()
        .init_resource::<lod::DuskLamps>()
        .add_systems(Startup, create_planet)
        .add_systems(Update, (lod::switch_dusk_lamps, lod::refresh_lod).chain())
        .add_systems(
            PostUpdate,
            update_planet_frame.before(TransformSystems::Propagate),
        )
        .add_systems(Update, |time: Res<Time>, mut clock: ResMut<PlanetClock>| {
            clock.0 = time.elapsed_secs();
        });
        weather_maps::build(app);
        splash::build(app);
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<SpecializedRenderPipelines<PlanetPipeline>>()
            .add_systems(RenderStartup, initialize_pipelines)
            .add_systems(
                Render,
                (
                    (upload_planet, upload_fine)
                        .chain()
                        .in_set(RenderSystems::PrepareResources),
                    prepare_views.in_set(RenderSystems::PrepareBindGroups),
                    queue_planet.in_set(RenderSystems::QueueMeshes),
                ),
            );
        water::build(render_app);
        ssr::build(render_app);
        reflect::build(render_app);
        splash::build_render(render_app);
        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(PlanetComputeLabel, PlanetComputeNode::default());
        graph.add_node_edge(PlanetComputeLabel, bevy::render::graph::CameraDriverLabel);
    }
}

/// What the generator makes of the planet round a place: its base records,
/// the contact tier over them, and the fine set there with the world's
/// edits, built synchronously on every core so the walker has a tile to
/// stand on before its first tick. At launch, and when a load switches the
/// generator (`bigger-biomes` decision 8).
struct BuiltPlanet {
    cells: Vec<topology::DualCell>,
    base: Arc<Vec<GpuCell>>,
    contacts: PlanetContact,
    fine: Arc<lod::FineSet>,
    anchor: Vec3,
}

fn build_planet(
    near: Vec3,
    columns: &crate::config::ColumnSettings,
    scatter: &crate::config::ScatterSettings,
    edits: &pbd_core::edits::Edits,
) -> BuiltPlanet {
    let cells = topology::dual_sphere(lod::BASE_LEVEL as u32);
    let base = Arc::new(lod::base_records(&cells));
    let mut contacts = PlanetContact::new(base.clone(), &cells);
    let anchor = contacts.find_land_near(near);
    // With the configured cross-fade ring, like every set after it
    // (`distance-lod-fade`).
    let fine = Arc::new(lod::generate_fine_live(
        anchor,
        lod::BAND_M,
        lod::REGEN_DISTANCE_M,
        columns,
        edits,
        std::thread::available_parallelism().map_or(1, |n| n.get()),
        None,
        scatter.lod_fade_width,
    ));
    contacts.set_fine(&fine);
    BuiltPlanet {
        cells,
        base,
        contacts,
        fine,
        anchor,
    }
}

/// A planet built before `Startup`, on the loading thread, with the opened
/// world's towns' ground already installed (`loading-screen` decision 1).
/// `create_planet` takes it in place of building one, so the planet is built
/// once.
#[derive(Resource)]
pub struct PreparedPlanet(Option<BuiltPlanet>);

impl PreparedPlanet {
    /// What `create_planet` would build round `near`, built now, on this
    /// thread, with whatever ground is installed.
    pub fn build(
        near: Vec3,
        columns: &crate::config::ColumnSettings,
        scatter: &crate::config::ScatterSettings,
        edits: &pbd_core::edits::Edits,
    ) -> Self {
        Self(Some(build_planet(near, columns, scatter, edits)))
    }

    /// Where a new world's walker starts on this planet, from `direction`
    /// ([`crate::walking::new_world_start`]).
    pub fn new_world_start(&self, sea_radius: f32, direction: Vec3) -> Option<Vec3> {
        self.0
            .as_ref()
            .map(|b| crate::walking::new_world_start(&b.contacts, sea_radius, direction))
    }

    /// Hashes of what the planet is made of: the base records, every fine
    /// level's records and neighbours, and the column tier's records. Two
    /// planets with equal hashes drew and collided the same.
    pub fn digest(&self) -> [u64; 3] {
        use std::hash::{Hash, Hasher};
        let built = self.0.as_ref().expect("a prepared planet not yet taken");
        let hash = |bytes: &[&[u8]]| {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            for b in bytes {
                b.hash(&mut h);
            }
            h.finish()
        };
        let base = hash(&[bytemuck::cast_slice(&built.base)]);
        let mut fine: Vec<&[u8]> = built
            .fine
            .levels
            .iter()
            .map(|l| bytemuck::cast_slice::<GpuCell, u8>(l))
            .collect();
        fine.push(bytemuck::cast_slice(&built.fine.finest_neighbors));
        let fine = hash(&fine);
        let tier = hash(&[bytemuck::cast_slice(&built.fine.columns.records)]);
        [base, fine, tier]
    }

    /// Where the fine set is anchored.
    pub fn anchor(&self) -> Option<Vec3> {
        self.0.as_ref().map(|b| b.anchor)
    }
}

/// Rebuild the planet for the generator a load just switched to, round
/// `near`, with the opened world's edits, and put it in place of the old one
/// in one step: nothing is ever removed, since a dozen systems take the
/// planet's resources as given. The fine set's version moves on from the old
/// one, and the base's generation too, so the render world uploads both; the
/// refresh in flight, begun on the old planet, is let go.
pub fn rebuild_planet(world: &mut World, near: Vec3) {
    let started = std::time::Instant::now();
    let columns = world.resource::<crate::config::ColumnSettings>().clone();
    let scatter = world.resource::<crate::config::ScatterSettings>().clone();
    let built = {
        let edits = &world.resource::<crate::saves::WorldSave>().edits;
        build_planet(near, &columns, &scatter, edits)
    };
    install(world, built);
    info!(
        "planet rebuilt for generator {} round {:?} in {:.2}s",
        terrain::generator_version(),
        near,
        started.elapsed().as_secs_f64()
    );
}

/// Put a planet the loading thread built in place of the one standing, as
/// [`rebuild_planet`] does with one it builds: a world opened from the saves
/// page (`new-world-arrival` decision 2). False when it was taken already.
pub fn install_prepared(world: &mut World, mut prepared: PreparedPlanet) -> bool {
    let Some(built) = prepared.0.take() else {
        return false;
    };
    let anchor = built.anchor;
    install(world, built);
    info!(
        "planet put in place for generator {} round {anchor:?}",
        terrain::generator_version()
    );
    true
}

fn install(world: &mut World, built: BuiltPlanet) {
    let dusk = world.resource::<lod::DuskLamps>().lit;
    let mut fine = Arc::unwrap_or_clone(built.fine);
    fine.columns.set_dusk(dusk);
    let fine = Arc::new(fine);
    let mut contacts = built.contacts;
    contacts.set_fine(&fine);
    let generation = world.resource::<PlanetBase>().generation + 1;
    let version = world.resource::<lod::PlanetFine>().version + 1;
    world.insert_resource(PlanetBase {
        cells: built.base,
        generation,
    });
    world.insert_resource(contacts);
    world.insert_resource(lod::PlanetFine { set: fine, version });
    world.resource_mut::<lod::LodRefresh>().abandon();
}

fn create_planet(
    mut commands: Commands,
    assets: Res<AssetServer>,
    flight: Res<crate::flight_view::FlightViewConfig>,
    columns: Res<crate::config::ColumnSettings>,
    scatter: Res<crate::config::ScatterSettings>,
    edits: Res<crate::saves::WorldSave>,
    prepared: Option<ResMut<PreparedPlanet>>,
) {
    let started = std::time::Instant::now();
    // Built already behind the loading screen, with the towns' ground in
    // place; otherwise (a test app, a run without the screen) built here.
    let taken = prepared.and_then(|mut p| p.0.take());
    if taken.is_some() {
        commands.remove_resource::<PreparedPlanet>();
    }
    let BuiltPlanet {
        cells,
        base,
        contacts,
        fine,
        anchor,
    } = taken
        .unwrap_or_else(|| build_planet(flight.spawn_direction, &columns, &scatter, &edits.edits));
    let (narrowest, mean_width, widest) = tile_widths(&cells);
    if !edits.edits.is_empty() {
        info!(
            "{} edits across {} cells loaded from the save",
            edits.edits.len(),
            edits.edits.cells()
        );
    }
    let fine_count: usize = fine.levels.iter().map(Vec::len).sum();
    info!(
        "Planet ready: base L{} {} columns, fine L{}-L{} {} columns around the spawn, \
         12 pentagons, {:.1} MiB records, {:.2}s generation",
        lod::BASE_LEVEL,
        base.len(),
        lod::FINE_LEVELS[0],
        lod::FINEST_LEVEL,
        fine_count,
        ((base.len() + fine_count) * size_of::<GpuCell>()) as f64 / 1_048_576.,
        started.elapsed().as_secs_f64()
    );
    let tier = &fine.columns;
    let caves = tier
        .columns
        .iter()
        .filter(|column| column.drawn_runs().len() > 1)
        .count();
    let mouths = fine
        .finest_records()
        .iter()
        .filter(|cell| {
            let direction = Vec3::from_slice(&cell.direction_height[..3]);
            cell.direction_height[3] < surface_height(direction) - 0.5
        })
        .count();
    // What the light bake costs, measured rather than asserted: it decides
    // whether a dig can afford to relight the whole tier, which is what a dig
    // does.
    let lit = std::time::Instant::now();
    let mut relit = tier.clone();
    relit.relight();
    let relight_ms = lit.elapsed().as_secs_f64() * 1000.;
    let dark = relit
        .columns
        .iter()
        .enumerate()
        .flat_map(|(slot, col)| {
            (1..pbd_core::column::LAYERS).map(move |layer| (slot, layer, col.solid(layer)))
        })
        .filter(|&(slot, layer, solid)| !solid && relit.sky(slot, layer) == 0)
        .count();
    info!(
        "Column tier: anchor {anchor:?}, {} columns within {:.0} m of it, {caves} of them carrying a cave, \
         {mouths} records lowered by a mouth, \
         {:.2} runs mean, {:.2} MiB of records, \
         {dark} unlit air cells, light bake {relight_ms:.1} ms, {:.2} MiB of light",
        tier.columns.len(),
        columns.reach_m,
        tier.columns
            .iter()
            .map(|c| c.drawn_runs().len())
            .sum::<usize>() as f64
            / tier.columns.len().max(1) as f64,
        (tier.columns.len() * size_of::<column::GpuColumn>()) as f64 / 1_048_576.,
        (tier.columns.len() * column::LIGHT_WORDS * 4) as f64 / 1_048_576.,
    );
    info!(
        "Planet scale: r={:.0} m; base L{} gives {:.2} m mean tile width ({:.2}-{:.2} m), \
         the finest L{} {:.3} m underfoot within {:.0} m; {:.0} m elevation step; \
         the walker's eye is {:.2} m",
        PLANET_RADIUS,
        lod::BASE_LEVEL,
        mean_width,
        narrowest,
        widest,
        lod::FINEST_LEVEL,
        lod::tile_width_m(lod::FINEST_LEVEL),
        lod::BAND_M[3],
        ELEVATION_STEP,
        crate::walking::EYE_HEIGHT,
    );
    commands.insert_resource(contacts);
    commands.insert_resource(PlanetBase {
        cells: base,
        generation: 1,
    });
    commands.insert_resource(lod::PlanetFine {
        set: fine,
        version: 1,
    });
    commands.insert_resource(PlanetArt(assets.load_with_settings(
        "tilesets/atlas.png",
        |settings: &mut ImageLoaderSettings| settings.sampler = ImageSampler::nearest(),
    )));
    commands.spawn((
        Name::new("GPU hex planet"),
        PlanetSurface,
        SyncToRenderWorld,
    ));
}

/// Centre-to-centre spacing of two neighbouring columns, in metres: the
/// flat-to-flat width of the shared tile. A dual cell sits on a primal vertex
/// and two cells adjoin exactly when their vertices share a primal edge, so
/// this chord IS that edge scaled onto the body. Slope shading and the scale
/// readout both need it; one function so they cannot disagree.
fn tile_width(cell: &topology::DualCell, neighbor: &topology::DualCell) -> f32 {
    cell.direction.distance(neighbor.direction) * PLANET_RADIUS
}

/// Measured `(min, mean, max)` tile width over the whole globe, in metres.
/// The running total is `f64` on purpose: the shipped level sums about 3.9
/// million widths, and an `f32` accumulator stops resolving an addend of ~19
/// once the total passes 2^24, which silently reported 18.66 m for a globe
/// whose real mean is 18.89 m.
/// Reported at startup because the number that decides whether a 1.6 m walker
/// reads as human-sized is a property of the topology, not of a document: it
/// moves the moment `SUBDIVISIONS` or `PLANET_RADIUS` does.
fn tile_widths(cells: &[topology::DualCell]) -> (f32, f32, f32) {
    let (mut min, mut max, mut total, mut count) = (f32::MAX, 0f32, 0f64, 0u32);
    for cell in cells {
        for &neighbor in &cell.neighbors {
            let width = tile_width(cell, &cells[neighbor]);
            min = min.min(width);
            max = max.max(width);
            total += width as f64;
            count += 1;
        }
    }
    (min, (total / f64::from(count.max(1))) as f32, max)
}

#[derive(Clone, ShaderType)]
struct PlanetParams {
    clip_from_body: Mat4,
    camera: Vec4,
    sun: Vec4,
    // Sea-level radius, cell count, elapsed seconds, foliage range (zero disables it).
    settings: Vec4,
    // RGB water absorption per metre; w the sheet's radius (sea level less the
    // depth offset), which is where submerged shading starts.
    water_absorption: Vec4,
    // The colour submerged terrain converges to with depth.
    water_deep: Vec4,
    // Ground wetness, rain intensity, cloud cover over the player, lightning.
    weather: Vec4,
    // The terrain-wetness knobs from `weather.ron`, then the overcast ones.
    rain: [Vec4; 6],
    // Base record count and fine-region capacity; the fine regions follow the
    // base at that stride, two halves of four (`two-set-dissolve`); z the half
    // the current set is in, w spare.
    lod_offsets: UVec4,
    // Live record count per fine level of the current set.
    lod_counts: UVec4,
    // Player direction in the body frame; w the base level.
    lod: Vec4,
    // cos(band radius / R) per fine level, coarsest first.
    bands: Vec4,
    // Clutter reach and fade in metres, blade count, base shade.
    clutter: Vec4,
    // Clutter chances: grass, flower, rock, bush.
    clutter_chance: Vec4,
    // Clutter sizes in metres: blade height, blade half-width, rock, bush.
    clutter_size: Vec4,
    // Flower stem height, dead-shrub chance and twig length, spare.
    clutter_more: Vec4,
    // Column tier reach in metres (zero is off), the cave-darkening floor, the
    // metres of burial it reaches that floor over, and the cosine of twice the
    // reach: the ANGULAR gate the visibility pass gives a column tier cell.
    column: Vec4,
    // How deep the sod and the soil run, in metres, and two spares. Fed from
    // `pbd_core::column`, which is where the cells themselves are stacked: a
    // wall shows what a shovel would find, because both read these two numbers.
    ground: Vec4,
    // Which slot of `atlas.png` each biome draws from, in `Biome` order: ocean,
    // beach, fields, desert, then jungle, swamp, mountains, tundra. The shader
    // reads the biome off the cell it is already given, so a sheet per biome
    // costs one lookup and no second binding.
    tilesets: [UVec4; 2],
    // `detail-fade`: how far a tree dithers out before the edge of its band
    // or the foliage range, metres; the landing cross-fade's progress 0..1;
    // one while it runs; spare.
    fade: Vec4,
    // The partition the cross-fade dissolves from: its anchor (w spare) and
    // band cosines, as `lod` and `bands`.
    lod_prev: Vec4,
    bands_prev: Vec4,
    // Live record count per fine level of the set the landing dissolves from,
    // in the other half (`two-set-dissolve`); zero outside a dissolve, which
    // leaves that half idle. Then a spare lane.
    lod_counts_prev: UVec4,
    lod_spare: Vec4,
    // `distance-lod-fade`: the fine set's anchor (what the records' rings are
    // measured from; `lod` is now the fade centre), and the inner edges of the
    // bands' cross-fade rings, cosines, for the partition and the one a
    // landing dissolves from.
    anchor: Vec4,
    bands_in: Vec4,
    bands_prev_in: Vec4,
    // `lighting-model`: x one for the new model (L1, L2), yzw spare.
    light: Vec4,
    // The sky's light from all round (L2), nine SH coefficients for a clear
    // sky then nine for a full deck, in the local frame of the player's sun
    // elevation (`pbd_core::sky_light`); a fragment mixes them by the cover
    // over it and reads them in its own up-and-sun frame.
    sky_sh: [Vec4; 18],
    // The new wet film (`lighting-model`, wet ground): roughness wet and in a
    // puddle, how bright a lamp's highlight is, and one while the rain rings
    // follow the rain's intensity (the new model).
    wet: Vec4,
    // The lights nearest the camera, planet frame, w the level 0..1 (zero
    // ends the list): the wet film's lamp highlights.
    lamps: [Vec4; NEAR_LAMPS],
    // L8: x one while a wet face's reflected ray walks the columns, y how far
    // it walks, metres; L4: z the sun's bounce's strength now (zero: none,
    // or not baked for this set yet).
    reflect: Vec4,
    // L7: x how far the newest lamp bake has faded in, 0..1; y one where the
    // bake before it holds this set's slots, z one where the newest does; w
    // which half of the buffer is the newest.
    lamp_bake: Vec4,
    // L8 (10.2): the frame the screen trace's history holds, its
    // clip_from_body and its eye (w one once there is one).
    ssr_clip: Mat4,
    ssr_eye: Vec4,
    // `wet-near-detail`: x how far from the eye rain's fine detail is drawn,
    // metres; y the band it fades out over before that; z a plant's
    // darkening at full wetness (`flora-v2` 13.2); w spare.
    rain_near: Vec4,
    // `baked-light-ease` 7.3: the light field across a dusk or dawn relight:
    // x how far the newest half has blended in, 0..1; y which half is the
    // newest; z one while the other half holds the field it blends from; w
    // spare.
    light_ease: Vec4,
    // `flora-v2` design 6: x tufts per cell (zero: the blades), y and z the
    // tuft's width and height, m; w how far round the year, 0..1.
    flora: Vec4,
    // x mushroom chance, y fern chance, zw spare.
    flora_more: Vec4,
    // Per biome, `Biome` order: fern weight, mushroom weight, flower colour
    // bits, one where deciduous (`pbd_core::flora::BIOME_FLORA`).
    flora_biome: [Vec4; 8],
    // Per biome: the tuft's three tones, `0xRRGGBB`, w one for red caps.
    tuft_ramps: [UVec4; 8],
    // `flora-v2` design 9.2, 9.6: per biome the mean of the leaves, bark and
    // stone tiles, `0xRRGGBB`; w the fruit its trees bear, as bits.
    plant_means: [UVec4; 8],
    // Design 9.4, 9.6: x leaf fall, y gathered under trees, z leaf size, w
    // fruit chance.
    flora_fall: Vec4,
    // `flora-v2` 13.3: the trees' cross-fade by time (`TreeFade`): the point
    // it fades from with its progress 0..1, then the point it fades to (w 1).
    tree_fade: [Vec4; 2],
    // `night-lights-from-space`: x how many far lights `night_lights` holds,
    // yzw spare; then two vec4 a light (`night_lights::FarLights::pack`).
    // Last, so the visibility pass, which reads a prefix of this uniform,
    // never sees them.
    night: Vec4,
    night_lights: [Vec4; 2 * crate::night_lights::MAX_FAR_LIGHTS],
    // `flora-v2` design 12.5: the ids of the felled trees and of the trees
    // picked bare near the camera, four a lane, zero empty
    // (`trees_taken::TreesTaken`).
    felled: [UVec4; crate::trees_taken::FELLED / 4],
    picked: [UVec4; crate::trees_taken::PICKED / 4],
}

/// Words of one half of the light buffer: a level per cell, four to a word,
/// every slot of the column tier (`baked-light-ease` 7.3).
const LIGHT_HALF_WORDS: usize = column::COLUMN_CAPACITY as usize * column::LIGHT_WORDS;

/// How far the newest lamp bake shows over the one before it
/// (`baked-light-ease`): whole from its first frame when it came with its
/// set's landing (7.2), else the rule's blend from when it landed (7.7).
/// The trees' cross-fade by time (`flora-v2` 13.3; owner, 2026-11-16: "dithering
/// should either stay or go, dont have it linger if the player isnt moving").
///
/// Two points the camera stood at, `from` and `to`, body-local, and the fade
/// between them: the surface shader decides each tree's representation (near
/// model, mid cards, far billboard, none) at both by hard thresholds, draws it
/// whole where they agree and dithers from the one to the other by the fade's
/// progress where they do not. A new fade starts only when the last has run
/// out and the camera has moved, so a still camera has `from == to` one fade
/// after it stops, and nothing is dithered. No state per tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeFade {
    pub from: Vec3,
    pub to: Vec3,
    start_s: f32,
    len_s: f32,
    last_s: f32,
    last_eye: Vec3,
    /// Whether `PBD_TREE_ARRIVE` has been applied.
    arrived: bool,
}

/// MEASUREMENT (`flora-v2` 13.3): `PBD_TREE_ARRIVE=<metres>@<seconds>`: at
/// that time on the planet clock the trees are taken as if the camera had
/// just come that far across the ground and stopped, so the ones it carried
/// across a threshold fade, and a capture run shows the frames 0, 0.3 and 2 s
/// after. It changes nothing else.
static TREE_ARRIVE: std::sync::LazyLock<Option<(f32, f32)>> = std::sync::LazyLock::new(|| {
    let spec = std::env::var("PBD_TREE_ARRIVE").ok()?;
    let (metres, at) = spec.split_once('@')?;
    let metres: f32 = metres.trim().parse().ok()?;
    let at: f32 = at.trim().parse().ok()?;
    (metres.is_finite() && metres > 0.0 && metres < TREE_FADE_JUMP_M && at.is_finite())
        .then_some((metres, at))
});

/// A camera that moves further than this in one frame has been put somewhere,
/// not walked or flown there: the trees are decided where it now is.
const TREE_FADE_JUMP_M: f32 = 200.0;

impl TreeFade {
    pub fn new(eye: Vec3, now_s: f32) -> Self {
        Self {
            from: eye,
            to: eye,
            start_s: now_s,
            len_s: 0.0,
            last_s: now_s,
            last_eye: eye,
            arrived: false,
        }
    }

    /// One frame: the camera at `eye` at `now_s`; the fade takes `fade_s`,
    /// shortened so `to` never trails a moving camera by more than `lag_m`.
    pub fn step(&mut self, eye: Vec3, now_s: f32, fade_s: f32, lag_m: f32) {
        let dt = now_s - self.last_s;
        let speed = if dt > 1e-4 {
            self.last_eye.distance(eye) / dt
        } else {
            0.0
        };
        self.last_s = now_s;
        self.last_eye = eye;
        if eye.distance(self.to) > TREE_FADE_JUMP_M || !eye.is_finite() {
            let arrived = self.arrived;
            *self = Self::new(eye, now_s);
            self.arrived = arrived;
            return;
        }
        if let Some((metres, at_s)) = *TREE_ARRIVE
            && !self.arrived
            && now_s >= at_s
            && let Some(up) = eye.try_normalize()
        {
            let across = Vec3::Y.cross(up).try_normalize().unwrap_or(Vec3::X);
            self.arrived = true;
            self.from = eye + across * metres;
            self.to = eye;
            self.start_s = now_s;
            self.len_s = fade_s;
            info!("TREE_ARRIVE {metres} m at {now_s:.2} s, fading over {fade_s} s");
            return;
        }
        if self.progress(now_s) < 1.0 {
            return;
        }
        self.from = self.to;
        if eye.distance(self.to) <= 0.01 {
            return;
        }
        let len = fade_s.min(lag_m / speed.max(1e-3));
        self.to = eye;
        self.start_s = now_s;
        self.len_s = len;
        if len <= 1e-3 {
            self.from = eye;
        }
    }

    /// How far the fade from `from` to `to` has come, 0..1; one when they agree.
    pub fn progress(&self, now_s: f32) -> f32 {
        if self.from == self.to || self.len_s <= 0.0 {
            1.0
        } else {
            ((now_s - self.start_s) / self.len_s).clamp(0.0, 1.0)
        }
    }

    /// The uniform's two lanes: `from` with the progress, `to` with a one.
    fn lanes(&self, now_s: f32) -> [Vec4; 2] {
        [self.from.extend(self.progress(now_s)), self.to.extend(1.0)]
    }
}

fn lamp_fade(at_once: bool, landed_s: f32, now_s: f32) -> f32 {
    if at_once {
        1.0
    } else {
        pbd_core::light::bake_blend(now_s - landed_s)
    }
}

/// The light field's blend across a relight, as `PlanetParams::light_ease`
/// carries it (`baked-light-ease` 7.3): the newest half, and the other blended
/// from while `blending`.
fn light_ease_state(newest: usize, blending: bool, from_s: f32, now_s: f32) -> Vec4 {
    let blend = if blending {
        pbd_core::light::bake_blend(now_s - from_s)
    } else {
        1.0
    };
    Vec4::new(blend, newest as f32, if blending { 1.0 } else { 0.0 }, 0.0)
}

/// Where a relit field goes on the GPU (`baked-light-ease` 7.3): into the
/// other half, blended from the one drawn now, only for a dusk or dawn relight
/// of the set drawn the frame before (the same columns, the version right
/// after it); anything else (an edit, a landing) overwrites the newest half
/// and is drawn whole at once. The half written, and whether it blends.
fn light_half(newest: usize, relight: bool, follows: bool, same_anchor: bool) -> (usize, bool) {
    if relight && follows && same_anchor {
        (1 - newest, true)
    } else {
        (newest, false)
    }
}

/// L7: the lamp bake's cross-fade, as `PlanetParams::lamp_bake` carries it.
fn lamp_bake_state(
    lighting: &crate::config::LightingSettings,
    planet: &PlanetGpu,
    fine: Option<&lod::PlanetFine>,
    now_s: f32,
) -> Vec4 {
    let Some(fine) = fine else {
        return Vec4::ZERO;
    };
    if !lighting.lamp_bake_on() {
        return Vec4::ZERO;
    }
    let valid = |half: usize| {
        planet.lamp_halves[half].is_some_and(|(_, anchor)| anchor.distance(fine.set.anchor) < 1e-3)
    };
    let new = planet.lamp_new;
    Vec4::new(
        lamp_fade(planet.lamp_at_once, planet.lamp_landed_s, now_s),
        if valid(1 - new) { 1.0 } else { 0.0 },
        if valid(new) { 1.0 } else { 0.0 },
        new as f32,
    )
}

/// L4: how strongly the bounce is drawn: none where the bake held is for a
/// set laid round another anchor, blended in by the rule after a bake for an
/// anchor no bounce was drawn for lands, and at full strength otherwise (a
/// set that brought its own bounce is drawn whole from its landing frame).
fn bounce_gain(
    lighting: &crate::config::LightingSettings,
    fine: Option<&lod::PlanetFine>,
    field: Option<&bounce::BounceField>,
    now_s: f32,
) -> f32 {
    let (Some(fine), Some(field)) = (fine, field) else {
        return 0.0;
    };
    if !lighting.bounce_on()
        || field.gpu.is_empty()
        || fine.set.anchor.distance(field.anchor) > 1e-3
    {
        return 0.0;
    }
    let fade = if field.fades {
        pbd_core::light::bake_blend(now_s - field.landed_s)
    } else {
        1.0
    };
    lighting.bounce_strength * fade
}

/// How many lights near the camera the terrain is told of.
pub(crate) const NEAR_LAMPS: usize = 8;

/// The lights of the column tier nearest `camera` (planet frame), nearest
/// first, within `reach` metres: the field's lamps, not its glowing flowers.
pub(crate) fn near_lamps(
    tier: &column::ColumnTier,
    camera: Vec3,
    reach: f32,
) -> [Vec4; NEAR_LAMPS] {
    // The nearest few, kept in order as they are found (`lighting-perf`
    // 2.5): no list of every lamp in reach is made and sorted each frame.
    // A tie keeps the lamp found first, as the stable sort it replaces did.
    let mut near = [(f32::INFINITY, Vec4::ZERO); NEAR_LAMPS];
    for l in tier
        .lamps()
        .iter()
        .filter(|l| l.level > pbd_core::flora::GLOW_FLOWER_LEVEL)
    {
        let at = l.direction * (PLANET_RADIUS + l.altitude);
        let d = at.distance_squared(camera);
        if d >= reach * reach || d >= near[NEAR_LAMPS - 1].0 {
            continue;
        }
        let at_rank = near.iter().position(|n| d < n.0).unwrap_or(NEAR_LAMPS - 1);
        near.copy_within(at_rank..NEAR_LAMPS - 1, at_rank + 1);
        near[at_rank] = (d, at.extend(l.level as f32 / pbd_core::light::MAX as f32));
    }
    near.map(|(_, lamp)| lamp)
}

/// The sky's light (`lighting-model` L2) for a sun at elevation sine `e`: a
/// clear sky's nine coefficients, then a full deck's. The ground under each
/// is lit by the sun that gets through it (`1 - 0.85 cover`, the mockup's).
///
/// Read off a table built once (513 elevations, each a projection of a few
/// hundred sky samples, about a tenth of a second in all) and interpolated:
/// the sun's elevation where the camera is changes every frame in flight,
/// and projecting per call cost two projections a frame on two threads.
pub(crate) fn sky_sh_pair(e: f32) -> [Vec4; 18] {
    use std::sync::OnceLock;
    const N: usize = 512;
    static TABLE: OnceLock<Vec<[Vec4; 18]>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        (0..=N)
            .map(|k| {
                let e = -1.0 + 2.0 * k as f32 / N as f32;
                let sun = pbd_core::sky_light::sun_air(e);
                let mut out = [Vec4::ZERO; 18];
                for (half, cover) in [0.0f32, 1.0].into_iter().enumerate() {
                    let dim = 1.0 - 0.85 * cover;
                    let c = pbd_core::sky_light::sky_sh(e, cover, sun.map(|v| v * dim));
                    for (i, v) in c.iter().enumerate() {
                        out[half * 9 + i] = Vec3::from_array(*v).extend(0.0);
                    }
                }
                out
            })
            .collect()
    });
    let x = (e.clamp(-1.0, 1.0) + 1.0) * 0.5 * N as f32;
    let k = (x.floor() as usize).min(N - 1);
    let t = x - k as f32;
    std::array::from_fn(|i| table[k][i].lerp(table[k + 1][i], t))
}

/// The eight biome slots, in `Biome` order, packed two vec4s wide for the
/// uniform. Written here rather than in the shader because which sheet a
/// biome draws from is the app's answer and the shader's lookup.
fn tileset_slots() -> [UVec4; 2] {
    use pbd_core::planet_gen::Biome;
    let of = |b| terrain::tileset_slot(b);
    [
        UVec4::new(
            of(Biome::Ocean),
            of(Biome::Beach),
            of(Biome::Fields),
            of(Biome::Desert),
        ),
        UVec4::new(
            of(Biome::Jungle),
            of(Biome::Swamp),
            of(Biome::Mountains),
            of(Biome::Tundra),
        ),
    ]
}

#[derive(Resource)]
struct PlanetGpu {
    cells: Buffer,
    /// One record per column tier slot, rewritten with the fine set that built
    /// it. The tier is the innermost part of the finest band, so this is small:
    /// a few thousand slots of 48 bytes.
    columns: Buffer,
    /// The sky level of every cell of every column tier slot, four layers to a
    /// word. What a face's corner samples to find out how much daylight
    /// reached the air it opens onto.
    light: Buffer,
    /// Every layer's render code, per column tier slot: what a column-pass
    /// face wears. `ColumnTier::gpu_materials` packs it.
    materials: Buffer,
    /// Every column tier slot's unit direction (w spare): what the reflected
    /// ray's walk over the columns steps from one cell to the next by
    /// (`lighting-model` L8). `ColumnTier::gpu_centres` packs it.
    centres: Buffer,
    /// L4: the sun's bounce over the tier (`planet_bounce.rs`): per slot the
    /// base layer of its window, then its words; and which bake it holds.
    bounce: Buffer,
    bounce_version: u64,
    /// L7: the lamp bake, two halves (`planet_bounce::lamp_half_words` each):
    /// the newest bake, and the one before it that it cross-fades from. Per
    /// half the versions and the anchor it was baked for; which half is the
    /// newest, and when it landed.
    lamp_bake: Buffer,
    /// `flora-v2` design 10.2: the detailed trees' kit (`planet_trees.rs`).
    tree_kit: Buffer,
    /// L8 (10.2): the screen trace's history (`planet_ssr.rs`).
    #[allow(dead_code)]
    ssr: Texture,
    ssr_view: TextureView,
    lamp_halves: [Option<((u64, u64), Vec3)>; 2],
    lamp_new: usize,
    lamp_landed_s: f32,
    /// Whether the newest lamp bake came with its set's landing, and shows
    /// whole from its first frame (`baked-light-ease` 7.2).
    lamp_at_once: bool,
    /// `baked-light-ease` 7.3: the light buffer's newest half, whether the
    /// other is blended from (a dusk or dawn relight), since when, and the
    /// anchor of the set the newest was written for.
    light_new: usize,
    light_blending: bool,
    light_from_s: f32,
    light_anchor: Option<Vec3>,
    /// Record slots in the buffer: the base then two halves of four fine
    /// regions (`two-set-dissolve`).
    slots: u32,
    base_count: u32,
    /// Which build of the base the buffer holds (`PlanetBase::generation`).
    base_generation: u64,
    /// Live records per fine level of the current set, in `FINE_LEVELS`
    /// order.
    counts: [u32; 4],
    /// The half of the fine regions the current set is in (0 or 1). The other
    /// half holds the set the last landing replaced, which its dissolve draws
    /// the old partition from (`two-set-dissolve`).
    half: u32,
    /// Live records per fine level of the set in the other half.
    counts_prev: [u32; 4],
    /// The fine set version the regions hold.
    uploaded: u64,
    /// The partition the uploaded regions can serve: the anchor they were
    /// generated around and the radius each level is complete to. It is
    /// written here, beside the buffer, so the rule that decides what to HIDE
    /// and the records that REPLACE it can never come from different frames.
    lod: lod::LodParams,
    /// The partition the last landing replaced and when it landed, on the
    /// planet clock, while its dissolve runs (`detail-fade`,
    /// `two-set-dissolve`); none when no landing has dissolved since startup
    /// or a change of generator.
    lod_prev: Option<(lod::LodParams, f32)>,
    /// The next set that moves the partition lands at once: the first after
    /// startup (behind the loading screen), and the first on a new generator,
    /// whose other half holds another planet.
    fresh: bool,
}

#[derive(Component)]
struct PlanetViewGpu {
    uniform: UniformBuffer<PlanetParams>,
    // Held by bind groups as well; retained explicitly to make lifetime clear.
    _visible: Buffer,
    _foliage: Buffer,
    _clutter: Buffer,
    _column_list: Buffer,
    _near: Buffer,
    /// Water cell IDs the visibility pass listed; the water pass draws them.
    water: Buffer,
    indirect: Buffer,
    draw_bind_group: BindGroup,
    foliage_bind_group: BindGroup,
    clutter_bind_group: BindGroup,
    column_bind_group: BindGroup,
    near_bind_group: BindGroup,
    compute_bind_group: BindGroup,
    /// The fade centre (`distance-lod-fade`): the camera's ground point, held
    /// within the fine set's fit of its anchor and caught up at a bounded
    /// speed after a landing; the landing it last saw (its time on the planet
    /// clock); and where it stood when that set landed, which the landing's
    /// dissolve starts from. Keyed to the landing, not the version: an edit
    /// during a dissolve makes a version and must not move the old partition
    /// (`two-set-dissolve`).
    centre: Vec3,
    fitted: Option<f32>,
    fade_from: Vec3,
    last: std::time::Instant,
    last_camera: Vec3,
    /// The frame's clip_from_body: next frame's screen trace reads the
    /// history this frame leaves with it.
    last_clip: Mat4,
    /// What this frame's screen trace reprojects with (the history's frame),
    /// for the sea's trace as well as the terrain's.
    ssr_clip: Mat4,
    ssr_eye: Vec4,
    /// The trees' cross-fade by time (`flora-v2` 13.3).
    tree_fade: TreeFade,
}

/// The slowest the fade centre catches up with the camera after a landing,
/// m/s; it goes at twice the camera's own speed when that is faster.
const CENTRE_CATCH_UP_MPS: f32 = 30.0;

/// Fine regions per half of the cell buffer, one per fine level.
const FINE_REGIONS: u32 = lod::FINE_LEVELS.len() as u32;

/// Where fine level `k` of half `half` starts in the cell buffer, in records.
fn fine_region(base_count: u32, half: u32, k: usize) -> u64 {
    u64::from(base_count)
        + (u64::from(half) * u64::from(FINE_REGIONS) + k as u64) * u64::from(lod::FINE_CAPACITY)
}

/// What a fine set's upload does to the two halves (`two-set-dissolve`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Landing {
    /// The partition did not move (an edit, a relight): written over the
    /// current half in place; a running dissolve runs on.
    InPlace,
    /// It moved, at startup or on a new generator: drawn at once.
    AtOnce,
    /// It moved: written into the other half, and the old partition dissolves
    /// out of the half it was drawn from.
    Dissolve,
}

/// How a version lands, from whether it moved the partition and whether the
/// buffer has a partition it may dissolve from.
fn landing(moved: bool, fresh: bool) -> Landing {
    match (moved, fresh) {
        (false, _) => Landing::InPlace,
        (true, true) => Landing::AtOnce,
        (true, false) => Landing::Dissolve,
    }
}

/// `from` turned toward `to` by at most `angle` radians, on the unit sphere.
fn toward(from: Vec3, to: Vec3, angle: f32) -> Vec3 {
    let between = from.dot(to).clamp(-1.0, 1.0).acos();
    if between <= angle || between < 1e-7 {
        return to;
    }
    let axis = from.cross(to).normalize_or_zero();
    if axis == Vec3::ZERO {
        return from;
    }
    Quat::from_axis_angle(axis, angle.max(0.0)) * from
}

fn upload_planet(
    mut commands: Commands,
    base: Option<Res<PlanetBase>>,
    existing: Option<ResMut<PlanetGpu>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let Some(base) = base else {
        return;
    };
    if let Some(mut planet) = existing {
        // A switch of generator rebuilt the base: it is written over the old
        // one in the SAME buffer, which every view's bind group holds, and the
        // topology is fixed, so it is the same size (`bigger-biomes` decision
        // 8). No cross-fade runs from the old planet's partition.
        if planet.base_generation != base.generation {
            queue.write_buffer(
                &planet.cells,
                0,
                bytemuck::cast_slice(base.cells.as_slice()),
            );
            planet.base_generation = base.generation;
            planet.lod_prev = None;
            planet.counts_prev = [0; 4];
            planet.fresh = true;
        }
        return;
    }
    let base_count = base.cells.len() as u32;
    // Two halves of the four fine regions: the current set and the one the
    // last landing replaced, which its dissolve still draws
    // (`two-set-dissolve`).
    let slots = base_count + 2 * FINE_REGIONS * lod::FINE_CAPACITY;
    // Zero-initialised: an empty slot has degree zero and the compute pass
    // skips it, so the fine regions are inert until their first upload.
    let cells = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet columns: base level, then four fine bands"),
        size: slots as u64 * size_of::<GpuCell>() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&cells, 0, bytemuck::cast_slice(base.cells.as_slice()));
    // Zero-initialised: a slot nothing has written is DARK, which is the safe
    // way round. A light buffer defaulting to full daylight would light every
    // cave in the tier on the frame before its bake arrived.
    // Two halves (`baked-light-ease` 7.3): the field drawn, and across a dusk
    // or dawn relight the field before it, which the shader blends from.
    let light = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet voxel light for the column tier, two halves"),
        size: 2 * LIGHT_HALF_WORDS as u64 * 4,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let materials = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet voxel materials for the column tier"),
        size: column::COLUMN_CAPACITY as u64 * column::MATERIAL_WORDS as u64 * 4,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bounce = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet sun bounce for the column tier"),
        size: column::COLUMN_CAPACITY as u64 * (1 + pbd_core::bounce::WORDS as u64) * 4,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let (ssr, ssr_view) = ssr::create_texture(&device);
    let lamp_bake = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet lamp bake for the column tier, two halves"),
        size: 2 * bounce::lamp_half_words(column::COLUMN_CAPACITY as usize) as u64 * 4,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let centres = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet voxel column centres for the column tier"),
        size: column::COLUMN_CAPACITY as u64 * 16,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    // Zero-initialised: every run word is the absent one, so a slot nothing has
    // written draws no face at all.
    let columns = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet voxel columns for the column tier"),
        size: column::COLUMN_CAPACITY as u64 * size_of::<column::GpuColumn>() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let tree_kit = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Detailed trees: models, anchors, tiles and baked impostors"),
        contents: bytemuck::cast_slice(&trees::kit().words),
        usage: BufferUsages::STORAGE,
    });
    commands.insert_resource(PlanetGpu {
        tree_kit,
        light,
        materials,
        centres,
        bounce,
        bounce_version: u64::MAX,
        lamp_bake,
        ssr,
        ssr_view,
        lamp_halves: [None, None],
        lamp_new: 0,
        lamp_landed_s: 0.0,
        lamp_at_once: false,
        light_new: 0,
        light_blending: false,
        light_from_s: 0.0,
        light_anchor: None,
        cells,
        columns,
        slots,
        base_count,
        base_generation: base.generation,
        counts: [0; 4],
        half: 0,
        counts_prev: [0; 4],
        uploaded: 0,
        lod: lod::LodParams::base_only(),
        lod_prev: None,
        fresh: true,
    });
}

/// Rewrite the fine regions when a new fine set has landed. Each level's
/// records go at its own offset; the count is what the compute pass reads.
// Nine parameters: the seven it had, the dissolve's length, which says
// whether a landing would cut a running dissolve short (`two-set-dissolve`),
// and the dusk relight's version, whose light blends in (`baked-light-ease`).
#[allow(clippy::too_many_arguments)]
fn upload_fine(
    fine: Option<Res<lod::PlanetFine>>,
    planet: Option<ResMut<PlanetGpu>>,
    queue: Res<RenderQueue>,
    bounce: Option<Res<bounce::BounceField>>,
    lamps: Option<Res<bounce::LampBakeField>>,
    clock: Res<PlanetClock>,
    lighting: Option<Res<crate::config::LightingSettings>>,
    scatter: Option<Res<crate::config::ScatterSettings>>,
    relight: Option<Res<lod::DuskRelight>>,
) {
    let (Some(fine), Some(mut planet)) = (fine, planet) else {
        return;
    };
    // L4: a new bake of the bounce goes up whenever one lands.
    if let Some(bounce) = bounce
        && bounce.version != planet.bounce_version
        && !bounce.gpu.is_empty()
    {
        let room = planet.bounce.size() as usize / 4;
        let words = &bounce.gpu[..bounce.gpu.len().min(room)];
        queue.write_buffer(&planet.bounce, 0, bytemuck::cast_slice(words));
        planet.bounce_version = bounce.version;
    }
    // L7: a new lamp bake goes into the older half, and the shader fades to
    // it from the other by the rule (`baked-light-ease` 7.7), or shows it
    // whole at once where it came with its set's landing (7.2).
    if let Some(lamps) = lamps
        && !lamps.gpu.is_empty()
        && planet.lamp_halves[planet.lamp_new].map(|h| h.0) != Some(lamps.version)
    {
        let half = 1 - planet.lamp_new;
        let words = bounce::lamp_half_words(column::COLUMN_CAPACITY as usize);
        let data = &lamps.gpu[..lamps.gpu.len().min(words)];
        queue.write_buffer(
            &planet.lamp_bake,
            (half * words * 4) as u64,
            bytemuck::cast_slice(data),
        );
        planet.lamp_halves[half] = Some((lamps.version, lamps.anchor));
        planet.lamp_new = half;
        planet.lamp_landed_s = clock.0;
        planet.lamp_at_once = lamps.with_set;
    }
    if planet.uploaded == fine.version {
        return;
    }
    let follows = planet.uploaded.checked_add(1) == Some(fine.version);
    let timer = std::time::Instant::now();
    let old = planet.lod;
    let new = lod::LodParams::of(&fine.set);
    // An edit's new version, or the dusk relight's, keeps the anchor and the
    // bands; a landing moves them.
    let moved = old.player != new.player || old.bands != new.bands;
    let how = landing(moved, planet.fresh);
    if how == Landing::Dissolve {
        // The set the old partition was drawn from stays where it is, in the
        // half it was drawn from, for the dissolve to draw it whole
        // (`two-set-dissolve`); the new set goes into the other half, which
        // is idle once a dissolve has ended. `LodRefresh` holds a set that
        // lands inside a dissolve until it has, so this never cuts one short.
        let fade_s = scatter.as_ref().map_or(0.0, |s| s.lod_fade_s);
        if let Some((_, landed)) = planet.lod_prev
            && clock.0 - landed < fade_s
        {
            warn!(
                "LOD_FADE cut short: set version {} landed {:.2} s into a {fade_s:.1} s dissolve",
                fine.version,
                clock.0 - landed
            );
        }
        planet.counts_prev = planet.counts;
        planet.half = 1 - planet.half;
    }
    let stride = size_of::<GpuCell>() as u64;
    for (k, level) in fine.set.levels.iter().enumerate() {
        let offset = fine_region(planet.base_count, planet.half, k) * stride;
        if !level.is_empty() {
            queue.write_buffer(
                &planet.cells,
                offset,
                bytemuck::cast_slice(level.as_slice()),
            );
        }
        planet.counts[k] = level.len().min(lod::FINE_CAPACITY as usize) as u32;
    }
    // With each column's reach top, which ends a reflected ray's walk once
    // it has risen past everything it could still meet (`lighting-perf` 2.1).
    let reach_m = lighting.map_or_else(
        || crate::config::LightingSettings::default().reflection_reach_m,
        |l| l.reflection_reach_m,
    );
    let records = fine.set.columns.gpu_records_with_reach(reach_m);
    if !records.is_empty() {
        queue.write_buffer(&planet.columns, 0, bytemuck::cast_slice(&records));
        // The light goes up with the records that decide what it lights. Two
        // uploads a frame apart would draw one frame of the new geometry lit
        // by the old world.
        //
        // A dusk or dawn relight of the set drawn until now goes into the
        // other half and is blended in from this one (`baked-light-ease`
        // 7.3); anything else overwrites the newest half and is whole at once.
        let light = fine.set.columns.gpu_light();
        let (half, blending) = light_half(
            planet.light_new,
            relight.is_some_and(|r| r.version == fine.version),
            follows,
            planet
                .light_anchor
                .is_some_and(|a| a.distance(fine.set.anchor) < 1e-3),
        );
        let words = &light[..light.len().min(LIGHT_HALF_WORDS)];
        queue.write_buffer(
            &planet.light,
            (half * LIGHT_HALF_WORDS * 4) as u64,
            bytemuck::cast_slice(words),
        );
        planet.light_new = half;
        planet.light_blending = blending;
        planet.light_from_s = clock.0;
        planet.light_anchor = Some(fine.set.anchor);
        // And the materials, for the same reason: a face drawn from a stale
        // layer is a placed stone wearing the earth that was there before.
        let materials = fine.set.columns.gpu_materials();
        queue.write_buffer(&planet.materials, 0, bytemuck::cast_slice(&materials));
        let centres = fine.set.columns.gpu_centres();
        queue.write_buffer(&planet.centres, 0, bytemuck::cast_slice(&centres));
    }
    planet.uploaded = fine.version;
    match how {
        // The partition is the one a running dissolve dissolves into: it runs
        // on.
        Landing::InPlace => {}
        Landing::AtOnce => {
            planet.lod_prev = None;
            planet.counts_prev = [0; 4];
            planet.fresh = false;
        }
        Landing::Dissolve => {
            // Every landing dissolves the whole of the old partition, out of
            // its own records (`two-set-dissolve`): no ring is drawn from the
            // new partition alone, because none of the old one is drawn from
            // the new records.
            info!(
                "LOD_FADE dissolve: set version {} into half {}, the old partition from half {}",
                fine.version,
                planet.half,
                1 - planet.half
            );
            planet.lod_prev = Some((old, clock.0));
        }
    }
    planet.lod = new;
    lod::spent("render: fine-set upload", timer);
    // The other half of an edit's own log line: the version it made is the
    // version the GPU now draws. An edit whose version never appears here is
    // an edit nobody can see.
    debug!("fine set version {} uploaded", fine.version);
}

#[derive(Resource)]
struct PlanetPipeline {
    shader: Handle<Shader>,
    draw_layout: BindGroupLayoutDescriptor,
    compute_layout: BindGroupLayoutDescriptor,
    clear: CachedComputePipelineId,
    compact: CachedComputePipelineId,
}

fn draw_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "planet vertex pulling",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                uniform_buffer::<PlanetParams>(false),
                storage_buffer_read_only_sized(false, NonZeroU64::new(size_of::<GpuCell>() as u64)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
                texture_2d(TextureSampleType::Float { filterable: false }),
                storage_buffer_read_only_sized(
                    false,
                    NonZeroU64::new(size_of::<column::GpuColumn>() as u64),
                ),
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(16)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
                texture_2d(TextureSampleType::Float { filterable: false }),
                // `flora-v2` design 10.2: the detailed trees' kit.
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
            ),
        ),
    )
}

fn compute_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "planet visible column compaction",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<PlanetParams>(false),
                storage_buffer_read_only_sized(false, NonZeroU64::new(size_of::<GpuCell>() as u64)),
                storage_buffer_sized(false, NonZeroU64::new(4)),
                storage_buffer_sized(false, NonZeroU64::new(INDIRECT_BYTES)),
                storage_buffer_sized(false, NonZeroU64::new(4)),
                storage_buffer_sized(false, NonZeroU64::new(4)),
                storage_buffer_sized(false, NonZeroU64::new(4)),
                storage_buffer_sized(false, NonZeroU64::new(4)),
                // The near trees, a region per species (design 10.5).
                storage_buffer_sized(false, NonZeroU64::new(NEAR_LIST_BYTES)),
            ),
        ),
    )
}

fn initialize_pipelines(
    mut commands: Commands,
    assets: Res<AssetServer>,
    cache: Res<PipelineCache>,
) {
    let draw_layout = draw_layout();
    let compute_layout = compute_layout();
    let compute_shader = assets.load("shaders/planet_visibility.wgsl");
    let compute = |entry: &'static str| {
        cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::Borrowed(entry)),
            layout: vec![compute_layout.clone()],
            shader: compute_shader.clone(),
            entry_point: Some(Cow::Borrowed(entry)),
            ..default()
        })
    };
    let clear = compute("clear_indirect");
    let compact = compute("compact_visible");
    commands.insert_resource(PlanetPipeline {
        shader: assets.load("shaders/planet_surface.wgsl"),
        draw_layout,
        compute_layout,
        clear,
        compact,
    });
}

impl SpecializedRenderPipeline for PlanetPipeline {
    /// MSAA samples, HDR, and whether L8's traces go to the half-size pass
    /// (`lighting-perf` 2.8): then the fragment writes the request and what a
    /// hit replaces beside the colour.
    type Key = (u32, bool, bool);
    fn specialize(&self, (samples, hdr, split): Self::Key) -> RenderPipelineDescriptor {
        let color = Some(ColorTargetState {
            format: if hdr {
                ViewTarget::TEXTURE_FORMAT_HDR
            } else {
                TextureFormat::bevy_default()
            },
            blend: None,
            write_mask: ColorWrites::ALL,
        });
        let targets = if split {
            vec![
                color,
                Some(ColorTargetState {
                    format: reflect::REQUEST_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                }),
                Some(ColorTargetState {
                    format: reflect::REPLACED_FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                }),
            ]
        } else {
            vec![color]
        };
        RenderPipelineDescriptor {
            label: Some(Cow::Borrowed("Opaque GPU hex world")),
            layout: vec![
                self.draw_layout.clone(),
                weather_maps::layout(),
                shadow::receiver_layout(),
            ],
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: Some(Cow::Borrowed("vertex")),
                ..default()
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                entry_point: Some(Cow::Borrowed(if split {
                    "fragment_split"
                } else {
                    "fragment"
                })),
                targets,
                ..default()
            }),
            primitive: PrimitiveState {
                cull_mode: Some(Face::Back),
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: default(),
                bias: default(),
            }),
            multisample: MultisampleState {
                count: samples,
                ..default()
            },
            ..default()
        }
    }
}

/// Every authored number this pass reads, in one place.
///
/// Four `Res<...Settings>` that always travel together, and the reason they
/// are a struct is Bevy's own limit rather than taste: adding the sun took
/// this system to seventeen parameters, and the error says nothing whatever
/// about arguments - it says a function is "not a system set". That is the
/// missing-struct smell `CLAUDE.md` names, arriving exactly as it warns.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct Tunables<'w> {
    water: Res<'w, crate::config::WaterSettings>,
    weather: Res<'w, crate::config::WeatherSettings>,
    scatter: Res<'w, crate::config::ScatterSettings>,
    columns: Res<'w, crate::config::ColumnSettings>,
    lighting: Res<'w, crate::config::LightingSettings>,
    /// The fine set, for the lights near the camera (`lighting-model`).
    fine: Option<Res<'w, lod::PlanetFine>>,
    /// L4: the bounce baked for it.
    bounce: Option<Res<'w, bounce::BounceField>>,
    /// Whether an overlay is showing: the map hides the clouds, and so their
    /// shadows.
    maps: Res<'w, weather_maps::WeatherMapsNow>,
    /// The night side's far lights (`night-lights-from-space`).
    far: Option<Res<'w, crate::night_lights::FarLights>>,
    /// The player's grass (`flora-v2`).
    flora: Option<Res<'w, FloraLook>>,
    /// The sea now, for the tide (`ocean-environments` 5.1).
    sea_now: Option<Res<'w, crate::sea::SeaNow>>,
    /// The trees felled and picked near the camera (`flora-v2` 12.5).
    taken: Option<Res<'w, crate::trees_taken::TreesTaken>>,
}

#[allow(clippy::too_many_arguments)]
fn prepare_views(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    cache: Res<PipelineCache>,
    pipeline: Res<PlanetPipeline>,
    planet: Option<Res<PlanetGpu>>,
    art: Option<Res<PlanetArt>>,
    images: Res<RenderAssets<GpuImage>>,
    clock: Res<PlanetClock>,
    sun: Res<crate::sky::Sun>,
    frame: Res<PlanetRenderFrame>,
    tunables: Tunables,
    weather: Res<crate::weather::Weather>,
    mut views: Query<(Entity, &ExtractedView, Option<&mut PlanetViewGpu>), With<Msaa>>,
) {
    let water_settings = &tunables.water;
    let (tide_here, sheet_reach) = tunables.sea_now.as_deref().map_or((0.0, 0.0), |now| {
        let range = now.gpu.tide.w;
        let reach = if range > 0.0 {
            pbd_core::sea::tide_high_m(range) - water_settings.depth_offset_m + 1.0
        } else {
            0.0
        };
        (now.tide_here, reach.max(0.0))
    });
    let weather_settings = &tunables.weather;
    let scatter = &tunables.scatter;
    let columns = &tunables.columns;
    let lighting = &tunables.lighting;
    let Some(planet) = planet else {
        return;
    };
    let Some(atlas) = art.as_ref().and_then(|art| images.get(&art.0)) else {
        return;
    };
    for (entity, view, existing) in &mut views {
        let (camera_position, clip_from_body) = frame.camera_and_clip(
            &view.world_from_view,
            view.clip_from_view,
            view.clip_from_world,
        );
        let foliage_range =
            if camera_position.length() - PLANET_RADIUS > FOLIAGE_DRAW_CUTOFF_ALTITUDE {
                0.
            } else {
                FOLIAGE_DRAW_DISTANCE
            };
        let w = &weather_settings;
        let (flora_biome, tuft_ramps, plant_means) = flora_tables();
        let (felled, picked) = tunables
            .taken
            .as_deref()
            .map(crate::trees_taken::TreesTaken::packed)
            .unwrap_or_default();
        let lod = &planet.lod;
        let landing = planet.lod_prev.map(|(_, landed)| landed);
        let mut tree_fade = existing.as_ref().map_or_else(
            || TreeFade::new(camera_position, clock.0),
            |gpu| gpu.tree_fade,
        );
        tree_fade.step(
            camera_position,
            clock.0,
            scatter.tree_fade_s,
            scatter.tree_fade_lag_m,
        );
        // The fade centre (`distance-lod-fade`): the ground under the camera,
        // held within what the records hold round the anchor, and caught up
        // after a landing at twice the camera's speed rather than jumping.
        let now = std::time::Instant::now();
        let anchor = lod.player;
        let fit = lod.fit_m / PLANET_RADIUS;
        let target = toward(anchor, camera_position.normalize_or(anchor), fit);
        let (centre, fade_from) = match existing.as_ref() {
            Some(gpu) => {
                let dt = now.duration_since(gpu.last).as_secs_f32().min(0.5);
                let speed = gpu.last_camera.distance(camera_position) / dt.max(1e-3);
                let reach = (2.0 * speed).max(CENTRE_CATCH_UP_MPS) * dt / PLANET_RADIUS;
                let centre = toward(anchor, toward(gpu.centre, target, reach), fit);
                let landed = gpu.fitted != landing;
                (centre, if landed { gpu.centre } else { gpu.fade_from })
            }
            None => (target, target),
        };
        // The landing's cross-fade (`detail-fade`): how far through it is, on
        // the planet clock (`two-set-dissolve`). The partition it dissolves
        // from stood round the fade centre as it was when the set landed
        // (`distance-lod-fade`), and is drawn from the other half's records,
        // which are live only while it runs.
        let (prev, prev_centre, fade_progress, fading, counts_prev) = match planet.lod_prev {
            Some((prev, landed)) if scatter.lod_fade_s > 0.0 => {
                let t = (clock.0 - landed) / scatter.lod_fade_s;
                if t < 1.0 {
                    (prev, fade_from, t.max(0.0), 1.0, planet.counts_prev)
                } else {
                    (*lod, centre, 1.0, 0.0, [0; 4])
                }
            }
            _ => (*lod, centre, 1.0, 0.0, [0; 4]),
        };
        trace!(
            "planet view {entity}: camera {camera_position:?}, lod player {:?}, base {} fine {:?}",
            lod.player, planet.base_count, planet.counts
        );
        let (night, night_lights) = tunables.far.as_ref().map_or(
            (
                Vec4::ZERO,
                [Vec4::ZERO; 2 * crate::night_lights::MAX_FAR_LIGHTS],
            ),
            |far| far.pack(sun.direction(), camera_position, PLANET_RADIUS),
        );
        let params = PlanetParams {
            clip_from_body,
            camera: camera_position.extend(1.),
            sun: sun.direction().extend(1.),
            settings: Vec4::new(PLANET_RADIUS, planet.slots as f32, clock.0, foliage_range),
            // The DOWNWELLING (`ocean-environments` B2): the seabed's light is
            // the daylight arriving at its depth. The lane keeps its name; its
            // only reader is that light, and the sea radius in w.
            water_absorption: Vec3::from_array(water_settings.downwell_per_m)
                .extend(PLANET_RADIUS - water_settings.depth_offset_m),
            // w: the tide at the camera, m, which lifts or lowers the sea's
            // tint on the ground with the drawn sheet (`ocean-environments`
            // 5.1). The tide moves by centimetres over a view.
            water_deep: Vec3::from_array(water_settings.deep_color).extend(tide_here),
            weather: Vec4::new(
                weather.wetness,
                weather.rain,
                weather.cover,
                // Lightning, as the ground takes it: the strike's brightness
                // now times `lightning_ground`.
                weather.flash.w * w.lightning_ground,
            ),
            rain: [
                Vec4::new(
                    w.rain_ripple_scale,
                    w.rain_ripple_strength,
                    w.rain_flow_across,
                    w.rain_flow_down,
                ),
                Vec4::new(
                    w.rain_flow_speed,
                    w.rain_flow_strength,
                    w.rain_wave_scale,
                    w.rain_wave_strength,
                ),
                Vec4::new(
                    w.rain_wave_speed,
                    w.rain_wet_darken,
                    w.rain_mirror_strength,
                    w.rain_glint_power,
                ),
                Vec4::new(
                    w.rain_glint_strength,
                    w.rain_puddle_scale_m,
                    w.rain_puddle_share,
                    w.rain_grass_rings,
                ),
                Vec4::new(
                    w.overcast_sun_dim,
                    w.overcast_amb_dim,
                    w.cloud_fog_add,
                    w.rain_fog_mult,
                ),
                // The sky's own overcast pair, for the sky colour the ground
                // hazes toward and mirrors: the same greying the dome takes.
                Vec4::new(w.overcast_sky_blue_cut, w.overcast_sky_dim, 0., 0.),
            ],
            lod_offsets: UVec4::new(planet.base_count, lod::FINE_CAPACITY, planet.half, 0),
            lod_counts: UVec4::from_array(planet.counts),
            lod: centre.extend(lod::BASE_LEVEL as f32),
            bands: lod.bands,
            clutter: Vec4::new(
                // Clutter rides the foliage cutoff: above it no cell of the
                // finest tier is near enough to grow anything, and one gate
                // for both keeps them from disagreeing about that.
                if foliage_range == 0. {
                    0.
                } else {
                    scatter.clutter_radius_m
                },
                scatter.clutter_fade_m,
                scatter.grass_blades,
                scatter.grass_base_shade,
            ),
            clutter_chance: Vec4::new(
                scatter.grass_chance,
                scatter.flower_chance,
                scatter.rock_chance,
                scatter.bush_chance,
            ),
            clutter_size: Vec4::new(
                scatter.grass_height_m,
                scatter.grass_blade_w_m,
                scatter.rock_size_m,
                scatter.bush_size_m,
            ),
            clutter_more: Vec4::new(
                scatter.flower_height_m,
                scatter.shrub_chance,
                scatter.shrub_size_m,
                // Where the ground's cloud shadows are cast from: a radius a
                // third of the way up the cloud layer.
                crate::sky::CLOUD_RADIUS + crate::sky::CLOUD_THICKNESS * 0.35,
            ),
            column: Vec4::new(
                // The tier rides the foliage cutoff for the same reason the
                // clutter does: above it nothing of the finest tier is near
                // enough to be looked into, and one gate keeps them in step.
                if foliage_range == 0. {
                    0.
                } else {
                    columns.reach_m
                },
                // Two lanes the burial stand-in used. It is gone: the voxel
                // field in `skylight` is what darkens a cave now, and a knob
                // nothing reads is a knob the next reader has to prove dead.
                0.,
                0.,
                (2.0 * columns.reach_m / PLANET_RADIUS).cos(),
            ),
            ground: Vec4::new(
                pbd_core::column::SOD_DEPTH_M,
                pbd_core::column::SOIL_DEPTH_M,
                terrain::snow_slot() as f32,
                if tunables.maps.overlay_kind.is_some() {
                    0.0
                } else {
                    w.cloud_shadow
                },
            ),
            tilesets: tileset_slots(),
            fade: Vec4::new(scatter.tree_fade_m, fade_progress, fading, 0.0),
            lod_prev: prev_centre.extend(0.0),
            bands_prev: prev.bands,
            lod_counts_prev: UVec4::from_array(counts_prev),
            // Caustics on the seabed (`ocean-environments` B5): strength,
            // the web's cell m, the depth they are gone by m. The lane is the
            // visibility pass's spare; it reads none of it.
            lod_spare: Vec4::new(
                water_settings.caustics,
                water_settings.caustic_cell_m,
                water_settings.caustic_fade_m,
                // The visibility pass lists the sea sheet over every cell
                // whose ground is under this height (m over sea level): the
                // highest the tide brings the sheet, so at high water it runs
                // on over the beach and never stands as a lip (B7).
                sheet_reach,
            ),
            anchor: anchor.extend(0.0),
            bands_in: lod.bands_in,
            bands_prev_in: prev.bands_in,
            light: Vec4::new(
                if lighting.new_model_on() { 1.0 } else { 0.0 },
                lighting.lamp_glow,
                lighting.shadow_spread(),
                lighting.shadow_cover_soften,
            ),
            sky_sh: sky_sh_pair(camera_position.normalize_or(Vec3::Y).dot(sun.direction())),
            wet: Vec4::new(
                w.rain_wet_roughness,
                w.rain_puddle_roughness,
                w.rain_lamp_specular,
                if lighting.new_model_on() { 1.0 } else { 0.0 },
            ),
            lamps: tunables
                .fine
                .as_ref()
                .map(|fine| near_lamps(&fine.set.columns, camera_position, 32.0))
                .unwrap_or([Vec4::ZERO; NEAR_LAMPS]),
            // x the voxel walk, w the screen trace (`PBD_REFLECT`).
            reflect: Vec4::new(
                if lighting.reflect_walk_on() { 1.0 } else { 0.0 },
                lighting.reflection_reach_m,
                bounce_gain(
                    lighting,
                    tunables.fine.as_deref(),
                    tunables.bounce.as_deref(),
                    clock.0,
                ),
                if lighting.reflect_screen_on() {
                    1.0
                } else {
                    0.0
                },
            ),
            lamp_bake: lamp_bake_state(lighting, &planet, tunables.fine.as_deref(), clock.0),
            ssr_clip: existing
                .as_ref()
                .map_or(Mat4::IDENTITY, |gpu| gpu.last_clip),
            rain_near: Vec4::new(
                w.rain_detail_reach_m,
                w.rain_detail_fade_m,
                w.rain_plant_darken,
                0.0,
            ),
            light_ease: light_ease_state(
                planet.light_new,
                planet.light_blending,
                planet.light_from_s,
                clock.0,
            ),
            flora: Vec4::new(
                if tunables.flora.as_ref().is_some_and(|f| f.tufts) {
                    scatter.tuft_count
                } else {
                    0.0
                },
                scatter.tuft_width_m,
                scatter.tuft_height_m,
                sun.clock.year_fraction() as f32,
            ),
            flora_more: Vec4::new(
                scatter.mushroom_chance,
                scatter.fern_chance,
                scatter.tree_near_m,
                scatter.tree_mid_m,
            ),
            flora_biome,
            tuft_ramps,
            plant_means,
            flora_fall: Vec4::new(
                scatter.leaf_fall,
                scatter.leaf_under,
                scatter.leaf_size,
                scatter.fruit_chance,
            ),
            tree_fade: tree_fade.lanes(clock.0),
            night,
            night_lights,
            felled,
            picked,
            // w: there is a history to read, so not while the capture is off.
            ssr_eye: existing.as_ref().map_or(Vec4::ZERO, |gpu| {
                gpu.last_camera.extend(if lighting.reflect_capture_on() {
                    1.0
                } else {
                    0.0
                })
            }),
        };
        let (ssr_clip, ssr_eye) = (params.ssr_clip, params.ssr_eye);
        if let Some(mut gpu) = existing {
            gpu.ssr_clip = ssr_clip;
            gpu.ssr_eye = ssr_eye;
            gpu.uniform.set(params);
            gpu.uniform.write_buffer(&device, &queue);
            gpu.centre = centre;
            gpu.fitted = landing;
            gpu.fade_from = fade_from;
            gpu.last = now;
            gpu.last_camera = camera_position;
            gpu.last_clip = clip_from_body;
            gpu.tree_fade = tree_fade;
            continue;
        }
        let mut uniform = UniformBuffer::from(params);
        uniform.write_buffer(&device, &queue);
        // The terrain and foliage lists hold a cell twice while a landing
        // cross-fades, once per partition (`detail-fade`).
        let visible = device.create_buffer(&BufferDescriptor {
            label: Some("GPU visible planet column IDs"),
            size: planet.slots as u64 * 8,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let indirect = device.create_buffer(&BufferDescriptor {
            label: Some(
                "GPU planet indirect draws: terrain, foliage, water, clutter, columns, near trees",
            ),
            size: INDIRECT_BYTES,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        queue.write_buffer(
            &indirect,
            NEAR_ARGS_OFFSET,
            bytemuck::cast_slice(&near_args()),
        );
        let near = device.create_buffer(&BufferDescriptor {
            label: Some("GPU near detailed tree IDs, a region per species"),
            size: NEAR_LIST_BYTES,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let foliage = device.create_buffer(&BufferDescriptor {
            label: Some("GPU nearby foliage column IDs"),
            size: planet.slots as u64 * 8,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        // Two entries a cell as well: the sea sheet dissolves with the ground
        // at a landing (`two-set-dissolve`).
        let water = device.create_buffer(&BufferDescriptor {
            label: Some("GPU visible water cell IDs"),
            size: planet.slots as u64 * 8,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let column_list = device.create_buffer(&BufferDescriptor {
            label: Some("GPU visible column tier cell IDs"),
            size: planet.slots as u64 * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let clutter = device.create_buffer(&BufferDescriptor {
            label: Some("GPU nearby ground clutter column IDs"),
            size: planet.slots as u64 * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        // One bind group per instance list, all reading the same records: what a
        // draw differs in is WHICH cells it is given, never what it may read.
        let view_bind_group = |label: &'static str, list: &Buffer| {
            device.create_bind_group(
                Some(label),
                &cache.get_bind_group_layout(&pipeline.draw_layout),
                &BindGroupEntries::sequential((
                    &uniform,
                    planet.cells.as_entire_binding(),
                    list.as_entire_binding(),
                    &atlas.texture_view,
                    planet.columns.as_entire_binding(),
                    planet.light.as_entire_binding(),
                    planet.materials.as_entire_binding(),
                    planet.centres.as_entire_binding(),
                    planet.bounce.as_entire_binding(),
                    planet.lamp_bake.as_entire_binding(),
                    &planet.ssr_view,
                    planet.tree_kit.as_entire_binding(),
                )),
            )
        };
        let draw_bind_group = view_bind_group("planet surface view", &visible);
        let compute_bind_group = device.create_bind_group(
            Some("planet visibility view"),
            &cache.get_bind_group_layout(&pipeline.compute_layout),
            &BindGroupEntries::sequential((
                &uniform,
                planet.cells.as_entire_binding(),
                visible.as_entire_binding(),
                indirect.as_entire_binding(),
                foliage.as_entire_binding(),
                water.as_entire_binding(),
                clutter.as_entire_binding(),
                column_list.as_entire_binding(),
                near.as_entire_binding(),
            )),
        );
        let foliage_bind_group = view_bind_group("planet foliage view", &foliage);
        let clutter_bind_group = view_bind_group("planet clutter view", &clutter);
        let column_bind_group = view_bind_group("planet column view", &column_list);
        let near_bind_group = view_bind_group("planet near trees view", &near);
        commands.entity(entity).insert(PlanetViewGpu {
            uniform,
            _visible: visible,
            _foliage: foliage,
            _clutter: clutter,
            _column_list: column_list,
            _near: near,
            water,
            indirect,
            draw_bind_group,
            foliage_bind_group,
            clutter_bind_group,
            column_bind_group,
            near_bind_group,
            compute_bind_group,
            centre,
            fitted: landing,
            fade_from,
            last: now,
            last_camera: camera_position,
            last_clip: clip_from_body,
            ssr_clip,
            ssr_eye,
            tree_fade,
        });
    }
}

/// The terrain's pipeline for a view, and whether it leaves L8's traces to
/// the half-size pass: `planet_reflect.rs`'s node draws with it.
#[derive(Component)]
struct PlanetDraw {
    pipeline: CachedRenderPipelineId,
    split: bool,
}

/// The terrain draws in a pass of its own (`lighting-perf` 2.8,
/// `planet_reflect.rs`), between Bevy's transmissive and transparent passes:
/// where it drew before as the transparent phase's first item, so after the
/// opaque towns and before every transparent mesh.
fn queue_planet(
    mut commands: Commands,
    pipeline: Res<PlanetPipeline>,
    mut specialized: ResMut<SpecializedRenderPipelines<PlanetPipeline>>,
    cache: Res<PipelineCache>,
    lighting: Option<Res<crate::config::LightingSettings>>,
    views: Query<(Entity, &ExtractedView, &Msaa)>,
    surfaces: Query<(), With<PlanetSurface>>,
) {
    if surfaces.is_empty() {
        for (entity, _, _) in &views {
            commands.entity(entity).remove::<PlanetDraw>();
        }
        return;
    }
    let split_on = lighting.is_some_and(|l| l.reflect_split_on());
    for (entity, view, msaa) in &views {
        // The half-size passes are built for the camera's MSAA and HDR
        // target; any other view traces inline.
        let split = split_on && msaa.samples() == reflect::SAMPLES && view.hdr;
        let pipeline = specialized.specialize(&cache, &pipeline, (msaa.samples(), view.hdr, split));
        commands
            .entity(entity)
            .insert(PlanetDraw { pipeline, split });
    }
}

/// The terrain's four indirect draws: the surface, the foliage, the clutter
/// and the columns, in that order.
fn draw_terrain<'w>(
    pass: &mut TrackedRenderPass<'w>,
    view: &'w PlanetViewGpu,
    maps: &'w weather_maps::WeatherMapGpu,
    receiver: &'w BindGroup,
) {
    pass.set_bind_group(1, &maps.bind_group, &[]);
    pass.set_bind_group(2, receiver, &[]);
    pass.set_bind_group(0, &view.draw_bind_group, &[]);
    pass.draw_indirect(&view.indirect, 0);
    pass.set_bind_group(0, &view.foliage_bind_group, &[]);
    pass.draw_indirect(&view.indirect, 16);
    // The near trees, one draw per species (`flora-v2` design 10.5).
    pass.set_bind_group(0, &view.near_bind_group, &[]);
    for s in 0..8 {
        pass.draw_indirect(&view.indirect, NEAR_ARGS_OFFSET + 16 * s);
    }
    pass.set_bind_group(0, &view.clutter_bind_group, &[]);
    pass.draw_indirect(&view.indirect, 48);
    // The inside of the world, last: everything before it is the surface,
    // and a cave face is only ever seen through a hole in that surface.
    pass.set_bind_group(0, &view.column_bind_group, &[]);
    pass.draw_indirect(&view.indirect, 64);
}

#[cfg(test)]
mod pipeline_tests {
    use super::*;

    #[test]
    fn live_pixel_material_uses_unfiltered_integer_texels() {
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        // The pixel atlas is loaded texel by texel and never sampled; the
        // weather maps, in their own group, are filtered on purpose.
        assert!(shader.contains("textureLoad(atlas,"));
        assert!(!shader.contains("textureSample(atlas"));
        assert!(!shader.contains("textureSampleLevel(atlas"));
        assert_eq!(
            draw_layout().entries[3].ty,
            BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: false },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            }
        );
        assert!(
            draw_layout()
                .entries
                .iter()
                .all(|entry| !matches!(entry.ty, BindingType::Sampler(_)))
        );
    }

    #[test]
    fn actual_pipeline_layouts_use_static_offsets_and_correct_storage_access() {
        // Held against the struct each shader declares, as naga lays it out:
        // a vec4 added on one side and not the other is a uniform read at the
        // wrong offsets, which draws wrong rather than failing.
        let rust = PlanetParams::min_size().get() as u32;
        let surface = crate::shader_tests::planet_surface_source();
        for (label, source) in [
            ("planet_surface.wgsl", surface.as_str()),
            (
                "planet_visibility.wgsl",
                include_str!("../../../assets/shaders/planet_visibility.wgsl"),
            ),
        ] {
            assert_eq!(
                rust,
                crate::shader_tests::wgsl_struct_size(label, source, "Params"),
                "the Rust PlanetParams must match {label}'s Params"
            );
        }
        for (layout, read_only_bindings) in [
            // 5 is the voxel sky light, 6 the layer materials, 7 the column
            // centres, 8 the sun's bounce, 9 the lamp bake and 11 the trees' kit: read only, like the
            // cells, the visible list and the column records they are sampled beside.
            (draw_layout(), &[1, 2, 4, 5, 6, 7, 8, 9, 11][..]),
            (compute_layout(), &[1][..]),
        ] {
            for entry in &layout.entries {
                if let BindingType::Buffer {
                    ty,
                    has_dynamic_offset,
                    ..
                } = entry.ty
                {
                    assert!(
                        !has_dynamic_offset,
                        "binding {} is supplied without dynamic offsets",
                        entry.binding
                    );
                    if let BufferBindingType::Storage { read_only } = ty {
                        assert_eq!(
                            read_only,
                            read_only_bindings.contains(&entry.binding),
                            "actual binding {} must agree with its WGSL access mode",
                            entry.binding
                        );
                    }
                }
            }
        }
        assert_eq!(
            draw_layout().entries[1].ty,
            BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(size_of::<GpuCell>() as u64),
            }
        );
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PlanetComputeLabel;

#[derive(Default)]
struct PlanetComputeNode {
    views: Vec<Entity>,
}

impl render_graph::Node for PlanetComputeNode {
    fn update(&mut self, world: &mut World) {
        self.views = world
            .query_filtered::<Entity, (With<PlanetViewGpu>, With<ExtractedView>)>()
            .iter(world)
            .collect();
    }
    fn run(
        &self,
        _: &mut render_graph::RenderGraphContext,
        ctx: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let (Some(planet), Some(pipeline)) = (
            world.get_resource::<PlanetGpu>(),
            world.get_resource::<PlanetPipeline>(),
        ) else {
            return Ok(());
        };
        let cache = world.resource::<PipelineCache>();
        let (Some(clear), Some(compact)) = (
            cache.get_compute_pipeline(pipeline.clear),
            cache.get_compute_pipeline(pipeline.compact),
        ) else {
            return Ok(());
        };
        let diagnostics = ctx.diagnostic_recorder();
        for entity in &self.views {
            let Some(view) = world.get::<PlanetViewGpu>(*entity) else {
                continue;
            };
            let mut pass = ctx
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Cull hex columns and publish complete indirect draw"),
                    ..default()
                });
            let span = diagnostics.pass_span(&mut pass, "planet_cull");
            pass.set_bind_group(0, &view.compute_bind_group, &[]);
            pass.set_pipeline(clear);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(compact);
            pass.dispatch_workgroups(planet.slots.div_ceil(128), 1, 1);
            span.end(&mut pass);
            drop(pass);
            // The sun's cascades, from the records this view draws
            // (`sun-shadows`), before the camera's pass reads them.
            shadow::draw_cascades(ctx, world, *entity, clear, compact, planet.slots);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `flora-v2` 13.3: a fade is a moment in time, never a place. A still
    /// camera has nothing to fade; one that moves fades from where it was to
    /// where it is over the fade time and then is whole there; a fast one
    /// fades quicker so the point the trees are decided at keeps up.
    #[test]
    fn the_trees_fade_by_time_and_a_still_camera_is_whole() {
        let eye = Vec3::new(0.0, 4800.0, 0.0);
        let mut fade = TreeFade::new(eye, 0.0);
        for k in 1..60 {
            fade.step(eye, k as f32 / 60.0, 0.5, 5.0);
        }
        assert_eq!(fade.from, fade.to);
        assert_eq!(fade.progress(1.0), 1.0, "still: nothing fading");
        // Walk 2 m over a second and stop: a fade starts there and ends 0.5 s on.
        let moved = eye + Vec3::X * 2.0;
        fade.step(moved, 2.0, 0.5, 5.0);
        assert_eq!(fade.to, moved);
        assert_ne!(fade.from, fade.to);
        assert!((fade.progress(2.25) - 0.5).abs() < 1e-4);
        let mut t = 2.0;
        while t < 2.6 {
            t += 1.0 / 60.0;
            fade.step(moved, t, 0.5, 5.0);
        }
        assert_eq!(
            fade.from, fade.to,
            "the fade ends and the camera stood still"
        );
        assert_eq!(fade.progress(t), 1.0);
        // Flying at 100 m/s the fade shortens to keep within 5 m.
        let mut fast = TreeFade::new(eye, 0.0);
        fast.step(eye + Vec3::X * (100.0 / 60.0), 1.0 / 60.0, 0.5, 5.0);
        fast.step(eye + Vec3::X * (200.0 / 60.0), 2.0 / 60.0, 0.5, 5.0);
        assert!(
            fast.len_s <= 0.05 + 1e-4,
            "fade {} s at 100 m/s",
            fast.len_s
        );
        // A teleport is no fade at all.
        let mut jump = TreeFade::new(eye, 0.0);
        jump.step(eye + Vec3::X * 500.0, 1.0 / 60.0, 0.5, 5.0);
        assert_eq!(jump.from, jump.to);
    }

    /// `baked-light-ease` 7.2 and 7.7: a lamp bake that came with its set's
    /// landing is whole from its first frame (the set before drew the same
    /// light there); any other blends in by the rule, from nothing on the
    /// frame it lands to all of it `BAKE_EASE_S` later.
    #[test]
    fn a_landing_lamp_bake_is_whole_at_once_and_a_relit_one_blends() {
        let ease = pbd_core::light::BAKE_EASE_S;
        assert_eq!(lamp_fade(true, 10.0, 10.0), 1.0, "with its set");
        assert_eq!(lamp_fade(false, 10.0, 10.0), 0.0, "a relight's, landing");
        let half = lamp_fade(false, 10.0, 10.0 + ease * 0.5);
        assert!((half - 0.5).abs() < 1e-6, "{half}");
        assert_eq!(lamp_fade(false, 10.0, 10.0 + ease), 1.0);
        // Never the old half second's linear ramp: a 60 Hz frame is a sliver.
        assert!(lamp_fade(false, 10.0, 10.0 + 1.0 / 60.0) < 0.01);
    }

    /// `baked-light-ease` 7.3: only a dusk or dawn relight of the set drawn
    /// the frame before goes into the other half and blends; an edit, a
    /// landing round a new anchor, or a relight that skipped a version
    /// overwrites the newest half and is whole at once, so a dug face never
    /// fades from the rock's dark.
    #[test]
    fn only_a_dusk_relight_blends_the_light_field() {
        assert_eq!(light_half(0, true, true, true), (1, true), "dusk");
        assert_eq!(light_half(1, true, true, true), (0, true), "dawn");
        assert_eq!(light_half(0, false, true, true), (0, false), "an edit");
        assert_eq!(light_half(1, true, false, true), (1, false), "skipped");
        assert_eq!(light_half(0, true, true, false), (0, false), "moved");
        let ease = pbd_core::light::BAKE_EASE_S;
        assert_eq!(
            light_ease_state(1, true, 5.0, 5.0),
            Vec4::new(0.0, 1.0, 1.0, 0.0),
            "the relight's frame shows the field before it"
        );
        assert_eq!(light_ease_state(1, true, 5.0, 5.0 + ease).x, 1.0);
        assert_eq!(
            light_ease_state(0, false, 5.0, 5.0),
            Vec4::new(1.0, 0.0, 0.0, 0.0),
            "an edit is whole at once"
        );
        // The shader blends the two halves the way this says.
        let shader =
            include_str!("../../../assets/shaders/planet_surface.wgsl").replace("\r\n", "\n");
        for line in [
            format!("const LIGHT_HALF_WORDS: u32 = {LIGHT_HALF_WORDS}u;"),
            "light_ease: vec4<f32>,".to_string(),
            "let newest = min(u32(params.light_ease.y), 1u);".to_string(),
            "if params.light_ease.z > 0.5 && params.light_ease.x < 1.0 {".to_string(),
            "return mix(light_cell(1u - newest, slot, layer), now, params.light_ease.x);"
                .to_string(),
        ] {
            assert!(
                shader.contains(&line),
                "planet_surface.wgsl should carry `{line}`"
            );
        }
        let visibility = include_str!("../../../assets/shaders/planet_visibility.wgsl");
        assert!(
            visibility.contains("light_ease: vec4<f32>,"),
            "the visibility pass reads the same uniform"
        );
    }

    /// The scale readout has to be a measurement, not a remembered number, and
    /// the cheap way to keep it honest is that the dual halves its edge on every
    /// subdivision: measure two low levels, check the halving, and the shipped
    /// level follows without meshing 655,362 cells in a debug test.
    #[test]
    fn measured_tile_width_follows_the_subdivision_law_and_pins_the_shipped_scale() {
        let coarse = tile_widths(&topology::dual_sphere(4)).1;
        let fine = tile_widths(&topology::dual_sphere(5)).1;
        assert!(
            (coarse / fine - 2.).abs() < 0.01,
            "a subdivision must halve the tile: {coarse} m -> {fine} m"
        );

        // The planet/scale capability quotes the equal-area hexagon instead of
        // measuring. The two have to agree or one of them is describing a
        // different planet.
        let cells = 10. * 4_f32.powi(5) + 2.;
        let area = 4. * std::f32::consts::PI * PLANET_RADIUS * PLANET_RADIUS / cells;
        let equal_area = (area / (3_f32.sqrt() / 2.)).sqrt();
        assert!(
            (fine / equal_area - 1.).abs() < 0.02,
            "measured {fine} m vs equal-area {equal_area} m"
        );

        // What the shipped base configuration actually is, measured AT that
        // level rather than extrapolated to it: the number the startup log
        // prints. The finest tier is pinned by the lattice's own test, since
        // it is never a whole sphere.
        let shipped = tile_widths(&topology::dual_sphere(lod::BASE_LEVEL as u32)).1;
        assert!(
            (shipped - lod::tile_width_m(lod::BASE_LEVEL)).abs() < 0.05,
            "shipped base tile width {shipped} m at L{} on r={PLANET_RADIUS} m",
            lod::BASE_LEVEL
        );
        let extrapolated = fine / 2_f32.powi(lod::BASE_LEVEL as i32 - 5);
        assert!(
            (extrapolated / shipped - 1.).abs() < 0.002,
            "extrapolated {extrapolated} m vs measured {shipped} m"
        );
        assert!(shipped > crate::walking::EYE_HEIGHT * 10.);
        // And the finest tier is the one the walker's eye is measured against.
        assert!(lod::tile_width_m(lod::FINEST_LEVEL) < crate::walking::EYE_HEIGHT * 2.);
    }

    /// How a fine set's version lands on the GPU (`two-set-dissolve`): one
    /// that keeps the partition (an edit, a relight) goes over the current
    /// half and leaves a running dissolve alone; one that moves it dissolves
    /// from the other half, except the first after startup or a change of
    /// generator. The two halves follow the base, four regions each, and never
    /// overlap.
    #[test]
    fn a_landing_dissolves_from_the_half_it_leaves() {
        assert_eq!(landing(false, false), Landing::InPlace);
        assert_eq!(landing(false, true), Landing::InPlace);
        assert_eq!(landing(true, true), Landing::AtOnce);
        assert_eq!(landing(true, false), Landing::Dissolve);
        let base = 163_842;
        let cap = u64::from(lod::FINE_CAPACITY);
        assert_eq!(fine_region(base, 0, 0), u64::from(base));
        assert_eq!(fine_region(base, 0, 3), u64::from(base) + 3 * cap);
        assert_eq!(fine_region(base, 1, 0), u64::from(base) + 4 * cap);
        assert_eq!(
            fine_region(base, 1, 3) + cap,
            u64::from(base + 2 * FINE_REGIONS * lod::FINE_CAPACITY),
            "the last region ends where the buffer does"
        );
    }

    /// A load that switches the generator puts a new planet in place of the
    /// old in one step (`bigger-biomes` decision 8): every resource the
    /// systems take as given is still there, the base and the fine set are
    /// marked new so the render world uploads both, the lamps stay as the
    /// clock has them, and the planet is the one a launch would have built.
    /// The switch itself is global, so it is tested in its own process
    /// (`tests/generator_switch.rs`); this rebuilds on the version in hand.
    #[test]
    fn a_rebuilt_planet_replaces_the_old_one_in_one_step() {
        let near = Vec3::new(0.3, 0.8, 0.5).normalize();
        let columns = crate::config::ColumnSettings::default();
        let scatter = crate::config::ScatterSettings::default();
        let mut save = crate::saves::WorldSave::memory_only();
        let launch = build_planet(near, &columns, &scatter, &save.edits);
        // One hole dug in the opened world, at the top of the first column
        // the launch built: the rebuilt planet must carry it.
        let (key, top) = launch
            .fine
            .finest_records()
            .iter()
            .enumerate()
            .find_map(|(record, cell)| {
                let top = launch.fine.columns.column(record)?.surface()?;
                Some((cell.key(), top))
            })
            .expect("the launch's fine set has columns");
        save.edits.set(pbd_core::edits::Edit {
            cell: key,
            layer: top as u16,
            material: pbd_core::terrain::Material::Air,
        });
        let mut world = World::new();
        world.insert_resource(columns);
        world.insert_resource(scatter);
        world.insert_resource(save);
        world.insert_resource(lod::DuskLamps { lit: true });
        world.init_resource::<lod::LodRefresh>();
        world.insert_resource(PlanetBase {
            cells: launch.base.clone(),
            generation: 1,
        });
        world.insert_resource(lod::PlanetFine {
            set: launch.fine.clone(),
            version: 7,
        });

        rebuild_planet(&mut world, near);

        let base = world.resource::<PlanetBase>();
        assert_eq!(base.generation, 2, "the render world must see a new base");
        assert!(
            bytemuck::cast_slice::<GpuCell, u8>(&base.cells)
                == bytemuck::cast_slice::<GpuCell, u8>(&launch.base),
            "the same generator rebuilds the same base records"
        );
        let fine = world.resource::<lod::PlanetFine>();
        assert_eq!(fine.version, 8, "the render world must see a new fine set");
        assert!(
            fine.set.columns.dusk(),
            "the lamps stay lit across a rebuild"
        );
        assert_eq!(
            fine.set.levels.iter().map(Vec::len).collect::<Vec<_>>(),
            launch.fine.levels.iter().map(Vec::len).collect::<Vec<_>>(),
            "the fine set is the one a launch round the same place builds"
        );
        let dug = fine
            .set
            .finest_records()
            .iter()
            .position(|cell| cell.key() == key)
            .and_then(|record| fine.set.columns.column(record))
            .expect("the dug cell is in the rebuilt set");
        assert_eq!(
            dug.material(top),
            pbd_core::terrain::Material::Air,
            "the rebuilt planet carries the opened world's edits"
        );
        assert!(world.contains_resource::<PlanetContact>());
        assert!(world.resource::<lod::LodRefresh>().in_flight_s().is_none());
    }
}

/// The finest set round `anchor` and a contact serving it, with no edits:
/// the ground an app test outside this crate stands on (the desktop
/// binary's showcase test, `step-videos` 5.1). Not for the game, which
/// builds its sets on the loading thread.
#[doc(hidden)]
pub fn test_ground(anchor: Vec3) -> (PlanetFine, PlanetContact) {
    let set = Arc::new(lod::generate_fine_live(
        anchor,
        lod::BAND_M,
        lod::REGEN_DISTANCE_M,
        &crate::config::ColumnSettings::default(),
        &pbd_core::edits::Edits::new(),
        4,
        None,
        0.0,
    ));
    let cells = topology::dual_sphere(5);
    let mut contact = PlanetContact::new(Arc::new(lod::base_records(&cells)), &cells);
    contact.set_fine(&set);
    (PlanetFine { set, version: 1 }, contact)
}
