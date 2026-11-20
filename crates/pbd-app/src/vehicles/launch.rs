//! Boats the player makes (`small-boats` design 4): a boat good launched on
//! water with the place button, and a made boat taken back up with the dig
//! button held on it.
//!
//! Both are two durable writes, ordered so a crash between them keeps the
//! boat rather than losing it ("Nothing a player made is dropped"): a launch
//! writes the fleet with the new craft first and then the slots without the
//! good; a take-up writes the slots with the good first and then the fleet
//! without the craft. At worst a crash leaves a boat and its good, never
//! neither.

use super::{Aboard, Fleet, Vehicle, fresh_water, ground_under, place};
use crate::fish::{FishingStatus, ToolSlot};
use crate::hotbar::Hotbar;
use crate::planet::PlanetContact;
use crate::saves::WorldSave;
use crate::sea::Sea;
use crate::walking::{WalkingCamera, WalkingState};
use bevy::prelude::*;
use pbd_core::inventory::{Item, Tool};
use pbd_core::vehicle::{Craft, Kind};
use pbd_core::{DQuat, DVec3};

/// How far along the look a boat can be launched, m.
const LAUNCH_M: f64 = 8.0;
/// The least water a launched boat needs under it, m.
const LAUNCH_DEPTH_M: f64 = 0.35;
/// How long the dig button is held on a made boat to take it up, s.
const TAKE_UP_S: f32 = 1.0;
/// How near the look must pass a boat's middle to be on it, m, and how far
/// the eye can be from it.
const AIM_M: f64 = 1.3;
const TAKE_UP_REACH_M: f64 = 5.0;

/// The take-up in hand: which craft, and how long the button has been held
/// on it.
#[derive(Resource, Default, Debug)]
pub struct TakingUp(pub Option<(Entity, f32)>);

/// The water a ray from `eye` along `look` meets first within reach: the
/// surface's radius and the point on it, or why not.
pub(super) fn water_along(
    sea_radius: f32,
    contact: Option<&PlanetContact>,
    eye: DVec3,
    look: DVec3,
) -> Result<(f64, DVec3), &'static str> {
    let steps = (LAUNCH_M / 0.25) as usize;
    for k in 1..=steps {
        let p = eye + look * (k as f64 * 0.25);
        let r = p.length();
        let floor = ground_under(contact, p);
        // A river's own surface, else the sea's.
        let surface = contact
            .and_then(|c| fresh_water(c, p))
            .map_or(f64::from(sea_radius), |f| f.surface);
        if r <= surface + 0.1 && floor < surface - LAUNCH_DEPTH_M {
            let up = p.normalize();
            return Ok((surface, up * surface));
        }
        if r <= floor {
            return Err("that is land, not water");
        }
    }
    Err("no water in reach")
}

/// A made craft of `kind` on the water at `on` (body-local, on the surface),
/// its bow along `look`.
pub(super) fn made_craft(fleet: &mut Fleet, kind: Kind, on: DVec3, look: DVec3) -> Craft {
    let up = on.normalize();
    let orientation: DQuat = place::facing(up, look);
    let mut craft = Craft::new(
        kind,
        fleet.next_id,
        fleet.specs.clone(),
        fleet.hulls.clone(),
        on,
        orientation,
    );
    craft.made = true;
    fleet.next_id += 1;
    craft
}

/// Write the fleet as it stands now. `false` if the save refused it.
pub(super) fn write_fleet(world: &mut World) -> bool {
    let crafts: Vec<Craft> = world
        .query::<&Vehicle>()
        .iter(world)
        .map(|v| v.craft.clone())
        .collect();
    let file = world.resource::<Fleet>().file(crafts.iter());
    let Some(mut save) = world.get_resource_mut::<WorldSave>() else {
        return true;
    };
    let ok = save.snapshot_vehicles(&file);
    if ok {
        world.resource_mut::<Fleet>().dirty = false;
    }
    ok
}

fn say(world: &mut World, text: &str) {
    if let Some(mut status) = world.get_resource_mut::<FishingStatus>() {
        status.say(text, 2.5);
    }
    info!("boats: {text}");
}

