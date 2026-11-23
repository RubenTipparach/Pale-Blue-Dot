//! Vehicles in the app: the Loon, the made boats and the harbours' canoes as entities, the
//! world sampled round each one every tick and handed to `pbd_core::vehicle`,
//! boarding and leaving, and the durable record of every craft.
//!
//! The core steps a craft itself, at the substep rate, against its
//! surroundings (see `pbd_core::vehicle::body` for why not Avian). This module
//! is what those surroundings are made of here: the atmosphere's sample at
//! the craft, the sea table and its state, the planet's gravity, and the
//! walker's own ground query for anything a craft touches.

pub mod carts;
mod chase;
mod draw;
pub(crate) mod harbour;
pub mod hold;
mod hud;
pub mod lamps;
pub mod launch;
mod model;
mod oar_hand;
pub(crate) mod place;
pub mod refund;
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
use pbd_core::vehicle::{Craft, Hulls, Input, Kind, Surroundings};
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
        app.add_plugins(carts::CartsPlugin);
        app.add_systems(PreStartup, init_fleet)
            .init_resource::<Aboard>()
            .init_resource::<VehicleControls>()
            .init_resource::<AnchorOrder>()
            .init_resource::<lamps::CraftLamps>()
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
                    float_log,
                    pole_run,
                )
                    .chain(),
            )
            .add_systems(
                RunFixedMainLoop,
                (anchor_order, board_or_leave, read_controls)
                    .chain()
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
            )
            .add_systems(FixedUpdate, step_vehicles)
            .add_systems(
                PostUpdate,
                (draw::place, view::follow, lamps::gather)
                    .chain()
                    .before(crate::carried_light::gather)
                    .after(crate::planet::update_planet_frame)
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(crate::world_open),
            )
            .init_resource::<launch::TakingUp>()
            .init_resource::<crate::controls::DebugOverlay>()
            .add_systems(
                Update,
                launch::launch_and_take_up
                    .run_if(crate::world_open)
                    .before(save_vehicles),
            )
            .add_systems(Update, (hud::show, save_vehicles, view::look))
            .add_systems(
                Update,
                refund::refund_retired
                    .after(place::spawn_fleet)
                    .run_if(crate::world_open),
            )
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
        let global = world.frame.physics_center() + craft.body.position + world.physics.0.origin;
        // A river or a lake over the sea's level floats the craft on its own
        // surface (`small-boats` design 2b): its waves shoaled to nothing,
        // and the surface's slope as the current.
        let fresh = contact.and_then(|c| fresh_water(c, craft.body.position));
        let env = Surroundings {
            sea: &sea.table,
            sea_state: sea.state_at(world.air.as_deref(), direction),
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
        // MEASUREMENT (`town-frame-time` decision 6): each kind's step.
        let started = std::time::Instant::now();
        step_one(&mut vehicle, dt, &input, &env, &mut fleet);
        let entry = timing.1.entry(vehicle.craft.kind.key()).or_default();
        entry.0 += 1;
        entry.1 += started.elapsed().as_secs_f64();
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
        // MEASUREMENT (`river-craft` 11.1): the oars' stage each time it
        // changes, with `PBD_ROW_TRACE` set, for the captures' timing.
        if aboard.0 == Some(entity)
            && std::env::var_os("PBD_ROW_TRACE").is_some()
            && let Some((_, rowing)) = vehicle.craft.rowing()
            && rowing.stage != timing.2
        {
            timing.2 = rowing.stage;
            info!(
                "ROW_TRACE {:.3} s: {:?}, {:.2} s into the stroke, input {:?}, right grip {:.2?}, eye {:.2?}",
                clock.0,
                rowing.stage,
                rowing.t,
                input.forward,
                rowing.oars[1].geom.grip.to_array(),
                vehicle.craft.eye_local().to_array()
            );
        }
        // The player aboard is carried: the walker stands at the craft's exit,
        // so a save taken aboard puts them on foot beside it on reload.
        if aboard.0 == Some(entity) {
            let exit = vehicle.craft.exit();
            let at = (world.frame.physics_center() + exit).as_vec3();
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
/// The third is the last stroke stage traced (`PBD_ROW_TRACE`).
type StepTiming = (
    f32,
    std::collections::BTreeMap<&'static str, (u32, f64)>,
    Option<pbd_core::vehicle::oars::Stage>,
);

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
    timing.0 = 0.0;
    timing.1.clear();
}

/// One craft's tick: stepped, held back a tick where it left its domain,
/// and its rest noted for the save.
fn step_one(vehicle: &mut Vehicle, dt: f64, input: &Input, env: &Surroundings, fleet: &mut Fleet) {
    // An anchor that comes to the bed, sets or comes aboard changes what
    // the save keeps: its line.
    let phase = |c: &Craft| {
        c.anchor
            .map(|a| std::mem::discriminant(&a.phase))
            .map(|d| (d, c.anchored()))
    };
    let was = phase(&vehicle.craft);
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
    if was != phase(&vehicle.craft) {
        fleet.dirty = true;
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
const FRESH_FLOW_MAX: f64 = 3.5;
/// A river's own current, m/s, down its course from the drainage map, where
/// its surface is level as well as where it falls: a river flows in its
/// level reaches too (the owner, 2026-11-18, floating a tube: "see if its
/// fast enough, otherwise we can tweak it"; on the slope alone a tube sat
/// still in a level reach).
const RIVER_FLOW_MPS: f64 = 2.0;
/// How hard the current turns a craft toward the river's line, m/s at the
/// river's edge: so a meander carries it round the bend rather than into
/// the bank.
const RIVER_CENTRING_MPS: f64 = 0.8;
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
            .find_map(|q| contact.drawn_water(q.as_vec3()).flatten())
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
    // A river's own flow, down its course, and toward its line.
    let cfg = crate::planet::terrain_config();
    let dir = up.as_vec3();
    if let Some(river) = pbd_core::rivers::at(cfg, dir)
        && river.across_m < river.half_width_m + 1.0
    {
        let down = (-river.upstream).as_dvec3();
        let down = (down - up * down.dot(up)).normalize_or_zero();
        current += down * RIVER_FLOW_MPS;
        // Toward the line: the side a step makes the river nearer.
        let side = up.cross(down).normalize_or_zero();
        let across = |sign: f64| {
            let q = (point + side * sign).normalize().as_vec3();
            pbd_core::rivers::at(cfg, q).map_or(f32::INFINITY, |h| h.across_m)
        };
        let toward = if across(1.0) < across(-1.0) {
            side
        } else {
            -side
        };
        let off = (f64::from(river.across_m) / f64::from(river.half_width_m.max(1.0))).min(1.0);
        current += toward * RIVER_CENTRING_MPS * off;
    }
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
        Kind::Loon | Kind::Raft => {
            input.forward = axis(KeyCode::KeyW, KeyCode::KeyS);
            input.steer = axis(KeyCode::KeyA, KeyCode::KeyD);
            input.rudder = axis(KeyCode::KeyQ, KeyCode::KeyE);
        }
        // The tube's rider kicks: ahead or back, and to turn
        // (`small-boats`).
        Kind::Tube => {
            input.forward = axis(KeyCode::KeyW, KeyCode::KeyS);
            input.steer = axis(KeyCode::KeyA, KeyCode::KeyD);
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
    let at = walker.0.as_dvec3() - frame.physics_center();
    if let Some((_, vehicle)) = nearest_in_reach(vehicles.iter(), at) {
        interact.offer(crate::interact::Offer::new(
            crate::interact::Act::Board,
            vehicle.craft.kind.name(),
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
    // Physics space: the walker's `Position` (`movement-smoothness` 6.1).
    let frame = world
        .resource::<crate::planet::PlanetRenderFrame>()
        .physics_center();
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
    let (name, rowed) = {
        let mut vehicle = world
            .get_mut::<Vehicle>(entity)
            .expect("the craft just found");
        vehicle.craft.board();
        (vehicle.craft.kind.name(), vehicle.craft.rowing().is_some())
    };
    world.resource_mut::<Aboard>().0 = Some(entity);
    let mut look = world.resource_mut::<view::VehicleView>();
    look.aboard(captured);
    // A rower in the seat looks down along the oars to the bow, the hands
    // on the handles in view (`river-craft` design 9.6).
    if rowed && look.seat {
        look.pitch = view::ROWER_PITCH;
    }
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
    /// Where to launch a tube or raft (a unit direction, `--at`) when a river
    /// runs there: on the river's water, facing downstream (the owner,
    /// 2026-11-18: "float an inner tube down a river"). Else by the Loon.
    pub at: Option<Vec3>,
    /// `--pole-run`: the rider poles the raft up the river against its
    /// current, then back down with it, by the craft's own keys (the owner,
    /// 2026-11-18: "the raft guy pushing the raft up and down the river").
    pub pole_run: bool,
    /// `--pitch` in the seat, rad: the look down from level.
    pub pitch: Option<f32>,
}

/// The pole run's script (`--pole-run`, a recording instrument): settle five
/// seconds, turn upstream and pole against the current to 32 s, then turn
/// and pole down with it. It steers toward the river's own course where the
/// raft is (A and D), and poles with W, as a player does.
pub fn pole_run(
    script: Option<Res<VehicleScript>>,
    log: Option<Res<FloatLog>>,
    aboard: Res<Aboard>,
    vehicles: Query<&Vehicle>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut said: Local<u8>,
) {
    if !script.is_some_and(|s| s.pole_run) {
        return;
    }
    let (Some(log), Some(entity)) = (log, aboard.0) else {
        return;
    };
    let Ok(vehicle) = vehicles.get(entity) else {
        return;
    };
    let t = log.started.elapsed().as_secs_f32();
    for key in [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD] {
        keys.release(key);
    }
    if t < 5.0 {
        return;
    }
    let upstream = t < 55.0;
    let phase = if upstream { 1 } else { 2 };
    if *said != phase {
        *said = phase;
        info!(
            "POLE_RUN {t:.0} s: poling {}",
            if upstream {
                "up the river, against its current"
            } else {
                "down the river, with it"
            }
        );
    }
    let at = vehicle.craft.body.position;
    let up = at.normalize();
    let cfg = crate::planet::terrain_config();
    let Some(river) = pbd_core::rivers::at(cfg, up.as_vec3()) else {
        keys.press(KeyCode::KeyW);
        return;
    };
    // Launched facing upstream (a 4.4 m raft turned round in a narrow
    // channel swings broadside and wedges): the rider poles it up the river
    // against the current, steering, then stops and rides the current back
    // down past where it started.
    let along = river.upstream.as_dvec3();
    let target = (along - up * along.dot(up)).normalize_or_zero();
    let forward = vehicle.craft.body.orientation * DVec3::NEG_Z;
    let forward = (forward - up * forward.dot(up)).normalize_or_zero();
    let left = up.cross(forward);
    let side = target.dot(left);
    let ahead = target.dot(forward);
    if upstream {
        if side > 0.15 {
            keys.press(KeyCode::KeyA);
        } else if side < -0.15 {
            keys.press(KeyCode::KeyD);
        }
        if ahead > 0.6 {
            keys.press(KeyCode::KeyW);
        }
    }
    if ((t * 10.0) as u32).is_multiple_of(10) {
        let v = vehicle.craft.body.velocity;
        info!(
            "POLE_DEBUG {t:.1} s: ahead {ahead:+.2} side {side:+.2}, speed up the river {:+.2} m/s, keys W {} S {} A {} D {}",
            v.dot(target),
            keys.pressed(KeyCode::KeyW),
            keys.pressed(KeyCode::KeyS),
            keys.pressed(KeyCode::KeyA),
            keys.pressed(KeyCode::KeyD)
        );
    }
}

/// The float log (`river-downhill`'s tube test): once a second while aboard a
/// scripted tube or raft, how far it has come from its launch and how fast it
/// goes, with no input, carried by the river alone.
#[derive(Resource)]
pub struct FloatLog {
    from: DVec3,
    started: std::time::Instant,
    next: std::time::Instant,
    travelled_m: f64,
    last: DVec3,
}

pub fn float_log(log: Option<ResMut<FloatLog>>, aboard: Res<Aboard>, vehicles: Query<&Vehicle>) {
    let Some(mut log) = log else {
        return;
    };
    let Some(entity) = aboard.0 else {
        return;
    };
    let Ok(vehicle) = vehicles.get(entity) else {
        return;
    };
    let at = vehicle.craft.body.position;
    let step = at.distance(log.last);
    if step < 50.0 {
        log.travelled_m += step;
    }
    log.last = at;
    let now = std::time::Instant::now();
    if now < log.next {
        return;
    }
    log.next = now + std::time::Duration::from_secs(1);
    let up = at.normalize().as_vec3();
    let speed = vehicle.craft.body.velocity;
    let speed = (speed - at.normalize() * speed.dot(at.normalize())).length();
    info!(
        "TUBE {:.0} s: {:.0} m along its path, {:.0} m from its launch, {speed:.1} m/s, {} | {} m elevation",
        now.duration_since(log.started).as_secs_f32(),
        log.travelled_m,
        up.angle_between(log.from.normalize().as_vec3()) * crate::planet::PLANET_RADIUS,
        crate::place::Place::of(up * crate::planet::PLANET_RADIUS).line(),
        (at.length() - crate::planet::PLANET_RADIUS as f64).round()
    );
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
    // On the river at `--at`, facing downstream: what a player floating a
    // river would do.
    let found = found.or_else(|| {
        if !matches!(kind, Kind::Tube | Kind::Raft) {
            return None;
        }
        let at = script.at?;
        let cfg = crate::planet::terrain_config();
        let top = pbd_core::rivers::water(cfg, at).last()?.0 as f64;
        // Facing downstream to float; upstream for a pole run, which poles
        // up the river first.
        let down = pbd_core::rivers::at(cfg, at).map_or(Vec3::ZERO, |h| {
            if script.pole_run {
                h.upstream
            } else {
                -h.upstream
            }
        });
        // On the surface the craft floats on (`fresh_water`, the tier's own
        // water, the flowing water over the last whole layer included), not
        // the generator's last layer: launched there it started 1.6 m under
        // and took seconds to come up.
        let layer = at.as_dvec3() * (crate::planet::PLANET_RADIUS as f64 + top);
        let surface = world
            .get_resource::<PlanetContact>()
            .and_then(|c| fresh_water(c, layer + at.as_dvec3() * 2.0))
            .map_or(crate::planet::PLANET_RADIUS as f64 + top + 1.0, |w| {
                w.surface
            });
        let on = at.as_dvec3() * surface;
        let toward = (down.as_dvec3()).normalize_or(on.any_orthonormal_vector());
        let craft = launch::made_craft(&mut world.resource_mut::<Fleet>(), kind, on, toward);
        let entity = place::spawn_craft(world, craft);
        launch::write_fleet(world);
        let now = std::time::Instant::now();
        world.insert_resource(FloatLog {
            from: on,
            started: now,
            next: now,
            travelled_m: 0.0,
            last: on,
        });
        info!(
            "--aboard: launched a {} on the river at {}, its water {top:.0} m",
            kind.name(),
            crate::place::Place::of(at * crate::planet::PLANET_RADIUS).line()
        );
        Some(entity)
    });
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
        let craft = launch::made_craft(&mut world.resource_mut::<Fleet>(), kind, on, toward);
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
    if let Some(pitch) = script.pitch {
        world.resource_mut::<view::VehicleView>().pitch = pitch;
    }
}

/// Step off: the craft stays, unattended; the walker is put at its exit.
pub fn leave(world: &mut World, entity: Entity) {
    // The walker is dropped in physics space.
    let frame = world
        .resource::<crate::planet::PlanetRenderFrame>()
        .physics_center();
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

/// A boat's one anchor (`river-craft` design 4): T or the boat's menu
/// drops it where the water is shallow enough, and weighs it (or stops
/// hauling) when it is out. A boat tied up is cast off; a kind with no
/// anchor (the tube) is only ever cast off.
pub fn make_fast_or_cast_off(world: &mut World, entity: Entity) {
    let Some(vehicle) = world.get::<Vehicle>(entity) else {
        return;
    };
    let craft = &vehicle.craft;
    let anchor_out = craft.anchor.is_some() || craft.mooring.is_some_and(|m| m.anchored);
    if anchor_out && craft.has_anchor() {
        let mut vehicle = world.get_mut::<Vehicle>(entity).expect("checked");
        vehicle.craft.weigh_anchor();
        let word = pbd_core::vehicle::anchor::word(vehicle.craft.anchor.as_ref());
        world.resource_mut::<Fleet>().dirty = true;
        info!("anchor: {word}");
        return;
    }
    if craft.mooring.is_some() {
        world
            .get_mut::<Vehicle>(entity)
            .expect("checked")
            .craft
            .mooring = None;
        world.resource_mut::<Fleet>().dirty = true;
        info!("cast off");
        return;
    }
    if !craft.has_anchor() {
        return;
    }
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
    // Let go: it falls to the bed and the rode stops there (design 4).
    world
        .get_mut::<Vehicle>(entity)
        .expect("checked")
        .craft
        .drop_anchor();
    world.resource_mut::<Fleet>().dirty = true;
    info!("anchor dropped in {depth:.1} m");
}

/// What the boat's menu asks of a craft's anchor (`river-craft` design
/// 10.3), done here where the water's depth is known: drop or weigh it.
#[derive(Resource, Default, Debug)]
pub struct AnchorOrder(pub Option<Entity>);

/// Carry out the boat's menu's anchor order.
fn anchor_order(world: &mut World) {
    let Some(entity) = world
        .get_resource_mut::<AnchorOrder>()
        .and_then(|mut o| o.0.take())
    else {
        return;
    };
    make_fast_or_cast_off(world, entity);
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
