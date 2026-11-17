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
    /// rgb absorption per metre (the view's path); w the turbidity at the
    /// eye, per metre (`ocean-environments` B3).
    absorption: Vec4,
    /// rgb the deep water's albedo; w the falling-water flow speed.
    deep_color: Vec4,
    /// `water-lit`: rgb the shallows' albedo; w the Fresnel's horizon floor.
    shallow: Vec4,
    /// rgb the light on open water at the camera, the terrain's for a cap
    /// under open sky (the sky's L2 irradiance up, plus the sun after the air
    /// as dim as the cloud makes it); w the depth over which the shallows
    /// become the deep, m.
    water_light: Vec4,
    /// The terrain's haze terms (`finish` in `planet_surface.wgsl`):
    /// `cloud_fog_add`, `rain_fog_mult`, `overcast_sky_blue_cut`,
    /// `overcast_sky_dim`.
    haze: Vec4,
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
    /// The partition a landing dissolves from, as the terrain has it
    /// (`two-set-dissolve`): its fade centre, w the dissolve's progress 0..1
    /// (one outside a dissolve, when no sheet is marked); its bands and their
    /// rings' inner edges.
    lod_prev: Vec4,
    bands_prev: Vec4,
    bands_prev_in: Vec4,
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
    /// x how far the sheet sits below sea level, m; y the surface of the
    /// water the eye is in or over, a body radius: an inland water's drawn
    /// top (`inland_surface`), else the sea's tided surface at the eye; zw
    /// the water's easing between ticks.
    sea_frame: Vec4,
    /// The lens mist (`lens-weather`): seconds to mist over and to clear
    /// standing still, how much faster per m/s of airspeed, its strength.
    mist: Vec4,
    /// Its beads per screen height, its blur in screen heights, how far it
    /// lifts the image toward its own brightness; w the drops' scale
    /// (`rain_lens_scale`).
    mist_look: Vec4,
    /// A lens drop's edge (`lens-drop-refraction`): x the light it loses
    /// there (`rain_lens_rim`), y the sky's light it gathers on the lower
    /// part (`rain_lens_glint`); zw spare.
    lens_look: Vec4,
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
    /// `lighting-model` L8 (10.3): the screen trace's history frame, as the
    /// terrain's (`PlanetViewGpu::ssr_clip`), so the sea mirrors the land,
    /// the houses and the trees as the wet ground does.
    ssr_clip: Mat4,
    ssr_eye: Vec4,
    /// `water-lit`: the cover and the rain over the camera, for the haze; zw
    /// spare.
    haze_weather: Vec4,
    /// The sky's nine L2 coefficients at the camera, mixed by the cover
    /// (`sky_sh_pair`): the sky the sheet mirrors where the screen trace
    /// finds nothing nearer (`sh_radiance`).
    sky_sh: [Vec4; 9],
    /// Under the water (`ocean-environments` B5): the shafts' strength and
    /// reach, m; the snow's strength and the depth it shows from, m.
    under: Vec4,
    /// The snow's own glow, linear; the wind at the sea, m/s; the spray's
    /// start, m/s, and its strength.
    under_more: Vec4,
    /// The column tier: xyz its anchor, w its reach, m (zero: none). The far
    /// river's sheet and the tier's water dither across its edge (design 13).
    column: Vec4,
    /// The height fog in the rain volume's place while RAIN SHADOW is off
    /// (`weather-rain` 7): x the extinction per metre at the reference level
    /// (zero: none), y the falloff height, m, z the reference level's radius
    /// from the body's centre (the ground or the sea under the eye), m.
    height_fog: Vec4,
    /// A fall's look (`water-flow` 6.6, M35), `water.ron`'s `falls`: x the
    /// feet told of, y the teal churn, z the ripples' height, w the metres
    /// between rings.
    fall_look: Vec4,
    /// x the ripples' reach m, y the pool flow, z its reach m, w the speed-up
    /// down a face, g m/s^2 (zero: one speed).
    fall_flow: Vec4,
    /// The glassy lip: x one when on, y smooth this far before the edge m,
    /// z glassy this far over it m, w the froth filled in by m.
    fall_lip: Vec4,
    /// x the glassy green, y the edge highlight, z the fastest a face moves
    /// m/s, w the bubbles' count.
    fall_more: Vec4,
    /// The splash: x the wall blend m, y the mist's opacity, z one when the
    /// effects are smooth (zero pixelated), w the viewport's height px.
    fall_splash: Vec4,
    /// rgb the aerated water's colour, linear; w the water's time scale.
    aer_color: Vec4,
    /// The nearest feet (`water_flow::FallFeetGpu`): xyz where each lands,
    /// body-local, w its drop m; then x the pool's depth m, y a seed, z its
    /// fade.
    feet: [Vec4; crate::water_flow::FALL_FEET_MAX],
    feet_more: [Vec4; crate::water_flow::FALL_FEET_MAX],
}

/// `weather-rain` 7: the height fog's lane. Only with the rain volume off,
/// some rain over the eye, out of the sea and under the cloud base, where the
/// volume would have drawn; zero otherwise, and then the composite takes no
/// branch for it. `PBD_HEIGHT_FOG=0` prices it (zero always).
fn height_fog_lane(volume_off: bool, rain: f32, s: &WeatherSettings, camera: Vec3) -> Vec4 {
    let rain = rain.clamp(0.0, 1.0);
    if !volume_off || rain <= 1e-3 || *HEIGHT_FOG_OFF || s.rain_volume_range_m <= 0.0 {
        return Vec4::ZERO;
    }
    Vec4::new(
        rain * s.rain_volume_density,
        s.rain_height_fog_falloff_m * (0.5 + 0.5 * rain),
        crate::planet::terrain::terrain_radius(camera),
        0.0,
    )
}

/// `PBD_HEIGHT_FOG=0`: a pricing instrument, not a setting (`weather-rain` 7).
static HEIGHT_FOG_OFF: std::sync::LazyLock<bool> =
    std::sync::LazyLock::new(|| std::env::var("PBD_HEIGHT_FOG").is_ok_and(|v| v == "0"));

/// The light on open water and the sky's coefficients, both by the cover:
/// the terrain's light for a cap under open sky (`water-lit`). `e` is the
/// sine of the sun's elevation at the camera.
fn water_light(e: f32, cover: f32) -> (Vec3, [Vec4; 9]) {
    let cover = cover.clamp(0.0, 1.0);
    let pair = super::sky_sh_pair(e);
    let sh: [Vec4; 9] = std::array::from_fn(|k| pair[k].lerp(pair[k + 9], cover));
    let coeffs: [[f32; 3]; 9] = std::array::from_fn(|k| sh[k].truncate().to_array());
    let sky = Vec3::from_array(pbd_core::sky_light::sh_irradiance(&coeffs, [0.0, 1.0, 0.0]));
    let sun = Vec3::from_array(pbd_core::sky_light::sun_air(e)) * e.max(0.0) * (1.0 - 0.85 * cover);
    (sky.max(Vec3::ZERO) + sun, sh)
}

/// Whether the rain volume draws: the rain fog and the animated streaks on
/// its shells behind the near rain, which the owner calls the rain shadow
/// (`player-settings` 8). The player's choice, off by default; on where
/// nothing has chosen (a capture, a test), as before the setting existed.
#[derive(Resource, Clone, Copy, ExtractResource)]
pub struct RainShadows(pub bool);

impl Default for RainShadows {
    fn default() -> Self {
        Self(true)
    }
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
    /// The tide: the moon's direction and the range (`pbd_core::sea::tide_m`).
    tide: Vec4,
}

