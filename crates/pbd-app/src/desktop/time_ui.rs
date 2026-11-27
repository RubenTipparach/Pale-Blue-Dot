//! The time of day on the pause panel (`lamps-and-lanterns` task 4.3): a bar
//! across the day that IS the world clock's hour, DAY and NIGHT presets, and
//! L to switch between noon and midnight, as the towns mockup has them.
//!
//! It moves `Sun::clock`, the one clock the sun, the stars and the street
//! lamps read. So moving it past dusk lights the lamps by the same path the
//! clock's own turning does (`switch_dusk_lamps`); there is no second way to
//! make it night.

use super::menu::{MenuAction, NameField, small};
use bevy::{prelude::*, ui::RelativeCursorPosition};
use pbd_app::sky::{Sun, TIME_SPEEDS, TimeSpeed};
use pbd_app::ui_kit::{self, KitText, UiKit};
use pbd_core::daylight::Clock;
use pbd_core::locale::{Localized, tr, tr_with};

/// The bar a press or a drag sets the hour from.
#[derive(Component)]
pub struct TimeSlider;

/// The lit part of the bar, as far across as the day has gone.
#[derive(Component)]
pub struct TimeFill;

/// The line that says what hour it is.
#[derive(Component)]
pub struct TimeLabel;

/// The oak grip on the groove.
#[derive(Component)]
pub struct TimeKnob;

/// The bar's width, px: the weather bar's, so the two read as a pair.
const BAR_WIDTH: f32 = 172.0;

/// Noon and midnight, the presets and the two ends of L.
pub const NOON: f32 = 12.0;
pub const MIDNIGHT: f32 = 0.0;

/// The same day at another hour. The day is kept, so the season does not
/// jump when a player asks for night.
pub fn at_hour(clock: Clock, hour: f32) -> Clock {
    Clock::at(clock.day() as u32, hour.clamp(0.0, 24.0))
}

/// What L does: midnight from the day, noon from the night.
pub fn flipped(clock: Clock) -> Clock {
    let hour = clock.hour();
    at_hour(
        clock,
        if (6.0..18.0).contains(&hour) {
            MIDNIGHT
        } else {
            NOON
        },
    )
}

/// The control, laid into the pause panel above its buttons.
pub fn spawn(panel: &mut ChildSpawnerCommands, kit: &UiKit) {
    panel.spawn((
        TimeLabel,
        KitText::new(tr("time.title"), ui_kit::TITLE).shadow(ui_kit::INK),
    ));
    super::menu::slider(
        panel,
        kit,
        BAR_WIDTH,
        (TimeSlider, Button, RelativeCursorPosition::default()),
        TimeFill,
        TimeKnob,
    );
    panel
        .spawn(Node {
            column_gap: px(2),
            margin: UiRect::bottom(px(3)),
            ..default()
        })
        .with_children(|row| {
            small(row, kit, tr("time.day"), MenuAction::Time(NOON as u8));
            small(row, kit, tr("time.night"), MenuAction::Time(MIDNIGHT as u8));
        });
}

/// The clock's speed and the jumps ahead, on the DEBUG page (`time-speed`;
/// owner, 2026-11-26: "buttons to help speed up time advancement in game").
pub fn spawn_speed(panel: &mut ChildSpawnerCommands, kit: &UiKit) {
    panel.spawn(KitText::new(tr("time.speed"), ui_kit::TITLE).shadow(ui_kit::INK));
    panel
        .spawn(Node {
            column_gap: px(2),
            margin: UiRect::bottom(px(3)),
            ..default()
        })
        .with_children(|row| {
            for rate in TIME_SPEEDS {
                small(
                    row,
                    kit,
                    tr_with("time.rate", &[("rate", &rate.to_string())]),
                    MenuAction::Speed(rate),
                );
            }
        });
    panel
        .spawn(Node {
            column_gap: px(2),
            margin: UiRect::bottom(px(3)),
            ..default()
        })
        .with_children(|row| {
            small(row, kit, tr("time.hour_on"), MenuAction::Skip(1));
            small(row, kit, tr("time.day_on"), MenuAction::Skip(24));
        });
}

