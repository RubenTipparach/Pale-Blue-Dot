//! The water's keys and its capture rig (`water-flow` E5, E7).
//!
//! F5 shows or hides the flow arrows for this session (Tenebris C's F5); F6
//! pours one water source where the place ray lands, through the edit path a
//! placed block takes, so the save records it and the rule starts moving it.
//! A held water block does the same through the ordinary place.

use super::digging::{Edited, Hands, apply_edit, sample_at};
use bevy::prelude::*;
use pbd_app::drops::Drops;
use pbd_app::planet::{PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_app::water_flow::{FlowArrows, WaterStep, run_ticks};
use pbd_core::aim;
use pbd_core::terrain::Material;

/// F5 and F6, on foot with the pointer captured.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut arrows: ResMut<FlowArrows>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut save: ResMut<WorldSave>,
    mut slots: ResMut<super::slots::Hotbar>,
    (mut drops, sun): (ResMut<Drops>, Res<pbd_app::sky::Sun>),
) {
    if keys.just_pressed(KeyCode::F5) {
        arrows.debug = !arrows.debug;
        info!("flow arrows {}", if arrows.debug { "on" } else { "off" });
    }
    if !keys.just_pressed(KeyCode::F6)
        || !walking.is_some_and(|readout| readout.active && readout.captured)
    {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let look = transform.forward().as_vec3();
    pour(
        &mut Edited {
            fine: &mut fine,
            contact: &mut contact,
            save: &mut save,
            slots: &mut slots,
            drops: &mut drops,
            now_s: sun.clock.seconds,
        },
        eye,
        look,
    );
}

/// The ground straight down from `from` (any height over it, unlike the
/// hands' reach): the solid cell met and the open cell over it.
fn drop_ray(world: &Edited, from: Vec3) -> Option<(aim::Sample, aim::Sample)> {
    let down = -from.normalize_or(Vec3::Y);
    let mut above = None;
    for step in 0..800 {
        let point = from + down * (step as f32 * 0.25);
        let Some(sample) = sample_at(world.fine, world.contact, point) else {
            continue;
        };
        if sample.solid {
            return above.map(|a| (sample, a));
        }
        above = Some(sample);
    }
    None
}

/// Pour one source where the ray from `eye` along `look` would place a block;
/// straight down from `eye` from any height when `look` is `None`.
fn pour(world: &mut Edited, eye: Vec3, look: Vec3) -> Option<(u32, usize)> {
    let place = if look == Vec3::ZERO {
        drop_ray(world, eye)?.1
    } else {
        aim::march(eye, look, |point| {
            sample_at(world.fine, world.contact, point)
        })?
        .place?
    };
    apply_edit(
        world,
        Hands::Empty,
        place.cell,
        place.layer,
        Material::Water,
    )?;
    info!(
        "poured water into cell {} layer {}",
        place.cell, place.layer
    );
    Some((place.cell, place.layer))
}

/// `--pour` pours one source where the capture camera looks, then runs
/// `--water-ticks` of the rule at once, so a capture shows water that has
/// had that long to move. `--arrows` turns the overlay on.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut save: ResMut<WorldSave>,
    mut slots: ResMut<super::slots::Hotbar>,
    (mut drops, sun, mut arrows): (ResMut<Drops>, Res<pbd_app::sky::Sun>, ResMut<FlowArrows>),
    refresh: Res<pbd_app::planet::LodRefresh>,
    mut done: Local<bool>,
    mut sources: ResMut<PourSources>,
    mut step: ResMut<WaterStep>,
) {
    // `--water-tick-frac F`: the water frozen F of the way through its last
    // tick, from the first frame, so the rule never ticks on its own.
    if let Some(frac) = launch.water_tick_frac
        && step.hold.is_none()
    {
        step.hold = Some(frac);
    }
    if *done || (launch.capture.is_none() && launch.pour_for.is_none()) {
        return;
    }
    if launch.arrows {
        arrows.debug = true;
    }
    if (launch.pour == 0 && launch.water_ticks == 0 && launch.pit == 0)
        || refresh.in_flight_s().is_some()
    {
        return;
    }
    // After any scripted dig, which runs on the first frame the tier is
    // ready: wait until it has (a dug channel is what the water pours into).
    if launch.dig > 0 && save.edits.is_empty() {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let up = eye.normalize_or(Vec3::Y);
    let mut look = transform.forward().as_vec3();
    let right = transform.right().as_vec3();
    // `--pour-ahead M`: straight down from M metres ahead of the eye (and half a metre
    // over it), past the hands' reach, so a capture can stand back from it.
    let ahead = if launch.pour_ahead > 0.0 {
        let level = (look - up * look.dot(up)).normalize_or(look);
        let from = eye + level * launch.pour_ahead + up * 40.0;
        look = Vec3::ZERO;
        from - eye
    } else {
        Vec3::ZERO
    };
    // `--pit DEPTH`: a pit two cells short of the pour, its cell and the six
    // round it dug DEPTH layers down, for the water to fall into.
    if launch.pit > 0 && ahead != Vec3::ZERO {
        let level = ahead.normalize_or(Vec3::X) - up * ahead.normalize_or(Vec3::X).dot(up);
        // With nothing to pour, the pit is where the pour would be: a pit
        // dug beside the sea, which the sea floods (`volumetric-water`).
        let short = if launch.pour == 0 { 0.0 } else { 5.6 };
        let from = eye + ahead - level.normalize_or(Vec3::X) * short;
        let dug = dig_pit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut save,
                slots: &mut slots,
                drops: &mut drops,
                now_s: sun.clock.seconds,
            },
            from,
            launch.pit,
        );
        if dug == 0 {
            return;
        }
        info!("scripted pit: {dug} cells dug");
    }
    let mut poured = 0;
    for i in 0..launch.pour {
        // One source on the look, more spread across it a cell apart.
        let aside = (i as f32 - (launch.pour as f32 - 1.0) * 0.5) * 2.9;
        let from = eye + ahead + right * aside;
        if let Some(at) = pour(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut save,
                slots: &mut slots,
                drops: &mut drops,
                now_s: sun.clock.seconds,
            },
            from,
            look,
        ) {
            poured += 1;
            sources.0.push(at);
        }
    }
    if launch.pour > 0 && poured == 0 {
        // Before the fine set is under the camera there is nothing to pour
        // on; the next frame tries again.
        return;
    }
    *done = true;
    let changed = run_ticks(
        &mut fine,
        &mut contact,
        &mut save,
        launch.water_ticks as usize,
        &mut step,
    );
    info!(
        "scripted water: {poured} poured, {} ticks run, {changed} cells changed, {} still queued",
        launch.water_ticks,
        fine.set.columns.water.queue.len()
    );
}

