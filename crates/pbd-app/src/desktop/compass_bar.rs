//! The compass bar across the top of the screen (`compass-bar` decisions 3,
//! 5 and 6), as Skyrim has it: N, E, S and W, the four between, a tick every
//! fifteen degrees, and a diamond for each town in reach, all sliding as the
//! view turns; the town nearest the middle named under it with how far it
//! is. It is a navigation control, as the crosshair is an aiming one, which
//! is why it may stand on the quiet screen (`menus-and-a-quiet-screen`).
//!
//! The sums are `pbd_app::compass`'s; this only lays the nodes out. North is
//! the compass's, so facing N the sun rises under E.

use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::compass::{self, CompassView, DIRECTIONS, MarkKind, Place};
use pbd_app::planet::PLANET_RADIUS;
use pbd_app::sites::WorldSites;

/// The bar's height and the width of a letter's box, px. The letters and
/// ticks take its upper part and the towns' diamonds a row along its foot,
/// so a town never hides a letter.
const BAR_H: f32 = 32.0;
const LETTER_W: f32 = 30.0;
/// The most towns drawn on the bar at once: the nearest.
const PLACES: usize = 12;
/// The diamond's side, px.
const DIAMOND: f32 = 7.0;

const INK: (f32, f32, f32) = (0.9, 0.95, 0.93);
/// North's own colour: the bar's one warm accent.
const NORTH: (f32, f32, f32) = (1.0, 0.72, 0.42);
const PLACE: (f32, f32, f32) = (0.98, 0.86, 0.55);

#[derive(Component)]
pub struct CompassRoot;

#[derive(Component)]
pub struct CompassBar;

/// One of the 24 directions, by its index in [`DIRECTIONS`].
#[derive(Component)]
pub struct CompassMark(usize);

/// One diamond of the pool the towns are drawn with.
#[derive(Component)]
pub struct CompassPlace(usize);

/// The line under the bar that names the town in the middle.
#[derive(Component)]
pub struct CompassName;

fn colour((r, g, b): (f32, f32, f32), alpha: f32) -> Color {
    Color::srgba(r, g, b, alpha.clamp(0.0, 1.0))
}

/// The bar's width for a window: 36% of it, held between 320 and 640 px.
pub fn bar_width(window_width: f32) -> f32 {
    (window_width * 0.36).clamp(320.0, 640.0)
}

pub fn spawn(mut commands: Commands) {
    commands
        .spawn((
            CompassRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(8),
                left: px(0),
                width: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                CompassBar,
                Node {
                    width: px(bar_width(1280.0)),
                    height: px(BAR_H),
                    border: UiRect::vertical(px(1)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.02, 0.05, 0.06, 0.45)),
                BorderColor::all(Color::srgba(0.85, 0.92, 0.9, 0.35)),
            ))
            .with_children(|bar| {
                for (i, label) in DIRECTIONS.iter().enumerate() {
                    if label.is_empty() {
                        bar.spawn((
                            CompassMark(i),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(1),
                                height: px(6),
                                top: px(8.0),
                                display: Display::None,
                                ..default()
                            },
                            BackgroundColor(colour(INK, 0.5)),
                        ));
                    } else {
                        let cardinal = i % 6 == 0;
                        bar.spawn((
                            CompassMark(i),
                            Text::new(*label),
                            TextFont {
                                font_size: if cardinal { 15.0 } else { 11.0 },
                                ..default()
                            },
                            TextLayout::new_with_justify(Justify::Center),
                            TextColor(colour(if i == 0 { NORTH } else { INK }, 1.0)),
                            TextShadow::default(),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(LETTER_W),
                                top: px(if cardinal { 1.0 } else { 3.5 }),
                                display: Display::None,
                                ..default()
                            },
                        ));
                    }
                }
                for k in 0..PLACES {
                    bar.spawn((
                        CompassPlace(k),
                        Node {
                            position_type: PositionType::Absolute,
                            width: px(DIAMOND),
                            height: px(DIAMOND),
                            top: px(BAR_H - 2.0 - DIAMOND - 4.0),
                            display: Display::None,
                            ..default()
                        },
                        UiTransform::from_rotation(Rot2::degrees(45.0)),
                        BackgroundColor(colour(PLACE, 1.0)),
                        BorderColor::all(Color::srgba(0.1, 0.07, 0.02, 0.7)),
                    ));
                }
                // The notch at the middle, where the view is facing.
                bar.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        margin: UiRect::left(px(-1)),
                        bottom: px(0),
                        width: px(2),
                        height: px(5),
                        ..default()
                    },
                    BackgroundColor(colour(INK, 0.85)),
                ));
            });
            root.spawn((
                CompassName,
                Text::new(""),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(colour(PLACE, 0.95)),
                TextShadow::default(),
                Node {
                    margin: UiRect::top(px(3)),
                    ..default()
                },
            ));
        });
}

