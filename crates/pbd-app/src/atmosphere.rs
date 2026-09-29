//! The simulated atmosphere in the app: `pbd_core::atmosphere` stepped with
//! the world clock and read by everything that shows weather.
//!
//! One state is PUBLISHED at a time (`Air::now`) and every reader uses it for
//! the whole frame, so the sky, the sea, the ground and the rain never disagree
//! about the weather. Stepping happens on a copy, on the async compute pool, so
//! a frame never waits on the weather; the copy is published whole when it is
//! done, never half-stepped. A capture steps in place instead, so its picture
//! is a function of its flags.
//!
//! With the state go the WEATHER MAPS: the atmosphere resampled onto two small
//! cube maps (cover, cloud top, precipitation and optical depth; the wind at
//! cloud height), which is how the GPU reads the weather per place. They are
//! built off the main thread with the state they describe.

use crate::config::AtmosphereConfig;
use crate::planet::terrain::terrain_config;
use crate::sky::Sun;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future};
use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings, Forcing};
use pbd_core::daylight::{Clock, DAY_S, START_HOUR, YEAR_DAYS};
use std::path::PathBuf;
use std::sync::Arc;

/// Texels along a weather map's cube face.
pub const MAP_SIZE: usize = 64;

/// How far behind the clock the weather may fall before it skips the gap
/// rather than stepping through it, seconds: a paused game, a load or a
/// `--day` jump should not make the next frames step an hour of weather.
const MAX_GAP_S: f64 = 60.0;

/// The most steps one task takes.
const MAX_STEPS: u32 = 30;

/// The weather maps, one `[f32; 4]` per texel, faces in the GPU's cube order
/// (+X, -X, +Y, -Y, +Z, -Z), rows down each face.
#[derive(Clone, Debug, Default)]
pub struct WeatherMaps {
    /// Cover 0..1, cloud top 0..1, precipitation in mm/h (negative for snow),
    /// the column's optical depth.
    pub cloud: Vec<[f32; 4]>,
    /// The wind at cloud height, m/s, body frame; w unused.
    pub wind: Vec<[f32; 4]>,
}

/// The atmosphere as the app holds it.
#[derive(Resource)]
pub struct Air {
    /// The latest complete state. Every reader reads this.
    pub now: Arc<Atmosphere>,
    /// The maps of that state.
    pub maps: Arc<WeatherMaps>,
    /// Bumped whenever `now` changes, so a consumer can tell a new state.
    pub generation: u64,
    /// World time the state stands at, seconds.
    pub at_seconds: f64,
    /// Step in place rather than on the pool: captures.
    pub in_place: bool,
    task: Option<Task<Stepped>>,
}

struct Stepped {
    atmosphere: Atmosphere,
    maps: WeatherMaps,
    at_seconds: f64,
}

/// The shipped settled climates, `assets/climate`, resolved as the config
/// directory is.
fn climate_dir() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/climate"))
}

/// Where in the year a shipped settled state stands, seconds: `START_HOUR` of
/// day 0, where `Clock::default` opens (`examples/settle_climate.rs` stops at a
/// whole number of years plus that).
pub fn settled_at_s() -> f64 {
    f64::from(START_HOUR) / 24.0 * f64::from(DAY_S)
}

/// The settled climate shipped for these settings and this world's terrain
/// generator, as `Atmosphere::to_bytes` wrote it:
/// `assets/climate/settled-g<generator>-l<level>.bin`, made by
/// `examples/settle_climate.rs` with the settings in the `.ron` beside it
/// (`climate-balance` decision 8). The generator is in the name because the
/// climate reads the biomes (`bigger-biomes` decision 6). `None`, with a
/// warning, when none is shipped for this generator and level or it was made
/// with other settings: a state settled under other physics or on other
/// ground is not this world's climate, and the world spins up from rest
/// instead, starting below its mean (finding 9) until it is remade.
pub fn shipped_settled(settings: &AtmosphereSettings) -> Option<Vec<u8>> {
    shipped_settled_for(crate::planet::generator_version(), settings)
}

