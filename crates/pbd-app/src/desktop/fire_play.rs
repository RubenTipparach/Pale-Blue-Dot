//! Fire in the player's hands (`wildfire` 4.1, 4.3 and 4.4), Survival only:
//!
//! - **F with a lit torch** on a plant lights it: a blaze where it is dry
//!   enough (`pbd_app::wildfire::light`), a small flame that dies in
//!   `damp_flame_s` where it is not; on a placed torch the rain put out, it
//!   lights it again.
//! - **The torch in hand** goes out after `held_out_s` in rain under the
//!   open sky (no light, no flame), and is lit again by F near any fire: a
//!   campfire, a brazier, a burning cell or a lit placed torch.
//! - **Danger**: a burning cell takes `in_cell_per_s` health a second from
//!   a player standing in it and `beside_per_s` beside it, and its warmth is
//!   survival.ron's `wildfire` fire row.

use super::digging::Aim;
use super::menu::Screen;
use super::slots::Hotbar;
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::saves::WorldSave;
use pbd_app::walking::{EYE_HEIGHT, Walker};
use pbd_app::wildfire::{self as fire, FireNow, FireRules, Scars, TorchOut};
use pbd_core::fire::{Plant, blaze, torch};
use pbd_core::inventory::Item;
use pbd_core::locale::Localized;
use pbd_core::survival::temperature::FireNear;
use pbd_core::terrain::Material;

pub struct FirePlayPlugin;

impl Plugin for FirePlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (soak_held, offer, kindle, harm)
                .chain()
                .after(pbd_app::wildfire::show)
                .run_if(pbd_app::world_open),
        );
    }
}

/// How near a fire must be to light the torch in hand, m.
const RELIGHT_M: f64 = 3.0;

/// The player's feet in the planet's frame (f64), and up.
fn feet_of(position: Vec3, space: &pbd_app::SimSpace) -> (DVec3, DVec3) {
    let o = space.origin();
    let p = o.to_world(position).pos;
    (p, p.try_normalize().unwrap_or(DVec3::Y))
}

fn holds_torch(hotbar: &Hotbar) -> bool {
    hotbar.0.held().map(|s| s.item) == Some(Item::Block(Material::Torch))
}

/// The torch in hand soaks in the rain under the open sky, and goes out.
#[allow(clippy::too_many_arguments)]
fn soak_held(
    rules: Res<FireRules>,
    save: Res<WorldSave>,
    hotbar: Res<Hotbar>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    space: pbd_app::SimSpace,
    time: Res<Time>,
    mut out: ResMut<TorchOut>,
) {
    if !save.mode().is_survival() {
        out.held = false;
        return;
    }
    let (Some(fine), Some(air), Ok(position)) = (fine, air, walkers.single()) else {
        return;
    };
    if !holds_torch(&hotbar) || out.held {
        out.soak = torch::Soak::default();
        return;
    }
    let (feet, up) = feet_of(position.0, &space);
    let sample = air.now.sample(up.as_vec3());
    // Open sky: nothing solid in the layers over the head.
    let head_m = feet.length() - f64::from(pbd_app::planet::PLANET_RADIUS) + f64::from(EYE_HEIGHT);
    let open_sky = pbd_core::roads::nearest_key(up.as_vec3())
        .and_then(|key| fine.set.finest_index_of(key))
        .and_then(|record| fine.set.columns.column(record))
        .and_then(|column| {
            let layer = pbd_core::column::layer_at(head_m as f32)? as u16;
            Some(torch::open_sky(&rules.0, layer, |l| {
                (l as usize) < pbd_core::column::LAYERS && column.solid(l as usize)
            }))
        })
        .unwrap_or(true);
    let exposure = torch::Exposure {
        rain_seen: sample.rain_seen,
        snow: sample.snow,
        open_sky,
    };
    let dt = time.delta_secs();
    let mut soak = out.soak;
    if soak.step(&rules.0, exposure, true, dt) {
        out.held = true;
        info!("FIRE the torch in hand is out in the rain");
    }
    out.soak = soak;
}

