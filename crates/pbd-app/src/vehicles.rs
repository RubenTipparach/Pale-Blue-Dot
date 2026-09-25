//! Vehicles in the app: the Kestrel, the Tern and the Loon as entities, the
//! world sampled round each one every tick and handed to `pbd_core::vehicle`,
//! boarding and leaving, and the durable record of every craft.
//!
//! The core steps a craft itself, at the substep rate, against its
//! surroundings (see `pbd_core::vehicle::body` for why not Avian). This module
//! is what those surroundings are made of here: the atmosphere's sample at
//! the craft, the sea table and its state, the planet's gravity, and the
//! walker's own ground query for anything a craft touches.

mod draw;
mod hud;
mod place;
mod view;

pub use view::VehicleCamera;

use crate::atmosphere::Air;
use crate::config::WeatherSettings;
use crate::planet::PlanetContact;
use crate::sea::Sea;
use crate::walking::{View, Walker, WalkingState};
use avian3d::prelude::*;
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::vehicle::record::{RECORD_VERSION, VehicleFile};
use pbd_core::vehicle::spec::VehicleSpecs;
use pbd_core::vehicle::{Craft, CraftState, Hulls, Input, Kind, Mooring, Surroundings};
use pbd_core::wind::AirHere;
use std::sync::Arc;

/// Substeps a craft takes per fixed tick: the engine's own 60 x 4.
pub const SUBSTEPS: u32 = 4;

/// The deepest water an anchor reaches, m.
const ANCHOR_DEPTH_M: f64 = 35.0;

/// How still a craft must be, and for how long, to count as at rest: a rest is
/// written to the save.
const REST_SPEED: f64 = 0.05;
const REST_S: f32 = 2.0;

/// The craft data (`assets/config/vehicles.ron`), the hulls built from it, and
/// the next ID to give a craft.
#[derive(Resource)]
pub struct Fleet {
    pub specs: Arc<VehicleSpecs>,
    pub hulls: Hulls,
    pub next_id: u64,
    /// A durable write is owed: something a save must not lose happened.
    pub dirty: bool,
    /// Whether this world's craft are in it yet.
    pub spawned: bool,
}

impl Fleet {
    pub fn new(specs: VehicleSpecs) -> Self {
        let hulls = Hulls::new(&specs);
        Self {
            specs: Arc::new(specs),
            hulls,
            next_id: 1,
            dirty: false,
            spawned: false,
        }
    }

    /// The whole fleet as the save file holds it.
    pub fn file<'a>(&self, crafts: impl Iterator<Item = &'a Craft>) -> VehicleFile {
        let mut vehicles: Vec<_> = crafts.map(Craft::record).collect();
        vehicles.sort_by_key(|record| record.id);
        VehicleFile {
            version: RECORD_VERSION,
            next_id: self.next_id,
            vehicles,
        }
    }
}

/// A craft in the world.
#[derive(Component)]
pub struct Vehicle {
    pub craft: Craft,
    /// Seconds it has been still, for writing a rest once.
    still: f32,
    rested: bool,
}

impl Vehicle {
    pub fn new(craft: Craft) -> Self {
        Self {
            craft,
            still: 0.0,
            rested: false,
        }
    }
}

/// The craft the player is aboard, if any.
#[derive(Resource, Default, Debug)]
pub struct Aboard(pub Option<Entity>);

/// What the pilot is asking for this tick.
#[derive(Resource, Default)]
pub struct VehicleControls(pub Input);

/// The world time the craft are stepping at: the sun's clock, advanced by the
/// fixed step between frames so every substep has its own time, and snapped
/// back to the sun when they part (a load, a pinned capture clock).
#[derive(Resource, Default)]
pub struct VehicleClock(pub f64);

pub struct VehiclePlugin;

impl Plugin for VehiclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, init_fleet)
            .init_resource::<Aboard>()
            .init_resource::<VehicleControls>()
            .init_resource::<VehicleClock>()
            .init_resource::<view::VehicleView>()
            .add_systems(Startup, (view::spawn_camera, hud::spawn))
            .add_systems(Update, (place::spawn_fleet, scripted_board).chain())
            .add_systems(
                RunFixedMainLoop,
                (board_or_leave, read_controls)
                    .chain()
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(FixedUpdate, step_vehicles)
            .add_systems(
                PostUpdate,
                (draw::place, view::follow)
                    .chain()
                    .before(bevy::transform::TransformSystems::Propagate),
            )
            .add_systems(Update, (hud::show, save_vehicles, view::look));
    }
}

