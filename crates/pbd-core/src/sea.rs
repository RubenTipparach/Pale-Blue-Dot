//! The sea's surface: one function that the water shader draws and the hulls
//! float on (`vehicles` change, design section 2).
//!
//! The surface is a sum of plane waves in the planet's own 3D frame, evaluated
//! at a point on the sea sphere. Twelve fixed directions and five fixed
//! wavelengths, plus a long swell in every direction, make a table whose
//! wavenumbers and frequencies never change: the wind sets only the
//! amplitudes. A change of wind therefore fades waves in and out, and never
//! moves a crest by more than the change of its own height.
//!
//! A plane wave on a sphere degenerates where its direction points along the
//! local normal: the crests become rings. So a direction takes part at a point
//! only where most of it lies along the surface, and its share fades out as it
//! turns toward the normal. With twelve directions spread over the sphere,
//! every point has several that lie flat.
//!
//! The shader cannot read Rust, so what it draws is the SAME table uploaded:
//! `SeaTable::gpu` writes the directions, the bands and each component's phase
//! at the current time, and `water.wgsl`'s `sea_surface` is this module's
//! `LocalSea` written out in WGSL. `pbd-app`'s parity test runs the shipped
//! WGSL on a GPU adapter and compares it with `LocalSea::at`.

use glam::{DVec3, Vec3, Vec4};
use serde::Deserialize;
use std::f64::consts::TAU;

/// Fixed wave directions, spread over the sphere.
pub const DIRECTIONS: usize = 12;
/// Wind-sea wavelengths.
pub const BANDS: usize = 5;
/// Every component: each wind-sea band and the swell, in every direction. The
/// swell is band index `BANDS`.
pub const COMPONENTS: usize = DIRECTIONS * (BANDS + 1);

/// The sea's knobs, in `water.ron` under `sea`. Units in the field docs.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SeaSettings {
    /// The wind-sea wavelengths, metres, shortest first. The shortest has to
    /// be at least `filter_spacings` times the water cap's vertex spacing,
    /// 1.64 m, or the physics floats on a wave the cap can never draw.
    pub wavelengths_m: [f32; BANDS],
    /// The swell that is always there, from distant weather: wavelength, m.
    pub swell_wavelength_m: f32,
    /// And its significant height, m.
    pub swell_height_m: f32,
    /// The largest `a k` any single component may reach.
    pub max_steepness: f32,
    /// The largest significant height any wind raises, m. `None` is no limit;
    /// a fetch limit for a planet whose storms are small.
    pub max_hs_m: Option<f32>,
    /// The waves may be at most this share of the local depth: shoaling.
    pub depth_limit: f32,
    /// A vertex draws a component only when its wavelength is this many times
    /// the vertex's own spacing, fading in over the next factor of 1.6.
    pub filter_spacings: f32,
    /// A direction takes part where the share of it lying along the surface
    /// rises through this range.
    pub tangent_fade: [f32; 2],
}

impl Default for SeaSettings {
    fn default() -> Self {
        Self {
            wavelengths_m: [7.0, 14.0, 28.0, 56.0, 112.0],
            swell_wavelength_m: 70.0,
            swell_height_m: 0.34,
            max_steepness: 0.1,
            max_hs_m: None,
            depth_limit: 0.6,
            filter_spacings: 2.5,
            tangent_fade: [0.75, 0.9],
        }
    }
}

/// The water cap's vertex spacing, m: a hexagon's circumradius at the
/// Tenebris tile width, 2.833 / sqrt(3).
pub const CAP_SPACING_M: f32 = 1.636;

