use super::*;

/// The planet-centred origin: the tests below are written in the planet's
/// frame, which is the local space of this origin (`world-frames`).
const PLANET: crate::world_frame::SimOrigin =
    crate::world_frame::SimOrigin::new(crate::world_frame::FrameId(1), glam::DVec3::ZERO);

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
fn the_defaults_validate_and_carry_the_eight_fish_then_the_water_life() {
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
            "serpent",
            // The water life (`ocean-environments` 7.1), appended.
            "shore_crab",
            "lobster",
            "giant_clam",
            "reef_turtle",
            "sea_otter",
            "reed_dabbler",
            "pond_diver",
            // The octopus, the plume worm and the seal (B9), appended.
            "octopus",
            "tube_worm",
            "seal",
            // The beaver and the river otter (B29), appended.
            "beaver",
            "river_otter"
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

/// A baited line draws bites more readily than a bare hook, under any sky
/// (`starter-missions` G1).
#[test]
fn bait_makes_fish_bite_more_readily() {
    let s = FishingSettings::default();
    for weather in [1.0, s.bite_factor(1.0, 50.0)] {
        assert_eq!(s.baited(weather, false), weather);
        assert!((s.baited(weather, true) - weather * 1.5).abs() < 1e-6);
    }
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
                // Version 8's rivers run above the sea too (design 13): a
                // river stands at most a few metres under it, at its mouth.
                assert!(depth < 4.0, "a river channel is shallow: {depth}");
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

/// `climate-balance` tasks 3.1, 3.2 and 3.2b: a new world starts from the
/// shipped settled climate (decision 8, survey K6), and the atmosphere holds
/// the planet's mean at 15 +/- 0.5 C from its first day, its clouds' net
/// effect stays inside Earth's range, and in the second game year every
/// species has water it can live in, for part of the year at least. On the
/// leaking step the whole sea was below freezing all of year 2 and no species
/// had any (`docs/wiki/fish-ranges/current`); from rest the mean started
/// below the target and took a year to come up (finding 9). It runs 200 game
/// days at level 3, which takes minutes, so it is ignored by default:
///
///     cargo test --release -p pbd-core -- --ignored second_year_is_held
#[test]
#[ignore]
fn a_new_worlds_second_year_is_held_at_fifteen_and_every_species_has_water() {
    use crate::atmosphere::{Atmosphere, AtmosphereSettings};
    use crate::daylight::{Clock, DAY_S, START_HOUR, YEAR_DAYS};
    let terrain = TerrainConfig::TENEBRIS;
    let settings = AtmosphereSettings {
        level: 3,
        ..Default::default()
    };
    let mut air = Atmosphere::new(&terrain, settings, terrain.seed);
    // The state the game opens a new world on, made with these settings on
    // this generator's ground.
    let shipped = format!(
        "{}/../../assets/climate/settled-g{}-l3",
        env!("CARGO_MANIFEST_DIR"),
        crate::terrain::GENERATOR_VERSION
    );
    let made_with: AtmosphereSettings =
        ron::from_str(&std::fs::read_to_string(format!("{shipped}.ron")).expect("shipped"))
            .expect("settings");
    assert_eq!(
        made_with, settings,
        "the shipped state is for these settings"
    );
    air.restore(&std::fs::read(format!("{shipped}.bin")).expect("shipped"))
        .expect("the shipped state fits");
    // It stands at `START_HOUR` of a year's first day.
    let opening = f64::from(START_HOUR) / 24.0 * f64::from(DAY_S);
    let fauna = FaunaSettings::default();
    let limits = fauna.water;
    let roster = fauna.roster(HOME_BODY);
    // The water points, their class, and the year's coldest and warmest
    // daily mean at each.
    let water: Vec<(Vec3, WaterClass)> = (0..6000)
        .map(|i| fibonacci(i, 6000))
        .filter_map(|d| {
            let depth = terrain.sea_level_m - surface_altitude(&terrain, d);
            water_class(&terrain, d, depth, &limits).map(|class| (d, class))
        })
        .collect();
    let mut cold = vec![f32::MAX; water.len()];
    let mut warm = vec![f32::MIN; water.len()];
    let steps_per_day = (DAY_S / settings.dt_s) as u64;
    let days = 2 * YEAR_DAYS as u64;
    let (mut cloud_sum, mut cloud_n) = (0.0f64, 0u32);
    for day in 0..days {
        let mut mean = vec![0.0f32; water.len()];
        for k in 0..4 {
            for step in 0..steps_per_day / 4 {
                let t = opening
                    + (day * steps_per_day + k * steps_per_day / 4 + step) as f64
                        * settings.dt_s as f64;
                air.step(Clock { seconds: t }.sun(), &[]);
            }
            for (m, (d, _)) in mean.iter_mut().zip(&water) {
                *m += air.sample(*d).temperature / 4.0;
            }
        }
        let planet = air.mean_surface_c();
        assert!(
            (planet - 15.0).abs() < 0.5,
            "day {day}: the planet is at {planet:.2} C"
        );
        if day >= YEAR_DAYS as u64 {
            cloud_sum += air.net_cloud_wm2();
            cloud_n += 1;
            for i in 0..water.len() {
                cold[i] = cold[i].min(mean[i]);
                warm[i] = warm[i].max(mean[i]);
            }
        }
    }
    let cloud = cloud_sum / cloud_n as f64;
    assert!(
        (-30.0..=-10.0).contains(&cloud),
        "the clouds' net effect is {cloud:.1} W/m^2"
    );
    for sp in roster {
        let homes = water
            .iter()
            .enumerate()
            .filter(|(i, (_, class))| {
                sp.water.contains(class)
                    && warm[*i] >= limits.freezes_c
                    && warm[*i] >= sp.temp_c.0
                    && cold[*i] <= sp.temp_c.1
            })
            .count();
        assert!(homes > 0, "{} has no water in year 2", sp.id);
    }
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
        School::spawn(&PLANET, i as u16, &r[i], anchor, seed, water),
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
            school.step(&PLANET, 1.0 / 30.0, &species, &flock, &water, None);
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
                &PLANET,
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

/// `fish-behaviour` design 4: a free fish's swim turns no faster than
/// `flock.turn_deg_s` from one step to the next, and its speed rises and
/// falls with its burst-glide cycle rather than holding one value.
#[test]
fn a_fish_turns_no_faster_than_its_limit_and_bursts_and_glides() {
    let water = Shore { max_depth: 8.0 };
    let flock = FlockSettings::default();
    let (mut school, species) = school_of("perch", at(0.0, 2.0, 14.0), 3, &water);
    let dt = 1.0 / 30.0;
    let limit = flock.turn_deg_s.to_radians() * dt;
    let mut speeds = Vec::new();
    for tick in 0..30 * 40 {
        let was = school.velocities.clone();
        school.step(&PLANET, dt, &species, &flock, &water, None);
        for (a, b) in was.iter().zip(&school.velocities) {
            let turn = a.angle_between(*b);
            assert!(
                turn <= limit + 1e-4,
                "tick {tick}: turned {:.1} deg in a step",
                turn.to_degrees()
            );
        }
        speeds.push(school.velocities[0].length());
    }
    let (lo, hi) = speeds.iter().fold((f32::INFINITY, 0.0f32), |(lo, hi), s| {
        (lo.min(*s), hi.max(*s))
    });
    assert!(
        hi - lo > 0.3,
        "one speed all the way: {lo:.2} to {hi:.2} m/s"
    );
}

/// A school told of a lure turns toward it and gets there.
#[test]
fn a_scented_school_reaches_the_lure() {
    let water = Shore { max_depth: 8.0 };
    let flock = FlockSettings::default();
    let (mut school, species) = school_of("silverfin", at(-10.0, 1.0, 14.0), 9, &water);
    let lure = at(8.0, 0.35, 11.0);
    let start = school.centre().distance(lure);
    school.scent(&PLANET, &species, lure, &water, 30.0);
    for _ in 0..30 * 20 {
        school.step(&PLANET, 1.0 / 30.0, &species, &flock, &water, None);
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

/// Only the eight fish take a lure; every protected creature refuses a catch
/// with the reason, and so does anything else that is not a fish
/// (`ocean-environments` 7.3).
#[test]
fn only_fish_bite_and_protected_creatures_refuse_the_catch() {
    let roster = roster();
    let biters: Vec<&str> = roster
        .iter()
        .filter(|s| s.can_bite())
        .map(|s| s.id.as_str())
        .collect();
    assert_eq!(
        biters,
        [
            "minnow",
            "silverfin",
            "perch",
            "ray",
            "eel",
            "reef",
            "deepback",
            "serpent"
        ]
    );
    for id in ["reef_turtle", "sea_otter", "reed_dabbler", "pond_diver"] {
        let sp = &roster[index_of(id)];
        assert_eq!(sp.status, Status::Protected, "{id}");
        assert_eq!(
            sp.catch_refusal(),
            Some("protected: it may not be caught"),
            "{id}"
        );
        assert!(!sp.can_bite(), "{id}");
    }
    for id in ["shore_crab", "lobster", "giant_clam"] {
        assert_eq!(
            roster[index_of(id)].catch_refusal(),
            Some("it is not taken on a line"),
            "{id}"
        );
    }
    assert_eq!(roster[index_of("minnow")].catch_refusal(), None);
}

/// A sea with a wave on it, for the floaters.
struct Swell;

impl Water for Swell {
    fn surface(&self, point: Vec3) -> f32 {
        let p = point.normalize_or(Vec3::Y) * R;
        R + 0.4 * (p.x * 0.3).sin() + 0.2 * (p.z * 0.5).cos()
    }
    fn bed(&self, point: Vec3) -> f32 {
        Shore { max_depth: 6.0 }.bed(point)
    }
}

/// A crawler stands on the bed and a floater rides the drawn sea, wherever
/// either goes, for two minutes (`ocean-environments` 7.1).
#[test]
fn a_crawler_walks_the_bed_and_a_floater_rides_the_sea() {
    let flock = FlockSettings::default();
    let r = roster();
    for (id, seed) in [("shore_crab", 11), ("lobster", 12), ("giant_clam", 13)] {
        let i = index_of(id);
        let water = Shore { max_depth: 6.0 };
        let mut school = School::spawn(&PLANET, i as u16, &r[i], at(0.0, 1.0, 12.0), seed, &water);
        assert!(!school.is_empty());
        for tick in 0..3600 {
            school.step(&PLANET, 1.0 / 30.0, &r[i], &flock, &water, None);
            for p in &school.positions {
                let off = p.length() - water.bed(*p);
                assert!(
                    p.is_finite() && off > 0.0 && off < 0.4,
                    "{id} {off} m off the bed at tick {tick}"
                );
            }
        }
    }
    for (id, seed) in [("sea_otter", 21), ("reed_dabbler", 22), ("pond_diver", 23)] {
        let i = index_of(id);
        let mut school = School::spawn(&PLANET, i as u16, &r[i], at(0.0, 1.0, 12.0), seed, &Swell);
        for tick in 0..3600 {
            school.step(&PLANET, 1.0 / 30.0, &r[i], &flock, &Swell, None);
            for p in &school.positions {
                let off = p.length() - Swell.surface(*p);
                assert!(
                    p.is_finite() && off.abs() < 1e-3,
                    "{id} {off} m off the sea at tick {tick}"
                );
            }
        }
    }
}

/// A clam shuts when the eye comes within its 4 m and opens again once the
/// eye has gone and the hold has run out (`ocean-environments` 7.2).
#[test]
fn a_clam_shuts_when_the_eye_comes_close_and_opens_after() {
    let water = Shore { max_depth: 6.0 };
    let flock = FlockSettings::default();
    let r = roster();
    let i = index_of("giant_clam");
    let mut school = School::spawn(&PLANET, i as u16, &r[i], at(0.0, 1.0, 12.0), 5, &water);
    let clam = school.positions[0];
    let up = clam.normalize();
    let pose_now = |s: &School| pose(&r[i], s, 0, 100.0, false);
    assert_eq!(pose_now(&school), Pose::Moving, "open to begin with");
    school.notice(clam + up * 5.0, &r[i], 6.0);
    assert_eq!(pose_now(&school), Pose::Moving, "5 m off: still open");
    school.notice(clam + up * 3.5, &r[i], 6.0);
    assert_eq!(pose_now(&school), Pose::Shut, "3.5 m off: shut");
    for _ in 0..(5 * 30) {
        school.step(&PLANET, 1.0 / 30.0, &r[i], &flock, &water, None);
    }
    assert_eq!(pose_now(&school), Pose::Shut, "still shut 5 s on");
    for _ in 0..(2 * 30) {
        school.step(&PLANET, 1.0 / 30.0, &r[i], &flock, &water, None);
    }
    assert_eq!(pose_now(&school), Pose::Moving, "open again after the hold");
}

/// A dabbler tips up some of the time by day, never while startled, and
/// sleeps at night; the same clock gives the same pose (7.2).
#[test]
fn a_dabbler_tips_up_by_day_paddles_off_when_startled_and_sleeps_at_night() {
    let r = roster();
    let i = index_of("reed_dabbler");
    let mut school = School::spawn(&PLANET, i as u16, &r[i], at(0.0, 1.0, 12.0), 9, &Swell);
    let n = school.len();
    let mut tipped = 0;
    let mut samples = 0;
    for step in 0..600 {
        let t = 1000.0 + step as f64 * 0.5;
        for fish in 0..n {
            let p = pose(&r[i], &school, fish, t, false);
            assert_eq!(p, pose(&r[i], &school, fish, t, false));
            assert!(matches!(p, Pose::Moving | Pose::TippedUp));
            tipped += usize::from(p == Pose::TippedUp);
            samples += 1;
            assert_eq!(pose(&r[i], &school, fish, t, true), Pose::Asleep);
        }
    }
    let share = tipped as f32 / samples as f32;
    assert!((0.2..0.4).contains(&share), "tipped up {share} of the time");
    let eye = school.positions[0];
    school.notice(eye, &r[i], 3.0);
    assert_eq!(pose(&r[i], &school, 0, 1000.0, false), Pose::Moving);
    assert_eq!(
        pose(&r[i], &school, 0, 1000.0, true),
        Pose::Moving,
        "awake once startled"
    );
}

/// The octopus (`ocean-environments` B9): out while the eye is far; within
/// its 6 m it inks and jets home; home, it hides, and stays hidden while the
/// eye is within 8 m, and for 12 s after the eye has gone.
#[test]
fn an_octopus_inks_jets_home_and_hides_until_the_eye_has_gone() {
    let water = Shore { max_depth: 30.0 };
    let flock = FlockSettings::default();
    let r = roster();
    let i = index_of("octopus");
    let sp = &r[i];
    let mut school = School::spawn(&PLANET, i as u16, sp, at(0.0, 1.0, 20.0), 21, &water);
    let home = school.homes[0];
    let pose_now = |s: &School| pose(sp, s, 0, 100.0, false);
    // A minute of crawling takes it from its den.
    for _ in 0..(60 * 30) {
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &water, None);
    }
    assert_eq!(pose_now(&school), Pose::Moving);
    let away = school.positions[0].distance(home);
    assert!(away > 0.3, "it crawled out, {away} m");
    let up = school.positions[0].normalize();
    // The eye 10 m off: nothing.
    school.notice(school.positions[0] + up * 10.0, sp, 6.0);
    assert_eq!(pose_now(&school), Pose::Moving, "10 m off: still out");
    // The eye 5 m off: it inks and jets.
    let eye = school.positions[0] + up * 5.0;
    school.notice(eye, sp, 6.0);
    assert_eq!(pose_now(&school), Pose::Inking, "5 m off: inking");
    let mut home_by = None;
    for tick in 0..(10 * 30) {
        school.notice(eye, sp, 6.0);
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &water, None);
        if home_by.is_none() && pose_now(&school) == Pose::Hidden {
            home_by = Some(tick as f32 / 30.0);
        }
    }
    let home_by = home_by.expect("it reached its den");
    assert!(
        home_by <= away / (sp.speed_mps.1 * JET) + 0.2,
        "it jetted home at four times its pace: {home_by} s for {away} m"
    );
    // The eye backs off to 7 m: still within 8, still hidden however long.
    let near = home + up * 7.0;
    for _ in 0..(30 * 30) {
        school.notice(near, sp, 6.0);
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &water, None);
    }
    assert_eq!(pose_now(&school), Pose::Hidden, "7 m off: hidden");
    // The eye 9 m off: 11 s later still hidden, 13 s later out.
    let far = home + up * 9.0;
    for _ in 0..(11 * 30) {
        school.notice(far, sp, 6.0);
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &water, None);
    }
    assert_eq!(pose_now(&school), Pose::Hidden, "hidden for 12 s");
    for _ in 0..(2 * 30) {
        school.notice(far, sp, 6.0);
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &water, None);
    }
    assert_eq!(pose_now(&school), Pose::Moving, "out again");
}

