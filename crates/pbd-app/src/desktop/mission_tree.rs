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

/// One zoom level of the tree: a designed view, not a scale. The cards are
/// always drawn at the UI kit's own scale (the tree's `UiTransform` scale is
/// 1, so a 9-slice frame is never resampled and never tears); a level sets
/// the card's size and detail, the gaps between cards and the type size, all
/// in whole UI pixels (owner, 2026-11-16: "I would opt to virtualize them,
/// so that way we still control how we want to zoom in").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Level {
    /// A card's width and height.
    pub cw: f32,
    pub ch: f32,
    /// The gaps between columns and rows (even, so a line's elbow sits on a
    /// whole pixel).
    pub gx: f32,
    pub gy: f32,
    /// The card's title and badge; without them a card is its frame only,
    /// the overview that fits the whole tree on a screen.
    pub text: bool,
    /// The whole card (objectives and unlocks), or its title and badge only.
    pub full: bool,
    /// The branch plaques drawn over each branch's first card.
    pub labels: bool,
    /// Type sizes: the title, the branch line, an objective, the unlocks.
    pub title: f32,
    pub sub: f32,
    pub line: f32,
    /// The badge's kit-text multiple (a whole number of art pixels).
    pub badge: u32,
    /// The card's padding, left and right, top and bottom.
    pub pad: (f32, f32),
    /// The mission's icon beside the title, px; nought where the card is
    /// its icon instead.
    pub icon: f32,
    /// The icon a card is at this level, px, filling its frame; nought on
    /// a full card.
    pub big: f32,
    /// An objective's tick, px: the kit's check at a whole multiple.
    pub check: f32,
}

/// The levels, farthest first (`starter-missions` TS.4, M33's look): one
/// card shape at every zoom, each level M33's scale (0.1 to 2) of the
/// native card (132 x 96) laid out at twice the native gaps (92 x 80), in
/// whole UI pixels. At 0.6 and below a card is its mission's icon filling
/// the frame; hovering one shows the full card (`TipCard`).
pub const LEVELS: [Level; 10] = [
    // x0.1
    Level {
        cw: 13.0,
        ch: 10.0,
        gx: 10.0,
        gy: 8.0,
        text: false,
        full: false,
        labels: false,
        title: 6.0,
        sub: 5.0,
        line: 6.0,
        badge: 1,
        pad: (1.0, 0.0),
        icon: 0.0,
        big: 8.0,
        check: 12.0,
    },
    // x0.14
    Level {
        cw: 18.0,
        ch: 12.0,
        gx: 12.0,
        gy: 12.0,
        text: false,
        full: false,
        labels: false,
        title: 6.0,
        sub: 5.0,
        line: 6.0,
        badge: 1,
        pad: (1.0, 1.0),
        icon: 0.0,
        big: 8.0,
        check: 12.0,
    },
    // x0.2
    Level {
        cw: 26.0,
        ch: 20.0,
        gx: 18.0,
        gy: 16.0,
        text: false,
        full: false,
        labels: false,
        title: 6.0,
        sub: 5.0,
        line: 6.0,
        badge: 1,
        pad: (1.0, 1.0),
        icon: 0.0,
        big: 8.0,
        check: 12.0,
    },
    // x0.3
    Level {
        cw: 40.0,
        ch: 28.0,
        gx: 28.0,
        gy: 24.0,
        text: false,
        full: false,
        labels: false,
        title: 6.0,
        sub: 5.0,
        line: 6.0,
        badge: 1,
        pad: (2.0, 2.0),
        icon: 0.0,
        big: 16.0,
        check: 12.0,
    },
    // x0.42
    Level {
        cw: 55.0,
        ch: 40.0,
        gx: 38.0,
        gy: 34.0,
        text: false,
        full: false,
        labels: false,
        title: 6.0,
        sub: 5.0,
        line: 6.0,
        badge: 1,
        pad: (3.0, 2.0),
        icon: 0.0,
        big: 16.0,
        check: 12.0,
    },
    // x0.6
    Level {
        cw: 79.0,
        ch: 58.0,
        gx: 56.0,
        gy: 48.0,
        text: false,
        full: false,
        labels: false,
        title: 6.0,
        sub: 5.0,
        line: 6.0,
        badge: 1,
        pad: (4.0, 3.0),
        icon: 0.0,
        big: 32.0,
        check: 12.0,
    },
    // x0.8
    Level {
        cw: 106.0,
        ch: 76.0,
        gx: 74.0,
        gy: 64.0,
        text: true,
        full: true,
        labels: true,
        title: 8.0,
        sub: 6.0,
        line: 7.0,
        badge: 1,
        pad: (6.0, 4.0),
        icon: 16.0,
        big: 0.0,
        check: 12.0,
    },
    // x1.0
    Level {
        cw: 132.0,
        ch: 96.0,
        gx: 92.0,
        gy: 80.0,
        text: true,
        full: true,
        labels: true,
        title: 10.0,
        sub: 8.0,
        line: 9.0,
        badge: 1,
        pad: (7.0, 5.0),
        icon: 16.0,
        big: 0.0,
        check: 12.0,
    },
    // x1.4
    Level {
        cw: 185.0,
        ch: 134.0,
        gx: 128.0,
        gy: 112.0,
        text: true,
        full: true,
        labels: true,
        title: 14.0,
        sub: 11.0,
        line: 13.0,
        badge: 1,
        pad: (10.0, 7.0),
        icon: 16.0,
        big: 0.0,
        check: 12.0,
    },
    // x2.0
    Level {
        cw: 264.0,
        ch: 192.0,
        gx: 184.0,
        gy: 160.0,
        text: true,
        full: true,
        labels: true,
        title: 20.0,
        sub: 16.0,
        line: 18.0,
        badge: 2,
        pad: (14.0, 10.0),
        icon: 32.0,
        big: 0.0,
        check: 24.0,
    },
];
/// The level the tab opens at: the native card, x1.
pub const OPEN_LEVEL: usize = 7;
/// The margin round the tree, with room above a row for a branch's name
/// where the level draws the names.
const OX: f32 = 20.0;
const OY: f32 = 34.0;
const OY_BARE: f32 = 12.0;
/// The side panel's width.
const SIDE_W: f32 = 190.0;
/// A line's thickness.
const LINE: f32 = 2.0;

