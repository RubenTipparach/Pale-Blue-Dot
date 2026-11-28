//! CPU time per frame, a measurement instrument (`play-frame-budget`): the
//! main world's update and the render world's `Render` schedule, with the
//! part of the latter spent taking the swapchain image (`ManageViews`),
//! which is waiting on the GPU and the display rather than CPU work.
//!
//! Bevy renders pipelined, so the two worlds run side by side on different
//! threads: the render world's numbers are handed over through atomics and
//! read a frame late, like the GPU's.
//!
//! locale: dev

use bevy::prelude::*;
use bevy::render::{Render, RenderApp, RenderSystems};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static RENDER_MS: AtomicU64 = AtomicU64::new(0);
static ACQUIRE_MS: AtomicU64 = AtomicU64::new(0);
static SUBMIT_MS: AtomicU64 = AtomicU64::new(0);
static PRESENT_MS: AtomicU64 = AtomicU64::new(0);
/// Microseconds inside Bevy's `submit_graph_commands` and `present_frames`
/// spans since the render world's last frame ended, summed by [`SpanClock`].
static SUBMIT_US: AtomicU64 = AtomicU64::new(0);
static PRESENT_US: AtomicU64 = AtomicU64::new(0);

/// A tracing layer timing Bevy's submit and present spans
/// (`play-frame-budget`): both run inside its `render_system`, where no
/// system can stand between them, and both can block on the GPU, so the
/// render world's own work is its schedule less these and the acquire.
pub struct SpanClock;

struct Entered(Instant);

/// A system span's `name` field (`play-slowdown`, the `system-times`
/// instrument).
#[cfg(feature = "system-times")]
struct SysName(String);

#[cfg(feature = "system-times")]
impl bevy::log::tracing::field::Visit for SysName {
    fn record_debug(
        &mut self,
        field: &bevy::log::tracing::field::Field,
        value: &dyn std::fmt::Debug,
    ) {
        if field.name() == "name" {
            self.0 = format!("{value:?}");
        }
    }
    fn record_str(&mut self, field: &bevy::log::tracing::field::Field, value: &str) {
        if field.name() == "name" {
            self.0 = value.to_string();
        }
    }
}

/// The `system-times` instrument (`play-slowdown`): every system's time,
/// summed by its span, logged every 30 s as `SYSTEM_TIMES` with the
/// entity count. Built only with the feature.
#[cfg(feature = "system-times")]
pub mod system_times {
    use bevy::prelude::*;
    use std::collections::HashMap;
    use std::sync::{LazyLock, Mutex};
    use std::time::Instant;

    /// Microseconds and calls by system name since the last report.
    /// Microseconds, calls and the slowest call's microseconds.
    type Times = HashMap<String, (u64, u64, u64)>;

    pub static TIMES: LazyLock<Mutex<Times>> = LazyLock::new(|| Mutex::new(HashMap::new()));

