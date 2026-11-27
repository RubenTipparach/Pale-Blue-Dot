//! Survival's places at hand (`survival-mode` tasks 4.2, 6.3, 6.6; design
//! "The rest of the non-combat pieces"): F at a still, a bucket's sip, a
//! cactus sipped through a straw, the bedroll laid with the right button and
//! taken up with F, the stills boiling beside their fires, and the laid
//! bedroll drawn as one shaped, textured piece lit as the ground. The rules
//! are `pbd_app::survival::places`'; nothing here acts in Explorer.

use super::digging::{Aim, material_of};
use super::menu::Screen;
use super::slots::Hotbar;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_app::interact::{Act as Verb, Interact, Offer};
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_app::survival::places::{self, StillUse};
use pbd_app::survival::{self as play, Survival};
use pbd_app::walking::{EYE_HEIGHT, HALF_HEIGHT, Walker, WalkingState};
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::locale::tr;
use pbd_core::survival::WaterSource;
use pbd_core::survival::water::Still;
use pbd_core::terrain::Material;
use pbd_server::survival::Act;
use std::collections::HashMap;

/// How far a hand reaches for a cactus or a bedroll, m.
const REACH_M: f32 = 3.0;
/// How near the eye ray passes a cactus's trunk or a bedroll to name it, m.
const NEAR_RAY_M: f32 = 0.6;
/// Seconds between the stills' steps.
const STILL_EVERY_S: f32 = 1.0;

/// The laid bedroll drawn in the world.
#[derive(Component)]
pub struct BedrollShown;

pub struct SurvivalPlacesPlugin;

impl Plugin for SurvivalPlacesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                offer_places.in_set(pbd_app::interact::InteractSet::Offer),
                use_places.after(pbd_app::interact::InteractSet::Resolve),
                lay_bedroll,
                step_stills,
                draw_bedroll,
            )
                .run_if(pbd_app::world_open),
        );
    }
}

/// The eye and the way it looks, in the planet's frame (a query point), and
/// the walker's local eye and up.
struct Eye {
    planet: Vec3,
    look: Vec3,
    local: Vec3,
    up: Vec3,
}

fn eye_of(
    walking: &WalkingState,
    walkers: &Query<&avian3d::prelude::Position, With<Walker>>,
    space: &pbd_app::SimSpace,
) -> Option<Eye> {
    let position = walkers.single().ok()?;
    let o = space.origin();
    let up = o.up_at(position.0);
    let rise = up * (EYE_HEIGHT - HALF_HEIGHT);
    let (heading, pitch) = walking.view();
    Some(Eye {
        planet: o.query_point(position.0) + rise,
        look: heading * pitch.cos() + up * pitch.sin(),
        local: position.0 + rise,
        up,
    })
}

/// How far along a ray from `eye` toward `look` (unit) it passes nearest
/// the segment `a`..`b`, if it passes within `near` of it within `reach`.
fn ray_meets_segment(
    eye: Vec3,
    look: Vec3,
    a: Vec3,
    b: Vec3,
    near: f32,
    reach: f32,
) -> Option<f32> {
    let mut best: Option<f32> = None;
    for k in 0..=8 {
        let p = a.lerp(b, k as f32 / 8.0);
        let along = (p - eye).dot(look);
        if !(0.0..=reach).contains(&along) {
            continue;
        }
        if (eye + look * along).distance(p) <= near && best.is_none_or(|d| along < d) {
            best = Some(along);
        }
    }
    best
}

/// The cactus the eye is on within reach: its cell's key.
fn aimed_cactus(fine: &PlanetFine, contact: &PlanetContact, eye: &Eye) -> Option<u32> {
    let toward = eye.planet.normalize_or(Vec3::Y);
    let cos_near = (8.0 / PLANET_RADIUS).cos();
    let mut best: Option<(f32, u32)> = None;
    for record in fine.set.finest_records() {
        let d = Vec3::from_slice(&record.direction_height[..3]).normalize_or(Vec3::Y);
        if d.dot(toward) < cos_near {
            continue;
        }
        if !pbd_core::tree_kit::grows_cactus(record.spare[0], record.metadata[1]) {
            continue;
        }
        let base = d * contact.sample(d).radius;
        let top = base + d * 2.0;
        if let Some(along) = ray_meets_segment(eye.planet, eye.look, base, top, NEAR_RAY_M, REACH_M)
            && best.is_none_or(|(b, _)| along < b)
        {
            best = Some((along, record.key()));
        }
    }
    best.map(|(_, key)| key)
}

