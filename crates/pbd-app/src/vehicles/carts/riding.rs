//! Riding a kept plodder or the bought horse (`land-transport` design 14).
//!
//! F beside a kept plodder or horse that is in no cart's shafts mounts it:
//! its record is written `ridden`, the farmyard stops drawing it, and it is
//! stepped here as a [`Ride`] (kinematic, body-local `f64`), the walker
//! carried at the rider's eye on its back with its own camera and raw look.
//! F again gets off: the record is written unridden with its home the cell
//! it stands on, and it is back in the yard where it stood. A save closed
//! while riding comes back with the walker on foot and the animal beside
//! them ([`restore`]).

use super::{
    Carts, HALF_HEIGHT, animal_body, animal_name, puller_species, say, species_id, walker_feet,
};
use crate::field_light::{LitByField, LitLikeTerrain};
use crate::husbandry::Farmyard;
use crate::missions::Did;
use crate::planet::PlanetRenderFrame;
use crate::saves::WorldSave;
use crate::walking::{EYE_HEIGHT, Pace, WalkingState};
use avian3d::prelude::{LinearVelocity, Position};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::husbandry::record::{self as animal, AnimalBody};
use pbd_core::records::Author;
use pbd_core::roads;
use pbd_core::vehicle::cart::{self, Drive};
use pbd_core::vehicle::ride::{Ride, RideSpec};

/// How near the player stands to a kept animal to mount it, m.
pub const MOUNT_M: f32 = 2.0;

/// A ridden species' spec, by the animal's record.
fn spec_of(world: &World, id: u64) -> Option<RideSpec> {
    let body = animal_body(world, id)?;
    let sp = species_id(world, body.sp)?;
    world
        .resource::<super::Fleet>()
        .specs
        .carts
        .riding
        .of(&sp)
        .copied()
}

/// The ground under a body-local point, m from the centre: the carts' own
/// query, terrain or a road piece's floor.
fn ground(world: &World, point: DVec3) -> f64 {
    super::ground_at(world, point)
}

/// A kept animal's feet as body-local `f64`, from the farmyard's local.
fn body_local(world: &World, local: Vec3) -> DVec3 {
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    local.as_dvec3() - centre
}

