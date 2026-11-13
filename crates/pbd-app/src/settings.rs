//! The player's own settings (`player-settings`): a frame limit, a render
//! scale and auto step, kept in a versioned file in the user's config
//! directory.
//!
//! These are the PLAYER's. The tunables in `assets/config/*.ron` are the
//! developer's and stay where they are. The file is not under `saves/`, names
//! no world, and nothing that opens, writes or deletes a world reads it.
//!
//! A capture or a benchmark (`--capture`, `--frame-log`) does not read it
//! either: it takes [`PlayerSettings::default`] and its own flags
//! ([`Overrides`]), so a picture or a perf number is a function of its command
//! line, whatever the player last chose.

use bevy::{
    camera::{ImageRenderTarget, RenderTarget},
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    image::ImageSampler,
    prelude::*,
    render::render_resource::TextureFormat,
    ui::IsDefaultUiCamera,
    window::{PrimaryWindow, WindowRef},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The file's version. A file with none is read as version 0, which had the
/// same fields.
pub const VERSION: u32 = 1;

/// The render scales offered, in percent, in the order a press cycles them.
pub const SCALES: [u32; 5] = [100, 85, 75, 67, 50];

/// The file's name inside the config directory.
const FILE_NAME: &str = "settings.ron";

/// A cap on the frame rate. Off is today's behaviour: whatever vsync, or
/// `--no-vsync`, gives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameLimit {
    #[default]
    Off,
    Fps60,
    Fps120,
}

impl FrameLimit {
    /// The frame period the cap holds, or none.
    pub fn period(self) -> Option<Duration> {
        match self {
            FrameLimit::Off => None,
            FrameLimit::Fps60 => Some(Duration::from_secs_f64(1.0 / 60.0)),
            FrameLimit::Fps120 => Some(Duration::from_secs_f64(1.0 / 120.0)),
        }
    }

    /// The next value a press on the row gives.
    pub fn next(self) -> Self {
        match self {
            FrameLimit::Off => FrameLimit::Fps60,
            FrameLimit::Fps60 => FrameLimit::Fps120,
            FrameLimit::Fps120 => FrameLimit::Off,
        }
    }

    /// From a flag's number: 0 is off.
    pub fn from_fps(fps: u32) -> Option<Self> {
        match fps {
            0 => Some(FrameLimit::Off),
            60 => Some(FrameLimit::Fps60),
            120 => Some(FrameLimit::Fps120),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FrameLimit::Off => "OFF",
            FrameLimit::Fps60 => "60 FPS",
            FrameLimit::Fps120 => "120 FPS",
        }
    }
}

/// The nearest offered scale to `percent`.
pub fn snap_scale(percent: u32) -> u32 {
    *SCALES
        .iter()
        .min_by_key(|s| s.abs_diff(percent))
        .expect("scales offered")
}

/// The next scale a press on the row gives.
pub fn next_scale(percent: u32) -> u32 {
    let at = SCALES
        .iter()
        .position(|s| *s == snap_scale(percent))
        .unwrap_or(0);
    SCALES[(at + 1) % SCALES.len()]
}

/// What the player chose.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerSettings {
    pub frame_limit: FrameLimit,
    /// Percent of the window the 3D view renders at, one of [`SCALES`].
    pub render_scale: u32,
    /// A grounded walker walks up one terrain layer without a key.
    pub auto_step: bool,
}

/// A new player's settings, the owner's choice (2026-11-13, "default to
/// this settings"): 60 fps, 85 %, auto step off.
impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            frame_limit: FrameLimit::Fps60,
            render_scale: 85,
            auto_step: false,
        }
    }
}

impl PlayerSettings {
    /// What a capture or a benchmark runs with before its flags: the engine
    /// as it was before these settings existed (uncapped, full size, auto
    /// step on), so no picture or perf number moved when the player's
    /// defaults did.
    pub const ISOLATED: Self = Self {
        frame_limit: FrameLimit::Off,
        render_scale: 100,
        auto_step: true,
    };
}