impl Level {
    /// Where the tree's first card sits.
    pub fn origin(&self) -> Vec2 {
        Vec2::new(OX, if self.labels { OY } else { OY_BARE })
    }
    /// From one card to the next, across and down.
    pub fn pitch(&self) -> Vec2 {
        Vec2::new(self.cw + self.gx, self.ch + self.gy)
    }
    /// A card's top left corner.
    pub fn card(&self, (col, row): (usize, usize)) -> Vec2 {
        self.origin() + Vec2::new(col as f32, row as f32) * self.pitch()
    }
    /// The whole tree's size, `cols` by `rows` cards.
    pub fn size(&self, cols: usize, rows: usize) -> Vec2 {
        self.card((cols, rows)) - Vec2::new(self.gx, self.gy) + Vec2::splat(OX)
    }
}

/// The tree's columns and rows.
fn extent(at: &[(usize, usize)]) -> (usize, usize) {
    (
        at.iter().map(|a| a.0).max().unwrap_or(0) + 1,
        at.iter().map(|a| a.1).max().unwrap_or(0) + 1,
    )
}

/// The three segments of the line from a parent's card to its child's:
/// out of the parent's side, along the gap, into the child's side. `None`
/// for the middle one when the two share a row.
fn edge_segments(lv: &Level, parent: (usize, usize), child: (usize, usize)) -> [Option<Rect>; 3] {
    let (p, c) = (lv.card(parent), lv.card(child));
    let (x1, y1) = (p.x + lv.cw, p.y + lv.ch / 2.0);
    let (x2, y2) = (c.x, c.y + lv.ch / 2.0);
    let xm = x1 + lv.gx / 2.0;
    let h = LINE / 2.0;
    let r = |x: f32, y: f32, w: f32, hh: f32| {
        Rect::from_corners(Vec2::new(x, y), Vec2::new(x + w, y + hh))
    };
    [
        Some(r(x1, y1 - h, xm - x1 + h, LINE)),
        ((y2 - y1).abs() > 0.5).then(|| r(xm - h, y1.min(y2) - h, LINE, (y2 - y1).abs() + LINE)),
        Some(r(xm - h, y2 - h, x2 - xm + h, LINE)),
    ]
}

