//! A measurement instrument (`ocean-environments` design B25): where a
//! beaver colony can settle on the shipped seed, so a capture can be taken
//! at one. Prints river points at 40 to 60 degrees of latitude on Fields
//! ground whose reach's water steps down within 30 m downstream: the lips
//! the app's `beaver::sites_near` looks for.
//!
//! `cargo run --profile fast -p pbd-core --example beaver_sites`

use glam::Vec3;
use pbd_core::geo::{LatLon, direction};
use pbd_core::planet_gen::{self, Biome, TerrainConfig};
use pbd_core::rivers;

fn step(d: Vec3, along: Vec3, metres: f32, radius: f32) -> Vec3 {
    (d + along * (metres / radius)).normalize()
}

fn main() {
    let cfg = TerrainConfig::TENEBRIS;
    let radius = cfg.radius_m;
    let mut found = 0;
    'scan: for lat10 in (400..=600).step_by(2) {
        for lon10 in (-1800..1800).step_by(2) {
            for south in [1.0f32, -1.0] {
                let lat = south * lat10 as f32 / 10.0;
                let lon = lon10 as f32 / 10.0;
                let d = direction(LatLon {
                    lat: lat.to_radians(),
                    lon: lon.to_radians(),
                });
                let Some(here) = rivers::at(&cfg, d) else {
                    continue;
                };
                if here.across_m > here.half_width_m * 0.3 {
                    continue;
                }
                let bank = step(
                    d,
                    here.upstream.cross(d).normalize(),
                    here.half_width_m + 4.0,
                    radius,
                );
                let ground = planet_gen::surface_altitude(&cfg, bank);
                if planet_gen::biome_at(&cfg, bank, ground) != Biome::Fields {
                    continue;
                }
                // Its lip within 30 m downstream.
                let mut at = d;
                let mut up = here.upstream;
                let mut lip = None;
                for k in 0..21 {
                    let next = step(at, -up, 1.4165, radius);
                    let Some(there) = rivers::at(&cfg, next) else {
                        break;
                    };
                    if there.level < here.level {
                        lip = Some((k as f32 * 1.4165, at));
                        break;
                    }
                    at = next;
                    up = there.upstream;
                }
                let Some((lip_m, lip_at)) = lip else {
                    continue;
                };
                // How deep the water stands at the lip, in layers: a beaver
                // dams no more than four.
                let bed = planet_gen::surface_altitude(&cfg, lip_at).floor();
                let deep = here.level as f32 - (bed - cfg.sea_level_m);
                if !(1.0..=4.0).contains(&deep) {
                    continue;
                }
                println!(
                    "--at {lat:.4} {lon:.4} --time {:.2}  level {} m, half width {:.1} m, lip {lip_m:.1} m downstream, {deep} m deep there",
                    noon(d),
                    here.level,
                    here.half_width_m
                );
                found += 1;
                if found >= 12 {
                    break 'scan;
                }
            }
        }
    }
    println!("{found} found");
}

/// The hour of day 0 the sun stands highest over `d`: its local noon, for
/// `--time`.
fn noon(d: glam::Vec3) -> f32 {
    (0..96)
        .map(|q| q as f32 / 4.0)
        .max_by(|a, b| {
            let e = |h: f32| pbd_core::daylight::Clock::at(0, h).elevation(d);
            e(*a).total_cmp(&e(*b))
        })
        .unwrap_or(12.0)
}