/// The file as written. Every field has a default, so a file that lacks one
/// gets that one's default and keeps the rest; a file with no version is
/// version 0.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct File {
    version: u32,
    frame_limit: FrameLimit,
    render_scale: u32,
    auto_step: bool,
}

impl Default for File {
    fn default() -> Self {
        let base = PlayerSettings::default();
        Self {
            version: 0,
            frame_limit: base.frame_limit,
            render_scale: base.render_scale,
            auto_step: base.auto_step,
        }
    }
}

/// How a load went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loaded {
    /// No file: defaults.
    Missing,
    /// Read, at this version.
    Read(u32),
    /// Not a settings file: defaults, and the file left as it was.
    Broken(String),
}

impl Loaded {
    /// Whether a change may be written back. A newer build's file is not:
    /// writing it would drop the fields this build does not know.
    pub fn writable(&self) -> bool {
        match self {
            Loaded::Missing | Loaded::Broken(_) => true,
            Loaded::Read(version) => *version <= VERSION,
        }
    }
}

impl PlayerSettings {
    /// Settings from a file's text, and its version (0 when it names none).
    pub fn parse(text: &str) -> Result<(Self, u32), String> {
        let file: File = ron::from_str(text).map_err(|e| e.to_string())?;
        let settings = Self {
            frame_limit: file.frame_limit,
            render_scale: snap_scale(file.render_scale),
            auto_step: file.auto_step,
        };
        Ok((settings, file.version))
    }

    /// The file's text for these settings, at [`VERSION`].
    pub fn to_text(&self) -> String {
        ron::ser::to_string_pretty(
            &File {
                version: VERSION,
                frame_limit: self.frame_limit,
                render_scale: self.render_scale,
                auto_step: self.auto_step,
            },
            ron::ser::PrettyConfig::default(),
        )
        .expect("settings serialise")
    }

    /// Read the file at `path`. Missing or broken gives defaults; a broken
    /// file is not touched.
    pub fn load_from(path: &Path) -> (Self, Loaded) {
        match std::fs::read_to_string(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                (Self::default(), Loaded::Missing)
            }
            Err(e) => (Self::default(), Loaded::Broken(e.to_string())),
            Ok(text) => match Self::parse(&text) {
                Ok((settings, version)) => (settings, Loaded::Read(version)),
                Err(why) => (Self::default(), Loaded::Broken(why)),
            },
        }
    }

    /// Write the settings to `path`, through a temporary file and a rename,
    /// so a crash mid-write leaves the old file rather than half a new one.
    pub fn keep_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temporary = path.with_extension("ron.tmp");
        std::fs::write(&temporary, self.to_text())?;
        std::fs::rename(&temporary, path)
    }

    /// These settings with a launch's flags laid over them.
    pub fn with(self, overrides: &Overrides) -> Self {
        Self {
            frame_limit: overrides.frame_limit.unwrap_or(self.frame_limit),
            render_scale: overrides.render_scale.map_or(self.render_scale, snap_scale),
            auto_step: overrides.auto_step.unwrap_or(self.auto_step),
        }
    }
}

/// A launch's flags: `--frame-limit`, `--render-scale`, `--no-auto-step`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Overrides {
    pub frame_limit: Option<FrameLimit>,
    pub render_scale: Option<u32>,
    pub auto_step: Option<bool>,
}

/// Where the file lives: `PBD_SETTINGS` if set, else the user's config
/// directory. None when the environment names no home at all.
pub fn settings_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("PBD_SETTINGS") {
        return Some(PathBuf::from(path));
    }
    let var = |name: &str| {
        std::env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let dir = if cfg!(target_os = "windows") {
        var("APPDATA").map(|d| d.join("PaleBlueDot"))
    } else if cfg!(target_os = "macos") {
        var("HOME").map(|h| h.join("Library/Application Support/PaleBlueDot"))
    } else {
        var("XDG_CONFIG_HOME")
            .or_else(|| var("HOME").map(|h| h.join(".config")))
            .map(|d| d.join("pale-blue-dot"))
    };
    dir.map(|d| d.join(FILE_NAME))
}

