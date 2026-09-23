//! A complete, closed hex/pentagon world drawn from persistent GPU topology.
//!
//! CPU columns are authoritative. A compute pass compacts visible column IDs
//! and writes an indirect draw; the vertex shader reconstructs caps, exposed
//! walls, and decorative trees. No expanded terrain vertex buffer is uploaded.
//! This is a surface-column prototype, not the editable volumetric chunk engine.

#[path = "planet_column.rs"]
pub mod column;
pub use column::mouth_of;
#[path = "planet_contact.rs"]
mod contact;
#[path = "planet_lattice.rs"]
pub(crate) mod lattice;
#[path = "planet_lod.rs"]
pub(crate) mod lod;
#[path = "planet_terrain.rs"]
pub(crate) mod terrain;
#[path = "planet_topology.rs"]
pub(crate) mod topology;
#[cfg(test)]
#[path = "planet_visibility_tests.rs"]
mod visibility_tests;
#[path = "planet_water.rs"]
mod water;

pub use contact::{PlanetContact, SurfaceContact};
pub use lod::{BAND_M, BASE_LEVEL, FINEST_LEVEL, LodRefresh, NearField, PlanetFine, tile_width_m};
pub use terrain::{
    DIRT, ELEVATION_STEP, GRASS_SIDE, PLANET_RADIUS, SNOW_SIDE, TERRAIN, WATER,
    nearest_ground_near, river_channel, snow_slot, surface_code, surface_height, terrain_radius,
    tileset_slot,
};
pub use water::{EyeWater, EyeWaterState, emerge, submersion};

use bevy::math::{DMat4, DVec3};
use bevy::{
    core_pipeline::core_3d::{CORE_3D_DEPTH_FORMAT, Transparent3d},
    ecs::system::SystemParamItem,
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph, RenderLabel},
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, RenderCommand,
            RenderCommandResult, SetItemPipeline, TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::{
            binding_types::{
                storage_buffer_read_only_sized, storage_buffer_sized, texture_2d, uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        sync_world::{MainEntity, SyncToRenderWorld},
        texture::GpuImage,
        view::{ExtractedView, ViewTarget},
    },
};
use bytemuck::{Pod, Zeroable};
use std::{borrow::Cow, num::NonZeroU64, sync::Arc};

// Trees live on the finest tier, so they reach exactly as far as it does.
// Above the band's radius plus the highest summit and a tree, no tree can be
// in range, and the foliage draw is disabled outright.
const FOLIAGE_DRAW_DISTANCE: f32 = lod::BAND_M[1];
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
    GRASS_BLADE_BUDGET * 2 * 6 + 18 + 54 + 108 + 36
}
/// Where the clutter branch starts in the shared vertex shader: after the
/// terrain's 60 and the foliage's 198.
#[cfg(test)]
pub(crate) fn clutter_first_vertex() -> u32 {
    60 + 198
}
/// Bytes of indirect draw arguments the compute pass publishes: five draws of
/// four words each - terrain, foliage, water, clutter, the column tier. The
/// shader's `array<DrawArgs,5>` is the same number, and it is the binding's
/// minimum size, which is what a fifth draw added to only one of the two fails
/// on: a pipeline whose layout still says four is refused at creation with
/// "shader global binding 3 is not available", which names the binding and not
/// the draw that outgrew it.
const INDIRECT_BYTES: u64 = 5 * 16;

/// Vertices per column instance: a cave ceiling fan and a cave floor fan per
/// run, then one flank quad per side per run per stretch of the NEIGHBOUR's
/// air, since a column of `MAX_RUNS` runs has one more air gap than it has
/// runs. Same arrangement as the clutter budget above, and for the same reason.
#[cfg(test)]
pub(crate) fn column_vertices() -> u32 {
    let runs = pbd_core::column::MAX_RUNS as u32;
    // ...and a torch: a four-sided post and a two-triangle head.
    runs * (18 + 18) + 6 * runs * (runs + 1) * 6 + 30
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
    pub spare: [f32; 4],
}

