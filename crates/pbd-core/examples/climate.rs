//! The climate report: spin a world's atmosphere up, run it, and print what
//! its weather does, band by band. The measurement instrument for
//! `openspec/changes/atmospheric-circulation`; its numbers go in that design.
//!
//!     cargo run --release -p pbd-core --example climate -- [days] [day-of-year] [level]
//!
//! Latitudes are the sine of +Y's; "prograde" wind blows the way the ground
//! turns (Earth's westerly).

use glam::Vec3;
use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S};
use pbd_core::planet_gen::TerrainConfig;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: f64 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(1.0);
    let start_day: u32 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(0);
    let level: u32 = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(5);
    let terrain = TerrainConfig::TENEBRIS;
    // `ATMOSPHERE='(strike_chance: 0.0)'` overrides any knob for a run, in
    // the RON the shipped file uses.
    let mut settings: AtmosphereSettings = std::env::var("ATMOSPHERE")
        .ok()
        .map(|text| ron::from_str(&text).expect("ATMOSPHERE must be a RON struct"))
        .unwrap_or_default();
    settings.level = level;
    settings.validate().expect("legal settings");
    let built = Instant::now();
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    let start = Clock::at(start_day, 0.0).seconds;
    println!(
        "{} cells, {:.0} m apart, built in {:.0} ms",
        air.grid.len(),
        air.grid.spacing(),
        built.elapsed().as_secs_f64() * 1e3
    );
    let spun = Instant::now();
    air.spin_up(|t| Clock { seconds: t }.sun(), start);
    let spin_s = spun.elapsed().as_secs_f64();
    println!(
        "spun up {:.0} s of weather in {:.1} s ({:.2} ms a step)",
        settings.spinup_s,
        spin_s,
        spin_s * 1e3 * settings.dt_s as f64 / settings.spinup_s as f64
    );
    let steps = (days * DAY_S as f64 / settings.dt_s as f64) as u64;
    let mut report = Report::new(&air);
    let water_before = air.water_kg();
    let run = Instant::now();
    for step in 0..steps {
        let clock = Clock {
            seconds: start + step as f64 * settings.dt_s as f64,
        };
        air.step(clock.sun(), &[]);
        if step % 60 == 0 {
            report.take(&air, clock.sun());
        }
        report.strikes += air
            .strikes
            .iter()
            .filter(|s| s.step + 1 == air.step)
            .count();
    }
    let per_step = run.elapsed().as_secs_f64() * 1e3 / steps.max(1) as f64;
    report.print(&air, days);
    distribution("lift m/s", &air.lift);
    distribution("cloud kg/m2", &air.cloud);
    distribution("charge", &air.charge);
    let humidity: Vec<f32> = (0..air.grid.len())
        .map(|i| air.vapour[i] / air.saturation(air.air_k[i]))
        .collect();
    distribution("vapour/saturation", &humidity);
    let mut phi_band = [(0.0f64, 0.0f64); BANDS];
    for i in 0..air.grid.len() {
        let b = latitude_band(air.grid.centre[i]);
        phi_band[b].0 += air.phi[i] as f64;
        phi_band[b].1 += 1.0;
    }
    println!(
        "pressure by band: {}",
        phi_band
            .iter()
            .map(|(s, n)| format!("{:.0}", s / n.max(1.0)))
            .collect::<Vec<_>>()
            .join(" ")
    );
    println!(
        "step {per_step:.2} ms; water in the air {:.3} of where it started",
        air.water_kg() / water_before
    );
    if let Ok(path) = std::env::var("CLIMATE_MAP") {
        write_map(&air, &path);
        println!("map written to {path}");
    }
}

/// An equirectangular picture of the cloud over the sea and the land, 720 x
/// 360, as a binary PPM: the instrument's eyeball.
fn write_map(air: &Atmosphere, path: &str) {
    let (w, h) = (720usize, 360usize);
    let mut bytes = format!("P6 {w} {h} 255\n").into_bytes();
    for row in 0..h {
        let latitude = (0.5 - (row as f32 + 0.5) / h as f32) * std::f32::consts::PI;
        for column in 0..w {
            let longitude = ((column as f32 + 0.5) / w as f32 - 0.5) * std::f32::consts::TAU;
            let d = Vec3::new(
                latitude.cos() * longitude.cos(),
                latitude.sin(),
                latitude.cos() * longitude.sin(),
            );
            let sample = air.sample(d);
            let (cells, _) = air.grid.locate(d);
            let ocean = air.surface.ocean[cells[0] as usize];
            let ground = if ocean {
                [20.0, 50.0, 110.0]
            } else {
                [70.0, 110.0, 50.0]
            };
            let white = sample.cover;
            let grey = 1.0 - 0.35 * (sample.cloud / 1.5).min(1.0);
            for g in ground.iter() {
                let v = g * (1.0 - white) + 255.0 * grey * white;
                bytes.push(v.clamp(0.0, 255.0) as u8);
            }
        }
    }
    std::fs::write(path, bytes).expect("the map could be written");
}

