//! The sun's cascaded shadows (`sun-shadows`).
//!
//! Four depth maps, one layer of a texture array each, drawn from the sun
//! before the camera's pass: the planet's own records through its own
//! vertex shader and cull (decision 3), and each town's pieces from one
//! static buffer. The terrain reads them in its group 2 and what the field
//! lights reads them in its material (decision 4), both through
//! `sun_shadow.wgsl`, from one texture and one buffer of cascades.
//!
//! Where each cascade stands is `pbd_core::shadow`'s answer, fitted here in
//! the main world to the camera; which ones are redrawn this frame is its
//! schedule. The render world records the matrices each map was actually
//! drawn with, and those are what the receivers read, so a map not redrawn
//! and the matrices that read it never disagree.

use super::*;
use bevy::asset::RenderAssetUsages;
use bevy::image::{
    ImageAddressMode, ImageCompareFunction, ImageFilterMode, ImageSamplerDescriptor,
};
use bevy::render::render_resource::binding_types::{sampler, texture_2d_array};
use bevy::render::storage::{GpuShaderStorageBuffer, ShaderStorageBuffer};
use pbd_core::shadow::{self as fit, CASCADES, Cascade, TEXELS};

/// The maps' depth format: 32-bit, so a box pulled kilometres toward a low
/// sun keeps millimetres of depth.
pub const SHADOW_FORMAT: TextureFormat = TextureFormat::Depth32Float;

/// How the cascades are drawn and read. Tuned against the captures in
/// `docs/screenshots/sun-shadows/`.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ShadowSettings {
    /// Off draws no maps and lights everything as if in the sun.
    pub enabled: bool,
    /// How far a receiver is lifted along its normal before it looks itself
    /// up, in texels of the cascade it reads: what keeps a lit face from
    /// shadowing itself (acne).
    pub normal_offset_texels: f32,
    /// How much nearer the sun a receiver counts itself, in texels.
    pub depth_bias_texels: f32,
    /// The share of a cascade's box, at its edge, across which it blends into
    /// the next.
    pub blend: f32,
}

/// The casters' slope-scaled depth bias, set in their pipelines: a caster
/// steep to the sun pushed back by twice its own slope across a texel.
const SLOPE_BIAS: f32 = 2.0;

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            normal_offset_texels: 1.5,
            depth_bias_texels: 1.0,
            blend: 0.1,
        }
    }
}

/// `SunCascades` in `sun_shadow.wgsl`: each cascade's matrix, its texel and
/// depth in metres, and the settings.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct GpuCascades {
    pub clip_from_body: [[f32; 16]; CASCADES],
    pub texel_m: [f32; 4],
    pub depth_m: [f32; 4],
    pub settings: [f32; 4],
}

const CASCADES_BYTES: u64 = size_of::<GpuCascades>() as u64;

/// The maps and the buffer the receivers read, made once. Handles, so a
/// material can bind them as it binds a texture.
#[derive(Resource, Clone, ExtractResource)]
pub struct SunShadowMaps {
    pub map: Handle<Image>,
    pub cascades: Handle<ShaderStorageBuffer>,
}

/// This frame's cascades: where each stands as last drawn, and which are
/// redrawn now.
#[derive(Resource, Clone, Copy, Default, ExtractResource)]
pub struct SunCascades {
    pub drawn: fit::Drawn,
    /// The sun they were fitted to, for telling a jump in time from its drift.
    pub sun: Vec3,
    pub settings: Option<ShadowSettings>,
}

/// Each town's pieces, as the triangles that cast: its planet-frame positions,
/// by site. A town adds itself when it is spawned (`towns.rs`).
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct TownCasters(pub Vec<(u32, Arc<Vec<[f32; 3]>>)>);

