//! The loading screen (`loading-screen`): drawn from the first frame until the
//! world is playable.
//!
//! Two parts. While the world is **prepared** (decision 1), a thread reads or
//! makes the sites and towns, installs the towns' ground and builds the
//! planet once; the frame's own schedules are held back (decision 2), because
//! every game system takes the planet as given, and only `First`, [`Loading`],
//! `PostUpdate` and `Last` run, enough to draw the screen. Then the world
//! **warms** (decision 3): `Startup` runs, the game plays under the opaque
//! screen, the camera looks round once, and the screen lifts when the
//! pipeline cache has had nothing waiting for a while and the towns in range
//! stand. A `READY` line marks the end (decision 4).

use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use bevy::app::MainScheduleOrder;
use bevy::camera::visibility::VisibilitySystems;
use bevy::ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy::prelude::*;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache, PipelineDescriptor};
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::transform::TransformSystems;
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::settlement::ground::{self, Ground};
use pbd_core::sites::{Site, SitesConfig};

use crate::config::{ColumnSettings, ScatterSettings};
use crate::planet::PreparedPlanet;
use crate::saves::WorldSave;
use crate::sites::{SitesRules, WorldSites};
use crate::towns::{Opened, TownAssets, Towns};
pub use crate::{WorldLoading, world_open};

/// Frames the world must run behind the screen, whatever else is done.
const MIN_WARM_FRAMES: u32 = 20;
/// Frames in a row the pipeline cache must have had nothing waiting.
const QUIET_FRAMES: u32 = 15;
/// The look round starts at this warm frame.
const LOOK_FROM: u32 = 2;
/// The look round: four headings, each held this many frames.
const LOOK_FRAMES: u32 = 3;
/// Frames held still after the look round, so the clouds' and reflections'
/// history is of the view the player will see.
const STILL_FRAMES: u32 = 30;
/// The screen lifts after this long behind it whatever is still waiting.
const WARM_CAP: Duration = Duration::from_secs(90);
/// The planet build's share of the bar fills against this, the last
/// measured build; it reports no progress of its own.
const PLANET_GUESS_S: f32 = 5.5;
/// Blocks in the bar.
const BLOCKS: usize = 32;

const INK: Color = Color::srgb(0.88, 0.94, 0.91);
const MINT: Color = Color::srgb(0.48, 0.8, 0.77);
const DIM: Color = Color::srgba(0.55, 0.75, 0.74, 0.18);
const EDGE: Color = Color::srgba(0.55, 0.75, 0.74, 0.6);
const NIGHT: Color = Color::srgb(0.002, 0.004, 0.012);

/// The schedule a loading frame runs between `First` and `PostUpdate`.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Loading;

/// Present while the world warms behind the screen.
#[derive(Resource)]
pub struct Warming {
    started: Instant,
    frames: u32,
    quiet: u32,
    /// Whether the world clock ran before the screen held it; `None` until
    /// the first warm frame holds it.
    clock_ran: Option<bool>,
}

/// Whether the loading screen has lifted (or there never was one).
pub fn ready(loading: Option<Res<WorldLoading>>, warming: Option<Res<Warming>>) -> bool {
    loading.is_none() && warming.is_none()
}

/// The step in hand, for the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Step {
    Reading = 0,
    Surveying,
    Laying,
    Ground,
    Planet,
    Shaders,
    Standing,
}

impl Step {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Reading,
            1 => Self::Surveying,
            2 => Self::Laying,
            3 => Self::Ground,
            4 => Self::Planet,
            5 => Self::Shaders,
            _ => Self::Standing,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Reading => "READING THE WORLD",
            Self::Surveying => "SURVEYING SITES",
            Self::Laying => "LAYING OUT TOWNS",
            Self::Ground => "THE TOWNS' GROUND",
            Self::Planet => "BUILDING THE PLANET",
            Self::Shaders => "COMPILING SHADERS",
            Self::Standing => "STANDING THE TOWNS",
        }
    }

    /// Where the step starts on the bar, and how much of it the step fills.
    fn span(self) -> (f32, f32) {
        match self {
            Self::Reading => (0.0, 0.03),
            Self::Surveying => (0.03, 0.12),
            Self::Laying => (0.15, 0.1),
            Self::Ground => (0.25, 0.05),
            Self::Planet => (0.3, 0.4),
            Self::Shaders => (0.7, 0.25),
            Self::Standing => (0.95, 0.05),
        }
    }
}

