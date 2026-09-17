//! First-person planetary walking. Avian integrates the body; CPU cap queries
//! resolve its contact against the same hex triangles drawn by the GPU.
//! This surface motor does not yet supply cave or decorative-tree collisions.

use avian3d::prelude::*;
use bevy::{
    app::{RunFixedMainLoop, RunFixedMainLoopSystems},
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    transform::TransformSystems,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    CelestialScene, PhysicsFrame,
    flight_view::{
        FlightCamera, FlightInputState, FlightViewConfig, FlightViewInput, FlightViewPostPhysics,
        MOUSE_LOOK_SENSITIVITY, PilotShip, teleport_pilot,
    },
    planet::{PLANET_RADIUS, PlanetContact},
};

pub const EYE_HEIGHT: f32 = 1.6;
const HALF_HEIGHT: f32 = 0.9;
const BODY_RADIUS: f32 = 0.3;
const CONTACT_SKIN: f32 = 0.015;
const PITCH_LIMIT: f32 = 89.0 * std::f32::consts::PI / 180.0;

#[derive(Resource, Clone, Copy)]
pub struct WalkingConfig {
    pub start_walking: bool,
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub jump_speed: f32,
    pub step_height: f32,
}

impl Default for WalkingConfig {
    fn default() -> Self {
        Self {
            start_walking: true,
            walk_speed: 8.0,
            sprint_speed: 14.0,
            jump_speed: 12.0,
            // One terrain cell (`planet::ELEVATION_STEP`) plus the contact skin:
            // Tenebris walks up one block, and a step it cannot climb is a wall.
            step_height: 1.05,
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct WalkingReadout {
    pub active: bool,
    pub grounded: bool,
    pub sprinting: bool,
    pub speed: f32,
    pub altitude: f32,
    pub latitude_deg: f32,
    pub longitude_deg: f32,
}

#[derive(Component)]
pub struct Walker;

#[derive(Component)]
pub struct WalkingCamera;

#[derive(Component)]
struct GroundState {
    previous: Vec3,
    grounded: bool,
}

#[derive(Resource)]
struct WalkingState {
    active: bool,
    captured: bool,
    axes: Vec2,
    sprinting: bool,
    jump: bool,
    heading: Vec3,
    up: Vec3,
    pitch: f32,
    spawn_direction: Vec3,
    body: Entity,
}

impl WalkingState {
    fn transport_up(&mut self, up: Vec3) {
        self.heading = Quat::from_rotation_arc(self.up, up) * self.heading;
        self.heading = tangent_heading(self.heading, up);
        self.up = up;
    }

    fn rotation(&self) -> Quat {
        Transform::IDENTITY
            .looking_to(
                self.heading * self.pitch.cos() + self.up * self.pitch.sin(),
                self.up,
            )
            .rotation
    }
}

pub struct WalkingPlugin;

impl Plugin for WalkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WalkingConfig>()
            .init_resource::<WalkingReadout>()
            .add_systems(PostStartup, setup_walking)
            .add_systems(
                RunFixedMainLoop,
                (switch_mode, read_walking_input)
                    .chain()
                    .before(FlightViewInput)
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(
                PhysicsSchedule,
                drive_walker
                    .after(PhysicsStepSystems::First)
                    .after(crate::apply_ship_controls)
                    .before(PhysicsStepSystems::BroadPhase),
            )
            .add_systems(
                PhysicsSchedule,
                resolve_ground
                    .in_set(PhysicsStepSystems::Last)
                    .before(FlightViewPostPhysics),
            )
            .add_systems(
                PostUpdate,
                follow_walker.before(TransformSystems::Propagate),
            );
    }
}

fn tangent_heading(forward: Vec3, up: Vec3) -> Vec3 {
    (forward - up * forward.dot(up)).normalize_or(Vec3::Y.cross(up).normalize_or(Vec3::X))
}

fn setup_walking(world: &mut World) {
    let config = *world.resource::<WalkingConfig>();
    assert!(config.walk_speed.is_finite() && config.walk_speed > 0.0);
    assert!(config.sprint_speed.is_finite() && config.sprint_speed >= config.walk_speed);
    assert!(config.jump_speed.is_finite() && config.jump_speed > 0.0);
    assert!(config.step_height.is_finite() && config.step_height >= 0.0);
    let direction = world.resource::<FlightViewConfig>().spawn_direction;
    let ground = world.resource::<PlanetContact>();
    let center = ground.find_land_near(direction).normalize();
    // Cosmetic trunks occupy cell centers. Start four metres beside the trunk,
    // still safely inside this cap, so the first-person view opens onto the land.
    let up = (center * ground.sample(center).radius
        + tangent_heading(Vec3::Y.cross(center), center) * 4.0)
        .normalize();
    let support = footprint(ground, up * ground.sample(up).radius).0;
    let position = up * (support + HALF_HEIGHT + CONTACT_SKIN);
    let body = world
        .spawn((
            Name::new("Planet walker"),
            Walker,
            RigidBody::Dynamic,
            Collider::capsule(BODY_RADIUS, 2.0 * (HALF_HEIGHT - BODY_RADIUS)),
            LockedAxes::ROTATION_LOCKED,
            Mass(80.0),
            SleepingDisabled,
            Position(position),
            Rotation(Quat::from_rotation_arc(Vec3::Y, up)),
            LinearVelocity::ZERO,
            AngularVelocity::ZERO,
            Transform::from_translation(position),
            GroundState {
                previous: position,
                grounded: true,
            },
        ))
        .id();
    let state = WalkingState {
        active: config.start_walking,
        captured: false,
        axes: Vec2::ZERO,
        sprinting: false,
        jump: false,
        heading: tangent_heading(Vec3::Y.cross(up), up),
        up,
        pitch: 0.0,
        spawn_direction: up,
        body,
    };
    world.spawn((
        Name::new("First-person walking camera"),
        Camera3d::default(),
        Camera {
            is_active: config.start_walking,
            ..default()
        },
        Transform::from_translation(position + up * (EYE_HEIGHT - HALF_HEIGHT))
            .with_rotation(state.rotation()),
        WalkingCamera,
    ));
    world.insert_resource(state);
    set_active_mode(world, config.start_walking);
}

fn set_active_mode(world: &mut World, walking: bool) {
    let body = world.resource::<WalkingState>().body;
    world.resource_mut::<WalkingState>().active = walking;
    world.resource_mut::<WalkingReadout>().active = walking;
    world
        .resource_mut::<FlightInputState>()
        .set_enabled(!walking);
    if walking {
        world
            .entity_mut(body)
            .remove::<(RigidBodyDisabled, ColliderDisabled)>();
    } else {
        world
            .entity_mut(body)
            .insert((RigidBodyDisabled, ColliderDisabled));
    }
    let ships: Vec<_> = world
        .query_filtered::<Entity, With<PilotShip>>()
        .iter(world)
        .collect();
    for ship in ships {
        if walking {
            world.entity_mut(ship).insert(ColliderDisabled);
        } else {
            world.entity_mut(ship).remove::<ColliderDisabled>();
        }
    }
    let mut camera_modes = Vec::new();
    for (entity, mut camera, walker, flight) in world
        .query::<(Entity, &mut Camera, Has<WalkingCamera>, Has<FlightCamera>)>()
        .iter_mut(world)
    {
        if walker {
            camera.is_active = walking;
        }
        if flight {
            camera.is_active = !walking;
        }
        if walker || flight {
            camera_modes.push((entity, camera.is_active));
        }
    }
    // Bevy's automatic UI camera selection includes inactive cameras. Keep an
    // explicit default on the active view so F also hands the HUD across.
    for (entity, active) in camera_modes {
        if active {
            world.entity_mut(entity).insert(bevy::ui::IsDefaultUiCamera);
        } else {
            world
                .entity_mut(entity)
                .remove::<bevy::ui::IsDefaultUiCamera>();
        }
    }
}

/// Mode handoff is explicit and instantaneous; the existing ship entity survives.
fn switch_mode(world: &mut World) {
    let Some(keys) = world.get_resource::<ButtonInput<KeyCode>>() else {
        return;
    };
    let toggle = keys.just_pressed(KeyCode::KeyF);
    let reset = keys.just_pressed(KeyCode::KeyR);
    let active = world.resource::<WalkingState>().active;
    if active && reset {
        let up = world.resource::<WalkingState>().spawn_direction;
        place_walker(world, up, None);
        let mut state = world.resource_mut::<WalkingState>();
        state.pitch = 0.0;
        state.heading = tangent_heading(Vec3::Y.cross(up), up);
    }
    if !toggle {
        return;
    }
    if active {
        let state = world.resource::<WalkingState>();
        let position = world.get::<Position>(state.body).unwrap().0;
        let rotation = state.rotation();
        let captured = state.captured;
        let eye = position + position.normalize() * (EYE_HEIGHT - HALF_HEIGHT);
        teleport_pilot(world, eye, rotation);
        set_active_mode(world, false);
        world
            .resource_mut::<FlightInputState>()
            .set_captured(captured);
    } else {
        let Some(position) = world
            .query_filtered::<&Position, With<PilotShip>>()
            .iter(world)
            .next()
            .map(|p| p.0)
        else {
            return;
        };
        let (rotation, captured) = {
            let input = world.resource::<FlightInputState>();
            (input.view_rotation(), input.is_captured())
        };
        let terrain = world.resource::<PlanetContact>();
        let direction = position.normalize();
        let up = if terrain.sample(direction).water_depth > 0.0 {
            terrain.find_land_near(direction)
        } else {
            direction
        };
        place_walker(world, up, Some(rotation));
        world.resource_mut::<WalkingState>().captured = captured;
        set_active_mode(world, true);
    }
}

fn place_walker(world: &mut World, mut up: Vec3, view: Option<Quat>) {
    let terrain = world.resource::<PlanetContact>();
    let (mut support, water) = footprint(terrain, up * terrain.sample(up).radius);
    if water {
        let center = terrain.find_land_near(up);
        up = (center * terrain.sample(center).radius
            + tangent_heading(Vec3::Y.cross(center), center) * 4.0)
            .normalize();
        support = footprint(terrain, up * terrain.sample(up).radius).0;
    }
    // Clear the whole footprint when a handoff lands beside a raised terrace.
    let position = up * (support + HALF_HEIGHT + CONTACT_SKIN);
    let body = world.resource::<WalkingState>().body;
    world.entity_mut(body).insert((
        Position(position),
        Rotation(Quat::from_rotation_arc(Vec3::Y, up)),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
        Transform::from_translation(position),
        GroundState {
            previous: position,
            grounded: true,
        },
    ));
    let mut state = world.resource_mut::<WalkingState>();
    state.transport_up(up);
    state.axes = Vec2::ZERO;
    state.jump = false;
    if let Some(view) = view {
        let forward = view * Vec3::NEG_Z;
        state.heading = tangent_heading(forward, up);
        state.pitch = forward
            .dot(up)
            .clamp(-1.0, 1.0)
            .asin()
            .clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }
}

#[allow(clippy::too_many_arguments)]
fn read_walking_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse: Option<Res<AccumulatedMouseMotion>>,
    mut state: ResMut<WalkingState>,
    walkers: Query<&Position, With<Walker>>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    if !state.active {
        return;
    }
    let Some(keys) = keys else { return };
    if keys.just_pressed(KeyCode::Escape) {
        state.captured = false;
    }
    if buttons.is_some_and(|b| b.just_pressed(MouseButton::Left)) {
        state.captured = true;
    }
    if let Ok((window, mut cursor)) = windows.single_mut() {
        if !window.focused {
            state.captured = false;
        }
        cursor.visible = !state.captured;
        cursor.grab_mode = if state.captured {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
    }
    state.axes = Vec2::ZERO;
    state.sprinting = false;
    if !state.captured {
        state.jump = false;
        return;
    }
    if let Ok(position) = walkers.single() {
        state.transport_up(position.0.normalize());
    }
    let delta = mouse.map_or(Vec2::ZERO, |m| m.delta);
    if delta.is_finite() {
        state.heading =
            Quat::from_axis_angle(state.up, -delta.x * MOUSE_LOOK_SENSITIVITY) * state.heading;
        state.pitch =
            (state.pitch - delta.y * MOUSE_LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }
    let axis = |positive, negative| {
        i32::from(keys.pressed(positive)) as f32 - i32::from(keys.pressed(negative)) as f32
    };
    state.axes = Vec2::new(
        axis(KeyCode::KeyD, KeyCode::KeyA),
        axis(KeyCode::KeyW, KeyCode::KeyS),
    )
    .clamp_length_max(1.0);
    state.sprinting = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    // Keep a jump press until a physics tick consumes it, even at >60 render FPS.
    state.jump |= keys.just_pressed(KeyCode::Space);
}

#[allow(clippy::type_complexity)]
fn drive_walker(
    mut state: ResMut<WalkingState>,
    config: Res<WalkingConfig>,
    scene: Res<CelestialScene>,
    frame: Res<PhysicsFrame>,
    mut walkers: ParamSet<(
        Query<
            (
                &Position,
                &mut LinearVelocity,
                &mut Rotation,
                &mut GroundState,
            ),
            With<Walker>,
        >,
        Query<Forces, With<Walker>>,
    )>,
) {
    if !state.active {
        return;
    }
    // Forces already writes LinearVelocity and reads Rotation internally. Keep
    // direct motor changes in a separate borrow to avoid overlapping ECS access.
    for (position, mut velocity, mut rotation, mut ground) in &mut walkers.p0() {
        let position = position.0;
        let up = position.normalize();
        state.transport_up(up);
        let right = state.heading.cross(up).normalize();
        let speed = if state.sprinting {
            config.sprint_speed
        } else {
            config.walk_speed
        };
        let planar = (right * state.axes.x + state.heading * state.axes.y) * speed;
        let mut vertical = velocity.0.dot(up);
        if state.jump && ground.grounded {
            vertical = config.jump_speed;
            ground.grounded = false;
        }
        state.jump = false;
        velocity.0 = planar + up * vertical;
        rotation.0 = Quat::from_rotation_arc(Vec3::Y, up);
    }
    for mut forces in &mut walkers.p1() {
        let position = frame.0.origin + forces.position().0.as_dvec3();
        forces.apply_linear_acceleration(scene.gravity_at(position).acceleration().as_vec3());
    }
}

fn footprint(terrain: &PlanetContact, position: Vec3) -> (f32, bool) {
    let up = position.normalize();
    let tangent = up.any_orthonormal_vector() * BODY_RADIUS;
    let cross = up.cross(tangent);
    let mut support = 0.0_f32;
    let mut water = false;
    for offset in [Vec3::ZERO, tangent, -tangent, cross, -cross] {
        let contact = terrain.sample(position + offset);
        support = support.max(contact.radius);
        water |= contact.water_depth > 0.0;
    }
    (support, water)
}

/// Swept support with a small capsule footprint. Steep rises block horizontal
/// travel unless the jump has already lifted the feet above the ledge.
fn resolve_ground(
    state: Res<WalkingState>,
    config: Res<WalkingConfig>,
    terrain: Res<PlanetContact>,
    mut walkers: Query<(&mut Position, &mut LinearVelocity, &mut GroundState), With<Walker>>,
) {
    if !state.active {
        return;
    }
    for (mut position, mut velocity, mut ground) in &mut walkers {
        let start = ground.previous;
        let destination = position.0;
        let segments = (start.distance(destination) / 0.2).ceil().clamp(1.0, 128.0) as u32;
        let mut accepted = start;
        let mut grounded = false;
        for i in 1..=segments {
            let candidate = start.lerp(destination, i as f32 / segments as f32);
            let up = candidate.normalize();
            let (support, water) = footprint(&terrain, candidate);
            let feet = candidate.length() - HALF_HEIGHT;
            let old_feet = accepted.length() - HALF_HEIGHT;
            let rise = support + CONTACT_SKIN - feet;
            let can_step =
                ground.grounded && support + CONTACT_SKIN - old_feet <= config.step_height;
            if water || (rise > 0.03 && !can_step && support + CONTACT_SKIN - old_feet > 0.03) {
                // Keep the last accepted angular position, allowing vertical
                // jump/fall along it to continue against a blocked wall.
                let old_up = accepted.normalize();
                // Only block tangential motion. Preserve the full fixed tick's
                // vertical displacement, independent of the first hit fraction.
                let floor = footprint(&terrain, accepted).0 + HALF_HEIGHT + CONTACT_SKIN;
                accepted = old_up * destination.length().max(floor);
                velocity.0 = old_up * velocity.0.dot(old_up);
                grounded = accepted.length() <= floor + 0.03 && velocity.0.dot(old_up) <= 0.0;
                if grounded {
                    velocity.0 = Vec3::ZERO;
                }
                break;
            }
            if rise >= -0.03 && velocity.0.dot(up) <= 0.0 {
                accepted = up * (support + HALF_HEIGHT + CONTACT_SKIN);
                let inward_speed = velocity.0.dot(up).min(0.0);
                velocity.0 -= up * inward_speed;
                grounded = true;
            } else if can_step && rise > 0.0 {
                accepted = up * (support + HALF_HEIGHT + CONTACT_SKIN);
                grounded = true;
            } else {
                accepted = candidate;
                grounded = false;
            }
        }
        position.0 = accepted;
        ground.previous = accepted;
        ground.grounded = grounded;
    }
}

fn follow_walker(
    mut state: ResMut<WalkingState>,
    mut readout: ResMut<WalkingReadout>,
    walkers: Query<(&Position, &LinearVelocity, &GroundState), With<Walker>>,
    mut cameras: Query<&mut Transform, With<WalkingCamera>>,
) {
    readout.active = state.active;
    if !state.active {
        return;
    }
    let Ok((position, velocity, ground)) = walkers.single() else {
        return;
    };
    let up = position.0.normalize();
    state.transport_up(up);
    for mut camera in &mut cameras {
        // Direct current pose and mouse orientation: no interpolation, easing,
        // head bob, follow spring, or dependence on the body's angular velocity.
        camera.translation = position.0 + up * (EYE_HEIGHT - HALF_HEIGHT);
        camera.rotation = state.rotation();
    }
    *readout = WalkingReadout {
        active: true,
        grounded: ground.grounded,
        sprinting: state.sprinting,
        speed: velocity.0.length(),
        altitude: position.0.length() - HALF_HEIGHT - PLANET_RADIUS,
        latitude_deg: up.y.clamp(-1.0, 1.0).asin().to_degrees(),
        longitude_deg: up.z.atan2(up.x).to_degrees(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CelestialScene, flight_view::FlightViewPlugin};
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn app() -> App {
        app_with_terrain(PlanetContact::test_planet(5))
    }

    fn app_with_terrain(terrain: PlanetContact) -> App {
        let mut app = crate::headless_app();
        app.insert_resource(terrain)
            .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
            .insert_resource(FlightViewConfig {
                minimum_clearance: EYE_HEIGHT,
                ..default()
            })
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .add_plugins((FlightViewPlugin, WalkingPlugin));
        app.finish();
        app.cleanup();
        app.update();
        app
    }

    #[test]
    fn default_walker_stands_on_rendered_ground_and_camera_is_at_eye_height() {
        let mut app = app();
        for _ in 0..180 {
            app.update();
        }
        let state = app.world().resource::<WalkingState>();
        assert!(state.active);
        assert!(!app.world().resource::<FlightInputState>().is_enabled());
        let body = app.world().entity(state.body);
        assert!(body.get::<GroundState>().unwrap().grounded);
        assert!(body.get::<LinearVelocity>().unwrap().0.length() < 0.01);
        let position = body.get::<Position>().unwrap().0;
        let support = app.world().resource::<PlanetContact>().sample(position);
        assert_eq!(support.water_depth, 0.0);
        assert!((position.length() - support.radius - HALF_HEIGHT - CONTACT_SKIN).abs() < 0.03);
        let mut cameras = app
            .world_mut()
            .query_filtered::<(&Camera, &Transform), With<WalkingCamera>>();
        let (camera, pose) = cameras.single(app.world()).unwrap();
        assert!(camera.is_active);
        assert!(
            (pose.translation.length() - support.radius - EYE_HEIGHT - CONTACT_SKIN).abs() < 0.03
        );
    }

    #[test]
    fn walking_mouse_is_immediate_and_movement_stays_tangent_when_looking_up() {
        let mut app = app();
        let original = app.world().resource::<WalkingState>().rotation();
        app.world_mut().resource_mut::<WalkingState>().captured = true;
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::new(280.0, -900.0);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        let ticks = app.world().resource::<crate::SimulationClock>().ticks;
        app.update();
        assert_eq!(
            app.world().resource::<crate::SimulationClock>().ticks,
            ticks
        );
        let rotation = app.world().resource::<WalkingState>().rotation();
        assert!(rotation.angle_between(original) > 1.0);
        let mut cameras = app
            .world_mut()
            .query_filtered::<&Transform, With<WalkingCamera>>();
        assert!(
            cameras
                .single(app.world())
                .unwrap()
                .rotation
                .angle_between(rotation)
                < 0.001
        );
        app.world_mut()
            .resource_mut::<AccumulatedMouseMotion>()
            .delta = Vec2::ZERO;
        app.update();
        assert!(
            app.world()
                .resource::<WalkingState>()
                .rotation()
                .angle_between(rotation)
                < 0.001
        );
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();
        let body = app
            .world()
            .entity(app.world().resource::<WalkingState>().body);
        let velocity = body.get::<LinearVelocity>().unwrap().0;
        let up = body.get::<Position>().unwrap().0.normalize();
        assert!(
            velocity.dot(up).abs() < 0.02,
            "look changed walking altitude: {velocity:?}"
        );
        assert!((velocity.length() - 8.0).abs() < 0.05);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        app.update();
        let body = app
            .world()
            .entity(app.world().resource::<WalkingState>().body);
        assert!((body.get::<LinearVelocity>().unwrap().0.length() - 14.0).abs() < 0.05);
    }

    #[test]
    fn one_metre_fall_matches_tenebris_and_reports_the_old_gravity_baseline() {
        for (acceleration, height) in [(9.0_f32, 1.0_f32), (25.0, 1.0), (9.0, 6.0), (25.0, 6.0)] {
            let mut app = app();
            app.world_mut().resource_mut::<CelestialScene>().gravity[0].gravity_g =
                acceleration as f64 / pbd_core::gravity::SURFACE_GRAVITY_MPS2_PER_G;
            let body = app.world().resource::<WalkingState>().body;
            let resting = app.world().get::<Position>(body).unwrap().0;
            let start = resting + resting.normalize() * height;
            app.world_mut().entity_mut(body).insert((
                Position(start),
                // Teleports must update Bevy's pose as well as Avian's pose,
                // just like place_walker; otherwise transform sync restores
                // the grounded spawn before the first physics tick.
                Transform::from_translation(start),
                LinearVelocity::ZERO,
                GroundState {
                    previous: start,
                    grounded: false,
                },
            ));
            let mut ticks = 0;
            while !app.world().get::<GroundState>(body).unwrap().grounded && ticks < 120 {
                app.update();
                ticks += 1;
            }
            let elapsed = ticks as f32 / crate::FIXED_HZ as f32;
            let expected = (2.0 * height / acceleration).sqrt();
            println!(
                "fall {height} m at {acceleration} m/s²: {elapsed:.3} s ({ticks} ticks), analytic {expected:.3} s"
            );
            assert!((elapsed - expected).abs() <= 1.0 / crate::FIXED_HZ as f32);
            assert!(
                (app.world().get::<Position>(body).unwrap().0.length() - resting.length()).abs()
                    < 0.03
            );
        }
    }

    #[test]
    fn walker_and_unassisted_ship_sample_the_same_banded_gravity() {
        use pbd_core::{
            DVec3,
            flight::{FlightInput, FlightLimits},
        };
        for radius_multiplier in [1.2, 1.6, 2.0] {
            let mut app = app();
            let walker = app.world().resource::<WalkingState>().body;
            let position = Vec3::Y * PLANET_RADIUS * radius_multiplier;
            app.world_mut().entity_mut(walker).insert((
                Position(position),
                Transform::from_translation(position),
                LinearVelocity::ZERO,
                GroundState {
                    previous: position,
                    grounded: false,
                },
            ));
            let ship = crate::spawn_ship(
                app.world_mut(),
                position,
                crate::ShipController {
                    limits: FlightLimits {
                        acceleration: 80.0,
                        ..Default::default()
                    },
                    input: FlightInput {
                        inertial_dampeners: false,
                        ..Default::default()
                    },
                },
            );
            // The test compares forces at identical positions, not collision response.
            app.world_mut()
                .entity_mut(ship)
                .remove::<Collider>()
                .insert(Transform::from_translation(position));
            let gravity = app
                .world()
                .resource::<CelestialScene>()
                .gravity_at(position.as_dvec3());
            let expected = gravity.acceleration() / crate::FIXED_HZ;
            app.update();
            for body in [walker, ship] {
                let actual = app
                    .world()
                    .get::<LinearVelocity>(body)
                    .unwrap()
                    .0
                    .as_dvec3();
                assert!(
                    (actual - expected).length() < 1e-5,
                    "body={body:?}, actual={actual:?}, expected={expected:?}"
                );
            }
            if radius_multiplier == 2.0 {
                assert!(gravity.is_in_space());
                assert_eq!(expected, DVec3::ZERO);
            }
        }
    }

    #[test]
    fn jump_uses_avian_gravity_and_lands_without_held_key_bunny_hopping() {
        let mut app = app();
        app.world_mut().resource_mut::<WalkingState>().captured = true;
        let body = app.world().resource::<WalkingState>().body;
        let start = app.world().get::<Position>(body).unwrap().0.length();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        // Bevy normally clears just_pressed at the end of an input frame.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(KeyCode::Space);
        let mut peak = start;
        for _ in 0..210 {
            app.update();
            peak = peak.max(app.world().get::<Position>(body).unwrap().0.length());
        }
        assert!(
            (2.7..3.0).contains(&(peak - start)),
            "jump rise={}",
            peak - start
        );
        assert!(app.world().get::<GroundState>(body).unwrap().grounded);
        assert!((app.world().get::<Position>(body).unwrap().0.length() - start).abs() < 0.03);
    }

    #[test]
    fn walk_fly_handoff_keeps_ship_and_switches_only_one_camera_on() {
        fn assert_active_ui_camera(app: &mut App) {
            let mut cameras = app
                .world_mut()
                .query::<(Entity, &Camera, Has<bevy::ui::IsDefaultUiCamera>)>();
            let mut active = None;
            for (entity, camera, ui_default) in cameras.iter(app.world()) {
                assert_eq!(camera.is_active, ui_default);
                if camera.is_active {
                    assert!(active.replace(entity).is_none(), "multiple active cameras");
                }
            }
            let mut ui_camera =
                bevy::ecs::system::SystemState::<bevy::ui::DefaultUiCamera>::new(app.world_mut());
            assert_eq!(ui_camera.get(app.world()).get(), active);
            assert!(active.is_some());
        }

        let mut app = app();
        assert_active_ui_camera(&mut app);
        let mut ships = app.world_mut().query_filtered::<Entity, With<PilotShip>>();
        let ship = ships.single(app.world()).unwrap();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        assert!(!app.world().resource::<WalkingState>().active);
        assert!(app.world().resource::<FlightInputState>().is_enabled());
        assert_eq!(ships.single(app.world()).unwrap(), ship);
        assert_active_ui_camera(&mut app);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyF);
            keys.clear();
        }
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        assert!(app.world().resource::<WalkingState>().active);
        assert_eq!(ships.single(app.world()).unwrap(), ship);
        assert_active_ui_camera(&mut app);
    }

    #[test]
    fn pushing_into_a_terrace_keeps_ground_velocity_zero_and_full_jump_height() {
        let (terrain, start, heading) = PlanetContact::test_terrace();
        let mut app = app_with_terrain(terrain);
        place_walker(
            app.world_mut(),
            start,
            Some(Transform::IDENTITY.looking_to(heading, start).rotation),
        );
        app.world_mut().resource_mut::<WalkingState>().captured = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        let body = app.world().resource::<WalkingState>().body;
        let initial = app.world().get::<Position>(body).unwrap().0;
        for _ in 0..120 {
            app.update();
        }
        let position = app.world().get::<Position>(body).unwrap().0;
        let velocity = app.world().get::<LinearVelocity>(body).unwrap().0;
        assert!(app.world().get::<GroundState>(body).unwrap().grounded);
        assert!(
            position.distance(initial) < 0.7,
            "walked through terrace: {position:?}"
        );
        assert!(
            velocity.length() < 0.02,
            "gravity accumulated while blocked: {velocity:?}"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(KeyCode::Space);
        let mut peak = position.length();
        for _ in 0..110 {
            app.update();
            peak = peak.max(app.world().get::<Position>(body).unwrap().0.length());
        }
        assert!(
            peak - position.length() > 2.7,
            "wall reduced jump rise to {}",
            peak - position.length()
        );
    }

    #[test]
    fn landing_beside_a_terrace_clears_footprint_and_allows_walking_away() {
        let (terrain, start, heading) = PlanetContact::test_terrace();
        let near_edge = (start * terrain.sample(start).radius + heading * 0.5).normalize();
        let mut app = app_with_terrain(terrain);
        place_walker(
            app.world_mut(),
            near_edge,
            Some(Transform::IDENTITY.looking_to(-heading, near_edge).rotation),
        );
        let body = app.world().resource::<WalkingState>().body;
        let before = app.world().get::<Position>(body).unwrap().0;
        let support = footprint(app.world().resource::<PlanetContact>(), before).0;
        assert!(before.length() - HALF_HEIGHT >= support);
        app.world_mut().resource_mut::<WalkingState>().captured = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        for _ in 0..15 {
            app.update();
        }
        let after = app.world().get::<Position>(body).unwrap().0;
        assert!(
            (after - before).dot(-heading) > 1.0,
            "ledge trapped separating motion"
        );
    }
}
