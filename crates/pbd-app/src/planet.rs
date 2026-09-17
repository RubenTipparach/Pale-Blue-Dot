//! A complete, closed hex/pentagon world drawn from persistent GPU topology.
//!
//! CPU columns are authoritative. A compute pass compacts visible column IDs
//! and writes an indirect draw; the vertex shader reconstructs caps, exposed
//! walls, and decorative trees. No expanded terrain vertex buffer is uploaded.
//! This is a surface-column prototype, not the editable volumetric chunk engine.

#[path = "planet_contact.rs"]
mod contact;
#[path = "planet_terrain.rs"]
mod terrain;
#[path = "planet_topology.rs"]
mod topology;
#[cfg(test)]
#[path = "planet_visibility_tests.rs"]
mod visibility_tests;

pub use contact::{PlanetContact, SurfaceContact};
pub use terrain::{ELEVATION_STEP, PLANET_RADIUS, surface_height, terrain_radius};

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

const SUBDIVISIONS: u32 = 8;
// At this sea-level altitude even the highest preview summit and its trees
// are beyond the shader's 2,300 m decorative-geometry range.
const FOLIAGE_DRAW_CUTOFF_ALTITUDE: f32 = 3_200.;
const FOLIAGE_DRAW_DISTANCE: f32 = 2_300.;

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
#[derive(Clone, Copy, Pod, Zeroable)]
struct GpuCell {
    direction_height: [f32; 4],
    // xyz is the shared corner ray; w is this edge's adjacent surface height.
    corners: [[f32; 4]; 6],
    // Degree, explicit biome, baked sky occlusion (0..65535), stable cell ID.
    metadata: [u32; 4],
}

// Match the WGSL Cell storage ABI. These inspect the actual upload type; the
// standalone shader validator separately inspects the parsed shader layouts.
const _: [(); 128] = [(); size_of::<GpuCell>()];
const _: [(); 16] = [(); std::mem::offset_of!(GpuCell, corners)];
const _: [(); 112] = [(); std::mem::offset_of!(GpuCell, metadata)];

#[derive(Resource, Clone, ExtractResource)]
struct PlanetColumns(Arc<Vec<GpuCell>>);

#[derive(Resource, Clone, Default, ExtractResource)]
struct PlanetClock(f32);

#[derive(Resource, Clone, ExtractResource)]
struct PlanetArt(Handle<Image>);

#[derive(Component, Clone, Copy, ExtractComponent)]
struct PlanetSurface;

/// Installs CPU generation, persistent storage buffers and actual GPU pipelines.
pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractResourcePlugin::<PlanetColumns>::default(),
            ExtractResourcePlugin::<PlanetClock>::default(),
            ExtractResourcePlugin::<PlanetArt>::default(),
            ExtractResourcePlugin::<PlanetRenderFrame>::default(),
            ExtractComponentPlugin::<PlanetSurface>::default(),
        ))
        .init_resource::<PlanetClock>()
        .init_resource::<PlanetRenderFrame>()
        .add_systems(Startup, create_planet)
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
                    upload_planet.in_set(RenderSystems::PrepareResources),
                    prepare_views.in_set(RenderSystems::PrepareBindGroups),
                    queue_planet.in_set(RenderSystems::QueueMeshes),
                ),
            );
        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(PlanetComputeLabel, PlanetComputeNode::default());
        graph.add_node_edge(PlanetComputeLabel, bevy::render::graph::CameraDriverLabel);
    }
}

