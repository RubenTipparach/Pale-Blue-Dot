//! A settled climate for a new world to start from (`climate-balance`
//! decisions 8 and 8a, survey K6). The measurement instrument that makes
//! `assets/climate/settled-g<generator>-l<level>.bin`: the game restores it
//! for a world with no weather of its own, so a new world's first day is its
//! settled one. It settles on a generator's terrain, whose biomes
//! the climate reads (wetness and albedo), so a state is only for worlds of
//! the generator in its name (`bigger-biomes` decision 6).
//!
//!     cargo run --release -p pbd-core --example settle_climate -- \
//!         [level] [fast years] [check days] [out dir] [generator]
//!
//! `generator` defaults to the current one. An older one remakes the state
//! its worlds start from when the weather's rules change
//! (`tropical-upper-wind` decision 4).
//!
//! It runs a new world forward on the shipped settings (the code defaults,
//! which `assets/config/atmosphere.ron` is held equal to) at `level`:
//! - for `fast years`, with the sea's heat capacity cut to a tenth, so the sea
//!   settles in weeks where it takes a year and a half: the accelerated
//!   spin-up climate models use;
//! - then the sea is set to its own mean over the last fast year, which
//!   drops the larger seasonal swing a light sea has, and true years run at
//!   the true capacity until one lands: its mean over the whole surface
//!   within 0.2 K of the target. A year that misses shifts the sea by the
//!   miss, capped at 2 K, and runs again; at most four (finding 10: at
//!   level 5 the fast years' sea leaves the surface a kelvin warm). A kelvin
//!   of sea moves the surface's year by about a kelvin, 0.9 to 1.2 measured
//!   at level 3, since the land follows the sea.
//!
//! It stops at a whole number of years, at `START_HOUR` of the day, the time
//! a new world's clock opens on, and writes the state (`Atmosphere::to_bytes`,
//! the weather save a world keeps) and the settings it was made with, as RON.
//! Then it runs `check days` more from that state as the game would, and
//! prints the whole surface's mean each day: what a new world will see.
//!
//! Every ten days it writes a checkpoint beside the output, and a run started
//! with the same settings picks up from it: a level-5 settle takes hours.

use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S, START_HOUR, YEAR_DAYS};
use pbd_core::planet_gen::TerrainConfig;
use std::path::Path;
use std::sync::Arc;

/// How much faster the sea runs in the fast years.
const FAST: f32 = 10.0;
/// How close a true year's mean must come to the target to be shipped, K.
const LANDED_K: f64 = 0.2;
/// The most the sea is shifted after one true year, K.
const MAX_SHIFT_K: f64 = 2.0;
/// The most true years run.
const MAX_TRUE_YEARS: u32 = 4;
/// Days between checkpoints.
const CHECKPOINT_DAYS: u64 = 10;
const MAGIC: &[u8; 8] = b"PBDSETL1";

/// Where the run is, as a checkpoint holds it.
struct Progress {
    step: u64,
    /// The sea's summed temperature over the last fast year, per cell.
    sea_mean: Vec<f64>,
    sea_samples: u32,
    /// True years finished, and the one under way's daily sum and count.
    true_years: u32,
    year_sum: f64,
    year_days: u32,
    /// Whether the last true year landed, and the run is only stepping on to
    /// the hour a new world opens at.
    landed: bool,
}

fn write_checkpoint(path: &Path, ron: &str, progress: &Progress, air: &Atmosphere) {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    let put = |out: &mut Vec<u8>, bytes: &[u8]| {
        out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(bytes);
    };
    put(&mut out, ron.as_bytes());
    out.extend_from_slice(&progress.step.to_le_bytes());
    out.extend_from_slice(&progress.sea_samples.to_le_bytes());
    out.extend_from_slice(&progress.true_years.to_le_bytes());
    out.extend_from_slice(&progress.year_sum.to_le_bytes());
    out.extend_from_slice(&progress.year_days.to_le_bytes());
    out.push(u8::from(progress.landed));
    let sea: Vec<u8> = progress
        .sea_mean
        .iter()
        .flat_map(|v| v.to_le_bytes())
        .collect();
    put(&mut out, &sea);
    put(&mut out, &air.to_bytes());
    let temporary = path.with_extension("partial.tmp");
    std::fs::write(&temporary, out).expect("write the checkpoint");
    std::fs::rename(&temporary, path).expect("move the checkpoint into place");
}

