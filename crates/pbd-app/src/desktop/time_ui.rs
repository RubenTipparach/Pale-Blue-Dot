//! The time of day on the pause panel (`lamps-and-lanterns` task 4.3): a bar
//! across the day that IS the world clock's hour, DAY and NIGHT presets, and
//! L to switch between noon and midnight, as the towns mockup has them.
//!
//! It moves `Sun::clock`, the one clock the sun, the stars and the street
//! lamps read. So moving it past dusk lights the lamps by the same path the
//! clock's own turning does (`switch_dusk_lamps`); there is no second way to
//! make it night.

use super::menu::{EDGE, MINT, MenuAction, NameField, small};
use bevy::{prelude::*, ui::RelativeCursorPosition};
use pbd_app::sky::Sun;
use pbd_core::daylight::Clock;

/// The bar a press or a drag sets the hour from.
#[derive(Component)]
pub struct TimeSlider;

/// The lit part of the bar, as far across as the day has gone.
#[derive(Component)]
pub struct TimeFill;

/// The line that says what hour it is.
#[derive(Component)]
pub struct TimeLabel;

/// The bar's width, px: the weather bar's, so the two read as a pair.
const BAR_WIDTH: f32 = 220.0;

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
pub fn spawn(panel: &mut ChildSpawnerCommands) {
    panel.spawn((
        TimeLabel,
        Text::new("TIME"),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(MINT),
    ));
    panel
        .spawn((
            TimeSlider,
            Button,
            RelativeCursorPosition::default(),
            Node {
                width: px(BAR_WIDTH),
                height: px(14),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(EDGE),
            BackgroundColor(Color::srgba(0.04, 0.10, 0.12, 0.9)),
        ))
        .with_children(|bar| {
            bar.spawn((
                TimeFill,
                Node {
                    width: percent(0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.95, 0.78, 0.40, 0.75)),
            ));
        });
    panel
        .spawn(Node {
            column_gap: px(6),
            margin: UiRect::bottom(px(6)),
            ..default()
        })
        .with_children(|row| {
            small(row, "DAY", MenuAction::Time(NOON as u8));
            small(row, "NIGHT", MenuAction::Time(MIDNIGHT as u8));
        });
}

/// A press on the bar, and every frame it is held, sets the hour to where the
/// cursor is along it; a press on DAY or NIGHT sets that hour.
pub fn press(
    bars: Query<(&Interaction, &RelativeCursorPosition), With<TimeSlider>>,
    buttons: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut sun: ResMut<Sun>,
) {
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
        if let (Interaction::Pressed, MenuAction::Time(hour)) = (interaction, action) {
            sun.clock = at_hour(sun.clock, f32::from(*hour));
        }
    }
}

/// L switches between noon and midnight, wherever the player is, except
/// while a world's name is being typed.
pub fn toggle(keys: Res<ButtonInput<KeyCode>>, name: Res<NameField>, mut sun: ResMut<Sun>) {
    if keys.just_pressed(KeyCode::KeyL) && !name.focused {
        sun.clock = flipped(sun.clock);
        info!("L: the hour is now {:.1}", sun.clock.hour());
    }
}

/// The bar and the label follow the clock, however it was set.
pub fn show(
    sun: Res<Sun>,
    mut fills: Query<&mut Node, With<TimeFill>>,
    mut labels: Query<&mut Text, With<TimeLabel>>,
) {
    let hour = sun.clock.hour();
    for mut node in &mut fills {
        node.width = percent(hour / 24.0 * 100.0);
    }
    for mut text in &mut labels {
        let minutes = (hour * 60.0).round() as u32 % (24 * 60);
        let shown = format!("TIME  {:02}:{:02}", minutes / 60, minutes % 60);
        if text.0 != shown {
            text.0 = shown;
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
