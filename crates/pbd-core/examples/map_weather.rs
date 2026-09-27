//! The world map mockup's live weather: the simulated atmosphere's cloud and
//! rain, drawn onto the map's equirectangular raster through a day. The
//! measurement instrument for `openspec/changes/world-map` task 1.3 (the live
//! layer); nothing in the game reads it. The game's map samples the same
//! cells from `Air`'s cube maps instead.
//!
//!     cargo run --release -p pbd-core --example map_weather -- [days] [out.bin]
//!     ATMOSPHERE='(solar_wm2: 1360.0)' cargo run ... # retuned
//!
//! It runs the weather `days` from a new world, then records FRAMES frames a
//! game day apart by DAY_S / FRAMES seconds. It writes little-endian f32
//! planes after a header (`PBDWTHR1`, width, height, frame count as u32):
//! for each frame the cloud cover (0..1), then the rain rate (kg/m^2/s),
//! then the surface temperature (deg C). Row 0 is north and column 0 is 180
//! degrees west, as the other map rasters are. `tools/world_map.py
//! --weather` draws them.

use glam::Vec3;
use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S};
use pbd_core::planet_gen::TerrainConfig;
use std::f32::consts::{PI, TAU};
use std::io::Write;

const WIDTH: usize = 720;
const HEIGHT: usize = 360;
const FRAMES: usize = 12;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: u64 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(20);
    let out = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| "map_weather.bin".into());
    let terrain = TerrainConfig::TENEBRIS;
    let settings: AtmosphereSettings = std::env::var("ATMOSPHERE")
        .ok()
        .map(|text| ron::from_str(&text).expect("ATMOSPHERE must be a RON struct"))
        .unwrap_or_default();
    settings.validate().expect("legal settings");
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    air.spin_up(|t| Clock { seconds: t }.sun(), 0.0);
    let steps_per_day = (DAY_S / settings.dt_s) as u64;
    let mut step = 0u64;
    let advance = |air: &mut Atmosphere, step: &mut u64, steps: u64| {
        for _ in 0..steps {
            let t = *step as f64 * settings.dt_s as f64;
            air.step(Clock { seconds: t }.sun(), &[]);
            *step += 1;
        }
    };
    advance(&mut air, &mut step, days * steps_per_day);
    println!(
        "ran {days} days: whole surface {:.2} C, trim {:.3}",
        air.mean_surface_c(),
        air.sun_trim
    );

    let directions: Vec<Vec3> = (0..HEIGHT)
        .flat_map(|row| {
            (0..WIDTH).map(move |col| {
                let lat = (0.5 - (row as f32 + 0.5) / HEIGHT as f32) * PI;
                let lon = ((col as f32 + 0.5) / WIDTH as f32 - 0.5) * TAU;
                Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin())
            })
        })
        .collect();
    let mut file = std::io::BufWriter::new(std::fs::File::create(&out).expect("create the output"));
    file.write_all(b"PBDWTHR1").unwrap();
    for n in [WIDTH as u32, HEIGHT as u32, FRAMES as u32] {
        file.write_all(&n.to_le_bytes()).unwrap();
    }
    for frame in 0..FRAMES {
        if frame > 0 {
            advance(&mut air, &mut step, steps_per_day / FRAMES as u64);
        }
        let samples: Vec<_> = directions.iter().map(|&d| air.sample(d)).collect();
        for plane in [
            samples.iter().map(|s| s.cover).collect::<Vec<_>>(),
            samples.iter().map(|s| s.rain_rate).collect(),
            samples.iter().map(|s| s.temperature).collect(),
        ] {
            for v in plane {
                file.write_all(&v.to_le_bytes()).unwrap();
            }
        }
        println!(
            "frame {frame}: day {:.2}",
            step as f64 * settings.dt_s as f64 / DAY_S as f64
        );
    }
    println!("wrote {out}");
}
