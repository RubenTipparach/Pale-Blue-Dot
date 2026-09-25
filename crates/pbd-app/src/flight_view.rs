//! Interactive and scripted piloting through the existing bounded Avian controller.
//! The radial terrain guard is prototype flight protection, not voxel collision.

mod input;
pub mod route;
mod tour;

pub use route::{RouteKind, RouteState};

use avian3d::prelude::*;
use bevy::{
    app::{RunFixedMainLoop, RunFixedMainLoopSystems},
    prelude::*,
    transform::TransformSystems,
};
use pbd_core::{DVec3, flight::FlightLimits};

use crate::{CelestialScene, LastFlightCommand, PhysicsFrame, ShipController, planet, spawn_ship};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlyMode {
    #[default]
    Manual,
    Tour,
    /// The far-side route: take off, climb, cruise round, land on the far side.
    Route,
}

/// Both the windowed demo and the headless tour use this configuration.
#[derive(Resource, Clone, Copy, Debug)]
pub struct FlightViewConfig {
    pub mode: FlyMode,
    pub spawn_direction: Vec3,
    /// Height above terrain for manual flight, in metres.
    pub spawn_altitude: f32,
    /// Tour altitude above the sea-level reference sphere, in metres.
    pub tour_altitude: f32,
    pub surface_speed: f32,
    pub cruise_speed: f32,
    pub acceleration: f32,
    pub minimum_clearance: f32,
    /// Camera/ship pitch below the local tangent at spawn, in radians.
    pub view_pitch_down: f32,
    pub startup_camera: bool,
    // ---- The far-side route (`route.rs`, `far-side-flight`).
    /// Cruise height above the ground, metres: above the atmosphere (960 m
    /// above sea level), high enough that the planet's disc fills the view.
    pub route_cruise_height_m: f32,
    /// The climb's and the glide's angle above the local horizon, degrees.
    pub route_climb_angle_deg: f32,
    /// Speed along the path at cruise, m/s; at most the ship's safety speed.
    pub route_speed: f32,
    /// Speed along the path on the climb and the descent, m/s: slow enough
    /// to round the corners into and out of the cruise.
    pub route_climb_speed: f32,
    /// The radius the height line's corners are rounded to, metres.
    pub route_corner_m: f32,
    /// How fast ground speed builds from the start and bleeds off before the
    /// destination, m/s^2.
    pub route_ground_accel: f32,
    /// Vertical speed at touchdown, m/s.
    pub route_touchdown_mps: f32,
    /// The camera's slow sway to each side, degrees, and its bank into it.
    pub route_sway_deg: f32,
    pub route_bank_deg: f32,
    /// The camera's ease time constant toward its target, seconds.
    pub route_camera_ease_s: f32,
    /// The fastest the route camera may turn, radians a second.
    pub route_camera_max_rate: f32,
    /// Reduced camera motion: the sway and bank at a third.
    pub route_reduced_motion: bool,
    /// Which route `FlyMode::Route` flies.
    pub route_kind: RouteKind,
    /// The scenic route: its height above the ground ahead, metres; its speed
    /// along the path, m/s; its climb and glide angle, degrees.
    pub route_scenic_height_m: f32,
    pub route_scenic_speed: f32,
    pub route_scenic_climb_deg: f32,
}

impl Default for FlightViewConfig {
    fn default() -> Self {
        Self {
            mode: FlyMode::Manual,
            spawn_direction: Vec3::new(0.8776, 0.4794, 0.0).normalize(),
            spawn_altitude: 180.0,
            tour_altitude: 1_000.0,
            surface_speed: 120.0,
            cruise_speed: 600.0,
            acceleration: 80.0,
            minimum_clearance: 45.0,
            view_pitch_down: 0.31,
            startup_camera: true,
            route_cruise_height_m: 3_000.0,
            route_climb_angle_deg: 60.0,
            route_speed: 550.0,
            route_climb_speed: 260.0,
            route_corner_m: 900.0,
            route_ground_accel: 25.0,
            route_touchdown_mps: 2.0,
            route_sway_deg: 10.0,
            route_bank_deg: 7.0,
            route_camera_ease_s: 0.9,
            route_camera_max_rate: 0.6,
            route_reduced_motion: false,
            route_kind: RouteKind::FarSide,
            route_scenic_height_m: 90.0,
            route_scenic_speed: 150.0,
            route_scenic_climb_deg: 45.0,
        }
    }
}

#[derive(Component)]
pub struct PilotShip;

#[derive(Component)]
pub struct FlightCamera;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct FlightReadout {
    pub position: Vec3,
    pub speed: f32,
    pub altitude: f32,
    pub clearance: f32,
    pub latitude_deg: f32,
    pub longitude_deg: f32,
    pub dampeners: bool,
    pub cruise: bool,
    pub protection_events: u64,
    /// No body's anchor field reaches the ship's current position.
    pub is_in_space: bool,
}

