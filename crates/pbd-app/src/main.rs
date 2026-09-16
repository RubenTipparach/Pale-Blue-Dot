use avian3d::prelude::{AngularVelocity, LinearVelocity, Position};
use bevy::prelude::*;
use pbd_app::{
    CelestialScene, FlightTelemetry, PhysicsFrame, ShipController, SimulationClock, headless_app,
    spawn_ship,
};
use pbd_core::{
    DVec3,
    flight::FlightInput,
    frame::LocalFrame,
    hex::{Hex, Voxel},
    terrain::TerrainGenerator,
};

fn main() {
    let steps = std::env::args()
        .nth(1)
        .map(|value| {
            value
                .parse::<u32>()
                .expect("usage: cargo run -p pbd-app -- [steps]")
        })
        .unwrap_or(600);
    assert!(
        (1..=360_000).contains(&steps),
        "steps must be between 1 and 360000"
    );
    let mut app = headless_app();
    let scene = CelestialScene::demo();
    let planet = scene.states[0];
    app.insert_resource(PhysicsFrame(LocalFrame {
        origin: planet.position,
        velocity: planet.velocity,
    }));
    app.insert_resource(scene);
    let ship = spawn_ship(
        app.world_mut(),
        Vec3::new(0.0, 4_050.0, 0.0),
        ShipController {
            input: FlightInput {
                thrust: DVec3::new(0.6, 1.0, 0.2),
                rotation: DVec3::Y * 0.1,
                ..Default::default()
            },
            ..Default::default()
        },
    );
    app.finish();
    app.cleanup();
    app.update();
    for _ in 0..steps {
        app.update();
    }
    let state = app.world().entity(ship);
    let clock = app.world().resource::<SimulationClock>();
    let telemetry = app.world().resource::<FlightTelemetry>();
    let generator = TerrainGenerator {
        seed: 42,
        sea_level: 0,
    };
    let checksum = (-32..32)
        .flat_map(|q| {
            (-32..32).map(move |r| Voxel {
                hex: Hex { q, r },
                layer: 5,
            })
        })
        .fold(0_u64, |hash, voxel| {
            hash.wrapping_mul(31)
                .wrapping_add(generator.sample(voxel) as u64)
        });
    println!("Pale Blue Dot: headless Bevy 0.18.1 / Avian 0.6.1 foundation");
    println!(
        "ticks={} simulated_seconds={:.3} rails_bodies={}",
        clock.ticks,
        clock.seconds,
        app.world().resource::<CelestialScene>().states.len()
    );
    println!(
        "ship_local_m={:?} speed_m_s={:.3} angular_rad_s={:.3}",
        state.get::<Position>().unwrap().0,
        state.get::<LinearVelocity>().unwrap().0.length(),
        state.get::<AngularVelocity>().unwrap().0.length()
    );
    println!(
        "peak_command_acceleration_m_s2={:.3} safety_clips={} terrain_checksum={checksum:016x}",
        telemetry.peak_command_acceleration, telemetry.safety_clips
    );
    println!(
        "Renderer, GPU meshing dispatch, spherical topology and gameplay are not connected in this foundation."
    );
}