/// Whether the eye is on the laid bedroll within reach.
fn aimed_bedroll(save: &WorldSave, eye: &Eye, space: &pbd_app::SimSpace) -> bool {
    let Some(at) = places::bedroll(save).and_then(|b| b.at) else {
        return false;
    };
    let o = space.origin();
    let centre = o.to_local(at);
    ray_meets_segment(
        eye.local,
        eye.look,
        centre,
        centre + eye.up * 0.1,
        NEAR_RAY_M + 0.4,
        REACH_M,
    )
    .is_some()
}

/// The still the aim is on, its cell and layer.
fn aimed_still(aim: &Aim, fine: &PlanetFine, save: &WorldSave) -> Option<(u32, u16)> {
    let target = aim.target?;
    let block = (target.dig.cell, target.dig.layer);
    (material_of(fine, save, block) == Some(Material::Still))
        .then_some((target.dig.cell, target.dig.layer as u16))
}

/// What the survival places offer F (the still's own offer is the farm's,
/// which names it `UseStill`): a full bucket in hand to drink, a cactus with
/// a straw in hand, the laid bedroll to take up.
#[allow(clippy::too_many_arguments)]
fn offer_places(
    save: Res<WorldSave>,
    walking: Option<Res<WalkingState>>,
    hotbar: Res<Hotbar>,
    fine: Option<Res<PlanetFine>>,
    contact: Option<Res<PlanetContact>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    mut interact: ResMut<Interact>,
    space: pbd_app::SimSpace,
) {
    if !save.mode().is_survival() {
        return;
    }
    let held = hotbar.0.held().map(|s| s.item);
    if let Some(Item::Good(good)) = held
        && (good == Good::WATER_BUCKET || good == Good::SEA_BUCKET)
    {
        interact.offer(Offer::new(Verb::Drink, good.name()));
    }
    let Some(eye) = walking.as_deref().and_then(|w| eye_of(w, &walkers, &space)) else {
        return;
    };
    if aimed_bedroll(&save, &eye, &space) {
        interact.offer(Offer::new(Verb::TakeUpBedroll, Good::BEDROLL.name()));
    }
    if held == Some(Item::Good(Good::STRAW))
        && let (Some(fine), Some(contact)) = (fine, contact)
        && aimed_cactus(&fine, &contact, &eye).is_some()
    {
        interact.offer(Offer::new(Verb::Sip, tr("survival.cactus")));
    }
}

/// A sip's water into the bar through the server, its note said, and a
/// drink counted for the missions where it is fresh.
fn take_sip(
    save: &mut WorldSave,
    survival: &mut Survival,
    did: &mut MessageWriter<pbd_app::missions::Did>,
    sipped: pbd_core::survival::water::Sip,
    source: WaterSource,
) {
    info!("SURVIVAL drink {source:?}: +{}", sipped.water);
    if let Some(note) = sipped.note {
        survival.say(note.text());
    }
    let counts = sipped.counts;
    save.survival_act(Act::Drink(sipped));
    survival.vitals = save.vitals();
    if counts && let Some(name) = pbd_core::missions::drink_name(source) {
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Drink,
            name,
            1,
        ));
    }
}

