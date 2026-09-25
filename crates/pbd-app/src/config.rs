//! Tunable look values, loaded from `assets/config/*.ron` at startup.
//!
//! The Rust `Default` impls are the one source of defaults. A shipped file is
//! the same set written out so a designer can turn a knob without a compiler;
//! `#[serde(default)]` means a missing field inherits the code default and a
//! present zero is zero, which is the explicit-optional rule in `CLAUDE.md`
//! rather than Tenebris's zero sentinel. Unknown fields are an error, because a
//! misspelled knob that silently does nothing is the failure that cost Tenebris
//! a week on white dock lights. Units are in the field docs.
//!
//! Loading is a startup read, not a Bevy `AssetLoader`; hot reload is the
//! `per-body-rendering` change's job and lands with the per-body files.

use bevy::{prelude::*, render::extract_resource::ExtractResource};
use serde::{Deserialize, de::DeserializeOwned};
use std::path::PathBuf;

/// The shipped `assets/config` directory, resolved the same way the desktop
/// app resolves its asset root.
fn config_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/config"))
}

/// Read `assets/config/<name>.ron`, or fall back to the code defaults when the
/// file is absent. A file that exists and fails to parse or validate is a
/// startup error with the path in it, never a silently default planet.
fn load<T: DeserializeOwned + Default + Validated>(name: &str) -> T {
    let path = config_dir().join(format!("{name}.ron"));
    let value: T = match std::fs::read_to_string(&path) {
        Ok(text) => {
            ron::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        }
        Err(_) => {
            warn!("{} not found; using built-in defaults", path.display());
            T::default()
        }
    };
    value
        .validate()
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    value
}

/// Every config type says what a legal value is; loading refuses the rest.
pub trait Validated {
    fn validate(&self) -> Result<(), String>;
}

fn finite(name: &str, values: &[f32]) -> Result<(), String> {
    values
        .iter()
        .all(|v| v.is_finite())
        .then_some(())
        .ok_or_else(|| format!("{name} must be finite"))
}

fn unit(name: &str, value: f32) -> Result<(), String> {
    (0.0..=1.0)
        .contains(&value)
        .then_some(())
        .ok_or_else(|| format!("{name} must be within 0..=1, got {value}"))
}

fn non_negative(name: &str, values: &[f32]) -> Result<(), String> {
    finite(name, values)?;
    values
        .iter()
        .all(|v| *v >= 0.0)
        .then_some(())
        .ok_or_else(|| format!("{name} must not be negative"))
}

fn positive(name: &str, values: &[f32]) -> Result<(), String> {
    finite(name, values)?;
    values
        .iter()
        .all(|v| *v > 0.0)
        .then_some(())
        .ok_or_else(|| format!("{name} must be greater than zero"))
}

/// The water cap pass and the composite's underwater terms. Names follow
/// Tenebris's `water.yaml` so the two can be read side by side.
#[derive(Resource, Clone, Debug, PartialEq, Deserialize, ExtractResource)]
#[serde(default, deny_unknown_fields)]
pub struct WaterSettings {
    /// Multiplier on elapsed seconds for every wave term.
    pub time_scale: f32,
    /// How far the sheet sits below the sea-level radius, metres, so a beach at
    /// sea level stands clear of the swell the way Tenebris's does.
    pub depth_offset_m: f32,
    /// The sea's surface: the table the water cap draws and the hulls float
    /// on (`pbd_core::sea`).
    pub sea: pbd_core::sea::SeaSettings,
    /// fbm sample scale, cells per metre.
    pub ripple_scale: f32,
    /// fbm time multiplier.
    pub ripple_speed: f32,
    /// How far the fbm gradient bends the normal.
    pub wave_steepness: f32,
    /// Cap on the tangent gradient length before it bends the normal.
    pub slope_max: f32,
    /// Screen-space refraction strength.
    pub refract_amount: f32,
    /// Maximum refraction offset, in screen UV.
    pub refract_max_uv: f32,
    /// Longest optical path through water, metres, used where scene depth is
    /// sky.
    pub max_path_m: f32,
    /// RGB absorption per metre.
    pub absorption_per_m: [f32; 3],
    /// Colour water converges to at infinite depth, linear RGB.
    pub deep_color: [f32; 3],
    /// Reflected sky at the horizon, linear RGB.
    pub sky_horizon_color: [f32; 3],
    /// Reflected sky at the zenith, linear RGB.
    pub sky_zenith_color: [f32; 3],
    /// The sky the sheet mirrors at night, linear RGB. The two colours above
    /// are the daytime gradient; this is what they ramp to across the
    /// terminator. It is authored against the sky this engine actually draws
    /// at night, which is dimmer than the fog's night colour, and it is the
    /// knob to turn if the sea reads as lit under a dark sky.
    pub night_sky_color: [f32; 3],
    /// Fresnel floor for grazing reflection, 0..1.
    pub sky_horizon_strength: f32,
    pub foam_crest_lo: f32,
    pub foam_crest_hi: f32,
    pub foam_crest_weight: f32,
    pub foam_slope_lo: f32,
    pub foam_slope_hi: f32,
    pub foam_slope_weight: f32,
    /// Overall foam mix, 0..1.
    pub foam_intensity: f32,
    pub foam_color: [f32; 3],
    pub specular_intensity: f32,
    /// Blinn-Phong exponent.
    pub specular_power: f32,
    /// Multiplier on the specular highlight, linear RGB.
    pub sun_tint: [f32; 3],
    /// Night-side brightness floor, 0..1.
    pub night_floor: f32,
    /// Ceiling on the distance-fog mix so far water never fully becomes sky.
    pub fog_max: f32,
    /// Rain ripple cells per metre on the surface.
    pub rain_ripple_scale: f32,
    /// Push of the rain ripple gradient into the wave gradient.
    pub rain_ripple_strength: f32,
    /// Underwater screen distortion amplitude, screen UV.
    pub underwater_distortion: f32,
    /// Depth-blur blend while the lens is wet, 0..1.
    pub wet_blur: f32,
    /// Half-width of the straddling band around the sea surface under the
    /// camera, metres.
    pub partial_band_m: f32,
    /// Seconds the lens stays wet after surfacing.
    pub emerge_dry_s: f32,
    /// Radial scroll speed of the wave field on vertical water faces.
    pub flow_uv_speed_falling: f32,
    /// How fast the wave normal and crest foam fade as the fbm's features fall
    /// under a pixel: the gradient is scaled by `1 / (1 + footprint * fade)`
    /// where footprint is the per-pixel change of the noise coordinate. Zero
    /// is Tenebris exactly, which sparkles from altitude.
    pub detail_fade: f32,
}