/// Where this run keeps the player's changes, if anywhere: none in a capture
/// or a benchmark, or for a newer build's file.
#[derive(Resource, Clone, Debug, Default)]
pub struct SettingsFile {
    pub path: Option<PathBuf>,
}

impl SettingsFile {
    /// Keep a change. A failed write is logged and the game goes on: it is a
    /// preference, not a save.
    pub fn keep(&self, settings: &PlayerSettings) {
        let Some(path) = &self.path else {
            info!("settings: changed for this run only");
            return;
        };
        if let Err(error) = settings.keep_to(path) {
            warn!("could not keep {}: {error}", path.display());
        }
    }
}

/// The settings a launch runs with, and where a change is kept. `isolated`
/// is a capture or a benchmark, which reads no file and keeps nothing.
pub fn for_launch(isolated: bool, overrides: &Overrides) -> (PlayerSettings, SettingsFile) {
    if isolated {
        return (
            PlayerSettings::ISOLATED.with(overrides),
            SettingsFile::default(),
        );
    }
    let Some(path) = settings_path() else {
        return (
            PlayerSettings::default().with(overrides),
            SettingsFile::default(),
        );
    };
    let (settings, loaded) = PlayerSettings::load_from(&path);
    match &loaded {
        Loaded::Broken(why) => warn!(
            "settings: {} is not readable ({why}); using defaults, file left as it is",
            path.display()
        ),
        Loaded::Read(v) if *v > VERSION => warn!(
            "settings: {} is from a newer build (version {v}); changes are kept for this run only",
            path.display()
        ),
        _ => {}
    }
    let file = SettingsFile {
        path: loaded.writable().then_some(path),
    };
    (settings.with(overrides), file)
}

/// Holds the main loop to a frame-rate cap. Deadlines are one period apart,
/// so the mean rate is the cap and does not drift by a sleep's error; a frame
/// more than a period late re-anchors them, so the game never runs fast to
/// catch up.
#[derive(Resource, Default, Debug)]
pub struct FramePacer {
    next: Option<Instant>,
}

impl FramePacer {
    /// At the end of a frame, `now`: the instant to hold until, if any.
    pub fn deadline(&mut self, now: Instant, period: Option<Duration>) -> Option<Instant> {
        let Some(period) = period else {
            self.next = None;
            return None;
        };
        match self.next {
            Some(next) if now < next => {
                self.next = Some(next + period);
                Some(next)
            }
            // Late, by less than a period: keep the cadence, no wait.
            Some(next) if now - next <= period => {
                self.next = Some(next + period);
                None
            }
            // The first capped frame, or a hitch: start again from now.
            _ => {
                self.next = Some(now + period);
                None
            }
        }
    }
}

/// Sleep to within 1.5 ms of `deadline`, then spin: a Windows sleep can
/// overshoot by about a millisecond.
pub fn hold_until(deadline: Instant) {
    const SPIN: Duration = Duration::from_micros(1500);
    let now = Instant::now();
    if deadline > now + SPIN {
        std::thread::sleep(deadline - now - SPIN);
    }
    while Instant::now() < deadline {
        std::hint::spin_loop();
    }
}

/// The frame limiter: last thing in a frame. Not in a capture, which steps
/// time itself.
pub fn pace_frames(settings: Res<PlayerSettings>, mut pacer: ResMut<FramePacer>) {
    if let Some(deadline) = pacer.deadline(Instant::now(), settings.frame_limit.period()) {
        hold_until(deadline);
    }
}

/// How the scaled view is stretched to the window: bilinear (the default)
/// or nearest, for the owner's comparison (`--upscale nearest`).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Upscale {
    #[default]
    Bilinear,
    Nearest,
}