/// What the loading thread is doing, read by the screen.
#[derive(Default)]
pub struct Progress {
    step: AtomicU8,
    since: Mutex<Option<Instant>>,
}

impl Progress {
    pub fn set(&self, step: Step) {
        self.step.store(step as u8, Ordering::Release);
        *self.since.lock().unwrap_or_else(|e| e.into_inner()) = Some(Instant::now());
    }

    pub fn step(&self) -> Step {
        Step::from_u8(self.step.load(Ordering::Acquire))
    }

    fn seconds(&self) -> f32 {
        self.since
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .map_or(0.0, |t| t.elapsed().as_secs_f32())
    }
}

/// What the loading thread is given: everything it reads besides the save.
#[derive(Clone)]
pub struct Inputs {
    pub rules: SitesConfig,
    pub assets: TownAssets,
    pub columns: ColumnSettings,
    pub scatter: ScatterSettings,
    /// Where the planet is built round: the flight view's spawn direction,
    /// as `create_planet` builds it.
    pub near: Vec3,
}

/// What the loading thread made.
pub struct Prepared {
    pub save: WorldSave,
    /// The world's sites, or `None` where it gets none.
    pub sites: Option<Vec<Site>>,
    /// The towns, read or laid and stored, their ground installed; `None`
    /// where there are no sites.
    pub towns: Option<Towns>,
    pub planet: PreparedPlanet,
}

