//! The hand cart and the horse cart in the app (`land-transport` design 9
//! to 11): set down from the pack, pulled on foot or driven from the seat,
//! slowed off the road, hitched to a bought horse, and kept as vehicle
//! records on the durable path.
//!
//! What is reused rather than written again: the fleet's id counter (so a
//! cart's hold is the boats' `hold` record and the pack's HOLD panel), the
//! boats' ground query (`ground_under`), the place button and the held dig
//! button the made boats use, the interact key and its prompt, the farmyard
//! for the horse when it is out of the shafts, and the walker for the
//! player, its camera and its raw mouse look.
//!
//! Positions: a cart is body-local `f64` in the planet's frame (`Cart::at`),
//! as a boat is; the walker's physics `Position` is converted at the
//! boundary in `f64`, and the farmyard's local feet through the one
//! `SimOrigin`.

pub mod model;

use super::{Fleet, ground_under};
use crate::field_light::{LitByField, LitLikeTerrain};
use crate::hotbar::Hotbar;
use crate::husbandry::Farmyard;
use crate::interact::{Act, Interact, Offer, Reader};
use crate::missions::Did;
use crate::planet::{PlanetContact, PlanetRenderFrame};
use crate::saves::WorldSave;
use crate::walking::{HALF_HEIGHT, Pace, Walker, WalkingCamera, WalkingState};
use avian3d::prelude::{LinearVelocity, Position};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::goods::Good;
use pbd_core::husbandry::record::{self as animal, AnimalBody};
use pbd_core::inventory::Item;
use pbd_core::locale::{tr, tr_with};
use pbd_core::missions::Kind as MissionKind;
use pbd_core::records::Author;
use pbd_core::roads::{self, WheelGround};
use pbd_core::vehicle::cart::{self, CART_KIND, CART_SCHEMA, Cart, CartKind, CartSpec, Drive};

/// How far along the look a cart can be set down, m.
const SET_DOWN_M: f64 = 6.0;
/// How near the player must stand to a cart's handles, seat or shafts to
/// use them, m, and to its middle to take it up.
const REACH_M: f64 = 1.8;
const TAKE_UP_REACH_M: f64 = 3.5;
/// How long the dig button is held on an empty cart to take it up, s (the
/// made boats' own).
const TAKE_UP_S: f32 = 1.0;
/// How near a horse must stand to the shafts to be hitched, m.
const HITCH_M: f32 = 6.0;
/// How long an unhitched or bought horse follows, s.
const FOLLOW_S: f32 = 60.0;
/// A cart that has rolled and then stood this long is written, s.
const REST_S: f32 = 0.5;
/// The seat: up from the ground and forward of the axle, m.
const SEAT_UP_M: f64 = 1.45;
const SEAT_FORWARD_M: f64 = 0.7;
/// How far over a point its ground is asked from, m: over a layer.
const PROBE_UP_M: f64 = 1.3;
/// The horse in the shafts stands this far ahead of the axle, m.
const HORSE_AHEAD_M: f64 = 2.75;

/// Up at a body-local point, in f64 before any cast.
fn up_of(p: DVec3) -> DVec3 {
    p.normalize_or(DVec3::Y)
}

/// A cart in the world.
#[derive(Component)]
pub struct CartThing {
    pub cart: Cart,
    mesh: Option<Handle<Mesh>>,
    horse: Option<(Entity, Handle<Mesh>)>,
    /// What the mesh was built for: rebuilt only when it changes.
    drawn: Option<(i32, i32, bool)>,
    /// Seconds still since it last moved; `None` once written at rest.
    still_s: Option<f32>,
    /// Metres hauled with something in the hold, and driven on road cells,
    /// not yet told to the missions.
    hauled_m: f64,
    road_m: f64,
}

impl CartThing {
    fn new(cart: Cart) -> Self {
        Self {
            cart,
            mesh: None,
            horse: None,
            drawn: None,
            still_s: None,
            hauled_m: 0.0,
            road_m: 0.0,
        }
    }
}

/// Who is with the carts: the hand cart whose handles the player holds, the
/// horse cart whose seat the player is on. Not saved: a player who quits
/// with a cart is on foot beside it on reload, and the cart is where it was.
#[derive(Resource, Default)]
pub struct Carts {
    /// This world's carts are in it.
    pub loaded: Option<u64>,
    pub holding: Option<Entity>,
    pub seated: Option<Entity>,
    material: Option<Handle<StandardMaterial>>,
    /// The take-up in hand: which cart, and how long the button has been
    /// held on it.
    taking_up: Option<(Entity, f32)>,
    /// A capture script's request (`--carts`).
    pub script: Option<CartScript>,
}

/// `--carts`: set a hand cart and a hitched horse cart down ahead of the
/// walker, and take the one named.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CartScript {
    pub pull: bool,
    pub drive: bool,
    pub done: bool,
    /// `--cart-close hand|horse`: only that cart, broadside across the
    /// view a few metres ahead, to look at close up.
    pub close: Option<CartKind>,
}

pub struct CartsPlugin;

impl Plugin for CartsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Carts>()
            .init_resource::<HorseOrders>()
            .add_systems(
                Update,
                (load, script, use_carts, set_down_and_take_up, sell_horses)
                    .chain()
                    .after(super::place::spawn_fleet)
                    .run_if(crate::world_open),
            )
            .add_systems(FixedUpdate, roll)
            .add_systems(
                PostUpdate,
                draw.after(crate::planet::update_planet_frame)
                    .before(bevy::transform::TransformSystems::Propagate)
                    .run_if(crate::world_open),
            )
            .add_systems(
                PostUpdate,
                offer
                    .in_set(crate::interact::InteractSet::Offer)
                    .run_if(resource_exists::<Interact>),
            );
    }
}

/// The specs of a cart.
fn spec_of(world: &World, kind: CartKind) -> CartSpec {
    *world.resource::<Fleet>().specs.carts.of(kind)
}