/// Dig the cell the ray meets and the cells round it `depth` layers down from
/// each one's own top. Returns how many cells were dug.
fn dig_pit(world: &mut Edited, from: Vec3, depth: u32) -> usize {
    let Some((ground, _)) = drop_ray(world, from) else {
        return 0;
    };
    let Some(record) = super::digging::record_of(world.fine, ground.cell) else {
        return 0;
    };
    let mut cells = vec![record];
    cells.extend(
        world.fine.set.finest_neighbors[record]
            .iter()
            .filter(|&&n| n != u32::MAX)
            .map(|&n| n as usize),
    );
    let mut dug = 0;
    for record in cells {
        let Some(key) = world.fine.set.finest_records().get(record).map(|c| c.key()) else {
            continue;
        };
        let Some(top) = world
            .fine
            .set
            .columns
            .column(record)
            .and_then(|c| c.surface())
        else {
            continue;
        };
        for k in 0..depth as usize {
            let Some(layer) = top.checked_sub(k).filter(|&l| l > 0) else {
                break;
            };
            apply_edit(world, Hands::Empty, key, layer, Material::Air);
        }
        dug += 1;
    }
    dug
}

/// The sources `--pour` poured, as (cell, layer): what `--pour-for` stops
/// and starts again.
#[derive(Resource, Default)]
pub struct PourSources(pub Vec<(u32, usize)>);

/// Seconds after the pour before `POUR_START`: the first flow settles in.
const POUR_SETTLE_S: f32 = 5.0;
/// Seconds each half of the `--pour-for` cycle runs: the sources on, then
/// off, so the water spreads, falls, then drains and dries, over and over.
const POUR_CYCLE_S: f32 = 12.0;

