//! The city sites on the world map (`city-sites` group 4, decision 7): the
//! world's stored list, drawn as the approved mockup draws it.
//!
//! A square for a walled town, a desert town, a cave town and the capital,
//! a circle for the rest, and an amber ring round the capital. Names at
//! 12.5 m a pixel or closer, and the capital's and the home town's always.
//! The footprint once it is more than six pixels across its radius, as an
//! ellipse as wide as the map stretches its latitude. With the map's night
//! on, each town lights a pool round it as strong as the dark is deep there.
//!
//! Each part of each site is its own UI node in [`SiteLayer`], drawn in
//! passes (the lamplight under the footprints, under the markers, under the
//! names), once for each of the map's three copies across the antimeridian.
//! A rounded border makes the circles and the ellipse, so a line is the same
//! width at every size.

use super::map_screen::{MapChoice, MapView};
use super::menu::Screen;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use pbd_app::sites::{SitesRules, WorldSites};
use pbd_app::sky::Sun;
use pbd_core::geo;
use pbd_core::sites::{Site, SiteKind};

/// Where the sites are drawn: over the planet and its layers, under the
/// player's and the craft's markers.
#[derive(Component)]
pub struct SiteLayer;

/// The legend's line saying how many sites there are, or that the world is
/// being surveyed.
#[derive(Component)]
pub struct SitesNote;

/// The mockup's label zoom: its zoom 0.9 over its 11.3 m base.
pub const LABELS_AT_M_PER_PX: f32 = 12.5;
/// A footprint is drawn once its radius is more than this, pixels.
pub const FOOTPRINT_MIN_PX: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Glow,
    Footprint,
    Icon,
    Crown,
    Label,
}

/// One part of one site, in one copy of the map.
#[derive(Component, Clone, Copy, Debug)]
pub struct SitePart {
    pub site: usize,
    pub copy: i32,
    pub part: Part,
}

/// Which list the layer's nodes were made for.
#[derive(Resource, Default)]
pub struct DrawnSites(Option<Vec<u32>>);

/// A town the map draws as a square, larger, with a larger pool of light.
pub fn is_big(site: &Site) -> bool {
    matches!(
        site.kind,
        SiteKind::Walled | SiteKind::Desert | SiteKind::Cave
    ) || site.capital
}

/// The marker's half-width, pixels.
fn half(site: &Site) -> f32 {
    if is_big(site) { 5.0 } else { 3.5 }
}

fn font_px(site: &Site) -> f32 {
    if is_big(site) { 14.0 } else { 12.0 }
}

/// How far a town's lamps light the ground round it, metres.
fn reach_m(site: &Site) -> f32 {
    if is_big(site) { 260.0 } else { 140.0 }
}

fn glow(night: f32) -> BackgroundGradient {
    BackgroundGradient::from(RadialGradient::new(
        UiPosition::CENTER,
        RadialGradientShape::ClosestSide,
        vec![
            ColorStop::percent(Color::srgba(1.0, 0.77, 0.43, 0.75 * night), 0.0),
            ColorStop::percent(Color::srgba(0.94, 0.63, 0.27, 0.32 * night), 35.0),
            ColorStop::percent(Color::srgba(0.94, 0.63, 0.27, 0.0), 100.0),
        ],
    ))
}

fn absolute() -> Node {
    Node {
        position_type: PositionType::Absolute,
        display: Display::None,
        ..default()
    }
}

/// Make the layer's nodes for a list: every part of every site in each of
/// the three copies, pass by pass, so each pass draws over the one before.
fn spawn_parts(commands: &mut Commands, parent: Entity, sites: &[Site]) {
    for part in [
        Part::Glow,
        Part::Footprint,
        Part::Icon,
        Part::Crown,
        Part::Label,
    ] {
        for (index, site) in sites.iter().enumerate() {
            if part == Part::Crown && !site.capital {
                continue;
            }
            for copy in -1..=1 {
                let tag = SitePart {
                    site: index,
                    copy,
                    part,
                };
                let child = match part {
                    Part::Glow => commands.spawn((tag, absolute(), glow(0.0))).id(),
                    Part::Footprint => commands
                        .spawn((
                            tag,
                            Node {
                                border: UiRect::all(px(1.2)),
                                border_radius: BorderRadius::MAX,
                                ..absolute()
                            },
                            BorderColor::all(Color::srgba(0.96, 0.94, 0.89, 0.7)),
                        ))
                        .id(),
                    Part::Icon => commands
                        .spawn((
                            tag,
                            Node {
                                width: px(2.0 * half(site)),
                                height: px(2.0 * half(site)),
                                border: UiRect::all(px(1.5)),
                                border_radius: if is_big(site) {
                                    BorderRadius::ZERO
                                } else {
                                    BorderRadius::MAX
                                },
                                ..absolute()
                            },
                            BackgroundColor(Color::srgb_u8(0x1b, 0x15, 0x10)),
                            BorderColor::all(Color::srgb_u8(0xf0, 0xe6, 0xd0)),
                        ))
                        .id(),
                    Part::Crown => commands
                        .spawn((
                            tag,
                            Node {
                                width: px(2.0 * (half(site) + 4.0)),
                                height: px(2.0 * (half(site) + 4.0)),
                                border: UiRect::all(px(1.5)),
                                border_radius: BorderRadius::MAX,
                                ..absolute()
                            },
                            BorderColor::all(Color::srgb_u8(0xf0, 0xb4, 0x4c)),
                        ))
                        .id(),
                    Part::Label => commands
                        .spawn((
                            tag,
                            absolute(),
                            Text::new(site.name.clone()),
                            TextFont {
                                font_size: font_px(site),
                                ..default()
                            },
                            TextColor(Color::srgb_u8(0xf4, 0xef, 0xe4)),
                            TextShadow {
                                offset: Vec2::new(1.0, 1.0),
                                color: Color::srgba(0.03, 0.05, 0.07, 0.9),
                            },
                            TextLayout::new_with_no_wrap(),
                        ))
                        .id(),
                };
                commands.entity(parent).add_child(child);
            }
        }
    }
}

type PartQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static SitePart,
        &'static mut Node,
        Option<&'static mut BackgroundGradient>,
        Option<&'static mut BackgroundColor>,
        Option<&'static mut BorderColor>,
        Option<&'static mut TextColor>,
    ),
>;

/// Keep the layer to the world's list and the view: make the nodes when the
/// list changes, and place, size, show and light each part every frame the
/// map is open.
#[allow(clippy::too_many_arguments)]
pub fn draw_sites(
    mut commands: Commands,
    screen: Res<Screen>,
    view: Res<MapView>,
    choice: Res<MapChoice>,
    sun: Res<Sun>,
    world_sites: Option<Res<WorldSites>>,
    rules: Option<Res<SitesRules>>,
    mut drawn: ResMut<DrawnSites>,
    windows: Query<&Window, With<PrimaryWindow>>,
    layer: Query<(Entity, Option<&Children>), With<SiteLayer>>,
    mut parts: PartQuery,
    mut notes: Query<&mut Text, With<SitesNote>>,
) {
    if *screen != Screen::Map {
        return;
    }
    let sites = world_sites.as_ref().and_then(|w| w.ready());
    for mut text in &mut notes {
        text.0 = match (sites, world_sites.as_ref().is_some_and(|w| w.surveying())) {
            (Some(list), _) => format!("{} settlements on the map.", list.len()),
            (None, true) => "Surveying the settlements...".into(),
            (None, false) => String::new(),
        };
    }
    let Ok((parent, children)) = layer.single() else {
        return;
    };
    let ids: Option<Vec<u32>> = sites.map(|list| list.iter().map(|s| s.id).collect());
    if drawn.0 != ids {
        for child in children.into_iter().flatten() {
            commands.entity(*child).despawn();
        }
        if let Some(list) = sites {
            spawn_parts(&mut commands, parent, list);
        }
        drawn.0 = ids;
        return;
    }
    let (Some(sites), Ok(window)) = (sites, windows.single()) else {
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    let view = view.clamped(size);
    let metres_per_px = view.metres_per_pixel();
    let labels = metres_per_px <= LABELS_AT_M_PER_PX;
    // A metre on the ground is this many pixels on the map at the equator,
    // and north and south at every latitude.
    let px_per_m = 1.0 / metres_per_px;
    for (tag, mut node, gradient, fill, border, text) in &mut parts {
        let Some(site) = sites.get(tag.site) else {
            node.display = Display::None;
            continue;
        };
        let centre = view.to_screen(geo::project(site.direction), size)
            + Vec2::X * tag.copy as f32 * view.px_per_turn;
        let night = if choice.night {
            1.0 - sun.clock.daylight(site.direction)
        } else {
            0.0
        };
        let lat = site.direction.normalize().y.clamp(-1.0, 1.0).asin();
        let stretch = 1.0 / lat.cos().max(0.05);
        // The part's half-extent, pixels, and whether it is drawn at all.
        let (half_w, half_h, show) = match tag.part {
            Part::Glow => {
                let r = (reach_m(site) * px_per_m).max(10.0);
                (r, r, night > 0.02)
            }
            Part::Footprint => {
                let footprint = rules.as_ref().map_or(0.0, |r| r.0.rule(site.kind).radius_m);
                let ry = footprint * px_per_m;
                (ry * stretch, ry, ry > FOOTPRINT_MIN_PX)
            }
            Part::Icon => (half(site), half(site), true),
            Part::Crown => (half(site) + 4.0, half(site) + 4.0, true),
            Part::Label => (0.0, 0.0, labels || site.capital || site.home),
        };
        let on_screen = centre.x + half_w > -80.0
            && centre.x - half_w < size.x + 80.0
            && centre.y + half_h > -40.0
            && centre.y - half_h < size.y + 40.0;
        if !(show && on_screen) {
            node.display = Display::None;
            continue;
        }
        node.display = Display::Flex;
        if tag.part == Part::Label {
            node.left = px(centre.x + half(site) + 5.0);
            node.top = px(centre.y - font_px(site) * 0.7);
        } else {
            node.left = px(centre.x - half_w);
            node.top = px(centre.y - half_h);
            if matches!(tag.part, Part::Glow | Part::Footprint) {
                node.width = px(2.0 * half_w);
                node.height = px(2.0 * half_h);
            }
        }
        let lit = night > 0.5;
        match tag.part {
            Part::Glow => {
                if let Some(mut g) = gradient {
                    *g = glow(night);
                }
            }
            Part::Icon => {
                if let Some(mut fill) = fill {
                    fill.0 = if lit {
                        Color::srgb_u8(0xff, 0xe2, 0xa8)
                    } else {
                        Color::srgb_u8(0x1b, 0x15, 0x10)
                    };
                }
                if let Some(mut border) = border {
                    *border = BorderColor::all(if lit {
                        Color::srgba(1.0, 0.78, 0.47, 0.9)
                    } else {
                        Color::srgb_u8(0xf0, 0xe6, 0xd0)
                    });
                }
            }
            Part::Label => {
                if let Some(mut colour) = text {
                    colour.0 = if lit {
                        Color::srgb_u8(0xff, 0xe6, 0xb8)
                    } else {
                        Color::srgb_u8(0xf4, 0xef, 0xe4)
                    };
                }
            }
            Part::Footprint | Part::Crown => {}
        }
    }
}

#[cfg(test)]
mod tests;