impl GpuCell {
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

#[derive(Resource, Clone, ExtractResource)]
struct PlanetBase(Arc<Vec<GpuCell>>);

#[derive(Resource, Clone, Default, ExtractResource)]
pub(crate) struct PlanetClock(f32);

#[derive(Resource, Clone, ExtractResource)]
pub struct PlanetArt(pub Handle<Image>);

#[derive(Component, Clone, Copy, ExtractComponent)]
struct PlanetSurface;

/// Installs CPU generation, persistent storage buffers and actual GPU pipelines.
pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<PlanetBase>::default(),
            ExtractResourcePlugin::<lod::PlanetFine>::default(),
            ExtractResourcePlugin::<PlanetClock>::default(),
            // The sun moves now, so the render world needs this frame's, not
            // the one the pipeline was built with.
            ExtractResourcePlugin::<crate::sky::Sun>::default(),
            ExtractResourcePlugin::<PlanetArt>::default(),
            ExtractResourcePlugin::<water::EyeWaterState>::default(),
            ExtractResourcePlugin::<PlanetRenderFrame>::default(),
            ExtractComponentPlugin::<PlanetSurface>::default(),
        ))
        .init_resource::<PlanetClock>()
        .init_resource::<PlanetRenderFrame>()
        .init_resource::<lod::LodRefresh>()
        .init_resource::<lod::NearField>()
        .init_resource::<water::EyeWaterState>()
        .add_systems(Startup, create_planet)
        .add_systems(Update, (lod::refresh_lod, water::publish_eye_water))
        .add_systems(
            PostUpdate,
            update_planet_frame.before(TransformSystems::Propagate),
        )
        .add_systems(Update, |time: Res<Time>, mut clock: ResMut<PlanetClock>| {
            clock.0 = time.elapsed_secs();
        });
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_render_command::<Transparent3d, DrawPlanet>()
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
        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(PlanetComputeLabel, PlanetComputeNode::default());
        graph.add_node_edge(PlanetComputeLabel, bevy::render::graph::CameraDriverLabel);
    }
}

fn create_planet(
    mut commands: Commands,
    assets: Res<AssetServer>,
    flight: Res<crate::flight_view::FlightViewConfig>,
    columns: Res<crate::config::ColumnSettings>,
    edits: Res<crate::saves::WorldSave>,
) {
    let started = std::time::Instant::now();
    let cells = topology::dual_sphere(lod::BASE_LEVEL as u32);
    let base = Arc::new(lod::base_records(&cells));
    let mut contacts = PlanetContact::new(base.clone(), &cells);
    let (narrowest, mean_width, widest) = tile_widths(&cells);
    // The fine bands around the spawn, synchronously, so the walker has its
    // tile to stand on before its first tick.
    let anchor = contacts.find_land_near(flight.spawn_direction);
    if !edits.edits.is_empty() {
        info!(
            "{} edits across {} cells loaded from the save",
            edits.edits.len(),
            edits.edits.cells()
        );
    }
    let fine = Arc::new(lod::generate_fine(anchor, &columns, &edits.edits));
    contacts.set_fine(&fine);
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
    commands.insert_resource(PlanetBase(base));
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
    // base at that stride.
    lod_offsets: UVec4,
    // Live record count per fine level.
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
    /// Record slots in the buffer: the base then four fine regions.
    slots: u32,
    base_count: u32,
    /// Live records per fine level, in `FINE_LEVELS` order.
    counts: [u32; 4],
    /// The fine set version the regions hold.
    uploaded: u64,
    /// The partition the uploaded regions can serve: the anchor they were
    /// generated around and the radius each level is complete to. It is
    /// written here, beside the buffer, so the rule that decides what to HIDE
    /// and the records that REPLACE it can never come from different frames.
    lod: lod::LodParams,
}

#[derive(Component)]
struct PlanetViewGpu {
    uniform: UniformBuffer<PlanetParams>,
    // Held by bind groups as well; retained explicitly to make lifetime clear.
    _visible: Buffer,
    _foliage: Buffer,
    _clutter: Buffer,
    _column_list: Buffer,
    /// Water cell IDs the visibility pass listed; the water pass draws them.
    water: Buffer,
    indirect: Buffer,
    draw_bind_group: BindGroup,
    foliage_bind_group: BindGroup,
    clutter_bind_group: BindGroup,
    column_bind_group: BindGroup,
    compute_bind_group: BindGroup,
}