const BANDS: usize = 18;

#[derive(Default, Clone, Copy)]
struct Band {
    cover: f64,
    rain: f64,
    surface_east: f64,
    surface_north: f64,
    upper_east: f64,
    ground: f64,
    sea: f64,
    sea_count: f64,
    count: f64,
}

struct Report {
    bands: [Band; BANDS],
    ocean_cover: (f64, f64),
    land_cover: (f64, f64),
    clear: f64,
    full: f64,
    cover_sum: f64,
    raining: f64,
    samples: f64,
    land_rain_by_hour: [f64; 24],
    /// Ground minus air over land, K, by local hour: what drives convection.
    land_excess_by_hour: [(f64, f64); 24],
    /// Lift, cloud water and humidity over land, by local hour.
    land_by_hour: [[f64; 4]; 24],
    vorticity: [(f64, f64); 2],
    current_max: f32,
    strikes: usize,
}

fn latitude_band(c: Vec3) -> usize {
    let latitude = c.y.clamp(-1.0, 1.0).asin().to_degrees();
    (((90.0 - latitude) / 10.0) as usize).min(BANDS - 1)
}

fn east(c: Vec3) -> Vec3 {
    c.cross(Vec3::Y).normalize_or_zero()
}

impl Report {
    fn new(_: &Atmosphere) -> Self {
        Report {
            bands: [Band::default(); BANDS],
            ocean_cover: (0.0, 0.0),
            land_cover: (0.0, 0.0),
            clear: 0.0,
            full: 0.0,
            cover_sum: 0.0,
            raining: 0.0,
            samples: 0.0,
            land_rain_by_hour: [0.0; 24],
            land_excess_by_hour: [(0.0, 0.0); 24],
            land_by_hour: [[0.0; 4]; 24],
            vorticity: [(0.0, 0.0); 2],
            current_max: 0.0,
            strikes: 0,
        }
    }

    fn take(&mut self, air: &Atmosphere, sun: Vec3) {
        let sun_azimuth = sun.z.atan2(sun.x);
        let turned: Vec<Vec3> = (0..air.grid.len())
            .map(|i| air.grid.centre[i].cross(air.current[i]))
            .collect();
        for i in 0..air.grid.len() {
            let c = air.grid.centre[i];
            let a = air.grid.area[i] as f64;
            let cover = air.cover(i) as f64;
            let b = &mut self.bands[latitude_band(c)];
            let e = east(c);
            let north = (Vec3::Y - c * c.y).normalize_or_zero();
            b.cover += cover * a;
            b.rain += air.rain_rate[i] as f64 * 3600.0 * a;
            b.surface_east += air.wind[i].dot(e) as f64 * a;
            b.surface_north += air.wind[i].dot(north) as f64 * a;
            b.upper_east += air.upper[i].dot(e) as f64 * a;
            b.ground += air.ground_k[i] as f64 * a;
            b.count += a;
            let ocean = air.surface.ocean[i];
            if ocean {
                b.sea += air.ground_k[i] as f64 * a;
                b.sea_count += a;
                self.ocean_cover.0 += cover * a;
                self.ocean_cover.1 += a;
                self.current_max = self.current_max.max(air.current[i].length());
                let latitude = c.y.asin().to_degrees();
                if (15.0..45.0).contains(&latitude.abs()) {
                    let vort = -air.grid.divergence(&turned, i, |k| !air.surface.ocean[k]) as f64;
                    let side = usize::from(latitude < 0.0);
                    self.vorticity[side].0 += vort * a;
                    self.vorticity[side].1 += a;
                }
            } else {
                self.land_cover.0 += cover * a;
                self.land_cover.1 += a;
                let azimuth = c.z.atan2(c.x);
                let hour = (12.0 + (azimuth - sun_azimuth) / std::f32::consts::TAU * 24.0)
                    .rem_euclid(24.0);
                let h = (hour as usize).min(23);
                self.land_rain_by_hour[h] += air.rain_rate[i] as f64 * a;
                self.land_excess_by_hour[h].0 += (air.ground_k[i] - air.air_k[i]) as f64 * a;
                self.land_excess_by_hour[h].1 += a;
                let humidity =
                    air.humidity_of(air.vapour[i], air.air_k[i], air.surface.elevation[i]);
                for (k, v) in [air.lift[i], air.cloud[i], humidity, 1.0]
                    .iter()
                    .enumerate()
                {
                    self.land_by_hour[h][k] += *v as f64 * a;
                }
            }
            self.clear += if cover < 0.05 { a } else { 0.0 };
            self.full += if cover > 0.95 { a } else { 0.0 };
            self.cover_sum += cover * a;
            if air.rain_rate[i] > air.settings.raining_rate {
                self.raining += a;
            }
            self.samples += a;
        }
    }