fn create_planet(mut commands: Commands, assets: Res<AssetServer>) {
    let started = std::time::Instant::now();
    let cells = topology::dual_sphere(SUBDIVISIONS);
    let columns = Arc::new(generate_columns(&cells));
    let contacts = PlanetContact::new(columns.clone(), &cells);
    let (narrowest, mean_width, widest) = tile_widths(&cells);
    info!(
        "Planet ready: {} columns, 12 pentagons, {:.1} MiB topology, {:.2}s generation",
        columns.len(),
        (columns.len() * size_of::<GpuCell>()) as f64 / 1_048_576.,
        started.elapsed().as_secs_f64()
    );
    info!(
        "Planet scale: L{} on r={:.0} m gives {:.2} m mean tile width ({:.2}-{:.2} m), \
         {:.0} m elevation step; the walker's eye is {:.2} m",
        SUBDIVISIONS,
        PLANET_RADIUS,
        mean_width,
        narrowest,
        widest,
        ELEVATION_STEP,
        crate::walking::EYE_HEIGHT,
    );
    commands.insert_resource(contacts);
    commands.insert_resource(PlanetColumns(columns));
    commands.insert_resource(PlanetArt(assets.load_with_settings(
        "tilesets/fields.png",
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

fn generate_columns(cells: &[topology::DualCell]) -> Vec<GpuCell> {
    let heights: Vec<_> = cells.iter().map(|c| surface_height(c.direction)).collect();
    cells
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let height = heights[index];
            let mut corners = [[0.; 4]; 6];
            let mut occlusion = 0.;
            for (side, corner) in cell.corners.iter().enumerate() {
                let neighbor_height = heights[cell.neighbors[side]].max(0.);
                corners[side] = [corner.x, corner.y, corner.z, neighbor_height];
                let separation = tile_width(cell, &cells[cell.neighbors[side]]);
                occlusion +=
                    ((neighbor_height - height.max(0.)) / separation.max(1.)).clamp(0., 1.);
            }
            let skylight = 1. - occlusion / cell.corners.len() as f32 * 0.55;
            GpuCell {
                direction_height: [cell.direction.x, cell.direction.y, cell.direction.z, height],
                corners,
                metadata: [
                    cell.corners.len() as u32,
                    terrain::biome(cell.direction, height),
                    (skylight * 65535.) as u32,
                    index as u32,
                ],
            }
        })
        .collect()
}

#[derive(Clone, ShaderType)]
struct PlanetParams {
    clip_from_body: Mat4,
    camera: Vec4,
    sun: Vec4,
    // Sea radius, cell count, elapsed seconds, foliage range (zero disables it).
    settings: Vec4,
}

#[derive(Resource)]
struct PlanetGpu {
    cells: Buffer,
    count: u32,
}

#[derive(Component)]
struct PlanetViewGpu {
    uniform: UniformBuffer<PlanetParams>,
    // Held by bind groups as well; retained explicitly to make lifetime clear.
    _visible: Buffer,
    _foliage: Buffer,
    indirect: Buffer,
    draw_bind_group: BindGroup,
    foliage_bind_group: BindGroup,
    compute_bind_group: BindGroup,
}

fn upload_planet(
    mut commands: Commands,
    columns: Option<Res<PlanetColumns>>,
    existing: Option<Res<PlanetGpu>>,
    device: Res<RenderDevice>,
) {
    if existing.is_some() {
        return;
    }
    let Some(columns) = columns else {
        return;
    };
    let cells = device.create_buffer_with_data(&BufferInitDescriptor {
        label: Some("Persistent planet columns: directions, corner rays, materials"),
        contents: bytemuck::cast_slice(columns.0.as_slice()),
        usage: BufferUsages::STORAGE,
    });
    commands.insert_resource(PlanetGpu {
        cells,
        count: columns.0.len() as u32,
    });
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
                storage_buffer_sized(false, NonZeroU64::new(32)),
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
    frame: Res<PlanetRenderFrame>,
    mut views: Query<(Entity, &ExtractedView, Option<&mut PlanetViewGpu>), With<Msaa>>,
) {
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
        let params = PlanetParams {
            clip_from_body,
            camera: camera_position.extend(1.),
            sun: crate::sky::SUN_DIRECTION.normalize().extend(1.),
            settings: Vec4::new(PLANET_RADIUS, planet.count as f32, clock.0, foliage_range),
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
            size: planet.count as u64 * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let indirect = device.create_buffer(&BufferDescriptor {
            label: Some("GPU planet indirect draw arguments"),
            size: 32,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });
        let foliage = device.create_buffer(&BufferDescriptor {
            label: Some("GPU nearby foliage column IDs"),
            size: planet.count as u64 * 4,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let draw_bind_group = device.create_bind_group(
            Some("planet surface view"),
            &cache.get_bind_group_layout(&pipeline.draw_layout),
            &BindGroupEntries::sequential((
                &uniform,
                planet.cells.as_entire_binding(),
                visible.as_entire_binding(),
                &atlas.texture_view,
            )),
        );
        let compute_bind_group = device.create_bind_group(
            Some("planet visibility view"),
            &cache.get_bind_group_layout(&pipeline.compute_layout),
            &BindGroupEntries::sequential((
                &uniform,
                planet.cells.as_entire_binding(),
                visible.as_entire_binding(),
                indirect.as_entire_binding(),
                foliage.as_entire_binding(),
            )),
        );
        let foliage_bind_group = device.create_bind_group(
            Some("planet foliage view"),
            &cache.get_bind_group_layout(&pipeline.draw_layout),
            &BindGroupEntries::sequential((
                &uniform,
                planet.cells.as_entire_binding(),
                foliage.as_entire_binding(),
                &atlas.texture_view,
            )),
        );
        commands.entity(entity).insert(PlanetViewGpu {
            uniform,
            _visible: visible,
            _foliage: foliage,
            indirect,
            draw_bind_group,
            foliage_bind_group,
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
                distance: f32::MAX,
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
        assert_eq!(
            PlanetParams::min_size().get(),
            112,
            "actual encoded Rust uniform must match WGSL Params"
        );
        for (layout, read_only_bindings) in
            [(draw_layout(), &[1, 2][..]), (compute_layout(), &[1][..])]
        {
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
            pass.dispatch_workgroups(planet.count.div_ceil(128), 1, 1);
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

        // What the shipped configuration actually is, measured AT that level
        // rather than extrapolated to it. The extrapolation is the cheap check
        // above; this is the number the startup log prints, and pinning the
        // extrapolation instead is what let an f32 accumulator under-report the
        // real globe by 1.2% without failing anything. About 0.9 s in debug.
        let shipped = tile_widths(&topology::dual_sphere(SUBDIVISIONS)).1;
        assert!(
            (shipped - 18.886).abs() < 0.01,
            "shipped tile width {shipped} m at L{SUBDIVISIONS} on r={PLANET_RADIUS} m"
        );
        // And the extrapolation must agree with it, which is what fails if the
        // accumulator loses precision at the level that has the most terms.
        let extrapolated = fine / 2_f32.powi(SUBDIVISIONS as i32 - 5);
        assert!(
            (extrapolated / shipped - 1.).abs() < 0.002,
            "extrapolated {extrapolated} m vs measured {shipped} m"
        );
        assert!(shipped > crate::walking::EYE_HEIGHT * 10.);
    }
}
