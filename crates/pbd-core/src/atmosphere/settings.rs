//! Every knob the atmosphere and the ocean have, with units.
//!
//! `AtmosphereSettings::default()` is the one source of the defaults;
//! `assets/config/atmosphere.ron` writes the same set out, and a field it omits
//! inherits the default here. None of these are physical constants. The owner's
//! word: "no real physics here". They were chosen and then measured by the
//! `climate` example, so the planet gets trade winds, a jet, desert belts and
//! storms on a world 4.8 km round with a 48-minute day.

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AtmosphereSettings {
    /// Subdivision level of the atmosphere's cells: 4 is 2,562 cells 363 m
    /// apart, 5 is 10,242 at 181 m, 6 is 40,962 at 91 m.
    pub level: u32,
    /// The fixed step, seconds of world time.
    pub dt_s: f32,
    /// How long a new world's weather is run before its first frame, seconds.
    pub spinup_s: f32,

    // --- Dynamics ---
    /// How many times faster than the planet's visible spin the air feels it
    /// turning. At 1 the Hadley cell reaches the pole (Held-Hou); see the
    /// `atmospheric-circulation` design.
    pub coriolis_scale: f32,
    /// Speed pressure disturbances spread at, m/s: the model's `sqrt(g H)`.
    pub gravity_wave_mps: f32,
    /// How fast pressure settles toward what the heat and the belts ask, s.
    pub pressure_relax_s: f32,
    /// How far a kelvin warmer than the planet's mean lowers pressure, m^2/s^2.
    pub thermal_pressure: f32,
    /// Strength of the three-cell pressure belts (lows at the thermal equator
    /// and near 60 deg, highs near 30 deg and at the poles), m^2/s^2. A single
    /// layer cannot make the belts itself: in the real atmosphere they are
    /// driven by the eddies of a deep, layered flow. So the belts are imposed,
    /// and everything else (land and sea, day and night, storms) moves the air
    /// on top of them.
    pub belt_pressure: f32,
    /// Share of the sun's latitude the belts follow through the year.
    pub belt_follow_sun: f32,
    /// Surface drag over land and over sea, as a damping time, s.
    pub drag_land_s: f32,
    pub drag_sea_s: f32,
    /// Share of the difference from the neighbours' mean that pressure and
    /// wind lose each second: numerical smoothing, which keeps a co-located
    /// grid from growing a checkerboard.
    pub smoothing: f32,
    /// Thermal wind at cloud height per unit of temperature gradient over the
    /// Coriolis parameter, m^2/s^2/K: the jet's strength.
    pub thermal_wind: f32,
    /// The jet's speed limit, m/s.
    pub jet_max_mps: f32,
    /// Share of the cloud-level wind (the rest is the surface wind) that
    /// carries the cloud.
    pub cloud_steering: f32,
    /// Share of the steering wind that actually carries CLOUD, 0..1. The winds
    /// are a real planet's and the cloud base is 300 m up, so cloud carried at
    /// the full steering wind crossed the whole sky in under half a minute
    /// (`calm-clouds`). Only cloud is slowed: vapour, heat, charge and the wind
    /// itself are carried as before, so the circulation is the same model.
    pub cloud_pace: f32,

    // --- Sun and heat ---
    /// Sunlight on a surface facing the sun, W/m^2.
    pub solar_wm2: f32,
    /// Share of sunlight a full cloud reflects.
    pub cloud_albedo: f32,
    /// Share of sunlight reflected by sea, by land, by snow and ice.
    pub ocean_albedo: f32,
    pub land_albedo: f32,
    pub snow_albedo: f32,
    /// Outgoing longwave: `olr_a + olr_b * T` W/m^2 with T in deg C (Budyko),
    /// less `cloud_greenhouse` under full cover.
    pub olr_a: f32,
    pub olr_b: f32,
    pub cloud_greenhouse: f32,
    /// Heat capacities, J/m^2/K. Small, because a day here is 48 minutes: land
    /// swings with the day, the sea with the year.
    pub land_heat_capacity: f32,
    pub ocean_heat_capacity: f32,
    /// How fast the air takes the ground's temperature, s.
    pub air_relax_s: f32,
    /// Sensible heat from ground to air, W/m^2/K.
    pub sensible_wm2k: f32,
    /// Heat spreading between neighbouring cells of ground and sea, per second:
    /// what carries the tropics' heat poleward where the model's winds do not.
    pub heat_spread: f32,
    /// Kelvin colder per metre of height, for snow and saturation: 0.08 makes
    /// a 150 m summit 12 K colder than the shore, which is the world's own
    /// snow line.
    pub lapse_k_per_m: f32,

    // --- Water ---
    /// Evaporation per second per kg/m^2 of saturation deficit.
    pub evaporation: f32,
    /// Wind speed that doubles evaporation, m/s.
    pub evaporation_wind_mps: f32,
    /// Heat evaporation takes from the ground, J per kg.
    pub evaporation_cooling: f32,
    /// A column's saturated water at 15 deg C, kg/m^2, and how fast that grows
    /// with temperature, per K (Clausius-Clapeyron, about 7%).
    pub saturation_kg: f32,
    pub saturation_per_k: f32,
    /// How far rising air lowers saturation, per m/s of ascent.
    pub lift_saturation: f32,
    /// Depth of the boundary layer that converging air rises out of, m.
    pub lift_depth_m: f32,
    /// Time condensation takes, s.
    pub condense_s: f32,
    /// Warming of the air per kg/m^2 condensed, K: the latent heat that
    /// drives a storm.
    pub latent_k_per_kg: f32,
    /// Cloud water beyond which a WARM cloud rains, kg/m^2, and how fast, s.
    /// Colder cloud rains out of less, in proportion to what the air can hold
    /// below 15 C (ice grows at the droplets' expense).
    pub rain_threshold_kg: f32,
    pub rain_s: f32,
    /// Ascent that halves the rain threshold, m/s: a vigorous updraft rains
    /// out of less water than a flat deck does, so storms and the afternoon's
    /// cumulus rain and a stratus deck mostly does not.
    pub convective_rain_mps: f32,
    /// How fast cloud in unsaturated air evaporates, s.
    pub cloud_evaporate_s: f32,
    /// Cloud water that starts to show, and that makes full cover, kg/m^2.
    pub cover_min_kg: f32,
    pub cover_full_kg: f32,
    /// Precipitation rate that counts as raining, kg/m^2/s.
    pub raining_rate: f32,
    /// Relative humidity above which air is partly cloudy without rising,
    /// 0..1: the critical humidity of Sundqvist's sub-grid cloud. That cover
    /// is a deck or a haze of small cumulus and rains nothing.
    pub humid_cover_rh: f32,
    /// The same over land, 0..1. Higher than the sea's: the decks humidity
    /// alone makes are mostly marine (stratocumulus under the subtropical
    /// highs), and with one threshold for both the land out-clouded the sea.
    pub humid_cover_rh_land: f32,
    /// Ascent per W/m^2 of sunlight the LAND absorbs past
    /// `convection_threshold_wm2`, m/s per W/m^2: heated ground lifts the air
    /// over it, so land clouds over by day and rains in the afternoon. The
    /// sea is left out, because its heat goes into the water. (Ground minus
    /// air was the first rule tried; measured over land it is negative at every
    /// hour, so it never fired. See the `cloud-detail` design.)
    pub convection_mps_per_wm2: f32,
    pub convection_threshold_wm2: f32,

    // --- Lightning ---
    /// Charge built per kg/m^2/s condensed per m/s of ascent.
    pub charge_rate: f32,
    /// How fast charge leaks away, s.
    pub charge_decay_s: f32,
    /// Charge at which a cell can strike, and the chance it does each step.
    pub strike_charge: f32,
    pub strike_chance: f32,
    /// What a strike drops: a cold pool of this pressure and chill, and this
    /// share of the cloud rained out at once.
    pub pool_pressure: f32,
    pub pool_k: f32,
    pub strike_rain_share: f32,
    /// How long a strike is kept for the renderer, s.
    pub strike_keep_s: f32,

    // --- Storms ---
    /// How often a storm is seeded where conditions favour one, per square
    /// kilometre per second. A single layer cannot grow its own cyclones (on
    /// Earth they come from instabilities of a deep, layered atmosphere and of
    /// convection far below a cell), so they are seeded, and the physics then
    /// feeds them over warm water and starves them over land.
    pub storm_rate: f32,
    /// Depth of a seeded low, m^2/s^2, and its radius, m.
    pub storm_depth: f32,
    pub storm_radius_m: f32,
    /// Sea-surface temperature above which a tropical storm can form, deg C.
    pub storm_sea_c: f32,
    /// Temperature gradient at which a frontal storm can form, K per km.
    pub storm_front_k_per_km: f32,
    /// Share by which the old weather field's drifting noise moves the
    /// condensation threshold: variability finer than a cell, which breaks a
    /// rain belt into clusters.
    pub mesoscale: f32,
    /// How often that noise is refreshed, s.
    pub mesoscale_every_s: f32,

    // --- Ocean ---
    /// Speed of the sea's surface waves in the model, m/s.
    pub ocean_wave_mps: f32,
    /// Current the wind drives, as a share of the wind speed.
    pub current_per_wind: f32,
    /// Drag on the current, s.
    pub ocean_drag_s: f32,
    /// How fast the sea-surface height settles, s.
    pub ocean_relax_s: f32,

    // --- Forcing (the weather slider) ---
    /// Radius round the player the slider brews a storm in, m.
    pub forcing_radius_m: f32,
    /// How fast the forcing drives the air toward a storm, s.
    pub forcing_s: f32,
    /// Cloud water the forcing brews at full, kg/m^2.
    pub forcing_cloud_kg: f32,
    /// The updraft the forcing drives at full, m/s: what makes its storm a
    /// storm, so it rains by the same convective rule any storm does.
    pub forcing_lift_mps: f32,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self {
            level: 5,
            dt_s: 1.0,
            spinup_s: 900.0,
            coriolis_scale: 4.0,
            gravity_wave_mps: 30.0,
            pressure_relax_s: 2000.0,
            thermal_pressure: 8.0,
            belt_pressure: 500.0,
            belt_follow_sun: 0.6,
            drag_land_s: 900.0,
            drag_sea_s: 2400.0,
            smoothing: 0.02,
            thermal_wind: 70.0,
            jet_max_mps: 45.0,
            cloud_steering: 0.7,
            cloud_pace: 0.2,
            solar_wm2: 1000.0,
            cloud_albedo: 0.6,
            ocean_albedo: 0.06,
            land_albedo: 0.25,
            snow_albedo: 0.7,
            olr_a: 203.0,
            olr_b: 2.09,
            cloud_greenhouse: 40.0,
            land_heat_capacity: 5.0e4,
            ocean_heat_capacity: 3.0e6,
            air_relax_s: 3000.0,
            sensible_wm2k: 15.0,
            heat_spread: 0.002,
            lapse_k_per_m: 0.08,
            evaporation: 3.0e-4,
            evaporation_wind_mps: 10.0,
            evaporation_cooling: 8.0e4,
            saturation_kg: 30.0,
            saturation_per_k: 0.068,
            lift_saturation: 0.35,
            lift_depth_m: 1000.0,
            condense_s: 60.0,
            latent_k_per_kg: 0.35,
            rain_threshold_kg: 4.0,
            rain_s: 150.0,
            convective_rain_mps: 0.5,
            cloud_evaporate_s: 120.0,
            cover_min_kg: 0.2,
            cover_full_kg: 0.8,
            raining_rate: 2.0e-4,
            humid_cover_rh: 0.45,
            humid_cover_rh_land: 0.7,
            convection_mps_per_wm2: 0.02,
            convection_threshold_wm2: 300.0,
            charge_rate: 0.3,
            charge_decay_s: 120.0,
            strike_charge: 1.0,
            strike_chance: 0.003,
            pool_pressure: 40.0,
            pool_k: 1.5,
            strike_rain_share: 0.5,
            strike_keep_s: 2.0,
            storm_rate: 8.0e-5,
            storm_depth: 250.0,
            storm_radius_m: 900.0,
            storm_sea_c: 24.0,
            storm_front_k_per_km: 6.0,
            mesoscale: 0.15,
            mesoscale_every_s: 30.0,
            ocean_wave_mps: 6.0,
            current_per_wind: 0.08,
            ocean_drag_s: 3000.0,
            ocean_relax_s: 20000.0,
            forcing_radius_m: 700.0,
            forcing_s: 5.0,
            forcing_cloud_kg: 1.2,
            forcing_lift_mps: 3.0,
        }
    }
}

