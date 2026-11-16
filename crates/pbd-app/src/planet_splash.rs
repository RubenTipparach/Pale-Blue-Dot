//! Splashes where rain lands (`weather-rain` 6.4, the approved M32 mockup).
//!
//! GPU particles with no state: `rain_splash.wgsl` computes every droplet's
//! whole life from its lane index and the clock, in one instanced draw after
//! the terrain, alpha blended, depth tested and writing no depth. The CPU's
//! share is the NEAR COLUMN LIST, the hex columns within the reach of the
//! eye: where each one's topmost up-face is, what it is made of and a stable
//! key for its slots. It is rebuilt and uploaded only when the camera
//! crosses a cell or the fine set changes (a landing or an edit), never per
//! frame; the draw itself is skipped on the CPU when no rain falls.
//!
//! The gates are the near shower's and the wet ground's: rain (liquid, not
//! snow), the altitude gate, not from under the sea, column tops only, none
//! under a roof (a column's top is by construction open to the sky; a town
//! building that shelters it takes its column out of the list), none where
//! water stands over the top, and a fade over the last metres of
//! `rain_detail_reach_m`. Wetness gates the crowns through the terrain's own
//! puddle mask in the shader.

use super::{PLANET_RADIUS, lod::FineSet, terrain::render_code};
use super::{PlanetContact, PlanetFine, PlanetRenderFrame, PlanetSurface, PlanetViewGpu};
use crate::config::WeatherSettings;
use bevy::{
    core_pipeline::core_3d::{
        CORE_3D_DEPTH_FORMAT,
        graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        Render, RenderStartup, RenderSystems,
        diagnostic::RecordDiagnostics,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{storage_buffer_read_only_sized, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
    },
};
use bytemuck::{Pod, Zeroable};
use pbd_core::column::layer_altitude;
use std::{borrow::Cow, num::NonZeroU64, sync::Arc};

/// The most columns the list holds: about 220 sit within 22 m at 2.833 m
/// cells, so this is twice that with room. The buffer is made once at this
/// size; a list that would pass it keeps its nearest columns.
pub const MAX_COLUMNS: usize = 512;

/// How far past the reach the list runs, metres: the camera moves up to half
/// a cell before the list is rebuilt, and the fade is measured from the eye.
const LIST_MARGIN_M: f32 = 2.0;

/// One near column, as the shader reads it (`SplashColumn`).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug, PartialEq)]
pub struct GpuSplashColumn {
    /// The centre of the topmost up-face, body-local; w the radius of the
    /// circle inside that face.
    pub top: [f32; 4],
    /// The face's render code and biome (`code | biome << 8`), the class
    /// (0 grass, 1 bare ground), the slot key, spare.
    pub info: [u32; 4],
}

/// The near column list and this frame's gate, extracted to the render world.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct RainSplashes {
    pub columns: Arc<Vec<GpuSplashColumn>>,
    /// Bumped on every rebuild; the render world uploads when it moves.
    pub version: u64,
    /// The liquid rain here, 0..1; zero skips the draw.
    pub rain: f32,
    /// Whether the splashes are drawn this frame at all.
    pub on: bool,
    /// The body-local eye the list was gated from this frame.
    pub eye: Vec3,
}

/// What the list was built for: the fine set's version and the camera's cell.
#[derive(Default)]
struct ListKey(Option<(u64, usize)>);

/// The class of a top: grass sprays, every other face is bare ground (a crown
/// where the puddle mask has standing water). The terrain's own `grassy`,
/// with its leaf code (9), which takes no puddle either.
pub fn splash_class(code: u32) -> u32 {
    match code & 0xff {
        2 | 3 | 7 | 9 => 0,
        _ => 1,
    }
}

/// A stable ten-bit key for a cell's slots, from its stable id: a slot keeps
/// its rhythm when the list is rebuilt in another order. Ten bits times the
/// 64 slots a column may have keeps the key exact in an f32.
pub fn slot_key(cell_id: u32) -> u32 {
    cell_id.wrapping_mul(0x9e37_79b9) >> 22
}

