//! The world map mockup's weather: every overlay the globe can show, read
//! off the simulated atmosphere and drawn onto the map's equirectangular
//! raster through a day. The measurement instrument for
//! `openspec/changes/world-map` tasks 1.3 and 1.3b (the live layer and the
//! weather overlays, decision 8); nothing in the game reads it. The game's map
//! samples the same cells from `Air`'s cube maps instead.
//!
//!     cargo run --release -p pbd-core --example map_weather -- [days] [out.bin]
//!     ATMOSPHERE='(solar_wm2: 1360.0)' cargo run ... # retuned
//!
//! It runs the weather `days` from a new world, then records FRAMES frames a
//! game day apart by DAY_S / FRAMES seconds. Each value is
//! `Overlay::texel`, the one reading the globe's overlay uses, so the map
//! and the globe cannot disagree about what an overlay shows.
//!
//! It writes little-endian after a header (`PBDWTHR2`, then width, height,
//! frame count and overlay count as u32). Then, for each overlay in
//! `Overlay::ALL`'s order, what the page needs to draw it, so nothing about an
//! overlay is restated downstream:
//! - its name, 16 bytes, and its unit, 8 bytes, ASCII padded with zeros;
//! - the scalar its ramp's two ends stand for, two f32;
//! - its ramp's row in `Ramp::ALL`, and flags (1: it flows, 2: it fades), u32.
//!
//! Then, for each frame and each overlay, the scalar as f32 planes, and for an
//! overlay that flows, its eastward and northward components, m/s, two more.
//! Row 0 is north and column 0 is 180 degrees west, as the other map rasters
//! are. `tools/world_map.py --weather` draws them.

use glam::Vec3;
use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S};
use pbd_core::overlay::Overlay;
use pbd_core::planet_gen::TerrainConfig;
use std::f32::consts::{PI, TAU};
use std::io::Write;

const WIDTH: usize = 720;
const HEIGHT: usize = 360;
const FRAMES: usize = 12;

fn padded<const N: usize>(text: &str) -> [u8; N] {
    let mut bytes = [0u8; N];
    for (slot, b) in bytes.iter_mut().zip(text.bytes()) {
        *slot = b;
    }
    bytes
}

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

    // Each pixel's direction, and its local east and north, for the flows.
    let pixels: Vec<(Vec3, Vec3, Vec3)> = (0..HEIGHT)
        .flat_map(|row| {
            (0..WIDTH).map(move |col| {
                let lat = (0.5 - (row as f32 + 0.5) / HEIGHT as f32) * PI;
                let lon = ((col as f32 + 0.5) / WIDTH as f32 - 0.5) * TAU;
                let (sl, cl, so, co) = (lat.sin(), lat.cos(), lon.sin(), lon.cos());
                let direction = Vec3::new(cl * co, sl, cl * so);
                let east = Vec3::new(-so, 0.0, co);
                let north = Vec3::new(-sl * co, cl, -sl * so);
                (direction, east, north)
            })
        })
        .collect();
    let mut file = std::io::BufWriter::new(std::fs::File::create(&out).expect("create the output"));
    file.write_all(b"PBDWTHR2").unwrap();
    for n in [WIDTH, HEIGHT, FRAMES, Overlay::ALL.len()] {
        file.write_all(&(n as u32).to_le_bytes()).unwrap();
    }
    for overlay in Overlay::ALL {
        file.write_all(&padded::<16>(overlay.name())).unwrap();
        file.write_all(&padded::<8>(overlay.unit())).unwrap();
        let (lo, hi) = overlay.range();
        file.write_all(&lo.to_le_bytes()).unwrap();
        file.write_all(&hi.to_le_bytes()).unwrap();
        file.write_all(&(overlay.ramp().index() as u32).to_le_bytes())
            .unwrap();
        let flags = u32::from(overlay.flows()) | (u32::from(overlay.fades()) << 1);
        file.write_all(&flags.to_le_bytes()).unwrap();
    }
    let mut plane = |values: &mut dyn Iterator<Item = f32>| {
        for v in values {
            file.write_all(&v.to_le_bytes()).unwrap();
        }
    };
    for frame in 0..FRAMES {
        if frame > 0 {
            advance(&mut air, &mut step, steps_per_day / FRAMES as u64);
        }
        let samples: Vec<_> = pixels.iter().map(|(d, _, _)| air.sample(*d)).collect();
        for overlay in Overlay::ALL {
            let texels: Vec<[f32; 4]> = samples.iter().map(|s| overlay.texel(s)).collect();
            plane(&mut texels.iter().map(|t| t[0]));
            if overlay.flows() {
                let flow = |t: &[f32; 4]| Vec3::new(t[1], t[2], t[3]);
                plane(&mut texels.iter().zip(&pixels).map(|(t, p)| flow(t).dot(p.1)));
                plane(&mut texels.iter().zip(&pixels).map(|(t, p)| flow(t).dot(p.2)));
            }
        }
        println!(
            "frame {frame}: day {:.2}",
            step as f64 * settings.dt_s as f64 / DAY_S as f64
        );
    }
    println!("wrote {out}");
}