/// What F does with fire this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Deed {
    /// Light the plant on this cell.
    Plant(u32),
    /// Light the placed torch at this cell and layer.
    Torch(u32, u16),
    /// Light the torch in hand from a fire near.
    Held,
}

/// The fire a press would light, if any.
#[allow(clippy::too_many_arguments)]
fn deed(
    save: &WorldSave,
    hotbar: &Hotbar,
    out: &TorchOut,
    aim: &Aim,
    fine: &pbd_app::planet::PlanetFine,
    now: &FireNow,
    scars: &Scars,
    scatter: Option<&pbd_app::config::ScatterSettings>,
    feet: DVec3,
) -> Option<(Deed, Localized)> {
    if !save.mode().is_survival() || !holds_torch(hotbar) {
        return None;
    }
    if out.held {
        return near_fire(save, fine, now, feet)
            .then(|| (Deed::Held, pbd_core::locale::tr("block.torch.name")));
    }
    let target = aim.target.as_ref()?;
    // A placed torch the rain put out, where the eye meets it.
    for sample in [Some(target.dig), target.place].into_iter().flatten() {
        let layer = sample.layer as u16;
        if torch::is_out(&save.records, sample.cell, layer) {
            return Some((
                Deed::Torch(sample.cell, layer),
                pbd_core::locale::tr("block.torch.name"),
            ));
        }
    }
    let cell = target.dig.cell;
    let record = fine.set.finest_index_of(cell)?;
    let r = &fine.set.finest_records()[record];
    let word = match scars.marked.get(&cell) {
        Some(orig) => (r.metadata[1] & !fire::MARK_BITS) | orig,
        None => r.metadata[1],
    };
    let plant = fire::plant_of(r, word, &fire::odds_of(scatter));
    let thing = match plant {
        Plant::Grass => pbd_core::locale::tr("fire.thing.grass"),
        Plant::Shrub => pbd_core::locale::tr("fire.thing.shrub"),
        p if p.is_tree() => pbd_core::locale::tr("fire.thing.tree"),
        _ => return None,
    };
    if now.flames.iter().any(|f| f.key == cell) {
        return None;
    }
    Some((Deed::Plant(cell), thing))
}

/// A fire within reach of the feet: a burning cell, a campfire or brazier,
/// or a placed torch that is lit.
fn near_fire(
    save: &WorldSave,
    fine: &pbd_app::planet::PlanetFine,
    now: &FireNow,
    feet: DVec3,
) -> bool {
    if now.nearest(feet).is_some_and(|(_, d)| d <= RELIGHT_M) {
        return true;
    }
    save.edits.all().into_iter().any(|e| {
        let burns = matches!(e.material, Material::Campfire | Material::Brazier)
            || (e.material == Material::Torch && !torch::is_out(&save.records, e.cell, e.layer));
        burns
            && fine.set.finest_index_of(e.cell).is_some_and(|i| {
                let r = &fine.set.finest_records()[i];
                let d = Vec3::from_slice(&r.direction_height[..3])
                    .normalize_or(Vec3::Y)
                    .as_dvec3()
                    * (f64::from(pbd_app::planet::PLANET_RADIUS)
                        + f64::from(r.direction_height[3]));
                d.distance(feet) <= RELIGHT_M + 1.0
            })
    })
}

#[allow(clippy::too_many_arguments)]
fn offer(
    save: Res<WorldSave>,
    hotbar: Res<Hotbar>,
    out: Res<TorchOut>,
    aim: Res<Aim>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    now: Res<FireNow>,
    scars: Res<Scars>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    space: pbd_app::SimSpace,
    mut interact: ResMut<Interact>,
) {
    let (Some(fine), Ok(position)) = (fine, walkers.single()) else {
        return;
    };
    let (feet, _) = feet_of(position.0, &space);
    if let Some((_, thing)) = deed(
        &save,
        &hotbar,
        &out,
        &aim,
        &fine,
        &now,
        &scars,
        scatter.as_deref(),
        feet,
    ) {
        interact.offer(Offer::new(Act::Kindle, thing));
    }
}