/// A plume worm's plume snaps in within 5 m and comes out after the hold, as
/// a clam; it never leaves its place.
#[test]
fn a_plume_worm_pulls_its_plume_in_within_five_metres() {
    let water = Shore { max_depth: 6.0 };
    let flock = FlockSettings::default();
    let r = roster();
    let i = index_of("tube_worm");
    let sp = &r[i];
    let mut school = School::spawn(&PLANET, i as u16, sp, at(0.0, 1.0, 12.0), 3, &water);
    assert!(school.len() >= 10, "a cluster");
    let worm = school.positions[0];
    let up = worm.normalize();
    school.notice(worm + up * 5.5, sp, 6.0);
    assert_eq!(pose(sp, &school, 0, 0.0, false), Pose::Moving);
    school.notice(worm + up * 4.5, sp, 6.0);
    assert_eq!(pose(sp, &school, 0, 0.0, false), Pose::Shut);
    for _ in 0..(10 * 30) {
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &water, None);
    }
    assert_eq!(pose(sp, &school, 0, 0.0, false), Pose::Moving);
    assert!(
        school.positions[0].distance(worm) < 0.2,
        "it stays in its tube"
    );
}

/// A seal lies out when calm, slides under within 9 m, and hauls out again
/// 20 s after the eye has gone.
#[test]
fn a_seal_slides_in_within_nine_metres_and_hauls_out_after() {
    let flock = FlockSettings::default();
    let r = roster();
    let i = index_of("seal");
    let sp = &r[i];
    assert_eq!(sp.catch_refusal(), Some("protected: it may not be caught"));
    let mut school = School::spawn(&PLANET, i as u16, sp, at(0.0, 1.0, 12.0), 4, &Swell);
    let seal = school.positions[0];
    let up = seal.normalize();
    assert_eq!(pose(sp, &school, 0, 0.0, false), Pose::HauledOut);
    school.notice(seal + up * 10.0, sp, 6.0);
    assert_eq!(pose(sp, &school, 0, 0.0, false), Pose::HauledOut);
    school.notice(seal + up * 8.0, sp, 6.0);
    assert_eq!(pose(sp, &school, 0, 0.0, false), Pose::Under);
    for _ in 0..(19 * 30) {
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &Swell, None);
    }
    assert_eq!(
        pose(sp, &school, 0, 0.0, false),
        Pose::Under,
        "under 19 s on"
    );
    for _ in 0..(2 * 30) {
        school.step(&PLANET, 1.0 / 30.0, sp, &flock, &Swell, None);
    }
    assert_eq!(pose(sp, &school, 0, 0.0, false), Pose::HauledOut);
}