/// A point snapped to whole physical pixels, `k` of them a UI pixel.
pub fn snap(p: Vec2, k: f32) -> Vec2 {
    if k <= 0.0 { p } else { (p * k).round() / k }
}

/// The pan that keeps the tree point under `at` under it from one level to
/// another, by its place between the cards.
pub fn rezoom(pan: Vec2, at: Vec2, from: &Level, to: &Level) -> Vec2 {
    let u = (at - pan - from.origin()) / from.pitch();
    at - to.origin() - u * to.pitch()
}

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

/// Where the tree is looked at from, and which card is open in the side
/// panel. `pan` is where the tree's corner is on screen, UI pixels, snapped
/// to whole physical pixels; `level` indexes `LEVELS`.
#[derive(Resource, Clone, PartialEq, Debug)]
pub struct TreeView {
    pub pan: Vec2,
    pub level: usize,
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
            level: OPEN_LEVEL,
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
/// Which of an edge's three segments.
#[derive(Component)]
pub struct EdgeSeg(pub usize);
/// A branch's plaque, over the card of the mission at this index.
#[derive(Component)]
pub struct BranchLabel(pub usize);
/// The mission's icon beside its title on a full card.
#[derive(Component)]
pub struct CardIcon;
/// The mission's icon filling the card zoomed out (`TS.4`).
#[derive(Component)]
pub struct CardBig;
/// A part of a hover card: the full card shown beside the pointer over an
/// icon tile, laid out once at the native level and never by `relayout`.
#[derive(Component)]
pub struct TipPart;
/// A hover card's root, for the mission at this index.
#[derive(Component)]
pub struct TipCard(pub usize);
/// A part of a card a collapsed card hides: its branch line, objectives
/// and unlocks.
#[derive(Component)]
pub struct CardDetail;
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

pub fn spawn(
    mut commands: Commands,
    kit: Res<UiKit>,
    chain: Res<MissionChain>,
    assets: Res<AssetServer>,
) {
    let kit = &*kit;
    let assets = &*assets;
    let chain = &chain.0;
    // `relayout` sizes and places every part for the view's level.
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
                                ..default()
                            },
                            UiTransform::default(),
                            TreeWorld,
                        ))
                        .with_children(|world| {
                            spawn_edges(world, chain);
                            spawn_labels(world, kit, chain);
                            for (i, m) in chain.missions.iter().enumerate() {
                                spawn_card(world, kit, assets, i, m, false);
                            }
                        });
                });
            spawn_side(layer, kit);
            // The hover cards: each mission's full card at the native
            // level, hidden until its tile is hovered zoomed out.
            for (i, m) in chain.missions.iter().enumerate() {
                spawn_card(layer, kit, assets, i, m, true);
            }
        });
}

fn spawn_edges(world: &mut ChildSpawnerCommands, chain: &Chain) {
    for (i, m) in chain.missions.iter().enumerate() {
        if m.needs.is_empty() {
            continue;
        }
        for k in 0..3 {
            world.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(ui_kit::STONE_MID),
                Edge(i),
                EdgeSeg(k),
            ));
        }
    }
}

fn spawn_labels(world: &mut ChildSpawnerCommands, kit: &UiKit, chain: &Chain) {
    for (n, (key, name)) in BRANCHES.iter().enumerate() {
        let Some(first) = chain.missions.iter().position(|m| m.branch == *key) else {
            continue;
        };
        world
            .spawn((
                kit.nine(Set::Oak, "button-idle"),
                Node {
                    position_type: PositionType::Absolute,
                    padding: UiRect::new(px(8), px(8), px(4), px(3)),
                    ..default()
                },
                BranchLabel(first),
            ))
            .with_children(|plaque| {
                plaque.spawn(KitText::new(format!("{n} · {name}"), ui_kit::OAK_LABEL));
            });
    }
}