/// Lay the bar out for the view: hidden off the world and high up, the
/// letters and ticks at their offsets, the nearest towns as diamonds, the one
/// in the middle named.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update(
    view: Res<CompassView>,
    screen: Res<Screen>,
    sites: Option<Res<WorldSites>>,
    windows: Query<&Window>,
    mut root: Query<&mut Visibility, With<CompassRoot>>,
    mut bar: Query<
        &mut Node,
        (
            With<CompassBar>,
            Without<CompassMark>,
            Without<CompassPlace>,
        ),
    >,
    mut marks: Query<
        (
            &CompassMark,
            &mut Node,
            Option<&mut TextColor>,
            Option<&mut BackgroundColor>,
        ),
        Without<CompassPlace>,
    >,
    mut diamonds: Query<
        (&CompassPlace, &mut Node, &mut BackgroundColor),
        (Without<CompassMark>, Without<CompassBar>),
    >,
    mut name: Query<&mut Text, (With<CompassName>, Without<CompassMark>)>,
) {
    let Ok(mut visibility) = root.single_mut() else {
        return;
    };
    let alpha = compass::altitude_fade(view.height_m);
    let (Some(heading), true, true) = (view.heading, *screen == Screen::Playing, alpha > 0.01)
    else {
        *visibility = Visibility::Hidden;
        return;
    };
    *visibility = Visibility::Inherited;
    let width = windows
        .iter()
        .next()
        .map_or(bar_width(1280.0), |w| bar_width(w.width()));
    if let Ok(mut node) = bar.single_mut()
        && node.width != px(width)
    {
        node.width = px(width);
    }
    let x = |offset: f32, box_w: f32| px(width / 2.0 * (1.0 + offset) - box_w / 2.0);

    // The towns in reach, nearest first, as many as the pool holds.
    let mut towns: Vec<(&str, Place)> = sites
        .as_ref()
        .and_then(|s| s.ready())
        .unwrap_or(&[])
        .iter()
        .map(|site| {
            (
                site.name.as_str(),
                Place::seen(view.up, site.direction, PLANET_RADIUS),
            )
        })
        .filter(|(_, p)| p.distance_m <= compass::PLACE_REACH_M)
        .collect();
    towns.sort_by(|a, b| a.1.distance_m.total_cmp(&b.1.distance_m));
    towns.truncate(PLACES);
    let places: Vec<Place> = towns.iter().map(|(_, p)| *p).collect();
    let laid = compass::marks(heading, &places);

    for (mark, mut node, text, background) in &mut marks {
        let Some(m) = laid
            .iter()
            .find(|m| !matches!(m.kind, MarkKind::Place(_)) && m.index == mark.0)
        else {
            node.display = Display::None;
            continue;
        };
        node.display = Display::Flex;
        let fade = m.fade * alpha;
        match m.kind {
            MarkKind::Tick => {
                node.left = x(m.offset, 1.0);
                if let Some(mut background) = background {
                    background.0 = colour(INK, 0.5 * fade);
                }
            }
            kind => {
                node.left = x(m.offset, LETTER_W);
                let (ink, strength) = match kind {
                    MarkKind::Cardinal if mark.0 == 0 => (NORTH, 1.0),
                    MarkKind::Cardinal => (INK, 1.0),
                    _ => (INK, 0.7),
                };
                if let Some(mut text) = text {
                    text.0 = colour(ink, strength * fade);
                }
            }
        }
    }
    let shown: Vec<_> = laid
        .iter()
        .filter(|m| matches!(m.kind, MarkKind::Place(_)))
        .collect();
    for (slot, mut node, mut background) in &mut diamonds {
        match shown.get(slot.0) {
            Some(m) => {
                node.display = Display::Flex;
                node.left = x(m.offset, DIAMOND);
                background.0 = colour(PLACE, m.fade * alpha);
            }
            None => node.display = Display::None,
        }
    }
    if let Ok(mut text) = name.single_mut() {
        let line = compass::named(heading, &places)
            .map(|i| {
                format!(
                    "{}  {}",
                    towns[i].0,
                    compass::distance_text(towns[i].1.distance_m)
                )
            })
            .unwrap_or_default();
        if text.0 != line {
            text.0 = line;
        }
    }
}
