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
    // And no thermostat, which came after.
    old.extend_from_slice(&current[header + n * 4 * SCALARS..current.len() - 12]);
    let mut b = air(quiet());
    b.restore(&old).expect("an old save");
    assert_eq!(b.wind, a.wind);
    assert_eq!(b.sea, b.settled_sea());
}

/// A save from before the thermostat (`PBDATM02`) loads with no trim built up,
/// and is saved again in the new layout.
#[test]
fn a_save_from_before_the_thermostat_loads_with_no_trim() {
    let mut a = air(quiet());
    for _ in 0..20 {
        a.step(SUN, &[]);
    }
    a.trim_integral = 0.1;
    a.balance_absorbed = 230.0;
    a.balance_greenhouse = 20.0;
    let current = a.to_bytes();
    let mut old = b"PBDATM02".to_vec();
    old.extend_from_slice(&current[MAGIC.len()..current.len() - 12]);
    let mut b = air(quiet());
    b.restore(&old).expect("a save from before the thermostat");
    assert_eq!(b.trim_integral, 0.0);
    assert_eq!(b.ground_k, a.ground_k);
    assert_eq!(&b.to_bytes()[..MAGIC.len()], MAGIC, "saved again as new");
    let mut c = air(quiet());
    c.restore(&current).expect("its own bytes");
    assert_eq!(c.trim_integral, 0.1, "and the new layout keeps the trim");
    assert_eq!((c.balance_absorbed, c.balance_greenhouse), (230.0, 20.0));
    // One from before the balance keeps its integral and starts its averages.
    let mut v3 = b"PBDATM03".to_vec();
    v3.extend_from_slice(&current[MAGIC.len()..current.len() - 8]);
    let mut d = air(quiet());
    d.restore(&v3).expect("a save from before the balance");
    assert_eq!(d.trim_integral, 0.1);
    assert_eq!(d.balance_absorbed, 0.0);
}

/// A planet whose heat answers in hours, lit by a sun that stands still: the
/// thermostat alone decides where its mean settles.
fn quick_planet(start_c: f32) -> Atmosphere {
    let mut a = air(AtmosphereSettings {
        solar_wm2: 1100.0,
        land_heat_capacity: 2.0e4,
        ocean_heat_capacity: 2.0e4,
        sun_balance_s: 2_000.0,
        sun_trim_s: 2_000.0,
        sun_trim_per_k: 0.02,
        ..quiet()
    });
    a.ground_k.fill(start_c);
    a.air_k.fill(start_c);
    a
}

/// `climate-balance` task 2.2: held too cold, the planet warms to the
/// target; too warm, it cools to it. Its local climate is its own.
#[test]
fn the_thermostat_warms_a_cold_planet_and_cools_a_warm_one_to_its_target() {
    for start in [-10.0, 40.0] {
        let mut a = quick_planet(start);
        for _ in 0..100_000 {
            a.heat(SUN, a.settings.dt_s);
        }
        let mean = a.mean_surface_c();
        assert!(
            (mean - 15.0).abs() < 0.5,
            "from {start} C it settled at {mean:.2} C"
        );
        let (cold, warm) = a
            .ground_k
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &t| (lo.min(t), hi.max(t)));
        assert!(warm - cold > 20.0, "one side of it is still the warm side");
        assert!(!a.sun_trim_at_limit(), "trim {}", a.sun_trim);
    }
}

