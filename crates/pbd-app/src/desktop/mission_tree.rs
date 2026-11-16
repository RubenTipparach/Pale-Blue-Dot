//! The Missions tab (O) drawn as the mission tree (`starter-missions` TS.2,
//! M33 version 3): a card per mission with its standing, the one line from
//! each mission to its parent, the branches' names, pan by drag and zoom by
//! the wheel, and a side panel for the card clicked.
//!
//! The layout is M33's: a mission is placed one column right of its parent;
//! a parent's children are placed in turn, its own branch's child first, so
//! a branch reads as one straight line and its side chains hang below; a
//! card sits on its first child's row. `after` gates are never drawn.

use super::menu::{self, Panel, Screen};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use pbd_app::missions::{MissionChain, Missions};
use pbd_app::ui_kit::{self, KitText, Set, UiKit};
use pbd_core::missions::{Chain, Standing};

/// A card's width and height, and the gaps between columns and rows, in
/// the UI's native pixels at a zoom of 1.
const CW: f32 = 132.0;
const CH: f32 = 96.0;
const GX: f32 = 46.0;
const GY: f32 = 40.0;
/// The margin round the tree, with room above a row for a branch's name.
const OX: f32 = 20.0;
const OY: f32 = 34.0;
/// The side panel's width.
const SIDE_W: f32 = 190.0;
/// A line's thickness.
const LINE: f32 = 2.0;

/// The branches in M33's order, with the names drawn over them.
pub const BRANCHES: [(&str, &str); 8] = [
    ("trunk", "HANDS TO TOOLS"),
    ("fishing", "FISHING"),
    ("farming", "FARMING"),
    ("herding", "HERDING"),
    ("mining", "MINING & SMELTING"),
    ("cooking", "COOKING"),
    ("light", "LIGHT & BUILDING"),
    ("about", "GETTING ABOUT"),
];

fn branch_rank(key: &str) -> usize {
    BRANCHES
        .iter()
        .position(|(k, _)| *k == key)
        .unwrap_or(BRANCHES.len())
}

fn branch_name(key: &str) -> &'static str {
    BRANCHES
        .iter()
        .find(|(k, _)| *k == key)
        .map_or("", |(_, n)| n)
}

/// Each mission's column and row, by its place in the chain.
pub fn layout(chain: &Chain) -> Vec<(usize, usize)> {
    let n = chain.missions.len();
    let index = |id: &str| chain.missions.iter().position(|m| m.id == id);
    let mut kids: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (i, m) in chain.missions.iter().enumerate() {
        if let Some(p) = m.needs.first().and_then(|p| index(p)) {
            kids[p].push(i);
        }
    }
    for (p, list) in kids.iter_mut().enumerate() {
        let own = chain.missions[p].branch.clone();
        list.sort_by_key(|&k| {
            let m = &chain.missions[k];
            (m.branch != own, branch_rank(&m.branch), k)
        });
    }
    let mut at = vec![(0, 0); n];
    let mut next = 0;
    fn place(
        i: usize,
        depth: usize,
        kids: &[Vec<usize>],
        at: &mut [(usize, usize)],
        next: &mut usize,
    ) {
        at[i].0 = depth;
        if kids[i].is_empty() {
            at[i].1 = *next;
            *next += 1;
            return;
        }
        for &k in &kids[i] {
            place(k, depth + 1, kids, at, next);
        }
        at[i].1 = at[kids[i][0]].1;
    }
    for (i, m) in chain.missions.iter().enumerate() {
        if m.needs.is_empty() {
            place(i, 0, &kids, &mut at, &mut next);
        }
    }
    at
}

fn card_x(col: usize) -> f32 {
    OX + col as f32 * (CW + GX)
}
fn card_y(row: usize) -> f32 {
    OY + row as f32 * (CH + GY)
}