/// Make the maps and the cascades' buffer.
fn create_maps(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut map = Image::new_uninit(
        Extent3d {
            width: TEXELS,
            height: TEXELS,
            depth_or_array_layers: CASCADES as u32,
        },
        TextureDimension::D2,
        SHADOW_FORMAT,
        RenderAssetUsages::RENDER_WORLD,
    );
    map.texture_descriptor.label = Some("sun cascades");
    map.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    map.texture_view_descriptor = Some(TextureViewDescriptor {
        label: Some("sun cascades"),
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
    });
    // A linear comparison: each tap is a bilinear 2 x 2 of lit and shadowed.
    map.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("sun cascades".into()),
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        address_mode_w: ImageAddressMode::ClampToEdge,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest,
        compare: Some(ImageCompareFunction::LessEqual),
        ..default()
    });
    let mut cascades = ShaderStorageBuffer::new(
        bytemuck::bytes_of(&GpuCascades::default()),
        RenderAssetUsages::RENDER_WORLD,
    );
    cascades.buffer_description.usage |= BufferUsages::COPY_DST;
    commands.insert_resource(SunShadowMaps {
        map: images.add(map),
        cascades: buffers.add(cascades),
    });
}

/// How far the sun may move between two frames before the cascades count it a
/// jump in time and redraw all four: a third of a degree, a few seconds of
/// the day at its own pace and one frame of the slider's.
const JUMP_COS: f32 = 0.999_983;

/// Fit the cascades to the camera, after it has moved.
fn fit_cascades(
    cameras: Query<(&Camera, &GlobalTransform, &Projection), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    sun: Option<Res<crate::sky::Sun>>,
    settings: Res<ShadowSettings>,
    mut cascades: ResMut<SunCascades>,
) {
    let (Some(frame), Some(sun)) = (frame, sun) else {
        return;
    };
    let Some((transform, projection)) = cameras
        .iter()
        .find(|(camera, ..)| camera.is_active)
        .map(|(_, t, p)| (t, p))
    else {
        return;
    };
    let Projection::Perspective(lens) = projection else {
        return;
    };
    let toward = sun.direction();
    let view = fit::View {
        eye: transform.translation().as_dvec3() - frame.center,
        forward: transform.forward().as_dvec3(),
        up: transform.up().as_dvec3(),
        fov_y: f64::from(lens.fov),
        aspect: f64::from(lens.aspect_ratio),
    };
    let jumped = cascades.sun.dot(toward) < JUMP_COS;
    let fitted = fit::fit(&view, toward.as_dvec3(), f64::from(PLANET_RADIUS));
    cascades.drawn.update(fitted, jumped);
    cascades.sun = toward;
    cascades.settings = Some(*settings);
}

/// The shadow pass's plugin: the maps, the fit, and the render world's
/// pipelines, bindings and draws (the draws run in `PlanetComputeNode`).
pub struct SunShadowPlugin;

impl Plugin for SunShadowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShadowSettings>()
            .init_resource::<SunCascades>()
            .init_resource::<TownCasters>()
            .add_plugins((
                ExtractResourcePlugin::<SunShadowMaps>::default(),
                ExtractResourcePlugin::<SunCascades>::default(),
                ExtractResourcePlugin::<TownCasters>::default(),
            ))
            .add_systems(Startup, (create_maps, load_module))
            .add_systems(PostUpdate, fit_cascades.after(TransformSystems::Propagate));
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ShadowState>()
            .add_systems(
                RenderStartup,
                initialize_shadow_pipelines.after(super::initialize_pipelines),
            )
            .add_systems(
                Render,
                prepare_shadows
                    .in_set(RenderSystems::PrepareBindGroups)
                    .after(super::prepare_views),
            );
    }
}

/// `shaders/sun_shadow.wgsl` is imported (`#import pbd::sun_shadow`), never
/// drawn, so nothing loads it unless this does.
#[derive(Resource)]
struct ShadowModule(#[allow(dead_code)] Handle<Shader>);

fn load_module(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(ShadowModule(assets.load("shaders/sun_shadow.wgsl")));
}

/// The terrain's group 2: the maps, their comparison sampler and the
/// cascades.
pub(super) fn receiver_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "planet sun shadows",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d_array(TextureSampleType::Depth),
                sampler(SamplerBindingType::Comparison),
                storage_buffer_read_only_sized(false, NonZeroU64::new(CASCADES_BYTES)),
            ),
        ),
    )
}

