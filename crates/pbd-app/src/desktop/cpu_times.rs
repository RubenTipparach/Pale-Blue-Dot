//! CPU time per frame, a measurement instrument (`play-frame-budget`): the
//! main world's update and the render world's `Render` schedule, with the
//! part of the latter spent taking the swapchain image (`ManageViews`),
//! which is waiting on the GPU and the display rather than CPU work.
//!
//! Bevy renders pipelined, so the two worlds run side by side on different
//! threads: the render world's numbers are handed over through atomics and
//! read a frame late, like the GPU's.

use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSystems};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static RENDER_MS: AtomicU64 = AtomicU64::new(0);
static ACQUIRE_MS: AtomicU64 = AtomicU64::new(0);

/// The main world's update, ms, from the start of `First` to the end of
/// `Last` before the frame limiter.
#[derive(Resource, Default)]
pub struct CpuTimes {
    started: Option<Instant>,
    pub main_ms: f64,
}

impl CpuTimes {
    /// The render world's newest `Render` schedule, ms.
    pub fn render_ms(&self) -> f64 {
        f64::from_bits(RENDER_MS.load(Ordering::Relaxed))
    }

    /// The part of [`Self::render_ms`] spent in `ManageViews`, ms.
    pub fn acquire_ms(&self) -> f64 {
        f64::from_bits(ACQUIRE_MS.load(Ordering::Relaxed))
    }
}

#[derive(Resource, Default)]
struct RenderClock {
    started: Option<Instant>,
    acquire: Option<Instant>,
    acquire_ms: f64,
}

/// Adds the clocks; `main_end` is placed by the caller, just before the
/// frame's own measurement.
pub fn add(app: &mut App) {
    app.init_resource::<CpuTimes>()
        .add_systems(First, main_start);
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.init_resource::<RenderClock>().add_systems(
            Render,
            (
                render_start.before(RenderSystems::ExtractCommands),
                acquire_start
                    .after(RenderSystems::PrepareMeshes)
                    .before(RenderSystems::ManageViews),
                acquire_end
                    .after(RenderSystems::ManageViews)
                    .before(RenderSystems::Queue),
                render_end.after(RenderSystems::PostCleanup),
            ),
        );
    }
}

fn main_start(mut times: ResMut<CpuTimes>) {
    times.started = Some(Instant::now());
}

pub fn main_end(mut times: ResMut<CpuTimes>) {
    if let Some(started) = times.started {
        times.main_ms = started.elapsed().as_secs_f64() * 1000.0;
    }
}

fn render_start(mut clock: ResMut<RenderClock>) {
    clock.started = Some(Instant::now());
}

fn acquire_start(mut clock: ResMut<RenderClock>) {
    clock.acquire = Some(Instant::now());
}

fn acquire_end(mut clock: ResMut<RenderClock>) {
    if let Some(at) = clock.acquire {
        clock.acquire_ms = at.elapsed().as_secs_f64() * 1000.0;
    }
}

fn render_end(clock: Res<RenderClock>) {
    if let Some(started) = clock.started {
        let ms = started.elapsed().as_secs_f64() * 1000.0;
        RENDER_MS.store(ms.to_bits(), Ordering::Relaxed);
        ACQUIRE_MS.store(clock.acquire_ms.to_bits(), Ordering::Relaxed);
    }
}