    /// Mesh assets modified, and those of them over 64 K vertices, since
    /// the last report.
    pub static MESH_CHANGES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    pub static BIG_CHANGES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    pub fn count_mesh_changes(
        mut events: MessageReader<AssetEvent<Mesh>>,
        meshes: Res<Assets<Mesh>>,
    ) {
        for e in events.read() {
            if let AssetEvent::Modified { id } = e {
                MESH_CHANGES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if meshes.get(*id).is_some_and(|m| m.count_vertices() > 65_536) {
                    BIG_CHANGES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }
    }

    pub fn report(world: &mut World, mut last: Local<Option<Instant>>, mut frames: Local<u64>) {
        *frames += 1;
        let now = Instant::now();
        let since = *last.get_or_insert(now);
        if now.duration_since(since).as_secs_f32() < 30.0 {
            return;
        }
        *last = Some(now);
        let n = (*frames).max(1) as f64;
        *frames = 0;
        let taken = TIMES
            .lock()
            .map(|mut m| std::mem::take(&mut *m))
            .unwrap_or_default();
        let mut rows: Vec<_> = taken.into_iter().collect();
        rows.sort_by_key(|r| std::cmp::Reverse(r.1.0));
        let top: Vec<String> = rows
            .iter()
            .filter(|(k, _)| !k.ends_with("run_main_gated"))
            .take(40)
            .map(|(k, (us, calls, _))| {
                format!(
                    "{k}={:.3}ms/{:.1}x",
                    *us as f64 / 1000.0 / n,
                    *calls as f64 / n
                )
            })
            .collect();
        let meshes = world.get_resource::<Assets<Mesh>>().map_or(0, |a| a.len());
        let mats = world
            .get_resource::<Assets<StandardMaterial>>()
            .map_or(0, |a| a.len());
        let images = world.get_resource::<Assets<Image>>().map_or(0, |a| a.len());
        let live = world.query::<Entity>().iter(world).count();
        let changed = MESH_CHANGES.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / n;
        let biggest = BIG_CHANGES.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / n;
        let mut worst: Vec<_> = rows.iter().filter(|(_, v)| v.2 > 50_000).collect();
        worst.sort_by_key(|r| std::cmp::Reverse(r.1.2));
        let worst: Vec<String> = worst
            .iter()
            .take(12)
            .map(|(k, v)| format!("{k}={:.0}", v.2 as f64 / 1000.0))
            .collect();
        info!("SYSTEM_WORST {}", worst.join(" "));
        info!(
            "SYSTEM_TIMES frames={} entities={} meshes={} changed={changed:.1}/frame big={biggest:.1}/frame materials={} images={} {}",
            n,
            live,
            meshes,
            mats,
            images,
            top.join(" ")
        );
    }
}

impl<S> bevy::log::tracing_subscriber::Layer<S> for SpanClock
where
    S: bevy::log::tracing::Subscriber
        + for<'a> bevy::log::tracing_subscriber::registry::LookupSpan<'a>,
{
    #[cfg(feature = "system-times")]
    fn on_new_span(
        &self,
        attrs: &bevy::log::tracing::span::Attributes<'_>,
        id: &bevy::log::tracing::span::Id,
        ctx: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        if attrs.metadata().name() != "system" {
            return;
        }
        let mut name = SysName(String::new());
        attrs.record(&mut name);
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().replace(name);
        }
    }

    fn on_enter(
        &self,
        id: &bevy::log::tracing::span::Id,
        ctx: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        if let Some(span) = ctx.span(id)
            && matches!(
                span.name(),
                "submit_graph_commands" | "present_frames" | "system"
            )
        {
            span.extensions_mut().replace(Entered(Instant::now()));
        }
    }

    fn on_exit(
        &self,
        id: &bevy::log::tracing::span::Id,
        ctx: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        let Some(span) = ctx.span(id) else {
            return;
        };
        #[cfg(feature = "system-times")]
        if span.name() == "system" {
            let mut ext = span.extensions_mut();
            if let Some(Entered(at)) = ext.remove::<Entered>()
                && let Some(SysName(name)) = ext.get_mut::<SysName>()
            {
                let us = at.elapsed().as_micros() as u64;
                if let Ok(mut map) = system_times::TIMES.lock() {
                    let e = map.entry(name.clone()).or_default();
                    e.0 += us;
                    e.1 += 1;
                    e.2 = e.2.max(us);
                }
            }
            return;
        }
        let counter = match span.name() {
            "submit_graph_commands" => &SUBMIT_US,
            "present_frames" => &PRESENT_US,
            _ => return,
        };
        if let Some(Entered(at)) = span.extensions_mut().remove::<Entered>() {
            counter.fetch_add(at.elapsed().as_micros() as u64, Ordering::Relaxed);
        }
    }
}

/// `LogPlugin::custom_layer`: the span clock.
pub fn span_layer(_: &mut App) -> Option<bevy::log::BoxedLayer> {
    Some(Box::new(SpanClock))
}

/// The main world's update, ms, from the start of `First` to the end of
/// `Last` before the frame limiter.
#[derive(Resource, Default)]
pub struct CpuTimes {
    started: Option<Instant>,
    ended: Option<Instant>,
    pub main_ms: f64,
    /// From the end of the last update to the start of this one: the
    /// extraction into the render world and any wait for it.
    pub between_ms: f64,
}

impl CpuTimes {
    /// The render world's newest `Render` schedule, ms.
    pub fn render_ms(&self) -> f64 {
        f64::from_bits(RENDER_MS.load(Ordering::Relaxed))
    }

    /// The part of [`Self::render_ms`] in Bevy's submit, ms.
    pub fn submit_ms(&self) -> f64 {
        f64::from_bits(SUBMIT_MS.load(Ordering::Relaxed))
    }

    /// The part of [`Self::render_ms`] in Bevy's present, ms.
    pub fn present_ms(&self) -> f64 {
        f64::from_bits(PRESENT_MS.load(Ordering::Relaxed))
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
    #[cfg(feature = "system-times")]
    app.add_systems(
        Last,
        (system_times::count_mesh_changes, system_times::report).chain(),
    );
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
    let now = Instant::now();
    if let Some(ended) = times.ended {
        times.between_ms = now.duration_since(ended).as_secs_f64() * 1000.0;
    }
    times.started = Some(now);
}

pub fn main_end(mut times: ResMut<CpuTimes>) {
    let now = Instant::now();
    if let Some(started) = times.started {
        times.main_ms = now.duration_since(started).as_secs_f64() * 1000.0;
    }
    times.ended = Some(now);
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
        let ms = |us: &AtomicU64| us.swap(0, Ordering::Relaxed) as f64 / 1000.0;
        SUBMIT_MS.store(ms(&SUBMIT_US).to_bits(), Ordering::Relaxed);
        PRESENT_MS.store(ms(&PRESENT_US).to_bits(), Ordering::Relaxed);
    }
}

/// With `--frame-log`, every two seconds: each nested span of the shadow pass
/// (`planet_shadow/cascade_N` and its draw's invocations), the newest
/// reading, as `SHADOW_SPANS` (`play-frame-budget`).
pub fn log_shadow_spans(
    store: Option<Res<bevy::diagnostic::DiagnosticsStore>>,
    mut frames: Local<u32>,
) {
    *frames += 1;
    if !(*frames).is_multiple_of(120) {
        return;
    }
    let Some(store) = store else {
        return;
    };
    let mut parts: Vec<String> = store
        .iter()
        .filter_map(|d| {
            let path = d.path().as_str();
            let rest = [
                "render/planet_shadow/",
                "render/planet_terrain/",
                "render/planet_cull/",
            ]
            .iter()
            .find_map(|p| path.strip_prefix(p).map(|r| (p, r)))
            .map(|(p, r)| format!("{}{r}", p[7..p.len() - 1].replace('/', "")))?;
            let value = d.smoothed()?;
            Some(format!("{rest}={value:.3}").replace("planet_shadow", "shadow/"))
        })
        .collect();
    parts.sort();
    info!("SHADOW_SPANS {}", parts.join(" "));
}