fn caster_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "town caster",
        &BindGroupLayoutEntries::single(ShaderStages::VERTEX, uniform_buffer::<Mat4>(false)),
    )
}

#[derive(Resource)]
pub(super) struct ShadowPipelines {
    planet: CachedRenderPipelineId,
    town: CachedRenderPipelineId,
    caster_layout: BindGroupLayoutDescriptor,
    receiver_layout: BindGroupLayoutDescriptor,
}

fn depth_state() -> DepthStencilState {
    DepthStencilState {
        format: SHADOW_FORMAT,
        depth_write_enabled: true,
        depth_compare: CompareFunction::LessEqual,
        stencil: default(),
        bias: DepthBiasState {
            constant: 0,
            slope_scale: SLOPE_BIAS,
            clamp: 0.0,
        },
    }
}

fn initialize_shadow_pipelines(
    mut commands: Commands,
    assets: Res<AssetServer>,
    cache: Res<PipelineCache>,
    planet: Res<PlanetPipeline>,
) {
    let caster_layout = caster_layout();
    // Both faces cast: a blade wound toward the camera and a wall seen from
    // behind are casters all the same.
    let primitive = PrimitiveState {
        cull_mode: None,
        ..default()
    };
    let planet_id = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some(Cow::Borrowed("sun cascade: planet")),
        layout: vec![planet.draw_layout.clone()],
        vertex: VertexState {
            shader: planet.shader.clone(),
            entry_point: Some(Cow::Borrowed("vertex")),
            ..default()
        },
        fragment: Some(FragmentState {
            shader: planet.shader.clone(),
            entry_point: Some(Cow::Borrowed("shadow_fragment")),
            targets: vec![],
            ..default()
        }),
        primitive,
        depth_stencil: Some(depth_state()),
        ..default()
    });
    let town_shader = assets.load("shaders/shadow_caster.wgsl");
    let town_id = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some(Cow::Borrowed("sun cascade: towns")),
        layout: vec![caster_layout.clone()],
        vertex: VertexState {
            shader: town_shader,
            entry_point: Some(Cow::Borrowed("vertex")),
            buffers: vec![bevy::mesh::VertexBufferLayout {
                array_stride: 12,
                step_mode: VertexStepMode::Vertex,
                attributes: vec![VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            }],
            ..default()
        },
        fragment: None,
        primitive,
        depth_stencil: Some(depth_state()),
        ..default()
    });
    commands.insert_resource(ShadowPipelines {
        planet: planet_id,
        town: town_id,
        caster_layout,
        receiver_layout: receiver_layout(),
    });
}

/// A town's triangles on the GPU.
struct TownBuffer {
    site: u32,
    source: Arc<Vec<[f32; 3]>>,
    buffer: Buffer,
    vertices: u32,
}

/// What the render world keeps between frames: the matrices each map was
/// last drawn with, the maps' layer views, the towns' buffers, and the
/// receivers' bind group.
#[derive(Resource, Default)]
pub(super) struct ShadowState {
    drawn: [Option<Cascade>; CASCADES],
    layers: Vec<TextureView>,
    texture: Option<TextureId>,
    towns: Vec<TownBuffer>,
    pub(super) receiver: Option<BindGroup>,
}

/// One view's cascades: a uniform, a cull and three draws per cascade, and
/// one set of lists they take turns with, since each cascade's cull runs
/// just before its own draw.
#[derive(Component)]
pub(super) struct PlanetShadowGpu {
    uniforms: Vec<UniformBuffer<PlanetParams>>,
    _lists: Vec<Buffer>,
    indirect: Buffer,
    compute: Vec<BindGroup>,
    terrain: Vec<BindGroup>,
    foliage: Vec<BindGroup>,
    column: Vec<BindGroup>,
    casters: Vec<(UniformBuffer<Mat4>, BindGroup)>,
    /// Which cascades are drawn this frame.
    draw: [bool; CASCADES],
}

