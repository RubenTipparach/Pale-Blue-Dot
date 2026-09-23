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

use bevy::{
    app::AppExit,
    ecs::system::SystemParam,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use pbd_app::controls::{BINDINGS, MenuOpen};
use pbd_app::saves::{self, Slot, WorldSave};

/// Which screen is open. One enum and not two flags, because two flags is how
/// a settings page ends up open over a closed pause menu.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Playing,
    Pause,
    Settings,
    Saves,
}

impl Screen {
    /// One step back out. Settings to pause, pause to the world, and the world
    /// to the menu - which is what `Escape` does from wherever it is pressed.
    ///
    /// The saves page is the FRONT DOOR when the game opened on it: stepping
    /// back from there goes into the open world, since a returning player
    /// who presses nothing means "continue", and goes nowhere at all when no
    /// world is open yet, since there is nothing to continue into.
    fn back(self, front_door: bool, has_world: bool) -> Self {
        if self == Screen::Saves && front_door {
            return if has_world {
                Screen::Playing
            } else {
                Screen::Saves
            };
        }
        match self {
            Screen::Playing => Screen::Pause,
            Screen::Pause => Screen::Playing,
            Screen::Settings | Screen::Saves => Screen::Pause,
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
    Saves,
    Quit,
    Back,
    /// Open the slot at this row.
    Load(usize),
    /// Ask about the slot at this row. A delete is not done on one press.
    Ask(usize),
    /// Do it.
    Delete(usize),
    /// Do not.
    Keep,
    New,
    /// Put the keyboard in the name field.
    EditName,
    /// Set the storm forcing to this percentage: a weather preset.
    Weather(u8),
}

/// Whether the saves page is the screen the game opened on and no world has
/// been chosen from it yet. Cleared by CONTINUE, LOAD and NEW WORLD, so a
/// later visit through the pause menu steps back to the pause menu.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrontDoor(pub bool);

/// The screen a launch opens on: what `--menu` names, else the saves page for
/// a plain desktop launch, else the world. A launch that names its world, or
/// takes a capture, does not want a menu in front of it.
pub fn opening_screen(menu: Option<&str>, world_named: bool, capture: bool) -> Screen {
    match menu {
        Some("pause") => Screen::Pause,
        Some("settings") => Screen::Settings,
        Some("saves") => Screen::Saves,
        _ if !world_named && !capture => Screen::Saves,
        _ => Screen::Playing,
    }
}

/// The longest name a world can be given. `saves::slot_id` derives a legal
/// directory from anything; this only keeps the row readable.
pub const NAME_MAX: usize = 32;

/// The name field on the saves page: what has been typed, whether the
/// keyboard is in it, and whether Enter asked for the world to be made.
#[derive(Resource, Default, Clone, PartialEq, Eq, Debug)]
pub struct NameField {
    pub text: String,
    pub focused: bool,
    /// The field was typed in, so the page's prefill leaves it alone.
    pub edited: bool,
    /// Enter was pressed in the field; `press` makes the world.
    pub submit: bool,
}

impl NameField {
    /// One typed key. Characters and spaces go in up to `NAME_MAX`, Backspace
    /// takes one off, and anything else is not text.
    pub fn typed(&mut self, key: &Key) {
        match key {
            Key::Character(text) => {
                for c in text.chars().filter(|c| !c.is_control()) {
                    if self.text.chars().count() < NAME_MAX {
                        self.text.push(c);
                        self.edited = true;
                    }
                }
            }
            Key::Space => {
                if self.text.chars().count() < NAME_MAX {
                    self.text.push(' ');
                    self.edited = true;
                }
            }
            Key::Backspace => {
                self.text.pop();
                self.edited = true;
            }
            _ => {}
        }
    }

    /// What the page shows in the field: the text, and a caret while the
    /// keyboard is in it.
    pub fn shown(&self) -> String {
        if self.focused {
            format!("{}_", self.text)
        } else {
            self.text.clone()
        }
    }

    /// The name a world would be made with: what was typed, or the number
    /// the page offered.
    pub fn name(&self, worlds: usize) -> String {
        let typed = self.text.trim();
        if typed.is_empty() {
            format!("World {}", worlds + 1)
        } else {
            typed.to_string()
        }
    }

    /// Offer the next number unless the player has typed something.
    pub fn prefill(&mut self, worlds: usize) {
        if !self.edited {
            self.text = format!("World {}", worlds + 1);
        }
    }
}

/// The name field's text node.
#[derive(Component)]
pub struct NameText;

/// The slots as the screen last listed them, and which one is being asked
/// about.
///
/// The list is a RESOURCE rather than read off the disk while drawing, because
/// a row's button carries an index into it: reading the directory twice, once
/// to draw and once to act, is two lists that can disagree about what row
/// three is.
#[derive(Resource, Default)]
pub struct SaveIndex {
    pub slots: Vec<Slot>,
    pub asking: Option<usize>,
    /// Why the last load or delete did not happen.
    pub trouble: Option<String>,
}

/// A load the next frame will carry out, since it touches more of the world
/// than a button press can borrow at once.
#[derive(Resource, Default)]
pub struct LoadRequest(pub Option<Slot>);

/// The saves page's own state, which a press reads and writes together: the
/// list and what is asked about it, the load it asked for, whether it is the
/// front door, and the name field.
#[derive(SystemParam)]
pub struct SavesPage<'w> {
    pub index: ResMut<'w, SaveIndex>,
    pub load: ResMut<'w, LoadRequest>,
    pub front: ResMut<'w, FrontDoor>,
    pub name: ResMut<'w, NameField>,
}

/// The part of the saves panel that is rebuilt when the list changes.
#[derive(Component)]
pub struct SaveList;

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
pub(super) const MINT: Color = Color::srgb(0.48, 0.8, 0.77);
const PANEL_FILL: Color = Color::srgba(0.02, 0.06, 0.08, 0.94);
pub(super) const EDGE: Color = Color::srgba(0.55, 0.75, 0.74, 0.6);

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
    spawn_saves(&mut commands);
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
                    super::weather_ui::spawn(panel);
                    button(panel, "RESUME", MenuAction::Resume);
                    button(panel, "SAVES", MenuAction::Saves);
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

/// The saves panel: a title, the list (rebuilt when it changes), and BACK.
fn spawn_saves(commands: &mut Commands) {
    commands
        .spawn((layer(), GlobalZIndex(10), Panel(Screen::Saves)))
        .with_children(|layer| {
            layer
                .spawn((
                    panel_node(520.0),
                    BorderColor::all(EDGE),
                    BackgroundColor(PANEL_FILL),
                ))
                .with_children(|panel| {
                    panel.spawn(title("WORLDS"));
                    panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            ..default()
                        },
                        SaveList,
                    ));
                    panel.spawn(Node {
                        height: px(10),
                        ..default()
                    });
                    button(panel, "BACK", MenuAction::Back);
                });
        });
}

