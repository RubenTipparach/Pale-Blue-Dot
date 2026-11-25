//! The starter missions on screen (`starter-missions` design 12-14): one HUD
//! line for the current mission, and the Missions tab beside the pack.
//!
//! The counting and the record are `pbd_app::missions`; this only draws them.
//!
//! - **The HUD line** sits top right under the compass strip, in the kit's
//!   face: `CATCH FISH 1/3`, then `DONE: CATCH FISH` for a few seconds when
//!   one closes. The player's "Mission line" setting hides it.
//! - **The tab** is a menu screen of its own (`Screen::Missions`), opened
//!   with O, as the pack is with I; a strip of two oak tabs over both
//!   switches between them. The tab draws the tree (`mission_tree`).

use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::missions::{MissionChain, Missions};
use pbd_app::settings::PlayerSettings;
use pbd_app::ui_kit::Localized;
use pbd_app::ui_kit::{self, KitText, Plaque, Set, UiKit};
use pbd_core::locale::{tr, tr_with};

/// The HUD's mission tracker (`mission-playthrough` 3.5): the one mission
/// the player is on, its icon and title, then each objective with its icon
/// and count.
#[derive(Component)]
pub struct MissionLine;
/// The tracker's title row's icon and words.
#[derive(Component)]
pub struct TrackerTitleIcon;
#[derive(Component)]
pub struct TrackerTitle;
/// An objective's row, its icon and its words, by the objective's place.
#[derive(Component)]
pub struct TrackerRow(pub usize);
#[derive(Component)]
pub struct TrackerIcon(pub usize);
/// An objective's tick box, as the tree's card has it.
#[derive(Component)]
pub struct TrackerCheck(pub usize);
#[derive(Component)]
pub struct TrackerText(pub usize);

/// The most objectives a mission has (the shipped tree's most is three).
const TRACKER_ROWS: usize = 4;
/// An icon in the tracker, UI pixels.
const TRACKER_ICON: f32 = 16.0;

/// The strip of tabs over the pack and the missions, and what each opens.
#[derive(Component)]
pub struct TabStrip;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub struct TabTo(pub Screen);

/// The strip over the pack and the missions, left to right: their own two
/// tabs, then links to the map, the guide and the pause menu
/// (`inventory-grid` decision 9; owner, 2026-11-15: "pack should buttons to
/// other menus like the pause menu or the map").
pub const TABS: [(&str, Screen); 5] = [
    ("missions.tab.pack", Screen::Pack),
    ("missions.tab.missions", Screen::Missions),
    ("missions.tab.map", Screen::Map),
    ("missions.tab.guide", Screen::Guide),
    ("missions.tab.menu", Screen::Pause),
];

pub fn spawn(mut commands: Commands, kit: Res<UiKit>) {
    let kit = &*kit;
    // The tracker, at the left under the position readout.
    let row = || Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: px(4),
        ..default()
    };
    let icon = || {
        (
            ImageNode::default(),
            Node {
                width: px(TRACKER_ICON),
                height: px(TRACKER_ICON),
                ..default()
            },
        )
    };
    commands
        .spawn((
            // On the kit's stone, so it reads over any sky or ground.
            kit.nine(Set::Stone, "frame"),
            Node {
                position_type: PositionType::Absolute,
                // At the left, under the position readout's two lines
                // (owner, 2026-11-17: "Move it to the left side of the
                // screen").
                left: px(6),
                top: px(62),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: px(3),
                padding: UiRect::axes(px(10), px(8)),
                min_width: px(150),
                ..default()
            },
            Visibility::Hidden,
            MissionLine,
        ))
        .with_children(|tracker| {
            // The mission's title is the header: no icon, outlined lettering,
            // a rule under it (owner, 2026-11-19: "if its a header, you
            // should maybe not place an icon in front of it, and make the
            // header bold and appera like a header").
            tracker.spawn((
                KitText::new(Localized::empty(), ui_kit::TITLE).outline(ui_kit::INK),
                TrackerTitle,
            ));
            tracker.spawn((
                Node {
                    width: percent(100),
                    height: px(1),
                    margin: UiRect::bottom(px(1)),
                    ..default()
                },
                BackgroundColor(ui_kit::STONE_MID),
            ));
            for k in 0..TRACKER_ROWS {
                tracker.spawn((row(), TrackerRow(k))).with_children(|r| {
                    r.spawn((
                        kit.whole(Set::Stone, "check-off"),
                        Node {
                            width: px(9),
                            height: px(9),
                            ..default()
                        },
                        TrackerCheck(k),
                    ));
                    r.spawn((icon(), TrackerIcon(k)));
                    r.spawn((
                        KitText::new(Localized::empty(), ui_kit::TEXT).shadow(ui_kit::INK),
                        TrackerText(k),
                    ));
                });
            }
        });
    // The tabs, over whichever of the two panels is open.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(10),
                width: percent(100),
                justify_content: JustifyContent::Center,
                column_gap: px(4),
                display: Display::None,
                ..default()
            },
            GlobalZIndex(12),
            TabStrip,
        ))
        .with_children(|strip| {
            for (label, to) in TABS {
                strip
                    .spawn((ui_kit::plaque(kit, Set::Oak, 70.0), TabTo(to)))
                    .with_children(|p| {
                        p.spawn(ui_kit::plaque_label(Set::Oak, tr(label)));
                    });
            }
        });
}

