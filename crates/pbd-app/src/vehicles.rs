//! Vehicles in the app: the Tern, the Loon and the harbours' boats as entities, the
//! world sampled round each one every tick and handed to `pbd_core::vehicle`,
//! boarding and leaving, and the durable record of every craft.
//!
//! The core steps a craft itself, at the substep rate, against its
//! surroundings (see `pbd_core::vehicle::body` for why not Avian). This module
//! is what those surroundings are made of here: the atmosphere's sample at
//! the craft, the sea table and its state, the planet's gravity, and the
//! walker's own ground query for anything a craft touches.

mod chase;
mod draw;
pub(crate) mod harbour;
pub mod hold;
mod hud;
pub mod launch;
mod model;
pub(crate) mod place;
pub(crate) mod view;

pub use view::{VehicleCamera, VehicleView};

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
use pbd_core::vehicle::{Craft, Hulls, Input, Kind, Mooring, Surroundings};
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

/// A resting craft this far from the camera sleeps, metres
/// (`town-frame-time` decision 6).
const SLEEP_BEYOND_M: f64 = 30.0;

/// A sleeping craft is stepped once in this many ticks, to see whether
/// anything has moved it: half a second at 60 Hz.
const SLEEP_CHECK_TICKS: u32 = 30;

/// Whether a craft sleeps through this tick: nobody aboard, at rest for
/// `REST_S`, beyond `SLEEP_BEYOND_M` of the camera, and not due its check.
/// Counts the ticks it sleeps, and is woken by anything that ends the rest
/// (the check step finds it moving, and `step_one` clears `rested`).
fn sleeps(vehicle: &mut Vehicle, aboard: bool, camera: Option<DVec3>) -> bool {
    let far = camera.is_none_or(|c| vehicle.craft.body.position.distance(c) > SLEEP_BEYOND_M);
    if aboard || !vehicle.rested || !far {
        vehicle.dozing = 0;
        return false;
    }
    vehicle.dozing += 1;
    !vehicle.dozing.is_multiple_of(SLEEP_CHECK_TICKS)
}

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
    /// Craft stowed as their records, far from the viewer (`cities-in-the-world`
    /// task 4.2b): in the save, not in the world, until the viewer comes near.
    pub stowed: Vec<pbd_core::vehicle::record::VehicleRecord>,
    /// Records of a kind the game has retired (`retire-kestrel`): held aside
    /// as the save read them, never in the world, and written back after the
    /// live ones, unchanged, on every rewrite.
    pub retired: Vec<pbd_core::vehicle::record::VehicleRecord>,
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
            stowed: Vec::new(),
            retired: Vec::new(),
        }
    }

    /// The whole fleet as the save file holds it: the craft in the world and
    /// the ones stowed, by ID, then the retired records as they were read.
    /// `next_id` is never below a held record's ID, so no new craft takes
    /// one.
    pub fn file<'a>(&self, crafts: impl Iterator<Item = &'a Craft>) -> VehicleFile {
        let mut vehicles: Vec<_> = crafts
            .map(Craft::record)
            .chain(self.stowed.iter().cloned())
            .collect();
        vehicles.sort_by_key(|record| record.id);
        vehicles.extend(self.retired.iter().cloned());
        let held = self.retired.iter().map(|r| r.id + 1).max().unwrap_or(1);
        VehicleFile {
            version: RECORD_VERSION,
            next_id: self.next_id.max(held),
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
    /// Ticks it has slept through (`town-frame-time` decision 6).
    dozing: u32,
}

impl Vehicle {
    pub fn new(craft: Craft) -> Self {
        Self {
            craft,
            still: 0.0,
            rested: false,
            dozing: 0,
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
            .add_systems(
                Update,
                (
                    place::spawn_fleet,
                    harbour::moor_harbours,
                    harbour::stow_and_wake,
                    scripted_board,
                )
                    .chain(),
            )
            .add_systems(
                RunFixedMainLoop,
                (board_or_leave, read_controls)
                    .chain()
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(
                FixedUpdate,
                (step_vehicles, harbour::swing_moored_cogs).chain(),
            )
            .add_systems(
                PostUpdate,
                (draw::place, view::follow)
                    .chain()
                    .after(crate::planet::update_planet_frame)
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(crate::world_open),
            )
            .init_resource::<launch::TakingUp>()
            .add_systems(
                Update,
                launch::launch_and_take_up
                    .run_if(crate::world_open)
                    .before(save_vehicles),
            )
            .add_systems(Update, (hud::show, save_vehicles, view::look))
            .add_systems(
                PostUpdate,
                offer_board
                    .in_set(crate::interact::InteractSet::Offer)
                    .run_if(resource_exists::<crate::interact::Interact>),
            );
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
    mut timing: Local<StepTiming>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
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
    // Where the camera is, body-local, as `towns` measures range.
    let camera = cameras
        .iter()
        .find(|(c, _)| c.is_active)
        .map(|(_, at)| at.translation().as_dvec3() - world.frame.center);
    for (entity, mut vehicle) in &mut vehicles {
        if sleeps(&mut vehicle, aboard.0 == Some(entity), camera) {
            continue;
        }
        let craft = &vehicle.craft;
        let up = craft.body.position.normalize_or(DVec3::Y);
        let direction = up.as_vec3();
        let sample = world.air.as_deref().map(|air| air.now.sample(direction));
        let over_land = crate::planet::terrain::surface_height(direction) >= 0.0;
        let global = world.frame.center + craft.body.position + world.physics.0.origin;
        // A river or a lake over the sea's level floats the craft on its own
        // surface (`small-boats` design 2b): no tide, its waves shoaled
        // to nothing, and the surface's slope as the current.
        let fresh = contact.and_then(|c| fresh_water(c, craft.body.position));
        let env = Surroundings {
            sea: &sea.table,
            sea_state: sea.state_at(world.air.as_deref(), direction),
            tide: if fresh.is_some() {
                Vec4::ZERO
            } else {
                sea.tide
            },
            sea_radius: fresh.map_or(sea.radius as f64, |f| f.surface),
            depth: fresh.map_or(sea.depth_at(direction), |f| f.depth),
            air: AirHere {
                wind: sample.map_or(Vec3::ZERO, |s| s.wind),
                upper: sample.map_or(Vec3::ZERO, |s| s.upper),
                rain_mmh: sample.map_or(0.0, |s| s.rain_rate * 3600.0),
                over_land,
            },
            gusts: &world.weather.gusts,
            gravity: world.scene.gravity_at(global).acceleration(),
            current: fresh.map_or_else(
                || sample.map_or(DVec3::ZERO, |s| s.current.as_dvec3()),
                |f| f.current,
            ),
            ground: &ground,
            seconds: clock.0,
        };
        let input = if aboard.0 == Some(entity) {
            controls.0
        } else {
            Input::default()
        };
        // A cog on its swing is moved by it, not stepped
        // (`harbour::swing_moored_cogs`).
        if !harbour::on_swing(&vehicle.craft) {
            // MEASUREMENT (`town-frame-time` decision 6): each kind's step.
            let started = std::time::Instant::now();
            step_one(&mut vehicle, dt, &input, &env, &mut fleet);
            let entry = timing.1.entry(vehicle.craft.kind.name()).or_default();
            entry.0 += 1;
            entry.1 += started.elapsed().as_secs_f64();
        }
        // MEASUREMENT (`small-boats` 2b): the water the craft aboard floats
        // on, about once a second, with `PBD_BOAT_TRACE` set.
        if aboard.0 == Some(entity)
            && std::env::var_os("PBD_BOAT_TRACE").is_some()
            && ((clock.0 * 60.0) as u64).is_multiple_of(60)
        {
            let craft = &vehicle.craft;
            let planet = f64::from(crate::planet::PLANET_RADIUS);
            let at = craft.reference_position();
            let freeboard = match &craft.telemetry {
                pbd_core::vehicle::Telemetry::Loon(t) => t.freeboard,
                _ => f64::NAN,
            };
            info!(
                "BOAT_TRACE {} ref {:+.3} m, water {:+.3} m ({}), ground {:+.3} m, vy {:+.3}, freeboard {:+.3}",
                craft.kind.name(),
                at.length() - planet,
                env.sea_radius - planet,
                if fresh.is_some() { "fresh" } else { "sea" },
                ground(craft.body.position) - planet,
                craft.body.velocity.dot(at.normalize()),
                freeboard
            );
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
    report_steps(&mut timing, dt as f32);
}

/// MEASUREMENT (`town-frame-time` decision 6): seconds since the last report,
/// and each kind's steps and seconds spent stepping.
type StepTiming = (f32, std::collections::BTreeMap<&'static str, (u32, f64)>);

/// MEASUREMENT (`town-frame-time` decision 6): every 5 s, each kind's mean
/// step and how many were stepped, then forget them.
fn report_steps(timing: &mut StepTiming, dt: f32) {
    timing.0 += dt;
    if timing.0 < 5.0 {
        return;
    }
    let ticks = (timing.0 * crate::FIXED_HZ as f32).round().max(1.0);
    let line: Vec<String> = timing
        .1
        .iter()
        .map(|(kind, (n, s))| {
            format!(
                "{kind} {:.3} ms a step, {:.1} a tick",
                s * 1000.0 / f64::from(*n).max(1.0),
                *n as f32 / ticks
            )
        })
        .collect();
    info!("VEHICLE_STEP {}", line.join("; "));
    *timing = Default::default();
}

/// One craft's tick: stepped, held back a tick where it left its domain,
/// and its rest noted for the save.
fn step_one(vehicle: &mut Vehicle, dt: f64, input: &Input, env: &Surroundings, fleet: &mut Fleet) {
    let before = vehicle.craft.clone();
    vehicle.craft.step(dt, SUBSTEPS, input, env);
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

/// Water standing over the sea's level where a craft is: a river or a lake
/// in the column tier (`small-boats` design 2b).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreshWater {
    /// The surface's radius, m.
    pub surface: f64,
    /// The depth the sea's waves are shoaled to: a river's ripples, not the
    /// open sea's swell.
    pub depth: f32,
    /// The water's flow along the surface, m/s: downhill, as the drawn
    /// water streams (`water_flow`'s pan).
    pub current: DVec3,
}

/// How deep the waves on fresh water are allowed to think it is, m.
const FRESH_WAVE_DEPTH_M: f32 = 0.15;
/// A river's surface current per unit slope, and its cap, m/s: the water
/// mesher's pan, 1.6 m/s at a slope of 0.12.
const FRESH_FLOW_PER_SLOPE: f64 = 1.6 / 0.12;
const FRESH_FLOW_MAX: f64 = 2.0;
/// How far round a craft the surface's slope is read, m.
const FRESH_PROBE_M: f64 = 1.5;

/// The fresh water at a body-local point, if a column holds water there whose
/// top stands more than a quarter metre over the sea's own cells' top; `None` over the
/// sea, on land, or off the column tier. The point is probed where it is and
/// half a metre under, so a hull riding a little high still finds its water.
pub fn fresh_water(contact: &PlanetContact, point: DVec3) -> Option<FreshWater> {
    let up = point.normalize_or(DVec3::Y);
    let top = |p: DVec3| -> Option<f32> {
        [p, p - up * 0.5, p - up * 1.0]
            .into_iter()
            .find_map(|q| contact.column_water(q.as_vec3()).flatten())
    };
    let here = top(point)?;
    // The sea's own cells fill every layer under the sea layer, so their
    // top is that layer's bottom; the sea is DRAWN `depth_offset_m` lower
    // (`Sea::radius`), and craft at sea float on the drawn sheet as before.
    // Only water standing over the sea's top is a river or a lake.
    let sea_top = pbd_core::column::layer_altitude(crate::planet::column::sea_layer());
    if here <= sea_top + 0.25 {
        return None;
    }
    let surface = f64::from(crate::planet::PLANET_RADIUS + here);
    // The slope, by the surface either side along two tangents; a side
    // with no water there reads as level.
    let east = up.any_orthonormal_vector();
    let north = up.cross(east);
    let mut slope = DVec3::ZERO;
    for axis in [east, north] {
        let at = |sign: f64| {
            let q = point + axis * (sign * FRESH_PROBE_M);
            top(q).unwrap_or(here)
        };
        slope += axis * f64::from(at(1.0) - at(-1.0)) / (2.0 * FRESH_PROBE_M);
    }
    let mut current = -slope * FRESH_FLOW_PER_SLOPE;
    if current.length() > FRESH_FLOW_MAX {
        current = current.normalize() * FRESH_FLOW_MAX;
    }
    Some(FreshWater {
        surface,
        depth: FRESH_WAVE_DEPTH_M,
        current,
    })
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
        Kind::Tern => {
            input.steer = axis(KeyCode::KeyA, KeyCode::KeyD);
            input.sheet = axis(KeyCode::KeyW, KeyCode::KeyS);
            input.crew = axis(KeyCode::KeyE, KeyCode::KeyQ);
        }
        Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft => {
            input.forward = axis(KeyCode::KeyW, KeyCode::KeyS);
            input.steer = axis(KeyCode::KeyA, KeyCode::KeyD);
            input.rudder = axis(KeyCode::KeyQ, KeyCode::KeyE);
        }
        Kind::Cog => {
            input.steer = axis(KeyCode::KeyA, KeyCode::KeyD);
            input.sheet = axis(KeyCode::KeyW, KeyCode::KeyS);
        }
    }
    controls.0 = input;
}

/// F boards the craft in reach and leaves the one you are in; T makes a boat
/// fast or casts it off; V swaps the vehicle camera between the seat and the
/// chase view.
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
    // F is the interact key (`interact`): E is a craft's own control
    // (hiking, the blade as a rudder) and rolls in flight, and G is the tool
    // picker. On foot F boards only when nothing usable is aimed.
    let (g, t, v) = (
        pressed(KeyCode::KeyF),
        pressed(KeyCode::KeyT),
        pressed(KeyCode::KeyV),
    );
    if !(g || t || v) {
        return;
    }
    let aboard = world.resource::<Aboard>().0;
    match aboard {
        None if g
            && crate::interact::may(
                world.get_resource::<crate::interact::Interact>(),
                crate::interact::Reader::Boats,
            ) =>
        {
            board(world)
        }
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
        }
    }
}

/// The nearest craft in reach of a walker at `walker` (render-frame
/// local, f64): what F boards and the prompt names.
fn nearest_in_reach<'a>(
    vehicles: impl Iterator<Item = (Entity, &'a Vehicle)>,
    walker: bevy::math::DVec3,
) -> Option<(Entity, &'a Vehicle)> {
    vehicles
        .map(|(e, v)| (e, v, v.craft.body.position.distance(walker)))
        .filter(|(_, v, distance)| *distance <= v.craft.seat().reach_m as f64)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(e, v, _)| (e, v))
}

/// Offer the craft in reach to the interact key: board it.
fn offer_board(
    walking: Option<Res<WalkingState>>,
    aboard: Res<Aboard>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    walkers: Query<&Position, With<Walker>>,
    vehicles: Query<(Entity, &Vehicle)>,
    mut interact: ResMut<crate::interact::Interact>,
) {
    if aboard.0.is_some() || !walking.is_some_and(|w| w.active) {
        return;
    }
    let Some(walker) = walkers.iter().next() else {
        return;
    };
    let at = walker.0.as_dvec3() - frame.center;
    if let Some((_, vehicle)) = nearest_in_reach(vehicles.iter(), at) {
        interact.offer(crate::interact::Offer::new(
            crate::interact::Act::Board,
            vehicle.craft.kind.name().to_lowercase(),
        ));
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
    let nearest =
        nearest_in_reach(world.query::<(Entity, &Vehicle)>().iter(world), walker).map(|(e, _)| e);
    let Some(entity) = nearest else {
        return;
    };
    take_seat(world, entity, captured);
}

/// Hand the player to `entity`: its controls, its camera. The one path by
/// which anything boards, the F key and a capture script alike.
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
/// since a headless run has no player to walk up to it and press F.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct VehicleScript {
    pub board: Option<Kind>,
    /// In the seat rather than the chase view.
    pub seat: bool,
    /// A cog boarded is cast off with its yard braced this many degrees
    /// off square and 2 m/s of way along its bow (`--sail`): a capture's,
    /// so a few seconds show it under way.
    pub sail: Option<f32>,
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
    // A harbour makes its cog and its rowboats when the fleet first meets
    // it, after the fleet is in: wait for them.
    if found.is_none() && matches!(kind, Kind::Cog | Kind::Rowboat) {
        return;
    }
    // A made boat a fresh world has none of: launched where its Loon lies,
    // as a player would launch it (`small-boats`).
    let found = found.or_else(|| {
        if !matches!(kind, Kind::Tube | Kind::Raft) {
            return None;
        }
        let loon = world
            .query::<&Vehicle>()
            .iter(world)
            .find(|v| v.craft.kind == Kind::Loon)
            .map(|v| v.craft.body.position)?;
        let sea = world.resource::<Sea>();
        // Clear of the Loon, in water deep enough to float free.
        let on = place::open_water_near(sea, loon.normalize().as_vec3(), 8.0, 1.5)?;
        let toward = (on - loon).normalize_or(on.normalize().any_orthonormal_vector());
        let tide = world.resource::<Sea>().tide;
        let mut craft = launch::made_craft(&mut world.resource_mut::<Fleet>(), kind, on, toward);
        place::on_the_tide(&mut craft, tide);
        let entity = place::spawn_craft(world, craft);
        launch::write_fleet(world);
        info!("--aboard: launched a {} for the capture", kind.name());
        Some(entity)
    });
    world.resource_mut::<VehicleScript>().board = None;
    let Some(entity) = found else {
        warn!("--aboard: this world has no {}", kind.name());
        return;
    };
    world.resource_mut::<view::VehicleView>().seat = script.seat;
    take_seat(world, entity, true);
    if let Some(deg) = script.sail
        && let Some(mut vehicle) = world.get_mut::<Vehicle>(entity)
        && let pbd_core::vehicle::CraftState::Cog(s) = &mut vehicle.craft.state
    {
        s.yard = f64::from(deg).to_radians();
        let craft = &mut vehicle.craft;
        craft.mooring = None;
        craft.body.velocity = craft.body.axis(pbd_core::vehicle::FORWARD) * 2.0;
        info!("--sail: cast off with the yard braced {deg} deg");
    }
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
    // Off a ship's helm onto its deck (`sail-the-cog` step 3): the walker
    // keeps the ship's way until its feet are down, and the deck carries it
    // from there.
    if world
        .get_resource::<crate::decks::CraftDecks>()
        .is_some_and(|d| d.at(entity).is_some())
    {
        let v = velocity.as_vec3();
        for mut ground in world
            .query_filtered::<&mut crate::walking::GroundState, With<Walker>>()
            .iter_mut(world)
        {
            ground.drift = v - up * v.dot(up);
        }
    }
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
    if make_fast_at_berth(world, entity) {
        return;
    }
    let Some(vehicle) = world.get::<Vehicle>(entity) else {
        return;
    };
    let bow = vehicle.craft.bow();
    let direction = bow.normalize();
    let floor = ground_under(
        world.get_resource::<PlanetContact>(),
        direction * bow.length(),
    );
    // The water it lies in: a river's own surface, else the sea's.
    let sea_radius = world.resource::<Sea>().radius;
    let sea = world
        .get_resource::<PlanetContact>()
        .and_then(|c| fresh_water(c, vehicle.craft.body.position))
        .map_or(sea_radius as f64, |f| f.surface);
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

/// How near its berth a harbour's cog makes fast there, m; how slow it must
/// be, m/s; and how near its berth's heading, rad.
const MAKE_FAST_M: f64 = 4.0;
const MAKE_FAST_SPEED: f64 = 0.8;
const MAKE_FAST_RAD: f64 = 30.0 * std::f64::consts::PI / 180.0;

/// A harbour's cog near its own berth, slow and heading as it lies there,
/// makes fast to its bollard and eases back onto its swing
/// (`sail-the-cog` step 3, part 3). Whether it did.
fn make_fast_at_berth(world: &mut World, entity: Entity) -> bool {
    let Some(vehicle) = world.get::<Vehicle>(entity) else {
        return false;
    };
    let craft = &vehicle.craft;
    let Some((site, harbour::COG)) = craft.berth else {
        return false;
    };
    let berthed = match world.get::<harbour::Berthed>(entity) {
        Some(b) => *b,
        None => {
            let (Some(towns), Some(assets)) = (
                world.get_resource::<crate::towns::Towns>(),
                world.get_resource::<crate::towns::TownAssets>(),
            ) else {
                return false;
            };
            let sea = world.resource::<Sea>().radius;
            let Some(b) = harbour::berth_of(towns, assets, sea, site) else {
                return false;
            };
            b
        }
    };
    let off = craft
        .reference_position()
        .distance(berthed.rest.origin.as_dvec3());
    let speed = craft.body.velocity.length();
    let heading = craft
        .body
        .axis(pbd_core::vehicle::FORWARD)
        .angle_between(berthed.bow.as_dvec3());
    if off > MAKE_FAST_M || speed > MAKE_FAST_SPEED || heading > MAKE_FAST_RAD {
        return false;
    }
    let from = (craft.reference_position(), craft.body.orientation);
    let at = craft.bow();
    let mut vehicle = world.get_mut::<Vehicle>(entity).expect("checked");
    vehicle.craft.mooring = Some(Mooring {
        at,
        length: 2.0,
        anchored: false,
    });
    world.entity_mut(entity).insert(harbour::Berthed {
        easing: Some((from.0, from.1, 0.0)),
        ..berthed
    });
    world.resource_mut::<Fleet>().dirty = true;
    info!("made fast at its berth, {off:.1} m off");
    true
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
    fleet.stowed.clear();
    fleet.retired.clear();
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
