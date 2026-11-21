//! Picking the glowing mushrooms (`glowing-mushrooms` design 3, 4): F on the
//! glowing plant the meadows grow, aimed at the plant itself (`fruit-aim`),
//! gives one `glowing-mushroom` and writes a `glow-plant` record by the cell's
//! exact key, with the hands in one line. The cell then draws no plant and
//! lights nothing until it grows back, out of sight.
//!
//! Which cells grow one is `pbd_core::flora::glows` on the key, as it always
//! was: nothing about where they grow is stored. Only the picks are, and the
//! column build reads them (`planet::column::glow_bits`); [`follow`] brings
//! the resident set round to a pick taken since it was built.

use super::farm::FarmNote;
use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::hotbar::{Gained, How};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::planet::column::{GLOW_FLOWER_BIT, GLOW_PICKED_BIT};
use pbd_app::planet::{GpuCell, PLANET_RADIUS, PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::tr;
use pbd_core::daylight::DAY_S;
use pbd_core::flora::{GLOW_PLANT_RECORD, GLOW_PLANT_SCHEMA, GlowPicked, glow_plant_point};
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::records::Record;
use std::sync::Arc;

/// How near the eye ray must pass the plant's stalk or cap to pick it, m
/// (design 3's `glow_pick_radius_m`).
pub const PICK_RADIUS_M: f32 = 0.25;

/// Where a finest record's glowing mushroom stands, if it grows one now: its
/// stalk's foot and cap's top, body-local m. Only on grass, as the flower
/// slot draws it.
fn plant_of(record: &GpuCell, flower_height_m: f32) -> Option<(Vec3, Vec3)> {
    if record.spare[2] & GLOW_FLOWER_BIT == 0 || record.spare[2] & GLOW_PICKED_BIT != 0 {
        return None;
    }
    if !pbd_core::scatter::grassy(record.metadata[1] & 0xff) {
        return None;
    }
    let axis = Vec3::from_slice(&record.direction_height[..3]).try_normalize()?;
    let corners: Vec<Vec3> = record.corners[..record.degree().clamp(1, 6)]
        .iter()
        .map(|c| Vec3::from_slice(&c[..3]))
        .collect();
    Some(glow_plant_point(
        record.metadata[3],
        axis,
        &corners,
        PLANET_RADIUS + record.direction_height[3],
        flower_height_m,
    ))
}

/// How near the ray from `eye` along `look` passes the segment `a`..`b`, and
/// how far along the ray that is.
fn ray_to_segment(eye: Vec3, look: Vec3, a: Vec3, b: Vec3) -> (f32, f32) {
    let ab = b - a;
    let w = eye - a;
    let (aa, ab_l, ll) = (ab.dot(ab), ab.dot(look), look.dot(look));
    let (aw, lw) = (ab.dot(w), look.dot(w));
    let den = aa * ll - ab_l * ab_l;
    let s = if den.abs() > 1e-9 {
        ((ll * aw - ab_l * lw) / den).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let on_seg = a + ab * s;
    let t = (on_seg - eye).dot(look).max(0.0);
    ((eye + look * t - on_seg).length(), t)
}

/// The cell whose glowing mushroom the eye ray passes nearest within reach
/// and [`PICK_RADIUS_M`] (`fruit-aim`: the plant, never its cell's ground).
pub fn on_ray(fine: &PlanetFine, flower_height_m: f32, eye: Vec3, look: Vec3) -> Option<u32> {
    let reach = pbd_core::aim::REACH_M;
    let toward = eye.normalize_or(Vec3::Y);
    let near = ((reach + 4.0) / PLANET_RADIUS).cos();
    let mut best: Option<(f32, u32)> = None;
    for record in fine.set.finest_records() {
        let Some(up) = Vec3::from_slice(&record.direction_height[..3]).try_normalize() else {
            continue;
        };
        if up.dot(toward) < near {
            continue;
        }
        let Some((foot, top)) = plant_of(record, flower_height_m) else {
            continue;
        };
        let (miss, t) = ray_to_segment(eye, look, foot, top);
        if miss <= PICK_RADIUS_M && t <= reach && best.is_none_or(|(bt, _)| t < bt) {
            best = Some((t, record.key()));
        }
    }
    best.map(|(_, key)| key)
}

/// The world day the sun's clock reads.
fn day_of(sun: Option<&pbd_app::sky::Sun>) -> f64 {
    sun.map_or(0.0, |s| s.clock.seconds / f64::from(DAY_S))
}

/// Offer the glowing mushroom aimed at to the interact key: `F  Pick`.
#[allow(clippy::type_complexity)]
pub fn offer(
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    fine: Option<Res<PlanetFine>>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    time: Res<Time>,
    mut interact: ResMut<Interact>,
    mut last: Local<(pbd_app::interact::Recheck, Option<Offer>)>,
) {
    let (Some(fine), Some(scatter)) = (fine, scatter) else {
        return;
    };
    if !walking.is_some_and(|w| w.active) {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let (eye, look) = (
        pbd_app::render_origin::local_of(frame.as_deref(), transform.translation()),
        transform.forward().as_vec3(),
    );
    let (recheck, offered) = &mut *last;
    if recheck.due(eye, look, fine.version, time.delta_secs()) {
        *offered = on_ray(&fine, scatter.flower_height_m, eye, look)
            .map(|_| Offer::new(Act::PickGlow, tr("item.glowing-mushroom.name")));
    }
    if let Some(offer) = offered.clone() {
        interact.offer(offer);
    }
}

/// F on a glowing mushroom: pick it. One `glowing-mushroom` and the
/// `glow-plant` record in one durable line; the plant goes out on this frame,
/// and the field is relit as after a dig.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn pick(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    fine: Option<ResMut<PlanetFine>>,
    sun: Option<Res<pbd_app::sky::Sun>>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    time: Res<Time>,
    (mut save, slots, note, mut gained, interact): (
        Option<ResMut<WorldSave>>,
        Option<ResMut<super::slots::Hotbar>>,
        Option<ResMut<FarmNote>>,
        MessageWriter<Gained>,
        Option<Res<Interact>>,
    ),
) {
    let (Some(scatter), Some(mut slots), Some(mut note)) = (scatter, slots, note) else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyF) || *screen != Screen::Playing {
        return;
    }
    if !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    if !pbd_app::interact::may(interact.as_deref(), Reader::Glow) {
        return;
    }
    let (Some(fine), Some(save)) = (fine, save.as_deref_mut()) else {
        return;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let (eye, look) = (
        pbd_app::render_origin::local_of(frame.as_deref(), transform.translation()),
        transform.forward().as_vec3(),
    );
    let Some(cell) = on_ray(&fine, scatter.flower_height_m, eye, look) else {
        return;
    };
    keys.clear_just_pressed(KeyCode::KeyF);
    let item = Item::Good(Good::GLOWING_MUSHROOM);
    let mut moved = slots.0.clone();
    if moved.give(item, 1) != 0 {
        note.say(time.elapsed_secs(), tr("farm.no_room"));
        return;
    }
    let day = day_of(sun.as_deref());
    let line = Record::of(
        GLOW_PLANT_RECORD,
        u64::from(cell),
        GLOW_PLANT_SCHEMA,
        &GlowPicked { day },
    );
    if !save.store_with_hands(line, GLOW_PLANT_SCHEMA, &moved) {
        error!("glow pick BLOCKED: the save did not accept it");
        return;
    }
    slots.0 = moved;
    gained.write(Gained {
        item,
        count: 1,
        how: How::Picked,
    });
    info!("picked the glowing mushroom at cell {cell} on day {day:.2}");
}

/// Keep the resident set's glowing mushrooms in step with the picks
/// (`glowing-mushrooms` design 4): a pick just taken goes out on its frame,
/// a set that landed from edits taken before a pick has it put out, and a
/// pick past its regrowth grows back where its cell is out of sight. Also
/// hands the build the clock and the distance it judges regrowth by.
#[allow(clippy::too_many_arguments)]
pub fn follow(
    mut fine: Option<ResMut<PlanetFine>>,
    mut contact: Option<ResMut<PlanetContact>>,
    mut save: Option<ResMut<WorldSave>>,
    sun: Option<Res<pbd_app::sky::Sun>>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    columns: Option<Res<pbd_app::config::ColumnSettings>>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    frame: Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    time: Res<Time>,
    mut seen: Local<(u64, u64, f32)>,
) {
    let (Some(fine), Some(contact), Some(save)) = (
        fine.as_deref_mut(),
        contact.as_deref_mut(),
        save.as_deref_mut(),
    ) else {
        return;
    };
    let far_m = scatter.map_or(120.0, |s| s.clutter_radius_m);
    let day = day_of(sun.as_deref());
    let glow = &mut save.edits.glow;
    if glow.far_m != far_m || (glow.day - day).abs() > 1e-3 {
        glow.far_m = far_m;
        glow.day = day;
    }
    let (count, version, wait) = &mut *seen;
    *wait -= time.delta_secs();
    // A pick, a landing, or every few seconds for a regrowth.
    if glow.count() == *count && fine.version == *version && *wait > 0.0 {
        return;
    }
    *wait = 5.0;
    *count = glow.count();
    if glow.count() == 0 {
        *version = fine.version;
        return;
    }
    let Some(eye) = cameras.iter().next().map(|t| {
        pbd_app::render_origin::local_of(frame.as_deref(), t.translation()).normalize_or(Vec3::Y)
    }) else {
        return;
    };
    let chance = columns.map_or(0.01, |c| c.glow_flower_chance);
    let mut set = (*fine.set).clone();
    let changed = set.apply_glow_picks(glow, chance, |direction| {
        direction.angle_between(eye) * PLANET_RADIUS
    });
    if changed > 0 {
        let set = Arc::new(set);
        contact.set_fine(&set);
        fine.set = set;
        fine.version += 1;
        info!("GLOW_PICKS {changed} glowing mushrooms brought round to the picks");
    }
    *version = fine.version;
}

/// `--pick-glow FRAME` (`glowing-mushrooms` 2.3): a pick played with the
/// player's own inputs, as `--pick-fruit` is. Once the fine set stands round
/// the walker, the nearest glowing mushroom within 60 m is chosen; the look
/// turns onto its cap every frame and W walks toward it until it is in
/// reach; then F is pressed on the FRAME-th frame after arriving, and
/// [`pick`] does the rest as it does for a player. Nothing is stored by the
/// script.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn pick_script(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    fine: Option<Res<PlanetFine>>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    mut count: Local<u32>,
    mut target: Local<Option<Vec3>>,
    mut arrived: Local<bool>,
) {
    let Some(press_at) = launch.pick_glow else {
        return;
    };
    if !state.active {
        return;
    }
    state.scripted = true;
    state.captured = true;
    let (Some(fine), Some(scatter)) = (fine, scatter) else {
        return;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    if target.is_none() {
        let under = fine
            .set
            .finest_records()
            .iter()
            .map(|r| {
                let d = Vec3::from_slice(&r.direction_height[..3]).normalize_or(Vec3::Y);
                d.angle_between(eye.normalize_or(Vec3::Y)) * PLANET_RADIUS
            })
            .fold(f32::MAX, f32::min);
        if under > 4.0 {
            return;
        }
        *count += 1;
        if *count < 30 {
            return;
        }
        let best = fine
            .set
            .finest_records()
            .iter()
            .filter_map(|r| plant_of(r, scatter.flower_height_m))
            .map(|(foot, top)| ((top - eye).length(), foot.lerp(top, 0.85)))
            .filter(|(d, _)| *d < 60.0)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((d, cap)) = best else {
            if *count == 30 {
                let records = fine.set.finest_records();
                warn!(
                    "PICK_GLOW no glowing mushroom within 60 m ({} of {} finest records grow one, {} on grass)",
                    records
                        .iter()
                        .filter(|r| r.spare[2] & GLOW_FLOWER_BIT != 0)
                        .count(),
                    records.len(),
                    records
                        .iter()
                        .filter(|r| plant_of(r, scatter.flower_height_m).is_some())
                        .count(),
                );
            }
            return;
        };
        let pbd_app::place::Place { lat, lon, .. } =
            pbd_app::place::Place::of(cap.normalize_or(Vec3::Y));
        info!("PICK_GLOW going to a glowing mushroom {d:.2} m from the eye, at {lat:.5} {lon:.5}");
        *target = Some(cap);
        *count = 0;
    }
    let Some(cap) = *target else {
        return;
    };
    state.look_toward(cap - eye);
    let d = (cap - eye).length();
    if !*arrived {
        *count += 1;
        if d > 2.2 && *count < 900 {
            keys.press(KeyCode::KeyW);
            return;
        }
        keys.release(KeyCode::KeyW);
        *arrived = true;
        *count = 0;
        info!("PICK_GLOW in reach, the cap {d:.2} m from the eye");
    }
    *count += 1;
    if *count == press_at {
        keys.press(KeyCode::KeyF);
        info!("PICK_GLOW pressed F on frame {press_at} in reach");
    } else if *count == press_at + 1 {
        keys.release(KeyCode::KeyF);
    }
}

/// Say what the selected hotbar slot holds, for the carried light and the
/// held view (`glowing-mushrooms` part B and C): only that slot counts, never
/// the pack or another slot.
pub fn hand_item(
    hotbar: Option<Res<super::slots::Hotbar>>,
    mut held: ResMut<pbd_app::carried_light::HeldItem>,
) {
    let now = hotbar.and_then(|h| h.0.held()).map(|stack| stack.item);
    if held.0 != now {
        held.0 = now;
    }
}

/// `--hold <item>`: a capture's instrument (`glowing-mushrooms` part B). The
/// item is put in the selected hotbar slot once, as a player holding it
/// would have it; the capture's save is memory-only.
pub fn hold_script(
    launch: Res<super::Launch>,
    hotbar: Option<ResMut<super::slots::Hotbar>>,
    mut done: Local<bool>,
) {
    let (Some(name), Some(mut hotbar)) = (launch.hold.as_deref(), hotbar) else {
        return;
    };
    if *done {
        return;
    }
    *done = true;
    let Some(item) = pbd_core::station::item_named(name) else {
        warn!("--hold: no item {name}");
        return;
    };
    let selected = hotbar.selected();
    let mut slots = hotbar.0.clone();
    slots.set(selected, Some(pbd_core::inventory::Stack::new(item, 4)));
    hotbar.0 = slots;
    info!("HOLD {name} in slot {selected}");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "Aim at the plant, not the cell": a ray through the stalk or the cap
    /// is within the pick radius, a ray at the cell's ground a metre off is
    /// not.
    #[test]
    fn the_ray_meets_the_plant_not_its_ground() {
        let foot = Vec3::new(0.0, 0.0, -2.0);
        let top = foot + Vec3::Y * 0.35;
        let eye = Vec3::new(0.0, 1.6, 0.0);
        let at_cap = (top - eye).normalize();
        let (miss, t) = ray_to_segment(eye, at_cap, foot, top);
        assert!(miss < 1e-4 && (t - (top - eye).length()).abs() < 1e-3);
        let at_ground = (foot + Vec3::X * 1.0 - eye).normalize();
        let (miss, _) = ray_to_segment(eye, at_ground, foot, top);
        assert!(miss > PICK_RADIUS_M, "a metre off: {miss}");
        let past = (foot + Vec3::X * 0.1 + Vec3::Y * 0.15 - eye).normalize();
        assert!(ray_to_segment(eye, past, foot, top).0 <= PICK_RADIUS_M);
    }
}