#[allow(clippy::too_many_arguments)]
fn prepare_shadows(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    cache: Res<PipelineCache>,
    pipelines: Option<Res<ShadowPipelines>>,
    planet_pipeline: Option<Res<PlanetPipeline>>,
    planet: Option<Res<PlanetGpu>>,
    art: Option<Res<PlanetArt>>,
    images: Res<RenderAssets<GpuImage>>,
    storage: Res<RenderAssets<GpuShaderStorageBuffer>>,
    maps: Option<Res<SunShadowMaps>>,
    cascades: Option<Res<SunCascades>>,
    casters: Option<Res<TownCasters>>,
    mut state: ResMut<ShadowState>,
    mut views: Query<(Entity, &PlanetViewGpu, Option<&mut PlanetShadowGpu>)>,
) {
    let (Some(pipelines), Some(maps), Some(planet), Some(planet_pipeline), Some(art)) =
        (pipelines, maps, planet, planet_pipeline, art)
    else {
        return;
    };
    let cascades = cascades.map(|c| *c).unwrap_or_default();
    let casters = casters.map(|c| c.clone()).unwrap_or_default();
    let (Some(map), Some(buffer), Some(atlas)) = (
        images.get(&maps.map),
        storage.get(&maps.cascades),
        images.get(&art.0),
    ) else {
        return;
    };
    // The maps' layers, and the receivers' bind group, once per texture.
    if state.texture != Some(map.texture.id()) {
        state.layers = (0..CASCADES as u32)
            .map(|layer| {
                map.texture.create_view(&TextureViewDescriptor {
                    label: Some("sun cascade layer"),
                    dimension: Some(TextureViewDimension::D2),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                    ..default()
                })
            })
            .collect();
        state.receiver = Some(device.create_bind_group(
            Some("planet sun shadows"),
            &cache.get_bind_group_layout(&pipelines.receiver_layout),
            &BindGroupEntries::sequential((
                &map.texture_view,
                &map.sampler,
                buffer.buffer.as_entire_binding(),
            )),
        ));
        state.texture = Some(map.texture.id());
    }
    // The towns' triangles, uploaded once a town.
    state.towns.retain(|t| {
        casters
            .0
            .iter()
            .any(|(site, source)| *site == t.site && Arc::ptr_eq(source, &t.source))
    });
    for (site, source) in &casters.0 {
        if state.towns.iter().any(|t| t.site == *site) || source.is_empty() {
            continue;
        }
        state.towns.push(TownBuffer {
            site: *site,
            source: source.clone(),
            buffer: device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("town caster"),
                contents: bytemuck::cast_slice(source.as_slice()),
                usage: BufferUsages::VERTEX,
            }),
            vertices: source.len() as u32,
        });
    }
    let settings = cascades.settings.unwrap_or_default();
    let ready = cache.get_render_pipeline(pipelines.planet).is_some()
        && cache.get_render_pipeline(pipelines.town).is_some();
    let fitted = cascades
        .drawn
        .cascades
        .filter(|_| settings.enabled && ready);
    let mut draw = [false; CASCADES];
    if let Some(fitted) = fitted {
        for i in 0..CASCADES {
            // A map never drawn is drawn now, whatever the schedule.
            draw[i] = cascades.drawn.redraw[i] || state.drawn[i].is_none();
            if draw[i] {
                state.drawn[i] = Some(fitted[i]);
            }
        }
    }
    // What the receivers read: the matrices each map was drawn with, and no
    // shadow at all until every map has been drawn once.
    let mut gpu = GpuCascades::default();
    let whole = settings.enabled && state.drawn.iter().all(Option::is_some);
    for (i, drawn) in state.drawn.iter().enumerate() {
        if let Some(c) = drawn {
            gpu.clip_from_body[i] = c.clip_from_body.as_mat4().to_cols_array();
            gpu.texel_m[i] = c.texel_m as f32;
            gpu.depth_m[i] = c.depth_m as f32;
        }
    }
    gpu.settings = [
        if whole { 1.0 } else { 0.0 },
        settings.normal_offset_texels,
        settings.depth_bias_texels,
        settings.blend,
    ];
    queue.write_buffer(&buffer.buffer, 0, bytemuck::bytes_of(&gpu));

    for (entity, view, existing) in &mut views {
        let camera = view.uniform.get().clone();
        let params_for = |i: usize| {
            let mut p = camera.clone();
            if let Some(c) = &state.drawn[i] {
                p.clip_from_body = c.clip_from_body.as_mat4();
            }
            p.fade.w = 1.0;
            p
        };
        if let Some(mut gpu) = existing {
            for (i, _) in draw.iter().enumerate().filter(|(_, d)| **d) {
                gpu.uniforms[i].set(params_for(i));
                gpu.uniforms[i].write_buffer(&device, &queue);
                gpu.casters[i]
                    .0
                    .set(state.drawn[i].unwrap().clip_from_body.as_mat4());
                gpu.casters[i].0.write_buffer(&device, &queue);
            }
            gpu.draw = draw;
            continue;
        }
        let list = |label: &'static str, bytes: u64| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size: bytes.max(4),
                usage: BufferUsages::STORAGE,
                mapped_at_creation: false,
            })
        };
        let slots = u64::from(planet.slots);
        let visible = list("sun cascade terrain IDs", slots * 8);
        let foliage = list("sun cascade tree IDs", slots * 8);
        let column = list("sun cascade column IDs", slots * 4);
        // The sea and the clutter are not listed for a cascade; the cull
        // still binds somewhere for them.
        let water = list("sun cascade (no sea)", 4);
        let clutter = list("sun cascade (no clutter)", 4);
        let indirect = device.create_buffer(&BufferDescriptor {
            label: Some("sun cascade indirect draws"),
            size: INDIRECT_BYTES,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT,
            mapped_at_creation: false,
        });
        let mut uniforms = Vec::new();
        let (mut compute, mut terrain, mut trees, mut columns, mut town) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for i in 0..CASCADES {
            let mut uniform = UniformBuffer::from(params_for(i));
            uniform.write_buffer(&device, &queue);
            let draw_group = |label: &'static str, list: &Buffer| {
                device.create_bind_group(
                    Some(label),
                    &cache.get_bind_group_layout(&planet_pipeline.draw_layout),
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
            terrain.push(draw_group("sun cascade terrain", &visible));
            trees.push(draw_group("sun cascade trees", &foliage));
            columns.push(draw_group("sun cascade columns", &column));
            compute.push(device.create_bind_group(
                Some("sun cascade cull"),
                &cache.get_bind_group_layout(&planet_pipeline.compute_layout),
                &BindGroupEntries::sequential((
                    &uniform,
                    planet.cells.as_entire_binding(),
                    visible.as_entire_binding(),
                    indirect.as_entire_binding(),
                    foliage.as_entire_binding(),
                    water.as_entire_binding(),
                    clutter.as_entire_binding(),
                    column.as_entire_binding(),
                )),
            ));
            let mut caster = UniformBuffer::from(
                state.drawn[i].map_or(Mat4::IDENTITY, |c| c.clip_from_body.as_mat4()),
            );
            caster.write_buffer(&device, &queue);
            let group = device.create_bind_group(
                Some("town caster"),
                &cache.get_bind_group_layout(&pipelines.caster_layout),
                &BindGroupEntries::single(&caster),
            );
            town.push((caster, group));
            uniforms.push(uniform);
        }
        commands.entity(entity).insert(PlanetShadowGpu {
            uniforms,
            _lists: vec![visible, foliage, column, water, clutter],
            indirect,
            compute,
            terrain,
            foliage: trees,
            column: columns,
            casters: town,
            draw,
        });
    }
}

