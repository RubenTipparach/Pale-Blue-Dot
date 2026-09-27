//! Where the surface's heat goes: the planet's area-weighted mean surface
//! heat budget, once a game day. The measurement instrument for
//! `openspec/changes/climate-balance` (its design, "Measured"); its numbers go
//! in that design, and nothing in the game reads it.
//!
//!     cargo run --release -p pbd-core --example heat_budget -- [days]
//!     ATMOSPHERE='(level: 4, solar_wm2: 1360.0)' cargo run ... # retuned
//!
//! Every term is W/m^2 over the whole surface, land and sea, averaged over the
//! day. The heat step (`atmosphere/step.rs`, `heat`) takes absorbed sunlight
//! in and emitted longwave (less the cloud's returned share) and sensible heat
//! to the air out; `stored` is the ground's and sea's change of heat.
//!
//! Two terms move heat between cells and would add nothing if they kept the
//! heat total, so they are measured rather than assumed: `spread`, the heat
//! the step conducts between neighbours (`Grid::conduct`), and `carry`, the
//! sea's heat moved by the current (`ocean.rs`, `carry_sea`). Each is computed
//! from the sampled state with the step's own functions. `evap` is the residual
//! of the ground's own budget: what else left the ground, which is evaporation
//! (the `water` stage).
//!
//! The planet's books (`climate-balance` decision 1a) count the air as a heat
//! store of `air_heat_capacity` and the vapour's latent heat as a store of
//! `latent_j_per_kg` a kilogram. `air` and `latent` are their changes, and
//! `leak` is what the planet gained or lost that the sun and the longwave do
//! not account for: absorbed, less emitted, less every store's change.

use pbd_core::atmosphere::{Atmosphere, AtmosphereSettings};
use pbd_core::daylight::{Clock, DAY_S};
use pbd_core::planet_gen::TerrainConfig;

/// Steps between samples: often enough that the day's mean is the day's.
const EVERY: u64 = 10;