/// A line of small text, for a slot's date or a reason something did not
/// happen.
fn note(text: String, colour: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(colour),
    )
}

/// How long ago, in words a player reads rather than a timestamp.
fn since(then: u64) -> String {
    let now = saves::now_unix_s();
    let seconds = now.saturating_sub(then);
    match seconds {
        0..=90 => "just now".into(),
        s if s < 5400 => format!("{} min ago", s / 60),
        s if s < 172_800 => format!("{} hours ago", s / 3600),
        s => format!("{} days ago", s / 86_400),
    }
}

/// Draw the list. Rebuilt rather than updated, because the list changes SHAPE
/// when a world is made or deleted and a row's button carries its index.
pub fn rebuild_saves(
    mut commands: Commands,
    index: Res<SaveIndex>,
    name: Res<NameField>,
    open: Option<Res<WorldSave>>,
    lists: Query<Entity, With<SaveList>>,
) {
    if !index.is_changed() {
        return;
    }
    let playing = open
        .as_ref()
        .and_then(|save| save.slot().map(|slot| slot.id.clone()));
    for list in &lists {
        commands.entity(list).despawn_related::<Children>();
        commands.entity(list).with_children(|rows| {
            if let Some(trouble) = index.trouble.as_ref() {
                rows.spawn(note(trouble.clone(), Color::srgb(0.95, 0.66, 0.45)));
            }
            // A delete REPLACES the list while it is pending, and says what
            // is lost rather than only asking. Tenebris's own dialog, ported
            // for its reason: a row that says "delete?" beside nine other
            // rows is a question a player answers without reading it.
            if let Some(row) = index.asking
                && let Some(slot) = index.slots.get(row)
            {
                rows.spawn((
                    Text::new(format!("Delete '{}'?", slot.file.name)),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.66, 0.45)),
                ));
                rows.spawn(note(
                    "Everything dug, built and carried in it is gone for good.".into(),
                    Color::srgba(0.88, 0.94, 0.91, 0.7),
                ));
                rows.spawn(Node {
                    height: px(8),
                    ..default()
                });
                rows.spawn(Node {
                    column_gap: px(10),
                    ..default()
                })
                .with_children(|line| {
                    small(line, "DELETE FOREVER", MenuAction::Delete(row));
                    small(line, "CANCEL", MenuAction::Keep);
                });
                return;
            }
            if index.slots.is_empty() {
                rows.spawn(note(
                    "no worlds yet".into(),
                    Color::srgba(0.88, 0.94, 0.91, 0.6),
                ));
            }
            for (row, slot) in index.slots.iter().enumerate() {
                let here = Some(&slot.id) == playing.as_ref();
                rows.spawn(Node {
                    column_gap: px(10),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|line| {
                    line.spawn((
                        Text::new(if here {
                            format!("{}  (open)", slot.file.name)
                        } else {
                            slot.file.name.clone()
                        }),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(INK),
                        Node {
                            width: px(220),
                            ..default()
                        },
                    ));
                    line.spawn((
                        note(
                            since(slot.file.played_unix_s),
                            Color::srgba(0.88, 0.94, 0.91, 0.6),
                        ),
                        Node {
                            width: px(110),
                            ..default()
                        },
                    ));
                    if here {
                        small(line, "CONTINUE", MenuAction::Resume);
                    } else {
                        small(line, "LOAD", MenuAction::Load(row));
                    }
                    small(line, "DELETE", MenuAction::Ask(row));
                });
            }
            rows.spawn(Node {
                height: px(6),
                ..default()
            });
            // The name for a new world, typed in the game's own box: a
            // native prompt over the window is the thing the inherited UI
            // rule forbids. Prefilled with the number the page would have
            // chosen, so Enter alone still makes one.
            rows.spawn(Node {
                column_gap: px(10),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|line| {
                line.spawn((
                    note("NAME".into(), Color::srgba(0.55, 0.75, 0.74, 0.85)),
                    Node {
                        width: px(50),
                        ..default()
                    },
                ));
                line.spawn((
                    Button,
                    Node {
                        width: px(280),
                        padding: UiRect::axes(px(9), px(4)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(EDGE),
                    BackgroundColor(idle()),
                    MenuAction::EditName,
                ))
                .with_children(|field| {
                    field.spawn((
                        Text::new(name.shown()),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(INK),
                        NameText,
                    ));
                });
            });
            small_wide(rows, "NEW WORLD", MenuAction::New);
        });
    }
}

/// A button that sits in a row rather than filling the panel.
pub(super) fn small(parent: &mut ChildSpawnerCommands, label: &str, action: MenuAction) {
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(px(9), px(4)),
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
                    font_size: 11.0,
                    ..default()
                },
                TextColor(INK),
            ));
        });
}