/// Completed means the physical ship has travelled an actual full great circle.
/// Time and angle are never substituted for an integrated position.
#[derive(Resource, Clone, Copy, Debug)]
pub struct TourProgress {
    pub completed: bool,
    pub angular_distance_rad: f64,
    pub elapsed_seconds: f64,
    pub minimum_clearance: f32,
    pub maximum_speed: f32,
    pub maximum_acceleration: f64,
    pub maximum_angular_speed: f32,
    pub maximum_angular_acceleration: f64,
    pub protection_events: u64,
    normal: Vec3,
    previous_direction: Vec3,
}

impl Default for TourProgress {
    fn default() -> Self {
        Self {
            completed: false,
            angular_distance_rad: 0.0,
            elapsed_seconds: 0.0,
            minimum_clearance: f32::INFINITY,
            maximum_speed: 0.0,
            maximum_acceleration: 0.0,
            maximum_angular_speed: 0.0,
            maximum_angular_acceleration: 0.0,
            protection_events: 0,
            normal: Vec3::Z,
            previous_direction: Vec3::Y,
        }
    }
}

/// Raw mouse displacement is measured in pixels, independent of frame duration.
pub const MOUSE_LOOK_SENSITIVITY: f32 = 0.002;

/// Current-frame view and movement intent, separate from bounded ship attitude.
/// A walking controller can disable flight input without despawning the ship.
#[derive(Resource, Clone, Copy, Debug)]
pub struct FlightInputState {
    enabled: bool,
    axes: Vec3,
    target_rotation: Quat,
    dampeners: bool,
    cruise: bool,
    brake: bool,
    captured: bool,
    reset: bool,
}

impl Default for FlightInputState {
    fn default() -> Self {
        Self {
            enabled: true,
            axes: Vec3::ZERO,
            target_rotation: Quat::IDENTITY,
            dampeners: true,
            cruise: false,
            brake: false,
            captured: false,
            reset: false,
        }
    }
}

impl FlightInputState {
    /// Release controls and brake when switching away from the persistent ship.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.axes = Vec3::ZERO;
            self.cruise = false;
            self.brake = true;
            self.captured = false;
            self.reset = false;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Request cursor capture when flight is the active control mode.
    pub fn set_captured(&mut self, captured: bool) {
        self.captured = captured && self.enabled;
    }

    pub fn is_captured(&self) -> bool {
        self.captured
    }

    /// Set the view immediately when boarding or changing camera ownership.
    pub fn sync_view_rotation(&mut self, rotation: Quat) {
        if rotation.is_finite() && rotation.length_squared() > f32::EPSILON {
            self.target_rotation = rotation.normalize();
        }
    }

    pub fn view_rotation(&self) -> Quat {
        self.target_rotation
    }
}

#[derive(Component)]
struct ProtectedPosition(Vec3);

pub struct FlightViewPlugin;

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlightViewStartup;

/// Mode switching must run before this set in `RunFixedMainLoop`.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlightViewInput;

/// Terrain protection and telemetry after Avian integration. Other character
/// contact solvers run before this set to give shared pose access a clear order.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlightViewPostPhysics;

impl Plugin for FlightViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlightViewConfig>()
            .init_resource::<FlightReadout>()
            .init_resource::<TourProgress>()
            .init_resource::<FlightInputState>()
            .init_resource::<RouteState>()
            .add_systems(Startup, setup_flight.in_set(FlightViewStartup))
            .add_systems(
                RunFixedMainLoop,
                (input::read_pilot_input, reset_flight)
                    .chain()
                    .in_set(FlightViewInput)
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(
                PhysicsSchedule,
                update_flight_command
                    .after(PhysicsStepSystems::First)
                    .before(crate::apply_ship_controls),
            )
            .add_systems(
                PhysicsSchedule,
                (protect_terrain_clearance, publish_flight_readout)
                    .chain()
                    .in_set(FlightViewPostPhysics)
                    .in_set(PhysicsStepSystems::Last)
                    .before(crate::enforce_safety_envelope),
            )
            .add_systems(Update, plan_scenic)
            .add_systems(
                PostUpdate,
                (update_route_camera, follow_flight_camera)
                    .chain()
                    .before(TransformSystems::Propagate),
            );
    }
}

fn spawn_pose(config: FlightViewConfig) -> (Vec3, Quat, Vec3) {
    let up = config.spawn_direction.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let normal = up.cross(east).normalize();
    let radius = if config.mode == FlyMode::Tour {
        planet::PLANET_RADIUS + config.tour_altitude
    } else if config.mode == FlyMode::Route {
        // Standing on the ground, as far up as the terrain guard keeps it.
        planet::terrain_radius(up) + config.minimum_clearance + 0.5
    } else {
        planet::terrain_radius(up) + config.spawn_altitude.max(config.minimum_clearance)
    };
    let position = up * radius;
    let view_direction = east * config.view_pitch_down.cos() - up * config.view_pitch_down.sin();
    let orientation = Transform::from_translation(position)
        .looking_to(view_direction, up)
        .rotation;
    (position, orientation, normal)
}

