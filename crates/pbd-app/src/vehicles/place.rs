//! Where a world's craft come from: back from its save exactly where they were
//! left, or, in a world that has none yet, placed once near the player - the
//! Loon at anchor in water deep enough for it. (The Kestrel, placed on dry,
//! level ground until `retire-kestrel`, and the Tern, placed at anchor beside
//! the Loon until `retire-tern`, are no longer placed; a save's Kestrel,
//! Tern, rowboat or cog is held aside as its record.)

use super::{Fleet, Vehicle, draw};
use crate::planet::PlanetContact;
use crate::planet::terrain::PLANET_RADIUS;
use crate::sea::Sea;
use crate::walking::Walker;
use avian3d::prelude::Position;
use bevy::math::DMat3;
use bevy::prelude::*;
use pbd_core::vehicle::{Craft, Kind, KindKey, Mooring};
use pbd_core::{DQuat, DVec3};

/// How far out the search for a berth goes, m, and its steps.
const SEARCH_MAX_M: f32 = 2500.0;
const SEARCH_STEP_M: f32 = 4.0;
const SEARCH_BEARINGS: usize = 24;
/// The inside of the band round a new world's start that is searched for
/// level ground, m: the ring `level_plain` samples begins 4 m past it. (It
/// was where the Kestrel's pad could stand, 16 to 40 m from the player; the
/// numbers are kept so a new world starts where it always has.)
const PLAIN_INNER_M: f32 = 16.0;
/// A spot is level if the ground four probes out round it is within this of
/// its own, m, the probes this far out from the centre.
const LEVEL_M: f32 = 0.6;
const LEVEL_HALF_M: f32 = 5.5;
/// A new world's start is a level plain: this many of the 120 samples in the
/// ring 20 to 36 m round it are level spots (`level_plain`).
const START_LEVEL_SPOTS: usize = 8;
/// Moors the Loon at a new world's spawn, as worlds were given one before
/// 2026-11-19. Only the vehicle tests insert it; the game never does.
#[derive(Resource)]
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct StarterLoon;

/// Water the Loon is put in, m under the sea surface.
const LOON_DEPTH_M: f32 = 1.2;
/// A new anchor's rode, per metre of depth, plus a margin, m.
const RODE_PER_DEPTH: f64 = 3.0;
const RODE_MARGIN_M: f64 = 2.0;

/// The fleet, once per world: the save's, else a new one near the player.
pub fn spawn_fleet(world: &mut World) {
    if world.get_resource::<Fleet>().is_none_or(|f| f.spawned) {
        return;
    }
    // The craft are placed round the walker, by its planet point.
    let o = crate::sim_origin::origin_of(world);
    let Some(local) = world
        .query_filtered::<&Position, With<Walker>>()
        .iter(world)
        .next()
        .map(|p| p.0)
    else {
        return;
    };
    let walker = o.query_point(local);
    let Some(Gathered {
        crafts,
        retired,
        next_id,
        fresh,
    }) = gather(world, walker)
    else {
        return;
    };
    // A player saved aboard a craft since retired loads at its seat, over
    // the sea where nothing now floats: put them ashore (`retire-tern`
    // decision 2), before the craft are placed round the new origin.
    // `walker` is already the planet's frame (`query_point`), as the
    // saved pose and the records are.
    let body = walker.as_dvec3();
    // Over the sea: the ground there (a harbour's dug berths included) lies
    // under the sea's surface.
    let sea_radius = world.resource::<Sea>().radius;
    if let Some(record) = at_a_retired_craft(body, &retired, |d| floor(d) < sea_radius) {
        info!(
            "the player was aboard craft {}, a retired {}: put ashore",
            record.id, record.kind
        );
        crate::walking::put_ashore(world, body.normalize_or(DVec3::Y).as_vec3());
    }
    let local = world
        .query_filtered::<&Position, With<Walker>>()
        .iter(world)
        .next()
        .map_or(local, |p| p.0);
    let center = world
        .resource::<crate::planet::PlanetRenderFrame>()
        .physics_center();
    for craft in crafts {
        let at = (center + craft.reference_position()).as_vec3();
        info!(
            "{} {} at {:.1} m from the player",
            craft.kind.name(),
            if fresh { "placed" } else { "restored" },
            at.distance(local)
        );
        spawn_craft(world, craft);
    }
    for record in &retired {
        info!(
            "craft {} is a retired {}: kept in the save, not in the world",
            record.id, record.kind
        );
    }
    let mut fleet = world.resource_mut::<Fleet>();
    fleet.next_id = next_id;
    fleet.retired = retired;
    fleet.spawned = true;
    // A new fleet is a world mutation like any other: written now.
    fleet.dirty |= fresh;
}

