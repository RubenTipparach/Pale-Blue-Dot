//! First-person planetary walking. Avian integrates the body; CPU cap queries
//! resolve its contact against the same hex triangles drawn by the GPU.
//! This surface motor does not yet supply cave or decorative-tree collisions.

use avian3d::prelude::*;
use bevy::ecs::system::SystemParam;
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
pub const HALF_HEIGHT: f32 = 0.9;
const BODY_RADIUS: f32 = 0.3;
const CONTACT_SKIN: f32 = 0.015;
const PITCH_LIMIT: f32 = 89.0 * std::f32::consts::PI / 180.0;

#[derive(Resource, Clone, Copy)]
pub struct WalkingConfig {
    pub start_walking: bool,
    /// Where a LOADED world puts the walker.
    ///
    /// The spawn rule finds land near a direction and steps four metres off
    /// the trunk, which is right for a new world and wrong for a save: a
    /// player who logged off in a cave, on a ledge or in the sea expects to
    /// come back THERE, and the spawn rule would put them on the surface
    /// nearby. Absent is a new world.
    pub restored: Option<RestoredPose>,
    /// The look pitch a walker starts with when nothing is restored, radians.
    /// A capture that digs along the look sets it; a fresh game looks level.
    pub pitch: f32,
    /// The turn to the right of the default heading a walker starts with when
    /// nothing is restored, radians. The default heading is the pole's east.
    pub yaw: f32,
    pub walk_speed: f32,
    pub sprint_speed: f32,
    pub jump_speed: f32,
    pub step_height: f32,
    /// Height above the feet that decides the body is in water, in metres.
    pub water_body_check_height: f32,
    /// Speed multiplier while the body is in water.
    pub water_movement_mult: f32,
    /// Gravity multiplier while the body is in water.
    pub water_gravity_mult: f32,
    /// Exponential damping on the radial velocity in water, per second.
    pub water_drag_per_s: f32,
    /// Upward acceleration while the swim control is held, in m/s^2.
    pub water_swim_force: f32,
    /// Ceiling on the swim ascent, in m/s.
    pub water_swim_max_rise: f32,
    /// Multiplier on a jump taken from the seabed with the body in water.
    pub water_submerged_jump_mult: f32,
}

/// Exactly where a save left the player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RestoredPose {
    pub position: Vec3,
    pub heading: Vec3,
    pub pitch: f32,
}

