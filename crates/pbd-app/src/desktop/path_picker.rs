//! The path picker (`survival-mode` design 18, M33 version 11): in a Survival
//! world, "Choose a path" over the mission tree when the fire is lit and
//! again when the chosen path runs out.
//!
//! - **The cards**: one a path with a mission open, in the side panel's
//!   stone kit (a stone card, carved oak under the pointer): the path's icon
//!   in a slot, its name, one plain line, its first mission and its count. A
//!   started path that waits keeps its card, greyed, naming what it waits on.
//! - **A pick** makes the path's first open mission Now (the HUD's line
//!   follows it), pans the tree to it and lights its lane. The other paths
//!   stay open and count.
//! - **Later** closes it; the HUD then reads "Choose a path" and the picker
//!   shows the next time the Missions screen opens. **Paths**, on the
//!   Missions screen, opens it at any time.
//! - **Explorer** never shows it: the rules give an Explorer world no cards.
//!
//! The rules are `pbd_core::missions` (`path_cards`, `choose_path`,
//! `offer_after`); the offer is raised by `pbd_app::missions::count`.

#![allow(clippy::type_complexity)] // Bevy queries.
use super::menu::{self, Screen};
use super::mission_tree::{self, BranchLabel, ShownTree, TreeView};
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use pbd_app::missions::{MissionChain, Missions};
use pbd_app::ui_kit::{self, KitText, Localized, Set, UiKit, tr};
use pbd_core::locale::tr_with;
use pbd_core::missions::{Offer, PATH_ORDER, PathCard};

/// Whether the picker is up, and why it was offered.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct Picker {
    pub shown: bool,
    pub reason: Option<Offer>,
}

/// The picker's layer, over the tree.
#[derive(Component)]
pub struct PickerLayer;
/// The line under the heading.
#[derive(Component)]
pub struct PickerSub;
/// The card in this place, its parts by the same place.
#[derive(Component)]
pub struct PathSlot(pub usize);
#[derive(Component)]
pub struct PathIcon(pub usize);
#[derive(Component)]
pub struct PathName(pub usize);
#[derive(Component)]
pub struct PathCount(pub usize);
#[derive(Component)]
pub struct PathLine(pub usize);
#[derive(Component)]
pub struct PathFirst(pub usize);
/// The branch the card in a place stands for, while it shows one; whether it
/// can be picked.
#[derive(Component, Default)]
pub struct PathOf(pub Option<(String, bool)>);
/// The Later plaque.
#[derive(Component)]
pub struct PickerLater;
/// The Missions screen's Paths plaque.
#[derive(Component)]
pub struct PathsButton;

/// The most cards: every path the picker knows, and one for a branch it
/// does not.
const SLOTS: usize = PATH_ORDER.len() + 1;
/// A card's size, UI pixels.
const CARD_W: f32 = 214.0;
const CARD_H: f32 = 76.0;
/// The icon in its slot.
const ICON: f32 = 24.0;