/// Advance every craft one fixed tick.
#[allow(clippy::too_many_arguments)]
fn step_vehicles(
    time: Res<Time>,
    mut vehicles: Query<(Entity, &mut Vehicle)>,
    aboard: Res<Aboard>,
    controls: Res<VehicleControls>,
    world: StepWorld,
    mut clock: ResMut<VehicleClock>,
    mut fleet: ResMut<Fleet>,
    mut walkers: Query<(&mut Position, &mut Transform), With<Walker>>,
) {
    let Some(sea) = world.sea.as_deref() else {
        return;
    };
    let dt = time.delta_secs_f64();
    if dt <= 0.0 {
        return;
    }
    let sun = world.sun.clock.seconds;
    clock.0 = if !world.sun.running || (clock.0 - sun).abs() > 0.5 {
        sun
    } else {
        clock.0 + dt
    };
    let contact = world.contact.as_deref();
    let ground = |p: DVec3| ground_under(contact, p);
    for (entity, mut vehicle) in &mut vehicles {
        let craft = &vehicle.craft;
        let up = craft.body.position.normalize_or(DVec3::Y);
        let direction = up.as_vec3();
        let sample = world.air.as_deref().map(|air| air.now.sample(direction));
        let over_land = crate::planet::terrain::surface_height(direction) >= 0.0;
        let global = world.frame.center + craft.body.position + world.physics.0.origin;
        let env = Surroundings {
            sea: &sea.table,
            sea_state: sea.state_at(world.air.as_deref(), direction),
            sea_radius: sea.radius as f64,
            depth: sea.depth_at(direction),
            air: AirHere {
                wind: sample.map_or(Vec3::ZERO, |s| s.wind),
                upper: sample.map_or(Vec3::ZERO, |s| s.upper),
                rain_mmh: sample.map_or(0.0, |s| s.rain_rate * 3600.0),
                over_land,
            },
            gusts: &world.weather.gusts,
            gravity: world.scene.gravity_at(global).acceleration(),
            current: sample.map_or(DVec3::ZERO, |s| s.current.as_dvec3()),
            ground: &ground,
            seconds: clock.0,
        };
        let input = if aboard.0 == Some(entity) {
            controls.0
        } else {
            Input::default()
        };
        let before = vehicle.craft.clone();
        vehicle.craft.step(dt, SUBSTEPS, &input, &env);
        if !vehicle.craft.is_finite() {
            // A state that left its domain goes back a tick, stopped, rather
            // than carrying a NaN into every frame after it.
            warn!("{} left its domain; held where it was", before.kind.name());
            vehicle.craft = before;
            vehicle.craft.body.velocity = DVec3::ZERO;
            vehicle.craft.body.angular_velocity = DVec3::ZERO;
        }
        // A rest is a place the save must keep.
        let still = vehicle.craft.body.velocity.length() < REST_SPEED
            && vehicle.craft.body.angular_velocity.length() < REST_SPEED;
        if still {
            vehicle.still += dt as f32;
            if vehicle.still >= REST_S && !vehicle.rested {
                vehicle.rested = true;
                fleet.dirty = true;
            }
        } else {
            vehicle.still = 0.0;
            vehicle.rested = false;
        }
        // The player aboard is carried: the walker stands at the craft's exit,
        // so a save taken aboard puts them on foot beside it on reload.
        if aboard.0 == Some(entity) {
            let exit = vehicle.craft.exit();
            let at = (world.frame.center + exit).as_vec3();
            let up = exit.normalize().as_vec3();
            if let Ok((mut position, mut transform)) = walkers.single_mut() {
                position.0 = at + up * crate::walking::HALF_HEIGHT;
                transform.translation = position.0;
            }
        }
    }
}

