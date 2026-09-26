use super::*;
use crate::fauna::{FaunaSettings, HOME_BODY};

const R: f32 = 4800.0;
const G: f32 = 25.0;

/// Dry for z < 0, shelving south at 0.4 m a metre to 8 m.
struct Shore;

impl Water for Shore {
    fn surface(&self, _point: Vec3) -> f32 {
        R
    }
    fn bed(&self, point: Vec3) -> f32 {
        let z = (point.normalize_or(Vec3::Y) * R).z;
        R - (0.4 * z).clamp(-2.0, 8.0)
    }
}

fn at(x: f32, depth: f32, z: f32) -> Vec3 {
    Vec3::new(x, R, z).normalize() * (R - depth)
}

/// An angler on the beach at z = -1, looking south over the water.
fn angler() -> Angler {
    let feet = at(0.0, -0.4, -1.0);
    let up = feet.normalize();
    Angler {
        tip: feet + up * 1.9,
        look: (Vec3::Z - up * Vec3::Z.dot(up) + up * 0.1).normalize(),
        feet,
    }
}

struct World {
    line: Line,
    schools: Vec<School>,
    roster: Vec<Species>,
    fauna: FaunaSettings,
}

impl World {
    fn new(seed: u64) -> Self {
        let fauna = FaunaSettings::default();
        Self {
            line: Line::new(seed),
            schools: Vec::new(),
            roster: fauna.roster(HOME_BODY).to_vec(),
            fauna,
        }
    }

    fn add(&mut self, id: &str, anchor: Vec3, seed: u64) {
        let i = self.roster.iter().position(|s| s.id == id).unwrap();
        self.schools.push(School::spawn(
            i as u16,
            &self.roster[i],
            anchor,
            seed,
            &Shore,
        ));
    }

    fn tick(&mut self, controls: Controls) -> Vec<Event> {
        let dt = 1.0 / 30.0;
        let events = self.line.step(
            dt,
            controls,
            &angler(),
            &mut self.schools,
            &self.roster,
            &self.fauna.fishing,
            &self.fauna.flock,
            &Shore,
            G,
            1.0,
        );
        let splash = self.line.splash();
        for school in &mut self.schools {
            let species = &self.roster[school.species as usize];
            school.step(dt, species, &self.fauna.flock, &Shore, splash);
        }
        events
    }

    /// Hold the button for `seconds` and let go: a cast.
    fn cast(&mut self, seconds: f32) -> Vec<Event> {
        let mut events = self.tick(Controls {
            pressed: true,
            held: true,
            ..Default::default()
        });
        for _ in 0..(seconds * 30.0) as usize {
            events.extend(self.tick(Controls {
                held: true,
                ..Default::default()
            }));
        }
        events.extend(self.tick(Controls {
            released: true,
            ..Default::default()
        }));
        events
    }

    /// Step with nothing pressed until the float settles or lands.
    fn wait_for(&mut self, phase: Phase, seconds: f32) -> Vec<Event> {
        let mut events = Vec::new();
        for _ in 0..(seconds * 30.0) as usize {
            if self.line.phase == phase {
                break;
            }
            events.extend(self.tick(Controls::default()));
        }
        events
    }
}

#[test]
fn a_cast_over_water_floats_and_a_cast_onto_the_beach_comes_back() {
    let mut world = World::new(1);
    let events = world.cast(0.8);
    assert!(matches!(events.last(), Some(Event::Cast { .. })));
    let events = world.wait_for(Phase::Floating, 5.0);
    assert!(events.contains(&Event::Splash), "{events:?}");
    assert_eq!(world.line.phase, Phase::Floating);
    let float = world.line.float;
    assert!(Shore.depth(float) > 0.0, "on the water");
    assert!((float.length() - R).abs() < 0.2, "riding the surface");
    let out = (float.normalize() * R).z;
    assert!(out > 2.0 && out < 16.0, "{out} m out");

    // Straight down at the beach: dry.
    let mut world = World::new(2);
    let mut a = angler();
    a.look = -a.feet.normalize();
    world.line.phase = Phase::Charging;
    let events = world.line.step(
        1.0 / 30.0,
        Controls {
            released: true,
            ..Default::default()
        },
        &a,
        &mut world.schools,
        &world.roster,
        &world.fauna.fishing,
        &world.fauna.flock,
        &Shore,
        G,
        1.0,
    );
    assert!(matches!(events[0], Event::Cast { .. }));
    let events = world.wait_for(Phase::Ready, 5.0);
    assert!(events.contains(&Event::Dry), "{events:?}");
}