/// [`shipped_settled`] for a given generator.
pub fn shipped_settled_for(generator: u32, settings: &AtmosphereSettings) -> Option<Vec<u8>> {
    let name = climate_dir().join(format!("settled-g{generator}-l{}", settings.level));
    let ron = name.with_extension("ron");
    let made_with: AtmosphereSettings = match std::fs::read_to_string(&ron) {
        Ok(text) => match ron::from_str(&text) {
            Ok(made_with) => made_with,
            Err(error) => {
                warn!("{}: {error}", ron.display());
                return None;
            }
        },
        Err(_) => {
            warn!(
                "no settled climate is shipped for generator {generator} at level {} ({}); spinning up from rest",
                settings.level,
                ron.display()
            );
            return None;
        }
    };
    if made_with != *settings {
        warn!(
            "{} was made with other atmosphere settings; spinning up from rest. Remake it with \
             `cargo run --release -p pbd-core --example settle_climate -- {}` (it settles on the \
             current generator, {})",
            ron.display(),
            settings.level,
            pbd_core::terrain::GENERATOR_VERSION
        );
        return None;
    }
    std::fs::read(name.with_extension("bin")).ok()
}

impl Air {
    /// A world's atmosphere: its saved state if it has one that fits; for a
    /// world with none, the settled climate shipped for these settings
    /// (`climate-balance` decision 8), brought to `seconds`; and only when
    /// neither fits, a new one spun up from rest to `seconds`. Any stepping
    /// runs here, before the first frame.
    pub fn open(
        settings: AtmosphereSettings,
        seed: u64,
        saved: Option<&[u8]>,
        seconds: f64,
    ) -> Self {
        let mut atmosphere = Atmosphere::new(terrain_config(), settings, seed);
        let restored = saved.is_some_and(|bytes| match atmosphere.restore(bytes) {
            Ok(()) => true,
            Err(error) => {
                warn!("the saved weather could not be used ({error}); starting fresh");
                false
            }
        });
        let settled = !restored
            && shipped_settled(&settings).is_some_and(|bytes| match atmosphere.restore(&bytes) {
                Ok(()) => true,
                Err(error) => {
                    warn!("the shipped settled climate could not be used ({error})");
                    false
                }
            });
        if settled {
            // The settled state stands at `START_HOUR` on a year's first day,
            // where a new world's clock opens. A clock a little past that is
            // stepped to exactly; one elsewhere in the year gets the usual
            // spin-up, from the settled state rather than from rest, so its
            // air and ground come round to the hour and its sea keeps its heat.
            let year_s = YEAR_DAYS * f64::from(DAY_S);
            let gap = (seconds - settled_at_s()).rem_euclid(year_s);
            let span = if gap <= f64::from(settings.spinup_s) {
                gap
            } else {
                f64::from(settings.spinup_s)
            };
            let steps = (span / f64::from(settings.dt_s)).round() as u64;
            for i in 0..steps {
                let t = seconds - (steps - i) as f64 * f64::from(settings.dt_s);
                atmosphere.step(Clock { seconds: t }.sun(), &[]);
            }
            info!(
                "weather: the settled climate for level {}, {:.2} C over the whole surface, \
                 {steps} steps to the clock",
                settings.level,
                atmosphere.mean_surface_c()
            );
        } else if !restored {
            let started = std::time::Instant::now();
            atmosphere.spin_up(|t| Clock { seconds: t }.sun(), seconds);
            info!(
                "weather spun up: {} cells, {:.0} s of weather in {:.1} s",
                atmosphere.grid.len(),
                settings.spinup_s,
                started.elapsed().as_secs_f32()
            );
        }
        let maps = weather_maps(&atmosphere);
        Air {
            now: Arc::new(atmosphere),
            maps: Arc::new(maps),
            generation: 1,
            at_seconds: seconds,
            in_place: false,
            task: None,
        }
    }

    /// Take `steps` steps at once, here, with this forcing: the capture
    /// harness's `--weather-at` and a forced storm's warm-up.
    pub fn run(&mut self, steps: u32, forcing: &[Forcing]) {
        let stepped = step_copy(&self.now, self.at_seconds, steps, forcing.to_vec());
        self.publish(stepped);
    }