/// Reads a checkpoint front to back.
struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let slice = self.bytes.get(self.at..self.at + n)?;
        self.at += n;
        Some(slice)
    }
    fn word<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }
    fn block(&mut self) -> Option<&'a [u8]> {
        let n = u64::from_le_bytes(self.word()?) as usize;
        self.take(n)
    }
}

/// A checkpoint made with these settings, restored onto `air`.
fn read_checkpoint(path: &Path, ron: &str, air: &mut Atmosphere) -> Option<Progress> {
    let bytes = std::fs::read(path).ok()?;
    let mut c = Cursor {
        bytes: &bytes,
        at: 0,
    };
    if c.take(8)? != MAGIC {
        return None;
    }
    if c.block()? != ron.as_bytes() {
        println!("a checkpoint made with other settings is ignored");
        return None;
    }
    let step = u64::from_le_bytes(c.word()?);
    let sea_samples = u32::from_le_bytes(c.word()?);
    let true_years = u32::from_le_bytes(c.word()?);
    let year_sum = f64::from_le_bytes(c.word()?);
    let year_days = u32::from_le_bytes(c.word()?);
    let landed = c.take(1)?[0] != 0;
    let sea_mean = c
        .block()?
        .chunks_exact(8)
        .map(|b| f64::from_le_bytes(b.try_into().expect("eight bytes")))
        .collect();
    air.restore(c.block()?).ok()?;
    Some(Progress {
        step,
        sea_mean,
        sea_samples,
        true_years,
        year_sum,
        year_days,
        landed,
    })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let level: u32 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(5);
    let fast_years: u64 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(2);
    let check_days: u64 = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(30);
    let out = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| "assets/climate".into());
    let generator: u32 = args
        .get(5)
        .map(|a| a.parse().expect("a generator version"))
        .unwrap_or(pbd_core::terrain::GENERATOR_VERSION);
    let terrain = TerrainConfig::for_version(generator).expect("a carried generator");
    let settings = AtmosphereSettings {
        level,
        ..Default::default()
    };
    settings.validate().expect("legal settings");
    let target = f64::from(settings.target_mean_c.expect("a target to settle on"));
    let ron = ron::ser::to_string_pretty(&settings, ron::ser::PrettyConfig::default())
        .expect("settings are plain data");
    std::fs::create_dir_all(&out).expect("the output directory");
    let name = format!("{out}/settled-g{generator}-l{level}");
    let checkpoint = Path::new(&name).with_extension("partial");
    let started = std::time::Instant::now();
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    let n = air.grid.len();
    let ocean = air.surface.ocean.clone();
    let steps_per_day = (DAY_S / settings.dt_s) as u64;
    let year_steps = YEAR_DAYS as u64 * steps_per_day;
    let opening_steps =
        (f64::from(START_HOUR) / 24.0 * f64::from(DAY_S) / f64::from(settings.dt_s)) as u64;
    let switch_step = fast_years * year_steps;
    let mut progress = match read_checkpoint(&checkpoint, &ron, &mut air) {
        Some(progress) => {
            println!(
                "resumed from the checkpoint at day {}",
                progress.step / steps_per_day
            );
            progress
        }
        None => {
            air.spin_up(|t| Clock { seconds: t }.sun(), 0.0);
            Progress {
                step: 0,
                sea_mean: vec![0.0; n],
                sea_samples: 0,
                true_years: 0,
                year_sum: 0.0,
                year_days: 0,
                landed: false,
            }
        }
    };
    let set_sea_capacity = |air: &mut Atmosphere, factor: f32| {
        let surface = Arc::make_mut(&mut air.surface);
        for (i, capacity) in surface.heat_capacity.iter_mut().enumerate() {
            if ocean[i] {
                *capacity *= factor;
            }
        }
    };
    if progress.step < switch_step {
        set_sea_capacity(&mut air, 1.0 / FAST);
    }
    let sea_c = |air: &Atmosphere| -> f64 {
        let (mut sum, mut area) = (0.0f64, 0.0f64);
        for ((&wet, &k), &a) in ocean.iter().zip(&air.ground_k).zip(&air.grid.area) {
            if wet {
                sum += f64::from(k) * f64::from(a);
                area += f64::from(a);
            }
        }
        sum / area
    };
    // The step the state is written at: set once a true year lands.
    let mut end_step: Option<u64> = progress
        .landed
        .then(|| switch_step + u64::from(progress.true_years) * year_steps + opening_steps);
    loop {
        if end_step.is_some_and(|end| progress.step >= end) {
            break;
        }
        let t = progress.step as f64 * f64::from(settings.dt_s);
        air.step(Clock { seconds: t }.sun(), &[]);
        progress.step += 1;
        let step = progress.step;
        let in_last_fast_year = step + year_steps >= switch_step && step < switch_step;
        if in_last_fast_year && step % 10 == 0 {
            for (mean, &k) in progress.sea_mean.iter_mut().zip(&air.ground_k) {
                *mean += f64::from(k);
            }
            progress.sea_samples += 1;
        }
        if step == switch_step {
            let samples = f64::from(progress.sea_samples);
            for ((k, &wet), &mean) in air.ground_k.iter_mut().zip(&ocean).zip(&progress.sea_mean) {
                if wet {
                    *k = (mean / samples) as f32;
                }
            }
            set_sea_capacity(&mut air, FAST);
            println!(
                "day {}: the sea set to its mean and its true capacity",
                step / steps_per_day
            );
        }
        if step > switch_step && !progress.landed && step.is_multiple_of(steps_per_day) {
            progress.year_sum += air.mean_surface_c();
            progress.year_days += 1;
            if (step - switch_step).is_multiple_of(year_steps) {
                progress.true_years += 1;
                let mean = progress.year_sum / f64::from(progress.year_days);
                let miss = mean - target;
                progress.year_sum = 0.0;
                progress.year_days = 0;
                if miss.abs() <= LANDED_K || progress.true_years >= MAX_TRUE_YEARS {
                    progress.landed = true;
                    end_step = Some(step + opening_steps);
                    println!(
                        "true year {}: the whole surface averaged {mean:.2} C, {miss:+.2} K; {}",
                        progress.true_years,
                        if miss.abs() <= LANDED_K {
                            "landed"
                        } else {
                            "the last year allowed, shipped as it is"
                        }
                    );
                } else {
                    let shift = (-miss).clamp(-MAX_SHIFT_K, MAX_SHIFT_K);
                    for (k, &wet) in air.ground_k.iter_mut().zip(&ocean) {
                        if wet {
                            *k += shift as f32;
                        }
                    }
                    println!(
                        "true year {}: the whole surface averaged {mean:.2} C, {miss:+.2} K; the sea shifted {shift:+.2} K and another year run",
                        progress.true_years
                    );
                }
            }
        }
        if step.is_multiple_of(10 * steps_per_day) {
            println!(
                "day {:>4}: whole surface {:.2} C, sea {:.2} C, trim {:.3}, {:.0} s",
                step / steps_per_day,
                air.mean_surface_c(),
                sea_c(&air),
                air.sun_trim,
                started.elapsed().as_secs_f64()
            );
        }
        if step.is_multiple_of(CHECKPOINT_DAYS * steps_per_day) {
            write_checkpoint(&checkpoint, &ron, &progress, &air);
        }
    }
    std::fs::write(format!("{name}.bin"), air.to_bytes()).expect("write the state");
    std::fs::write(format!("{name}.ron"), ron.clone() + "\n").expect("write the settings");
    let _ = std::fs::remove_file(&checkpoint);
    println!(
        "wrote {name}.bin at day {:.3}: whole surface {:.2} C, sea {:.2} C",
        progress.step as f64 / steps_per_day as f64,
        air.mean_surface_c(),
        sea_c(&air)
    );
    // What a new world sees from it.
    let mut step = progress.step;
    for day in 1..=check_days {
        for _ in 0..steps_per_day {
            let t = step as f64 * f64::from(settings.dt_s);
            air.step(Clock { seconds: t }.sun(), &[]);
            step += 1;
        }
        println!(
            "new world day {day:>3}: whole surface {:.2} C, trim {:.3}",
            air.mean_surface_c(),
            air.sun_trim
        );
    }
}