/// The trim stops at its limits, says so, and does not wind up while it is
/// held there: back at the target, the sun comes straight back to its
/// balance.
#[test]
fn the_trim_stops_at_its_limits_and_does_not_wind_up() {
    let mut a = quick_planet(-50.0);
    // A budget that balances at the target with a trim of exactly 1.
    let s = a.settings;
    let (absorbed, greenhouse) = (s.olr_a + s.olr_b * 15.0 - 20.0, 20.0);
    for _ in 0..20_000 {
        a.sun_trim = a.thermostat(absorbed, greenhouse, 1.0);
    }
    assert_eq!(a.sun_trim, a.settings.sun_trim_max);
    assert!(a.sun_trim_at_limit());
    a.ground_k.fill(15.0);
    a.sun_trim = a.thermostat(absorbed, greenhouse, 1.0);
    assert!((a.sun_trim - 1.0).abs() < 0.02, "back at {}", a.sun_trim);
    a.ground_k.fill(80.0);
    for _ in 0..20_000 {
        a.sun_trim = a.thermostat(absorbed, greenhouse, 1.0);
    }
    assert_eq!(a.sun_trim, a.settings.sun_trim_min);
    a.ground_k.fill(15.0);
    a.sun_trim = a.thermostat(absorbed, greenhouse, 1.0);
    assert!((a.sun_trim - 1.0).abs() < 0.02, "back at {}", a.sun_trim);
    let mut off = quick_planet(-50.0);
    off.settings.target_mean_c = None;
    assert_eq!(
        off.thermostat(absorbed, greenhouse, 1.0),
        1.0,
        "no target, no trim"
    );
    assert!(!off.sun_trim_at_limit());
}

/// Runs X and Y rang: a sea much slower than the land kept an integral
/// pushing long after the planet was on its way. A planet with a sea thirty
/// times slower than its land, started three kelvin cold, settles at the
/// target by its energy balance and does not overshoot it by half a kelvin;
/// and once settled, its sun is steady.
#[test]
fn a_planet_with_a_slow_sea_settles_without_ringing() {
    // The toy was tuned on the spread every level had before decision 7,
    // 0.002 a second, which this diffusivity gives on its level-3 grid.
    let span = Grid::new(quiet().level, TerrainConfig::TENEBRIS.radius_m).mean_span;
    let mut a = air(AtmosphereSettings {
        heat_diffusivity_m2s: 0.002 * span * span,
        solar_wm2: 1100.0,
        land_heat_capacity: 2.0e4,
        ocean_heat_capacity: 6.0e5,
        sun_balance_s: 4_000.0,
        sun_trim_s: 40_000.0,
        sun_trim_per_k: 0.03,
        ..quiet()
    });
    a.ground_k.fill(12.0);
    a.air_k.fill(12.0);
    let (mut highest, mut trims) = (f64::MIN, Vec::<f32>::new());
    for k in 0..250_000 {
        a.heat(SUN, a.settings.dt_s);
        if k % 1000 == 0 {
            highest = highest.max(a.mean_surface_c());
            if k >= 150_000 {
                trims.push(a.sun_trim);
            }
        }
    }
    let mean = a.mean_surface_c();
    assert!((mean - 15.0).abs() < 1.0, "settled at {mean:.2} C");
    assert!(highest < 15.5, "overshot to {highest:.2} C");
    // Settling, the trim only comes down onto the balance, never across it
    // and back: no ringing.
    let s = a.settings;
    let balance = (s.olr_a + s.olr_b * 15.0 - a.balance_greenhouse) / a.balance_absorbed;
    assert!(
        trims.windows(2).all(|w| w[1] <= w[0] + 1e-3),
        "the trim turned back: {trims:?}"
    );
    assert!(
        (a.sun_trim - balance).abs() < 0.02,
        "the trim {} is near the balance {balance}",
        a.sun_trim
    );
}

/// Finding 7: every strike's cold pool took a kelvin and a half from its
/// cell's air, and on a stormy planet that was 15 W/m^2. The pool stays cold
/// where it struck, and the air as a whole keeps its heat.
#[test]
fn a_strikes_cold_pool_stays_cold_and_the_air_keeps_its_heat() {
    let mut a = air(AtmosphereSettings {
        strike_chance: 1.0,
        ..quiet()
    });
    let struck = 7;
    a.charge.fill(0.0);
    a.charge[struck] = 3.0;
    a.air_k.fill(20.0);
    let before = a.grid.total(&a.air_k);
    a.lightning(a.settings.dt_s);
    assert_eq!(a.strikes.len(), 1, "one strike");
    assert!(
        a.air_k[struck] < 20.0 - a.settings.pool_k * 0.9,
        "the pool is cold"
    );
    let drift = (a.grid.total(&a.air_k) - before).abs() / before.abs();
    assert!(drift < 1e-6, "{drift:e}");
}