/// The nearest kept plodder or horse within reach of `feet` (body-local)
/// that is in no cart's shafts: its distance and record id.
pub fn mountable(world: &World, feet: DVec3) -> Option<(f64, u64)> {
    let c = world.get_resource::<Carts>()?;
    if c.riding.is_some() || c.holding.is_some() || c.seated.is_some() {
        return None;
    }
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    let local = (centre + feet).as_vec3();
    let yard = world.get_resource::<Farmyard>()?;
    let hitched = world
        .get_resource::<WorldSave>()
        .map(|s| cart::hitched(&s.records))
        .unwrap_or_default();
    puller_species(world)
        .into_iter()
        .flat_map(|sp| yard.kept_near(sp, local, MOUNT_M))
        .filter(|(id, _)| !hitched.contains(id))
        .map(|(id, p)| (f64::from(p.distance(local)), id))
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

/// Mount the kept animal `id`: its record written ridden, the yard told to
/// stop drawing it, and the walker set on its back.
pub fn mount(world: &mut World, id: u64) -> bool {
    let Some(mut body) = animal_body(world, id) else {
        return false;
    };
    if world
        .get_resource::<WorldSave>()
        .is_some_and(|s| cart::hitched(&s.records).contains(&id))
    {
        return false;
    }
    let Some((_, feet)) = walker_feet(world) else {
        return false;
    };
    // Where it stands and faces, from the yard.
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    let local = (centre + feet).as_vec3();
    let Some((pos, facing)) = world
        .get_resource::<Farmyard>()
        .and_then(|y| y.kept_pose(id, local, MOUNT_M + 1.0))
    else {
        return false;
    };
    body.ridden = true;
    let stored = world
        .get_resource_mut::<WorldSave>()
        .is_some_and(|mut s| s.store(&Author::Player(0), vec![body.record(id)]).is_some());
    if !stored {
        say(world, &pbd_core::locale::tr("vehicles.cart.unsaved"));
        return false;
    }
    if let Some(mut yard) = world.get_resource_mut::<Farmyard>() {
        yard.hand_over(id);
    }
    // Its coat, made here if no cart has made it yet.
    super::materials(world);
    let at = body_local(world, pos);
    let up = at.normalize_or(DVec3::Y);
    let at = up * ground(world, at);
    world.resource_mut::<Carts>().riding = Some(Ride::new(id, at, facing.as_dvec3()));
    world.resource_mut::<Pace>().cap_mps = Some(0.0);
    let name = animal_name(world, id);
    info!("carts: riding {}", name.into_string());
    true
}

/// Get off: the animal's record written unridden with its home the cell it
/// stands on, the walker set down beside it, and the animal back in the
/// yard where it stood. It is never ended or dropped.
pub fn get_off(world: &mut World) {
    let Some(ride) = world.resource_mut::<Carts>().riding.take() else {
        return;
    };
    world.resource_mut::<Pace>().cap_mps = None;
    if let Some((e, _)) = world.resource_mut::<Carts>().ride_mesh.take() {
        world.entity_mut(e).despawn();
    }
    let up = ride.up();
    let right = ride.heading.cross(up);
    let Some(mut body) = animal_body(world, ride.animal) else {
        return;
    };
    body.ridden = false;
    if let Some(key) = roads::nearest_key(up.as_vec3()) {
        body.home = key;
        body.pen = None;
    }
    if let Some(mut save) = world.get_resource_mut::<WorldSave>()
        && save
            .store(&Author::Player(0), vec![body.record(ride.animal)])
            .is_none()
    {
        say(world, &pbd_core::locale::tr("vehicles.cart.unsaved"));
    }
    // The walker beside it, on its left.
    let side = ride.at - right * 1.3;
    let side_up = side.normalize_or(DVec3::Y);
    let stood = side_up * (ground(world, side) + f64::from(HALF_HEIGHT) + 0.05);
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    if let Some((walker, _)) = walker_feet(world)
        && let Some(mut p) = world.get_mut::<Position>(walker)
    {
        p.0 = (centre + stood).as_vec3();
    }
    back_in_yard(world, ride.animal, &body, ride.at, ride.heading, 0.0);
    info!("carts: got off");
}

/// Put a kept animal back in the yard at `at` (body-local), facing
/// `heading`, following for `follow_s`.
fn back_in_yard(
    world: &mut World,
    id: u64,
    body: &AnimalBody,
    at: DVec3,
    heading: DVec3,
    follow_s: f32,
) {
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    let local = (centre + at).as_vec3();
    let up = at.normalize_or(DVec3::Y).as_vec3();
    if let Some(mut yard) = world.get_resource_mut::<Farmyard>() {
        yard.take_back(id, body, local, up, heading.as_vec3(), follow_s);
    }
}

/// Each fixed tick while riding: the move keys ride on, rein back and turn,
/// Shift runs; the walker is carried at the rider's eye on its back, the
/// view carried round its turns. Metres ridden go to the missions.
pub fn step(world: &mut World) {
    let dt = world.resource::<Time>().delta_secs_f64();
    let Some(mut ride) = world.resource::<Carts>().riding.clone() else {
        return;
    };
    if dt <= 0.0 {
        return;
    }
    let Some(spec) = spec_of(world, ride.animal) else {
        return;
    };
    let axes = world
        .get_resource::<WalkingState>()
        .map_or(Vec2::ZERO, |w| w.move_axes());
    // Shift runs, as it sprints on foot.
    let run = world
        .get_resource::<ButtonInput<KeyCode>>()
        .is_some_and(|k| k.pressed(KeyCode::ShiftLeft) || k.pressed(KeyCode::ShiftRight));
    let input = Drive {
        forward: axes.y,
        turn: -axes.x,
    };
    let before = ride.heading;
    let ground_fn = |p: DVec3| ground(world, p);
    let moved = ride.step(&spec, input, run, dt, &ground_fn);
    let up = ride.up();
    let turned = before.cross(ride.heading).dot(up).clamp(-1.0, 1.0).asin();
    if let Some(mut w) = world.get_resource_mut::<WalkingState>() {
        w.turn_with(turned as f32);
    }
    // The rider: the walker's eye at `eye_m` over the ground on its back.
    let seat =
        ride.at + up * (f64::from(spec.eye_m) - f64::from(EYE_HEIGHT) + f64::from(HALF_HEIGHT));
    let centre = world.resource::<PlanetRenderFrame>().physics_center();
    if let Some((walker, _)) = walker_feet(world) {
        if let Some(mut p) = world.get_mut::<Position>(walker) {
            p.0 = (centre + seat).as_vec3();
        }
        if let Some(mut v) = world.get_mut::<LinearVelocity>(walker) {
            v.0 = Vec3::ZERO;
        }
    }
    let whole = (ride.walked_m.floor() - (ride.walked_m - moved).floor()) as u32;
    world.resource_mut::<Carts>().riding = Some(ride);
    if let Some(happened) = missions::ridden(whole) {
        world.write_message(Did(happened));
    }
}

/// What riding counts for the missions.
pub mod missions {
    use pbd_core::missions::{Happened, Kind};

    /// Metres ridden: `travel` (`ride`). No mission asks for it yet.
    pub fn ridden(metres: u32) -> Option<Happened> {
        (metres > 0).then(|| Happened::new(Kind::Travel, "ride", metres))
    }
}

/// Each frame while riding: the animal drawn where it stands, in its own
/// form and coat, its gait by the distance gone over its stride.
pub fn draw(
    mut commands: Commands,
    frame: Res<PlanetRenderFrame>,
    mut carts: ResMut<Carts>,
    save: Option<Res<WorldSave>>,
    land: Option<Res<crate::config::HusbandryConfig>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut roots: Query<&mut Transform>,
) {
    let Some(ride) = carts.riding.clone() else {
        return;
    };
    let Some(materials) = carts.materials.clone() else {
        return;
    };
    let roster = land
        .as_deref()
        .map(|l| l.0.roster(pbd_core::fauna::HOME_BODY).to_vec())
        .unwrap_or_default();
    let body = save
        .as_deref()
        .and_then(|s| s.records.get(animal::ANIMAL, ride.animal))
        .and_then(|r| r.read::<AnimalBody>());
    let species = body.as_ref().and_then(|b| roster.get(b.sp as usize));
    let genome = body.as_ref().map(|b| b.genome()).unwrap_or_default();
    let stride = species.map_or(1.7, |s| s.stride_m) as f64;
    let walk = (ride.speed.abs() / 1.2).min(1.0) as f32;
    let mut b = crate::husbandry::rig::Builder::default();
    super::model::puller(
        &mut b,
        species,
        &genome,
        &crate::husbandry::forms::Stance {
            phase: (ride.walked_m / stride * std::f64::consts::TAU) as f32,
            walk,
            head_down: if walk < 0.05 { 0.2 } else { 0.0 },
            saddled: true,
            ..default()
        },
    );
    let place = Transform::from_translation((frame.center + ride.at).as_vec3())
        .with_rotation(crate::vehicles::place::facing(ride.up(), ride.heading).as_quat());
    match &carts.ride_mesh {
        Some((e, h)) => {
            if let Some(mesh) = meshes.get_mut(h) {
                b.write(mesh);
            }
            if let Ok(mut t) = roots.get_mut(*e) {
                *t = place;
            }
        }
        None => {
            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            );
            b.write(&mut mesh);
            let handle = meshes.add(mesh);
            let root = commands
                .spawn((
                    Name::new("Ridden animal"),
                    place,
                    Visibility::default(),
                    LitByField,
                    LitLikeTerrain,
                ))
                .with_child((
                    Mesh3d(handle.clone()),
                    MeshMaterial3d(materials.coat.clone()),
                    Transform::default(),
                    bevy::camera::visibility::NoFrustumCulling,
                ))
                .id();
            carts.ride_mesh = Some((root, handle));
        }
    }
}

