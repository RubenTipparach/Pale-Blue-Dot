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
    gravity: f64,
    current: DVec3,
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
            gravity: G,
            current: DVec3::ZERO,
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
                gravity: -up * self.gravity,
                current: self.current,
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
fn kestrel_assist_stays_finite_as_gravity_fades_to_zero() {
    for gravity in [0.0, 1e-12, 1e-5] {
        let mut world = World::new(RADIUS - 40.0);
        world.gravity = gravity;
        let mut craft = at_pole(Kind::Kestrel, 100.0, 0.0);
        craft.board();
        world.run(&mut craft, 1.0, |_| Input::default());
        let CraftState::Kestrel(state) = &craft.state else {
            unreachable!()
        };
        assert!(state.throttle.is_finite() && state.lever.is_finite());
    }
}

#[test]
fn kestrel_rotor_controls_need_power_and_preserve_their_signs() {
    for power in [0.0, 0.5] {
        let mut world = World::new(RADIUS - 40.0);
        world.gravity = 0.0;
        let mut craft = at_pole(Kind::Kestrel, 100.0, 0.0);
        craft.board();
        if let CraftState::Kestrel(state) = &mut craft.state {
            state.assist = false;
            state.throttle = power;
            state.lever = power;
        }
        world.run(&mut craft, TICK, |_| Input {
            pitch: 1.0,
            roll: 1.0,
            yaw: 1.0,
            ..Default::default()
        });
        if power == 0.0 {
            assert!(
                craft.body.angular_velocity.length() < 1e-12,
                "unpowered rotation {:?}",
                craft.body.angular_velocity
            );
            assert!(craft.body.velocity.length() < 1e-12, "unpowered thrust");
        } else {
            let spin = craft.body.local(craft.body.angular_velocity);
            assert!(
                spin.x > 0.0 && spin.y < 0.0 && spin.z < 0.0,
                "powered signs {spin:?}"
            );
            assert!(craft.body.velocity.y > 0.0);
        }
    }
}

#[test]
fn tern_leeway_does_not_confuse_a_cross_current_with_sliding_through_water() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
    world.current = DVec3::X * 2.0;
    let mut craft = at_pole(Kind::Tern, 0.2, 0.0);
    craft.board();
    craft.body.velocity = world.current;
    world.run(&mut craft, TICK, |_| Input::default());
    let t = tern_telemetry(&craft);
    assert!(t.speed > 1.9);
    assert!(t.water_speed < 0.005, "water speed {}", t.water_speed);
    assert!(
        // Windage has already begun moving it relative to the water during
        // this tick; that small physical slip is under one degree, not 89.
        t.leeway.abs() < 1.0_f64.to_radians(),
        "leeway {} degrees while moving with the water",
        t.leeway.to_degrees()
    );
}

#[test]
fn loon_instruments_separate_water_motion_from_ground_motion() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
    world.current = DVec3::new(2.0, 0.0, -1.0);
    let mut craft = at_pole(Kind::Loon, 0.1, 0.0);
    craft.board();
    craft.body.velocity = world.current;
    world.run(&mut craft, TICK, |_| Input::default());
    let Telemetry::Loon(t) = &craft.telemetry else {
        unreachable!()
    };
    assert!((t.speed - 1.0).abs() < 0.005 && (t.drift - 2.0).abs() < 0.005);
    assert!(t.water_speed.abs() < 0.005 && t.water_drift.abs() < 0.005);
}

#[test]
fn kestrel_can_convert_to_wing_support_with_pilot_control_of_the_nacelles() {
    let mut world = World::new(RADIUS - 40.0);
    let mut craft = at_pole(Kind::Kestrel, 100.0, 0.0);
    craft.board();
    world.run(&mut craft, 5.0, |_| Input::default());
    let Telemetry::Kestrel(hover) = &craft.telemetry else {
        unreachable!()
    };
    assert!(hover.climb_hold);
    let mut lowest = 100.0_f64;
    let mut pitch = 0.0_f64;
    world.run(&mut craft, 6.1, |c| {
        lowest = lowest.min(c.body.position.length() - RADIUS);
        pitch = pitch.max(
            c.body
                .axis(FORWARD)
                .dot(c.body.position.normalize())
                .asin()
                .abs(),
        );
        Input {
            tilt: -1.0,
            ..Default::default()
        }
    });
    let Telemetry::Kestrel(t) = &craft.telemetry else {
        unreachable!()
    };
    assert!(t.airspeed > 60.0 && t.wing_share > 0.8, "conversion {t:?}");
    assert!(!t.climb_hold && t.nacelle < 0.01);
    assert!(lowest > 80.0, "lost too much height: {lowest}");
    assert!(
        pitch < 12.0_f64.to_radians(),
        "pitched {} degrees",
        pitch.to_degrees()
    );
}