impl Default for WalkingConfig {
    fn default() -> Self {
        Self {
            start_walking: true,
            restored: None,
            pitch: 0.0,
            yaw: 0.0,
            walk_speed: 8.0,
            sprint_speed: 14.0,
            jump_speed: 12.0,
            // One terrain cell plus the contact skin: Tenebris walks up one
            // block, and a step it cannot climb is a wall. The cave mouth rule
            // asks the same constant, so a doorway the count offers is a
            // doorway this walker can take.
            step_height: crate::planet::column::STEP_M,
            // Tenebris's water block, measured off its lod.yaml. The feel these
            // make: hold the jump control to rise at about 4.2 m/s, release it
            // and sink at about 2.5 m/s, both being the terminal speeds of
            // `v' = (v - g_w dt) e^{-k dt}`. There is no buoyancy and nowhere
            // to hover, which is the reference's design and not an omission.
            water_body_check_height: 0.50,
            water_movement_mult: 0.50,
            water_gravity_mult: 0.30,
            water_drag_per_s: 3.0,
            water_swim_force: 20.0,
            water_swim_max_rise: 14.0,
            water_submerged_jump_mult: 0.30,
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct WalkingReadout {
    pub active: bool,
    /// Whether the walker has the pointer. Anything the player aims has to
    /// ask: a shovel that swings while the cursor is free swings through
    /// whatever the cursor was over, which is a menu button or another
    /// window.
    pub captured: bool,
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
pub struct GroundState {
    pub previous: Vec3,
    pub grounded: bool,
}

#[derive(Resource)]
pub struct WalkingState {
    pub active: bool,
    pub captured: bool,
    /// A capture script is driving the keys. Pointer capture normally follows
    /// the window's focus, and a headless window never reports any, so without
    /// this a scripted walker stands still for the whole run.
    pub scripted: bool,
    axes: Vec2,
    sprinting: bool,
    /// The jump control on this frame's edge: a standing jump takes it once.
    jump: bool,
    /// The jump control HELD, which is what a swimmer rises on. A ground jump
    /// is an impulse and an edge; a swim thrust is an acceleration and needs
    /// to know the control is still down.
    jump_held: bool,
    heading: Vec3,
    up: Vec3,
    pitch: f32,
    spawn_direction: Vec3,
    body: Entity,
}

impl WalkingState {
    /// Take the look from a camera rotation: its forward becomes the tangent
    /// heading and the pitch off it, clamped as any look is.
    fn look_along(&mut self, view: Quat, up: Vec3) {
        let forward = view * Vec3::NEG_Z;
        self.heading = tangent_heading(forward, up);
        self.pitch = forward
            .dot(up)
            .clamp(-1.0, 1.0)
            .asin()
            .clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Where the player is looking: the tangent heading and the pitch off it.
    /// What an autosave writes, and the only thing outside this module that
    /// needs the view's two halves apart.
    pub fn view(&self) -> (Vec3, f32) {
        (self.heading, self.pitch)
    }

    /// Point the walker at `target` while standing on `up`. The capture
    /// scripts need it; gameplay turns with the mouse.
    pub fn face(&mut self, up: Vec3, target: Vec3) {
        self.up = up.normalize_or(Vec3::Y);
        self.heading = tangent_heading(target, self.up);
        self.pitch = 0.0;
    }

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
    let (position, up) = match config.restored {
        // A save says exactly where, and the ground rule is not consulted:
        // the position it holds is one the player was standing at.
        Some(pose) => (pose.position, pose.position.normalize_or(direction)),
        None => {
            let center = ground.find_land_near(direction).normalize();
            // Cosmetic trunks occupy cell centers. Start four metres beside the
            // trunk, still safely inside this cap, so the first-person view
            // opens onto the land.
            let up = (center * ground.sample(center).radius
                + tangent_heading(Vec3::Y.cross(center), center) * 4.0)
                .normalize();
            let support = footprint(ground, up * (ground.sample(up).radius + HALF_HEIGHT)).support;
            (up * (support + HALF_HEIGHT + CONTACT_SKIN), up)
        }
    };
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
        jump_held: false,
        scripted: false,
        heading: match config.restored {
            Some(pose) => tangent_heading(pose.heading, up),
            None => tangent_heading(
                Quat::from_axis_angle(up, -config.yaw) * Vec3::Y.cross(up),
                up,
            ),
        },
        up,
        pitch: config
            .restored
            .map_or(config.pitch, |pose| pose.pitch)
            .clamp(-PITCH_LIMIT, PITCH_LIMIT),
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

/// Who the player is being: on foot, in the skiff, or aboard a vehicle.
/// Exactly one camera is active and it is the view's; every system that reads
/// "the camera" reads that one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    Walking,
    Flying,
    Vehicle,
}

fn set_active_mode(world: &mut World, walking: bool) {
    set_view(world, if walking { View::Walking } else { View::Flying });
}

/// Hand the player to a view: its body, its input and its camera, and nothing
/// else's.
pub fn set_view(world: &mut World, view: View) {
    let walking = view == View::Walking;
    let body = world.resource::<WalkingState>().body;
    world.resource_mut::<WalkingState>().active = walking;
    world.resource_mut::<WalkingReadout>().active = walking;
    world
        .resource_mut::<FlightInputState>()
        .set_enabled(view == View::Flying);
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
        if view != View::Flying {
            world.entity_mut(ship).insert(ColliderDisabled);
        } else {
            world.entity_mut(ship).remove::<ColliderDisabled>();
        }
    }
    let mut camera_modes = Vec::new();
    for (entity, mut camera, walker, flight, vehicle) in world
        .query::<(
            Entity,
            &mut Camera,
            Has<WalkingCamera>,
            Has<FlightCamera>,
            Has<crate::vehicles::VehicleCamera>,
        )>()
        .iter_mut(world)
    {
        if walker {
            camera.is_active = walking;
        }
        if flight {
            camera.is_active = view == View::Flying;
        }
        if vehicle {
            camera.is_active = view == View::Vehicle;
        }
        if walker || flight || vehicle {
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
    // A menu holds the keyboard: swapping to flight from behind the settings
    // page is the same defect as walking off a cliff while reading it.
    if world
        .get_resource::<crate::controls::MenuOpen>()
        .is_some_and(|open| open.0)
    {
        return;
    }
    // Aboard a vehicle, F is not the way out: G is, and it is the vehicle's.
    if world
        .get_resource::<crate::vehicles::Aboard>()
        .is_some_and(|aboard| aboard.0.is_some())
    {
        return;
    }
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
        let Some((position, velocity)) = world
            .query_filtered::<(&Position, &LinearVelocity), With<PilotShip>>()
            .iter(world)
            .next()
            .map(|(p, v)| (p.0, v.0))
        else {
            return;
        };
        let (rotation, captured) = {
            let input = world.resource::<FlightInputState>();
            (input.view_rotation(), input.is_captured())
        };
        // Where the ship is, in the air or over water, and the walker FALLS
        // from there. This used to stand the walker on the ground under the
        // ship, which the owner called snapping: leaving flight is the same
        // body with gravity back, as Tenebris's `toggle_fly` has it, so the
        // eye stays where the ship's was and the ground contact does the
        // rest, into the sea if that is what is under it.
        drop_walker(world, position, velocity, rotation);
        world.resource_mut::<WalkingState>().captured = captured;
        set_active_mode(world, true);
    }
}

/// Put the walker in the air with its eye at `eye`, moving at `velocity`,
/// looking along `view`, and not grounded: what leaving flight is. The body's
/// centre is the eye less the eye height, which is the inverse of where the
/// walk-to-fly handoff puts the ship.
pub fn drop_walker(world: &mut World, eye: Vec3, velocity: Vec3, view: Quat) {
    let up = eye.normalize_or(Vec3::Y);
    let position = eye - up * (EYE_HEIGHT - HALF_HEIGHT);
    let body = world.resource::<WalkingState>().body;
    world.entity_mut(body).insert((
        Position(position),
        Rotation(Quat::from_rotation_arc(Vec3::Y, up)),
        LinearVelocity(velocity),
        AngularVelocity::ZERO,
        Transform::from_translation(position),
        GroundState {
            previous: position,
            grounded: false,
        },
    ));
    let mut state = world.resource_mut::<WalkingState>();
    state.transport_up(up);
    state.axes = Vec2::ZERO;
    state.jump = false;
    state.look_along(view, up);
}

/// Put the walker exactly where a loaded save says.
///
/// Not [`place_walker`], which finds the ground under a DIRECTION: that is the
/// spawn rule and it is right for a new world, where any patch of land will
/// do. A save holds a place the player was actually standing - a ledge, a
/// cave floor, the sea - and putting them on the surface near it instead is a
/// load that quietly moved them.
pub fn restore(world: &mut World, pose: RestoredPose) {
    let up = pose.position.normalize_or(Vec3::Y);
    let body = world.resource::<WalkingState>().body;
    world.entity_mut(body).insert((
        Position(pose.position),
        Rotation(Quat::from_rotation_arc(Vec3::Y, up)),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
        Transform::from_translation(pose.position),
        // Not grounded: whether the feet are on anything is the ground check's
        // answer on the next tick, and claiming it here would let a player
        // loaded into mid-air jump off nothing.
        GroundState {
            previous: pose.position,
            grounded: false,
        },
    ));
    let mut state = world.resource_mut::<WalkingState>();
    state.transport_up(up);
    state.heading = tangent_heading(pose.heading, up);
    state.pitch = pose.pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    state.axes = Vec2::ZERO;
    state.jump = false;
}

/// Put the walker back at this world's spawn, which is what a NEW world and
/// the reset key both want.
pub fn respawn(world: &mut World) {
    let up = world.resource::<WalkingState>().spawn_direction;
    place_walker(world, up, None);
}

fn place_walker(world: &mut World, up: Vec3, view: Option<Quat>) {
    let terrain = world.resource::<PlanetContact>();
    let sheet = PLANET_RADIUS
        - world
            .resource::<crate::config::WaterSettings>()
            .depth_offset_m;
    // Clear the whole footprint when a handoff lands beside a raised terrace.
    // Over water the walker arrives floating at the sheet rather than standing
    // on the seabed, and is not grounded: the swim model takes it from there.
    let Footprint { support, water, .. } = footprint(
        terrain,
        up * (terrain.sample(up).floor_radius + HALF_HEIGHT),
    );
    let floating = water && support < sheet;
    let feet = if floating {
        sheet
    } else {
        support + CONTACT_SKIN
    };
    let position = up * (feet + HALF_HEIGHT);
    let body = world.resource::<WalkingState>().body;
    world.entity_mut(body).insert((
        Position(position),
        Rotation(Quat::from_rotation_arc(Vec3::Y, up)),
        LinearVelocity::ZERO,
        AngularVelocity::ZERO,
        Transform::from_translation(position),
        GroundState {
            previous: position,
            grounded: !floating,
        },
    ));
    let mut state = world.resource_mut::<WalkingState>();
    state.transport_up(up);
    state.axes = Vec2::ZERO;
    state.jump = false;
    if let Some(view) = view {
        state.look_along(view, up);
    }
}

#[allow(clippy::too_many_arguments)]
fn read_walking_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    mouse: Option<Res<AccumulatedMouseMotion>>,
    mut pointer: crate::controls::Pointer,
    mut state: ResMut<WalkingState>,
    walkers: Query<&Position, With<Walker>>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    if !state.active {
        return;
    }
    let Some(keys) = keys else { return };
    // A menu is a second claimant on the pointer. While it holds it the walker
    // reads nothing - not the look, not the keys, and above all not the click
    // that presses a button, which would otherwise grab the mouse on its way
    // through to the world. `Escape` is the menu's key now and is consumed
    // before this system runs, so there is no arm for it here.
    let menu_open = pointer.menu_holds(&mut state.captured);
    if !menu_open && buttons.is_some_and(|b| b.just_pressed(MouseButton::Left)) {
        state.captured = true;
    }
    if state.scripted {
        state.captured = true;
    }
    if let Ok((window, mut cursor)) = windows.single_mut() {
        if !window.focused && !state.scripted {
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
        state.jump_held = false;
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
    state.jump_held = keys.pressed(KeyCode::Space);
}

#[allow(clippy::type_complexity)]
fn drive_walker(
    mut state: ResMut<WalkingState>,
    config: Res<WalkingConfig>,
    scene: Res<CelestialScene>,
    frame: Res<PhysicsFrame>,
    sea: Sea,
    time: Res<Time>,
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
    let dt = time.delta_secs();
    let mut submerged = false;
    for (position, mut velocity, mut rotation, mut ground) in &mut walkers.p0() {
        let position = position.0;
        let up = position.normalize();
        state.transport_up(up);
        let right = state.heading.cross(up).normalize();
        let wet = sea.state(&config, position);
        submerged = wet.body;
        // Fully submerged is never grounded, even standing on the seabed: it
        // is what makes a swimmer always take gravity and never get the
        // standing jump. Wading, feet down and head out, stays grounded.
        if wet.eyes {
            ground.grounded = false;
        }
        let mut speed = if state.sprinting {
            config.sprint_speed
        } else {
            config.walk_speed
        };
        if wet.body {
            speed *= config.water_movement_mult;
        }
        let planar = (right * state.axes.x + state.heading * state.axes.y) * speed;
        let mut vertical = velocity.0.dot(up);
        let swimming = wet.eyes || (wet.body && !ground.grounded);
        if state.jump || (state.jump_held && swimming) {
            if swimming {
                // Continuous while held, as the reference does it. The second
                // arm is this project's own: with no jetpack behind it, a
                // swimmer floating at the surface beside a bank would have no
                // way out, so the thrust keeps working while the body is in
                // water and the feet are off the bottom.
                vertical =
                    (vertical + config.water_swim_force * dt).min(config.water_swim_max_rise);
            } else if ground.grounded {
                vertical = config.jump_speed
                    * if wet.body {
                        config.water_submerged_jump_mult
                    } else {
                        1.0
                    };
                ground.grounded = false;
            }
        }
        if wet.body {
            // Water takes the fall off, so a swimmer neither plummets nor
            // rockets. Horizontal is written outright from the input every
            // frame here, as it is in the reference, so a tangential drag
            // rate would have nothing to act on.
            vertical *= (-config.water_drag_per_s * dt).exp();
        }
        state.jump = false;
        velocity.0 = planar + up * vertical;
        rotation.0 = Quat::from_rotation_arc(Vec3::Y, up);
    }
    let gravity_scale = if submerged {
        config.water_gravity_mult
    } else {
        1.0
    };
    for mut forces in &mut walkers.p1() {
        let position = frame.0.origin + forces.position().0.as_dvec3();
        forces.apply_linear_acceleration(
            scene.gravity_at(position).acceleration().as_vec3() * gravity_scale,
        );
    }
}

/// Where the walker is against the water, as the reference asks it: three
/// probes up one column. The body probe gates speed, gravity and drag; the eye
/// probe gates the swim thrust and whether the walker can be grounded at all.
/// The surface is the radius the water pass DRAWS the sheet at, so the physics
/// and the picture cannot disagree about where the sea is.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct WaterState {
    pub body: bool,
    pub eyes: bool,
}

/// The terrain and the sheet it is under, together, because answering where
/// the walker is against the water needs both and every caller needs the same
/// answer. The sheet radius is the one the water pass draws at.
#[derive(SystemParam)]
pub(crate) struct Sea<'w> {
    terrain: Res<'w, PlanetContact>,
    settings: Res<'w, crate::config::WaterSettings>,
}

impl Sea<'_> {
    fn sheet_radius(&self) -> f32 {
        PLANET_RADIUS - self.settings.depth_offset_m
    }

    fn state(&self, config: &WalkingConfig, position: Vec3) -> WaterState {
        let direction = position.normalize_or(Vec3::Y);
        // Over land there is no sheet at any height, however low the ground is.
        if self.terrain.sample(direction).water_depth <= 0.0 {
            return WaterState::default();
        }
        let feet = position.length() - HALF_HEIGHT;
        let sheet = self.sheet_radius();
        WaterState {
            body: feet + config.water_body_check_height < sheet,
            eyes: feet + EYE_HEIGHT < sheet,
        }
    }
}

/// What the body at `position` stands between: the highest floor and the
/// lowest ceiling over its five-point footprint.
struct Footprint {
    support: f32,
    ceiling: Option<f32>,
    water: bool,
}

fn footprint(terrain: &PlanetContact, position: Vec3) -> Footprint {
    let up = position.normalize();
    let tangent = up.any_orthonormal_vector() * BODY_RADIUS;
    let cross = up.cross(tangent);
    // The query is made at the FEET: a column answers the run at or below the
    // point asked, and the body's centre can be a metre up a wall the feet
    // are standing beside.
    let feet_radius = position.length() - HALF_HEIGHT;
    let mut support = 0.0_f32;
    let mut ceiling: Option<f32> = None;
    let mut water = false;
    for offset in [Vec3::ZERO, tangent, -tangent, cross, -cross] {
        let feet = (position + offset).normalize() * feet_radius;
        let stand = terrain.stand(feet);
        // The SOLID ground, which under a water cap is the seabed. Taking
        // the sheet put the walker on top of the sea as if it were a floor,
        // which is the other half of why water read as a wall.
        support = support.max(stand.floor_radius);
        // The max floor keeps an edge stable; the MIN ceiling is the same
        // rule read upward, so a head against any part of a roof is a head
        // against the roof.
        ceiling = match (ceiling, stand.ceiling_radius) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        water |= stand.water_depth > 0.0;
    }
    Footprint {
        support,
        ceiling,
        water,
    }
}

/// Whether a body standing on `support` has room under `ceiling`.
fn headroom(support: f32, ceiling: Option<f32>) -> bool {
    ceiling.is_none_or(|c| c - (support + CONTACT_SKIN) >= 2.0 * HALF_HEIGHT + CONTACT_SKIN)
}

/// Swept support with a small capsule footprint. Steep rises block horizontal
/// travel unless the jump has already lifted the feet above the ledge.
fn resolve_ground(
    state: Res<WalkingState>,
    config: Res<WalkingConfig>,
    terrain: Res<PlanetContact>,
    sea: Sea,
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
            let Footprint {
                support, ceiling, ..
            } = footprint(&terrain, candidate);
            let feet = candidate.length() - HALF_HEIGHT;
            let old_feet = accepted.length() - HALF_HEIGHT;
            let rise = support + CONTACT_SKIN - feet;
            let can_step =
                ground.grounded && support + CONTACT_SKIN - old_feet <= config.step_height;
            // A passage lower than the body is a wall, exactly as a tall rise
            // is: Tenebris's headroom check in `try_horizontal_step`, and what
            // stops a walker forcing their head into a low tunnel.
            let low = i > 0 && !headroom(support, ceiling);
            // Water used to be a wall here, which is why the sea could be
            // looked at and never entered. It is passable now: the seabed is
            // ordinary ground, and what stops a swimmer is the seabed's own
            // rise, exactly as on land.
            if low || (rise > 0.03 && !can_step && support + CONTACT_SKIN - old_feet > 0.03) {
                // Keep the last accepted angular position, allowing vertical
                // jump/fall along it to continue against a blocked wall.
                let old_up = accepted.normalize();
                // Only block tangential motion. Preserve the full fixed tick's
                // vertical displacement, independent of the first hit fraction.
                let floor = footprint(&terrain, accepted).support + HALF_HEIGHT + CONTACT_SKIN;
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
        // The head against a ceiling. A jump under a cave roof stops at the
        // roof: the body drops to hang under it and the rise is taken off,
        // tangential motion kept. Underwater only the rise goes, so a swimmer
        // against rock stops rather than being snapped.
        let wet = sea.state(&config, accepted);
        if let Some(ceiling) = footprint(&terrain, accepted).ceiling {
            let up = accepted.normalize();
            let head = accepted.length() + HALF_HEIGHT;
            if head > ceiling - CONTACT_SKIN {
                if !wet.body {
                    accepted = up * (ceiling - HALF_HEIGHT - CONTACT_SKIN);
                }
                let rise = velocity.0.dot(up).max(0.0);
                velocity.0 -= up * rise;
            }
        }
        // The eye probe has the last word: a swimmer is never grounded, so
        // gravity keeps acting and the standing jump stays out of reach.
        let submerged = wet.eyes;
        position.0 = accepted;
        ground.previous = accepted;
        ground.grounded = grounded && !submerged;
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
        captured: state.captured,
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
        let spawn = FlightViewConfig::default().spawn_direction;
        app_with_terrain_at(terrain, spawn)
    }

    fn app_with_terrain_at(terrain: PlanetContact, spawn_direction: Vec3) -> App {
        let mut app = crate::headless_app();
        app.insert_resource(terrain)
            .insert_resource(crate::config::WaterSettings::default())
            .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
            .insert_resource(FlightViewConfig {
                minimum_clearance: EYE_HEIGHT,
                spawn_direction,
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

    /// Put the walker at a direction and a radius, with nothing under way.
    fn place_at(app: &mut App, position: Vec3, grounded: bool) -> Entity {
        let walker = app.world().resource::<WalkingState>().body;
        app.world_mut().entity_mut(walker).insert((
            Position(position),
            Transform::from_translation(position),
            LinearVelocity::ZERO,
            GroundState {
                previous: position,
                grounded,
            },
        ));
        walker
    }

    /// A direction whose seabed is at least `depth` metres under the sheet.
    fn deep_water(terrain: &PlanetContact, depth: f32) -> Vec3 {
        let mut best = Vec3::Y;
        let mut deepest = 0.0_f32;
        for index in 0..4096 {
            let y = 1.0 - 2.0 * (index as f32 + 0.5) / 4096.0;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = std::f32::consts::PI * (3.0 - 5_f32.sqrt()) * index as f32;
            let direction = Vec3::new(r * a.cos(), y, r * a.sin());
            let sample = terrain.sample(direction).water_depth;
            if sample > deepest {
                deepest = sample;
                best = direction;
            }
        }
        assert!(
            deepest >= depth,
            "the test planet has no water {depth} m deep; deepest is {deepest} m"
        );
        best
    }

    /// The sea was a wall: `resolve_ground` rejected any step whose footprint
    /// was wet, so the water could be looked at and never entered. This walks
    /// a walker at the surface straight out over deep water and asserts they
    /// travel, which they could not do at all before.
    #[test]
    fn a_walker_can_walk_into_the_sea_instead_of_being_held_at_the_waterline() {
        let mut app = app();
        let sheet = PLANET_RADIUS
            - app
                .world()
                .resource::<crate::config::WaterSettings>()
                .depth_offset_m;
        let direction = deep_water(app.world().resource::<PlanetContact>(), 4.0);
        let start = direction * (sheet + HALF_HEIGHT);
        let walker = place_at(&mut app, start, true);
        {
            let mut state = app.world_mut().resource_mut::<WalkingState>();
            state.heading = tangent_heading(Vec3::Y.cross(direction), direction);
            state.captured = true;
        }
        // Through the real input path: the state is rewritten from the keys
        // every frame, so a test that sets the axes directly tests nothing.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        for _ in 0..30 {
            app.update();
        }
        let now = app.world().entity(walker).get::<Position>().unwrap().0;
        let travelled = (now.normalize().dot(direction).clamp(-1.0, 1.0)).acos() * PLANET_RADIUS;
        assert!(
            travelled > 1.0,
            "the walker went {travelled:.2} m into the sea"
        );
    }

    /// Leaving flight is the same body with gravity back: the walker starts
    /// with its eye where the ship's was, not grounded, and falls to the
    /// ground rather than being stood on it.
    #[test]
    fn toggling_to_walk_in_the_air_falls_rather_than_snapping() {
        let mut app = app();
        let direction = app.world().resource::<WalkingState>().spawn_direction;
        let ground = app
            .world()
            .resource::<PlanetContact>()
            .sample(direction)
            .floor_radius;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        assert!(!app.world().resource::<WalkingState>().active);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyF);
            keys.clear();
        }
        app.update();
        let ship_at = direction * (ground + 30.0);
        let mut ships = app
            .world_mut()
            .query_filtered::<&mut Position, With<PilotShip>>();
        for mut position in ships.iter_mut(app.world_mut()) {
            position.0 = ship_at;
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        let state = app.world().resource::<WalkingState>();
        assert!(state.active, "the second F should walk");
        let body = state.body;
        let eye =
            app.world().get::<Position>(body).unwrap().0.length() + (EYE_HEIGHT - HALF_HEIGHT);
        assert!(
            (eye - (ground + 30.0)).abs() < 1.5,
            "the eye should stay where the ship was: {eye:.1} vs {:.1}",
            ground + 30.0
        );
        assert!(
            !app.world().get::<GroundState>(body).unwrap().grounded,
            "nothing is underfoot thirty metres up"
        );
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyF);
            keys.clear();
        }
        let mut lowest = f32::MAX;
        for _ in 0..600 {
            app.update();
            lowest = lowest.min(app.world().get::<Position>(body).unwrap().0.length());
        }
        let ground_state = app.world().get::<GroundState>(body).unwrap();
        assert!(ground_state.grounded, "the walker should have landed");
        assert!(
            lowest < ground + 2.0,
            "the walker never came down: lowest centre {lowest:.1} over ground {ground:.1}"
        );
    }

    /// Toggling from flight to walking over open water used to walk to the
    /// nearest land first, which was the only sane thing while the sea was a
    /// wall. It is not now, and it snapped a pilot over the ocean to a shore
    /// they were nowhere near. The walker starts where the ship is and falls
    /// into the water.
    #[test]
    fn toggling_to_walk_over_the_sea_lands_in_the_water_not_on_a_shore() {
        let mut app = app();
        let sheet = PLANET_RADIUS
            - app
                .world()
                .resource::<crate::config::WaterSettings>()
                .depth_offset_m;
        let direction = deep_water(app.world().resource::<PlanetContact>(), 4.0);
        // Fly first, then park the ship over deep water and press F.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        assert!(
            !app.world().resource::<WalkingState>().active,
            "the first F should fly"
        );
        {
            // The harness has no input clear system: a press stays "just
            // pressed" until it is cleared by hand.
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyF);
            keys.clear();
        }
        app.update();
        assert!(
            !app.world().resource::<WalkingState>().active,
            "a cleared F must not toggle again"
        );
        let ship_at = direction * (sheet + 30.0);
        let mut ships = app
            .world_mut()
            .query_filtered::<&mut Position, With<PilotShip>>();
        for mut position in ships.iter_mut(app.world_mut()) {
            position.0 = ship_at;
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();
        let state = app.world().resource::<WalkingState>();
        assert!(state.active, "the second F should walk");
        let body = app.world().entity(state.body);
        let position = body.get::<Position>().unwrap().0;
        let drift = position.normalize().dot(direction).clamp(-1.0, 1.0).acos() * PLANET_RADIUS;
        assert!(
            drift < 1.0,
            "the walker was moved {drift:.1} m away from the ship"
        );
        let eye = position.length() + (EYE_HEIGHT - HALF_HEIGHT);
        assert!(
            (eye - (sheet + 30.0)).abs() < 1.5,
            "the eye should stay where the ship was: {eye:.1} vs {:.1}",
            sheet + 30.0
        );
        assert!(!body.get::<GroundState>().unwrap().grounded);
        let body = state.body;
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyF);
            keys.clear();
        }
        for _ in 0..600 {
            app.update();
        }
        let feet = app.world().get::<Position>(body).unwrap().0.length() - HALF_HEIGHT;
        assert!(
            feet < sheet + 1.0,
            "the walker should have fallen into the sea: feet at {feet:.2} against {sheet:.2}"
        );
        assert!(!app.world().get::<GroundState>(body).unwrap().grounded);
    }

    /// The reference's water model, at its own numbers: sinking is drag
    /// limited rather than a free fall, holding the swim control rises, and a
    /// submerged walker is never grounded, so gravity never stops acting on
    /// them and the standing jump is out of reach.
    #[test]
    fn a_submerged_walker_sinks_to_a_terminal_speed_rises_on_the_control_and_is_never_grounded() {
        let mut app = app();
        let (sheet, config) = {
            let world = app.world();
            (
                PLANET_RADIUS
                    - world
                        .resource::<crate::config::WaterSettings>()
                        .depth_offset_m,
                *world.resource::<WalkingConfig>(),
            )
        };
        let direction = deep_water(app.world().resource::<PlanetContact>(), 4.0);
        // Two metres under the sheet: the eyes are well below it.
        let under = direction * (sheet - 2.0);
        let walker = place_at(&mut app, under, false);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 64.0,
        )));
        for _ in 0..90 {
            app.update();
            assert!(
                !app.world()
                    .entity(walker)
                    .get::<GroundState>()
                    .unwrap()
                    .grounded,
                "a submerged walker must never be grounded"
            );
        }
        let sinking = {
            let entity = app.world().entity(walker);
            let up = entity.get::<Position>().unwrap().0.normalize();
            entity.get::<LinearVelocity>().unwrap().0.dot(up)
        };
        // Terminal is -g_w/k with the water gravity and the drag rate; the
        // reference's numbers put it near -2.5 m/s on a 25 m/s^2 body.
        let gravity =
            pbd_core::gravity::SURFACE_GRAVITY_MPS2_PER_G as f32 * config.water_gravity_mult;
        let terminal = -gravity / config.water_drag_per_s;
        assert!(
            (sinking - terminal).abs() < 0.35 * terminal.abs(),
            "sinking at {sinking:.2} m/s against a terminal of {terminal:.2}"
        );

        app.world_mut().resource_mut::<WalkingState>().captured = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        for _ in 0..90 {
            app.update();
        }
        let rising = {
            let entity = app.world().entity(walker);
            let up = entity.get::<Position>().unwrap().0.normalize();
            entity.get::<LinearVelocity>().unwrap().0.dot(up)
        };
        let rise_terminal = (config.water_swim_force - gravity) / config.water_drag_per_s;
        assert!(
            rising > 0.0,
            "holding the swim control must rise: {rising:.2} m/s"
        );
        assert!(
            (rising - rise_terminal).abs() < 0.4 * rise_terminal,
            "rising at {rising:.2} m/s against a terminal of {rise_terminal:.2}"
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

    /// The column tier at a real cave: the walker settles on the CAVE floor
    /// rather than the surface far above it or the void far below, and a
    /// jump under the roof stops at the roof and comes back down.
    #[test]
    fn a_walker_stands_on_a_cave_floor_and_bumps_its_head_on_the_roof() {
        use pbd_core::column::layer_altitude;
        use std::sync::Arc;
        let settings = crate::config::ColumnSettings::default();
        let spawn = FlightViewConfig::default().spawn_direction;
        let mut terrain = PlanetContact::test_planet(5);
        let anchor = terrain.find_land_near(spawn);
        let set = Arc::new(crate::planet::lod::generate_fine(
            anchor,
            &settings,
            &pbd_core::edits::Edits::new(),
        ));
        terrain.set_fine(&set);
        // A chamber to stand in: the same pick the cave capture makes.
        let records = set.finest_records();
        let mut best: Option<(f32, Vec3, f32, f32)> = None;
        for (index, &slot) in set.columns.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let column = &set.columns.columns[slot];
            let runs = column.drawn_runs();
            let surface = layer_altitude(column.surface().unwrap()) + 1.0;
            for pair in runs.windows(2) {
                let floor = layer_altitude(pair[0].to);
                let roof = layer_altitude(pair[1].from);
                let buried = surface - roof;
                if (2.5..=12.0).contains(&(roof - floor))
                    && (4.0..=40.0).contains(&buried)
                    && best.is_none_or(|(had, _, _, _)| buried > had)
                {
                    let direction = Vec3::from_slice(&records[index].direction_height[..3]);
                    best = Some((buried, direction, floor, roof));
                }
            }
        }
        let (_, here, floor, roof) = best.expect("a chamber in the tier");
        let floor_radius = PLANET_RADIUS + floor;
        let roof_radius = PLANET_RADIUS + roof;
        let mut app = app_with_terrain_at(terrain, spawn);
        let body = place_at(&mut app, here * (floor_radius + 0.5 + HALF_HEIGHT), false);
        let mut ticks = 0;
        while !app.world().get::<GroundState>(body).unwrap().grounded && ticks < 120 {
            app.update();
            ticks += 1;
        }
        let feet = app.world().get::<Position>(body).unwrap().0.length() - HALF_HEIGHT;
        assert!(
            (feet - floor_radius).abs() < 0.03,
            "the walker must settle on the cave floor at {floor} m, not {:.2} m",
            feet - PLANET_RADIUS
        );
        // Jump, the way a player does: the input system clears a jump flag
        // set by hand, so the press has to be a press. The roof is under a
        // jump's reach, so the head meets it.
        app.world_mut().resource_mut::<WalkingState>().captured = true;
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();
        // And let go. Without an input plugin a press stays just-pressed, and
        // the walker jumped again the tick it landed: the trace showed a
        // clean hop to the roof, a landing on the floor at tick 49, and a
        // second hop.
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::Space);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear_just_pressed(KeyCode::Space);
        let mut highest_head = 0.0f32;
        for _ in 0..90 {
            app.update();
            let head = app.world().get::<Position>(body).unwrap().0.length() + HALF_HEIGHT;
            highest_head = highest_head.max(head);
        }
        assert!(
            highest_head <= roof_radius + 0.02,
            "the head reached {:.2} m over a roof at {roof} m",
            highest_head - PLANET_RADIUS
        );
        assert!(
            highest_head > floor_radius + 2.0 * HALF_HEIGHT + 0.2,
            "the jump must actually have left the floor"
        );
        let landed = app.world().get::<Position>(body).unwrap().0.length() - HALF_HEIGHT;
        assert!(app.world().get::<GroundState>(body).unwrap().grounded);
        assert!(
            (landed - floor_radius).abs() < 0.03,
            "and come back down onto the floor"
        );
    }

    #[test]
    fn one_metre_fall_matches_tenebris_and_reports_the_old_gravity_baseline() {
        for (acceleration, height) in [(9.0_f32, 1.0_f32), (25.0, 1.0), (9.0, 6.0), (25.0, 6.0)] {
            let (terrain, flat) = PlanetContact::test_flat_land(5);
            let mut app = app_with_terrain_at(terrain, flat);
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
                "fall {height} m at {acceleration} m/s²: {elapsed:.3} s ({ticks} ticks), analytic {expected:.3} s, landed {:.3} m off resting",
                app.world().get::<Position>(body).unwrap().0.length() - resting.length()
            );
            // Contact is caught by a swept test once a tick, so the reported
            // time carries the tick the walker crossed the floor in plus the
            // one it is resolved in. A six-metre fall at 25 m/s^2 arrives at
            // 17 m/s, which is 0.29 m of travel per tick: two ticks is the
            // granularity, not slack. The landing height below is exact and
            // is what proves nothing drifted.
            assert!(
                (elapsed - expected).abs() <= 2.0 / crate::FIXED_HZ as f32,
                "fell for {elapsed:.3} s against an analytic {expected:.3}"
            );
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
        let support = footprint(app.world().resource::<PlanetContact>(), before).support;
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