/// F on what the places offered: the still, the bucket, the cactus, the
/// bedroll.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn use_places(
    keys: Res<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    interact: Option<Res<Interact>>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    mut survival: ResMut<Survival>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    (aim, fine, contact, walking, sun): (
        Res<Aim>,
        Option<Res<PlanetFine>>,
        Option<Res<PlanetContact>>,
        Option<Res<WalkingState>>,
        Res<pbd_app::sky::Sun>,
    ),
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    space: pbd_app::SimSpace,
) {
    if !keys.just_pressed(KeyCode::KeyF) || *screen != Screen::Playing {
        return;
    }
    if !save.mode().is_survival() {
        return;
    }
    let Some(act) = interact
        .as_deref()
        .and_then(|i| i.now.as_ref())
        .map(|o| o.act)
    else {
        return;
    };
    let water_now = save.vitals().map_or(0.0, |v| v.water as f32);
    match act {
        Verb::UseStill => {
            let Some(fine) = fine.as_deref() else { return };
            let Some(at) = aimed_still(&aim, fine, &save) else {
                return;
            };
            match places::use_still(&mut save, &mut hotbar.0, at, water_now) {
                Some(StillUse::Drank(sipped)) => {
                    take_sip(
                        &mut save,
                        &mut survival,
                        &mut did,
                        sipped,
                        WaterSource::Still,
                    );
                }
                Some(StillUse::Poured) => info!("SURVIVAL still poured at {at:?}"),
                Some(StillUse::NeedsFire) => survival.say(tr("survival.note.needs_fire")),
                Some(StillUse::Empty) => survival.say(tr("survival.note.empty")),
                None => {}
            }
        }
        Verb::Drink => {
            let held = hotbar.0.held().map(|s| s.item);
            let salt = held == Some(Item::Good(Good::SEA_BUCKET));
            if let Some(sipped) = places::drink_bucket(&mut save, &mut hotbar.0, water_now) {
                take_sip(
                    &mut save,
                    &mut survival,
                    &mut did,
                    sipped,
                    WaterSource::Bucket { salt },
                );
            }
        }
        Verb::Sip => {
            let (Some(fine), Some(contact), Some(walking)) =
                (fine.as_deref(), contact.as_deref(), walking.as_deref())
            else {
                return;
            };
            let Some(eye) = eye_of(walking, &walkers, &space) else {
                return;
            };
            let Some(key) = aimed_cactus(fine, contact, &eye) else {
                return;
            };
            let day = places::world_day(sun.clock.seconds);
            if let Some((sipped, broke)) =
                places::sip_cactus(&mut save, &mut hotbar.0, key, day, water_now)
            {
                take_sip(
                    &mut save,
                    &mut survival,
                    &mut did,
                    sipped,
                    WaterSource::Cactus,
                );
                if broke {
                    info!("SURVIVAL a straw is used up");
                }
            }
        }
        Verb::TakeUpBedroll if places::take_up_bedroll(&mut save, &mut hotbar.0) => {
            info!("SURVIVAL bedroll taken up");
        }
        _ => {}
    }
}

/// The right button with a bedroll in hand, on foot: lay it on the ground a
/// step ahead, its head away from the player (task 6.3).
#[allow(clippy::too_many_arguments)]
fn lay_bedroll(
    buttons: Res<ButtonInput<MouseButton>>,
    screen: Res<Screen>,
    readout: Option<Res<pbd_app::walking::WalkingReadout>>,
    walking: Option<Res<WalkingState>>,
    contact: Option<Res<PlanetContact>>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    space: pbd_app::SimSpace,
) {
    if *screen != Screen::Playing || !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    if !readout.is_some_and(|w| w.active && w.captured) {
        return;
    }
    if hotbar.0.held().map(|s| s.item) != Some(Item::Good(Good::BEDROLL)) {
        return;
    }
    let (Some(walking), Some(contact), Ok(position)) = (walking, contact, walkers.single()) else {
        return;
    };
    let o = space.origin();
    let up = o.up_at(position.0);
    let (heading, _) = walking.view();
    let Some(ground) = ground_ahead(&contact, &space, position.0, up, heading) else {
        info!("SURVIVAL bedroll refused: no ground ahead");
        return;
    };
    let at = o.to_world(ground);
    if places::lay_bedroll(&mut save, &mut hotbar.0, at, heading) {
        info!("SURVIVAL bedroll laid");
    }
}

/// The ground's top a step ahead of the walker at `centre` (local), within
/// a step up or down: where a bedroll lies.
pub fn ground_ahead(
    contact: &PlanetContact,
    space: &pbd_app::SimSpace,
    centre: Vec3,
    up: Vec3,
    heading: Vec3,
) -> Option<Vec3> {
    let o = space.origin();
    let feet = centre - up * HALF_HEIGHT;
    let start = feet + heading * 1.4 + up * 1.2;
    let mut down = 0.0;
    while down < 3.0 {
        let p = start - up * down;
        if play::solid_at(contact, o.query_point(p)) {
            // Back up to the top of what was met.
            let mut top = p;
            for _ in 0..10 {
                let q = top + up * 0.01;
                if play::solid_at(contact, o.query_point(q)) {
                    top = q;
                } else {
                    break;
                }
            }
            return (down > 0.05).then_some(top + up * 0.01);
        }
        down += 0.05;
    }
    None
}

/// The planet direction of a fine cell, where the tier has it.
fn cell_direction(fine: &PlanetFine, cell: u32) -> Option<Vec3> {
    let i = fine.set.finest_index_of(cell)?;
    let record = fine.set.finest_records().get(i)?;
    Some(Vec3::from_slice(&record.direction_height[..3]).normalize_or(Vec3::Y))
}

