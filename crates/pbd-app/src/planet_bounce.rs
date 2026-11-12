//! `lighting-model` L4: the sun's bounce over the column tier, baked off the
//! main thread (`pbd_core::bounce`) and uploaded beside the voxel light; and
//! L7, the lamp bake (`pbd_core::lamp_bake`), the same way.
//!
//! A bake is started when the fine set changes or the sun has moved more than
//! two degrees since the last one; one runs at a time. Until a new set's bake
//! lands, the old field stays drawn where the set kept its anchor (an edit),
//! so a dig does not flicker the bounce; a set laid round a new anchor starts
//! dark and fades its bounce in over half a second when it lands.

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
    /// Whether the fade-in is due (a new anchor), or it shows at once.
    pub fades: bool,
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
    sun: Vec3,
    version: u64,
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
            *field = BounceField {
                version,
                anchor,
                gpu: Arc::new(gpu),
                landed_s: time.elapsed_secs(),
                fades: new_anchor,
            };
            baking.sun = toward;
            baking.version = version;
        }
    }
    if baking.task.is_some() || !lighting.bounce_on() {
        return;
    }
    let toward = sun.direction();
    let moved = baking.sun.angle_between(toward) > 2f32.to_radians();
    if fine.version == baking.version && !moved {
        return;
    }
    let set = fine.set.clone();
    let version = fine.version;
    // The sun's colour after the air at the anchor, as the terrain takes it.
    let e = set.anchor.normalize_or(Vec3::Y).dot(toward);
    let light = Vec3::from_array(pbd_core::sky_light::sun_air(e));
    baking.task = Some(AsyncComputeTaskPool::get().spawn(async move {
        let field = set.columns.bake_bounce(toward, light);
        (
            version,
            set.anchor,
            toward,
            gpu_words(&field, super::column::COLUMN_CAPACITY as usize),
        )
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
) {
    let Some(fine) = fine else {
        return;
    };
    let want = (fine.version, fine.set.columns.light_version());
    if let Some(task) = baking.task.as_mut()
        && let Some((version, anchor, gpu)) = block_on(future::poll_once(task))
    {
        baking.task = None;
        *field = LampBakeField {
            version,
            anchor,
            gpu: Arc::new(gpu),
        };
    }
    if baking.task.is_some() || !lighting.lamp_bake_on() || field.version == want {
        return;
    }
    let set = fine.set.clone();
    baking.task = Some(AsyncComputeTaskPool::get().spawn(async move {
        let baked = set.columns.bake_lamps();
        (
            want,
            set.anchor,
            lamp_gpu_words(&baked, super::column::COLUMN_CAPACITY as usize),
        )
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

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