/// The planet's heat, J: the ground's and the air's, `sum(C T A)`, plus the
/// latent heat the vapour carries, which is what condensing will give back.
fn heat_j(a: &Atmosphere) -> f64 {
    let s = a.settings;
    (0..a.grid.len())
        .map(|i| {
            let area = a.grid.area[i] as f64;
            (a.surface.heat_capacity[i] as f64 * a.ground_k[i] as f64
                + s.air_heat_capacity() as f64 * a.air_k[i] as f64
                + s.latent_j_per_kg() as f64 * a.vapour[i] as f64)
                * area
        })
        .sum()
}

/// A sun and a sky that neither heat nor cool, so what is left of the heat
/// step is the spread and the air's exchange with the ground.
fn no_radiation() -> AtmosphereSettings {
    AtmosphereSettings {
        solar_wm2: 0.0,
        olr_a: 0.0,
        olr_b: 0.0,
        cloud_greenhouse: 0.0,
        ..quiet()
    }
}

/// The leak that froze the planet (`climate-balance` finding 2): the spread
/// moved temperature, so a coastal sea cell lost sixty times the heat its land
/// neighbour gained. Now each edge carries one flux of heat, and a grid of
/// land and sea keeps its heat to rounding while its temperatures even out.
#[test]
fn the_spread_moves_heat_between_land_and_sea_and_keeps_it() {
    let grid = Grid::new(3, 4_800.0);
    let n = grid.len();
    // Every third cell is sea, sixty times the land's capacity, and the
    // temperatures run from -30 to +30 C.
    let capacity: Vec<f32> = (0..n)
        .map(|i| if i % 3 == 0 { 3.0e6 } else { 5.0e4 })
        .collect();
    let mut t: Vec<f32> = (0..n).map(|i| ((i * 37) % 61) as f32 - 30.0).collect();
    let heat = |t: &[f32]| -> f64 {
        (0..n)
            .map(|i| (capacity[i] * t[i] * grid.area[i]) as f64)
            .sum()
    };
    // Measured against the heat's size, not its sum, which the signs cancel.
    let size: f64 = (0..n)
        .map(|i| (capacity[i] * t[i].abs() * grid.area[i]) as f64)
        .sum();
    let start = heat(&t);
    // The old spread, in kelvin, on the same grid: it makes or loses heat
    // wherever land meets sea.
    let mut old = t.clone();
    for _ in 0..2_000 {
        let rate: Vec<f32> = (0..n)
            .map(|i| grid.neighbour_excess(&old, i, |_| false) * 0.002)
            .collect();
        for i in 0..n {
            old[i] += rate[i] * 10.0;
        }
    }
    let leaked = (heat(&old) - start).abs() / size;
    assert!(
        leaked > 1e-4,
        "the old spread leaked {leaked:e} of the heat"
    );
    let spread = |t: &[f32]| -> f32 {
        let mean = t.iter().sum::<f32>() / n as f32;
        t.iter().map(|v| (v - mean).abs()).sum::<f32>() / n as f32
    };
    let spread_before = spread(&t);
    for _ in 0..2_000 {
        let rate = grid.conduct(&t, &capacity, 0.002);
        for i in 0..n {
            t[i] += rate[i] * 10.0;
        }
    }
    let drift = (heat(&t) - start).abs() / size;
    assert!(drift < 1e-5, "heat drifted by {drift:e} of itself");
    assert!(spread(&t) < spread_before * 0.8, "and it did spread");

    // Two land cells move as the old spread moved them; across a coast the
    // sea moves a sixtieth of what the land does.
    let (land, sea) = (1, 0);
    let k = grid.neighbour[land][0] as usize;
    let flat = vec![5.0e4; n];
    let mut hot = vec![0.0f32; n];
    hot[k] = 1.0;
    let rate = grid.conduct(&hot, &flat, 0.002);
    let old = grid.neighbour_excess(&hot, land, |_| false) * 0.002;
    assert!(
        (rate[land] - old).abs() < old * 0.15,
        "{} against {old}",
        rate[land]
    );
    // A cold sea cell in warm land, against the same cell as land: the land
    // beside it moves exactly as it would beside land, and the sea moves by
    // the land's capacity over its own.
    let mut coast = vec![5.0e4; n];
    coast[sea] = 3.0e6;
    let mut cold_sea = vec![1.0f32; n];
    cold_sea[sea] = 0.0;
    let as_sea = grid.conduct(&cold_sea, &coast, 0.002);
    let as_land = grid.conduct(&cold_sea, &flat, 0.002);
    let k = grid.neighbour[sea][0] as usize;
    assert_eq!(as_sea[k], as_land[k], "the land moves as it did");
    let ratio = as_sea[sea] / as_land[sea];
    assert!((ratio - 5.0e4 / 3.0e6).abs() < 1e-6, "{ratio}");
}

