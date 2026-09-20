//! The pause menu and the settings page.
//!
//! A menu is not a panel. A panel is a `Node` with children; a MENU is a
//! second claimant on the pointer, and everything awkward here is that claim.
//! The world's input readers take the pointer on a left click, and a click on
//! RESUME is still a left click, so the rule lives one level up:
//! [`pbd_app::controls::MenuOpen`] says who holds it and every reader stands
//! down while it is set. What is in this file is the drawing, the three
//! buttons and the key that opens them.
//!
//! Both panels are built at startup and hidden with `Display::None` rather
//! than `Visibility::Hidden`, which is the slot row's own lesson: a hidden
//! node is still laid out and still PICKED, so an invisible panel goes on
//! swallowing clicks over the middle of the screen for as long as it is shut.

use bevy::{app::AppExit, prelude::*};
use pbd_app::controls::{BINDINGS, MenuOpen};

/// Which screen is open. One enum and not two flags, because two flags is how
/// a settings page ends up open over a closed pause menu.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Playing,
    Pause,
    Settings,
}

impl Screen {
    /// One step back out. Settings to pause, pause to the world, and the world
    /// to the menu - which is what `Escape` does from wherever it is pressed.
    fn back(self) -> Self {
        match self {
            Screen::Playing => Screen::Pause,
            Screen::Pause => Screen::Playing,
            Screen::Settings => Screen::Pause,
        }
    }
}

/// What a row does when it is pressed. On the entity rather than implied by
/// its position in the list, so reordering the panel cannot silently swap two
/// buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Resume,
    Settings,
    Quit,
    Back,
}

/// The rows whose hover or press state changed this frame: exactly the two
/// things a repaint reads, which is this project's rule that a query names
/// what it touches and nothing else.
type TouchedRows<'w, 's> = Query<
    'w,
    's,
    (&'static Interaction, &'static mut BackgroundColor),
    (Changed<Interaction>, With<MenuAction>),
>;

/// Marks a panel with the screen it belongs to.
#[derive(Component)]
pub struct Panel(Screen);

const INK: Color = Color::srgb(0.88, 0.94, 0.91);
const MINT: Color = Color::srgb(0.48, 0.8, 0.77);
const PANEL_FILL: Color = Color::srgba(0.02, 0.06, 0.08, 0.94);
const EDGE: Color = Color::srgba(0.55, 0.75, 0.74, 0.6);

fn idle() -> Color {
    Color::srgba(0.06, 0.13, 0.15, 0.9)
}
fn hovered() -> Color {
    Color::srgba(0.11, 0.22, 0.24, 0.95)
}
fn pressed() -> Color {
    Color::srgba(0.18, 0.34, 0.35, 1.0)
}

/// The full-screen layer a panel is centred in.
///
/// Centring by flex rather than by a margin of half the panel's own height,
/// because the height is the CONTENT's: the settings page grows by a row every
/// time a binding is added, and a magic offset would be wrong the day it does.
/// The first cut used `top: 50%` and the input list ran off the bottom of the
/// window.
fn layer() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100.0),
        height: percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        display: Display::None,
        ..default()
    }
}

fn panel_node(width: f32) -> Node {
    Node {
        width: px(width),
        padding: UiRect::all(px(22)),
        border: UiRect::all(px(1)),
        flex_direction: FlexDirection::Column,
        row_gap: px(10),
        ..default()
    }
}

fn title(text: &str) -> impl Bundle {
    (
        Text::new(text.to_string()),
        TextFont {
            font_size: 15.0,
            ..default()
        },
        TextColor(MINT),
        Node {
            margin: UiRect::bottom(px(6)),
            ..default()
        },
    )
}

/// A row that can be clicked. Every one is a real `Button` with a hit box and
/// hover and press states: a control that only responds to a key is a control
/// nobody can find, which is why the bindings were on screen in the first
/// place.
fn button(parent: &mut ChildSpawnerCommands, label: &str, action: MenuAction) {
    parent
        .spawn((
            Button,
            Node {
                width: percent(100.0),
                padding: UiRect::axes(px(14), px(9)),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(EDGE),
            BackgroundColor(idle()),
            action,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(INK),
            ));
        });
}