    fn print(&self, air: &Atmosphere, days: f64) {
        println!(
            "\n  lat   cover  rain mm/h  sfc prograde  sfc poleward+Y  jet prograde  ground C  sea C"
        );
        let mut covers = Vec::new();
        for (index, b) in self.bands.iter().enumerate() {
            let top = 90 - index as i32 * 10;
            let w = b.count.max(1e-9);
            covers.push(b.cover / w);
            println!(
                "{:>3}..{:<3} {:>5.2} {:>10.3} {:>13.2} {:>15.2} {:>13.2} {:>9.1} {:>6}",
                top,
                top - 10,
                b.cover / w,
                b.rain / w,
                b.surface_east / w,
                b.surface_north / w,
                b.upper_east / w,
                b.ground / w,
                if b.sea_count > 0.0 {
                    format!("{:.1}", b.sea / b.sea_count)
                } else {
                    "-".into()
                }
            );
        }
        let mean = covers.iter().sum::<f64>() / covers.len() as f64;
        let spread =
            (covers.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / covers.len() as f64).sqrt();
        println!(
            "\ncover over ocean {:.3}, over land {:.3}; clear {:.1}%, full {:.1}%; band spread {:.3}",
            self.ocean_cover.0 / self.ocean_cover.1.max(1e-9),
            self.land_cover.0 / self.land_cover.1.max(1e-9),
            100.0 * self.clear / self.samples,
            100.0 * self.full / self.samples,
            spread
        );
        println!(
            "whole planet: mean cover {:.3}, partly covered {:.1}%, raining {:.1}%",
            self.cover_sum / self.samples,
            100.0 * (self.samples - self.clear - self.full) / self.samples,
            100.0 * self.raining / self.samples
        );
        let peak = self
            .land_rain_by_hour
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map_or(0, |(h, _)| h);
        println!("land rain peaks at local hour {peak}");
        let total: f64 = self.land_rain_by_hour.iter().sum::<f64>().max(1e-12);
        println!(
            "land rain in the afternoon (12-18h) {:.0}%, before dawn (00-06h) {:.0}%",
            100.0 * self.land_rain_by_hour[12..18].iter().sum::<f64>() / total,
            100.0 * self.land_rain_by_hour[0..6].iter().sum::<f64>() / total
        );
        println!(
            "land rain by hour, % of the day's: {}",
            self.land_rain_by_hour
                .iter()
                .map(|r| format!("{:.0}", 100.0 * r / total))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let excess = |h: usize| {
            let (sum, area) = self.land_excess_by_hour[h];
            sum / area.max(1e-9)
        };
        for (name, k) in [("lift m/s", 0), ("cloud kg/m2", 1), ("humidity", 2)] {
            println!(
                "land {name} by hour 00/04/08/12/16/20: {}",
                [0, 4, 8, 12, 16, 20]
                    .iter()
                    .map(|&h| format!(
                        "{:.2}",
                        self.land_by_hour[h][k] / self.land_by_hour[h][3].max(1e-9)
                    ))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
        }
        println!(
            "land ground minus air, K: 06h {:.1}, 09h {:.1}, 12h {:.1}, 15h {:.1}, 18h {:.1}, 00h {:.1}",
            excess(6),
            excess(9),
            excess(12),
            excess(15),
            excess(18),
            excess(0)
        );
        println!(
            "ocean: fastest current {:.2} m/s; mean vorticity 15-45 deg, +Y side {:.2e}, -Y side {:.2e} /s",
            self.current_max,
            self.vorticity[0].0 / self.vorticity[0].1.max(1e-9),
            self.vorticity[1].0 / self.vorticity[1].1.max(1e-9)
        );
        println!(
            "lightning: {} strikes in {days} days; {} kept now",
            self.strikes,
            air.strikes.len()
        );
    }
}

fn distribution(name: &str, field: &[f32]) {
    let mut v = field.to_vec();
    v.sort_by(f32::total_cmp);
    let at = |q: f64| v[((v.len() - 1) as f64 * q) as usize];
    println!(
        "{name}: p10 {:.3} p50 {:.3} p90 {:.3} p99 {:.3} max {:.3}",
        at(0.1),
        at(0.5),
        at(0.9),
        at(0.99),
        v[v.len() - 1]
    );
}
