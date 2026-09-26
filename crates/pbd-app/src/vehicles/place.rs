//! Where a world's craft come from: back from its save exactly where they were
//! left, or, in a world that has none yet, placed once near the player - the
//! Kestrel on dry, level ground, the Tern and the Loon at anchor in water deep
//! enough for each.

use super::{Fleet, Vehicle, draw};
use crate::planet::PlanetContact;
use crate::planet::terrain::PLANET_RADIUS;
use crate::sea::Sea;
use crate::walking::Walker;
use avian3d::prelude::Position;
use bevy::math::DMat3;
use bevy::prelude::*;
use pbd_core::vehicle::{Craft, Kind, Mooring};
use pbd_core::{DQuat, DVec3};

/// How far out the search for a berth goes, m, and its steps.
const SEARCH_MAX_M: f32 = 2500.0;
const SEARCH_STEP_M: f32 = 4.0;
const SEARCH_BEARINGS: usize = 24;
/// The Kestrel stands this far from the player, m.
const PAD_M: [f32; 2] = [16.0, 40.0];
/// The pad is level if its corners are within this of each other, m, over
/// this far out from the centre.
const PAD_LEVEL_M: f32 = 0.6;
const PAD_HALF_M: f32 = 5.5;
/// Water a boat is put in, m under the sea surface, and how far apart the two
/// boats lie.
const TERN_DEPTH_M: f32 = 3.0;
const LOON_DEPTH_M: f32 = 1.2;
const BOATS_APART_M: f32 = 14.0;
/// The Kestrel's reference point over the ground, m: its gear reaches
/// 1.25 m down, and it settles the rest.
const GEAR_CLEARANCE_M: f64 = 1.3;
/// A new anchor's rode, per metre of depth, plus a margin, m.
const RODE_PER_DEPTH: f64 = 3.0;
const RODE_MARGIN_M: f64 = 2.0;

/// The fleet, once per world: the save's, else a new one near the player.
pub fn spawn_fleet(world: &mut World) {
    if world.get_resource::<Fleet>().is_none_or(|f| f.spawned) {
        return;
    }
    let Some(walker) = world
        .query_filtered::<&Position, With<Walker>>()
        .iter(world)
        .next()
        .map(|p| p.0)
    else {
        return;
    };
    let Some((crafts, next_id, fresh)) = gather(world, walker) else {
        return;
    };
    let center = world.resource::<crate::planet::PlanetRenderFrame>().center;
    for craft in crafts {
        let at = (center + craft.reference_position()).as_vec3();
        info!(
            "{} {} at {:.1} m from the player",
            craft.kind.name(),
            if fresh { "placed" } else { "restored" },
            at.distance(walker)
        );
        let entity = world
            .spawn((
                Name::new(craft.kind.name()),
                Transform::from_translation(at),
                Visibility::default(),
            ))
            .id();
        draw::build(world, entity, &craft);
        world.entity_mut(entity).insert(Vehicle::new(craft));
    }
    let mut fleet = world.resource_mut::<Fleet>();
    fleet.next_id = next_id;
    fleet.spawned = true;
    // A new fleet is a world mutation like any other: written now.
    fleet.dirty |= fresh;
}

