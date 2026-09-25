use super::*;

pub(crate) const R: f32 = 4800.0;

/// A shore on a sphere: near the +Y pole, dry for z < 0 and shelving to the
/// south at 0.4 m of depth per metre, levelling out at `max_depth`.
pub(crate) struct Shore {
    pub max_depth: f32,
}

impl Water for Shore {
    fn surface(&self, _point: Vec3) -> f32 {
        R
    }
    fn bed(&self, point: Vec3) -> f32 {
        let z = (point.normalize_or(Vec3::Y) * R).z;
        R - (0.4 * z).clamp(-2.0, self.max_depth)
    }
}

pub(crate) fn at(x: f32, depth: f32, z: f32) -> Vec3 {
    Vec3::new(x, R, z).normalize() * (R - depth)
}

fn roster() -> Vec<Species> {
    FaunaSettings::default().roster(HOME_BODY).to_vec()
}

fn index_of(id: &str) -> usize {
    roster().iter().position(|s| s.id == id).unwrap()
}

#[test]
fn the_defaults_validate_and_carry_eight_species_on_the_home_body() {
    let fauna = FaunaSettings::default();
    fauna.validate().unwrap();
    let ids: Vec<&str> = fauna
        .roster(HOME_BODY)
        .iter()
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "minnow",
            "silverfin",
            "perch",
            "ray",
            "eel",
            "reef",
            "deepback",
            "serpent"
        ],
        "the roster is append-only: a saved fish is its index"
    );
    assert!(
        fauna.roster("a-barren-moon").is_empty(),
        "no roster, no fish"
    );
}

/// No species lives on two bodies, and every one has an entry, a tip and an
/// icon: the checks the field guide and the life-roster rule rest on.
#[test]
fn a_species_on_two_bodies_or_without_an_entry_is_refused() {
    let mut fauna = FaunaSettings::default();
    let minnow = fauna.roster(HOME_BODY)[0].clone();
    fauna.bodies.insert("second".into(), vec![minnow]);
    assert!(
        fauna.validate().is_err(),
        "a species leaked to a second body"
    );
    let mut fauna = FaunaSettings::default();
    fauna.bodies.get_mut(HOME_BODY).unwrap()[2].guide.entry = " ".into();
    assert!(fauna.validate().is_err(), "a species with no entry");
    let mut fauna = FaunaSettings::default();
    fauna.bodies.get_mut(HOME_BODY).unwrap()[5].icon.clear();
    assert!(fauna.validate().is_err(), "a species with no icon");
    let mut fauna = FaunaSettings::default();
    fauna.bodies.get_mut(HOME_BODY).unwrap()[1].temp_c = (20.0, 10.0);
    assert!(fauna.validate().is_err(), "a window upside down");
}

/// Tenebris's window: 0.9 s for the weakest fish, 13% shorter per step, never
/// under a quarter second.
#[test]
fn the_hook_window_is_tenebris_formula() {
    let s = FishingSettings::default();
    let windows: Vec<f32> = (1..=5).map(|k| s.hook_window(k)).collect();
    for (got, want) in windows.iter().zip([0.9, 0.783, 0.666, 0.549, 0.432]) {
        assert!((got - want).abs() < 1e-3, "{windows:?}");
    }
    let floor = FishingSettings {
        hook_step: 0.3,
        ..s
    };
    assert_eq!(floor.hook_window(5), 0.25, "never under the floor");
}

#[test]
fn clouds_and_rain_make_fish_bite_more_readily() {
    let s = FishingSettings::default();
    assert_eq!(s.bite_factor(0.0, 0.0), 1.0);
    assert!((s.bite_factor(1.0, 0.0) - 1.35).abs() < 1e-6);
    assert!((s.bite_factor(1.0, 50.0) - 2.05).abs() < 1e-6);
    assert_eq!(s.bite_factor(f32::NAN, f32::NAN), 1.0);
}