/// Where the tree is looked at from, and which card is open in the side
/// panel. `pan` is where the tree's corner is on screen, native pixels.
#[derive(Resource, Clone, PartialEq, Debug)]
pub struct TreeView {
    pub pan: Vec2,
    pub zoom: f32,
    pub selected: Option<usize>,
    /// The view was placed on the current mission since the tab opened.
    pub placed: bool,
    /// The player moved the view since it opened: it no longer follows the
    /// current mission.
    pub touched: bool,
}

impl Default for TreeView {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            selected: None,
            placed: false,
            touched: false,
        }
    }
}

#[derive(Component)]
pub struct TreeViewport;
#[derive(Component)]
pub struct TreeWorld;
#[derive(Component)]
pub struct Card(pub usize);
#[derive(Component)]
pub struct CardTitle(pub usize);
#[derive(Component)]
pub struct CardSub(pub usize);
#[derive(Component)]
pub struct CardBadge(pub usize);
#[derive(Component)]
pub struct CardObjective(pub usize, pub usize);
#[derive(Component)]
pub struct CardCheck(pub usize, pub usize);
#[derive(Component)]
pub struct CardUnlocks(pub usize);
/// One segment of the line from a mission's parent to it.
#[derive(Component)]
pub struct Edge(pub usize);
#[derive(Component)]
pub struct SidePanel;
#[derive(Component)]
pub struct SideClose;
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SideText {
    Title,
    Sub,
    Objectives,
    Help,
    Unlocks,
    Needs,
    Next,
}

fn body(text: &str, size: f32, colour: Color) -> impl Bundle {
    menu::body(text.to_string(), size, colour)
}

pub fn spawn(mut commands: Commands, kit: Res<UiKit>, chain: Res<MissionChain>) {
    let kit = &*kit;
    let chain = &chain.0;
    let at = layout(chain);
    let cols = at.iter().map(|a| a.0).max().unwrap_or(0) + 1;
    let rows = at.iter().map(|a| a.1).max().unwrap_or(0) + 1;
    let width = card_x(cols) - GX + OX;
    let height = card_y(rows) - GY + OX;
    commands
        .spawn((
            menu::layer(),
            BackgroundColor(ui_kit::NIGHT),
            GlobalZIndex(11),
            Panel(Screen::Missions),
        ))
        .with_children(|layer| {
            layer
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        width: percent(100),
                        height: percent(100),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    Interaction::default(),
                    TreeViewport,
                ))
                .with_children(|viewport| {
                    viewport
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(0),
                                top: px(0),
                                width: px(width),
                                height: px(height),
                                ..default()
                            },
                            UiTransform::default(),
                            TreeWorld,
                        ))
                        .with_children(|world| {
                            spawn_edges(world, chain, &at);
                            spawn_labels(world, kit, chain, &at);
                            for (i, m) in chain.missions.iter().enumerate() {
                                spawn_card(world, kit, i, m, at[i]);
                            }
                        });
                });
            spawn_side(layer, kit);
        });
}

fn spawn_edges(world: &mut ChildSpawnerCommands, chain: &Chain, at: &[(usize, usize)]) {
    let index = |id: &str| chain.missions.iter().position(|m| m.id == id);
    for (i, m) in chain.missions.iter().enumerate() {
        let Some(p) = m.needs.first().and_then(|p| index(p)) else {
            continue;
        };
        let (x1, y1) = (card_x(at[p].0) + CW, card_y(at[p].1) + CH / 2.0);
        let (x2, y2) = (card_x(at[i].0), card_y(at[i].1) + CH / 2.0);
        let xm = x1 + GX / 2.0;
        let mut segments = vec![(x1, y1 - LINE / 2.0, xm - x1 + LINE / 2.0, LINE)];
        if (y2 - y1).abs() > 0.5 {
            let top = y1.min(y2) - LINE / 2.0;
            segments.push((xm - LINE / 2.0, top, LINE, (y2 - y1).abs() + LINE));
        }
        segments.push((xm - LINE / 2.0, y2 - LINE / 2.0, x2 - xm + LINE / 2.0, LINE));
        for (x, y, w, h) in segments {
            world.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(x),
                    top: px(y),
                    width: px(w),
                    height: px(h),
                    ..default()
                },
                BackgroundColor(ui_kit::STONE_MID),
                Edge(i),
            ));
        }
    }
}

