//! A harbour's boats (`cities-in-the-world` task 4.2b, survey T7: "you can
//! use any boat you find"), and the stowing that keeps a world of them cheap.
//!
//! Each boat the harbour's template moors on the water is a craft record,
//! made once when the fleet first meets the harbour: a sailing boat is a
//! Tern, a rowboat or a canoe a Loon. It lies at its berth on the planet's
//! water, anchored to the seabed under it, and is tagged with its berth so it
//! is never made twice, wherever the player sails it. A craft further than
//! [`STOW_M`] from the viewer is kept as its record only, and comes back
//! within [`WAKE_M`]: the towns' own ranges.

use super::{Aboard, Fleet, Vehicle, place};
use crate::sea::Sea;
use crate::towns::{TownAssets, Towns, patch_m, patch_round};
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::settlement::record::{self, Town};
use pbd_core::settlement::{Template, chart::Patch, sea};
use pbd_core::sites::SiteKind;
use pbd_core::vehicle::spec::VehicleSpecs;
use pbd_core::vehicle::{Craft, Hulls, Kind, Mooring};
use std::collections::BTreeSet;
use std::sync::Arc;

/// A craft further than this from the viewer is stowed, m.
pub const STOW_M: f64 = 1500.0;
/// A stowed craft nearer than this comes back, m.
pub const WAKE_M: f64 = 1200.0;

/// The game's craft for a mockup boat: a sailing boat is a Tern, a rowboat
/// or a canoe a Loon, the game's nearest small boat.
pub fn kind_of(boat: &str) -> Kind {
    match boat {
        "sail" => Kind::Tern,
        _ => Kind::Loon,
    }
}

/// The water a craft needs under it at a mooring, m, from its hull in
/// `vehicles.ron`: the Tern's keel reaches about 1.7 m under its waterline,
/// and the Loon's hull is 0.36 m deep. A harbour's water is whole layers, so
/// a metre floats a Loon and two a Tern. (A new world's own two boats are
/// placed with more margin, `place::TERN_DEPTH_M` and `LOON_DEPTH_M`, in
/// open water.)
fn draws(kind: Kind) -> f32 {
    match kind {
        Kind::Tern => 2.0,
        _ => 1.0,
    }
}

/// A harbour's boats the fleet does not hold yet (`made` names the berths it
/// does), each at its berth on the water, anchored to the seabed under it
/// and tagged with its berth, numbered from `next_id`. `floor` is the solid
/// ground's radius under a direction and `sea_radius` the water's. Returns
/// the craft, and how many berths were skipped: over land, or over water
/// shallower than a Loon draws.
#[allow(clippy::too_many_arguments)]
pub fn boats_for(
    site: u32,
    town: &Town,
    template: &Template,
    patch: &Patch,
    sea_radius: f32,
    made: &BTreeSet<(u32, u32)>,
    specs: &Arc<VehicleSpecs>,
    hulls: &Hulls,
    next_id: &mut u64,
    floor: impl Fn(Vec3) -> f32,
) -> (Vec<Craft>, usize) {
    let Ok(chart) = record::chart_of(town, patch) else {
        return (Vec::new(), template.boats.len());
    };
    let mut crafts = Vec::new();
    let mut skipped = 0;
    for (n, boat) in template.boats.iter().enumerate() {
        let berth = (site, n as u32);
        if made.contains(&berth) {
            continue;
        }
        let Some((direction, bow)) = sea::boat_pose(&chart, patch, boat, template.grid.cell_m)
        else {
            skipped += 1;
            continue;
        };
        let bed = floor(direction);
        let depth = sea_radius - bed;
        // A sailing berth too shallow for a Tern takes a Loon, so the berth
        // keeps a boat.
        let kind = match kind_of(&boat.kind) {
            Kind::Tern if depth < draws(Kind::Tern) => Kind::Loon,
            kind => kind,
        };
        if depth < draws(kind) {
            skipped += 1;
            continue;
        }
        let up = direction.as_dvec3();
        let mut craft = Craft::new(
            kind,
            *next_id,
            specs.clone(),
            hulls.clone(),
            up * f64::from(sea_radius),
            place::facing(up, bow.as_dvec3()),
        );
        *next_id += 1;
        craft.mooring = Some(Mooring {
            at: craft.bow().normalize() * f64::from(bed),
            length: place::rode(f64::from(depth)),
            anchored: true,
        });
        craft.berth = Some(berth);
        crafts.push(craft);
    }
    (crafts, skipped)
}

/// Whether a craft at `at` stays in the world, comes into it, or is stowed,
/// with the viewer at `viewer`, both planet-local: stowed past [`STOW_M`],
/// back within [`WAKE_M`], as it was between.
pub fn in_range(at: DVec3, viewer: DVec3, was: bool) -> bool {
    let d = at.distance(viewer);
    if was { d <= STOW_M } else { d < WAKE_M }
}

