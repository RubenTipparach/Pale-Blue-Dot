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
    /// The tide: the moon's direction and the range (`sea::tide_m`).
    tide: glam::Vec4,
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
            tide: glam::Vec4::ZERO,
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
                tide: self.tide,
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
            tide: glam::Vec4::ZERO,
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
        tide: glam::Vec4::ZERO,
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
    let mut craft = at_pole(Kind::Loon, -1.0, 0.0);
    craft.body.velocity = DVec3::new(0.4, 0.0, -2.0);
    craft.body.angular_velocity = DVec3::Z * 0.2;
    let ground = |_: DVec3| RADIUS - 40.0;
    let env = Surroundings {
        sea: &world.sea,
        sea_state: world.sea.state(12.0, Vec3::X * 12.0),
        tide: glam::Vec4::ZERO,
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
    for kind in [Kind::Loon, Kind::Rowboat, Kind::Cog] {
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
    let mut craft = at_pole(Kind::Rowboat, 0.2, 0.0);
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
    assert_eq!(at_pole(Kind::Loon, 0.2, 0.0).record().yard, None);
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
    for kind in [Kind::Loon, Kind::Rowboat] {
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
    let (kind, wood) = match craft.kind {
        Kind::Rowboat => ("rowboat", "boards"),
        _ => ("canoe", "driftwood"),
    };
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
    for kind in [Kind::Loon, Kind::Rowboat] {
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
fn a_rowboat_floats_upright_empty_and_with_its_rower_and_rows_ahead() {
    // Across the sea's face: its swell lifts and drops a boat riding it.
    let drift = |craft: &Craft| {
        let up = craft.body.position.normalize();
        let v = craft.body.velocity;
        (v - up * v.dot(up)).length()
    };
    for occupied in [false, true] {
        let mut craft = at_pole(Kind::Rowboat, 0.0, 0.0);
        craft.occupied = occupied;
        let mut world = World::new(RADIUS - 40.0);
        world.run(&mut craft, 30.0, |_| Input::default());
        let up = craft.body.position.normalize();
        let upright = craft.body.axis(DVec3::Y).dot(up);
        let keel = over_the_sea(&world, craft.reference_point([0.0, -0.25, 0.0]));
        let gunwale = over_the_sea(&world, craft.reference_point([0.0, 0.35, 0.0]));
        println!(
            "rowboat, occupied {occupied}: upright {upright:.4}, keel {keel:+.3} m, gunwale {gunwale:+.3} m, drift {:.3}",
            drift(&craft)
        );
        assert!(upright > 3f64.to_radians().cos(), "heeled: {upright}");
        assert!(
            keel < 0.0 && gunwale > 0.2,
            "keel {keel}, gunwale {gunwale}"
        );
        assert!(drift(&craft) < 0.05);
    }
    let mut craft = at_pole(Kind::Rowboat, 0.0, 0.0);
    craft.occupied = true;
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
    println!("rowboat rowed 30 s: {made:.1} m ahead, {speed:.2} m/s");
    assert!(made > 20.0 && speed > 0.8, "{made} m, {speed} m/s");
}

#[test]
#[ignore = "instrument: an empty boat settling on a still sea"]
fn print_an_empty_boat_settling() {
    for kind in [Kind::Loon, Kind::Rowboat] {
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
    for kind in [Kind::Loon, Kind::Rowboat] {
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
    for kind in [Kind::Tube, Kind::Raft, Kind::Rowboat] {
        let code = kind.good_code().expect("a made kind");
        assert_eq!(Kind::of_good_code(code), Some(kind));
        assert!(
            crate::goods::Good::by_code(code).is_some(),
            "{code} is a good"
        );
    }
    for kind in [Kind::Loon, Kind::Cog] {
        assert_eq!(kind.good_code(), None);
    }
    assert!(Kind::Tube.paddled() && Kind::Raft.paddled() && !Kind::Cog.paddled());
}

/// At high water every boat floats on the tide (`small-boats`): with the moon
/// overhead and a 1.2 m range the sea stands 0.8 m over its mean, and each
/// open or decked boat rides up with it, its gunwale clear and its bilge dry.
/// (Until the sea's reach counted the tide, every cell more than the waves'
/// height over the mean was taken as dry, and boats sat on the mean and
/// swamped under the tide.)
#[test]
fn every_boat_floats_on_the_tide() {
    for kind in [Kind::Loon, Kind::Rowboat, Kind::Tube, Kind::Raft] {
        // Put on the water where it stands, as a launch or a new world's
        // placement puts it.
        let high = f64::from(crate::sea::tide_m(
            glam::Vec3::Y,
            glam::Vec4::new(0.0, 1.0, 0.0, 1.2),
        ));
        let mut craft = at_pole(kind, high, 0.0);
        craft.occupied = true;
        craft.weigh();
        let mut world = World::new(RADIUS - 40.0);
        world.tide = glam::Vec4::new(0.0, 1.0, 0.0, 1.2);
        world.run(&mut craft, 30.0, |_| Input::default());
        let freeboard = match &craft.telemetry {
            crate::vehicle::Telemetry::Loon(t) => t.freeboard,
            _ => unreachable!(),
        };
        let lift = craft.reference_position().length() - RADIUS;
        println!(
            "{} at high water: lifted {lift:+.3} m, freeboard {freeboard:+.3} m, bilge {:.1} kg",
            kind.name(),
            craft.bilge_kg
        );
        assert!(lift > 0.4, "{kind:?} did not rise with the tide: {lift}");
        assert!(freeboard > 0.05, "{kind:?} under the tide: {freeboard}");
        assert!(
            craft.bilge_kg < 5.0,
            "{kind:?} took water: {}",
            craft.bilge_kg
        );
    }
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
