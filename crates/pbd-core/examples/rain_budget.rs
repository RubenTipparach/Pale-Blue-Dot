//! The rain budget: how much of the planet rains, and how hard, over a span
//! of world days, from the shipped settled climate. The measurement
//! instrument for `openspec/changes/rain-matches-map` (the owner, 2026-11-16:
//! "50% less rain on the planet, and avoid rain cells over desert").
//!
//!     cargo run --profile fast -p pbd-core --example rain_budget -- [days] [level] [generator]
//!
//! It restores `assets/climate/settled-g<generator>-l<level>.bin` and runs it
//! forward on the running settings (`ATMOSPHERE='(evaporation: 1.5e-4)'`
//! overrides any knob, as `examples/climate.rs` takes it), skipping the first
//! world day while the new rules take hold, and samples every 60 s of world
//! time. It prints, area-weighted over the whole planet and over the desert:
//! - the share of the area raining (the rate at or over `raining_rate`);
//! - the mean rain rate, mm/h;
//! - the mean rain seen, 0..1 (what a player stands in);
//! - the mean cloud cover.

use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S};
use pbd_core::planet_gen::{self, Biome, TerrainConfig};

#[derive(Default)]
struct Tally {
    area: f64,
    raining: f64,
    rate_mmh: f64,
    seen: f64,
    cover: f64,
    /// The heaviest rate seen anywhere in the set, mm/h.
    peak_mmh: f64,
    /// Area raining (not snowing, as the sample decides) where the air at
    /// the ground is at or below freezing: rain in the cold.
    cold_rain: f64,
    /// Area precipitating at all (rain or snow).
    falling: f64,
    /// Area where what falls is snow.
    snowing: f64,
    /// The air at the ground where anything falls, deg C, area-summed.
    falling_air_c: f64,
    /// Each sample of it, for its percentiles.
    falling_air: Vec<f32>,
}

impl Tally {
    fn add(&mut self, air: &Atmosphere, i: usize, area: f64) {
        let rate = f64::from(air.rain_rate[i]) * 3600.0;
        self.area += area;
        if air.rain_rate[i] >= air.settings.raining_rate {
            self.raining += area;
        }
        if air.rain_seen[i] > pbd_core::weather::RAIN_SEEN_MIN {
            self.falling += area;
            let sample = air.sample(air.grid.centre[i]);
            let falls = pbd_core::weather::shown_precipitation(&sample);
            if falls > 0.0 && sample.air_c <= 0.0 {
                self.cold_rain += area;
            }
            if falls < 0.0 {
                self.snowing += area;
            }
            self.falling_air_c += f64::from(sample.air_c) * area;
            self.falling_air.push(sample.air_c);
        }
        self.rate_mmh += rate * area;
        self.seen += f64::from(air.rain_seen[i]) * area;
        self.cover += f64::from(air.cover(i)) * area;
        self.peak_mmh = self.peak_mmh.max(rate);
    }

    fn print(&self, name: &str) {
        let a = self.area.max(1e-12);
        println!(
            "{name}: raining {:.2}% of the area, mean rate {:.4} mm/h, mean rain seen {:.4}, \
             mean cover {:.3}, peak {:.2} mm/h",
            100.0 * self.raining / a,
            self.rate_mmh / a,
            self.seen / a,
            self.cover / a,
            self.peak_mmh
        );
        println!(
            "{name}: of the area where anything falls, {:.2}% is snow and {:.2}% rain in air \
             at or below 0 C; the air there averages {:.1} C",
            100.0 * self.snowing / self.falling.max(1e-12),
            100.0 * self.cold_rain / self.falling.max(1e-12),
            self.falling_air_c / self.falling.max(1e-12)
        );
        let mut air = self.falling_air.clone();
        air.sort_by(f32::total_cmp);
        if !air.is_empty() {
            let at = |q: usize| air[(air.len() * q / 100).min(air.len() - 1)];
            println!(
                "{name}: the air where anything falls, C: 10th {:.1}, 25th {:.1}, median {:.1}, \
                 75th {:.1}, 90th {:.1}",
                at(10),
                at(25),
                at(50),
                at(75),
                at(90)
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: f64 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(30.0);
    let level: u32 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(5);
    let generator: u32 = args
        .get(3)
        .and_then(|a| a.parse().ok())
        .unwrap_or(pbd_core::terrain::GENERATOR_VERSION);
    let terrain = TerrainConfig::for_version(generator).expect("a carried generator");
    let mut settings: AtmosphereSettings = std::env::var("ATMOSPHERE")
        .ok()
        .map(|text| ron::from_str(&text).expect("ATMOSPHERE must be a RON struct"))
        .unwrap_or_default();
    settings.level = level;
    settings.validate().expect("legal settings");
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    let path = format!(
        "{}/../../assets/climate/settled-g{generator}-l{level}.bin",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytes = std::fs::read(&path).expect("the shipped settled climate");
    air.restore(&bytes).expect("a settled state for this grid");
    let biome: Vec<Option<Biome>> = air
        .grid
        .centre
        .iter()
        .map(|&c| {
            let height = planet_gen::surface_altitude(&terrain, c);
            (height >= terrain.sea_level_m).then(|| planet_gen::biome_at(&terrain, c, height))
        })
        .collect();
    let desert: Vec<bool> = biome.iter().map(|b| *b == Some(Biome::Desert)).collect();
    let tundra: Vec<bool> = biome.iter().map(|b| *b == Some(Biome::Tundra)).collect();
    println!(
        "level {level}, generator {generator}: {} cells, {} desert, {} tundra; {days} days",
        air.grid.len(),
        desert.iter().filter(|d| **d).count(),
        tundra.iter().filter(|d| **d).count()
    );
    let start = pbd_core::daylight::START_HOUR as f64 / 24.0 * DAY_S as f64;
    let dt = f64::from(settings.dt_s);
    let steps = (days * f64::from(DAY_S) / dt).round() as u64;
    let warm = (f64::from(DAY_S) / dt).round() as u64;
    let every = (60.0 / dt).round().max(1.0) as u64;
    let (mut planet, mut dry, mut cold) = (Tally::default(), Tally::default(), Tally::default());
    let started = std::time::Instant::now();
    for step in 0..steps + warm {
        let t = start + (step + 1) as f64 * dt;
        air.step(Clock { seconds: t }.sun(), &[]);
        if step < warm || step % every != 0 {
            continue;
        }
        for i in 0..air.grid.len() {
            let area = f64::from(air.grid.area[i]);
            planet.add(&air, i, area);
            if desert[i] {
                dry.add(&air, i, area);
            }
            if tundra[i] {
                cold.add(&air, i, area);
            }
        }
    }
    planet.print("whole planet");
    dry.print("desert");
    cold.print("tundra");
    println!(
        "mean surface {:.2} C, sun trim {:.3}; {:.0} s",
        air.mean_surface_c(),
        air.sun_trim,
        started.elapsed().as_secs_f64()
    );
}