/// A card for mission `i`: in the tree (`tip` false), or its hover card
/// beside the pointer (`tip` true), every part of which carries [`TipPart`]
/// so `relayout` leaves it at the native level.
fn spawn_card(
    world: &mut ChildSpawnerCommands,
    kit: &UiKit,
    assets: &AssetServer,
    i: usize,
    m: &pbd_core::missions::Mission,
    tip: bool,
) {
    let native = &LEVELS[OPEN_LEVEL];
    let icon = (!m.icon.is_empty()).then(|| assets.load(format!("items/{}.png", m.icon)));
    let mut root = world.spawn((
        kit.nine(Set::Stone, "tooltip"),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            border: UiRect::all(px(1)),
            overflow: Overflow::clip(),
            ..default()
        },
        BorderColor::all(Color::NONE),
        Card(i),
    ));
    if tip {
        root.insert((
            TipPart,
            TipCard(i),
            GlobalZIndex(30),
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                border: UiRect::all(px(1)),
                overflow: Overflow::clip(),
                width: px(native.cw),
                height: px(native.ch),
                padding: UiRect::axes(px(native.pad.0), px(native.pad.1)),
                display: Display::None,
                ..default()
            },
        ));
    } else {
        root.insert(Button);
    }
    let part = move |e: &mut EntityCommands| {
        if tip {
            e.insert(TipPart);
        }
    };
    root.with_children(|card| {
        // Zoomed out, the card is its icon, centred in the frame.
        if !tip {
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..default()
                },
                CardBig,
            ))
            .with_children(|big| {
                if let Some(icon) = icon.clone() {
                    big.spawn((ImageNode::new(icon), Node::default(), CardBig));
                }
            });
        }
        let mut head = card.spawn(Node {
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::FlexStart,
            width: percent(100),
            ..default()
        });
        part(&mut head);
        head.with_children(|head| {
            let mut row = head.spawn(Node {
                column_gap: px(4),
                align_items: AlignItems::Center,
                ..default()
            });
            part(&mut row);
            row.with_children(|row| {
                if let Some(icon) = icon.clone() {
                    let mut e = row.spawn((
                        ImageNode::new(icon),
                        Node {
                            width: px(native.icon),
                            height: px(native.icon),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        CardIcon,
                    ));
                    part(&mut e);
                }
                let mut e = row.spawn((body(&m.title, native.title, ui_kit::TITLE), CardTitle(i)));
                part(&mut e);
            });
            let mut e = head.spawn((
                KitText::new("", ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                Node {
                    margin: UiRect::left(px(3)),
                    flex_shrink: 0.0,
                    ..default()
                },
                CardBadge(i),
            ));
            part(&mut e);
        });
        let mut e = card.spawn((
            body("", native.sub, ui_kit::LABEL_DIM),
            CardSub(i),
            CardDetail,
        ));
        part(&mut e);
        for (k, objective) in m.objectives.iter().enumerate() {
            let mut line = card.spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(3),
                    width: percent(100),
                    ..default()
                },
                CardDetail,
            ));
            part(&mut line);
            line.with_children(|line| {
                let mut e = line.spawn((
                    kit.whole(Set::Stone, "check-off"),
                    Node {
                        width: px(native.check),
                        height: px(native.check),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    CardCheck(i, k),
                ));
                part(&mut e);
                let mut e = line.spawn((
                    body(&objective.label, native.line, ui_kit::TEXT),
                    CardObjective(i, k),
                ));
                part(&mut e);
            });
        }
        let mut e = card.spawn((
            body("", native.sub, ui_kit::RUNE_DIM),
            Node {
                margin: UiRect::top(px(2)),
                ..default()
            },
            CardUnlocks(i),
            CardDetail,
        ));
        part(&mut e);
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

