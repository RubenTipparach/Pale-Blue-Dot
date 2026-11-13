//! `lighting-model` L4: the sun's bounce over the column tier, baked off the
//! main thread (`pbd_core::bounce`) and uploaded beside the voxel light; and
//! L7, the lamp bake (`pbd_core::lamp_bake`), the same way.
//!
//! A bake is started when the fine set changes or the sun has moved more than
//! two degrees since the last one; one runs at a time. Until a new set's bake
//! lands, the old field stays drawn where the set kept its anchor (an edit),
//! so a dig does not flicker the bounce.
//!
//! A set laid round a new anchor brings both bakes with it (`baked-light-ease`
//! 7.2): they are made in the set's own pool task, after the set, and drawn
//! from the frame it lands. Before, its bounce was dark and its lamps were
//! the flood until bakes begun after the landing came back.

use super::column::ColumnTier;
use super::lod::PlanetFine;
use bevy::{
    prelude::*,
    render::extract_resource::ExtractResource,
    tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future},
};
use std::sync::Arc;

/// The baked field the terrain draws.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct BounceField {
    /// The fine set's version it was baked for, and that set's anchor.
    pub version: u64,
    pub anchor: Vec3,
    /// What goes up: per slot its window's base layer, then the words.
    pub gpu: Arc<Vec<u32>>,
    /// When it landed, seconds of the app's clock: the fade-in's start.
    pub landed_s: f32,
    /// Whether the fade-in is due (no bounce was drawn for this anchor
    /// before it), or it shows at once.
    pub fades: bool,
    /// The way to the sun it was baked for: the next is due two degrees on.
    pub sun: Vec3,
}

/// A bake's answer: the set's version and anchor, the sun it was baked for,
/// and the words that go up.
type Baked = (u64, Vec3, Vec3, Vec<u32>);
/// A lamp bake's answer: the versions it was made from, the set's anchor,
/// and the words that go up.
type LampBaked = ((u64, u64), Vec3, Vec<u32>);

#[derive(Default)]
pub(super) struct Baking {
    task: Option<Task<Baked>>,
}

/// What a set's own task bakes beside it, so the set lands with its light
/// (`baked-light-ease` 7.2).
#[derive(Default)]
pub struct SetLight {
    /// The sun's bounce, and the way to the sun it was baked for.
    pub bounce: Option<(Vec3, Vec<u32>)>,
    /// The lamp bake, and the light version of the set it was baked from.
    pub lamps: Option<(u64, Vec<u32>)>,
}

/// Bake a set's bounce (for the sun toward `sun`, when given) and its lamps
/// (when `lamps`), on the set's own task, so they land with it. A bounce bake
/// of a tier is 4-10 ms and a lamp bake 3-6 ms on a worker (design 1).
pub fn bake_with_set(
    columns: &ColumnTier,
    anchor: Vec3,
    sun: Option<Vec3>,
    lamps: bool,
) -> SetLight {
    SetLight {
        bounce: sun.map(|toward| (toward, bounce_words(columns, anchor, toward))),
        lamps: lamps.then(|| (columns.light_version(), lamp_words(columns))),
    }
}

/// Draw what a landing set brought from the frame it lands, whole: what the
/// set before it drew there was the same light, cell for cell, so there is
/// nothing to fade from and a fade from nothing would be a flash.
pub fn publish_with_set(
    light: SetLight,
    version: u64,
    anchor: Vec3,
    now_s: f32,
    bounce: &mut BounceField,
    lamps: &mut LampBakeField,
) {
    if let Some((sun, gpu)) = light.bounce {
        *bounce = BounceField {
            version,
            anchor,
            gpu: Arc::new(gpu),
            landed_s: now_s,
            fades: false,
            sun,
        };
    }
    if let Some((light_version, gpu)) = light.lamps {
        *lamps = LampBakeField {
            version: (version, light_version),
            anchor,
            gpu: Arc::new(gpu),
            with_set: true,
        };
    }
}

