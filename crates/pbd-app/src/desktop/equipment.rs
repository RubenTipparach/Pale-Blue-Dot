//! The tool slot: one square left of the hotbar holding the tool in hand, and
//! the picker that opens beside it while G is held.
//!
//! Holding G on foot opens the picker, the wheel moves its highlight, and
//! letting go puts the highlighted tool in hand. The picker's first entry is
//! bare hands, which a new world starts with (`inventory-grid` decision 8). G is read once, in
//! `controls::read_picker_key`, and read here: the picker never looks at the
//! key itself. Boarding a craft is F, a key of its own.
//!
//! The tool slot is not one of the ten hotbar slots. Those carry what the
//! player gathers; this one carries the tool the left button uses, which is
//! why the rod can never be dropped by filling the hotbar with fish.

use super::slots::ItemIcons;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use pbd_app::controls::PickerKey;
use pbd_app::fish::ToolSlot;
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::Localized;
use pbd_app::ui_kit::{self, KitPiece, KitText, Set, Tooltip, UiKit};
use pbd_core::inventory::{Equipment, Tool, tool_title};

/// The picker's highlight while G is held: a tool, or `None` for bare hands.
/// Separate from the tool in hand, so wheeling past a tool does not put it in
/// hand: only letting go does.
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Picker {
    pub highlight: Option<Tool>,
}

/// The picker's claim on the wheel, for the one system that reads the wheel.
#[derive(SystemParam)]
pub struct PickerWheel<'w> {
    key: Option<Res<'w, PickerKey>>,
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

const SIZE: f32 = super::slots::SLOT;
/// The hotbar's own width: ten 44 px slots four apart.
const HOTBAR: f32 = 10.0 * super::slots::SLOT;
/// How far left of the hotbar the tool slot stands.
const APART: f32 = 4.0;
/// The lit row of the picker: the rune's dim step under it.
fn lit_row() -> Color {
    ui_kit::RUNE_DIM.with_alpha(0.45)
}

#[derive(Component)]
pub struct ToolIcon;

#[derive(Component)]
pub struct PickerColumn;

/// A row of the picker, for the tool it names, or `None` for bare hands.
#[derive(Component)]
pub struct PickerRow(pub Option<Tool>);

#[derive(Component)]
pub struct PickerIcon;

/// The hand cell's square and its tool picture: the HUD's, left of the
/// hotbar, and the pack's, over its HOTBAR row (`pack-tool-slot` decision 2).
/// Both carry `ToolSlotCell` and `ToolIcon`, so [`paint`] paints both.
pub fn tool_cell(kit: &UiKit, node: Node, extra: impl Bundle) -> impl Bundle {
    (
        Node {
            width: px(SIZE),
            height: px(SIZE),
            flex_shrink: 0.0,
            ..node
        },
        kit.whole(Set::Stone, "slot-idle"),
        Interaction::default(),
        Tooltip(Localized::legacy(hand_tip(&Equipment::new_world()))),
        ToolSlotCell,
        extra,
    )
}

/// The tool's picture inside a [`tool_cell`].
pub fn tool_icon(icons: &ItemIcons) -> impl Bundle {
    (
        ImageNode::new(icons.in_hand(None)),
        Node {
            position_type: PositionType::Absolute,
            left: px(4),
            top: px(4),
            width: px(16),
            height: px(16),
            ..default()
        },
        ToolIcon,
    )
}

/// The pack's hand cell (`pack-tool-slot` decision 3).
#[derive(Component)]
pub struct PackToolCell;

/// The list a click on the pack's hand cell opens: bare hands, then each
/// tool owned (`pack-tool-slot` decision 4).
#[derive(Component)]
pub struct HandList;

/// A row of that list, for the tool it holds, or `None` for bare hands.
#[derive(Component)]
pub struct HandListRow(pub Option<Tool>);

/// Whether the pack's hand list is open.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandListOpen(pub bool);

