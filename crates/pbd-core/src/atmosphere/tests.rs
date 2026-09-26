//! The atmosphere's contracts: deterministic, saved and restored whole,
//! pressure evening out, storms turning the way the spin says, coasts that no
//! current crosses, lightning that discharges and pushes air out, the slider
//! brewing a storm where it is pointed, and nothing going non-finite.

use super::*;
use crate::planet_gen::TerrainConfig;

/// A small, quick planet: level 3 is 642 cells.
fn quiet() -> AtmosphereSettings {
    AtmosphereSettings {
        level: 3,
        ..Default::default()
    }
}

fn air(settings: AtmosphereSettings) -> Atmosphere {
    Atmosphere::new(&TerrainConfig::TENEBRIS, settings, 99)
}

const SUN: Vec3 = Vec3::new(0.8, 0.4, 0.45);

#[test]
fn two_runs_from_one_seed_agree_to_the_bit() {
    let mut a = air(quiet());
    let mut b = air(quiet());
    for _ in 0..80 {
        a.step(SUN, &[]);
        b.step(SUN, &[]);
    }
    assert_eq!(a.to_bytes(), b.to_bytes());
}

#[test]
fn saved_and_resumed_equals_stepped_straight_on() {
    let mut straight = air(quiet());
    for _ in 0..40 {
        straight.step(SUN, &[]);
    }
    let saved = straight.to_bytes();
    let mut resumed = air(quiet());
    resumed.restore(&saved).expect("its own bytes");
    for _ in 0..40 {
        straight.step(SUN, &[]);
        resumed.step(SUN, &[]);
    }
    assert_eq!(straight.to_bytes(), resumed.to_bytes());
    assert_eq!(resumed.step, 80);
}

#[test]
fn a_damaged_save_is_refused_whole() {
    let mut a = air(quiet());
    let before = a.to_bytes();
    assert!(a.restore(&before[..before.len() - 4]).is_err());
    let mut other = before.clone();
    other[8] ^= 1; // the level
    assert!(a.restore(&other).is_err());
    assert_eq!(a.to_bytes(), before, "a refused restore changes nothing");
}

/// No belts, no heat-driven pressure, no storms: a single bump released on a
/// planet at rest spreads out, and its variance falls.
fn dynamics_only() -> AtmosphereSettings {
    AtmosphereSettings {
        belt_pressure: 0.0,
        thermal_pressure: 0.0,
        storm_rate: 0.0,
        strike_chance: 0.0,
        pressure_relax_s: 1.0e9,
        ..quiet()
    }
}

fn variance(a: &Atmosphere) -> f64 {
    let mean = mean_phi(a);
    (0..a.grid.len())
        .map(|i| ((a.phi[i] as f64 - mean).powi(2)) * a.grid.area[i] as f64)
        .sum::<f64>()
}

fn mean_phi(a: &Atmosphere) -> f64 {
    let area: f64 = a.grid.area.iter().map(|&x| x as f64).sum();
    (0..a.grid.len())
        .map(|i| (a.phi[i] * a.grid.area[i]) as f64)
        .sum::<f64>()
        / area
}

#[test]
fn a_pressure_bump_spreads_and_evens_out() {
    let mut a = air(dynamics_only());
    a.phi.iter_mut().for_each(|p| *p = 0.0);
    a.wind.iter_mut().for_each(|w| *w = Vec3::ZERO);
    let at = a.grid.nearest(Vec3::new(1.0, 0.3, 0.2).normalize());
    a.phi[at] = 400.0;
    let start = (variance(&a), mean_phi(&a));
    for _ in 0..200 {
        a.step(SUN, &[]);
    }
    let end = (variance(&a), mean_phi(&a));
    assert!(end.0 < start.0 * 0.2, "variance {} -> {}", start.0, end.0);
    assert!(
        (end.1 - start.1).abs() < start.1.abs() * 0.1 + 1e-3,
        "mean pressure {} -> {}",
        start.1,
        end.1
    );
}