/// Wait for the writer's mark to pass `seq`. False when the save failed or
/// the disk did not answer in a minute.
fn committed(save: &WorldSave, seq: u64) -> bool {
    let started = Instant::now();
    while save.committed() < seq {
        if save.failure().is_some() {
            return false;
        }
        if started.elapsed() > Duration::from_secs(60) {
            error!("the save did not commit line {seq} in a minute");
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    save.failure().is_none()
}

/// Prepare a world as its opening does (`loading-screen` decision 1): its
/// sites read or surveyed and stored, its towns read or laid and stored,
/// their ground installed, and the planet built once with it. The same
/// functions `keep_sites`, `build_towns` and `settle` call, in the same
/// order, so the save is written as it was.
pub fn prepare(mut save: WorldSave, inputs: Inputs, progress: &Progress) -> Prepared {
    let started = Instant::now();
    let config: TerrainConfig = *crate::planet::terrain_config();
    progress.set(Step::Reading);
    let mut sites = crate::sites::stored(&save);
    if let Some(list) = &sites {
        info!("{} sites read from the save", list.len());
    } else if save.identity.generator != crate::planet::generator_version() {
        warn!(
            "the world is generator {} and the planet {}; no sites are made",
            save.identity.generator,
            crate::planet::generator_version()
        );
    } else {
        progress.set(Step::Surveying);
        let surveyed = Instant::now();
        let list = pbd_core::sites::generate(
            &inputs.rules,
            &config,
            crate::sites::list_spawn(),
            crate::sites::threads(),
        );
        info!(
            "the world's sites surveyed: {} in {:.1} s",
            list.sites.len(),
            surveyed.elapsed().as_secs_f32()
        );
        for (kind, missing) in &list.shortfall {
            info!("{missing} fewer {} than the rules ask for", kind.name());
        }
        let generator = save.identity.generator;
        match crate::sites::store(&mut save, &list, &inputs.rules, generator) {
            Some(seq) if committed(&save, seq) => {
                info!("{} sites are in the save", list.sites.len());
                sites = Some(list.sites);
            }
            _ => error!("the world's sites could not be saved; none are shown"),
        }
    }
    ground::install(None);
    let towns = sites.as_ref().map(|list| {
        progress.set(Step::Laying);
        let ids: Vec<u32> = list.iter().map(|s| s.id).collect();
        let Opened {
            mut held,
            waiting,
            wait,
            unsettled,
        } = crate::towns::read_or_lay(&mut save, list, &inputs.assets, &config);
        if wait > 0 {
            if committed(&save, wait) {
                held.extend(crate::towns::read_back(&save, waiting));
            } else {
                error!(
                    "the towns were not saved, so none is shown: {}",
                    save.failure().unwrap_or_default()
                );
                held.clear();
            }
        }
        progress.set(Step::Ground);
        let (grounds, kept) = crate::towns::grounds_of(held, &inputs.assets, &config);
        if !grounds.is_empty() {
            let count = grounds.len();
            ground::install(Some(Ground::new(config, grounds)));
            info!("{count} towns' ground installed before the planet is built");
        }
        Towns::opened(ids, kept, unsettled)
    });
    progress.set(Step::Planet);
    let built = Instant::now();
    let planet = PreparedPlanet::build(inputs.near, &inputs.columns, &inputs.scatter, &save.edits);
    info!(
        "the planet built once, with the towns' ground, in {:.2} s; the world prepared in {:.2} s",
        built.elapsed().as_secs_f32(),
        started.elapsed().as_secs_f32()
    );
    Prepared {
        save,
        sites,
        towns,
        planet,
    }
}

/// The loading thread and what it reports.
#[derive(Resource)]
struct Preparing {
    thread: Option<JoinHandle<Prepared>>,
    progress: Arc<Progress>,
    started: Instant,
    world_s: f32,
}

/// The pipeline cache as the render world last saw it.
#[derive(Default)]
struct WatchState {
    total: AtomicU32,
    waiting: AtomicU32,
    /// Set at `READY`: from then on new pipelines are logged by name.
    ready: AtomicBool,
}

/// Shared between the main and the render world.
#[derive(Resource, Clone, Default)]
pub struct PipelineWatch(Arc<WatchState>);

/// Photograph the loading screen itself, then exit (`--capture-loading`).
#[derive(Resource, Clone)]
pub struct CaptureLoading(pub std::path::PathBuf);

/// The frame number, counted in `First`: the frame log's `frame` column.
#[derive(Resource, Default)]
pub struct FrameCount(pub u64);

/// When the process started, for the `READY` line.
#[derive(Resource, Clone, Copy)]
pub struct Launched(pub Instant);

#[derive(Component)]
struct LoadingRoot;
#[derive(Component)]
struct LoadingCamera;
#[derive(Component)]
struct StepText;
#[derive(Component)]
struct DetailText;
#[derive(Component)]
struct Block(usize);

/// The loading screen.
pub struct LoadingPlugin {
    pub launched: Instant,
}

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        let watch = PipelineWatch::default();
        // `Main`, gated: while the world is prepared only the schedules that
        // draw the screen run. The same as Bevy's `Main::run_main` after.
        let mut main = Schedule::new(Main);
        main.set_executor_kind(ExecutorKind::SingleThreaded);
        main.add_systems(run_main_gated);
        app.add_schedule(main);
        let mut loading = Schedule::new(Loading);
        loading.set_executor_kind(ExecutorKind::SingleThreaded);
        app.add_schedule(loading);
        app.insert_resource(Launched(self.launched))
            .init_resource::<WorldLoading>()
            .init_resource::<FrameCount>()
            .insert_resource(watch.clone())
            .add_systems(First, |mut count: ResMut<FrameCount>| count.0 += 1)
            .add_systems(
                Loading,
                (
                    drive_loading,
                    bevy::window::close_when_requested,
                    capture_loading,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                warm.run_if(resource_exists::<Warming>)
                    .before(crate::sky::run_clock),
            )
            .add_systems(
                PostUpdate,
                look_round
                    .run_if(resource_exists::<Warming>)
                    .after(TransformSystems::Propagate)
                    .before(VisibilitySystems::UpdateFrusta),
            );
        if let Some(render) = app.get_sub_app_mut(RenderApp) {
            render
                .insert_resource(watch)
                .add_systems(Render, watch_pipelines.in_set(RenderSystems::Cleanup));
        }
    }
}

/// Bevy's `Main::run_main`, held back while the world is prepared.
fn run_main_gated(world: &mut World, mut started: Local<bool>) {
    if let Some(loading) = world.get_resource::<WorldLoading>() {
        if loading.open_next {
            world.remove_resource::<WorldLoading>();
        } else {
            for label in [
                First.intern(),
                Loading.intern(),
                PostUpdate.intern(),
                Last.intern(),
            ] {
                let _ = world.try_run_schedule(label);
            }
            return;
        }
    }
    if !*started {
        world.resource_scope(|world, order: Mut<MainScheduleOrder>| {
            for &label in &order.startup_labels {
                let _ = world.try_run_schedule(label);
            }
        });
        *started = true;
    }
    world.resource_scope(|world, order: Mut<MainScheduleOrder>| {
        for &label in &order.labels {
            let _ = world.try_run_schedule(label);
        }
    });
}

fn spawn_screen(world: &mut World) {
    let camera = world
        .spawn((
            Camera2d,
            Camera {
                order: 1000,
                clear_color: ClearColorConfig::Custom(NIGHT),
                ..default()
            },
            LoadingCamera,
        ))
        .id();
    let text = |size: f32| TextFont {
        font_size: size,
        ..default()
    };
    world
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(NIGHT),
            GlobalZIndex(i32::MAX),
            UiTargetCamera(camera),
            LoadingRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(14),
                    padding: UiRect::axes(px(36), px(28)),
                    border: UiRect::all(px(1)),
                    ..default()
                },
                BorderColor::all(EDGE),
            ))
            .with_children(|panel| {
                panel.spawn((Text::new("PALE BLUE DOT"), text(26.0), TextColor(INK)));
                panel.spawn((
                    Text::new(Step::Reading.label()),
                    text(14.0),
                    TextColor(MINT),
                    StepText,
                ));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(2),
                        ..default()
                    })
                    .with_children(|bar| {
                        for i in 0..BLOCKS {
                            bar.spawn((
                                Node {
                                    width: px(10),
                                    height: px(14),
                                    ..default()
                                },
                                BackgroundColor(DIM),
                                Block(i),
                            ));
                        }
                    });
                panel.spawn((
                    Text::new(""),
                    text(11.0),
                    TextColor(INK.with_alpha(0.6)),
                    DetailText,
                ));
            });
        });
}