/// Once a second of play, each still on the fine tier boils a sip of salt
/// into fresh beside a lit campfire and catches rain under open sky
/// (design 18). What each holds is the save's; the boil's seconds are kept
/// here between steps.
#[allow(clippy::too_many_arguments)]
fn step_stills(
    time: Res<Time>,
    mut save: ResMut<WorldSave>,
    fine: Option<Res<PlanetFine>>,
    contact: Option<Res<PlanetContact>>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    mut every: Local<f32>,
    mut boiling: Local<HashMap<(u32, u16), Still>>,
) {
    if !save.mode().is_survival() {
        boiling.clear();
        return;
    }
    *every += time.delta_secs().min(0.25);
    if *every < STILL_EVERY_S {
        return;
    }
    let dt = std::mem::take(&mut *every);
    let Some(fine) = fine else { return };
    let edits = save.edits.all();
    let stills: Vec<(u32, u16)> = edits
        .iter()
        .filter(|e| e.material == Material::Still)
        .map(|e| (e.cell, e.layer))
        .collect();
    if stills.is_empty() {
        return;
    }
    let fires: Vec<(Vec3, u16)> = edits
        .iter()
        .filter(|e| e.material == Material::Campfire)
        .filter_map(|e| Some((cell_direction(&fine, e.cell)?, e.layer)))
        .collect();
    let cfg = play::rules();
    let reach_m = cfg.still.fire_reach_cells as f32 * places::CELL_M + 0.5;
    for (cell, layer) in stills {
        // A still the tier does not hold now is not boiling: the world is
        // played where the player is.
        let Some(d) = cell_direction(&fine, cell) else {
            continue;
        };
        let lit = fires.iter().any(|&(f, fl)| {
            f.angle_between(d) * PLANET_RADIUS <= reach_m && fl.abs_diff(layer) <= 1
        });
        let raining = match (air.as_deref(), contact.as_deref()) {
            (Some(air), Some(contact)) => {
                let s = air.now.sample(d);
                let top = d * (contact.sample(d).radius + 2.5);
                (s.rain_rate > 0.0 || s.rain_seen > 0.05) && contact.open_to_sky(top)
            }
            _ => false,
        };
        let mut still = boiling
            .remove(&(cell, layer))
            .unwrap_or_else(|| places::still_at(&save, cell, layer));
        // The save's counts win (a sip drunk, a bucket poured); the boil's
        // seconds are ours.
        let saved = places::still_at(&save, cell, layer);
        still.salt_sips = saved.salt_sips;
        still.fresh_sips = saved.fresh_sips;
        if places::step_still(&mut save, (cell, layer), &mut still, lit, raining, dt) {
            info!(
                "SURVIVAL still at {cell}/{layer}: {} salt, {} fresh",
                still.salt_sips, still.fresh_sips
            );
        }
        boiling.insert((cell, layer), still);
    }
}

// ------------------------------------------------------- the bedroll's look

/// The bedroll's size, m: across, along, and its pad's thickness.
const BEDROLL_W: f32 = 0.78;
const BEDROLL_L: f32 = 1.95;
const PAD_M: f32 = 0.05;
/// The rolled head end: its height over the pad, and its length along.
const ROLL_M: f32 = 0.15;
const ROLL_SHARE: f32 = 0.09;

/// The bedroll's height over the ground at `(u, v)` across and along it,
/// 0..1 each (`v` 0 at the head): one surface, the pad's long edges rolled
/// down into the ground, the foot rounded off, and the rolled blanket at
/// the head swelling out of the pad rather than set on it.
pub fn bedroll_height(u: f32, v: f32) -> f32 {
    let s = (2.0 * u - 1.0).abs();
    let smooth = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    // The pad's long edge rolls down over its last few centimetres; the
    // head's roll tapers into it over a wider band, as a rolled blanket's
    // ends do.
    let across = smooth((1.0 - s) / 0.12);
    let roll_across = smooth((1.0 - s) / 0.35);
    let foot = ((1.0 - v) / 0.04).clamp(0.0, 1.0);
    let foot = foot * foot * (3.0 - 2.0 * foot);
    let head_edge = (v / 0.02).clamp(0.0, 1.0);
    let r = ((v - ROLL_SHARE) / ROLL_SHARE).clamp(-1.0, 1.0);
    let roll = ROLL_M * (1.0 - r * r).max(0.0) * roll_across;
    across * PAD_M * foot * head_edge + roll
}

