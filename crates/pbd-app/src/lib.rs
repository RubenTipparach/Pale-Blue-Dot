//! Bevy + Avian simulation shared by the desktop explorer and headless checks.
//! `PaleBlueDotPlugin` submits accelerations; Avian alone integrates ship pose.

#[cfg(feature = "desktop")]
pub mod flight_view;
#[cfg(feature = "desktop")]
pub mod planet;
#[cfg(feature = "desktop")]
pub mod sky;
#[cfg(feature = "desktop")]
pub mod walking;

use std::time::Duration;

use avian3d::prelude::*;
use bevy::{prelude::*, time::TimeUpdateStrategy};
use pbd_core::{
    DQuat, DVec3,
    flight::{
        FlightAcceleration, FlightInput, FlightLimits, FlightMotion, GravityWell,
        control_acceleration,
    },
    frame::{KinematicState, LocalFrame},
    orbit::{CircularOrbit, Ephemeris, RailsBody, RailsBodyKind},
};

pub const FIXED_HZ: f64 = 60.0;
/// Absolute impact/emergency envelope; mode speed targets live in FlightLimits.
pub const SAFETY_SPEED: f32 = 600.0;
pub const SAFETY_ANGULAR_SPEED: f32 = 1.5;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimulationClock {
    pub ticks: u64,
    pub seconds: f64,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct PhysicsFrame(pub LocalFrame);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct ShipController {
    pub limits: FlightLimits,
    pub input: FlightInput,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct GlobalShipState(pub KinematicState);

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct LastFlightCommand(pub FlightAcceleration);

#[derive(Resource, Clone, Debug)]
pub struct CelestialScene {
    pub ephemeris: Ephemeris,
    pub states: Vec<KinematicState>,
    /// (body index, surface radius in metres, surface acceleration in m/s²).
    pub gravity: Vec<(usize, f64, f64)>,
}

impl CelestialScene {
    pub fn planet_at_origin(radius: f64, surface_gravity: f64) -> Self {
        let ephemeris = Ephemeris::new(vec![RailsBody {
            kind: RailsBodyKind::Planet,
            parent: None,
            orbit: CircularOrbit::stationary(),
        }])
        .unwrap();
        let states = ephemeris.sample(0.0);
        Self {
            ephemeris,
            states,
            gravity: vec![(0, radius, surface_gravity)],
        }
    }

    pub fn vacuum() -> Self {
        Self {
            ephemeris: Ephemeris::new(vec![]).unwrap(),
            states: vec![],
            gravity: vec![],
        }
    }

    pub fn demo() -> Self {
        let orbit =
            |radius, period| CircularOrbit::new(radius, period, 0.0, DQuat::IDENTITY).unwrap();
        let ephemeris = Ephemeris::new(vec![
            RailsBody {
                kind: RailsBodyKind::Planet,
                parent: None,
                orbit: orbit(1e9, 1e7),
            },
            RailsBody {
                kind: RailsBodyKind::Moon,
                parent: Some(0),
                orbit: orbit(20_000.0, 60_000.0),
            },
            RailsBody {
                kind: RailsBodyKind::Station,
                parent: Some(0),
                orbit: orbit(6_500.0, 4_000.0),
            },
        ])
        .unwrap();
        let states = ephemeris.sample(0.0);
        Self {
            ephemeris,
            states,
            gravity: vec![(0, 4_000.0, 9.0), (1, 500.0, 1.0)],
        }
    }

    fn acceleration_at(&self, global_position: DVec3) -> DVec3 {
        self.gravity
            .iter()
            .map(|&(index, radius, surface_acceleration)| {
                GravityWell {
                    center: self.states[index].position,
                    radius,
                    surface_acceleration,
                }
                .acceleration_at(global_position)
            })
            .sum()
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct FlightTelemetry {
    pub peak_command_acceleration: f64,
    pub peak_command_angular_acceleration: f64,
    pub peak_speed: f32,
    pub peak_angular_speed: f32,
    /// Post-solver clips are safety impulses, NOT bounded controller acceleration.
    pub safety_clips: u64,
}

pub struct PaleBlueDotPlugin;

impl Plugin for PaleBlueDotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimulationClock>()
            .init_resource::<PhysicsFrame>()
            .init_resource::<FlightTelemetry>()
            .insert_resource(CelestialScene::vacuum())
            .insert_resource(Gravity(Vec3::ZERO))
            // Avian has advanced its own clock before this schedule runs. Forces,
            // ephemerides and frame motion therefore share pause/time-scale rules.
            .add_systems(
                PhysicsSchedule,
                apply_ship_controls
                    .after(PhysicsStepSystems::First)
                    .before(PhysicsStepSystems::BroadPhase),
            )
            .add_systems(
                PhysicsSchedule,
                (enforce_safety_envelope, publish_simulation_state)
                    .chain()
                    .in_set(PhysicsStepSystems::Last),
            );
    }
}

fn apply_ship_controls(
    time: Res<Time<Physics>>,
    frame: Res<PhysicsFrame>,
    scene: Res<CelestialScene>,
    mut telemetry: ResMut<FlightTelemetry>,
    mut ships: Query<(&ShipController, &mut LastFlightCommand, Forces)>,
) {
    for (controller, mut last_command, mut forces) in &mut ships {
        let global_position = frame.0.origin + forces.position().0.as_dvec3();
        let motion = FlightMotion {
            orientation: forces.rotation().0.as_dquat(),
            velocity: forces.linear_velocity().as_dvec3(),
            angular_velocity: forces.angular_velocity().as_dvec3(),
        };
        let command = control_acceleration(
            motion,
            controller.input,
            scene.acceleration_at(global_position),
            controller.limits,
            time.delta_secs_f64(),
        );
        telemetry.peak_command_acceleration = telemetry
            .peak_command_acceleration
            .max(command.linear.length());
        telemetry.peak_command_angular_acceleration = telemetry
            .peak_command_angular_acceleration
            .max(command.angular.length());
        forces.apply_linear_acceleration(command.linear.as_vec3());
        forces.apply_angular_acceleration(command.angular.as_vec3());
        last_command.0 = command;
    }
}

fn enforce_safety_envelope(
    mut telemetry: ResMut<FlightTelemetry>,
    mut ships: Query<
        (
            &mut LinearVelocity,
            &mut AngularVelocity,
            &MaxLinearSpeed,
            &MaxAngularSpeed,
        ),
        With<ShipController>,
    >,
) {
    for (mut linear, mut angular, linear_limit, angular_limit) in &mut ships {
        let safe_linear = linear.0.clamp_length_max(linear_limit.0);
        let safe_angular = angular.0.clamp_length_max(angular_limit.0);
        if safe_linear != linear.0 || safe_angular != angular.0 {
            telemetry.safety_clips += 1;
        }
        // Avian already caps during integration; this closes the post-contact gap.
        linear.0 = safe_linear;
        angular.0 = safe_angular;
        telemetry.peak_speed = telemetry.peak_speed.max(linear.0.length());
        telemetry.peak_angular_speed = telemetry.peak_angular_speed.max(angular.0.length());
    }
}

fn publish_simulation_state(
    time: Res<Time<Physics>>,
    mut clock: ResMut<SimulationClock>,
    mut frame: ResMut<PhysicsFrame>,
    mut scene: ResMut<CelestialScene>,
    mut ships: Query<(&Position, &LinearVelocity, &mut GlobalShipState)>,
) {
    clock.ticks += 1;
    // Time<Physics> accumulates Duration, including changes of timestep or speed.
    clock.seconds = time.elapsed_secs_f64();
    frame.0 = frame.0.advanced(time.delta_secs_f64());
    let scene = scene.as_mut();
    scene
        .ephemeris
        .sample_into(clock.seconds, &mut scene.states);
    for (position, velocity, mut global) in &mut ships {
        global.0 = frame.0.to_global(KinematicState {
            position: position.0.as_dvec3(),
            velocity: velocity.0.as_dvec3(),
        });
    }
}

/// Returns a finite-step, manually clocked app; no window, renderer, or GPU needed.
pub fn headless_app() -> App {
    let mut app = App::new();
    let step = Duration::from_secs_f64(1.0 / FIXED_HZ);
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        PhysicsPlugins::default(),
        PaleBlueDotPlugin,
    ))
    .insert_resource(Time::<Fixed>::from_duration(step))
    .insert_resource(TimeUpdateStrategy::ManualDuration(step))
    .insert_resource(SubstepCount(4));
    app
}

