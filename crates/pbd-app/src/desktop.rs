mod digging;
mod equipment;
mod guide;
mod hud;
mod menu;
mod overlay_ui;
mod scene;
mod slots;
mod weather_ui;

use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    asset::AssetPlugin,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::render_resource::TextureUsages,
    render::view::{
        Hdr,
        screenshot::{Screenshot, ScreenshotCaptured},
    },
    time::TimeUpdateStrategy,
    window::PresentMode,
};
use pbd_app::{
    CelestialScene, FIXED_HZ, PaleBlueDotPlugin, PhysicsFrame,
    config::ConfigPlugin,
    flight_view::{FlightViewConfig, FlightViewPlugin, FlyMode, TourProgress},
    planet::{
        FINEST_LEVEL, PLANET_RADIUS, PlanetPlugin, TERRAIN, river_channel, surface_code,
        surface_height, terrain_radius, tile_width_m,
    },
    saves::{self, Pose, WorldSave},
    sky::SkyPlugin,
    walking::{EYE_HEIGHT, RestoredPose, WalkingConfig, WalkingPlugin},
    weather::WeatherPlugin,
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource, Clone)]
pub struct Launch {
    pub capture: Option<PathBuf>,
    /// `--frame-log PATH`: a measurement instrument. Every frame's wall time,
    /// written as CSV with the fine set's state on the same clock, so a hitch
    /// can be attributed rather than guessed at (`far-side-flight`).
    pub frame_log: Option<PathBuf>,
    pub view: String,
    pub frames: u32,
    pub tour: bool,
    /// `--route far-side`: take off, climb, cruise round the planet and land
    /// on the far side (`far-side-flight`). Implies the tour's scripted setup.
    pub route: bool,
    pub fixed: bool,
    pub fly: bool,
    pub walk: bool,
    /// Walk mode, placed at the shoreline and holding forward, so a capture can
    /// photograph the water being entered. A walker with no input never moves.
    pub swim: bool,
    /// `--fish`: stand on a warm shore facing the sea with the rod, and cast
    /// once, scripted, so the float, the line and the schools can be
    /// photographed and the fishery's numbers logged. A capture instrument,
    /// like `--swim`: a headless run has no hand on the mouse.
    pub fish: bool,
    /// `--aboard KIND` boards the Kestrel, Tern or Loon once the fleet is in,
    /// and `--seat` takes the seat rather than the chase view: a headless run
    /// has nobody to walk up to a craft and press G. Implies `--walk`.
    pub aboard: Option<pbd_core::vehicle::Kind>,
    pub seat: bool,
    /// Static capture instrument: translate the scene within the local frame.
    pub render_offset: Vec3,
    /// Capture instrument for the `shore` view: camera height above the last
    /// land cell in metres. Absent means standing eye height.
    pub height: Option<f32>,
    /// `--dig N` digs N blocks straight down from the camera on the frame the
    /// tier is ready, and `--place N` stacks N stones on the layer above the last
    /// hole. A headless run has no mouse, and a picture of a hole is the only
    /// thing that says the verb works end to end.
    pub dig: u32,
    /// `--place N` stacks N stones on the last hole, so a tower somebody built
    /// can be photographed wearing the stone it is made of.
    pub place: u32,
    /// `--spawn mouth` puts the spawn, and so the column tier, at the nearest
    /// cave mouth to the default spawn. Mouth patches cover a few percent of
    /// the land and the default spawn has none, so without this a walker has
    /// nothing to walk into for the first few hundred metres.
    pub spawn: Option<String>,
    /// Rain intensity at launch, 0..1.
    pub rain: f32,
    /// `--weather-at SECONDS` starts the weather field that far into its own
    /// time, so a capture can be taken under a chosen sky: the spawn is a wet
    /// meadow whose cover sits near 0.6 at launch, and the body's average is
    /// 0.18. Moves the field only, not the sun or the sea.
    pub weather_at: f32,
    /// `--dig-ahead` digs along the camera's LOOK rather than straight down.
    /// Digging down is right for proving the verb and useless for judging the
    /// result: the walker falls into its own pit and the eye ends up inside
    /// the wall, so every capture of a hole is a screen of dirt. Digging ahead
    /// leaves them standing on the rim looking at what they made.
    pub dig_ahead: bool,
    /// `--pitch <degrees>` starts the walker looking that far below (negative)
    /// or above the horizon. A capture that digs along the look needs to look
    /// DOWN to dig a pit the way a player does - a slanted run of cells, each
    /// taken from a different column at a different layer - and the headless
    /// walker otherwise looks dead level at the horizon.
    pub pitch: Option<f32>,
    /// `--yaw <degrees>` turns the walker's starting heading that far to the
    /// right of the default. With `--time` the launch log says where the sun
    /// stands from the spawn, as the yaw and pitch that would centre it, so a
    /// sky capture is aimed off the clock rather than guessed.
    pub yaw: Option<f32>,
    /// `--torch` puts one torch on the ground under the capture camera. A
    /// headless run has no hands, and a lamp is the one thing in this world
    /// whose whole point is what it does to a dark place.
    pub torch: bool,
    /// `--time <hour>` pins the clock, 0..24, and STOPS it. A capture whose
    /// world has a day in it is a different picture every run, and a harness
    /// cannot wait six minutes for dusk.
    pub time: Option<f32>,
    /// `--day N` puts the clock on that day of the year, for a season. It
    /// pins the clock the way `--time` does.
    pub day: Option<u32>,
    /// `--overlay <name>` opens with that weather overlay showing (`wind`,
    /// `jet`, `currents`, `cloud`, `rain`, `humidity`, `sunlight`,
    /// `temperature`), as M would.
    pub overlay: Option<pbd_core::overlay::Overlay>,
    /// `--world <name>` opens that save, creating it if it is not there.
    /// Absent, an interactive run opens the one played most recently and a
    /// capture writes to no world at all.
    pub world: Option<String>,
    /// `--menu pause|settings|saves` opens that screen at startup. A headless run has
    /// no pointer and no keyboard, so a screen a player reaches with `Escape`
    /// has to be reachable by a flag or it can never be photographed.
    pub menu: Option<String>,
}