pub fn spawn(mut commands: Commands, kit: Res<UiKit>) {
    let kit = &*kit;
    // The Paths plaque, at the Missions screen's top left.
    commands
        .spawn((
            ui_kit::plaque(kit, Set::Stone, 60.0),
            PathsButton,
            GlobalZIndex(12),
        ))
        .insert(Node {
            position_type: PositionType::Absolute,
            left: px(8),
            top: px(42),
            min_width: px(60),
            height: px(22),
            padding: UiRect::new(px(10), px(10), px(8), px(0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            display: Display::None,
            ..default()
        })
        .with_children(|p| {
            p.spawn(ui_kit::plaque_label(Set::Stone, tr("mission_path.paths")));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(12)),
                display: Display::None,
                ..default()
            },
            BackgroundColor(ui_kit::DIMMER),
            // The tree under it takes no press or drag while it is up.
            Interaction::default(),
            FocusPolicy::Block,
            GlobalZIndex(25),
            PickerLayer,
        ))
        .with_children(|layer| {
            layer
                .spawn((
                    kit.nine(Set::Stone, "frame"),
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6),
                        padding: UiRect::all(px(16)),
                        max_width: px(3.0 * CARD_W + 2.0 * 6.0 + 32.0),
                        ..default()
                    },
                ))
                .with_children(|frame| {
                    // The heading: an oak lintel, and the line beside it.
                    frame
                        .spawn(Node {
                            align_items: AlignItems::Center,
                            column_gap: px(8),
                            ..default()
                        })
                        .with_children(|head| {
                            head.spawn((
                                kit.nine(Set::Oak, "button-idle"),
                                Node {
                                    padding: UiRect::new(px(8), px(8), px(5), px(3)),
                                    ..default()
                                },
                            ))
                            .with_children(|lintel| {
                                lintel.spawn(KitText::new(
                                    tr("mission_path.choose"),
                                    ui_kit::OAK_LABEL,
                                ));
                            });
                            head.spawn((menu::body("", 11.0, ui_kit::TEXT_DIM), PickerSub));
                        });
                    frame
                        .spawn(Node {
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(6),
                            row_gap: px(6),
                            width: px(3.0 * CARD_W + 2.0 * 6.0),
                            ..default()
                        })
                        .with_children(|grid| {
                            for k in 0..SLOTS {
                                spawn_card(grid, kit, k);
                            }
                        });
                    frame
                        .spawn(Node {
                            justify_content: JustifyContent::FlexEnd,
                            width: percent(100),
                            ..default()
                        })
                        .with_children(|foot| {
                            foot.spawn((ui_kit::plaque(kit, Set::Oak, 64.0), PickerLater))
                                .with_children(|p| {
                                    p.spawn(ui_kit::plaque_label(
                                        Set::Oak,
                                        tr("mission_path.later"),
                                    ));
                                });
                        });
                });
        });
}

fn spawn_card(grid: &mut ChildSpawnerCommands, kit: &UiKit, k: usize) {
    grid.spawn((
        Button,
        kit.nine(Set::Stone, "tooltip"),
        Node {
            width: px(CARD_W),
            min_height: px(CARD_H),
            padding: UiRect::axes(px(8), px(7)),
            column_gap: px(7),
            align_items: AlignItems::FlexStart,
            display: Display::None,
            ..default()
        },
        PathSlot(k),
        PathOf::default(),
    ))
    .with_children(|card| {
        // The path's icon, in a slot well.
        card.spawn((
            kit.nine(Set::Stone, "slot-idle"),
            Node {
                width: px(ICON + 8.0),
                height: px(ICON + 8.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|well| {
            well.spawn((
                ImageNode::default(),
                Node {
                    width: px(ICON),
                    height: px(ICON),
                    ..default()
                },
                PathIcon(k),
            ));
        });
        card.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            flex_grow: 1.0,
            min_width: px(0),
            ..default()
        })
        .with_children(|words| {
            words
                .spawn(Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::FlexStart,
                    column_gap: px(4),
                    width: percent(100),
                    ..default()
                })
                .with_children(|top| {
                    top.spawn((
                        KitText::new(Localized::empty(), ui_kit::TITLE).shadow(ui_kit::INK),
                        PathName(k),
                    ));
                    top.spawn((menu::body("", 10.0, ui_kit::LABEL_DIM), PathCount(k)));
                });
            words.spawn((menu::body("", 10.0, ui_kit::TEXT), PathLine(k)));
            words.spawn((menu::body("", 10.0, ui_kit::RUNE), PathFirst(k)));
        });
    });
}

/// Each path's icon on its card (M33 version 11's picker), a path under
/// `assets/items/`: what the path is for, not its first step.
const PATH_ICONS: [(&str, &str); 11] = [
    ("eat-drink", "goods/water-bucket"),
    ("hunting", "goods/roast-meat"),
    ("fishing", "tools/rod"),
    ("farming", "goods/wheat"),
    ("shelter", "lights/campfire"),
    ("water", "water_life/sea_otter"),
    ("herding", "missions/yard-hen"),
    ("mining", "goods/bronze-ingot"),
    ("light", "lights/torch"),
    ("cooking", "goods/grilled-fish"),
    ("about", "missions/map"),
];