fn setup_flight(world: &mut World) {
    let config = *world.resource::<FlightViewConfig>();
    assert!(config.acceleration.is_finite() && config.acceleration > 0.0);
    assert!(config.cruise_speed > 0.0 && config.cruise_speed <= crate::SAFETY_SPEED);
    assert!(config.surface_speed > 0.0 && config.surface_speed <= config.cruise_speed);
    assert!(config.minimum_clearance.is_finite() && config.minimum_clearance >= 1.0);
    assert!(
        config.route_cruise_height_m > 1_000.0,
        "the route cruises above the atmosphere"
    );
    assert!((10.0..=85.0).contains(&config.route_climb_angle_deg));
    assert!(config.route_speed > 0.0 && config.route_speed <= crate::SAFETY_SPEED);
    assert!(config.route_ground_accel > 0.0 && config.route_touchdown_mps > 0.0);
    assert!(config.route_camera_ease_s > 0.0 && config.route_camera_max_rate > 0.0);
    assert!(config.route_climb_speed > 0.0 && config.route_climb_speed <= config.route_speed);
    assert!(config.route_corner_m >= 0.0);
    let (position, orientation, normal) = spawn_pose(config);
    if config.mode == FlyMode::Route {
        *world.resource_mut::<RouteState>() = match config.route_kind {
            RouteKind::FarSide => RouteState::new(position.normalize(), normal),
            RouteKind::Scenic => RouteState::scenic_pending(position.normalize()),
        };
    }
    let controller = ShipController {
        limits: FlightLimits {
            acceleration: config.acceleration as f64,
            ..Default::default()
        },
        ..Default::default()
    };
    let ship = spawn_ship(world, position, controller);
    world.entity_mut(ship).insert((
        PilotShip,
        Rotation(orientation),
        ProtectedPosition(position),
        Transform::from_translation(position).with_rotation(orientation),
        SweptCcd::default(),
    ));
    world.resource_mut::<FlightInputState>().target_rotation = orientation;
    *world.resource_mut::<TourProgress>() = TourProgress {
        normal,
        previous_direction: position.normalize(),
        ..Default::default()
    };
    if config.startup_camera {
        world.spawn((
            Name::new("Skiff cockpit camera"),
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                fov: 75.0_f32.to_radians(),
                near: 0.2,
                far: 100_000.0,
                ..Default::default()
            }),
            Transform::from_translation(position).with_rotation(orientation),
            FlightCamera,
        ));
    }
}

/// Relocate the existing pilot ship for an explicit mode switch or reset.
/// Velocity, swept terrain state, view target, and readout change together;
/// ordinary flight still moves only through Avian. Invalid poses are rejected.
pub fn teleport_pilot(world: &mut World, position: Vec3, orientation: Quat) -> bool {
    if !position.is_finite()
        || position.length_squared() < 1.0
        || !orientation.is_finite()
        || orientation.length_squared() <= f32::EPSILON
    {
        return false;
    }
    let orientation = orientation.normalize();
    let frame = world.resource::<crate::PhysicsFrame>().0;
    let mut ships = world.query_filtered::<(
        &mut Position,
        &mut Rotation,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &mut ProtectedPosition,
        &mut Transform,
        &mut crate::GlobalShipState,
        &mut LastFlightCommand,
    ), With<PilotShip>>();
    let mut found = false;
    for (
        mut pos,
        mut rot,
        mut velocity,
        mut angular,
        mut previous,
        mut transform,
        mut global,
        mut command,
    ) in ships.iter_mut(world)
    {
        pos.0 = position;
        rot.0 = orientation;
        velocity.0 = Vec3::ZERO;
        angular.0 = Vec3::ZERO;
        previous.0 = position;
        transform.translation = position;
        transform.rotation = orientation;
        global.0 = frame.to_global(pbd_core::frame::KinematicState {
            position: position.as_dvec3(),
            velocity: DVec3::ZERO,
        });
        *command = LastFlightCommand::default();
        found = true;
    }
    if !found {
        return false;
    }
    let mut intent = world.resource_mut::<FlightInputState>();
    intent.target_rotation = orientation;
    intent.axes = Vec3::ZERO;
    intent.cruise = false;
    intent.brake = !intent.enabled;
    intent.reset = false;
    let dampeners = intent.dampeners || !intent.captured;
    let direction = position.normalize();
    let east = Vec3::Y.cross(direction).normalize_or(Vec3::X);
    *world.resource_mut::<TourProgress>() = TourProgress {
        normal: direction.cross(east).normalize(),
        previous_direction: direction,
        ..Default::default()
    };
    let surface_radius = flight_surface_radius(
        direction,
        world.resource::<FlightViewConfig>().mode,
        world.get_resource::<planet::PlanetContact>(),
    );
    let is_in_space = world
        .resource::<CelestialScene>()
        .gravity_at(frame.origin + position.as_dvec3())
        .is_in_space();
    *world.resource_mut::<FlightReadout>() = FlightReadout {
        position,
        altitude: position.length() - planet::PLANET_RADIUS,
        clearance: position.length() - surface_radius,
        latitude_deg: direction.y.clamp(-1.0, 1.0).asin().to_degrees(),
        longitude_deg: direction.z.atan2(direction.x).to_degrees(),
        dampeners,
        is_in_space,
        ..Default::default()
    };
    true
}