impl Launch {
    fn parse(args: &[String]) -> Self {
        let mut result = Self {
            capture: None,
            frame_log: None,
            view: "coast".into(),
            frames: 180,
            dig: 0,
            place: 0,
            tour: false,
            route: false,
            fixed: false,
            fly: false,
            walk: false,
            swim: false,
            fish: false,
            aboard: None,
            seat: false,
            render_offset: Vec3::ZERO,
            height: None,
            spawn: None,
            rain: 0.0,
            weather_at: 0.0,
            world: None,
            time: None,
            day: None,
            overlay: None,
            torch: false,
            dig_ahead: false,
            pitch: None,
            yaw: None,
            menu: None,
        };
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--capture" => {
                    i += 1;
                    result.capture =
                        Some(args.get(i).expect("--capture requires a PNG path").into());
                }
                "--dig" => {
                    i += 1;
                    result.dig = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--dig requires a count");
                }
                "--place" => {
                    i += 1;
                    result.place = args
                        .get(i)
                        .and_then(|n| n.parse().ok())
                        .expect("--place requires a count");
                }
                "--torch" => result.torch = true,
                "--dig-ahead" => result.dig_ahead = true,
                "--pitch" => {
                    i += 1;
                    let degrees: f32 = args
                        .get(i)
                        .and_then(|d| d.parse().ok())
                        .expect("--pitch requires degrees");
                    assert!(
                        (-89.0..=89.0).contains(&degrees),
                        "--pitch takes degrees in -89..89"
                    );
                    result.pitch = Some(degrees);
                }
                "--yaw" => {
                    i += 1;
                    let degrees: f32 = args
                        .get(i)
                        .and_then(|d| d.parse().ok())
                        .expect("--yaw requires degrees");
                    assert!(degrees.is_finite(), "--yaw takes finite degrees");
                    result.yaw = Some(degrees);
                }
                "--time" => {
                    i += 1;
                    let hour: f32 = args
                        .get(i)
                        .and_then(|h| h.parse().ok())
                        .expect("--time requires an hour");
                    assert!(
                        (0.0..=24.0).contains(&hour),
                        "--time takes an hour in 0..24"
                    );
                    result.time = Some(hour);
                }
                "--day" => {
                    i += 1;
                    let day: u32 = args
                        .get(i)
                        .and_then(|d| d.parse().ok())
                        .expect("--day requires a day number");
                    result.day = Some(day);
                }
                "--world" => {
                    i += 1;
                    result.world = Some(args.get(i).expect("--world requires a name").clone());
                }
                "--menu" => {
                    i += 1;
                    let screen = args
                        .get(i)
                        .expect("--menu requires pause, settings or saves");
                    assert!(
                        matches!(screen.as_str(), "pause" | "settings" | "saves"),
                        "--menu takes pause, settings or saves"
                    );
                    result.menu = Some(screen.clone());
                }
                "--view" => {
                    i += 1;
                    result.view = args.get(i).expect("--view requires a view name").clone();
                }
                "--spawn" => {
                    i += 1;
                    let spawn = args.get(i).expect("--spawn requires a place").clone();
                    assert!(
                        spawn == "mouth" || spawn == "snow",
                        "--spawn knows mouth and snow"
                    );
                    result.spawn = Some(spawn);
                }
                "--frame-log" => {
                    i += 1;
                    result.frame_log = Some(PathBuf::from(
                        args.get(i).expect("--frame-log requires a path"),
                    ));
                }
                "--frames" => {
                    i += 1;
                    result.frames = args
                        .get(i)
                        .expect("--frames requires a number")
                        .parse()
                        .expect("invalid frame count");
                }
                "--tour" => result.tour = true,
                "--route" => {
                    i += 1;
                    let name = args.get(i).expect("--route requires a route name");
                    assert!(name == "far-side", "--route knows far-side");
                    result.route = true;
                    result.tour = true;
                }
                "--fly" => result.fly = true,
                "--walk" => result.walk = true,
                "--swim" => {
                    result.walk = true;
                    result.swim = true;
                }
                "--fish" => {
                    result.walk = true;
                    result.fish = true;
                }
                "--aboard" => {
                    i += 1;
                    let key = args.get(i).expect("--aboard requires a craft");
                    result.aboard = Some(
                        pbd_core::vehicle::Kind::from_key(key)
                            .expect("--aboard knows kestrel, tern and loon"),
                    );
                    result.walk = true;
                }
                "--seat" => result.seat = true,
                "--fixed-dt" => result.fixed = true,
                "--render-offset" => {
                    let mut components = [0.0; 3];
                    for component in &mut components {
                        i += 1;
                        *component = args
                            .get(i)
                            .expect("--render-offset requires x y z in metres")
                            .parse::<f32>()
                            .expect("invalid render offset");
                    }
                    result.render_offset = Vec3::from_array(components);
                    assert!(
                        result.render_offset.is_finite(),
                        "render offset must be finite"
                    );
                }
                "--height" => {
                    i += 1;
                    let height: f32 = args
                        .get(i)
                        .expect("--height requires metres above the shore")
                        .parse()
                        .expect("invalid height");
                    assert!(
                        height.is_finite() && height >= 0.0,
                        "height must be finite and non-negative"
                    );
                    result.height = Some(height);
                }
                "--overlay" => {
                    i += 1;
                    let name = args.get(i).expect("--overlay requires a name");
                    result.overlay = Some(
                        pbd_core::overlay::Overlay::ALL
                            .into_iter()
                            .find(|o| o.name().eq_ignore_ascii_case(name))
                            .unwrap_or_else(|| panic!("no overlay called {name}")),
                    );
                }
                "--rain" => {
                    i += 1;
                    let rain: f32 = args
                        .get(i)
                        .expect("--rain requires an intensity 0..1")
                        .parse()
                        .expect("invalid rain intensity");
                    assert!((0.0..=1.0).contains(&rain), "rain must be within 0..1");
                    result.rain = rain;
                }
                "--weather-at" => {
                    i += 1;
                    let seconds: f32 = args
                        .get(i)
                        .expect("--weather-at requires seconds")
                        .parse()
                        .expect("invalid weather seconds");
                    assert!(seconds.is_finite(), "weather seconds must be finite");
                    result.weather_at = seconds;
                }
                "--verify-flight" => {}
                "--verify-route" => {
                    i += 1;
                }
                unknown => panic!("unknown argument {unknown}; use --help"),
            }
            i += 1;
        }
        assert!(
            !(result.walk && (result.fly || result.tour)),
            "--walk cannot be combined with --fly or --tour"
        );
        assert!(
            [
                "orbit",
                "coast",
                "surface",
                "seam",
                "night",
                "nightshore",
                "midnight",
                "meadow",
                "river",
                "pole",
                "shore",
                "wade",
                "dive",
                "cave",
                "overhang",
                "mouth",
                "seacave",
                "column"
            ]
            .contains(&result.view.as_str()),
            "unknown capture view"
        );
        assert!(
            result.frames >= 60 && result.frames <= 100_000,
            "capture frames must be 60..100000"
        );
        assert!(
            result.height.is_none()
                || ([
                    "shore",
                    "nightshore",
                    "midnight",
                    "meadow",
                    "river",
                    "dive",
                    "column"
                ]
                .contains(&result.view.as_str())
                    && result.capture.is_some()),
            "--height requires --view shore or dive with a static --capture"
        );
        assert!(
            result.render_offset == Vec3::ZERO
                || (result.capture.is_some() && !result.walk && !result.fly && !result.tour),
            "--render-offset requires a static --capture"
        );
        result
    }
}

#[derive(Resource)]
struct CaptureState {
    frame: u32,
    requested: bool,
    saved: bool,
    finish_frame: u32,
}

#[derive(Resource)]
pub struct FrameStats {
    pub frame_ms: f32,
    pub samples: Vec<f64>,
    previous: Instant,
    /// When the app started: the frame log's clock, to line a slow frame up
    /// with what the log says happened then.
    started: Instant,
}