/// The craft this world should have: its save's, or a new placement round
/// the walker. `None` until the planet and the sea are ready to ask.
fn gather(world: &World, walker: Vec3) -> Option<(Vec<Craft>, u64, bool)> {
    // The contact field has to exist (it is how the walker stands, and it
    // says the planet is built) but the berths are judged on the height
    // field below, not on it.
    let (Some(_), Some(sea)) = (
        world.get_resource::<PlanetContact>(),
        world.get_resource::<Sea>(),
    ) else {
        return None;
    };
    let saved = world
        .get_resource::<crate::saves::WorldSave>()
        .and_then(|save| save.vehicles.clone());
    let fleet = world.resource::<Fleet>();
    let (specs, hulls) = (fleet.specs.clone(), fleet.hulls.clone());
    let mut crafts = Vec::new();
    let mut next_id = 1;
    let fresh = saved.is_none();
    match saved {
        Some(file) => {
            next_id = file.next_id;
            for record in &file.vehicles {
                match Craft::from_record(record, specs.clone(), hulls.clone()) {
                    Ok(craft) => {
                        next_id = next_id.max(craft.id + 1);
                        crafts.push(craft);
                    }
                    Err(error) => warn!("craft {} not restored: {error}", record.id),
                }
            }
        }
        None => {
            let center = world.resource::<crate::planet::PlanetRenderFrame>().center;
            let local = walker.as_dvec3() - center;
            let berths = Berths::find(sea, local.normalize_or(DVec3::Y).as_vec3());
            for (kind, berth) in berths.0 {
                let Some(berth) = berth else {
                    warn!("no berth for the {} near the spawn", kind.name());
                    continue;
                };
                let mut craft = Craft::new(
                    kind,
                    next_id,
                    specs.clone(),
                    hulls.clone(),
                    berth.position,
                    berth.orientation,
                );
                next_id += 1;
                if let Some(floor) = berth.anchor_floor {
                    let depth = sea.radius as f64 - floor;
                    craft.mooring = Some(Mooring {
                        at: craft.bow().normalize() * floor,
                        length: rode(depth),
                        anchored: true,
                    });
                }
                crafts.push(craft);
            }
        }
    }
    Some((crafts, next_id, fresh))
}

/// A place and a pose for a new craft.
#[derive(Clone, Copy, Debug)]
struct Berth {
    position: DVec3,
    orientation: DQuat,
    /// For a boat: the seabed radius its anchor lies on.
    anchor_floor: Option<f64>,
}

struct Berths([(Kind, Option<Berth>); 3]);

#[cfg(test)]
mod frame_tests {
    use super::*;

    #[test]
    fn translated_walker_finds_the_same_body_local_berths() {
        let mut world = World::new();
        world.insert_resource(Fleet::new(Default::default()));
        world.insert_resource(PlanetContact::test_planet(5));
        world.insert_resource(Sea::new(&Default::default()));
        world.insert_resource(crate::saves::WorldSave::memory_only());
        world.insert_resource(crate::planet::PlanetRenderFrame::default());
        // Integer coordinates preserve exactly the same f32 local input after translation.
        let walker = (crate::flight_view::FlightViewConfig::default().spawn_direction
            * PLANET_RADIUS)
            .round();
        let original = gather(&world, walker).unwrap().0;
        let offset = DVec3::new(8192.0, -4096.0, 2048.0);
        world
            .resource_mut::<crate::planet::PlanetRenderFrame>()
            .center = offset;
        let translated = gather(&world, (walker.as_dvec3() + offset).as_vec3())
            .unwrap()
            .0;
        assert_eq!(original.len(), 3);
        for (a, b) in original.iter().zip(&translated) {
            assert_eq!(a.kind, b.kind);
            assert!(a.body.position.distance(b.body.position) < 1e-6);
            assert!(a.body.orientation.angle_between(b.body.orientation).abs() < 1e-6);
        }
    }
}

impl Berths {
    /// The Kestrel on the first level pad round the player; the Loon in the
    /// nearest water deep enough for it; and the Tern in the nearest water
    /// deep enough for IT round the Loon, so both boats lie off one shore
    /// rather than wherever each happened to find water first.
    fn find(sea: &Sea, up: Vec3) -> Self {
        let radius = sea.radius;
        let depth = |direction: Vec3| radius - floor(direction);
        let pad = rings(radius, up, |direction, distance| {
            (PAD_M[0]..=PAD_M[1]).contains(&distance)
                && depth(direction) < -0.3
                && level(direction, floor(direction))
        });
        let loon = rings(radius, up, |direction, _| depth(direction) >= LOON_DEPTH_M);
        let tern = loon.and_then(|(from, _)| {
            rings(radius, from, |direction, distance| {
                distance >= BOATS_APART_M && depth(direction) >= TERN_DEPTH_M
            })
        });
        let boat = |(direction, outward): (Vec3, Vec3)| Berth {
            position: direction.as_dvec3() * radius as f64,
            // Bow out to sea, stern to the shore it was found from.
            orientation: facing(direction.as_dvec3(), outward.as_dvec3()),
            anchor_floor: Some(floor(direction) as f64),
        };
        Self([
            (
                Kind::Kestrel,
                pad.map(|(direction, outward)| Berth {
                    position: direction.as_dvec3() * (floor(direction) as f64 + GEAR_CLEARANCE_M),
                    // Nose toward the player, so the first thing seen is it.
                    orientation: facing(direction.as_dvec3(), -outward.as_dvec3()),
                    anchor_floor: None,
                }),
            ),
            (Kind::Tern, tern.map(boat)),
            (Kind::Loon, loon.map(boat)),
        ])
    }
}