fn reset_flight(world: &mut World) {
    if world.resource::<FlightInputState>().reset {
        let (position, orientation, _) = spawn_pose(*world.resource::<FlightViewConfig>());
        teleport_pilot(world, position, orientation);
    }
}

fn flight_surface_radius(
    direction: Vec3,
    mode: FlyMode,
    terrain: Option<&planet::PlanetContact>,
) -> f32 {
    if mode == FlyMode::Manual
        && let Some(terrain) = terrain
    {
        return terrain.sample(direction).radius;
    }
    // Scripted tour verification keeps its original conservative terrain field,
    // including headless runs that deliberately do not initialize GPU geometry.
    planet::terrain_radius(direction)
}

fn update_flight_command(
    config: Res<FlightViewConfig>,
    scene: Res<CelestialScene>,
    frame: Res<PhysicsFrame>,
    mut intent: ResMut<FlightInputState>,
    progress: Res<TourProgress>,
    route: Res<RouteState>,
    mut ships: Query<
        (
            &Position,
            &Rotation,
            &LinearVelocity,
            &AngularVelocity,
            &mut ShipController,
        ),
        With<PilotShip>,
    >,
) {
    for (position, rotation, velocity, angular, mut controller) in &mut ships {
        let gravity = scene
            .gravity_at(frame.0.origin + position.0.as_dvec3())
            .acceleration()
            .as_vec3();
        controller.limits.acceleration = config.acceleration as f64;
        controller.limits.speed = if config.mode == FlyMode::Route {
            config.route_speed as f64
        } else if intent.cruise || config.mode == FlyMode::Tour {
            config.cruise_speed as f64
        } else {
            config.surface_speed as f64
        };
        if config.mode == FlyMode::Route {
            let (acceleration, target_rotation) =
                route::route_command(position.0, velocity.0, &route, &config);
            intent.target_rotation = target_rotation;
            controller.input.thrust =
                (rotation.0.inverse() * (acceleration - gravity) / config.acceleration).as_dvec3();
            controller.input.inertial_dampeners = false;
        } else if config.mode == FlyMode::Tour {
            let (acceleration, target_rotation) =
                tour::tour_command(position.0, velocity.0, progress.normal, *config);
            intent.target_rotation = target_rotation;
            controller.input.thrust =
                (rotation.0.inverse() * (acceleration - gravity) / config.acceleration).as_dvec3();
            controller.input.inertial_dampeners = false;
        } else {
            let braking = intent.brake || (intent.axes == Vec3::ZERO && !intent.captured);
            let assistance = intent.dampeners || braking;
            let compensation = if assistance { -gravity } else { Vec3::ZERO };
            let axes = if braking { Vec3::ZERO } else { intent.axes };
            // Movement follows the immediate camera view while the physical ship
            // turns through its bounded angular acceleration and speed limits.
            controller.input.thrust = (rotation.0.inverse()
                * (intent.target_rotation * axes + compensation / config.acceleration))
                .as_dvec3();
            controller.input.inertial_dampeners = assistance;
            controller.limits.linear_damping = if braking {
                3.0
            } else if intent.cruise {
                0.08
            } else {
                0.45
            };
        }
        // Quaternion target tracking commands angular acceleration, never a pose.
        let angular_acceleration = tour::attitude_acceleration(
            rotation.0,
            intent.target_rotation,
            angular.0,
            controller.limits.angular_speed as f32,
            controller.limits.angular_acceleration as f32,
        );
        controller.input.rotation = (rotation.0.inverse() * angular_acceleration
            / controller.limits.angular_acceleration as f32)
            .as_dvec3();
        controller.input.rotational_dampeners = false;
    }
}

fn protect_terrain_clearance(
    config: Res<FlightViewConfig>,
    terrain: Option<Res<planet::PlanetContact>>,
    mut progress: ResMut<TourProgress>,
    mut ships: Query<(&mut Position, &mut LinearVelocity, &mut ProtectedPosition), With<PilotShip>>,
) {
    for (mut position, mut velocity, mut previous) in &mut ships {
        // Short conservative samples also catch a narrow ridge crossed in one tick.
        // This height-shell guard cannot represent caves or overhangs.
        let start = previous.0;
        let end = position.0;
        let samples = ((end - start).length() / 3.0).ceil().clamp(1.0, 64.0) as u32;
        for sample in 1..=samples {
            let point = start.lerp(end, sample as f32 / samples as f32);
            let up = point.normalize_or(config.spawn_direction.normalize_or(Vec3::Y));
            let safe_radius = flight_surface_radius(up, config.mode, terrain.as_deref())
                + config.minimum_clearance;
            if point.length() < safe_radius {
                position.0 = up * safe_radius;
                let inward_speed = velocity.0.dot(up).min(0.0);
                velocity.0 -= up * inward_speed;
                progress.protection_events += 1;
                break;
            }
        }
        previous.0 = position.0;
    }
}

