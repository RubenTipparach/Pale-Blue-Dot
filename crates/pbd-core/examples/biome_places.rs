//! A measurement instrument for `memory-leak` (towns at 60 fps, the biome
//! sweep; owner, 2026-11-26: "It could also be a number of biomes adding
//! lag"): for each biome, the nearest place to Holbrook's spawn well inside
//! it, to point `--at` at, so one fixed showcase can be run in each.
//!
//! `cargo run -p pbd-core --example biome_places --profile fast`

use pbd_core::geo;
use pbd_core::planet_gen::{Biome, TerrainConfig, biome_at, surface_altitude};

/// Holbrook's spawn, as the route and the soak start there.
const SPAWN: (f32, f32) = (29.5703, 1.36795);

fn sphere(n: usize) -> impl Iterator<Item = glam::Vec3> {
    let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
    (0..n).map(move |i| {
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
        let r = (1.0 - y * y).max(0.0).sqrt();
        let t = golden * i as f32;
        glam::Vec3::new(r * t.cos(), y, r * t.sin())
    })
}

fn main() {
    let cfg = TerrainConfig::TENEBRIS;
    let radius = cfg.radius_m;
    let here = |d: glam::Vec3| {
        let f = surface_altitude(&cfg, d).floor();
        (biome_at(&cfg, d, f), f)
    };
    // The spawn's direction: the sample whose lat-long is nearest it.
    let mut spawn = glam::Vec3::Y;
    let mut best = f32::MAX;
    for d in sphere(400_000) {
        let (lat, lon) = geo::lat_lon(d).degrees();
        let e = (lat as f32 - SPAWN.0).powi(2) + (lon as f32 - SPAWN.1).powi(2);
        if e < best {
            best = e;
            spawn = d;
        }
    }
    let mut nearest: std::collections::BTreeMap<String, (f32, glam::Vec3, f32)> =
        Default::default();
    for d in sphere(4_000_000) {
        let metres = spawn.angle_between(d) * radius;
        if metres > 40_000.0 {
            continue;
        }
        let (biome, f) = here(d);
        let key = format!("{biome:?}");
        if nearest.get(&key).is_some_and(|(m, ..)| *m <= metres) {
            continue;
        }
        // Well inside it: the same biome round it on every side, 150 m, or
        // 30 m for the narrow ones (a beach's rim, a swamp's pools).
        let east = glam::Vec3::Y.cross(d).normalize_or(glam::Vec3::X);
        let north = d.cross(east);
        let inside = (0..8).all(|k| {
            let a = k as f32 * std::f32::consts::FRAC_PI_4;
            let reach = if matches!(biome, Biome::Beach | Biome::Swamp) { 30.0 } else { 150.0 };
            let off = (east * a.cos() + north * a.sin()) * (reach / radius);
            here((d + off).normalize()).0 == biome
        });
        if inside && (biome == Biome::Ocean || f > 1.0) {
            nearest.insert(key, (metres, d, f));
        }
    }
    for (biome, (metres, d, f)) in nearest {
        let (lat, lon) = geo::lat_lon(d).degrees();
        println!("{biome:10} --at {lat:.4} {lon:.4}  {metres:6.0} m from the spawn, ground {f} m");
    }
}