#[test]
fn loon_stern_rudder_turns_toward_the_selected_side_underway() {
    for rudder in [-1.0, 1.0] {
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
        world.run(&mut craft, 5.0, |_| Input::default());
        craft.body.velocity = FORWARD * 2.0;
        world.run(&mut craft, 1.0, |_| Input {
            rudder,
            ..Default::default()
        });
        assert!(heading(&craft) * rudder as f64 > 0.05);
    }
}

#[test]
fn loon_skeg_immersion_is_independent_of_the_lateral_plane() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
    for (pitch, height, wet) in [(0.5, 0.2, true), (-0.5, -0.2, false)] {
        let mut craft = at_pole(Kind::Loon, height, 0.0);
        craft.board();
        craft.set_reference_pose(DVec3::Y * (RADIUS + height), DQuat::from_rotation_x(pitch));
        craft.body.velocity = DVec3::new(0.4, 0.0, -2.0);
        let mut without_skeg = craft.clone();
        // Remove this surface only to isolate its contribution to the total wrench.
        Arc::make_mut(&mut without_skeg.specs).loon.skeg.area_m2 = 0.0;
        let ground = |_: DVec3| RADIUS - 40.0;
        let env = Surroundings {
            sea: &world.sea,
            sea_state: world.sea.state(0.0, Vec3::ZERO),
            sea_radius: RADIUS,
            depth: 40.0,
            air: AirHere {
                wind: Vec3::ZERO,
                upper: Vec3::ZERO,
                rain_mmh: 0.0,
                over_land: false,
            },
            gusts: &world.gusts,
            gravity: DVec3::NEG_Y * G,
            current: DVec3::ZERO,
            ground: &ground,
            seconds: 1000.0,
        };
        let cx = Context {
            env: &env,
            sea: world.sea.local(&env.sea_state, Vec3::Y, 40.0, env.seconds),
            up: DVec3::Y,
            ground: RADIUS - 40.0,
            seconds: env.seconds,
            dt: TICK / 4.0,
        };
        let skeg_depth = -cx.above_sea(craft.reference_point(craft.specs.loon.skeg.at));
        let lateral_depth = -cx.above_sea(craft.reference_point(craft.specs.loon.lateral.at));
        assert_eq!(skeg_depth > 0.0, wet);
        assert_eq!(lateral_depth > 0.0, !wet);
        loon::forces(&mut craft, &Input::default(), &cx);
        loon::forces(&mut without_skeg, &Input::default(), &cx);
        let force = craft.body.gathered().0 - without_skeg.body.gathered().0;
        if wet {
            assert!(force.length() > 1.0, "wet skeg has no force");
        } else {
            assert!(force.length() < 1e-9, "dry skeg force {force:?}");
        }
    }
}

#[test]
fn a_wet_foil_uses_orbital_velocity_and_current_at_its_own_depth() {
    let world = World::new(RADIUS - 40.0);
    let mut craft = at_pole(Kind::Tern, -1.0, 0.0);
    craft.body.velocity = DVec3::new(0.4, 0.0, -2.0);
    craft.body.angular_velocity = DVec3::Z * 0.2;
    let ground = |_: DVec3| RADIUS - 40.0;
    let env = Surroundings {
        sea: &world.sea,
        sea_state: world.sea.state(12.0, Vec3::X * 12.0),
        sea_radius: RADIUS,
        depth: 40.0,
        air: AirHere {
            wind: Vec3::ZERO,
            upper: Vec3::ZERO,
            rain_mmh: 0.0,
            over_land: false,
        },
        gusts: &world.gusts,
        gravity: DVec3::NEG_Y * G,
        current: DVec3::new(0.3, 0.0, 0.1),
        ground: &ground,
        seconds: 1000.0,
    };
    let cx = Context {
        env: &env,
        sea: world.sea.local(&env.sea_state, Vec3::Y, 40.0, env.seconds),
        up: DVec3::Y,
        ground: RADIUS - 40.0,
        seconds: env.seconds,
        dt: TICK / 4.0,
    };
    let spec = craft.specs.tern.keel;
    let at = craft.reference_point(spec.at);
    let depth = -cx.above_sea(at);
    assert!(depth > 0.0);
    let (_, water) = cx.water(at, depth);
    assert!(
        (water - cx.water(at, 0.0).1).length() > 1e-4,
        "orbital flow varies with depth"
    );
    let mut expected_body = craft.body;
    let expected = foil::apply(
        &mut expected_body,
        &spec,
        craft.com,
        water,
        SEA_DENSITY,
        0.2,
        1.0,
        1.0,
    );
    let actual = cx.wet_foil(&mut craft.body, &spec, craft.com, 0.2);
    assert!((actual.force - expected.force).length() < 1e-9);
    assert_eq!(actual.at, at);
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