/// The spawn rule reads the place (6.1): a reef creature only on the reef,
/// a vent creature only at a vent, a duck (whose places wait for fresh
/// water) by its water classes as before; on a world made before the sea's
/// places, everything but the vent's life as it always did.
#[test]
fn a_species_spawns_only_in_its_place_once_the_places_exist() {
    let r = roster();
    let get = |id: &str| &r[index_of(id)];
    use Environment::*;
    assert!(get("giant_clam").env_allows(Some(Reef)));
    assert!(!get("giant_clam").env_allows(Some(Open)));
    assert!(!get("giant_clam").env_allows(Some(Kelp)));
    assert!(get("tube_worm").env_allows(Some(Vent)));
    assert!(!get("tube_worm").env_allows(Some(Reef)));
    assert!(
        !get("tube_worm").env_allows(None),
        "no vents on an old world"
    );
    assert!(get("seal").env_allows(Some(UnderIce)));
    assert!(!get("seal").env_allows(Some(Reef)));
    assert!(
        get("reed_dabbler").env_allows(Some(Open)),
        "fresh water waits"
    );
    assert!(get("minnow").env_allows(Some(Trench)), "no tags: anywhere");
    for sp in &r {
        if sp.id != "tube_worm" {
            assert!(sp.env_allows(None), "{} on an old world", sp.id);
        }
    }
}