/// The path's icon: its own, or its first mission's for a path not named.
fn icon_of(chain: &pbd_core::missions::Chain, branch: &str) -> String {
    PATH_ICONS
        .iter()
        .find(|(b, _)| *b == branch)
        .map(|(_, icon)| icon.to_string())
        .or_else(|| {
            chain
                .missions
                .iter()
                .find(|m| m.branch == branch && !m.icon.is_empty())
                .map(|m| m.icon.clone())
        })
        .unwrap_or_default()
}

/// The title of a mission by id, or the id.
fn title_of(chain: &pbd_core::missions::Chain, id: &str) -> String {
    chain.get(id).map_or(id.to_string(), |m| m.title.clone())
}

/// The path's name in a line ("Eat and drink is done.").
fn path_name(branch: &str) -> Localized {
    tr(&format!("mission_path.{branch}.name"))
}

/// The heading's line for why the picker is up.
pub fn reason_line(chain: &pbd_core::missions::Chain, reason: Option<&Offer>) -> String {
    match reason {
        Some(Offer::FireLit) => tr("mission_path.fire_lit").into_string(),
        Some(Offer::PathDone(path)) => {
            tr_with("mission_path.done", &[("path", &path_name(path))]).into_string()
        }
        Some(Offer::PathWaits(path, on)) => tr_with(
            "mission_path.waits",
            &[
                ("path", &path_name(path)),
                ("mission", &title_of(chain, on)),
            ],
        )
        .into_string(),
        None => String::new(),
    }
}

/// Open the picker when it is offered, once the player is back in play (a
/// fire just lit, a path just run out), and with the Missions screen while
/// a path is to be picked; close it with the screen.
pub fn offer(
    mut missions: ResMut<Missions>,
    chain: Res<MissionChain>,
    mut screen: ResMut<Screen>,
    mut shown: ResMut<ShownTree>,
    mut picker: ResMut<Picker>,
    mut was: Local<Option<Screen>>,
) {
    let opened = *was != Some(*screen) && *screen == Screen::Missions;
    *was = Some(*screen);
    if missions.offer.is_some() && *screen == Screen::Playing {
        let reason = missions.offer.take();
        if missions.choosing(&chain.0) {
            *screen = Screen::Missions;
            *shown = ShownTree::Starter;
            *picker = Picker {
                shown: true,
                reason,
            };
            *was = Some(Screen::Missions);
            return;
        }
    }
    if *screen != Screen::Missions {
        if picker.shown {
            picker.shown = false;
        }
        return;
    }
    if opened && !picker.shown && missions.choosing(&chain.0) {
        *shown = ShownTree::Starter;
        *picker = Picker {
            shown: true,
            reason: None,
        };
    }
}

/// A card picks its path; Later closes the picker; Paths opens it.
#[allow(clippy::too_many_arguments)]
pub fn press(
    cards: Query<(&Interaction, &PathOf), Changed<Interaction>>,
    later: Query<&Interaction, (With<PickerLater>, Changed<Interaction>)>,
    paths: Query<&Interaction, (With<PathsButton>, Changed<Interaction>)>,
    chain: Res<MissionChain>,
    mut missions: ResMut<Missions>,
    mut picker: ResMut<Picker>,
    mut view: ResMut<TreeView>,
    mut shown: ResMut<ShownTree>,
) {
    if picker.shown {
        let picked = cards.iter().find_map(|(i, of)| match (i, &of.0) {
            (Interaction::Pressed, Some((branch, true))) => Some(branch.clone()),
            _ => None,
        });
        if let Some(branch) = picked {
            pick(&mut missions, &chain.0, &mut picker, &mut view, &branch);
            return;
        }
        if later.iter().any(|i| *i == Interaction::Pressed) {
            picker.shown = false;
        }
    } else if paths.iter().any(|i| *i == Interaction::Pressed) {
        *shown = ShownTree::Starter;
        *picker = Picker {
            shown: true,
            reason: None,
        };
    }
}

/// Take `branch` as the path: the picker closes and the tree opens again on
/// its Now.
fn pick(
    missions: &mut Missions,
    chain: &pbd_core::missions::Chain,
    picker: &mut Picker,
    view: &mut TreeView,
    branch: &str,
) {
    if missions.choose_path(chain, branch) {
        info!("path chosen: {branch}");
        picker.shown = false;
        picker.reason = None;
        view.placed = false;
        view.touched = false;
        view.selected = None;
    }
}