/// The pack's hand cell and its (hidden) list, beside the HOTBAR label.
pub fn spawn_pack_hand(parent: &mut ChildSpawnerCommands, kit: &UiKit, icons: &ItemIcons) {
    parent
        .spawn(tool_cell(kit, Node::default(), (Button, PackToolCell)))
        .with_children(|cell| {
            cell.spawn(tool_icon(icons));
        });
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(SIZE + 4.0),
                bottom: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(1),
                padding: UiRect::all(px(14)),
                display: Display::None,
                ..default()
            },
            kit.nine(Set::Stone, "frame"),
            KitPiece("frame".into()),
            GlobalZIndex(15),
            HandList,
        ))
        .with_children(|list| {
            let entries = std::iter::once(None).chain(Tool::ALL.into_iter().map(Some));
            for tool in entries {
                list.spawn((
                    Button,
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: px(4),
                        padding: UiRect::axes(px(3), px(2)),
                        width: px(110),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    HandListRow(tool),
                ))
                .with_children(|row| {
                    row.spawn((
                        ImageNode::new(icons.in_hand(tool)),
                        Node {
                            width: px(16),
                            height: px(16),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        PickerIcon,
                        PickerRow(tool),
                    ));
                    row.spawn((
                        KitText::new(
                            Localized::legacy(tool.map_or(BARE, |t| t.name()).to_uppercase()),
                            ui_kit::TITLE,
                        )
                        .shadow(ui_kit::INK),
                        PickerName(tool),
                    ));
                });
            }
        });
}

/// A click on the pack's hand cell opens or shuts its list; a click on a
/// row puts that tool in hand, saved as the G picker saves it.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn pack_hand(
    screen: Res<super::menu::Screen>,
    mut open: ResMut<HandListOpen>,
    mut tools: ResMut<ToolSlot>,
    mut save: Option<ResMut<WorldSave>>,
    cell: Query<&Interaction, (Changed<Interaction>, With<PackToolCell>)>,
    rows: Query<(&Interaction, &HandListRow), Changed<Interaction>>,
    mut list: Query<&mut Node, With<HandList>>,
    mut shown: Query<(&HandListRow, &mut Node, &mut BackgroundColor), Without<HandList>>,
) {
    let mut want = open.0 && *screen == super::menu::Screen::Pack;
    if cell.iter().any(|i| *i == Interaction::Pressed) {
        want = !want;
    }
    for (interaction, row) in &rows {
        if *interaction == Interaction::Pressed {
            want = false;
            if tools.hold(row.0) {
                if let Some(save) = save.as_deref_mut() {
                    save.record_hand(&tools);
                }
                info!("in hand: {}", hand_tip(&tools));
            }
        }
    }
    if open.0 != want {
        open.0 = want;
    }
    for mut node in &mut list {
        let display = if want { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
    if !want {
        return;
    }
    for (row, mut node, mut fill) in &mut shown {
        let display = if row.0.is_none_or(|t| tools.owns(t)) {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        let lit = row.0 == tools.held();
        let color = if lit { lit_row() } else { Color::NONE };
        if fill.0 != color {
            fill.0 = color;
        }
    }
}

/// Build the tool slot and the (hidden) picker above it. `PostStartup` beside
/// the hotbar, after the icons are loaded.
pub fn spawn(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    kit: Res<UiKit>,
    existing: Query<(), With<ToolIcon>>,
) {
    if !existing.is_empty() {
        return;
    }
    let left = -HOTBAR / 2.0 - APART - SIZE;
    // Hovered while the pack has the pointer: what is in hand.
    commands
        .spawn(tool_cell(
            &kit,
            Node {
                position_type: PositionType::Absolute,
                bottom: px(super::slots::HOTBAR_BOTTOM),
                left: percent(50),
                margin: UiRect::left(px(left)),
                ..default()
            },
            super::menu::InPlayHud,
        ))
        .with_children(|cell| {
            cell.spawn(tool_icon(&icons));
        });
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(super::slots::HOTBAR_BOTTOM + SIZE + 6.0),
                left: percent(50),
                margin: UiRect::left(px(left)),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                padding: UiRect::all(px(14)),
                display: Display::None,
                ..default()
            },
            kit.nine(Set::Stone, "frame"),
            KitPiece("frame".into()),
            GlobalZIndex(5),
            PickerColumn,
        ))
        .with_children(|column| {
            let entries = std::iter::once(None).chain(Tool::ALL.into_iter().map(Some));
            for tool in entries {
                let (name, purpose) =
                    tool.map_or((BARE, BARE_PURPOSE), |t| (t.name(), t.purpose()));
                column
                    .spawn((
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: px(4),
                            padding: UiRect::axes(px(3), px(2)),
                            width: px(140),
                            ..default()
                        },
                        BackgroundColor(Color::NONE),
                        PickerRow(tool),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            ImageNode::new(icons.in_hand(tool)),
                            Node {
                                width: px(16),
                                height: px(16),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            PickerIcon,
                            PickerRow(tool),
                        ));
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(1),
                            ..default()
                        })
                        .with_children(|words| {
                            words.spawn((
                                KitText::new(Localized::legacy(name.to_uppercase()), ui_kit::TITLE)
                                    .shadow(ui_kit::INK),
                                PickerName(tool),
                            ));
                            words.spawn((
                                Text::new(purpose),
                                TextFont {
                                    font_size: 9.0,
                                    ..default()
                                },
                                TextColor(ui_kit::TEXT_DIM),
                            ));
                        });
                    });
            }
            column.spawn((
                Text::new("wheel to choose, let go of G"),
                TextFont {
                    font_size: 9.0,
                    ..default()
                },
                TextColor(ui_kit::TEXT_DIM),
            ));
        });
}