impl SeaSettings {
    pub fn validate(&self) -> Result<(), String> {
        let finite = self
            .wavelengths_m
            .iter()
            .chain([
                &self.swell_wavelength_m,
                &self.swell_height_m,
                &self.max_steepness,
                &self.depth_limit,
                &self.filter_spacings,
            ])
            .chain(&self.tangent_fade)
            .all(|v| v.is_finite());
        if !finite {
            return Err("sea: every value must be finite".into());
        }
        if !self.wavelengths_m.windows(2).all(|w| w[0] < w[1]) || self.wavelengths_m[0] <= 0.0 {
            return Err("sea.wavelengths_m must be positive and rising".into());
        }
        let shortest = self.filter_spacings * CAP_SPACING_M;
        if self.wavelengths_m[0] < shortest {
            return Err(format!(
                "sea.wavelengths_m starts at {} m, shorter than the {shortest} m the water cap can draw",
                self.wavelengths_m[0]
            ));
        }
        if self.swell_wavelength_m <= 0.0 || self.swell_height_m < 0.0 {
            return Err("sea.swell must have a positive wavelength and no negative height".into());
        }
        if !(self.max_steepness > 0.0 && self.max_steepness < 0.3) {
            return Err("sea.max_steepness must be within 0..0.3".into());
        }
        if self.max_hs_m.is_some_and(|h| !(h.is_finite() && h > 0.0)) {
            return Err("sea.max_hs_m must be positive".into());
        }
        if !(self.depth_limit > 0.0 && self.filter_spacings >= 2.0) {
            return Err(
                "sea.depth_limit must be positive and filter_spacings at least 2 (Nyquist)".into(),
            );
        }
        let [lo, hi] = self.tangent_fade;
        if !(0.0 < lo && lo < hi && hi <= 1.0) {
            return Err("sea.tangent_fade must rise within 0..=1".into());
        }
        Ok(())
    }
}

/// One wavelength's constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub wavelength: f32,
    /// Wavenumber, rad/m.
    pub k: f32,
    /// Deep-water angular frequency, rad/s: sqrt(g k).
    pub omega: f32,
}

/// The constants of a planet's sea: fixed for its gravity and settings.
#[derive(Clone, Debug, PartialEq)]
pub struct SeaTable {
    pub settings: SeaSettings,
    pub gravity: f32,
    pub directions: [Vec3; DIRECTIONS],
    /// The wind-sea bands, then the swell.
    pub bands: [Band; BANDS + 1],
    /// Each component's phase at time nought, rad; index `band * DIRECTIONS +
    /// direction`.
    pub phase0: [f32; COMPONENTS],
}

/// The sea state at a place: how much of each band there is, and which way
/// the wind sea runs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SeaState {
    /// Each band's amplitude summed over directions (root of the sum of
    /// squares), m; the swell last.
    pub amplitude: [f32; BANDS + 1],
    /// Where the wind sea runs: a unit tangent, or zero for no preference.
    pub heading: Vec3,
}

impl SeaState {
    /// Significant wave height of this state, m.
    pub fn significant_height(&self) -> f32 {
        let m0: f32 = self.amplitude.iter().map(|a| a * a / 2.0).sum();
        4.0 * m0.sqrt()
    }
}

/// Twelve directions on a Fibonacci spiral: fixed, so the table is a constant
/// of the planet.
fn fibonacci_directions() -> [Vec3; DIRECTIONS] {
    let golden = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    std::array::from_fn(|i| {
        let y = 1.0 - (i as f32 + 0.5) * 2.0 / DIRECTIONS as f32;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let a = i as f32 * golden;
        Vec3::new(r * a.cos(), y, r * a.sin())
    })
}

/// A fixed, well-spread phase per component: a hash, not a random draw, so
/// every run and every client has the same sea.
fn phase_of(index: usize) -> f32 {
    let mut x = (index as u32).wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x as f32 / u32::MAX as f32) * std::f32::consts::TAU
}

impl SeaTable {
    pub fn new(settings: SeaSettings, gravity: f32) -> Self {
        let band = |wavelength: f32| {
            let k = std::f32::consts::TAU / wavelength;
            Band {
                wavelength,
                k,
                omega: (gravity * k).sqrt(),
            }
        };
        let mut bands = [band(1.0); BANDS + 1];
        for (b, w) in bands.iter_mut().zip(settings.wavelengths_m) {
            *b = band(w);
        }
        bands[BANDS] = band(settings.swell_wavelength_m);
        Self {
            settings,
            gravity,
            directions: fibonacci_directions(),
            bands,
            phase0: std::array::from_fn(phase_of),
        }
    }