impl Default for WaterSettings {
    fn default() -> Self {
        Self {
            time_scale: 0.75,
            depth_offset_m: 0.5,
            sea: pbd_core::sea::SeaSettings::default(),
            ripple_scale: 1.5,
            ripple_speed: 1.0,
            wave_steepness: 0.45,
            slope_max: 0.7,
            refract_amount: 0.04,
            refract_max_uv: 0.03,
            max_path_m: 200.0,
            absorption_per_m: [0.90, 0.25, 0.08],
            deep_color: [0.0, 0.12, 0.28],
            sky_horizon_color: [0.10, 0.36, 0.72],
            sky_zenith_color: [0.03, 0.18, 0.55],
            night_sky_color: [0.035, 0.070, 0.100],
            sky_horizon_strength: 0.22,
            foam_crest_lo: 0.35,
            foam_crest_hi: 0.60,
            foam_crest_weight: 0.55,
            foam_slope_lo: 0.50,
            foam_slope_hi: 1.20,
            foam_slope_weight: 0.26,
            foam_intensity: 0.10,
            foam_color: [0.95, 0.97, 1.00],
            specular_intensity: 0.06,
            specular_power: 140.0,
            sun_tint: [1.10, 1.15, 1.20],
            night_floor: 0.18,
            fog_max: 0.82,
            rain_ripple_scale: 3.0,
            rain_ripple_strength: 6.0,
            underwater_distortion: 0.0015,
            wet_blur: 0.02,
            partial_band_m: 0.8,
            emerge_dry_s: 2.6,
            flow_uv_speed_falling: 1.0,
            detail_fade: 4.0,
        }
    }
}

impl Validated for WaterSettings {
    fn validate(&self) -> Result<(), String> {
        let s = self;
        non_negative(
            "water scalars",
            &[
                s.time_scale,
                s.depth_offset_m,
                s.ripple_scale,
                s.ripple_speed,
                s.wave_steepness,
                s.slope_max,
                s.refract_amount,
                s.refract_max_uv,
                s.max_path_m,
                s.foam_crest_lo,
                s.foam_crest_hi,
                s.foam_slope_lo,
                s.foam_slope_hi,
                s.specular_intensity,
                s.specular_power,
                s.rain_ripple_scale,
                s.rain_ripple_strength,
                s.underwater_distortion,
                s.partial_band_m,
                s.emerge_dry_s,
                s.flow_uv_speed_falling,
                s.detail_fade,
            ],
        )?;
        s.sea.validate()?;
        non_negative("absorption_per_m", &s.absorption_per_m)?;
        non_negative("deep_color", &s.deep_color)?;
        non_negative("sky_horizon_color", &s.sky_horizon_color)?;
        non_negative("sky_zenith_color", &s.sky_zenith_color)?;
        non_negative("night_sky_color", &s.night_sky_color)?;
        non_negative("foam_color", &s.foam_color)?;
        non_negative("sun_tint", &s.sun_tint)?;
        for (name, value) in [
            ("sky_horizon_strength", s.sky_horizon_strength),
            ("foam_crest_weight", s.foam_crest_weight),
            ("foam_slope_weight", s.foam_slope_weight),
            ("foam_intensity", s.foam_intensity),
            ("night_floor", s.night_floor),
            ("fog_max", s.fog_max),
            ("wet_blur", s.wet_blur),
        ] {
            unit(name, value)?;
        }
        (s.fog_max < 1.0)
            .then_some(())
            .ok_or("fog_max must stay below 1 so far water never becomes sky")?;
        (s.specular_power >= 1.0)
            .then_some(())
            .ok_or("specular_power must be at least 1")?;
        Ok(())
    }
}

/// Rain, and what it does to the lens, the ground and the air. Names follow
/// Tenebris's `weather.yaml`.
#[derive(Resource, Clone, Debug, PartialEq, Deserialize, ExtractResource)]
#[serde(default, deny_unknown_fields)]
pub struct WeatherSettings {
    /// Seconds for ground wetness to follow the rain intensity (e-fold).
    pub wet_fade_tau_s: f32,
    /// The wind a craft feels: the atmosphere's wind sheared with height and
    /// gusting (`pbd_core::wind`).
    pub gusts: pbd_core::wind::GustSettings,