    /// Whether a step is running on the pool.
    pub fn stepping(&self) -> bool {
        self.task.is_some()
    }

    fn publish(&mut self, stepped: Stepped) {
        // A trim held at its limit means the heat terms are badly off again
        // (`climate-balance` decision 4), so it is said once when it starts
        // and once when it ends.
        let (was, now) = (
            self.now.sun_trim_at_limit(),
            stepped.atmosphere.sun_trim_at_limit(),
        );
        if now && !was {
            warn!(
                "the sun's trim is held at its limit, {:.2}: the planet's mean is {:.1} C against a target of {:?}",
                stepped.atmosphere.sun_trim,
                stepped.atmosphere.mean_surface_c(),
                stepped.atmosphere.settings.target_mean_c
            );
        } else if was && !now {
            info!(
                "the sun's trim is back inside its limits, at {:.2}",
                stepped.atmosphere.sun_trim
            );
        }
        self.now = Arc::new(stepped.atmosphere);
        self.maps = Arc::new(stepped.maps);
        self.at_seconds = stepped.at_seconds;
        self.generation += 1;
    }
}

/// Step a copy of `from` for `steps` fixed steps starting at world time
/// `start`, with the sun off the clock, and make its maps.
fn step_copy(from: &Atmosphere, start: f64, steps: u32, forcing: Vec<Forcing>) -> Stepped {
    let mut atmosphere = from.clone();
    let dt = atmosphere.settings.dt_s as f64;
    for i in 0..steps {
        let t = start + (i as f64 + 1.0) * dt;
        atmosphere.step(Clock { seconds: t }.sun(), &forcing);
    }
    let maps = weather_maps(&atmosphere);
    Stepped {
        atmosphere,
        maps,
        at_seconds: start + steps as f64 * dt,
    }
}

/// The direction a cube-map texel looks along, in the GPU's convention (the
/// one WGSL's `textureSample` on a `texture_cube` uses).
pub fn cube_direction(face: usize, row: usize, column: usize, size: usize) -> Vec3 {
    let s = (column as f32 + 0.5) / size as f32 * 2.0 - 1.0;
    let t = (row as f32 + 0.5) / size as f32 * 2.0 - 1.0;
    let d = match face {
        0 => Vec3::new(1.0, -t, -s),
        1 => Vec3::new(-1.0, -t, s),
        2 => Vec3::new(s, 1.0, t),
        3 => Vec3::new(s, -1.0, -t),
        4 => Vec3::new(s, -t, 1.0),
        _ => Vec3::new(-s, -t, -1.0),
    };
    d.normalize()
}

/// Resample the atmosphere onto the two weather maps.
pub fn weather_maps(atmosphere: &Atmosphere) -> WeatherMaps {
    let texels = 6 * MAP_SIZE * MAP_SIZE;
    let settings = atmosphere.settings;
    let mut maps = WeatherMaps {
        cloud: Vec::with_capacity(texels),
        wind: Vec::with_capacity(texels),
    };
    for face in 0..6 {
        for row in 0..MAP_SIZE {
            for column in 0..MAP_SIZE {
                let s = smoothed(atmosphere, cube_direction(face, row, column, MAP_SIZE));
                let rain_mmh = s.rain_rate * 3600.0 * if s.snow { -1.0 } else { 1.0 };
                maps.cloud
                    .push([s.cover, s.cloud_top, rain_mmh, s.optical_depth]);
                // The wind that CARRIES the cloud, which is what its detail has
                // to drift with: the steering blend at the cloud's pace. The
                // full upper wind, which this was, slid the texture across its
                // own cloud faster than the cloud moved (`calm-clouds`).
                let carried = s.wind.lerp(s.upper, settings.cloud_steering) * settings.cloud_pace;
                maps.wind.push([carried.x, carried.y, carried.z, 0.0]);
            }
        }
    }
    maps
}