    /// The sea state a wind of `speed` m/s blowing along `wind` raises once it
    /// has had time to: a Pierson-Moskowitz spectrum sampled at the bands and
    /// scaled to its significant height, 0.21 U^2 / g.
    pub fn state(&self, speed: f32, wind: Vec3) -> SeaState {
        let g = self.gravity;
        let u = speed.max(0.0);
        let mut amplitude = [0.0; BANDS + 1];
        if u > 0.05 {
            let peak = 0.877 * g / u;
            // Bins a factor of two apart in wavelength are a factor sqrt(2)
            // apart in frequency: each covers 2^(1/4) - 2^(-1/4) of its own.
            let width = 2f32.powf(0.25) - 2f32.powf(-0.25);
            let mut m0 = 0.0;
            for (a, b) in amplitude.iter_mut().zip(&self.bands[..BANDS]) {
                let s =
                    0.0081 * g * g * b.omega.powi(-5) * (-1.25 * (peak / b.omega).powi(4)).exp();
                *a = (2.0 * s * b.omega * width).sqrt();
                m0 += *a * *a / 2.0;
            }
            let mut target = 0.21 * u * u / g;
            if let Some(cap) = self.settings.max_hs_m {
                target = target.min(cap);
            }
            let binned = 4.0 * m0.sqrt();
            // Five bins cannot hold the spectrum's shape, so they are scaled
            // to its height; but not when the peak is below the shortest band,
            // where the waves are too short to exist here at all.
            if binned > target * 0.3 {
                let scale = target / binned;
                amplitude[..BANDS].iter_mut().for_each(|a| *a *= scale);
            }
        }
        amplitude[BANDS] = self.settings.swell_height_m / (2.0 * std::f32::consts::SQRT_2);
        SeaState {
            amplitude,
            heading: wind.normalize_or_zero(),
        }
    }

    /// The table as the shader reads it, at world time `seconds`: the
    /// directions, each band as (wavelength, k, omega, amplitude), and each
    /// component's phase with the time folded in (in f64, then wrapped) so the
    /// GPU never multiplies a large time in f32.
    pub fn gpu(&self, state: &SeaState, seconds: f64) -> SeaGpu {
        let mut phase = [Vec4::ZERO; COMPONENTS / 4];
        for i in 0..COMPONENTS {
            phase[i / 4][i % 4] = self.phase_at(i, seconds);
        }
        SeaGpu {
            directions: self.directions.map(|d| d.extend(0.0)),
            bands: std::array::from_fn(|b| {
                let band = self.bands[b];
                Vec4::new(band.wavelength, band.k, band.omega, state.amplitude[b])
            }),
            phase,
            heading: state.heading.extend(self.settings.max_steepness),
            limits: Vec4::new(
                self.settings.tangent_fade[0],
                self.settings.tangent_fade[1],
                self.settings.filter_spacings,
                self.settings.depth_limit,
            ),
        }
    }

    /// A component's phase at a time, wrapped to 0..2pi.
    fn phase_at(&self, index: usize, seconds: f64) -> f32 {
        let omega = self.bands[index / DIRECTIONS].omega as f64;
        (self.phase0[index] as f64 - omega * seconds).rem_euclid(TAU) as f32
    }