/// How many of the bar's blocks a step `fraction` of the way through fills.
fn filled(step: Step, fraction: f32) -> usize {
    let (start, share) = step.span();
    ((start + share * fraction.clamp(0.0, 1.0)) * BLOCKS as f32).round() as usize
}

/// Draw the screen: the step, the bar and the detail line.
fn paint(world: &mut World, step: Step, fraction: f32, detail: String) {
    let filled = filled(step, fraction);
    let mut steps = world.query_filtered::<&mut Text, With<StepText>>();
    for mut text in steps.iter_mut(world) {
        if text.0 != step.label() {
            text.0 = step.label().into();
        }
    }
    let mut details = world.query_filtered::<&mut Text, With<DetailText>>();
    for mut text in details.iter_mut(world) {
        if text.0 != detail {
            text.0.clone_from(&detail);
        }
    }
    let mut blocks = world.query::<(&Block, &mut BackgroundColor)>();
    for (block, mut colour) in blocks.iter_mut(world) {
        let want = if block.0 < filled { MINT } else { DIM };
        if colour.0 != want {
            colour.0 = want;
        }
    }
}

/// While the world is prepared: start the thread, show what it is doing,
/// and when it is done put what it made in place for `Startup`.
fn drive_loading(world: &mut World) {
    if !world.contains_resource::<Preparing>() {
        spawn_screen(world);
        let save = world
            .remove_resource::<WorldSave>()
            .unwrap_or_else(WorldSave::memory_only);
        let inputs = Inputs {
            rules: world.resource::<SitesRules>().0.clone(),
            assets: world.resource::<TownAssets>().clone(),
            columns: world.resource::<ColumnSettings>().clone(),
            scatter: world.resource::<ScatterSettings>().clone(),
            near: world
                .resource::<crate::flight_view::FlightViewConfig>()
                .spawn_direction,
        };
        let progress = Arc::new(Progress::default());
        let thread = {
            let progress = progress.clone();
            std::thread::Builder::new()
                .name("loading".into())
                .spawn(move || prepare(save, inputs, &progress))
                .expect("the loading thread")
        };
        info!("loading: the world is being prepared");
        world.insert_resource(Preparing {
            thread: Some(thread),
            progress,
            started: Instant::now(),
            world_s: 0.0,
        });
        return;
    }
    let (step, seconds, done, total) = {
        let p = world.resource::<Preparing>();
        (
            p.progress.step(),
            p.progress.seconds(),
            p.thread.as_ref().is_some_and(JoinHandle::is_finished),
            p.started.elapsed().as_secs_f32(),
        )
    };
    let fraction = if step == Step::Planet {
        (seconds / PLANET_GUESS_S).min(0.95)
    } else {
        0.5
    };
    paint(world, step, fraction, format!("{total:.1} s"));
    if !done {
        return;
    }
    let prepared = {
        let mut p = world.resource_mut::<Preparing>();
        p.world_s = p.started.elapsed().as_secs_f32();
        p.thread.take().expect("joined once").join()
    };
    let Prepared {
        save,
        sites,
        towns,
        planet,
    } = match prepared {
        Ok(prepared) => prepared,
        Err(panic) => std::panic::resume_unwind(panic),
    };
    world.insert_resource(WorldSites::opened(&save, sites));
    if let Some(towns) = towns {
        world.insert_resource(towns);
    }
    world.insert_resource(save);
    world.insert_resource(planet);
    // The world clock is held by `warm` from its first frame, not here:
    // `Startup` must see it as the launch set it, or the air opens as a
    // capture's (`Air::in_place`) and steps on the main thread for the rest
    // of the run.
    world.insert_resource(Warming {
        started: Instant::now(),
        frames: 0,
        quiet: 0,
        clock_ran: None,
    });
    world.resource_mut::<WorldLoading>().open_next = true;
    // From here the screen draws on the game's UI camera, above everything.
    let mut cameras = world.query_filtered::<Entity, With<LoadingCamera>>();
    for camera in cameras.iter(world).collect::<Vec<_>>() {
        world.despawn(camera);
    }
    let mut roots = world.query_filtered::<Entity, With<LoadingRoot>>();
    for root in roots.iter(world).collect::<Vec<_>>() {
        world.entity_mut(root).remove::<UiTargetCamera>();
    }
}