/// The spawn gate: the class must match, the temperature must be inside the
/// window, and nothing spawns in frozen water.
#[test]
fn a_species_spawns_only_in_its_water_at_its_temperature() {
    let limits = WaterLimits::default();
    let r = roster();
    let reef = &r[index_of("reef")];
    assert!(reef.lives_in(WaterClass::Shallows, 26.0, &limits));
    assert!(
        !reef.lives_in(WaterClass::Shallows, 20.0, &limits),
        "too cool"
    );
    assert!(
        !reef.lives_in(WaterClass::Shelf, 26.0, &limits),
        "wrong water"
    );
    let perch = &r[index_of("perch")];
    let silverfin = &r[index_of("silverfin")];
    assert!(perch.lives_in(WaterClass::River, 12.0, &limits));
    assert!(
        !silverfin.lives_in(WaterClass::River, 12.0, &limits),
        "a river is not the sea"
    );
    for sp in &r {
        for class in [
            WaterClass::River,
            WaterClass::Shallows,
            WaterClass::Shelf,
            WaterClass::Deep,
        ] {
            assert!(
                !sp.lives_in(class, -2.5, &limits),
                "{} in frozen water",
                sp.id
            );
        }
    }
}

/// On the real terrain a river is water only because of the carve, and the
/// sea is classed by depth.
#[test]
fn the_water_class_comes_from_the_terrain() {
    let terrain = TerrainConfig::TENEBRIS;
    let limits = WaterLimits::default();
    assert_eq!(
        water_class(&terrain, Vec3::Y, 0.0, &limits),
        None,
        "dry land"
    );
    let mut rivers = 0;
    let mut seas = 0;
    for i in 0..4000 {
        let d = fibonacci(i, 4000);
        let depth = terrain.sea_level_m - surface_altitude(&terrain, d);
        match water_class(&terrain, d, depth, &limits) {
            Some(WaterClass::River) => {
                rivers += 1;
                assert!(
                    depth > 0.0 && depth < 4.0,
                    "a river channel is shallow: {depth}"
                );
            }
            Some(WaterClass::Shallows) => {
                seas += 1;
                assert!(depth <= 6.0);
            }
            Some(WaterClass::Deep) => {
                seas += 1;
                assert!(depth > 40.0);
            }
            Some(WaterClass::Shelf) => seas += 1,
            None => assert!(depth <= 0.0),
        }
    }
    assert!(rivers > 0 && seas > rivers, "rivers {rivers}, sea {seas}");
}

/// On day one of the reference world every point of open water is inside at
/// least one species' range. The first draft of the windows left 4.6% of all
/// water empty; the atlas measured this at 0.000% once they overlapped.
#[test]
fn on_day_one_no_open_water_is_without_a_species() {
    use crate::atmosphere::{Atmosphere, AtmosphereSettings};
    let terrain = TerrainConfig::TENEBRIS;
    let air = Atmosphere::new(&terrain, AtmosphereSettings::default(), terrain.seed);
    let fauna = FaunaSettings::default();
    let limits = fauna.water;
    let r = fauna.roster(HOME_BODY);
    let (mut open, mut empty) = (0, Vec::new());
    for i in 0..6000 {
        let d = fibonacci(i, 6000);
        let depth = terrain.sea_level_m - surface_altitude(&terrain, d);
        let Some(class) = water_class(&terrain, d, depth, &limits) else {
            continue;
        };
        let t = air.sample(d).temperature;
        if t < limits.freezes_c {
            continue;
        }
        open += 1;
        if !r.iter().any(|sp| sp.lives_in(class, t, &limits)) {
            empty.push((class, t));
        }
    }
    assert!(open > 1000, "only {open} open water points");
    assert!(
        empty.is_empty(),
        "{} of {open} have no species: {:?}",
        empty.len(),
        &empty[..empty.len().min(5)]
    );
}

fn fibonacci(i: usize, n: usize) -> Vec3 {
    let y = 1.0 - 2.0 * (i as f32 + 0.5) / n as f32;
    let r = (1.0 - y * y).sqrt();
    let phi = i as f32 * 2.399_963;
    Vec3::new(r * phi.cos(), y, r * phi.sin())
}