/// The walker's feet, body-local in `f64`, and its entity.
fn walker_feet(world: &mut World) -> Option<(Entity, DVec3)> {
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    world
        .query_filtered::<(Entity, &Position), With<Walker>>()
        .iter(world)
        .next()
        .map(|(e, p)| {
            let body = p.0.as_dvec3() - centre;
            let up = body.normalize_or(DVec3::Y);
            (e, body - up * f64::from(HALF_HEIGHT))
        })
}

/// The ground a wheel stands on at a body-local point: the terrain, or a
/// road piece's floor over it (a raised cell, a bridge's deck).
fn ground_at(world: &World, point: DVec3) -> f64 {
    // Asked from a little over a layer up, so the top of a block a step
    // higher is found (and stops the wheel) rather than the floor under it.
    let point = point + up_of(point) * PROBE_UP_M;
    let contact = world.get_resource::<PlanetContact>();
    let terrain = ground_under(contact, point);
    let Some(pieces) = world.get_resource::<crate::roads::RoadPieces>() else {
        return terrain;
    };
    let near = point.as_vec3();
    let mut best = terrain;
    for solids in &pieces.solids {
        let p = solids.frame.local(near);
        if Vec2::new(p.x, p.z).length() > solids.reach_m {
            continue;
        }
        if let (Some(floor), _) = solids.stand(near, PROBE_UP_M as f32 + 0.7) {
            best = best.max(f64::from(floor));
        }
    }
    best
}

/// What a wheel at a body-local point stands on (design 9): the road cell
/// there, else a town's lanes, else off the road.
fn wheel_ground(world: &World, point: DVec3) -> WheelGround {
    let direction = point.normalize_or(DVec3::Y).as_vec3();
    if let (Some(net), Some(key)) = (
        world.get_resource::<crate::roads::RoadNet>(),
        roads::nearest_key(direction),
    ) && let Some(laid) = net.cells.iter().find(|l| l.cell.key == key)
    {
        return roads::wheel_ground(laid.cell.kind, laid.surface);
    }
    if pbd_core::settlement::ground::cleared(crate::planet::terrain_config(), direction) {
        return WheelGround::Town;
    }
    WheelGround::Off
}

/// The factor a cart runs at where it stands: the lower of its two wheels'
/// (`roads::wheel_factor`).
fn factor_of(world: &World, c: &Cart, spec: &CartSpec) -> (f32, bool) {
    let under = c.wheels(spec).map(|w| wheel_ground(world, w));
    let factors = world
        .get_resource::<crate::roads::RoadsSettings>()
        .map(|s| s.0.wheel)
        .unwrap_or_default();
    let on_road = under.iter().all(|g| *g != WheelGround::Off);
    (roads::wheel_factor(&factors, &under), on_road)
}

/// The player's free hand: none while a cart is held or the seat is taken.
fn busy(world: &World) -> bool {
    let c = world.resource::<Carts>();
    c.holding.is_some() || c.seated.is_some()
}

fn say(world: &mut World, text: &str) {
    if let Some(mut status) = world.get_resource_mut::<crate::fish::FishingStatus>() {
        status.say(text, 2.5);
    }
    info!("carts: {text}");
}

/// The horse species' index in the home roster.
fn horse_species(world: &World) -> Option<u16> {
    let land = world.get_resource::<crate::config::HusbandryConfig>()?;
    land.0
        .roster(pbd_core::fauna::HOME_BODY)
        .iter()
        .position(|s| s.id == "horse")
        .map(|i| i as u16)
}

// ---- Loading --------------------------------------------------------------

/// This world's carts, once its fleet is in: every cart record the save
/// holds, and the fleet's counter raised over every cart id.
fn load(world: &mut World) {
    let spawned = world.get_resource::<Fleet>().is_some_and(|f| f.spawned);
    let serial = world.get_resource::<WorldSave>().map(|s| s.serial());
    let loaded = world.resource::<Carts>().loaded;
    if !spawned || serial.is_none() || loaded == serial {
        return;
    }
    // A world opened again, or another: what stood is cleared first.
    let old: Vec<Entity> = world
        .query_filtered::<Entity, With<CartThing>>()
        .iter(world)
        .collect();
    for e in old {
        world.entity_mut(e).despawn();
    }
    {
        let mut c = world.resource_mut::<Carts>();
        c.loaded = serial;
        c.holding = None;
        c.seated = None;
    }
    let (carts, highest) = {
        let save = world.resource::<WorldSave>();
        (cart::carts(&save.records), cart::highest_id(&save.records))
    };
    if let Some(highest) = highest {
        let mut fleet = world.resource_mut::<Fleet>();
        if fleet.next_id <= highest {
            fleet.next_id = highest + 1;
            fleet.dirty = true;
        }
    }
    let n = carts.len();
    for c in carts {
        spawn(world, c);
    }
    if n > 0 {
        info!("carts: {n} back where they were left");
    }
}

/// Put a cart in the world: its root lit as the terrain is, its mesh built
/// on the first draw.
fn spawn(world: &mut World, c: Cart) -> Entity {
    if world.resource::<Carts>().material.is_none() {
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.92,
                reflectance: 0.2,
                ..default()
            });
        world.resource_mut::<Carts>().material = Some(material);
    }
    let centre = world.resource::<PlanetRenderFrame>().center;
    world
        .spawn((
            Name::new(c.kind.name().into_string()),
            Transform::from_translation((centre + c.at).as_vec3()),
            Visibility::default(),
            LitByField,
            LitLikeTerrain,
            CartThing::new(c),
        ))
        .id()
}

/// Write a cart's record as the player's. `false` if the save refused it.
fn write(world: &mut World, entity: Entity) -> bool {
    let Some(record) = world.get::<CartThing>(entity).map(|t| t.cart.record()) else {
        return false;
    };
    let Some(mut save) = world.get_resource_mut::<WorldSave>() else {
        return true;
    };
    save.note_record_kinds(&[(CART_KIND, CART_SCHEMA)]);
    save.store(&Author::Player(0), vec![record]).is_some()
}