/// Relative vorticity at a cell, per second, from the circulation of the
/// wind round its sides.
fn vorticity(a: &Atmosphere, field: &[Vec3], cell: usize) -> f32 {
    let turned: Vec<Vec3> = (0..a.grid.len())
        .map(|i| a.grid.centre[i].cross(field[i]))
        .collect();
    -a.grid.divergence(&turned, cell, |_| false)
}

/// Air falling into a low turns with the spin: cyclonic, its vorticity the
/// sign of the Coriolis parameter, which here is negative over +Y.
#[test]
fn a_low_turns_the_way_the_spin_says() {
    for y in [0.7f32, -0.7] {
        let mut a = air(dynamics_only());
        a.phi.iter_mut().for_each(|p| *p = 0.0);
        a.wind.iter_mut().for_each(|w| *w = Vec3::ZERO);
        let d = Vec3::new(0.7, y, 0.1).normalize();
        let at = a.grid.nearest(d);
        for k in 0..a.grid.len() {
            let along = a.grid.centre[k].dot(a.grid.centre[at]);
            if along > 0.995 {
                a.phi[k] = -300.0 * (along - 0.995) / 0.005;
            }
        }
        for _ in 0..300 {
            a.step(SUN, &[]);
        }
        let zeta = vorticity(&a, &a.wind, at);
        let f = step::coriolis(&a.settings, a.grid.centre[at]);
        assert!(zeta * f > 0.0, "at y {y}: vorticity {zeta}, f {f}");
    }
}

#[test]
fn no_current_runs_into_a_coast() {
    let mut a = air(quiet());
    for _ in 0..300 {
        a.step(SUN, &[]);
    }
    for i in 0..a.grid.len() {
        if !a.surface.ocean[i] {
            assert_eq!(a.current[i], Vec3::ZERO, "land cell {i} has a current");
            continue;
        }
        for side in 0..a.grid.sides[i] as usize {
            let k = a.grid.neighbour[i][side] as usize;
            if !a.surface.ocean[k] {
                let into = a.current[i].dot(a.grid.normal[i][side]);
                assert!(into < 0.05, "cell {i} flows {into} m/s into the coast");
            }
        }
    }
}

fn total(area: &[f32], field: &[f32]) -> f64 {
    area.iter().zip(field).map(|(a, v)| (a * v) as f64).sum()
}

#[test]
fn moving_water_neither_makes_nor_loses_any() {
    let a = air(quiet());
    let wind: Vec<Vec3> = a
        .grid
        .centre
        .iter()
        .map(|c| (Vec3::new(0.3, 1.0, -0.2).cross(*c) * 15.0) + Vec3::new(4.0, 0.0, 2.0))
        .map(|w| w - Vec3::ZERO)
        .collect();
    let fluxes = a.grid.fluxes(&wind, |_| false);
    let mut water = a.vapour.clone();
    let before = total(&a.grid.area, &water);
    for _ in 0..100 {
        water = a.grid.upwind(&water, &fluxes, 1.0, true);
    }
    let after = total(&a.grid.area, &water);
    assert!(
        ((after - before) / before).abs() < 1e-4,
        "{before} -> {after}"
    );
    assert!(water.iter().all(|w| *w >= 0.0));
}

#[test]
fn a_strike_discharges_and_pushes_air_out() {
    let settings = AtmosphereSettings {
        strike_chance: 1.0,
        storm_rate: 0.0,
        ..quiet()
    };
    let mut a = air(settings);
    a.wind.iter_mut().for_each(|w| *w = Vec3::ZERO);
    let at = a.grid.nearest(Vec3::new(0.2, 0.3, 1.0).normalize());
    a.charge.iter_mut().for_each(|c| *c = 0.0);
    a.charge[at] = 3.0;
    a.cloud[at] = 1.0;
    a.step(SUN, &[]);
    assert!(
        a.strikes.iter().any(|s| s.direction == a.grid.centre[at]),
        "it struck"
    );
    assert!(a.charge[at] < 0.5, "and discharged: {}", a.charge[at]);
    for _ in 0..3 {
        a.step(SUN, &[]);
    }
    let out = a.grid.divergence(&a.wind, at, |_| false);
    assert!(out > 0.0, "the cold pool's air flows out: divergence {out}");
}

