//! The pack: thirty slots behind the hotbar, opened with I or Tab
//! (`inventory-grid` tasks 4.1 and 4.2, decision 9).
//!
//! It is a menu screen of its own, as the field guide is, so it holds the
//! pointer and closes on Escape with no code of its own for either. Its grid
//! is the slot row's own squares (`slots::cell`): the pack's thirty, three
//! rows of ten, and the hotbar's ten again under them, so a stack can go
//! either way. The slot row's painter paints every square by its index.
//!
//! **A click picks a stack up and a second click puts it down** (decision 5),
//! and the stack stays in its slot in between, drawn at the pointer. So the
//! second click is the whole move, one saved line, and closing the pack with
//! a stack in hand leaves it where it always was: nothing is ever off the
//! slots, and nothing can be lost by a close, a crash or a full pack.

use super::menu::{self, Panel, Screen};
use super::slots::{self, Hotbar, ItemIcons, SlotCell};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::{self, KitText, UiKit};
use pbd_app::ui_kit::{Localized, tr};
use pbd_app::walking::WalkingReadout;
use pbd_core::accept::PackMove;
use pbd_core::inventory::{CARRIED, PACK, SLOTS, Slots, Stack};
use pbd_core::vehicle::hold::{self, HOLD_MAX, Hold, Place};

/// How near a raft must be, m, for the pack opened on foot to show its hold.
pub const HOLD_REACH_M: f64 = 3.5;

/// The raft whose hold the pack shows (`small-boats` design 5): the one
/// aboard, or the nearest in reach on foot; and its hold as last read or
/// written. Its slots are the pack's squares from `CARRIED` on.
#[derive(Resource, Default, Debug, Clone)]
pub struct HoldView {
    pub craft: Option<(Entity, u64)>,
    pub hold: Hold,
}

/// The HOLD panel under the pack's.
#[derive(Component)]
pub struct HoldPanel;

/// The row under the pack that holds the HOLD and, beside it, the boat's
/// own panel (`boat`).
#[derive(Component)]
pub struct HoldRow;

/// The stack picked up: which slot, and how many of it go on the next click
/// (all of it, or the larger half for a right-click).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Held {
    pub from: usize,
    pub count: u16,
}

/// What is on the pointer while the pack is open.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackHand(pub Option<Held>);

/// A square of the pack's screen, by slot index.
#[derive(Component)]
pub struct PackCell(pub usize);

/// The pack's trash plaque: a stack put on it is destroyed
/// (`drop-and-trash` design 3).
#[derive(Component)]
pub struct TrashCell;

/// One of the pack screen's two panels: a click with a stack held that is
/// on neither (the background) drops the stack (`drop-and-trash` design 3).
#[derive(Component)]
pub struct PackPanel;

/// The stack drawn at the pointer.
#[derive(Component)]
pub struct HeldIcon;
#[derive(Component)]
pub struct HeldCount;

/// Which mouse button clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    Left,
    Right,
}

/// What a click on slot `index` does, from the slots and what is in hand:
/// the slots after it, where it moved anything, and what is in hand after.
///
/// - Nothing in hand: a click picks the stack up, a right-click the larger
///   half of it, and a shift-click sends it between the hotbar and the pack.
/// - A stack in hand: either click puts it down on `index` (`Slots::shift`),
///   and a click on the slot it came from puts it back.
#[cfg(test)]
pub fn click(
    slots: &Slots,
    hand: Option<Held>,
    index: usize,
    button: Click,
    shift: bool,
) -> (Option<Slots>, Option<Held>) {
    let (mv, held) = resolve(slots, hand, index, button, shift);
    let moved = mv
        .and_then(|mv| pbd_core::accept::pack(slots, mv).ok())
        .filter(|moved| moved != slots);
    (moved, held)
}

/// What a click asks for: the move (`multiplayer` design 19), which the
/// server decides, and what is in hand after. [`click`] is this with the
/// move applied.
pub fn resolve(
    slots: &Slots,
    hand: Option<Held>,
    index: usize,
    button: Click,
    shift: bool,
) -> (Option<PackMove>, Option<Held>) {
    match hand {
        None => {
            let Some(stack) = slots.get(index) else {
                return (None, None);
            };
            if shift {
                return (Some(PackMove::Send { index }), None);
            }
            let count = match button {
                Click::Left => stack.count,
                Click::Right => stack.count.div_ceil(2),
            };
            (None, Some(Held { from: index, count }))
        }
        Some(held) if held.from == index => (None, None),
        Some(held) => (
            Some(PackMove::Shift {
                from: held.from,
                to: index,
                count: held.count,
            }),
            None,
        ),
    }
}

