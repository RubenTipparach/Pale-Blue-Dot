//! The water a fish would live in, measured off the real world: the depth and
//! the river channels from the terrain generator, and the water temperature
//! over the seasons from the atmosphere and ocean simulation run forward from
//! a new world. The measurement instrument for
//! `openspec/changes/fishing-and-equipment` (design section 7); its numbers
//! and its maps go in that design, and nothing in the game reads it.
//!
//!     cargo run --release -p pbd-core --example fish_ranges -- [days] [out.bin]
//!     ATMOSPHERE='(solar_wm2: 1360.0)' cargo run ... # the same, retuned
//!
//! It writes an equirectangular raster, 1440 x 720, of little-endian f32
//! planes after a header (`PBDFISH1`, width, height, plane count as u32):
//! the surface altitude, the altitude with the river carve switched off (a
//! pixel that is water only because of the carve is a river), then for each
//! simulated year the mean, the coldest and the warmest daily-mean water
//! temperature, deg C. `tools/fish_ranges.py` draws the maps from it.
//!
//! The temperature is `ground_k`, which the atmosphere documents as the sea
//! surface over the sea and the ground at sea level on land; a river pixel
//! reads the latter, which is the water standing in the channel.

use glam::Vec3;
use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S, YEAR_DAYS};
use pbd_core::planet_gen::{TerrainConfig, surface_altitude};
use std::time::Instant;

const WIDTH: usize = 1440;
const HEIGHT: usize = 720;
/// Samples of the temperature per day, averaged to a daily mean so the
/// day's own heating and cooling does not decide a year's extremes.
const SAMPLES_PER_DAY: u64 = 8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: u32 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(200);
    let out = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "fish_fields.bin".into());
    let terrain = TerrainConfig::TENEBRIS;
    // `ATMOSPHERE='(solar_wm2: 1360.0)'` overrides any knob for a run, in the
    // RON the shipped file uses, exactly as the climate report takes it: the
    // way a proposed retune is measured before anybody makes it.
    let settings: AtmosphereSettings = std::env::var("ATMOSPHERE")
        .ok()
        .map(|text| ron::from_str(&text).expect("ATMOSPHERE must be a RON struct"))
        .unwrap_or_default();
    settings.validate().expect("legal settings");
    let years = (days as f64 / YEAR_DAYS).ceil().max(1.0) as usize;

    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    let cells = air.grid.len();
    air.spin_up(|t| Clock { seconds: t }.sun(), 0.0);
    println!(
        "{cells} cells, {:.0} m apart; running {days} days ({years} years of {YEAR_DAYS} days)",
        air.grid.spacing()
    );

    // Per year, per cell: sum of daily means, count, coldest, warmest.
    let mut sum = vec![vec![0.0f64; cells]; years];
    let mut count = vec![0u32; years];
    let mut cold = vec![vec![f32::INFINITY; cells]; years];
    let mut warm = vec![vec![f32::NEG_INFINITY; cells]; years];
    let steps_per_day = (DAY_S / settings.dt_s) as u64;
    let every = steps_per_day / SAMPLES_PER_DAY;
    let mut day_sum = vec![0.0f64; cells];
    let run = Instant::now();
    for day in 0..days as u64 {
        day_sum.iter_mut().for_each(|s| *s = 0.0);
        for step in 0..steps_per_day {
            let t = (day * steps_per_day + step) as f64 * settings.dt_s as f64;
            air.step(Clock { seconds: t }.sun(), &[]);
            if step % every == 0 {
                for (s, k) in day_sum.iter_mut().zip(&air.ground_k) {
                    *s += *k as f64;
                }
            }
        }
        let year = (day as f64 / YEAR_DAYS) as usize;
        count[year] += 1;
        for i in 0..cells {
            let mean = (day_sum[i] / SAMPLES_PER_DAY as f64) as f32;
            sum[year][i] += mean as f64;
            cold[year][i] = cold[year][i].min(mean);
            warm[year][i] = warm[year][i].max(mean);
        }
        if day % 10 == 9 {
            let sea: Vec<usize> = (0..cells).filter(|&i| air.surface.ocean[i]).collect();
            let mean = sea.iter().map(|&i| air.ground_k[i] as f64).sum::<f64>() / sea.len() as f64;
            println!(
                "day {:>3}: mean sea surface {mean:.2} C, {:.0} s so far",
                day + 1,
                run.elapsed().as_secs_f64()
            );
        }
    }

    let per_cell: Vec<[Vec<f32>; 3]> = (0..years)
        .map(|y| {
            let n = count[y].max(1) as f64;
            [
                sum[y].iter().map(|s| (s / n) as f32).collect(),
                cold[y].clone(),
                warm[y].clone(),
            ]
        })
        .collect();

    let mut no_rivers = terrain;
    // The channel field never exceeds one, so no carve fires.
    no_rivers.river_threshold = 2.0;
    let planes = 2 + 3 * years;
    let mut data = vec![0.0f32; planes * WIDTH * HEIGHT];
    let plane = |p: usize, i: usize| p * WIDTH * HEIGHT + i;
    let raster = Instant::now();
    for row in 0..HEIGHT {
        let latitude = (0.5 - (row as f32 + 0.5) / HEIGHT as f32) * std::f32::consts::PI;
        for column in 0..WIDTH {
            let longitude = ((column as f32 + 0.5) / WIDTH as f32 - 0.5) * std::f32::consts::TAU;
            let d = Vec3::new(
                latitude.cos() * longitude.cos(),
                latitude.sin(),
                latitude.cos() * longitude.sin(),
            );
            let i = row * WIDTH + column;
            data[plane(0, i)] = surface_altitude(&terrain, d);
            data[plane(1, i)] = surface_altitude(&no_rivers, d);
            let (at, w) = air.grid.locate(d);
            for (y, fields) in per_cell.iter().enumerate() {
                for (f, field) in fields.iter().enumerate() {
                    let v: f32 = at.iter().zip(w).map(|(&c, w)| field[c as usize] * w).sum();
                    data[plane(2 + 3 * y + f, i)] = v;
                }
            }
        }
    }
    let mut bytes = b"PBDFISH1".to_vec();
    for v in [WIDTH as u32, HEIGHT as u32, planes as u32] {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    for v in &data {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(&out, bytes).expect("the fields could be written");
    println!(
        "simulated in {:.0} s, rastered in {:.1} s, wrote {out} ({planes} planes)",
        run.elapsed().as_secs_f64() - raster.elapsed().as_secs_f64(),
        raster.elapsed().as_secs_f64()
    );
}