/// A startled seal goes under over time and comes back up over time, never
/// in one step, and swims away from the eye while it is startled
/// (`ocean-environments` B18: "they disappear when I get close").
#[test]
fn a_startled_seal_dives_over_time_and_swims_away() {
    let flock = FlockSettings::default();
    let r = roster();
    let i = index_of("seal");
    let sp = &r[i];
    let mut school = School::spawn(&PLANET, i as u16, sp, at(0.0, 1.0, 12.0), 4, &Swell);
    let start = school.centre();
    let up = start.normalize();
    let side = up.cross(Vec3::X).normalize();
    // The eye 5 m to one side, on the surface.
    let eye = start + side * 5.0;
    let dt = 1.0 / 30.0;
    let most = dt / school::DIVE_S.min(school::SURFACE_S) + 1e-5;
    let mut last = school.under.clone();
    let mut shares = Vec::new();
    for tick in 0..(30 * 40) {
        if tick < 30 * 3 {
            school.notice(eye, sp, 6.0);
        }
        school.step(&PLANET, dt, sp, &flock, &Swell, None);
        for (a, b) in school.under.iter().zip(&last) {
            assert!((a - b).abs() <= most, "under moved {} in a step", a - b);
        }
        last = school.under.clone();
        shares.push(under_share(sp, &school, 0, 0.0, false));
        if tick == 30 * 3 {
            let off = school.centre() - eye;
            assert!(
                off.length() > 6.0 && off.dot(side) < 0.0,
                "it swam away from the eye: {} m off",
                off.length()
            );
        }
    }
    assert!(shares.iter().any(|s| *s > 0.99), "it went under");
    assert!(shares[2] < 0.2, "not at once");
    assert_eq!(*shares.last().unwrap(), 0.0, "and came back up");
}