/// A pack square's place: the player's slots below `CARRIED`, the hold's
/// from it.
fn place_of(index: usize) -> Place {
    if index >= CARRIED {
        Place::Hold(index - CARRIED)
    } else {
        Place::Carried(index)
    }
}

/// What a click does with a hold open (`small-boats` design 5): the same as
/// [`click`], across the slots and the hold. A shift-click sends a stack
/// between the hold and the player's slots. Changes `slots` and `hold` in
/// place and returns what is in hand after.
pub fn hold_click(
    slots: &mut Slots,
    hold_now: &mut Hold,
    hand: Option<Held>,
    index: usize,
    button: Click,
    shift: bool,
) -> Option<Held> {
    if index >= CARRIED + hold_now.slots.len() {
        return hand;
    }
    let at = place_of(index);
    let stack = match at {
        Place::Carried(i) => slots.get(i),
        Place::Hold(i) => hold_now.get(i),
    };
    match hand {
        None => {
            let stack = stack?;
            if shift {
                hold::send(slots, hold_now, at);
                return None;
            }
            let count = match button {
                Click::Left => stack.count,
                Click::Right => stack.count.div_ceil(2),
            };
            Some(Held { from: index, count })
        }
        Some(held) if held.from == index => None,
        Some(held) => {
            hold::shift(slots, hold_now, place_of(held.from), at, held.count);
            None
        }
    }
}