/// `--capture-loading`: the screen photographed while the planet builds.
fn capture_loading(
    mut commands: Commands,
    capture: Option<Res<CaptureLoading>>,
    preparing: Option<Res<Preparing>>,
    mut asked: Local<bool>,
) {
    let (Some(capture), Some(preparing)) = (capture, preparing) else {
        return;
    };
    if *asked || preparing.progress.step() != Step::Planet || preparing.progress.seconds() < 1.0 {
        return;
    }
    *asked = true;
    let path = capture.0.clone();
    commands
        .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
        .observe(
            move |event: On<bevy::render::view::screenshot::ScreenshotCaptured>,
                  mut exit: MessageWriter<AppExit>| {
                let saved = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|e| format!("{e:?}"))
                    .and_then(|image| image.to_rgb8().save(&path).map_err(|e| e.to_string()));
                match saved {
                    Ok(()) => println!("CAPTURE_SAVED {}", path.display()),
                    Err(why) => eprintln!("CAPTURE_FAILED {}: {why}", path.display()),
                }
                exit.write(AppExit::Success);
            },
        );
}

/// Which quarter turn the look round is on at warm frame `frames`, or none.
fn look_heading(frames: u32) -> Option<u32> {
    (LOOK_FROM..LOOK_FROM + 4 * LOOK_FRAMES)
        .contains(&frames)
        .then(|| (frames - LOOK_FROM) / LOOK_FRAMES)
}