/// The atmosphere at a texel, averaged over a small disc round it: the cells
/// are 181 m apart and a texel 118 m, so a single sample would hand the GPU
/// the cells' own facets, and a cloud's edge would trace them.
fn smoothed(atmosphere: &Atmosphere, direction: Vec3) -> pbd_core::atmosphere::Sample {
    let reach = 90.0 / atmosphere.grid.radius;
    let u = direction.any_orthonormal_vector();
    let v = direction.cross(u);
    let mut sum = atmosphere.sample(direction);
    let mut weight = 1.0;
    for k in 0..6 {
        let angle = k as f32 * std::f32::consts::TAU / 6.0;
        let (sin, cos) = angle.sin_cos();
        let s = atmosphere.sample((direction + (u * cos + v * sin) * reach).normalize());
        sum.cover += s.cover;
        sum.cloud_top += s.cloud_top;
        sum.rain_rate += s.rain_rate;
        sum.optical_depth += s.optical_depth;
        sum.upper += s.upper;
        weight += 1.0;
    }
    sum.cover /= weight;
    sum.cloud_top /= weight;
    sum.rain_rate /= weight;
    sum.optical_depth /= weight;
    sum.upper /= weight;
    sum
}

/// Where the weather slider is brewing a storm: at the player, at the
/// slider's strength.
fn forcing_here(weather: &crate::weather::Weather, strength: f32) -> Vec<Forcing> {
    if strength > 0.0 && weather.here != Vec3::ZERO {
        vec![Forcing {
            direction: weather.here,
            strength,
        }]
    } else {
        Vec::new()
    }
}

/// Collect a finished step and start the next one when the clock has moved a
/// step past the published state.
pub fn advance_air(
    mut air: ResMut<Air>,
    sun: Res<Sun>,
    forcing: Res<crate::weather::StormForcing>,
    weather: Res<crate::weather::Weather>,
    lod: Option<Res<crate::planet::LodRefresh>>,
) {
    if let Some(task) = air.task.as_mut() {
        match block_on(future::poll_once(task)) {
            Some(stepped) => {
                air.task = None;
                air.publish(stepped);
            }
            None => return,
        }
    }
    let dt = air.now.settings.dt_s as f64;
    let now = sun.clock.seconds;
    if now < air.at_seconds || now - air.at_seconds > MAX_GAP_S {
        // The clock went back (a world loaded) or leapt ahead: the weather
        // picks up from here rather than replaying or skipping in steps.
        air.at_seconds = now;
        return;
    }
    let behind = ((now - air.at_seconds) / dt).floor() as u32;
    if behind == 0 {
        return;
    }
    // Never step while the fine set is being rebuilt: each is harmless alone,
    // but the two at once left the frame no core, and a hitch landed every
    // second of a low flight (`far-side-flight`: 11 frames over 16.7 ms on the
    // route, 2 with the air frozen). The step waits and catches up in one go.
    if !air.in_place && lod.is_some_and(|lod| lod.in_flight_s().is_some()) {
        return;
    }
    let steps = behind.min(MAX_STEPS);
    let forcing = forcing_here(&weather, forcing.0);
    if air.in_place {
        air.run(steps, &forcing);
        return;
    }
    let from = air.now.clone();
    let start = air.at_seconds;
    air.task = Some(
        AsyncComputeTaskPool::get().spawn(async move { step_copy(&from, start, steps, forcing) }),
    );
}

/// A capture with the storm forcing on brews its storm before the picture:
/// the atmosphere is stepped here with the forcing at the spawn until the
/// storm has had time to form. Runs once, on the first frame that knows where
/// the player is.
pub fn warm_capture(
    mut air: ResMut<Air>,
    forcing: Res<crate::weather::StormForcing>,
    weather: Res<crate::weather::Weather>,
    config: Res<AtmosphereConfig>,
    mut done: Local<bool>,
) {
    if *done || !air.in_place || weather.here == Vec3::ZERO {
        return;
    }
    *done = true;
    if forcing.0 > 0.0 {
        let steps = (config.0.forcing_s * 8.0 / config.0.dt_s).ceil() as u32;
        let brew = forcing_here(&weather, forcing.0);
        air.run(steps, &brew);
    }
}

#[cfg(test)]
mod settled_tests {
    use super::*;