/// The near columns round a body-local `eye`: a walk out from the eye's cell
/// over the finest neighbours, keeping the cells whose centre is within
/// `reach_m` along the ground, nearest-first order by the walk. `sheltered`
/// says whether a body-local point is under a building's roof.
pub fn near_columns(
    set: &FineSet,
    start: usize,
    eye: Vec3,
    reach_m: f32,
    sheltered: &dyn Fn(Vec3) -> bool,
) -> Vec<GpuSplashColumn> {
    let records = set.finest_records();
    let up = eye.normalize_or(Vec3::Y);
    let cos_reach = (reach_m / PLANET_RADIUS).cos();
    let mut seen = std::collections::HashSet::new();
    let mut pending = std::collections::VecDeque::from([start]);
    seen.insert(start);
    let mut out = Vec::new();
    while let Some(id) = pending.pop_front() {
        if out.len() >= MAX_COLUMNS {
            break;
        }
        let Some(cell) = records.get(id) else {
            continue;
        };
        let axis = Vec3::from_slice(&cell.direction_height[..3]);
        if axis.dot(up) < cos_reach {
            continue;
        }
        let neighbors = set
            .finest_neighbors
            .get(id)
            .copied()
            .unwrap_or([u32::MAX; 6]);
        for &n in &neighbors[..cell.degree().min(6)] {
            if n != u32::MAX && seen.insert(n as usize) {
                pending.push_back(n as usize);
            }
        }
        if let Some(column) = top_of(set, id) {
            let top = axis * (PLANET_RADIUS + column.0);
            if sheltered(top + axis * 0.3) {
                continue;
            }
            out.push(GpuSplashColumn {
                top: top
                    .extend(inner_radius(cell, PLANET_RADIUS + column.0) * 0.97)
                    .to_array(),
                info: [
                    column.1,
                    splash_class(column.1),
                    slot_key(cell.metadata[3]),
                    0,
                ],
            });
        }
    }
    out
}

/// A cell's topmost up-face: its altitude, metres against sea level, and
/// its code (`code | biome << 8`). `None` where nothing stands to be rained
/// on: the sea's cap, or water standing over the top.
fn top_of(set: &FineSet, id: usize) -> Option<(f32, u32)> {
    let cell = set.finest_records().get(id)?;
    let height = cell.direction_height[3];
    let record_code = cell.metadata[1] & 0xffff;
    let Some(column) = set.columns.column(id) else {
        // Off the column tier: the heightfield's cap.
        return (height >= 0.0).then_some((height, record_code));
    };
    let top = column.surface()?;
    if column.material(top + 1).is_water() {
        return None;
    }
    let altitude = layer_altitude(top) + 1.0;
    // The untouched surface wears the record's code (its biome's own
    // grass); a top a player made wears its own material.
    let code = if (altitude - height).abs() < 0.01 {
        record_code
    } else {
        render_code(column.material(top)) | (record_code & 0xff00)
    };
    Some((altitude, code))
}

/// The radius of the circle inside a cell's cap at `radius`: the nearest of
/// its edges to its centre.
fn inner_radius(cell: &super::GpuCell, radius: f32) -> f32 {
    let centre = Vec3::from_slice(&cell.direction_height[..3]) * radius;
    let degree = cell.degree().clamp(3, 6);
    (0..degree)
        .map(|side| {
            let a = Vec3::from_slice(&cell.corners[side][..3]) * radius;
            let b = Vec3::from_slice(&cell.corners[(side + 1) % degree][..3]) * radius;
            let along = (b - a).normalize_or_zero();
            let to = centre - a;
            (to - along * to.dot(along)).length()
        })
        .fold(f32::MAX, f32::min)
}

/// `PBD_PASS_OFF=splash`: the splashes skipped, a pricing instrument beside
/// the shower's.
static SPLASH_OFF: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| {
    std::env::var("PBD_PASS_OFF").is_ok_and(|list| list.split(',').any(|p| p.trim() == "splash"))
});