// ---- The interact key ------------------------------------------------------

/// What F would do with the carts now, and to which.
fn what_f_does(world: &mut World) -> Option<(Act, Entity, pbd_core::locale::Localized)> {
    let c = world.resource::<Carts>();
    if let Some(e) = c.holding {
        let name = world.get::<CartThing>(e)?.cart.kind.name();
        return Some((Act::LetGo, e, name));
    }
    if let Some(e) = c.seated {
        let name = world.get::<CartThing>(e)?.cart.kind.name();
        return Some((Act::StepDown, e, name));
    }
    let (_, feet) = walker_feet(world)?;
    let horse_sp = horse_species(world);
    let specs = world.resource::<Fleet>().specs.carts;
    let mut best: Option<(f64, Act, Entity, pbd_core::locale::Localized)> = None;
    let mut found = world.query::<(Entity, &CartThing)>();
    let things: Vec<(Entity, Cart)> = found
        .iter(world)
        .map(|(e, t)| (e, t.cart.clone()))
        .collect();
    for (e, c) in things {
        let spec = specs.of(c.kind);
        let front = c.front(spec);
        let d = front.distance(feet);
        let (act, thing) = match c.kind {
            CartKind::HandCart if d <= REACH_M => (Act::Pull, c.kind.name()),
            CartKind::HorseCart => {
                let seat = c.at + c.heading * SEAT_FORWARD_M;
                let to_seat = seat.distance(feet);
                if to_seat <= REACH_M + 0.4 {
                    (Act::Drive, c.kind.name())
                } else if d <= REACH_M + 1.2 {
                    match (c.horse, horse_sp) {
                        (Some(_), _) => (Act::Unhitch, horse_name(world)),
                        (None, Some(sp)) => {
                            if led_horse(world, sp, front).is_none() {
                                continue;
                            }
                            (Act::Hitch, horse_name(world))
                        }
                        (None, None) => continue,
                    }
                } else {
                    continue;
                }
            }
            _ => continue,
        };
        if best.as_ref().is_none_or(|b| d < b.0) {
            best = Some((d, act, e, thing));
        }
    }
    best.map(|(_, a, e, t)| (a, e, t))
}

fn horse_name(world: &World) -> pbd_core::locale::Localized {
    let _ = world;
    tr("species.horse.name")
}

/// The nearest kept horse to a body-local point, within reach of the
/// shafts: its record id and its local feet.
fn led_horse(world: &World, species: u16, near: DVec3) -> Option<(u64, Vec3)> {
    let o = crate::sim_origin::origin_of(world);
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    // Body-local to the walker's physics space, then to the origin's local.
    let physics = (centre + near).as_vec3();
    let local = o.from_query_point(o.query_point(physics));
    let yard = world.get_resource::<Farmyard>()?;
    let hitched = world
        .get_resource::<WorldSave>()
        .map(|s| cart::hitched(&s.records))
        .unwrap_or_default();
    yard.kept_near(species, local, HITCH_M)
        .into_iter()
        .find(|(id, _)| !hitched.contains(id))
}

fn offer(world: &mut World) {
    let walking = world
        .get_resource::<WalkingState>()
        .is_some_and(|w| w.active);
    if !walking || world.resource::<Carts>().loaded.is_none() {
        return;
    }
    if let Some((act, _, thing)) = what_f_does(world) {
        world
            .resource_mut::<Interact>()
            .offer(Offer::new(act, thing));
    }
}

/// F with the carts: take or let go of the handles, take or leave the seat,
/// hitch or unhitch the horse.
fn use_carts(world: &mut World) {
    if world
        .get_resource::<crate::controls::MenuOpen>()
        .is_some_and(|m| m.0)
    {
        return;
    }
    let key = world
        .get_resource::<crate::controls::KeyBinds>()
        .map_or(KeyCode::KeyF, crate::interact::key);
    let pressed = world
        .get_resource::<ButtonInput<KeyCode>>()
        .is_some_and(|k| k.just_pressed(key));
    if !pressed || !crate::interact::may(world.get_resource::<Interact>(), Reader::Carts) {
        return;
    }
    let Some((act, entity, _)) = what_f_does(world) else {
        return;
    };
    match act {
        Act::Pull => take_handles(world, entity),
        Act::LetGo => let_go(world, entity),
        Act::Drive => take_seat(world, entity),
        Act::StepDown => step_down(world, entity),
        Act::Hitch => hitch(world, entity),
        Act::Unhitch => unhitch(world, entity),
        _ => {}
    }
}

/// Take a hand cart's handles: the walker's pace held to the cart's.
pub fn take_handles(world: &mut World, entity: Entity) {
    world.resource_mut::<Carts>().holding = Some(entity);
    if let Some(mut t) = world.get_mut::<CartThing>(entity) {
        t.still_s = Some(0.0);
    }
    info!("carts: handles taken");
}

/// Let go: the cart stands where it is with its hold, in the save.
pub fn let_go(world: &mut World, entity: Entity) {
    world.resource_mut::<Carts>().holding = None;
    world.resource_mut::<Pace>().cap_mps = None;
    if let Some(mut t) = world.get_mut::<CartThing>(entity) {
        t.cart.speed = 0.0;
    }
    if !write(world, entity) {
        say(world, &tr("vehicles.cart.unsaved"));
    }
    info!("carts: let go");
}

/// Take a horse cart's seat: the walker is carried on it, its own camera
/// and look kept, and the move keys drive.
pub fn take_seat(world: &mut World, entity: Entity) {
    world.resource_mut::<Carts>().seated = Some(entity);
    world.resource_mut::<Pace>().cap_mps = Some(0.0);
    if let Some(mut t) = world.get_mut::<CartThing>(entity) {
        t.still_s = Some(0.0);
    }
    info!("carts: on the seat");
}