/// World seconds per real second while `]` or `[` is held: a game hour every
/// five seconds (owner, 2026-11-26: "if held down advance 1 hour = 5
/// seconds"), and with Shift a game day every five ("shift + [,] would be
/// like 1 day per 5 seconds").
pub const HELD_RATE: f32 = pbd_core::daylight::DAY_S / 24.0 / 5.0;
pub const SHIFT_HELD_RATE: f32 = pbd_core::daylight::DAY_S / 5.0;

/// How fast a held bracket runs the clock now, or `None` with none held.
fn held_rate(keys: &ButtonInput<KeyCode>) -> Option<f32> {
    let way = f32::from(u8::from(keys.pressed(KeyCode::BracketRight)))
        - f32::from(u8::from(keys.pressed(KeyCode::BracketLeft)));
    if way == 0.0 {
        return None;
    }
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    Some(way * if shift { SHIFT_HELD_RATE } else { HELD_RATE })
}

/// The tag under the compass while the clock runs fast.
#[derive(Component)]
pub struct FastTag;

/// A press on the bar, and every frame it is held, sets the hour to where the
/// cursor is along it; a press on DAY or NIGHT sets that hour.
pub fn press(
    bars: Query<(&Interaction, &RelativeCursorPosition), With<TimeSlider>>,
    buttons: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut sun: ResMut<Sun>,
    mut speed: ResMut<TimeSpeed>,
    player: Res<pbd_app::settings::PlayerSettings>,
) {
    // The time controls off: the clock back at its own pace.
    if !player.time_controls && speed.0 != 1.0 {
        speed.0 = 1.0;
    }
    for (interaction, cursor) in &bars {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(at) = cursor.normalized {
            // Normalised is centred on the node: -0.5 at the left edge.
            let hour = (at.x + 0.5).clamp(0.0, 1.0) * 24.0;
            if (hour - sun.clock.hour()).abs() > 0.01 {
                sun.clock = at_hour(sun.clock, hour);
            }
        }
    }
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MenuAction::Time(hour) => sun.clock = at_hour(sun.clock, f32::from(*hour)),
            MenuAction::Speed(_) | MenuAction::Skip(_) if !player.time_controls => {}
            MenuAction::Speed(rate) => {
                speed.0 = f32::from(*rate);
                info!("time speed {rate}x");
            }
            MenuAction::Skip(hours) => {
                sun.clock
                    .advance(f32::from(*hours) * pbd_core::daylight::DAY_S / 24.0);
                info!(
                    "time skipped {hours} h: the hour is now {:.1}",
                    sun.clock.hour()
                );
            }
            _ => {}
        }
    }
}

/// L switches between noon and midnight, wherever the player is, except
/// while a world's name is being typed.
///
/// `]` held runs the clock forward a game hour every five seconds, `[` held
/// runs it back at the same rate, and with Shift a day every five seconds
/// (`time-speed` design 1).
pub fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    name: Res<NameField>,
    time: Res<Time>,
    player: Res<pbd_app::settings::PlayerSettings>,
    mut sun: ResMut<Sun>,
) {
    if name.focused {
        return;
    }
    if keys.just_pressed(KeyCode::KeyL) {
        sun.clock = flipped(sun.clock);
        info!("L: the hour is now {:.1}", sun.clock.hour());
    }
    if player.time_controls
        && let Some(rate) = held_rate(&keys)
    {
        sun.clock.advance(rate * time.delta_secs());
    }
}

