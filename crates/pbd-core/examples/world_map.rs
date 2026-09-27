//! The world map's rasters, drawn by the function the game's map will call
//! (`pbd_core::map::base_texel`). The measurement instrument for
//! `openspec/changes/world-map` task 1.1 and `bigger-biomes` task 1.1: the
//! map mockup is drawn from what it writes, and nothing in the game reads it.
//!
//!     cargo run --release -p pbd-core --example world_map -- [out.bin] [width]
//!
//! It writes an equirectangular raster, 2,048 x 1,024 by default, of
//! little-endian f32 planes after a header (`PBDMAP01`, width, height, plane
//! count as u32). Row 0 is the north edge and column 0 is 180 degrees west,
//! as `fish_ranges` lays its rasters out, so the two overlay each other.
//! The planes are:
//!
//! 1. the surface altitude, m, floored to the layer as the column is;
//! 2. the top block, as its `Material` code;
//! 3. the biome, as its `Biome` code, on today's generator;
//! 4. onward, the biome for each moisture scale `bigger-biomes` weighs, with
//!    its thresholds retuned to a third each (the owner, survey B2).
//!
//! It prints how long the base raster took, and the moisture's quantiles
//! over the temperate land for each scale, which `bigger-biomes` quotes.
//! `tools/world_map.py` draws the PNGs from it.

use glam::Vec3;
use pbd_core::map::base_texel;
use pbd_core::planet_gen::{Biome, TerrainConfig, biome_at, moisture};
use std::f32::consts::{PI, TAU};
use std::io::Write;
use std::time::Instant;

/// The moisture scales `bigger-biomes` decision 1 weighs, in metres: today,
/// twice as wide, and four times (the owner's choice, survey B1).
const SCALES: [f32; 3] = [188.0, 375.0, 750.0];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "world_map.bin".into());
    let width: usize = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(2048);
    let height = width / 2;
    let cfg = TerrainConfig::TENEBRIS;
    let directions: Vec<Vec3> = (0..height)
        .flat_map(|row| (0..width).map(move |col| pixel(col, row, width, height)))
        .collect();

    // The base raster, timed on one thread: what the game's build on the
    // async pool will cost, before it is split up.
    let start = Instant::now();
    let texels: Vec<_> = directions.iter().map(|&d| base_texel(&cfg, d)).collect();
    let took = start.elapsed();
    println!(
        "base raster {width} x {height}: {:.2} s on one thread, {:.2} us a texel",
        took.as_secs_f64(),
        took.as_secs_f64() * 1e6 / texels.len() as f64
    );

    let mut planes: Vec<Vec<f32>> = vec![
        texels.iter().map(|t| t.altitude_m).collect(),
        texels.iter().map(|t| t.top as u16 as f32).collect(),
        texels.iter().map(|t| t.biome as u32 as f32).collect(),
    ];
    report("today, 188 m at 0.36 / 0.64", &planes[2], &directions);

    // The land that reaches the moisture split: the ocean, the beach, the
    // tundra and the mountains are decided before moisture is read, and do
    // not change with its scale.
    let temperate: Vec<bool> = texels
        .iter()
        .map(|t| {
            matches!(
                t.biome,
                Biome::Fields | Biome::Desert | Biome::Jungle | Biome::Swamp
            )
        })
        .collect();
    println!();
    println!(
        "| moisture_m | q10 | q33 | q50 | q67 | q90 | desert_below | wet_above | fields | desert | jungle | swamp |"
    );
    println!(
        "| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"
    );
    for scale in SCALES {
        let at_scale = TerrainConfig {
            moisture_m: scale,
            ..cfg
        };
        // Weighted by the cosine of the latitude, which is each pixel's share
        // of the sphere's area.
        let mut samples: Vec<(f32, f32)> = directions
            .iter()
            .zip(&temperate)
            .filter(|(_, t)| **t)
            .map(|(&d, _)| (moisture(&at_scale, d), area(d)))
            .collect();
        samples.sort_by(|a, b| a.0.total_cmp(&b.0));
        let q = |p: f32| quantile(&samples, p);
        let tuned = TerrainConfig {
            desert_below: q(1.0 / 3.0),
            wet_above: q(2.0 / 3.0),
            ..at_scale
        };
        let biomes: Vec<f32> = directions
            .iter()
            .zip(&texels)
            .map(|(&d, t)| biome_at(&tuned, d, t.altitude_m) as u32 as f32)
            .collect();
        let shares = temperate_shares(&biomes, &directions);
        println!(
            "| {scale:.0} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.0}% | {:.0}% | {:.0}% | {:.0}% |",
            q(0.10),
            q(1.0 / 3.0),
            q(0.50),
            q(2.0 / 3.0),
            q(0.90),
            tuned.desert_below,
            tuned.wet_above,
            shares[0] * 100.0,
            shares[1] * 100.0,
            shares[2] * 100.0,
            shares[3] * 100.0
        );
        planes.push(biomes);
    }
    println!();

    let mut file = std::io::BufWriter::new(std::fs::File::create(&out).expect("create the output"));
    file.write_all(b"PBDMAP01").unwrap();
    for n in [width as u32, height as u32, planes.len() as u32] {
        file.write_all(&n.to_le_bytes()).unwrap();
    }
    for plane in &planes {
        for v in plane {
            file.write_all(&v.to_le_bytes()).unwrap();
        }
    }
    println!("wrote {out}: {} planes", planes.len());
}

