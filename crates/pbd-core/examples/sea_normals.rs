//! The sea's yearly normals for the ocean places (`ocean-environments`
//! design B10): a measurement instrument whose output is generator version
//! 7's input, `assets/climate/sea-normals-g7.bin`.
//!
//!     cargo run --profile fast -p pbd-core --example sea_normals -- [out.bin]
//!
//! It restores the settled climate a new world starts from
//! (`assets/climate/settled-g6-l5.bin`, which version 7 shares since it moves
//! nothing the climate reads), runs one year of `YEAR_DAYS` days on the
//! shipped settings, and writes per atmosphere cell:
//! - the coldest daily-mean sea temperature, deg C;
//! - the warmest daily-mean sea temperature, deg C;
//! - the share of the year's days whose daily mean is at or below the
//!   freezing point of the sea, -1.8 C.
//!
//! The temperature is `ground_k`, the sea surface over the sea, as
//! `examples/fish_ranges.rs` reads it. A land cell's planes are the ground's;
//! the generator only asks about the sea.
//!
//! The file: `PBDSEAN1`, the grid level and the cell count as u32, then the
//! three planes of little-endian f32, cell by cell. It is frozen once a
//! version ships: a retuned atmosphere makes a later version's file.

use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S, START_HOUR, YEAR_DAYS};
use pbd_core::planet_gen::TerrainConfig;
use std::time::Instant;

const SAMPLES_PER_DAY: u64 = 8;
const FREEZES_C: f32 = -1.8;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "assets/climate/sea-normals-g7.bin".into());
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/climate");
    let settled = std::fs::read(format!("{root}/settled-g6-l5.bin")).expect("the settled state");
    let made_with: AtmosphereSettings = ron::from_str(
        &std::fs::read_to_string(format!("{root}/settled-g6-l5.ron")).expect("its settings"),
    )
    .expect("its settings parse");
    let settings = AtmosphereSettings::default();
    assert_eq!(
        made_with, settings,
        "the settled state was made with the shipped settings"
    );
    let terrain = TerrainConfig::TENEBRIS_V6;
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    air.restore(&settled).expect("the settled state restores");
    let cells = air.grid.len();
    let days = YEAR_DAYS as u64;
    println!("{cells} cells; one year of {days} days from the settled state");

    let mut cold = vec![f32::INFINITY; cells];
    let mut warm = vec![f32::NEG_INFINITY; cells];
    let mut frozen = vec![0u32; cells];
    let mut day_sum = vec![0.0f64; cells];
    let steps_per_day = (DAY_S / settings.dt_s) as u64;
    let every = steps_per_day / SAMPLES_PER_DAY;
    let start = f64::from(START_HOUR) / 24.0 * f64::from(DAY_S);
    let run = Instant::now();
    for day in 0..days {
        day_sum.iter_mut().for_each(|s| *s = 0.0);
        for step in 0..steps_per_day {
            let t = start + (day * steps_per_day + step) as f64 * settings.dt_s as f64;
            air.step(Clock { seconds: t }.sun(), &[]);
            if step % every == 0 {
                for (s, k) in day_sum.iter_mut().zip(&air.ground_k) {
                    *s += *k as f64;
                }
            }
        }
        for i in 0..cells {
            let mean = (day_sum[i] / SAMPLES_PER_DAY as f64) as f32;
            cold[i] = cold[i].min(mean);
            warm[i] = warm[i].max(mean);
            if mean <= FREEZES_C {
                frozen[i] += 1;
            }
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
    let mut bytes = b"PBDSEAN1".to_vec();
    for v in [settings.level, cells as u32] {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    for plane in [
        cold.clone(),
        warm.clone(),
        frozen.iter().map(|&d| d as f32 / days as f32).collect(),
    ] {
        for v in plane {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    std::fs::write(&out, bytes).expect("the normals could be written");
    let sea: Vec<usize> = (0..cells).filter(|&i| air.surface.ocean[i]).collect();
    let reef = sea.iter().filter(|&&i| cold[i] > 23.0).count();
    let ice = sea
        .iter()
        .filter(|&&i| frozen[i] * 4 >= days as u32)
        .count();
    let kelp = sea
        .iter()
        .filter(|&&i| warm[i] >= 5.0 && warm[i] <= 20.0)
        .count();
    println!(
        "wrote {out} in {:.0} s: of {} sea cells, coldest above 23 C {reef}, frozen a quarter of the year {ice}, warmest 5 to 20 C {kelp}",
        run.elapsed().as_secs_f64(),
        sea.len()
    );
}
