//! The river cache kept full and kept on disk (`river-cache` design 11).
//!
//! - **The fill.** Every two seconds, the river nearest the camera (within
//!   about 600 m) is found on the drainage map. A thread fills the river
//!   cache along its whole course, from its source to the sea, the first
//!   time: every fine level's cells in the corridor round the river. A
//!   rebuild near any part of it then finds its river terms already worked
//!   out.
//! - **The file.** Every 30 s, while the cache has grown, a thread writes
//!   it to `<saves>/cache/`; and on exit, before the window closes.
//!
//! Both are derived data: nothing here is a record, and a failed write is a
//! log line.

use bevy::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::planet::{FINEST_LEVEL, PLANET_RADIUS, terrain_config};

/// How far either side of a river's line its corridor is filled, m: past the
/// levee's foot (half width, bank, levee and its slope, about 31 m at the
/// widest), with a cell's room.
const CORRIDOR_M: f32 = 40.0;
/// The spacing of the fill's points along the river, m.
const STEP_M: f32 = 25.0;
/// How far round the camera a river is looked for, lattice cells (42 m).
const LOOK_CELLS: i64 = 14;

#[derive(Resource, Default)]
pub struct RiverFill {
    /// The source cells of the rivers filled or being filled.
    filled: HashSet<[u32; 3]>,
    busy: Option<Arc<AtomicBool>>,
    saving: Option<Arc<AtomicBool>>,
    next_look: Option<Instant>,
    next_save: Option<Instant>,
}

pub struct RiverFillPlugin;

impl Plugin for RiverFillPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RiverFill>()
            .add_systems(Update, (fill_rivers, save_now_and_then))
            .add_systems(Last, save_on_exit);
    }
}

/// The points along a river's course, every `STEP_M`, source to sea.
fn course_points(line: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::new();
    for w in line.windows(2) {
        let metres = w[0].angle_between(w[1]) * PLANET_RADIUS;
        let steps = (metres / STEP_M).ceil().max(1.0) as usize;
        for i in 0..steps {
            out.push(w[0].slerp(w[1], i as f32 / steps as f32));
        }
    }
    out.extend(line.last());
    out
}

fn fill_rivers(
    mut fill: ResMut<RiverFill>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    space: crate::SimSpace,
) {
    let now = Instant::now();
    if !pbd_core::river_cache::enabled() || fill.next_look.is_some_and(|t| now < t) {
        return;
    }
    fill.next_look = Some(now + std::time::Duration::from_secs(2));
    if fill
        .busy
        .as_ref()
        .is_some_and(|b| !b.load(Ordering::Relaxed))
    {
        return;
    }
    let cfg = terrain_config();
    let Some(settings) = cfg.rivers.as_ref() else {
        return;
    };
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    let here = space.origin().query_point(camera.translation());
    let up = here.normalize_or(Vec3::Y);
    let course = pbd_core::rivers::map(cfg, settings).course(up, LOOK_CELLS);
    let Some(source) = course.first() else {
        return;
    };
    let key = source.centre.to_array().map(f32::to_bits);
    if !fill.filled.insert(key) {
        return;
    }
    let line: Vec<Vec3> = course.iter().map(|s| s.centre).collect();
    let done = Arc::new(AtomicBool::new(false));
    fill.busy = Some(done.clone());
    std::thread::Builder::new()
        .name("river fill".into())
        .spawn(move || {
            let started = Instant::now();
            let points = course_points(&line);
            let metres = line
                .windows(2)
                .map(|w| w[0].angle_between(w[1]) * PLANET_RADIUS)
                .sum::<f32>();
            let cells = crate::planet::lod::fill_river_cache(&points, CORRIDOR_M);
            info!(
                "RIVER_FILL {cells} cells along {metres:.0} m of river, source to sea, in {:.1} s",
                started.elapsed().as_secs_f32()
            );
            done.store(true, Ordering::Relaxed);
        })
        .ok();
}

fn save(saving: Arc<AtomicBool>) {
    let started = Instant::now();
    let written = pbd_core::rivers::save_caches();
    if written > 0 {
        info!(
            "RIVER_CACHE_SAVED {written} values in {:.2} s",
            started.elapsed().as_secs_f32()
        );
    }
    saving.store(true, Ordering::Relaxed);
}

fn save_now_and_then(mut fill: ResMut<RiverFill>) {
    let now = Instant::now();
    let next = *fill
        .next_save
        .get_or_insert(now + std::time::Duration::from_secs(30));
    if now < next
        || fill
            .saving
            .as_ref()
            .is_some_and(|s| !s.load(Ordering::Relaxed))
    {
        return;
    }
    fill.next_save = Some(now + std::time::Duration::from_secs(30));
    let saving = Arc::new(AtomicBool::new(false));
    fill.saving = Some(saving.clone());
    std::thread::Builder::new()
        .name("river cache save".into())
        .spawn(move || save(saving))
        .ok();
}

fn save_on_exit(mut exits: MessageReader<AppExit>) {
    if exits.read().next().is_some() {
        save(Arc::new(AtomicBool::new(false)));
    }
}

/// The finest level's cell width, m, at `level`.
pub fn cell_m(level: u8) -> f32 {
    2.833 * (1u32 << FINEST_LEVEL.saturating_sub(level) as u32) as f32
}