    // ---- Where it rains, and how hard, is the simulated atmosphere's
    // (`atmosphere.ron`); these are how rain LOOKS.
    /// Streak fall speed, metres per second.
    pub rain_fall_mps: f32,
    /// Streak half-width, metres.
    pub rain_width_m: f32,
    /// Streak length, metres.
    pub rain_streak_m: f32,
    /// Streak colour, display (sRGB) RGB as Tenebris authored it.
    pub rain_color: [f32; 3],
    /// Snow: fall speed (m/s), flake half-size (m), sideways sway (m), and
    /// colour (display sRGB). Tenebris's snow falls at 3 m/s.
    pub snow_fall_mps: f32,
    pub snow_size_m: f32,
    pub snow_sway_m: f32,
    pub snow_color: [f32; 3],
    /// How many more flakes than streaks the near shower draws in snow.
    pub snow_density: f32,
    /// Flake opacity at the camera and at the edge of the shower disk.
    pub snow_alpha_near: f32,
    pub snow_alpha_far: f32,
    /// Streak alpha at the camera and at the edge of the shower disk.
    pub rain_alpha_near: f32,
    pub rain_alpha_far: f32,
    /// Radius of the near shower disk around the camera, metres.
    pub shower_radius_m: f32,
    /// Height streaks fall through above the surface, metres.
    pub shower_column_m: f32,
    /// Streaks at full intensity.
    pub shower_max_streaks: u32,
    /// Lens droplet coverage (below 1 culls, above 1 stacks layers).
    pub rain_lens_density: f32,
    /// Lens droplet refraction strength.
    pub rain_lens_refract: f32,
    /// Lens droplet animation speed.
    pub rain_lens_speed: f32,
    /// Lens droplet grid frequency: higher is smaller drops.
    pub rain_lens_size: f32,
    /// Terrain impact-ring cells per metre.
    pub rain_ripple_scale: f32,
    pub rain_ripple_strength: f32,
    /// Rivulet lanes across a face (U scale) and down it (V scale).
    pub rain_flow_across: f32,
    pub rain_flow_down: f32,
    pub rain_flow_speed: f32,
    pub rain_flow_strength: f32,
    /// Wet-sheet wave scale, strength and speed on up-faces.
    pub rain_wave_scale: f32,
    pub rain_wave_strength: f32,
    pub rain_wave_speed: f32,
    /// Albedo multiplier when fully wet, 0..1.
    pub rain_wet_darken: f32,
    /// Sun glint exponent and strength.
    pub rain_glint_power: f32,
    pub rain_glint_strength: f32,
    /// Size of the noise cells puddles are cut from, metres.
    pub rain_puddle_scale_m: f32,
    /// Share of a flat, fully soaked face that stands in puddles, 0..1.
    pub rain_puddle_share: f32,
    /// Weight of the sky mirrored in a puddle, 0..1. Wet ground outside a
    /// puddle takes a quarter of it.
    pub rain_mirror_strength: f32,
    /// Strength of the raindrop rings on grass against bare ground, 0..1.
    pub rain_grass_rings: f32,

    // ---- Overcast: what the cover over the player does to the light. Names
    // and values are Tenebris's. Each is a fraction taken off (or, for the
    // haze, added on) at full cover, scaled linearly by the cover.
    /// Direct sun on the ground, the water's sun specular and the wet glint.
    pub overcast_sun_dim: f32,
    /// Fill (sky ambient) on the ground.
    pub overcast_amb_dim: f32,
    /// Sky dome Rayleigh scattering: the blue goes grey.
    pub overcast_sky_blue_cut: f32,
    /// Sky dome Mie scattering, as a gain: the white haze comes up.
    pub overcast_sky_haze: f32,
    /// Sky dome sun radiance, which is also the sun disc.
    pub overcast_sky_dim: f32,
    /// Extra distance-haze density per unit cover.
    pub cloud_fog_add: f32,
    /// Distance-haze density multiplier in full rain.
    pub rain_fog_mult: f32,
    /// How much of the sun a full, thick cloud keeps off the ground under it:
    /// the depth of a cloud's shadow, 0..1.
    pub cloud_shadow: f32,

    // ---- The clouds (`shaders/clouds.wgsl`): where they are is the
    // atmosphere's weather map; this is how they are lit. See the
    // cloud-lighting change for the model and what each knob does to it.
    /// How much a metre of full-density cloud absorbs, per metre, in fair
    /// weather: for the view AND for the light, one number for both.
    pub cloud_extinction: f32,
    /// Extinction at full cover, per metre, mixed in by the local cover: a
    /// storm is denser cloud, not only more of it.
    pub cloud_storm_extinction: f32,
    /// Cloud brightness with the sun down.
    pub cloud_night_floor: f32,
    /// The sun's strength on a cloud.
    pub cloud_sun: f32,
    /// Sky light from above, through the cloud over a sample.
    pub cloud_ambient_sky: f32,
    /// Light bounced up off the ground onto a cloud's base.
    pub cloud_ambient_ground: f32,
    /// The phase function's forward lobe (a silver lining toward the sun),
    /// its back lobe, and their blend: Henyey-Greenstein g in -1..1.
    pub cloud_phase_forward: f32,
    pub cloud_phase_back: f32,
    pub cloud_phase_blend: f32,
    /// Multiple scattering (three octaves): each octave's extinction, energy
    /// and phase are these shares of the last, 0..1.
    pub cloud_scatter_extinction_falloff: f32,
    pub cloud_scatter_energy_falloff: f32,
    pub cloud_scatter_phase_falloff: f32,
    /// Detail finer than the atmosphere's cells (`cloud_density`). The floor
    /// under the cover remap, 0..1: at full cover the noise's troughs below
    /// it stay open, so a deck breaks into cells and lanes; tall convective
    /// cloud keeps none and stands solid.
    pub cloud_deck_floor: f32,
    /// How much the fine octaves eat into a cloud's soft edges, 0..1.
    pub cloud_erosion: f32,
    /// How far detail is combed along the wind aloft at `cloud_shear_mps`
    /// and above: 1 for none, and each unit past it one noise cell (about
    /// 230 m) of push, varying from place to place so bands slide past one
    /// another along the flow.
    pub cloud_shear: f32,
    pub cloud_shear_mps: f32,
    /// How far convective cloud (a tall top) takes the cellular texture of
    /// cumulus rather than the smooth one of a deck, 0..1.
    pub cloud_cells: f32,
    // ---- What the clouds cost (`cloud-budget`): the march's resolution and
    // its steps. Quality knobs, not weather.
    /// The resolution the cloud march runs at, as a fraction of the view's
    /// along each axis, 0.25..=1. A full-resolution composite lays it over
    /// the scene.
    pub cloud_render_scale: f32,
    /// The march's nearest step, metres. Steps grow with distance from the
    /// eye from here.
    pub cloud_step_m: f32,
    /// The most steps a ray takes through the layer, 8..=256.
    pub cloud_max_steps: u32,