    /// Every shipped settled state was made with the settings the game runs:
    /// the code defaults, which `atmosphere.ron` is held equal to, at its
    /// level (`climate-balance` task 3.2b). A knob turned without remaking
    /// the states fails here rather than quietly starting worlds cold.
    ///
    /// Each is named for the generator it settled on (`bigger-biomes`
    /// decision 6), and the current generator's ship at levels 3 and 5.
    #[test]
    fn the_shipped_settled_climates_are_made_with_the_running_settings() {
        let mut shipped = Vec::new();
        for entry in std::fs::read_dir(climate_dir()).expect("assets/climate") {
            let name = entry.expect("an entry").file_name();
            let name = name.to_string_lossy();
            // A state is its settings (`.ron`) and its bytes (`.bin`); the
            // settings are what is checked, and the bytes must be beside them.
            let Some((stem, extension)) = name.rsplit_once('.') else {
                continue;
            };
            let Some((generator, level)) = stem
                .strip_prefix("settled-g")
                .and_then(|rest| rest.split_once("-l"))
            else {
                assert!(
                    !name.starts_with("settled-"),
                    "{name} does not name its generator and level"
                );
                continue;
            };
            let generator: u32 = generator.parse().expect("a generator");
            let level: u32 = level.parse().expect("a level");
            if extension != "ron" {
                continue;
            }
            let settings = AtmosphereSettings {
                level,
                ..Default::default()
            };
            assert!(
                shipped_settled_for(generator, &settings).is_some(),
                "{name} was not made with the running settings, or has no state beside it"
            );
            shipped.push((generator, level));
        }
        let current = pbd_core::terrain::GENERATOR_VERSION;
        assert!(
            shipped.contains(&(current, 3)),
            "level 3 ships, for the fish test"
        );
        assert!(
            shipped.contains(&(current, 5)),
            "level 5 ships, the game's own level"
        );
    }

    /// Each generator's worlds open on the climate settled on its own ground:
    /// the climate reads the biomes, so a version-4 world never opens on a
    /// state settled on version 5's, and a generator with none shipped spins
    /// up from rest (`bigger-biomes` decision 6, task 3.1a).
    #[test]
    fn a_world_opens_on_its_own_generators_settled_climate() {
        let settings = AtmosphereSettings {
            level: 3,
            ..Default::default()
        };
        let old = shipped_settled_for(4, &settings).expect("version 4 ships level 3");
        let new = shipped_settled_for(5, &settings).expect("version 5 ships level 3");
        assert_ne!(old, new, "two grounds, two settled climates");
        assert_eq!(shipped_settled_for(99, &settings), None);
    }