fn upload_planet(
    mut commands: Commands,
    base: Option<Res<PlanetBase>>,
    existing: Option<Res<PlanetGpu>>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    if existing.is_some() {
        return;
    }
    let Some(base) = base else {
        return;
    };
    let base_count = base.0.len() as u32;
    let slots = base_count + 4 * lod::FINE_CAPACITY;
    // Zero-initialised: an empty slot has degree zero and the compute pass
    // skips it, so the fine regions are inert until their first upload.
    let cells = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet columns: base level, then four fine bands"),
        size: slots as u64 * size_of::<GpuCell>() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&cells, 0, bytemuck::cast_slice(base.0.as_slice()));
    // Zero-initialised: a slot nothing has written is DARK, which is the safe
    // way round. A light buffer defaulting to full daylight would light every
    // cave in the tier on the frame before its bake arrived.
    let light = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet voxel sky light for the column tier"),
        size: column::COLUMN_CAPACITY as u64 * column::LIGHT_WORDS as u64 * 4,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let materials = device.create_buffer(&BufferDescriptor {
        label: Some("Persistent planet voxel materials for the column tier"),
        size: column::COLUMN_CAPACITY as u64 * column::MATERIAL_WORDS as u64 * 4,
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
    commands.insert_resource(PlanetGpu {
        light,
        materials,
        cells,
        columns,
        slots,
        base_count,
        counts: [0; 4],
        uploaded: 0,
        lod: lod::LodParams::base_only(),
    });
}

/// Rewrite the fine regions when a new fine set has landed. Each level's
/// records go at its own offset; the count is what the compute pass reads.
fn upload_fine(
    fine: Option<Res<lod::PlanetFine>>,
    planet: Option<ResMut<PlanetGpu>>,
    queue: Res<RenderQueue>,
) {
    let (Some(fine), Some(mut planet)) = (fine, planet) else {
        return;
    };
    if planet.uploaded == fine.version {
        return;
    }
    let stride = size_of::<GpuCell>() as u64;
    for (k, level) in fine.set.levels.iter().enumerate() {
        let offset = (planet.base_count as u64 + k as u64 * lod::FINE_CAPACITY as u64) * stride;
        if !level.is_empty() {
            queue.write_buffer(
                &planet.cells,
                offset,
                bytemuck::cast_slice(level.as_slice()),
            );
        }
        planet.counts[k] = level.len().min(lod::FINE_CAPACITY as usize) as u32;
    }
    let records = fine.set.columns.gpu_records();
    if !records.is_empty() {
        queue.write_buffer(&planet.columns, 0, bytemuck::cast_slice(records));
        // The light goes up with the records that decide what it lights. Two
        // uploads a frame apart would draw one frame of the new geometry lit
        // by the old world.
        let light = fine.set.columns.gpu_light();
        queue.write_buffer(&planet.light, 0, bytemuck::cast_slice(&light));
        // And the materials, for the same reason: a face drawn from a stale
        // layer is a placed stone wearing the earth that was there before.
        let materials = fine.set.columns.gpu_materials();
        queue.write_buffer(&planet.materials, 0, bytemuck::cast_slice(&materials));
    }
    planet.uploaded = fine.version;
    planet.lod = lod::LodParams::of(&fine.set);
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
    type Key = (u32, bool);
    fn specialize(&self, (samples, hdr): Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some(Cow::Borrowed("Opaque GPU hex world")),
            layout: vec![self.draw_layout.clone()],
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: Some(Cow::Borrowed("vertex")),
                ..default()
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                entry_point: Some(Cow::Borrowed("fragment")),
                targets: vec![Some(ColorTargetState {
                    format: if hdr {
                        ViewTarget::TEXTURE_FORMAT_HDR
                    } else {
                        TextureFormat::bevy_default()
                    },
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
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
}

#[allow(clippy::too_many_arguments)]
fn prepare_views(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    cache: Res<PipelineCache>,
    pipeline: Res<PlanetPipeline>,
    planet: Option<Res<PlanetGpu>>,
    art: Res<PlanetArt>,
    images: Res<RenderAssets<GpuImage>>,
    clock: Res<PlanetClock>,
    sun: Res<crate::sky::Sun>,
    frame: Res<PlanetRenderFrame>,
    tunables: Tunables,
    weather: Res<crate::weather::Weather>,
    mut views: Query<(Entity, &ExtractedView, Option<&mut PlanetViewGpu>), With<Msaa>>,
) {
    let water_settings = &tunables.water;
    let weather_settings = &tunables.weather;
    let scatter = &tunables.scatter;
    let columns = &tunables.columns;
    let Some(planet) = planet else {
        return;
    };
    let Some(atlas) = images.get(&art.0) else {
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
        let lod = &planet.lod;
        trace!(
            "planet view {entity}: camera {camera_position:?}, lod player {:?}, base {} fine {:?}",
            lod.player, planet.base_count, planet.counts
        );
        let params = PlanetParams {
            clip_from_body,
            camera: camera_position.extend(1.),
            sun: sun.direction().extend(1.),
            settings: Vec4::new(PLANET_RADIUS, planet.slots as f32, clock.0, foliage_range),
            water_absorption: Vec3::from_array(water_settings.absorption_per_m)
                .extend(PLANET_RADIUS - water_settings.depth_offset_m),
            water_deep: Vec3::from_array(water_settings.deep_color).extend(0.),
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
            lod_offsets: UVec4::new(planet.base_count, lod::FINE_CAPACITY, 0, 0),
            lod_counts: UVec4::from_array(planet.counts),
            lod: lod.player.extend(lod::BASE_LEVEL as f32),
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
                0.,
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
                0.,
            ),
            tilesets: tileset_slots(),
        };
        if let Some(mut gpu) = existing {
            gpu.uniform.set(params);
            gpu.uniform.write_buffer(&device, &queue);
            continue;
        }
        let mut uniform = UniformBuffer::from(params);
        uniform.write_buffer(&device, &queue);
        let visible = device.create_buffer(&BufferDescriptor {
            label: Some("GPU visible planet column IDs"),
            size: planet.slots as u64 * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let indirect = device.create_buffer(&BufferDescriptor {
            label: Some("GPU planet indirect draws: terrain, foliage, water, clutter, columns"),
            size: INDIRECT_BYTES,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });
        let foliage = device.create_buffer(&BufferDescriptor {
            label: Some("GPU nearby foliage column IDs"),
            size: planet.slots as u64 * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let water = device.create_buffer(&BufferDescriptor {
            label: Some("GPU visible water cell IDs"),
            size: planet.slots as u64 * 4,
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
            )),
        );
        let foliage_bind_group = view_bind_group("planet foliage view", &foliage);
        let clutter_bind_group = view_bind_group("planet clutter view", &clutter);
        let column_bind_group = view_bind_group("planet column view", &column_list);
        commands.entity(entity).insert(PlanetViewGpu {
            uniform,
            _visible: visible,
            _foliage: foliage,
            _clutter: clutter,
            _column_list: column_list,
            water,
            indirect,
            draw_bind_group,
            foliage_bind_group,
            clutter_bind_group,
            column_bind_group,
            compute_bind_group,
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_planet(
    draws: Res<DrawFunctions<Transparent3d>>,
    pipeline: Res<PlanetPipeline>,
    mut specialized: ResMut<SpecializedRenderPipelines<PlanetPipeline>>,
    cache: Res<PipelineCache>,
    mut phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa)>,
    surfaces: Query<(Entity, &MainEntity), With<PlanetSurface>>,
) {
    let draw = draws.read().id::<DrawPlanet>();
    for (view, msaa) in &views {
        let Some(phase) = phases.get_mut(&view.retained_view_entity) else {
            continue;
        };
        let pipeline = specialized.specialize(&cache, &pipeline, (msaa.samples(), view.hdr));
        for (entity, main_entity) in &surfaces {
            phase.add(Transparent3d {
                // Bevy sorts this phase ascending with values increasing toward
                // the camera, so the opaque globe goes first: at f32::MAX it
                // drew LAST and painted over every transparent mesh in front
                // of it, which is how the rain shower rendered as nothing.
                distance: f32::MIN,
                pipeline,
                entity: (entity, *main_entity),
                draw_function: draw,
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: false,
            });
        }
    }
}

type DrawPlanet = (SetItemPipeline, DrawPlanetIndirect);
struct DrawPlanetIndirect;
impl<P: PhaseItem> RenderCommand<P> for DrawPlanetIndirect {
    type Param = ();
    type ViewQuery = Option<&'static PlanetViewGpu>;
    type ItemQuery = ();
    fn render<'w>(
        _: &P,
        view: Option<&'w PlanetViewGpu>,
        _: Option<()>,
        _: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let Some(view) = view else {
            return RenderCommandResult::Skip;
        };
        pass.set_bind_group(0, &view.draw_bind_group, &[]);
        pass.draw_indirect(&view.indirect, 0);
        pass.set_bind_group(0, &view.foliage_bind_group, &[]);
        pass.draw_indirect(&view.indirect, 16);
        pass.set_bind_group(0, &view.clutter_bind_group, &[]);
        pass.draw_indirect(&view.indirect, 48);
        // The inside of the world, last: everything before it is the surface,
        // and a cave face is only ever seen through a hole in that surface.
        pass.set_bind_group(0, &view.column_bind_group, &[]);
        pass.draw_indirect(&view.indirect, 64);
        RenderCommandResult::Success
    }
}

#[cfg(test)]
mod pipeline_tests {
    use super::*;

    #[test]
    fn live_pixel_material_uses_unfiltered_integer_texels() {
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        assert!(shader.contains("textureLoad(atlas,"));
        assert!(!shader.contains("textureSample"));
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
        for (label, source) in [
            (
                "planet_surface.wgsl",
                include_str!("../../../assets/shaders/planet_surface.wgsl"),
            ),
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
            // 5 is the voxel sky light and 6 the layer materials: read
            // only, like the cells, the visible list and the column records
            // they are sampled beside.
            (draw_layout(), &[1, 2, 4, 5, 6][..]),
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
            pass.set_bind_group(0, &view.compute_bind_group, &[]);
            pass.set_pipeline(clear);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(compact);
            pass.dispatch_workgroups(planet.slots.div_ceil(128), 1, 1);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