fn small_wide(parent: &mut ChildSpawnerCommands, label: &str, action: MenuAction) {
    parent
        .spawn(Node { ..default() })
        .with_children(|line| small(line, label, action));
}

/// `Escape` steps back one screen, and is CONSUMED.
///
/// It runs in `PreUpdate` after the input systems and before
/// `RunFixedMainLoop`, where both of the world's input readers live, so
/// clearing it here means they never see it. That is why neither of them has
/// an `Escape` arm any more: a key with one owner needs no agreement about who
/// acts on it.
pub fn toggle(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    front: Res<FrontDoor>,
    save: Option<Res<WorldSave>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    keys.clear_just_pressed(KeyCode::Escape);
    let has_world = save.as_ref().is_some_and(|save| save.slot().is_some());
    let next = screen.back(front.0, has_world);
    if next != *screen {
        *screen = next;
    }
}

/// The keyboard while it is in the name field. Runs before `toggle` so that
/// Escape leaves the field rather than the page, and before the world's
/// readers, which stand down anyway while a menu is open.
pub fn name_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    mut name: ResMut<NameField>,
    mut texts: Query<&mut Text, With<NameText>>,
) {
    if !name.focused {
        typed.clear();
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        keys.clear_just_pressed(KeyCode::Escape);
        name.focused = false;
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        keys.clear_just_pressed(KeyCode::Enter);
        keys.clear_just_pressed(KeyCode::NumpadEnter);
        name.submit = true;
    }
    for event in typed.read() {
        if event.state.is_pressed() {
            name.typed(&event.logical_key);
        }
    }
    if name.is_changed() {
        for mut text in &mut texts {
            text.0 = name.shown();
        }
    }
}

