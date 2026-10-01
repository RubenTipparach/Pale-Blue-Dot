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
use bevy::math::{DMat3, DQuat};
use bevy::prelude::*;
use pbd_core::DVec3;
use pbd_core::settlement::pieces::Frame;
use pbd_core::settlement::record::{self, Town};
use pbd_core::settlement::{Template, chart::Patch, sea};
use pbd_core::sites::SiteKind;
use pbd_core::vehicle::spec::VehicleSpecs;
use pbd_core::vehicle::{Craft, Hulls, Kind, Mooring};
use std::collections::BTreeSet;
use std::sync::Arc;

/// The cog's number among its harbour's berths (`sail-the-cog` step 3,
/// part 3): past any boat's.
pub const COG: u32 = u32::MAX;

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

/// The water a craft needs under it at a mooring, m of the drawn sea, from
/// its hull in `vehicles.ron`: the Tern's keel reaches about 1.7 m under its
/// waterline, and the Loon's hull is 0.36 m deep. The drawn sea lies half a
/// metre under the layers' sea level (`water.ron`), so a seabed one layer
/// down holds 0.5 m of it, which floats a Loon; a Tern wants 2 m. (A new
/// world's own two boats are placed with more margin, `place::TERN_DEPTH_M`
/// and `LOON_DEPTH_M`, in open water.)
fn draws(kind: Kind) -> f32 {
    match kind {
        Kind::Tern => 2.0,
        _ => 0.5,
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

/// Where a harbour's cog lies at rest at its berth: its waterline's middle
/// on the drawn sea (`sea_radius`) and its bow, planet-local, from the
/// town's chart (`pieces::cog::berth`). `None` where the town has no cog or
/// its chart cannot be made.
pub fn cog_berth(
    town: &Town,
    template: &Template,
    patch: &Patch,
    radius_m: f32,
    sea_radius: f32,
) -> Option<(Vec3, Vec3)> {
    let cog = template.cog.as_ref()?;
    let chart = record::chart_of(town, patch).ok()?;
    pbd_core::settlement::pieces::cog::berth(
        patch,
        &chart,
        template,
        cog,
        radius_m,
        sea_radius - radius_m,
    )
}

/// A harbour's cog, if the fleet does not hold it yet (`made` names the
/// berths it does): at rest at its berth on a bollard, tagged `(site,
/// COG)`, numbered `next_id`.
#[allow(clippy::too_many_arguments)]
pub fn cog_for(
    site: u32,
    town: &Town,
    template: &Template,
    patch: &Patch,
    radius_m: f32,
    sea_radius: f32,
    made: &BTreeSet<(u32, u32)>,
    specs: &Arc<VehicleSpecs>,
    hulls: &Hulls,
    next_id: &mut u64,
) -> Option<Craft> {
    let berth = (site, COG);
    if made.contains(&berth) {
        return None;
    }
    let (origin, bow) = cog_berth(town, template, patch, radius_m, sea_radius)?;
    let up = origin.normalize().as_dvec3();
    let mut craft = Craft::new(
        Kind::Cog,
        *next_id,
        specs.clone(),
        hulls.clone(),
        origin.as_dvec3(),
        place::facing(up, bow.as_dvec3()),
    );
    *next_id += 1;
    craft.mooring = Some(Mooring {
        at: craft.bow(),
        length: 2.0,
        anchored: false,
    });
    craft.berth = Some(berth);
    Some(craft)
}

/// Whether a craft is a harbour's cog made fast at its own berth, where it
/// rides the mooring swing and is not stepped.
pub fn on_swing(craft: &Craft) -> bool {
    craft.kind == Kind::Cog
        && craft.berth.is_some_and(|(_, n)| n == COG)
        && craft.mooring.is_some_and(|m| !m.anchored)
}

/// How long a cog just made fast takes to ease onto its swing, s.
pub const EASE_S: f32 = 3.0;

/// A harbour's cog's berth, worked out once from its town: the frame it
/// rests in (its waterline's middle, +y up, +z aft) and its bow; and, just
/// made fast, the pose it is easing from and how long it has eased.
#[derive(Component, Clone, Copy, Debug)]
pub struct Berthed {
    pub rest: Frame,
    pub bow: Vec3,
    pub easing: Option<(DVec3, DQuat, f32)>,
}

impl Berthed {
    pub fn new(origin: Vec3, bow: Vec3) -> Self {
        let y = origin.normalize();
        let z = -bow;
        Self {
            rest: Frame {
                origin,
                x: y.cross(z),
                y,
                z,
            },
            bow,
            easing: None,
        }
    }
}

/// A held harbour's cog's berth, by its site.
pub fn berth_of(towns: &Towns, assets: &TownAssets, sea_radius: f32, site: u32) -> Option<Berthed> {
    let held = towns.held.iter().find(|h| h.site.id == site)?;
    let template = assets.template_named(&held.town.template)?;
    let radius_m = crate::planet::terrain_config().radius_m;
    let patch = patch_round(held.site.direction, radius_m, patch_m(held.site.kind));
    let (origin, bow) = cog_berth(&held.town, template, &patch, radius_m, sea_radius)?;
    Some(Berthed::new(origin, bow))
}

/// Ride each harbour's cog made fast at its berth on the mooring swing
/// (`sail-the-cog` step 3, part 3): not stepped, its pose the swing's about
/// its berth and its velocity the swing's, eased onto it over [`EASE_S`]
/// when it has just been made fast. Its rest is the swing's, so it is never
/// a moving craft the save must keep writing.
#[allow(clippy::type_complexity)]
pub fn swing_moored_cogs(
    mut commands: Commands,
    time: Res<Time>,
    swing: Option<Res<crate::decks::CogSwing>>,
    towns: Option<Res<Towns>>,
    assets: Option<Res<TownAssets>>,
    sea: Option<Res<Sea>>,
    mut cogs: Query<(Entity, &mut Vehicle, Option<&mut Berthed>)>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let t = time.elapsed_secs();
    let motion = crate::decks::Swing::MOORED.scaled(swing.map_or(1.0, |s| s.0));
    for (entity, mut vehicle, berthed) in &mut cogs {
        if !on_swing(&vehicle.craft) {
            continue;
        }
        vehicle.rested = true;
        let mut found = None;
        let berthed = match berthed {
            Some(b) => b.into_inner(),
            None => {
                let (Some(towns), Some(assets), Some(sea), Some((site, _))) = (
                    towns.as_deref(),
                    assets.as_deref(),
                    sea.as_deref(),
                    vehicle.craft.berth,
                ) else {
                    continue;
                };
                let Some(b) = berth_of(towns, assets, sea.radius, site) else {
                    continue;
                };
                found.insert(b)
            }
        };
        let now = motion.at(&berthed.rest, berthed.bow, t);
        let mut position = now.origin.as_dvec3();
        let mut orientation = DQuat::from_mat3(&DMat3::from_cols(
            now.x.as_dvec3(),
            now.y.as_dvec3(),
            now.z.as_dvec3(),
        ))
        .normalize();
        if let Some((p0, q0, age)) = berthed.easing.as_mut() {
            *age += dt;
            let w = f64::from(crate::decks::ease(*age / EASE_S));
            position = p0.lerp(position, w);
            orientation = q0.slerp(orientation, w);
            if *age >= EASE_S {
                berthed.easing = None;
            }
        }
        let body = &mut vehicle.craft.body;
        let (p, q) = (body.position, body.orientation);
        vehicle.craft.set_reference_pose(position, orientation);
        let body = &mut vehicle.craft.body;
        let h = f64::from(dt);
        body.velocity = (body.position - p) / h;
        let (axis, angle) = (body.orientation * q.inverse()).to_axis_angle();
        let angle =
            (angle + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI;
        body.angular_velocity = axis * (angle / h);
        if let Some(b) = found {
            commands.entity(entity).insert(b);
        }
    }
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
        let (crafts, skipped, cog) = {
            let mut fleet = world.resource_mut::<Fleet>();
            let (specs, hulls) = (fleet.specs.clone(), fleet.hulls.clone());
            let mut next = fleet.next_id;
            let (mut crafts, skipped) = boats_for(
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
            // Its cog, the craft its ship is (`sail-the-cog` step 3).
            let cog = cog_for(
                h.site.id,
                &h.town,
                &template,
                &patch,
                config.radius_m,
                sea_radius,
                &berths,
                &specs,
                &hulls,
                &mut next,
            );
            let has_cog = cog.is_some();
            crafts.extend(cog);
            fleet.next_id = next;
            (crafts, skipped, has_cog)
        };
        if crafts.is_empty() && skipped == 0 {
            continue;
        }
        info!(
            "{}: {} boats moored{}, {skipped} berths skipped (over land or too shallow)",
            h.site.name,
            crafts.len() - usize::from(cog),
            if cog { " and its cog" } else { "" }
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