/// The heat step with no sun and no sky: the spread and the air's exchange
/// with the ground only move heat, so a planet's heat is kept to rounding.
#[test]
fn the_heat_step_without_radiation_keeps_the_planets_heat() {
    let mut a = air(no_radiation());
    let start = heat_j(&a);
    for _ in 0..600 {
        a.heat(SUN, a.settings.dt_s);
    }
    let drift = (heat_j(&a) - start).abs() / start.abs();
    assert!(drift < 1e-5, "{drift:e}");
}

/// The second leak (`climate-balance` finding 5): evaporation took 8.0e4 J
/// a kilogram from the ground and condensing gave back 15,750. Now the water
/// step takes and gives one number, and cloud that evaporates again takes
/// back what it gave, so the ground's and the air's heat plus the vapour's
/// latent heat is kept while water evaporates, condenses and rains.
#[test]
fn the_water_cycle_moves_heat_and_neither_makes_nor_loses_it() {
    let mut a = air(no_radiation());
    // Wet and lifted, so everything happens: evaporation over the sea,
    // condensation in the rising air, rain, and cloud drying at the edges.
    let lift: Vec<f32> = (0..a.grid.len())
        .map(|i| if i % 2 == 0 { -2e-3 } else { 1e-3 })
        .collect();
    let start = heat_j(&a);
    let (mut condensed, mut rained) = (0.0f64, 0.0f64);
    for _ in 0..600 {
        let cloud = total(&a.grid.area, &a.cloud);
        a.water(&lift, a.settings.dt_s);
        rained += total(&a.grid.area, &a.rain_rate) * a.settings.dt_s as f64;
        condensed += (total(&a.grid.area, &a.cloud) - cloud).max(0.0);
    }
    assert!(rained > 0.0 && condensed > 0.0, "the cycle ran");
    let drift = (heat_j(&a) - start).abs() / start.abs();
    assert!(drift < 1e-5, "{drift:e}");
}

/// Finding 6: carrying the air's temperature in the advective form lost a
/// quarter of the absorbed sunlight where warm air converged. The carry now
/// gives back what the form gains or loses, so the air's heat is kept to
/// rounding under a wind that converges on one side of the planet.
#[test]
fn the_winds_carry_keeps_the_airs_heat() {
    let mut a = air(quiet());
    for i in 0..a.grid.len() {
        let c = a.grid.centre[i];
        // Converging on the +X side, warm there and cold elsewhere.
        a.wind[i] = (Vec3::X - c * c.x) * -8.0;
        a.air_k[i] = 10.0 + 20.0 * c.x;
    }
    let start = a.grid.total(&a.air_k);
    for _ in 0..200 {
        a.carry(a.settings.dt_s);
    }
    let size: f64 = a
        .air_k
        .iter()
        .zip(&a.grid.area)
        .map(|(&t, &area)| (t.abs() * area) as f64)
        .sum();
    let drift = (a.grid.total(&a.air_k) - start).abs() / size;
    assert!(drift < 1e-5, "{drift:e}");
}

