//! Reproducible handling measurements; no alternate physics implementation.
//! See openspec/changes/vehicles-improve/measurement-protocol.md.
use glam::{DQuat, DVec3, Vec3};
use pbd_core::sea::{SeaSettings, SeaTable};
use pbd_core::vehicle::spec::VehicleSpecs;
use pbd_core::vehicle::*;
use pbd_core::wind::{AirHere, GustSettings};
use std::sync::Arc;

const R: f64 = 4799.5;
const DT: f64 = 1.0 / 60.0;

struct World {
    sea: SeaTable,
    gusts: GustSettings,
    wind: Vec3,
    current: DVec3,
    gravity: f64,
    seconds: f64,
}

impl World {
    fn new() -> Self {
        Self {
            sea: SeaTable::new(
                SeaSettings {
                    swell_height_m: 0.0,
                    ..Default::default()
                },
                25.0,
            ),
            gusts: GustSettings {
                gustiness: 0.0,
                ..Default::default()
            },
            wind: Vec3::ZERO,
            current: DVec3::ZERO,
            gravity: 25.0,
            seconds: 1000.0,
        }
    }

    fn run(&mut self, craft: &mut Craft, seconds: f64, mut input: impl FnMut(&Craft) -> Input) {
        for _ in 0..(seconds / DT).round() as usize {
            let env = Surroundings {
                sea: &self.sea,
                sea_state: self.sea.state(0.0, self.wind),
                sea_radius: R,
                depth: 40.0,
                air: AirHere {
                    wind: self.wind,
                    upper: self.wind,
                    rain_mmh: 0.0,
                    over_land: false,
                },
                gusts: &self.gusts,
                gravity: -craft.body.position.normalize() * self.gravity,
                current: self.current,
                ground: &|_| R - 40.0,
                seconds: self.seconds,
            };
            craft.step(DT, 4, &input(craft), &env);
            self.seconds += DT;
            if !craft.is_finite() {
                break;
            }
        }
    }
}

fn craft(kind: Kind, height: f64, heading: f64) -> Craft {
    let specs = Arc::new(VehicleSpecs::default());
    let hulls = Hulls::new(&specs);
    let mut c = Craft::new(
        kind,
        1,
        specs,
        hulls,
        DVec3::Y * (R + height),
        DQuat::from_rotation_y(heading),
    );
    c.board();
    c
}

fn heading(c: &Craft) -> f64 {
    let bow = c.body.axis(FORWARD);
    (-bow.x).atan2(-bow.z)
}

fn main() {
    println!("defaults; R=4799.5m; g=25m/s^2; 60Hz x4; flat sea; no gusts; clock=1000s");
    let mut w = World::new();
    w.wind = Vec3::X * 11.0;
    let course = 30_f64.to_radians();
    let mut c = craft(Kind::Tern, 0.2, course);
    if let CraftState::Tern(s) = &mut c.state {
        s.sheet = 0.3;
    }
    w.run(&mut c, 60.0, |c| {
        let error = (course - heading(c) + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        Input {
            steer: (error * 3.0).clamp(-1.0, 1.0) as f32,
            ..Default::default()
        }
    });
    if let Telemetry::Tern(t) = &c.telemetry {
        println!(
            "Tern 60deg close reach 11m/s wind 60s sheet=.3 helm: speed={:.4}m/s VMG={:.4}m/s heel={:.4}deg",
            t.speed,
            t.upwind,
            t.heel.to_degrees()
        );
    }
    let mut w = World::new();
    w.current = DVec3::X * 2.0;
    let mut c = craft(Kind::Tern, 0.2, 0.0);
    c.body.velocity = w.current;
    w.run(&mut c, DT, |_| Input::default());
    if let Telemetry::Tern(t) = &c.telemetry {
        println!(
            "Tern comoving 2m/s cross-current one tick: leeway={:.4}deg",
            t.leeway.to_degrees()
        );
    }

    let mut w = World::new();
    let mut c = craft(Kind::Loon, 0.1, 0.0);
    w.run(&mut c, 20.0, |_| Input {
        forward: 1.0,
        ..Default::default()
    });
    if let Telemetry::Loon(t) = &c.telemetry {
        println!(
            "Loon alternating 20s: speed={:.4}m/s cadence={:.1}/min",
            t.speed, t.strokes_per_minute
        );
    }
    for rudder in [-1.0, 1.0] {
        let mut w = World::new();
        let mut c = craft(Kind::Loon, 0.1, 0.0);
        w.run(&mut c, 5.0, |_| Input::default());
        c.body.velocity = FORWARD * 2.0;
        w.run(&mut c, 1.0, |_| Input {
            rudder,
            ..Default::default()
        });
        println!(
            "Loon stern rudder={rudder:+} 1s from2m/s: heading={:.4}deg",
            heading(&c).to_degrees()
        );
    }
    // A symmetric horizontal foil, a tailplane's size (the retired Kestrel's
    // tail's numbers).
    let spec = foil::FoilSpec {
        at: [0.0, 0.95, 4.9],
        chord: [0.0, 0.0, -1.0],
        normal: [0.0, 1.0, 0.0],
        area_m2: 3.4,
        aspect: 4.5,
        cl_max: 1.2,
        cd0: 0.012,
    };
    for z in [-20.0, 20.0] {
        let mut b = body::RigidBody::new(1.0, DVec3::ONE);
        b.velocity = DVec3::new(0.0, -2.0, z);
        let f = foil::apply(
            &mut b,
            &spec,
            DVec3::ZERO,
            DVec3::ZERO,
            1.225,
            0.0,
            1.0,
            1.0,
        );
        println!("foil velocity=(0,-2,{z}) force_y={:.4}N", f.force.y);
    }
}