/// Whether the bounce wants baking again: the set changed, or the sun has
/// moved two degrees from the one it was baked for.
fn bounce_due(field: &BounceField, version: u64, toward: Vec3) -> bool {
    field.version != version
        || field.sun == Vec3::ZERO
        || field.sun.angle_between(toward) > 2f32.to_radians()
}

/// A tier's bounce for the sun toward `toward`, packed for the GPU.
fn bounce_words(columns: &ColumnTier, anchor: Vec3, toward: Vec3) -> Vec<u32> {
    // The sun's colour after the air at the anchor, as the terrain takes it.
    let e = anchor.normalize_or(Vec3::Y).dot(toward);
    let light = Vec3::from_array(pbd_core::sky_light::sun_air(e));
    let started = std::time::Instant::now();
    let field = columns.bake_bounce(toward, light);
    // MEASUREMENT (`baked-light-ease`): what a bounce bake costs a worker.
    info!(
        "BAKED_LIGHT bounce bake of {} columns took {:.1} ms",
        columns.columns.len(),
        started.elapsed().as_secs_f64() * 1000.0
    );
    gpu_words(&field, super::column::COLUMN_CAPACITY as usize)
}

/// A tier's lamp bake, packed for the GPU.
fn lamp_words(columns: &ColumnTier) -> Vec<u32> {
    let started = std::time::Instant::now();
    let baked = columns.bake_lamps();
    // MEASUREMENT (`baked-light-ease`): what a lamp bake costs a worker.
    info!(
        "BAKED_LIGHT lamp bake of {} columns, {} lamps took {:.1} ms",
        columns.columns.len(),
        baked.lamps.len(),
        started.elapsed().as_secs_f64() * 1000.0
    );
    lamp_gpu_words(&baked, super::column::COLUMN_CAPACITY as usize)
}

/// Packs a field for the GPU: `capacity` bases (zero past the tier), then
/// the words of every slot.
fn gpu_words(field: &pbd_core::bounce::Field, capacity: usize) -> Vec<u32> {
    let mut out = vec![0u32; capacity];
    for (slot, base) in field.bases.iter().enumerate().take(capacity) {
        out[slot] = *base;
    }
    out.extend_from_slice(&field.words);
    out
}

pub(super) fn update_bounce(
    time: Res<Time>,
    fine: Option<Res<PlanetFine>>,
    sun: Option<Res<crate::sky::Sun>>,
    lighting: Res<crate::config::LightingSettings>,
    mut field: ResMut<BounceField>,
    mut baking: Local<Baking>,
    count: Option<Res<crate::loading::FrameCount>>,
) {
    let (Some(fine), Some(sun)) = (fine, sun) else {
        return;
    };
    if let Some(task) = baking.task.as_mut()
        && let Some((version, anchor, toward, gpu)) = block_on(future::poll_once(task))
    {
        baking.task = None;
        if version == fine.version {
            let new_anchor = anchor.distance(field.anchor) > 1e-3;
            // MEASUREMENT (`baked-light-ease`): when a bounce reaches the GPU.
            info!(
                "BAKED_LIGHT frame={} bounce v{version} new_anchor={new_anchor} sun_moved_deg={:.2}",
                count.map_or(0, |c| c.0),
                field.sun.angle_between(toward).to_degrees()
            );
            *field = BounceField {
                version,
                anchor,
                gpu: Arc::new(gpu),
                landed_s: time.elapsed_secs(),
                fades: new_anchor,
                sun: toward,
            };
        }
    }
    if baking.task.is_some() || !lighting.bounce_on() {
        return;
    }
    let toward = sun.direction();
    if !bounce_due(&field, fine.version, toward) {
        return;
    }
    let set = fine.set.clone();
    let version = fine.version;
    baking.task = Some(AsyncComputeTaskPool::get().spawn(async move {
        let words = bounce_words(&set.columns, set.anchor, toward);
        (version, set.anchor, toward, words)
    }));
}