/// The HUD line: the current mission, or the one just done, unless the
/// player hid it or a menu is open.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn show_line(
    missions: Res<Missions>,
    chain: Res<MissionChain>,
    screen: Res<Screen>,
    settings: Res<PlayerSettings>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    (assets, items, images, kit, recipes): (
        Res<AssetServer>,
        Option<Res<super::slots::ItemIcons>>,
        Res<Assets<Image>>,
        Res<UiKit>,
        Res<pbd_app::config::Recipes>,
    ),
    mut checks: Query<
        (&TrackerCheck, &mut ImageNode),
        (Without<TrackerIcon>, Without<TrackerTitleIcon>),
    >,
    mut tracker: Query<&mut Visibility, With<MissionLine>>,
    mut title: Query<&mut KitText, (With<TrackerTitle>, Without<TrackerText>)>,
    mut title_icon: Query<&mut ImageNode, (With<TrackerTitleIcon>, Without<TrackerIcon>)>,
    mut rows: Query<(&TrackerRow, &mut Node), Without<TrackerIcon>>,
    mut texts: Query<(&TrackerText, &mut KitText), Without<TrackerTitle>>,
    mut icons: Query<
        (&TrackerIcon, &mut ImageNode, &mut Node),
        (Without<TrackerTitleIcon>, Without<TrackerRow>),
    >,
    mut mission_icon: Local<Option<(String, Handle<Image>)>>,
    (town, town_chain): (
        Option<Res<pbd_app::town_missions::TownMissions>>,
        Option<Res<pbd_app::town_missions::TownChain>>,
    ),
) {
    // One focus across the two trees (`town-missions-tree` design 6): a
    // town mission picked on its tree, else the starter tree's, else the
    // town tree's first open one.
    let town_progress = town.as_deref().and_then(|t| t.progress());
    let town_tree = town_chain.as_deref().map(|c| &c.0);
    let town_focus = town_progress.zip(town_tree).and_then(|(p, c)| {
        p.focus
            .as_deref()
            .and_then(|_| p.focused(c))
            .map(|m| (m, p))
    });
    let mission = town_focus
        .or_else(|| {
            missions
                .progress
                .focused(&chain.0)
                .map(|m| (m, &missions.progress))
        })
        .or_else(|| {
            town_progress
                .zip(town_tree)
                .and_then(|(p, c)| p.focused(c).map(|m| (m, p)))
        });
    let (mission, progress) = match mission {
        Some((m, p)) => (Some(m), Some(p)),
        None => (None, None),
    };
    let notice = missions
        .notice
        .as_ref()
        .map(|(t, _)| t.clone())
        .or_else(|| {
            town.as_deref()
                .and_then(|t| t.notice.as_ref())
                .map(|(t, _, _)| t.clone())
        });
    let shown = settings.mission_line
        && *screen == Screen::Playing
        && walking.is_some_and(|w| w.active)
        && (mission.is_some() || notice.is_some());
    for mut visibility in &mut tracker {
        let want = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
    }
    if !shown {
        return;
    }
    let set = |words: &mut KitText, text: Localized, colour: Color| {
        if words.text != text || words.colour != colour {
            words.text = text;
            words.colour = colour;
        }
    };
    // Just finished: its title alone, in the rune's colour, for a moment.
    let (heading, colour, objectives) = match (&notice, mission) {
        (Some(done), _) => (
            tr_with("missions.done", &[("title", &done)]),
            ui_kit::RUNE,
            None,
        ),
        (None, Some(m)) => (Localized::verbatim(m.title.clone()), ui_kit::TITLE, Some(m)),
        (None, None) => return,
    };
    for mut words in &mut title {
        set(&mut words, heading.clone(), colour);
    }
    // The mission's own icon, by the title and for objectives with no item.
    let icon_path = mission.map(|m| m.icon.clone()).unwrap_or_default();
    if mission_icon.as_ref().is_none_or(|(p, _)| *p != icon_path) {
        *mission_icon = (!icon_path.is_empty()).then(|| {
            (
                icon_path.clone(),
                assets.load(format!("items/{icon_path}.png")),
            )
        });
    }
    let own = mission_icon.as_ref().map(|(_, h)| h.clone());
    for mut node in &mut title_icon {
        match &own {
            Some(h) => {
                if node.image != *h {
                    node.image = h.clone();
                }
                node.rect = None;
                node.color = Color::WHITE;
            }
            None => node.color = Color::NONE,
        }
    }
    let counts = objectives
        .zip(progress)
        .map(|(m, p)| p.counts(m))
        .unwrap_or_default();
    let objective = |k: usize| objectives.and_then(|m| m.objectives.get(k));
    for (row, mut node) in &mut rows {
        let display = if objective(row.0).is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (which, mut words) in &mut texts {
        if let Some(o) = objective(which.0) {
            let n = counts.get(which.0).copied().unwrap_or(0).min(o.count);
            let c = if n >= o.count {
                ui_kit::TEXT_DIM
            } else {
                ui_kit::TEXT
            };
            set(
                &mut words,
                // A distance counts in metres ("Swim 4/20 m").
                if matches!(
                    o.kind,
                    pbd_core::missions::Kind::Swim | pbd_core::missions::Kind::Travel
                ) {
                    tr_with(
                        "missions.objective_metres",
                        &[("label", &o.label), ("count", &n), ("total", &o.count)],
                    )
                } else {
                    Localized::verbatim(format!("{} {n}/{}", o.label, o.count))
                },
                c,
            );
        }
    }
    for (which, mut image) in &mut checks {
        let Some(o) = objective(which.0) else {
            continue;
        };
        let full = counts.get(which.0).copied().unwrap_or(0) >= o.count;
        let want = kit.whole(Set::Stone, if full { "check-on" } else { "check-off" });
        if image.image != want.image {
            image.image = want.image;
        }
    }
    for (which, mut node, mut layout) in &mut icons {
        let Some(o) = objective(which.0) else {
            continue;
        };
        // The objective's item, when it names one; the mission's icon else.
        let item = match &o.want {
            // An item by its name, or what a named recipe makes.
            pbd_core::missions::Want::Names(names) => names.iter().find_map(|n| {
                pbd_core::station::item_named(n).or_else(|| {
                    n.strip_prefix("recipe:")
                        .and_then(|code| recipes.0.get(code))
                        .and_then(|r| r.outputs.first().map(|(item, _)| *item))
                })
            }),
            _ => None,
        };
        // An act with no item is shown by the tool that does it: the rod
        // for fishing, the axe for felling, the shovel for tilling.
        use pbd_core::inventory::Tool;
        use pbd_core::missions::Kind;
        let item = item.or(match o.kind {
            Kind::Cast | Kind::Catch | Kind::Baited | Kind::Species | Kind::Waters => {
                Some(pbd_core::inventory::Item::Tool(Tool::Rod))
            }
            Kind::Fell => Some(pbd_core::inventory::Item::Tool(Tool::Axe)),
            Kind::Till => Some(pbd_core::inventory::Item::Tool(Tool::Shovel)),
            _ => None,
        });
        // Only an icon that says something the title's does not: the
        // mission's own thing (a campfire to make and place) shows once, by
        // the title (owner: "why does it say make twice").
        let repeats = item.is_some_and(|i| {
            let code = pbd_core::missions::item_code(i);
            let tail = code.rsplit(':').next().unwrap_or(&code).to_string();
            objectives.is_some_and(|m| m.icon.ends_with(&tail))
        });
        let display = if item.is_some() && !repeats {
            Display::Flex
        } else {
            Display::None
        };
        if layout.display != display {
            layout.display = display;
        }
        let done = counts.get(which.0).copied().unwrap_or(0) >= o.count;
        match (item, items.as_deref()) {
            (Some(item), Some(items)) => {
                super::slots::paint_icon(
                    &mut node,
                    Some(pbd_core::inventory::Stack { item, count: 1 }),
                    items,
                    &images,
                );
            }
            _ => match &own {
                Some(h) => {
                    if node.image != *h {
                        node.image = h.clone();
                    }
                    node.rect = None;
                    node.color = Color::WHITE;
                }
                None => node.color = Color::NONE,
            },
        }
        if done {
            node.color = node.color.with_alpha(0.45);
        }
    }
}

/// Keep the tab strip current (the tab itself is `mission_tree`).
pub fn paint(
    screen: Res<Screen>,
    mut strip: Query<&mut Node, With<TabStrip>>,
    mut tabs: Query<(&TabTo, &mut Plaque)>,
) {
    let open = matches!(*screen, Screen::Pack | Screen::Missions);
    for mut node in &mut strip {
        let want = if open { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
    }
    for (to, mut plaque) in &mut tabs {
        let latched = to.0 == *screen;
        if plaque.latched != latched {
            plaque.latched = latched;
        }
    }
}

/// A press on a tab switches between the pack and the missions.
pub fn press_tabs(
    tabs: Query<(&Interaction, &TabTo), Changed<Interaction>>,
    mut screen: ResMut<Screen>,
    mut view: ResMut<super::map_screen::MapView>,
    player: super::map_screen::PlayerPlace,
) {
    for (interaction, to) in &tabs {
        if *interaction != Interaction::Pressed || *screen == to.0 {
            continue;
        }
        // The map opens centred on the player, as M does.
        match to.0 {
            Screen::Map => super::map_screen::show(&mut screen, &mut view, &player),
            other => *screen = other,
        }
    }
}

/// One step of the capture instrument's chain: a message the game raises.
#[derive(Clone, Debug)]
enum Scripted {
    Gained(pbd_core::inventory::Item, u16, pbd_app::hotbar::How),
    Did(pbd_core::missions::Kind, &'static str),
    Planted(&'static str),
    Deed(pbd_app::husbandry::Deed),
    Cooked(&'static str, u16),
}

/// The whole chain as the systems would raise it in play, from a fresh
/// world's first dig to the farm meal (`--missions <n>` raises the first
/// `n`, one every few frames). It plays no fishing or farming: it raises
/// the messages, so the counting, the record, the HUD line and the tab are
/// the game's own.
fn script() -> Vec<Scripted> {
    use Scripted as S;
    use pbd_app::hotbar::How::*;
    use pbd_app::husbandry::Deed;
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Tool};
    use pbd_core::missions::Kind;
    use pbd_core::terrain::Material;
    let good = |code| Item::Good(Good::by_code(code).expect("a shipped good"));
    let n = |k: usize, step: Scripted| std::iter::repeat_n(step, k);
    // The tree (`starter-missions` T.2) in the order the bot plays it.
    let mut steps = vec![
        S::Gained(good("fibre"), 8, Found),
        S::Gained(good("stone"), 9, Found),
    ];
    steps.extend(n(6, S::Did(Kind::Gather, "block:dirt")));
    steps.push(S::Did(Kind::Open, "pack"));
    steps.push(S::Did(Kind::Craft, "recipe:stone-axe"));
    steps.extend(n(2, S::Did(Kind::Fell, "tree")));
    steps.extend([
        S::Gained(good("log"), 10, Found),
        S::Gained(good("planks"), 8, Made),
        S::Gained(good("rope"), 2, Made),
        S::Gained(Item::Block(Material::Workbench), 1, Made),
        S::Did(Kind::Place, "block:workbench"),
        S::Gained(Item::Tool(Tool::Pickaxe), 1, Made),
        S::Gained(Item::Tool(Tool::Shovel), 1, Made),
        S::Gained(Item::Block(Material::Campfire), 1, Made),
        S::Did(Kind::Place, "block:campfire"),
        // The storage mission (`storage-chests` 9).
        S::Gained(Item::Block(Material::Chest), 1, Made),
        S::Did(Kind::Place, "block:chest"),
        S::Did(Kind::Store, "block:dirt"),
    ]);
    steps.extend(n(20, S::Did(Kind::Place, "block:dirt")));
    steps.extend(n(2, S::Did(Kind::Craft, "recipe:torch")));
    steps.extend(n(4, S::Did(Kind::Place, "block:torch")));
    steps.extend([
        S::Did(Kind::Night, "light"),
        S::Gained(Item::Tool(Tool::Rod), 1, Made),
        S::Did(Kind::Cast, "water"),
    ]);
    steps.extend(n(3, S::Did(Kind::Catch, "minnow")));
    steps.push(S::Gained(good("bait"), 3, Found));
    steps.extend(n(3, S::Did(Kind::Baited, "minnow")));
    steps.extend([
        S::Did(Kind::Species, "minnow"),
        S::Did(Kind::Species, "banded perch"),
        S::Did(Kind::Species, "eel"),
    ]);
    steps.extend([S::Did(Kind::Open, "map"), S::Did(Kind::Reach, "marked")]);
    steps.extend(n(3, S::Did(Kind::Waters, "sea")));
    steps.extend(n(3, S::Did(Kind::Waters, "fresh")));
    steps.extend(n(20, S::Did(Kind::Swim, "water")));
    steps.extend([
        S::Did(Kind::Gather, "block:reef rock"),
        S::Did(Kind::Reach, "town"),
        S::Did(Kind::Board, "boat"),
    ]);
    steps.extend(n(100, S::Did(Kind::Travel, "boat")));
    steps.extend([
        S::Cooked("grilled-fish", 2),
        S::Did(Kind::Eat, "grilled-fish"),
    ]);
    steps.push(S::Gained(good("wheat-seed"), 10, Found));
    steps.extend(n(6, S::Did(Kind::Till, "block:grass")));
    steps.extend(n(6, S::Planted("wheat")));
    steps.push(S::Did(Kind::Pour, "water"));
    steps.extend([
        S::Gained(good("wheat"), 6, Harvested),
        S::Cooked("flatbread", 2),
        S::Gained(good("turnip"), 3, Harvested),
        S::Gained(good("chili"), 3, Harvested),
        S::Gained(good("frost-kale"), 3, Harvested),
        S::Cooked("roast-turnip", 1),
    ]);
    steps.extend(n(6, S::Did(Kind::Craft, "recipe:fence")));
    steps.extend([
        S::Did(Kind::Craft, "recipe:gate"),
        S::Deed(Deed::Tamed { species: 0 }),
        S::Deed(Deed::PenBuilt { pen: 1, cells: 7 }),
        S::Deed(Deed::Penned { species: 0 }),
        S::Gained(good("egg"), 5, Collected),
        S::Deed(Deed::Tamed { species: 1 }),
        S::Gained(good("wool"), 3, Collected),
        S::Cooked("fried-egg", 1),
        S::Did(Kind::Eat, "grilled-fish"),
        S::Did(Kind::Eat, "flatbread"),
        S::Did(Kind::Eat, "fried-egg"),
    ]);
    steps.extend(n(10, S::Did(Kind::Gather, "block:stone")));
    steps.extend(n(4, S::Did(Kind::Gather, "block:ore")));
    steps.extend([
        S::Gained(good("coal"), 3, Found),
        S::Gained(Item::Block(Material::Forge), 1, Made),
        S::Did(Kind::Place, "block:forge"),
        S::Gained(good("bronze-ingot"), 2, Made),
        S::Did(Kind::Craft, "recipe:bronze-pickaxe"),
    ]);
    steps.extend(n(5, S::Did(Kind::Gather, "block:stone")));
    steps.extend([
        S::Gained(good("iron-ingot"), 3, Made),
        S::Did(Kind::Craft, "recipe:iron-pickaxe"),
    ]);
    steps
}

/// How many frames apart the instrument raises its steps.
const SCRIPT_EVERY: u32 = 2;

/// Write one scripted step as the message its system raises.
fn raise(step: Scripted, world: &mut World) {
    match step {
        Scripted::Gained(item, count, how) => {
            world.write_message(pbd_app::hotbar::Gained { item, count, how });
        }
        Scripted::Did(kind, what) => {
            world.write_message(pbd_app::missions::Did::new(kind, what, 1));
        }
        Scripted::Planted(crop) => {
            if let Some(crop) = pbd_core::farming::Crop::by_code(crop) {
                world.write_message(pbd_app::hotbar::Planted { crop });
            }
        }
        Scripted::Deed(deed) => {
            world.write_message(deed);
        }
        Scripted::Cooked(recipe, count) => {
            if let Some(dish) = pbd_core::goods::Good::by_code(recipe) {
                world.write_message(pbd_app::kitchen::Cooked {
                    recipe: recipe.to_string(),
                    dish: pbd_core::inventory::Item::Good(dish),
                    count,
                    fish: 0,
                });
            }
        }
    }
}

/// `--missions <n>`: raise the first `n` steps of [`script`], one every
/// [`SCRIPT_EVERY`] frames once the world is open (a measurement
/// instrument, `starter-missions` design 17).
pub fn scripted(world: &mut World, mut at: Local<(usize, u32)>) {
    let Some(n) = world.resource::<super::Launch>().missions else {
        return;
    };
    at.1 += 1;
    if at.1 < SCRIPT_EVERY {
        return;
    }
    at.1 = 0;
    let Some(step) = script().into_iter().nth(at.0).filter(|_| at.0 < n) else {
        return;
    };
    at.0 += 1;
    info!("scripted mission step {}: {step:?}", at.0);
    raise(step, world);
}

/// `--town-focus <id>`: once the town tree is open, make that mission the
/// one the player is on, as a pick on the tree would (a capture
/// instrument, `town-missions-tree` design 6).
pub fn scripted_town(
    launch: Res<super::Launch>,
    chain: Option<Res<pbd_app::town_missions::TownChain>>,
    town: Option<ResMut<pbd_app::town_missions::TownMissions>>,
    mut done: Local<bool>,
) {
    let (Some(id), Some(chain), Some(mut town)) = (launch.town_focus.as_deref(), chain, town)
    else {
        return;
    };
    if *done || !town.is_open() {
        return;
    }
    let picked = town
        .progress_mut()
        .is_some_and(|p| p.focus_on(&chain.0, id));
    if picked {
        info!("town mission {id} is the focus");
        town.refocused = true;
    }
    *done = true;
}

/// The hour a night by the lights is counted from.
pub const NIGHT_HOUR: f32 = 20.0;
/// How near a light the player placed they have to be, metres.
pub const NIGHT_NEAR_M: f32 = 30.0;

/// `night` (`starter-missions` H.1): the clock passes 20:00 with the
/// player within 30 m of a light they placed. Raised once each evening.
pub fn night_watch(
    sun: Res<pbd_app::sky::Sun>,
    save: Res<pbd_app::saves::WorldSave>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    mut was: Local<Option<f32>>,
    space: pbd_app::SimSpace,
) {
    let hour = sun.clock.hour();
    let crossed = was.is_some_and(|h| h < NIGHT_HOUR && hour >= NIGHT_HOUR);
    *was = Some(hour);
    if !crossed {
        return;
    }
    let (Some(fine), Ok(feet)) = (fine, walkers.single()) else {
        return;
    };
    let lights: Vec<_> = save
        .edits
        .all()
        .into_iter()
        .filter(|e| pbd_app::hotbar::light_icon(e.material).is_some())
        .collect();
    if lights.is_empty() {
        return;
    }
    let toward = space.origin().up_at(feet.0);
    let near = fine.set.finest_records().iter().any(|r| {
        lights.iter().any(|e| e.cell == r.key())
            && Vec3::from_slice(&r.direction_height[..3])
                .normalize_or(Vec3::Y)
                .distance(toward)
                * pbd_app::planet::PLANET_RADIUS
                < NIGHT_NEAR_M
    });
    if near {
        info!("night by the lights at {hour:.2}");
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Night,
            "light",
            1,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tab of the strip, and every sea place on the chart, has its
    /// words in the locale file.
    #[test]
    fn every_tab_and_place_has_its_words() {
        let english = pbd_core::locale::Locale::english();
        let keys = super::super::mission_tree::BRANCHES
            .iter()
            .map(|(_, key)| key.to_string())
            .chain(TABS.iter().map(|(key, _)| key.to_string()))
            .chain(
                pbd_app::world_map::PLACE_KEY
                    .iter()
                    .chain(pbd_app::world_map::BIOME_KEY)
                    .map(|(key, _)| key.to_string()),
            );
        let layers = [
            super::super::map_screen::biomes_layer(),
            super::super::map_screen::places_layer(),
        ];
        let keys = keys.chain(
            layers
                .iter()
                .flat_map(|l| [l.name.to_string(), l.note.to_string()]),
        );
        for key in keys {
            assert!(english.get(&key).is_some(), "{key}");
        }
    }

    /// The instrument's script walks the shipped chain to its end, through
    /// the same counting the game runs.
    #[test]
    fn the_script_walks_the_shipped_chain() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let (recipes, _) = pbd_app::config::recipe_book();
        app.insert_resource(recipes)
            .insert_resource(pbd_app::config::FaunaConfig(
                ron::from_str(include_str!("../../../../assets/config/fauna.ron")).unwrap(),
            ))
            .insert_resource(pbd_app::config::HusbandryConfig(Default::default()))
            .insert_resource(pbd_app::saves::WorldSave::memory_only())
            .insert_resource(pbd_app::hotbar::Hotbar::default())
            .insert_resource(pbd_app::fish::ToolSlot(
                pbd_core::inventory::Equipment::new_world(),
            ))
            .add_plugins(pbd_app::missions::MissionsPlugin);
        app.update();
        for step in script() {
            raise(step, app.world_mut());
            app.update();
        }
        let chain = &app.world().resource::<MissionChain>().0;
        let missions = app.world().resource::<Missions>();
        assert_eq!(missions.progress.current(chain).map(|m| &m.id), None);
        // Every mission whose kind the game raises; the pending ones wait.
        assert_eq!(
            missions.progress.done.len(),
            chain
                .missions
                .iter()
                .filter(|m| m.pending.is_none())
                .count()
        );
    }

    /// Each plaque in the strip puts its own screen up from the pack
    /// (`inventory-grid` decision 9).
    #[test]
    fn each_strip_tab_opens_its_screen() {
        for (label, to) in TABS {
            let mut app = App::new();
            app.insert_resource(Screen::Pack)
                .init_resource::<super::super::map_screen::MapView>()
                .add_systems(Update, press_tabs);
            app.world_mut()
                .spawn((Button, Interaction::Pressed, TabTo(to)));
            app.update();
            assert_eq!(*app.world().resource::<Screen>(), to, "{label}");
        }
    }
}

/// How near a town's terrace counts as reaching it.
pub const TOWN_NEAR_M: f32 = 60.0;

/// What `about_watch` carries between frames.
#[derive(Default)]
pub struct AboutState {
    /// The screen last frame.
    screen: Option<Screen>,
    /// The waypoint last counted as reached, so standing at one counts it
    /// once.
    reached_mark: Option<[f32; 3]>,
    /// The last place the walker was counted from, in the water.
    swam_from: Option<Vec3>,
    swum_m: f32,
    /// The craft boarded, and where it was last counted from.
    aboard: Option<(Entity, Vec3)>,
    travelled_m: f32,
    town: Option<u32>,
    reef: bool,
}

/// Getting about (`starter-missions` T.2): the map opened, the player's
/// mark reached (G3), metres swum, a boat boarded and the metres it went, a
/// town and a reef reached. Each is raised where the game already knows it:
/// the screen, the walker's position against the sea and the towns, the
/// craft the player is aboard.
#[allow(clippy::too_many_arguments)]
pub fn about_watch(
    screen: Res<Screen>,
    aboard: Option<Res<pbd_app::vehicles::Aboard>>,
    vehicles: Query<&pbd_app::vehicles::Vehicle>,
    towns: Option<Res<pbd_app::towns::Towns>>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    save: Option<Res<pbd_app::saves::WorldSave>>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    mut state: Local<AboutState>,
    space: pbd_app::SimSpace,
) {
    use pbd_app::missions::Did;
    use pbd_app::planet::{PLANET_RADIUS, surface_height};
    use pbd_core::missions::Kind;
    let at = walkers
        .single()
        .ok()
        .map(|p| space.origin().query_point(p.0));
    if state.screen != Some(*screen) {
        if *screen == Screen::Map {
            did.write(Did::new(Kind::Open, "map", 1));
        }
        if *screen == Screen::Pack {
            did.write(Did::new(Kind::Open, "pack", 1));
        }
        state.screen = Some(*screen);
    }
    let Some(feet) = at else { return };
    let up = feet.normalize_or(Vec3::Y);
    // A waypoint reached: one set far enough away counts once.
    let marks = save
        .as_deref()
        .map(super::map_screen::marks_of)
        .unwrap_or_default();
    if let Some(mark) = marks
        .iter()
        .find(|m| state.reached_mark != Some(m.at) && m.reached_far(feet, PLANET_RADIUS))
    {
        state.reached_mark = Some(mark.at);
        did.write(Did::new(Kind::Reach, "marked", 1));
    }
    // Swimming: the feet under the sea's level over the sea floor.
    let ground = surface_height(up);
    let altitude = feet.length() - PLANET_RADIUS;
    if ground < -1.0 && altitude < -0.3 {
        if let Some(from) = state.swam_from {
            state.swum_m += (feet - from).length();
        }
        state.swam_from = Some(feet);
        while state.swum_m >= 1.0 {
            state.swum_m -= 1.0;
            did.write(Did::new(Kind::Swim, "water", 1));
        }
        // A reef under the swimmer, counted each time one is come onto
        // (`mission-playthrough` 3.2), as a town is.
        let on_reef = pbd_core::ocean::places::environment(pbd_app::planet::terrain_config(), up)
            == Some(pbd_core::fauna::Environment::Reef);
        if on_reef && !state.reef {
            did.write(Did::new(Kind::Reach, "reef", 1));
        }
        state.reef = on_reef;
    } else {
        state.swam_from = None;
        state.reef = false;
    }
    // A town reached.
    if let Some(towns) = towns {
        let near = towns.held.iter().find(|h| {
            h.point(PLANET_RADIUS).normalize_or(Vec3::Y).distance(up) * PLANET_RADIUS < TOWN_NEAR_M
        });
        let id = near.map(|h| h.site.id);
        if id.is_some() && id != state.town {
            did.write(Did::new(Kind::Reach, "town", 1));
        }
        state.town = id;
    }
    // A boat boarded, and the metres it goes.
    let craft = aboard
        .and_then(|a| a.0)
        .and_then(|e| vehicles.get(e).ok().map(|v| (e, v)))
        // A town's boat (a craft with a harbour berth, `retire-rowboat-and-cog`:
        // the harbours' canoes) or a made one.
        .filter(|(_, v)| {
            v.craft.berth.is_some()
                || matches!(
                    v.craft.kind,
                    pbd_core::vehicle::Kind::Raft | pbd_core::vehicle::Kind::Tube
                )
        });
    match (craft, state.aboard) {
        (Some((e, v)), Some((was, from))) if e == was => {
            let here = v.craft.body.position.as_vec3();
            state.travelled_m += (here - from).length();
            state.aboard = Some((e, here));
            while state.travelled_m >= 1.0 {
                state.travelled_m -= 1.0;
                did.write(Did::new(Kind::Travel, "boat", 1));
            }
        }
        (Some((e, v)), _) => {
            did.write(Did::new(Kind::Board, "boat", 1));
            state.aboard = Some((e, v.craft.body.position.as_vec3()));
        }
        (None, _) => state.aboard = None,
    }
}
