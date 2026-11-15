//! Where the sea's places are on generator version 7 (`ocean-environments`
//! design B10): a measurement instrument. It samples the sphere, asks
//! `ocean::places::environment` at each point, and prints each place's share
//! of the sea and a few spots to stand on (latitude and longitude in degrees,
//! for `pbd-app --at`, and the depth), for the captures and the write-up.
//!
//!     cargo run --profile fast -p pbd-core --example ocean_places -- [samples]

use glam::Vec3;
use pbd_core::fauna::Environment;
use pbd_core::ocean::places::environment_at;
use pbd_core::planet_gen::{TerrainConfig, surface_altitude};
use std::collections::BTreeMap;

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(400_000);
    if std::env::args().nth(2).is_some() {
        let a: Vec<f32> = std::env::args()
            .skip(2)
            .map(|s| s.parse().unwrap())
            .collect();
        check(a[0], a[1]);
        return;
    }
    let cfg = TerrainConfig::for_version(7).expect("version 7");
    // The depth band a capture wants for each place.
    let band = |env: Environment| match env {
        Environment::Reef => (2.0, 4.5),
        Environment::Kelp => (8.0, 16.0),
        Environment::UnderIce => (4.0, 20.0),
        Environment::Coast => (0.5, 1.2),
        Environment::Vent => (90.0, 125.0),
        Environment::Trench => (100.0, 145.0),
        _ => (10.0, 40.0),
    };
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut spots: BTreeMap<String, Vec<(f32, f32, f32)>> = BTreeMap::new();
    let mut sea = 0usize;
    let mut deepest = 0.0f32;
    for i in 0..n {
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let t = golden * i as f32;
        let d = Vec3::new(r * t.cos(), y, r * t.sin());
        let h = surface_altitude(&cfg, d);
        deepest = deepest.min(h);
        let Some(env) = environment_at(&cfg, d, h) else {
            continue;
        };
        sea += 1;
        let name = format!("{env:?}");
        *counts.entry(name.clone()).or_default() += 1;
        let depth = cfg.sea_level_m - h;
        let (lo, hi) = band(env);
        let list = spots.entry(name).or_default();
        if depth >= lo && depth <= hi && list.len() < 6 {
            let ll = pbd_core::geo::lat_lon(d);
            let (lat, lon) = ll.degrees();
            list.push((lat, lon, depth));
        }
    }
    println!("{n} samples, {sea} at sea; the deepest floor {deepest:.1} m");
    for (name, count) in &counts {
        println!(
            "{name:>9}: {count:>7} ({:.2}% of the sea)",
            100.0 * *count as f64 / sea as f64
        );
        for (lat, lon, depth) in spots.get(name).into_iter().flatten() {
            println!("           --at {lat:.3} {lon:.3}   depth {depth:.1} m");
        }
    }
}

#[allow(dead_code)]
fn check(lat: f32, lon: f32) {
    let cfg = TerrainConfig::for_version(7).unwrap();
    let d = pbd_core::geo::direction(pbd_core::geo::LatLon {
        lat: lat.to_radians(),
        lon: lon.to_radians(),
    });
    let h = surface_altitude(&cfg, d);
    println!(
        "{lat} {lon}: altitude {h:.1}, {:?}",
        environment_at(&cfg, d, h)
    );
}
