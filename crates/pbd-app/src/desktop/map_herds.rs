//! Animals on the map (`world-map` group 7, owner 2026-11-21): each wild
//! herd of a tameable species within [`REACH_M`] of the player as one round
//! marker at its centre, and each of the player's tamed animals with a light
//! ring round it.
//!
//! The herds come from `pbd_app::husbandry::wild_herds_near`, the one rule
//! the yard spawns them by, so a marker is always where a herd stands. They
//! are found when the map opens and again only once the player has moved
//! [`REFIND_M`], not every frame. A marker is the species' first colour
//! (`colours` in `husbandry.ron`) with a 1 px dark rim, as the map's other
//! markers have; the legend's Herds toggle hides them.

use super::map_screen::{MapChoice, MapView, PlayerPlace};
use super::menu::Screen;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use pbd_app::config::HusbandryConfig;
use pbd_app::husbandry::{Farmyard, wild_herds_near};
use pbd_app::planet::PLANET_RADIUS;
use pbd_app::saves::WorldSave;
use pbd_app::sea::Sea;
use pbd_core::fauna::HOME_BODY;
use pbd_core::geo;
use pbd_core::husbandry::{HerdSettings, LandSpecies, record};
use std::collections::{BTreeSet, HashMap};

/// How far from the player the map shows wild herds, metres.
pub const REACH_M: f32 = 1500.0;
/// How far the player moves before the herds are found again, metres.
pub const REFIND_M: f32 = 100.0;
/// A herd marker's width, UI pixels: under the waypoint pin's 16.
pub const DOT_PX: f32 = 7.0;
/// The rim every marker on the map has.
pub const RIM: Color = Color::srgb(20.0 / 255.0, 24.0 / 255.0, 30.0 / 255.0);
/// The ring round a tamed animal's marker.
const TAMED_RING: Color = Color::srgb(1.0, 0.98, 0.92);

/// Where the herd markers are drawn: over the sites, under the player's,
/// the craft's and the waypoints' markers.
#[derive(Component)]
pub struct HerdLayer;

/// A herd's or a tamed animal's marker, by its place in [`MapHerds`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HerdMarker(pub usize);

/// One marker: where, which species, and whether it is the player's own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HerdMark {
    /// A unit direction in the body's frame.
    pub at: Vec3,
    /// Roster index on the home body.
    pub species: u16,
    pub tamed: bool,
}

/// The markers found for the map, and where the player stood when they
/// were found.
#[derive(Resource, Default)]
pub struct MapHerds {
    pub marks: Vec<HerdMark>,
    /// Bumped whenever `marks` is refilled, so the nodes are remade.
    pub generation: u64,
    found_at: Option<Vec3>,
    open: bool,
}

/// A species' marker colour: its first colour, as its mesh is tinted.
pub fn species_colour(sp: &LandSpecies) -> Color {
    let [r, g, b] = sp.colours[0];
    Color::linear_rgb(r, g, b)
}

/// The tameable species the legend's Herds row shows, in roster order.
pub fn tameable(roster: &[LandSpecies]) -> impl Iterator<Item = &LandSpecies> {
    roster.iter().filter(|sp| sp.keep.is_some())
}

/// The wild herds the map shows round `up`: the yard's herds out to
/// [`REACH_M`], of tameable species, and not a herd whose every member the
/// player has tamed (`tamed` holds the tamed animals' wild seeds).
pub fn wild_marks(
    up: Vec3,
    settings: &HerdSettings,
    roster: &[LandSpecies],
    sea: Option<&Sea>,
    tamed: &BTreeSet<u64>,
) -> Vec<HerdMark> {
    wild_herds_near(up, REACH_M, settings, roster, sea)
        .into_iter()
        .filter(|h| {
            roster
                .get(h.species as usize)
                .is_some_and(|sp| sp.keep.is_some())
        })
        .filter(|h| h.members.iter().any(|m| !tamed.contains(&m.seed)))
        .map(|h| HerdMark {
            at: h.centre,
            species: h.species,
            tamed: false,
        })
        .collect()
}

