//! The prototype's checks (`docs/mockups/vehicles.html`, design section 8),
//! run on the real code: each craft on a sea at the pole of a planet the
//! engine's size, stepped as the app steps it, 60 ticks a second of four
//! substeps. The bands are the prototype's numbers, loosened where the port
//! was expected to differ.

use super::*;
use crate::sea::{SeaSettings, SeaTable};
use crate::wind::GustSettings;
use glam::Vec3;

const RADIUS: f64 = 4799.5;
const G: f64 = 25.0;
const TICK: f64 = 1.0 / 60.0;

struct World {
    sea: SeaTable,
    gusts: GustSettings,
    ground: f64,
    wind: Vec3,
    sea_wind: f32,
    rain: f32,
    seconds: f64,
}

impl World {
    fn new(ground: f64) -> Self {
        Self {
            sea: SeaTable::new(SeaSettings::default(), G as f32),
            gusts: GustSettings {
                gustiness: 0.0,
                ..Default::default()
            },
            ground,
            wind: Vec3::ZERO,
            sea_wind: 0.0,
            rain: 0.0,
            seconds: 1000.0,
        }
    }

    fn run(&mut self, craft: &mut Craft, seconds: f64, mut input: impl FnMut(&Craft) -> Input) {
        let ground = self.ground;
        let ground_at = move |_: DVec3| ground;
        for _ in 0..(seconds / TICK).round() as usize {
            let up = craft.body.position.normalize();
            let env = Surroundings {
                sea: &self.sea,
                sea_state: self.sea.state(self.sea_wind, self.wind),
                sea_radius: RADIUS,
                depth: (RADIUS - self.ground).max(0.0) as f32,
                air: AirHere {
                    wind: self.wind,
                    upper: self.wind,
                    rain_mmh: self.rain,
                    over_land: self.ground > RADIUS,
                },
                gusts: &self.gusts,
                gravity: -up * G,
                current: DVec3::ZERO,
                ground: &ground_at,
                seconds: self.seconds,
            };
            let i = input(craft);
            craft.step(TICK, 4, &i, &env);
            self.seconds += TICK;
            assert!(craft.is_finite(), "{:?} left its domain", craft.kind);
        }
    }
}

fn specs() -> (Arc<VehicleSpecs>, Hulls) {
    let specs = VehicleSpecs::default();
    let hulls = Hulls::new(&specs);
    (Arc::new(specs), hulls)
}

fn at_pole(kind: Kind, height: f64, heading: f64) -> Craft {
    let (specs, hulls) = specs();
    let q = DQuat::from_rotation_y(heading);
    Craft::new(
        kind,
        1,
        specs,
        hulls,
        DVec3::new(0.0, RADIUS + height, 0.0),
        q,
    )
}

/// Heading of the bow in the pole's tangent plane: 0 is -z, positive toward -x.
fn heading(craft: &Craft) -> f64 {
    let f = craft.body.axis(FORWARD);
    (-f.x).atan2(-f.z)
}

fn tern_telemetry(craft: &Craft) -> TernTelemetry {
    match &craft.telemetry {
        Telemetry::Tern(t) => t.clone(),
        _ => panic!("not a Tern"),
    }
}

#[test]
fn the_shipped_specs_are_valid() {
    VehicleSpecs::default().validate().unwrap();
}

#[test]
fn a_boat_settles_to_displace_its_own_weight() {
    for kind in [Kind::Tern, Kind::Loon] {
        let mut world = World::new(RADIUS - 40.0);
        world.sea = SeaTable::new(
            SeaSettings {
                swell_height_m: 0.0,
                ..Default::default()
            },
            G as f32,
        );
        let mut craft = at_pole(kind, 0.2, 0.0);
        craft.board();
        world.run(&mut craft, 20.0, |_| Input::default());
        let displaced = match &craft.telemetry {
            Telemetry::Tern(t) => t.displaced_m3,
            Telemetry::Loon(t) => t.displaced_m3,
            _ => unreachable!(),
        };
        let want = craft.body.mass / SEA_DENSITY;
        assert!(
            (displaced - want).abs() < want * 0.02,
            "{kind:?}: displaces {displaced} m^3, weighs {want} m^3"
        );
        assert!(craft.body.velocity.length() < 0.05, "{kind:?} still moving");
    }
}