/// The solid ground under a body-local point, m from the centre: the one
/// answer every craft, its berth and its anchor use. Where the fine column
/// tier is resident it is the tier's (`PlanetContact::stand`, caves and edits
/// included), exactly as the walker gets. Elsewhere it is the exact height
/// field the tier is generated from, NOT the coarse level `stand` falls back
/// to: that is metres off, and a boat left a kilometre from the player sat on
/// a coarse seabed above its own waterline and rolled over.
pub fn ground_under(contact: Option<&PlanetContact>, point: DVec3) -> f64 {
    let at = point.as_vec3();
    let direction = at.normalize_or(Vec3::Y);
    match contact {
        Some(contact) if contact.finest_cell(direction).is_some() => {
            contact.stand(at).floor_radius as f64
        }
        _ => {
            (crate::planet::terrain::PLANET_RADIUS
                + crate::planet::terrain::surface_height(direction)) as f64
        }
    }
}

/// Everything round a craft the step reads.
#[derive(bevy::ecs::system::SystemParam)]
struct StepWorld<'w> {
    sea: Option<Res<'w, Sea>>,
    air: Option<Res<'w, Air>>,
    weather: Res<'w, WeatherSettings>,
    contact: Option<Res<'w, PlanetContact>>,
    scene: Res<'w, crate::CelestialScene>,
    physics: Res<'w, crate::PhysicsFrame>,
    frame: Res<'w, crate::planet::PlanetRenderFrame>,
    sun: Res<'w, crate::sky::Sun>,
}

/// The keys that are a craft's, held this tick.
fn read_controls(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    aboard: Res<Aboard>,
    vehicles: Query<&Vehicle>,
    mut controls: ResMut<VehicleControls>,
) {
    controls.0 = Input::default();
    let (Some(keys), Some(entity)) = (keys, aboard.0) else {
        return;
    };
    if menu.is_some_and(|m| m.0) {
        return;
    }
    let Ok(vehicle) = vehicles.get(entity) else {
        return;
    };
    let axis = |positive: KeyCode, negative: KeyCode| {
        keys.pressed(positive) as i32 as f32 - keys.pressed(negative) as i32 as f32
    };
    let mut input = Input {
        bail: keys.pressed(KeyCode::KeyB),
        ..default()
    };
    match vehicle.craft.kind {
        Kind::Kestrel => {
            input.pitch = axis(KeyCode::KeyS, KeyCode::KeyW);
            input.roll = axis(KeyCode::KeyD, KeyCode::KeyA);
            input.yaw = axis(KeyCode::KeyE, KeyCode::KeyQ);
            input.collective = axis(KeyCode::Space, KeyCode::ControlLeft);
            input.tilt = axis(KeyCode::KeyC, KeyCode::KeyZ);
            input.bail = false;
        }
        Kind::Tern => {
            input.steer = axis(KeyCode::KeyA, KeyCode::KeyD);
            input.sheet = axis(KeyCode::KeyW, KeyCode::KeyS);
            input.crew = axis(KeyCode::KeyE, KeyCode::KeyQ);
        }
        Kind::Loon => {
            input.forward = axis(KeyCode::KeyW, KeyCode::KeyS);
            input.steer = axis(KeyCode::KeyA, KeyCode::KeyD);
            input.rudder = axis(KeyCode::KeyQ, KeyCode::KeyE);
        }
    }
    controls.0 = input;
}

/// A tap of G boards the craft in reach and leaves the one you are in; T makes a boat
/// fast or casts it off; the Kestrel's X and B are its assist and its brake;
/// V swaps the vehicle camera between the seat and the chase view.
fn board_or_leave(world: &mut World) {
    if world
        .get_resource::<crate::controls::MenuOpen>()
        .is_some_and(|open| open.0)
    {
        return;
    }
    let Some(keys) = world.get_resource::<ButtonInput<KeyCode>>() else {
        return;
    };
    let pressed = |k| keys.just_pressed(k);
    // G is read once, by `controls::read_interact_key`, which tells a tap
    // (board or leave) from a hold (the tool picker).
    let tapped = world
        .get_resource::<crate::controls::InteractKey>()
        .is_some_and(|key| key.tapped);
    let (g, t, x, b, v) = (
        tapped,
        pressed(KeyCode::KeyT),
        pressed(KeyCode::KeyX),
        pressed(KeyCode::KeyB),
        pressed(KeyCode::KeyV),
    );
    if !(g || t || x || b || v) {
        return;
    }
    let aboard = world.resource::<Aboard>().0;
    match aboard {
        None if g => board(world),
        None => {}
        Some(entity) => {
            if g {
                leave(world, entity);
                return;
            }
            if v {
                let mut view = world.resource_mut::<view::VehicleView>();
                view.seat = !view.seat;
            }
            if t {
                make_fast_or_cast_off(world, entity);
            }
            let Some(mut vehicle) = world.get_mut::<Vehicle>(entity) else {
                return;
            };
            if let CraftState::Kestrel(state) = &mut vehicle.craft.state {
                if x {
                    state.assist = !state.assist;
                    info!("Kestrel assist {}", if state.assist { "on" } else { "off" });
                }
                if b {
                    state.brake = !state.brake;
                }
            }
        }
    }
}

