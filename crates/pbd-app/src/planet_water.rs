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
use crate::config::{WaterSettings, WeatherSettings};
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
        extract_resource::ExtractResource,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{
                sampler, storage_buffer_read_only_sized, texture_2d, texture_depth_2d,
                texture_depth_2d_multisampled, uniform_buffer,
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
    /// The rain on the LENS, then three spare lanes. Its own lane rather than
    /// `fx.z`, which is the rain on the SEA: a camera in a cave mouth is dry
    /// while the sea it looks out at is still being rained on.
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
}

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
pub fn submersion(camera_body: Vec3, sea_radius: f32, band: f32, eye: EyeWater) -> f32 {
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
    let body = transform.translation() - frame.center.as_vec3();
    let Some(direction) = body.try_normalize() else {
        return EyeWater::Unknown;
    };
    let Some(column) = contact
        .finest_cell(direction)
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
    sampler: Sampler,
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
    fullscreen: Res<FullscreenShader>,
) {
    commands.insert_resource(WaterPipelines {
        shader: assets.load("shaders/water.wgsl"),
        fullscreen: fullscreen.clone(),
        data_layout: data_layout(),
        scene_layout: scene_layout(false),
        scene_layout_multisampled: scene_layout(true),
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
    Clouds,
    Rain,
    Overlay,
    Lens,
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
                "Water lens: droplets and emerge drips",
                self.fullscreen.to_vertex_state(),
                "lens",
            ),
        };
        // The sheet self-sorts against its own single-sample depth buffer, so
        // a near crest occludes a far trough whatever order the cells come in
        // (Tenebris: "depth write + LESS_EQUAL to self-sort"). Compose shares
        // the pass and so declares the same attachment, without touching it;
        // the scene's own occlusion is the shader's discard against the
        // sampled main-pass depth, which may be multisampled.
        let depth_stencil = match key.pass {
            Pass::Lens | Pass::Rain | Pass::Clouds | Pass::Overlay => None,
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
            layout: vec![
                self.data_layout.clone(),
                scene,
                super::weather_maps::layout(),
            ],
            vertex,
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: defs,
                entry_point: Some(Cow::Borrowed(entry)),
                targets: vec![Some(ColorTargetState {
                    format: key.format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
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
    clouds: CachedRenderPipelineId,
    lens: CachedRenderPipelineId,
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
    } = sky;
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
        let band = settings.swell_amplitude_m + settings.partial_band_m;
        let state = submersion(camera, sea_radius, band, eye.0);
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
        let params = WaterView {
            clip_from_local: clip_from_body,
            local_from_clip: clip_from_body.as_dmat4().inverse().as_mat4(),
            camera_time: camera.extend(clock.0 * s.time_scale),
            planet_center: Vec3::ZERO.extend(sea_radius),
            sun: sun.direction().extend(s.specular_intensity * sun_dim),
            waves: Vec4::new(
                s.swell_amplitude_m,
                s.swell_frequency,
                s.swell_speed,
                s.wave_steepness,
            ),
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
            lod: planet.lod.player.extend(super::lod::BASE_LEVEL as f32),
            bands: planet.lod.bands,
            rain: Vec4::new(lens_rain, 0.0, 0.0, 0.0),
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
        };
        let lens_needed = lens_rain > 0.001 || drips > 0.001;
        let map = rain_map.as_deref().filter(|map| {
            !map.values.is_empty() && map.values.iter().any(|value| value.abs() > 1e-3)
        });
        let rain_needed = map.is_some()
            && camera.length() < clouds.clouds.x
            && weather_settings.rain_volume_range_m > 0.0;
        let size = UVec2::new(view.viewport.z.max(1), view.viewport.w.max(1));
        if let Some(mut gpu) = existing {
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
            clouds: pipeline(Pass::Clouds),
            lens: pipeline(Pass::Lens),
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
        });
    }
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
struct WaterCompositeLabel;

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
            Some(clouds),
            Some(lens),
            Some(rain),
            Some(overlay),
            Some(maps),
        ) = (
            cache.get_render_pipeline(water.cap),
            cache.get_render_pipeline(water.compose),
            cache.get_render_pipeline(water.clouds),
            cache.get_render_pipeline(water.lens),
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
            pass.set_render_pipeline(compose);
            pass.draw(0..3, 0..1);
            pass.set_render_pipeline(cap);
            pass.draw_indirect(&planet_view.indirect, 32);
        }
        for (needed, pipeline, label) in [
            // An overlay is a map: the clouds would hide the data under them,
            // and the cloud overlay is where cloud is shown then.
            (!water.overlay_needed, clouds, "Clouds"),
            (water.rain_needed, rain, "Rain volume"),
            // After the rain, so the map reads through a storm, and before
            // the lens, so the drops stay on top of it.
            (water.overlay_needed, overlay, "Weather overlay"),
            (water.lens_needed, lens, "Water lens"),
        ] {
            if !needed {
                continue;
            }
            let post = target.post_process_write();
            let scene = scene_bind_group(post.source);
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
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &water.data_bind_group, &[]);
            pass.set_bind_group(1, &scene, &[]);
            pass.set_bind_group(2, &maps.bind_group, &[]);
            pass.set_render_pipeline(pipeline);
            pass.draw(0..3, 0..1);
        }
        Ok(())
    }
}

pub(super) fn build(render_app: &mut SubApp) {
    render_app
        .init_resource::<SpecializedRenderPipelines<WaterPipelines>>()
        .add_systems(RenderStartup, initialize_pipelines)
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
    fn the_uniform_matches_the_wgsl_struct_size() {
        // Two mat4 and the vec4 lanes in water.wgsl's WaterView, read off the
        // shipped shader rather than remembered.
        let shader = include_str!("../../../assets/shaders/water.wgsl");
        let start = shader.find("struct WaterView {").unwrap();
        let block = &shader[start..start + shader[start..].find('}').unwrap()];
        let mat4 = block.matches("mat4x4<f32>").count();
        let vec4 = block.matches("vec4<f32>").count();
        assert_eq!((mat4, vec4), (2, 39));
        assert_eq!(
            WaterView::min_size().get() as usize,
            mat4 * 64 + vec4 * 16,
            "the Rust uniform must be the WGSL struct's size"
        );
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
        assert_eq!(submersion(deep, sea, band, EyeWater::Unknown), 1.0);
        assert_eq!(submersion(deep, sea, band, EyeWater::Air), 0.0);
        // A pool whose surface is ten metres down: over it is dry, in it is
        // under, and the band straddles its own surface rather than the sea's.
        let pool = EyeWater::Water {
            surface_radius: sea - 10.0,
        };
        assert_eq!(submersion(ocean * (sea - 5.0), sea, band, pool), 0.0);
        assert_eq!(submersion(ocean * (sea - 10.5), sea, band, pool), 0.5);
        assert_eq!(submersion(deep, sea, band, pool), 1.0);
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
            submersion(land * (sea - 10.0), sea, band, EyeWater::Unknown),
            0.0
        );
        assert_eq!(
            submersion(ocean * (sea + 10.0), sea, band, EyeWater::Unknown),
            0.0
        );
        assert_eq!(
            submersion(ocean * (sea + 1.0), sea, band, EyeWater::Unknown),
            0.5
        );
        assert_eq!(
            submersion(ocean * (sea - 1.0), sea, band, EyeWater::Unknown),
            0.5
        );
        assert_eq!(
            submersion(ocean * (sea - 3.0), sea, band, EyeWater::Unknown),
            1.0
        );
        assert_eq!(submersion(Vec3::ZERO, sea, band, EyeWater::Unknown), 0.0);
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