    /// The sea about one place: which components take part there and how
    /// high each is. `normal` is the unit direction of the place, `depth` the
    /// water's depth there (m; use a large number offshore).
    pub fn local(&self, state: &SeaState, normal: Vec3, depth: f32, seconds: f64) -> LocalSea {
        let [lo, hi] = self.settings.tangent_fade;
        let mut fade = [0.0f32; DIRECTIONS];
        let mut along = [0.0f32; DIRECTIONS];
        for (i, d) in self.directions.iter().enumerate() {
            let tangent = *d - normal * normal.dot(*d);
            let share = tangent.length();
            fade[i] = smoothstep(lo, hi, share);
            if state.heading != Vec3::ZERO && share > 1e-4 {
                let c = (tangent / share).dot(state.heading);
                along[i] = if c > 0.0 { c * c } else { 0.0 };
            } else {
                along[i] = 1.0;
            }
        }
        let weight = |i: usize| fade[i] * along[i];
        let mut wind_norm: f32 = (0..DIRECTIONS).map(weight).sum();
        let wind_weight: [f32; DIRECTIONS] = if wind_norm > 1e-4 {
            std::array::from_fn(weight)
        } else {
            wind_norm = fade.iter().sum();
            fade
        };
        let swell_norm: f32 = fade.iter().sum::<f32>().max(1e-4);
        let wind_norm = wind_norm.max(1e-4);
        let mut out = LocalSea {
            normal,
            count: 0,
            components: [Component::default(); COMPONENTS],
        };
        let mut total = 0.0;
        for b in 0..=BANDS {
            let band = self.bands[b];
            let shoal = smoothstep(0.0, band.wavelength * 0.5, depth);
            for i in 0..DIRECTIONS {
                let share = if b == BANDS {
                    fade[i] / swell_norm
                } else {
                    wind_weight[i] / wind_norm
                };
                let a = (state.amplitude[b] * share.sqrt() * shoal)
                    .min(self.settings.max_steepness / band.k);
                if a < 1e-4 {
                    continue;
                }
                let d = self.directions[i];
                let tangent = (d - normal * normal.dot(d)).normalize_or_zero();
                out.components[out.count] = Component {
                    amplitude: a,
                    k: d * band.k,
                    omega: band.omega,
                    phase: self.phase_at(b * DIRECTIONS + i, seconds),
                    travel: tangent,
                    wavenumber: band.k,
                };
                out.count += 1;
                total += a;
            }
        }
        // Never higher than the water is deep.
        let cap = self.settings.depth_limit * depth.max(0.0);
        if total > cap {
            let scale = cap / total;
            out.components[..out.count]
                .iter_mut()
                .for_each(|c| c.amplitude *= scale);
        }
        out
    }
}

/// One plane wave, ready to evaluate.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Component {
    pub amplitude: f32,
    /// The wave vector in the planet's frame, rad/m.
    pub k: Vec3,
    pub omega: f32,
    /// Phase at the time the local sea was taken, rad.
    pub phase: f32,
    /// Which way its crests run along the surface: unit tangent.
    pub travel: Vec3,
    pub wavenumber: f32,
}

/// The sea about one place at one time.
#[derive(Clone, Debug)]
pub struct LocalSea {
    pub normal: Vec3,
    pub count: usize,
    pub components: [Component; COMPONENTS],
}

/// The sea at a point.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SeaPoint {
    /// Height of the surface above the sea radius, m.
    pub height: f32,
    /// The water's velocity at the point, m/s, in the planet's frame: the
    /// orbital motion along the surface and along the normal.
    pub velocity: Vec3,
}

impl LocalSea {
    /// The surface over body-local point `p`, and the water's velocity `depth`
    /// metres under that surface. `p` is projected to the sea radius first, as
    /// the shader does.
    pub fn at(&self, p: Vec3, sea_radius: f32, depth: f32) -> SeaPoint {
        let on_sea = p.normalize_or_zero() * sea_radius;
        let mut out = SeaPoint::default();
        for c in &self.components[..self.count] {
            let (s, co) = (c.k.dot(on_sea) + c.phase).sin_cos();
            out.height += c.amplitude * s;
            let decay = if depth > 0.0 {
                (-c.wavenumber * depth).exp()
            } else {
                1.0
            };
            let aw = c.amplitude * c.omega * decay;
            out.velocity += c.travel * (aw * s) - self.normal * (aw * co);
        }
        out
    }

    /// Height only: the loop the hull's cells run.
    pub fn height(&self, p: Vec3, sea_radius: f32) -> f32 {
        let on_sea = p.normalize_or_zero() * sea_radius;
        self.components[..self.count]
            .iter()
            .map(|c| c.amplitude * (c.k.dot(on_sea) + c.phase).sin())
            .sum()
    }

