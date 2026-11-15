//! Where the strongest wind over the sea blows in the settled climate a new
//! world opens on, hour by hour through its first day: a measurement
//! instrument for the gale-spray capture (`ocean-environments` 4.3, B11).
use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S, START_HOUR};
use pbd_core::planet_gen::TerrainConfig;

fn main() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/climate");
    let settled = std::fs::read(format!("{root}/settled-g7-l5.bin")).unwrap();
    let settings = AtmosphereSettings::default();
    let terrain = TerrainConfig::TENEBRIS_V7;
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    air.restore(&settled).unwrap();
    let start = f64::from(START_HOUR) / 24.0 * f64::from(DAY_S);
    let per_hour = (DAY_S as f64 / 24.0 / settings.dt_s as f64) as u64;
    for hour in 0..=24u64 {
        let (mut best, mut at) = (0.0f32, 0usize);
        for i in 0..air.grid.len() {
            let w = air.wind[i].length();
            if air.surface.ocean[i] && w > best {
                best = w;
                at = i;
            }
        }
        let ll = pbd_core::geo::lat_lon(air.grid.centre[at]);
        let (lat, lon) = ll.degrees();
        println!(
            "hour {:>4.1}: {best:.1} m/s at --at {lat:.2} {lon:.2}",
            START_HOUR as f64 + hour as f64
        );
        for s in 0..per_hour {
            let t = start + (hour * per_hour + s) as f64 * settings.dt_s as f64;
            air.step(Clock { seconds: t }.sun(), &[]);
        }
    }
}