#[allow(clippy::too_many_arguments)]
fn update_splashes(
    settings: Option<Res<WeatherSettings>>,
    weather: Option<Res<crate::weather::Weather>>,
    frame: Res<PlanetRenderFrame>,
    contact: Option<Res<PlanetContact>>,
    fine: Option<Res<PlanetFine>>,
    structures: Option<Res<crate::walking::Structures>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut splashes: ResMut<RainSplashes>,
    mut key: Local<ListKey>,
) {
    let active = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform);
    let (Some(camera), Some(contact), Some(fine), Some(settings), Some(weather)) =
        (active, contact, fine, settings, weather)
    else {
        splashes.on = false;
        return;
    };
    let eye = (camera.translation().as_dvec3() - frame.center).as_vec3();
    let radius = eye.length();
    let up = eye / radius.max(1e-3);
    let submerged = super::surface_height(up) < 0.0 && radius < PLANET_RADIUS;
    let rain = weather.liquid();
    splashes.rain = rain;
    splashes.eye = eye;
    splashes.on = rain > 0.0
        && radius >= 1.0
        && !submerged
        && !*SPLASH_OFF
        && crate::weather::rain_drawn_at(crate::weather::height_above_ground(eye), &settings);
    if !splashes.on || !contact.serves(&fine.set) {
        return;
    }
    let Some(start) = contact.finest_cell(up) else {
        splashes.on = false;
        return;
    };
    let now = Some((fine.version, start));
    if key.0 == now {
        return;
    }
    key.0 = now;
    let reach = settings.rain_detail_reach_m + LIST_MARGIN_M;
    let shelter = |p: Vec3| {
        structures
            .as_ref()
            .is_some_and(|s| s.0.iter().any(|b| b.shelters(p)))
    };
    splashes.columns = Arc::new(near_columns(&fine.set, start, eye, reach, &shelter));
    splashes.version += 1;
}

// ---- The render world --------------------------------------------------

/// The view's uniform, as `rain_splash.wgsl` declares `Splash`.
#[derive(ShaderType, Clone, Copy, Default)]
struct SplashUniform {
    shape: UVec4,
    timing: Vec4,
    size_m: Vec4,
    up_mps: Vec4,
    out_mps: Vec4,
    keep: Vec4,
    view: Vec4,
    eye: Vec4,
}

#[derive(Resource)]
struct SplashPipeline {
    shader: Handle<Shader>,
    layout: BindGroupLayoutDescriptor,
}

impl SpecializedRenderPipeline for SplashPipeline {
    /// MSAA samples and HDR.
    type Key = (u32, bool);
    fn specialize(&self, (samples, hdr): Self::Key) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some(Cow::Borrowed("rain splashes")),
            layout: vec![super::draw_layout(), self.layout.clone()],
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
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                ..default()
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                cull_mode: None,
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: false,
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

/// The near column list on the GPU, made once at `MAX_COLUMNS` and written
/// only when the list's version moves.
#[derive(Resource)]
struct SplashColumnsGpu {
    buffer: Buffer,
    version: u64,
    count: u32,
}

/// A view's splash draw: its pipeline, and this frame's bind group and lane
/// count, `None` when nothing is drawn.
#[derive(Component)]
struct SplashDraw {
    pipeline: CachedRenderPipelineId,
    bind: Option<(BindGroup, u32)>,
}

fn layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "rain splashes",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX,
            (
                uniform_buffer::<SplashUniform>(false),
                storage_buffer_read_only_sized(
                    false,
                    NonZeroU64::new(size_of::<GpuSplashColumn>() as u64),
                ),
            ),
        ),
    )
}

