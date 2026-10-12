//! The weather control on the pause panel: a bar from clear to storm that IS
//! the storm forcing, and three presets. It moves the field exactly as the P
//! key and `--rain` do (`StormForcing`), so there is one path to the weather
//! and the menu is only another hand on it.

use super::menu::{EDGE, MINT, MenuAction, small};
use bevy::{prelude::*, ui::RelativeCursorPosition};
use pbd_app::weather::StormForcing;

/// The bar a press or a drag sets the forcing from.
#[derive(Component)]
pub struct WeatherSlider;

/// The lit part of the bar, as wide as the forcing.
#[derive(Component)]
pub struct WeatherFill;

/// The line that says what the bar is set to.
#[derive(Component)]
pub struct WeatherLabel;

/// The bar's width, px.
const BAR_WIDTH: f32 = 220.0;

/// What a forcing reads as, in words a player would use.
pub fn describe(forcing: f32) -> &'static str {
    match forcing {
        f if f < 0.05 => "NATURAL",
        f if f < 0.4 => "SHOWERS",
        f if f < 0.8 => "RAIN",
        _ => "STORM",
    }
}

/// The control, laid into the pause panel above its buttons.
pub fn spawn(panel: &mut ChildSpawnerCommands) {
    panel.spawn((
        WeatherLabel,
        Text::new("WEATHER"),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(MINT),
    ));
    panel
        .spawn((
            WeatherSlider,
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
                WeatherFill,
                Node {
                    width: percent(0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.48, 0.8, 0.77, 0.75)),
            ));
        });
    panel
        .spawn(Node {
            column_gap: px(6),
            margin: UiRect::bottom(px(6)),
            ..default()
        })
        .with_children(|row| {
            small(row, "NATURAL", MenuAction::Weather(0));
            small(row, "RAIN", MenuAction::Weather(60));
            small(row, "STORM", MenuAction::Weather(100));
        });
}

/// A press on the bar, and every frame it is held, sets the forcing to where
/// the cursor is along it.
pub fn drag(
    bars: Query<(&Interaction, &RelativeCursorPosition), With<WeatherSlider>>,
    mut forcing: ResMut<StormForcing>,
) {
    for (interaction, cursor) in &bars {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(at) = cursor.normalized {
            // Normalised is centred on the node: -0.5 at the left edge.
            let value = (at.x + 0.5).clamp(0.0, 1.0);
            if (value - forcing.0).abs() > 1e-3 {
                forcing.0 = value;
            }
        }
    }
}

/// The bar and the label follow the forcing, however it was set.
pub fn show(
    forcing: Res<StormForcing>,
    mut fills: Query<&mut Node, With<WeatherFill>>,
    mut labels: Query<&mut Text, With<WeatherLabel>>,
) {
    if !forcing.is_changed() {
        return;
    }
    for mut node in &mut fills {
        node.width = percent(forcing.0.clamp(0.0, 1.0) * 100.0);
    }
    for mut text in &mut labels {
        text.0 = format!(
            "WEATHER  {}  {:.0}%",
            describe(forcing.0),
            forcing.0.clamp(0.0, 1.0) * 100.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_reads_in_words_across_its_whole_range() {
        assert_eq!(describe(0.0), "NATURAL");
        assert_eq!(describe(0.2), "SHOWERS");
        assert_eq!(describe(0.6), "RAIN");
        assert_eq!(describe(1.0), "STORM");
    }
}