#[test]
fn a_hull_rides_a_long_wave() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea_wind = 12.0;
    world.wind = Vec3::X * 12.0;
    let mut craft = at_pole(Kind::Tern, 0.2, 0.0);
    craft.board();
    world.run(&mut craft, 10.0, |_| Input::default());
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    world.run(&mut craft, 20.0, |c| {
        let h = c.body.position.length() - RADIUS;
        lo = lo.min(h);
        hi = hi.max(h);
        Input::default()
    });
    assert!(
        hi - lo > 0.3,
        "heaved only {} m on a {} m sea",
        hi - lo,
        world.sea.state(12.0, Vec3::X).significant_height()
    );
}

#[test]
fn the_kestrel_climbs_on_the_collective_and_holds_its_attitude() {
    let mut world = World::new(RADIUS + 5.0);
    let mut craft = at_pole(Kind::Kestrel, 5.0 + 1.2, 0.0);
    craft.board();
    world.run(&mut craft, 2.0, |_| Input::default());
    world.run(&mut craft, 3.0, |_| Input {
        collective: 1.0,
        ..Default::default()
    });
    let Telemetry::Kestrel(t) = craft.telemetry.clone() else {
        panic!()
    };
    assert!(
        (t.vertical_speed - 7.0).abs() < 1.0,
        "climbing at {}",
        t.vertical_speed
    );
    let up = craft.body.position.normalize();
    let tilt = craft.body.axis(UP).angle_between(up).to_degrees();
    assert!(tilt < 3.0, "tilted {tilt} deg");
}

#[test]
fn an_empty_kestrel_holds_its_ground_in_a_crosswind_and_sets_down() {
    let mut world = World::new(RADIUS + 5.0);
    world.wind = Vec3::X * 10.0;
    let mut craft = at_pole(Kind::Kestrel, 40.0, 0.0);
    craft.body.velocity = DVec3::ZERO;
    // Left in the hover at 35 m over the ground, in a 10 m/s wind.
    world.run(&mut craft, 8.0, |_| Input::default());
    let up = craft.body.position.normalize();
    let drift = craft.body.velocity - up * craft.body.velocity.dot(up);
    assert!(drift.length() < 2.0, "drifting at {} m/s", drift.length());
    world.run(&mut craft, 30.0, |_| Input::default());
    let Telemetry::Kestrel(t) = craft.telemetry.clone() else {
        panic!()
    };
    assert!(t.touching >= 2, "not down: {} m up", t.height);
    assert!(craft.body.velocity.length() < 0.5);
}

#[test]
fn converting_below_the_stall_speed_sinks_it() {
    // Nacelles fully forward at 25 m/s, little power, and a pilot holding the
    // nose up to stop the sink: the wing cannot carry her, and stalls.
    let mut world = World::new(RADIUS - 200.0);
    let mut craft = at_pole(Kind::Kestrel, 150.0, 0.0);
    craft.board();
    if let CraftState::Kestrel(s) = &mut craft.state {
        s.nacelle = 0.0;
        s.lever = 0.1;
        s.throttle = 0.1;
    }
    craft.body.velocity = craft.body.axis(FORWARD) * 25.0;
    let mut stalled: f64 = 0.0;
    let mut sink: f64 = 0.0;
    world.run(&mut craft, 2.0, |c| {
        if let Telemetry::Kestrel(t) = &c.telemetry {
            stalled = stalled.max(t.stall);
            sink = sink.min(t.vertical_speed);
        }
        Input {
            pitch: 1.0,
            ..Default::default()
        }
    });
    assert!(sink < -5.0, "sank at most {sink} m/s");
    assert!(stalled > 0.5, "the wing never showed the stall");
}