#[test]
fn a_calm_sky_does_not_strike() {
    let mut a = air(AtmosphereSettings {
        storm_rate: 0.0,
        ..quiet()
    });
    a.charge.iter_mut().for_each(|c| *c = 0.0);
    a.cloud.iter_mut().for_each(|c| *c = 0.0);
    a.vapour.iter_mut().for_each(|v| *v = 0.0);
    a.step(SUN, &[]);
    assert!(a.strikes.is_empty());
}

#[test]
fn the_slider_brews_a_storm_where_it_points() {
    let mut a = air(AtmosphereSettings {
        forcing_radius_m: 1500.0,
        ..quiet()
    });
    let here = Vec3::new(-0.3, 0.2, 0.9).normalize();
    let far = -here;
    for _ in 0..40 {
        a.step(
            SUN,
            &[Forcing {
                direction: here,
                strength: 1.0,
            }],
        );
    }
    assert!(a.sample(here).cover > 0.8, "cover {}", a.sample(here).cover);
    assert!(a.sample(far).cover < a.sample(here).cover);
}

#[test]
fn nothing_goes_non_finite_at_the_extremes() {
    let mut a = air(AtmosphereSettings {
        storm_rate: 1.0e-2,
        storm_depth: 2000.0,
        latent_k_per_kg: 3.0,
        strike_chance: 1.0,
        charge_rate: 50.0,
        coriolis_scale: 20.0,
        ..quiet()
    });
    let all = [Forcing {
        direction: Vec3::Y,
        strength: 1.0,
    }];
    for step in 0..400 {
        let sun = Vec3::new((step as f32 * 0.05).cos(), 0.3, (step as f32 * 0.05).sin());
        a.step(sun, &all);
    }
    let bytes = a.to_bytes();
    let mut back = air(a.settings);
    back.restore(&bytes)
        .expect("every value finite, so the state restores");
    let sample = a.sample(Vec3::new(0.3, 0.4, 0.5));
    assert!(sample.cover.is_finite() && sample.wind.is_finite() && sample.temperature.is_finite());
}

#[test]
fn a_sample_reads_the_cells_under_it() {
    let mut a = air(quiet());
    for _ in 0..30 {
        a.step(SUN, &[]);
    }
    for i in (0..a.grid.len()).step_by(37) {
        let s = a.sample(a.grid.centre[i]);
        assert!((s.cloud - a.cloud[i]).abs() < 1e-3, "cell {i}");
        assert!((s.cover - a.cover(i)).abs() < 1e-3);
        assert!((0.0..=1.0).contains(&s.humidity));
    }
}

/// A still planet: no wind, no storms, no noise, so a test controls every
/// term the water step reads.
fn still() -> Atmosphere {
    let mut a = air(AtmosphereSettings {
        storm_rate: 0.0,
        mesoscale: 0.0,
        belt_pressure: 0.0,
        thermal_pressure: 0.0,
        ..quiet()
    });
    for w in &mut a.wind {
        *w = Vec3::ZERO;
    }
    a.phi.fill(0.0);
    a.cloud.fill(0.0);
    a
}