#[test]
fn right_click_winds_the_line_in() {
    let mut world = World::new(3);
    world.cast(0.5);
    world.wait_for(Phase::Floating, 5.0);
    let events = world.tick(Controls {
        reel_in: true,
        ..Default::default()
    });
    assert_eq!(events, vec![Event::ReeledIn]);
    assert_eq!(world.line.phase, Phase::Ready);
}

#[test]
fn with_no_school_near_nothing_bites() {
    let mut world = World::new(4);
    world.add("minnow", at(200.0, 1.0, 20.0), 1);
    world.cast(0.6);
    world.wait_for(Phase::Floating, 5.0);
    let events = world.wait_for(Phase::Nibble, 60.0);
    assert_eq!(world.line.phase, Phase::Floating, "{events:?}");
}

/// Cast into a school and follow the whole sequence: a fish leaves the school
/// for the lure, nibbles, bites, is hooked, reeled patiently and landed. The
/// fish landed is the one that bit: its school is one fish smaller.
fn fish_until_hooked(world: &mut World) -> u16 {
    world.cast(0.6);
    world.wait_for(Phase::Floating, 5.0);
    let events = world.wait_for(Phase::Bite, 90.0);
    assert!(
        events.iter().any(|e| matches!(e, Event::Interest { .. })),
        "{events:?}"
    );
    assert!(events.iter().any(|e| matches!(e, Event::Nibble { .. })));
    assert_eq!(world.line.phase, Phase::Bite, "{events:?}");
    let events = world.tick(Controls {
        pressed: true,
        held: true,
        ..Default::default()
    });
    let Some(Event::Hooked { species }) = events.first().copied() else {
        panic!("{events:?}");
    };
    species
}

#[test]
fn a_patient_reel_lands_the_fish_that_bit() {
    for (id, seed) in [("minnow", 10), ("silverfin", 11), ("deepback", 12)] {
        let mut world = World::new(seed);
        let lure_side = at(0.0, 1.0, 8.0);
        world.add(id, lure_side, seed);
        let before = world.schools[0].len();
        let species = fish_until_hooked(&mut world);
        let mut caught = None;
        for _ in 0..30 * 120 {
            let held = world.line.tension < 0.6;
            for e in world.tick(Controls {
                held,
                ..Default::default()
            }) {
                if let Event::Caught { species, length_cm } = e {
                    caught = Some((species, length_cm));
                }
                assert!(
                    !matches!(e, Event::Snapped { .. }),
                    "{id}: a patient reel snapped"
                );
            }
            if caught.is_some() {
                break;
            }
        }
        let (got, cm) = caught.unwrap_or_else(|| panic!("{id}: never landed"));
        assert_eq!(got, species);
        assert_eq!(world.roster[got as usize].id, id);
        assert!(cm > 0);
        assert_eq!(
            world.schools[0].len(),
            before - 1,
            "{id}: the fish came out of its school"
        );
        assert_eq!(world.line.phase, Phase::Ready);
    }
}

#[test]
fn reeling_through_every_run_snaps_the_line() {
    let mut world = World::new(20);
    world.add("deepback", at(0.0, 1.0, 8.0), 20);
    let species = fish_until_hooked(&mut world);
    let mut snapped = false;
    for _ in 0..30 * 60 {
        let events = world.tick(Controls {
            held: true,
            ..Default::default()
        });
        assert!(
            !events.iter().any(|e| matches!(e, Event::Caught { .. })),
            "landed on a line that should have snapped"
        );
        if events.contains(&Event::Snapped { species }) {
            snapped = true;
            break;
        }
    }
    assert!(
        snapped,
        "a deepback held on a taut line the whole way should snap it"
    );
}

#[test]
fn hooking_during_a_nibble_spooks_the_fish() {
    let mut world = World::new(30);
    world.add("minnow", at(0.0, 1.0, 8.0), 30);
    world.cast(0.6);
    world.wait_for(Phase::Floating, 5.0);
    world.wait_for(Phase::Nibble, 90.0);
    assert_eq!(world.line.phase, Phase::Nibble);
    let events = world.tick(Controls {
        pressed: true,
        held: true,
        ..Default::default()
    });
    assert!(
        matches!(events.first(), Some(Event::TooEarly { .. })),
        "{events:?}"
    );
    assert_eq!(world.line.phase, Phase::Floating);
    assert!(world.line.target.is_none());
}

#[test]
fn a_missed_bite_lets_the_fish_go() {
    let mut world = World::new(40);
    world.add("minnow", at(0.0, 1.0, 8.0), 40);
    world.cast(0.6);
    world.wait_for(Phase::Floating, 5.0);
    world.wait_for(Phase::Bite, 90.0);
    let events = world.wait_for(Phase::Floating, 3.0);
    assert!(
        events.iter().any(|e| matches!(e, Event::Missed { .. })),
        "{events:?}"
    );
}