/// Board the nearest craft in reach of the walker.
fn board(world: &mut World) {
    let Some(state) = world.get_resource::<WalkingState>() else {
        return;
    };
    if !state.active {
        return;
    }
    let captured = state.captured;
    let frame = world.resource::<crate::planet::PlanetRenderFrame>().center;
    let Some(walker) = world
        .query_filtered::<&Position, With<Walker>>()
        .iter(world)
        .next()
        .map(|p| p.0.as_dvec3() - frame)
    else {
        return;
    };
    let nearest = world
        .query::<(Entity, &Vehicle)>()
        .iter(world)
        .map(|(e, v)| {
            (
                e,
                v.craft.body.position.distance(walker),
                v.craft.seat().reach_m as f64,
            )
        })
        .filter(|(_, distance, reach)| distance <= reach)
        .min_by(|a, b| a.1.total_cmp(&b.1));
    let Some((entity, _, _)) = nearest else {
        return;
    };
    take_seat(world, entity, captured);
}

/// Hand the player to `entity`: its controls, its camera. The one path by
/// which anything boards, the G key and a capture script alike.
fn take_seat(world: &mut World, entity: Entity, captured: bool) {
    let name = {
        let mut vehicle = world
            .get_mut::<Vehicle>(entity)
            .expect("the craft just found");
        vehicle.craft.board();
        vehicle.craft.kind.name()
    };
    world.resource_mut::<Aboard>().0 = Some(entity);
    world.resource_mut::<view::VehicleView>().aboard(captured);
    crate::walking::set_view(world, View::Vehicle);
    world.resource_mut::<Fleet>().dirty = true;
    info!("aboard the {name}");
}

/// A capture script's request: board this kind as soon as the fleet is in,
/// since a headless run has no player to walk up to it and press G.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct VehicleScript {
    pub board: Option<Kind>,
    /// In the seat rather than the chase view.
    pub seat: bool,
}

fn scripted_board(world: &mut World) {
    let Some(script) = world.get_resource::<VehicleScript>().copied() else {
        return;
    };
    let Some(kind) = script.board else {
        return;
    };
    if !world.resource::<Fleet>().spawned || world.resource::<Aboard>().0.is_some() {
        return;
    }
    let found = world
        .query::<(Entity, &Vehicle)>()
        .iter(world)
        .find(|(_, v)| v.craft.kind == kind)
        .map(|(e, _)| e);
    world.resource_mut::<VehicleScript>().board = None;
    let Some(entity) = found else {
        warn!("--aboard: this world has no {}", kind.name());
        return;
    };
    world.resource_mut::<view::VehicleView>().seat = script.seat;
    take_seat(world, entity, true);
}

/// Step off: the craft stays, unattended; the walker is put at its exit.
pub fn leave(world: &mut World, entity: Entity) {
    let frame = world.resource::<crate::planet::PlanetRenderFrame>().center;
    let (exit, velocity, name) = {
        let Some(mut vehicle) = world.get_mut::<Vehicle>(entity) else {
            world.resource_mut::<Aboard>().0 = None;
            return;
        };
        vehicle.craft.leave();
        (
            vehicle.craft.exit(),
            vehicle.craft.body.velocity,
            vehicle.craft.kind.name(),
        )
    };
    world.resource_mut::<Aboard>().0 = None;
    let up = exit.normalize().as_vec3();
    let eye = (frame + exit).as_vec3() + up * crate::walking::EYE_HEIGHT;
    let look = world
        .query_filtered::<&Transform, With<VehicleCamera>>()
        .iter(world)
        .next()
        .map_or(Quat::IDENTITY, |t| t.rotation);
    let captured = world.resource::<view::VehicleView>().captured;
    crate::walking::drop_walker(world, eye, velocity.as_vec3(), look);
    world.resource_mut::<WalkingState>().captured = captured;
    crate::walking::set_view(world, View::Walking);
    world.resource_mut::<Fleet>().dirty = true;
    info!("left the {name}; it stays where it is");
}