/// The world warming behind the screen: lift it when the pipelines are
/// quiet, the look round is done and the towns stand.
fn warm(world: &mut World) {
    let watch = world.resource::<PipelineWatch>().0.clone();
    let (total, waiting) = (
        watch.total.load(Ordering::Acquire),
        watch.waiting.load(Ordering::Acquire),
    );
    let towns_settled = world.get_resource::<Towns>().is_none_or(Towns::settled);
    // Hold the world clock from the first warm frame (after `Startup`, before
    // `run_clock`), so loading spends no world time.
    if world.resource::<Warming>().clock_ran.is_none() {
        let ran = world
            .get_resource_mut::<crate::sky::Sun>()
            .is_some_and(|mut sun| std::mem::replace(&mut sun.running, false));
        world.resource_mut::<Warming>().clock_ran = Some(ran);
    }
    let (frames, quiet, elapsed) = {
        let mut warming = world.resource_mut::<Warming>();
        warming.frames += 1;
        warming.quiet = if waiting == 0 { warming.quiet + 1 } else { 0 };
        (warming.frames, warming.quiet, warming.started.elapsed())
    };
    let looked = frames >= LOOK_FROM + 4 * LOOK_FRAMES + STILL_FRAMES;
    let compiling = waiting > 0 || quiet < QUIET_FRAMES || !looked;
    let (step, fraction) = if compiling {
        let done = total.saturating_sub(waiting) as f32 / total.max(1) as f32;
        (Step::Shaders, done)
    } else {
        (Step::Standing, 0.5)
    };
    let world_s = world.get_resource::<Preparing>().map_or(0.0, |p| p.world_s);
    paint(
        world,
        step,
        fraction,
        format!(
            "{} / {total} pipelines   {:.1} s",
            total.saturating_sub(waiting),
            world_s + elapsed.as_secs_f32()
        ),
    );
    let done = frames >= MIN_WARM_FRAMES && !compiling && towns_settled;
    let capped = elapsed > WARM_CAP;
    if !done && !capped {
        return;
    }
    if !done {
        warn!(
            "loading: lifted after {} s with {waiting} pipelines waiting and the towns {}",
            WARM_CAP.as_secs(),
            if towns_settled {
                "standing"
            } else {
                "still standing up"
            }
        );
    }
    let warming = world.remove_resource::<Warming>().expect("warming");
    if warming.clock_ran == Some(true)
        && let Some(mut sun) = world.get_resource_mut::<crate::sky::Sun>()
    {
        sun.running = true;
    }
    let mut roots = world.query_filtered::<Entity, With<LoadingRoot>>();
    for root in roots.iter(world).collect::<Vec<_>>() {
        world.entity_mut(root).despawn();
    }
    if world.contains_resource::<crate::towns::RespawnInTown>()
        && world.contains_resource::<crate::walking::WalkingState>()
    {
        crate::walking::respawn(world);
    }
    watch.ready.store(true, Ordering::Release);
    let launched = world.resource::<Launched>().0;
    let frame = world.resource::<FrameCount>().0;
    world.remove_resource::<Preparing>();
    info!(
        "READY playable {:.2} s after launch at frame {frame} (world {world_s:.2} s, warm {:.2} s, {total} pipelines)",
        launched.elapsed().as_secs_f64(),
        warming.started.elapsed().as_secs_f64()
    );
}

