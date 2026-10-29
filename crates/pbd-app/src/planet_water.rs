//! The water cap pass and the composite that lets the sea be seen from inside.
//!
//! One render-graph node after Bevy's main pass owns three sub-passes in
//! Tenebris's order: compose (underwater fog, distortion and depth blur into
//! the post-process destination, a copy where the pixel is dry), the water cap
//! drawn over it reading the same scene, and a lens pass (rain droplets and
//! emerge drips) on a second swap so the drops refract the real sea. The
//! camera's side of the surface is decided here on the CPU as a tri-state.
//! Design: `openspec/changes/water-composite/design.md`.

use super::{GpuCell, PlanetClock, PlanetGpu, PlanetRenderFrame, PlanetViewGpu, terrain};
use crate::config::{CloudRenderer, RainRenderer, WaterSettings, WeatherSettings};
use crate::weather::Weather;
use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        Render, RenderStartup, RenderSystems,
        diagnostic::RecordDiagnostics,
        extract_resource::ExtractResource,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{
                sampler, storage_buffer_read_only_sized, texture_2d, texture_2d_array, texture_3d,
                texture_depth_2d, texture_depth_2d_multisampled, uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        view::{ExtractedView, Msaa, ViewDepthTexture, ViewTarget},
    },
    shader::ShaderDefVal,
};
use std::{borrow::Cow, num::NonZeroU64};

/// The terrain shader's haze, fed to the sheet so both fog out together. The
/// terrain still carries these as literals in `planet_surface.wgsl`; lifting
/// them into one uniform is task 2 of `preview-scale-and-shader-parity`.
const FOG_NIGHT_SKY: Vec3 = Vec3::new(0.10, 0.20, 0.29);
const FOG_DAY_SKY: Vec3 = Vec3::new(0.32, 0.49, 0.57);
const FOG_DENSITY_PER_M: f32 = 0.00036;
const FOG_HEIGHT_M: f32 = 1050.0;
const FOG_MIX: f32 = 0.55;
const TERMINATOR: (f32, f32) = (-0.13, 0.20);
/// The sheet's own depth buffer, single-sample like the post-process targets.
const WATER_DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

/// Matches `WaterView` in `water.wgsl` field for field; a test pins the size.
#[derive(Clone, ShaderType)]
pub(super) struct WaterView {
    clip_from_local: Mat4,
    local_from_clip: Mat4,
    /// Last frame's `clip_from_local`: where the clouds pass finds its history
    /// (`calm-clouds`).
    prev_clip_from_local: Mat4,
    camera_time: Vec4,
    planet_center: Vec4,
    sun: Vec4,
    waves: Vec4,
    ripple: Vec4,
    refraction: Vec4,
    absorption: Vec4,
    deep_color: Vec4,
    horizon_color: Vec4,
    zenith_color: Vec4,
    /// The sky the sheet mirrors at night; the day gradient ramps to it.
    night_sky: Vec4,
    foam_color: Vec4,
    foam_crest: Vec4,
    foam_slope: Vec4,
    sun_tint: Vec4,
    fog_night: Vec4,
    fog_day: Vec4,
    limits: Vec4,
    fx: Vec4,
    lens: Vec4,
    screen: Vec4,
    lod: Vec4,
    bands: Vec4,
    /// The bands' cross-fade rings' inner edges, cosines, round `lod`, which
    /// is the terrain's fade centre (`distance-lod-fade`).
    bands_in: Vec4,
    /// The rain on the LENS; then the frame's seconds and the eye's airspeed,
    /// m/s, which the lens mist runs on (`lens-weather`); w spare. Its own
    /// lane rather than `fx.z`, which is the rain on the SEA: a camera in a
    /// cave mouth is dry while the sea it looks out at is still being rained
    /// on.
    rain: Vec4,
    /// The cloud layer (`sky::CloudNow`): the clouds pass marches it over
    /// the whole scene, and the rain is lit by its lightning.
    cloud_clouds: Vec4,
    cloud_slab: Vec4,
    cloud_storm: Vec4,
    cloud_flash: Vec4,
    cloud_light: Vec4,
    cloud_shape: Vec4,
    cloud_cells: Vec4,
    /// The precipitation map's frame (`weather::RainMap`) and the volume's
    /// look; see `rain` in `water.wgsl` for what each lane is.
    rain_map: Vec4,
    rain_u: Vec4,
    rain_v: Vec4,
    rain_look: Vec4,
    rain_tint: Vec4,
    snow_tint: Vec4,
    /// The overlay (`overlay::overlay_lanes`); zero when none is showing.
    overlay: Vec4,
    overlay_flow: Vec4,
    /// The clouds' accumulation: x the share of the new frame in the blend,
    /// y one when the previous history can be used, z the frame number the
    /// jitter hashes, w spare.
    cloud_history: Vec4,
    /// xyz the previous frame's eye, body-local: where the previous frame's
    /// cloud distances were measured from (`cloud-ghosting`); w spare.
    cloud_prev_eye: Vec4,
    /// The sea's table (`pbd::sea` in `sea.wgsl`), the one the hulls float on.
    sea: SeaView,
    /// x how far the sheet sits below sea level, m; yzw spare.
    sea_frame: Vec4,
    /// The lens mist (`lens-weather`): seconds to mist over and to clear
    /// standing still, how much faster per m/s of airspeed, its strength.
    mist: Vec4,
    /// Its beads per screen height, its blur in screen heights, how far it
    /// lifts the image toward its own brightness; w the drops' scale
    /// (`rain_lens_scale`).
    mist_look: Vec4,
    /// The puffs' look (`cloud-puffs`): the sun's wrap, a white and a storm
    /// column's base shade, the silver lining.
    cloud_puff: Vec4,
    /// Their edge erosion, the fade in front of the ground (m), the fade-in
    /// from the eye (m); w one when the puffs draw the clouds.
    cloud_puff_fade: Vec4,
    /// The rain shells (`rain-shells`): xyz the eye's velocity, body-local
    /// m/s; w one when the shells draw the rain instead of the volume.
    rain_shell: Vec4,
    /// How far the rain and the snow have fallen past the eye, m, wrapped at
    /// `RAIN_SHELL_WRAP_M`; the fade in front of the ground, m; w spare.
    rain_shell_fall: Vec4,
    /// `lighting-model`, the rain fog lit: rgb the sky's mean radiance where
    /// the eye is (L2, by the cover), w one while the new model draws; rgb
    /// the sun after the air and the cloud, w the forward scatter's g; and
    /// the lights nearest the eye (planet frame, w the level 0..1).
    rain_sky: Vec4,
    rain_sun: Vec4,
    rain_lamps: [Vec4; super::NEAR_LAMPS],
}

/// Where the shells' fall wraps, m: a multiple of every shell's streak cycle
/// (`RAIN_SHELL_CYCLE_M` in `water.wgsl`, powers of two up to 512), so the
/// pattern never jumps when it wraps.
const RAIN_SHELL_WRAP_M: f32 = 8192.0;
/// How fast the shells' sense of the eye's velocity follows it, per second:
/// a frame's step is noisy, and the lean must not shiver.
const RAIN_SHELL_VELOCITY_RATE: f32 = 8.0;

/// `SeaView` in `sea.wgsl`: `pbd_core::sea::SeaGpu` laid out for the GPU.
#[derive(Clone, ShaderType)]
pub(crate) struct SeaView {
    directions: [Vec4; pbd_core::sea::DIRECTIONS],
    bands: [Vec4; pbd_core::sea::BANDS + 1],
    phase: [Vec4; pbd_core::sea::COMPONENTS / 4],
    heading: Vec4,
    limits: Vec4,
}

impl From<&pbd_core::sea::SeaGpu> for SeaView {
    fn from(sea: &pbd_core::sea::SeaGpu) -> Self {
        Self {
            directions: sea.directions,
            bands: sea.bands,
            phase: sea.phase,
            heading: sea.heading,
            limits: sea.limits,
        }
    }
}

/// The format of the clouds' history: the pass's own result, light
/// premultiplied by coverage and the coverage, before it is composited.
const CLOUD_HISTORY_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

/// The format of the march's distances, metres along the ray: r where its
/// cloud is, g where the march stopped (the scene or the sea). What the
/// composite's depth-aware upsample compares with each pixel's own.
const CLOUD_DEPTH_FORMAT: TextureFormat = TextureFormat::Rg16Float;

/// The lens mist (`lens-weather`): one texel, full float, because it creeps
/// by a few thousandths a frame and a half float would stall short of one.
const MIST_FORMAT: TextureFormat = TextureFormat::R32Float;

/// The share of each new frame in the clouds' history: about sixteen frames of
/// samples in every pixel (`calm-clouds` design).
const CLOUD_BLEND: f32 = 0.06;

/// A camera that moves further than this between frames has jumped (a
/// teleport, a load) and the history is dropped, metres.
const CLOUD_HISTORY_JUMP_M: f32 = 50.0;

/// The largest precipitation map the rain buffer holds, cells on a side; the
/// config validates `rain_map_size` against it.
const RAIN_MAP_MAX: u64 = 128;

/// What the column tier says is at the camera's own cell: nothing, because
/// there is no column there and the height field is the whole truth; air, so
/// the camera is dry whatever its radius; or water, whose surface stands at
/// this body radius.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum EyeWater {
    #[default]
    Unknown,
    Air,
    Water {
        surface_radius: f32,
    },
}

/// What the tier says is at the player's eye, published by the main world for
/// the render world to shade with. The terrain is CPU-authoritative and this
/// is one reader of it, extracted like every other derived fact.
#[derive(Resource, Clone, Copy, Default, ExtractResource)]
pub struct EyeWaterState(pub EyeWater);

