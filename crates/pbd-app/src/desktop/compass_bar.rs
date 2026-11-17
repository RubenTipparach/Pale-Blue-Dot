//! The compass bar across the top of the screen (`compass-bar` decisions 3,
//! 5 and 6), as Skyrim has it: N, E, S and W, the four between, a tick every
//! fifteen degrees, and a diamond for each town in reach, all sliding as the
//! view turns; the town nearest the middle named under it with how far it
//! is. The player's waypoints ride the same row as small red pins at every
//! distance, and the one faced is named ahead of a town (`waypoints`). It is
//! a navigation control, as the crosshair is an aiming one, which is why it
//! may stand on the quiet screen (`menus-and-a-quiet-screen`).
//!
//! The sums are `pbd_app::compass`'s; this only lays the nodes out. North is
//! the compass's, so facing N the sun rises under E.

use super::menu::Screen;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pbd_app::compass::{self, CompassView, DIRECTIONS, MarkKind, Named, Place};
use pbd_app::planet::PLANET_RADIUS;
use pbd_app::saves::WorldSave;
use pbd_app::sites::WorldSites;
use pbd_app::ui_kit::{self, KitPiece, KitText, Set, UiKit};
use pbd_core::map::{MARK_ID, MARK_KIND, MARKS_MAX, Mark, read_marks};

/// The strip, native pixels: the light dressed-stone `compass` piece
/// (`stone-and-wood-ui`), as the mockup's HUD draws it. Its letters and ticks
/// are cut into its upper part and the towns' brass studs sit along its foot,
/// so a town never hides a letter.
const BAR_W: f32 = 280.0;
const BAR_H: f32 = 18.0;
/// The window the marks slide in: the strip less its end caps.
const INSET: f32 = 4.0;
const INNER_W: f32 = BAR_W - 2.0 * INSET;
/// The most towns drawn on the bar at once: the nearest.
const PLACES: usize = 12;
/// A town's brass stud.
const STUD: f32 = 5.0;
/// A waypoint's pin, native pixels: the map pin's teardrop, its point on the
/// bar's foot (`waypoints` decision 6). `k` the ink rim, `r` the red, `w` its
/// pale glint, `.` clear.
const PIN: [&str; 6] = [".kkk.", "krwrk", "krrrk", ".krk.", ".krk.", "..k.."];

/// The bar's root, shown only while walking or flying low.
#[derive(Component)]
pub struct CompassRoot;

/// The strip.
#[derive(Component)]
pub struct CompassBar;

/// One of the 24 marks: a tick, or a letter, by its index in `DIRECTIONS`.
#[derive(Component)]
pub struct CompassMark(usize);

/// One of the town studs, by slot.
#[derive(Component)]
pub struct CompassPlace(usize);

/// One of the waypoint pins, by slot.
#[derive(Component)]
pub struct CompassWaypoint(usize);

/// The line naming the town nearest the middle.
#[derive(Component)]
pub struct CompassName;