/// Which raft's hold the pack shows, read from the save when it changes,
/// and the HOLD panel shown with it. Aboard a raft, that raft; on foot, the
/// nearest within [`HOLD_REACH_M`].
#[allow(clippy::too_many_arguments)]
pub fn find_hold(
    screen: Res<Screen>,
    aboard: Option<Res<pbd_app::vehicles::Aboard>>,
    vehicles: Query<(Entity, &pbd_app::vehicles::Vehicle, &GlobalTransform)>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    save: Res<WorldSave>,
    mut view: ResMut<HoldView>,
    frame: Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    mut panels: Query<&mut Node, (With<HoldPanel>, Without<PackCell>)>,
    (carts, cart_state, fleet, mut squares): (
        Query<(Entity, &pbd_app::vehicles::carts::CartThing)>,
        Option<Res<pbd_app::vehicles::carts::Carts>>,
        Option<Res<pbd_app::vehicles::Fleet>>,
        Query<(&PackCell, &mut Node), Without<HoldPanel>>,
    ),
) {
    let raft = |kind| kind == pbd_core::vehicle::Kind::Raft;
    // A cart's hold (`land-transport` design 10): the one in hand or under
    // the seat, else the nearest in reach on foot, at its own size.
    let frame_now = frame.as_deref().copied().unwrap_or_default();
    let cart_size = |kind| fleet.as_deref().map_or(9, |f| f.specs.carts.of(kind).hold);
    let with_cart = cart_state
        .as_deref()
        .and_then(|c| c.holding.or(c.seated))
        .and_then(|e| carts.get(e).ok())
        .or_else(|| {
            let feet = walkers
                .iter()
                .next()
                .map(|p| p.0.as_dvec3() - frame_now.physics_center())?;
            carts
                .iter()
                .map(|(e, t)| (e, t, t.cart.at.distance(feet)))
                .filter(|(_, _, d)| *d <= HOLD_REACH_M)
                .min_by(|a, b| a.2.total_cmp(&b.2))
                .map(|(e, t, _)| (e, t))
        })
        .map(|(e, t)| (e, t.cart.id, cart_size(t.cart.kind)));
    let found_cart = with_cart.filter(|_| *screen == Screen::Pack);
    let found = if let Some((e, id, _)) = found_cart {
        Some((e, id))
    } else if *screen != Screen::Pack {
        None
    } else if let Some(entity) = aboard.and_then(|a| a.0) {
        vehicles
            .get(entity)
            .ok()
            .filter(|(_, v, _)| raft(v.craft.kind))
            .map(|(e, v, _)| (e, v.craft.id))
    } else {
        // Both in the render frame: the walker's position and the craft's
        // drawn place.
        // Planet-local, both: the walker's physics position less the
        // physics centre, the craft's drawn place less the render centre.
        let frame = frame.as_deref().copied().unwrap_or_default();
        let feet = walkers
            .iter()
            .next()
            .map(|p| p.0.as_dvec3() - frame.physics_center());
        feet.and_then(|feet| {
            vehicles
                .iter()
                .filter(|(_, v, _)| raft(v.craft.kind))
                .map(|(e, v, at)| {
                    let at = at.translation().as_dvec3() - frame.center;
                    (e, v.craft.id, at.distance(feet))
                })
                .filter(|(_, _, d)| *d <= HOLD_REACH_M)
                .min_by(|a, b| a.2.total_cmp(&b.2))
                .map(|(e, id, _)| (e, id))
        })
    };
    if found != view.craft {
        view.hold = match (found, found_cart) {
            (Some((_, id)), Some((_, _, size))) => {
                pbd_app::vehicles::hold::read_sized(&save, id, size)
            }
            (Some((_, id)), None) => pbd_app::vehicles::hold::read(&save, id),
            (None, _) => Hold::default(),
        };
        view.craft = found;
    }
    // Only the hold's own squares show.
    let size = view.hold.slots.len();
    for (cell, mut node) in &mut squares {
        if cell.0 < CARRIED {
            continue;
        }
        let want = if cell.0 - CARRIED < size {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
    }
    let want = if found.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut panels {
        if node.display != want {
            node.display = want;
        }
    }
}

/// Paint the hold's squares from the hold. After `slots::update`, which
/// paints every square from the player's slots and so leaves these empty.
#[allow(clippy::type_complexity)]
pub fn paint_hold(
    screen: Res<Screen>,
    view: Res<HoldView>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    mut icon_nodes: Query<(&slots::SlotIcon, &mut ImageNode)>,
    mut counts: Query<(&slots::SlotCount, &mut KitText)>,
    mut tips: Query<(&SlotCell, &mut ui_kit::Tooltip)>,
) {
    let Some(icons) = icons else {
        return;
    };
    if *screen != Screen::Pack || view.craft.is_none() {
        return;
    }
    let hold = |i: usize| (CARRIED..super::furniture::STORE_AT).contains(&i);
    for (icon, mut node) in &mut icon_nodes {
        if hold(icon.0) {
            slots::paint_icon(&mut node, view.hold.get(icon.0 - CARRIED), &icons, &images);
        }
    }
    for (count, mut text) in &mut counts {
        if !hold(count.0) {
            continue;
        }
        let label = match view.hold.get(count.0 - CARRIED) {
            Some(stack) if stack.count > 1 => Localized::verbatim(stack.count.to_string()),
            _ => Localized::empty(),
        };
        if text.text != label {
            text.text = label;
        }
    }
    for (cell, mut tip) in &mut tips {
        if !hold(cell.0) {
            continue;
        }
        let want = slots::stack_tip(view.hold.get(cell.0 - CARRIED), &icons);
        if tip.0 != want {
            tip.0 = want;
        }
    }
}

/// I or Tab opens the pack from the world on foot and closes it again. `PreUpdate`
/// after `menu::toggle`, before the world's readers, like the guide's J. The
/// pack does not open in a ship's seat, where the hotbar is not drawn either.
pub fn open(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    walking: Option<Res<WalkingReadout>>,
    aboard: Option<Res<pbd_app::vehicles::Aboard>>,
    vehicles: Query<&pbd_app::vehicles::Vehicle>,
) {
    // Aboard a paddled boat the pack opens too (`small-boats` design 5): a
    // raft's hold is filled and emptied from it, and a catch looked at.
    let afloat = aboard
        .and_then(|a| a.0)
        .and_then(|e| vehicles.get(e).ok())
        .is_some_and(|v| v.craft.kind.paddled());
    // I is the pack and O the starter missions (`starter-missions` design
    // 13): each opens its own screen from the world, shuts it again, and
    // switches to it from the other.
    let to = if keys.any_just_pressed([KeyCode::KeyI, KeyCode::Tab]) {
        Screen::Pack
    } else if keys.just_pressed(KeyCode::KeyO) {
        Screen::Missions
    } else {
        return;
    };
    match *screen {
        Screen::Playing if afloat || walking.is_some_and(|readout| readout.active) => *screen = to,
        open if open == to => *screen = Screen::Playing,
        Screen::Pack | Screen::Missions => *screen = to,
        _ => {}
    }
}

/// The pack's column, native px (`tool-crafting` decision 12): the ten
/// slots. The Craft panel stands beside it, a panel of its own.
pub const PACK_COLUMN: f32 = 10.0 * slots::SLOT;
/// The two panels, native px: the pack (its ten slots, three rows, the hand
/// and the hotbar, in the rim) and the craft panel beside it (the lintel, the
/// six recipes, the craft row and its bar, in the rim).
// The pack's height fits its rows, with a small gap over the hotbar (owner,
// 2026-11-20: "why so much empty space on inventory?").
pub const PACK_PANEL: (f32, f32) = (PACK_COLUMN + 2.0 * RIM, 160.0);
pub const CRAFT_PANEL: (f32, f32) = (super::craft::CRAFT_COLUMN + 2.0 * RIM, 214.0);
/// The rock's rim, inside which everything stands.
pub const RIM: f32 = 14.0;

/// The pack's panel: the approved base rock baked at the pack's own size
/// (`rock_panels.pack-wide`, 464 x 200 native px), in two columns
/// (`tool-crafting` decision 12; owner, 2026-11-16: "this menu should be
/// wider", "I expect craft to be a menu on the side not on the bottom").
/// Left, under the PACK lintel: the thirty slots, and the hotbar's ten at
/// the column's foot. Right, under the CRAFT lintel: the hands' recipes and
/// the chosen one's row. No close button (owner, 2026-11-14: "In desktop esc
/// will do just fine"; I shuts it too) and no line of instructions.
pub fn spawn(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    kit: Res<UiKit>,
    recipes: Res<pbd_app::config::Recipes>,
) {
    let atlas = icons.atlas.clone();
    let kit = &*kit;
    // Two panels side by side (owner, 2026-11-19, with a sketch: "make
    // crafting box sepereate", the pack on the left, a taller craft panel on
    // the right), each on the kit's stone frame at the size it holds: no
    // baked rock is the pack's own size alone.
    let (hold_rock, (hold_w, _)) = kit.rock("pack");
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..menu::layer()
            },
            GlobalZIndex(10),
            Panel(Screen::Pack),
        ))
        .with_children(|layer| {
            layer
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::FlexStart,
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|pair| {
                    pair.spawn((
                        kit.nine(pbd_app::ui_kit::Set::Stone, "frame"),
                        PackPanel,
                        Interaction::default(),
                        // Sized here: a framed node is otherwise its picture's
                        // size, not its contents'.
                        Node {
                            width: px(PACK_PANEL.0),
                            height: px(PACK_PANEL.1),
                            flex_shrink: 0.0,
                            padding: UiRect::all(px(RIM)),
                            ..default()
                        },
                    ))
                    .with_children(|panel| {
                        panel
                            .spawn(Node {
                                width: px(PACK_COLUMN),
                                flex_shrink: 0.0,
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            })
                            .with_children(|column| {
                                column
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        ..default()
                                    })
                                    .with_children(|top| {
                                        menu::lintel(top, kit, tr("pack.pack"));
                                        for row in 0..PACK / SLOTS {
                                            slot_line(top, kit, &atlas, SLOTS + row * SLOTS);
                                        }
                                    });
                                column
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Column,
                                        ..default()
                                    })
                                    .with_children(|foot| {
                                        // The hand over the hotbar's first slot,
                                        // as the HUD has it beside the hotbar
                                        // (`pack-tool-slot` decision 3): the
                                        // baked rock is the ten slots wide.
                                        foot.spawn(Node {
                                            align_items: AlignItems::End,
                                            column_gap: px(4),
                                            margin: UiRect::bottom(px(3)),
                                            ..default()
                                        })
                                        .with_children(
                                            |line| {
                                                super::equipment::spawn_pack_hand(
                                                    line, kit, &icons,
                                                );
                                                line.spawn(
                                                    KitText::new(
                                                        tr("pack.hotbar"),
                                                        ui_kit::TEXT_DIM,
                                                    )
                                                    .shadow(ui_kit::INK),
                                                );
                                                // The trash, at the line's far end.
                                                line.spawn(Node {
                                                    flex_grow: 1.0,
                                                    ..default()
                                                });
                                                line.spawn((
                                                    ui_kit::plaque(
                                                        kit,
                                                        pbd_app::ui_kit::Set::Stone,
                                                        0.0,
                                                    ),
                                                    TrashCell,
                                                ))
                                                .with_child(ui_kit::plaque_label(
                                                    pbd_app::ui_kit::Set::Stone,
                                                    tr("pack.trash"),
                                                ));
                                            },
                                        );
                                        slot_line(foot, kit, &atlas, 0);
                                    });
                            });
                    });
                    pair.spawn((
                        kit.nine(pbd_app::ui_kit::Set::Stone, "frame"),
                        PackPanel,
                        Interaction::default(),
                        // Sized here: a framed node is otherwise its picture's
                        // size, not its contents'.
                        Node {
                            width: px(CRAFT_PANEL.0),
                            height: px(CRAFT_PANEL.1),
                            flex_shrink: 0.0,
                            padding: UiRect::all(px(RIM)),
                            ..default()
                        },
                    ))
                    .with_children(|panel| {
                        super::craft::spawn_pack_craft(panel, kit, &atlas, &recipes);
                    });
                });
            // A raft's hold, under the pack, while one is aboard or in reach
            // (`small-boats` design 5); the boat's menu puts its panel
            // beside it (`river-craft` design 10).
            layer
                .spawn((
                    Node {
                        column_gap: px(6),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    },
                    HoldRow,
                ))
                .with_children(|layer| {
                    layer
                        .spawn((
                            hold_rock,
                            Node {
                                width: px(hold_w),
                                padding: UiRect::all(px(RIM)),
                                flex_direction: FlexDirection::Column,
                                display: Display::None,
                                ..default()
                            },
                            HoldPanel,
                        ))
                        .with_children(|panel| {
                            menu::lintel(panel, kit, tr("pack.hold"));
                            // As many squares as the largest hold (a horse
                            // cart's, `land-transport` design 10), in rows of
                            // ten; a smaller hold hides the rest.
                            for first in (CARRIED..CARRIED + HOLD_MAX).step_by(SLOTS) {
                                slot_row(
                                    panel,
                                    kit,
                                    &atlas,
                                    first,
                                    (CARRIED + HOLD_MAX - first).min(SLOTS),
                                );
                            }
                        });
                    // A container's panel, beside the hold's, while one is
                    // open (`furniture` design 11).
                    super::furniture::spawn_panel(layer, kit, &atlas);
                });
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: px(16),
                height: px(16),
                display: Display::None,
                ..default()
            },
            GlobalZIndex(20),
            HeldIcon,
            ImageNode {
                image: atlas,
                color: Color::NONE,
                ..default()
            },
        ))
        .with_children(|icon| {
            icon.spawn((
                KitText::new(Localized::empty(), ui_kit::LABEL).outline(ui_kit::INK),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(-3),
                    right: px(-2),
                    ..default()
                },
                HeldCount,
            ));
        });
}