/// The viewer, planet-local: the walker, or the camera where there is none.
fn viewer(world: &mut World) -> Option<DVec3> {
    let centre = world
        .get_resource::<crate::planet::PlanetRenderFrame>()?
        .center;
    let mut walkers =
        world.query_filtered::<&avian3d::prelude::Position, With<crate::walking::Walker>>();
    if let Some(p) = walkers.iter(world).next() {
        return Some(centre + p.0.as_dvec3());
    }
    let mut cameras = world.query_filtered::<(&Camera, &GlobalTransform), With<Camera3d>>();
    cameras
        .iter(world)
        .find(|(c, _)| c.is_active)
        .map(|(_, at)| at.translation().as_dvec3() - centre)
        .map(|local| centre + local)
}

/// Every berth the fleet holds a craft of, in the world or stowed.
fn made(world: &mut World) -> BTreeSet<(u32, u32)> {
    let mut query = world.query::<&Vehicle>();
    let mut out: BTreeSet<(u32, u32)> = query.iter(world).filter_map(|v| v.craft.berth).collect();
    if let Some(fleet) = world.get_resource::<Fleet>() {
        out.extend(fleet.stowed.iter().filter_map(|r| r.berth));
    }
    out
}

/// Moor each harbour's boats, once, when the fleet first meets it: new craft
/// are a world mutation, written at once.
pub fn moor_harbours(world: &mut World, mut done: Local<BTreeSet<u32>>) {
    if !world.get_resource::<Fleet>().is_some_and(|f| f.spawned) {
        return;
    }
    let (Some(towns), Some(assets), Some(sea)) = (
        world.get_resource::<Towns>(),
        world.get_resource::<TownAssets>(),
        world.get_resource::<Sea>(),
    ) else {
        return;
    };
    let harbours: Vec<_> = towns
        .held
        .iter()
        .filter(|h| h.site.kind == SiteKind::Harbour && !done.contains(&h.site.id))
        .cloned()
        .collect();
    if harbours.is_empty() {
        return;
    }
    let sea_radius = sea.radius;
    let templates: Vec<Option<Template>> = harbours
        .iter()
        .map(|h| assets.template_named(&h.town.template).cloned())
        .collect();
    let config = *crate::planet::terrain_config();
    let mut berths = made(world);
    let viewer = viewer(world);
    for (h, template) in harbours.iter().zip(templates) {
        done.insert(h.site.id);
        let Some(template) = template else {
            continue;
        };
        let patch = patch_round(h.site.direction, config.radius_m, patch_m(h.site.kind));
        let (crafts, skipped) = {
            let mut fleet = world.resource_mut::<Fleet>();
            let (specs, hulls) = (fleet.specs.clone(), fleet.hulls.clone());
            let mut next = fleet.next_id;
            let made = boats_for(
                h.site.id,
                &h.town,
                &template,
                &patch,
                sea_radius,
                &berths,
                &specs,
                &hulls,
                &mut next,
                place::floor,
            );
            fleet.next_id = next;
            made
        };
        if crafts.is_empty() && skipped == 0 {
            continue;
        }
        info!(
            "{}: {} boats moored, {skipped} berths skipped (over land or too shallow)",
            h.site.name,
            crafts.len()
        );
        for craft in crafts {
            berths.extend(craft.berth);
            let near = viewer.is_some_and(|v| in_range(craft.reference_position(), v, false));
            if near {
                place::spawn_craft(world, craft);
            } else {
                world.resource_mut::<Fleet>().stowed.push(craft.record());
            }
        }
        world.resource_mut::<Fleet>().dirty = true;
    }
}

/// Once a second, stow the craft far from the viewer and bring back the
/// stowed ones near it. The craft the player is aboard is never stowed.
pub fn stow_and_wake(world: &mut World, mut due: Local<f32>) {
    let dt = world.resource::<Time>().delta_secs();
    *due -= dt;
    if *due > 0.0 || !world.get_resource::<Fleet>().is_some_and(|f| f.spawned) {
        return;
    }
    *due = 1.0;
    let Some(viewer) = viewer(world) else {
        return;
    };
    let aboard = world.get_resource::<Aboard>().and_then(|a| a.0);
    let mut query = world.query::<(Entity, &Vehicle)>();
    let far: Vec<(Entity, pbd_core::vehicle::record::VehicleRecord)> = query
        .iter(world)
        .filter(|(e, v)| {
            Some(*e) != aboard && !in_range(v.craft.reference_position(), viewer, true)
        })
        .map(|(e, v)| (e, v.craft.record()))
        .collect();
    for (entity, record) in far {
        world.entity_mut(entity).despawn();
        world.resource_mut::<Fleet>().stowed.push(record);
    }
    let (wake, keep): (Vec<_>, Vec<_>) = std::mem::take(&mut world.resource_mut::<Fleet>().stowed)
        .into_iter()
        .partition(|r| in_range(DVec3::from(r.position), viewer, false));
    world.resource_mut::<Fleet>().stowed = keep;
    let (specs, hulls) = {
        let fleet = world.resource::<Fleet>();
        (fleet.specs.clone(), fleet.hulls.clone())
    };
    for record in wake {
        match Craft::from_record(&record, specs.clone(), hulls.clone()) {
            Ok(craft) => {
                place::spawn_craft(world, craft);
            }
            Err(error) => warn!("craft {} not brought back: {error}", record.id),
        }
    }
}
