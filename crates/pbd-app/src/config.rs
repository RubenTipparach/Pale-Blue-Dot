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
    /// Vertex swell amplitude, metres.
    pub swell_amplitude_m: f32,
    /// Vertex swell spatial scale, cycles per metre (dimensionless multiplier
    /// on the three sine frequencies).
    pub swell_frequency: f32,
    /// Vertex swell speed multiplier.
    pub swell_speed: f32,
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
    /// Half-width of the straddling band around the sea surface, metres,
    /// added to the swell amplitude.
    pub partial_band_m: f32,
    /// Seconds the lens stays wet after surfacing.
    pub emerge_dry_s: f32,
    /// Radial scroll speed of the wave field on vertical water faces.
    pub flow_uv_speed_falling: f32,
}

impl Default for WaterSettings {
    fn default() -> Self {
        Self {
            time_scale: 0.75,
            depth_offset_m: 0.5,
            swell_amplitude_m: 0.5,
            swell_frequency: 2.0,
            swell_speed: 1.0,
            ripple_scale: 1.5,
            ripple_speed: 1.0,
            wave_steepness: 0.65,
            slope_max: 1.6,
            refract_amount: 0.04,
            refract_max_uv: 0.03,
            max_path_m: 200.0,
            absorption_per_m: [0.60, 0.20, 0.10],
            deep_color: [0.02, 0.10, 0.22],
            sky_horizon_color: [0.85, 0.92, 0.98],
            sky_zenith_color: [0.35, 0.55, 0.85],
            sky_horizon_strength: 0.5,
            foam_crest_lo: 0.35,
            foam_crest_hi: 0.60,
            foam_crest_weight: 0.55,
            foam_slope_lo: 0.50,
            foam_slope_hi: 1.20,
            foam_slope_weight: 0.26,
            foam_intensity: 0.10,
            foam_color: [0.95, 0.97, 1.00],
            specular_intensity: 0.30,
            specular_power: 140.0,
            sun_tint: [1.35, 1.25, 1.10],
            night_floor: 0.18,
            fog_max: 0.82,
            rain_ripple_scale: 3.0,
            rain_ripple_strength: 6.0,
            underwater_distortion: 0.0015,
            wet_blur: 0.02,
            partial_band_m: 0.8,
            emerge_dry_s: 2.6,
            flow_uv_speed_falling: 1.0,
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
                s.swell_amplitude_m,
                s.swell_frequency,
                s.swell_speed,
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
            ],
        )?;
        non_negative("absorption_per_m", &s.absorption_per_m)?;
        non_negative("deep_color", &s.deep_color)?;
        non_negative("sky_horizon_color", &s.sky_horizon_color)?;
        non_negative("sky_zenith_color", &s.sky_zenith_color)?;
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
    /// Streak fall speed, metres per second.
    pub rain_fall_mps: f32,
    /// Streak half-width, metres.
    pub rain_width_m: f32,
    /// Streak length, metres.
    pub rain_streak_m: f32,
    /// Streak colour, linear RGB.
    pub rain_color: [f32; 3],
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
    /// Sky-light sheen on the tilt of the wet normal.
    pub rain_sky_sheen: f32,
    /// Sun glint exponent and strength.
    pub rain_glint_power: f32,
    pub rain_glint_strength: f32,
}

impl Default for WeatherSettings {
    fn default() -> Self {
        Self {
            wet_fade_tau_s: 1.6,
            rain_fall_mps: 70.0,
            rain_width_m: 0.012,
            rain_streak_m: 1.5,
            rain_color: [0.52, 0.62, 0.90],
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
            rain_sky_sheen: 0.8,
            rain_glint_power: 24.0,
            rain_glint_strength: 0.5,
        }
    }
}

impl Validated for WeatherSettings {
    fn validate(&self) -> Result<(), String> {
        let s = self;
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
                s.rain_sky_sheen,
                s.rain_glint_power,
                s.rain_glint_strength,
            ],
        )?;
        finite("rain_flow_strength", &[s.rain_flow_strength])?;
        non_negative("rain_color", &s.rain_color)?;
        for (name, value) in [
            ("rain_alpha_near", s.rain_alpha_near),
            ("rain_alpha_far", s.rain_alpha_far),
            ("rain_wet_darken", s.rain_wet_darken),
        ] {
            unit(name, value)?;
        }
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

/// Loads every config file once at startup. Inserted before any plugin that
/// reads them, so a system can take `Res<WaterSettings>` unconditionally.
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load::<WaterSettings>("water"))
            .insert_resource(load::<WeatherSettings>("weather"))
            .add_plugins((
                bevy::render::extract_resource::ExtractResourcePlugin::<WaterSettings>::default(),
                bevy::render::extract_resource::ExtractResourcePlugin::<WeatherSettings>::default(),
            ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATER_RON: &str = include_str!("../../../assets/config/water.ron");
    const WEATHER_RON: &str = include_str!("../../../assets/config/weather.ron");

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
    }

    #[test]
    fn a_missing_field_inherits_and_a_present_zero_is_zero() {
        let partial: WaterSettings = ron::from_str("(fog_max: 0.0)").unwrap();
        assert_eq!(partial.fog_max, 0.0);
        assert_eq!(
            partial.swell_amplitude_m,
            WaterSettings::default().swell_amplitude_m
        );
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
