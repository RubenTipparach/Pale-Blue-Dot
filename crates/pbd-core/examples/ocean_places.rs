//! Where the sea's places are (`ocean-environments` design B10, B30): a
//! measurement instrument. It samples the sphere, asks
//! `ocean::places::environment` at each point, and prints each place's share
//! of the sea, the share of the sea whose top cell is reef rock
//! (`ocean::cells::top`, version 14 and later), and a few spots to stand on
//! (latitude and longitude in degrees, for `pbd-app --at`, and the depth),
//! for the captures and the write-up.
//!
//!     cargo run --profile fast -p pbd-core --example ocean_places -- [samples]
//!     cargo run --profile fast -p pbd-core --example ocean_places -- [samples] <lat> <lon>
//!
//! `GEN=<version>` picks the generator (the newest by default). For tuning
//! (B30), these override the version's numbers without editing it:
//! `REEF_DEPTH`, `REEF_COLD`, `REEF_PATCH`, `REEF_PATCH_DEPTH`,
//! `REEF_PATCH_ABOVE`, `REEF_OVER_KELP`, `KELP_WARM_HI`, `KELP_LO`, `REEF_ROCK_PATCH`,
//! `REEF_ABOVE`. `HIST=1` also prints the sea's area by depth and coldest
//! day's temperature.

use glam::Vec3;
use pbd_core::fauna::Environment;
use pbd_core::ocean::{cells, places::OceanPlaces, places::SeaNormals, places::environment_at};
use pbd_core::planet_gen::{TerrainConfig, surface_altitude};
use pbd_core::terrain::Material;
use std::collections::BTreeMap;

fn var(name: &str) -> Option<f32> {
    std::env::var(name).ok().and_then(|v| v.parse().ok())
}

fn config() -> TerrainConfig {
    let version = var("GEN").map_or(pbd_core::terrain::GENERATOR_VERSION, |v| v as u32);
    let mut cfg = TerrainConfig::for_version(version).expect("a carried version");
    if let Some(p) = cfg.ocean.as_mut() {
        let p: &mut OceanPlaces = p;
        if let Some(v) = var("REEF_DEPTH") {
            p.reef_depth_m = v;
        }
        if let Some(v) = var("REEF_COLD") {
            p.reef_coldest_c = v;
        }
        if let Some(v) = var("REEF_PATCH") {
            p.reef_patch_m = v;
        }
        if let Some(v) = var("REEF_PATCH_DEPTH") {
            p.reef_patch_depth_m = v;
        }
        if let Some(v) = var("REEF_PATCH_ABOVE") {
            p.reef_patch_above = v;
        }
        if let Some(v) = var("KELP_WARM_HI") {
            p.kelp_warmest_c.1 = v;
        }
        if let Some(v) = var("REEF_OVER_KELP") {
            p.reef_over_kelp_c = v;
        }
        if let Some(v) = var("KELP_LO") {
            p.kelp_depth_m.0 = v;
        }
    }
    if let Some(c) = cfg.sea_cells.as_mut() {
        if let Some(v) = var("REEF_ROCK_PATCH") {
            c.reef_patch_m = v;
        }
        if let Some(v) = var("REEF_ABOVE") {
            c.reef_above = v;
        }
    }
    cfg
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(400_000);
    let cfg = config();
    if std::env::args().nth(2).is_some() {
        let a: Vec<f32> = std::env::args()
            .skip(2)
            .map(|s| s.parse().unwrap())
            .collect();
        check(&cfg, a[0], a[1]);
        return;
    }
    // The depth band a capture wants for each place.
    let band = |env: Environment| match env {
        Environment::Reef => (2.0, 4.5),
        Environment::Kelp => (8.0, 16.0),
        Environment::UnderIce => (0.3, 1.5),
        Environment::Coast => (0.5, 1.2),
        Environment::Vent => (90.0, 125.0),
        Environment::Trench => (100.0, 145.0),
        _ => (10.0, 40.0),
    };
    let hist = std::env::var("HIST").is_ok();
    let depth_bins = [2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 16.0, 20.0, 40.0, f32::MAX];
    let cold_bins = [16.0, 18.0, 19.0, 20.0, f32::MAX];
    let mut grid = [[0usize; 5]; 10];
    let mut kelp_by_cold = [0usize; 8];
    let kelp_cold_bins = [8.0, 10.0, 12.0, 14.0, 15.0, 16.0, 17.0, f32::MAX];
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut spots: BTreeMap<String, Vec<(f32, f32, f32)>> = BTreeMap::new();
    let mut sea = 0usize;
    let mut reef_rock = 0usize;
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
        if cells::top(&cfg, d, h.floor()) == Some(Material::ReefRock) {
            reef_rock += 1;
        }
        let depth = cfg.sea_level_m - h;
        if hist && let Some(places) = cfg.ocean.as_ref() {
            let (coldest, _, _) = SeaNormals::of(places, cfg.radius_m).at(d);
            let di = depth_bins.iter().position(|b| depth < *b).unwrap();
            let ci = cold_bins.iter().position(|b| coldest < *b).unwrap();
            grid[di][ci] += 1;
            if env == Environment::Kelp {
                kelp_by_cold[kelp_cold_bins.iter().position(|b| coldest < *b).unwrap()] += 1;
            }
        }
        let name = format!("{env:?}");
        *counts.entry(name.clone()).or_default() += 1;
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
    println!(
        "reef rock top: {reef_rock:>7} ({:.2}% of the sea)",
        100.0 * reef_rock as f64 / sea as f64
    );
    if hist {
        println!("% of the sea by depth (rows, m, under) and coldest day (columns, C, under)");
        println!("         <16     <18     <19     <20    >=20");
        for (di, row) in grid.iter().enumerate() {
            print!(
                "{:>6}",
                if di == 9 {
                    ">40".into()
                } else {
                    format!("{}", depth_bins[di])
                }
            );
            for v in row {
                print!(" {:>7.2}", 100.0 * *v as f64 / sea as f64);
            }
            println!();
        }
        println!("kelp, % of the sea, by coldest day (C, under): 8 10 12 14 15 16 17 rest");
        for v in kelp_by_cold {
            print!(" {:>6.2}", 100.0 * v as f64 / sea as f64);
        }
        println!();
    }
}

fn check(cfg: &TerrainConfig, lat: f32, lon: f32) {
    let d = pbd_core::geo::direction(pbd_core::geo::LatLon {
        lat: lat.to_radians(),
        lon: lon.to_radians(),
    });
    let h = surface_altitude(cfg, d);
    println!(
        "{lat} {lon}: altitude {h:.1}, {:?}",
        environment_at(cfg, d, h)
    );
}