pub fn run(args: &[String]) {
    if args.iter().any(|s| s == "--verify-route") {
        verify_route();
        return;
    }
    if args.iter().any(|s| s == "--verify-flight") {
        verify_flight(args.windows(2).any(|w| w[0] == "--view" && w[1] == "pole"));
        return;
    }
    let launch = Launch::parse(args);
    if let Some(path) = &launch.capture
        && let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).expect("capture directory");
    }
    let photo = launch.capture.is_some() && !launch.tour && !launch.walk && !launch.fly;
    // The world is opened BEFORE the app is built, because where the player
    // was standing decides where the planet's fine set and the column tier are
    // anchored. Restoring the pose afterwards would build the world around the
    // spawn and then teleport away from it.
    let mut world = open_world(&launch);
    let saved_seconds = world.world_seconds;
    let hotbar = slots::Hotbar::restore(&mut world);
    let tools = pbd_app::fish::ToolSlot::restore(&world);
    // The saves page is the front door of a plain launch: a player picks the
    // world rather than being put in the last one.
    let opening = menu::opening_screen(
        launch.menu.as_deref(),
        launch.world.is_some(),
        launch.capture.is_some(),
    );
    let restored = world.pose;
    let step = Duration::from_secs_f64(1.0 / FIXED_HZ);
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").into(),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Pale Blue Dot | Planet Explorer".into(),
                    resolution: (1440, 900).into(),
                    present_mode: if launch.capture.is_some() {
                        PresentMode::AutoNoVsync
                    } else {
                        PresentMode::AutoVsync
                    },
                    // Keep interactive input close to presentation instead of
                    // queuing multiple old camera poses behind the current one.
                    desired_maximum_frame_latency: if launch.capture.is_some() {
                        None
                    } else {
                        std::num::NonZeroU32::new(1)
                    },
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        PhysicsPlugins::default(),
        PaleBlueDotPlugin,
        ConfigPlugin,
        WeatherPlugin {
            rain: launch.rain,
            weather_at: launch.weather_at,
        },
        PlanetPlugin,
        FlightViewPlugin,
        SkyPlugin,
    ))
    .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
    .insert_resource(PhysicsFrame(pbd_core::frame::LocalFrame {
        origin: -launch.render_offset.as_dvec3(),
        ..default()
    }))
    .insert_resource(Time::<Fixed>::from_duration(step))
    .insert_resource(SubstepCount(4))
    .insert_resource(FlightViewConfig {
        mode: if launch.route {
            FlyMode::Route
        } else if launch.tour {
            FlyMode::Tour
        } else {
            FlyMode::Manual
        },
        // The route always starts from the walker's dry-land spawn, not
        // wherever the world's last flight ended.
        spawn_direction: if launch.route {
            spawn_direction(&launch)
        } else {
            restored
                .map(|pose| pose.position.normalize_or(Vec3::Y))
                .unwrap_or_else(|| spawn_direction(&launch))
        },
        spawn_altitude: 240.0,
        // The tour keeps 45 m above the ground; the route lands, so it keeps
        // only the walker's eye height.
        minimum_clearance: if launch.route {
            1.6
        } else if launch.tour {
            45.0
        } else {
            1.6
        },
        startup_camera: !photo,
        ..default()
    })
    // What the save recorded, or the starting kit in a new world. The hotbar
    // rides the edit log rather than a timer, so what comes back is what was
    // held when the last block moved.
    .insert_resource(hotbar)
    // And the tool in hand, which the save records whenever it changes.
    .insert_resource(tools)
    .init_resource::<equipment::Picker>()
    // Whose roster the guide and the icons read, in every mode; the fish
    // themselves come with the walker.
    .init_resource::<pbd_app::fish::Body>()
    .init_resource::<guide::GuidePage>()
    // The world, loaded before the planet is built: `create_planet` reads its
    // edits for the first tier, so a save's holes are there on the first frame
    // rather than appearing when the player first walks.
    .insert_resource(world)
    // The clock: pinned and stopped where a capture asked for an hour, so a
    // picture is a function of its flags rather than of when it was taken.
    .insert_resource(pbd_app::sky::Sun {
        clock: sun_clock(&launch, saved_seconds),
        running: launch.time.is_none() && launch.day.is_none() && launch.capture.is_none(),
    })
    .init_resource::<digging::Aim>()
    .insert_resource(ClearColor(if std::env::var("PBD_NO_SKY").is_ok() {
        // The hole detector's background: nothing in the palette is near it,
        // so a magenta pixel is a pixel with no world behind it.
        Color::srgb(1.0, 0.0, 1.0)
    } else {
        Color::srgb(0.002, 0.004, 0.012)
    }))
    .insert_resource(CaptureState {
        frame: 0,
        requested: false,
        saved: false,
        finish_frame: 0,
    })
    .insert_resource(FrameStats {
        frame_ms: 0.0,
        samples: Vec::new(),
        previous: Instant::now(),
        started: Instant::now(),
    })
    .insert_resource(launch.clone())
    .insert_resource(pbd_app::overlay::OverlayMode(launch.overlay))
    .add_systems(
        Startup,
        (scene::setup, hud::setup, photo_camera, overlay_ui::spawn),
    )
    // The column tier is built by a startup system and its records land when
    // that schedule's commands apply, so a camera that wants to stand inside a
    // cave has to be placed a schedule later.
    .add_systems(PostStartup, cave_camera)
    .insert_resource(opening)
    .insert_resource(menu::FrontDoor(opening == menu::Screen::Saves))
    .init_resource::<menu::NameField>()
    .add_systems(Startup, menu::spawn)
    .add_systems(
        PostStartup,
        (
            slots::spawn,
            (slots::load_icons, (equipment::spawn, guide::spawn)).chain(),
        ),
    )
    // Escape is read before either of the world's input readers, which live in
    // `RunFixedMainLoop`, and is cleared there so neither ever sees it.
    .add_systems(
        PreUpdate,
        (menu::name_input, menu::toggle, guide::open)
            .chain()
            .after(bevy::input::InputSystems),
    )
    .add_systems(
        Update,
        (
            configure_camera,
            scene::move_moon,
            scene::turn_stars,
            scene::follow_sun,
            (equipment::pick, slots::input, equipment::paint).chain(),
            slots::update,
            guide::press,
            guide::paint,
            hud::near_field,
            (menu::press, menu::paint, menu::rebuild_saves).chain(),
            (weather_ui::drag, weather_ui::show).chain(),
            overlay_ui::show,
            autosave,
            save_weather,
            digging::dig_and_place.before(pbd_app::fish::FishSet),
            digging::scripted_dig,
            capture,
        ),
    )
    .init_resource::<menu::SaveIndex>()
    .init_resource::<menu::LoadRequest>()
    .add_systems(PreUpdate, load_world.after(menu::toggle))
    .insert_resource(FrameLog::open(launch.frame_log.as_deref()))
    .add_systems(
        Last,
        ((measure_frames, write_frame_log).chain(), drain_saves),
    );
    if !photo && !launch.tour {
        app.insert_resource(WalkingConfig {
            start_walking: !launch.fly,
            restored: restored.map(|pose| RestoredPose {
                position: pose.position,
                heading: pose.heading,
                pitch: pose.pitch,
            }),
            pitch: launch.pitch.unwrap_or(0.0).to_radians(),
            yaw: launch.yaw.unwrap_or(0.0).to_radians(),
            ..default()
        })
        .add_plugins((
            WalkingPlugin,
            pbd_app::vehicles::VehiclePlugin,
            pbd_app::fish::FishPlugin,
        ))
        .insert_resource(pbd_app::vehicles::VehicleScript {
            board: launch.aboard,
            seat: launch.seat,
        });
        if launch.fish {
            app.add_systems(
                PreUpdate,
                (swim_script, fish_script)
                    .chain()
                    .after(bevy::input::InputSystems),
            );
        }
        if launch.swim {
            // The scripted keys have to be written where the real ones are:
            // after the input clear and before the walking input reads them,
            // which is in RunFixedMainLoop. Pressed in `Update` they were
            // wiped by the next frame's clear before anything looked, and the
            // walker stood still. Registered only beside the walker it drives:
            // a plain photo has no `WalkingState`, and a system that asks for
            // one unconditionally panics the first frame.
            app.add_systems(PreUpdate, swim_script.after(bevy::input::InputSystems));
        }
    }
    if launch.fixed || launch.capture.is_some() {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(step));
    }
    app.run();
}

/// Which save this run opens.
///
/// A CAPTURE with no `--world` writes to nothing, which is the LOD camera's
/// own lesson in another costume: a harness that pins a pose every frame and
/// then saves it poisons the world for every later run. A capture that means
/// to write says which world.
fn open_world(launch: &Launch) -> WorldSave {
    let root = std::path::PathBuf::from(saves::ROOT);
    let seed = TERRAIN.seed;
    let asked = launch.world.clone();
    if asked.is_none() && launch.capture.is_some() {
        return WorldSave::memory_only();
    }
    let listed = saves::list(&root);
    let slot = match asked.as_deref() {
        Some(name) => {
            let id = saves::slot_id(name);
            listed
                .into_iter()
                .find(|slot| slot.id == id || slot.file.name == name)
                .or_else(|| saves::create(&root, name, seed).ok())
        }
        // No name: the one played most recently, which is what logging back
        // on means. A first run has none, and nothing is made behind the
        // player's back: the saves page opens and offers NEW WORLD.
        None => listed.into_iter().next(),
    };
    let Some(slot) = slot else {
        if asked.is_some() {
            warn!("no world could be opened; this run will not be saved");
        } else {
            info!("no worlds yet; the saves page will make the first");
        }
        return WorldSave::memory_only();
    };
    if slot.file.seed != seed {
        // A save is OF a world. Loading it into a different one would make it
        // silently become somebody else's.
        warn!(
            "world '{}' was made in seed {} and this is {seed}; it will not be loaded",
            slot.file.name, slot.file.seed
        );
        return WorldSave::memory_only();
    }
    WorldSave::open(root, slot)
}

