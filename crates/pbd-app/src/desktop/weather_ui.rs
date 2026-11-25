//! The weather control on the pause panel: a bar from clear to storm that IS
//! the storm forcing, and three presets. It moves the field exactly as the P
//! key and `--rain` do (`StormForcing`), so there is one path to the weather
//! and the menu is only another hand on it.

use super::menu::{MenuAction, small};
use bevy::{prelude::*, ui::RelativeCursorPosition};
use pbd_app::ui_kit::{self, KitText, UiKit};
use pbd_app::ui_kit::{Localized, tr};
use pbd_app::weather::StormForcing;
use pbd_core::locale::tr_with;

/// The bar a press or a drag sets the forcing from.
#[derive(Component)]
pub struct WeatherSlider;

/// The lit part of the bar, as wide as the forcing.
#[derive(Component)]
pub struct WeatherFill;

/// The line that says what the bar is set to.
#[derive(Component)]
pub struct WeatherLabel;

/// The oak grip on the groove.
#[derive(Component)]
pub struct WeatherKnob;

/// The bar's width, px.
const BAR_WIDTH: f32 = 172.0;

/// What a forcing reads as, in words a player would use.
pub fn describe(forcing: f32) -> Localized {
    tr(match forcing {
        f if f < 0.05 => "weather.natural",
        f if f < 0.4 => "weather.showers",
        f if f < 0.8 => "weather.rain",
        _ => "weather.storm",
    })
}

/// The control, laid into the pause panel above its buttons.
pub fn spawn(panel: &mut ChildSpawnerCommands, kit: &UiKit) {
    panel.spawn((
        WeatherLabel,
        KitText::new(tr("weather.title"), ui_kit::TITLE).shadow(ui_kit::INK),
    ));
    super::menu::slider(
        panel,
        kit,
        BAR_WIDTH,
        (WeatherSlider, Button, RelativeCursorPosition::default()),
        WeatherFill,
        WeatherKnob,
    );
    panel
        .spawn(Node {
            column_gap: px(2),
            margin: UiRect::bottom(px(3)),
            ..default()
        })
        .with_children(|row| {
            small(row, kit, tr("weather.natural"), MenuAction::Weather(0));
            small(row, kit, tr("weather.rain"), MenuAction::Weather(60));
            small(row, kit, tr("weather.storm"), MenuAction::Weather(100));
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
    mut fills: Query<&mut Node, (With<WeatherFill>, Without<WeatherKnob>)>,
    mut knobs: Query<&mut Node, (With<WeatherKnob>, Without<WeatherFill>)>,
    mut labels: Query<&mut KitText, With<WeatherLabel>>,
) {
    if !forcing.is_changed() {
        return;
    }
    let at = percent(forcing.0.clamp(0.0, 1.0) * 100.0);
    for mut node in &mut fills {
        node.width = at;
    }
    for mut node in &mut knobs {
        node.left = at;
    }
    for mut text in &mut labels {
        text.text = tr_with(
            "weather.line",
            &[
                ("forcing", &describe(forcing.0)),
                (
                    "percent",
                    &format!("{:.0}", forcing.0.clamp(0.0, 1.0) * 100.0),
                ),
            ],
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