/// The eye and the look, body-local, of the walking camera.
fn walking_eye(world: &mut World) -> Option<(DVec3, DVec3)> {
    let centre = world.resource::<crate::planet::PlanetRenderFrame>().center;
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

/// On foot: the place button with a boat good selected launches it; the dig
/// button held on a made boat takes it up.
pub fn launch_and_take_up(world: &mut World) {
    let on_foot = world
        .get_resource::<WalkingState>()
        .is_some_and(|w| w.active && (w.captured || w.scripted));
    let menu = world
        .get_resource::<crate::controls::MenuOpen>()
        .is_some_and(|m| m.0);
    let aboard = world
        .get_resource::<Aboard>()
        .is_some_and(|a| a.0.is_some());
    let spawned = world.get_resource::<Fleet>().is_some_and(|f| f.spawned);
    if !on_foot || menu || aboard || !spawned {
        world.resource_mut::<TakingUp>().0 = None;
        return;
    }
    let Some(buttons) = world.get_resource::<ButtonInput<MouseButton>>() else {
        return;
    };
    let place = buttons.just_pressed(MouseButton::Right);
    let dig = buttons.pressed(MouseButton::Left);
    if place {
        launch(world);
    }
    let rod = world
        .get_resource::<ToolSlot>()
        .is_some_and(|t| t.held() == Some(Tool::Rod));
    if dig && !rod {
        let dt = world.resource::<Time>().delta_secs();
        take_up(world, dt);
    } else {
        world.resource_mut::<TakingUp>().0 = None;
    }
}

/// The boat good in the selected slot, if any, and its kind.
fn held_boat(world: &World) -> Option<Kind> {
    let hotbar = world.get_resource::<Hotbar>()?;
    match hotbar.0.held()?.item {
        Item::Good(good) => Kind::of_good_code(good.code()),
        _ => None,
    }
}

fn launch(world: &mut World) {
    let Some(kind) = held_boat(world) else {
        return;
    };
    let Some((eye, look)) = walking_eye(world) else {
        return;
    };
    let sea_radius = world.resource::<Sea>().radius;
    let found = water_along(sea_radius, world.get_resource::<PlanetContact>(), eye, look);
    let on = match found {
        Ok((_, on)) => on,
        Err(why) => {
            say(
                world,
                &format!(
                    "The {} goes on water: {why}",
                    kind.name().as_str().to_lowercase()
                ),
            );
            return;
        }
    };
    let craft = made_craft(&mut world.resource_mut::<Fleet>(), kind, on, look);
    let entity = place::spawn_craft(world, craft);
    // The fleet first, with the new craft: a crash after it keeps the boat.
    if !write_fleet(world) {
        world.entity_mut(entity).despawn();
        say(world, "The save did not take the launch");
        return;
    }
    let good = kind.good_code().and_then(pbd_core::goods::Good::by_code);
    let mut slots = world.resource::<Hotbar>().0.clone();
    if let Some(good) = good {
        slots.take_item(Item::Good(good), 1);
    }
    if let Some(mut save) = world.get_resource_mut::<WorldSave>()
        && !save.record_pack(&slots)
    {
        error!("launch: the slots were not saved");
    }
    world.resource_mut::<Hotbar>().0 = slots;
    world.resource_mut::<Fleet>().dirty = true;
    say(world, &format!("{} launched", kind.name()));
}

/// The made craft the look is on, nobody aboard, in reach.
fn aimed_made(world: &mut World, eye: DVec3, look: DVec3) -> Option<(Entity, Kind, u64)> {
    world
        .query::<(Entity, &Vehicle)>()
        .iter(world)
        .filter(|(_, v)| v.craft.made && !v.craft.occupied)
        .filter_map(|(e, v)| {
            let to = v.craft.body.position - eye;
            let along = to.dot(look);
            let off = (to - look * along).length();
            (along > 0.0 && along <= TAKE_UP_REACH_M && off <= AIM_M).then_some((
                e,
                v.craft.kind,
                v.craft.id,
                along,
            ))
        })
        .min_by(|a, b| a.3.total_cmp(&b.3))
        .map(|(e, k, id, _)| (e, k, id))
}

fn take_up(world: &mut World, dt: f32) {
    let Some((eye, look)) = walking_eye(world) else {
        return;
    };
    let Some((entity, kind, id)) = aimed_made(world, eye, look) else {
        world.resource_mut::<TakingUp>().0 = None;
        return;
    };
    let held = {
        let mut taking = world.resource_mut::<TakingUp>();
        let held = match taking.0 {
            Some((e, s)) if e == entity => s + dt,
            _ => dt,
        };
        taking.0 = Some((entity, held));
        held
    };
    if held < TAKE_UP_S {
        return;
    }
    world.resource_mut::<TakingUp>().0 = None;
    let full = world
        .get_resource::<WorldSave>()
        .is_some_and(|save| !super::hold::read(save, id).is_empty());
    if full {
        say(world, "Empty the hold first");
        return;
    }
    let Some(good) = kind.good_code().and_then(pbd_core::goods::Good::by_code) else {
        return;
    };
    let mut slots = world.resource::<Hotbar>().0.clone();
    if slots.give(Item::Good(good), 1) > 0 {
        say(world, "No room in the pack");
        return;
    }
    // The slots first, with the good: a crash after it keeps the boat.
    if let Some(mut save) = world.get_resource_mut::<WorldSave>()
        && !save.record_pack(&slots)
    {
        say(world, "The save did not take it");
        return;
    }
    world.resource_mut::<Hotbar>().0 = slots;
    world.entity_mut(entity).despawn();
    if !write_fleet(world) {
        world.resource_mut::<Fleet>().dirty = true;
    }
    say(world, &format!("{} taken up", kind.name()));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On the test planet: looking down at the water a new world's Loon is
    /// berthed in finds it, a launched craft there is made and named in the
    /// fleet's file, and looking down at the start's own dry ground is
    /// refused as land.
    #[test]
    fn a_boat_launches_on_water_and_not_on_land() {
        let contact = PlanetContact::test_planet(5);
        let sea = Sea::new(&Default::default());
        let start = crate::walking::new_world_start(
            &contact,
            sea.radius,
            crate::flight_view::FlightViewConfig::default().spawn_direction,
        )
        .as_dvec3();
        let land = water_along(
            sea.radius,
            Some(&contact),
            start * (ground_under(Some(&contact), start * 4800.0) + 1.6),
            -start,
        );
        assert_eq!(land, Err("that is land, not water"));
        let mut fleet = Fleet::new(Default::default());
        let berth = place::berth_near(&sea, start.as_vec3()).expect("water near the start");
        let eye = berth.normalize() * (f64::from(sea.radius) + 1.6);
        let (surface, on) =
            water_along(sea.radius, Some(&contact), eye, -berth.normalize()).expect("water");
        assert!((surface - f64::from(sea.radius)).abs() < 1e-3);
        let raft = made_craft(&mut fleet, Kind::Raft, on, DVec3::X);
        assert!(raft.made && raft.kind == Kind::Raft);
        let file = fleet.file(std::iter::once(&raft));
        assert!(file.vehicles[0].made);
        assert!(file.next_id > raft.id);
    }
}
