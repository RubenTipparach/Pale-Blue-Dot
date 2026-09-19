use super::{FrameStats, Launch};
use bevy::prelude::*;
use pbd_app::flight_view::{FlightReadout, TourProgress};
use pbd_app::walking::WalkingReadout;

#[derive(Component)]
pub struct Instruments;
#[derive(Component)]
pub struct Status;
#[derive(Component)]
pub struct Controls;

pub fn setup(mut commands: Commands) {
    let pale = Color::srgb(0.88, 0.94, 0.91);
    let mint = Color::srgb(0.48, 0.8, 0.77);
    // Keep small flight instruments readable over sea glints and snow.
    for (top, bottom, height) in [(px(0), Val::Auto, 94.0), (Val::Auto, px(0), 148.0)] {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top,
                bottom,
                left: px(0),
                right: px(0),
                height: px(height),
                ..default()
            },
            BackgroundColor(Color::srgba(0.003, 0.012, 0.018, 0.64)),
            GlobalZIndex(-1),
        ));
    }
    commands.spawn((
        Text::new("DAMPENERS ONLINE"),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(mint),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            top: px(36),
            right: px(36),
            ..default()
        },
        Status,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(3),
            height: px(3),
            ..default()
        },
        BackgroundColor(pale.with_alpha(0.75)),
    ));
    commands.spawn((
        Text::new("INITIALIZING FLIGHT"),
        TextFont {
            font_size: 17.0,
            ..default()
        },
        TextColor(pale),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(30),
            left: px(36),
            ..default()
        },
        Instruments,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(mint),
        TextShadow::default(),
        TextLayout::new_with_justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(74),
            left: px(0),
            right: px(0),
            ..default()
        },
        Controls,
    ));
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update(
    time: Res<Time>,
    readout: Res<FlightReadout>,
    walking: Option<Res<WalkingReadout>>,
    progress: Res<TourProgress>,
    launch: Res<Launch>,
    stats: Res<FrameStats>,
    mut instruments: Query<&mut Text, (With<Instruments>, Without<Controls>)>,
    mut status: Query<&mut Text, (With<Status>, Without<Instruments>, Without<Controls>)>,
    mut controls: Query<&mut Text, (With<Controls>, Without<Status>, Without<Instruments>)>,
    mut last: Local<f64>,
) {
    if time.elapsed_secs_f64() - *last < 0.1 {
        return;
    }
    *last = time.elapsed_secs_f64();
    let walker = walking.as_deref().filter(|w| w.active);
    for mut text in &mut controls {
        // One line naming what is to hand, not every binding there is. The
        // wall it replaced was two lines of twelve, which is a manual rather
        // than a HUD.
        let label = if walker.is_some() {
            "WASD  walk    SPACE  jump    1-0  slot    F  fly    H  keys"
        } else {
            "WASD  fly    SPACE / CTRL  lift    Q / E  roll    F  walk    H  keys"
        };
        if text.0 != label {
            text.0 = label.into();
        }
    }
    for mut text in &mut instruments {
        text.0 = if let Some(walker) = walker {
            format!(
                "{:>5.1} m/s       {:>6.0} m ALT\n{:+06.1} LAT     {:+07.1} LON",
                walker.speed, walker.altitude, walker.latitude_deg, walker.longitude_deg
            )
        } else {
            format!(
                "{:>5.0} m/s       {:>6.0} m ALT\n{:+06.1} LAT     {:+07.1} LON",
                readout.speed, readout.altitude, readout.latitude_deg, readout.longitude_deg
            )
        };
    }
    for mut text in &mut status {
        text.0 = if let Some(walker) = walker {
            format!(
                "{}  /  {}    {:03.0} FPS",
                if walker.sprinting { "SPRINT" } else { "WALK" },
                if walker.grounded {
                    "GROUNDED"
                } else {
                    "AIRBORNE"
                },
                1000.0 / stats.frame_ms.max(0.1)
            )
        } else if launch.tour {
            format!(
                "SURVEY AUTOPILOT  /  {:03.0} OF 360 DEG",
                progress.angular_distance_rad.to_degrees()
            )
        } else {
            format!(
                "{}  /  {}    {:03.0} FPS",
                if readout.dampeners {
                    "DAMPENERS ON"
                } else {
                    "DAMPENERS OFF"
                },
                if readout.cruise { "CRUISE" } else { "SURFACE" },
                1000.0 / stats.frame_ms.max(0.1)
            )
        };
    }
}
