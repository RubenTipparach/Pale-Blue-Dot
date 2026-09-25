//! The tool slot: one square left of the hotbar holding the tool in hand, and
//! the picker that opens beside it while G is held.
//!
//! A tap of G still boards a craft; holding it on foot opens the picker, the
//! wheel moves its highlight, and letting go puts the highlighted tool in hand.
//! What G is doing is decided once, in `controls::read_interact_key`, and read
//! here: the picker never looks at the key itself.
//!
//! The tool slot is not one of the ten hotbar slots. Those carry what the
//! player gathers; this one carries the tool the left button uses, which is
//! why the rod can never be dropped by filling the hotbar with fish.

use super::slots::ItemIcons;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use pbd_app::controls::InteractKey;
use pbd_app::fish::ToolSlot;
use pbd_app::saves::WorldSave;
use pbd_core::inventory::Tool;

/// The picker's highlight while G is held. Separate from the tool in hand, so
/// wheeling past a tool does not put it in hand: only letting go does.
#[derive(Resource, Clone, Copy, Debug)]
pub struct Picker {
    pub highlight: Tool,
}

impl Default for Picker {
    fn default() -> Self {
        Self {
            highlight: Tool::Rod,
        }
    }
}

/// The picker's claim on the wheel, for the one system that reads the wheel.
#[derive(SystemParam)]
pub struct PickerWheel<'w> {
    key: Option<Res<'w, InteractKey>>,
    tools: Res<'w, ToolSlot>,
    picker: ResMut<'w, Picker>,
}

impl PickerWheel<'_> {
    /// Hand this frame's wheel steps to the picker if it is open. `true`
    /// when it was, and nothing else may act on the wheel.
    pub fn take(&mut self, step: i32) -> bool {
        if !self.key.as_ref().is_some_and(|key| key.holding) {
            return false;
        }
        if step != 0 {
            self.picker.highlight = self.tools.step_from(self.picker.highlight, step);
        }
        true
    }
}

const SIZE: f32 = 52.0;
/// The hotbar's own width: ten 44 px slots four apart.
const HOTBAR: f32 = 10.0 * 48.0 - 4.0;
/// How far left of the hotbar the tool slot stands.
const APART: f32 = 14.0;
const AMBER: Color = Color::srgb(0.96, 0.72, 0.30);
const AMBER_DIM: Color = Color::srgba(0.96, 0.72, 0.30, 0.45);

#[derive(Component)]
pub struct ToolIcon;

#[derive(Component)]
pub struct PickerColumn;

/// A row of the picker, for the tool it names.
#[derive(Component)]
pub struct PickerRow(pub Tool);

#[derive(Component)]
pub struct PickerIcon;

/// Build the tool slot and the (hidden) picker above it. `PostStartup` beside
/// the hotbar, after the icons are loaded.
pub fn spawn(mut commands: Commands, icons: Res<ItemIcons>, existing: Query<(), With<ToolIcon>>) {
    if !existing.is_empty() {
        return;
    }
    let left = -HOTBAR / 2.0 - APART - SIZE;
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: px(18),
            left: percent(50),
            margin: UiRect::left(px(left)),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(3),
            ..default()
        })
        .with_children(|column| {
            column
                .spawn((
                    Node {
                        width: px(SIZE),
                        height: px(SIZE),
                        border: UiRect::all(px(2)),
                        padding: UiRect::all(px(6)),
                        ..default()
                    },
                    BorderColor::all(AMBER),
                    BackgroundColor(Color::srgba(0.08, 0.06, 0.03, 0.85)),
                ))
                .with_children(|cell| {
                    cell.spawn((
                        ImageNode::new(icons.tool(Tool::Rod)),
                        Node {
                            width: percent(100.0),
                            height: percent(100.0),
                            ..default()
                        },
                        ToolIcon,
                    ));
                });
        });
    // The picker opens upward from the tool slot, one row per tool, so its
    // bottom sits just over the slot and the wheel runs along it.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(18.0 + SIZE + 10.0),
                left: percent(50),
                margin: UiRect::left(px(left)),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                padding: UiRect::all(px(6)),
                border: UiRect::all(px(1)),
                display: Display::None,
                ..default()
            },
            BorderColor::all(AMBER_DIM),
            BackgroundColor(Color::srgba(0.03, 0.05, 0.06, 0.92)),
            GlobalZIndex(5),
            PickerColumn,
        ))
        .with_children(|column| {
            for tool in Tool::ALL {
                column
                    .spawn((
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: px(8),
                            padding: UiRect::axes(px(6), px(4)),
                            border: UiRect::all(px(1)),
                            width: px(230),
                            ..default()
                        },
                        BorderColor::all(Color::NONE),
                        BackgroundColor(Color::NONE),
                        PickerRow(tool),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            ImageNode::new(icons.tool(tool)),
                            Node {
                                width: px(32),
                                height: px(32),
                                ..default()
                            },
                            PickerIcon,
                        ));
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .with_children(|words| {
                            words.spawn((
                                Text::new(tool.name().to_uppercase()),
                                TextFont {
                                    font_size: 13.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.92, 0.97, 0.95)),
                            ));
                            words.spawn((
                                Text::new(tool.purpose()),
                                TextFont {
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.62, 0.72, 0.72)),
                            ));
                        });
                    });
            }
            column.spawn((
                Text::new("wheel to choose, let go of G"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(AMBER_DIM),
            ));
        });
}