/// The direction through the centre of a pixel, as `fish_ranges` has it.
fn pixel(col: usize, row: usize, width: usize, height: usize) -> Vec3 {
    let lat = (0.5 - (row as f32 + 0.5) / height as f32) * PI;
    let lon = ((col as f32 + 0.5) / width as f32 - 0.5) * TAU;
    Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin())
}

/// A pixel's share of the sphere, up to a constant.
fn area(direction: Vec3) -> f32 {
    (1.0 - direction.y * direction.y).max(0.0).sqrt()
}

/// The weighted `p` quantile of samples sorted by value.
fn quantile(sorted: &[(f32, f32)], p: f32) -> f32 {
    let total: f32 = sorted.iter().map(|s| s.1).sum();
    let mut run = 0.0;
    for &(value, weight) in sorted {
        run += weight;
        if run >= p * total {
            return value;
        }
    }
    sorted.last().map_or(0.5, |s| s.0)
}

/// Fields, desert, jungle and swamp as shares of the temperate land.
fn temperate_shares(biomes: &[f32], directions: &[Vec3]) -> [f32; 4] {
    let mut sums = [0.0f64; 4];
    for (&b, &d) in biomes.iter().zip(directions) {
        let slot = match b as u32 {
            x if x == Biome::Fields as u32 => 0,
            x if x == Biome::Desert as u32 => 1,
            x if x == Biome::Jungle as u32 => 2,
            x if x == Biome::Swamp as u32 => 3,
            _ => continue,
        };
        sums[slot] += area(d) as f64;
    }
    let total: f64 = sums.iter().sum();
    sums.map(|s| (s / total.max(1e-9)) as f32)
}

/// Every biome's share of the land, for the design's table.
fn report(label: &str, biomes: &[f32], directions: &[Vec3]) {
    let names = [
        "ocean",
        "beach",
        "fields",
        "desert",
        "jungle",
        "swamp",
        "mountains",
        "tundra",
    ];
    let mut sums = [0.0f64; 8];
    for (&b, &d) in biomes.iter().zip(directions) {
        sums[b as usize] += area(d) as f64;
    }
    let land: f64 = sums[1..].iter().sum();
    let total: f64 = sums.iter().sum();
    print!("{label}: land {:.0}% of the surface;", land / total * 100.0);
    for (name, sum) in names.iter().zip(sums).skip(1) {
        print!(" {name} {:.0}%", sum / land * 100.0);
    }
    println!(" (of the land)");
    let shares = temperate_shares(biomes, directions);
    println!(
        "  temperate land: fields {:.0}%, desert {:.0}%, jungle {:.0}%, swamp {:.0}%",
        shares[0] * 100.0,
        shares[1] * 100.0,
        shares[2] * 100.0,
        shares[3] * 100.0
    );
}