// One cell's terms read eight arrays by its index, so the loop is by index.
#[allow(clippy::needless_range_loop)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let days: u64 = args.get(1).and_then(|a| a.parse().ok()).unwrap_or(30);
    let terrain = TerrainConfig::TENEBRIS;
    let settings: AtmosphereSettings = std::env::var("ATMOSPHERE")
        .ok()
        .map(|text| ron::from_str(&text).expect("ATMOSPHERE must be a RON struct"))
        .unwrap_or_default();
    settings.validate().expect("legal settings");
    let s = settings;
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    air.spin_up(|t| Clock { seconds: t }.sun(), 0.0);
    let n = air.grid.len();
    let total_area: f64 = air.grid.area.iter().map(|&a| a as f64).sum();
    let sea_area: f64 = (0..n)
        .filter(|&i| air.surface.ocean[i])
        .map(|i| air.grid.area[i] as f64)
        .sum();
    println!(
        "{n} cells; solar {} W/m^2, cloud albedo {}, cloud greenhouse {} W/m^2",
        s.solar_wm2, s.cloud_albedo, s.cloud_greenhouse
    );
    println!(
        "{:>4} {:>7} {:>7} {:>6} | {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} | {:>8} {:>8} {:>8} {:>9} {:>8}",
        "day",
        "mean C",
        "sea C",
        "cover",
        "absorbed",
        "cloud-",
        "emitted",
        "cloud+",
        "sensible",
        "stored",
        "spread",
        "carry",
        "evap",
        "air",
        "latent",
        "leak",
        "air carry",
        "pools"
    );
    let steps_per_day = (DAY_S / s.dt_s) as u64;
    let sample_dt = EVERY as f64 * s.dt_s as f64;
    let mut prev = air.ground_k.clone();
    let mut prev_air = air.air_k.clone();
    let mut prev_vapour = air.vapour.clone();
    let air_capacity = s.air_heat_capacity() as f64;
    let latent = s.latent_j_per_kg() as f64;
    let ocean = air.surface.ocean.clone();
    let land = |k: usize| !ocean[k];
    for day in 0..days {
        // absorbed, cloud-reflected, emitted, cloud greenhouse, sensible,
        // stored, temp, sea temp, cover, spread, carry, air, latent
        let mut acc = [0.0f64; 14];
        let mut samples = 0u32;
        // The heat the lightning's cold pools take from the air, J.
        let mut pools = 0.0f64;
        for step in 0..steps_per_day {
            let t = (day * steps_per_day + step) as f64 * s.dt_s as f64;
            air.step(Clock { seconds: t }.sun(), &[]);
            let struck = air
                .strikes
                .iter()
                .filter(|k| k.step + 1 == air.step)
                .count();
            pools += struck as f64 * s.pool_k as f64 * air_capacity * total_area / n as f64;
            if step % EVERY != 0 {
                continue;
            }
            let mut v = [0.0f64; 14];
            // The sea's carry as `carry_sea` makes it, from this state.
            let fluxes = air.grid.fluxes(&air.current, land);
            let carried = air.grid.upwind(&air.ground_k, &fluxes, s.dt_s, false);
            // The air's temperature as `carry` moves it, in the advective form.
            let winds = air.grid.fluxes(&air.wind, |_| false);
            let air_carried = air.grid.upwind(&air.air_k, &winds, s.dt_s, false);
            let conducted = air.grid.conduct(
                &air.ground_k,
                &air.surface.heat_capacity,
                s.spread_per_s(air.grid.mean_span),
            );
            for i in 0..n {
                let a = air.grid.area[i] as f64;
                let t_c = air.ground_k[i] as f64;
                let cover = air.cover(i) as f64;
                let lost = 1.0 - s.cloud_albedo as f64 * cover;
                let sunlight = air.sunlight[i] as f64;
                let keep = 1.0 - air.surface.albedo[i] as f64;
                let clear = if lost > 1e-6 { sunlight / lost } else { 0.0 };
                v[0] += a * sunlight * keep;
                v[1] += a * clear * s.cloud_albedo as f64 * cover * keep;
                v[2] += a * (s.olr_a as f64 + s.olr_b as f64 * t_c);
                v[3] += a * s.cloud_greenhouse as f64 * cover;
                v[4] += a * s.sensible_wm2k as f64 * (t_c - air.air_k[i] as f64);
                v[5] +=
                    a * air.surface.heat_capacity[i] as f64 * (t_c - prev[i] as f64) / sample_dt;
                v[6] += a * t_c;
                if air.surface.ocean[i] {
                    v[7] += a * t_c;
                }
                v[8] += a * cover;
                let capacity = air.surface.heat_capacity[i] as f64;
                v[9] += a * capacity * conducted[i] as f64;
                if air.surface.ocean[i] {
                    v[10] += a * capacity * (carried[i] - air.ground_k[i]) as f64 / s.dt_s as f64;
                }
                v[11] += a * air_capacity * (air.air_k[i] - prev_air[i]) as f64 / sample_dt;
                v[12] += a * latent * (air.vapour[i] - prev_vapour[i]) as f64 / sample_dt;
                v[13] += a * air_capacity * (air_carried[i] - air.air_k[i]) as f64 / s.dt_s as f64;
            }
            prev.copy_from_slice(&air.ground_k);
            prev_air.copy_from_slice(&air.air_k);
            prev_vapour.copy_from_slice(&air.vapour);
            for k in 0..14 {
                acc[k] += v[k] / if k == 7 { sea_area } else { total_area };
            }
            samples += 1;
        }
        let m: Vec<f64> = acc.iter().map(|x| x / samples as f64).collect();
        // The first sample of the day measured the change since the last
        // sample of the day before, the same stride, so nothing is dropped.
        let evap = m[0] - (m[2] - m[3]) - m[4] - m[5] + m[9] + m[10];
        let leak = m[0] - (m[2] - m[3]) - m[5] - m[11] - m[12];
        println!(
            "{:>4} {:>7.2} {:>7.2} {:>6.3} | {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} {:>8.1} | {:>8.1} {:>8.1} {:>8.1} {:>9.1} {:>8.1}",
            day + 1,
            m[6],
            m[7],
            m[8],
            m[0],
            m[1],
            m[2],
            m[3],
            m[4],
            m[5],
            m[9],
            m[10],
            evap,
            m[11],
            m[12],
            leak,
            m[13],
            -pools / (steps_per_day as f64 * s.dt_s as f64) / total_area
        );
    }
}