#[allow(clippy::too_many_arguments)]
fn publish_flight_readout(
    time: Res<Time<Physics>>,
    config: Res<FlightViewConfig>,
    intent: Res<FlightInputState>,
    scene: Res<CelestialScene>,
    frame: Res<PhysicsFrame>,
    terrain: Option<Res<planet::PlanetContact>>,
    mut readout: ResMut<FlightReadout>,
    mut progress: ResMut<TourProgress>,
    mut route: ResMut<RouteState>,
    ships: Query<
        (
            &Position,
            &LinearVelocity,
            &AngularVelocity,
            &LastFlightCommand,
        ),
        With<PilotShip>,
    >,
) {
    for (position, velocity, angular, command) in &ships {
        let direction = position.0.normalize_or(Vec3::Y);
        let clearance =
            position.0.length() - flight_surface_radius(direction, config.mode, terrain.as_deref());
        *readout = FlightReadout {
            position: position.0,
            speed: velocity.0.length(),
            altitude: position.0.length() - planet::PLANET_RADIUS,
            clearance,
            latitude_deg: direction.y.clamp(-1.0, 1.0).asin().to_degrees(),
            longitude_deg: direction.z.atan2(direction.x).to_degrees(),
            dampeners: intent.dampeners
                || intent.brake
                || !intent.captured
                || config.mode != FlyMode::Manual,
            cruise: intent.cruise || config.mode != FlyMode::Manual,
            protection_events: progress.protection_events,
            is_in_space: scene
                .gravity_at(frame.0.origin + position.0.as_dvec3())
                .is_in_space(),
        };
        if config.mode == FlyMode::Route && !route.completed {
            let before = route.elapsed_s;
            route.advance(direction, time.delta_secs());
            // Lines a recording (the obs-record skill's --start-on) or a log
            // reader can time the flight from.
            if before < route::HOLD_S && route.elapsed_s >= route::HOLD_S {
                info!(
                    "ROUTE_LIFTOFF {}: {:.1} degrees round",
                    route.kind.name(),
                    route.destination_rad.to_degrees()
                );
            }
            let (_, left) = route.arcs_m();
            let speed = velocity.0.length();
            route.peak_height_m = route.peak_height_m.max(clearance);
            route.max_speed_mps = route.max_speed_mps.max(speed);
            // Airborne: after the lift, before the final approach.
            if route.elapsed_s > route::HOLD_S + 5.0
                && left > 300.0
                && clearance < route.min_airborne_clearance_m
            {
                route.min_airborne_clearance_m = clearance;
                route.min_airborne_at_m = route.arcs_m().0;
            }
            if route.landed_at_s.is_none()
                && left < route::LANDED_WITHIN_M
                && clearance < route::LANDED_BELOW_M
                && speed < 3.0
            {
                route.landed_at_s = Some(route.elapsed_s);
                let site = route.destination();
                route.touchdown_error_m =
                    direction.dot(site).clamp(-1.0, 1.0).acos() * planet::PLANET_RADIUS;
                info!(
                    "ROUTE_TOUCHDOWN {}: {:.1} s after the start, {:.1} m from the site",
                    route.kind.name(),
                    route.elapsed_s,
                    route.touchdown_error_m
                );
            }
            if let Some(landed) = route.landed_at_s {
                route.completed = route.elapsed_s - landed >= route::SETTLE_S;
                if route.completed {
                    info!("ROUTE_COMPLETE {}", route.kind.name());
                }
            }
        }
        if config.mode == FlyMode::Tour && !progress.completed {
            let sine = progress
                .normal
                .dot(progress.previous_direction.cross(direction));
            let cosine = progress.previous_direction.dot(direction).clamp(-1.0, 1.0);
            progress.angular_distance_rad += sine.atan2(cosine) as f64;
            progress.previous_direction = direction;
            progress.elapsed_seconds += time.delta_secs_f64();
            progress.minimum_clearance = progress.minimum_clearance.min(clearance);
            progress.maximum_speed = progress.maximum_speed.max(readout.speed);
            progress.maximum_acceleration =
                progress.maximum_acceleration.max(command.0.linear.length());
            progress.maximum_angular_speed = progress.maximum_angular_speed.max(angular.0.length());
            progress.maximum_angular_acceleration = progress
                .maximum_angular_acceleration
                .max(command.0.angular.length());
            progress.completed = progress.angular_distance_rad >= std::f64::consts::TAU;
        }
    }
}

/// Lay out a pending scenic route once the weather is in hand, so it can fly
/// into the clouds that are actually there. A headless run (no camera, no
/// weather) plans on the land alone.
fn plan_scenic(
    config: Res<FlightViewConfig>,
    air: Option<Res<crate::atmosphere::Air>>,
    sun: Option<Res<crate::sky::Sun>>,
    mut route: ResMut<RouteState>,
) {
    if config.mode != FlyMode::Route || route.kind != RouteKind::Scenic || route.planned {
        return;
    }
    if air.is_none() && config.startup_camera {
        return;
    }
    let start = route.start;
    *route = RouteState::scenic(
        start,
        &config,
        air.as_deref().map(|air| &*air.now),
        sun.map(|sun| sun.direction()),
    );
}