impl From<&pbd_core::sea::SeaGpu> for SeaView {
    fn from(sea: &pbd_core::sea::SeaGpu) -> Self {
        Self {
            directions: sea.directions,
            bands: sea.bands,
            phase: sea.phase,
            heading: sea.heading,
            limits: sea.limits,
            tide: sea.tide,
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

/// The format of the puffs' coverage as the air leaves it (`puff-haze`), the
/// history's size: one channel, blended "over" by each puff's own coverage,
/// which the history's alpha cannot also be.
const PUFF_COVER_FORMAT: TextureFormat = TextureFormat::R16Float;

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
    // The one surface every reader judges the eye against (B18, second
    // review): it took the column's own sea top, half a metre over the drawn
    // sheet, plus the tide, so an eye over the tided sheet was "under" and
    // its whole sky went to murk.
    let Some(surface) = eye_surface_radius(eye, camera_body, sea_radius, wave) else {
        return 0.0;
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

/// The surface of the water the eye is in, as a body radius, where that
/// water is not the sea's: a river, a pool, flowing water standing over the
/// sea's level (`ocean-environments` B17). `None` for the sea, for air and
/// off the tier. The underwater view is measured from this surface instead
/// of the sea's sphere, which a river stands above.
pub fn inland_surface(eye: EyeWater) -> Option<f32> {
    let sea = terrain::PLANET_RADIUS + terrain::terrain_config().sea_level_m;
    match eye {
        // Any water but the sea's own: a river over it, a pool in a cave
        // under it.
        EyeWater::Water { surface_radius } if (surface_radius - sea).abs() > 0.05 => {
            Some(surface_radius)
        }
        _ => None,
    }
}

/// The surface the eye's side of the water is judged against, as a body
/// radius: an inland water's drawn top; the sea's mean surface lifted by the
/// tide (never the waves) where the eye's water is the sea's, or where off
/// the tier the ground under it is below the sea; `None` in air. The ONE
/// answer the tri-state, the composite's measure and the capture label read.
pub fn eye_surface_radius(
    eye: EyeWater,
    camera_body: Vec3,
    sea_radius: f32,
    tide: f32,
) -> Option<f32> {
    match eye {
        EyeWater::Air => None,
        EyeWater::Water { .. } => Some(inland_surface(eye).unwrap_or(sea_radius + tide)),
        EyeWater::Unknown => (camera_body.length_squared() > 1e-6
            && terrain::surface_height(camera_body) < 0.0)
            .then_some(sea_radius + tide),
    }
}

/// The straddle band about an inland surface, m: none. A river or a pool has
/// no swell for the band to hold, so the eye is under its water as soon as it
/// is below the drawn surface, and over it as soon as it is above: a band
/// let the eye a few centimetres under the surface see the bank above it
/// past the sheet's edge, dry and bright (the frames of the waterline video).
pub const INLAND_BAND_M: f32 = 0.0;

/// Publish what the tier says is at the active camera's eye.
pub(super) fn install_eye_water(app: &mut App) {
    app.init_resource::<EyeWaterState>().add_systems(
        PostUpdate,
        publish_eye_water
            .after(bevy::transform::TransformSystems::Propagate)
            .run_if(crate::world_open),
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
    settings: Option<Res<WaterSettings>>,
    mut eye: ResMut<EyeWaterState>,
) {
    let high_water = settings
        .as_deref()
        .map_or(0.0, |s| pbd_core::sea::tide_high_m(s.tide_range_m));
    let answer = eye_water(
        &cameras,
        &frame,
        contact.as_deref(),
        fine.as_deref(),
        high_water,
    );
    if eye.0 != answer {
        eye.0 = answer;
    }
}

fn eye_water(
    cameras: &Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: &PlanetRenderFrame,
    contact: Option<&crate::planet::PlanetContact>,
    fine: Option<&crate::planet::PlanetFine>,
    high_water: f32,
) -> EyeWater {
    let (Some(contact), Some(fine)) = (contact, fine) else {
        return EyeWater::Unknown;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return EyeWater::Unknown;
    };
    let body = (transform.translation().as_dvec3() - frame.center).as_vec3();
    eye_water_at(body, contact, fine, high_water)
}

/// What the tier says is at a body-local eye: the one rule
/// `publish_eye_water` applies to the active camera, for any reader that
/// must give the same answer (the capture label).
pub fn eye_water_at(
    body: Vec3,
    contact: &crate::planet::PlanetContact,
    fine: &crate::planet::PlanetFine,
    high_water: f32,
) -> EyeWater {
    let Some(direction) = body.try_normalize() else {
        return EyeWater::Unknown;
    };
    let Some(record) = contact
        .finest_cell(direction)
        .filter(|_| contact.serves(&fine.set))
    else {
        return EyeWater::Unknown;
    };
    let Some(column) = fine.set.columns.column(record) else {
        return EyeWater::Unknown;
    };
    let altitude = body.length() - terrain::PLANET_RADIUS;
    match column.water_surface(altitude) {
        Some(surface) => {
            // The surface the eye SEES: a flowing top is drawn through its
            // corners, up to a layer's share over the cell's own level, and
            // the eye between the two is under the drawn water.
            let top = pbd_core::column::layer_at(surface - 1e-3);
            let drawn = fine
                .set
                .columns
                .slot(record)
                .zip(top)
                .and_then(|(slot, top)| {
                    crate::water_flow::drawn_top_m(&fine.set.columns, slot, top)
                });
            EyeWater::Water {
                surface_radius: terrain::PLANET_RADIUS + drawn.unwrap_or(surface),
            }
        }
        // Air the sea's surface reaches at some tide: the sea's, at its
        // mean; `submersion` adds the tide and the swell, so an eye under the
        // tided sheet over a beach is under the sea there (the owner's
        // "submerged at just the right height": one layer down at high
        // water the column held only air, and the eye was dry under the
        // sheet). The sea's reach is the mark the sheet is listed by
        // (`ocean-environments` B22): a pit or a cavern the tide overtops is
        // the sea's, roofed or not, and a hollow no water joins is dry.
        None if altitude < terrain::terrain_config().sea_level_m + high_water
            && pbd_core::column::layer_at(altitude)
                .is_some_and(|l| column.material(l) == pbd_core::terrain::Material::Air)
            && fine
                .set
                .finest_records()
                .get(record)
                .is_some_and(crate::planet::column::sea_reached) =>
        {
            EyeWater::Water {
                surface_radius: terrain::PLANET_RADIUS + terrain::terrain_config().sea_level_m,
            }
        }
        // Rock reads as air: a camera inside a wall sees nothing, and what it
        // must not do is fill with sea.
        None => EyeWater::Air,
    }
}

/// The surface the composite measures the waterline from, as a body radius
/// (`sea_frame.y`): the eye's own water's ([`eye_surface_radius`]), and for an
/// eye in air the sea's TIDED surface (`ocean-environments` B21). It was
/// nought in air, and the shader fell back to the sea's mean sphere: at low
/// tide a dry beach between the tided water and the mean was taken for
/// under water, tinted blue-green and wobbled, whenever the eye rose out of
/// the high tide's reach over it, as in a jump (owner, 2026-11-16: "this
/// only happens when you jump above water").
pub fn composite_surface_radius(
    eye: EyeWater,
    camera_body: Vec3,
    sea_radius: f32,
    tide: f32,
) -> f32 {
    eye_surface_radius(eye, camera_body, sea_radius, tide).unwrap_or(sea_radius + tide)
}

/// The drops on the lens (`weather-rain` R3): how wet the glass is, 0..1,
/// following the rain that reaches it exponentially, up on `lens_wet_tau_s`
/// and down on `lens_dry_tau_s`; and the drops' clock, seconds, which runs
/// while rain lands on the glass and stops while none does, so no new drop
/// appears on a drying lens and the drops there stay put as they shrink and
/// fade. On the planet clock, so a fixed-step capture dries as play does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LensWet {
    pub wet: f32,
    pub clock: f32,
    last_s: f32,
    /// When the instrument last logged (`weather::weather_scripted`).
    last_trace: f32,
}

impl LensWet {
    pub fn step(&mut self, rain: f32, now_s: f32, wet_tau_s: f32, dry_tau_s: f32) {
        let dt = (now_s - self.last_s).clamp(0.0, 0.5);
        self.last_s = now_s;
        let rain = rain.clamp(0.0, 1.0);
        let tau = if rain > self.wet {
            wet_tau_s
        } else {
            dry_tau_s
        };
        self.wet = crate::weather::settle_wetness(self.wet, rain, dt, tau);
        if self.wet < 1e-3 && rain <= 0.0 {
            self.wet = 0.0;
        }
        // The drops run at the rain's pace against what is on the glass: in
        // steady rain one, while it builds up one, while it dries none.
        let rate = (rain / self.wet.max(1e-3)).min(1.0);
        self.clock += dt * rate;
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
    rain_compose_layout: BindGroupLayoutDescriptor,
    /// The puffs' fourth group (`cloud-puffs`): this frame's instances.
    puff_layout: BindGroupLayoutDescriptor,
    /// The fall splash's fourth group (`water-flow` 6.1): its quads.
    splash_layout: BindGroupLayoutDescriptor,
    /// The rain's fourth group (`lighting-model`, the rain fog lit): the sun's
    /// cascades, their comparison sampler and where each stands.
    rain_light_layout: BindGroupLayoutDescriptor,
    /// The compose and the cap's fourth group (`lighting-model` L8): the
    /// screen trace's history.
    ssr_layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    cells: TextureView,
    cells_sampler: Sampler,
    /// The mist puffs' sheet (`water-flow` 6.6): 4 x 4 cells of 32 texels,
    /// sRGB, so it samples linear.
    mist_puffs: TextureView,
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
                // The puffs' coverage as the air leaves it (`puff-haze`).
                (
                    16,
                    texture_2d(TextureSampleType::Float { filterable: false }),
                ),
                // The sheet's own depth: a sky pixel the sheet did not draw
                // is the gap at the sea's horizon (`horizon-haze` 4).
                (20, texture_depth_2d()),
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

/// The fall splash's quads (`water-flow` 6.1), as `SplashQuads` holds them.
fn splash_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "water: the fall splash",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::VERTEX_FRAGMENT,
            (
                (
                    17,
                    storage_buffer_read_only_sized(false, NonZeroU64::new(16)),
                ),
                // The mist puffs' sheet and its sampler (`water-flow` 6.6).
                (
                    18,
                    texture_2d(TextureSampleType::Float { filterable: true }),
                ),
                (19, sampler(SamplerBindingType::Filtering)),
                // The water's own depth, for the soft fade and the test.
                (20, texture_depth_2d()),
            ),
        ),
    )
}

/// Room for the splash's whole budget: the camera's right and up, then
/// `SPLASH_LANES` a quad.
const SPLASH_BUFFER_BYTES: u64 =
    ((2 + crate::water_flow::SPLASH_LANES * crate::water_flow::SPLASH_BUDGET) * 16) as u64;

fn ssr_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "water: the screen trace's history",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            ((
                14,
                texture_2d(TextureSampleType::Float { filterable: false }),
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
                // The sheet's own depth (`horizon-haze` 4), as the clouds'.
                (20, texture_depth_2d()),
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

/// The mist puffs' sheet (`assets/textures/fx/mist_puffs`, `SOURCE.md`),
/// built into the binary and uploaded once.
fn mist_puffs(device: &RenderDevice, queue: &RenderQueue) -> TextureView {
    let png = include_bytes!("../../../assets/textures/fx/mist_puffs/albedo.png");
    let image = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .map(|i| i.to_rgba8())
        .unwrap_or_else(|_| image::RgbaImage::new(1, 1));
    let extent = Extent3d {
        width: image.width(),
        height: image.height(),
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("fall mist puffs"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8UnormSrgb,
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
        image.as_raw(),
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * image.width()),
            rows_per_image: Some(image.height()),
        },
        extent,
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
                storage_buffer_read_only_sized(false, NonZeroU64::new(16)),
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
        mist_puffs: mist_puffs(&device, &queue),
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
        ssr_layout: ssr_layout(),
        composite_layout: composite_layout(),
        resolve_layout: resolve_layout(),
        mist_layout: mist_layout(),
        lens_layout: lens_layout(),
        rain_compose_layout: rain_compose_layout(),
        puff_layout: puff_layout(),
        splash_layout: splash_layout(),
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
    /// The game's own water, by the cap's fragment (`water-flow` E6).
    Flowing,
    /// The fall splash, after the water and against its depth (`water-flow`
    /// 6.1).
    Splash,
    /// The bubbles under a fall's foot (`water-flow` 6.6), in the splash's
    /// pass before it.
    Bubbles,
    Compose,
    CloudMarch,
    CloudResolve,
    CloudPuffs,
    Clouds,
    Rain,
    /// The shells' light and transmittance, at `rain_render_scale`.
    RainHalf,
    /// The scene under the small target's rain, at full size.
    RainCompose,
    Overlay,
    Lens,
    LensMist,
}

/// The rain volume's small target (`lighting-perf` 2.9): rgb the light the
/// shells gather, a their transmittance.
const RAIN_HALF_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

/// What the rain's full-size compose reads besides the scene: the small
/// target.
fn rain_compose_layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "rain: the small target",
        &BindGroupLayoutEntries::with_indices(
            ShaderStages::FRAGMENT,
            (
                (
                    15,
                    texture_2d(TextureSampleType::Float { filterable: false }),
                ),
                // The sheet's own depth (`horizon-haze` 4), as the clouds'.
                (20, texture_depth_2d()),
            ),
        ),
    )
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
            Pass::Flowing => (
                "Flowing water by the sea's own fragment",
                VertexState {
                    shader: self.shader.clone(),
                    shader_defs: defs.clone(),
                    entry_point: Some(Cow::Borrowed("vertex_flowing")),
                    ..default()
                },
                "fragment",
            ),
            Pass::Splash => (
                "Fall splash over the water, against its depth",
                VertexState {
                    shader: self.shader.clone(),
                    shader_defs: defs.clone(),
                    entry_point: Some(Cow::Borrowed("splash_vertex")),
                    ..default()
                },
                "splash_fragment",
            ),
            Pass::Bubbles => (
                "Bubbles under a fall's foot",
                VertexState {
                    shader: self.shader.clone(),
                    shader_defs: defs.clone(),
                    entry_point: Some(Cow::Borrowed("bubble_vertex")),
                    ..default()
                },
                "bubble_fragment",
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
            Pass::RainHalf => (
                "Rain volume's shells, gathered small",
                self.fullscreen.to_vertex_state(),
                "rain_half",
            ),
            Pass::RainCompose => (
                "Rain volume laid over the scene, weighed by depth",
                self.fullscreen.to_vertex_state(),
                "rain_compose",
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
            | Pass::Splash
            | Pass::Bubbles
            | Pass::Rain
            | Pass::RainHalf
            | Pass::RainCompose
            | Pass::CloudMarch
            | Pass::CloudResolve
            | Pass::CloudPuffs
            | Pass::Clouds
            | Pass::Overlay => None,
            pass => Some(DepthStencilState {
                format: WATER_DEPTH_FORMAT,
                depth_write_enabled: matches!(pass, Pass::Cap | Pass::Flowing),
                depth_compare: if matches!(pass, Pass::Cap | Pass::Flowing) {
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
            } else if matches!(key.pass, Pass::Splash | Pass::Bubbles) {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                    self.splash_layout.clone(),
                ]
            } else if matches!(key.pass, Pass::Cap | Pass::Flowing | Pass::Compose) {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                    self.ssr_layout.clone(),
                ]
            } else if matches!(key.pass, Pass::Rain | Pass::RainHalf) {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                    self.rain_light_layout.clone(),
                ]
            } else if key.pass == Pass::RainCompose {
                vec![
                    self.data_layout.clone(),
                    scene,
                    super::weather_maps::layout(),
                    self.rain_compose_layout.clone(),
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
                targets: if key.pass == Pass::RainHalf {
                    vec![Some(ColorTargetState {
                        format: RAIN_HALF_FORMAT,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })]
                } else if key.pass == Pass::CloudResolve {
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
                    // puff's distance and the scene's stop under it; and the
                    // coverage the air leaves, "over" by each puff's own
                    // coverage like the light (`puff-haze`).
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
                        Some(ColorTargetState {
                            format: PUFF_COVER_FORMAT,
                            blend: Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                            write_mask: ColorWrites::ALL,
                        }),
                    ]
                } else if matches!(key.pass, Pass::Splash | Pass::Bubbles) {
                    // Straight alpha "over", as the splash's `AlphaMode::Blend`
                    // material blended in the main pass.
                    vec![Some(ColorTargetState {
                        format: key.format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    })]
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
    /// The game's own water, three lanes a vertex (`water-flow` E6): the
    /// buffer the data bind group holds, its version and how many vertices.
    flowing: Buffer,
    flowing_version: u64,
    flowing_count: u32,
    /// The fall splash (`water-flow` 6.1): its quads, how many, its pipeline.
    splash: Buffer,
    splash_count: u32,
    splash_pipe: CachedRenderPipelineId,
    /// The feet whose bubbles draw, and their pipeline (`water-flow` 6.6).
    bubble_feet: u32,
    bubbles_pipe: CachedRenderPipelineId,
    data_bind_group: BindGroup,
    cap: CachedRenderPipelineId,
    flowing_pipe: CachedRenderPipelineId,
    compose: CachedRenderPipelineId,
    cloud_march: CachedRenderPipelineId,
    cloud_resolve: CachedRenderPipelineId,
    cloud_puffs: CachedRenderPipelineId,
    clouds: CachedRenderPipelineId,
    lens: CachedRenderPipelineId,
    lens_mist: CachedRenderPipelineId,
    rain: CachedRenderPipelineId,
    rain_half_pipe: CachedRenderPipelineId,
    rain_compose_pipe: CachedRenderPipelineId,
    /// The rain's small target (`lighting-perf` 2.9) and its size; whether
    /// this frame's rain is gathered there (shells on, a scale under one).
    rain_half: TextureView,
    rain_half_size: UVec2,
    rain_half_on: bool,
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
    /// The drops on the lens (`weather-rain` R3).
    lens_wet: LensWet,
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
    /// The puffs' coverage as the air leaves it, the history's size: the
    /// puff pass writes it and the composite reads it in place of the
    /// history's alpha under the puffs (`puff-haze`).
    puff_cover: TextureView,
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
    rain_shadows: Option<Res<'w, RainShadows>>,
    /// The player's REFLECTIONS (`reflection-quality`).
    reflect_quality: Option<Res<'w, crate::config::ReflectQuality>>,
    /// The game's own water the sheet does not draw (`water-flow` E6).
    flowing: Option<Res<'w, crate::water_flow::FlowingWater>>,
    /// How far the drawing is through the water's last tick.
    ease: Option<Res<'w, crate::water_flow::WaterEase>>,
    /// The fall splash's quads (`water-flow` 6.1).
    splash: Option<Res<'w, crate::water_flow::SplashQuads>>,
    /// The nearest falls' feet (`water-flow` 6.6).
    fall_feet: Option<Res<'w, crate::water_flow::FallFeetGpu>>,
    /// The column tier's reach, for the far rivers' fade (design 13).
    columns: Option<Res<'w, crate::config::ColumnSettings>>,
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
        rain_shadows,
        reflect_quality,
        flowing,
        ease,
        splash,
        fall_feet,
        columns,
    } = sky;
    // The fall splash's quads, within the buffer's room.
    let splash_lanes: &[[f32; 4]] = splash.as_deref().map_or(&[], |q| {
        let room = (SPLASH_BUFFER_BYTES / 16) as usize;
        &q.lanes[..q.lanes.len().min(room)]
    });
    let splash_count =
        (splash_lanes.len().saturating_sub(2) / crate::water_flow::SPLASH_LANES) as u32;
    // The nearest feet (`water-flow` 6.6), and the bubbles under them.
    const FEET: usize = crate::water_flow::FALL_FEET_MAX;
    let fall_count = fall_feet.as_deref().map_or(0, |f| f.count.min(FEET as u32));
    let bubble_feet = if settings.falls.bubbles > 0.0 {
        fall_count
    } else {
        0
    };
    let reflect_level = reflect_quality.map_or_else(default, |q| q.0);
    let ease = ease.as_deref().copied().unwrap_or_default();
    let lit_rain = lighting.as_ref().is_some_and(|l| l.new_model_on());
    let rain_shadows = rain_shadows.is_none_or(|r| r.0);
    // `lighting-model`: under the new model the rain's rings on the water are
    // as many as it is raining (the terrain's own rule), not one a cell.
    let rings_follow_rain = lighting.as_ref().is_some_and(|l| l.new_model_on());
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
        // The straddle band about the surface (B18): the swell is in the
        // surface the CPU measures (`camera_height` is the sea's own height
        // at the eye, tide and waves), so no band is left to hold it. With
        // one, an eye a hand under the surface was "straddling": the sheet
        // over it was not drawn from below and the bank past the sheet's
        // edge was seen dry and bright ("submerged at just the right
        // height", "I dont see water surface above the player").
        let band = settings.partial_band_m.min(INLAND_BAND_M);
        // Against the sea's TIDED surface, not its waves (B18, second
        // review): with no band left to hold the swell, a crest passing a
        // hand over the mean put an eye that is above the drawn sheet "under"
        // for that frame, and the whole sky above the water went to murk.
        // The sheet drawn over a crest is the sheet's own business, seen
        // from below where it stands over the eye.
        let state = submersion(camera, sea_radius, sea_now.tide_here, band, eye.0);
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
        // What the lens holds of it, eased both ways, and the drops' own
        // clock, which stops while no rain reaches the glass (`weather-rain`
        // R3; owner: "fade out instead of abruptly cut", "gradually increase
        // as well").
        let mut lens = existing.as_ref().map_or(
            LensWet {
                wet: lens_rain,
                clock: clock.0,
                last_s: clock.0,
                last_trace: clock.0,
            },
            |gpu| gpu.lens_wet,
        );
        lens.step(
            lens_rain,
            clock.0,
            weather_settings.lens_wet_tau_s,
            weather_settings.lens_dry_tau_s,
        );
        if crate::weather::weather_scripted()
            && (clock.0 * 4.0).floor() != (lens.last_trace * 4.0).floor()
        {
            info!(
                "LENS_TRACE t {:.2} lens_rain {lens_rain:.3} lens_wet {:.3} drop_clock {:.2}",
                clock.0, lens.wet, lens.clock
            );
        }
        lens.last_trace = clock.0;
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
        let (open_water_light, sky_sh) = water_light(sun.elevation(camera), weather.cover);
        let params = WaterView {
            clip_from_local: clip_from_body,
            local_from_clip: clip_from_body.as_dmat4().inverse().as_mat4(),
            prev_clip_from_local: existing
                .as_ref()
                .map_or(clip_from_body, |gpu| gpu.prev_clip),
            camera_time: camera.extend(clock.0 * s.time_scale),
            planet_center: Vec3::ZERO.extend(sea_radius),
            sun: sun.direction().extend(s.specular_intensity * sun_dim),
            // xyz the downwelling (`ocean-environments` B2).
            waves: v3(s.downwell_per_m).extend(s.wave_steepness),
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
            // w the turbidity at the eye, eased (B3).
            absorption: v3(s.absorption_per_m).extend(sea_now.turbidity.max(0.0)),
            deep_color: v3(s.deep_color).extend(s.flow_uv_speed_falling),
            shallow: v3(s.shallow_color).extend(s.sky_horizon_strength),
            water_light: open_water_light.extend(s.shallow_depth_m),
            haze: Vec4::new(
                weather_settings.cloud_fog_add,
                weather_settings.rain_fog_mult,
                weather_settings.overcast_sky_blue_cut,
                weather_settings.overcast_sky_dim,
            ),
            foam_color: v3(s.foam_color).extend(s.foam_intensity),
            // w: rain's fine detail's reach and fade (`wet-near-detail`), which
            // the water's rings honour as the ground's do (`water-flow` "Rain
            // rings that alias").
            foam_crest: Vec4::new(
                s.foam_crest_lo,
                s.foam_crest_hi,
                s.foam_crest_weight,
                weather_settings.rain_detail_reach_m,
            ),
            foam_slope: Vec4::new(
                s.foam_slope_lo,
                s.foam_slope_hi,
                s.foam_slope_weight,
                weather_settings.rain_detail_fade_m,
            ),
            sun_tint: v3(s.sun_tint).extend(s.specular_power),
            fog_night: FOG_NIGHT_SKY.extend(FOG_DENSITY_PER_M),
            fog_day: FOG_DAY_SKY.extend(FOG_HEIGHT_M),
            limits: Vec4::new(s.fog_max, TERMINATOR.0, TERMINATOR.1, FOG_MIX),
            fx: Vec4::new(s.underwater_distortion, state, sea_rain, drips),
            lens: Vec4::new(
                weather_settings.rain_lens_density,
                weather_settings.rain_lens_bend,
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
            // The landing's dissolve, the terrain's own lanes: a sheet listed
            // for the old partition is split by it and keeps the old half of
            // the pixels, as the seabed under it does (`two-set-dissolve`).
            lod_prev: {
                let terrain = planet_view.uniform.get();
                terrain.lod_prev.truncate().extend(terrain.fade.y)
            },
            bands_prev: planet_view.uniform.get().bands_prev,
            bands_prev_in: planet_view.uniform.get().bands_prev_in,
            rain: Vec4::new(
                lens.wet,
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
            // y the surface of the water the eye is in or over (B17, B18);
            // z and w the water's easing
            // between ticks, smoothstepped and linear (`water_flow::WaterEase`).
            sea_frame: Vec4::new(
                s.depth_offset_m,
                // The sea's own surface over the eye is its tided, swelling
                // one: the composite measured the sea from its mean, so the
                // tide's water between was dry (a bright band at high water).
                // An eye in the air is over the same tided sea (`fishing-and-equipment`
                // design 4, `ocean-environments` B21): one helper says it.
                composite_surface_radius(eye.0, camera, sea_radius, sea_now.tide_here),
                ease.smooth,
                ease.linear,
            ),
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
            lens_look: Vec4::new(
                weather_settings.rain_lens_rim,
                weather_settings.rain_lens_glint,
                lens.clock,
                0.0,
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
                rain_kinds(rain_map.as_deref()),
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
            ssr_clip: planet_view.ssr_clip,
            // The rain fog is rain's: none in snow.
            haze_weather: Vec4::new(weather.cover, weather.liquid(), 0.0, 0.0),
            sky_sh,
            under: Vec4::new(s.shafts, s.shaft_reach_m, s.snow, s.snow_from_m),
            under_more: Vec4::new(s.snow_glow, sea_now.wind_mps, s.spray_from_mps, s.spray),
            column: fine
                .as_deref()
                .filter(|f| !f.set.columns.columns.is_empty())
                .map_or(Vec4::ZERO, |f| {
                    f.set
                        .anchor
                        .extend(columns.as_deref().map_or(0.0, |c| c.reach_m))
                }),
            fall_look: Vec4::new(
                fall_count as f32,
                s.falls.teal_churn,
                s.falls.ripple_height,
                s.falls.ripple_length_m,
            ),
            fall_flow: Vec4::new(
                s.falls.ripple_reach_m,
                s.falls.pool_flow,
                s.falls.pool_flow_reach_m,
                3.0 * s.falls.fall_speed_up,
            ),
            fall_lip: Vec4::new(
                f32::from(u8::from(s.falls.glassy)),
                s.falls.glass_top_m,
                s.falls.glass_face_m,
                s.falls.froth_fill_m,
            ),
            fall_more: Vec4::new(
                s.falls.glass_tint,
                s.falls.edge_highlight,
                s.falls.fall_speed_max_mps,
                s.falls.bubbles,
            ),
            fall_splash: Vec4::new(
                s.falls.wall_blend_m,
                s.falls.mist_opacity,
                f32::from(u8::from(s.falls.smooth)),
                view.viewport.w.max(1) as f32,
            ),
            aer_color: Vec3::from_array(s.falls.aer_color).extend(s.time_scale.max(1e-3)),
            feet: fall_feet.as_deref().map_or([Vec4::ZERO; FEET], |f| f.feet),
            feet_more: fall_feet.as_deref().map_or([Vec4::ZERO; FEET], |f| f.more),
            height_fog: height_fog_lane(
                !rain_shadows
                    && state <= 0.75
                    && camera.length() < clouds.clouds.x
                    && rain_map.as_deref().is_some_and(|m| !m.values.is_empty()),
                weather.rain,
                &weather_settings,
                camera,
            ),
            ssr_eye: if lighting.as_ref().is_some_and(|l| {
                l.reflect_sea_on(reflect_level) && l.reflect_capture_on(reflect_level)
            }) {
                planet_view.ssr_eye
            } else {
                Vec4::ZERO
            },
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
                let view = pbd_core::frustum::Planes::from_clip(clip_from_body);
                let started = std::time::Instant::now();
                let counts = sites.build(
                    maps.generation,
                    &maps.from,
                    &maps.maps,
                    maps.t,
                    camera,
                    Some(&view),
                    &weather_settings,
                    instances,
                );
                // The cull's measure (`puff-frustum-cull`): one line in 120
                // frames, which `tools/frame_graph.py` lines up with the log.
                static BUILDS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                if BUILDS
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    .is_multiple_of(120)
                {
                    info!(
                        "PUFFS built {}, over the horizon {}, in view {}, build {:.2} ms",
                        counts.built,
                        counts.over_horizon,
                        counts.in_view,
                        started.elapsed().as_secs_f64() * 1e3
                    );
                }
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
        let lens_needed = lens.wet > 0.001 || drips > 0.001 || mist_active;
        let map = rain_map.as_deref().filter(|map| {
            !map.values.is_empty() && map.values.iter().any(|value| value.abs() > 1e-3)
        });
        // RAIN SHADOW off (the player's setting): no rain volume at all;
        // the near shower, the wet ground and the lens stay. Nor under the
        // sea (`rain-under-water`): the rain is in the air, and what the eye
        // sees of the air from under the surface is the sheet's.
        let rain_needed = map.is_some()
            && rain_shadows
            && state <= 0.75
            && camera.length() < clouds.clouds.x
            && weather_settings.rain_volume_range_m > 0.0;
        let size = UVec2::new(view.viewport.z.max(1), view.viewport.w.max(1));
        let history_size = (size.as_vec2() * weather_settings.cloud_render_scale)
            .ceil()
            .as_uvec2()
            .max(UVec2::ONE);
        let rain_half_size = (size.as_vec2() * weather_settings.rain_render_scale)
            .ceil()
            .as_uvec2()
            .max(UVec2::ONE);
        let rain_half_on = weather_settings.rain_render_scale < 1.0;
        if let Some(mut gpu) = existing {
            // The game's own water, when the mesher has made it again.
            if let Some(flowing) = flowing.as_deref()
                && flowing.version != gpu.flowing_version
            {
                let bytes: &[u8] = bytemuck::cast_slice(flowing.lanes.as_slice());
                if gpu.flowing.size() < bytes.len() as u64 {
                    gpu.flowing = flowing_buffer(&device, flowing.lanes.len());
                    gpu.data_bind_group = device.create_bind_group(
                        Some("water data"),
                        &cache.get_bind_group_layout(&pipelines.data_layout),
                        &BindGroupEntries::sequential((
                            &gpu.uniform,
                            planet.cells.as_entire_binding(),
                            planet_view.water.as_entire_binding(),
                            gpu.flowing.as_entire_binding(),
                            gpu.rain_buffer.as_entire_binding(),
                        )),
                    );
                }
                if !bytes.is_empty() {
                    queue.write_buffer(&gpu.flowing, 0, bytes);
                }
                gpu.flowing_count =
                    (flowing.lanes.len() / crate::water_flow::LANES_PER_VERTEX) as u32;
                gpu.flowing_version = flowing.version;
            }
            let resized = gpu.history_size != history_size;
            if resized {
                gpu.history = cloud_history(&device, history_size);
                gpu.cloud_depth = cloud_distances(&device, history_size);
                gpu.current = cloud_current(&device, history_size);
                gpu.puff_cover = cloud_target(&device, history_size, PUFF_COVER_FORMAT);
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
            gpu.splash_count = splash_count;
            gpu.bubble_feet = bubble_feet;
            if splash_count > 0 {
                queue.write_buffer(&gpu.splash, 0, bytemuck::cast_slice(splash_lanes));
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
            gpu.rain_half_on = rain_half_on;
            if gpu.rain_half_size != rain_half_size {
                gpu.rain_half = cloud_target(&device, rain_half_size, RAIN_HALF_FORMAT);
                gpu.rain_half_size = rain_half_size;
            }
            gpu.overlay_needed = overlay_needed;
            if let Some(map) = map {
                queue.write_buffer(&gpu.rain_buffer, 0, bytemuck::cast_slice(&map.values));
            }
            gpu.was_under = was_under;
            gpu.emerge_until = emerge_until;
            gpu.lens_wet = lens;
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
        // The game's own water (`water-flow` E6), as the mesher last made it.
        let (lanes, flowing_version) = flowing
            .as_deref()
            .map_or((Vec::new(), 0), |f| ((*f.lanes).clone(), f.version));
        let flow = flowing_buffer(&device, lanes.len());
        if !lanes.is_empty() {
            queue.write_buffer(&flow, 0, bytemuck::cast_slice(lanes.as_slice()));
        }
        let flowing_count = (lanes.len() / crate::water_flow::LANES_PER_VERTEX) as u32;
        let splash_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("fall splash quads"),
            size: SPLASH_BUFFER_BYTES,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        if splash_count > 0 {
            queue.write_buffer(&splash_buffer, 0, bytemuck::cast_slice(splash_lanes));
        }
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
            flowing_pipe: pipeline(Pass::Flowing),
            flowing: flow,
            flowing_version,
            flowing_count,
            splash: splash_buffer,
            splash_count,
            splash_pipe: pipeline(Pass::Splash),
            bubble_feet,
            bubbles_pipe: pipeline(Pass::Bubbles),
            compose: pipeline(Pass::Compose),
            cloud_march: pipeline(Pass::CloudMarch),
            cloud_resolve: pipeline(Pass::CloudResolve),
            cloud_puffs: pipeline(Pass::CloudPuffs),
            clouds: pipeline(Pass::Clouds),
            lens: pipeline(Pass::Lens),
            lens_mist: pipeline(Pass::LensMist),
            rain: pipeline(Pass::Rain),
            rain_half_pipe: pipeline(Pass::RainHalf),
            rain_compose_pipe: pipeline(Pass::RainCompose),
            rain_half: cloud_target(&device, rain_half_size, RAIN_HALF_FORMAT),
            rain_half_size,
            rain_half_on,
            overlay: pipeline(Pass::Overlay),
            overlay_needed,
            rain_buffer,
            rain_needed,
            depth: water_depth(&device, size),
            depth_size: size,
            uniform,
            data_bind_group,
            multisampled,
            lens_needed,
            was_under,
            emerge_until,
            lens_wet: lens,
            history: cloud_history(&device, history_size),
            history_read: 0,
            history_size,
            cloud_depth: cloud_distances(&device, history_size),
            current: cloud_current(&device, history_size),
            puff_cover: cloud_target(&device, history_size, PUFF_COVER_FORMAT),
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

/// The game's own water's buffer, room for `lanes` vec4s (one at least).
fn flowing_buffer(device: &RenderDevice, lanes: usize) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label: Some("flowing water vertices"),
        size: (lanes.max(crate::water_flow::LANES_PER_VERTEX) as u64 * 16).next_power_of_two(),
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
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
            // The splash reads it (`water-flow` 6.6).
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
        .create_view(&TextureViewDescriptor::default())
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub(super) struct WaterCompositeLabel;

/// Which kinds of fall the rain map holds anywhere, as the shells read them
/// (`lighting-perf` 2.10): bit 0 rain (a value over the shader's 1e-3), bit 1
/// snow (under -1e-3). The shader's sample is a bilinear mix of the map's
/// values, so it can only cross a threshold some value crosses; the margin
/// keeps a kind on for a value at the threshold itself.
fn rain_kinds(map: Option<&crate::weather::RainMap>) -> f32 {
    let Some(map) = map else {
        return 0.0;
    };
    let rain = map.values.iter().any(|v| *v > 0.5e-3);
    let snow = map.values.iter().any(|v| *v < -0.5e-3);
    f32::from(u8::from(rain) | u8::from(snow) << 1)
}

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

/// The most bubbles under one foot (`water-flow` 6.6): `BUBBLES_MAX` in
/// `water.wgsl`; a test holds the two together.
const BUBBLES_MAX: u32 = 256;

/// `PBD_PASS_OFF=splash`: the fall splash undrawn, to see the water under
/// it (`water-flow` 6.1).
static SPLASH_OFF: std::sync::LazyLock<bool> = std::sync::LazyLock::new(|| pass_off("splash"));

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
        // The screen trace's history (`lighting-model` L8), for the cap.
        let Some(planet_gpu) = world.get_resource::<PlanetGpu>() else {
            return Ok(());
        };
        let ssr_group = device.create_bind_group(
            Some("water: the screen trace's history"),
            &cache.get_bind_group_layout(&pipelines.ssr_layout),
            &BindGroupEntries::with_indices(((14, &planet_gpu.ssr_view),)),
        );
        // The fall splash's quads (`water-flow` 6.1), drawn after the water,
        // and the bubbles (6.6): in their own pass, reading the water's depth.
        let splash_on = water.splash_count > 0 && !*SPLASH_OFF;
        let bubbles_on = water.bubble_feet > 0 && !*SPLASH_OFF;
        let splash_group = device.create_bind_group(
            Some("water: the fall splash"),
            &cache.get_bind_group_layout(&pipelines.splash_layout),
            &BindGroupEntries::with_indices((
                (17, water.splash.as_entire_binding()),
                (18, &pipelines.mist_puffs),
                (19, &pipelines.sampler),
                (20, &water.depth),
            )),
        );
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
                        // Kept for the splash's pass when it draws, and for
                        // the clouds' composite and the rain, which tell the
                        // sea's horizon gap from the sheet by it
                        // (`horizon-haze` 4).
                        store: if splash_on || bubbles_on || clouds_on || water.rain_needed {
                            StoreOp::Store
                        } else {
                            StoreOp::Discard
                        },
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &water.data_bind_group, &[]);
            pass.set_bind_group(1, &scene, &[]);
            pass.set_bind_group(2, &maps.bind_group, &[]);
            pass.set_bind_group(3, &ssr_group, &[]);
            let span = diagnostics.pass_span(&mut pass, "water_compose");
            pass.set_render_pipeline(compose);
            pass.draw(0..3, 0..1);
            pass.set_render_pipeline(cap);
            pass.draw_indirect(&planet_view.indirect, 32);
            // The game's own water, in the same pass and depth, so the sheet
            // and a poured pool sort as one water.
            if water.flowing_count > 0
                && let Some(flowing) = cache.get_render_pipeline(water.flowing_pipe)
            {
                pass.set_render_pipeline(flowing);
                pass.draw(0..water.flowing_count, 0..1);
            }
            span.end(&mut pass);
            drop(pass);
            // The fall splash last (`water-flow` 6.1, 6.6), in its own pass:
            // over the water it stands in front of and behind the terrain,
            // both tested in `splash_fragment` against the depths it reads,
            // and faded softly against the nearer. The bubbles first, under
            // the pool's surface.
            if splash_on || bubbles_on {
                let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                    label: Some("Fall splash and bubbles"),
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
                pass.set_bind_group(3, &splash_group, &[]);
                let span = diagnostics.pass_span(&mut pass, "water_splash");
                if bubbles_on && let Some(bubbles) = cache.get_render_pipeline(water.bubbles_pipe) {
                    pass.set_render_pipeline(bubbles);
                    pass.draw(0..water.bubble_feet * BUBBLES_MAX * 6, 0..1);
                }
                if splash_on && let Some(splash) = cache.get_render_pipeline(water.splash_pipe) {
                    pass.set_render_pipeline(splash);
                    pass.draw(0..water.splash_count * 6, 0..1);
                }
                span.end(&mut pass);
            }
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
                (16, &water.puff_cover),
                (20, &water.depth),
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
                    color_attachments: &[
                        (history_write, clears[0]),
                        (depth_write, clears[1]),
                        (&water.puff_cover, clears[0]),
                    ]
                    .map(|(view, clear)| {
                        Some(RenderPassColorAttachment {
                            view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(clear.into()),
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
                            (20, &water.depth),
                        )),
                    )
                })
        };
        // The rain's small target, when it is used and both its pipelines are
        // ready; otherwise the full-size pass draws the rain as before.
        let rain_half = (water.rain_half_on && water.rain_needed)
            .then(|| {
                Some((
                    cache.get_render_pipeline(water.rain_half_pipe)?,
                    cache.get_render_pipeline(water.rain_compose_pipe)?,
                ))
            })
            .flatten();
        let rain_compose_group = device.create_bind_group(
            Some("rain: the small target"),
            &cache.get_bind_group_layout(&pipelines.rain_compose_layout),
            &BindGroupEntries::with_indices(((15, &water.rain_half), (20, &water.depth))),
        );
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
            let is_rain = label == "Rain volume";
            // Gathered small first (`lighting-perf` 2.9), into its own target,
            // reading the scene's depth; the compose below lays it over.
            let mut pipeline = pipeline;
            if is_rain && let (Some((half, compose)), Some(group)) = (rain_half, &rain_light_group)
            {
                let source = target.main_texture_view();
                let scene = scene_bind_group(source);
                let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                    label: Some("Rain volume, gathered small"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &water.rain_half,
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
                pass.set_bind_group(3, group, &[]);
                let span = diagnostics.pass_span(&mut pass, "rain_half");
                pass.set_render_pipeline(half);
                pass.draw(0..3, 0..1);
                span.end(&mut pass);
                drop(pass);
                pipeline = compose;
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
            } else if is_rain && rain_half.is_some() && rain_light_group.is_some() {
                pass.set_bind_group(3, &rain_compose_group, &[]);
            } else if is_rain && let Some(group) = &rain_light_group {
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

    /// The river the owner's picture stands in, 60 m from the version 10
    /// spawn (`docs/screenshots/river-fixes/`): a direction where the
    /// generator's river stands two layers over a meadow's bed.
    fn river_at_the_spawn() -> Vec3 {
        let d = pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: 28.25_f32.to_radians(),
            lon: 0.75_f32.to_radians(),
        });
        assert!(
            pbd_core::rivers::floods(terrain::terrain_config(), d),
            "the river is still there"
        );
        d
    }

    /// `ocean-environments` B17: an eye in a river's water, over the sea's
    /// level, reads as under water against the RIVER's surface: the column
    /// at the eye says water, the tri-state says fully under whatever the
    /// sea's swell is doing, and the view uniform carries the river's
    /// surface for the composite to measure from. Out of the water, dry.
    #[test]
    fn an_eye_in_a_river_reads_as_under_the_river() {
        use std::sync::Arc;
        let river = river_at_the_spawn();
        let set = Arc::new(super::super::lod::generate_fine(
            river,
            &crate::config::ColumnSettings::default(),
            &pbd_core::edits::Edits::new(),
        ));
        let mut contact = crate::planet::PlanetContact::test_planet(5);
        contact.set_fine(&set);
        let column = contact
            .finest_cell(river)
            .and_then(|record| set.columns.column(record))
            .expect("the river's column is on the tier");
        let bed = column.surface().expect("a bed");
        let bed_m = pbd_core::column::layer_altitude(bed) + 1.0;
        let surface_m = column
            .water_surface(bed_m + 0.5)
            .expect("water stands on the river's bed");
        assert!(surface_m > terrain::terrain_config().sea_level_m + 1.0);
        // The surface the eye is under is the DRAWN top (Luanti's corners),
        // at or over the cell's own level and inside its layer.
        let record = contact.finest_cell(river).unwrap();
        let top_layer = pbd_core::column::layer_at(surface_m - 1e-3).unwrap();
        let surface_m = set
            .columns
            .slot(record)
            .and_then(|slot| crate::water_flow::drawn_top_m(&set.columns, slot, top_layer))
            .unwrap_or(surface_m);
        assert!(surface_m <= pbd_core::column::layer_altitude(top_layer) + 1.0);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin));
        install_eye_water(&mut app);
        let offset = bevy::math::DVec3::new(8192.0, -4096.0, 2048.0);
        app.insert_resource(PlanetRenderFrame { center: offset })
            .insert_resource(contact)
            .insert_resource(crate::planet::PlanetFine { set, version: 1 });
        let at = |m: f32| (offset + (river * (terrain::PLANET_RADIUS + m)).as_dvec3()).as_vec3();
        let camera = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Camera::default(),
                Transform::from_translation(at(surface_m - 0.8)),
            ))
            .id();
        app.update();
        let eye = app.world().resource::<EyeWaterState>().0;
        let surface = terrain::PLANET_RADIUS + surface_m;
        assert_eq!(
            eye,
            EyeWater::Water {
                surface_radius: surface
            }
        );
        assert_eq!(
            inland_surface(eye),
            Some(surface),
            "the river's, not the sea's"
        );
        let sea = terrain::PLANET_RADIUS - WaterSettings::default().depth_offset_m;
        let body = river * (surface - 0.8);
        for swell in [-1.5, 0.0, 1.5] {
            assert_eq!(submersion(body, sea, swell, 0.3, eye), 1.0, "swell {swell}");
        }
        // Out over the river: dry.
        app.world_mut()
            .get_mut::<Transform>(camera)
            .unwrap()
            .translation = at(surface_m + 1.5);
        app.update();
        let eye = app.world().resource::<EyeWaterState>().0;
        assert_eq!(eye, EyeWater::Air);
        assert_eq!(inland_surface(eye), None);
        // The sea's own water keeps the sea's sphere.
        let at_sea = EyeWater::Water {
            surface_radius: terrain::PLANET_RADIUS + terrain::terrain_config().sea_level_m,
        };
        assert_eq!(inland_surface(at_sea), None);
    }

    /// `ocean-environments` B17: a walker in the same river swims in it to
    /// its own surface (the walker's rule, `walking::swim_surface`, over the
    /// column's answer), and with its feet on the bank over the water is out.
    #[test]
    fn a_walker_swims_in_a_river_and_climbs_out() {
        let river = river_at_the_spawn();
        let set = super::super::lod::generate_fine(
            river,
            &crate::config::ColumnSettings::default(),
            &pbd_core::edits::Edits::new(),
        );
        let mut contact = crate::planet::PlanetContact::test_planet(5);
        contact.set_fine(&std::sync::Arc::new(set));
        let r = terrain::PLANET_RADIUS;
        let bed = r + terrain::surface_height(river);
        let water = contact
            .column_water(river * (bed + 0.5))
            .flatten()
            .map(|top| r + top)
            .expect("the river's water on its bed");
        let sea = r + terrain::terrain_config().sea_level_m;
        let sheet = r - WaterSettings::default().depth_offset_m;
        let surface = crate::walking::swim_surface(true, Some(water), sheet, sea, bed, true);
        assert_eq!(
            surface,
            Some(water),
            "swum to the river's surface, not the sea's"
        );
        // Feet a metre over the river's surface, on its bank: dry.
        let over = contact.column_water(river * (water + 1.0)).flatten();
        assert_eq!(over, None, "no water over the river's surface");
        assert_eq!(
            crate::walking::swim_surface(true, over, sheet, sea, water + 1.0, true),
            None
        );
    }

    #[test]
    fn the_light_on_open_water_falls_with_the_sun_and_under_cloud() {
        let luma = |v: Vec3| v.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let (noon, _) = water_light(0.9, 0.0);
        let (overcast, _) = water_light(0.9, 1.0);
        let (dusk, _) = water_light(0.02, 0.0);
        let (night, _) = water_light(-0.5, 0.0);
        assert!(luma(overcast) < 0.5 * luma(noon), "{overcast} {noon}");
        assert!(luma(dusk) < 0.5 * luma(noon), "{dusk} {noon}");
        assert!(luma(night) < 0.05 * luma(noon), "{night} {noon}");
        assert!(luma(night) > 0.0, "a night is never black");
    }

    #[test]
    fn the_uniform_matches_the_wgsl_struct_size() {
        // Four mat4 and the vec4 lanes in water.wgsl's WaterView, read off the
        // shipped shader rather than remembered.
        let shader = include_str!("../../../assets/shaders/water.wgsl");
        let start = shader.find("struct WaterView {").unwrap();
        let block = &shader[start..start + shader[start..].find('}').unwrap()];
        let mat4 = block.matches("mat4x4<f32>").count();
        // The lit rain's lamps are one array of eight lanes (`lighting-model`),
        // and a fall's feet two (`water-flow` 6.6).
        let eights = block.matches("array<vec4<f32>, 8>").count();
        // And the sky's nine coefficients (`water-lit`).
        let sh = block.matches("array<vec4<f32>, 9>").count();
        let vec4 = block.matches("vec4<f32>").count() + 7 * eights + 8 * sh;
        assert_eq!((mat4, vec4, eights, sh), (4, 100, 3, 1));
        assert_eq!(crate::water_flow::FALL_FEET_MAX, 8);
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

    /// The splash's lanes a quad and the bubbles' room are one number each in
    /// Rust and in the shader (`water-flow` 6.6).
    #[test]
    fn the_splash_s_stride_and_the_bubbles_agree_with_the_shader() {
        let shader = include_str!("../../../assets/shaders/water.wgsl");
        let lanes = format!(
            "const SPLASH_LANES: u32 = {}u;",
            crate::water_flow::SPLASH_LANES
        );
        assert!(shader.contains(&lanes), "{lanes}");
        let bubbles = format!("const BUBBLES_MAX: u32 = {BUBBLES_MAX}u;");
        assert!(shader.contains(&bubbles), "{bubbles}");
        let feet = format!(
            "const FALL_FEET: u32 = {}u;",
            crate::water_flow::FALL_FEET_MAX
        );
        assert!(shader.contains(&feet), "{feet}");
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

    /// `ocean-environments` B21 (owner, 2026-11-16: "dry ground had the weird
    /// water blur pass ... only happens when you jump above water"). On a
    /// beach at low tide the eye standing within the high tide's reach is
    /// over the sea's water, and the same eye a jump higher is in air; the
    /// composite measures the waterline from the sea's tided surface for
    /// both, never the mean sphere, which lies over the dry sand at low tide.
    #[test]
    fn a_jump_over_a_beach_keeps_the_composites_waterline_at_the_tide() {
        let sea = terrain::PLANET_RADIUS - 0.5;
        let low_tide = -1.2;
        let up = Vec3::Y;
        let sea_level = terrain::PLANET_RADIUS + terrain::terrain_config().sea_level_m;
        let standing = EyeWater::Water {
            surface_radius: sea_level,
        };
        let stand = composite_surface_radius(standing, up * (sea + 1.0), sea, low_tide);
        let jump = composite_surface_radius(EyeWater::Air, up * (sea + 2.6), sea, low_tide);
        assert_eq!(stand, sea + low_tide);
        assert_eq!(jump, stand, "the jump moved the composite's waterline");
        // Both dry: no tint, wobble or murk is armed for the eye itself.
        assert_eq!(
            submersion(up * (sea + 1.0), sea, low_tide, 0.0, standing),
            0.0
        );
        assert_eq!(
            submersion(up * (sea + 2.6), sea, low_tide, 0.0, EyeWater::Air),
            0.0
        );
        // An inland water's own top is still the eye's.
        let river = EyeWater::Water {
            surface_radius: sea + 30.0,
        };
        assert_eq!(
            composite_surface_radius(river, up * (sea + 29.0), sea, low_tide),
            sea + 30.0
        );
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

#[cfg(test)]
mod lens_wet_tests {
    use super::LensWet;

    /// `weather-rain` R3: the lens wets and dries on its own time constants,
    /// and the drops' clock stops while no rain reaches the glass.
    #[test]
    fn the_lens_wets_and_dries_over_time_and_grows_no_drop_while_dry() {
        let (wet_tau, dry_tau) = (3.0, 2.5);
        let mut lens = LensWet {
            wet: 1.0,
            clock: 0.0,
            last_s: 0.0,
            last_trace: 0.0,
        };
        // Under cover: one e-fold of drying in dry_tau, and the clock still.
        let mut t = 0.0;
        while t < dry_tau - 1e-4 {
            t += 1.0 / 60.0;
            lens.step(0.0, t, wet_tau, dry_tau);
        }
        assert!(
            (lens.wet - (-1.0f32).exp()).abs() < 0.02,
            "wet {}",
            lens.wet
        );
        assert!(lens.clock < 0.02, "the drops kept running: {}", lens.clock);
        // No step on any frame either way.
        let before = lens.wet;
        lens.step(1.0, t + 1.0 / 60.0, wet_tau, dry_tau);
        assert!((lens.wet - before).abs() < 0.02);
        // Back in the rain: up on wet_tau, the clock running.
        let start = lens.clock;
        let mut u = t + 1.0 / 60.0;
        while u < t + 3.0 * wet_tau {
            u += 1.0 / 60.0;
            lens.step(1.0, u, wet_tau, dry_tau);
        }
        assert!(lens.wet > 0.94);
        assert!(lens.clock - start > 8.0);
    }
}

/// `ocean-environments` B18 (owner, 2026-11-16: "there's two rivers: above
/// sea level river and sea level river, both needs to merge", "we should also
/// validate wherever they merge"): where two waters meet, sampled off the
/// generator's own rivers on a real column tier. Along a line across each
/// junction, every finest cell is read as the eye and the drawing read it:
/// the sea's water (drawn by the sheet), inland water (its drawn top), or dry.
/// What must hold everywhere: no cell draws two surfaces (the sea's sheet
/// under an inland water's top), and the eye's decision follows the local
/// water (the sea's sphere exactly where the water is the sea's, the drawn
/// top elsewhere), so it never flips where the water does not. The steps and
/// gaps along each line are printed for the report.
#[cfg(test)]
mod water_junction_tests {
    use super::*;
    use crate::planet::column::sea_layer;
    use pbd_core::rivers;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Here {
        Dry,
        Sea(f32),
        Inland(f32),
    }

    struct Line {
        samples: Vec<Here>,
        overlaps: usize,
        flips: usize,
    }

    fn read_line(points: &[Vec3]) -> Line {
        let anchor = points[points.len() / 2];
        let set = std::sync::Arc::new(super::super::lod::generate_fine(
            anchor,
            &crate::config::ColumnSettings::default(),
            &pbd_core::edits::Edits::new(),
        ));
        let mut contact = crate::planet::PlanetContact::test_planet(5);
        contact.set_fine(&set);
        let sheet = terrain::terrain_config().sea_level_m - WaterSettings::default().depth_offset_m;
        let sea = sea_layer();
        let (mut samples, mut overlaps, mut flips) = (Vec::new(), 0, 0);
        for &p in points {
            let Some(record) = contact.finest_cell(p) else {
                continue;
            };
            let (Some(column), Some(slot)) = (set.columns.column(record), set.columns.slot(record))
            else {
                continue;
            };
            let ground = column.surface().unwrap_or(0);
            let top = (ground + 1..pbd_core::column::LAYERS)
                .take_while(|&l| column.material(l).is_water())
                .last();
            let here = match top {
                None => Here::Dry,
                Some(t)
                    if t + 1 == sea && column.material(t) == pbd_core::terrain::Material::Water =>
                {
                    Here::Sea(sheet)
                }
                Some(t) => Here::Inland(
                    crate::water_flow::drawn_top_m(&set.columns, slot, t).unwrap_or(
                        pbd_core::column::layer_altitude(t)
                            + pbd_core::liquid::top_fraction(column.material(t)),
                    ),
                ),
            };
            // Two surfaces in one cell: the sea's sheet drawn under an inland
            // top (the record under sea level, its sheet not marked dry).
            let cell = &set.levels[3][record];
            let sheet_drawn = crate::planet::column::sheet_listed(cell);
            if sheet_drawn && matches!(here, Here::Inland(_)) {
                overlaps += 1;
            }
            // The eye 0.3 m under the local water: the decision follows it.
            if let Here::Inland(s) | Here::Sea(s) = here {
                let water_top = match here {
                    Here::Sea(_) => terrain::terrain_config().sea_level_m,
                    _ => s,
                };
                let eye_alt = water_top - 0.3;
                let eye = match column.water_surface(eye_alt) {
                    Some(surface) => {
                        let t = pbd_core::column::layer_at(surface - 1e-3);
                        let drawn =
                            t.and_then(|t| crate::water_flow::drawn_top_m(&set.columns, slot, t));
                        EyeWater::Water {
                            surface_radius: terrain::PLANET_RADIUS + drawn.unwrap_or(surface),
                        }
                    }
                    None => EyeWater::Air,
                };
                let inland = inland_surface(eye).is_some();
                if inland != matches!(here, Here::Inland(_)) {
                    flips += 1;
                }
            }
            samples.push(here);
        }
        Line {
            samples,
            overlaps,
            flips,
        }
    }

    /// The largest step between neighbouring wet samples, and the dry
    /// samples between the first and the last wet one (a gap in the water).
    fn steps_and_gaps(samples: &[Here]) -> (f32, usize) {
        let level = |h: &Here| match h {
            Here::Sea(s) | Here::Inland(s) => Some(*s),
            Here::Dry => None,
        };
        let wet: Vec<usize> = (0..samples.len())
            .filter(|&i| level(&samples[i]).is_some())
            .collect();
        let (Some(&first), Some(&last)) = (wet.first(), wet.last()) else {
            return (0.0, 0);
        };
        let gaps = (first..=last)
            .filter(|&i| level(&samples[i]).is_none())
            .count();
        let step = samples
            .windows(2)
            .filter_map(|w| Some((level(&w[0])? - level(&w[1])?).abs()))
            .fold(0.0, f32::max);
        (step, gaps)
    }

    fn line(a: Vec3, b: Vec3, beyond_m: f32) -> Vec<Vec3> {
        let r = terrain::PLANET_RADIUS;
        let dir = (b - a).normalize_or(Vec3::X);
        let end = (b + dir * (beyond_m / r)).normalize();
        let len_m = a.angle_between(end) * r;
        let n = (len_m / 1.0).ceil().max(2.0) as usize;
        (0..=n)
            .map(|i| a.lerp(end, i as f32 / n as f32).normalize())
            .collect()
    }

    #[test]
    fn where_waters_meet_the_surfaces_join_and_the_eye_follows_them() {
        let cfg = terrain::terrain_config();
        let Some(settings) = cfg.rivers else {
            return;
        };
        let map = rivers::map(cfg, &settings);
        let nodes: Vec<usize> = map.rivers().collect();
        let mut upstream: std::collections::HashMap<usize, Vec<usize>> = Default::default();
        for &k in &nodes {
            if let Some(d) = map.down(k) {
                upstream.entry(d).or_default().push(k);
            }
        }
        let centre = |k: usize| map.describe(k).0;
        let spawn = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let mut by_distance = nodes.clone();
        by_distance.sort_by(|&a, &b| {
            centre(a)
                .angle_between(spawn)
                .total_cmp(&centre(b).angle_between(spawn))
                .then(a.cmp(&b))
        });
        // Mouths: the river's last cell, draining into a cell of the sea:
        // the line runs from it down into the sea.
        let is_river: std::collections::HashSet<usize> = nodes.iter().copied().collect();
        let mouths: Vec<(usize, usize)> = by_distance
            .iter()
            .filter_map(|&k| Some((k, map.down(k)?)))
            .filter(|&(_, d)| !is_river.contains(&d))
            .take(5)
            .collect();
        // Tributaries: a cell two rivers drain into.
        let tributaries: Vec<(usize, usize)> = by_distance
            .iter()
            .filter(|&&k| upstream.get(&k).is_some_and(|u| u.len() >= 2) && map.down(k).is_some())
            .map(|&k| (upstream[&k][1], k))
            .take(3)
            .collect();
        // Falls of three layers or more: into a river's pool, into the sea.
        let falls: Vec<(usize, usize)> = by_distance
            .iter()
            .filter(|&&k| map.describe(k).2 >= 3)
            .filter_map(|&k| Some((*upstream.get(&k)?.first()?, k)))
            .take(3)
            .collect();
        let falls_to_sea = falls
            .iter()
            .filter(|&&(_, k)| map.down(k).is_some_and(|d| !is_river.contains(&d)))
            .count();
        let mut report = Vec::new();
        let (mut overlaps, mut flips) = (0, 0);
        for (kind, list) in [
            ("mouth", &mouths),
            ("tributary", &tributaries),
            ("fall", &falls),
        ] {
            for &(up, k) in list.iter() {
                let l = read_line(&line(centre(up), centre(k), 40.0));
                let (step, gaps) = steps_and_gaps(&l.samples);
                let sea = l
                    .samples
                    .iter()
                    .filter(|h| matches!(h, Here::Sea(_)))
                    .count();
                let inland = l
                    .samples
                    .iter()
                    .filter(|h| matches!(h, Here::Inland(_)))
                    .count();
                let ll = pbd_core::geo::lat_lon(centre(k)).degrees();
                let lu = pbd_core::geo::lat_lon(centre(up)).degrees();
                report.push(format!(
                    "{kind} from {:.4} {:.4} at {:.4} {:.4}: {} samples, inland {inland}, sea {sea}, max step {step:.2} m, dry gaps {gaps}, double surfaces {}, eye flips {}",
                    lu.0,
                    lu.1,
                    ll.0,
                    ll.1,
                    l.samples.len(),
                    l.overlaps,
                    l.flips
                ));
                overlaps += l.overlaps;
                flips += l.flips;
            }
        }
        report.push(format!(
            "mouths {}, tributaries {}, falls {} ({falls_to_sea} into the sea)",
            mouths.len(),
            tributaries.len(),
            falls.len()
        ));
        for line in &report {
            eprintln!("JUNCTION {line}");
        }
        assert!(mouths.len() >= 5, "five mouths sampled");
        assert_eq!(
            overlaps,
            0,
            "no cell draws two surfaces:\n{}",
            report.join("\n")
        );
        assert_eq!(
            flips,
            0,
            "the eye follows the local water:\n{}",
            report.join("\n")
        );
    }
}
/// MEASUREMENT (`ocean-environments` B22, the owner's sea caverns): a map of
/// what the sea's sheet lies over round a place, one character a metre,
/// compass north up, read off a fine set built there as the game builds it.
/// `PBD_AT='lat lon'` (the game's `--at`), `PBD_SPAN` metres each way.
#[cfg(test)]
mod sea_cave_report {
    use super::*;
    use crate::planet::column::{DRY_SHEET_BIT, sea_layer, sheet_listed};
    use pbd_core::terrain::Material;

    #[test]
    #[ignore = "a report: PBD_AT='41.636 11.664' cargo test -p pbd-app sea_cave_report -- --ignored --nocapture"]
    fn sea_cave_report() {
        let at = std::env::var("PBD_AT").unwrap_or_else(|_| "41.636 11.664".into());
        let v: Vec<f32> = at
            .split_whitespace()
            .filter_map(|x| x.parse().ok())
            .collect();
        let span: i32 = std::env::var("PBD_SPAN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        let up = pbd_core::geo::direction(pbd_core::geo::LatLon {
            lat: v[0].to_radians(),
            lon: v[1].to_radians(),
        });
        let set = std::sync::Arc::new(super::super::lod::generate_fine(
            up,
            &crate::config::ColumnSettings::default(),
            &pbd_core::edits::Edits::new(),
        ));
        let mut contact = crate::planet::PlanetContact::test_planet(5);
        contact.set_fine(&set);
        let sea = sea_layer();
        let high = terrain::high_water_m();
        let (n, e) = pbd_core::geo::compass_north_east(up);
        let r = terrain::PLANET_RADIUS;
        // `~` the sheet over the sea's water; `s` the sheet over ground
        // under the high water; `D` dry under the high water (no sheet); `C`
        // air under a roof at the sea's top layer; `W` the sea's water under
        // a roof; lower case where the sheet lies over the pocket; `w` other
        // water; `#` land; `@` the place; ` ` off the tier.
        let mut pockets = [0usize; 2];
        for j in (-span..=span).rev() {
            let mut row = String::new();
            for i in -span..=span {
                let p = (up + (n * j as f32 + e * i as f32) * (1.0 / r)).normalize();
                let ch = (|| {
                    let record = contact.finest_cell(p)?;
                    let column = set.columns.column(record)?;
                    let cell = &set.levels[3][record];
                    let h = cell.direction_height[3];
                    let dry = cell.spare[2] & DRY_SHEET_BIT != 0;
                    let roof = (sea..pbd_core::column::LAYERS).any(|l| column.solid(l));
                    let top = column.material(sea - 1);
                    // `PBD_ROW=j`: that row's stacks round the sea's level,
                    // for laying a capture path through open cells.
                    if std::env::var("PBD_ROW").ok().and_then(|r| r.parse().ok()) == Some(j) {
                        let stack: String = (sea - 4..sea + 4)
                            .map(|l| match column.material(l) {
                                Material::Air => '.',
                                m if m.is_water() => 'w',
                                _ => '#',
                            })
                            .collect();
                        eprintln!("ROW i {i:>3} layers -4..+4 m [{stack}]");
                    }
                    let ch = if i == 0 && j == 0 {
                        '@'
                    } else if roof && top == Material::Air {
                        'C'
                    } else if roof && top.is_water() {
                        'W'
                    } else if h < high && dry {
                        'D'
                    } else if h < high && top.is_water() {
                        '~'
                    } else if h < high {
                        's'
                    } else if top.is_water() || column.material(sea).is_water() {
                        'w'
                    } else {
                        '#'
                    };
                    let listed = sheet_listed(cell);
                    if matches!(ch, 'D' | 'C' | 'W') {
                        pockets[usize::from(listed)] += 1;
                    }
                    Some(if listed && matches!(ch, 'D' | 'C' | 'W') {
                        ch.to_ascii_lowercase()
                    } else {
                        ch
                    })
                })()
                .unwrap_or(' ');
                row.push(ch);
            }
            println!("{row}");
        }
        println!(
            "pockets under no sheet {}, under the sheet {}",
            pockets[0], pockets[1]
        );
    }
}