    // ---- Rain near: Tenebris's shafts of streaks over every raining cell of
    // a lattice fixed to the body, inside the detail range. Beyond it the rain
    // is the volume below.
    /// Size of a rain cell, metres.
    pub rain_cell_m: f32,
    /// Cells nearer than this draw streaks, metres.
    pub rain_detail_range_m: f32,
    /// The band ending at the detail range over which streaks fade out, metres.
    pub rain_lod_blend_m: f32,
    /// Share of a cell's streaks kept at the far edge of the detail range.
    pub rain_lod_far_frac: f32,
    /// Streaks per square metre of a raining cell at the camera.
    pub rain_cell_density: f32,
    /// Width multiplier on a cell streak at the detail range over a near-shower
    /// streak, reached linearly from one at the camera, so a shaft a hundred
    /// metres off is not all sub-pixel and one beside you is not a pole.
    pub rain_cell_width_mult: f32,
    /// Cap on cell streaks per frame.
    pub rain_max_cell_streaks: u32,
    /// Camera altitude above which no rain is drawn and none runs down the
    /// lens, metres. The storm still reads through clouds, overcast and haze.
    pub rain_lod_alt_m: f32,

    // ---- Rain far: a volume marched per pixel between the camera and the
    // ground, the sea or the cloud base, off a map of the field around the
    // camera. See `openspec/changes/storm`.
    /// Cells along a side of the precipitation map (at most 128).
    pub rain_map_size: u32,
    /// Size of a map cell, metres.
    pub rain_map_cell_m: f32,
    /// Extinction per metre of full rain.
    pub rain_volume_density: f32,
    /// How far along a ray the volume is marched, metres.
    pub rain_volume_range_m: f32,
    /// How much longer than wide a falling streak of the volume is.
    pub rain_volume_stretch: f32,
    /// The volume's colour, display (sRGB) RGB. Snow is `snow_color`.
    pub rain_volume_color: [f32; 3],

    // ---- Lightning: where it strikes is the atmosphere's; this is how a
    // strike looks.
    /// How long one strike's flashes last, seconds.
    pub lightning_flash_s: f32,
    /// Brightness a strike lights the cloud from inside with.
    pub lightning_cloud: f32,
    /// Brightness a strike lights the ground and the rain with.
    pub lightning_ground: f32,

    // ---- The overlays (`overlay.rs`, the `overlay` pass in `water.wgsl`):
    // what the map looks like, never what it shows.
    /// How much of the overlay's colour is laid over the scene, 0..1.
    pub overlay_opacity: f32,
    /// One streamline step along the flow, metres. A streak is twelve.
    pub overlay_streak_step_m: f32,
    /// How fast the streaks crawl at the top of the overlay's range: streak
    /// lengths per second. Slower flows crawl slower in proportion.
    pub overlay_streak_scroll: f32,
    /// How bright a streak is laid over the colour, 0..1.
    pub overlay_streak_strength: f32,
}

impl Default for WeatherSettings {
    fn default() -> Self {
        Self {
            wet_fade_tau_s: 1.6,
            gusts: pbd_core::wind::GustSettings::default(),
            rain_fall_mps: 70.0,
            rain_width_m: 0.012,
            rain_streak_m: 1.5,
            rain_color: [0.52, 0.62, 0.90],
            snow_fall_mps: 3.0,
            snow_size_m: 0.07,
            snow_sway_m: 0.5,
            snow_color: [0.93, 0.95, 1.0],
            snow_density: 3.0,
            snow_alpha_near: 0.95,
            snow_alpha_far: 0.6,
            rain_alpha_near: 0.78,
            rain_alpha_far: 0.10,
            shower_radius_m: 18.0,
            shower_column_m: 46.0,
            shower_max_streaks: 1700,
            rain_lens_density: 1.4,
            rain_lens_refract: 1.7,
            rain_lens_speed: 1.0,
            rain_lens_size: 0.7,
            rain_ripple_scale: 5.0,
            rain_ripple_strength: 2.0,
            rain_flow_across: 128.0,
            rain_flow_down: 1.0,
            rain_flow_speed: 1.3,
            rain_flow_strength: -2.0,
            rain_wave_scale: 7.0,
            rain_wave_strength: 0.6,
            rain_wave_speed: 1.4,
            rain_wet_darken: 0.72,
            rain_glint_power: 24.0,
            rain_glint_strength: 0.5,
            rain_puddle_scale_m: 0.4,
            rain_puddle_share: 0.35,
            rain_mirror_strength: 1.0,
            rain_grass_rings: 0.5,
            overcast_sun_dim: 0.72,
            overcast_amb_dim: 0.48,
            overcast_sky_blue_cut: 0.75,
            overcast_sky_haze: 0.6,
            overcast_sky_dim: 0.6,
            cloud_fog_add: 0.35,
            rain_fog_mult: 1.6,
            cloud_shadow: 0.8,
            cloud_extinction: 0.0115,
            cloud_storm_extinction: 0.03,
            cloud_night_floor: 0.045,
            cloud_sun: 0.55,
            cloud_ambient_sky: 0.9,
            cloud_ambient_ground: 0.35,
            cloud_phase_forward: 0.8,
            cloud_phase_back: -0.3,
            cloud_phase_blend: 0.5,
            cloud_scatter_extinction_falloff: 0.5,
            cloud_scatter_energy_falloff: 0.5,
            cloud_scatter_phase_falloff: 0.5,
            cloud_deck_floor: 0.45,
            cloud_erosion: 0.5,
            cloud_shear: 2.0,
            cloud_shear_mps: 25.0,
            cloud_cells: 0.6,
            cloud_render_scale: 0.4,
            cloud_step_m: 12.0,
            cloud_max_steps: 48,
            rain_cell_m: 60.0,
            rain_detail_range_m: 150.0,
            rain_lod_blend_m: 60.0,
            rain_lod_far_frac: 0.25,
            rain_cell_density: 0.058,
            rain_cell_width_mult: 6.0,
            rain_max_cell_streaks: 9000,
            rain_lod_alt_m: 200.0,
            rain_map_size: 64,
            rain_map_cell_m: 50.0,
            rain_volume_density: 0.003,
            rain_volume_range_m: 2000.0,
            rain_volume_stretch: 10.0,
            rain_volume_color: [0.55, 0.58, 0.62],
            lightning_flash_s: 0.7,
            lightning_cloud: 6.0,
            lightning_ground: 0.8,
            overlay_opacity: 0.7,
            overlay_streak_step_m: 50.0,
            overlay_streak_scroll: 0.35,
            overlay_streak_strength: 0.8,
        }
    }
}