/// Carry out a load the saves screen asked for.
///
/// An exclusive system, because a load touches more of the world than one set
/// of borrows can hold: the save, the hotbar, the tier's refresh and the
/// walker's own body. It happens between frames rather than inside the press
/// that asked for it, which is also why `LoadRequest` exists at all.
fn load_world(world: &mut World) {
    let Some(slot) = world.resource_mut::<menu::LoadRequest>().0.take() else {
        return;
    };
    let name = slot.file.name.clone();
    // The craft of the world being left are written into it before the queue
    // is drained, and are gone from the scene before the next world's arrive.
    if let Some(file) = pbd_app::vehicles::put_away(world) {
        world.resource_mut::<WorldSave>().snapshot_vehicles(&file);
    }
    let root = {
        let open = world.resource::<WorldSave>();
        // Everything queued for the world being left goes down before the
        // writer moves: the queue is ordered, and a half-written world is the
        // one thing a save must never leave behind.
        open.drain();
        open.root().to_path_buf()
    };
    let mut opened = WorldSave::open(root, slot);
    let hotbar = slots::Hotbar::restore(&mut opened);
    let tools = pbd_app::fish::ToolSlot::restore(&opened);
    let pose = opened.pose;
    // The world resumes in its season and at its hour; one never played
    // keeps the clock it had.
    if let Some(seconds) = opened.world_seconds {
        world.resource_mut::<pbd_app::sky::Sun>().clock.seconds = seconds;
    }
    // And under its own sky: the saved weather, or a new one spun up.
    let air = {
        let config = world.resource::<pbd_app::config::AtmosphereConfig>().0;
        let sun = *world.resource::<pbd_app::sky::Sun>();
        let mut air = pbd_app::atmosphere::Air::open(
            config,
            pbd_app::planet::TERRAIN.seed,
            opened.weather.as_deref(),
            sun.clock.seconds,
        );
        air.in_place = !sun.running;
        air
    };
    world.insert_resource(air);
    world.insert_resource(opened);
    world.insert_resource(hotbar);
    world.insert_resource(tools);
    // The water of the world being left is not this world's water.
    if let Some(mut fishery) = world.get_resource_mut::<pbd_app::fish::Fishery>() {
        *fishery = pbd_app::fish::Fishery::default();
    }
    // The tier is standing where the last world left it with the last world's
    // holes in it. The distance rule cannot know that, so the load says so.
    if let Some(mut refresh) = world.get_resource_mut::<pbd_app::planet::LodRefresh>() {
        refresh.force();
    }
    if world
        .get_resource::<pbd_app::walking::WalkingState>()
        .is_some()
    {
        match pose {
            Some(pose) => pbd_app::walking::restore(
                world,
                RestoredPose {
                    position: pose.position,
                    heading: pose.heading,
                    pitch: pose.pitch,
                },
            ),
            // A world nobody has played yet starts where a new world starts.
            None => pbd_app::walking::respawn(world),
        }
    }
    if let Some(pose) = pose {
        world.resource_mut::<slots::Hotbar>().select(pose.selected);
    }
    // The list shows which world is open, so it is drawn again now one is.
    world.resource_mut::<menu::SaveIndex>().set_changed();
    info!("loaded world '{name}'");
}

/// Write the pose on a timer, and whenever a menu opens.
///
/// Pose is the one part of a save that is genuinely cheap to lose, which is
/// exactly why it is the only part on a clock: the edits and the hotbar are
/// written per edit and are already down. A menu opening counts because the
/// player who opens one is usually the player about to quit.
#[allow(clippy::too_many_arguments)]
fn autosave(
    time: Res<Time>,
    screen: Res<menu::Screen>,
    walking: Option<Res<pbd_app::walking::WalkingState>>,
    slots: Res<slots::Hotbar>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    sun: Res<pbd_app::sky::Sun>,
    mut save: ResMut<WorldSave>,
    mut due: Local<f32>,
) {
    let opening = screen.is_changed() && *screen != menu::Screen::Playing;
    *due -= time.delta_secs();
    if !opening && *due > 0.0 {
        return;
    }
    *due = saves::AUTOSAVE_S;
    let (Some(state), Ok(position)) = (walking, walkers.single()) else {
        return;
    };
    let (heading, pitch) = state.view();
    save.snapshot(
        Pose {
            position: position.0,
            heading,
            pitch,
            selected: slots.selected(),
        },
        sun.clock.seconds,
    );
}

/// Wait for the disk on the way out.
///
/// The one place blocking is right is the place the player is already
/// waiting: losing the last two digs to a quit would be the whole feature
/// failing at its most visible moment.
fn drain_saves(
    exits: MessageReader<AppExit>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    fleet: Option<Res<pbd_app::vehicles::Fleet>>,
    vehicles: Query<&pbd_app::vehicles::Vehicle>,
    mut save: ResMut<WorldSave>,
) {
    if !exits.is_empty() {
        if let Some(air) = air {
            save.snapshot_weather(air.now.to_bytes());
        }
        if let Some(file) = fleet.and_then(|f| pbd_app::vehicles::fleet_file(&f, &vehicles)) {
            save.snapshot_vehicles(&file);
        }
        save.drain();
    }
}

/// Write the atmosphere on its own, slower timer, and whenever a menu opens:
/// half a megabyte is not a thing to write every five seconds, and losing a
/// minute of weather costs a minute of sky.
fn save_weather(
    time: Res<Time>,
    screen: Res<menu::Screen>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    mut save: ResMut<WorldSave>,
    mut due: Local<f32>,
) {
    let opening = screen.is_changed() && *screen != menu::Screen::Playing;
    *due -= time.delta_secs();
    if !opening && *due > 0.0 {
        return;
    }
    *due = saves::WEATHER_SAVE_S;
    if let Some(air) = air {
        save.snapshot_weather(air.now.to_bytes());
    }
}

/// The clock a launch starts on: pinned where `--time` asked, and then the
/// log says where the sun stands from the spawn, as the `--yaw` and `--pitch`
/// that would centre it, so a sky capture is aimed rather than guessed.
fn sun_clock(launch: &Launch, saved_seconds: Option<f64>) -> pbd_core::daylight::Clock {
    use pbd_core::daylight::{Clock, START_HOUR};
    let clock = match (launch.day, launch.time, saved_seconds) {
        (None, None, Some(seconds)) => Clock { seconds },
        (day, hour, _) => Clock::at(day.unwrap_or(0), hour.unwrap_or(START_HOUR)),
    };
    if launch.day.is_some() {
        info!(
            "day {} of {}: the sun stands {:.1} deg off the equator",
            clock.day(),
            pbd_core::daylight::YEAR_DAYS,
            clock.declination().to_degrees()
        );
    }
    if launch.time.is_some() {
        let up = spawn_direction(launch);
        let heading = Vec3::Y.cross(up).normalize_or(Vec3::X);
        let right = heading.cross(up);
        let sun = clock.sun();
        let yaw = sun.dot(right).atan2(sun.dot(heading)).to_degrees();
        let pitch = sun.dot(up).clamp(-1.0, 1.0).asin().to_degrees();
        info!("sun from the spawn at --time: --yaw {yaw:.1} --pitch {pitch:.1}");
    }
    clock
}

/// Where the walker, and with it the column tier, is anchored.
fn spawn_direction(launch: &Launch) -> Vec3 {
    let default = Vec3::new(0.8776, 0.4794, 0.0).normalize();
    if launch.tour && launch.view == "pole" {
        return Vec3::Y;
    }
    if launch.view == "seacave" {
        // The tier has to be somewhere that HAS caves below sea level, and
        // the spawn is seventy-four metres up. The shore is where the ground
        // meets the waterline; `nearest_ground_near` is the same locator the
        // change's measuring test uses, so the picture and the numbers are
        // taken of one place.
        if let Some(shore) = pbd_app::planet::nearest_ground_near(default, 0.5..4.0) {
            info!(
                "seacave spawn moved {:.0} m to the shore",
                shore.dot(default).clamp(-1., 1.).acos() * PLANET_RADIUS
            );
            return shore;
        }
    }
    if launch.spawn.as_deref() == Some("snow") {
        // The nearest dry land where the field's precipitation is snow, so a
        // capture can photograph a snowfall: a Fibonacci sweep of the sphere,
        // nearest first. A measurement instrument, like `--weather-at`.
        let count = 20_000;
        let golden = std::f32::consts::PI * (3.0 - 5f32.sqrt());
        let found = (0..count)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f32 + 0.5) / count as f32;
                let r = (1.0 - y * y).max(0.0).sqrt();
                let a = golden * i as f32;
                Vec3::new(a.cos() * r, y, a.sin() * r)
            })
            .filter(|d| {
                pbd_app::planet::surface_height(*d) > 1.0
                    && matches!(
                        pbd_core::planet_gen::biome(&pbd_app::planet::TERRAIN, *d),
                        pbd_core::planet_gen::Biome::Tundra
                            | pbd_core::planet_gen::Biome::Mountains
                    )
            })
            .max_by(|a, b| a.dot(default).total_cmp(&b.dot(default)));
        if let Some(snow) = found {
            info!(
                "spawn moved {:.0} m to the nearest snowfield",
                snow.dot(default).clamp(-1., 1.).acos() * PLANET_RADIUS
            );
            return snow;
        }
        warn!("no snowfield found; spawning at the default");
    }
    if launch.spawn.as_deref() == Some("mouth") || launch.view == "mouth" {
        // The nearest worm that starts at the surface within a kilometre.
        let columns = pbd_app::config::ColumnSettings::default();
        let found = pbd_core::worms::gather(
            &columns.worms(),
            &pbd_app::planet::TERRAIN,
            default,
            1_000.0,
        )
        .nearest_opening(default);
        match found {
            Some(mouth) => {
                info!(
                    "spawn moved {:.0} m to the nearest cave mouth",
                    mouth.dot(default).clamp(-1., 1.).acos() * PLANET_RADIUS
                );
                return mouth;
            }
            None => warn!("no cave mouth within a kilometre of the spawn; spawning at the default"),
        }
    }
    default
}