#[test]
fn humid_air_is_partly_cloudy_and_does_not_rain() {
    let mut a = still();
    let s = a.settings;
    // The Sundqvist curve: nothing up to the critical humidity, everything at
    // saturation, rising between; and the land's threshold is the higher.
    assert_eq!(a.humid_cover(s.humid_cover_rh, 1.0), 0.0);
    assert!((a.humid_cover(1.0, 1.0) - 1.0).abs() < 1e-6);
    assert!(a.humid_cover(0.8, 1.0) > a.humid_cover(0.7, 1.0));
    assert!(a.humid_cover(0.8, 1.0) > a.humid_cover(0.8, 0.0));
    // Humid air held below saturation, with nothing rising: cloudy, dry.
    for i in 0..a.grid.len() {
        let saturation = a.saturation(a.air_k[i] - s.lapse_k_per_m * a.surface.elevation[i]);
        a.vapour[i] = 0.85 * saturation;
    }
    let sea = (0..a.grid.len())
        .find(|&i| a.surface.ocean[i])
        .expect("the planet has sea");
    assert!(a.cover(sea) > 0.4, "cover {}", a.cover(sea));
    assert_eq!(a.cloud[sea], 0.0, "that cover is not condensed water");
    // Night everywhere the sea cell is, so nothing is lifted by the sun.
    a.step(-a.grid.centre[sea], &[]);
    assert_eq!(a.rain_rate[sea], 0.0, "humid air alone does not rain");
}

#[test]
fn sunlit_land_lifts_the_air_and_night_land_does_not() {
    let a = still();
    let land = (0..a.grid.len())
        .find(|&i| !a.surface.ocean[i] && a.surface.slope[i].length() < 1e-3)
        .or_else(|| (0..a.grid.len()).find(|&i| !a.surface.ocean[i]))
        .expect("the planet has land");
    let overhead = a.grid.centre[land];
    let mut day = a.clone();
    let mut night = a.clone();
    day.step(overhead, &[]);
    night.step(-overhead, &[]);
    assert!(
        day.lift[land] > night.lift[land] + 1.0,
        "noon lift {} against midnight {}",
        day.lift[land],
        night.lift[land]
    );
}

#[test]
fn cold_cloud_rains_out_of_less_water_than_warm() {
    let mut a = still();
    let sea: Vec<usize> = (0..a.grid.len()).filter(|&i| a.surface.ocean[i]).collect();
    let (cold, warm) = (sea[0], sea[sea.len() / 2]);
    let s = a.settings;
    // The same cloud water, well under the warm threshold, in air at -20 C
    // and at 25 C; the vapour just saturated so nothing condenses or dries.
    for (i, k) in [(cold, -20.0), (warm, 25.0)] {
        a.air_k[i] = k;
        a.ground_k[i] = k;
        a.cloud[i] = 0.5 * s.rain_threshold_kg;
        a.vapour[i] = a.saturation(k - s.lapse_k_per_m * a.surface.elevation[i]);
    }
    a.step(-Vec3::Y, &[]);
    assert!(a.rain_rate[cold] > 0.0, "cold cloud snows");
    assert_eq!(
        a.rain_rate[warm], 0.0,
        "the same water in warm cloud does not rain"
    );
}

/// The menu's RAIN preset (0.6) rains where it points. Its storm is a storm
/// because of the updraft the forcing drives: once a warm cloud needed 4 kg/m^2
/// to rain, a slider that only added cloud water brought full cover and no rain.
#[test]
fn the_rain_preset_rains() {
    let here = Vec3::new(-0.3, 0.2, 0.9).normalize();
    let rain = |lift: f32| {
        let mut a = air(AtmosphereSettings {
            forcing_radius_m: 1500.0,
            forcing_lift_mps: lift,
            ..quiet()
        });
        for _ in 0..40 {
            a.step(
                SUN,
                &[Forcing {
                    direction: here,
                    strength: 0.6,
                }],
            );
        }
        (a.sample(here).rain_rate, a.settings.raining_rate)
    };
    let (with_updraft, raining) = rain(AtmosphereSettings::default().forcing_lift_mps);
    assert!(
        with_updraft > raining,
        "rain {with_updraft} against the raining rate {raining}"
    );
    // And it is the updraft that does it: the same cloud without one is dry.
    let (without, _) = rain(0.0);
    assert!(without < raining, "rain {without} with no updraft");
}

