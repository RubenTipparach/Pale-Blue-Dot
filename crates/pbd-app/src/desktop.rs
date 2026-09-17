mod hud;
mod scene;

use avian3d::prelude::*;
use bevy::{
    app::AppExit,
    asset::AssetPlugin,
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::view::{
        Hdr,
        screenshot::{Screenshot, ScreenshotCaptured},
    },
    time::TimeUpdateStrategy,
    window::PresentMode,
};
use pbd_app::{
    CelestialScene, FIXED_HZ, PaleBlueDotPlugin, PhysicsFrame,
    flight_view::{FlightViewConfig, FlightViewPlugin, FlyMode, TourProgress},
    planet::{PLANET_RADIUS, PlanetPlugin, terrain_radius},
    sky::SkyPlugin,
    walking::{WalkingConfig, WalkingPlugin},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

#[derive(Resource, Clone)]
pub struct Launch {
    pub capture: Option<PathBuf>,
    pub view: String,
    pub frames: u32,
    pub tour: bool,
    pub fixed: bool,
    pub fly: bool,
    pub walk: bool,
    /// Static capture instrument: translate the scene within the local frame.
    pub render_offset: Vec3,
}

impl Launch {
    fn parse(args: &[String]) -> Self {
        let mut result = Self {
            capture: None,
            view: "coast".into(),
            frames: 180,
            tour: false,
            fixed: false,
            fly: false,
            walk: false,
            render_offset: Vec3::ZERO,
        };
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--capture" => {
                    i += 1;
                    result.capture =
                        Some(args.get(i).expect("--capture requires a PNG path").into());
                }
                "--view" => {
                    i += 1;
                    result.view = args.get(i).expect("--view requires a view name").clone();
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
                "--fly" => result.fly = true,
                "--walk" => result.walk = true,
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
                "--verify-flight" => {}
                unknown => panic!("unknown argument {unknown}; use --help"),
            }
            i += 1;
        }
        assert!(
            !(result.walk && (result.fly || result.tour)),
            "--walk cannot be combined with --fly or --tour"
        );
        assert!(
            ["orbit", "coast", "surface", "night", "pole"].contains(&result.view.as_str()),
            "unknown capture view"
        );
        assert!(
            result.frames >= 60 && result.frames <= 100_000,
            "capture frames must be 60..100000"
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
}

pub fn run(args: &[String]) {
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
        mode: if launch.tour {
            FlyMode::Tour
        } else {
            FlyMode::Manual
        },
        spawn_direction: if launch.tour && launch.view == "pole" {
            Vec3::Y
        } else {
            Vec3::new(0.8776, 0.4794, 0.0).normalize()
        },
        spawn_altitude: 240.0,
        minimum_clearance: if launch.tour { 45.0 } else { 1.6 },
        startup_camera: !photo,
        ..default()
    })
    .insert_resource(ClearColor(Color::srgb(0.002, 0.004, 0.012)))
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
    })
    .insert_resource(launch.clone())
    .add_systems(Startup, (scene::setup, hud::setup, photo_camera))
    .add_systems(
        Update,
        (configure_camera, scene::move_moon, hud::update, capture),
    )
    .add_systems(Last, measure_frames);
    if !photo && !launch.tour {
        app.insert_resource(WalkingConfig {
            start_walking: !launch.fly,
            ..default()
        })
        .add_plugins(WalkingPlugin);
    }
    if launch.fixed || launch.capture.is_some() {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(step));
    }
    app.run();
}

fn configure_camera(mut commands: Commands, cameras: Query<Entity, Added<Camera3d>>) {
    for entity in &cameras {
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

fn photo_camera(mut commands: Commands, launch: Res<Launch>) {
    if launch.capture.is_none() || launch.tour || launch.walk || launch.fly {
        return;
    }
    let (direction, altitude, look_down) = match launch.view.as_str() {
        "orbit" => (Vec3::new(0.85, 0.48, 1.05).normalize(), 7800.0, true),
        "night" => (Vec3::new(-0.75, 0.12, 0.8).normalize(), 6200.0, true),
        "pole" => (Vec3::new(0.01, 1.0, 0.05).normalize(), 4200.0, true),
        "surface" => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 90.0, false),
        _ => (Vec3::new(0.8776, 0.4794, 0.0).normalize(), 420.0, false),
    };
    let position = direction * (terrain_radius(direction) + altitude);
    let tangent = Vec3::Y.cross(direction).normalize_or_zero();
    let target = if look_down {
        Vec3::ZERO
    } else {
        position + tangent * 1600.0 - direction * if altitude > 300.0 { 900.0 } else { 150.0 }
    };
    let up = if look_down { Vec3::Y } else { direction };
    let mut transform = Transform::from_translation(position).looking_at(target, up);
    transform.translation += launch.render_offset;
    commands.spawn((Camera3d::default(), transform));
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