fn configure_camera(
    mut commands: Commands,
    mut cameras: Query<(Entity, &mut Camera3d), Added<Camera3d>>,
) {
    for (entity, mut camera) in &mut cameras {
        // The water composite reads the main pass depth; Bevy only allocates
        // a sampleable depth texture when a camera asks for one.
        let usages =
            TextureUsages::from(camera.depth_texture_usages) | TextureUsages::TEXTURE_BINDING;
        camera.depth_texture_usages = usages.into();
        commands.entity(entity).insert((
            Hdr,
            Tonemapping::TonyMcMapface,
            Msaa::Sample4,
            Projection::Perspective(PerspectiveProjection {
                near: 0.05,
                far: 500_000.0,
                fov: 60.0_f32.to_radians(),
                ..default()
            }),
        ));
    }
}

/// Stand inside the world: the two frames the voxel column tier exists to
/// produce, and neither could be taken before it.
///
/// It reads the LIVE TIER rather than calling the generator at a direction of
/// its own. That is not tidiness, it is the whole reason the first ten captures
/// were wrong: a column belongs to a CELL and is generated at that cell's own
/// centre, so a chamber found at an arbitrary point 1.4 m away need not exist
/// in the cell that is actually drawn. The camera stood inside solid rock, and
/// from inside rock nothing draws a face toward you, so the frame came back
/// showing the whole world from impossible angles - which reads exactly like a
/// renderer full of holes.
/// Whether this capture view's camera is placed a schedule later, off the
/// LIVE column tier, rather than from a direction and an altitude.
///
/// One list, because there were two and they disagreed: `seacave` was added
/// to the one `cave_camera` reads and not to the one `capture_camera` reads,
/// so BOTH spawned a camera and the frame came out of whichever won - a wide
/// shot of the shore from four hundred metres up, with the cave camera
/// standing in a chamber nobody photographed. `mouth` had been in the same
/// state since it landed.
fn placed_off_the_tier(view: &str) -> bool {
    ["cave", "overhang", "mouth", "seacave"].contains(&view)
}

fn cave_camera(
    mut commands: Commands,
    launch: Res<Launch>,
    fine: Res<pbd_app::planet::PlanetFine>,
) {
    if launch.capture.is_none() || launch.tour || launch.walk || launch.fly {
        return;
    }
    if !placed_off_the_tier(&launch.view) {
        return;
    }
    // The volumetric-water case is a chamber below sea level UNDER LAND: the
    // column holds no water at all, so the pocket is air, and only a height
    // field would call it sea. A pocket under the SEA is not it - the column
    // there really does hold water over the rock - and the first cut of this
    // pick took one, which came out as a frame of open ocean.
    let dry_cave_below_the_sea = launch.view == "seacave";
    use pbd_core::column::{layer_altitude, layer_at};
    let tier = &fine.set.columns;
    let records = fine.set.finest_records();
    // Air at this altitude in this cell's column, which is the one question
    // both the pick and the aim ask.
    let air = |cell: usize, altitude: f32| -> bool {
        let slot = tier.slots.get(cell).copied().unwrap_or(usize::MAX);
        if slot == usize::MAX {
            return false;
        }
        layer_at(altitude).is_some_and(|layer| !tier.columns[slot].solid(layer))
    };
    // The chamber a player can SEE DOWN, not the one with the most rock over
    // it. Burial was the sheet carve's pick and it was right for a slab, where
    // every chamber is the same two metres across and depth is all that is
    // left to choose by; with worms the frames differ by whether the tunnel
    // carries on, so the pick is the sight line, which is the instrument the
    // carve is measured with applied at capture time. Walking cell to cell,
    // always taking the neighbour most nearly ahead.
    let walk = |start: usize, heading: Vec3, altitude: f32| -> (usize, Vec3) {
        let mut cell = start;
        let mut at = Vec3::from_slice(&records[start].direction_height[..3]);
        let mut ahead = heading;
        for step in 1..=24 {
            let mut next: Option<(f32, usize, Vec3)> = None;
            for (side, &neighbor) in fine.set.finest_neighbors[cell]
                .iter()
                .enumerate()
                .take(records[cell].degree())
            {
                if neighbor == u32::MAX {
                    continue;
                }
                let there = Vec3::from_slice(&records[neighbor as usize].direction_height[..3]);
                let toward = (there - at).normalize_or_zero();
                let score = toward.dot(ahead);
                let _ = side;
                if next.is_none_or(|(had, ..)| score > had) {
                    next = Some((score, neighbor as usize, toward));
                }
            }
            let Some((_, neighbor, toward)) = next else {
                return (step - 1, ahead);
            };
            if !air(neighbor, altitude) {
                return (step - 1, ahead);
            }
            cell = neighbor;
            at = Vec3::from_slice(&records[neighbor].direction_height[..3]);
            ahead = toward;
        }
        (24, ahead)
    };
    let mut best: Option<(usize, Vec3, f32, f32, Vec3)> = None;
    for (index, &slot) in tier.slots.iter().enumerate() {
        if slot == usize::MAX {
            continue;
        }
        let column = &tier.columns[slot];
        let runs = column.drawn_runs();
        let surface = column
            .surface()
            .map_or(0.0, |top| layer_altitude(top) + 1.0);
        for pair in runs.windows(2) {
            let floor = layer_altitude(pair[0].to);
            let roof = layer_altitude(pair[1].from);
            let gap = roof - floor;
            let buried = surface - roof;
            if !(2.5..=12.0).contains(&gap) || !(4.0..=40.0).contains(&buried) {
                continue;
            }
            if dry_cave_below_the_sea && (roof >= 0.0 || surface <= 0.0) {
                continue;
            }
            let here = Vec3::from_slice(&records[index].direction_height[..3]);
            let eye_altitude = floor + EYE_HEIGHT.min(gap - 0.5);
            let tangent = Vec3::Y.cross(here).normalize_or_zero();
            let bitangent = here.cross(tangent);
            for turn in 0..6 {
                let angle = turn as f32 * std::f32::consts::TAU / 6.0;
                let heading = tangent * angle.cos() + bitangent * angle.sin();
                let (reach, _) = walk(index, heading, eye_altitude);
                if best.is_none_or(|(had, ..)| reach > had) {
                    best = Some((reach, here, floor, roof, heading));
                }
            }
        }
    }
    if launch.view == "mouth" {
        // Stand on the ground outside a tunnel opening and look into it. What
        // counts as an opening is `planet_column::mouth_of`, the same function
        // the mouth count and the walker read, so a frame cannot be taken of
        // something the count does not call a mouth.
        let mut best: Option<(f32, usize, usize, f32, f32)> = None;
        for (index, &slot) in tier.slots.iter().enumerate() {
            if slot == usize::MAX {
                continue;
            }
            let runs = tier.columns[slot].drawn_runs();
            let cell = &records[index];
            let Some((floor, roof, side)) = pbd_app::planet::mouth_of(cell, &runs) else {
                continue;
            };
            let neighbor = fine.set.finest_neighbors[index][side];
            if neighbor == u32::MAX || roof - floor < 1.8 {
                continue;
            }
            // The deepest tunnel behind the opening is the one worth looking
            // into: a one-cell notch is a doorway with a wall behind it.
            let depth = roof - floor;
            if best.is_none_or(|(had, ..)| depth > had) {
                best = Some((depth, index, neighbor as usize, floor, roof));
            }
        }
        let Some((depth, index, outside, floor, roof)) = best else {
            panic!(
                "no tunnel opening in the column tier; use --spawn mouth or raise worm_surface_share"
            )
        };
        let inside = Vec3::from_slice(&records[index].direction_height[..3]);
        let ground = Vec3::from_slice(&records[outside].direction_height[..3]);
        info!(
            "mouth capture: a {depth:.0} m opening, floor {floor:.0} m, roof {roof:.0} m, from ground at {:.0} m",
            records[outside].direction_height[3]
        );
        // Three cells back from the opening on the outside ground, at eye
        // height THERE: the ground a cell back is another cell's, and a frame
        // that stood at the outside cell's height from a cell back was inside
        // the rock of that cell, seeing the world from below through its cap.
        let back = (ground - inside).normalize_or_zero();
        let vantage = (ground + back * (4.0 / PLANET_RADIUS)).normalize();
        let there = pbd_app::planet::surface_height(vantage);
        let eye = vantage * (PLANET_RADIUS + there + EYE_HEIGHT);
        // Aim at the middle of the opening, which is where a walker's eyes go.
        let target = inside * (PLANET_RADIUS + (floor + roof) * 0.5 + EYE_HEIGHT * 0.5);
        let mut transform = Transform::from_translation(eye).looking_at(target, vantage);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }

    let Some((reach, here, floor, roof, along)) = best else {
        panic!("no chamber in the column tier to photograph; raise reach_m or worm_density")
    };
    let gap = roof - floor;
    info!(
        "cave capture: a {gap:.0} m chamber with {reach} cells of open tunnel ahead, floor \
         {floor:.0} m, roof {roof:.0} m, {:.0} m from the tier anchor",
        here.dot(fine.set.anchor).clamp(-1., 1.).acos() * PLANET_RADIUS
    );
    // `cave` stands on the chamber floor at eye height and looks along the
    // tunnel; `overhang` looks UP at the rock over it, which is the frame that
    // says the world has a ceiling.
    let eye_altitude = floor + EYE_HEIGHT.min(gap - 0.5);
    let eye = here * (PLANET_RADIUS + eye_altitude);
    let target = if launch.view == "overhang" {
        // The roof EIGHT metres down the tunnel, not the one overhead: a
        // ceiling a metre above the lens is one flat plane at a grazing angle,
        // which is a grey wash rather than a picture of a roof. Eight metres
        // out it recedes and the walls either side give it depth.
        (here + along * (8.0 / PLANET_RADIUS)).normalize() * (PLANET_RADIUS + roof - 0.3)
    } else {
        (here + along * (12.0 / PLANET_RADIUS)).normalize() * (PLANET_RADIUS + eye_altitude)
    };
    let mut transform = Transform::from_translation(eye).looking_at(target, here);
    transform.translation += launch.render_offset;
    commands.spawn((Camera3d::default(), transform));
}