/// L7: the lamp bake the terrain draws, and what it was baked from.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct LampBakeField {
    /// The fine set's version and its light bake's, and the set's anchor.
    pub version: (u64, u64),
    pub anchor: Vec3,
    /// What goes up (`lamp_gpu_words`).
    pub gpu: Arc<Vec<u32>>,
    /// Whether it came with its set's landing (`publish_with_set`): drawn
    /// whole from that frame, not faded in from the flood.
    pub with_set: bool,
}

/// The lamps a bake holds, as the GPU reads them: four words each.
pub const LAMP_WORDS: usize = 4 * pbd_core::lamp_bake::MAX_LAMPS + 4;
/// One bake on the GPU: its lamps, then `capacity` bases, then the cells.
pub fn lamp_half_words(capacity: usize) -> usize {
    LAMP_WORDS + capacity + capacity * pbd_core::lamp_bake::WORDS
}

fn lamp_gpu_words(field: &pbd_core::lamp_bake::LampField, capacity: usize) -> Vec<u32> {
    let mut out = vec![0u32; LAMP_WORDS + capacity];
    for (i, lamp) in field.lamps.iter().enumerate() {
        let v = [lamp.at.x, lamp.at.y, lamp.at.z, lamp.level];
        for (k, x) in v.iter().enumerate() {
            out[i * 4 + k] = x.to_bits();
        }
    }
    for (slot, base) in field.bases.iter().enumerate().take(capacity) {
        out[LAMP_WORDS + slot] = *base;
    }
    let room = capacity * pbd_core::lamp_bake::WORDS;
    out.extend_from_slice(&field.words[..field.words.len().min(room)]);
    out
}

#[derive(Default)]
pub(super) struct LampBaking {
    task: Option<Task<LampBaked>>,
}