/// The tag under the compass while the clock runs faster than its own
/// pace, so a fast clock is never left on unseen: the pause panel's speed,
/// or a bracket held.
pub fn tag(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    speed: Res<TimeSpeed>,
    player: Res<pbd_app::settings::PlayerSettings>,
    mut tags: Query<(&mut KitText, &mut Visibility), With<FastTag>>,
    mut spawned: Local<bool>,
) {
    if !*spawned {
        *spawned = true;
        commands
            .spawn((Node {
                position_type: PositionType::Absolute,
                top: px(46),
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            },))
            .with_children(|row| {
                row.spawn((
                    KitText::new(Localized::empty(), ui_kit::TEXT).outline(ui_kit::INK),
                    Visibility::Hidden,
                    FastTag,
                ));
            });
        return;
    }
    let held = held_rate(&keys).filter(|_| player.time_controls);
    // A held bracket's pace as a multiple of the clock's own (a world
    // second a second), signed.
    let rate = match held {
        Some(r) => r.round() as i32,
        None => speed.0.round() as i32,
    };
    let fast = rate != 1;
    for (mut text, mut shown) in &mut tags {
        let want = if fast {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *shown != want {
            *shown = want;
        }
        if fast {
            let words = tr_with("time.fast", &[("rate", &rate.to_string())]);
            if text.text != words {
                text.text = words;
            }
        }
    }
}

/// The bar and the label follow the clock, however it was set.
pub fn show(
    sun: Res<Sun>,
    mut fills: Query<&mut Node, (With<TimeFill>, Without<TimeKnob>)>,
    mut knobs: Query<&mut Node, (With<TimeKnob>, Without<TimeFill>)>,
    mut labels: Query<&mut KitText, With<TimeLabel>>,
) {
    let hour = sun.clock.hour();
    let at = percent(hour / 24.0 * 100.0);
    for mut node in &mut fills {
        if node.width != at {
            node.width = at;
        }
    }
    for mut node in &mut knobs {
        if node.left != at {
            node.left = at;
        }
    }
    for mut text in &mut labels {
        let minutes = (hour * 60.0).round() as u32 % (24 * 60);
        let shown = tr_with(
            "time.line",
            &[
                ("hours", &format!("{:02}", minutes / 60)),
                ("minutes", &format!("{:02}", minutes % 60)),
            ],
        );
        if text.text != shown {
            text.text = shown;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l_goes_from_day_to_midnight_and_back_to_noon_on_the_same_day() {
        let morning = Clock::at(7, 9.5);
        let night = flipped(morning);
        assert_eq!(night.day(), 7, "the same day");
        assert!(night.hour().abs() < 1e-3, "midnight: {}", night.hour());
        let day = flipped(night);
        assert_eq!(day.day(), 7);
        assert!((day.hour() - NOON).abs() < 1e-3, "noon: {}", day.hour());
    }

    /// Moving the clock past dusk is what lights the street lamps, by the
    /// clock's own path: the control sets the hour and the lamps read it.
    #[test]
    fn the_control_moves_the_clock_the_lamps_read() {
        let noon = at_hour(Clock::at(3, 12.0), NOON);
        let here = noon.sun();
        assert!(!noon.is_dusk_lit(here, false), "out at noon");
        let night = at_hour(noon, MIDNIGHT);
        assert!(night.is_dusk_lit(here, false), "lit once moved to midnight");
    }
}

#[cfg(test)]
mod speed_tests {
    use super::*;
    use pbd_core::daylight::DAY_S;

    /// A held bracket moves the clock an hour in five seconds, and with
    /// Shift a day (`time-speed` design 1); +1 DAY moves the day by one.
    #[test]
    fn held_brackets_and_the_jumps_move_the_clock_as_asked() {
        assert!((HELD_RATE * 5.0 - DAY_S / 24.0).abs() < 1e-3);
        assert!((SHIFT_HELD_RATE * 5.0 - DAY_S).abs() < 1e-3);
        let mut keys = ButtonInput::<KeyCode>::default();
        assert_eq!(held_rate(&keys), None);
        keys.press(KeyCode::BracketRight);
        assert_eq!(held_rate(&keys), Some(HELD_RATE));
        keys.press(KeyCode::ShiftLeft);
        assert_eq!(held_rate(&keys), Some(SHIFT_HELD_RATE));
        keys.release_all();
        keys.press(KeyCode::BracketLeft);
        assert_eq!(held_rate(&keys), Some(-HELD_RATE));
        let mut clock = Clock::at(3, 10.0);
        clock.advance(24.0 * DAY_S / 24.0);
        assert_eq!(clock.day(), 4);
        assert!((clock.hour() - 10.0).abs() < 1e-3);
    }
}