/// F: light what [`deed`] names.
#[allow(clippy::too_many_arguments)]
fn kindle(
    keys: Res<ButtonInput<KeyCode>>,
    interact: Option<Res<Interact>>,
    screen: Res<Screen>,
    rules: Res<FireRules>,
    mut save: ResMut<WorldSave>,
    hotbar: Res<Hotbar>,
    mut out: ResMut<TorchOut>,
    aim: Res<Aim>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    sun: Option<Res<pbd_app::sky::Sun>>,
    mut now: ResMut<FireNow>,
    scars: Res<Scars>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    space: pbd_app::SimSpace,
) {
    if !keys.just_pressed(KeyCode::KeyF) || *screen != Screen::Playing {
        return;
    }
    if !pbd_app::interact::may(interact.as_deref(), Reader::Fire) {
        return;
    }
    let (Some(fine), Some(air), Some(sun), Ok(position)) = (fine, air, sun, walkers.single())
    else {
        return;
    };
    let (feet, _) = feet_of(position.0, &space);
    let Some((what, _)) = deed(
        &save,
        &hotbar,
        &out,
        &aim,
        &fine,
        &now,
        &scars,
        scatter.as_deref(),
        feet,
    ) else {
        return;
    };
    match what {
        Deed::Held => {
            out.held = false;
            out.soak = torch::Soak::default();
        }
        Deed::Torch(cell, layer) => {
            fire::relight(&mut save, cell, layer);
        }
        Deed::Plant(cell) => {
            let lit = fire::light(
                &rules.0,
                &mut save,
                &fine.set,
                &air.now,
                &scars.marked,
                &fire::odds_of(scatter.as_deref()),
                cell,
                sun.clock.seconds,
                sun.clock.year_fraction(),
            );
            if lit == blaze::Lighting::Damp
                && let Some(i) = fine.set.finest_index_of(cell)
            {
                // A small flame that dies, saving nothing.
                let r = &fine.set.finest_records()[i];
                let at = Vec3::from_slice(&r.direction_height[..3])
                    .normalize_or(Vec3::Y)
                    .as_dvec3()
                    * (f64::from(pbd_app::planet::PLANET_RADIUS)
                        + f64::from(r.direction_height[3]));
                now.damp.push((
                    cell,
                    at,
                    sun.clock.seconds + f64::from(rules.0.torch.damp_flame_s),
                ));
            }
        }
    }
}

/// A burning cell hurts whoever stands in or beside it, a second at a time,
/// and warms them.
#[allow(clippy::too_many_arguments)]
fn harm(
    rules: Res<FireRules>,
    mut save: ResMut<WorldSave>,
    now: Res<FireNow>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    space: pbd_app::SimSpace,
    time: Res<Time>,
    mut survival: ResMut<pbd_app::survival::Survival>,
    mut fires: ResMut<pbd_app::survival::FiresNear>,
    mut owed: Local<f32>,
) {
    if !save.mode().is_survival() || now.flames.is_empty() {
        *owed = 0.0;
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    let (feet, _) = feet_of(position.0, &space);
    if let Some((_, d)) = now.nearest(feet)
        && !fires.0.iter().any(|f| f.source == "wildfire")
    {
        fires.0.push(FireNear {
            source: "wildfire".into(),
            distance_m: d as f32,
        });
    }
    *owed += fire::harm_at(&rules.0, &now, feet) * time.delta_secs();
    if *owed >= 1.0 {
        let pts = owed.floor();
        *owed -= pts;
        pbd_app::survival::hit(
            &mut save,
            &mut survival,
            pbd_core::survival::health::Cause::Fire,
            pts,
        );
    }
}