pub fn spawn_ship(world: &mut World, position: Vec3, controller: ShipController) -> Entity {
    assert!(controller.limits.validate().is_ok());
    assert!(controller.limits.speed <= SAFETY_SPEED as f64);
    assert!(controller.limits.angular_speed <= SAFETY_ANGULAR_SPEED as f64);
    let frame = world.resource::<PhysicsFrame>().0;
    world
        .spawn((
            Name::new("Survey skiff"),
            RigidBody::Dynamic,
            Collider::sphere(0.75),
            Mass(1_000.0),
            AngularInertia::new(Vec3::splat(800.0)),
            Position(position),
            Rotation(Quat::IDENTITY),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
            MaxLinearSpeed(SAFETY_SPEED),
            MaxAngularSpeed(SAFETY_ANGULAR_SPEED),
            SleepingDisabled,
            controller,
            LastFlightCommand::default(),
            GlobalShipState(frame.to_global(KinematicState {
                position: position.as_dvec3(),
                velocity: DVec3::ZERO,
            })),
        ))
        .id()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn initialized_app() -> App {
        let mut app = headless_app();
        app.finish();
        app.cleanup();
        app.update(); // Bevy's first time update establishes the epoch (delta=0).
        app
    }

    #[test]
    fn avian_integrates_constant_acceleration_exactly_once() {
        let mut app = initialized_app();
        let controller = ShipController {
            input: FlightInput {
                thrust: DVec3::X,
                inertial_dampeners: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let ship = spawn_ship(app.world_mut(), Vec3::ZERO, controller);
        for _ in 0..60 {
            app.update();
        }
        let body = app.world().entity(ship);
        let velocity = body.get::<LinearVelocity>().unwrap().0;
        let position = body.get::<Position>().unwrap().0;
        assert!((velocity.x - 20.0).abs() < 0.002, "velocity={velocity:?}");
        assert!((position.x - 10.0).abs() < 0.1, "position={position:?}");
        assert!(position.y.abs() < 1e-5);
        assert_eq!(app.world().resource::<SimulationClock>().ticks, 60);
    }

    #[test]
    fn time_scaling_stopping_and_timestep_changes_keep_all_clocks_together() {
        let mut app = initialized_app();
        let controller = ShipController {
            input: FlightInput {
                thrust: DVec3::X,
                inertial_dampeners: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let ship = spawn_ship(app.world_mut(), Vec3::ZERO, controller);
        app.world_mut()
            .resource_mut::<Time<Physics>>()
            .set_relative_speed(0.5);
        for _ in 0..60 {
            app.update();
        }
        assert!((app.world().resource::<SimulationClock>().seconds - 0.5).abs() < 1e-6);
        assert!(
            (app.world()
                .entity(ship)
                .get::<LinearVelocity>()
                .unwrap()
                .0
                .x
                - 10.0)
                .abs()
                < 0.002
        );
        let stopped_position = app.world().entity(ship).get::<Position>().unwrap().0;
        app.world_mut()
            .resource_mut::<Time<Physics>>()
            .set_relative_speed(0.0);
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(
            app.world().entity(ship).get::<Position>().unwrap().0,
            stopped_position
        );
        assert_eq!(app.world().resource::<SimulationClock>().ticks, 60);
        app.world_mut()
            .resource_mut::<Time<Physics>>()
            .set_relative_speed(1.0);
        let faster_step = Duration::from_secs_f64(1.0 / 120.0);
        app.insert_resource(Time::<Fixed>::from_duration(faster_step));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(faster_step));
        for _ in 0..60 {
            app.update();
        }
        assert!((app.world().resource::<SimulationClock>().seconds - 1.0).abs() < 1e-6);
        assert!(
            (app.world()
                .entity(ship)
                .get::<LinearVelocity>()
                .unwrap()
                .0
                .x
                - 20.0)
                .abs()
                < 0.003
        );
        assert_eq!(app.world().resource::<SimulationClock>().ticks, 120);
    }

    #[test]
    fn local_collision_stops_a_ship_instead_of_flying_through_a_wall() {
        let mut app = initialized_app();
        app.world_mut().spawn((
            RigidBody::Static,
            Collider::cuboid(1.0, 20.0, 20.0),
            Position(Vec3::X * 6.0),
        ));
        let controller = ShipController {
            input: FlightInput {
                inertial_dampeners: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let ship = spawn_ship(app.world_mut(), Vec3::ZERO, controller);
        app.world_mut()
            .entity_mut(ship)
            .insert(LinearVelocity(Vec3::X * 10.0));
        for _ in 0..120 {
            app.update();
        }
        let body = app.world().entity(ship);
        let position = body.get::<Position>().unwrap().0;
        assert!(
            position.x > 3.0 && position.x < 5.0,
            "position={position:?}"
        );
        assert!(body.get::<LinearVelocity>().unwrap().0.x < 0.2);
    }

    #[test]
    fn quaternion_rotation_and_safety_limits_remain_valid_under_sustained_input() {
        let mut app = initialized_app();
        let controller = ShipController {
            input: FlightInput {
                thrust: DVec3::ONE,
                rotation: DVec3::Y,
                inertial_dampeners: false,
                rotational_dampeners: false,
            },
            ..Default::default()
        };
        let ship = spawn_ship(app.world_mut(), Vec3::ZERO, controller);
        for _ in 0..600 {
            app.update();
        }
        let body = app.world().entity(ship);
        let rotation = body.get::<Rotation>().unwrap().0;
        assert!((rotation.length() - 1.0).abs() < 1e-4);
        assert!(rotation.angle_between(Quat::IDENTITY) > 0.1);
        assert!(body.get::<LinearVelocity>().unwrap().0.length() <= 120.001);
        assert!(body.get::<AngularVelocity>().unwrap().0.length() <= 1.5001);
        let telemetry = app.world().resource::<FlightTelemetry>();
        assert!(telemetry.peak_command_acceleration <= 20.0 + 1e-8);
        assert!(telemetry.peak_command_angular_acceleration <= 3.0 + 1e-8);
    }
}
