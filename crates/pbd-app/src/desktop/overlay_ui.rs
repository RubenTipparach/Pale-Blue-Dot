//! The overlay's legend and its row on the pause panel. The legend names the
//! overlay, draws its colour bar from the same ramp table the shader draws the
//! map with, gives the range in its unit, the day and season, and the key. The
//! row sets `OverlayMode` as the map's "Show on globe" does: one mode, two
//! hands on it. M opens the map (`world-map` task 5.2) rather than cycling.

use super::menu::{MenuAction, small};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use pbd_app::overlay::{OverlayMode, ramp_colour};
use pbd_app::sky::Sun;
use pbd_app::ui_kit::tr;
use pbd_app::ui_kit::{self, KitPiece, KitText, Set, UiKit};
use pbd_core::overlay::Overlay;

/// The legend panel; shown only while an overlay is.
#[derive(Component)]
pub struct Legend;

/// The legend's texts, each told apart by what it says.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum LegendText {
    Title,
    Low,
    High,
    Date,
}

/// The colour bar's image, rewritten when the overlay changes.
#[derive(Resource)]
pub struct LegendBar(Handle<Image>);

/// Texels along the colour bar.
const BAR_TEXELS: u32 = 256;
/// The bar's size on screen, px.
const BAR_WIDTH: f32 = 120.0;
const BAR_HEIGHT: f32 = 6.0;

/// The colour bar of an overlay: its ramp, low end on the left, as sRGB bytes.
pub fn bar_texels(overlay: Overlay) -> Vec<u8> {
    (0..BAR_TEXELS)
        .flat_map(|i| {
            let t = (i as f32 + 0.5) / BAR_TEXELS as f32;
            let [r, g, b] = ramp_colour(overlay.ramp(), t);
            [r, g, b, 1.0].map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
        })
        .collect()
}

fn text(parent: &mut ChildSpawnerCommands, which: LegendText, size: f32, colour: Color) {
    parent.spawn((
        which,
        Text::new(""),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(colour),
    ));
}

pub fn spawn(mut commands: Commands, mut images: ResMut<Assets<Image>>, kit: Res<UiKit>) {
    let bar = images.add(Image::new(
        Extent3d {
            width: BAR_TEXELS,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bar_texels(Overlay::Wind),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    commands.insert_resource(LegendBar(bar.clone()));
    commands
        .spawn((
            Legend,
            // The dressed-stone moulding of an in-play sub-panel.
            kit.nine(Set::Stone, "frame"),
            KitPiece("frame".into()),
            Node {
                position_type: PositionType::Absolute,
                top: px(6),
                right: px(6),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                padding: UiRect::all(px(14)),
                display: Display::None,
                ..default()
            },
        ))
        .with_children(|panel| {
            text(panel, LegendText::Title, 11.0, ui_kit::TITLE);
            panel.spawn((
                ImageNode::new(bar),
                Node {
                    width: px(BAR_WIDTH),
                    height: px(BAR_HEIGHT),
                    ..default()
                },
            ));
            panel
                .spawn(Node {
                    width: px(BAR_WIDTH),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|ends| {
                    text(ends, LegendText::Low, 9.0, ui_kit::TEXT);
                    text(ends, LegendText::High, 9.0, ui_kit::TEXT);
                });
            text(panel, LegendText::Date, 9.0, ui_kit::TEXT);
            panel.spawn(KitText::new(tr("overlay.map_chooses"), ui_kit::TEXT_DIM));
        });
}

/// What one end of the range reads: the value in its unit, and for the rain,
/// which end is snow.
fn end_label(overlay: Overlay, value: f32) -> String {
    let unit = overlay.unit();
    match overlay {
        Overlay::Rain if value < 0.0 => format!("{} {unit} SNOW", -value),
        Overlay::Rain => format!("{value} {unit} RAIN"),
        _ => format!("{value} {unit}"),
    }
}

/// The legend follows the mode and the clock, however the mode was set.
pub fn show(
    mode: Res<OverlayMode>,
    sun: Res<Sun>,
    bar: Res<LegendBar>,
    mut images: ResMut<Assets<Image>>,
    mut panels: Query<&mut Node, With<Legend>>,
    mut texts: Query<(&LegendText, &mut Text)>,
    mut day: Local<Option<u64>>,
) {
    let date_due = *day != Some(sun.clock.day());
    if !mode.is_changed() && !date_due {
        return;
    }
    *day = Some(sun.clock.day());
    for mut node in &mut panels {
        let display = if mode.0.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let Some(overlay) = mode.0 else {
        return;
    };
    if mode.is_changed()
        && let Some(image) = images.get_mut(&bar.0)
    {
        image.data = Some(bar_texels(overlay));
    }
    let (low, high) = overlay.range();
    for (which, mut label) in &mut texts {
        label.0 = match which {
            LegendText::Title => overlay.name().to_string(),
            LegendText::Low => end_label(overlay, low),
            LegendText::High => end_label(overlay, high),
            LegendText::Date => format!(
                "DAY {} - {}",
                sun.clock.day() % pbd_core::daylight::YEAR_DAYS as u64,
                sun.clock.season().to_uppercase()
            ),
        };
    }
}

/// The pause panel's overlay row: off, then one button per overlay.
pub fn spawn_row(panel: &mut ChildSpawnerCommands, kit: &UiKit) {
    panel.spawn(KitText::new(tr("overlay.title"), ui_kit::TITLE).shadow(ui_kit::INK));
    panel
        .spawn(Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(2),
            row_gap: px(2),
            margin: UiRect::bottom(px(3)),
            ..default()
        })
        .with_children(|row| {
            small(row, kit, "OFF", MenuAction::Overlay(0));
            for (i, overlay) in Overlay::ALL.iter().enumerate() {
                small(row, kit, overlay.name(), MenuAction::Overlay(i as u8 + 1));
            }
        });
}

pub fn mode_for(row: u8) -> Option<Overlay> {
    row.checked_sub(1)
        .and_then(|i| Overlay::ALL.get(i as usize).copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_names_its_own_overlay_and_zero_is_off() {
        assert_eq!(mode_for(0), None);
        for (i, overlay) in Overlay::ALL.iter().enumerate() {
            assert_eq!(mode_for(i as u8 + 1), Some(*overlay));
        }
        assert_eq!(mode_for(Overlay::ALL.len() as u8 + 1), None);
    }

    #[test]
    fn the_bar_runs_from_the_ramps_low_end_to_its_high_end() {
        let bar = bar_texels(Overlay::Temperature);
        assert_eq!(bar.len(), BAR_TEXELS as usize * 4);
        // Cold is blue, hot is red.
        assert!(bar[2] > bar[0], "the low end is blue");
        let last = bar.len() - 4;
        assert!(bar[last] > bar[last + 2], "the high end is red");
    }

    #[test]
    fn the_rain_says_which_end_is_snow() {
        assert_eq!(end_label(Overlay::Rain, -100.0), "100 % SNOW");
        assert_eq!(end_label(Overlay::Rain, 100.0), "100 % RAIN");
        assert_eq!(end_label(Overlay::Wind, 20.0), "20 m/s");
    }
}