/// Draw this frame's cascades for one view: each one's cull, then its
/// terrain, trees, column tier and towns into its layer. Called from
/// `PlanetComputeNode` after the camera's own cull.
pub(super) fn draw_cascades(
    ctx: &mut RenderContext,
    world: &World,
    entity: Entity,
    clear: &ComputePipeline,
    compact: &ComputePipeline,
    slots: u32,
) {
    let (Some(gpu), Some(state), Some(pipelines)) = (
        world.get::<PlanetShadowGpu>(entity),
        world.get_resource::<ShadowState>(),
        world.get_resource::<ShadowPipelines>(),
    ) else {
        return;
    };
    if !gpu.draw.iter().any(|d| *d) || state.layers.len() != CASCADES {
        return;
    }
    let cache = world.resource::<PipelineCache>();
    let (Some(planet), Some(town)) = (
        cache.get_render_pipeline(pipelines.planet),
        cache.get_render_pipeline(pipelines.town),
    ) else {
        return;
    };
    let diagnostics = ctx.diagnostic_recorder();
    let span = diagnostics.time_span(ctx.command_encoder(), "planet_shadow");
    for i in 0..CASCADES {
        if !gpu.draw[i] {
            continue;
        }
        {
            let mut pass = ctx
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor {
                    label: Some("Cull hex columns for a sun cascade"),
                    ..default()
                });
            pass.set_bind_group(0, &*gpu.compute[i], &[]);
            pass.set_pipeline(clear);
            pass.dispatch_workgroups(1, 1, 1);
            pass.set_pipeline(compact);
            pass.dispatch_workgroups(slots.div_ceil(128), 1, 1);
        }
        let mut pass = ctx
            .command_encoder()
            .begin_render_pass(&RenderPassDescriptor {
                label: Some("sun cascade"),
                color_attachments: &[],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &state.layers[i],
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        pass.set_pipeline(planet);
        pass.set_bind_group(0, &*gpu.terrain[i], &[]);
        pass.draw_indirect(&gpu.indirect, 0);
        pass.set_bind_group(0, &*gpu.foliage[i], &[]);
        pass.draw_indirect(&gpu.indirect, 16);
        pass.set_bind_group(0, &*gpu.column[i], &[]);
        pass.draw_indirect(&gpu.indirect, 64);
        pass.set_pipeline(town);
        pass.set_bind_group(0, &*gpu.casters[i].1, &[]);
        for t in &state.towns {
            pass.set_vertex_buffer(0, (*t.buffer).slice(..));
            pass.draw(0..t.vertices, 0..1);
        }
    }
    span.end(ctx.command_encoder());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cascades' buffer is the struct `sun_shadow.wgsl` declares, as naga
    /// lays it out.
    #[test]
    fn the_cascades_buffer_is_the_shaders_struct() {
        let source = crate::shader_tests::sun_shadow_source();
        assert_eq!(
            CASCADES_BYTES as u32,
            crate::shader_tests::wgsl_struct_size("sun_shadow.wgsl", &source, "SunCascades")
        );
    }

    /// The pass is timed as a top-level span of its own, `planet_shadow`,
    /// which `--frame-log`'s GPU total sums with the others
    /// (`desktop::gpu_times`) and `tools/perf_suite.py` reports.
    #[test]
    fn the_shadow_pass_is_timed_as_its_own_span() {
        let source = include_str!("planet_shadow.rs");
        assert!(source.contains(&format!(
            "time_span(ctx.command_encoder(), \"{}\")",
            "planet_shadow"
        )));
    }

    /// The town caster's shader parses and validates, and names the entry
    /// its pipeline asks for.
    #[test]
    fn the_caster_shader_compiles() {
        let source = include_str!("../../../assets/shaders/shadow_caster.wgsl");
        let module = naga::front::wgsl::parse_str(source).expect("shadow_caster.wgsl parses");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("shadow_caster.wgsl validates");
        assert!(module.entry_points.iter().any(|e| e.name == "vertex"));
    }
}