/// Which side of the surface the camera is on, from its body-local position:
/// 0 dry, 0.5 straddling the surface band, 1 fully under. The band is wide
/// enough to hold the swell so the wave function is never evaluated a second
/// time on the CPU.
///
/// `eye` is the column's answer where there is a column. Without it the only
/// question a height field can answer is "is the camera under sea level over
/// a cell whose ground is", which drowns a camera standing in a dry cave
/// carved below sea level.
pub fn submersion(camera_body: Vec3, sea_radius: f32, wave: f32, band: f32, eye: EyeWater) -> f32 {
    if camera_body.length_squared() < 1e-6 {
        return 0.0;
    }
    let surface = match eye {
        EyeWater::Air => return 0.0,
        EyeWater::Water { surface_radius } => surface_radius,
        // Off the tier there are no caves, so the ground's own height is the
        // whole of it: over land, dry.
        EyeWater::Unknown => {
            if terrain::surface_height(camera_body) >= 0.0 {
                return 0.0;
            }
            sea_radius
        }
    };
    // The band stands about the surface as it is under the camera now: a
    // trough passing under a camera just over the sea leaves it dry.
    let surface = surface + wave;
    let radius = camera_body.length();
    if radius < surface - band {
        1.0
    } else if radius <= surface + band {
        0.5
    } else {
        0.0
    }
}

/// Publish what the tier says is at the active camera's eye.
pub(super) fn install_eye_water(app: &mut App) {
    app.init_resource::<EyeWaterState>().add_systems(
        PostUpdate,
        publish_eye_water.after(bevy::transform::TransformSystems::Propagate),
    );
}

/// Publish what the tier says is at the active camera's eye.
///
/// In the MAIN world, because that is where the authoritative columns live;
/// the render world reads the extracted answer. The camera rather than the
/// walker, because a flying player has an eye too and the sheet is composited
/// for whatever is looking.
pub fn publish_eye_water(
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Res<PlanetRenderFrame>,
    contact: Option<Res<crate::planet::PlanetContact>>,
    fine: Option<Res<crate::planet::PlanetFine>>,
    mut eye: ResMut<EyeWaterState>,
) {
    let answer = eye_water(&cameras, &frame, contact.as_deref(), fine.as_deref());
    if eye.0 != answer {
        eye.0 = answer;
    }
}

fn eye_water(
    cameras: &Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: &PlanetRenderFrame,
    contact: Option<&crate::planet::PlanetContact>,
    fine: Option<&crate::planet::PlanetFine>,
) -> EyeWater {
    let (Some(contact), Some(fine)) = (contact, fine) else {
        return EyeWater::Unknown;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return EyeWater::Unknown;
    };
    let body = (transform.translation().as_dvec3() - frame.center).as_vec3();
    let Some(direction) = body.try_normalize() else {
        return EyeWater::Unknown;
    };
    let Some(column) = contact
        .finest_cell(direction)
        .filter(|_| contact.serves(&fine.set))
        .and_then(|record| fine.set.columns.column(record))
    else {
        return EyeWater::Unknown;
    };
    let altitude = body.length() - terrain::PLANET_RADIUS;
    match column.water_surface(altitude) {
        Some(surface) => EyeWater::Water {
            surface_radius: terrain::PLANET_RADIUS + surface,
        },
        // Rock reads as air: a camera inside a wall sees nothing, and what it
        // must not do is fill with sea.
        None => EyeWater::Air,
    }
}

/// The emerge window: armed when the camera leaves the water and kept topped
/// up while it straddles the surface, so the lens stays wet until it clears.
/// Returns the drip intensity, 1 down to 0 over `dry_seconds`.
pub fn emerge(
    now: f32,
    submersion: f32,
    was_under: bool,
    emerge_until: &mut f32,
    dry_seconds: f32,
) -> f32 {
    let under = submersion > 0.75;
    let straddling = submersion > 0.25 && !under;
    if (was_under && !under) || straddling {
        *emerge_until = now + dry_seconds;
    }
    if under {
        0.0
    } else {
        ((*emerge_until - now) / dry_seconds.max(1e-6)).clamp(0.0, 1.0)
    }
}

#[derive(Resource)]
struct WaterPipelines {
    shader: Handle<Shader>,
    fullscreen: FullscreenShader,
    data_layout: BindGroupLayoutDescriptor,
    scene_layout: BindGroupLayoutDescriptor,
    scene_layout_multisampled: BindGroupLayoutDescriptor,
    /// The march's fourth group: the previous history and a sampler, the
    /// baked cellular noise and its repeating sampler, and the previous
    /// frame's cloud distances.
    history_layout: BindGroupLayoutDescriptor,
    /// The composite's fourth group: this frame's march, its sampler and its
    /// cloud distances.
    composite_layout: BindGroupLayoutDescriptor,
    /// The resolve's fourth group (`cloud-history-clip`): the previous
    /// history, its sampler and its distances, and this frame's march and
    /// distances.
    resolve_layout: BindGroupLayoutDescriptor,
    /// The lens mist probe's fourth group: the cellular noise and its
    /// sampler, and the previous mist (`lens-weather`).
    mist_layout: BindGroupLayoutDescriptor,
    /// The lens's fourth group: the mist the probe just wrote.
    lens_layout: BindGroupLayoutDescriptor,
    /// The puffs' fourth group (`cloud-puffs`): this frame's instances.
    puff_layout: BindGroupLayoutDescriptor,
    /// The rain's fourth group (`lighting-model`, the rain fog lit): the sun's
    /// cascades, their comparison sampler and where each stands.
    rain_light_layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    cells: TextureView,
    cells_sampler: Sampler,
}

fn history_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "clouds history and cells",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                texture_3d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                // The previous frame's cloud distances (`cloud-ghosting`).
                texture_2d(TextureSampleType::Float { filterable: false }),
            ),
        ),
    )
}

fn resolve_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "clouds resolve: history, march, distances",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (0, texture_2d(TextureSampleType::Float { filterable: true })),
                (1, sampler(SamplerBindingType::Filtering)),
                (
                    4,
                    texture_2d(TextureSampleType::Float { filterable: false }),
                ),
                (
                    5,
                    texture_2d(TextureSampleType::Float { filterable: false }),
                ),
                (
                    6,
                    texture_2d(TextureSampleType::Float { filterable: false }),
                ),
            ),
        ),
    )
}

fn composite_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "clouds march and its distances",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (0, texture_2d(TextureSampleType::Float { filterable: true })),
                (1, sampler(SamplerBindingType::Filtering)),
                (
                    4,
                    texture_2d(TextureSampleType::Float { filterable: false }),
                ),
                // The near field's fog, sampled bilinearly (`cloud-entry`).
                (7, texture_2d(TextureSampleType::Float { filterable: true })),
            ),
        ),
    )
}

fn puff_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "cloud puffs",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::VERTEX_FRAGMENT,
            ((
                10,
                storage_buffer_read_only_sized(
                    false,
                    NonZeroU64::new(size_of::<super::cloud_puffs::PuffInstance>() as u64),
                ),
            ),),
        ),
    )
}

fn rain_light_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "rain: the sun's cascades",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (11, texture_2d_array(TextureSampleType::Depth)),
                (12, sampler(SamplerBindingType::Comparison)),
                (
                    13,
                    storage_buffer_read_only_sized(
                        false,
                        NonZeroU64::new(size_of::<super::shadow::GpuCascades>() as u64),
                    ),
                ),
            ),
        ),
    )
}

fn mist_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "lens mist probe: cells, previous mist",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (2, texture_3d(TextureSampleType::Float { filterable: true })),
                (3, sampler(SamplerBindingType::Filtering)),
                (
                    8,
                    texture_2d(TextureSampleType::Float { filterable: false }),
                ),
            ),
        ),
    )
}

fn lens_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "lens: the mist",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            ((
                9,
                texture_2d(TextureSampleType::Float { filterable: false }),
            ),),
        ),
    )
}

/// The clouds' cellular noise on the GPU (`planet_cloud_noise.rs`), baked and
/// uploaded once.
fn cloud_cells(device: &RenderDevice, queue: &RenderQueue) -> TextureView {
    let started = std::time::Instant::now();
    let size = super::cloud_noise::SIZE;
    let bytes = super::cloud_noise::bake(size, super::cloud_noise::PERIOD);
    let extent = Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: size,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("clouds cellular noise"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: TextureFormat::R8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &bytes,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size),
            rows_per_image: Some(size),
        },
        extent,
    );
    info!(
        "clouds cellular noise: {size}^3 texels over {} cells, baked in {:.1} ms",
        super::cloud_noise::PERIOD,
        started.elapsed().as_secs_f64() * 1e3
    );
    texture.create_view(&TextureViewDescriptor::default())
}

fn data_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "water view, cells, water ids, flow, precipitation map",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                uniform_buffer::<WaterView>(false),
                storage_buffer_read_only_sized(false, NonZeroU64::new(size_of::<GpuCell>() as u64)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(8)),
                storage_buffer_read_only_sized(false, NonZeroU64::new(4)),
            ),
        ),
    )
}

fn scene_layout(multisampled: bool) -> BindGroupLayoutDescriptor {
    let depth = if multisampled {
        texture_depth_2d_multisampled()
    } else {
        texture_depth_2d()
    };
    BindGroupLayoutDescriptor::new(
        "water scene colour and depth",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                depth,
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    )
}

