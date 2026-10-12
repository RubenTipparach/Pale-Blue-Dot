//! The wind at a point: the atmosphere's wind at the place, sheared with
//! height above the surface, with gusts in it (`vehicles` change, design
//! section 3).
//!
//! The atmosphere's cells are 181 m apart and step once a second, so its wind
//! is the mean a place feels over minutes. What a wing or a sail feels is that
//! mean, weaker near the water, with gusts riding along in it. The gusts are
//! frozen turbulence carried downwind: a pure function of where, when and how
//! stormy, so every client and every replay computes the same gust.

use glam::Vec3;
use serde::Deserialize;

/// The gusts' knobs, in `weather.ron` under `gusts`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GustSettings {
    /// Gust strength in dry weather, as a share of the mean wind.
    pub gustiness: f32,
    /// How much stronger per mm/h of rain: a squall gusts.
    pub gustiness_per_mmh: f32,
    /// Roughness length over the sea and over land, m.
    pub roughness_sea_m: f32,
    pub roughness_land_m: f32,
    /// The height profile's cap, as a share of the 10 m wind.
    pub shear_max: f32,
    /// The downdraft under rain, m/s per sqrt(mm/h).
    pub downdraft: f32,
    /// Where the wind turns from the surface wind to the cloud-level wind:
    /// begins, ends, metres above the surface.
    pub aloft_m: [f32; 2],
}

impl Default for GustSettings {
    fn default() -> Self {
        Self {
            gustiness: 0.25,
            gustiness_per_mmh: 1.0 / 80.0,
            roughness_sea_m: 0.0005,
            roughness_land_m: 0.03,
            shear_max: 1.4,
            downdraft: 0.3,
            aloft_m: [150.0, 300.0],
        }
    }
}

impl GustSettings {
    pub fn validate(&self) -> Result<(), String> {
        let v = [
            self.gustiness,
            self.gustiness_per_mmh,
            self.roughness_sea_m,
            self.roughness_land_m,
            self.shear_max,
            self.downdraft,
            self.aloft_m[0],
            self.aloft_m[1],
        ];
        if v.iter().any(|x| !x.is_finite() || *x < 0.0) {
            return Err("gusts: every value must be finite and not negative".into());
        }
        if !(self.roughness_sea_m > 0.0 && self.roughness_land_m > 0.0) {
            return Err("gusts: roughness lengths must be positive".into());
        }
        if self.roughness_sea_m >= 10.0 || self.roughness_land_m >= 10.0 {
            return Err("gusts: a roughness length must be well under 10 m".into());
        }
        if self.aloft_m[0] >= self.aloft_m[1] {
            return Err("gusts.aloft_m must rise".into());
        }
        Ok(())
    }
}

/// The weather at a place, as the wind needs it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AirHere {
    /// The atmosphere's surface wind there, m/s, tangent: the wind at 10 m.
    pub wind: Vec3,
    /// Its wind at cloud height, m/s.
    pub upper: Vec3,
    /// Rain, mm/h.
    pub rain_mmh: f32,
    /// Whether the ground there is land, which is rougher than the sea.
    pub over_land: bool,
}

/// The log-law profile: the share of the 10 m wind at `height` metres.
pub fn shear(settings: &GustSettings, height: f32, over_land: bool) -> f32 {
    let z0 = if over_land {
        settings.roughness_land_m
    } else {
        settings.roughness_sea_m
    };
    let h = height.max(z0 * 1.5);
    ((h / z0).ln() / (10.0 / z0).ln()).clamp(0.0, settings.shear_max)
}