/// Step down beside the seat: the cart stays, with its horse and hold.
pub fn step_down(world: &mut World, entity: Entity) {
    world.resource_mut::<Carts>().seated = None;
    world.resource_mut::<Pace>().cap_mps = None;
    let Some(c) = world.get::<CartThing>(entity).map(|t| t.cart.clone()) else {
        return;
    };
    // Beside the seat, on the left, on the ground.
    let side = c.at + c.heading * SEAT_FORWARD_M - c.right() * 1.4;
    let ground = ground_at(world, side);
    let up = side.normalize_or(DVec3::Y);
    let stood = up * (ground + f64::from(HALF_HEIGHT) + 0.05);
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    if let Some((walker, _)) = walker_feet(world)
        && let Some(mut p) = world.get_mut::<Position>(walker)
    {
        p.0 = (centre + stood).as_vec3();
    }
    if let Some(mut t) = world.get_mut::<CartThing>(entity) {
        t.cart.speed = 0.0;
    }
    if !write(world, entity) {
        say(world, &tr("vehicles.cart.unsaved"));
    }
    info!("carts: stepped down");
}

/// Hitch the led horse at the shafts. One record is written, the cart's,
/// naming the horse: hitched is that record alone, so the save can never
/// hold a half-hitched horse (design 11).
pub fn hitch(world: &mut World, entity: Entity) {
    let Some(sp) = horse_species(world) else {
        return;
    };
    let Some(front) = world
        .get::<CartThing>(entity)
        .map(|t| t.cart.front(&spec_of(world, t.cart.kind)))
    else {
        return;
    };
    let Some((id, _)) = led_horse(world, sp, front) else {
        say(world, &tr("vehicles.cart.no_horse"));
        return;
    };
    hitch_horse(world, entity, id);
}

/// Put horse `id` in the shafts of the cart at `entity`, and say so.
pub fn hitch_horse(world: &mut World, entity: Entity, id: u64) -> bool {
    if let Some(mut t) = world.get_mut::<CartThing>(entity) {
        t.cart.horse = Some(id);
        t.drawn = None;
    }
    if !write(world, entity) {
        if let Some(mut t) = world.get_mut::<CartThing>(entity) {
            t.cart.horse = None;
        }
        say(world, &tr("vehicles.cart.unsaved"));
        return false;
    }
    if let Some(mut yard) = world.get_resource_mut::<Farmyard>() {
        yard.hand_over(id);
    }
    if let Some(happened) = missions::hitched() {
        world.write_message(Did(happened));
    }
    let name = horse_name(world);
    say(
        world,
        &tr_with("vehicles.cart.hitched", &[("horse", &name)]),
    );
    true
}

/// Take the horse out of the shafts: its record's home becomes the cell
/// beside the cart, and it follows. The animal is never ended or dropped.
pub fn unhitch(world: &mut World, entity: Entity) {
    let Some(c) = world.get::<CartThing>(entity).map(|t| t.cart.clone()) else {
        return;
    };
    let Some(id) = c.horse else {
        return;
    };
    let at = c.at + c.heading * HORSE_AHEAD_M - c.right() * 1.2;
    let up = at.normalize_or(DVec3::Y);
    let stood = up * ground_at(world, at);
    let record = world
        .get_resource::<WorldSave>()
        .and_then(|s| s.records.get(animal::ANIMAL, id))
        .and_then(|r| r.read::<AnimalBody>());
    let Some(mut body) = record else {
        return;
    };
    if let Some(key) = roads::nearest_key(up.as_vec3()) {
        body.home = key;
        body.pen = None;
    }
    // The horse's record first, then the cart's: a crash between them
    // leaves the horse in the shafts where it was, never lost.
    if let Some(mut save) = world.get_resource_mut::<WorldSave>()
        && save
            .store(&Author::Player(0), vec![body.record(id)])
            .is_none()
    {
        say(world, &tr("vehicles.cart.unsaved"));
        return;
    }
    if let Some(mut t) = world.get_mut::<CartThing>(entity) {
        t.cart.horse = None;
        t.drawn = None;
    }
    write(world, entity);
    // Back into the yard at once, following, from where it stood.
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    let o = crate::sim_origin::origin_of(world);
    let local = o.from_query_point(o.query_point((centre + stood).as_vec3()));
    let facing = c.heading.as_vec3();
    if let Some(mut yard) = world.get_resource_mut::<Farmyard>() {
        yard.take_back(id, &body, local, up.as_vec3(), facing, FOLLOW_S);
    }
    if let Some((e, _)) = world
        .get_mut::<CartThing>(entity)
        .and_then(|mut t| t.horse.take())
    {
        world.entity_mut(e).despawn();
    }
    let name = horse_name(world);
    say(
        world,
        &tr_with("vehicles.cart.unhitched", &[("horse", &name)]),
    );
}

// ---- Setting down and taking up ---------------------------------------------

/// The cart good in the selected slot, if any.
fn held_cart(world: &World) -> Option<CartKind> {
    match world.get_resource::<Hotbar>()?.0.held()?.item {
        Item::Good(good) => CartKind::of_code(good.code()),
        _ => None,
    }
}

/// The eye and the look, body-local, of the walking camera.
fn eye(world: &mut World) -> Option<(DVec3, DVec3)> {
    let centre = world.resource::<PlanetRenderFrame>().center;
    world
        .query_filtered::<&GlobalTransform, With<WalkingCamera>>()
        .iter(world)
        .next()
        .map(|t| {
            (
                t.translation().as_dvec3() - centre,
                t.forward().as_vec3().as_dvec3(),
            )
        })
}