fn initialize_pipelines(
    mut commands: Commands,
    assets: Res<AssetServer>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    fullscreen: Res<FullscreenShader>,
) {
    commands.insert_resource(WaterPipelines {
        cells: cloud_cells(&device, &queue),
        cells_sampler: device.create_sampler(&SamplerDescriptor {
            label: Some("clouds cellular noise sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            ..default()
        }),
        shader: assets.load("shaders/water.wgsl"),
        fullscreen: fullscreen.clone(),
        data_layout: data_layout(),
        scene_layout: scene_layout(false),
        scene_layout_multisampled: scene_layout(true),
        history_layout: history_layout(),
        rain_light_layout: rain_light_layout(),
        composite_layout: composite_layout(),
        resolve_layout: resolve_layout(),
        mist_layout: mist_layout(),
        lens_layout: lens_layout(),
        puff_layout: puff_layout(),
        sampler: device.create_sampler(&SamplerDescriptor {
            label: Some("water scene sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..default()
        }),
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Pass {
    Cap,
    Compose,
    CloudMarch,
    CloudResolve,
    CloudPuffs,
    Clouds,
    Rain,
    Overlay,
    Lens,
    LensMist,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct WaterKey {
    pass: Pass,
    format: TextureFormat,
    multisampled: bool,
}

impl SpecializedRenderPipeline for WaterPipelines {
    type Key = WaterKey;
    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut defs = Vec::new();
        if key.multisampled {
            defs.push(ShaderDefVal::from("MULTISAMPLED"));
        }
        let scene = if key.multisampled {
            self.scene_layout_multisampled.clone()
        } else {
            self.scene_layout.clone()
        };
        let (label, vertex, entry) = match key.pass {
            Pass::Cap => (
                "Water cap over the composed scene",
                VertexState {
                    shader: self.shader.clone(),
                    shader_defs: defs.clone(),
                    entry_point: Some(Cow::Borrowed("vertex")),
                    ..default()
                },
                "fragment",
            ),
            Pass::Compose => (
                "Water compose: underwater fog, distortion, blur",
                self.fullscreen.to_vertex_state(),
                "compose",
            ),
            Pass::CloudResolve => (
                "Cloud history resolve: reprojected, tested, clipped, blended",
                self.fullscreen.to_vertex_state(),
                "cloud_resolve",
            ),
            Pass::CloudMarch => (
                "Cloud march at the clouds' resolution, into the history",
                self.fullscreen.to_vertex_state(),
                "cloud_march_pass",
            ),
            Pass::CloudPuffs => (
                "Cloud puffs into the history, far to near",
                VertexState {
                    shader: self.shader.clone(),
                    shader_defs: defs.clone(),
                    entry_point: Some(Cow::Borrowed("cloud_puff_vertex")),
                    ..default()
                },
                "cloud_puff_fragment",
            ),
            Pass::Clouds => (
                "Clouds over the whole scene, against its depth",
                self.fullscreen.to_vertex_state(),
                "clouds",
            ),
            Pass::Rain => (
                "Rain volume over the composed scene",
                self.fullscreen.to_vertex_state(),
                "rain",
            ),
            Pass::Overlay => (
                "Weather overlay over the planet",
                self.fullscreen.to_vertex_state(),
                "overlay",
            ),
            Pass::Lens => (
                "Water lens: droplets, emerge drips and mist",
                self.fullscreen.to_vertex_state(),
                "lens",
            ),
            Pass::LensMist => (
                "Lens mist probe: the cloud at the eye, one pixel",
                self.fullscreen.to_vertex_state(),
                "lens_mist_pass",
            ),
        };
        // The sheet self-sorts against its own single-sample depth buffer, so
        // a near crest occludes a far trough whatever order the cells come in
        // (Tenebris: "depth write + LESS_EQUAL to self-sort"). Compose shares
        // the pass and so declares the same attachment, without touching it;
        // the scene's own occlusion is the shader's discard against the
        // sampled main-pass depth, which may be multisampled.
        let depth_stencil = match key.pass {
            Pass::Lens
            | Pass::LensMist
            | Pass::Rain
            | Pass::CloudMarch
            | Pass::CloudResolve
            | Pass::CloudPuffs
            | Pass::Clouds
            | Pass::Overlay => None,
            pass => Some(DepthStencilState {
                format: WATER_DEPTH_FORMAT,
                depth_write_enabled: pass == Pass::Cap,
                depth_compare: if pass == Pass::Cap {
                    CompareFunction::GreaterEqual
                } else {
                    CompareFunction::Always
                },
                stencil: default(),
                bias: default(),
            }),
        };
        RenderPipelineDescriptor {
            label: Some(Cow::Borrowed(label)),
            layout: if matches!(
                key.pass,
                Pass::Clouds | Pass::CloudMarch | Pass::CloudResolve | Pass::CloudPuffs
            ) {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                    match key.pass {
                        Pass::CloudMarch => self.history_layout.clone(),
                        Pass::CloudResolve => self.resolve_layout.clone(),
                        Pass::CloudPuffs => self.puff_layout.clone(),
                        _ => self.composite_layout.clone(),
                    },
                ]
            } else if key.pass == Pass::Rain {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                    self.rain_light_layout.clone(),
                ]
            } else if matches!(key.pass, Pass::Lens | Pass::LensMist) {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                    if key.pass == Pass::Lens {
                        self.lens_layout.clone()
                    } else {
                        self.mist_layout.clone()
                    },
                ]
            } else {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                ]
            },
            vertex,
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: defs,
                entry_point: Some(Cow::Borrowed(entry)),
                // The march writes this frame's clouds, their distances and
                // the near field's fog; the resolve writes the history and its
                // distances (`cloud-history-clip`, `cloud-entry`).
                targets: if key.pass == Pass::CloudResolve {
                    [CLOUD_HISTORY_FORMAT, CLOUD_DEPTH_FORMAT]
                        .map(|format| {
                            Some(ColorTargetState {
                                format,
                                blend: None,
                                write_mask: ColorWrites::ALL,
                            })
                        })
                        .to_vec()
                } else if key.pass == Pass::CloudMarch {
                    [
                        CLOUD_HISTORY_FORMAT,
                        CLOUD_DEPTH_FORMAT,
                        CLOUD_HISTORY_FORMAT,
                    ]
                    .map(|format| {
                        Some(ColorTargetState {
                            format,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })
                    })
                    .to_vec()
                } else if key.pass == Pass::CloudPuffs {
                    // Premultiplied light "over" what is behind; the nearest
                    // puff's distance and the scene's stop under it.
                    let min = BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Min,
                    };
                    vec![
                        Some(ColorTargetState {
                            format: CLOUD_HISTORY_FORMAT,
                            blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: ColorWrites::ALL,
                        }),
                        Some(ColorTargetState {
                            format: CLOUD_DEPTH_FORMAT,
                            blend: Some(BlendState {
                                color: min,
                                alpha: min,
                            }),
                            write_mask: ColorWrites::ALL,
                        }),
                    ]
                } else if key.pass == Pass::LensMist {
                    vec![Some(ColorTargetState {
                        format: MIST_FORMAT,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })]
                } else {
                    vec![Some(ColorTargetState {
                        format: key.format,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })]
                },
            }),
            primitive: PrimitiveState {
                // Back faces are the underwater view; nothing is culled.
                cull_mode: None,
                ..default()
            },
            depth_stencil,
            // The post-process targets are single-sample whatever the main
            // pass's MSAA; only the depth texture read is multisampled.
            multisample: MultisampleState::default(),
            ..default()
        }
    }
}

/// Per-view GPU state and the submersion memory the emerge window needs.
#[derive(Component)]
pub(super) struct WaterViewGpu {
    uniform: UniformBuffer<WaterView>,
    // Held by the bind group; kept so its lifetime is explicit.
    _flow: Buffer,
    data_bind_group: BindGroup,
    cap: CachedRenderPipelineId,
    compose: CachedRenderPipelineId,
    cloud_march: CachedRenderPipelineId,
    cloud_resolve: CachedRenderPipelineId,
    cloud_puffs: CachedRenderPipelineId,
    clouds: CachedRenderPipelineId,
    lens: CachedRenderPipelineId,
    lens_mist: CachedRenderPipelineId,
    rain: CachedRenderPipelineId,
    overlay: CachedRenderPipelineId,
    overlay_needed: bool,
    /// The precipitation map, rewritten each frame from `weather::RainMap`.
    rain_buffer: Buffer,
    rain_needed: bool,
    /// The sheet's private depth, recreated when the view's size changes.
    depth: TextureView,
    depth_size: UVec2,
    multisampled: bool,
    lens_needed: bool,
    was_under: bool,
    emerge_until: f32,
    /// The clouds' history pair, at `cloud_render_scale` of the view. This
    /// frame reads `history[history_read]` and writes the other; the index
    /// flips every frame.
    history: [TextureView; 2],
    history_read: usize,
    history_size: UVec2,
    /// The cloud distances, the history's size, a pair indexed as the history
    /// is: the march writes this frame's and reads the previous frame's, to
    /// test each history texel against what it saw (`cloud-ghosting`).
    cloud_depth: [TextureView; 2],
    /// This frame's march alone, before the resolve folds it into the
    /// history, the history's size (`cloud-history-clip`): its clouds, their
    /// distances, and the near field's fog (`cloud-entry`).
    current: [TextureView; 3],
    /// The previous frame's camera and projection, for reprojecting the
    /// history and for telling a jump from a step.
    prev_clip: Mat4,
    prev_camera: Vec3,
    /// Whether the clouds pass ran on the previous frame, so its history is
    /// current.
    clouds_ran: bool,
    frame: u32,
    /// The lens mist pair (`lens-weather`), one texel each: the probe reads
    /// `mist[mist_read]` and writes the other, which the lens reads.
    mist: [TextureView; 2],
    mist_read: usize,
    /// Until when the probe and the lens run for the mist: while the eye is
    /// in the cloud layer and five clearing times after, so it can clear.
    mist_until: Option<std::time::Instant>,
    mist_active: bool,
    /// When this view was last prepared, for the mist's clock.
    last_frame: Option<std::time::Instant>,
    /// Whether the puffs draw the clouds this frame, and their instances,
    /// sorted far to near (`cloud-puffs`). The buffer grows, never shrinks.
    puffs_on: bool,
    puff_buffer: Buffer,
    puff_count: u32,
    /// Whether the march pass runs: always for the march, and for the puffs
    /// only while the eye is within the near fog's reach of the layer.
    near_fog_needed: bool,
    /// The eye's velocity as the rain shells lean to it, eased, body-local
    /// m/s, and how far the rain (x) and the snow (y) have fallen past the
    /// eye, m (`rain-shells`).
    rain_velocity: Vec3,
    rain_fallen: Vec2,
}