/// The view's image while the scale is under 100 %.
#[derive(Resource, Default)]
pub struct ScaledView {
    image: Option<Handle<Image>>,
    size: UVec2,
}

/// A 3D camera as the render scale handles it: its target, its projection
/// (touched to make Bevy recompute the target's size), whether it is the
/// view, and whether it holds the HUD.
type ViewCamera = (
    Entity,
    &'static mut RenderTarget,
    &'static mut Projection,
    &'static Camera,
    Has<IsDefaultUiCamera>,
);

/// The present camera and its sprite, which come and go together.
type Presenting = Or<(With<PresentCamera>, With<PresentSprite>)>;

/// The camera that draws the scaled view across the window, and the UI over
/// it at the window's own resolution.
#[derive(Component)]
pub struct PresentCamera;

/// The sprite the present camera draws.
#[derive(Component)]
pub struct PresentSprite;

/// The image size for a window at a scale, in physical pixels.
pub fn scaled_size(window: UVec2, percent: u32) -> UVec2 {
    (window.as_vec2() * (percent as f32 / 100.0))
        .round()
        .max(Vec2::ONE)
        .as_uvec2()
}

fn view_image(size: UVec2, upscale: Upscale) -> Image {
    let mut image = Image::new_target_texture(size.x, size.y, TextureFormat::Rgba8UnormSrgb, None);
    image.sampler = match upscale {
        Upscale::Bilinear => ImageSampler::linear(),
        Upscale::Nearest => ImageSampler::nearest(),
    };
    image
}

/// The render scale, every frame. At 100 % every 3D camera targets the
/// window, as before this setting existed, and nothing else is drawn. Below,
/// the 3D cameras render into an image of the scaled size and the present
/// camera stretches it over the window; the image is remade when the scale
/// or the window's size changes.
///
/// Anything that projects through a 3D camera INTO the UI must scale by
/// window / image while the view is scaled. Nothing does today: the HUD
/// reads camera transforms, not viewport coordinates.
#[allow(clippy::too_many_arguments)]
pub fn apply_render_scale(
    mut commands: Commands,
    settings: Res<PlayerSettings>,
    upscale: Res<Upscale>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut view: ResMut<ScaledView>,
    mut cameras: Query<ViewCamera, With<Camera3d>>,
    present: Query<Entity, Presenting>,
    mut sprites: Query<&mut Sprite, With<PresentSprite>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    if settings.render_scale >= 100 {
        if view.image.take().is_some() {
            for (entity, mut target, mut projection, camera, _) in &mut cameras {
                *target = RenderTarget::Window(WindowRef::Primary);
                // Bevy recomputes a camera's target size on a window or image
                // event or a projection change, never on a new target alone:
                // without this the view kept the image's size in the window.
                projection.set_changed();
                // The HUD goes back to the view's camera (`walking.rs` keeps
                // it on the active one).
                if camera.is_active {
                    commands.entity(entity).insert(IsDefaultUiCamera);
                }
            }
            for entity in &present {
                commands.entity(entity).despawn();
            }
        }
        return;
    }
    let want = scaled_size(window.physical_size(), settings.render_scale);
    if view.image.is_none() || view.size != want {
        let handle = images.add(view_image(want, *upscale));
        view.image = Some(handle.clone());
        view.size = want;
        for mut sprite in &mut sprites {
            sprite.image = handle.clone();
        }
    }
    let handle = view.image.clone().expect("made above");
    for (entity, mut target, mut projection, _, ui) in &mut cameras {
        if target.as_image() != Some(&handle) {
            *target = RenderTarget::Image(ImageRenderTarget::from(handle.clone()));
            projection.set_changed();
        }
        // The HUD is the present camera's while the view is scaled: a camera
        // that renders to an image never sees the cursor, so a menu drawn by
        // one takes no clicks (`ui_focus_system`). A mode switch puts the
        // marker back on the active view; it is taken off again here.
        if ui {
            commands.entity(entity).remove::<IsDefaultUiCamera>();
        }
    }
    let fill = window.size();
    if present.is_empty() {
        commands.spawn((
            Name::new("Render scale present camera"),
            Camera2d,
            Camera {
                order: 1,
                ..default()
            },
            // Bevy's default MSAA, as every other 2D camera on the window
            // has (the loading screen's): Bevy keys a 2D camera's depth
            // texture by its target alone, so two on one window share one,
            // made at whichever's sample count came first. With `Msaa::Off`
            // here, a launch at a reduced scale met the loading screen's 4x
            // depth with a 1x colour and wgpu refused the pass.
            Msaa::default(),
            Tonemapping::None,
            DebandDither::Disabled,
            IsDefaultUiCamera,
            PresentCamera,
        ));
        commands.spawn((
            Name::new("Render scale view"),
            Sprite {
                image: handle,
                custom_size: Some(fill),
                ..default()
            },
            PresentSprite,
        ));
    } else {
        for mut sprite in &mut sprites {
            if sprite.custom_size != Some(fill) {
                sprite.custom_size = Some(fill);
            }
        }
    }
}