impl Validated for WeatherSettings {
    fn validate(&self) -> Result<(), String> {
        let s = self;
        s.gusts.validate()?;
        non_negative(
            "weather scalars",
            &[
                s.wet_fade_tau_s,
                s.rain_fall_mps,
                s.rain_width_m,
                s.rain_streak_m,
                s.shower_radius_m,
                s.shower_column_m,
                s.rain_lens_density,
                s.rain_lens_refract,
                s.rain_lens_speed,
                s.rain_lens_size,
                s.rain_ripple_scale,
                s.rain_ripple_strength,
                s.rain_flow_across,
                s.rain_flow_down,
                s.rain_flow_speed,
                s.rain_wave_scale,
                s.rain_wave_strength,
                s.rain_wave_speed,
                s.rain_glint_power,
                s.rain_glint_strength,
                s.overcast_sky_haze,
                s.cloud_fog_add,
                s.rain_fog_mult,
                s.cloud_extinction,
                s.cloud_storm_extinction,
                s.rain_map_cell_m,
                s.rain_volume_density,
                s.rain_volume_range_m,
                s.rain_volume_stretch,
                s.lightning_flash_s,
                s.lightning_cloud,
                s.lightning_ground,
                s.rain_detail_range_m,
                s.rain_lod_blend_m,
                s.rain_cell_density,
                s.rain_cell_width_mult,
                s.rain_lod_alt_m,
            ],
        )?;
        finite("rain_flow_strength", &[s.rain_flow_strength])?;
        non_negative("rain_color", &s.rain_color)?;
        non_negative("snow_color", &s.snow_color)?;
        positive("snow fall and size", &[s.snow_fall_mps, s.snow_size_m])?;
        non_negative("snow_sway_m", &[s.snow_sway_m])?;
        (0.0..=10.0)
            .contains(&s.snow_density)
            .then_some(())
            .ok_or("snow_density must be within 0..=10")?;
        non_negative("rain_volume_color", &s.rain_volume_color)?;
        for (name, value) in [
            ("rain_alpha_near", s.rain_alpha_near),
            ("rain_alpha_far", s.rain_alpha_far),
            ("snow_alpha_near", s.snow_alpha_near),
            ("snow_alpha_far", s.snow_alpha_far),
            ("rain_wet_darken", s.rain_wet_darken),
            ("rain_puddle_share", s.rain_puddle_share),
            ("rain_mirror_strength", s.rain_mirror_strength),
            ("rain_grass_rings", s.rain_grass_rings),
            ("overcast_sun_dim", s.overcast_sun_dim),
            ("overcast_amb_dim", s.overcast_amb_dim),
            ("overcast_sky_blue_cut", s.overcast_sky_blue_cut),
            ("overcast_sky_dim", s.overcast_sky_dim),
            ("cloud_shadow", s.cloud_shadow),
            ("cloud_night_floor", s.cloud_night_floor),
            ("cloud_phase_forward", s.cloud_phase_forward),
            ("cloud_phase_blend", s.cloud_phase_blend),
            (
                "cloud_scatter_extinction_falloff",
                s.cloud_scatter_extinction_falloff,
            ),
            (
                "cloud_scatter_energy_falloff",
                s.cloud_scatter_energy_falloff,
            ),
            ("cloud_scatter_phase_falloff", s.cloud_scatter_phase_falloff),
            ("cloud_deck_floor", s.cloud_deck_floor),
            ("cloud_erosion", s.cloud_erosion),
            ("cloud_cells", s.cloud_cells),
            ("rain_lod_far_frac", s.rain_lod_far_frac),
            ("overlay_opacity", s.overlay_opacity),
            ("overlay_streak_strength", s.overlay_streak_strength),
        ] {
            unit(name, value)?;
        }
        positive(
            "overlay streaks",
            &[s.overlay_streak_step_m, s.overlay_streak_scroll],
        )?;
        (0.25..=1.0)
            .contains(&s.cloud_render_scale)
            .then_some(())
            .ok_or("cloud_render_scale must be within 0.25..=1")?;
        positive("cloud_step_m", &[s.cloud_step_m])?;
        (8..=256)
            .contains(&s.cloud_max_steps)
            .then_some(())
            .ok_or("cloud_max_steps must be within 8..=256")?;
        (s.cloud_shear >= 1.0)
            .then_some(())
            .ok_or("cloud_shear must be at least 1 (1 draws no shear)")?;
        positive("cloud_shear_mps", &[s.cloud_shear_mps])?;
        (s.rain_puddle_scale_m > 0.0)
            .then_some(())
            .ok_or("rain_puddle_scale_m must be positive")?;
        // A fog multiplier under one would CLEAR the air when it rains.
        (s.rain_fog_mult >= 1.0)
            .then_some(())
            .ok_or("rain_fog_mult must be at least 1")?;
        // Clear must be the higher threshold, or cover would thin the clouds.
        (-1.0..=0.0)
            .contains(&s.cloud_phase_back)
            .then_some(())
            .ok_or("cloud_phase_back must be within -1..0")?;
        non_negative(
            "cloud light",
            &[s.cloud_sun, s.cloud_ambient_sky, s.cloud_ambient_ground],
        )?;
        (s.rain_cell_m >= 1.0)
            .then_some(())
            .ok_or("rain_cell_m must be at least a metre")?;
        (s.rain_map_size >= 2 && s.rain_map_size <= 128)
            .then_some(())
            .ok_or("rain_map_size must be within 2..=128")?;
        (s.lightning_flash_s > 0.0)
            .then_some(())
            .ok_or("lightning_flash_s must be positive")?;
        (s.rain_lod_blend_m <= s.rain_detail_range_m)
            .then_some(())
            .ok_or("rain_lod_blend_m must not exceed rain_detail_range_m")?;
        (s.rain_max_cell_streaks <= 100_000)
            .then_some(())
            .ok_or("rain_max_cell_streaks is capped at 100000")?;
        (s.wet_fade_tau_s > 0.0)
            .then_some(())
            .ok_or("wet_fade_tau_s must be positive")?;
        (s.rain_glint_power >= 1.0)
            .then_some(())
            .ok_or("rain_glint_power must be at least 1")?;
        (s.shower_max_streaks <= 100_000)
            .then_some(())
            .ok_or("shower_max_streaks is capped at 100000")?;
        Ok(())
    }
}