fn init_pipeline(mut commands: Commands, assets: Res<AssetServer>, device: Res<RenderDevice>) {
    commands.insert_resource(SplashPipeline {
        shader: assets.load("shaders/rain_splash.wgsl"),
        layout: layout(),
    });
    commands.insert_resource(SplashColumnsGpu {
        buffer: device.create_buffer(&BufferDescriptor {
            label: Some("rain splash columns"),
            size: (MAX_COLUMNS * size_of::<GpuSplashColumn>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }),
        version: 0,
        count: 0,
    });
}

fn upload_columns(
    splashes: Option<Res<RainSplashes>>,
    queue: Res<RenderQueue>,
    mut gpu: ResMut<SplashColumnsGpu>,
) {
    let Some(splashes) = splashes else {
        return;
    };
    if splashes.version == gpu.version {
        return;
    }
    let columns = &splashes.columns[..splashes.columns.len().min(MAX_COLUMNS)];
    if !columns.is_empty() {
        queue.write_buffer(&gpu.buffer, 0, bytemuck::cast_slice(columns));
    }
    gpu.count = columns.len() as u32;
    gpu.version = splashes.version;
}

/// The pipeline for every view the planet draws in, whether or not it is
/// raining: queued from the first frame, so it is made behind the loading
/// screen (no `LATE_PIPELINE` the first time it rains).
fn queue_splashes(
    mut commands: Commands,
    pipeline: Res<SplashPipeline>,
    mut specialized: ResMut<SpecializedRenderPipelines<SplashPipeline>>,
    cache: Res<PipelineCache>,
    views: Query<(Entity, &ExtractedView, &Msaa)>,
    surfaces: Query<(), With<PlanetSurface>>,
) {
    if surfaces.is_empty() {
        for (entity, _, _) in &views {
            commands.entity(entity).remove::<SplashDraw>();
        }
        return;
    }
    for (entity, view, msaa) in &views {
        let id = specialized.specialize(&cache, &pipeline, (msaa.samples(), view.hdr));
        commands.entity(entity).insert(SplashDraw {
            pipeline: id,
            bind: None,
        });
    }
}

fn prepare_splashes(
    device: Res<RenderDevice>,
    cache: Res<PipelineCache>,
    pipeline: Res<SplashPipeline>,
    gpu: Res<SplashColumnsGpu>,
    splashes: Option<Res<RainSplashes>>,
    settings: Option<Res<WeatherSettings>>,
    mut views: Query<(&ExtractedView, &mut SplashDraw)>,
) {
    let (Some(splashes), Some(s)) = (splashes, settings) else {
        return;
    };
    let lanes_per_column = s.splash_slots * s.splash_droplets;
    if !splashes.on || gpu.count == 0 || lanes_per_column == 0 {
        return;
    }
    for (view, mut draw) in &mut views {
        let size = view.viewport.zw().as_vec2();
        let focal = view.clip_from_view.y_axis.y * size.y * 0.5;
        let uniform = SplashUniform {
            shape: UVec4::new(s.splash_slots, s.splash_droplets, gpu.count, 0),
            timing: Vec4::new(
                s.splash_period_s,
                s.rain_detail_reach_m,
                s.rain_detail_fade_m,
                splashes.rain,
            ),
            size_m: Vec4::from_array(s.splash_size_m),
            up_mps: Vec4::from_array(s.splash_up_mps),
            out_mps: Vec4::from_array(s.splash_out_mps),
            keep: Vec3::from_array(s.splash_keep).extend(s.splash_alpha),
            view: Vec4::new(size.y, size.x, focal, 0.0),
            eye: splashes.eye.extend(1.0),
        };
        let mut bytes = encase::UniformBuffer::new(Vec::<u8>::new());
        bytes.write(&uniform).expect("the splash uniform encodes");
        let buffer = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("rain splash view"),
            contents: bytes.as_ref(),
            usage: BufferUsages::UNIFORM,
        });
        let group = device.create_bind_group(
            Some("rain splashes"),
            &cache.get_bind_group_layout(&pipeline.layout),
            &BindGroupEntries::sequential((
                buffer.as_entire_binding(),
                gpu.buffer.as_entire_binding(),
            )),
        );
        draw.bind = Some((group, gpu.count * lanes_per_column));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct RainSplashLabel;

#[derive(Default)]
struct RainSplashNode;

impl ViewNode for RainSplashNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static PlanetViewGpu,
        &'static SplashDraw,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext<'w>,
        (target, depth, view, draw): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let Some((group, lanes)) = draw.bind.as_ref() else {
            return Ok(());
        };
        let Some(pipeline) = world
            .resource::<PipelineCache>()
            .get_render_pipeline(draw.pipeline)
        else {
            return Ok(());
        };
        let diagnostics = ctx.diagnostic_recorder();
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("rain splashes"),
            color_attachments: &[Some(target.get_color_attachment())],
            depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let span = diagnostics.pass_span(&mut pass, "rain_splash");
        pass.set_render_pipeline(pipeline);
        pass.set_bind_group(0, &view.draw_bind_group, &[]);
        pass.set_bind_group(1, group, &[]);
        pass.draw(0..4, 0..*lanes);
        span.end(&mut pass);
        Ok(())
    }
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<RainSplashes>()
        .add_plugins(ExtractResourcePlugin::<RainSplashes>::default())
        .add_systems(
            Update,
            update_splashes
                .after(super::lod::refresh_lod)
                .after(crate::weather::WeatherSystems),
        );
}