fn spawn_labels(
    world: &mut ChildSpawnerCommands,
    kit: &UiKit,
    chain: &Chain,
    at: &[(usize, usize)],
) {
    for (n, (key, name)) in BRANCHES.iter().enumerate() {
        let Some(first) = chain.missions.iter().position(|m| m.branch == *key) else {
            continue;
        };
        world
            .spawn((
                kit.nine(Set::Oak, "button-idle"),
                Node {
                    position_type: PositionType::Absolute,
                    left: px(card_x(at[first].0)),
                    top: px(card_y(at[first].1) - 24.0),
                    padding: UiRect::new(px(8), px(8), px(4), px(3)),
                    ..default()
                },
            ))
            .with_children(|plaque| {
                plaque.spawn(KitText::new(format!("{n} · {name}"), ui_kit::OAK_LABEL));
            });
    }
}

fn spawn_card(
    world: &mut ChildSpawnerCommands,
    kit: &UiKit,
    i: usize,
    m: &pbd_core::missions::Mission,
    (col, row): (usize, usize),
) {
    world
        .spawn((
            Button,
            kit.nine(Set::Stone, "tooltip"),
            Node {
                position_type: PositionType::Absolute,
                left: px(card_x(col)),
                top: px(card_y(row)),
                width: px(CW),
                height: px(CH),
                padding: UiRect::new(px(7), px(7), px(5), px(5)),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                border: UiRect::all(px(1)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(Color::NONE),
            Card(i),
        ))
        .with_children(|card| {
            card.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                width: percent(100),
                ..default()
            })
            .with_children(|head| {
                head.spawn((body(&m.title, 10.0, ui_kit::TITLE), CardTitle(i)));
                head.spawn((
                    KitText::new("", ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                    Node {
                        margin: UiRect::left(px(3)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    CardBadge(i),
                ));
            });
            card.spawn((body("", 8.0, ui_kit::LABEL_DIM), CardSub(i)));
            for (k, objective) in m.objectives.iter().enumerate() {
                card.spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: px(3),
                    width: percent(100),
                    ..default()
                })
                .with_children(|line| {
                    line.spawn((
                        kit.whole(Set::Stone, "check-off"),
                        Node {
                            width: px(9),
                            height: px(9),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        CardCheck(i, k),
                    ));
                    line.spawn((
                        body(&objective.label, 9.0, ui_kit::TEXT),
                        CardObjective(i, k),
                    ));
                });
            }
            card.spawn((
                body("", 8.0, ui_kit::RUNE_DIM),
                Node {
                    margin: UiRect::top(px(2)),
                    ..default()
                },
                CardUnlocks(i),
            ));
        });
}

fn spawn_side(layer: &mut ChildSpawnerCommands, kit: &UiKit) {
    layer
        .spawn((
            kit.nine(Set::Stone, "frame"),
            Node {
                position_type: PositionType::Absolute,
                right: px(6),
                top: px(34),
                width: px(SIDE_W),
                max_height: percent(88),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                display: Display::None,
                overflow: Overflow::clip(),
                ..default()
            },
            SidePanel,
        ))
        .with_children(|side| {
            side.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                width: percent(100),
                ..default()
            })
            .with_children(|head| {
                head.spawn((body("", 13.0, ui_kit::TITLE), SideText::Title));
                head.spawn((ui_kit::plaque(kit, Set::Stone, 18.0), SideClose))
                    .with_children(|p| {
                        p.spawn(ui_kit::plaque_label(Set::Stone, "X"));
                    });
            });
            side.spawn((body("", 9.0, ui_kit::LABEL_DIM), SideText::Sub));
            for (heading, text) in [
                ("OBJECTIVES", SideText::Objectives),
                ("HELP", SideText::Help),
                ("UNLOCKS", SideText::Unlocks),
                ("NEEDS", SideText::Needs),
                ("OPENS NEXT", SideText::Next),
            ] {
                side.spawn((
                    KitText::new(heading, ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                    Node {
                        margin: UiRect::top(px(4)),
                        ..default()
                    },
                ));
                side.spawn((body("", 10.0, ui_kit::TEXT), text));
            }
        });
}

/// What a card's badge says, and its colour.
fn badge(standing: Standing, now: bool) -> (&'static str, Color) {
    match standing {
        Standing::Done => ("DONE", ui_kit::RUNE),
        Standing::Open if now => ("NOW", ui_kit::TITLE),
        Standing::Open => ("OPEN", ui_kit::TEXT),
        Standing::Locked => ("LOCKED", ui_kit::LABEL_DIM),
        Standing::Pending => ("NOT YET", ui_kit::LABEL_DIM),
    }
}

/// The recipe's name for an unlock, or its code.
fn recipe_name(recipes: &pbd_app::config::Recipes, code: &str) -> String {
    recipes
        .0
        .recipes
        .iter()
        .find(|r| r.code == code)
        .map_or(code.to_string(), |r| r.name.clone())
}

fn set_text(text: &mut Text, colour: &mut TextColor, words: String, c: Color) {
    if text.0 != words {
        text.0 = words;
    }
    if colour.0 != c {
        colour.0 = c;
    }
}

/// Keep the cards, the lines and the side panel current.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn paint(
    missions: Res<Missions>,
    chain: Res<MissionChain>,
    screen: Res<Screen>,
    view: Res<TreeView>,
    recipes: Option<Res<pbd_app::config::Recipes>>,
    kit: Res<UiKit>,
    mut cards: Query<(&Card, &mut BorderColor)>,
    mut texts: ParamSet<(
        Query<(&CardTitle, &mut Text, &mut TextColor)>,
        Query<(&CardSub, &mut Text, &mut TextColor)>,
        Query<(&CardObjective, &mut Text, &mut TextColor)>,
        Query<(&CardUnlocks, &mut Text, &mut TextColor)>,
        Query<(&SideText, &mut Text, &mut TextColor)>,
    )>,
    mut badges: Query<(&CardBadge, &mut KitText)>,
    mut checks: Query<(&CardCheck, &mut ImageNode)>,
    mut edges: Query<(&Edge, &mut BackgroundColor)>,
    mut side: Query<&mut Node, With<SidePanel>>,
) {
    if *screen != Screen::Missions
        || !(missions.is_changed() || screen.is_changed() || view.is_changed())
    {
        return;
    }
    let chain = &chain.0;
    let progress = &missions.progress;
    let current = progress.current(chain).map(|m| m.id.clone());
    let standing = |i: usize| {
        let m = &chain.missions[i];
        (
            progress.standing(chain, m),
            current.as_deref() == Some(m.id.as_str()),
        )
    };
    let title_of = |id: &str| chain.get(id).map_or(id.to_string(), |m| m.title.clone());
    for (card, mut border) in &mut cards {
        let (s, now) = standing(card.0);
        let c = if view.selected == Some(card.0) {
            ui_kit::LABEL
        } else if now {
            ui_kit::TITLE
        } else {
            Color::NONE
        };
        if border.top != c {
            *border = BorderColor::all(c);
        }
        let _ = s;
    }
    for (b, mut text) in &mut badges {
        let (s, now) = standing(b.0);
        let (word, colour) = badge(s, now);
        if text.text != word || text.colour != colour {
            text.text = word.to_string();
            text.colour = colour;
        }
    }
    for (t, mut text, mut colour) in &mut texts.p0() {
        let (s, now) = standing(t.0);
        let c = match s {
            Standing::Done => ui_kit::TEXT_DIM,
            Standing::Open if now => ui_kit::TITLE,
            Standing::Open => ui_kit::TEXT,
            _ => ui_kit::LABEL_DIM,
        };
        set_text(&mut text, &mut colour, chain.missions[t.0].title.clone(), c);
    }
    for (t, mut text, mut colour) in &mut texts.p1() {
        let m = &chain.missions[t.0];
        let (s, _) = standing(t.0);
        let words = if s == Standing::Locked {
            let mut gates: Vec<String> = m.needs.iter().map(|id| title_of(id)).collect();
            gates.extend(m.after.iter().map(|id| title_of(id)));
            format!("after {}", gates.join(" + "))
        } else {
            branch_name(&m.branch).to_string()
        };
        set_text(&mut text, &mut colour, words, ui_kit::LABEL_DIM);
    }
    for (o, mut text, mut colour) in &mut texts.p2() {
        let m = &chain.missions[o.0];
        let (s, _) = standing(o.0);
        let objective = &m.objectives[o.1];
        let n = if s == Standing::Done {
            objective.count
        } else {
            progress.counts(m).get(o.1).copied().unwrap_or(0)
        };
        let c = if s == Standing::Locked || s == Standing::Pending {
            ui_kit::LABEL_DIM
        } else {
            ui_kit::TEXT
        };
        set_text(
            &mut text,
            &mut colour,
            format!("{} {n}/{}", objective.label, objective.count),
            c,
        );
    }
    for (u, mut text, mut colour) in &mut texts.p3() {
        let m = &chain.missions[u.0];
        let words = if m.unlocks.is_empty() {
            String::new()
        } else {
            let names: Vec<String> = m
                .unlocks
                .iter()
                .map(|code| {
                    recipes
                        .as_deref()
                        .map_or(code.clone(), |r| recipe_name(r, code))
                })
                .collect();
            format!("UNLOCKS {}", names.join(", "))
        };
        let (s, _) = standing(u.0);
        let c = if s == Standing::Done {
            ui_kit::RUNE
        } else {
            ui_kit::RUNE_DIM
        };
        set_text(&mut text, &mut colour, words, c);
    }
    for (c, mut image) in &mut checks {
        let m = &chain.missions[c.0];
        let (s, _) = standing(c.0);
        let full = s == Standing::Done
            || progress.counts(m).get(c.1).copied().unwrap_or(0) >= m.objectives[c.1].count;
        let want = kit.whole(Set::Stone, if full { "check-on" } else { "check-off" });
        if image.image != want.image {
            image.image = want.image;
        }
    }
    let index = |id: &str| chain.missions.iter().position(|m| m.id == id);
    for (e, mut colour) in &mut edges {
        let m = &chain.missions[e.0];
        let parent_done = m.needs.first().is_some_and(|p| progress.is_done(p));
        let (s, _) = standing(e.0);
        let lit = view
            .selected
            .is_some_and(|sel| sel == e.0 || m.needs.first().and_then(|p| index(p)) == Some(sel));
        let c = if lit {
            ui_kit::LABEL
        } else if parent_done && s == Standing::Done {
            ui_kit::RUNE_DIM
        } else if parent_done {
            ui_kit::TITLE
        } else {
            ui_kit::STONE_MID.with_alpha(0.6)
        };
        if colour.0 != c {
            colour.0 = c;
        }
    }
    // The side panel.
    let display = if view.selected.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut side {
        if node.display != display {
            node.display = display;
        }
    }
    let Some(sel) = view.selected else {
        return;
    };
    let m = &chain.missions[sel];
    let (s, now) = standing(sel);
    let counts = progress.counts(m);
    for (which, mut text, mut colour) in &mut texts.p4() {
        let (words, c) = match which {
            SideText::Title => (m.title.clone(), ui_kit::TITLE),
            SideText::Sub => (
                format!("{} · {}", branch_name(&m.branch), badge(s, now).0),
                ui_kit::LABEL_DIM,
            ),
            SideText::Objectives => (
                m.objectives
                    .iter()
                    .enumerate()
                    .map(|(k, o)| {
                        let n = if s == Standing::Done {
                            o.count
                        } else {
                            counts.get(k).copied().unwrap_or(0)
                        };
                        let tick = if n >= o.count { "[x]" } else { "[ ]" };
                        format!("{tick} {} {n}/{}", o.label, o.count)
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                ui_kit::TEXT,
            ),
            SideText::Help => (m.help.clone(), ui_kit::TEXT_DIM),
            SideText::Unlocks => (
                if m.unlocks.is_empty() {
                    "-".to_string()
                } else {
                    m.unlocks
                        .iter()
                        .map(|code| {
                            recipes
                                .as_deref()
                                .map_or(code.clone(), |r| recipe_name(r, code))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                },
                ui_kit::RUNE,
            ),
            SideText::Needs => {
                let mut gates: Vec<String> = m.needs.iter().map(|id| title_of(id)).collect();
                gates.extend(m.after.iter().map(|id| title_of(id)));
                (
                    if gates.is_empty() {
                        "-".to_string()
                    } else {
                        gates.join(", ")
                    },
                    ui_kit::TEXT,
                )
            }
            SideText::Next => {
                let next: Vec<String> = chain
                    .missions
                    .iter()
                    .filter(|c| c.needs.contains(&m.id))
                    .map(|c| c.title.clone())
                    .collect();
                (
                    if next.is_empty() {
                        "-".to_string()
                    } else {
                        next.join(", ")
                    },
                    ui_kit::TEXT,
                )
            }
        };
        set_text(&mut text, &mut colour, words, c);
    }
}

/// The zoom the tab opens at.
const OPEN_ZOOM: f32 = 0.75;
/// The view's zoom limits.
const ZOOM: (f32, f32) = (0.2, 2.0);

/// Pan by dragging, zoom by the wheel about the pointer, a click on a card
/// opens it in the side panel, the ✕ or Esc-free click on the ground
/// closes it; the view opens on the current mission.
#[allow(clippy::too_many_arguments)]
pub fn steer(
    screen: Res<Screen>,
    missions: Res<Missions>,
    chain: Res<MissionChain>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Option<Res<UiScale>>,
    viewport: Query<&Interaction, With<TreeViewport>>,
    cards: Query<(&Card, &Interaction)>,
    close: Query<&Interaction, (With<SideClose>, Changed<Interaction>)>,
    mut view: ResMut<TreeView>,
    launch: Res<super::Launch>,
    keys: Res<ButtonInput<KeyCode>>,
    mut press: Local<Option<(Vec2, Option<usize>, bool)>>,
) {
    if *screen != Screen::Missions {
        wheel.clear();
        *press = None;
        if view.placed {
            view.placed = false;
            view.touched = false;
        }
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let scale = ui_scale.map_or(1.0, |s| s.0);
    let size = Vec2::new(window.width(), window.height()) / scale;
    let pointer = window.cursor_position().map(|p| p / scale);
    let mut next = view.clone();
    if !next.placed || (!next.touched && missions.is_changed()) {
        // Open on the current mission, until the player moves the view.
        let at = layout(&chain.0);
        let current = missions
            .progress
            .current(&chain.0)
            .and_then(|m| chain.0.missions.iter().position(|n| n.id == m.id))
            .unwrap_or(0);
        let (col, row) = at[current];
        next.zoom = OPEN_ZOOM;
        next.pan =
            size / 2.0 - Vec2::new(card_x(col) + CW / 2.0, card_y(row) + CH / 2.0) * OPEN_ZOOM;
        if !next.placed
            && let Some(id) = launch.tree_select.as_deref()
        {
            next.selected = chain.0.missions.iter().position(|m| m.id == id);
            if id == "fit" {
                next = fitted(&chain.0, size, next);
                next.touched = true;
            }
        }
        next.placed = true;
    }
    let over = viewport.iter().any(|i| *i != Interaction::None);
    for event in wheel.read() {
        let steps = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        };
        if over && let Some(at) = pointer {
            next.touched = true;
            let zoom = (next.zoom * 1.2f32.powf(steps)).clamp(ZOOM.0, ZOOM.1);
            // The point under the pointer stays under it.
            next.pan = at - (at - next.pan) * (zoom / next.zoom);
            next.zoom = zoom;
        }
    }
    let on_card = cards
        .iter()
        .find(|(_, i)| **i != Interaction::None)
        .map(|(c, _)| c.0);
    if mouse.just_pressed(MouseButton::Left)
        && (over || on_card.is_some())
        && let Some(at) = pointer
    {
        *press = Some((at, on_card, false));
    }
    if let (Some((from, card, moved)), Some(at)) = (press.as_mut(), pointer)
        && mouse.pressed(MouseButton::Left)
    {
        let d = at - *from;
        if *moved || d.length() > 3.0 {
            *moved = true;
            next.touched = true;
            next.pan += d;
            *from = at;
        }
        let _ = card;
    }
    if mouse.just_released(MouseButton::Left)
        && let Some((_, card, moved)) = press.take()
        && !moved
    {
        next.selected = card;
    }
    // F fits the whole tree to the screen.
    if keys.just_pressed(KeyCode::KeyF) {
        next = fitted(&chain.0, size, next);
        next.touched = true;
    }
    if close.iter().any(|i| *i == Interaction::Pressed) {
        next.selected = None;
        *press = None;
    }
    if next != *view {
        *view = next;
    }
}

/// The whole tree on screen, under the tab strip.
fn fitted(chain: &Chain, size: Vec2, mut view: TreeView) -> TreeView {
    let at = layout(chain);
    let cols = at.iter().map(|a| a.0).max().unwrap_or(0) + 1;
    let rows = at.iter().map(|a| a.1).max().unwrap_or(0) + 1;
    let tree = Vec2::new(card_x(cols) - GX + OX, card_y(rows) - GY + OX);
    let room = size - Vec2::new(0.0, 30.0);
    view.zoom = (room.x / tree.x).min(room.y / tree.y).clamp(ZOOM.0, ZOOM.1);
    view.pan = Vec2::new((size.x - tree.x * view.zoom) / 2.0, 30.0);
    view
}

/// Move and scale the tree to the view.
pub fn place(
    view: Res<TreeView>,
    mut world: Query<(&ComputedNode, &mut UiTransform), With<TreeWorld>>,
) {
    for (node, mut transform) in &mut world {
        // `UiTransform` scales about the node's centre: put the corner at
        // `pan`.
        let half = node.size() * node.inverse_scale_factor() / 2.0;
        let t = view.pan + half * view.zoom - half;
        let want = UiTransform {
            translation: Val2::px(t.x, t.y),
            scale: Vec2::splat(view.zoom),
            ..default()
        };
        if *transform != want {
            *transform = want;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped() -> Chain {
        let (recipes, _) = pbd_app::config::recipe_book();
        let file: pbd_core::missions::MissionFile =
            ron::from_str(include_str!("../../../../assets/config/missions.ron")).unwrap();
        let fish: Vec<String> = Vec::new();
        let catalog = pbd_core::missions::Catalog::new(
            &recipes.0,
            fish.iter().map(String::as_str),
            ["yard hen", "woolback", "plodder"],
        );
        file.resolve(&catalog).unwrap()
    }

    /// M33's layout of the shipped tree: no two cards on one place, a child
    /// one column right of its parent, a parent on its first child's row,
    /// and the trunk on one row.
    #[test]
    fn the_tree_is_laid_out_as_m33_lays_it_out() {
        let chain = shipped();
        let at = layout(&chain);
        let mut seen = std::collections::BTreeSet::new();
        for (i, m) in chain.missions.iter().enumerate() {
            assert!(seen.insert(at[i]), "{} on another card", m.id);
            if let Some(p) = m.needs.first() {
                let p = chain.missions.iter().position(|n| &n.id == p).unwrap();
                assert_eq!(at[i].0, at[p].0 + 1, "{}", m.id);
            }
        }
        let trunk: Vec<usize> = chain
            .missions
            .iter()
            .enumerate()
            .filter(|(_, m)| m.branch == "trunk")
            .map(|(i, _)| at[i].1)
            .collect();
        assert!(trunk.windows(2).all(|w| w[0] == w[1]), "{trunk:?}");
        // Every mission names a branch the tab draws.
        for m in &chain.missions {
            assert!(branch_rank(&m.branch) < BRANCHES.len(), "{}", m.id);
        }
    }
}
