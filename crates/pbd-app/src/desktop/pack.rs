//! The pack: thirty slots behind the hotbar, opened with I (`inventory-grid`
//! tasks 4.1 and 4.2).
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
use pbd_app::walking::WalkingReadout;
use pbd_core::inventory::{PACK, SLOTS, Slots, Stack};

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
pub fn click(
    slots: &Slots,
    hand: Option<Held>,
    index: usize,
    button: Click,
    shift: bool,
) -> (Option<Slots>, Option<Held>) {
    match hand {
        None => {
            let Some(stack) = slots.get(index) else {
                return (None, None);
            };
            if shift {
                let mut sent = slots.clone();
                sent.send(index);
                return ((sent != *slots).then_some(sent), None);
            }
            let count = match button {
                Click::Left => stack.count,
                Click::Right => stack.count.div_ceil(2),
            };
            (None, Some(Held { from: index, count }))
        }
        Some(held) if held.from == index => (None, None),
        Some(held) => {
            let mut moved = slots.clone();
            moved.shift(held.from, index, held.count);
            ((moved != *slots).then_some(moved), None)
        }
    }
}

/// I opens the pack from the world on foot and closes it again. `PreUpdate`
/// after `menu::toggle`, before the world's readers, like the guide's J. The
/// pack does not open in a ship's seat, where the hotbar is not drawn either.
pub fn open(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    walking: Option<Res<WalkingReadout>>,
) {
    if !keys.just_pressed(KeyCode::KeyI) {
        return;
    }
    match *screen {
        Screen::Playing if walking.is_some_and(|readout| readout.active) => {
            *screen = Screen::Pack;
        }
        Screen::Pack => *screen = Screen::Playing,
        _ => {}
    }
}

/// The pack's panel: the approved base rock baked at the pack's own size
/// (`rock_panels.pack`, 268 x 152 native px), the oak lintel carrying its
/// title, the thirty slots and the hotbar's ten under them. Nothing else on
/// it: no close button (owner, 2026-11-14: "In desktop esc will do just
/// fine"; I shuts it too) and no line of instructions.
pub fn spawn(mut commands: Commands, icons: Res<ItemIcons>, kit: Res<UiKit>) {
    let atlas = icons.atlas.clone();
    let kit = &*kit;
    let (rock, (w, h)) = kit.rock("pack");
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Pack)))
        .with_children(|layer| {
            layer
                .spawn((
                    rock,
                    Node {
                        width: px(w),
                        height: px(h),
                        padding: UiRect::all(px(14)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|panel| {
                    menu::lintel(panel, kit, "PACK");
                    let rows = PACK / SLOTS;
                    for row in 0..=rows {
                        let first = if row < rows { SLOTS + row * SLOTS } else { 0 };
                        if row == rows {
                            panel.spawn((
                                KitText::new("HOTBAR", ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                                Node {
                                    margin: UiRect::new(px(0), px(0), px(5), px(3)),
                                    ..default()
                                },
                            ));
                        }
                        panel.spawn(Node::default()).with_children(|line| {
                            for index in first..first + SLOTS {
                                slots::cell(line, kit, index, &atlas, (Button, PackCell(index)));
                            }
                        });
                    }
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
                KitText::new("", ui_kit::LABEL).outline(ui_kit::INK),
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

#[allow(clippy::too_many_arguments)]
pub fn press(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    cells: Query<(&Interaction, &PackCell)>,
    mut hand: ResMut<PackHand>,
    mut hotbar: ResMut<Hotbar>,
    mut save: ResMut<WorldSave>,
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
    let Some(index) = cells
        .iter()
        .find(|(interaction, _)| **interaction != Interaction::None)
        .map(|(_, cell)| cell.0)
    else {
        return;
    };
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let (moved, held) = click(&hotbar.0, hand.0, index, button, shift);
    if let Some(moved) = moved {
        if !save.record_pack(&moved) {
            error!("pack move BLOCKED: the save did not accept it");
            hand.0 = None;
            return;
        }
        hotbar.0 = moved;
    }
    hand.0 = held;
}

/// Draw what is in hand at the pointer, and mark the square it came from.
/// After `slots::update`, whose border it overrides.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn paint(
    screen: Res<Screen>,
    hand: Res<PackHand>,
    hotbar: Res<Hotbar>,
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
        hotbar
            .get(h.from)
            .map(|s| Stack::new(s.item, h.count.min(s.count)))
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
            Some(stack) if stack.count > 1 => stack.count.to_string(),
            _ => String::new(),
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
    use pbd_core::inventory::Item;
    use pbd_core::terrain::Material;

    /// In play is rock: the pack is framed in the base rock baked at the
    /// pack's own size, drawn whole (never tiled), its slots are dressed
    /// stone, and its title is on the oak lintel.
    #[test]
    fn the_pack_is_framed_in_its_own_rock() {
        let mut app = App::new();
        pbd_app::ui_kit::install_bare(&mut app);
        app.insert_resource(ItemIcons {
            atlas: Handle::default(),
            fish: Vec::new(),
            tools: Vec::new(),
            hand: Handle::default(),
            lights: Vec::new(),
            fish_names: Vec::new(),
            goods: Vec::new(),
        });
        app.add_systems(Update, spawn);
        app.update();
        let world = app.world_mut();
        let (rock, lintel, slots) = {
            let kit = world.resource::<UiKit>();
            (
                kit.image("rock/pack"),
                kit.image("lintel"),
                [kit.image("slot-idle"), kit.image("slot-selected")],
            )
        };
        let mut panels = world.query::<(&Panel, &Children)>();
        let (_, children) = panels
            .iter(world)
            .find(|(p, _)| p.0 == Screen::Pack)
            .expect("the pack's panel");
        let frame = children[0];
        let image = world.get::<ImageNode>(frame).unwrap();
        assert_eq!(image.image, rock);
        assert!(
            matches!(image.image_mode, NodeImageMode::Stretch),
            "drawn whole"
        );
        let node = world.get::<Node>(frame).unwrap();
        assert_eq!((node.width, node.height), (px(268), px(152)));
        let mut images = world.query::<&ImageNode>();
        assert_eq!(
            images.iter(world).filter(|i| i.image == lintel).count(),
            1,
            "one oak lintel"
        );
        let mut cells = world.query_filtered::<&ImageNode, With<PackCell>>();
        let n = cells.iter(world).count();
        assert_eq!(n, PACK + SLOTS);
        assert!(cells.iter(world).all(|i| slots.contains(&i.image)));
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
}