/// The place button with a cart good sets it down on dry ground ahead; the
/// dig button held on an empty, unhitched cart takes it up.
fn set_down_and_take_up(world: &mut World) {
    let on_foot = world
        .get_resource::<WalkingState>()
        .is_some_and(|w| w.active && (w.captured || w.scripted));
    let menu = world
        .get_resource::<crate::controls::MenuOpen>()
        .is_some_and(|m| m.0);
    if !on_foot || menu || busy(world) || world.resource::<Carts>().loaded.is_none() {
        world.resource_mut::<Carts>().taking_up = None;
        return;
    }
    let Some(buttons) = world.get_resource::<ButtonInput<MouseButton>>() else {
        return;
    };
    let place = buttons.just_pressed(MouseButton::Right);
    let dig = buttons.pressed(MouseButton::Left);
    if place && let Some(kind) = held_cart(world) {
        let Some((eye, look)) = eye(world) else {
            return;
        };
        match ground_along(world, eye, look) {
            Some(on) => {
                set_down(world, kind, on, look);
            }
            None => say(world, &tr("vehicles.cart.no_room")),
        }
    }
    if dig {
        let dt = world.resource::<Time>().delta_secs();
        take_up(world, dt);
    } else {
        world.resource_mut::<Carts>().taking_up = None;
    }
}

/// The dry ground a look meets within reach, body-local, or `None` (water,
/// or nothing in reach).
fn ground_along(world: &World, eye: DVec3, look: DVec3) -> Option<DVec3> {
    let contact = world.get_resource::<PlanetContact>();
    let sea = world
        .get_resource::<crate::sea::Sea>()
        .map(|s| f64::from(s.radius));
    let steps = (SET_DOWN_M / 0.2) as usize;
    for k in 1..=steps {
        let p = eye + look * (k as f64 * 0.2);
        let floor = ground_at(world, p);
        if p.length() <= floor {
            let up = p.normalize();
            let wet = sea.is_some_and(|s| floor < s + 0.05)
                || contact.is_some_and(|c| super::fresh_water(c, up * (floor + 0.3)).is_some());
            return (!wet).then_some(up * floor);
        }
    }
    None
}

/// A new cart of `kind` at `on`, facing along `look`, written with the
/// slots that no longer hold its good in one line.
pub fn set_down(world: &mut World, kind: CartKind, on: DVec3, look: DVec3) -> Option<Entity> {
    let id = {
        let mut fleet = world.resource_mut::<Fleet>();
        let id = fleet.next_id;
        fleet.next_id += 1;
        fleet.dirty = true;
        id
    };
    let c = Cart::new(id, kind, on, look);
    let good = Good::by_code(kind.code())?;
    let mut slots = world.resource::<Hotbar>().0.clone();
    if !slots.take_item(Item::Good(good), 1) {
        return None;
    }
    let stored = world
        .get_resource_mut::<WorldSave>()
        .is_none_or(|mut save| {
            let ok = save.store_with_hands(c.record(), CART_SCHEMA, &slots);
            save.note_record_kinds(&[(CART_KIND, CART_SCHEMA)]);
            ok
        });
    if !stored {
        say(world, &tr("vehicles.cart.unsaved"));
        return None;
    }
    world.resource_mut::<Hotbar>().0 = slots;
    let entity = spawn(world, c);
    world.write_message(Did::new(MissionKind::Place, kind.code(), 1));
    say(
        world,
        &tr_with("vehicles.cart.set_down", &[("cart", &kind.name())]),
    );
    Some(entity)
}