/// Find the herds when the map opens, and again once the player has moved
/// [`REFIND_M`]. The tamed animals are read from the save's records, each
/// where the yard has it standing, else at its home cell.
#[allow(clippy::too_many_arguments)]
pub fn find_herds(
    screen: Res<Screen>,
    player: PlayerPlace,
    config: Option<Res<HusbandryConfig>>,
    sea: Option<Res<Sea>>,
    save: Option<Res<WorldSave>>,
    yard: Option<Res<Farmyard>>,
    mut herds: ResMut<MapHerds>,
) {
    if *screen != Screen::Map {
        if herds.open {
            herds.open = false;
        }
        return;
    }
    let (Some(config), Some(at)) = (config, player.get()) else {
        return;
    };
    let up = at.normalize_or(Vec3::Y);
    let opened = !herds.open;
    let moved = herds
        .found_at
        .is_none_or(|was| was.angle_between(up) * PLANET_RADIUS > REFIND_M);
    if !opened && !moved {
        return;
    }
    herds.open = true;
    herds.found_at = Some(up);
    let roster = config.0.roster(HOME_BODY);
    let kept: Vec<(u64, record::AnimalBody)> = save
        .as_deref()
        .map(|s| record::animals(&s.records))
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, b)| b.state == record::Life::Alive && b.body == HOME_BODY)
        .collect();
    let tamed: BTreeSet<u64> = kept.iter().map(|(_, b)| b.wild).collect();
    let mut marks = wild_marks(up, &config.0.herds, roster, sea.as_deref(), &tamed);
    let standing: HashMap<u64, Vec3> = yard
        .as_deref()
        .map(|y| y.kept(player.origin()).collect())
        .unwrap_or_default();
    for (id, body) in &kept {
        let at = standing
            .get(id)
            .map(|p| p.normalize_or(Vec3::Y))
            .or_else(|| pbd_core::cell_key::key_direction(body.home));
        if let Some(at) = at {
            marks.push(HerdMark {
                at,
                species: body.sp,
                tamed: true,
            });
        }
    }
    if herds.marks != marks {
        herds.marks = marks;
        herds.generation += 1;
    }
}

/// Make a node for each marker when they are found again, and place them
/// while the map is open; the legend's Herds toggle hides them all.
#[allow(clippy::too_many_arguments)]
pub fn draw_herds(
    mut commands: Commands,
    screen: Res<Screen>,
    view: Res<MapView>,
    choice: Res<MapChoice>,
    herds: Res<MapHerds>,
    config: Option<Res<HusbandryConfig>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    layer: Query<(Entity, Option<&Children>), With<HerdLayer>>,
    mut markers: Query<(&HerdMarker, &mut Node)>,
    ui_scale: Option<Res<UiScale>>,
    mut drawn: Local<Option<u64>>,
) {
    if *screen != Screen::Map {
        return;
    }
    let Ok((parent, children)) = layer.single() else {
        return;
    };
    if *drawn != Some(herds.generation) {
        for child in children.into_iter().flatten() {
            commands.entity(*child).despawn();
        }
        let roster = config.as_ref().map_or(&[][..], |c| c.0.roster(HOME_BODY));
        for (index, mark) in herds.marks.iter().enumerate() {
            let fill = roster
                .get(mark.species as usize)
                .map_or(Color::WHITE, species_colour);
            let mut child = commands.spawn((
                HerdMarker(index),
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    width: px(DOT_PX),
                    height: px(DOT_PX),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(RIM),
                BackgroundColor(fill),
            ));
            if mark.tamed {
                child.insert(Outline::new(px(1), px(1), TAMED_RING));
            }
            let child = child.id();
            commands.entity(parent).add_child(child);
        }
        *drawn = Some(herds.generation);
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let s = ui_scale.map_or(1.0, |u| u.0.max(1e-3));
    let size = Vec2::new(window.width(), window.height());
    let view = view.clamped(size);
    for (marker, mut node) in &mut markers {
        let mark = herds.marks.get(marker.0).filter(|_| choice.herds);
        let Some(mark) = mark else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let pixel = view.to_screen(geo::project(mark.at), size);
        node.left = px(pixel.x / s - DOT_PX / 2.0);
        node.top = px(pixel.y / s - DOT_PX / 2.0);
        node.display = Display::Flex;
    }
}