/// The cloud pace slows the cloud and nothing else: one carry at half pace
/// moves cloud half as far as at full pace (the upwind flux is linear in the
/// wind), at nought not at all, and vapour, heat, charge and the wind itself
/// are carried identically whatever the pace (`calm-clouds`).
#[test]
fn the_cloud_pace_slows_the_cloud_and_nothing_else() {
    let mut spun = air(quiet());
    for _ in 0..120 {
        spun.step(SUN, &[]);
    }
    let carried = |pace: f32| {
        let mut a = spun.clone();
        a.settings.cloud_pace = pace;
        a.carry(1.0);
        a
    };
    let full = carried(1.0);
    let half = carried(0.5);
    let none = carried(0.0);
    assert_eq!(full.vapour, half.vapour);
    assert_eq!(full.air_k, half.air_k);
    assert_eq!(full.charge, half.charge);
    assert_eq!(full.wind, half.wind);
    assert_eq!(none.cloud, spun.cloud, "at nought the cloud stays put");
    let moved = |a: &Atmosphere| -> f64 {
        a.cloud
            .iter()
            .zip(&spun.cloud)
            .map(|(x, y)| (x - y).abs() as f64)
            .sum()
    };
    let (full_moved, half_moved) = (moved(&full), moved(&half));
    assert!(full_moved > 0.0, "the cloud has to move at full pace");
    let ratio = half_moved / full_moved;
    assert!(
        (ratio - 0.5).abs() < 0.01,
        "half pace moved the cloud {ratio:.3} as far as full pace"
    );
}

/// The waves follow the wind with a lag, and only over the sea.
#[test]
fn the_sea_state_follows_the_wind_over_the_sea_with_a_lag() {
    let mut a = air(quiet());
    let n = a.grid.len();
    let sea = (0..n).find(|&i| a.surface.ocean[i]).expect("a sea cell");
    let land = (0..n).find(|&i| !a.surface.ocean[i]).expect("a land cell");
    let dt = a.settings.dt_s;
    let tau = a.settings.sea_build_s;
    a.wind[sea] = a.grid.centre[sea].any_orthonormal_vector() * 10.0;
    a.sea[sea] = 0.0;
    a.waves(dt);
    let one = 10.0 * (1.0 - (-dt / tau).exp());
    assert!((a.sea[sea] - one).abs() < 1e-4, "{} vs {one}", a.sea[sea]);
    for _ in 0..(5.0 * tau / dt) as usize {
        a.waves(dt);
    }
    assert!((a.sea[sea] - 10.0).abs() < 0.1);
    a.wind[land] = a.grid.centre[land].any_orthonormal_vector() * 10.0;
    a.waves(dt);
    assert_eq!(a.sea[land], 0.0);
}

/// A save from before the sea state existed still loads; its sea is taken to
/// have caught up with its wind.
#[test]
fn a_save_from_before_the_sea_state_still_loads() {
    let mut a = air(quiet());
    for _ in 0..20 {
        a.step(SUN, &[]);
    }
    let n = a.grid.len();
    let current = a.to_bytes();
    // The old layout: the old header, and every scalar field but the last.
    let header = MAGIC.len() + 16;
    let mut old = b"PBDATM01".to_vec();
    old.extend_from_slice(&current[MAGIC.len()..header]);
    old.extend_from_slice(&current[header..header + n * 4 * (SCALARS - 1)]);
    old.extend_from_slice(&current[header + n * 4 * SCALARS..]);
    let mut b = air(quiet());
    b.restore(&old).expect("an old save");
    assert_eq!(b.wind, a.wind);
    assert_eq!(b.sea, b.settled_sea());
}
