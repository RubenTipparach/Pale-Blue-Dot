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

#[test]
fn kestrel_reports_the_deflections_used_by_its_foils() {
    let mut craft = at_pole(Kind::Kestrel, 100.0, 0.0);
    craft.occupied = true;
    let CraftState::Kestrel(state) = &mut craft.state else {
        unreachable!()
    };
    state.assist = false;
    let input = Input {
        roll: 0.5,
        pitch: -0.25,
        yaw: 0.75,
        ..Default::default()
    };
    World::new(RADIUS - 100.0).run(&mut craft, TICK, |_| input);
    let Telemetry::Kestrel(t) = &craft.telemetry else {
        unreachable!()
    };
    let s = &craft.specs().kestrel;
    let expected = [
        s.wing.flap_rad as f64 + 0.5 * s.wing.aileron_rad as f64,
        s.wing.flap_rad as f64 - 0.5 * s.wing.aileron_rad as f64,
        0.25 * s.elevator_rad as f64,
        -0.75 * s.rudder_rad as f64,
    ];
    for (actual, expected) in t.surface_deflections.into_iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-10);
    }
}

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
fn loon_reports_a_working_rudder_at_rest_and_during_recovery() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
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
        dt: 0.0,
    };
    for phase in [None, Some(1.5), Some(0.5)] {
        for rudder in [-1.0, 1.0] {
            let mut craft = at_pole(Kind::Loon, 0.1, 0.0);
            craft.board();
            let CraftState::Loon(st) = &mut craft.state else {
                unreachable!()
            };
            st.stroke = phase.map(|phase| Stroke {
                side: 1.0,
                direction: 1.0,
                phase,
            });
            loon::forces(
                &mut craft,
                &Input {
                    rudder,
                    ..Default::default()
                },
                &cx,
            );
            let Telemetry::Loon(t) = &craft.telemetry else {
                unreachable!()
            };
            if phase.is_some_and(|p| p < 1.0) {
                assert_eq!(t.rudder_at, None, "the power stroke has priority");
            } else {
                let p = craft.specs.loon.paddle;
                let at = t.rudder_at.expect("working even at rest");
                assert_eq!(
                    at,
                    DVec3::new(
                        -rudder as f64 * p.rudder_at[0] as f64,
                        p.depth_m as f64,
                        p.rudder_at[1] as f64
                    )
                );
                assert_eq!(t.blade.force, DVec3::ZERO);
                assert!(
                    t.blade
                        .at
                        .distance(craft.reference_point(at.as_vec3().to_array()))
                        < 1e-6
                );
            }
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
    for kind in [Kind::Tern, Kind::Loon, Kind::Cog] {
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
            Telemetry::Cog(t) => t.displaced_m3,
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

/// Instrument (`sail-the-cog` step 3): where the cog floats at rest, and
/// what it makes at each point of sail over the yard's angles.
#[test]
#[ignore = "instrument: prints the cog's waterline and its speed and heel by yard"]
fn print_the_cogs_sailing() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
    let mut craft = at_pole(Kind::Cog, 0.0, 0.0);
    world.run(&mut craft, 30.0, |_| Input::default());
    let Telemetry::Cog(t) = &craft.telemetry else {
        panic!("a cog");
    };
    println!(
        "mass {:.0} kg, displaced {:.1} m^3 (weighs {:.1}), waterline origin {:+.2} m, hull {} cells",
        craft.body.mass,
        t.displaced_m3,
        craft.body.mass / SEA_DENSITY,
        craft.reference_position().length() - RADIUS,
        craft.hull().unwrap().cells.len()
    );
    // The wind blows toward +x, from -x. A heading `off` the wind's source.
    let from = std::f64::consts::FRAC_PI_2;
    for off in [70.0f64, 90.0, 120.0, 180.0] {
        let mut row = format!("{off:>4} off:");
        for yard in [-60.0f64, -40.0, -20.0, 0.0, 20.0, 40.0, 60.0] {
            let mut world = World::new(RADIUS - 40.0);
            world.wind = Vec3::X * 8.0;
            world.sea_wind = 8.0;
            let course = from - off.to_radians();
            let mut craft = at_pole(Kind::Cog, 0.0, course);
            craft.board();
            if let CraftState::Cog(s) = &mut craft.state {
                s.yard = yard.to_radians();
            }
            world.run(&mut craft, 90.0, helm(course));
            let Telemetry::Cog(t) = &craft.telemetry else {
                panic!("a cog");
            };
            row += &format!(
                "  {yard:+3}: {:.2} m/s {:+.1} up {:.1} deg",
                t.speed,
                t.upwind,
                t.heel.to_degrees()
            );
        }
        println!("{row}");
    }
}

fn cog_telemetry(craft: &Craft) -> CogTelemetry {
    match &craft.telemetry {
        Telemetry::Cog(t) => t.clone(),
        _ => panic!("not a cog"),
    }
}

/// `sail-the-cog` task 2.1: the cog floats on the waterline its reference
/// frame is drawn about, the harbour's moored piece's, to a hand.
#[test]
fn the_cog_floats_on_its_waterline() {
    let mut world = World::new(RADIUS - 40.0);
    world.sea = SeaTable::new(
        SeaSettings {
            swell_height_m: 0.0,
            ..Default::default()
        },
        G as f32,
    );
    let mut craft = at_pole(Kind::Cog, 0.0, 0.0);
    world.run(&mut craft, 30.0, |_| Input::default());
    let waterline = craft.reference_position().length() - RADIUS;
    assert!(
        waterline.abs() < 0.15,
        "its waterline {waterline:+.2} m off"
    );
}

/// `sail-the-cog` task 2.2: the braces turn the yard at their rate and no
/// further than they reach, and only with someone at the helm; nobody at
/// it, the yard and the tiller stay as they were left.
#[test]
fn the_cogs_yard_follows_its_braces() {
    let mut world = World::new(RADIUS - 40.0);
    let mut craft = at_pole(Kind::Cog, 0.0, 0.0);
    if let CraftState::Cog(s) = &mut craft.state {
        s.tiller = 0.3;
    }
    let brace = Input {
        sheet: 1.0,
        ..Default::default()
    };
    world.run(&mut craft, 2.0, |_| brace);
    assert_eq!(cog_telemetry(&craft).yard, 0.0, "nobody at the helm");
    assert!(
        matches!(&craft.state, CraftState::Cog(s) if s.tiller == 0.3),
        "the tiller left over stays over"
    );
    craft.board();
    world.run(&mut craft, 2.0, |_| brace);
    let yard = cog_telemetry(&craft).yard.to_degrees();
    assert!((yard - 24.0).abs() < 1.0, "braced {yard} deg in 2 s");
    world.run(&mut craft, 6.0, |_| brace);
    let yard = cog_telemetry(&craft).yard.to_degrees();
    assert!(
        (yard - 60.0).abs() < 0.01,
        "braced to {yard} deg, past its reach"
    );
}

/// How a cog sails a course `off` degrees from where an 8 m/s wind comes
/// from, the yard braced at `yard` degrees and a helmsman holding the course:
/// the mean over 60-120 s, once it has settled.
#[derive(Debug)]
struct CogRun {
    /// Speed made good to windward and along its own bow, m/s.
    upwind: f64,
    ahead: f64,
    /// Leeway and the most the bow wandered off the course, and the most it
    /// heeled, deg.
    leeway: f64,
    wander: f64,
    heel: f64,
}

fn sail_cog(off: f64, yard: f64) -> CogRun {
    let mut world = World::new(RADIUS - 40.0);
    // The wind blows toward +x, from -x, where a heading of 90 deg points.
    world.wind = Vec3::X * 8.0;
    world.sea_wind = 8.0;
    let course = std::f64::consts::FRAC_PI_2 - off.to_radians();
    let mut craft = at_pole(Kind::Cog, 0.0, course);
    craft.board();
    if let CraftState::Cog(s) = &mut craft.state {
        s.yard = yard.to_radians();
    }
    world.run(&mut craft, 60.0, helm(course));
    let mut run = CogRun {
        upwind: 0.0,
        ahead: 0.0,
        leeway: 0.0,
        wander: 0.0,
        heel: 0.0,
    };
    let n = 60;
    for _ in 0..n {
        world.run(&mut craft, 1.0, helm(course));
        let t = cog_telemetry(&craft);
        run.upwind += t.upwind / n as f64;
        run.ahead += craft.body.velocity.dot(craft.body.axis(FORWARD)) / n as f64;
        run.leeway += t.leeway.to_degrees() / n as f64;
        let wander = (heading(&craft) - course + std::f64::consts::PI)
            .rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        run.wander = run.wander.max(wander.abs().to_degrees());
        run.heel = run.heel.max(t.heel.abs().to_degrees());
    }
    run
}

/// `sail-the-cog` task 2.2: a square sail luffs with the wind along its
/// yard, from either end, and draws with the wind across it.
#[test]
fn a_square_sail_luffs_with_the_wind_along_its_yard() {
    let sail = VehicleSpecs::default().cog.sail;
    for deg in [0.0f64, 10.0, 175.0, -178.0] {
        assert_eq!(cog::fill(&sail, deg.to_radians()), 0.0, "at {deg} deg");
    }
    for deg in [35.0f64, -60.0, 90.0, 140.0] {
        assert_eq!(cog::fill(&sail, deg.to_radians()), 1.0, "at {deg} deg");
    }
}

/// `sail-the-cog` task 2.2: in an 8 m/s wind the cog runs and reaches at
/// better than 2 m/s and heels under 15 degrees on the reach. To windward it
/// does best braced hard 60 degrees off the wind, where it makes about 9
/// degrees of leeway, so it makes good a track little closer than 70
/// degrees; pinched to 45 it makes less, and it cannot hold 40.
#[test]
fn the_cog_reaches_runs_and_cannot_point_high() {
    let reach = sail_cog(90.0, -40.0);
    assert!(reach.ahead > 2.0, "reaching at {} m/s", reach.ahead);
    assert!(reach.heel < 15.0, "heeled {} deg on the reach", reach.heel);
    let run = sail_cog(180.0, 0.0);
    assert!(run.ahead > 2.0, "running at {} m/s", run.ahead);
    let close = sail_cog(60.0, -60.0);
    assert!(
        close.wander < 10.0,
        "wandered {} deg close-hauled",
        close.wander
    );
    assert!(
        close.upwind > 0.3,
        "made good {} m/s to windward",
        close.upwind
    );
    assert!(
        close.leeway > 4.0 && 60.0 + close.leeway > 65.0,
        "a track {} deg off the wind",
        60.0 + close.leeway
    );
    let pinched = sail_cog(45.0, -60.0);
    assert!(
        pinched.upwind < close.upwind,
        "made good {} m/s to windward at 45 deg against {} at 60",
        pinched.upwind,
        close.upwind
    );
    let lost = sail_cog(40.0, -60.0);
    assert!(lost.wander > 20.0, "held 40 deg within {} deg", lost.wander);
}

/// `sail-the-cog` task 5.1: a cog's yard comes back from its record, and
/// every other craft's record carries none.
#[test]
fn a_cogs_yard_is_saved_with_it() {
    let (specs, hulls) = specs();
    let mut craft = at_pole(Kind::Cog, 0.0, 0.3);
    if let CraftState::Cog(s) = &mut craft.state {
        s.yard = 0.4;
    }
    let record = craft.record();
    assert_eq!(record.kind, "cog");
    assert_eq!(record.yard, Some(0.4));
    let back = Craft::from_record(&record, specs, hulls).unwrap();
    assert!(matches!(&back.state, CraftState::Cog(s) if (s.yard - 0.4).abs() < 1e-12));
    assert_eq!(at_pole(Kind::Tern, 0.2, 0.0).record().yard, None);
}

#[test]
#[ignore = "instrument: the cog's settled polar, by course and brace"]
fn print_the_cogs_polar() {
    for off in [40.0f64, 50.0, 60.0, 70.0, 80.0, 90.0] {
        let mut row = format!("{off:>4} off:");
        for yard in [-20.0f64, -30.0, -40.0, -50.0, -60.0] {
            let r = sail_cog(off, yard);
            row += &format!(
                " | {yard:+3}: {:+.2} up {:+.2} ahead lee {:+4.1} wander {:4.1}",
                r.upwind, r.ahead, r.leeway, r.wander
            );
        }
        println!("{row}");
    }
}

/// `sail-the-cog` step 3: the craft sails the ship the harbour moors, its
/// sail's force where the ship's cut draws its mast, yard and sail.
#[test]
fn vehicle_specs_match_the_ship_they_sail() {
    use crate::settlement::pieces::cog as ship;
    let sail = VehicleSpecs::default().cog.sail;
    assert!(Vec3::from(sail.mast).distance(ship::MAST_STEP) < 1e-5);
    assert_eq!(sail.yard_height_m, ship::YARD_M);
    assert_eq!(sail.width_m, ship::SAIL_W);
    assert_eq!(sail.foot_height_m, ship::SAIL_FOOT_M);
}

/// `sail-the-cog` task 2.1: the cog's settings refuse a hull too coarse for
/// its beam, a sail with no yard over its foot, and a luff that does not
/// rise to where the sail fills.
#[test]
fn a_cogs_settings_refuse_a_coarse_hull_a_yardless_sail_and_a_backward_luff() {
    let refused = |change: &dyn Fn(&mut spec::CogSpec)| {
        let mut specs = VehicleSpecs::default();
        change(&mut specs.cog);
        specs.validate().unwrap_err()
    };
    assert!(refused(&|c| c.hull.cell_m = c.hull.beam_m / 2.0).contains("third of the beam"));
    assert!(refused(&|c| c.sail.yard_height_m = c.sail.foot_height_m).contains("yard"));
    assert!(refused(&|c| c.sail.fill_deg = c.sail.luff_deg).contains("luff"));
}

#[test]
#[ignore = "instrument: an empty moored boat left alone, by depth: does it stay upright?"]
fn print_an_empty_moored_boat_by_depth() {
    for kind in [Kind::Loon, Kind::Tern] {
        for depth in [0.3f64, 0.5, 0.8, 1.2, 2.0, 3.0, 5.0, 40.0] {
            let mut world = World::new(RADIUS - depth);
            let mut craft = at_pole(kind, 0.0, 0.0);
            let bed = RADIUS - depth;
            craft.mooring = Some(Mooring {
                at: craft.bow().normalize() * bed,
                length: depth + 2.0,
                anchored: true,
            });
            let mut worst: f64 = 1.0;
            for _ in 0..30 {
                world.run(&mut craft, 1.0, |_| Input::default());
                let up = craft.body.position.normalize();
                worst = worst.min(craft.body.axis(DVec3::Y).dot(up));
            }
            let up = craft.body.position.normalize();
            println!(
                "{:>4} in {depth:>4} m: deck up . up {:+.3} (worst {:+.3}), waterline {:+.2} m, speed {:.3}",
                kind.name(),
                craft.body.axis(DVec3::Y).dot(up),
                worst,
                craft.reference_position().length() - RADIUS,
                craft.body.velocity.length()
            );
        }
    }
}

#[test]
#[ignore = "instrument: a Tern with its keel raised and its ballast in the hull, moored in shallow water in a wind"]
fn print_a_tern_with_its_keel_raised() {
    let mut specs = VehicleSpecs::default();
    // The keel raised into the hull: its ballast at the hull's bottom, its
    // foil and its grounding point just under it.
    specs.tern.parts[1].at[1] = -0.3;
    specs.tern.keel.at[1] = -0.35;
    specs.tern.contacts[0].at[1] = -0.42;
    let hulls = Hulls::new(&specs);
    let specs = Arc::new(specs);
    for wind in [0.0f32, 6.0, 10.0, 14.0] {
        for depth in [0.5f64, 0.8] {
            let mut world = World::new(RADIUS - depth);
            world.wind = Vec3::X * wind;
            world.sea_wind = wind;
            let mut craft = Craft::new(
                Kind::Tern,
                1,
                specs.clone(),
                hulls.clone(),
                DVec3::new(0.0, RADIUS, 0.0),
                DQuat::IDENTITY,
            );
            craft.mooring = Some(Mooring {
                at: craft.bow().normalize() * (RADIUS - depth),
                length: depth + 2.0,
                anchored: true,
            });
            let mut worst: f64 = 1.0;
            for _ in 0..60 {
                world.run(&mut craft, 1.0, |_| Input::default());
                let up = craft.body.position.normalize();
                worst = worst.min(craft.body.axis(DVec3::Y).dot(up));
            }
            println!(
                "wind {wind:>4} m/s, {depth} m of water: worst heel {:.1} deg, waterline {:+.2} m",
                worst.clamp(-1.0, 1.0).acos().to_degrees(),
                craft.reference_position().length() - RADIUS
            );
        }
    }
}

/// `cities-in-the-world` task 4.2b: a Tern's keel goes as deep as the water
/// under it allows. Moored in half a metre of water it lifts clear of the
/// seabed at once, and the Tern floats upright there; in open water it is
/// all the way down; when the water deepens it lowers in about three
/// seconds.
#[test]
fn a_terns_keel_lifts_to_the_water_under_it() {
    let down = |craft: &Craft| match &craft.state {
        CraftState::Tern(s) => s.keel,
        _ => panic!("a Tern"),
    };
    let lift = VehicleSpecs::default().tern.lift;
    let tip = VehicleSpecs::default().tern.contacts[lift.tip].at;
    let mut world = World::new(RADIUS - 0.5);
    world.wind = Vec3::X * 6.0;
    world.sea_wind = 6.0;
    let mut craft = at_pole(Kind::Tern, 0.0, 0.0);
    craft.mooring = Some(Mooring {
        at: craft.bow().normalize() * (RADIUS - 0.5),
        length: 2.5,
        anchored: true,
    });
    world.run(&mut craft, TICK, |_| Input::default());
    assert!(down(&craft) < 0.1, "lifted at once: {} down", down(&craft));
    let mut worst: f64 = 1.0;
    for _ in 0..30 {
        world.run(&mut craft, 1.0, |_| Input::default());
        let up = craft.body.position.normalize();
        worst = worst.min(craft.body.axis(DVec3::Y).dot(up));
        let at = DVec3::new(
            tip[0] as f64,
            tip[1] as f64 + lift.raised_m(down(&craft)),
            tip[2] as f64,
        );
        let clear = craft.body.point(at - craft.com).length() - (RADIUS - 0.5);
        assert!(clear > 0.0, "the keel's tip {clear:.2} m into the seabed");
    }
    assert!(
        worst.acos().to_degrees() < 8.0,
        "heeled {:.1} deg at its berth",
        worst.acos().to_degrees()
    );
    // Out over deep water it lowers at its rate.
    world.ground = RADIUS - 40.0;
    world.run(&mut craft, 1.5, |_| Input::default());
    let half = down(&craft);
    assert!((0.4..0.75).contains(&half), "{half} down after 1.5 s");
    world.run(&mut craft, 2.0, |_| Input::default());
    assert_eq!(down(&craft), 1.0, "all the way down");
}