pub fn pick(
    key: Res<PickerKey>,
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
        info!("in hand: {}", hand_tip(&tools));
    }
}

/// The tool slot itself, which carries its tooltip.
#[derive(Component)]
pub struct ToolSlotCell;

/// Bare hands, in the picker and the tool slot's tooltip.
const BARE: &str = "Bare hands";
/// What they do (`tool-crafting` decision 1).
const BARE_PURPOSE: &str = "digs soft ground";

/// A tool's name as the picker shows it: with its tier ("Iron pickaxe").
pub fn tool_name(equipment: &Equipment, tool: Option<Tool>) -> String {
    match tool {
        None => BARE.to_string(),
        Some(tool) => equipment
            .tier(tool)
            .map_or_else(|| tool.name().to_string(), |tier| tool_title(tool, tier)),
    }
}

/// What the tool slot's tooltip says: the tool in hand, with its tier, and
/// what it is for.
pub fn hand_tip(equipment: &Equipment) -> String {
    let purpose = equipment.held().map_or(BARE_PURPOSE, Tool::purpose);
    format!("{}: {purpose}", tool_name(equipment, equipment.held()))
}

/// The name line of a picker row, renamed when its tool's tier changes.
#[derive(Component)]
pub struct PickerName(pub Option<Tool>);

/// Paint the tool slot and the picker from the resources.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn paint(
    key: Res<PickerKey>,
    tools: Res<ToolSlot>,
    picker: Res<Picker>,
    icons: Res<ItemIcons>,
    mut slot: Query<&mut ImageNode, (With<ToolIcon>, Without<PickerIcon>)>,
    mut column: Query<&mut Node, With<PickerColumn>>,
    mut rows: Query<
        (&PickerRow, &mut Node, &mut BackgroundColor),
        (Without<PickerColumn>, Without<PickerIcon>),
    >,
    mut tips: Query<&mut Tooltip, With<ToolSlotCell>>,
    mut names: Query<(&PickerName, &mut KitText)>,
    mut picker_icons: Query<(&PickerRow, &mut ImageNode), (With<PickerIcon>, Without<ToolIcon>)>,
) {
    for mut tip in &mut tips {
        let want = hand_tip(&tools);
        if tip.0 != want {
            tip.0 = Localized::legacy(want);
        }
    }
    for mut image in &mut slot {
        let want = icons.in_hand_at(&tools);
        if image.image != want {
            image.image = want;
        }
    }
    if tools.is_changed() {
        for (name, mut text) in &mut names {
            let want = tool_name(&tools, name.0).to_uppercase();
            if text.text != want {
                text.text = Localized::legacy(want);
            }
        }
        for (row, mut image) in &mut picker_icons {
            let want = match row.0 {
                Some(tool) => tools
                    .tier(tool)
                    .map_or_else(|| icons.tool(tool), |tier| icons.tool_at(tool, tier)),
                None => icons.hand.clone(),
            };
            if image.image != want {
                image.image = want;
            }
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
    for (row, mut node, mut fill) in &mut rows {
        node.display = if row.0.is_none_or(|t| tools.owns(t)) {
            Display::Flex
        } else {
            Display::None
        };
        let lit = row.0 == picker.highlight;
        fill.0 = if lit { lit_row() } else { Color::NONE };
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
            .insert_resource(PickerKey {
                holding: true,
                ..default()
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
        assert_eq!(
            app.world().resource::<Picker>().highlight,
            Some(Tool::Rod),
            "one notch from bare hands is the first tool"
        );
        assert_eq!(app.world().resource::<Hotbar>().selected(), before);
        app.world_mut().resource_mut::<PickerKey>().holding = false;
        notch(&mut app);
        assert_eq!(app.world().resource::<Picker>().highlight, Some(Tool::Rod));
        assert_ne!(app.world().resource::<Hotbar>().selected(), before);
    }
}