/// One frame of the rain shells' motion (`rain-shells`): the eye's velocity
/// eased toward this frame's `step` over `dt`, and how far the rain and the
/// snow (`fall`, m/s, x and y) have fallen past the eye, which is the speed of
/// the fall against the moving eye, accumulated and wrapped at
/// `RAIN_SHELL_WRAP_M`. Accumulated rather than `clock * speed`, so a change of
/// speed moves the streaks on from where they are instead of jumping them.
fn rain_shell_step(
    velocity: Vec3,
    step: Vec3,
    dt: f32,
    fallen: Vec2,
    down: Vec3,
    fall: Vec2,
) -> (Vec3, Vec2) {
    if dt.is_nan() || dt <= 0.0 || !step.is_finite() || !velocity.is_finite() {
        return (Vec3::ZERO, fallen);
    }
    let ease = 1.0 - (-RAIN_SHELL_VELOCITY_RATE * dt).exp();
    let velocity = velocity + (step / dt - velocity) * ease;
    let speed = |f: f32| (down * f - velocity).length() * dt;
    let fallen = Vec2::new(
        (fallen.x + speed(fall.x)).rem_euclid(RAIN_SHELL_WRAP_M),
        (fallen.y + speed(fall.y)).rem_euclid(RAIN_SHELL_WRAP_M),
    );
    (velocity, fallen)
}

/// What the sky is doing, as the water pass reads it: the weather, its
/// settings, the clouds and the precipitation map. A struct because Bevy
/// caps a system at sixteen parameters and the rain volume took this one to
/// seventeen, which is the missing-struct smell the rules name.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct WaterSky<'w> {
    weather_settings: Res<'w, WeatherSettings>,
    weather: Res<'w, Weather>,
    clouds: Res<'w, crate::sky::CloudNow>,
    rain_map: Option<Res<'w, crate::weather::RainMap>>,
    maps: Res<'w, super::weather_maps::WeatherMapsNow>,
    sea: Res<'w, crate::sea::SeaNow>,
    lighting: Option<Res<'w, crate::config::LightingSettings>>,
    fine: Option<Res<'w, super::lod::PlanetFine>>,
}

/// The sky's mean radiance for a sun at elevation sine `e` under `cover`:
/// the zeroth of L2's nine coefficients, which is what light from all round
/// gives a drop of rain.
fn rain_sky(e: f32, cover: f32) -> Vec3 {
    let sh = super::sky_sh_pair(e);
    sh[0].lerp(sh[9], cover.clamp(0.0, 1.0)).truncate() * 0.282_095
}