/// Decorative ground clutter: Tenebris's surface scatter, rebuilt as a GPU
/// draw. Names follow its `scatter.yaml` so the two can be read side by side,
/// and the shipped values are its shipped values. What is ours rather than the
/// reference's is the REACH: Tenebris meshes scatter over its whole 300 m
/// planet, and this body is sixteen times that radius, so the clutter needs a
/// range of its own. See `openspec/changes/tenebris-ground-clutter`.
#[derive(Resource, Clone, Debug, PartialEq, Deserialize, ExtractResource)]
#[serde(default, deny_unknown_fields)]
pub struct ScatterSettings {
    /// How far from the camera clutter is drawn at all, metres. Beyond about
    /// 60 m a blade is under two pixels wide and costs a vertex to shimmer.
    pub clutter_radius_m: f32,
    /// The last stretch of that reach, metres, over which a piece shrinks into
    /// the ground rather than popping out of existence.
    pub clutter_fade_m: f32,
    /// Probability a grass cell grows a tuft.
    pub grass_chance: f32,
    /// Blades per covered cell, upper bound; a hash picks 60%..100% of it.
    pub grass_blades: f32,
    /// Blade height, metres; a per-blade hash varies it +-20%.
    pub grass_height_m: f32,
    /// Blade half-width at the base, metres.
    pub grass_blade_w_m: f32,
    /// How dark the blade root is against its tip, 0..1, which is what makes a
    /// sward read as lush rather than flat.
    pub grass_base_shade: f32,
    /// Probability a grass cell gets a flower.
    pub flower_chance: f32,
    /// Flower stem height, metres.
    pub flower_height_m: f32,
    /// Probability a bare cell gets a pebble; a grass cell rolls a third of it.
    pub rock_chance: f32,
    /// Pebble footprint radius, metres.
    pub rock_size_m: f32,
    /// Probability a grass or dirt cell gets a leafy bush.
    pub bush_chance: f32,
    /// Bush footprint radius, metres.
    pub bush_size_m: f32,
    /// Probability a desert sand or tundra snow cell grows a dead shrub.
    pub shrub_chance: f32,
    /// Dead-shrub twig length, metres.
    pub shrub_size_m: f32,
}

impl Default for ScatterSettings {
    fn default() -> Self {
        Self {
            clutter_radius_m: 60.,
            clutter_fade_m: 15.,
            grass_chance: 0.8,
            grass_blades: 18.,
            grass_height_m: 0.55,
            grass_blade_w_m: 0.085,
            grass_base_shade: 0.55,
            flower_chance: 0.12,
            flower_height_m: 0.32,
            rock_chance: 0.10,
            rock_size_m: 0.16,
            bush_chance: 0.05,
            bush_size_m: 0.34,
            shrub_chance: 0.14,
            shrub_size_m: 0.38,
        }
    }
}