/// A press does what its own `MenuAction` says.
///
/// The world-changing ones (load, delete, new) go through `SaveIndex` and
/// `LoadRequest` rather than acting here, because a press cannot borrow the
/// planet, the tier, the walker and the hotbar at once - and because a list
/// that is read to draw and read again to act is two lists.
pub fn press(
    mut screen: ResMut<Screen>,
    mut exit: MessageWriter<AppExit>,
    mut page: SavesPage,
    save: Option<Res<WorldSave>>,
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut forcing: ResMut<pbd_app::weather::StormForcing>,
) {
    let SavesPage {
        index,
        load,
        front,
        name,
    } = &mut page;
    let root = save.as_ref().map_or_else(
        || std::path::PathBuf::from(saves::ROOT),
        |s| s.root().into(),
    );
    let seed = pbd_app::planet::TERRAIN.seed;
    // Enter in the name field is the NEW WORLD button pressed.
    let submitted = std::mem::take(&mut name.submit);
    let pressed = rows
        .iter()
        .filter(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, action)| *action)
        .chain(submitted.then_some(MenuAction::New));
    for action in pressed {
        // A press anywhere but the field takes the keyboard out of it.
        if action != MenuAction::EditName && name.focused {
            name.focused = false;
        }
        match action {
            MenuAction::EditName => name.focused = true,
            MenuAction::Weather(percent) => forcing.0 = f32::from(percent.min(100)) / 100.0,
            MenuAction::Resume => {
                front.0 = false;
                *screen = Screen::Playing;
            }
            MenuAction::Settings => *screen = Screen::Settings,
            // Only the screen: what is ON it is filled by `paint`, from the
            // one place that knows the screen has changed. Filling it here
            // meant the list was empty for anything that opened the screen
            // another way, which is exactly what `--menu saves` found on its
            // first run - and is this project's own lesson about two paths to
            // one job, where the one nobody presses is the one that is wrong.
            MenuAction::Saves => *screen = Screen::Saves,
            MenuAction::Back => {
                let has_world = save.as_ref().is_some_and(|s| s.slot().is_some());
                *screen = screen.back(front.0, has_world);
            }
            MenuAction::Quit => {
                exit.write(AppExit::Success);
            }
            MenuAction::Ask(row) => {
                index.asking = Some(row);
                index.trouble = None;
            }
            MenuAction::Keep => index.asking = None,
            MenuAction::Delete(row) => {
                index.asking = None;
                let Some(slot) = index.slots.get(row).cloned() else {
                    continue;
                };
                let open = save
                    .as_ref()
                    .and_then(|s| s.slot().map(|s| s.id.clone()))
                    .is_some_and(|id| id == slot.id);
                if open {
                    // Deleting the world you are standing in would leave the
                    // writer appending to a directory that is not there.
                    index.trouble = Some("that world is open; load another first".into());
                } else {
                    index.trouble = saves::delete(&root, &slot.id)
                        .err()
                        .map(|error| format!("could not delete {}: {error}", slot.file.name));
                }
                index.slots = saves::list(&root);
            }
            MenuAction::New => {
                let wanted = name.name(index.slots.len());
                index.asking = None;
                match saves::create(&root, &wanted, seed) {
                    Ok(slot) => {
                        // Made, and entered: a world a player just named is
                        // the world they want to be in.
                        index.trouble = None;
                        name.edited = false;
                        name.focused = false;
                        front.0 = false;
                        load.0 = Some(slot);
                        *screen = Screen::Playing;
                    }
                    Err(error) => {
                        index.trouble = Some(format!("could not make a world: {error}"));
                    }
                }
                index.slots = saves::list(&root);
            }
            MenuAction::Load(row) => {
                let Some(slot) = index.slots.get(row).cloned() else {
                    continue;
                };
                if slot.file.seed != seed {
                    // A save is OF a world. Loading it into a different one
                    // would make it silently become somebody else's.
                    index.trouble = Some(format!(
                        "{} was made in another world (seed {})",
                        slot.file.name, slot.file.seed
                    ));
                    continue;
                }
                front.0 = false;
                load.0 = Some(slot);
                *screen = Screen::Playing;
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
    mut index: ResMut<SaveIndex>,
    mut name: ResMut<NameField>,
    save: Option<Res<WorldSave>>,
    mut panels: Query<(&Panel, &mut Node)>,
    mut rows: TouchedRows,
) {
    if screen.is_changed() {
        menu.0 = *screen != Screen::Playing;
        if *screen == Screen::Saves {
            // Read the directory on the way IN, however the screen was
            // opened, so the list is always what is on disk right now.
            let root = save.as_ref().map_or_else(
                || std::path::PathBuf::from(saves::ROOT),
                |s| s.root().into(),
            );
            index.slots = saves::list(&root);
            index.asking = None;
            index.trouble = None;
            name.focused = false;
            let worlds = index.slots.len();
            name.prefill(worlds);
        }
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
        assert_eq!(Screen::Playing.back(false, true), Screen::Pause);
        assert_eq!(Screen::Settings.back(false, true), Screen::Pause);
        assert_eq!(Screen::Pause.back(false, true), Screen::Playing);
        assert_eq!(Screen::Saves.back(false, true), Screen::Pause);
    }

    /// The saves page as the front door: back goes into the open world, and
    /// nowhere when there is none to go into. Through the pause menu it is
    /// the page it always was.
    #[test]
    fn the_front_door_steps_into_the_world_or_stays() {
        assert_eq!(Screen::Saves.back(true, true), Screen::Playing);
        assert_eq!(Screen::Saves.back(true, false), Screen::Saves);
        assert_eq!(Screen::Saves.back(false, false), Screen::Pause);
    }

    /// A plain launch opens on the saves page; a named world or a capture
    /// opens on the world; `--menu` names its own.
    #[test]
    fn a_plain_launch_opens_on_the_saves() {
        assert_eq!(opening_screen(None, false, false), Screen::Saves);
        assert_eq!(opening_screen(None, true, false), Screen::Playing);
        assert_eq!(opening_screen(None, false, true), Screen::Playing);
        assert_eq!(opening_screen(Some("pause"), true, true), Screen::Pause);
        assert_eq!(opening_screen(Some("saves"), true, true), Screen::Saves);
    }

    /// The name field: typed text goes in, Backspace takes it out, the
    /// prefill yields to anything typed, and an empty field makes the
    /// numbered world the page offered.
    #[test]
    fn the_name_field_takes_typing_and_offers_a_number() {
        let mut field = NameField::default();
        field.prefill(2);
        assert_eq!(field.text, "World 3");
        assert_eq!(field.name(2), "World 3");
        field.text.clear();
        field.typed(&Key::Character("Ca".into()));
        field.typed(&Key::Character("v".into()));
        field.typed(&Key::Space);
        field.typed(&Key::Character("e".into()));
        field.typed(&Key::Backspace);
        field.typed(&Key::Enter);
        assert_eq!(field.text, "Cav ");
        assert_eq!(field.name(2), "Cav");
        field.prefill(5);
        assert_eq!(field.text, "Cav ", "typed text is not overwritten");
        field.focused = true;
        assert_eq!(field.shown(), "Cav _");
        for _ in 0..100 {
            field.typed(&Key::Character("x".into()));
        }
        assert_eq!(field.text.chars().count(), NAME_MAX);
        let mut blank = NameField::default();
        blank.typed(&Key::Character("   ".into()));
        assert_eq!(blank.name(0), "World 1", "blank is the offered number");
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