#[allow(clippy::too_many_arguments)]
fn prepare_water_views(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    cache: Res<PipelineCache>,
    pipelines: Res<WaterPipelines>,
    mut specialized: ResMut<SpecializedRenderPipelines<WaterPipelines>>,
    planet: Option<Res<PlanetGpu>>,
    settings: Res<WaterSettings>,
    sky: WaterSky,
    clock: Res<PlanetClock>,
    sun: Res<crate::sky::Sun>,
    frame: Res<PlanetRenderFrame>,
    eye: Res<EyeWaterState>,
    mut puffs: Local<(
        Option<super::cloud_puffs::PuffSites>,
        Vec<super::cloud_puffs::PuffInstance>,
    )>,
    mut views: Query<(
        Entity,
        &ExtractedView,
        &Msaa,
        &PlanetViewGpu,
        Option<&mut WaterViewGpu>,
    )>,
) {
    let Some(planet) = planet else {
        return;
    };
    let WaterSky {
        weather_settings,
        weather,
        clouds,
        rain_map,
        maps,
        sea: sea_now,
        lighting,
        fine,
    } = sky;
    let lit_rain = lighting.as_ref().is_some_and(|l| l.new_model_on());
    // `lighting-model`: under the new model the rain's rings on the water are
    // as many as it is raining (the terrain's own rule), not one a cell.
    let rings_follow_rain = lighting.is_some_and(|l| l.new_model_on());
    let (overlay, overlay_flow) =
        crate::overlay::overlay_lanes(maps.overlay_kind, &weather_settings);
    let overlay_needed = maps.overlay_kind.is_some();
    for (entity, view, msaa, planet_view, existing) in &mut views {
        let (camera, clip_from_body) = frame.camera_and_clip(
            &view.world_from_view,
            view.clip_from_view,
            view.clip_from_world,
        );
        let sea_radius = terrain::PLANET_RADIUS - settings.depth_offset_m;
        let band = settings.partial_band_m;
        let state = submersion(camera, sea_radius, sea_now.camera_height, band, eye.0);
        let (mut was_under, mut emerge_until) = existing
            .as_ref()
            .map(|gpu| (gpu.was_under, gpu.emerge_until))
            .unwrap_or((false, f32::NEG_INFINITY));
        let drips = emerge(
            clock.0,
            state,
            was_under,
            &mut emerge_until,
            settings.emerge_dry_s,
        );
        was_under = state > 0.75;
        let aspect = view.viewport.z as f32 / view.viewport.w.max(1) as f32;
        let s = &settings;
        // Rain below the altitude the rain itself stops at: from 400 m up a
        // storm reads through the clouds and the haze, not through drops on
        // the glass or rings on a sea a pixel wide. One gate, in `weather`.
        let altitude = crate::weather::height_above_ground(camera);
        let sea_rain = if crate::weather::rain_drawn_at(altitude, &weather_settings) {
            weather.liquid()
        } else {
            0.0
        };
        // And on the lens only when nothing solid is over the eye.
        let lens_rain = if weather.sheltered { 0.0 } else { sea_rain };
        // The lens mist's clock and the eye's airspeed (`lens-weather`), and
        // whether the eye is in the cloud layer's shell, the only place the
        // density the probe samples can be above zero.
        let now = std::time::Instant::now();
        let (frame_s, airspeed) = existing.as_ref().map_or((0.0, 0.0), |gpu| {
            let dt = gpu
                .last_frame
                .map_or(0.0, |t| now.duration_since(t).as_secs_f32())
                .min(0.5);
            let speed = if dt > 0.0 {
                camera.distance(gpu.prev_camera) / dt
            } else {
                0.0
            };
            (dt, speed)
        });
        let eye_radius = camera.length();
        let mist_on = weather_settings.lens_mist_strength > 0.0
            && !overlay_needed
            && !*NO_CLOUDS
            && !PASSES_OFF[3];
        let in_layer = mist_on
            && eye_radius >= clouds.clouds.x
            && eye_radius <= clouds.clouds.x + clouds.slab.x;
        let hold = std::time::Duration::from_secs_f32(
            (5.0 * weather_settings.lens_mist_clear_s).clamp(1.0, 120.0),
        );
        let was_until = existing.as_ref().and_then(|gpu| gpu.mist_until);
        let mist_until = if in_layer {
            Some(now + hold)
        } else {
            was_until.filter(|until| *until > now && mist_on)
        };
        let mist_active = mist_until.is_some();
        if mist_active != was_until.is_some_and(|until| until > now) {
            info!(
                "LENS_MIST {}",
                if mist_active {
                    "window opens: the eye is in the cloud layer"
                } else {
                    "window closes"
                }
            );
        }
        // The sea's sun specular takes the ground's overcast dim, so a storm
        // does not glitter.
        let sun_dim = 1.0 - weather.cover.clamp(0.0, 1.0) * weather_settings.overcast_sun_dim;
        let v3 = |c: [f32; 3]| Vec3::from_array(c);
        // `weather.ron`'s rain and snow colours are display values, as the
        // shower's vertex colours take them.
        let linear = |c: [f32; 3]| {
            let l = LinearRgba::from(Srgba::rgb(c[0], c[1], c[2]));
            Vec3::new(l.red, l.green, l.blue)
        };
        let puffs_on = weather_settings.cloud_renderer.resolved() == CloudRenderer::Puffs;
        // The eye's velocity, eased, and how far the rain and the snow have
        // fallen past it (`rain-shells`).
        let shells_on = weather_settings.rain_renderer.resolved() == RainRenderer::Shells;
        let (rain_velocity, rain_fallen) =
            existing.as_ref().map_or((Vec3::ZERO, Vec2::ZERO), |gpu| {
                rain_shell_step(
                    gpu.rain_velocity,
                    camera - gpu.prev_camera,
                    frame_s,
                    gpu.rain_fallen,
                    -camera.normalize_or_zero(),
                    Vec2::new(
                        weather_settings.rain_fall_mps,
                        weather_settings.snow_fall_mps,
                    ),
                )
            });
        let params = WaterView {
            clip_from_local: clip_from_body,
            local_from_clip: clip_from_body.as_dmat4().inverse().as_mat4(),
            prev_clip_from_local: existing
                .as_ref()
                .map_or(clip_from_body, |gpu| gpu.prev_clip),
            camera_time: camera.extend(clock.0 * s.time_scale),
            planet_center: Vec3::ZERO.extend(sea_radius),
            sun: sun.direction().extend(s.specular_intensity * sun_dim),
            waves: Vec4::new(0.0, 0.0, 0.0, s.wave_steepness),
            ripple: Vec4::new(
                s.ripple_scale,
                s.ripple_speed,
                s.rain_ripple_scale,
                s.rain_ripple_strength,
            ),
            refraction: Vec4::new(
                s.refract_amount,
                s.refract_max_uv,
                s.slope_max,
                s.max_path_m,
            ),
            absorption: v3(s.absorption_per_m).extend(s.night_floor),
            deep_color: v3(s.deep_color).extend(s.flow_uv_speed_falling),
            horizon_color: v3(s.sky_horizon_color).extend(s.sky_horizon_strength),
            zenith_color: v3(s.sky_zenith_color).extend(0.0),
            night_sky: v3(s.night_sky_color).extend(crate::sky::ATMOSPHERE_RADIUS),
            foam_color: v3(s.foam_color).extend(s.foam_intensity),
            foam_crest: Vec4::new(s.foam_crest_lo, s.foam_crest_hi, s.foam_crest_weight, 0.0),
            foam_slope: Vec4::new(s.foam_slope_lo, s.foam_slope_hi, s.foam_slope_weight, 0.0),
            sun_tint: v3(s.sun_tint).extend(s.specular_power),
            fog_night: FOG_NIGHT_SKY.extend(FOG_DENSITY_PER_M),
            fog_day: FOG_DAY_SKY.extend(FOG_HEIGHT_M),
            limits: Vec4::new(s.fog_max, TERMINATOR.0, TERMINATOR.1, FOG_MIX),
            fx: Vec4::new(s.underwater_distortion, state, sea_rain, drips),
            lens: Vec4::new(
                weather_settings.rain_lens_density,
                weather_settings.rain_lens_refract,
                weather_settings.rain_lens_speed,
                weather_settings.rain_lens_size,
            ),
            screen: Vec4::new(aspect, band, s.wet_blur, s.detail_fade),
            // The partition the uploaded records can serve, read off the same
            // place the surface pass reads it, so a sheet and the terrain
            // under it can never be split on different anchors.
            // The terrain's fade centre and rings, so the sheet is split
            // exactly as the ground under it is (`distance-lod-fade`).
            lod: planet_view.centre.extend(super::lod::BASE_LEVEL as f32),
            bands: planet.lod.bands,
            bands_in: planet.lod.bands_in,
            rain: Vec4::new(
                lens_rain,
                frame_s,
                airspeed,
                if rings_follow_rain { 1.0 } else { 0.0 },
            ),
            cloud_clouds: clouds.clouds,
            cloud_slab: clouds.slab,
            cloud_storm: clouds.storm,
            cloud_flash: clouds.flash,
            cloud_light: clouds.light,
            cloud_shape: clouds.shape,
            cloud_cells: clouds.cells,
            rain_map: rain_map
                .as_deref()
                .map_or(Vec4::ZERO, |m| m.anchor.extend(m.cell_m)),
            rain_u: rain_map
                .as_deref()
                .map_or(Vec4::ZERO, |m| m.u.extend(m.size as f32)),
            rain_v: rain_map.as_deref().map_or(Vec4::ZERO, |m| {
                m.v.extend(weather_settings.rain_volume_density)
            }),
            rain_look: Vec4::new(
                weather_settings.rain_fall_mps,
                weather_settings.rain_volume_stretch,
                crate::weather::rain_light(sun.elevation(camera), weather.cover, &weather_settings),
                weather_settings.rain_volume_range_m,
            ),
            rain_tint: linear(weather_settings.rain_volume_color)
                .extend(weather_settings.snow_fall_mps),
            snow_tint: linear(weather_settings.snow_color).extend(clock.0),
            overlay,
            overlay_flow,
            cloud_history: Vec4::ZERO,
            cloud_prev_eye: existing
                .as_ref()
                .map_or(camera, |gpu| gpu.prev_camera)
                .extend(0.0),
            sea: SeaView::from(&sea_now.gpu),
            sea_frame: Vec4::new(s.depth_offset_m, 0.0, 0.0, 0.0),
            mist: Vec4::new(
                weather_settings.lens_mist_fog_s,
                weather_settings.lens_mist_clear_s,
                weather_settings.lens_mist_per_mps,
                if mist_on {
                    weather_settings.lens_mist_strength
                } else {
                    0.0
                },
            ),
            mist_look: Vec4::new(
                weather_settings.lens_mist_beads,
                weather_settings.lens_mist_blur,
                weather_settings.lens_mist_whiten,
                weather_settings.rain_lens_scale,
            ),
            cloud_puff: Vec4::new(
                weather_settings.cloud_puff_wrap,
                weather_settings.cloud_puff_base_shade,
                weather_settings.cloud_puff_storm_shade,
                weather_settings.cloud_puff_silver,
            ),
            cloud_puff_fade: Vec4::new(
                weather_settings.cloud_puff_erosion,
                weather_settings.cloud_puff_soft_m,
                weather_settings.cloud_puff_near_m,
                if puffs_on { 1.0 } else { 0.0 },
            ),
            rain_shell: rain_velocity.extend(if shells_on { 1.0 } else { 0.0 }),
            rain_shell_fall: Vec4::new(
                rain_fallen.x,
                rain_fallen.y,
                weather_settings.rain_shell_soft_m,
                0.0,
            ),
            rain_sky: rain_sky(sun.elevation(camera), weather.cover).extend(if lit_rain {
                1.0
            } else {
                0.0
            }),
            rain_sun: (Vec3::from_array(pbd_core::sky_light::sun_air(sun.elevation(camera)))
                * (1.0 - 0.85 * weather.cover.clamp(0.0, 1.0)))
            .extend(0.55),
            rain_lamps: fine
                .as_ref()
                .filter(|_| lit_rain)
                .map(|fine| super::near_lamps(&fine.set.columns, camera, 32.0))
                .unwrap_or([Vec4::ZERO; super::NEAR_LAMPS]),
        };
        // The puffs under the shown weather, far to near from this eye
        // (`cloud-puffs`).
        let puffs_drawn = puffs_on && !overlay_needed && !*NO_CLOUDS;
        let (sites, instances) = &mut *puffs;
        instances.clear();
        if puffs_drawn {
            let spacing = weather_settings.cloud_puff_spacing_m;
            if sites.as_ref().is_none_or(|s| s.spacing_m() != spacing) {
                *sites = Some(super::cloud_puffs::PuffSites::new(spacing));
            }
            if let Some(sites) = sites.as_mut() {
                sites.build(
                    &maps.from,
                    &maps.maps,
                    maps.t,
                    camera,
                    &weather_settings,
                    instances,
                );
            }
        }
        let puff_count = instances.len() as u32;
        // With the puffs, the march pass only makes the near fog, which the
        // composite reads only where the layer is within its reach of the eye
        // (design 3a): further off, the pass is skipped.
        let reach = weather_settings.cloud_puff_near_m;
        let eye_r = camera.length();
        let near_fog_needed = !puffs_drawn
            || (eye_r > crate::sky::CLOUD_RADIUS - reach
                && eye_r < crate::sky::CLOUD_RADIUS + crate::sky::CLOUD_THICKNESS + reach);
        let lens_needed = lens_rain > 0.001 || drips > 0.001 || mist_active;
        let map = rain_map.as_deref().filter(|map| {
            !map.values.is_empty() && map.values.iter().any(|value| value.abs() > 1e-3)
        });
        let rain_needed = map.is_some()
            && camera.length() < clouds.clouds.x
            && weather_settings.rain_volume_range_m > 0.0;
        let size = UVec2::new(view.viewport.z.max(1), view.viewport.w.max(1));
        let history_size = (size.as_vec2() * weather_settings.cloud_render_scale)
            .ceil()
            .as_uvec2()
            .max(UVec2::ONE);
        if let Some(mut gpu) = existing {
            let resized = gpu.history_size != history_size;
            if resized {
                gpu.history = cloud_history(&device, history_size);
                gpu.cloud_depth = cloud_distances(&device, history_size);
                gpu.current = cloud_current(&device, history_size);
                gpu.history_size = history_size;
            }
            // The history is usable when the pass wrote it on the previous
            // frame, the view kept its size and the camera stepped rather
            // than jumped.
            let valid = gpu.clouds_ran
                && !resized
                && camera.distance(gpu.prev_camera) < CLOUD_HISTORY_JUMP_M;
            gpu.history_read = 1 - gpu.history_read;
            gpu.frame = gpu.frame.wrapping_add(1);
            if mist_active {
                gpu.mist_read = 1 - gpu.mist_read;
            }
            gpu.mist_until = mist_until;
            gpu.mist_active = mist_active;
            gpu.last_frame = Some(now);
            let mut params = params;
            params.cloud_history = Vec4::new(
                CLOUD_BLEND,
                if valid { 1.0 } else { 0.0 },
                (gpu.frame % 65_536) as f32,
                weather_settings.cloud_haze,
            );
            gpu.puffs_on = puffs_drawn;
            gpu.near_fog_needed = near_fog_needed;
            gpu.puff_count = puff_count;
            if puff_count > 0 {
                let bytes: &[u8] = bytemuck::cast_slice(instances.as_slice());
                if gpu.puff_buffer.size() < bytes.len() as u64 {
                    gpu.puff_buffer = puff_buffer(&device, instances.len());
                }
                queue.write_buffer(&gpu.puff_buffer, 0, bytes);
            }
            gpu.prev_clip = clip_from_body;
            gpu.prev_camera = camera;
            gpu.rain_velocity = rain_velocity;
            gpu.rain_fallen = rain_fallen;
            gpu.clouds_ran = !overlay_needed;
            gpu.uniform.set(params);
            gpu.uniform.write_buffer(&device, &queue);
            gpu.lens_needed = lens_needed;
            gpu.rain_needed = rain_needed;
            gpu.overlay_needed = overlay_needed;
            if let Some(map) = map {
                queue.write_buffer(&gpu.rain_buffer, 0, bytemuck::cast_slice(&map.values));
            }
            gpu.was_under = was_under;
            gpu.emerge_until = emerge_until;
            if gpu.depth_size != size {
                gpu.depth = water_depth(&device, size);
                gpu.depth_size = size;
            }
            continue;
        }
        let mut uniform = UniformBuffer::from(params);
        uniform.write_buffer(&device, &queue);
        let puff_buffer = puff_buffer(&device, instances.len());
        if puff_count > 0 {
            queue.write_buffer(&puff_buffer, 0, bytemuck::cast_slice(instances.as_slice()));
        }
        let puffs_on = puffs_drawn;
        // The flow field: zero for every cell, because the world has no rivers
        // and no water voxels yet. See openspec/changes/water-flow.
        let flow = device.create_buffer(&BufferDescriptor {
            label: Some("GPU water flow vectors (zero until a fluid state exists)"),
            size: (planet.slots as u64 * 8).max(8),
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let rain_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("precipitation map"),
            size: RAIN_MAP_MAX * RAIN_MAP_MAX * 4,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if let Some(map) = map {
            queue.write_buffer(&rain_buffer, 0, bytemuck::cast_slice(&map.values));
        }
        let data_bind_group = device.create_bind_group(
            Some("water data"),
            &cache.get_bind_group_layout(&pipelines.data_layout),
            &BindGroupEntries::sequential((
                &uniform,
                planet.cells.as_entire_binding(),
                planet_view.water.as_entire_binding(),
                flow.as_entire_binding(),
                rain_buffer.as_entire_binding(),
            )),
        );
        let format = if view.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };
        let multisampled = msaa.samples() > 1;
        let mut pipeline = |pass| {
            specialized.specialize(
                &cache,
                &pipelines,
                WaterKey {
                    pass,
                    format,
                    multisampled,
                },
            )
        };
        commands.entity(entity).insert(WaterViewGpu {
            cap: pipeline(Pass::Cap),
            compose: pipeline(Pass::Compose),
            cloud_march: pipeline(Pass::CloudMarch),
            cloud_resolve: pipeline(Pass::CloudResolve),
            cloud_puffs: pipeline(Pass::CloudPuffs),
            clouds: pipeline(Pass::Clouds),
            lens: pipeline(Pass::Lens),
            lens_mist: pipeline(Pass::LensMist),
            rain: pipeline(Pass::Rain),
            overlay: pipeline(Pass::Overlay),
            overlay_needed,
            rain_buffer,
            rain_needed,
            depth: water_depth(&device, size),
            depth_size: size,
            uniform,
            _flow: flow,
            data_bind_group,
            multisampled,
            lens_needed,
            was_under,
            emerge_until,
            history: cloud_history(&device, history_size),
            history_read: 0,
            history_size,
            cloud_depth: cloud_distances(&device, history_size),
            current: cloud_current(&device, history_size),
            prev_clip: clip_from_body,
            prev_camera: camera,
            clouds_ran: !overlay_needed,
            frame: 0,
            mist: std::array::from_fn(|_| cloud_target(&device, UVec2::ONE, MIST_FORMAT)),
            mist_read: 0,
            mist_until,
            mist_active,
            last_frame: Some(now),
            puffs_on,
            puff_buffer,
            puff_count,
            near_fog_needed,
            rain_velocity,
            rain_fallen,
        });
    }
}

/// Where the march stops for a ray that meets nothing, metres:
/// `CLOUD_FAR_CAP_M` in `water.wgsl`. The puffs clear their distances to it.
const CLOUD_FAR_CAP_M: f32 = 60_000.0;

/// The clouds' history pair for a view of this size, cleared by the first
/// pass that writes it.
fn cloud_history(device: &RenderDevice, size: UVec2) -> [TextureView; 2] {
    std::array::from_fn(|_| cloud_target(device, size, CLOUD_HISTORY_FORMAT))
}

/// The clouds' distances pair, the history's size.
fn cloud_distances(device: &RenderDevice, size: UVec2) -> [TextureView; 2] {
    std::array::from_fn(|_| cloud_target(device, size, CLOUD_DEPTH_FORMAT))
}

/// This frame's march targets (`cloud-entry`): its clouds, their distances,
/// and the near field's fog.
fn cloud_current(device: &RenderDevice, size: UVec2) -> [TextureView; 3] {
    [
        cloud_target(device, size, CLOUD_HISTORY_FORMAT),
        cloud_target(device, size, CLOUD_DEPTH_FORMAT),
        cloud_target(device, size, CLOUD_HISTORY_FORMAT),
    ]
}

/// The puffs' instances (`cloud-puffs`), room for at least `count` and never
/// fewer than a full sky's worth, so it is rarely remade.
fn puff_buffer(device: &RenderDevice, count: usize) -> Buffer {
    let room = count.max(8_192).next_power_of_two();
    device.create_buffer(&BufferDescriptor {
        label: Some("cloud puff instances"),
        size: (room * size_of::<super::cloud_puffs::PuffInstance>()) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// One of the clouds' march targets, drawn to and then read.
fn cloud_target(device: &RenderDevice, size: UVec2, format: TextureFormat) -> TextureView {
    device
        .create_texture(&TextureDescriptor {
            label: Some("clouds march target"),
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&TextureViewDescriptor::default())
}

fn water_depth(device: &RenderDevice, size: UVec2) -> TextureView {
    device
        .create_texture(&TextureDescriptor {
            label: Some("water sheet depth"),
            size: Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: WATER_DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&TextureViewDescriptor::default())
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(super) struct WaterCompositeLabel;

/// `PBD_NO_CLOUDS=1`: a pricing instrument (`perf-rig`), not a setting. It skips
/// the cloud march and composite so a run can say what the clouds cost on the
/// same flight. `PBD_PASS_OFF=rain,clouds,overlay,lens` does the same for any
/// of the composite's passes, to find which one draws an artefact.
static NO_CLOUDS: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var_os("PBD_NO_CLOUDS").is_some() || pass_off("clouds"));
fn pass_off(name: &str) -> bool {
    std::env::var("PBD_PASS_OFF").is_ok_and(|list| list.split(',').any(|p| p.trim() == name))
}
static PASSES_OFF: std::sync::LazyLock<[bool; 4]> = std::sync::LazyLock::new(|| {
    [
        pass_off("rain"),
        pass_off("overlay"),
        pass_off("lens"),
        pass_off("mist"),
    ]
});

#[derive(Default)]
struct WaterCompositeNode;

impl ViewNode for WaterCompositeNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static PlanetViewGpu,
        &'static WaterViewGpu,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext<'w>,
        (target, depth, planet_view, water): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let pipelines = world.resource::<WaterPipelines>();
        let cache = world.resource::<PipelineCache>();
        let (
            Some(cap),
            Some(compose),
            Some(cloud_march),
            Some(cloud_resolve),
            Some(clouds),
            Some(lens),
            Some(lens_mist),
            Some(rain),
            Some(overlay),
            Some(maps),
        ) = (
            cache.get_render_pipeline(water.cap),
            cache.get_render_pipeline(water.compose),
            cache.get_render_pipeline(water.cloud_march),
            cache.get_render_pipeline(water.cloud_resolve),
            cache.get_render_pipeline(water.clouds),
            cache.get_render_pipeline(water.lens),
            cache.get_render_pipeline(water.lens_mist),
            cache.get_render_pipeline(water.rain),
            cache.get_render_pipeline(water.overlay),
            world.get_resource::<super::weather_maps::WeatherMapGpu>(),
        )
        else {
            return Ok(());
        };
        let scene_layout = if water.multisampled {
            &pipelines.scene_layout_multisampled
        } else {
            &pipelines.scene_layout
        };
        let device = ctx.render_device().clone();
        // GPU time per pass, recorded only when `RenderDiagnosticsPlugin` is
        // present (`--frame-log`); otherwise these spans do nothing.
        let diagnostics = ctx.diagnostic_recorder();
        let clouds_on = !water.overlay_needed && !*NO_CLOUDS;
        // The clouds' GPU time at their passes' boundaries, where Bevy's
        // spans cannot reach (`planet_cloud_timer.rs`).
        let timer = world.get_resource::<super::cloud_timer::CloudTimer>();
        if clouds_on && let Some(timer) = timer {
            timer.begin_frame();
        }
        let scene_bind_group = |source: &TextureView| {
            device.create_bind_group(
                Some("water scene"),
                &cache.get_bind_group_layout(scene_layout),
                &BindGroupEntries::sequential((source, depth.view(), &pipelines.sampler)),
            )
        };
        // Compose then the cap, both reading the scene and writing the other
        // main texture. Compose touches every pixel, so nothing needs clearing.
        {
            let post = target.post_process_write();
            let scene = scene_bind_group(post.source);
            let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("Water compose and cap"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: post.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                // Reverse-Z: clear to the far plane so the first sheet fragment
                // at any pixel wins and nearer ones overwrite it.
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &water.depth,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &water.data_bind_group, &[]);
            pass.set_bind_group(1, &scene, &[]);
            pass.set_bind_group(2, &maps.bind_group, &[]);
            let span = diagnostics.pass_span(&mut pass, "water_compose");
            pass.set_render_pipeline(compose);
            pass.draw(0..3, 0..1);
            pass.set_render_pipeline(cap);
            pass.draw_indirect(&planet_view.indirect, 32);
            span.end(&mut pass);
        }
        // The march reads last frame's history and writes this frame's, and
        // the cloud's distances; the composite reads both.
        let history_write = &water.history[1 - water.history_read];
        let depth_write = &water.cloud_depth[1 - water.history_read];
        let composite_group = device.create_bind_group(
            Some("clouds march and its distances"),
            &cache.get_bind_group_layout(&pipelines.composite_layout),
            &BindGroupEntries::with_indices((
                (0, history_write),
                (1, &pipelines.sampler),
                (4, depth_write),
                (7, &water.current[2]),
            )),
        );
        if clouds_on {
            let march_group = device.create_bind_group(
                Some("clouds history and cells"),
                &cache.get_bind_group_layout(&pipelines.history_layout),
                &BindGroupEntries::sequential((
                    &water.history[water.history_read],
                    &pipelines.sampler,
                    &pipelines.cells,
                    &pipelines.cells_sampler,
                    &water.cloud_depth[water.history_read],
                )),
            );
            // The march reads only the scene's depth; the colour bound beside
            // it is whichever main texture is current.
            let scene = scene_bind_group(target.main_texture_view());
            if water.near_fog_needed {
                // The march writes every texel of its history, so nothing there
                // needs keeping.
                let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                    label: Some("Cloud march"),
                    color_attachments: &[&water.current[0], &water.current[1], &water.current[2]]
                        .map(|view| {
                            Some(RenderPassColorAttachment {
                                view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(LinearRgba::NONE.into()),
                                    store: StoreOp::Store,
                                },
                            })
                        }),
                    depth_stencil_attachment: None,
                    timestamp_writes: timer.and_then(|t| t.writes(0)),
                    occlusion_query_set: None,
                });
                pass.set_bind_group(0, &water.data_bind_group, &[]);
                pass.set_bind_group(1, &scene, &[]);
                pass.set_bind_group(2, &maps.bind_group, &[]);
                pass.set_bind_group(3, &march_group, &[]);
                let span = diagnostics.pass_span(&mut pass, "cloud_march");
                pass.set_render_pipeline(cloud_march);
                pass.draw(0..3, 0..1);
                span.end(&mut pass);
                drop(pass);
            }
            if water.puffs_on {
                // The puffs (`cloud-puffs`) are drawn straight into the
                // history the composite reads: no noise, so nothing to
                // accumulate. Its distances clear to the far cap, which the
                // composite reads as no cloud, and keep the nearest.
                let Some(puffs) = cache.get_render_pipeline(water.cloud_puffs) else {
                    return Ok(());
                };
                let puff_group = device.create_bind_group(
                    Some("cloud puffs"),
                    &cache.get_bind_group_layout(&pipelines.puff_layout),
                    &BindGroupEntries::with_indices(((10, water.puff_buffer.as_entire_binding()),)),
                );
                let clears = [
                    LinearRgba::NONE,
                    LinearRgba::rgb(CLOUD_FAR_CAP_M, CLOUD_FAR_CAP_M, 0.0),
                ];
                let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                    label: Some("Cloud puffs"),
                    color_attachments: &[(history_write, clears[0]), (depth_write, clears[1])].map(
                        |(view, clear)| {
                            Some(RenderPassColorAttachment {
                                view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(clear.into()),
                                    store: StoreOp::Store,
                                },
                            })
                        },
                    ),
                    depth_stencil_attachment: None,
                    timestamp_writes: timer.and_then(|t| t.writes(1)),
                    occlusion_query_set: None,
                });
                pass.set_bind_group(0, &water.data_bind_group, &[]);
                pass.set_bind_group(1, &scene, &[]);
                pass.set_bind_group(2, &maps.bind_group, &[]);
                pass.set_bind_group(3, &puff_group, &[]);
                let span = diagnostics.pass_span(&mut pass, "cloud_puffs");
                pass.set_render_pipeline(puffs);
                pass.draw(0..6, 0..water.puff_count);
                span.end(&mut pass);
            } else {
                // The resolve folds this frame's march into the history
                // (`cloud-history-clip`): it reads the previous history and
                // distances and this frame's march and distances, and writes the
                // history the composite reads.
                let read = water.history_read;
                let resolve_group = device.create_bind_group(
                    Some("clouds resolve"),
                    &cache.get_bind_group_layout(&pipelines.resolve_layout),
                    &BindGroupEntries::with_indices((
                        (0, &water.history[read]),
                        (1, &pipelines.sampler),
                        (4, &water.cloud_depth[read]),
                        (5, &water.current[0]),
                        (6, &water.current[1]),
                    )),
                );
                let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                    label: Some("Cloud resolve"),
                    color_attachments: &[history_write, depth_write].map(|view| {
                        Some(RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(LinearRgba::NONE.into()),
                                store: StoreOp::Store,
                            },
                        })
                    }),
                    depth_stencil_attachment: None,
                    timestamp_writes: timer.and_then(|t| t.writes(1)),
                    occlusion_query_set: None,
                });
                pass.set_bind_group(0, &water.data_bind_group, &[]);
                pass.set_bind_group(1, &scene, &[]);
                pass.set_bind_group(2, &maps.bind_group, &[]);
                pass.set_bind_group(3, &resolve_group, &[]);
                let span = diagnostics.pass_span(&mut pass, "cloud_resolve");
                pass.set_render_pipeline(cloud_resolve);
                pass.draw(0..3, 0..1);
                span.end(&mut pass);
            }
        }
        // The lens mist's probe (`lens-weather`): one pixel, the cloud's
        // density at the eye, eased into the mist the lens draws.
        let mist_write = &water.mist[1 - water.mist_read];
        if water.mist_active {
            let group = device.create_bind_group(
                Some("lens mist probe"),
                &cache.get_bind_group_layout(&pipelines.mist_layout),
                &BindGroupEntries::with_indices((
                    (2, &pipelines.cells),
                    (3, &pipelines.cells_sampler),
                    (8, &water.mist[water.mist_read]),
                )),
            );
            let scene = scene_bind_group(target.main_texture_view());
            let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("Lens mist probe"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: mist_write,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(LinearRgba::NONE.into()),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &water.data_bind_group, &[]);
            pass.set_bind_group(1, &scene, &[]);
            pass.set_bind_group(2, &maps.bind_group, &[]);
            pass.set_bind_group(3, &group, &[]);
            let span = diagnostics.pass_span(&mut pass, "lens_mist");
            pass.set_render_pipeline(lens_mist);
            pass.draw(0..3, 0..1);
            span.end(&mut pass);
        }
        // The sun's cascades for the lit rain (`lighting-model`): the rain
        // waits a frame for them where they are not up yet.
        let rain_light_group = {
            let images = world.resource::<bevy::render::render_asset::RenderAssets<
                bevy::render::texture::GpuImage,
            >>();
            let storage = world.resource::<bevy::render::render_asset::RenderAssets<
                bevy::render::storage::GpuShaderStorageBuffer,
            >>();
            world
                .get_resource::<super::shadow::SunShadowMaps>()
                .and_then(|m| Some((images.get(&m.map)?, storage.get(&m.cascades)?)))
                .map(|(map, cascades)| {
                    device.create_bind_group(
                        Some("rain: the sun's cascades"),
                        &cache.get_bind_group_layout(&pipelines.rain_light_layout),
                        &BindGroupEntries::with_indices((
                            (11, &map.texture_view),
                            (12, &map.sampler),
                            (13, cascades.buffer.as_entire_binding()),
                        )),
                    )
                })
        };
        let lens_group = device.create_bind_group(
            Some("lens mist"),
            &cache.get_bind_group_layout(&pipelines.lens_layout),
            &BindGroupEntries::with_indices(((9, mist_write),)),
        );
        for (needed, pipeline, label, span_name) in [
            // An overlay is a map: the clouds would hide the data under them,
            // and the cloud overlay is where cloud is shown then.
            (clouds_on, clouds, "Clouds", "clouds"),
            (
                water.rain_needed && !PASSES_OFF[0] && rain_light_group.is_some(),
                rain,
                "Rain volume",
                "rain",
            ),
            // After the rain, so the map reads through a storm, and before
            // the lens, so the drops stay on top of it.
            (
                water.overlay_needed && !PASSES_OFF[1],
                overlay,
                "Weather overlay",
                "weather_overlay",
            ),
            (
                water.lens_needed && !PASSES_OFF[2],
                lens,
                "Water lens",
                "water_lens",
            ),
        ] {
            if !needed {
                continue;
            }
            let post = target.post_process_write();
            let scene = scene_bind_group(post.source);
            let is_clouds = label == "Clouds";
            let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some(label),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: post.destination,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: if is_clouds {
                    timer.and_then(|t| t.writes(2))
                } else {
                    None
                },
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &water.data_bind_group, &[]);
            pass.set_bind_group(1, &scene, &[]);
            pass.set_bind_group(2, &maps.bind_group, &[]);
            if is_clouds {
                pass.set_bind_group(3, &composite_group, &[]);
            } else if label == "Water lens" {
                pass.set_bind_group(3, &lens_group, &[]);
            } else if label == "Rain volume"
                && let Some(group) = &rain_light_group
            {
                pass.set_bind_group(3, group, &[]);
            }
            let span = diagnostics.pass_span(&mut pass, span_name);
            pass.set_render_pipeline(pipeline);
            pass.draw(0..3, 0..1);
            span.end(&mut pass);
        }
        if clouds_on && let Some(timer) = timer {
            timer.end_frame(ctx.command_encoder());
        }
        Ok(())
    }
}