impl Validated for ScatterSettings {
    fn validate(&self) -> Result<(), String> {
        non_negative(
            "clutter distances",
            &[self.clutter_radius_m, self.clutter_fade_m],
        )?;
        // A reach of zero is the off switch, and an off switch that a config
        // cannot reach is not one: the shader stops at the reach before it ever
        // looks at the fade, so the fade is moot there. Everywhere else a fade
        // longer than the reach would mean every piece is part-faded, which is
        // a config nobody means to write.
        (self.clutter_radius_m == 0. || self.clutter_fade_m <= self.clutter_radius_m)
            .then_some(())
            .ok_or("clutter_fade_m cannot exceed clutter_radius_m")?;
        unit("grass_chance", self.grass_chance)?;
        unit("flower_chance", self.flower_chance)?;
        unit("rock_chance", self.rock_chance)?;
        unit("bush_chance", self.bush_chance)?;
        unit("shrub_chance", self.shrub_chance)?;
        unit("grass_base_shade", self.grass_base_shade)?;
        non_negative(
            "clutter sizes",
            &[
                self.grass_height_m,
                self.grass_blade_w_m,
                self.flower_height_m,
                self.rock_size_m,
                self.bush_size_m,
                self.shrub_size_m,
            ],
        )?;
        // The vertex budget covers this many blades and no more; a config that
        // asked for more would silently draw fewer, which is the kind of quiet
        // disagreement between a file and the code this project has a rule
        // about. GRASS_BLADE_BUDGET in planet.rs is the same number.
        (self.grass_blades >= 0. && self.grass_blades <= crate::planet::GRASS_BLADE_BUDGET as f32)
            .then_some(())
            .ok_or("grass_blades must be within 0..=18, the vertex budget")?;
        Ok(())
    }
}

/// The voxel column tier: how far it reaches, how the worms that carve its
/// caves are grown, and the stand-in that keeps a cave interior from being lit
/// like a hillside.
///
/// The worm fields default to `pbd_core::worms::WormField::DEFAULT`, which is
/// where the measured values are recorded, so there is one source for them
/// and this is its override. See `openspec/changes/voxel-columns-and-mining`.
#[derive(Resource, Clone, Debug, PartialEq, Deserialize, ExtractResource)]
#[serde(default, deny_unknown_fields)]
pub struct ColumnSettings {
    /// Great-circle metres from the band anchor out to which columns exist.
    pub reach_m: f32,
    /// Seed lattice cell for the worms, metres of arc. About a base tile.
    pub worm_cell_m: f32,
    /// Mean worms per seed cell.
    pub worm_density: f32,
    /// Shortest and longest worm, metres.
    pub worm_length_m: (f32, f32),
    /// Narrowest and widest tunnel radius, metres.
    pub worm_radius_m: (f32, f32),
    /// Metres a worm advances per step.
    pub worm_step_m: f32,
    /// Most a worm turns in one step, radians.
    pub worm_turn: f32,
    /// Steepest pitch off the tangent plane, radians.
    pub worm_pitch_max: f32,
    /// Share of worms that start at the surface heading down: the openings.
    pub worm_surface_share: f32,
    /// How far under the ground a buried worm starts, metres.
    pub worm_start_depth_m: (f32, f32),
    /// Metres across the noise a worm steers on.
    pub worm_steer_scale_m: f32,
    /// Layers of solid a worm keeps above the bedrock floor.
    pub cave_floor_layers: u32,
}

impl Default for ColumnSettings {
    fn default() -> Self {
        let worms = pbd_core::worms::WormField::DEFAULT;
        Self {
            reach_m: 90.,
            worm_cell_m: worms.cell_m,
            worm_density: worms.density,
            worm_length_m: worms.length_m,
            worm_radius_m: worms.radius_m,
            worm_step_m: worms.step_m,
            worm_turn: worms.turn,
            worm_pitch_max: worms.pitch_max,
            worm_surface_share: worms.surface_share,
            worm_start_depth_m: worms.start_depth_m,
            worm_steer_scale_m: worms.steer_scale_m,
            cave_floor_layers: worms.floor_layers as u32,
        }
    }
}

impl ColumnSettings {
    /// The worms these settings grow.
    pub fn worms(&self) -> pbd_core::worms::WormField {
        pbd_core::worms::WormField {
            cell_m: self.worm_cell_m,
            density: self.worm_density,
            length_m: self.worm_length_m,
            radius_m: self.worm_radius_m,
            step_m: self.worm_step_m,
            turn: self.worm_turn,
            pitch_max: self.worm_pitch_max,
            surface_share: self.worm_surface_share,
            start_depth_m: self.worm_start_depth_m,
            steer_scale_m: self.worm_steer_scale_m,
            floor_layers: self.cave_floor_layers as usize,
        }
    }
}

fn ordered(name: &str, range: (f32, f32)) -> Result<(), String> {
    positive(name, &[range.0, range.1])?;
    (range.0 <= range.1)
        .then_some(())
        .ok_or_else(|| format!("{name} must be (min, max) with min <= max"))
}

impl Validated for ColumnSettings {
    fn validate(&self) -> Result<(), String> {
        // A reach of zero is the off switch: no columns, and every cell answers
        // from the heightfield exactly as it did before this tier existed.
        non_negative("reach_m", &[self.reach_m])?;
        positive("worm_cell_m", &[self.worm_cell_m])?;
        non_negative("worm_density", &[self.worm_density])?;
        ordered("worm_length_m", self.worm_length_m)?;
        ordered("worm_radius_m", self.worm_radius_m)?;
        ordered("worm_start_depth_m", self.worm_start_depth_m)?;
        positive("worm_step_m", &[self.worm_step_m, self.worm_steer_scale_m])?;
        non_negative("worm_turn", &[self.worm_turn, self.worm_pitch_max])?;
        unit("worm_surface_share", self.worm_surface_share)?;
        ((self.cave_floor_layers as usize) < pbd_core::column::LAYERS)
            .then_some(())
            .ok_or("cave_floor_layers must be inside the column span")?;
        Ok(())
    }
}