/// A boat: drop anchor where the water is shallow enough, or weigh it.
fn make_fast_or_cast_off(world: &mut World, entity: Entity) {
    let Some(vehicle) = world.get::<Vehicle>(entity) else {
        return;
    };
    if vehicle.craft.kind == Kind::Kestrel {
        return;
    }
    if vehicle.craft.mooring.is_some() {
        world
            .get_mut::<Vehicle>(entity)
            .expect("checked")
            .craft
            .mooring = None;
        world.resource_mut::<Fleet>().dirty = true;
        info!("cast off");
        return;
    }
    let bow = vehicle.craft.bow();
    let direction = bow.normalize();
    let floor = ground_under(
        world.get_resource::<PlanetContact>(),
        direction * bow.length(),
    );
    let sea = world.resource::<Sea>().radius as f64;
    let depth = sea - floor;
    if !(0.0..=ANCHOR_DEPTH_M).contains(&depth) {
        info!("too deep to anchor: {depth:.0} m");
        return;
    }
    world
        .get_mut::<Vehicle>(entity)
        .expect("checked")
        .craft
        .mooring = Some(Mooring {
        at: direction * floor,
        length: place::rode(depth),
        anchored: true,
    });
    world.resource_mut::<Fleet>().dirty = true;
    info!("anchored in {depth:.1} m");
}

/// Write the fleet whenever something a save must keep happened (a craft
/// made, boarded, left, anchored, cast off or come to rest), on the pose
/// autosave's cadence while any craft moves, and when a menu opens.
fn save_vehicles(
    time: Res<Time>,
    vehicles: Query<&Vehicle>,
    mut fleet: ResMut<Fleet>,
    mut save: Option<ResMut<crate::saves::WorldSave>>,
    mut due: Local<f32>,
) {
    let Some(save) = save.as_deref_mut() else {
        return;
    };
    if !fleet.spawned {
        return;
    }
    *due -= time.delta_secs();
    let moving = vehicles.iter().any(|v| !v.rested);
    let owed = fleet.dirty || (moving && *due <= 0.0);
    if !owed {
        return;
    }
    *due = crate::saves::AUTOSAVE_S;
    let file = fleet.file(vehicles.iter().map(|v| &v.craft));
    if save.snapshot_vehicles(&file) {
        fleet.dirty = false;
    }
}

/// Take this world's craft out of it, for loading another: the player steps
/// off, and what the craft were is handed back for the leaving world's save.
/// The next world's craft come in on the next frame, from its own save.
pub fn put_away(world: &mut World) -> Option<VehicleFile> {
    if let Some(entity) = world.get_resource::<Aboard>().and_then(|a| a.0) {
        leave(world, entity);
    }
    let mut query = world.query::<(Entity, &Vehicle)>();
    let crafts: Vec<(Entity, Craft)> = query
        .iter(world)
        .map(|(e, v)| (e, v.craft.clone()))
        .collect();
    let fleet = world.get_resource::<Fleet>()?;
    let file = fleet
        .spawned
        .then(|| fleet.file(crafts.iter().map(|(_, c)| c)));
    for (entity, _) in crafts {
        world.entity_mut(entity).despawn();
    }
    let mut fleet = world.resource_mut::<Fleet>();
    fleet.spawned = false;
    fleet.dirty = false;
    fleet.next_id = 1;
    file
}

/// The fleet as it stands, for the exit path's last write.
pub fn fleet_file(fleet: &Fleet, vehicles: &Query<&Vehicle>) -> Option<VehicleFile> {
    fleet
        .spawned
        .then(|| fleet.file(vehicles.iter().map(|v| &v.craft)))
}

fn init_fleet(mut commands: Commands, config: Res<crate::config::VehiclesConfig>) {
    commands.insert_resource(Fleet::new(config.0.clone()));
}

#[cfg(test)]
mod tests;