pub(super) fn build(render_app: &mut SubApp) {
    render_app
        .init_resource::<SpecializedRenderPipelines<WaterPipelines>>()
        .add_systems(
            RenderStartup,
            (initialize_pipelines, super::cloud_timer::init),
        )
        .add_systems(
            Render,
            super::cloud_timer::read_back.in_set(RenderSystems::Cleanup),
        )
        .add_systems(
            Render,
            prepare_water_views
                .in_set(RenderSystems::PrepareBindGroups)
                .after(super::prepare_views),
        )
        .add_render_graph_node::<ViewNodeRunner<WaterCompositeNode>>(Core3d, WaterCompositeLabel)
        .add_render_graph_edges(
            Core3d,
            (
                Node3d::EndMainPass,
                WaterCompositeLabel,
                Node3d::StartMainPassPostProcessing,
            ),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn occupied_camera_water_state_uses_this_frames_translated_eye() {
        use std::sync::Arc;
        let ocean = super::super::topology::dual_sphere(3)
            .into_iter()
            .find(|c| terrain::surface_height(c.direction) < -8.0)
            .unwrap()
            .direction;
        let set = Arc::new(super::super::lod::generate_fine(
            ocean,
            &crate::config::ColumnSettings::default(),
            &pbd_core::edits::Edits::new(),
        ));
        let mut contact = crate::planet::PlanetContact::test_planet(5);
        contact.set_fine(&set);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin));
        install_eye_water(&mut app);
        let offset = bevy::math::DVec3::new(8192.0, -4096.0, 2048.0);
        app.insert_resource(PlanetRenderFrame { center: offset })
            .insert_resource(contact)
            .insert_resource(crate::planet::PlanetFine { set, version: 1 });
        let above = (offset + (ocean * (terrain::PLANET_RADIUS + 3.0)).as_dvec3()).as_vec3();
        let below = (offset + (ocean * (terrain::PLANET_RADIUS - 2.0)).as_dvec3()).as_vec3();
        let parked = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Camera {
                    is_active: false,
                    ..default()
                },
                Transform::from_translation(above),
            ))
            .id();
        let occupied = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Camera::default(),
                crate::vehicles::VehicleCamera,
                Transform::from_translation(above),
            ))
            .id();
        app.update();
        assert_eq!(app.world().resource::<EyeWaterState>().0, EyeWater::Air);
        // Vehicle following runs here, after Update but before propagation.
        app.add_systems(
            PostUpdate,
            (move |mut camera: Query<&mut Transform, With<crate::vehicles::VehicleCamera>>| {
                camera.single_mut().unwrap().translation = below;
            })
            .before(bevy::transform::TransformSystems::Propagate),
        );
        app.update();
        assert!(matches!(
            app.world().resource::<EyeWaterState>().0,
            EyeWater::Water { .. }
        ));
        app.world_mut()
            .get_mut::<Camera>(occupied)
            .unwrap()
            .is_active = false;
        app.world_mut().get_mut::<Camera>(parked).unwrap().is_active = true;
        app.update();
        assert_eq!(app.world().resource::<EyeWaterState>().0, EyeWater::Air);
    }

    #[test]
    fn the_uniform_matches_the_wgsl_struct_size() {
        // Three mat4 and the vec4 lanes in water.wgsl's WaterView, read off the
        // shipped shader rather than remembered.
        let shader = include_str!("../../../assets/shaders/water.wgsl");
        let start = shader.find("struct WaterView {").unwrap();
        let block = &shader[start..start + shader[start..].find('}').unwrap()];
        let mat4 = block.matches("mat4x4<f32>").count();
        // The lit rain's lamps are one array of eight lanes (`lighting-model`).
        let lamps = block.matches("array<vec4<f32>, 8>").count();
        let vec4 = block.matches("vec4<f32>").count() + 7 * lamps;
        assert_eq!((mat4, vec4, lamps), (3, 59, 1));
        assert!(block.contains("sea: SeaView,"));
        let sea = crate::shader_tests::sea_source();
        let sea_size = crate::shader_tests::wgsl_struct_size("sea.wgsl", &sea, "SeaView");
        assert_eq!(SeaView::min_size().get() as u32, sea_size);
        assert_eq!(
            WaterView::min_size().get() as usize,
            mat4 * 64 + vec4 * 16 + sea_size as usize,
            "the Rust uniform must be the WGSL struct's size"
        );
    }

    /// The shells' fall wraps where every streak cycle does, so the pattern
    /// never jumps (`rain-shells`).
    #[test]
    fn the_rain_shells_fall_wraps_on_every_cycle() {
        let shader = include_str!("../../../assets/shaders/water.wgsl");
        let line = shader
            .lines()
            .find(|l| l.starts_with("const RAIN_SHELL_CYCLE_M"))
            .unwrap();
        let list = &line[line.rfind('(').unwrap() + 1..line.rfind(')').unwrap()];
        let cycles: Vec<f32> = list.split(',').map(|v| v.trim().parse().unwrap()).collect();
        assert_eq!(cycles.len(), 6);
        for cycle in cycles {
            assert_eq!(RAIN_SHELL_WRAP_M % cycle, 0.0, "{cycle}");
        }
    }

    /// The shells lean to the eye's velocity and fall at the speed of the
    /// fall against it, eased, from where they were (`rain-shells`).
    #[test]
    fn the_rain_shells_follow_the_eye_and_fall_on() {
        let down = Vec3::NEG_Y;
        let fall = Vec2::new(9.0, 1.0);
        // Standing still: the rain falls past at its own speed.
        let (v, fallen) = rain_shell_step(Vec3::ZERO, Vec3::ZERO, 0.5, Vec2::ZERO, down, fall);
        assert_eq!(v, Vec3::ZERO);
        assert!((fallen.x - 4.5).abs() < 1e-4 && (fallen.y - 0.5).abs() < 1e-4);
        // Flying level at 50 m/s for a while: the velocity is followed, and
        // the fall past the eye is the slanted speed, so the streaks rush by.
        let (mut v, mut fallen) = (Vec3::ZERO, Vec2::ZERO);
        for _ in 0..120 {
            (v, fallen) = rain_shell_step(v, Vec3::X * 50.0 / 60.0, 1.0 / 60.0, fallen, down, fall);
        }
        assert!((v - Vec3::X * 50.0).length() < 0.5, "{v}");
        let (_, next) = rain_shell_step(v, Vec3::X * 50.0 / 60.0, 1.0 / 60.0, fallen, down, fall);
        let rushed = (next.x - fallen.x).rem_euclid(RAIN_SHELL_WRAP_M) * 60.0;
        assert!(
            (rushed - (50.0f32.powi(2) + 81.0).sqrt()).abs() < 0.5,
            "{rushed}"
        );
        // It wraps, and a bad frame neither moves it nor poisons it.
        let (_, wrapped) = rain_shell_step(
            Vec3::ZERO,
            Vec3::ZERO,
            1.0,
            Vec2::splat(RAIN_SHELL_WRAP_M - 1.0),
            down,
            Vec2::splat(3.0),
        );
        assert!((wrapped - Vec2::splat(2.0)).length() < 1e-3, "{wrapped}");
        let (v, same) = rain_shell_step(Vec3::ONE, Vec3::NAN, 0.1, Vec2::ONE, down, fall);
        assert_eq!((v, same), (Vec3::ZERO, Vec2::ONE));
    }

    /// The column has the last word: air at the eye is dry at any depth,
    /// which is the dry cave below sea level, and water at the eye is read
    /// against ITS OWN surface, which is what lets a pool be shallower than
    /// the sea.
    #[test]
    fn the_column_at_the_eye_decides_and_a_dry_cave_is_dry() {
        let sea = terrain::PLANET_RADIUS - 0.5;
        let band = 1.3;
        let cells = super::super::topology::dual_sphere(3);
        let ocean = cells
            .iter()
            .find(|c| terrain::surface_height(c.direction) < 0.0)
            .unwrap()
            .direction;
        // Twenty metres under the sea's own surface, over a cell whose ground
        // is under it: the height field drowns it and the column does not.
        let deep = ocean * (sea - 20.0);
        assert_eq!(submersion(deep, sea, 0.0, band, EyeWater::Unknown), 1.0);
        assert_eq!(submersion(deep, sea, 0.0, band, EyeWater::Air), 0.0);
        // A pool whose surface is ten metres down: over it is dry, in it is
        // under, and the band straddles its own surface rather than the sea's.
        let pool = EyeWater::Water {
            surface_radius: sea - 10.0,
        };
        assert_eq!(submersion(ocean * (sea - 5.0), sea, 0.0, band, pool), 0.0);
        assert_eq!(submersion(ocean * (sea - 10.5), sea, 0.0, band, pool), 0.5);
        assert_eq!(submersion(deep, sea, 0.0, band, pool), 1.0);
    }

    #[test]
    fn submersion_is_dry_over_land_and_a_tri_state_over_water() {
        let sea = terrain::PLANET_RADIUS - 0.5;
        let band = 1.3;
        // Find one land and one ocean direction off the real generator.
        let cells = super::super::topology::dual_sphere(3);
        let land = cells
            .iter()
            .find(|c| terrain::surface_height(c.direction) >= 0.0)
            .unwrap()
            .direction;
        let ocean = cells
            .iter()
            .find(|c| terrain::surface_height(c.direction) < 0.0)
            .unwrap()
            .direction;
        assert_eq!(
            submersion(land * (sea - 10.0), sea, 0.0, band, EyeWater::Unknown),
            0.0
        );
        assert_eq!(
            submersion(ocean * (sea + 10.0), sea, 0.0, band, EyeWater::Unknown),
            0.0
        );
        assert_eq!(
            submersion(ocean * (sea + 1.0), sea, 0.0, band, EyeWater::Unknown),
            0.5
        );
        assert_eq!(
            submersion(ocean * (sea - 1.0), sea, 0.0, band, EyeWater::Unknown),
            0.5
        );
        assert_eq!(
            submersion(ocean * (sea - 3.0), sea, 0.0, band, EyeWater::Unknown),
            1.0
        );
        assert_eq!(
            submersion(Vec3::ZERO, sea, 0.0, band, EyeWater::Unknown),
            0.0
        );
    }

    /// The band stands about the surface under the camera as it is now, not
    /// about the mean sea: a trough passing under a camera just over the sea
    /// leaves it dry, and a crest over it puts it in the water.
    #[test]
    fn in_a_trough_the_camera_is_dry() {
        let sea = terrain::PLANET_RADIUS - 0.5;
        let band = 0.8;
        let cells = super::super::topology::dual_sphere(3);
        let ocean = cells
            .iter()
            .find(|c| terrain::surface_height(c.direction) < 0.0)
            .unwrap()
            .direction;
        let eye = ocean * (sea + 0.3);
        assert_eq!(submersion(eye, sea, -1.0, band, EyeWater::Unknown), 0.0);
        assert_eq!(submersion(eye, sea, 2.0, band, EyeWater::Unknown), 1.0);
    }

    #[test]
    fn the_emerge_window_arms_on_surfacing_and_dries_off() {
        let dry = 2.6;
        let mut until = f32::NEG_INFINITY;
        assert_eq!(emerge(0.0, 1.0, false, &mut until, dry), 0.0);
        // Straddling keeps the lens wet.
        assert_eq!(emerge(1.0, 0.5, true, &mut until, dry), 1.0);
        // Clear of the water: full drips, then fading.
        assert_eq!(emerge(1.0, 0.0, false, &mut until, dry), 1.0);
        let half = emerge(1.0 + dry / 2.0, 0.0, false, &mut until, dry);
        assert!((half - 0.5).abs() < 1e-5);
        assert_eq!(emerge(1.0 + dry + 1.0, 0.0, false, &mut until, dry), 0.0);
        // Diving again: no drips while under, then a fresh window on surfacing.
        assert_eq!(emerge(10.0, 1.0, false, &mut until, dry), 0.0);
        assert_eq!(emerge(11.0, 0.0, true, &mut until, dry), 1.0);
    }
}
