//! A settled climate for a new world to start from (`climate-balance`
//! decision 8, survey K6). The measurement instrument that makes
//! `assets/climate/settled-l<level>.bin`: the game restores it for a world
//! with no weather of its own, so a new world's first day is its settled one.
//!
//!     cargo run --release -p pbd-core --example settle_climate -- \
//!         [level] [fast years] [check days] [out dir]
//!
//! It runs a new world forward on the shipped settings (the code defaults,
//! which `assets/config/atmosphere.ron` is held equal to) at `level`:
//! - for `fast years`, with the sea's heat capacity cut to a tenth, so the sea
//!   settles in weeks where it takes a year and a half. The year's mean does
//!   not depend on how much heat the sea holds, only on how fast it gets
//!   there: the accelerated spin-up climate models use;
//! - then the sea is set to its own mean over the last fast year, which
//!   drops the larger seasonal swing a light sea has, and one year runs at
//!   the true capacity, so the seasons come back at their true size.
//!
//! It stops at a whole number of years, at `START_HOUR` of the day, the time
//! a new world's clock opens on, and writes the state (`Atmosphere::to_bytes`,
//! the weather save a world keeps) and the settings it was made with, as RON.
//! Then it runs `check days` more from that state as the game would, and
//! prints the whole surface's mean each day: what a new world will see.

use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S, START_HOUR, YEAR_DAYS};
use pbd_core::planet_gen::TerrainConfig;
use std::sync::Arc;

/// How much faster the sea runs in the fast years.
const FAST: f32 = 10.0;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let level: u32 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(5);
    let fast_years: u64 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(2);
    let check_days: u64 = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(30);
    let out = args
        .get(4)
        .cloned()
        .unwrap_or_else(|| "assets/climate".into());
    let terrain = TerrainConfig::TENEBRIS;
    let settings = AtmosphereSettings {
        level,
        ..Default::default()
    };
    settings.validate().expect("legal settings");
    let started = std::time::Instant::now();
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    air.spin_up(|t| Clock { seconds: t }.sun(), 0.0);
    let n = air.grid.len();
    let ocean = air.surface.ocean.clone();
    let set_sea_capacity = |air: &mut Atmosphere, factor: f32| {
        let surface = Arc::make_mut(&mut air.surface);
        for (i, capacity) in surface.heat_capacity.iter_mut().enumerate() {
            if ocean[i] {
                *capacity *= factor;
            }
        }
    };
    set_sea_capacity(&mut air, 1.0 / FAST);
    let steps_per_day = (DAY_S / settings.dt_s) as u64;
    let year_days = YEAR_DAYS as u64;
    let opening = f64::from(START_HOUR) / 24.0 * f64::from(DAY_S);
    let total_days = (fast_years + 1) * year_days;
    let end_step = (total_days as f64 * f64::from(DAY_S) + opening) / f64::from(settings.dt_s);
    let switch_step = (fast_years * year_days * steps_per_day) as f64;
    let mut sea_mean = vec![0.0f64; n];
    let mut sea_samples = 0u32;
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
    let mut step = 0u64;
    while (step as f64) < end_step {
        let t = step as f64 * f64::from(settings.dt_s);
        air.step(Clock { seconds: t }.sun(), &[]);
        step += 1;
        let in_last_fast_year = step as f64 >= switch_step - (year_days * steps_per_day) as f64
            && (step as f64) < switch_step;
        if in_last_fast_year && step.is_multiple_of(10) {
            for (mean, &k) in sea_mean.iter_mut().zip(&air.ground_k) {
                *mean += f64::from(k);
            }
            sea_samples += 1;
        }
        if step as f64 == switch_step {
            for i in 0..n {
                if ocean[i] {
                    air.ground_k[i] = (sea_mean[i] / f64::from(sea_samples)) as f32;
                }
            }
            set_sea_capacity(&mut air, FAST);
            println!(
                "day {}: the sea set to its mean and its true capacity",
                step / steps_per_day
            );
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
    }
    std::fs::create_dir_all(&out).expect("the output directory");
    let name = format!("{out}/settled-l{level}");
    std::fs::write(format!("{name}.bin"), air.to_bytes()).expect("write the state");
    let ron = ron::ser::to_string_pretty(&settings, ron::ser::PrettyConfig::default())
        .expect("settings are plain data");
    std::fs::write(format!("{name}.ron"), ron + "\n").expect("write the settings");
    println!(
        "wrote {name}.bin at day {:.3}: whole surface {:.2} C, sea {:.2} C",
        step as f64 / steps_per_day as f64,
        air.mean_surface_c(),
        sea_c(&air)
    );
    // What a new world sees from it.
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