/// What a card's badge says, and its colour. A locked card has none: its dim
/// silhouettes say so (owner, 2026-11-16: "we dont need locked, its obvious").
fn badge(standing: Standing, now: bool) -> (&'static str, Color) {
    match standing {
        Standing::Done => ("DONE", ui_kit::RUNE),
        Standing::Open if now => ("NOW", ui_kit::TITLE),
        Standing::Open => ("OPEN", ui_kit::TEXT),
        Standing::Locked => ("", ui_kit::LABEL_DIM),
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
                match badge(s, now).0 {
                    "" => branch_name(&m.branch).to_string(),
                    word => format!("{} · {}", branch_name(&m.branch), word),
                },
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

/// How far a pixel-unit wheel must turn for one level.
const WHEEL_PIXELS: f32 = 40.0;

/// The pan that puts a card at the middle of the screen.
fn centred(lv: &Level, at: (usize, usize), size: Vec2) -> Vec2 {
    size / 2.0 - (lv.card(at) + Vec2::new(lv.cw, lv.ch) / 2.0)
}

/// Pan by dragging, step through the levels by the wheel about the pointer,
/// a click on a card opens it in the side panel, the ✕ or a click on the
/// ground closes it; the view opens on the current mission.
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
    mut spin: Local<f32>,
) {
    if *screen != Screen::Missions {
        wheel.clear();
        *press = None;
        *spin = 0.0;
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
    // Physical pixels a UI pixel: the kit's whole art scale.
    let k = scale * window.scale_factor();
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
        if !next.placed
            && let Some(id) = launch.tree_select.as_deref()
        {
            next.selected = chain.0.missions.iter().position(|m| m.id == id);
        }
        next.level = launch.tree_zoom.unwrap_or(OPEN_LEVEL).min(LEVELS.len() - 1);
        next.pan = centred(
            &LEVELS[next.level],
            at[next.selected.unwrap_or(current)],
            size,
        );
        if !next.placed && launch.tree_select.as_deref() == Some("fit") {
            next = fitted(&chain.0, size, next);
            next.touched = true;
        }
        next.placed = true;
    }
    let over = viewport.iter().any(|i| *i != Interaction::None);
    for event in wheel.read() {
        let steps = match event.unit {
            MouseScrollUnit::Line => event.y.signum() * event.y.abs().round().max(1.0),
            MouseScrollUnit::Pixel => {
                *spin += event.y;
                let whole = (*spin / WHEEL_PIXELS).trunc();
                *spin -= whole * WHEEL_PIXELS;
                whole
            }
        };
        if steps == 0.0 || !over {
            continue;
        }
        if let Some(at) = pointer {
            let to = (next.level as i32 + steps as i32).clamp(0, LEVELS.len() as i32 - 1) as usize;
            if to != next.level {
                next.touched = true;
                // The point under the pointer stays under it.
                next.pan = rezoom(next.pan, at, &LEVELS[next.level], &LEVELS[to]);
                next.level = to;
            }
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
    if let (Some((from, _, moved)), Some(at)) = (press.as_mut(), pointer)
        && mouse.pressed(MouseButton::Left)
    {
        let d = at - *from;
        if *moved || d.length() > 3.0 {
            *moved = true;
            next.touched = true;
            next.pan += d;
            *from = at;
        }
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
    // Bounds (M33, owner 2026-11-16: "a max bounds so the user doesnt get
    // lost"): the tree may slide only so far that some card keeps 40 px in
    // view.
    next.pan = bounded(next.pan, &LEVELS[next.level], &layout(&chain.0), size);
    next.pan = snap(next.pan, k);
    if next != *view {
        *view = next;
    }
}

/// How much of a card must stay in view, UI px.
const KEEP: f32 = 40.0;

/// The pan moved back, if need be, so that some card keeps [`KEEP`] px on a
/// screen of `size` (M33's bounds): the view's middle is pulled toward the
/// nearest card until it does.
pub fn bounded(pan: Vec2, lv: &Level, at: &[(usize, usize)], size: Vec2) -> Vec2 {
    let mut c = size / 2.0 - pan;
    let half = (size / 2.0 - Vec2::splat(KEEP)).max(Vec2::ZERO);
    let mut best: Option<(f32, Vec2)> = None;
    for &cell in at {
        let p = lv.card(cell);
        let dx = (p.x - c.x).max(0.0).max(c.x - (p.x + lv.cw));
        let dy = (p.y - c.y).max(0.0).max(c.y - (p.y + lv.ch));
        let over = (dx - half.x).max(0.0) + (dy - half.y).max(0.0);
        if best.is_none_or(|(b, _)| over < b) {
            best = Some((over, p));
        }
        if over == 0.0 {
            return pan;
        }
    }
    let Some((_, p)) = best else {
        return pan;
    };
    c.x = c.x.clamp(p.x - half.x, p.x + lv.cw + half.x);
    c.y = c.y.clamp(p.y - half.y, p.y + lv.ch + half.y);
    size / 2.0 - c
}

/// Zoomed out, the card under the pointer shows its full card beside it
/// (M33, owner 2026-11-16: "I can hover over nodes to get more info").
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn hover_tip(
    screen: Res<Screen>,
    view: Res<TreeView>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Option<Res<UiScale>>,
    cards: Query<(&Card, &Interaction), Without<TipPart>>,
    mut tips: Query<(&TipCard, &mut Node)>,
    chain: Res<MissionChain>,
    launch: Res<super::Launch>,
) {
    let lv = &LEVELS[view.level.min(LEVELS.len() - 1)];
    let zoomed_out = lv.big > 0.0;
    let scale = ui_scale.map_or(1.0, |s| s.0);
    let window = windows.single().ok();
    let mut pointer = window.and_then(|w| w.cursor_position()).map(|p| p / scale);
    let mut hovered = (*screen == Screen::Missions && zoomed_out)
        .then(|| {
            cards
                .iter()
                .find(|(_, i)| **i != Interaction::None)
                .map(|(c, _)| c.0)
        })
        .flatten();
    // A capture's hover (`--tree-hover <id>`, a capture instrument): a
    // headless run has no pointer, so the pointer is put on that card.
    if *screen == Screen::Missions
        && zoomed_out
        && let Some(id) = launch.tree_hover.as_deref()
        && let Some(i) = chain.0.missions.iter().position(|m| m.id == id)
    {
        let at = layout(&chain.0);
        hovered = Some(i);
        pointer = Some(view.pan + lv.card(at[i]) + Vec2::new(lv.cw, lv.ch) / 2.0);
    }
    let screen_size = window.map_or(Vec2::splat(800.0), |w| {
        Vec2::new(w.width(), w.height()) / scale
    });
    let native = &LEVELS[OPEN_LEVEL];
    for (tip, mut node) in &mut tips {
        let show = hovered == Some(tip.0) && pointer.is_some();
        let want = if show { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
        if let (true, Some(at)) = (show, pointer) {
            // Right of the pointer, or left where it would leave the
            // screen; level with it, kept on screen.
            let x = if at.x + 18.0 + native.cw > screen_size.x {
                at.x - 18.0 - native.cw
            } else {
                at.x + 18.0
            };
            let y = (at.y - native.ch / 2.0).clamp(8.0, (screen_size.y - native.ch - 8.0).max(8.0));
            node.left = px(x.round());
            node.top = px(y.round());
        }
    }
}

/// The whole tree on screen, under the tab strip: the largest level it
/// fits at, or the farthest.
fn fitted(chain: &Chain, size: Vec2, mut view: TreeView) -> TreeView {
    let (cols, rows) = extent(&layout(chain));
    let room = size - Vec2::new(0.0, 30.0);
    view.level = (0..LEVELS.len())
        .rev()
        .find(|&l| {
            let tree = LEVELS[l].size(cols, rows);
            tree.x <= room.x && tree.y <= room.y
        })
        .unwrap_or(0);
    let tree = LEVELS[view.level].size(cols, rows);
    view.pan = Vec2::new(
        (size.x - tree.x) / 2.0,
        30.0 + ((room.y - tree.y) / 2.0).max(0.0),
    );
    view
}

/// Size and place every card, line and plaque for the view's level, when
/// the level changes or the tab is built.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn relayout(
    view: Res<TreeView>,
    chain: Res<MissionChain>,
    added: Query<(), Added<Card>>,
    mut last: Local<Option<usize>>,
    mut nodes: ParamSet<(
        Query<(&Card, &mut Node), Without<TipPart>>,
        Query<(&Edge, &EdgeSeg, &mut Node)>,
        Query<(&BranchLabel, &mut Node)>,
        Query<&mut Node, (With<CardDetail>, Without<TipPart>)>,
        Query<&mut Node, With<TreeWorld>>,
        Query<&mut Node, (Or<(With<CardTitle>, With<CardBadge>)>, Without<TipPart>)>,
        Query<(&mut Node, Has<ImageNode>), (With<CardBig>, Without<TipPart>)>,
        Query<
            (&mut Node, Has<CardIcon>),
            (Or<(With<CardIcon>, With<CardCheck>)>, Without<TipPart>),
        >,
    )>,
    mut fonts: ParamSet<(
        Query<(&mut TextFont, &mut TextLayout), (With<CardTitle>, Without<TipPart>)>,
        Query<&mut TextFont, (With<CardSub>, Without<TipPart>)>,
        Query<&mut TextFont, (With<CardObjective>, Without<TipPart>)>,
        Query<&mut TextFont, (With<CardUnlocks>, Without<TipPart>)>,
    )>,
    mut badges: Query<&mut KitText, (With<CardBadge>, Without<TipPart>)>,
) {
    if added.is_empty() && *last == Some(view.level) {
        return;
    }
    *last = Some(view.level);
    let lv = &LEVELS[view.level.min(LEVELS.len() - 1)];
    let chain = &chain.0;
    let at = layout(chain);
    let (cols, rows) = extent(&at);
    for mut node in &mut nodes.p4() {
        let size = lv.size(cols, rows);
        node.width = px(size.x);
        node.height = px(size.y);
    }
    for (card, mut node) in &mut nodes.p0() {
        let p = lv.card(at[card.0]);
        node.left = px(p.x);
        node.top = px(p.y);
        node.width = px(lv.cw);
        node.height = px(lv.ch);
        node.padding = UiRect::axes(px(lv.pad.0), px(lv.pad.1));
    }
    let index = |id: &str| chain.missions.iter().position(|m| m.id == id);
    for (edge, seg, mut node) in &mut nodes.p1() {
        let m = &chain.missions[edge.0];
        let Some(p) = m.needs.first().and_then(|p| index(p)) else {
            continue;
        };
        match edge_segments(lv, at[p], at[edge.0])[seg.0] {
            Some(r) => {
                node.display = Display::Flex;
                node.left = px(r.min.x);
                node.top = px(r.min.y);
                node.width = px(r.width());
                node.height = px(r.height());
            }
            None => node.display = Display::None,
        }
    }
    for (label, mut node) in &mut nodes.p2() {
        let p = lv.card(at[label.0]);
        node.display = if lv.labels {
            Display::Flex
        } else {
            Display::None
        };
        node.left = px(p.x);
        node.top = px(p.y - 24.0);
    }
    let detail = if lv.full {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut nodes.p3() {
        node.display = detail;
    }
    let text = if lv.text {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut nodes.p5() {
        node.display = text;
    }
    // Zoomed out, the card is its icon, filling the frame.
    let big = if lv.big > 0.0 {
        Display::Flex
    } else {
        Display::None
    };
    for (mut node, image) in &mut nodes.p6() {
        if image {
            node.width = px(lv.big);
            node.height = px(lv.big);
        } else {
            node.display = big;
        }
    }
    // The title's icon and the objectives' ticks, at the level's sizes.
    for (mut node, icon) in &mut nodes.p7() {
        let side = if icon { lv.icon } else { lv.check };
        node.width = px(side);
        node.height = px(side);
    }
    for (mut font, mut text) in &mut fonts.p0() {
        font.font_size = lv.title;
        text.linebreak = if lv.full {
            LineBreak::WordBoundary
        } else {
            LineBreak::NoWrap
        };
    }
    for mut font in &mut fonts.p1() {
        font.font_size = lv.sub;
    }
    for mut font in &mut fonts.p2() {
        font.font_size = lv.line;
    }
    for mut font in &mut fonts.p3() {
        font.font_size = lv.sub;
    }
    for mut text in &mut badges {
        if text.size != lv.badge {
            text.size = lv.badge;
        }
    }
}

/// Move the tree to the view. Its scale is always 1: the cards are drawn at
/// the kit's own scale, and the pan is on whole physical pixels.
pub fn place(view: Res<TreeView>, mut world: Query<&mut UiTransform, With<TreeWorld>>) {
    for mut transform in &mut world {
        let want = UiTransform {
            translation: Val2::px(view.pan.x, view.pan.y),
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

    /// Every level is whole UI pixels, so with the tree's scale at 1 one
    /// art pixel is the kit's whole scale in physical pixels: every card,
    /// line and plaque lands on whole pixels, and a snapped pan does too.
    #[test]
    fn every_level_is_whole_pixels() {
        let chain = shipped();
        let at = layout(&chain);
        let whole = |v: f32| v.fract() == 0.0;
        for (n, lv) in LEVELS.iter().enumerate() {
            for v in [
                lv.cw, lv.ch, lv.gx, lv.gy, lv.title, lv.sub, lv.line, lv.pad.0, lv.pad.1, lv.icon,
                lv.big, lv.check,
            ] {
                assert!(whole(v), "level {n}: {v}");
            }
            assert!(lv.badge >= 1, "level {n}");
            for (i, m) in chain.missions.iter().enumerate() {
                let p = lv.card(at[i]);
                assert!(whole(p.x) && whole(p.y), "level {n}: {} at {p}", m.id);
                let Some(parent) = m.needs.first() else {
                    continue;
                };
                let parent = chain.missions.iter().position(|q| &q.id == parent).unwrap();
                for r in edge_segments(lv, at[parent], at[i]).into_iter().flatten() {
                    for v in [r.min.x, r.min.y, r.max.x, r.max.y] {
                        assert!(whole(v), "level {n}: {} line {r:?}", m.id);
                    }
                }
            }
        }
        // The levels widen from the farthest to the nearest.
        for w in LEVELS.windows(2) {
            assert!(w[1].pitch().x >= w[0].pitch().x && w[1].pitch().y >= w[0].pitch().y);
        }
        // A pan snapped at an art scale of k is k whole physical pixels a
        // UI pixel.
        for k in 1..=4 {
            let k = k as f32;
            for pan in [Vec2::new(10.37, -3.2), Vec2::new(-511.5, 77.75)] {
                let p = snap(pan, k) * k;
                assert!(whole(p.x) && whole(p.y), "{pan} at {k}");
            }
        }
    }

    /// Every card is one shape at every zoom (TS.4): each level is the
    /// native card in proportion, a card is its icon at 0.6 and below and
    /// the full card above, and every mission names an icon that exists, at
    /// a whole multiple of the art.
    #[test]
    fn every_level_is_the_native_card_in_proportion_with_its_icon() {
        let native = &LEVELS[OPEN_LEVEL];
        for lv in &LEVELS {
            let (sx, sy) = (lv.cw / native.cw, lv.ch / native.ch);
            assert!(
                (sx - sy).abs() < 0.08,
                "{lv:?} is not the native card's shape"
            );
            assert_eq!(lv.big > 0.0, !lv.full, "icon tile or full card, never both");
            if lv.big > 0.0 {
                assert!(lv.big <= lv.ch.min(lv.cw) && lv.big % 8.0 == 0.0, "{lv:?}");
            }
        }
        for m in &shipped().missions {
            assert!(!m.icon.is_empty(), "{} has no icon", m.id);
            let path = format!(
                "{}/../../assets/items/{}.png",
                env!("CARGO_MANIFEST_DIR"),
                m.icon
            );
            let size =
                image::image_dimensions(&path).unwrap_or_else(|e| panic!("{}: {path}: {e}", m.id));
            assert!(
                size.0 == size.1 && size.0 % 8 == 0 && size.0 <= 32,
                "{}: {} is {size:?}",
                m.id,
                m.icon
            );
        }
    }

    /// A step of the wheel keeps the point under the pointer between the
    /// same cards.
    #[test]
    fn a_level_step_keeps_the_point_under_the_pointer() {
        let at = Vec2::new(400.0, 300.0);
        let pan = Vec2::new(-120.0, 40.0);
        let (a, b) = (&LEVELS[OPEN_LEVEL], &LEVELS[1]);
        let u = (at - pan - a.origin()) / a.pitch();
        let pan2 = rezoom(pan, at, a, b);
        let u2 = (at - pan2 - b.origin()) / b.pitch();
        assert!((u - u2).length() < 1e-4);
    }
}
