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
    let mut specs = VehicleSpecs::default();
    // Optional measurement-only calibration of cyclic/yaw ratings; runtime
    // defaults and the shipped config are not modified by this instrument.
    let scale = torque_scale();
    specs.kestrel.rotor.pitch_torque *= scale;
    specs.kestrel.rotor.yaw_torque *= scale;
    let specs = Arc::new(specs);
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

fn torque_scale() -> f32 {
    let scale = std::env::args().nth(1).map_or(1.0, |v| {
        v.parse::<f32>()
            .expect("optional positive rotor torque scale")
    });
    assert!(scale.is_finite() && scale > 0.0);
    scale
}

fn heading(c: &Craft) -> f64 {
    let bow = c.body.axis(FORWARD);
    (-bow.x).atan2(-bow.z)
}

fn main() {
    println!("measurement rotor torque scale={}", torque_scale());
    println!("defaults; R=4799.5m; g=25m/s^2; 60Hz x4; flat sea; no gusts; clock=1000s");
    let mut w = World::new();
    let mut c = craft(Kind::Kestrel, 100.0, 0.0);
    w.run(&mut c, 3.0, |_| Input {
        collective: 1.0,
        ..Default::default()
    });
    if let Telemetry::Kestrel(t) = &c.telemetry {
        println!(
            "Kestrel climb 3s: vertical={:.4}m/s tilt={:.4}deg",
            t.vertical_speed,
            c.body
                .axis(UP)
                .angle_between(c.body.position.normalize())
                .to_degrees()
        );
    }
    let mut w = World::new();
    w.wind = Vec3::X * 10.0;
    let mut c = craft(Kind::Kestrel, 100.0, 0.0);
    w.run(&mut c, 20.0, |_| Input::default());
    let up = c.body.position.normalize();
    println!(
        "Kestrel 10m/s crosswind 20s: drift={:.4}m/s height={:.4}m",
        (c.body.velocity - up * c.body.velocity.dot(up)).length(),
        c.body.position.length() - R
    );

    let mut w = World::new();
    let mut c = craft(Kind::Kestrel, 100.0, 0.0);
    w.run(&mut c, 5.0, |_| Input::default());
    let mut max_pitch = 0.0_f64;
    let mut min_height = 100.0_f64;
    w.run(&mut c, 6.1, |c| {
        max_pitch = max_pitch.max(
            c.body
                .axis(FORWARD)
                .dot(c.body.position.normalize())
                .asin()
                .abs()
                .to_degrees(),
        );
        min_height = min_height.min(c.body.position.length() - R);
        Input {
            tilt: -1.0,
            ..Default::default()
        }
    });
    if let Telemetry::Kestrel(t) = &c.telemetry {
        println!(
            "Kestrel conversion tilt=-1 6.1s after 5s hover: air={:.4}m/s climb={:.4}m/s wing={:.4} max_pitch={max_pitch:.4}deg min_height={min_height:.4}m",
            t.airspeed, t.vertical_speed, t.wing_share
        );
    }
    let mut w = World::new();
    w.gravity = 0.0;
    let mut c = craft(Kind::Kestrel, 100.0, 0.0);
    w.run(&mut c, DT, |_| Input::default());
    println!("Kestrel zero gravity one tick: finite={}", c.is_finite());
    let mut w = World::new();
    w.gravity = 0.0;
    let mut c = craft(Kind::Kestrel, 100.0, 0.0);
    if let CraftState::Kestrel(s) = &mut c.state {
        s.assist = false;
    }
    w.run(&mut c, DT, |_| Input {
        pitch: 1.0,
        yaw: 1.0,
        ..Default::default()
    });
    println!(
        "Kestrel unpowered pitch+yaw one tick: angular_speed={:.6}rad/s",
        c.body.angular_velocity.length()
    );

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
    let spec = VehicleSpecs::default().kestrel.tail;
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