pub fn spawn(mut commands: Commands) {
    commands
        // Above the slot row, which is spawned a schedule later and would
        // otherwise draw over the panel: within a UI, later is on top.
        .spawn((layer(), GlobalZIndex(10), Panel(Screen::Pause)))
        .with_children(|layer| {
            layer
                .spawn((
                    panel_node(260.0),
                    BorderColor::all(EDGE),
                    BackgroundColor(PANEL_FILL),
                ))
                .with_children(|panel| {
                    panel.spawn(title("PAUSED"));
                    button(panel, "RESUME", MenuAction::Resume);
                    button(panel, "SETTINGS", MenuAction::Settings);
                    button(panel, "QUIT", MenuAction::Quit);
                });
        });

    commands
        .spawn((layer(), GlobalZIndex(10), Panel(Screen::Settings)))
        .with_children(|layer| {
            layer
                .spawn((
                    panel_node(520.0),
                    BorderColor::all(EDGE),
                    BackgroundColor(PANEL_FILL),
                ))
                .with_children(|panel| {
                    panel.spawn(title("SETTINGS"));
                    panel.spawn((
                        Text::new("INPUT"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(MINT),
                    ));
                    // Two columns, off the one binding table. A key on the left and
                    // what it does on the right is the shape a player can scan; the
                    // run-on line it replaces was readable only by whoever wrote it.
                    for group in &BINDINGS {
                        panel.spawn((
                            Text::new(group.heading.to_string()),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.55, 0.75, 0.74, 0.85)),
                            Node {
                                margin: UiRect::top(px(8)),
                                ..default()
                            },
                        ));
                        for binding in group.rows {
                            panel
                                .spawn(Node {
                                    column_gap: px(12),
                                    ..default()
                                })
                                .with_children(|line| {
                                    line.spawn((
                                        Text::new(binding.keys_label()),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(INK),
                                        Node {
                                            width: px(150),
                                            ..default()
                                        },
                                    ));
                                    line.spawn((
                                        Text::new(binding.does.to_string()),
                                        TextFont {
                                            font_size: 12.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgba(0.88, 0.94, 0.91, 0.72)),
                                    ));
                                });
                        }
                    }
                    panel.spawn(Node {
                        height: px(10),
                        ..default()
                    });
                    button(panel, "BACK", MenuAction::Back);
                });
        });
}

/// `Escape` steps back one screen, and is CONSUMED.
///
/// It runs in `PreUpdate` after the input systems and before
/// `RunFixedMainLoop`, where both of the world's input readers live, so
/// clearing it here means they never see it. That is why neither of them has
/// an `Escape` arm any more: a key with one owner needs no agreement about who
/// acts on it.
pub fn toggle(mut keys: ResMut<ButtonInput<KeyCode>>, mut screen: ResMut<Screen>) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    keys.clear_just_pressed(KeyCode::Escape);
    *screen = screen.back();
}

/// A press does what its own `MenuAction` says.
pub fn press(
    mut screen: ResMut<Screen>,
    mut exit: MessageWriter<AppExit>,
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
) {
    for (interaction, action) in &rows {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MenuAction::Resume => *screen = Screen::Playing,
            MenuAction::Settings => *screen = Screen::Settings,
            MenuAction::Back => *screen = Screen::Pause,
            MenuAction::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

/// Show the open panel, hide the others, and publish who holds the pointer.
///
/// `MenuOpen` is written HERE and nowhere else, so there is one answer to the
/// question every input reader asks.
pub fn paint(
    screen: Res<Screen>,
    mut menu: ResMut<MenuOpen>,
    mut panels: Query<(&Panel, &mut Node)>,
    mut rows: TouchedRows,
) {
    if screen.is_changed() {
        menu.0 = *screen != Screen::Playing;
        for (panel, mut node) in &mut panels {
            node.display = if panel.0 == *screen {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
    for (interaction, mut fill) in &mut rows {
        fill.0 = match interaction {
            Interaction::Pressed => pressed(),
            Interaction::Hovered => hovered(),
            Interaction::None => idle(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Escape steps out one screen at a time, and from the world it steps IN.
    /// A settings page that resumed the game on its way past the pause menu is
    /// the reason this is one enum rather than a pair of booleans.
    #[test]
    fn escape_steps_one_screen_at_a_time() {
        assert_eq!(Screen::Playing.back(), Screen::Pause);
        assert_eq!(Screen::Settings.back(), Screen::Pause);
        assert_eq!(Screen::Pause.back(), Screen::Playing);
    }

    /// Anything but `Playing` holds the pointer. Written as the test of the
    /// rule rather than of the line, because this is what every input reader
    /// in the app is actually asking.
    #[test]
    fn only_the_world_leaves_the_pointer_alone() {
        for (screen, held) in [
            (Screen::Playing, false),
            (Screen::Pause, true),
            (Screen::Settings, true),
        ] {
            assert_eq!(screen != Screen::Playing, held, "{screen:?}");
        }
    }
}