/// A diving duck sinks and rises over time: its share under never jumps.
#[test]
fn a_diving_duck_sinks_and_rises_never_in_one_frame() {
    let r = roster();
    let i = index_of("pond_diver");
    let sp = &r[i];
    let school = School::spawn(&PLANET, i as u16, sp, at(0.0, 1.0, 12.0), 9, &Swell);
    let mut last = under_share(sp, &school, 0, 0.0, false);
    let mut deepest: f32 = 0.0;
    for k in 1..(30 * 30) {
        let now = under_share(sp, &school, 0, k as f64 / 30.0, false);
        assert!((now - last).abs() < 0.1, "jumped {last} -> {now} at {k}");
        deepest = deepest.max(now);
        last = now;
    }
    assert!(deepest > 0.99, "it dives");
}

/// A school's scale shortens every fish in it by that much (`fishing-anywhere`
/// design 3), and a school in the sea keeps its species' size.
#[test]
fn a_smaller_school_has_smaller_fish() {
    let r = roster();
    let i = r.iter().position(|s| s.can_bite()).expect("a fish");
    let mut school = School::spawn(&PLANET, i as u16, &r[i], at(0.0, 1.0, 12.0), 77, &Swell);
    assert_eq!(school.scale, 1.0);
    let full: Vec<u32> = (0..school.len())
        .map(|f| school.length_cm(f, &r[i]))
        .collect();
    school.scale = 0.5;
    for (f, was) in full.iter().enumerate() {
        let now = school.length_cm(f, &r[i]);
        assert!(
            (now as f32 - *was as f32 * 0.5).abs() <= 1.0,
            "fish {f}: {now} cm against {was} cm at full size"
        );
    }
}