/// The wind at body-local `point`, `height` metres above the ground or sea
/// under it, at world time `seconds`.
pub fn wind_at(
    settings: &GustSettings,
    air: &AirHere,
    point: Vec3,
    height: f32,
    seconds: f64,
) -> Vec3 {
    let up = point.normalize_or(Vec3::Y);
    let [lo, hi] = settings.aloft_m;
    let aloft = smoothstep(lo, hi, height);
    let near = air.wind * shear(settings, height, air.over_land);
    let mean = near * (1.0 - aloft) + air.upper * aloft;
    let speed = mean.length();
    let rain = air.rain_mmh.max(0.0);
    let gust_share = (settings.gustiness * (1.0 + rain * settings.gustiness_per_mmh)).min(1.2);
    let (along, g1, g2, g3) = if speed > 0.05 {
        let along = mean / speed;
        let across = up.cross(along);
        let xi = point.dot(along) as f64;
        let eta = point.dot(across) as f64;
        // The gust pattern rides downwind at the mean speed.
        let t = seconds - xi / (speed as f64).max(1.5);
        let s = |f: f64, phase: f64, e: f64| (t * f + phase + eta * e).sin() as f32;
        let g1 = s(0.71, 0.0, 0.013) * 0.55 + s(1.93, 1.1, 0.041) * 0.3 + s(4.7, 2.3, 0.09) * 0.15;
        let g2 = s(0.53, 2.0, 0.017) * 0.55 + s(2.61, 0.3, 0.05) * 0.3 + s(5.3, 0.8, 0.0) * 0.15;
        let g3 = s(1.37, 0.7, 0.02) * 0.6 + s(3.9, 1.9, 0.0) * 0.4;
        (along, g1, g2, g3)
    } else {
        (Vec3::ZERO, 0.0, 0.0, 0.0)
    };
    let across = up.cross(along);
    let gust = along * (speed * gust_share * g1)
        + across * (speed * gust_share * 0.6 * g2)
        + up * (speed * gust_share * 0.35 * g3 * height / (height + 8.0));
    // Rain-cooled air sinks: the microburst a hovering craft fights.
    let downdraft = settings.downdraft * rain.sqrt() * (1.0 + 0.4 * g1);
    mean + gust - up * downdraft
}

fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f32 = 4800.0;

    fn air() -> AirHere {
        AirHere {
            wind: Vec3::new(8.0, 0.0, 0.0),
            upper: Vec3::new(30.0, 0.0, 0.0),
            rain_mmh: 0.0,
            over_land: false,
        }
    }

    #[test]
    fn equal_inputs_give_equal_gusts() {
        let s = GustSettings::default();
        let p = Vec3::new(0.0, R, 0.0) + Vec3::new(13.0, 0.0, -7.0);
        let a = wind_at(&s, &air(), p, 12.0, 98_765.432_1);
        let b = wind_at(&s, &air(), p, 12.0, 98_765.432_1);
        assert_eq!(a.to_array(), b.to_array());
    }

    #[test]
    fn gusts_average_out_to_the_sheared_wind() {
        let s = GustSettings::default();
        let p = Vec3::new(0.0, R, 0.0);
        let steps = 600 * 20;
        let mut sum = Vec3::ZERO;
        for i in 0..steps {
            sum += wind_at(&s, &air(), p, 10.0, 1000.0 + i as f64 * 0.05);
        }
        let mean = sum / steps as f32;
        let want = air().wind;
        assert!(
            (mean - want).length() < want.length() * 0.02,
            "mean {mean} vs {want}"
        );
        // And they are there: the wind is not the mean at every instant.
        let spread = (0..200)
            .map(|i| (wind_at(&s, &air(), p, 10.0, i as f64 * 0.7) - want).length())
            .fold(0.0f32, f32::max);
        assert!(spread > 1.0);
    }

    #[test]
    fn the_wind_is_weaker_near_the_water() {
        let s = GustSettings::default();
        assert!((shear(&s, 10.0, false) - 1.0).abs() < 1e-6);
        let low = shear(&s, 1.0, false);
        let z0 = s.roughness_sea_m;
        let want = (1.0 / z0).ln() / (10.0 / z0).ln();
        assert!((low - want).abs() < 1e-6 && low < 1.0);
        assert!(shear(&s, 1.0, true) < low, "land is rougher than the sea");
    }

    #[test]
    fn rain_sinks_the_air() {
        let s = GustSettings::default();
        let mut wet = air();
        wet.rain_mmh = 90.0;
        let p = Vec3::new(0.0, R, 0.0);
        let steps = 12_000;
        let sink: f32 = (0..steps)
            .map(|i| -wind_at(&s, &wet, p, 20.0, i as f64 * 0.05).dot(Vec3::Y))
            .sum::<f32>()
            / steps as f32;
        assert!((sink - 2.85).abs() < 0.2, "sinks at {sink} m/s");
    }

    #[test]
    fn high_up_it_is_the_cloud_level_wind() {
        let s = GustSettings {
            gustiness: 0.0,
            ..Default::default()
        };
        let w = wind_at(&s, &air(), Vec3::new(0.0, R + 400.0, 0.0), 400.0, 0.0);
        assert!((w - air().upper).length() < 1e-3);
    }
}