fn take_up(world: &mut World, dt: f32) {
    let Some((eye, look)) = eye(world) else {
        return;
    };
    let aimed = {
        let mut q = world.query::<(Entity, &CartThing)>();
        q.iter(world)
            .filter_map(|(e, t)| {
                let middle = t.cart.at + t.cart.up() * 0.6;
                let to = middle - eye;
                let along = to.dot(look);
                let off = (to - look * along).length();
                (along > 0.0 && along <= TAKE_UP_REACH_M && off <= 1.3).then_some((
                    e,
                    t.cart.clone(),
                    along,
                ))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
    };
    let Some((entity, c, _)) = aimed else {
        world.resource_mut::<Carts>().taking_up = None;
        return;
    };
    let held = {
        let mut carts = world.resource_mut::<Carts>();
        let held = match carts.taking_up {
            Some((e, s)) if e == entity => s + dt,
            _ => dt,
        };
        carts.taking_up = Some((entity, held));
        held
    };
    if held < TAKE_UP_S {
        return;
    }
    world.resource_mut::<Carts>().taking_up = None;
    if c.horse.is_some() {
        say(world, &tr("vehicles.cart.unhitch_first"));
        return;
    }
    let spec = spec_of(world, c.kind);
    let full = world
        .get_resource::<WorldSave>()
        .is_some_and(|s| !super::hold::read_sized(s, c.id, spec.hold).is_empty());
    if full {
        say(world, &tr("vehicles.cart.empty_first"));
        return;
    }
    let Some(good) = Good::by_code(c.kind.code()) else {
        return;
    };
    let mut slots = world.resource::<Hotbar>().0.clone();
    if slots.give(Item::Good(good), 1) > 0 {
        say(world, &tr("farm.no_room"));
        return;
    }
    // The good back and the record marked gone in one line.
    let stored = world
        .get_resource_mut::<WorldSave>()
        .is_none_or(|mut s| s.store_with_hands(cart::gone_record(&c), CART_SCHEMA, &slots));
    if !stored {
        say(world, &tr("vehicles.cart.unsaved"));
        return;
    }
    world.resource_mut::<Hotbar>().0 = slots;
    world.entity_mut(entity).despawn();
    say(
        world,
        &tr_with("vehicles.cart.taken_up", &[("cart", &c.kind.name())]),
    );
}

// ---- Rolling ------------------------------------------------------------------

/// Each fixed tick: the cart in hand follows the walker, the cart whose
/// seat is taken is driven by the move keys, and a cart that has come to
/// rest is written.
fn roll(world: &mut World) {
    let dt = world.resource::<Time>().delta_secs_f64();
    if dt <= 0.0 || world.resource::<Carts>().loaded.is_none() {
        return;
    }
    let (holding, seated) = {
        let c = world.resource::<Carts>();
        (c.holding, c.seated)
    };
    if let Some(e) = holding {
        pull(world, e, dt);
    }
    if let Some(e) = seated {
        drive(world, e, dt);
    }
    // A cart that rolled and now stands is written once.
    let mut rested = Vec::new();
    let mut q = world.query::<(Entity, &mut CartThing)>();
    for (e, mut t) in q.iter_mut(world) {
        if t.cart.speed.abs() > 0.02 {
            t.still_s = Some(0.0);
        } else if let Some(s) = t.still_s.as_mut() {
            *s += dt as f32;
            if *s >= REST_S {
                t.still_s = None;
                rested.push(e);
            }
        }
    }
    for e in rested {
        if !write(world, e) {
            warn!("carts: a cart at rest was not saved");
        }
    }
    report(world);
}

/// The hand cart in hand: pulled by its handles to the walker, the walker
/// held to its pace on the ground under its wheels.
fn pull(world: &mut World, entity: Entity, dt: f64) {
    let Some((_, feet)) = walker_feet(world) else {
        return;
    };
    let Some(mut c) = world.get::<CartThing>(entity).map(|t| t.cart.clone()) else {
        world.resource_mut::<Carts>().holding = None;
        return;
    };
    let spec = spec_of(world, c.kind);
    let (factor, on_road) = factor_of(world, &c, &spec);
    let top = f64::from(spec.road_mps * factor);
    // The walker walks between the handles: a little ahead of them.
    let ground = |p: DVec3| ground_at(world, p);
    let moved = c.pull(&spec, feet, top * 1.5, dt, &ground);
    world.resource_mut::<Pace>().cap_mps = Some(top as f32);
    // A player who walks off from it (a fall, a climb out) lets it go.
    if c.front(&spec).distance(feet) > 4.0 {
        world.resource_mut::<Carts>().holding = None;
        world.resource_mut::<Pace>().cap_mps = None;
    }
    keep(world, entity, c, moved, on_road);
}

/// The horse cart from its seat: the move keys drive it, the walker rides
/// on the seat with the view carried round its turns.
fn drive(world: &mut World, entity: Entity, dt: f64) {
    let Some(mut c) = world.get::<CartThing>(entity).map(|t| t.cart.clone()) else {
        world.resource_mut::<Carts>().seated = None;
        return;
    };
    let spec = spec_of(world, c.kind);
    let axes = world
        .get_resource::<WalkingState>()
        .map_or(Vec2::ZERO, |w| w.move_axes());
    let input = if c.horse.is_some() {
        Drive {
            forward: axes.y,
            turn: -axes.x,
        }
    } else {
        Drive::default()
    };
    let (factor, on_road) = factor_of(world, &c, &spec);
    let before = c.heading;
    let ground = |p: DVec3| ground_at(world, p);
    let moved = c.drive(&spec, input, factor, dt, &ground);
    // The turn the seat took, about up.
    let up = c.up();
    let turned = before.cross(c.heading).dot(up).asin();
    if let Some(mut w) = world.get_resource_mut::<WalkingState>() {
        w.turn_with(turned as f32);
    }
    // The rider sits on the seat.
    let seat = c.at + c.heading * SEAT_FORWARD_M + up * (SEAT_UP_M + f64::from(HALF_HEIGHT) - 0.6);
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    if let Some((walker, _)) = walker_feet(world) {
        if let Some(mut p) = world.get_mut::<Position>(walker) {
            p.0 = (centre + seat).as_vec3();
        }
        if let Some(mut v) = world.get_mut::<LinearVelocity>(walker) {
            v.0 = Vec3::ZERO;
        }
    }
    keep(world, entity, c, moved, on_road);
}

/// Store a cart's new state and count what it did for the missions.
fn keep(world: &mut World, entity: Entity, c: Cart, moved: f64, on_road: bool) {
    let spec = spec_of(world, c.kind);
    let loaded = world
        .get_resource::<WorldSave>()
        .is_some_and(|s| !super::hold::read_sized(s, c.id, spec.hold).is_empty());
    if let Some(mut t) = world.get_mut::<CartThing>(entity) {
        t.cart = c;
        if loaded {
            t.hauled_m += moved;
        }
        if on_road {
            t.road_m += moved;
        }
    }
}

/// Whole metres hauled and driven on the road, to the missions.
fn report(world: &mut World) {
    let mut hauled = 0u32;
    let mut road = 0u32;
    let mut q = world.query::<&mut CartThing>();
    for mut t in q.iter_mut(world) {
        let h = t.hauled_m.floor();
        if h >= 1.0 {
            hauled += h as u32;
            t.hauled_m -= h;
        }
        let r = t.road_m.floor();
        if r >= 1.0 {
            road += r as u32;
            t.road_m -= r;
        }
    }
    if road > 0 {
        world.write_message(Did::new(MissionKind::Travel, missions::CART_ROAD, road));
    }
    if hauled > 0
        && let Some(happened) = missions::hauled(hauled)
    {
        world.write_message(Did(happened));
    }
}

/// The town tree's cart events (`town-missions-tree` design 4): `Travel`
/// with `cart-road`, `Place` with a cart's code, `Buy` with `<site>:horse`,
/// `haul` (`cart`) and `hitch` (`horse`).
pub mod missions {
    use pbd_core::missions::{Happened, Kind};

    /// What a cart driven on road cells counts under `Travel`.
    pub const CART_ROAD: &str = "cart-road";

    /// Metres hauled by a cart with something in its hold: `haul`
    /// (`cart`).
    pub fn hauled(metres: u32) -> Option<Happened> {
        (metres > 0).then(|| Happened::new(Kind::Haul, "cart", metres))
    }

    /// A horse put in a horse cart's shafts: `hitch` (`horse`).
    pub fn hitched() -> Option<Happened> {
        Some(Happened::new(Kind::Hitch, "horse", 1))
    }
}

// ---- Drawing -----------------------------------------------------------------

/// Each frame: every cart placed where it stands, its mesh rebuilt as its
/// wheels turn, and the horse in its shafts walked by the distance rolled.
fn draw(
    mut commands: Commands,
    frame: Res<PlanetRenderFrame>,
    fleet: Option<Res<Fleet>>,
    carts: Res<Carts>,
    save: Option<Res<WorldSave>>,
    land: Option<Res<crate::config::HusbandryConfig>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut things: Query<(Entity, &mut CartThing, &mut Transform)>,
) {
    let (Some(fleet), Some(material)) = (fleet, carts.material.clone()) else {
        return;
    };
    let roster = land
        .as_deref()
        .map(|l| l.0.roster(pbd_core::fauna::HOME_BODY).to_vec())
        .unwrap_or_default();
    for (entity, mut t, mut transform) in &mut things {
        let c = t.cart.clone();
        let spec = *fleet.specs.carts.of(c.kind);
        let up = c.up();
        transform.translation = (frame.center + c.at).as_vec3();
        transform.rotation = super::place::facing(up, c.heading).as_quat();
        // A hand cart let go rests on its handles' ends.
        let held = carts.holding == Some(entity);
        let tilt = if c.kind == CartKind::HandCart && !held {
            model::rest_tilt(&spec)
        } else {
            0.0
        };
        let key = (
            (c.wheel_turn * 40.0) as i32,
            (tilt * 100.0) as i32,
            c.horse.is_some(),
        );
        if t.drawn != Some(key) {
            t.drawn = Some(key);
            let mut b = crate::husbandry::rig::Builder::default();
            model::cart(
                &mut b,
                c.kind,
                &spec,
                model::CartPose {
                    wheel_turn: c.wheel_turn as f32,
                    tilt,
                },
            );
            match &t.mesh {
                Some(h) => {
                    if let Some(mesh) = meshes.get_mut(h) {
                        b.write(mesh);
                    }
                }
                None => {
                    let mut mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::default(),
                    );
                    b.write(&mut mesh);
                    let handle = meshes.add(mesh);
                    commands.entity(entity).with_child((
                        Mesh3d(handle.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::default(),
                    ));
                    t.mesh = Some(handle);
                }
            }
        }
        // The horse in the shafts: its gait by the distance rolled.
        let Some(id) = c.horse else {
            if let Some((e, _)) = t.horse.take() {
                commands.entity(e).despawn();
            }
            continue;
        };
        let body = save
            .as_deref()
            .and_then(|s| s.records.get(animal::ANIMAL, id))
            .and_then(|r| r.read::<AnimalBody>());
        let species = body.as_ref().and_then(|b| roster.get(b.sp as usize));
        let colours = match (species, body.as_ref()) {
            (Some(sp), Some(b)) => model::colours_of(sp, &b.genome()),
            _ => model::HorseColours::default(),
        };
        let stride = species.map_or(1.7, |s| s.stride_m) as f64;
        let walk = (c.speed.abs() / 1.2).min(1.0) as f32;
        let mut b = crate::husbandry::rig::Builder::default();
        model::horse(
            &mut b,
            &colours,
            model::HorsePose {
                phase: (c.rolled_m / stride * std::f64::consts::TAU) as f32,
                walk,
                head_down: if walk < 0.05 { 0.25 } else { 0.0 },
                tail: 0.0,
            },
            1.0,
        );
        let place = Transform::from_xyz(0.0, 0.0, -(HORSE_AHEAD_M as f32));
        match &t.horse {
            Some((_, h)) => {
                if let Some(mesh) = meshes.get_mut(h) {
                    b.write(mesh);
                }
            }
            None => {
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                b.write(&mut mesh);
                let handle = meshes.add(mesh);
                let child = commands
                    .spawn((
                        Mesh3d(handle.clone()),
                        MeshMaterial3d(material.clone()),
                        place,
                        bevy::camera::visibility::NoFrustumCulling,
                    ))
                    .id();
                commands.entity(entity).add_child(child);
                t.horse = Some((child, handle));
            }
        }
    }
}

// ---- The capture script -------------------------------------------------------

/// `--carts`: once the world's carts are in, a hand cart set down 4 m ahead
/// of the walker and a horse cart 7 m ahead and to the right, with a horse
/// bought into its shafts, by the game's own paths; then the handles or the
/// seat taken if asked.
fn script(world: &mut World) {
    let Some(s) = world.resource::<Carts>().script else {
        return;
    };
    if s.done || world.resource::<Carts>().loaded.is_none() {
        return;
    }
    let Some((_, feet)) = walker_feet(world) else {
        return;
    };
    let Some((_, look)) = eye(world) else {
        return;
    };
    let up = up_of(feet);
    let ahead = (look - up * look.dot(up)).normalize_or(DVec3::X);
    let right = ahead.cross(up);
    let stood = |world: &World, p: DVec3| p.normalize() * ground_at(world, p);
    let (hand_at, horse_at, facing) = match s.close {
        Some(CartKind::HandCart) => (feet + ahead * 3.2, feet + ahead * 1e4, right),
        Some(CartKind::HorseCart) => (feet + ahead * 1e4, feet + ahead * 6.5 - right * 1.4, right),
        None => (
            feet + ahead * 4.0 - right * 1.5,
            feet + ahead * 7.0 + right * 3.5,
            ahead,
        ),
    };
    let (hand_at, horse_at) = (stood(world, hand_at), stood(world, horse_at));
    let want = |k: CartKind| s.close.is_none_or(|c| c == k);
    for kind in CartKind::ALL
        .into_iter()
        .filter(|k| s.close.is_none_or(|c| c == *k))
    {
        if let Some(good) = Good::by_code(kind.code()) {
            world.resource_mut::<Hotbar>().0.give(Item::Good(good), 1);
        }
    }
    let hand = want(CartKind::HandCart)
        .then(|| set_down(world, CartKind::HandCart, hand_at, facing))
        .flatten();
    let horse_cart = want(CartKind::HorseCart)
        .then(|| set_down(world, CartKind::HorseCart, horse_at, facing))
        .flatten();
    if let Some(e) = horse_cart
        && let Some(id) = buy_horse(world, horse_at, facing, false)
    {
        hitch_horse(world, e, id);
    }
    // Close to the horse cart, a second horse stands loose beside it, drawn
    // by the farmyard as any kept animal is.
    if s.close == Some(CartKind::HorseCart) {
        let loose = stood(world, feet + ahead * 4.0 + right * 3.0);
        buy_horse(world, loose, -right, false);
    }
    world.resource_mut::<Carts>().script = Some(CartScript { done: true, ..s });
    if s.pull
        && let Some(e) = hand
    {
        take_handles(world, e);
    }
    if s.drive
        && let Some(e) = horse_cart
    {
        take_seat(world, e);
    }
    info!("carts: the scripted carts are set down");
}

// ---- The horse, bought ----------------------------------------------------------

/// Horses asked for at a town's trader this frame, by the town's site: the
/// trade panel's button (`land-transport` design 10).
#[derive(Resource, Default, Debug)]
pub struct HorseOrders(pub Vec<u32>);

/// Sell each horse asked for: the purse paid and the animal written in one
/// store, and the horse beside the player, following.
fn sell_horses(world: &mut World) {
    let orders = std::mem::take(&mut world.resource_mut::<HorseOrders>().0);
    for site in orders {
        let Some(coins) = world
            .get_resource::<crate::economy::EconomyConfig>()
            .map(|c| c.trade.horse_coins)
        else {
            continue;
        };
        let Some(mut purse) = world
            .get_resource::<crate::economy::Economy>()
            .map(|e| e.purse)
        else {
            continue;
        };
        if purse
            .change(pbd_core::economy::coins::Reason::Bought, coins)
            .is_err()
        {
            say(world, &tr("vehicles.cart.short"));
            continue;
        }
        let Some((_, feet)) = walker_feet(world) else {
            continue;
        };
        let look = eye(world).map_or(DVec3::X, |(_, l)| l);
        let up = up_of(feet);
        let ahead = (look - up * look.dot(up)).normalize_or(DVec3::X);
        let at = feet + ahead * 2.5 - ahead.cross(up) * 1.5;
        let line = pbd_core::records::Record::of(
            pbd_core::economy::coins::PURSE_KIND,
            pbd_core::economy::coins::PURSE_ID,
            pbd_core::economy::coins::PURSE_SCHEMA,
            &purse,
        );
        let Some(_) = buy_horse_with(world, at, -ahead, true, vec![line]) else {
            say(world, &tr("vehicles.cart.unsaved"));
            continue;
        };
        if let Some(mut eco) = world.get_resource_mut::<crate::economy::Economy>() {
            eco.purse = purse;
        }
        world.write_message(Did::new(
            MissionKind::Buy,
            pbd_core::economy::jobs::scoped(site, "horse"),
            1,
        ));
        let name = horse_name(world);
        say(
            world,
            &tr_with(
                "vehicles.cart.bought",
                &[("horse", &name), ("coins", &coins)],
            ),
        );
    }
}

/// A horse bought: a kept-animal record of the horse species, `via:
/// Bought`, at the cell under `at` (body-local), written with the purse by
/// the caller; here the record alone. Its id, or `None`.
pub fn buy_horse(world: &mut World, at: DVec3, facing: DVec3, follow: bool) -> Option<u64> {
    buy_horse_with(world, at, facing, follow, Vec::new())
}

/// [`buy_horse`], with `with` (the purse that paid for it) stored beside
/// the animal's record.
fn buy_horse_with(
    world: &mut World,
    at: DVec3,
    facing: DVec3,
    follow: bool,
    with: Vec<pbd_core::records::Record>,
) -> Option<u64> {
    let sp = horse_species(world)?;
    let now_day = world.get_resource::<crate::sky::Sun>().map_or(0.0, |s| {
        s.clock.seconds / f64::from(pbd_core::daylight::DAY_S)
    });
    let (id, body) = {
        let save = world.get_resource::<WorldSave>()?;
        let id = animal::next_animal_id(&save.records);
        let keep_days = world
            .get_resource::<crate::config::HusbandryConfig>()?
            .0
            .roster(pbd_core::fauna::HOME_BODY)[sp as usize]
            .keep
            .as_ref()
            .map_or(20.0, |k| f64::from(k.adult_days));
        let up = at.normalize_or(DVec3::Y).as_vec3();
        let seed = pbd_core::fauna::hash2(save.seed(), 0x4085e ^ id);
        let genome = pbd_core::husbandry::Genome::roll(seed, 3, 2);
        let body = AnimalBody {
            sp,
            sex: if seed & 1 == 0 {
                pbd_core::husbandry::Sex::F
            } else {
                pbd_core::husbandry::Sex::M
            },
            born: now_day - keep_days,
            genes: genome.0.to_vec(),
            home: roads::nearest_key(up).unwrap_or(0),
            via: animal::Via::Bought,
            as_of: now_day,
            fed_until: now_day + 2.0,
            ..AnimalBody::default()
        };
        (id, body)
    };
    {
        let mut save = world.get_resource_mut::<WorldSave>()?;
        save.note_record_kinds(&[(animal::ANIMAL, animal::ANIMAL_SCHEMA)]);
        let mut lines = vec![body.record(id)];
        lines.extend(with);
        save.store(&Author::Player(0), lines)?;
    }
    if follow {
        let centre = world.resource::<PlanetRenderFrame>().physics_center();
        let o = crate::sim_origin::origin_of(world);
        let stood = at.normalize() * ground_at(world, at);
        let local = o.from_query_point(o.query_point((centre + stood).as_vec3()));
        let up = at.normalize_or(DVec3::Y).as_vec3();
        if let Some(mut yard) = world.get_resource_mut::<Farmyard>() {
            yard.take_back(id, &body, local, up, facing.as_vec3(), FOLLOW_S);
        }
    }
    Some(id)
}

/// The facing a cart's root is drawn turned by, for a test.
#[cfg(test)]
fn facing(c: &Cart) -> pbd_core::DQuat {
    super::place::facing(c.up(), c.heading)
}

#[cfg(test)]
mod tests;