/// The route's camera rig, eased toward its target once a frame. Runs whether
/// or not a camera entity exists, so the headless route check measures the
/// same rig the window draws.
fn update_route_camera(
    config: Res<FlightViewConfig>,
    time: Res<Time>,
    mut route: ResMut<RouteState>,
    ships: Query<&Position, With<PilotShip>>,
) {
    if config.mode != FlyMode::Route {
        return;
    }
    let Ok(position) = ships.single() else {
        return;
    };
    let target = route::camera_target(position.0, &route, &config, route.elapsed_s);
    route::ease_camera(
        &mut route,
        target,
        time.delta_secs(),
        config.route_camera_ease_s,
        config.route_camera_max_rate,
    );
}

fn follow_flight_camera(
    config: Res<FlightViewConfig>,
    intent: Res<FlightInputState>,
    route: Option<Res<RouteState>>,
    ships: Query<(&Position, &Rotation), With<PilotShip>>,
    mut cameras: Query<&mut Transform, (With<FlightCamera>, Without<PilotShip>)>,
) {
    if !intent.enabled && config.mode == FlyMode::Manual {
        return;
    }
    let Ok((position, rotation)) = ships.single() else {
        return;
    };
    for mut transform in &mut cameras {
        transform.translation = position.0;
        transform.rotation = match config.mode {
            FlyMode::Manual => intent.target_rotation,
            FlyMode::Route => route.as_ref().map_or(rotation.0, |route| route.camera),
            FlyMode::Tour => rotation.0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_input_reaches_this_frames_physics_and_reset_updates_camera_immediately() {
        let mut app = crate::headless_app();
        app.add_plugins(FlightViewPlugin)
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(FlightViewConfig {
                startup_camera: false,
                ..Default::default()
            })
            .insert_resource(crate::CelestialScene::planet_at_origin(
                planet::PLANET_RADIUS as f64,
                1.0,
            ));
        app.finish();
        app.cleanup();
        app.update();
        let camera = app
            .world_mut()
            .spawn((FlightCamera, Transform::IDENTITY))
            .id();
        app.world_mut().resource_mut::<FlightInputState>().captured = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();
        let mut ships = app
            .world_mut()
            .query_filtered::<(&Position, &Rotation, &LinearVelocity), With<PilotShip>>();
        let (position, rotation, velocity) = ships.single(app.world()).unwrap();
        assert!(velocity.0.dot(rotation.0 * Vec3::NEG_Z) > 1.0);
        let initial = spawn_pose(*app.world().resource::<FlightViewConfig>()).0;
        assert!(position.0.distance(initial) > 0.001);

        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyW);
            keys.press(KeyCode::KeyH);
        }
        app.update();
        let (position, rotation, velocity) = ships.single(app.world()).unwrap();
        assert!(position.0.distance(initial) < 0.001);
        assert!(velocity.0.length() < 0.001);
        let view = app.world().entity(camera).get::<Transform>().unwrap();
        assert!(view.translation.distance(position.0) < 0.001);
        assert!(view.rotation.angle_between(rotation.0) < 0.001);
    }

    #[test]
    fn mouse_look_reaches_camera_this_frame_even_without_a_physics_tick() {
        use bevy::{input::mouse::AccumulatedMouseMotion, time::TimeUpdateStrategy};

        let mut app = crate::headless_app();
        app.add_plugins(FlightViewPlugin)
            .insert_resource(FlightViewConfig {
                startup_camera: false,
                ..Default::default()
            })
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<AccumulatedMouseMotion>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(
                std::time::Duration::ZERO,
            ));
        app.finish();
        app.cleanup();
        app.update();
        let mut ships = app
            .world_mut()
            .query_filtered::<(Entity, &Position, &Rotation), With<PilotShip>>();
        let (ship, position, rotation) = ships.single(app.world()).unwrap();
        let position = position.0;
        let rotation = rotation.0;
        let delta = Vec2::new(1_400.0, -350.0);
        app.world_mut()
            .resource_mut::<FlightInputState>()
            .set_captured(true);
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = delta;
        let camera = app
            .world_mut()
            .spawn((FlightCamera, Transform::IDENTITY))
            .id();
        let ticks = app.world().resource::<crate::SimulationClock>().ticks;
        app.update();
        let view = app.world().entity(camera).get::<Transform>().unwrap();
        let expected = rotation
            * Quat::from_rotation_y(-delta.x * MOUSE_LOOK_SENSITIVITY)
            * Quat::from_rotation_x(-delta.y * MOUSE_LOOK_SENSITIVITY);
        assert_eq!(
            app.world().resource::<crate::SimulationClock>().ticks,
            ticks
        );
        assert_eq!(view.translation, position);
        assert!(view.rotation.angle_between(expected) < 0.001);
        assert_eq!(
            app.world().entity(ship).get::<Position>().unwrap().0,
            position
        );
        assert_eq!(
            app.world().entity(ship).get::<Rotation>().unwrap().0,
            rotation
        );
    }

    #[test]
    fn terrain_projection_moves_camera_directly_to_safe_position() {
        let mut app = App::new();
        let config = FlightViewConfig::default();
        let up = config.spawn_direction;
        let radius = planet::terrain_radius(up) + config.minimum_clearance;
        let start = up * (radius + 10.0);
        let end = up * (radius - 10.0);
        app.insert_resource(config)
            .init_resource::<TourProgress>()
            .init_resource::<FlightInputState>()
            .add_systems(Update, protect_terrain_clearance)
            .add_systems(PostUpdate, follow_flight_camera);
        let ship = app
            .world_mut()
            .spawn((
                PilotShip,
                Position(end),
                Rotation(Quat::IDENTITY),
                LinearVelocity(-up * 100.0),
                ProtectedPosition(start),
            ))
            .id();
        let camera = app
            .world_mut()
            .spawn((FlightCamera, Transform::from_translation(start)))
            .id();
        app.update();
        let body = app.world().entity(ship);
        let position = body.get::<Position>().unwrap().0;
        assert_eq!(app.world().resource::<TourProgress>().protection_events, 1);
        assert!((position.length() - radius).abs() < 0.001);
        assert_eq!(
            app.world()
                .entity(camera)
                .get::<Transform>()
                .unwrap()
                .translation,
            position
        );
    }

    #[test]
    fn manual_thrust_follows_instant_view_while_body_attitude_stays_bounded() {
        let mut app = crate::headless_app();
        app.add_plugins(FlightViewPlugin)
            .insert_resource(FlightViewConfig {
                startup_camera: false,
                ..Default::default()
            });
        app.finish();
        app.cleanup();
        app.update();
        let original_view = app.world().resource::<FlightInputState>().view_rotation();
        let target_view = original_view * Quat::from_rotation_y(2.5);
        {
            let mut intent = app.world_mut().resource_mut::<FlightInputState>();
            intent.set_captured(true);
            intent.sync_view_rotation(target_view);
            intent.axes = Vec3::NEG_Z;
            intent.dampeners = false;
        }
        app.update();
        let mut ships = app
            .world_mut()
            .query_filtered::<(&Rotation, &LinearVelocity, &LastFlightCommand), With<PilotShip>>();
        let (rotation, velocity, command) = ships.single(app.world()).unwrap();
        assert!(velocity.0.normalize().dot(target_view * Vec3::NEG_Z) > 0.999);
        assert!(rotation.0.angle_between(original_view) < 0.01);
        assert!(rotation.0.angle_between(target_view) > 2.0);
        assert!(command.0.angular.length() <= 3.0001);
        assert!(command.0.linear.length() <= 80.0001);
    }

    #[test]
    fn mode_switch_relocates_same_ship_and_resets_motion_and_swept_position() {
        let mut app = crate::headless_app();
        app.add_plugins(FlightViewPlugin)
            .insert_resource(FlightViewConfig {
                startup_camera: false,
                ..Default::default()
            });
        app.finish();
        app.cleanup();
        app.update();
        let mut ships = app.world_mut().query_filtered::<Entity, With<PilotShip>>();
        let ship = ships.single(app.world()).unwrap();
        app.world_mut()
            .entity_mut(ship)
            .insert((LinearVelocity(Vec3::splat(30.0)), AngularVelocity(Vec3::Y)));
        let position = Vec3::Y * (planet::PLANET_RADIUS + 500.0);
        let rotation = Quat::from_rotation_x(1.0);
        assert!(teleport_pilot(app.world_mut(), position, rotation));
        assert_eq!(ships.single(app.world()).unwrap(), ship);
        let body = app.world().entity(ship);
        assert_eq!(body.get::<Position>().unwrap().0, position);
        assert_eq!(body.get::<ProtectedPosition>().unwrap().0, position);
        assert_eq!(body.get::<LinearVelocity>().unwrap().0, Vec3::ZERO);
        assert_eq!(body.get::<AngularVelocity>().unwrap().0, Vec3::ZERO);
        let frame = app.world().resource::<crate::PhysicsFrame>().0;
        let global = body.get::<crate::GlobalShipState>().unwrap().0;
        assert_eq!(global.position, frame.origin + position.as_dvec3());
        assert_eq!(app.world().resource::<FlightReadout>().position, position);
        assert!(
            app.world()
                .resource::<FlightInputState>()
                .view_rotation()
                .angle_between(rotation)
                < 0.001
        );
        assert!(!teleport_pilot(app.world_mut(), Vec3::NAN, rotation));
        assert_eq!(
            app.world().entity(ship).get::<Position>().unwrap().0,
            position
        );
    }

    #[test]
    fn manual_handoff_guard_and_readout_use_actual_rendered_caps() {
        let contact = planet::PlanetContact::test_planet(3);
        let direction = (0..64)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 64.0;
                Vec3::new(angle.cos(), 0.4, angle.sin()).normalize()
            })
            .find(|&up| (contact.sample(up).radius - planet::terrain_radius(up)).abs() > 1.0)
            .expect("coarse rendered caps differ from point-sampled terrain near cell edges");
        let actual_radius = contact.sample(direction).radius;
        assert_eq!(
            flight_surface_radius(direction, FlyMode::Tour, Some(&contact)),
            planet::terrain_radius(direction),
            "tour guard must retain its original field with or without rendered contacts"
        );
        let mut app = crate::headless_app();
        app.add_plugins(FlightViewPlugin)
            .insert_resource(contact)
            .insert_resource(FlightViewConfig {
                spawn_direction: direction,
                startup_camera: false,
                minimum_clearance: 1.6,
                ..Default::default()
            })
            .insert_resource(crate::CelestialScene::planet_at_origin(
                planet::PLANET_RADIUS as f64,
                1.0,
            ));
        app.finish();
        app.cleanup();
        app.update();
        let eye = direction * (actual_radius + 1.615);
        assert!(teleport_pilot(app.world_mut(), eye, Quat::IDENTITY));
        assert!((app.world().resource::<FlightReadout>().clearance - 1.615).abs() < 0.002);
        app.update();
        assert_eq!(app.world().resource::<TourProgress>().protection_events, 0);
        assert!(
            app.world()
                .resource::<FlightReadout>()
                .position
                .distance(eye)
                < 0.002
        );

        assert!(teleport_pilot(
            app.world_mut(),
            direction * (actual_radius + 1.0),
            Quat::IDENTITY,
        ));
        app.update();
        assert_eq!(app.world().resource::<TourProgress>().protection_events, 1);
        assert!((app.world().resource::<FlightReadout>().clearance - 1.6).abs() < 0.002);
    }

    fn run_tour(spawn_direction: Vec3) -> TourProgress {
        let mut app = crate::headless_app();
        app.add_plugins(FlightViewPlugin)
            .insert_resource(FlightViewConfig {
                mode: FlyMode::Tour,
                spawn_direction,
                startup_camera: false,
                ..Default::default()
            })
            .insert_resource(crate::CelestialScene::planet_at_origin(
                planet::PLANET_RADIUS as f64,
                1.0,
            ));
        app.finish();
        app.cleanup();
        app.update();
        for _ in 0..7_200 {
            app.update();
            if app.world().resource::<TourProgress>().completed {
                break;
            }
        }
        *app.world().resource::<TourProgress>()
    }

    fn assert_valid_tour(report: TourProgress) {
        assert!(report.completed, "tour failed: {report:?}");
        assert!(report.minimum_clearance > 45.0, "tour failed: {report:?}");
        assert!(report.maximum_speed <= 600.01, "tour failed: {report:?}");
        assert!(
            report.maximum_acceleration <= 80.0001,
            "tour failed: {report:?}"
        );
        assert_eq!(report.protection_events, 0, "tour failed: {report:?}");
        assert!(
            report.maximum_angular_speed <= 1.5001,
            "tour failed: {report:?}"
        );
        assert!(
            report.maximum_angular_acceleration <= 3.0001,
            "tour failed: {report:?}"
        );
    }

    #[test]
    fn complete_circumnavigation_uses_avian_and_needs_no_terrain_projection() {
        assert_valid_tour(run_tour(FlightViewConfig::default().spawn_direction));
    }

    #[test]
    fn polar_circumnavigation_crosses_both_poles_with_quaternion_steering() {
        assert_valid_tour(run_tour(Vec3::Y));
    }

    #[test]
    fn flight_space_readout_uses_the_anchor_edge_and_honours_overrides() {
        let mut app = crate::headless_app();
        let mut scene = crate::CelestialScene::planet_at_origin(planet::PLANET_RADIUS as f64, 1.0);
        scene.gravity[0].bands.outer = Some(2.2);
        app.insert_resource(scene)
            .insert_resource(FlightViewConfig {
                startup_camera: false,
                ..Default::default()
            })
            .add_plugins(FlightViewPlugin);
        app.finish();
        app.cleanup();
        app.update();
        for (radius_multiplier, in_space) in [(2.0, false), (2.2, true)] {
            let position = Vec3::Y * planet::PLANET_RADIUS * radius_multiplier;
            assert!(teleport_pilot(app.world_mut(), position, Quat::IDENTITY));
            assert_eq!(
                app.world().resource::<FlightReadout>().is_in_space,
                in_space
            );
            app.update();
            assert_eq!(
                app.world().resource::<FlightReadout>().is_in_space,
                in_space
            );
            assert!(app.world().resource::<FlightReadout>().speed < 0.001);
        }
    }

    #[test]
    fn released_controls_hover_in_the_actual_planet_gravity_field() {
        let mut app = crate::headless_app();
        app.add_plugins(FlightViewPlugin)
            .insert_resource(FlightViewConfig {
                startup_camera: false,
                ..Default::default()
            })
            .insert_resource(crate::CelestialScene::planet_at_origin(
                planet::PLANET_RADIUS as f64,
                1.0,
            ));
        app.finish();
        app.cleanup();
        app.update();
        let mut ships = app
            .world_mut()
            .query_filtered::<&Position, With<PilotShip>>();
        let initial = ships.single(app.world()).unwrap().0;
        for _ in 0..120 {
            app.update();
        }
        let final_position = ships.single(app.world()).unwrap().0;
        assert!(initial.distance(final_position) < 0.02);
        assert!(app.world().resource::<FlightReadout>().speed < 0.001);
        assert_eq!(app.world().resource::<TourProgress>().protection_events, 0);
    }
}