/// Loads every config file once at startup. Inserted before any plugin that
/// reads them, so a system can take `Res<WaterSettings>` unconditionally.
/// The simulated atmosphere's knobs (`pbd_core::atmosphere`), from
/// `atmosphere.ron`. The defaults are the core's; this only carries them into
/// the app as a resource.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Default)]
pub struct AtmosphereConfig(pub pbd_core::atmosphere::AtmosphereSettings);

impl Validated for pbd_core::atmosphere::AtmosphereSettings {
    fn validate(&self) -> Result<(), String> {
        pbd_core::atmosphere::AtmosphereSettings::validate(self)
    }
}

/// The three craft (`pbd_core::vehicle::spec`), from `vehicles.ron`.
impl Validated for pbd_core::vehicle::spec::VehicleSpecs {
    fn validate(&self) -> Result<(), String> {
        pbd_core::vehicle::spec::VehicleSpecs::validate(self)
    }
}

/// Every number the craft are made of, loaded once.
#[derive(Resource, Clone, Debug, PartialEq, Default)]
pub struct VehiclesConfig(pub pbd_core::vehicle::spec::VehicleSpecs);

/// What lives in the water (`pbd_core::fauna`), from `fauna.ron`.
impl Validated for pbd_core::fauna::FaunaSettings {
    fn validate(&self) -> Result<(), String> {
        pbd_core::fauna::FaunaSettings::validate(self)
    }
}

/// Every body's species, how they school and how the rod works, loaded once.
#[derive(Resource, Clone, Debug, PartialEq, Default)]
pub struct FaunaConfig(pub pbd_core::fauna::FaunaSettings);

/// How long each material takes to break with each tool (`pbd_core::dig`).
impl Validated for pbd_core::dig::DigSettings {
    fn validate(&self) -> Result<(), String> {
        pbd_core::dig::DigSettings::validate(self)
    }
}

/// The break times, loaded once.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Default)]
pub struct DigConfig(pub pbd_core::dig::DigSettings);

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load::<WaterSettings>("water"))
            .insert_resource(load::<WeatherSettings>("weather"))
            .insert_resource(load::<ScatterSettings>("scatter"))
            .insert_resource(load::<ColumnSettings>("column"))
            .insert_resource(AtmosphereConfig(load("atmosphere")))
            .insert_resource(VehiclesConfig(load("vehicles")))
            .insert_resource(FaunaConfig(load("fauna")))
            .insert_resource(DigConfig(load("dig")))
            .add_plugins((
                bevy::render::extract_resource::ExtractResourcePlugin::<WaterSettings>::default(),
                bevy::render::extract_resource::ExtractResourcePlugin::<WeatherSettings>::default(),
                bevy::render::extract_resource::ExtractResourcePlugin::<ScatterSettings>::default(),
                bevy::render::extract_resource::ExtractResourcePlugin::<ColumnSettings>::default(),
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATER_RON: &str = include_str!("../../../assets/config/water.ron");
    const WEATHER_RON: &str = include_str!("../../../assets/config/weather.ron");
    const ATMOSPHERE_RON: &str = include_str!("../../../assets/config/atmosphere.ron");
    const VEHICLES_RON: &str = include_str!("../../../assets/config/vehicles.ron");
    const FAUNA_RON: &str = include_str!("../../../assets/config/fauna.ron");
    const DIG_RON: &str = include_str!("../../../assets/config/dig.ron");

    /// The shipped files are the defaults written out. If either drifts from
    /// the code, one of them is describing a different ocean, and this is the
    /// test that says which.
    #[test]
    fn shipped_files_parse_validate_and_agree_with_the_defaults() {
        let water: WaterSettings = ron::from_str(WATER_RON).unwrap();
        water.validate().unwrap();
        assert_eq!(water, WaterSettings::default());
        let weather: WeatherSettings = ron::from_str(WEATHER_RON).unwrap();
        weather.validate().unwrap();
        assert_eq!(weather, WeatherSettings::default());
        let atmosphere: pbd_core::atmosphere::AtmosphereSettings =
            ron::from_str(ATMOSPHERE_RON).unwrap();
        Validated::validate(&atmosphere).unwrap();
        assert_eq!(atmosphere, Default::default());
        let vehicles: pbd_core::vehicle::spec::VehicleSpecs = ron::from_str(VEHICLES_RON).unwrap();
        Validated::validate(&vehicles).unwrap();
        assert_eq!(vehicles, Default::default());
        let fauna: pbd_core::fauna::FaunaSettings = ron::from_str(FAUNA_RON).unwrap();
        Validated::validate(&fauna).unwrap();
        assert_eq!(fauna, Default::default());
        let dig: pbd_core::dig::DigSettings = ron::from_str(DIG_RON).unwrap();
        Validated::validate(&dig).unwrap();
        assert_eq!(dig, Default::default());
    }

    #[test]
    fn a_missing_field_inherits_and_a_present_zero_is_zero() {
        let partial: WaterSettings = ron::from_str("(fog_max: 0.0)").unwrap();
        assert_eq!(partial.fog_max, 0.0);
        assert_eq!(partial.sea, WaterSettings::default().sea);
    }

    #[test]
    fn unknown_fields_and_illegal_values_are_errors() {
        assert!(ron::from_str::<WaterSettings>("(fog_maximum: 0.5)").is_err());
        let bad: WaterSettings = ron::from_str("(fog_max: 1.0)").unwrap();
        assert!(bad.validate().is_err());
        let bad: WeatherSettings = ron::from_str("(wet_fade_tau_s: 0.0)").unwrap();
        assert!(bad.validate().is_err());
        let nan: WaterSettings = ron::from_str("(slope_max: NaN)").unwrap();
        assert!(nan.validate().is_err());
    }

    #[test]
    fn defaults_validate() {
        WaterSettings::default().validate().unwrap();
        WeatherSettings::default().validate().unwrap();
    }
}