fn photo_camera(
    mut commands: Commands,
    launch: Res<Launch>,
    water_settings: Res<pbd_app::config::WaterSettings>,
) {
    if launch.capture.is_none() || launch.tour || launch.walk || launch.fly {
        return;
    }
    // The column-tier views are placed a schedule later, off the live tier.
    if placed_off_the_tier(&launch.view) {
        return;
    }
    if launch.view == "river" {
        // A river is a channel pulled to a bed below the sea on lowland only,
        // so it is water with LAND on both sides a short way off - which is
        // what tells one from a bay. Walk the spawn's latitude for a wet cell
        // whose banks stand clear within sixty metres, and stand on one.
        // One latitude circle does not reliably cross a channel, so search the
        // sphere and take the one nearest the spawn: a wet cell whose banks
        // both stand clear thirty metres off, on each of two axes, which is a
        // watercourse rather than the edge of a bay.
        let seed = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let bank = 30.0 / PLANET_RADIUS;
        let golden = std::f32::consts::PI * (3.0 - 5_f32.sqrt());
        let samples = 300_000;
        let mut best: Option<(f32, Vec3, Vec3)> = None;
        for i in 0..samples {
            let y = 1.0 - 2.0 * (i as f32 + 0.5) / samples as f32;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let t = golden * i as f32;
            let here = Vec3::new(r * t.cos(), y, r * t.sin());
            let toward = here.dot(seed);
            let depth = surface_height(here);
            // The generator's own carve, not a guess from heights: a shallow
            // wet cell where the river field fires. Both banks must stand
            // clear on both axes without being a cliff, or this is a canyon
            // floor or the edge of a bay.
            if toward < 0.85 || !(-4.0..0.0).contains(&depth) {
                continue;
            }
            if river_channel(&TERRAIN, here) <= TERRAIN.river_threshold {
                continue;
            }
            let (a, b) = here.any_orthonormal_pair();
            let bankside =
                |axis: Vec3, sign: f32| surface_height((here + axis * bank * sign).normalize());
            if [(a, 1.0), (a, -1.0), (b, 1.0), (b, -1.0)]
                .into_iter()
                .any(|(axis, sign)| !(0.5..10.0).contains(&bankside(axis, sign)))
            {
                continue;
            }
            if best.is_none_or(|(near, _, _)| toward > near) {
                best = Some((toward, here, a));
            }
        }
        let (_, channel, across_axis) = best.unwrap_or((1.0, seed, Vec3::X));
        // Stand back on the near bank and look across the water.
        let here = (channel - across_axis * bank * 0.8).normalize();
        let across = (channel + across_axis * bank * 0.9).normalize();
        let height = launch.height.unwrap_or(EYE_HEIGHT);
        let eye = here * (terrain_radius(here) + height);
        let mut transform =
            Transform::from_translation(eye).looking_at(across * terrain_radius(across), here);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    if launch.view == "meadow" {
        // Every other ground preset stands at the spawn, and the spawn is in
        // jungle: a closed canopy at the reference's 115-of-256, which is
        // under three percent of this world. Pasture is a third of it and no
        // preset could photograph it. Walk east along the spawn's latitude to
        // the first pasture cell well clear of the shore and stand there.
        let seed = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let lat = seed.y.clamp(-1.0, 1.0).asin();
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let step = 4.0 * tile_width_m(FINEST_LEVEL) / PLANET_RADIUS;
        let mut lon = seed.z.atan2(seed.x);
        let start = lon;
        while lon < start + std::f32::consts::TAU {
            let here = at(lon);
            let height = surface_height(here);
            if height > 8.0 && surface_code(here, height) & 0xff == 2 {
                break;
            }
            lon += step;
        }
        let ground = at(lon);
        let height = launch.height.unwrap_or(EYE_HEIGHT);
        let east = (at(lon + step) - ground).normalize_or_zero();
        let eye = ground * (terrain_radius(ground) + height);
        let look = at(lon + 20.0 * step);
        let mut transform = Transform::from_translation(eye)
            .looking_at(look * terrain_radius(look) + east * height, ground);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    if ["shore", "nightshore", "midnight", "wade", "dive"].contains(&launch.view.as_str()) {
        // A capture instrument, nothing more: the eye-height polar shoreline the
        // owner asked to see. Above ~70 N the polar snow line reaches the sea,
        // so walk east from 72 N until land meets water, stand on the last land
        // cell at eye height, and look at the sea surface on the first water
        // cell. Bounded to one circuit so a latitude with no coast cannot spin.
        // `--height` lifts the eye and pushes the aim point out to sea by the
        // same distance, so every height in a series looks down at about 45
        // degrees instead of straight down.
        // `nightshore` is the same instrument on the night side. The sun is a
        // fixed direction, so a latitude can be in permanent day: 72 N is, at
        // every longitude. The equator is not, and its antisolar longitude is
        // the deepest night the body has, so that is where this one starts.
        // `midnight` is the antisolar POINT itself: the sun sits 48 degrees
        // north, so the equator's antisolar longitude is still 132 degrees
        // from the sun and its sky is lit by the upper atmosphere over the
        // limb; only at the antisolar latitude is the sky black in every
        // direction, which is the frame the owner's night report was taken in.
        let night = launch.view == "nightshore" || launch.view == "midnight";
        let sun = pbd_app::sky::SUN_DIRECTION.normalize();
        let lat = if launch.view == "midnight" {
            (-sun.y).clamp(-1.0, 1.0).asin()
        } else if night {
            0.0
        } else {
            72_f32.to_radians()
        };
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let step = 2.0 * tile_width_m(FINEST_LEVEL) / PLANET_RADIUS;
        let mut lon = if night { (-sun.z).atan2(-sun.x) } else { 0.0 };
        let start = lon;
        while surface_height(at(lon)) < 0.0 && lon < start + std::f32::consts::TAU {
            lon += step;
        }
        while surface_height(at(lon)) >= 0.0 && lon < start + 2.0 * std::f32::consts::TAU {
            lon += step;
        }
        let land = at(lon - step);
        let shore = at(lon);
        // `dive` needs water deep enough to hold the camera. The rescaled
        // shelf is one metre deep and stays under three for hundreds of
        // metres, so a fixed descent below the first water cell puts the eye
        // inside the seabed and the frame is flat deep-water colour with no
        // seabed, no surface and no Snell's window in it. Keep walking out
        // until the floor clears the requested depth. Bounded like the walk
        // above, and it falls back to the deepest cell the walk found.
        let depth = launch.height.unwrap_or(3.0);
        let water = if launch.view == "dive" {
            let clearance = -(depth + 1.0);
            let mut out = lon;
            let mut deepest = lon;
            while out < lon + std::f32::consts::TAU {
                let here = surface_height(at(out));
                if here <= clearance {
                    break;
                }
                if here < surface_height(at(deepest)) {
                    deepest = out;
                }
                out += step;
            }
            at(if surface_height(at(out)) <= clearance {
                out
            } else {
                deepest
            })
        } else {
            shore
        };
        let east = (shore - land).normalize_or_zero();
        // The sheet sits below sea level by the configured offset; `wade` puts
        // the eye inside the surface band over the first water cell, and
        // `dive` `--height` metres under over the first cell deep enough,
        // both looking out to sea.
        let sheet = PLANET_RADIUS - water_settings.depth_offset_m;
        let (eye, sea) = match launch.view.as_str() {
            "wade" => (water * (sheet + 0.1), water * sheet + east * 40.0),
            "dive" => (
                water * (sheet - depth),
                water * (sheet - depth - 1.0) + east * 40.0,
            ),
            _ => {
                let height = launch.height.unwrap_or(EYE_HEIGHT);
                (
                    land * (terrain_radius(land) + height),
                    water * terrain_radius(water) + east * height,
                )
            }
        };
        let mut transform = Transform::from_translation(eye).looking_at(sea, land);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    if launch.view == "column" {
        // A camera standing straight over the spawn at `--height` metres
        // above its ground, facing east and tilted by `--pitch` (-89 looks
        // straight down): a descent through the weather is this view at a
        // run of heights. The weather's "here" is the camera's own direction,
        // so a forced storm (`--rain`) brews directly under it at any height.
        let direction = Vec3::new(0.8776, 0.4794, 0.0).normalize();
        let height = launch.height.unwrap_or(EYE_HEIGHT);
        let position = direction * (terrain_radius(direction) + height);
        let east = Vec3::Y.cross(direction).normalize_or_zero();
        let pitch = launch.pitch.unwrap_or(-30.0).to_radians();
        let forward = east * pitch.cos() + direction * pitch.sin();
        let mut transform = Transform::from_translation(position).looking_to(forward, direction);
        transform.translation += launch.render_offset;
        commands.spawn((Camera3d::default(), transform));
        return;
    }
    let (direction, altitude, look_down) = match launch.view.as_str() {
        "orbit" => (Vec3::new(0.85, 0.48, 1.05).normalize(), 7800.0, true),
        "night" => (Vec3::new(-0.75, 0.12, 0.8).normalize(), 6200.0, true),
        "pole" => (Vec3::new(0.01, 1.0, 0.05).normalize(), 4200.0, true),
        "surface" => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 90.0, false),
        // A low eye at the spawn, looking down across the 300 m band boundary
        // at about eleven degrees: the still frame the hexagon-lod change asks
        // for. Higher than eye level so the spawn's own terraces and trees do
        // not fill the frame.
        "seam" => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 60.0, false),
        _ => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 420.0, false),
    };
    let position = direction * (terrain_radius(direction) + altitude);
    let tangent = Vec3::Y.cross(direction).normalize_or_zero();
    let target = if look_down {
        Vec3::ZERO
    } else if altitude < 80.0 {
        position + tangent * 300.0 - direction * altitude
    } else {
        position + tangent * 1600.0 - direction * if altitude > 300.0 { 900.0 } else { 150.0 }
    };
    let up = if look_down { Vec3::Y } else { direction };
    let mut transform = Transform::from_translation(position).looking_at(target, up);
    transform.translation += launch.render_offset;
    commands.spawn((Camera3d::default(), transform));
}