/// The laid bedroll's mesh, in its own frame: x across, y up, z along with
/// the head at -z. Normals are smooth over the whole surface (worked out
/// from the height's own slope); UVs run once across and twice along, so
/// the hide binding is at the long edges and the wool's weave repeats.
pub fn bedroll_mesh() -> Mesh {
    const NX: u32 = 14;
    const NZ: u32 = 48;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let du = 1.0 / NX as f32;
    let dv = 1.0 / NZ as f32;
    for j in 0..=NZ {
        for i in 0..=NX {
            let (u, v) = (i as f32 * du, j as f32 * dv);
            let h = bedroll_height(u, v);
            positions.push([(u - 0.5) * BEDROLL_W, 0.008 + h, (v - 0.5) * BEDROLL_L]);
            let e = 1e-3;
            let dhdx = (bedroll_height((u + e).min(1.0), v) - bedroll_height((u - e).max(0.0), v))
                / (2.0 * e * BEDROLL_W);
            let dhdz = (bedroll_height(u, (v + e).min(1.0)) - bedroll_height(u, (v - e).max(0.0)))
                / (2.0 * e * BEDROLL_L);
            normals.push(Vec3::new(-dhdx, 1.0, -dhdz).normalize().to_array());
            uvs.push([u, v * 2.0]);
        }
    }
    let mut indices = Vec::new();
    let row = NX + 1;
    for j in 0..NZ {
        for i in 0..NX {
            let a = j * row + i;
            let (b, c, d) = (a + 1, a + row, a + row + 1);
            indices.extend([a, c, b, b, c, d]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

/// The laid bedroll, drawn where it lies (render frame from its
/// body-relative f64 point, as a bag is), lit by the field as the ground.
#[allow(clippy::too_many_arguments)]
fn draw_bedroll(
    mut commands: Commands,
    save: Res<WorldSave>,
    frame: Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut look: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
    mut shown: Query<(Entity, &mut Transform), With<BedrollShown>>,
) {
    let laid = places::bedroll(&save).and_then(|b| Some((b.at?, b.facing)));
    let (Some((at, facing)), Some(frame)) = (laid, frame) else {
        for (entity, _) in &shown {
            commands.entity(entity).despawn();
        }
        return;
    };
    let up64 = at.pos.normalize_or(bevy::math::DVec3::Y);
    let up = up64.as_vec3();
    let facing = Vec3::from_array(facing);
    let along = (facing - up * facing.dot(up)).normalize_or(up.any_orthonormal_vector());
    let transform =
        Transform::from_translation((frame.center + at.pos).as_vec3()).looking_to(along, up);
    if let Some((_, mut t)) = shown.iter_mut().next() {
        *t = transform;
        return;
    }
    let (mesh, material) = look
        .get_or_insert_with(|| {
            (
                meshes.add(bedroll_mesh()),
                materials.add(StandardMaterial {
                    base_color_texture: Some(pbd_app::towns::texture(&assets, "bedroll")),
                    perceptual_roughness: 0.95,
                    ..default()
                }),
            )
        })
        .clone();
    info!("SURVIVAL bedroll drawn");
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        BedrollShown,
        pbd_app::field_light::LitByField,
        pbd_app::field_light::LitLikeTerrain,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One shape: the pad's long edges meet the ground, the head's roll
    /// rises out of the pad, and the surface has no step anywhere (its
    /// normals are smooth).
    #[test]
    fn the_bedroll_is_one_shaped_piece() {
        assert!(
            bedroll_height(0.0, 0.5) < 1e-3,
            "the long edge meets the ground"
        );
        assert!(bedroll_height(1.0, 0.5) < 1e-3);
        let pad = bedroll_height(0.5, 0.5);
        assert!((pad - PAD_M).abs() < 1e-3, "{pad}");
        let roll = bedroll_height(0.5, ROLL_SHARE);
        assert!(roll > pad + 0.09, "the head is rolled: {roll}");
        for j in 0..200 {
            for i in 0..50 {
                let (u, v) = (i as f32 / 50.0, j as f32 / 200.0);
                let a = bedroll_height(u, v);
                let b = bedroll_height(u + 0.02, v);
                let c = bedroll_height(u, v + 0.005);
                assert!(
                    (a - b).abs() < 0.04 && (a - c).abs() < 0.04,
                    "a step at {u} {v}"
                );
            }
        }
        let mesh = bedroll_mesh();
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }

    #[test]
    fn a_ray_names_a_segment_it_passes() {
        let eye = Vec3::ZERO;
        let look = Vec3::Z;
        let met = ray_meets_segment(
            eye,
            look,
            Vec3::new(0.3, -1.0, 2.0),
            Vec3::new(0.3, 1.0, 2.0),
            0.6,
            3.0,
        );
        assert!(met.is_some_and(|d| (d - 2.0).abs() < 0.01));
        assert!(
            ray_meets_segment(
                eye,
                look,
                Vec3::new(2.0, 0.0, 2.0),
                Vec3::new(2.0, 1.0, 2.0),
                0.6,
                3.0
            )
            .is_none()
        );
    }
}