/// How near a held retired craft a loaded walker over the sea is taken to
/// have been aboard it, m: a seat is within a few metres of its craft's
/// reference point.
pub(crate) const ABOARD_REACH_M: f64 = 6.0;
/// The same for a held cog, m: its deck was walked on, and its 15 m length
/// puts a player standing at its bow or stern 7.5 m from its middle
/// (`retire-rowboat-and-cog` decision 4).
pub(crate) const COG_REACH_M: f64 = 9.0;

/// How near a held record of `kind` a loaded walker is taken to be on it.
fn reach_of(kind: &str) -> f64 {
    if kind == "cog" {
        COG_REACH_M
    } else {
        ABOARD_REACH_M
    }
}

/// The retired craft a loaded walker at `walker` (body-local) was aboard: one
/// within [`ABOARD_REACH_M`] of it ([`COG_REACH_M`] of a cog), where `over_sea` says the walker's
/// direction is over the sea. `None` anywhere else, so a load that is not at
/// a retired craft is never moved.
pub(crate) fn at_a_retired_craft(
    walker: DVec3,
    retired: &[pbd_core::vehicle::record::VehicleRecord],
    over_sea: impl Fn(Vec3) -> bool,
) -> Option<&pbd_core::vehicle::record::VehicleRecord> {
    let near = retired.iter().find(|r| {
        r.frame == pbd_core::vehicle::record::PLANET_FRAME
            && DVec3::from(r.position).distance(walker) <= reach_of(&r.kind)
    })?;
    over_sea(walker.normalize_or(DVec3::Y).as_vec3()).then_some(near)
}

/// A craft into the world as an entity, drawn, at its pose.
pub(crate) fn spawn_craft(world: &mut World, craft: Craft) -> Entity {
    let center = world.resource::<crate::planet::PlanetRenderFrame>().center;
    let at = (center + craft.reference_position()).as_vec3();
    let entity = world
        .spawn((
            Name::new(craft.kind.name()),
            Transform::from_translation(at),
            Visibility::default(),
        ))
        .id();
    draw::build(world, entity, &craft);
    world.entity_mut(entity).insert(Vehicle::new(craft));
    entity
}

/// What [`gather`] finds: the craft to put in the world, the save's records
/// of retired kinds to hold aside, the next ID to give, and whether the fleet
/// is new.
struct Gathered {
    crafts: Vec<Craft>,
    retired: Vec<pbd_core::vehicle::record::VehicleRecord>,
    next_id: u64,
    fresh: bool,
}