/// Hold a heading off the true wind with the tiller, as a helmsman does.
fn helm(target: f64) -> impl FnMut(&Craft) -> Input {
    move |c: &Craft| {
        let error = (target - heading(c) + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        Input {
            steer: (error * 3.0).clamp(-1.0, 1.0) as f32,
            ..Default::default()
        }
    }
}

#[test]
fn the_tern_makes_way_to_windward_on_a_close_reach() {
    let mut world = World::new(RADIUS - 40.0);
    // Wind blowing toward +x: from -x. A heading 60 deg off the wind's source.
    world.wind = Vec3::X * 11.0;
    world.sea_wind = 11.0;
    let from = std::f64::consts::FRAC_PI_2; // bow toward -x points into it
    let mut craft = at_pole(Kind::Tern, 0.2, from - 60f64.to_radians());
    craft.board();
    if let CraftState::Tern(s) = &mut craft.state {
        s.sheet = 0.3;
    }
    world.run(&mut craft, 60.0, helm(from - 60f64.to_radians()));
    let t = tern_telemetry(&craft);
    assert!(t.speed > 1.0, "only {} m/s", t.speed);
    assert!(t.upwind > 0.2, "made good {} m/s to windward", t.upwind);
    assert!(
        t.heel.abs() < 45f64.to_radians(),
        "heeled {} deg",
        t.heel.to_degrees()
    );
}

#[test]
fn pointed_into_the_wind_the_tern_stops() {
    let mut world = World::new(RADIUS - 40.0);
    world.wind = Vec3::X * 11.0;
    let into = std::f64::consts::FRAC_PI_2;
    let mut craft = at_pole(Kind::Tern, 0.2, into);
    craft.board();
    craft.body.velocity = craft.body.axis(FORWARD) * 2.0;
    // She loses her way within seconds. Left longer, she gathers sternway,
    // the rudder works backwards, her bow falls off and she sails away on a
    // reach: what a boat in irons does, so the check is made before that.
    world.run(&mut craft, 11.0, helm(into));
    let t = tern_telemetry(&craft);
    let ahead = craft.body.velocity.dot(craft.body.axis(FORWARD));
    assert!(ahead < 0.3, "still making {ahead} m/s ahead in irons");
    assert_eq!(t.sail_state, SailState::Luffing);
}

#[test]
fn the_tern_cannot_beat_hull_speed() {
    let mut world = World::new(RADIUS - 40.0);
    world.wind = Vec3::X * 24.0;
    // A broad reach: the wind from abaft the beam.
    let course = -std::f64::consts::FRAC_PI_2 + 50f64.to_radians();
    let mut craft = at_pole(Kind::Tern, 0.2, course);
    craft.board();
    if let CraftState::Tern(s) = &mut craft.state {
        s.sheet = 0.7;
        s.crew = 1.1;
    }
    world.run(&mut craft, 60.0, helm(course));
    let t = tern_telemetry(&craft);
    let hull_speed = (G * 5.6 / std::f64::consts::TAU).sqrt();
    assert!(
        t.speed < hull_speed * 1.25,
        "{} m/s past a hull speed of {hull_speed}",
        t.speed
    );
}

#[test]
fn the_loon_paddles_and_turns_away_from_its_strokes() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
    let mut craft = at_pole(Kind::Loon, 0.1, 0.0);
    craft.board();
    world.run(&mut craft, 20.0, |_| Input {
        forward: 1.0,
        ..Default::default()
    });
    let Telemetry::Loon(t) = craft.telemetry.clone() else {
        panic!()
    };
    assert!((0.8..3.5).contains(&t.speed), "paddles at {} m/s", t.speed);
    assert!(t.strokes_per_minute > 40.0);
    let before = heading(&craft);
    world.run(&mut craft, 4.0, |_| Input {
        steer: 1.0,
        ..Default::default()
    });
    let turned = (heading(&craft) - before + std::f64::consts::PI)
        .rem_euclid(std::f64::consts::TAU)
        - std::f64::consts::PI;
    assert!(turned > 0.2, "paddling on the right turned it {turned} rad");
}

#[test]
fn rain_fills_an_open_canoe() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
    world.rain = 60.0;
    let mut craft = at_pole(Kind::Loon, 0.1, 0.0);
    craft.board();
    world.run(&mut craft, 60.0, |_| Input::default());
    assert!(
        (craft.bilge_kg - 3.6).abs() < 0.1,
        "{} kg aboard",
        craft.bilge_kg
    );
}

#[test]
fn an_untied_boat_drifts_downwind_and_a_moored_one_does_not() {
    for moored in [false, true] {
        let mut world = World::new(RADIUS - 40.0);
        world.wind = Vec3::X * 12.0;
        let mut craft = at_pole(Kind::Loon, 0.1, 0.0);
        let start = craft.body.position;
        if moored {
            craft.mooring = Some(Mooring {
                at: craft.bow(),
                length: 1.0,
                anchored: false,
            });
        }
        world.run(&mut craft, 20.0, |_| Input::default());
        let moved = (craft.body.position - start).dot(DVec3::X);
        if moored {
            assert!(moved < 4.0, "moored and moved {moved} m");
        } else {
            assert!(moved > 5.0, "drifted only {moved} m downwind");
        }
    }
}
