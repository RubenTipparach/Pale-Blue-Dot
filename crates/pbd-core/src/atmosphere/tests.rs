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
        assert!((s.cover - a.cover_of(a.cloud[i])).abs() < 1e-3);
        assert!((0.0..=1.0).contains(&s.humidity));
    }
}