/// The nearest direction round `centre`, ring by ring outward, that `good`
/// accepts, with the outward tangent it was reached along. Stable: the same
/// planet and centre give the same answer.
fn rings(radius: f32, centre: Vec3, good: impl Fn(Vec3, f32) -> bool) -> Option<(Vec3, Vec3)> {
    let east = Vec3::Y.cross(centre).normalize_or(Vec3::X);
    let north = centre.cross(east);
    let mut distance = SEARCH_STEP_M;
    while distance <= SEARCH_MAX_M {
        for i in 0..SEARCH_BEARINGS {
            let bearing = i as f32 / SEARCH_BEARINGS as f32 * std::f32::consts::TAU;
            let tangent = east * bearing.cos() + north * bearing.sin();
            let angle = distance / radius;
            let direction = centre * angle.cos() + tangent * angle.sin();
            if good(direction, distance) {
                return Some((direction, tangent));
            }
        }
        distance += SEARCH_STEP_M;
    }
    None
}

/// The solid ground under a direction, m from the centre, as a craft will meet
/// it (`ground_under`) away from the tier: the exact height field. NOT
/// `PlanetContact::sample`: far from the player that answers from the coarse
/// level, which put the Tern's berth in 3.5 m of water that was 1.5 m deep once
/// the tier arrived, and ran it aground on its first tick.
fn floor(direction: Vec3) -> f32 {
    super::ground_under(
        None,
        direction.normalize_or(Vec3::Y).as_dvec3() * PLANET_RADIUS as f64,
    ) as f32
}

/// Whether the ground round `direction` is level enough to land a gear on.
fn level(direction: Vec3, ground: f32) -> bool {
    let east = Vec3::Y.cross(direction).normalize_or(Vec3::X);
    let north = direction.cross(east);
    let r = ground.max(1.0);
    [east, -east, north, -north].iter().all(|tangent| {
        let probe = (direction + *tangent * (PAD_HALF_M / r)).normalize();
        (floor(probe) - ground).abs() <= PAD_LEVEL_M
    })
}

/// The rode an anchor pays out in `depth` metres of water, m.
pub fn rode(depth: f64) -> f64 {
    depth * RODE_PER_DEPTH + RODE_MARGIN_M
}

/// The pose standing on `up` with its bow (-z) along `forward`.
pub fn facing(up: DVec3, forward: DVec3) -> DQuat {
    let up = up.normalize_or(DVec3::Y);
    let forward = (forward - up * forward.dot(up)).normalize_or(up.any_orthonormal_vector());
    let right = forward.cross(up);
    DQuat::from_mat3(&DMat3::from_cols(right, up, -forward)).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_facing_pose_stands_up_and_points_its_bow() {
        let up = DVec3::new(0.3, 0.9, -0.2).normalize();
        let forward = DVec3::X;
        let q = facing(up, forward);
        assert!((q * DVec3::Y - up).length() < 1e-9);
        let bow = q * DVec3::NEG_Z;
        let want = (forward - up * forward.dot(up)).normalize();
        assert!((bow - want).length() < 1e-9, "{bow} vs {want}");
    }
}