/// The heat spread is one diffusivity at every level (`climate-balance`
/// decision 7): level 5, the game's, steps at the 0.002 a second it shipped
/// with, and each coarser level at a quarter of the next finer one's rate,
/// since its cells are twice as far apart. Before, every level stepped at
/// 0.002, so the level-3 instruments spread heat sixteen times harder than
/// the game.
#[test]
fn the_heat_spread_is_the_same_diffusivity_at_every_level() {
    let settings = AtmosphereSettings::default();
    let rate = |level| settings.spread_per_s(Grid::new(level, 4_800.0).mean_span);
    let game = rate(5);
    assert!((game / 0.002 - 1.0).abs() < 1e-3, "level 5 steps at {game}");
    for level in 3..5 {
        let ratio = rate(level) / rate(level + 1);
        assert!(
            (ratio - 0.25).abs() < 0.0025,
            "level {} steps at {ratio} of level {level}'s rate",
            level + 1
        );
    }
}

/// The wind at cloud height in bands `width` degrees of latitude wide, from
/// `reach` degrees south to `reach` north: each band's area-weighted zonal
/// mean speed, and of its prograde part (Earth's westerly positive), m/s.
fn aloft_bands(a: &Atmosphere, width: f32, reach: f32) -> Vec<(f32, f32)> {
    let count = (2.0 * reach / width).round() as usize;
    let mut bands = vec![(0.0f64, 0.0f64, 0.0f64); count];
    for i in 0..a.grid.len() {
        let c = a.grid.centre[i];
        let latitude = c.y.clamp(-1.0, 1.0).asin().to_degrees();
        if latitude.abs() >= reach {
            continue;
        }
        let band = &mut bands[((latitude + reach) / width) as usize];
        let area = f64::from(a.grid.area[i]);
        let upper = a.upper[i];
        band.0 += area;
        band.1 += f64::from(upper.length()) * area;
        band.2 += f64::from(upper.dot(super::step::prograde(c))) * area;
    }
    bands
        .iter()
        .map(|(area, speed, prograde)| ((speed / area) as f32, (prograde / area) as f32))
        .collect()
}