/// Auto step follows the setting, on the walker's config.
pub fn follow_auto_step(
    settings: Res<PlayerSettings>,
    config: Option<ResMut<crate::walking::WalkingConfig>>,
) {
    if let Some(mut config) = config
        && config.auto_step != settings.auto_step
    {
        config.auto_step = settings.auto_step;
    }
}

/// The settings' systems: the render scale and auto step each frame, the
/// pacer last. `capture` stands the pacer down.
pub struct PlayerSettingsPlugin {
    pub capture: bool,
}

impl Plugin for PlayerSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSettings>()
            .init_resource::<SettingsFile>()
            .init_resource::<Upscale>()
            .init_resource::<ScaledView>()
            .init_resource::<FramePacer>()
            .add_systems(
                PostUpdate,
                (
                    // Before Bevy sizes the cameras, so a new target is sized
                    // the frame it is set.
                    apply_render_scale.before(bevy::camera::CameraUpdateSystems),
                    follow_auto_step,
                ),
            );
        if !self.capture {
            // After everything else in the frame, the frame log included.
            app.add_systems(Last, pace_frames.after(PacingSet));
        }
    }
}

/// What the pacer runs after: the frame's own measurements.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PacingSet;

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pbd-settings-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("settings.ron")
    }

    #[test]
    fn a_new_player_gets_the_owners_defaults_and_a_benchmark_todays_engine() {
        // The owner's choice, 2026-11-13: "default to this settings".
        let s = PlayerSettings::default();
        assert_eq!(s.frame_limit, FrameLimit::Fps60);
        assert_eq!(s.render_scale, 85);
        assert!(!s.auto_step);
        // Captures and benchmarks keep the engine as it was.
        let i = PlayerSettings::ISOLATED;
        assert_eq!(i.frame_limit.period(), None);
        assert_eq!(i.render_scale, 100);
        assert!(i.auto_step);
    }

    #[test]
    fn settings_round_trip_through_the_file() {
        let path = scratch("round-trip");
        let chosen = PlayerSettings {
            frame_limit: FrameLimit::Fps120,
            render_scale: 67,
            auto_step: false,
        };
        chosen.keep_to(&path).unwrap();
        let (read, loaded) = PlayerSettings::load_from(&path);
        assert_eq!(read, chosen);
        assert_eq!(loaded, Loaded::Read(VERSION));
        assert!(!path.with_extension("ron.tmp").exists());
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn a_missing_file_gives_defaults_and_writes_nothing() {
        let path = scratch("missing");
        let (read, loaded) = PlayerSettings::load_from(&path);
        assert_eq!(read, PlayerSettings::default());
        assert_eq!(loaded, Loaded::Missing);
        assert!(loaded.writable());
        assert!(!path.exists(), "a load never writes");
    }

    #[test]
    fn a_broken_file_gives_defaults_and_is_left_as_it_was() {
        let path = scratch("broken");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "(frame_limit: Fps60, render_scale: ").unwrap();
        let (read, loaded) = PlayerSettings::load_from(&path);
        assert_eq!(read, PlayerSettings::default());
        assert!(matches!(loaded, Loaded::Broken(_)));
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "(frame_limit: Fps60, render_scale: "
        );
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn an_old_file_keeps_what_it_says_and_defaults_the_rest() {
        let (read, version) = PlayerSettings::parse("(render_scale: 75)").unwrap();
        assert_eq!(version, 0);
        assert_eq!(
            read,
            PlayerSettings {
                render_scale: 75,
                ..default()
            }
        );
        let (empty, _) = PlayerSettings::parse("()").unwrap();
        assert_eq!(empty, PlayerSettings::default());
    }

    #[test]
    fn a_newer_file_is_read_and_never_written_back() {
        let text = "(version: 7, frame_limit: Fps60, brightness: 3)";
        let (read, version) = PlayerSettings::parse(text).unwrap();
        assert_eq!(read.frame_limit, FrameLimit::Fps60);
        assert_eq!(version, 7);
        assert!(!Loaded::Read(7).writable());
        assert!(Loaded::Read(VERSION).writable());
    }

    #[test]
    fn a_scale_off_the_list_snaps_to_it() {
        assert_eq!(snap_scale(66), 67);
        assert_eq!(snap_scale(10), 50);
        assert_eq!(snap_scale(250), 100);
        let (read, _) = PlayerSettings::parse("(render_scale: 80)").unwrap();
        assert!(SCALES.contains(&read.render_scale));
        // A press cycles every scale and comes back.
        let mut s = 100;
        for _ in 0..SCALES.len() {
            s = next_scale(s);
        }
        assert_eq!(s, 100);
    }

    #[test]
    fn flags_lie_over_the_file() {
        let file = PlayerSettings {
            frame_limit: FrameLimit::Fps60,
            render_scale: 50,
            auto_step: false,
        };
        let flags = Overrides {
            render_scale: Some(100),
            ..default()
        };
        let run = file.with(&flags);
        assert_eq!(run.render_scale, 100);
        assert_eq!(run.frame_limit, FrameLimit::Fps60);
        assert!(!run.auto_step);
        // A capture or a benchmark reads no file and keeps nothing.
        let (isolated, keep) = for_launch(true, &Overrides::default());
        assert_eq!(isolated, PlayerSettings::ISOLATED);
        assert!(keep.path.is_none());
    }

    #[test]
    fn the_pacer_holds_deadlines_one_period_apart() {
        let period = FrameLimit::Fps60.period();
        let p = period.unwrap();
        let t0 = Instant::now();
        let mut pacer = FramePacer::default();
        // The first capped frame anchors and does not wait.
        assert_eq!(pacer.deadline(t0, period), None);
        // Frames that take 5 ms are each held to the next deadline, and the
        // deadlines are exactly a period apart: no drift from the work.
        let mut end = t0;
        for k in 1..=120u32 {
            let now = end + Duration::from_millis(5);
            let deadline = pacer.deadline(now, period).expect("held");
            assert_eq!(deadline, t0 + p * k);
            end = deadline;
        }
        // Over two seconds, 120 frames: the mean is the cap.
        assert_eq!(end - t0, p * 120);
    }

    #[test]
    fn a_hitch_reanchors_rather_than_running_fast() {
        let period = FrameLimit::Fps120.period();
        let p = period.unwrap();
        let t0 = Instant::now();
        let mut pacer = FramePacer::default();
        pacer.deadline(t0, period);
        // A frame a little late keeps the cadence and does not wait.
        assert_eq!(pacer.deadline(t0 + p + p / 4, period), None);
        // The next is held to the cadence's own deadline.
        assert_eq!(pacer.deadline(t0 + p + p / 2, period), Some(t0 + p * 2));
        // A 100 ms hitch: no wait, and the next deadline is a period from it,
        // not the missed ones, so no burst of short frames follows.
        let hitch = t0 + Duration::from_millis(100);
        assert_eq!(pacer.deadline(hitch, period), None);
        assert_eq!(
            pacer.deadline(hitch + Duration::from_millis(1), period),
            Some(hitch + p)
        );
    }

    #[test]
    fn with_no_cap_the_pacer_never_waits() {
        let mut pacer = FramePacer::default();
        let t0 = Instant::now();
        for k in 0..10 {
            assert_eq!(pacer.deadline(t0 + Duration::from_micros(k), None), None);
        }
    }

    #[test]
    fn holding_reaches_the_deadline() {
        let deadline = Instant::now() + Duration::from_millis(4);
        hold_until(deadline);
        let late = Instant::now() - deadline;
        assert!(late < Duration::from_millis(2), "overshot by {late:?}");
    }

    fn scale_app(percent: u32) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .insert_resource(PlayerSettings {
                render_scale: percent,
                ..default()
            })
            .init_resource::<Upscale>()
            .init_resource::<ScaledView>()
            .add_systems(Update, apply_render_scale);
        let mut window = Window::default();
        window.resolution.set_physical_resolution(1440, 900);
        app.world_mut().spawn((window, PrimaryWindow));
        // The view's camera holds the HUD, as `walking.rs` leaves it.
        app.world_mut()
            .spawn((Camera3d::default(), IsDefaultUiCamera));
        app
    }

    fn targets(app: &mut App) -> Vec<RenderTarget> {
        let mut q = app
            .world_mut()
            .query_filtered::<&RenderTarget, With<Camera3d>>();
        q.iter(app.world()).cloned().collect()
    }

    fn presents(app: &mut App) -> usize {
        let mut q = app
            .world_mut()
            .query_filtered::<Entity, With<PresentCamera>>();
        q.iter(app.world()).count()
    }

    #[test]
    fn at_full_scale_the_cameras_draw_to_the_window_and_nothing_is_added() {
        let mut app = scale_app(100);
        for _ in 0..3 {
            app.update();
        }
        assert_eq!(presents(&mut app), 0);
        for target in targets(&mut app) {
            assert!(matches!(target, RenderTarget::Window(WindowRef::Primary)));
        }
    }

    #[test]
    fn a_reduced_scale_renders_smaller_and_comes_back() {
        let mut app = scale_app(67);
        app.update();
        app.update();
        assert_eq!(presents(&mut app), 1);
        let handle = targets(&mut app)[0].as_image().cloned().expect("an image");
        let size = app
            .world()
            .resource::<Assets<Image>>()
            .get(&handle)
            .unwrap()
            .size();
        assert_eq!(size, UVec2::new(965, 603));
        assert_eq!(size, scaled_size(UVec2::new(1440, 900), 67));
        // The HUD is on the present camera, which sees the cursor, and not
        // on the view's, which renders to an image and would take no clicks.
        let mut holders = app
            .world_mut()
            .query_filtered::<(Has<PresentCamera>, Has<Camera3d>), With<IsDefaultUiCamera>>();
        let held: Vec<_> = holders.iter(app.world()).collect();
        assert_eq!(held, vec![(true, false)], "the HUD's camera while scaled");
        app.world_mut()
            .resource_mut::<PlayerSettings>()
            .render_scale = 100;
        app.update();
        app.update();
        assert_eq!(presents(&mut app), 0);
        let held: Vec<_> = holders.iter(app.world()).collect();
        assert_eq!(held, vec![(false, true)], "the HUD back on the view");
        assert!(matches!(
            targets(&mut app)[0],
            RenderTarget::Window(WindowRef::Primary)
        ));
    }
}