    /// A new world opens on the settled state, exactly, when its clock opens
    /// where the state stands; a little later, it is stepped there and still
    /// holds 15 +/- 0.5 C; and with other settings it spins up from rest.
    #[test]
    fn a_new_world_opens_on_the_settled_climate() {
        let settings = AtmosphereSettings {
            level: 3,
            ..Default::default()
        };
        let shipped = shipped_settled(&settings).expect("level 3 is shipped");
        let opening = Clock::default().seconds;
        assert!((opening.rem_euclid(YEAR_DAYS * f64::from(DAY_S)) - settled_at_s()).abs() < 1e-6);
        let air = Air::open(settings, terrain_config().seed, None, opening);
        assert_eq!(
            air.now.to_bytes(),
            shipped,
            "opened on the shipped state as it is"
        );
        let mean = air.now.mean_surface_c();
        assert!(
            (mean - 15.0).abs() < 0.5,
            "a new world opens at {mean:.2} C"
        );
        let noon = Clock::at_hour(12.0).seconds;
        let later = Air::open(settings, terrain_config().seed, None, noon);
        let mean = later.now.mean_surface_c();
        assert!(
            (mean - 15.0).abs() < 0.5,
            "at noon a new world is at {mean:.2} C"
        );
        assert_ne!(later.now.to_bytes(), shipped, "stepped on to noon");
        let other = AtmosphereSettings {
            target_mean_c: Some(20.0),
            ..settings
        };
        assert!(
            shipped_settled(&other).is_none(),
            "other settings are not settled"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every texel of a face looks out through that face, and neighbouring
    /// texels look nearly the same way: the convention is the cube's, not a
    /// scramble of it.
    #[test]
    fn cube_texels_look_out_through_their_own_face() {
        let axes = [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z];
        for (face, axis) in axes.iter().enumerate() {
            for (row, column) in [(0, 0), (0, 63), (63, 0), (31, 32), (63, 63)] {
                let d = cube_direction(face, row, column, 64);
                assert!(
                    d.dot(*axis) >= 0.57,
                    "face {face} texel {row},{column}: {d:?}"
                );
            }
            let a = cube_direction(face, 10, 10, 64);
            let b = cube_direction(face, 10, 11, 64);
            assert!(a.angle_between(b) < 0.05);
        }
        // The face orientation the GPU uses: +X's first column looks toward +Z
        // and its first row toward +Y.
        assert!(cube_direction(0, 32, 0, 64).z > 0.5);
        assert!(cube_direction(0, 0, 32, 64).y > 0.5);
        assert!(cube_direction(4, 32, 0, 64).x < -0.5);
    }

    /// A measurement instrument for the `calm-clouds` change: how fast the
    /// cloud the player sees moves and changes, off the shipped atmosphere.
    /// The wind at cloud height (what drifts the detail), what that is as an
    /// angle a second overhead at the cloud base, and how much of the cover
    /// map changes between one published map and the next. Run with
    /// `cargo test -p pbd-app --release --lib cloud_pace -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn cloud_pace() {
        let settings = pbd_core::atmosphere::AtmosphereSettings::default();
        let start = 0.4 * pbd_core::daylight::DAY_S as f64;
        let air = Air::open(settings, terrain_config().seed, None, start);
        let mut atmosphere = (*air.now).clone();
        let mut before = weather_maps(&atmosphere);
        let speeds: Vec<f32> = before
            .wind
            .iter()
            .map(|w| Vec3::new(w[0], w[1], w[2]).length())
            .collect();
        let mut sorted = speeds.clone();
        sorted.sort_by(f32::total_cmp);
        let mean = speeds.iter().sum::<f32>() / speeds.len() as f32;
        let p90 = sorted[sorted.len() * 9 / 10];
        let base_m = crate::sky::CLOUD_RADIUS - crate::planet::terrain::PLANET_RADIUS;
        eprintln!(
            "wind the cloud detail drifts with: mean {mean:.1} m/s, 90th percentile {p90:.1} m/s; \
             overhead at the {base_m:.0} m base that is {:.2} deg/s mean, {:.2} deg/s p90",
            (mean / base_m).to_degrees(),
            (p90 / base_m).to_degrees()
        );
        let steering: Vec<f32> = atmosphere
            .wind
            .iter()
            .zip(&atmosphere.upper)
            .map(|(w, u)| (w.lerp(*u, settings.cloud_steering) * settings.cloud_pace).length())
            .collect();
        let surface: f32 =
            atmosphere.wind.iter().map(|w| w.length()).sum::<f32>() / atmosphere.wind.len() as f32;
        let steer = steering.iter().sum::<f32>() / steering.len() as f32;
        eprintln!(
            "per cell: surface wind {surface:.1} m/s, the steering wind that carries the \
             cloud {steer:.1} m/s ({:.2} deg/s overhead)",
            (steer / base_m).to_degrees()
        );
        let mut t = start;
        for steps in [1u32, 1, 1, 5, 30] {
            for _ in 0..steps {
                t += settings.dt_s as f64;
                atmosphere.step(Clock { seconds: t }.sun(), &[]);
            }
            let after = weather_maps(&atmosphere);
            let deltas: Vec<f32> = before
                .cloud
                .iter()
                .zip(&after.cloud)
                .map(|(a, b)| (a[0] - b[0]).abs())
                .collect();
            let mean = deltas.iter().sum::<f32>() / deltas.len() as f32;
            let moved = deltas.iter().filter(|d| **d > 0.05).count();
            eprintln!(
                "cover after {steps} step(s) of {} s: mean change {mean:.4}, {:.1}% of texels \
                 change by more than 0.05",
                settings.dt_s,
                100.0 * moved as f32 / deltas.len() as f32
            );
            before = after;
        }
    }
}
