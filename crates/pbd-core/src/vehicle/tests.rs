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
            let CraftState::Loon(st) = &mut craft.state;
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
    let mut craft = at_pole(Kind::Loon, -1.0, 0.0);
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
    let spec = craft.specs.loon.skeg;
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
    for kind in [Kind::Loon, Kind::Tube, Kind::Raft] {
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
    let mut craft = at_pole(Kind::Loon, 0.2, 0.0);
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

#[test]
#[ignore = "instrument: an empty moored boat left alone, by depth: does it stay upright?"]
fn print_an_empty_moored_boat_by_depth() {
    for kind in [Kind::Loon] {
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

/// Where the sea's surface is under a point of a craft, m over it: the
/// sea of `World::run`, its swell running, sampled as `float` samples it.
fn over_the_sea(world: &World, at: DVec3) -> f64 {
    let state = world.sea.state(world.sea_wind, world.wind);
    let up = at.normalize().as_vec3();
    let sea = world.sea.local(&state, up, 40.0, world.seconds);
    at.length() - RADIUS - sea.at(at.as_vec3(), RADIUS as f32, 0.0).height as f64
}

/// The corners of the floorboards an open boat is drawn with, and the rest
/// of its planks, in its reference frame.
fn floorboards_of(craft: &Craft) -> (Vec<Vec3>, Vec<Vec3>) {
    let h = craft.loon_spec().hull;
    let (kind, wood) = ("canoe", "driftwood");
    let (middle, rise) = craft.floor().expect("an open boat");
    let cut = crate::settlement::pieces::dressing::small_boat(
        &|_| 1.5,
        kind,
        Vec3::new(h.length_m, h.beam_m, h.depth_m),
        h.sheer_m,
        (middle, rise),
    );
    cut[wood]
        .positions
        .iter()
        .map(|p| Vec3::from(*p))
        .partition(|p| (p.y - (middle + rise * p.z)).abs() < 1e-4)
}

#[test]
fn an_open_boats_floorboards_stand_over_the_sea_inside_its_hull() {
    for kind in [Kind::Loon] {
        let mut craft = at_pole(kind, 0.0, 0.0);
        craft.occupied = true;
        let mut world = World::new(RADIUS - 40.0);
        world.run(&mut craft, 30.0, |_| Input::default());
        let floor = floorboards_of(&craft).0;
        assert!(floor.len() > 40, "{}: {} corners", kind.name(), floor.len());
        // Inside the hull as drawn: its planks at the floor's height are
        // at least as far out to that side, near that station.
        let skin = floorboards_of(&craft).1;
        for p in &floor {
            assert!(
                skin.iter().any(|q| (q.z - p.z).abs() < 0.2
                    && q.x * p.x >= 0.0
                    && q.x.abs() >= p.x.abs()
                    && (q.y - p.y).abs() < 0.08),
                "{p} through the planks"
            );
        }
        // Through a whole swell, the boat riding it.
        let mut least = f64::MAX;
        for _ in 0..50 {
            world.run(&mut craft, 0.5, |_| Input::default());
            for p in &floor {
                least = least.min(over_the_sea(&world, craft.reference_point(p.to_array())));
            }
        }
        let (middle, rise) = craft.floor().unwrap();
        println!(
            "{}: floor {middle:.3} m amidships rising {rise:.4} aft, least over the sea {least:.3} m",
            kind.name()
        );
        assert!(
            least > FLOOR_OVER_M as f64 / 2.0,
            "{}: the floor comes within {least:.3} m of the sea",
            kind.name()
        );
    }
}

#[test]
#[ignore = "instrument: an empty boat settling on a still sea"]
fn print_an_empty_boat_settling() {
    for kind in [Kind::Loon] {
        let mut craft = at_pole(kind, 0.0, 0.0);
        let mut world = World::new(RADIUS - 40.0);
        for _ in 0..12 {
            world.run(&mut craft, 5.0, |_| Input::default());
            let v = craft.body.velocity;
            println!(
                "{} t {:>3.0}: v {:+.3} {:+.3} {:+.3}, w {:.3}, pos {:+.3} {:+.3}",
                kind.name(),
                world.seconds - 1000.0,
                v.x,
                v.y,
                v.z,
                craft.body.angular_velocity.length(),
                craft.body.position.x,
                craft.body.position.z
            );
        }
    }
}

#[test]
#[ignore = "instrument: an open boat with its paddler aboard, heel and trim"]
fn print_an_open_boat_with_its_paddler() {
    for kind in [Kind::Loon] {
        let mut craft = at_pole(kind, 0.0, 0.0);
        craft.occupied = true;
        let mut world = World::new(RADIUS - 40.0);
        for _ in 0..8 {
            world.run(&mut craft, 5.0, |_| Input::default());
            let up = craft.body.position.normalize();
            let heel = craft.body.axis(RIGHT).dot(up).asin().to_degrees();
            let trim = craft.body.axis(FORWARD).dot(up).asin().to_degrees();
            println!(
                "{} t {:>3.0}: heel {heel:+.2} deg, bow up {trim:+.2} deg, com {:?}, mass {:.0}",
                kind.name(),
                world.seconds - 1000.0,
                craft.com,
                craft.body.mass
            );
        }
        println!("  floor {:?}", craft.floor());
    }
}

/// The inner tube and the raft (`small-boats` 1.1): each floats upright,
/// empty and with its paddler, its bottom under the sea and its top over it,
/// and moves ahead under a forward stroke.
#[test]
fn a_tube_and_a_raft_float_upright_and_paddle_ahead() {
    for (kind, bottom_y, top_y, least_m) in
        [(Kind::Tube, -0.14, 0.2, 2.0), (Kind::Raft, -0.1, 0.2, 6.0)]
    {
        for occupied in [false, true] {
            let mut craft = at_pole(kind, 0.0, 0.0);
            craft.occupied = occupied;
            craft.weigh();
            let mut world = World::new(RADIUS - 40.0);
            world.run(&mut craft, 30.0, |_| Input::default());
            let up = craft.body.position.normalize();
            let upright = craft.body.axis(DVec3::Y).dot(up);
            let bottom = over_the_sea(&world, craft.reference_point([0.0, bottom_y, 0.0]));
            let top = over_the_sea(&world, craft.reference_point([0.0, top_y, 0.0]));
            println!(
                "{}, occupied {occupied}: upright {upright:.4}, bottom {bottom:+.3} m, top {top:+.3} m, bilge {:.1} kg",
                kind.name(),
                craft.bilge_kg
            );
            assert!(
                upright > 5f64.to_radians().cos(),
                "{kind:?} heeled: {upright}"
            );
            assert!(
                bottom < 0.0 && top > 0.05,
                "{kind:?}: bottom {bottom}, top {top}"
            );
            assert!(craft.bilge_kg < 1.0, "{kind:?} took water");
        }
        let mut craft = at_pole(kind, 0.0, 0.0);
        craft.occupied = true;
        craft.weigh();
        let mut world = World::new(RADIUS - 40.0);
        world.run(&mut craft, 5.0, |_| Input::default());
        let start = craft.body.position;
        let ahead = craft.body.axis(FORWARD);
        world.run(&mut craft, 30.0, |_| Input {
            forward: 1.0,
            ..Default::default()
        });
        let made = (craft.body.position - start).dot(ahead);
        let speed = craft.body.velocity.dot(craft.body.axis(FORWARD));
        println!(
            "{} paddled 30 s: {made:.1} m ahead, {speed:.2} m/s",
            kind.name()
        );
        assert!(made > least_m, "{kind:?}: {made} m, {speed} m/s");
    }
}

/// A made boat's good names its kind, both ways, and only made kinds have one.
#[test]
fn a_boat_good_names_its_kind() {
    for kind in [Kind::Tube, Kind::Raft] {
        let code = kind.good_code().expect("a made kind");
        assert_eq!(Kind::of_good_code(code), Some(kind));
        assert!(
            crate::goods::Good::by_code(code).is_some(),
            "{code} is a good"
        );
    }
    assert_eq!(Kind::Loon.good_code(), None);
    assert_eq!(
        Kind::of_good_code("rowboat"),
        None,
        "the rowboat is retired"
    );
    assert!(Kind::Tube.paddled() && Kind::Raft.paddled() && Kind::Loon.paddled());
}

/// The tube's rider turns it by kicking on one side (`small-boats`): ten
/// seconds of A turns it a good way round, with no rudder at all.
#[test]
fn a_tube_turns_on_its_kicks() {
    let mut craft = at_pole(Kind::Tube, 0.0, 0.0);
    craft.occupied = true;
    craft.weigh();
    let mut world = World::new(RADIUS - 40.0);
    world.run(&mut craft, 3.0, |_| Input::default());
    let before = heading(&craft);
    world.run(&mut craft, 10.0, |_| Input {
        steer: 1.0,
        ..Default::default()
    });
    let turned = (heading(&craft) - before).abs().to_degrees();
    println!("tube turned {turned:.0} deg in 10 s of kicking to one side");
    assert!(turned > 30.0, "turned {turned} deg");
}

/// `river-craft` design 9.4: one oar alone pushes 2.75 m off the centre
/// line and turns the raft with no turn force of its own: A rows the right
/// oar and the raft turns left, D the left oar and it turns right.
#[test]
fn a_raft_rowed_on_one_oar_turns_away_from_it() {
    for (steer, sign) in [(1.0, 1.0), (-1.0, -1.0)] {
        let mut craft = at_pole(Kind::Raft, 0.0, 0.0);
        craft.board();
        let mut world = World::new(RADIUS - 40.0);
        world.run(&mut craft, 3.0, |_| Input::default());
        // Summed tick by tick, so a turn past half round does not wrap.
        let mut last = heading(&craft);
        let mut turned = 0.0;
        world.run(&mut craft, 8.0, |c| {
            let now = heading(c);
            let step = (now - last + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
                - std::f64::consts::PI;
            turned += step.to_degrees();
            last = now;
            Input {
                steer,
                ..Default::default()
            }
        });
        println!("raft on one oar (steer {steer}): turned {turned:+.0} deg in 8 s");
        assert!(turned * sign > 15.0, "turned {turned} deg");
        let (_, rowing) = craft.rowing().expect("the raft is rowed");
        let rowed = [rowing.oars[0].catches, rowing.oars[1].catches];
        assert!(
            (sign > 0.0 && rowed[0] == 0 && rowed[1] > 3)
                || (sign < 0.0 && rowed[1] == 0 && rowed[0] > 3),
            "the wrong oar rowed: {rowed:?}"
        );
    }
}

/// `river-craft` design 9.2: where the bed is shallower than the blade's
/// stroke the blade rests on it and pulls only with its share still in the
/// water; nothing pushes off the bed, so the pull is far less than in deep
/// water and the raft makes little way.
#[test]
fn an_oar_over_a_shallow_bed_rests_on_it_and_pushes_nothing_off_it() {
    let row = |ground: f64| {
        let mut craft = at_pole(Kind::Raft, 0.0, 0.0);
        craft.board();
        let mut world = World::new(ground);
        world.run(&mut craft, 3.0, |_| Input::default());
        let start = craft.body.position;
        let mut strongest: f64 = 0.0;
        let mut on_bed = false;
        world.run(&mut craft, 10.0, |c| {
            if let Telemetry::Loon(t) = &c.telemetry {
                strongest = strongest.max(t.blade.force.length());
            }
            on_bed |= c.rowing().is_some_and(|(_, r)| r.on_bed);
            Input {
                forward: 1.0,
                ..Default::default()
            }
        });
        let made = (craft.body.position - start).length();
        (made, strongest, on_bed)
    };
    let (deep_m, deep_n, deep_bed) = row(RADIUS - 40.0);
    let (shallow_m, shallow_n, shallow_bed) = row(RADIUS - 0.16);
    println!(
        "rowed 10 s: deep {deep_m:.1} m at up to {deep_n:.0} N; on a 0.16 m bed {shallow_m:.1} m at up to {shallow_n:.0} N"
    );
    assert!(
        !deep_bed && shallow_bed,
        "the blade rests on the shallow bed"
    );
    assert!(shallow_n < deep_n * 0.6, "{shallow_n} N against {deep_n} N");
    assert!(shallow_m < deep_m * 0.6, "{shallow_m} m against {deep_m} m");
}

/// The bow's horizontal distance from the anchor, m.
fn swing(craft: &Craft) -> f64 {
    let a = craft.anchor.expect("the anchor is out").at;
    let bow = craft.bow();
    let up = a.normalize();
    let d = bow - a;
    (d - up * d.dot(up)).length()
}

/// `river-craft` design 4 (version 5): the rode pays out only until the
/// anchor touches the bed; it does not veer the old three times the depth.
#[test]
fn a_dropped_anchors_rode_stops_at_the_bed() {
    let mut craft = at_pole(Kind::Raft, 0.0, 0.0);
    craft.board();
    let mut world = World::new(RADIUS - 3.0);
    world.run(&mut craft, 2.0, |_| Input::default());
    craft.drop_anchor();
    world.run(&mut craft, 6.0, |_| Input::default());
    let a = craft.anchor.expect("on the bed");
    let plumb = a.plumb(craft.bow());
    println!(
        "anchor in 3 m: {}, rode {:.2} m, bow {plumb:.2} m over it",
        anchor::word(Some(&a)),
        a.rode
    );
    assert!(a.holds(), "{:?}", a.phase);
    assert!((a.at.length() - (RADIUS - 3.0)).abs() < 1e-6, "on the bed");
    assert!(a.rode < plumb + 1.0, "rode {} over {plumb}", a.rode);
    assert!(craft.anchored());
}

/// `river-craft` design 4 (version 4): a set anchor holds the boat within
/// its swing on the rode whatever rows or flows; only weighing frees it,
/// and a longer rope from the boat's menu lets it swing further.
#[test]
fn a_set_anchor_holds_a_rowing_raft_and_only_weighing_frees_it() {
    let mut craft = at_pole(Kind::Raft, 0.0, 0.0);
    craft.board();
    let mut world = World::new(RADIUS - 3.0);
    world.current = DVec3::new(0.8, 0.0, 0.0);
    world.run(&mut craft, 1.0, |_| Input::default());
    craft.drop_anchor();
    world.run(&mut craft, 5.0, |_| Input::default());
    let set_at = craft.anchor.unwrap().at;
    let rode = craft.anchor.unwrap().rode;
    let mut widest: f64 = 0.0;
    world.run(&mut craft, 30.0, |c| {
        widest = widest.max(swing(c));
        Input {
            forward: 1.0,
            ..Default::default()
        }
    });
    let held = craft.anchor.unwrap();
    println!(
        "rowed 30 s at anchor in a 0.8 m/s current: swung {widest:.2} m on a {rode:.2} m rode; anchor moved {:.3} m",
        held.at.distance(set_at)
    );
    assert!(widest < rode + 0.3, "swung {widest} m on {rode} m");
    assert!(held.at.distance(set_at) < 1e-6, "a set anchor never moves");
    // A longer rope: veered out, the boat drops back further.
    craft.set_rope(Some(12.0));
    for _ in 0..8 {
        world.run(&mut craft, 5.0, |_| Input::default());
        println!(
            "  swing {:.2} m, rode {:.2}, speed {:.2} m/s",
            swing(&craft),
            craft.anchor.unwrap().rode,
            craft.body.velocity.length()
        );
    }
    let a = craft.anchor.unwrap();
    assert!((a.rode - 12.0).abs() < 1e-6, "veered to {}", a.rode);
    let out = swing(&craft);
    println!("rope 12 m: swung {out:.2} m");
    assert!(out > widest + 4.0 && out < 12.3, "{out} m");
    // Back to the bed: hauled in.
    craft.set_rope(None);
    world.run(&mut craft, 20.0, |_| Input::default());
    assert!(craft.anchor.unwrap().rode < rode + 0.5);
    // Weighing: the only way it lets go, and the boat moves on.
    craft.weigh_anchor();
    world.run(&mut craft, 10.0, |_| Input::default());
    assert!(craft.anchor.is_none() && craft.mooring.is_none());
    let start = craft.body.position;
    world.run(&mut craft, 5.0, |_| Input::default());
    assert!((craft.body.position - start).length() > 2.0, "carried off");
}

/// `river-craft` design 10.5: the rope setting and the lantern put out are
/// written only when set, so an older record reads and writes as it was;
/// an anchored record comes back set and holding.
#[test]
fn the_boats_settings_survive_a_save_and_old_records_read_unchanged() {
    use record::{VehicleFile, VehicleRecord};
    let mut craft = at_pole(Kind::Raft, 0.0, 0.0);
    let plain = ron::to_string(&craft.record()).unwrap();
    assert!(
        !plain.contains("rope_m") && !plain.contains("lantern_out"),
        "{plain}"
    );
    craft.rope_m = Some(14.0);
    craft.lantern_out = true;
    craft.mooring = Some(Mooring {
        at: DVec3::new(0.0, RADIUS - 3.0, 0.0),
        length: 3.6,
        anchored: true,
    });
    let text = ron::to_string(&craft.record()).unwrap();
    let (specs, hulls) = specs();
    let back =
        Craft::from_record(&ron::from_str(&text).unwrap(), specs.clone(), hulls.clone()).unwrap();
    assert_eq!(back.rope_m, Some(14.0));
    assert!(back.lantern_out);
    assert!(back.anchored() && back.anchor.unwrap().holds());
    assert_eq!(back.anchor.unwrap().rode, 3.6);
    // A record from before: no rope, the lantern on.
    let old: VehicleRecord = ron::from_str(&plain).unwrap();
    assert_eq!((old.rope_m, old.lantern_out), (None, false));
    let file = VehicleFile {
        version: record::RECORD_VERSION,
        next_id: 2,
        vehicles: vec![old],
    };
    assert_eq!(ron::to_string(&file).unwrap().matches("rope_m").count(), 0);
}

/// `river-craft` design 10.4: the tube carries no anchor.
#[test]
fn the_tube_has_no_anchor() {
    assert!(!at_pole(Kind::Tube, 0.0, 0.0).has_anchor());
    assert!(at_pole(Kind::Raft, 0.0, 0.0).has_anchor());
    assert!(at_pole(Kind::Loon, 0.0, 0.0).has_anchor());
}