    /// The highest the surface can stand above the mean here, m.
    pub fn reach(&self) -> f32 {
        self.components[..self.count]
            .iter()
            .map(|c| c.amplitude)
            .sum()
    }
}

/// The table as the shader's uniform lays it out (`SeaView` in
/// `water.wgsl`): the directions; each band as (wavelength, k, omega,
/// amplitude); every component's phase now, four to a lane; the wind sea's
/// heading with the steepness cap; and (fade lo, fade hi, filter spacings,
/// depth limit).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SeaGpu {
    pub directions: [Vec4; DIRECTIONS],
    pub bands: [Vec4; BANDS + 1],
    pub phase: [Vec4; COMPONENTS / 4],
    pub heading: Vec4,
    pub limits: Vec4,
}

fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Double-precision convenience for the vehicle code.
pub fn at_point(local: &LocalSea, p: DVec3, sea_radius: f64, depth: f64) -> SeaPoint {
    local.at(p.as_vec3(), sea_radius as f32, depth as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: f32 = 25.0;
    const R: f32 = 4799.5;

    fn table() -> SeaTable {
        SeaTable::new(SeaSettings::default(), G)
    }

    #[test]
    fn the_shipped_defaults_are_valid_and_drawable() {
        SeaSettings::default().validate().unwrap();
        let mut short = SeaSettings::default();
        short.wavelengths_m[0] = 2.0;
        assert!(
            short.validate().is_err(),
            "a 2 m wave is an alias on the cap"
        );
    }

    #[test]
    fn significant_height_follows_the_wind_and_gravity() {
        let t = table();
        for u in [6.0, 11.0, 16.0, 24.0] {
            let mut s = t.state(u, Vec3::X);
            s.amplitude[BANDS] = 0.0;
            let want = 0.21 * u * u / G;
            let got = s.significant_height();
            assert!((got - want).abs() < want * 0.05, "U {u}: {got} vs {want}");
        }
        let earth = SeaTable::new(SeaSettings::default(), 9.81).state(11.0, Vec3::X);
        assert!(earth.significant_height() > t.state(11.0, Vec3::X).significant_height() * 2.0);
    }

    #[test]
    fn a_cap_holds_the_storm_sea() {
        let s = SeaSettings {
            max_hs_m: Some(1.5),
            ..Default::default()
        };
        let mut state = SeaTable::new(s, G).state(24.0, Vec3::X);
        state.amplitude[BANDS] = 0.0;
        assert!(state.significant_height() <= 1.5 * 1.01);
    }

    /// Every point on the planet has directions that lie along it, and the
    /// local sea carries the whole state's energy there.
    #[test]
    fn every_place_has_its_full_sea() {
        let t = table();
        let state = t.state(11.0, Vec3::ZERO);
        for i in 0..200 {
            let y = 1.0 - (i as f32 + 0.5) / 100.0;
            let a = i as f32 * 2.4;
            let r = (1.0 - y * y).sqrt();
            let n = Vec3::new(r * a.cos(), y, r * a.sin());
            let local = t.local(&state, n, 1000.0, 0.0);
            let m0: f32 = local.components[..local.count]
                .iter()
                .map(|c| c.amplitude * c.amplitude / 2.0)
                .sum();
            let hs = 4.0 * m0.sqrt();
            let want = state.significant_height();
            assert!((hs - want).abs() < want * 0.08, "at {n}: {hs} vs {want}");
        }
    }

    /// Measured over a patch of sea, the height varies as much as the state
    /// says: the table is the sea the formula describes, not a shadow of it.
    #[test]
    fn the_surface_has_the_states_height() {
        let t = table();
        let n = Vec3::new(0.3, 0.9, 0.1).normalize();
        let heading = n.cross(Vec3::Z).normalize();
        let state = t.state(11.0, heading);
        let local = t.local(&state, n, 1000.0, 0.0);
        let (e1, e2) = (heading, n.cross(heading));
        let mut sum = 0.0f64;
        let mut sq = 0.0f64;
        let mut count = 0;
        for x in 0..60 {
            for z in 0..60 {
                let p = n * R + e1 * (x as f32 * 7.3) + e2 * (z as f32 * 6.1);
                let h = local.height(p, R) as f64;
                sum += h;
                sq += h * h;
                count += 1;
            }
        }
        let mean = sum / count as f64;
        let hs = 4.0 * (sq / count as f64 - mean * mean).sqrt();
        let want = state.significant_height() as f64;
        assert!(mean.abs() < 0.1, "mean {mean}");
        assert!((hs - want).abs() < want * 0.25, "measured {hs} vs {want}");
    }

    #[test]
    fn a_change_of_wind_never_moves_a_crest_more_than_its_height_changes() {
        let t = table();
        let n = Vec3::Y;
        let before = t.state(8.0, Vec3::X);
        let after = t.state(8.5, Vec3::new(1.0, 0.0, 0.1).normalize());
        for i in 0..50 {
            let p = n * R + Vec3::new(i as f32 * 13.7, 0.0, i as f32 * -5.3);
            let a = t.local(&before, n, 1000.0, 100.0).height(p, R);
            let b = t.local(&after, n, 1000.0, 100.0).height(p, R);
            let change = (after.significant_height() - before.significant_height()).abs()
                + 0.1 * before.significant_height();
            assert!((a - b).abs() <= change, "{a} -> {b} at {p}");
        }
    }

    /// The vertical velocity is the time derivative of the height, so a hull
    /// rides the wave rather than being pushed through it.
    #[test]
    fn the_water_moves_as_the_surface_does() {
        let t = table();
        let n = Vec3::new(0.0, 0.8, 0.6).normalize();
        let state = t.state(12.0, n.cross(Vec3::X).normalize());
        let p = n * R + Vec3::X * 17.0;
        let dt = 1e-3;
        let h0 = t.local(&state, n, 1000.0, 50.0).at(p, R, 0.0);
        let h1 = t.local(&state, n, 1000.0, 50.0 + dt).at(p, R, 0.0);
        let rate = (h1.height - h0.height) / dt as f32;
        let normal_velocity = h0.velocity.dot(n);
        assert!(
            (rate - normal_velocity).abs() < 0.02 + 0.02 * rate.abs(),
            "dh/dt {rate} vs w {normal_velocity}"
        );
    }

    #[test]
    fn waves_shoal_to_nothing_at_the_shore() {
        let t = table();
        let state = t.state(16.0, Vec3::X);
        let shallow = t.local(&state, Vec3::Y, 1.0, 0.0);
        assert!(shallow.reach() <= 0.6 + 1e-4);
        assert_eq!(t.local(&state, Vec3::Y, 0.0, 0.0).reach(), 0.0);
    }

    #[test]
    fn the_wind_sea_runs_downwind() {
        let t = table();
        let n = Vec3::Y;
        let wind = Vec3::X;
        let local = t.local(&t.state(11.0, wind), n, 1000.0, 0.0);
        let downwind: f32 = local.components[..local.count]
            .iter()
            .filter(|c| c.wavenumber != t.bands[BANDS].k)
            .map(|c| c.amplitude * c.amplitude * c.travel.dot(wind))
            .sum();
        assert!(downwind > 0.0);
        for c in &local.components[..local.count] {
            if c.wavenumber != t.bands[BANDS].k {
                assert!(c.travel.dot(wind) > -1e-3, "a wind wave running upwind");
            }
        }
    }

    #[test]
    fn the_gpu_phases_are_the_cpu_phases() {
        let t = table();
        let state = t.state(9.0, Vec3::X);
        let gpu = t.gpu(&state, 12_345.678);
        for i in 0..COMPONENTS {
            assert_eq!(gpu.phase[i / 4][i % 4], t.phase_at(i, 12_345.678));
        }
        assert_eq!(gpu.bands[BANDS].w, state.amplitude[BANDS]);
    }
}