/// Once a world is open and the walker stands in it: every animal its save
/// says is ridden is written unridden with its home the walker's cell, and
/// stands beside the walker, following. A reload while riding comes back on
/// foot beside the animal (design 14).
pub fn restore(world: &mut World) {
    let Some(serial) = world.get_resource::<WorldSave>().map(|s| s.serial()) else {
        return;
    };
    if world.resource::<Carts>().ride_restored == Some(serial)
        || world.resource::<Carts>().loaded != Some(serial)
    {
        return;
    }
    let Some((_, feet)) = walker_feet(world) else {
        return;
    };
    if world.get_resource::<Farmyard>().is_none() {
        return;
    }
    world.resource_mut::<Carts>().ride_restored = Some(serial);
    let ridden: Vec<(u64, AnimalBody)> = world
        .get_resource::<WorldSave>()
        .map(|s| {
            animal::animals(&s.records)
                .into_iter()
                .filter(|(_, b)| b.ridden)
                .collect()
        })
        .unwrap_or_default();
    for (id, mut body) in ridden {
        body.ridden = false;
        let up = feet.normalize_or(DVec3::Y);
        if let Some(key) = roads::nearest_key(up.as_vec3()) {
            body.home = key;
            body.pen = None;
        }
        if world
            .get_resource_mut::<WorldSave>()
            .is_some_and(|mut s| s.store(&Author::Player(0), vec![body.record(id)]).is_some())
        {
            let facing = up.any_orthonormal_vector();
            let right = facing.cross(up);
            let beside = feet + right * 1.6;
            let beside = beside.normalize() * ground(world, beside);
            back_in_yard(world, id, &body, beside, facing, super::FOLLOW_S);
            info!("carts: animal {id} was ridden; it stands beside the walker");
        }
    }
}

/// `--cart-close ride`: a plodder kept just ahead, mounted by the player's
/// own path, and ridden on a few metres.
pub fn script(world: &mut World, feet: DVec3, ahead: DVec3, right: DVec3) {
    let at = feet + ahead * 1.4;
    let at = at.normalize() * ground(world, at);
    let Some(id) = super::keep_animal(world, "plodder", at, ahead, false, Vec::new()) else {
        return;
    };
    // Into the yard at once, where the player's own mount finds it.
    if let Some(body) = animal_body(world, id) {
        back_in_yard(world, id, &body, at, ahead, 0.0);
    }
    if !mount(world, id) {
        warn!("carts: the scripted ride did not mount");
    }
    let _ = right;
}