/// `--pour-for SECONDS` (`water-flow` 5.3, `perf_suite.py`'s `pour`): a
/// measurement instrument. From the scripted pour, wait `POUR_SETTLE_S`, log
/// `POUR_START`, then stop the poured sources and start them again every
/// `POUR_CYCLE_S`, so water is always flowing, falling or drying near the
/// eye; after SECONDS log `POUR_DONE` and quit. The edits take the ordinary
/// edit path, as a player's would.
#[allow(clippy::too_many_arguments)]
pub fn pour_cycle(
    launch: Res<super::Launch>,
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut save: ResMut<WorldSave>,
    mut slots: ResMut<super::slots::Hotbar>,
    (mut drops, sun): (ResMut<Drops>, Res<pbd_app::sky::Sun>),
    sources: Res<PourSources>,
    mut exit: MessageWriter<AppExit>,
    mut clock: Local<Option<(std::time::Instant, Option<std::time::Instant>, u32)>>,
) {
    let Some(seconds) = launch.pour_for else {
        return;
    };
    if sources.0.is_empty() {
        return;
    }
    let now = std::time::Instant::now();
    let (poured, started, flips) = clock.get_or_insert((now, None, 0));
    let Some(started) = *started else {
        if now.duration_since(*poured).as_secs_f32() >= POUR_SETTLE_S {
            info!(
                "POUR_START {seconds:.0} s, {} sources, {} cells queued",
                sources.0.len(),
                fine.set.columns.water.queue.len()
            );
            clock.as_mut().unwrap().1 = Some(now);
        }
        return;
    };
    let run = now.duration_since(started).as_secs_f32();
    if run >= seconds {
        info!("POUR_DONE {run:.1} s, {flips} flips");
        exit.write(AppExit::Success);
        return;
    }
    let due = (run / POUR_CYCLE_S) as u32;
    if due <= *flips {
        return;
    }
    *flips = due;
    // Odd flips stop the sources, even ones pour them again.
    let material = if due % 2 == 1 {
        Material::Air
    } else {
        Material::Water
    };
    let mut world = Edited {
        fine: &mut fine,
        contact: &mut contact,
        save: &mut save,
        slots: &mut slots,
        drops: &mut drops,
        now_s: sun.clock.seconds,
    };
    let mut changed = 0;
    for &(cell, layer) in &sources.0 {
        if apply_edit(&mut world, Hands::Empty, cell, layer, material).is_some() {
            changed += 1;
        }
    }
    info!("POUR_FLIP {due} {material:?}: {changed} sources");
}

/// MEASUREMENT (`water-flow` 5.4, the owner's "the despawn is still step
/// based"): `PBD_POUR_STOP_S=<seconds>` in a capture stops the sources
/// `--pour` poured that many seconds of game time after the pour, through
/// the ordinary edit path (a source taken, as a bucket takes one), so a
/// `PBD_CAPTURE_RUN` sequence can film a pour draining frame by frame. In a
/// capture the clock is the fixed step, so the stop lands on the same frame
/// in every run. Nothing of it exists without the variable.
#[allow(clippy::too_many_arguments)]
pub fn pour_stop(
    time: Res<Time>,
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut save: ResMut<WorldSave>,
    mut slots: ResMut<super::slots::Hotbar>,
    (mut drops, sun): (ResMut<Drops>, Res<pbd_app::sky::Sun>),
    sources: Res<PourSources>,
    mut since: Local<Option<f32>>,
    mut stop_s: Local<Option<Option<f32>>>,
) {
    let Some(stop_s) = *stop_s.get_or_insert_with(|| {
        std::env::var("PBD_POUR_STOP_S")
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .filter(|s| s.is_finite() && *s >= 0.0)
    }) else {
        return;
    };
    if sources.0.is_empty() {
        return;
    }
    let Some(elapsed) = since.as_mut() else {
        *since = Some(0.0);
        return;
    };
    if *elapsed > stop_s {
        return;
    }
    *elapsed += time.delta_secs();
    if *elapsed <= stop_s {
        return;
    }
    let mut world = Edited {
        fine: &mut fine,
        contact: &mut contact,
        save: &mut save,
        slots: &mut slots,
        drops: &mut drops,
        now_s: sun.clock.seconds,
    };
    let mut changed = 0;
    for &(cell, layer) in &sources.0 {
        if apply_edit(&mut world, Hands::Empty, cell, layer, Material::Air).is_some() {
            changed += 1;
        }
    }
    info!("POUR_STOP after {elapsed:.2} s: {changed} sources taken");
}