/// A school is drawn between its last two steps (`movement-smoothness`):
/// at the step before for alpha 0, at the last step for 1, halfway for a
/// half, never at the raw step while its clock runs between them. A fish
/// held at the lure is drawn at the lure, and a caught fish leaves the two
/// steps in step with each other.
#[test]
fn a_school_is_drawn_between_its_steps_and_a_held_fish_at_the_lure() {
    let water = Shore { max_depth: 8.0 };
    let flock = FlockSettings::default();
    let (mut school, species) = school_of("silverfin", at(-10.0, 1.0, 14.0), 9, &water);
    // No step behind it yet: drawn where it is.
    assert_eq!(school.drawn(0, 0.0), school.positions[0]);
    school.step(&PLANET, 1.0 / 30.0, &species, &flock, &water, None);
    let (was, now) = (school.before[0], school.positions[0]);
    assert!(was.distance(now) > 1e-4, "the fish moved in the step");
    assert!(school.drawn(0, 0.0).distance(was) < 1e-6);
    assert!(school.drawn(0, 1.0).distance(now) < 1e-6);
    assert!(school.drawn(0, 0.5).distance(was.lerp(now, 0.5)) < 1e-6);
    let lure = at(8.0, 0.35, 11.0);
    school.held = Some((0, lure));
    assert_eq!(school.drawn(0, 0.3), lure);
    school.remove(0);
    assert_eq!(school.before.len(), school.positions.len());
}