/// The waypoint pin's picture, from the palette, drawn nearest.
fn pin_image() -> Image {
    let (w, h) = (PIN[0].len(), PIN.len());
    let rgba = |c: Color| c.to_srgba().to_u8_array();
    let mut data = Vec::with_capacity(w * h * 4);
    for row in PIN {
        for c in row.chars() {
            data.extend(match c {
                'k' => rgba(ui_kit::INK),
                'r' => rgba(ui_kit::WAYPOINT),
                'w' => rgba(ui_kit::LABEL),
                _ => [0; 4],
            });
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

pub fn spawn(mut commands: Commands, kit: Res<UiKit>, mut images: ResMut<Assets<Image>>) {
    let kit = &*kit;
    let pin = images.add(pin_image());
    commands
        .spawn((
            CompassRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(4),
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
                kit.nine(Set::Stone, "compass"),
                KitPiece("compass".into()),
                Node {
                    width: px(BAR_W),
                    height: px(BAR_H),
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .with_children(|bar| {
                bar.spawn(Node {
                    position_type: PositionType::Absolute,
                    left: px(INSET),
                    top: px(0),
                    width: px(INNER_W),
                    height: px(BAR_H),
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|inner| {
                    for (i, label) in DIRECTIONS.iter().enumerate() {
                        if label.is_empty() {
                            inner.spawn((
                                CompassMark(i),
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: px(1),
                                    height: px(4),
                                    top: px(4),
                                    display: Display::None,
                                    ..default()
                                },
                                BackgroundColor(ui_kit::CUT),
                            ));
                        } else {
                            inner.spawn((
                                CompassMark(i),
                                KitText::new(
                                    *label,
                                    if i == 0 { ui_kit::NORTH } else { ui_kit::CUT },
                                ),
                                Node {
                                    position_type: PositionType::Absolute,
                                    top: px(3),
                                    display: Display::None,
                                    ..default()
                                },
                            ));
                        }
                    }
                    for k in 0..PLACES {
                        inner.spawn((
                            CompassPlace(k),
                            kit.whole(Set::Stone, "compass-town"),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(STUD),
                                height: px(STUD),
                                top: px(11),
                                display: Display::None,
                                ..default()
                            },
                        ));
                    }
                    // Spawned after the studs, so a pin stands over a town.
                    for k in 0..MARKS_MAX {
                        inner.spawn((
                            CompassWaypoint(k),
                            ImageNode::new(pin.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                width: px(PIN[0].len() as f32),
                                height: px(PIN.len() as f32),
                                top: px(10),
                                display: Display::None,
                                ..default()
                            },
                        ));
                    }
                });
                // The oak pointer over the middle (design 3, accent 3).
                bar.spawn((
                    kit.whole(Set::Stone, "compass-pointer"),
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        margin: UiRect::left(px(-3)),
                        top: px(-1),
                        width: px(7),
                        height: px(6),
                        ..default()
                    },
                ));
            });
            root.spawn((
                CompassName,
                KitText::new("", ui_kit::TEXT).shadow(ui_kit::INK),
                Node {
                    margin: UiRect::top(px(2)),
                    ..default()
                },
            ));
        });
}

/// Lay the marks out for this frame's heading, fade the whole bar with
/// altitude, and name the town nearest the middle. Hidden on any menu
/// screen, and when the view is high enough that a ground compass means
/// nothing.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn update(
    view: Res<CompassView>,
    screen: Res<Screen>,
    sites: Option<Res<WorldSites>>,
    save: Option<Res<WorldSave>>,
    kit: Res<UiKit>,
    mut root: Query<&mut Visibility, With<CompassRoot>>,
    mut bar: Query<
        &mut ImageNode,
        (
            With<CompassBar>,
            Without<CompassPlace>,
            Without<CompassWaypoint>,
        ),
    >,
    mut marks: Query<
        (
            &CompassMark,
            &mut Node,
            Option<&mut KitText>,
            Option<&mut BackgroundColor>,
        ),
        (Without<CompassPlace>, Without<CompassWaypoint>),
    >,
    mut studs: Query<
        (&CompassPlace, &mut Node, &mut ImageNode),
        (
            Without<CompassMark>,
            Without<CompassBar>,
            Without<CompassWaypoint>,
        ),
    >,
    mut pins: Query<
        (&CompassWaypoint, &mut Node, &mut ImageNode),
        (Without<CompassMark>, Without<CompassBar>),
    >,
    mut name: Query<&mut KitText, (With<CompassName>, Without<CompassMark>)>,
    mut read: Local<(String, Vec<Mark>)>,
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
    for mut image in &mut bar {
        let want = Color::WHITE.with_alpha(alpha);
        if image.color != want {
            image.color = want;
        }
    }
    // A mark's box `w` wide centred at `offset` (-1..1 across the window),
    // on a whole native pixel.
    let x = |offset: f32, w: f32| px((INNER_W / 2.0 * (1.0 + offset) - w / 2.0).round());

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
                    background.0 = ui_kit::CUT.with_alpha(fade);
                }
            }
            kind => {
                let label = DIRECTIONS[mark.0];
                node.left = x(m.offset, kit.font.width(label) as f32);
                let ink = if kind == MarkKind::Cardinal && mark.0 == 0 {
                    ui_kit::NORTH
                } else {
                    ui_kit::CUT
                };
                if let Some(mut text) = text {
                    let want = ink.with_alpha(fade);
                    if text.colour != want {
                        text.colour = want;
                    }
                }
            }
        }
    }
    let shown: Vec<_> = laid
        .iter()
        .filter(|m| matches!(m.kind, MarkKind::Place(_)))
        .collect();
    for (slot, mut node, mut image) in &mut studs {
        match shown.get(slot.0) {
            Some(m) => {
                node.display = Display::Flex;
                node.left = x(m.offset, STUD);
                image.color = Color::WHITE.with_alpha(m.fade * alpha);
            }
            None => node.display = Display::None,
        }
    }
    // The player's waypoints, read again only when their record changes.
    let body = save
        .as_ref()
        .and_then(|s| s.records.get(MARK_KIND, MARK_ID))
        .map_or("", |r| r.body.as_str());
    if read.0 != body {
        *read = (body.to_string(), read_marks(body).unwrap_or_default());
    }
    let waypoints: Vec<Place> = read
        .1
        .iter()
        .map(|m| Place::seen(view.up, m.direction(), PLANET_RADIUS))
        .collect();
    let on_bar = compass::waypoint_marks(heading, &waypoints);
    for (slot, mut node, mut image) in &mut pins {
        match on_bar.get(slot.0) {
            Some(m) => {
                node.display = Display::Flex;
                node.left = x(m.offset, PIN[0].len() as f32);
                image.color = Color::WHITE.with_alpha(m.fade * alpha);
            }
            None => node.display = Display::None,
        }
    }
    if let Ok(mut text) = name.single_mut() {
        let line = match compass::named_ahead(heading, &waypoints, &places) {
            Some(Named::Waypoint(i)) => {
                compass::name_line(&read.1[i].label(), waypoints[i].distance_m)
            }
            Some(Named::Place(i)) => compass::name_line(towns[i].0, towns[i].1.distance_m),
            None => String::new(),
        };
        if text.text != line {
            text.text = line;
        }
    }
}
