//! A measurement instrument for `mountain-ground` and `sand-tiles`' captures:
//! places on the shipped body to point `--at` at, and the share of version
//! 15's mountain cells each top takes.
//!
//! `cargo run -p pbd-core --example mountain_places --release`

use pbd_core::geo;
use pbd_core::mountain::{MountainGround, relief};
use pbd_core::planet_gen::{Biome, TerrainConfig, biome_at, surface_altitude, top_material};
use pbd_core::terrain::Material;

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
    let v15 = TerrainConfig::TENEBRIS_V15;
    let m = MountainGround::V15;
    let mut counts = std::collections::BTreeMap::new();
    let mut shown = std::collections::BTreeMap::new();
    for d in sphere(2_000_000) {
        let f = surface_altitude(&v15, d).floor();
        let biome = biome_at(&v15, d, f);
        let at = |d| {
            let (lat, lon) = geo::lat_lon(d).degrees();
            format!("--at {lat:.4} {lon:.4}  ({f} m)")
        };
        let mut note = |key: &'static str, d| {
            let n = shown.entry(key).or_insert(0);
            if *n < 3 {
                println!("{key:22} {}", at(d));
                *n += 1;
            }
        };
        match biome {
            Biome::Mountains => {
                let top = top_material(&v15, d, f);
                *counts.entry(format!("{top:?}")).or_insert(0) += 1;
                let r = relief(&v15, d, f);
                if top == Material::Grass && (118.0..132.0).contains(&f) {
                    note("meadow (forest band)", d);
                }
                if top == Material::Grass && (142.0..152.0).contains(&f) {
                    note("meadow (tree line)", d);
                }
                if r.flat && (f - m.snow_line(&v15, d)).abs() < 6.0 {
                    note("flat at the snow line", d);
                }
                if r.scree && top == Material::Stone && f < 150.0 {
                    note("scree", d);
                }
            }
            Biome::Desert
                if d.y.abs() < 0.6 && top_material(&v15, d, f) == Material::Sand && f > 20.0 =>
            {
                if relief(&v15, d, f).flat {
                    note("desert sand (flat)", d);
                }
            }
            Biome::Beach
                if f >= 0.0 && d.y.abs() < 0.6 && top_material(&v15, d, f) == Material::Sand =>
            {
                note("beach sand", d)
            }
            _ => {}
        }
    }
    let total: usize = counts.values().sum();
    for (top, n) in counts {
        println!(
            "{top:8} {:5.1}% of version 15's mountain cells",
            100.0 * n as f32 / total as f32
        );
    }
}