/// The current generator's level-5 settled climate: a real day's temperature
/// and wind, as a new world starts from.
fn settled_level_5(settings: AtmosphereSettings) -> Atmosphere {
    let generator = crate::terrain::GENERATOR_VERSION;
    let path = format!(
        "{}/../../assets/climate/settled-g{generator}-l5.bin",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytes = std::fs::read(&path).expect("the current generator's level-5 climate ships");
    let terrain = TerrainConfig::for_version(generator).expect("the current generator");
    let mut a = Atmosphere::new(
        &terrain,
        AtmosphereSettings {
            level: 5,
            ..settings
        },
        terrain.seed,
    );
    a.restore(&bytes).expect("a settled climate restores");
    a
}

/// The steepest change of zonal-mean speed between neighbouring bands, m/s a
/// degree of latitude.
fn steepest(bands: &[(f32, f32)], width: f32) -> f32 {
    bands
        .windows(2)
        .map(|pair| (pair[1].0 - pair[0].0).abs() / width)
        .fold(0.0, f32::max)
}

/// `tropical-upper-wind` task 2.2: on a settled climate the tropics blow from
/// the east aloft, and the jet grows out of them across a band of latitude,
/// not at an edge. The rule it replaced left the equator calm (0.5 m/s) and
/// changed 6.2 m/s a degree at the jet's edge.
#[test]
fn the_tropics_blow_easterly_aloft_and_the_jet_has_no_edge() {
    let mut a = settled_level_5(AtmosphereSettings::default());
    a.aloft();
    let easterly = a.settings.tropical_easterly_mps;
    let equator = aloft_bands(&a, 5.0, 5.0);
    for (_, prograde) in &equator {
        assert!(
            *prograde <= -0.5 * easterly,
            "the equator aloft blows {prograde:.1} m/s prograde, not from the east"
        );
    }
    let edge = steepest(&aloft_bands(&a, 2.5, 35.0), 2.5);
    assert!(edge <= 3.5, "the jet's edge changes {edge:.2} m/s a degree");
}

/// With no easterly asked for, the tropics aloft have the surface wind alone
/// (inside 5.7 degrees the jet has wholly faded), and the fade still leaves
/// no edge.
#[test]
fn with_no_easterly_the_tropics_aloft_have_the_surface_wind() {
    let mut a = settled_level_5(AtmosphereSettings {
        tropical_easterly_mps: 0.0,
        ..Default::default()
    });
    a.aloft();
    let mut inside = 0;
    for i in 0..a.grid.len() {
        if a.grid.centre[i].y.abs() < super::step::JET_FADE_SIN_LAT.0 {
            assert_eq!(a.upper[i], a.wind[i]);
            inside += 1;
        }
    }
    assert!(inside > 0);
    let edge = steepest(&aloft_bands(&a, 2.5, 35.0), 2.5);
    assert!(edge <= 3.5, "the jet's edge changes {edge:.2} m/s a degree");
}

/// The rain a person sees builds in and dies away (`smooth-weather` decision
/// 1): a burst that switches on in one step moves it only the rise's share of
/// the way, and a 20 s gap between two bursts lets it fall to a lull, not to
/// nothing.
#[test]
fn the_rain_seen_builds_in_and_dies_away() {
    let mut a = air(quiet());
    let s = a.settings;
    let dt = s.dt_s;
    let cell = 0;
    a.cloud[cell] = s.cover_full_kg * 2.0;
    let cover = a.cover(cell);
    assert!(cover > 0.99, "a full cell");
    a.rain_rate.fill(0.0);
    a.follow_rain(dt);
    assert_eq!(
        a.rain_seen[cell], 0.0,
        "a new state starts at the rain as it is"
    );
    a.rain_rate[cell] = s.raining_rate * 50.0;
    a.follow_rain(dt);
    let rise = 1.0 - (-dt / s.rain_rise_s).exp();
    assert!(
        (a.rain_seen[cell] - rise * cover).abs() < 1e-4,
        "one step of a burst moves it {} of the way, not all of it",
        a.rain_seen[cell]
    );
    for _ in 0..60 {
        a.follow_rain(dt);
    }
    let full = a.rain_seen[cell];
    assert!(full > 0.99 * cover, "a minute of rain is all of it: {full}");
    a.rain_rate[cell] = 0.0;
    for _ in 0..20 {
        a.follow_rain(dt);
    }
    let lull = a.rain_seen[cell] / full;
    let expected = (-20.0 * dt / s.rain_fall_s).exp();
    assert!(
        (lull - expected).abs() < 1e-3 && lull > 0.3,
        "a 20 s gap falls to {lull:.3} of the rain, not to nothing"
    );
}

/// The rain seen is derived: the saved bytes do not hold it, and a restored
/// state's first step sets it to the rain as it stands rather than fading it
/// in from nothing.
#[test]
fn the_rain_seen_is_not_saved_and_starts_at_the_rain_as_it_stands() {
    let mut a = air(quiet());
    let storm = Vec3::new(0.3, 0.8, 0.2).normalize();
    for _ in 0..60 {
        a.step(
            SUN,
            &[Forcing {
                direction: storm,
                strength: 1.0,
            }],
        );
    }
    let bytes = a.to_bytes();
    let mut other = a.clone();
    other.rain_seen.fill(0.7);
    assert_eq!(other.to_bytes(), bytes, "the rain seen is not saved");
    let mut restored = air(quiet());
    restored.restore(&bytes).expect("its own bytes");
    restored.step(SUN, &[]);
    let s = restored.settings;
    let mut raining = 0;
    for i in 0..restored.grid.len() {
        let target = if restored.rain_rate[i] >= s.raining_rate {
            raining += 1;
            restored.cover(i)
        } else {
            0.0
        };
        assert_eq!(restored.rain_seen[i], target, "cell {i}");
    }
    assert!(raining > 0, "the brewed storm still rains");
}