/// The craft this world should have: its save's, or a new placement round
/// the walker. `None` until the planet and the sea are ready to ask.
fn gather(world: &World, walker: Vec3) -> Option<Gathered> {
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
    let mut retired = Vec::new();
    let mut next_id = 1;
    let fresh = saved.is_none();
    match saved {
        Some(file) => {
            next_id = file.next_id;
            for record in &file.vehicles {
                // A retired kind is held aside exactly as read, and its ID
                // is never given again (`retire-kestrel` decision 1).
                if Kind::parse(&record.kind) == Some(KindKey::Retired) {
                    next_id = next_id.max(record.id + 1);
                    retired.push(record.clone());
                    continue;
                }
                match Craft::from_record(record, specs.clone(), hulls.clone()) {
                    Ok(craft) => {
                        next_id = next_id.max(craft.id + 1);
                        crafts.push(craft);
                    }
                    Err(error) => warn!("craft {} not restored: {error}", record.id),
                }
            }
        }
        // A new world moors no canoe at the spawn (owner, 2026-11-19, on the
        // starter Loon at the shore: "this boat shoulve been removed"):
        // boats are made now (the raft, the tube), and a town's quay moors
        // its canoes. An old world keeps the one it was given. The vehicle
        // tests still moor one, to board and sail (`StarterLoon`).
        None if !world.contains_resource::<StarterLoon>() => {}
        None => {
            // The walker is a physics `Position`.
            let center = world
                .resource::<crate::planet::PlanetRenderFrame>()
                .physics_center();
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
    Some(Gathered {
        crafts,
        retired,
        next_id,
        fresh,
    })
}

/// A place and a pose for a new craft.
#[derive(Clone, Copy, Debug)]
struct Berth {
    position: DVec3,
    orientation: DQuat,
    /// For a boat: the seabed radius its anchor lies on.
    anchor_floor: Option<f64>,
}

struct Berths([(Kind, Option<Berth>); 1]);

#[cfg(test)]
mod frame_tests {
    use super::*;

    #[test]
    fn translated_walker_finds_the_same_body_local_berths() {
        let mut world = World::new();
        world.insert_resource(Fleet::new(Default::default()));
        world.insert_resource(StarterLoon);
        world.insert_resource(PlanetContact::test_planet(5));
        world.insert_resource(Sea::new(&Default::default()));
        world.insert_resource(crate::saves::WorldSave::memory_only());
        world.insert_resource(crate::planet::PlanetRenderFrame::default());
        // Integer coordinates preserve exactly the same f32 local input after translation.
        let start = crate::walking::new_world_start(
            world.resource::<PlanetContact>(),
            world.resource::<Sea>().radius,
            crate::flight_view::FlightViewConfig::default().spawn_direction,
        );
        let walker = (start * PLANET_RADIUS).round();
        let original = gather(&world, walker).unwrap().crafts;
        let offset = DVec3::new(8192.0, -4096.0, 2048.0);
        world
            .resource_mut::<crate::planet::PlanetRenderFrame>()
            .center = offset;
        let translated = gather(&world, (walker.as_dvec3() + offset).as_vec3())
            .unwrap()
            .crafts;
        assert_eq!(
            original.len(),
            1,
            "{:?}",
            original.iter().map(|c| c.kind).collect::<Vec<_>>()
        );
        for (a, b) in original.iter().zip(&translated) {
            assert_eq!(a.kind, b.kind);
            assert!(a.body.position.distance(b.body.position) < 1e-6);
            assert!(a.body.orientation.angle_between(b.body.orientation).abs() < 1e-6);
        }
    }
}

impl Berths {
    /// The Loon in the nearest water deep enough for it. (The Tern was put
    /// in the nearest water deep enough for it round the Loon, until
    /// `retire-tern`; the Loon's search never depended on it, so the Loon
    /// lies where it did.)
    fn find(sea: &Sea, up: Vec3) -> Self {
        let radius = sea.radius;
        let depth = |direction: Vec3| radius - floor(direction);
        let loon = rings(radius, up, |direction, _| depth(direction) >= LOON_DEPTH_M);
        let boat = |(direction, outward): (Vec3, Vec3)| Berth {
            position: direction.as_dvec3() * radius as f64,
            // Bow out to sea, stern to the shore it was found from.
            orientation: facing(direction.as_dvec3(), outward.as_dvec3()),
            anchor_floor: Some(floor(direction) as f64),
        };
        Self([(Kind::Loon, loon.map(boat))])
    }
}

/// Open water `depth_m` deep and at least `apart_m` from `from`, nearest
/// first, with its bow out along the way it was found: a berth for a craft a
/// test puts in deeper water than the Loon's (where the retired Tern lay,
/// 14 m off the Loon in 3 m of water).
#[cfg(test)]
pub(crate) fn deep_berth_near(
    sea: &Sea,
    from: Vec3,
    apart_m: f32,
    depth_m: f32,
) -> Option<(DVec3, DQuat)> {
    let radius = sea.radius;
    rings(radius, from, |direction, distance| {
        distance >= apart_m && radius - floor(direction) >= depth_m
    })
    .map(|(direction, outward)| {
        (
            direction.as_dvec3() * radius as f64,
            facing(direction.as_dvec3(), outward.as_dvec3()),
        )
    })
}

/// The water a new world's Loon would be berthed in round `up`, body-local.
#[cfg(test)]
pub(crate) fn berth_near(sea: &Sea, up: Vec3) -> Option<DVec3> {
    Berths::find(sea, up).0[0].1.map(|b| b.position)
}

/// Open water at least `apart_m` from `from` and `depth_m` deep, nearest
/// first, body-local on the sea's surface: where a capture script launches
/// a made boat (`small-boats`), clear of the boats already there.
pub(crate) fn open_water_near(sea: &Sea, from: Vec3, apart_m: f32, depth_m: f32) -> Option<DVec3> {
    let radius = sea.radius;
    rings(radius, from, |direction, distance| {
        distance >= apart_m && radius - floor(direction) >= depth_m
    })
    .map(|(direction, _)| direction.as_dvec3() * radius as f64)
}

/// Whether `direction` stands on a level plain: the ground there is over the
/// sea and its four probes lie within [`LEVEL_M`] of it, and the ring 20 to
/// 36 m round it holds such a level spot again and again. Where a new world
/// on a generator with ranges starts (`taller-mountains` decision 8, survey
/// H4: "recommended"), so a player opens facing land they can walk. The rule
/// is the one the Kestrel's pad was found by, kept exactly when the Kestrel
/// was retired (`retire-kestrel` decision 4), so new worlds start where they
/// did.
///
/// Read on the generator's own ground ([`natural_floor`]), not the towns':
/// a town's terrace is a level plain, and a world whose towns stood before
/// its walker did (the loading screen, `loading-screen` decision 1) started
/// on Holbrook's instead of where the same seed had always started, under
/// another sky.
pub fn level_plain(sea_radius: f32, direction: Vec3) -> bool {
    let good = |direction: Vec3| {
        let ground = natural_floor(direction);
        ground - sea_radius > 0.3 && level_on(direction, ground, natural_floor)
    };
    let centre = direction.normalize_or(Vec3::Y);
    if !good(centre) {
        return false;
    }
    // A level spot on one-metre layers is a knife edge: a probe half a metre
    // over can cross a layer. So the start is a level plain, not a level
    // spot: at least `START_LEVEL_SPOTS` of the ring's samples are level.
    let east = Vec3::Y.cross(centre).normalize_or(Vec3::X);
    let north = centre.cross(east);
    let mut spots = 0;
    for k in 0..=4 {
        let distance = PLAIN_INNER_M + 4.0 + k as f32 * 4.0;
        for i in 0..SEARCH_BEARINGS {
            let bearing = i as f32 / SEARCH_BEARINGS as f32 * std::f32::consts::TAU;
            let tangent = east * bearing.cos() + north * bearing.sin();
            let angle = distance / sea_radius;
            if good(centre * angle.cos() + tangent * angle.sin()) {
                spots += 1;
                if spots >= START_LEVEL_SPOTS {
                    return true;
                }
            }
        }
    }
    false
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
/// level, which put the (since retired) Tern's berth in 3.5 m of water that was 1.5 m deep once
/// the tier arrived, and ran it aground on its first tick.
pub(crate) fn floor(direction: Vec3) -> f32 {
    super::ground_under(
        None,
        direction.normalize_or(Vec3::Y).as_dvec3() * PLANET_RADIUS as f64,
    ) as f32
}

/// The generator's own ground under a direction, m from the centre: [`floor`]
/// with no town's ground (`column::natural_surface_m`), and with no version 8
/// river (`TerrainConfig::site_ground`), so a new world starts where the same
/// seed always started and the rivers are not a reason to move it
/// (`ocean-environments` design 13).
pub(crate) fn natural_floor(direction: Vec3) -> f32 {
    PLANET_RADIUS
        + pbd_core::column::natural_surface_m(
            &crate::planet::terrain_config().site_ground(),
            direction.normalize_or(Vec3::Y),
        )
}

/// Whether the ground round `direction`, `ground` from the centre, is level
/// on the ground `floor` gives: its four probes lie within [`LEVEL_M`].
fn level_on(direction: Vec3, ground: f32, floor: impl Fn(Vec3) -> f32) -> bool {
    let east = Vec3::Y.cross(direction).normalize_or(Vec3::X);
    let north = direction.cross(east);
    let r = ground.max(1.0);
    [east, -east, north, -north].iter().all(|tangent| {
        let probe = (direction + *tangent * (LEVEL_HALF_M / r)).normalize();
        (floor(probe) - ground).abs() <= LEVEL_M
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