/// Keep the picker, its cards and the Paths plaque current.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    picker: Res<Picker>,
    missions: Res<Missions>,
    chain: Res<MissionChain>,
    screen: Res<Screen>,
    shown: Res<ShownTree>,
    kit: Res<UiKit>,
    assets: Res<AssetServer>,
    mut layer: Query<&mut Node, (With<PickerLayer>, Without<PathSlot>, Without<PathsButton>)>,
    mut paths_button: Query<&mut Node, (With<PathsButton>, Without<PathSlot>)>,
    mut sub: Query<&mut Text, (With<PickerSub>, Without<PathCount>, Without<PathLine>)>,
    mut slots: Query<(
        &PathSlot,
        &mut Node,
        &mut PathOf,
        &mut ImageNode,
        &Interaction,
    )>,
    mut icons: Query<(&PathIcon, &mut ImageNode), Without<PathSlot>>,
    mut names: Query<(&PathName, &mut KitText)>,
    mut texts: Query<
        (
            &mut Text,
            &mut TextColor,
            Option<&PathCount>,
            Option<&PathLine>,
            Option<&PathFirst>,
        ),
        (
            Without<PickerSub>,
            Or<(With<PathCount>, With<PathLine>, With<PathFirst>)>,
        ),
    >,
) {
    let chain = &chain.0;
    let cards: Vec<PathCard> = missions.progress.path_cards(chain, missions.mode);
    let on_tree = *screen == Screen::Missions && *shown == ShownTree::Starter;
    let display = |on: bool| if on { Display::Flex } else { Display::None };
    for mut node in &mut paths_button {
        let want = display(on_tree && !picker.shown && !cards.is_empty());
        if node.display != want {
            node.display = want;
        }
    }
    let up = on_tree && picker.shown;
    for mut node in &mut layer {
        let want = display(up);
        if node.display != want {
            node.display = want;
        }
    }
    if !up {
        return;
    }
    let line = reason_line(chain, picker.reason.as_ref());
    for mut text in &mut sub {
        if text.0 != line {
            text.0.clone_from(&line);
        }
    }
    for (slot, mut node, mut of, mut image, interaction) in &mut slots {
        let card = cards.get(slot.0);
        let want = display(card.is_some());
        if node.display != want {
            node.display = want;
        }
        let now = card.map(|c| (c.branch.clone(), c.first.is_some()));
        if of.0 != now {
            of.0 = now;
        }
        // Carved oak under the pointer, for a card that can be picked.
        let open = card.is_some_and(|c| c.first.is_some());
        let set = if open && *interaction != Interaction::None {
            Set::Oak
        } else {
            Set::Stone
        };
        let face = kit.nine(set, "tooltip");
        if image.image != face.image {
            *image = face;
        }
        image.color = if card.is_some() && !open {
            Color::srgba(1.0, 1.0, 1.0, 0.7)
        } else {
            Color::WHITE
        };
    }
    for (icon, mut image) in &mut icons {
        let Some(card) = cards.get(icon.0) else {
            continue;
        };
        let path = icon_of(chain, &card.branch);
        let want: Handle<Image> = if path.is_empty() {
            Handle::default()
        } else {
            assets.load(format!("items/{path}.png"))
        };
        if image.image != want {
            image.image = want;
        }
        image.color = if card.first.is_some() {
            Color::WHITE
        } else {
            Color::srgba(1.0, 1.0, 1.0, 0.45)
        };
    }
    for (name, mut words) in &mut names {
        let Some(card) = cards.get(name.0) else {
            continue;
        };
        let text = mission_tree::branch_name(&card.branch);
        let colour = if card.first.is_some() {
            ui_kit::TITLE
        } else {
            ui_kit::LABEL_DIM
        };
        if words.text != text || words.colour != colour {
            words.text = text;
            words.colour = colour;
        }
    }
    for (mut text, mut colour, count, line, first) in &mut texts {
        let (k, want, c) = if let Some(PathCount(k)) = count {
            let Some(card) = cards.get(*k) else { continue };
            (
                *k,
                tr_with(
                    "mission_path.count",
                    &[("done", &card.done), ("total", &card.total)],
                ),
                ui_kit::LABEL_DIM,
            )
        } else if let Some(PathLine(k)) = line {
            let Some(card) = cards.get(*k) else { continue };
            (
                *k,
                tr(&format!("mission_path.{}.line", card.branch)),
                if card.first.is_some() {
                    ui_kit::TEXT
                } else {
                    ui_kit::LABEL_DIM
                },
            )
        } else if let Some(PathFirst(k)) = first {
            let Some(card) = cards.get(*k) else { continue };
            match (&card.first, &card.waits_on) {
                (Some(id), _) => (
                    *k,
                    tr_with("mission_path.first", &[("mission", &title_of(chain, id))]),
                    ui_kit::RUNE,
                ),
                (None, Some(on)) => (
                    *k,
                    tr_with(
                        "mission_path.waits_on",
                        &[("mission", &title_of(chain, on))],
                    ),
                    ui_kit::LABEL_DIM,
                ),
                (None, None) => (*k, Localized::empty(), ui_kit::LABEL_DIM),
            }
        } else {
            continue;
        };
        let _ = k;
        let words = want.into_string();
        if text.0 != words {
            text.0 = words;
        }
        if colour.0 != c {
            colour.0 = c;
        }
    }
}