/// The render world's half, after the terrain's own node exists.
pub(super) fn build_render(render_app: &mut SubApp) {
    render_app
        .init_resource::<SpecializedRenderPipelines<SplashPipeline>>()
        .add_systems(RenderStartup, init_pipeline)
        .add_systems(
            Render,
            (
                upload_columns.in_set(RenderSystems::PrepareResources),
                queue_splashes.in_set(RenderSystems::QueueMeshes),
                prepare_splashes.in_set(RenderSystems::PrepareBindGroups),
            ),
        )
        .add_render_graph_node::<ViewNodeRunner<RainSplashNode>>(Core3d, RainSplashLabel)
        .add_render_graph_edges(
            Core3d,
            (
                super::reflect::PlanetTerrainLabel,
                RainSplashLabel,
                Node3d::MainTransparentPass,
            ),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The items `rain_splash.wgsl` copies from `planet_surface.wgsl` are
    /// that file's own, word for word: the same params, the same puddle
    /// mask and hashes, the same sky and sun, the same lamp share.
    #[test]
    fn the_splash_shader_carries_the_terrains_own_terms() {
        let surface = include_str!("../../../assets/shaders/planet_surface.wgsl");
        let splash = include_str!("../../../assets/shaders/rain_splash.wgsl");
        for head in [
            "struct Params {",
            "fn normalized(",
            "fn rr_hash12(",
            "fn rr_hash22(",
            "fn puddle_noise(",
            "fn ground_sky(",
            "fn airmass(",
            "fn sun_air(",
            "fn sh_irradiance(",
            "fn lamp_share(",
            "fn tileset_slot(",
        ] {
            let item = |source: &str| {
                let at = source
                    .find(head)
                    .unwrap_or_else(|| panic!("{head} is missing"));
                let mut depth = 0;
                for (i, c) in source[at..].char_indices() {
                    match c {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                return source[at..at + i + 1].to_owned();
                            }
                        }
                        _ => {}
                    }
                }
                panic!("{head} never closes");
            };
            assert_eq!(
                item(splash),
                item(surface),
                "{head} drifted from the terrain's"
            );
        }
        for constant in [
            "const PUDDLE_MASK_MEAN: f32 = 0.5003;",
            "const PUDDLE_MASK_SPREAD: f32 = 0.1587;",
            "const SKY_TAU = vec3<f32>(0.040, 0.100, 0.240);",
            "const SKY_TAU_M = 0.07;",
            "const SUN_NOW = vec3<f32>(1.12, 1.03, 0.87);",
            "const SUN_NORM = vec3<f32>(0.880787, 0.821865, 0.699257);",
            "const TORCH_TINT: vec3<f32> = vec3<f32>(1.00, 0.70, 0.30);",
            "const TORCH_GAIN: f32 = 1.25;",
            "const TILLED_CODE = 14u;",
            "const FORGE_CODE = 15u;",
            "const DIRT_CODE = 10u;",
            "const FIELDS_BIOME = 2u;",
            "const MOUNTAINS_BIOME = 6u;",
        ] {
            assert!(surface.contains(constant), "the terrain lost {constant}");
            assert!(splash.contains(constant), "the splashes lost {constant}");
        }
    }

    /// The uniform and the column record are laid out as the shader declares
    /// them, and the shader parses and validates.
    #[test]
    fn the_splash_shader_reads_what_rust_writes() {
        let splash = include_str!("../../../assets/shaders/rain_splash.wgsl");
        let module = naga::front::wgsl::parse_str(splash).unwrap_or_else(|e| {
            panic!("rain_splash.wgsl must parse:\n{}", e.emit_to_string(splash))
        });
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("rain_splash.wgsl must validate");
        assert_eq!(
            crate::shader_tests::wgsl_struct_size("rain_splash.wgsl", splash, "Splash"),
            SplashUniform::min_size().get() as u32
        );
        assert_eq!(
            crate::shader_tests::wgsl_struct_size("rain_splash.wgsl", splash, "SplashColumn"),
            size_of::<GpuSplashColumn>() as u32
        );
        let names: Vec<_> = module
            .entry_points
            .iter()
            .map(|e| e.name.as_str())
            .collect();
        assert!(names.contains(&"vertex") && names.contains(&"fragment"));
    }

    /// Grass and leaves spray; every other top is bare ground, where the
    /// puddle mask decides a crown. The terrain's own `grassy` and `floor_like`.
    #[test]
    fn grass_sprays_and_bare_ground_splashes() {
        for code in [2, 3, 7, 9] {
            assert_eq!(splash_class(code | 4 << 8), 0, "code {code}");
        }
        for code in [0, 1, 4, 5, 6, 8, 10, 14, 15] {
            assert_eq!(splash_class(code), 1, "code {code}");
        }
        let surface = include_str!("../../../assets/shaders/planet_surface.wgsl");
        assert!(surface.contains(
            "fn grassy(material: u32) -> bool { return material==2u || material==3u || material==7u; }"
        ));
        assert!(
            surface.contains(
                "let floor_like = !grassy(code) && code != 9u && code != GRASS_SIDE_CODE;"
            )
        );
    }

    /// A slot's key fits ten bits, so `key * 64 + slot` stays exact in an f32,
    /// and different cells mostly get different keys.
    #[test]
    fn slot_keys_are_small_and_spread() {
        let keys: std::collections::HashSet<u32> = (0..2000u32).map(slot_key).collect();
        assert!(keys.iter().all(|k| *k < 1024));
        assert!(keys.len() > 700, "only {} distinct keys", keys.len());
    }

    /// The near list on a real fine set: the columns round the eye within
    /// the reach, each top on its column's axis at its surface, none twice,
    /// none past the reach, and a shelter takes columns out.
    #[test]
    fn the_near_list_holds_the_columns_within_reach() {
        let settings = crate::config::ColumnSettings::default();
        let spawn = crate::flight_view::FlightViewConfig::default().spawn_direction;
        let mut contact = PlanetContact::test_planet(5);
        let anchor = contact.find_land_near(spawn);
        let set = Arc::new(super::super::lod::generate_fine(
            anchor,
            &settings,
            &pbd_core::edits::Edits::new(),
        ));
        contact.set_fine(&set);
        let start = contact
            .finest_cell(anchor)
            .expect("the anchor is on the fine set");
        let eye = anchor * (PLANET_RADIUS + super::super::surface_height(anchor) + 1.7);
        let list = near_columns(&set, start, eye, 22.0, &|_| false);
        assert!(
            (120..=MAX_COLUMNS).contains(&list.len()),
            "{} columns within 22 m",
            list.len()
        );
        let mut tops = std::collections::HashSet::new();
        for column in &list {
            let top = Vec3::from_slice(&column.top[..3]);
            assert!(tops.insert(column.top.map(f32::to_bits)), "a column twice");
            let along = top.normalize().angle_between(anchor) * PLANET_RADIUS;
            assert!(along <= 22.01, "a column {along} m out");
            assert!(
                (0.8..1.6).contains(&column.top[3]),
                "inner radius {}",
                column.top[3]
            );
            assert!(column.info[2] < 1024);
            assert_eq!(column.info[1], splash_class(column.info[0]));
        }
        let none = near_columns(&set, start, eye, 22.0, &|_| true);
        assert!(none.is_empty(), "a roof over every column leaves none");
    }
}