fn school_of(id: &str, anchor: Vec3, seed: u64, water: &Shore) -> (School, Species) {
    let r = roster();
    let i = index_of(id);
    (
        School::spawn(i as u16, &r[i], anchor, seed, water),
        r[i].clone(),
    )
}

/// Two minutes of flocking over a shelving bed: every fish stays in the
/// water, off the bed and under the surface, and never goes non-finite.
#[test]
fn a_school_stays_in_the_water() {
    let water = Shore { max_depth: 6.0 };
    let flock = FlockSettings::default();
    for (id, seed) in [
        ("minnow", 1),
        ("silverfin", 2),
        ("ray", 3),
        ("eel", 4),
        ("deepback", 5),
    ] {
        let (mut school, species) = school_of(id, at(0.0, 1.0, 12.0), seed, &water);
        assert!(!school.is_empty());
        let n = school.len();
        assert!(
            (species.school.0..=species.school.1).contains(&(n as u16)),
            "{id}: {n} fish"
        );
        for tick in 0..3600 {
            school.step(1.0 / 30.0, &species, &flock, &water, None);
            for p in &school.positions {
                assert!(p.is_finite(), "{id} tick {tick}");
                let r = p.length();
                assert!(
                    r <= water.surface(*p) - 0.2,
                    "{id} out of the water at tick {tick}"
                );
                assert!(
                    r >= water.bed(*p) + 0.2 || water.depth(*p) < 0.5,
                    "{id} in the bed at tick {tick}"
                );
            }
        }
        let bed_distance: f32 = school
            .positions
            .iter()
            .map(|p| p.length() - water.bed(*p))
            .sum::<f32>()
            / n as f32;
        if species.bed {
            assert!(bed_distance < 1.0, "{id} keeps to the bed: {bed_distance}");
        }
    }
}

/// The same seed and the same steps make the same school: nothing here reads
/// a clock or a global generator.
#[test]
fn a_school_is_a_function_of_its_seed_and_its_steps() {
    let water = Shore { max_depth: 6.0 };
    let flock = FlockSettings::default();
    let run = || {
        let (mut school, species) = school_of("minnow", at(3.0, 1.0, 10.0), 77, &water);
        for _ in 0..600 {
            school.step(
                1.0 / 30.0,
                &species,
                &flock,
                &water,
                Some(at(3.0, 0.0, 12.0)),
            );
        }
        school
    };
    assert_eq!(run(), run());
}

/// A school told of a lure turns toward it and gets there.
#[test]
fn a_scented_school_reaches_the_lure() {
    let water = Shore { max_depth: 8.0 };
    let flock = FlockSettings::default();
    let (mut school, species) = school_of("silverfin", at(-10.0, 1.0, 14.0), 9, &water);
    let lure = at(8.0, 0.35, 11.0);
    let start = school.centre().distance(lure);
    school.scent(&species, lure, &water, 30.0);
    for _ in 0..30 * 20 {
        school.step(1.0 / 30.0, &species, &flock, &water, None);
    }
    // Inside its sense range is what matters: there one of it can commit to
    // the lure. The school keeps its own depth and spread round the goal.
    let end = school.centre().distance(lure);
    assert!(
        end < species.sense_m && end < start / 3.0,
        "from {start:.1} m to {end:.1} m"
    );
}

#[test]
fn a_fish_measures_the_same_whoever_is_caught_first() {
    let water = Shore { max_depth: 6.0 };
    let (mut school, species) = school_of("perch", at(0.0, 1.0, 10.0), 5, &water);
    let last = school.len() - 1;
    let id = school.ids[last];
    let before = school.length_cm(last, &species);
    school.remove(0);
    let now = school.ids.iter().position(|i| *i == id).unwrap();
    assert_eq!(school.length_cm(now, &species), before);
    let (lo, hi) = (species.length_m * 85.0, species.length_m * 125.0);
    assert!(
        (lo.floor() as u32..=hi.ceil() as u32).contains(&before),
        "{before} cm"
    );
}