/// The scripted swim: put the walker at the last dry cell of the `shore` walk
/// and hold forward. It is the only way a headless capture can photograph the
/// water being entered, since a walker with no input stands still, and it is
/// the same shoreline the `shore`, `wade` and `dive` camera presets frame.
fn swim_script(
    launch: Res<Launch>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<pbd_app::walking::WalkingState>,
    mut placed: Local<bool>,
    terrain: Res<pbd_app::planet::PlanetContact>,
    mut walkers: Query<
        (
            &mut Position,
            &mut pbd_app::walking::GroundState,
            &mut Transform,
        ),
        With<pbd_app::walking::Walker>,
    >,
) {
    if !state.active {
        return;
    }
    if !*placed {
        // The swim's shore is a cold one; a fishing shore is warm enough
        // for the schools' windows on day one.
        let lat = if launch.fish { 24_f32 } else { 72_f32 }.to_radians();
        let at = |lon: f32| Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        let step = 2.0 * tile_width_m(FINEST_LEVEL) / PLANET_RADIUS;
        let mut lon = 0.0_f32;
        while surface_height(at(lon)) < 0.0 && lon < std::f32::consts::TAU {
            lon += step;
        }
        while surface_height(at(lon)) >= 0.0 && lon < 2.0 * std::f32::consts::TAU {
            lon += step;
        }
        // A fishing shore is the edge of open water: a river channel is also
        // below the datum, and the first one found at 24 degrees was a dry
        // canyon the float landed in. Twenty wet cells in a row is a sea.
        if launch.fish {
            let open = |lon: f32| (0..20).all(|k| surface_height(at(lon + k as f32 * step)) < -1.0);
            while !(surface_height(at(lon - step)) >= 0.0 && open(lon))
                && lon < 3.0 * std::f32::consts::TAU
            {
                lon += step;
            }
        }
        // The last dry cell, a few cells back from the water, facing the sea.
        // An angler stands at the water's edge rather than up the bank: a
        // cast carries ten to twenty metres.
        let land = at(lon - if launch.fish { 1.0 } else { 4.0 } * step);
        let sea = at(lon);
        let Ok((mut position, mut ground, mut transform)) = walkers.single_mut() else {
            return;
        };
        let stand = land * (terrain.sample(land).floor_radius + 1.0);
        position.0 = stand;
        transform.translation = stand;
        ground.previous = stand;
        ground.grounded = true;
        state.captured = true;
        state.scripted = true;
        state.face(land, (sea - land).normalize_or_zero());
        *placed = true;
    }
    if launch.swim {
        keys.press(KeyCode::KeyW);
    }
}

/// One scripted cast for `--fish`: the left button held for most of a second
/// from frame 90, and the fishery's state logged every second, so a run says
/// what the water held as well as showing it.
fn fish_script(
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    fishery: Option<Res<pbd_app::fish::Fishery>>,
    status: Option<Res<pbd_app::fish::FishingStatus>>,
    fauna: Res<pbd_app::config::FaunaConfig>,
    mut frame: Local<u32>,
) {
    *frame += 1;
    match *frame {
        90 => buttons.press(MouseButton::Left),
        91..=149 => {}
        150 => buttons.release(MouseButton::Left),
        _ => {}
    }
    if frame.is_multiple_of(60)
        && let (Some(fishery), Some(status)) = (fishery, status)
    {
        let roster = fauna.0.roster(pbd_core::fauna::HOME_BODY);
        let schools: Vec<String> = fishery
            .schools
            .iter()
            .map(|s| {
                let name = roster
                    .get(s.species as usize)
                    .map_or("?", |sp| sp.id.as_str());
                format!("{name} x{}", s.len())
            })
            .collect();
        info!(
            "fish script frame {}: line {:?}, {} schools [{}], status {:?}",
            *frame,
            fishery.line.phase,
            schools.len(),
            schools.join(", "),
            status.text
        );
    }
}