/// Turn the active 3D cameras through four headings behind the screen, so
/// what stands behind the spawn view is queued and its pipelines made too.
/// Only the global transform turns, after propagation and before the frusta:
/// the camera's own transform is never touched, and is marked changed at the
/// end so the next propagation puts it back.
fn look_round(
    warming: Res<Warming>,
    mut base: Local<Vec<(Entity, GlobalTransform)>>,
    mut cameras: Query<(Entity, &Camera, &mut GlobalTransform, &mut Transform), With<Camera3d>>,
) {
    // `warm` counts in `Update`, before this.
    let Some(quarter) = look_heading(warming.frames) else {
        for (entity, at) in base.drain(..) {
            if let Ok((_, _, mut global, mut local)) = cameras.get_mut(entity) {
                *global = at;
                local.set_changed();
            }
        }
        return;
    };
    if base.is_empty() {
        *base = cameras
            .iter()
            .filter(|(_, c, _, _)| c.is_active)
            .map(|(e, _, g, _)| (e, *g))
            .collect();
    }
    let turn = Quat::from_rotation_y(quarter as f32 * std::f32::consts::FRAC_PI_2);
    for (entity, at) in base.iter() {
        if let Ok((_, _, mut global, _)) = cameras.get_mut(*entity) {
            let (scale, rotation, translation) = at.to_scale_rotation_translation();
            *global = GlobalTransform::from(Transform {
                translation,
                rotation: rotation * turn,
                scale,
            });
        }
    }
}

/// The render world's count of pipelines and of those still waiting, and,
/// once the screen has lifted, each new one by name.
fn watch_pipelines(cache: Res<PipelineCache>, watch: Res<PipelineWatch>, mut seen: Local<usize>) {
    let total = cache.pipelines().count();
    // Waiting: queued, being made, or held back for a shader not yet loaded.
    let in_hand = cache
        .pipelines()
        .filter(|p| {
            matches!(
                p.state,
                CachedPipelineState::Queued | CachedPipelineState::Creating(_)
            )
        })
        .count();
    let waiting = in_hand.max(cache.waiting_pipelines().count());
    let w = &watch.0;
    w.total.store(total as u32, Ordering::Release);
    w.waiting.store(waiting as u32, Ordering::Release);
    if w.ready.load(Ordering::Acquire) {
        for pipeline in cache.pipelines().skip(*seen) {
            let label = match &pipeline.descriptor {
                PipelineDescriptor::RenderPipelineDescriptor(d) => d.label.clone(),
                PipelineDescriptor::ComputePipelineDescriptor(d) => d.label.clone(),
            };
            info!(
                "LATE_PIPELINE {}",
                label.as_deref().unwrap_or("(unlabelled)")
            );
        }
    }
    *seen = total;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_look_round_turns_four_quarters_then_stops() {
        assert_eq!(look_heading(0), None);
        assert_eq!(look_heading(LOOK_FROM), Some(0));
        assert_eq!(look_heading(LOOK_FROM + LOOK_FRAMES), Some(1));
        assert_eq!(look_heading(LOOK_FROM + 4 * LOOK_FRAMES - 1), Some(3));
        assert_eq!(look_heading(LOOK_FROM + 4 * LOOK_FRAMES), None);
    }

    #[test]
    fn the_bar_fills_in_step_order_and_ends_full() {
        let steps = [
            Step::Reading,
            Step::Surveying,
            Step::Laying,
            Step::Ground,
            Step::Planet,
            Step::Shaders,
            Step::Standing,
        ];
        let mut end = 0.0;
        for step in steps {
            let (start, share) = step.span();
            assert!(
                (start - end).abs() < 1e-6,
                "{step:?} starts where the last ended"
            );
            end = start + share;
            assert_eq!(Step::from_u8(step as u8), step);
            assert!(filled(step, 0.0) <= filled(step, 1.0));
        }
        assert!((end - 1.0).abs() < 1e-6);
        assert_eq!(filled(Step::Standing, 1.0), BLOCKS);
    }
}
