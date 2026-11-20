//! A measurement instrument (`ocean-environments` design B24): where the
//! sea's cells of generator version 14 lie on the shipped seed, so a capture
//! can be taken at each. Prints the first few places of reef rock, basalt,
//! mussel beds and sea ice found on a scan of the globe, with their depth.
//!
//! `cargo run --profile fast -p pbd-core --example sea_cells`

use pbd_core::geo::{LatLon, direction};
use pbd_core::ocean::cells;
use pbd_core::planet_gen::{self, TerrainConfig};
use pbd_core::terrain::Material;

fn main() {
    let cfg = TerrainConfig::TENEBRIS_V14;
    let mut counts = [0usize; 4];
    let names = ["reef rock", "basalt", "mussel bed", "sea ice"];
    for lat10 in (-890..=890).step_by(3) {
        for lon10 in (-1800..1800).step_by(3) {
            let (lat, lon) = (lat10 as f32 / 10.0, lon10 as f32 / 10.0);
            let d = direction(LatLon {
                lat: lat.to_radians(),
                lon: lon.to_radians(),
            });
            let surface = planet_gen::surface_altitude(&cfg, d).floor();
            let kind = match cells::top(&cfg, d, surface) {
                Some(Material::ReefRock) => Some(0),
                Some(Material::Basalt) => Some(1),
                Some(Material::MusselBed) => Some(2),
                _ if cells::sea_ice(&cfg, d, surface) => Some(3),
                _ => None,
            };
            let Some(k) = kind else { continue };
            if counts[k] < 4 && (k != 3 || surface <= -4.0) {
                // The day and hour its sun stands highest: the polar sea is
                // dark half the year.
                let years = pbd_core::daylight::YEAR_DAYS as u32;
                let (day, hour) = (0..years)
                    .flat_map(|day| (0..96).map(move |q| (day, q as f32 / 4.0)))
                    .max_by(|a, b| {
                        let e = |(day, h): (u32, f32)| {
                            pbd_core::daylight::Clock::at(day, h).elevation(d)
                        };
                        e(*a).total_cmp(&e(*b))
                    })
                    .unwrap_or((0, 12.0));
                println!(
                    "{}: --at {lat:.4} {lon:.4} --day {day} --time {hour:.2}, floor {surface} m",
                    names[k]
                );
            }
            counts[k] += 1;
        }
    }
    for (name, n) in names.iter().zip(counts) {
        println!("{name}: {n} samples");
    }
}