fn capture(
    mut commands: Commands,
    launch: Res<Launch>,
    input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<CaptureState>,
    stats: Res<FrameStats>,
    progress: Res<TourProgress>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frame += 1;
    let automatic = launch.capture.is_some() && state.frame >= launch.frames && !state.requested;
    if automatic || input.just_pressed(KeyCode::F12) {
        let path = if automatic {
            launch.capture.clone().unwrap()
        } else {
            let _ = std::fs::create_dir_all("output/captures");
            PathBuf::from(format!("output/captures/flight-{}.png", state.frame))
        };
        let saved_path = path.clone();
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>,
                  mut state: ResMut<CaptureState>,
                  mut exit: MessageWriter<AppExit>| {
                // Strip the HDR brightness alpha just as Bevy's saver does,
                // but only acknowledge success after the actual PNG write.
                let result = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .map_err(|error| format!("{error:?}"))
                    .and_then(|image| {
                        image
                            .to_rgb8()
                            .save(&saved_path)
                            .map_err(|error| error.to_string())
                    });
                match result {
                    Ok(()) => {
                        println!("CAPTURE_SAVED {}", saved_path.display());
                        if automatic {
                            state.saved = true;
                            state.finish_frame = state.frame + 4;
                        }
                    }
                    Err(error) => {
                        eprintln!("CAPTURE_FAILED {}: {error}", saved_path.display());
                        if automatic {
                            exit.write(AppExit::error());
                        }
                    }
                }
            },
        );
        if automatic {
            state.requested = true;
        }
        println!("CAPTURE_REQUEST {} frame={}", path.display(), state.frame);
    }
    if launch.capture.is_some() && state.saved && state.frame >= state.finish_frame {
        if launch.tour {
            println!(
                "RENDERED_TOUR completed={} degrees={:.3} seconds={:.3} min_clearance_m={:.3} max_speed_m_s={:.3} protection_events={}",
                progress.completed,
                progress.angular_distance_rad.to_degrees(),
                progress.elapsed_seconds,
                progress.minimum_clearance,
                progress.maximum_speed,
                progress.protection_events
            );
        }
        // Exclude startup and the screenshot's GPU readback / PNG disk write.
        let mut samples: Vec<f64> = stats
            .samples
            .iter()
            .copied()
            .take(launch.frames.saturating_sub(1) as usize)
            .skip(60)
            .collect();
        samples.sort_by(f64::total_cmp);
        if !samples.is_empty() {
            let at = |q: f64| samples[((samples.len() - 1) as f64 * q) as usize];
            println!(
                "FRAME_WALL_MS p50={:.2} p95={:.2} p99={:.2} samples={}",
                at(0.5),
                at(0.95),
                at(0.99),
                samples.len()
            );
        }
        exit.write(AppExit::Success);
    }
    if launch.capture.is_some() && state.frame > launch.frames + 900 && !state.saved {
        eprintln!("Screenshot did not complete");
        exit.write(AppExit::error());
    }
}

/// The `--frame-log` CSV, open for the run when asked for.
#[derive(Resource)]
struct FrameLog(Option<std::io::BufWriter<std::fs::File>>);

impl FrameLog {
    fn open(path: Option<&std::path::Path>) -> Self {
        Self(path.map(|path| {
            use std::io::Write;
            let file = std::fs::File::create(path)
                .unwrap_or_else(|error| panic!("--frame-log {}: {error}", path.display()));
            let mut out = std::io::BufWriter::new(file);
            writeln!(
                out,
                "frame,since_start_s,wall_ms,fine_version,rebuild_s,clearance_m,speed_mps"
            )
            .expect("frame log header");
            out
        }))
    }
}

/// One row per frame: the frame's wall time, the fine set drawn, the age of
/// the rebuild in flight (empty when none), and how high and fast the player
/// is, so an over-budget frame lines up with what the streaming was doing.
fn write_frame_log(
    mut log: ResMut<FrameLog>,
    stats: Res<FrameStats>,
    fine: Res<pbd_app::planet::PlanetFine>,
    near: Res<pbd_app::planet::NearField>,
    readout: Option<Res<pbd_app::flight_view::FlightReadout>>,
) {
    use std::io::Write;
    let Some(out) = log.0.as_mut() else {
        return;
    };
    let Some(ms) = stats.samples.last() else {
        return;
    };
    let rebuild = near.rebuild_s.map_or(String::new(), |s| format!("{s:.3}"));
    let (clearance, speed) = readout.map_or((f32::NAN, f32::NAN), |r| (r.clearance, r.speed));
    let _ = writeln!(
        out,
        "{},{:.3},{ms:.3},{},{rebuild},{clearance:.1},{speed:.1}",
        stats.samples.len(),
        stats.started.elapsed().as_secs_f64(),
        fine.version
    );
}

fn measure_frames(mut stats: ResMut<FrameStats>) {
    // Presentation handles interactive pacing. A second main-thread sleep
    // delayed freshly sampled mouse input without improving displayed motion.
    let now = Instant::now();
    let ms = now.duration_since(stats.previous).as_secs_f64() * 1000.0;
    stats.previous = now;
    stats.frame_ms = stats.frame_ms * 0.9 + ms as f32 * 0.1;
    if stats.samples.len() < 100_000 {
        stats.samples.push(ms);
    }
}

/// `--verify-route far-side`: fly the whole route headless and report what the
/// window would show: that it completes and lands where it meant to, its peak
/// height, its clearance while airborne, and the camera rig's fastest turn.
fn verify_route() {
    let mut app = pbd_app::headless_app();
    app.add_plugins(FlightViewPlugin)
        .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
        .insert_resource(FlightViewConfig {
            mode: FlyMode::Route,
            spawn_direction: Vec3::new(0.8776, 0.4794, 0.0).normalize(),
            minimum_clearance: 1.6,
            startup_camera: false,
            ..default()
        });
    app.finish();
    app.cleanup();
    app.update();
    for _ in 0..60_000 {
        app.update();
        if app
            .world()
            .resource::<pbd_app::flight_view::RouteState>()
            .completed
        {
            break;
        }
    }
    let config = *app.world().resource::<FlightViewConfig>();
    let r = *app.world().resource::<pbd_app::flight_view::RouteState>();
    let t = *app.world().resource::<TourProgress>();
    let report = format!(
        "ROUTE far-side completed={} seconds={:.1} destination_deg={:.2} touchdown_error_m={:.1} peak_height_m={:.0} min_airborne_clearance_m={:.1} max_speed_m_s={:.1} max_camera_rate_rad_s={:.3} protection_events={}\n",
        r.completed,
        r.elapsed_s,
        r.destination_rad.to_degrees(),
        r.touchdown_error_m,
        r.peak_height_m,
        r.min_airborne_clearance_m,
        r.max_speed_mps,
        r.max_camera_rate_rad_s,
        t.protection_events,
    );
    print!("{report}");
    std::fs::create_dir_all("output/captures").unwrap();
    std::fs::write("output/captures/route-far-side.txt", report).unwrap();
    assert!(r.completed, "the route did not land and settle");
    assert!(
        r.touchdown_error_m < 30.0,
        "landed away from the destination"
    );
    assert!(
        r.peak_height_m > config.route_cruise_height_m * 0.95,
        "never reached cruise height"
    );
    assert!(
        r.max_speed_mps <= config.route_speed + 0.5,
        "route speed exceeded"
    );
    assert!(r.max_camera_rate_rad_s < 1.0, "the camera turned too fast");
}

fn verify_flight(polar: bool) {
    let mut app = pbd_app::headless_app();
    app.add_plugins(FlightViewPlugin)
        .insert_resource(CelestialScene::planet_at_origin(PLANET_RADIUS as f64, 1.0))
        .insert_resource(FlightViewConfig {
            mode: FlyMode::Tour,
            spawn_direction: if polar {
                Vec3::Y
            } else {
                Vec3::new(0.8776, 0.4794, 0.0).normalize()
            },
            startup_camera: false,
            ..default()
        });
    app.finish();
    app.cleanup();
    app.update();
    for _ in 0..30_000 {
        app.update();
        if app.world().resource::<TourProgress>().completed {
            break;
        }
    }
    let p = app.world().resource::<TourProgress>();
    let report = format!(
        "CIRCUMNAVIGATION completed={} degrees={:.3} seconds={:.3} min_clearance_m={:.3} max_speed_m_s={:.3} max_command_acceleration_m_s2={:.3} max_angular_speed_rad_s={:.3} max_angular_acceleration_rad_s2={:.3} protection_events={}\n",
        p.completed,
        p.angular_distance_rad.to_degrees(),
        p.elapsed_seconds,
        p.minimum_clearance,
        p.maximum_speed,
        p.maximum_acceleration,
        p.maximum_angular_speed,
        p.maximum_angular_acceleration,
        p.protection_events
    );
    print!("{report}");
    std::fs::create_dir_all("output/captures").unwrap();
    std::fs::write(
        if polar {
            "output/captures/circumnavigation-polar.txt"
        } else {
            "output/captures/circumnavigation.txt"
        },
        report,
    )
    .unwrap();
    assert!(
        p.completed,
        "tour did not complete a full orbit around the surface"
    );
    assert!(p.minimum_clearance >= 45.0, "terrain clearance violated");
    assert!(p.maximum_speed <= 600.1, "flight speed limit violated");
    assert!(
        p.maximum_acceleration <= 80.01,
        "command acceleration limit violated"
    );
    assert!(
        p.maximum_angular_speed <= 1.51,
        "angular speed limit violated"
    );
    assert!(
        p.maximum_angular_acceleration <= 3.01,
        "angular acceleration limit violated"
    );
    assert_eq!(
        p.protection_events, 0,
        "tour needed emergency terrain projection"
    );
}