impl AtmosphereSettings {
    /// What a legal set is. The step divides by every time here and assumes
    /// the rest in range; a file that breaks one is refused at load.
    pub fn validate(&self) -> Result<(), String> {
        if !(3..=6).contains(&self.level) {
            return Err("level must be 3..6".into());
        }
        let positive = [
            ("dt_s", self.dt_s),
            ("pressure_relax_s", self.pressure_relax_s),
            ("drag_land_s", self.drag_land_s),
            ("drag_sea_s", self.drag_sea_s),
            ("land_heat_capacity", self.land_heat_capacity),
            ("ocean_heat_capacity", self.ocean_heat_capacity),
            ("air_relax_s", self.air_relax_s),
            ("condense_s", self.condense_s),
            ("rain_s", self.rain_s),
            ("convective_rain_mps", self.convective_rain_mps),
            ("cloud_evaporate_s", self.cloud_evaporate_s),
            ("charge_decay_s", self.charge_decay_s),
            ("ocean_drag_s", self.ocean_drag_s),
            ("ocean_relax_s", self.ocean_relax_s),
            ("forcing_s", self.forcing_s),
            ("storm_radius_m", self.storm_radius_m),
            ("mesoscale_every_s", self.mesoscale_every_s),
            ("saturation_kg", self.saturation_kg),
            ("evaporation_wind_mps", self.evaporation_wind_mps),
            ("jet_max_mps", self.jet_max_mps),
            ("cloud_pace", self.cloud_pace),
        ];
        for (name, value) in positive {
            if !(value.is_finite() && value > 0.0) {
                return Err(format!("{name} must be positive"));
            }
        }
        let every = [
            self.spinup_s,
            self.coriolis_scale,
            self.gravity_wave_mps,
            self.thermal_pressure,
            self.belt_pressure,
            self.belt_follow_sun,
            self.smoothing,
            self.thermal_wind,
            self.cloud_steering,
            self.solar_wm2,
            self.cloud_albedo,
            self.ocean_albedo,
            self.land_albedo,
            self.snow_albedo,
            self.olr_a,
            self.olr_b,
            self.cloud_greenhouse,
            self.sensible_wm2k,
            self.heat_spread,
            self.lapse_k_per_m,
            self.evaporation,
            self.evaporation_cooling,
            self.saturation_per_k,
            self.lift_saturation,
            self.lift_depth_m,
            self.latent_k_per_kg,
            self.rain_threshold_kg,
            self.cover_min_kg,
            self.cover_full_kg,
            self.raining_rate,
            self.convection_mps_per_wm2,
            self.convection_threshold_wm2,
            self.charge_rate,
            self.strike_charge,
            self.strike_chance,
            self.pool_pressure,
            self.pool_k,
            self.strike_rain_share,
            self.strike_keep_s,
            self.ocean_wave_mps,
            self.current_per_wind,
            self.storm_rate,
            self.storm_depth,
            self.storm_front_k_per_km,
            self.mesoscale,
            self.forcing_radius_m,
            self.forcing_cloud_kg,
            self.forcing_lift_mps,
        ];
        if every.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err("every atmosphere setting must be finite and not negative".into());
        }
        for (name, value) in [
            ("cloud_albedo", self.cloud_albedo),
            ("ocean_albedo", self.ocean_albedo),
            ("land_albedo", self.land_albedo),
            ("snow_albedo", self.snow_albedo),
            ("belt_follow_sun", self.belt_follow_sun),
            ("cloud_steering", self.cloud_steering),
            ("cloud_pace", self.cloud_pace),
            ("strike_chance", self.strike_chance),
            ("strike_rain_share", self.strike_rain_share),
            ("smoothing", self.smoothing),
        ] {
            if value > 1.0 {
                return Err(format!("{name} must be at most 1"));
            }
        }
        for (name, value) in [
            ("humid_cover_rh", self.humid_cover_rh),
            ("humid_cover_rh_land", self.humid_cover_rh_land),
        ] {
            if !(0.0..1.0).contains(&value) {
                return Err(format!("{name} must be within 0..1"));
            }
        }
        if self.cover_full_kg <= self.cover_min_kg {
            return Err("cover_full_kg must be above cover_min_kg".into());
        }
        // Explicit gravity waves: the step must not outrun them across a cell
        // (level 6's 91 m cells are the tightest).
        let spacing = 1.2087 * 4_800.0 / 2f32.powi(self.level as i32);
        let fastest = self.gravity_wave_mps.max(self.ocean_wave_mps);
        if fastest * self.dt_s > 0.5 * spacing {
            return Err(format!(
                "dt_s {} is too long for waves at {fastest} m/s on {spacing:.0} m cells",
                self.dt_s
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_are_legal() {
        AtmosphereSettings::default().validate().unwrap();
    }

    #[test]
    fn a_step_that_outruns_the_waves_is_refused() {
        let s = AtmosphereSettings {
            dt_s: 10.0,
            level: 6,
            ..Default::default()
        };
        assert!(s.validate().is_err());
    }
}