/// `count` slots in a row, from slot `first`.
fn slot_row(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    first: usize,
    count: usize,
) {
    parent.spawn(Node::default()).with_children(|line| {
        for index in first..first + count {
            slots::cell(line, kit, index, atlas, (Button, PackCell(index)));
        }
    });
}

/// Ten slots in a row, from slot `first`.
fn slot_line(parent: &mut ChildSpawnerCommands, kit: &UiKit, atlas: &Handle<Image>, first: usize) {
    parent.spawn(Node::default()).with_children(|line| {
        for index in first..first + SLOTS {
            slots::cell(line, kit, index, atlas, (Button, PackCell(index)));
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn press(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    cells: Query<(&Interaction, &PackCell)>,
    mut hand: ResMut<PackHand>,
    mut hotbar: ResMut<Hotbar>,
    mut save: ResMut<WorldSave>,
    mut view: ResMut<HoldView>,
    (trash, panels, mut toss): (
        Query<&Interaction, With<TrashCell>>,
        Query<&Interaction, With<PackPanel>>,
        ResMut<pbd_app::drops::Toss>,
    ),
) {
    if *screen != Screen::Pack {
        if hand.0.is_some() {
            // Closed with a stack in hand: it never left its slot.
            hand.0 = None;
        }
        return;
    }
    let button = if mouse.just_pressed(MouseButton::Left) {
        Click::Left
    } else if mouse.just_pressed(MouseButton::Right) {
        Click::Right
    } else {
        return;
    };
    // The square pressed, before one merely hovered (a hover left on another
    // square took the click: seen with the showcase bot, 2026-11-20).
    // A stack held over the trash: destroyed, the slots recorded.
    if button == Click::Left
        && let Some(held) = hand.0
        && held.from < CARRIED
        && trash.iter().any(|i| *i != Interaction::None)
    {
        let trash = PackMove::Trash {
            index: held.from,
            count: held.count,
        };
        match save.decide_pack(trash, &hotbar.0) {
            Ok(slots) => {
                info!("TRASH {} from slot {}", held.count, held.from);
                hotbar.0 = slots;
            }
            Err(refusal) => warn!("trash refused: {refusal:?}"),
        }
        hand.0 = None;
        return;
    }
    // A stack held and clicked on the background (no panel, slot or button
    // under the pointer): dropped in front of the player.
    if button == Click::Left
        && let Some(held) = hand.0
        && held.from < CARRIED
        && panels.iter().all(|i| *i == Interaction::None)
        && cells.iter().all(|(i, _)| *i == Interaction::None)
    {
        toss.0 = Some((held.from, held.count));
        hand.0 = None;
        return;
    }
    let Some(index) = cells
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Pressed)
        .or_else(|| {
            cells
                .iter()
                .find(|(interaction, _)| **interaction != Interaction::None)
        })
        .map(|(_, cell)| cell.0)
    else {
        return;
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if let Some((_, id)) = view.craft
        && (index >= CARRIED || hand.0.is_some_and(|h| h.from >= CARRIED) || shift)
    {
        let mut slots = hotbar.0.clone();
        let mut held_hold = view.hold.clone();
        let held = hold_click(&mut slots, &mut held_hold, hand.0, index, button, shift);
        if slots != hotbar.0 || held_hold != view.hold {
            if !pbd_app::vehicles::hold::store(&mut save, id, &held_hold, &slots) {
                error!("hold move BLOCKED: the save did not accept it");
                hand.0 = None;
                return;
            }
            hotbar.0 = slots;
            view.hold = held_hold;
        }
        hand.0 = held;
        return;
    }
    if index >= CARRIED {
        return;
    }
    // The move is the server's to decide (`multiplayer` design 19); one that
    // would change nothing is not asked.
    let (mv, held) = resolve(&hotbar.0, hand.0, index, button, shift);
    if let Some(mv) = mv
        && pbd_core::accept::pack(&hotbar.0, mv).is_ok_and(|moved| moved != hotbar.0)
    {
        match save.decide_pack(mv, &hotbar.0) {
            Ok(moved) => hotbar.0 = moved,
            Err(refusal) => {
                error!("pack move BLOCKED: {refusal:?}");
                hand.0 = None;
                return;
            }
        }
    }
    hand.0 = held;
}

/// Q on foot drops the held hotbar stack in front of the player
/// (`drop-and-trash` design 2).
pub fn drop_key(
    keys: Res<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    hotbar: Res<Hotbar>,
    mut toss: ResMut<pbd_app::drops::Toss>,
) {
    if *screen != Screen::Playing
        || !walking.is_some_and(|w| w.active && w.captured)
        || !keys.just_pressed(KeyCode::KeyQ)
    {
        return;
    }
    let slot = hotbar.0.selected();
    if let Some(stack) = hotbar.0.get(slot) {
        toss.0 = Some((slot, stack.count));
    }
}

/// Draw what is in hand at the pointer, and mark the square it came from.
/// After `slots::update`, whose border it overrides.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn paint(
    screen: Res<Screen>,
    hand: Res<PackHand>,
    hotbar: Res<Hotbar>,
    view: Res<HoldView>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    kit: Res<UiKit>,
    mut held: Query<(&mut Node, &mut ImageNode), (With<HeldIcon>, Without<PackCell>)>,
    mut count: Query<&mut KitText, With<HeldCount>>,
    mut cells: Query<
        (&SlotCell, &Interaction, &mut ImageNode),
        (With<PackCell>, Without<HeldIcon>),
    >,
) {
    let Some(icons) = icons else {
        return;
    };
    let in_hand = hand.0.filter(|_| *screen == Screen::Pack);
    let stack = in_hand.and_then(|h| {
        let from = if h.from >= CARRIED {
            view.hold.get(h.from - CARRIED)
        } else {
            hotbar.get(h.from)
        };
        from.map(|s| Stack::new(s.item, h.count.min(s.count)))
    });
    let cursor = windows.iter().next().and_then(Window::cursor_position);
    for (mut node, mut image) in &mut held {
        match (stack, cursor) {
            (Some(stack), Some(at)) => {
                node.display = Display::Flex;
                // The cursor is in window pixels, a node in native ones.
                let at = at / ui_scale.0.max(1e-3);
                node.left = px(at.x - 8.0);
                node.top = px(at.y - 8.0);
                slots::paint_icon(&mut image, Some(stack), &icons, &images);
            }
            _ => node.display = Display::None,
        }
    }
    for mut text in &mut count {
        let label = match stack {
            Some(stack) if stack.count > 1 => Localized::verbatim(stack.count.to_string()),
            _ => Localized::empty(),
        };
        if text.text != label {
            text.text = label;
        }
    }
    if *screen != Screen::Pack {
        return;
    }
    // The slot a stack is lifted from, and the one under the pointer, are lit;
    // the selected slot wears the rune.
    for (cell, interaction, mut image) in &mut cells {
        let lit = in_hand.is_some_and(|h| h.from == cell.0) || *interaction != Interaction::None;
        let key = if lit {
            "slot-hover"
        } else {
            slots::slot_piece(cell.0 == hotbar.selected())
        };
        let want = kit.image(key);
        if image.image != want {
            image.image = want;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// With a raft's hold open, a click picks a stack up out of the hold and
    /// a second puts it down in the pack, and a shift-click sends a stack of
    /// the pack's into the hold (`small-boats` design 5).
    #[test]
    fn a_click_moves_stacks_across_the_hold() {
        use pbd_core::goods::Good;
        let log = pbd_core::inventory::Item::Good(Good::LOG);
        let rope = pbd_core::inventory::Item::Good(Good::ROPE);
        let mut slots = Slots::new();
        slots.set(SLOTS + 3, Some(Stack::new(rope, 4)));
        let mut hold = Hold::default();
        hold.slots[1] = Some(Stack::new(log, 20));
        let held = hold_click(&mut slots, &mut hold, None, CARRIED + 1, Click::Left, false);
        assert_eq!(
            held,
            Some(Held {
                from: CARRIED + 1,
                count: 20
            })
        );
        let held = hold_click(&mut slots, &mut hold, held, SLOTS + 5, Click::Left, false);
        assert_eq!(held, None);
        assert_eq!(slots.get(SLOTS + 5), Some(Stack::new(log, 20)));
        assert!(hold.get(1).is_none());
        hold_click(&mut slots, &mut hold, None, SLOTS + 3, Click::Left, true);
        assert_eq!(hold.get(0), Some(Stack::new(rope, 4)));
        assert_eq!(slots.get(SLOTS + 3), None);
    }
    use pbd_core::inventory::Item;
    use pbd_core::terrain::Material;

    /// In play is rock: the pack is framed in the base rock baked at the
    /// pack's own size, drawn whole (never tiled), its slots are dressed
    /// stone, and each column's title is on an oak lintel.
    #[test]
    fn the_pack_is_framed_in_its_own_rock() {
        let mut app = App::new();
        pbd_app::ui_kit::install_bare(&mut app);
        app.insert_resource(ItemIcons {
            atlas: Handle::default(),
            fish: Vec::new(),
            tools: vec![Handle::default(); pbd_core::inventory::Tool::ALL.len()],
            hand: Handle::default(),
            lights: Vec::new(),
            fish_names: Vec::new(),
            goods: Vec::new(),
            forge: Handle::default(),
            workbench: Handle::default(),
            carpenter: Handle::default(),
            drafting: Handle::default(),
            tiered: Vec::new(),
        });
        app.insert_resource(pbd_app::config::Recipes(
            pbd_core::station::RecipeFile::default().resolve().unwrap(),
        ));
        app.add_systems(Update, spawn);
        app.update();
        let world = app.world_mut();
        let (rock, lintel, slots) = {
            let kit = world.resource::<UiKit>();
            (
                kit.nine(pbd_app::ui_kit::Set::Stone, "frame").image,
                kit.image("lintel"),
                [kit.image("slot-idle"), kit.image("slot-selected")],
            )
        };
        let mut panels = world.query::<(&Panel, &Children)>();
        let (_, children) = panels
            .iter(world)
            .find(|(p, _)| p.0 == Screen::Pack)
            .expect("the pack's panel");
        // Two panels side by side (`mission-playthrough` 4.4): the pack and
        // the craft column, each on the stone frame.
        let pair = world.get::<Children>(children[0]).unwrap().to_vec();
        assert_eq!(pair.len(), 2, "the pack and the craft panel");
        for frame in pair {
            let image = world.get::<ImageNode>(frame).unwrap();
            assert_eq!(image.image, rock);
        }
        let mut wells = world.query::<&super::super::craft::Well>();
        assert!(wells.iter(world).count() >= 2, "the needs and the result");
        let mut rows = world.query::<&super::super::craft::RecipeRow>();
        assert_eq!(
            rows.iter(world).count(),
            world
                .resource::<pbd_app::config::Recipes>()
                .0
                .at("hand")
                .count(),
            "a row for every recipe the hands make"
        );
        let mut images = world.query::<&ImageNode>();
        assert_eq!(
            images.iter(world).filter(|i| i.image == lintel).count(),
            3,
            "an oak lintel over each column, PACK and CRAFT, and over a raft's HOLD"
        );
        let mut cells = world.query_filtered::<&ImageNode, With<PackCell>>();
        let n = cells.iter(world).count();
        assert_eq!(
            n,
            PACK + SLOTS + HOLD_MAX,
            "and the hold's squares, as many as a horse cart's"
        );
        assert!(cells.iter(world).all(|i| slots.contains(&i.image)));
        // The hand over the HOTBAR row (`pack-tool-slot` decision 3), and its
        // list: bare hands and every tool.
        let mut hand = world.query_filtered::<(), With<super::super::equipment::PackToolCell>>();
        assert_eq!(hand.iter(world).count(), 1, "the pack's hand cell");
        let mut rows = world.query::<&super::super::equipment::HandListRow>();
        assert_eq!(
            rows.iter(world).count(),
            1 + pbd_core::inventory::Tool::ALL.len()
        );
    }

    fn kit() -> Slots {
        let mut slots = Slots::new();
        slots.set(0, Some(Stack::new(Item::Block(Material::Dirt), 64)));
        slots.set(1, Some(Stack::new(Item::Block(Material::Sand), 7)));
        slots
    }

    /// A click picks up, a second click on an empty pack slot puts down, and
    /// only the second click changes the slots.
    #[test]
    fn a_click_picks_up_and_a_second_puts_down() {
        let slots = kit();
        let (moved, hand) = click(&slots, None, 0, Click::Left, false);
        assert_eq!(moved, None, "picking up moves nothing yet");
        assert_eq!(hand, Some(Held { from: 0, count: 64 }));
        let (moved, hand) = click(&slots, hand, 15, Click::Left, false);
        let moved = moved.expect("the put is the move");
        assert_eq!(moved.get(15).map(|s| s.count), Some(64));
        assert_eq!(moved.get(0), None);
        assert_eq!(hand, None);
    }

    /// A right-click takes the larger half, and putting it on another item
    /// does nothing, since half a stack cannot swap.
    #[test]
    fn a_right_click_takes_half_and_half_never_swaps() {
        let slots = kit();
        let (_, hand) = click(&slots, None, 1, Click::Right, false);
        assert_eq!(hand, Some(Held { from: 1, count: 4 }));
        let (moved, hand) = click(&slots, hand, 0, Click::Left, false);
        assert_eq!(moved, None);
        assert_eq!(hand, None);
        let (_, hand) = click(&slots, None, 1, Click::Right, false);
        let (moved, _) = click(&slots, hand, 30, Click::Left, false);
        let moved = moved.unwrap();
        assert_eq!(moved.get(30).map(|s| s.count), Some(4));
        assert_eq!(moved.get(1).map(|s| s.count), Some(3));
    }

    /// A shift-click sends a hotbar stack into the pack at once; a click back
    /// on the slot a stack came from puts it back; a click on nothing picks
    /// nothing up.
    #[test]
    fn shift_sends_and_a_click_home_puts_back() {
        let slots = kit();
        let (moved, hand) = click(&slots, None, 0, Click::Left, true);
        assert_eq!(moved.unwrap().get(SLOTS).map(|s| s.count), Some(64));
        assert_eq!(hand, None);
        let (_, hand) = click(&slots, None, 0, Click::Left, false);
        assert_eq!(click(&slots, hand, 0, Click::Left, false), (None, None));
        assert_eq!(click(&slots, None, 25, Click::Left, false), (None, None));
    }

    /// I opens the pack on foot and closes it; I does nothing in a ship's
    /// seat; and closing with a stack in hand leaves every slot as it was.
    #[test]
    fn i_opens_and_closes_the_pack_on_foot_only() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(Screen::Playing)
            .insert_resource(WalkingReadout {
                active: true,
                ..default()
            })
            .insert_resource(Hotbar(kit()))
            .insert_resource(PackHand(Some(Held { from: 0, count: 64 })))
            .insert_resource(WorldSave::memory_only())
            .init_resource::<HoldView>()
            .init_resource::<pbd_app::drops::Toss>()
            .add_systems(Update, (open, press).chain());
        let tap_i = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::KeyI);
            keys.clear();
            keys.press(KeyCode::KeyI);
            app.update();
        };
        tap_i(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Pack);
        tap_i(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Playing);
        assert_eq!(
            app.world().resource::<PackHand>().0,
            None,
            "the hand empties"
        );
        assert_eq!(app.world().resource::<Hotbar>().0, kit(), "nothing moved");
        app.world_mut().resource_mut::<WalkingReadout>().active = false;
        tap_i(&mut app);
        assert_eq!(
            *app.world().resource::<Screen>(),
            Screen::Playing,
            "not in a seat"
        );
    }

    /// Tab opens and shuts the pack as I does (decision 9).
    #[test]
    fn tab_opens_and_closes_the_pack() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Screen::Playing)
            .insert_resource(WalkingReadout {
                active: true,
                ..default()
            })
            .add_systems(Update, open);
        for want in [Screen::Pack, Screen::Playing] {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.release(KeyCode::Tab);
            keys.clear();
            keys.press(KeyCode::Tab);
            app.update();
            assert_eq!(*app.world().resource::<Screen>(), want);
        }
    }
}