/// The chosen path's lane lit on the tree: its branch plaque ringed.
pub fn light_lane(
    missions: Res<Missions>,
    chain: Res<MissionChain>,
    shown: Res<ShownTree>,
    mut labels: Query<(&BranchLabel, &mut Outline)>,
) {
    let path = missions.progress.path.as_deref();
    for (label, mut outline) in &mut labels {
        let lit = *shown == ShownTree::Starter
            && path.is_some()
            && chain.0.missions.get(label.0).map(|m| m.branch.as_str()) == path;
        let want = if lit { ui_kit::TITLE } else { Color::NONE };
        if outline.color != want {
            outline.color = want;
        }
    }
}

/// `--pick-path <branch>`: once the picker is up, pick that path as a click
/// on its card would, then go back to play so the HUD's line shows it (a
/// capture instrument, `survival-mode` 6.5).
pub fn scripted_pick(
    launch: Res<super::Launch>,
    chain: Res<MissionChain>,
    mut missions: ResMut<Missions>,
    mut picker: ResMut<Picker>,
    mut view: ResMut<TreeView>,
    mut screen: ResMut<Screen>,
    mut wait: Local<u32>,
) {
    let Some(branch) = launch.pick_path.as_deref() else {
        return;
    };
    if !picker.shown || missions.progress.path.as_deref() == Some(branch) {
        return;
    }
    // A few frames with the picker up, as a player would see it.
    *wait += 1;
    if *wait < 30 {
        return;
    }
    pick(&mut missions, &chain.0, &mut picker, &mut view, branch);
    if !picker.shown {
        *screen = Screen::Playing;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every path's name and line, and the picker's words, are in the
    /// locale file.
    #[test]
    fn every_path_has_its_words() {
        let english = pbd_core::locale::Locale::english();
        let mut keys: Vec<String> = PATH_ORDER
            .iter()
            .flat_map(|b| {
                [
                    format!("mission_path.{b}.name"),
                    format!("mission_path.{b}.line"),
                ]
            })
            .collect();
        keys.extend(
            [
                "choose",
                "choose_hud",
                "fire_lit",
                "done",
                "waits",
                "waits_on",
                "first",
                "count",
                "later",
                "paths",
            ]
            .map(|k| format!("mission_path.{k}")),
        );
        for key in keys {
            assert!(english.get(&key).is_some(), "{key}");
        }
        for branch in PATH_ORDER {
            assert!(
                mission_tree::BRANCHES.iter().any(|(k, _)| *k == branch),
                "{branch} has no name on the tree"
            );
            let (_, icon) = PATH_ICONS.iter().find(|(b, _)| *b == branch).unwrap();
            let file = format!(
                "{}/../../assets/items/{icon}.png",
                env!("CARGO_MANIFEST_DIR")
            );
            assert!(std::path::Path::new(&file).exists(), "{file}");
        }
    }

    /// An app counting the shipped tree into a memory-only world of `mode`,
    /// with the picker's offer and the screen.
    fn app(mode: pbd_core::missions::Mode) -> App {
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
            .insert_resource(pbd_app::missions::ForcedMode(Some(mode)))
            .insert_resource(Screen::Playing)
            .insert_resource(ShownTree::Starter)
            .init_resource::<Picker>()
            .add_plugins(pbd_app::missions::MissionsPlugin)
            .add_systems(Update, offer);
        app.update();
        app
    }

    /// The script's steps up to and including the fire set down.
    fn to_the_fire(app: &mut App) {
        use super::super::missions::{Scripted, raise, script};
        for step in script() {
            let last = matches!(
                step,
                Scripted::Did(pbd_core::missions::Kind::Place, "block:campfire")
            );
            raise(step, app.world_mut());
            app.update();
            if last {
                break;
            }
        }
        // The offer is taken on the frame after the count.
        app.update();
    }

    /// 6.5: in a Survival world the Missions screen opens with the picker
    /// when the fire is done; picking a path makes its first open mission
    /// the goal and stores the path; the record reads it back.
    #[test]
    fn the_picker_opens_at_the_fire_in_survival() {
        let mut app = app(pbd_core::missions::Mode::Survival);
        let chain = &app.world().resource::<MissionChain>().0;
        assert!(
            chain.get("drink").is_some(),
            "a Survival world has its paths"
        );
        to_the_fire(&mut app);
        let missions = app.world().resource::<Missions>();
        assert!(missions.progress.is_done("fire"));
        assert_eq!(*app.world().resource::<Screen>(), Screen::Missions);
        let picker = app.world().resource::<Picker>();
        assert!(picker.shown);
        assert_eq!(picker.reason, Some(Offer::FireLit));
        // Hunting picked: its first open mission is the goal, and the record
        // holds the path once it is stored.
        let world = app.world_mut();
        world.resource_scope(|world, mut missions: Mut<Missions>| {
            let chain = &world.resource::<MissionChain>().0;
            assert!(missions.choose_path(chain, "hunting"));
            let now = missions.progress.focused(chain).unwrap();
            assert_eq!(now.branch, "hunting");
            assert!(!missions.choosing(chain));
        });
        app.update();
        let save = app.world().resource::<pbd_app::saves::WorldSave>();
        let stored = pbd_app::missions::progress_of(save);
        assert_eq!(stored.path.as_deref(), Some("hunting"));
    }

    /// 6.1: the missions follow the world's own mode, from its identity:
    /// a Survival world's tree has the paths, an older (Explorer) one not.
    #[test]
    fn the_tree_follows_the_worlds_mode() {
        let mut save = pbd_app::saves::WorldSave::memory_only();
        assert_eq!(
            pbd_app::missions::mode_of(&save),
            pbd_core::missions::Mode::Explorer
        );
        save.identity.mode = Some(pbd_core::survival::WorldMode::Survival);
        assert_eq!(
            pbd_app::missions::mode_of(&save),
            pbd_core::missions::Mode::Survival
        );
    }

    /// 6.5: an Explorer world never shows the picker, has no survival
    /// mission, and its Now is the first open one, as before.
    #[test]
    fn an_explorer_world_has_no_picker() {
        let mut app = app(pbd_core::missions::Mode::Explorer);
        to_the_fire(&mut app);
        let chain = &app.world().resource::<MissionChain>().0;
        assert!(chain.missions.iter().all(|m| m.mode.is_none()));
        let missions = app.world().resource::<Missions>();
        assert!(missions.progress.is_done("fire"));
        assert_eq!(missions.offer, None);
        assert!(!missions.choosing(chain));
        assert_eq!(*app.world().resource::<Screen>(), Screen::Playing);
        assert!(!app.world().resource::<Picker>().shown);
        assert_eq!(
            missions.progress.focused(chain).map(|m| m.id.as_str()),
            Some("storage")
        );
    }
}