/// Re-bakes the lamps whenever the set or its light changes (a lamp placed,
/// lit or put out, an edit near one), one bake at a time, off the main thread.
pub(super) fn update_lamp_bake(
    fine: Option<Res<PlanetFine>>,
    lighting: Res<crate::config::LightingSettings>,
    mut field: ResMut<LampBakeField>,
    mut baking: Local<LampBaking>,
    count: Option<Res<crate::loading::FrameCount>>,
) {
    let Some(fine) = fine else {
        return;
    };
    let want = (fine.version, fine.set.columns.light_version());
    if let Some(task) = baking.task.as_mut()
        && let Some((version, anchor, gpu)) = block_on(future::poll_once(task))
    {
        baking.task = None;
        // A bake begun on a set older than the one whose bake is drawn now
        // (a landing brought its own while this ran) is dropped: drawn, it
        // would fade the ground back toward a set that is gone.
        if version.0 >= field.version.0 {
            // MEASUREMENT (`baked-light-ease`): when a lamp bake reaches the GPU.
            info!(
                "BAKED_LIGHT frame={} lamp bake v{version:?} new_anchor={} (set v{})",
                count.map_or(0, |c| c.0),
                anchor.distance(field.anchor) > 1e-3,
                fine.version
            );
            *field = LampBakeField {
                version,
                anchor,
                gpu: Arc::new(gpu),
                with_set: false,
            };
        }
    }
    if baking.task.is_some() || !lighting.lamp_bake_on() || field.version == want {
        return;
    }
    let set = fine.set.clone();
    baking.task = Some(
        AsyncComputeTaskPool::get()
            .spawn(async move { (want, set.anchor, lamp_words(&set.columns)) }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `baked-light-ease` 7.2: a set that lands with its own bakes is drawn
    /// with them from its landing frame, whole: its bounce does not fade in
    /// from dark, its lamps do not stand in the flood, and neither waits on a
    /// bake begun after the landing.
    #[test]
    fn a_landing_set_brings_its_bounce_and_lamps() {
        let (mut bounce, mut lamps) = (BounceField::default(), LampBakeField::default());
        let anchor = Vec3::new(0.0, 0.6, 0.8);
        let sun = Vec3::new(0.0, 1.0, 0.0);
        publish_with_set(
            SetLight {
                bounce: Some((sun, vec![7; 4])),
                lamps: Some((3, vec![9; 4])),
            },
            12,
            anchor,
            40.0,
            &mut bounce,
            &mut lamps,
        );
        assert_eq!((bounce.version, bounce.anchor), (12, anchor));
        assert!(!bounce.fades, "drawn whole from the landing frame");
        assert_eq!(bounce.sun, sun);
        assert_eq!(*bounce.gpu, vec![7; 4]);
        assert_eq!((lamps.version, lamps.anchor), ((12, 3), anchor));
        assert!(lamps.with_set, "shown at once, not faded from the flood");
        // Nothing to bake again for that set and sun...
        assert!(!bounce_due(&bounce, 12, sun));
        // ...until the sun has moved two degrees, or the set changes.
        let turned = |deg: f32| Quat::from_rotation_x(deg.to_radians()) * sun;
        assert!(!bounce_due(&bounce, 12, turned(1.5)));
        assert!(bounce_due(&bounce, 12, turned(2.5)));
        assert!(bounce_due(&bounce, 13, sun));
        assert!(bounce_due(&BounceField::default(), 0, sun), "never baked");
        // A set whose task baked nothing (the bake switched off) leaves what
        // is drawn alone.
        let before = bounce.version;
        publish_with_set(
            SetLight::default(),
            13,
            anchor,
            41.0,
            &mut bounce,
            &mut lamps,
        );
        assert_eq!(bounce.version, before);
        assert_eq!(lamps.version, (12, 3));
    }

    /// The terrain reads the field with the layout `gpu_words` writes.
    #[test]
    fn the_shader_reads_the_field_as_it_is_packed() {
        let shader = include_str!("../../../assets/shaders/planet_surface.wgsl");
        for line in [
            format!(
                "const BOUNCE_BASES: u32 = {}u;",
                super::super::column::COLUMN_CAPACITY
            ),
            format!("const BOUNCE_WORDS: u32 = {}u;", pbd_core::bounce::WORDS),
            format!("const BOUNCE_SPAN: u32 = {}u;", pbd_core::bounce::SPAN),
            format!("const BOUNCE_MAX: f32 = {:?};", pbd_core::bounce::MAX),
        ] {
            assert!(
                shader.contains(&line),
                "planet_surface.wgsl should carry `{line}`"
            );
        }
        let field = pbd_core::bounce::Field {
            bases: vec![7, 9],
            words: vec![1; 2 * pbd_core::bounce::WORDS],
        };
        let gpu = gpu_words(&field, 4);
        assert_eq!(&gpu[..4], &[7, 9, 0, 0]);
        assert_eq!(gpu.len(), 4 + 2 * pbd_core::bounce::WORDS);
        for line in [
            format!("const LAMP_LIST_WORDS: u32 = {}u;", LAMP_WORDS),
            format!(
                "const LAMP_HALF_WORDS: u32 = {}u;",
                lamp_half_words(super::super::column::COLUMN_CAPACITY as usize)
            ),
            format!(
                "const LAMP_CELL_WORDS: u32 = {}u;",
                pbd_core::lamp_bake::CELL_WORDS
            ),
            format!(
                "const LAMP_BOUNCE_MAX: f32 = {:?};",
                pbd_core::lamp_bake::BOUNCE_MAX
            ),
            format!(
                "const LAMP_REACH_M: f32 = {:?};",
                pbd_core::lamp_bake::REACH_M
            ),
            format!(
                "const LAMP_BAKE_GAIN: f32 = {:?};",
                pbd_core::lamp_bake::GAIN
            ),
        ] {
            assert!(
                shader.contains(&line),
                "planet_surface.wgsl should carry `{line}`"
            );
        }
    }
}