/// Open, steer and close the picker, and put the chosen tool in hand.
///
/// The wheel itself is read by `slots::input`, which owns the one wheel reader
/// and hands the step to the picker while G is held: two readers of one
/// message stream would each see every notch.
pub fn pick(
    key: Res<InteractKey>,
    mut tools: ResMut<ToolSlot>,
    mut picker: ResMut<Picker>,
    mut save: Option<ResMut<WorldSave>>,
    mut was_holding: Local<bool>,
) {
    if key.holding && !*was_holding {
        // Open on the tool in hand, so a hold and a release with no wheel
        // changes nothing.
        picker.highlight = tools.held();
    }
    *was_holding = key.holding;
    if key.let_go && tools.hold(picker.highlight) {
        // A tool changed hands: that is world state, so it is written now,
        // the same frame, like every other change a player makes.
        if let Some(save) = save.as_deref_mut() {
            save.record_hand(&tools);
        }
        info!("in hand: {}", tools.held().name());
    }
}

/// Paint the tool slot and the picker from the resources.
pub fn paint(
    key: Res<InteractKey>,
    tools: Res<ToolSlot>,
    picker: Res<Picker>,
    icons: Res<ItemIcons>,
    mut slot: Query<&mut ImageNode, (With<ToolIcon>, Without<PickerIcon>)>,
    mut column: Query<&mut Node, With<PickerColumn>>,
    mut rows: Query<
        (
            &PickerRow,
            &mut Node,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        Without<PickerColumn>,
    >,
) {
    for mut image in &mut slot {
        let want = icons.tool(tools.held());
        if image.image != want {
            image.image = want;
        }
    }
    let open = key.holding;
    for mut node in &mut column {
        let display = if open { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
    if !open {
        return;
    }
    for (row, mut node, mut border, mut fill) in &mut rows {
        node.display = if tools.owns(row.0) {
            Display::Flex
        } else {
            Display::None
        };
        let lit = row.0 == picker.highlight;
        *border = BorderColor::all(if lit { AMBER } else { Color::NONE });
        fill.0 = if lit {
            Color::srgba(0.30, 0.20, 0.06, 0.6)
        } else {
            Color::NONE
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
    use pbd_app::hotbar::Hotbar;

    /// With G held the wheel steps the picker's highlight and leaves the item
    /// slot where it was; with G up the same notch steps the item slot.
    #[test]
    fn the_open_picker_takes_the_wheel() {
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .add_message::<MouseWheel>()
            .insert_resource(pbd_app::walking::WalkingReadout {
                active: true,
                ..default()
            })
            .insert_resource(pbd_app::controls::MenuOpen(false))
            .insert_resource({
                let mut key = InteractKey::default();
                key.holding = true;
                key
            })
            .init_resource::<ToolSlot>()
            .init_resource::<Picker>()
            .init_resource::<Hotbar>()
            .add_systems(Update, super::super::slots::input);
        let notch = |app: &mut App| {
            app.world_mut().write_message(MouseWheel {
                unit: MouseScrollUnit::Line,
                x: 0.0,
                y: -1.0,
                window: Entity::PLACEHOLDER,
            });
            app.update();
        };
        let before = app.world().resource::<Hotbar>().selected();
        notch(&mut app);
        assert_eq!(app.world().resource::<Picker>().highlight, Tool::Shovel);
        assert_eq!(app.world().resource::<Hotbar>().selected(), before);
        app.world_mut().resource_mut::<InteractKey>().holding = false;
        notch(&mut app);
        assert_eq!(app.world().resource::<Picker>().highlight, Tool::Shovel);
        assert_ne!(app.world().resource::<Hotbar>().selected(), before);
    }
}
