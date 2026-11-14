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
use pbd_app::settings::{PlayerSettings, SettingsFile};
use pbd_app::ui_kit::{self, KitPiece, KitText, Plaque, Set, UiKit, plaque, plaque_label};

/// Which screen is open. One enum and not two flags, because two flags is how
/// a settings page ends up open over a closed pause menu.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Playing,
    Pause,
    Settings,
    Saves,
    /// The field guide, opened with J.
    Guide,
    /// The pack, opened with I (`inventory-grid`).
    Pack,
    /// The world map, opened with M (`world-map`).
    Map,
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
            Screen::Pause | Screen::Guide | Screen::Pack | Screen::Map => Screen::Playing,
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
    /// Set the clock to this hour: a time preset (`time_ui` acts).
    Time(u8),
    /// Show this overlay: 0 is off, then `Overlay::ALL` in order.
    Overlay(u8),
    /// Open this species' entry in the field guide (`guide::press` acts).
    Species(u16),
    /// Turn bloom on or off (`press_graphics` acts).
    Bloom,
    /// Cycle the frame limit: off, 60, 120 (`player-settings`).
    FrameLimit,
    /// Cycle the render scale through `settings::SCALES`.
    RenderScale,
    /// Turn auto step on or off.
    AutoStep,
    /// Turn the rain shadow (the rain volume) on or off.
    RainShadows,
}

/// A panel the mouse wheel scrolls.
#[derive(Component)]
pub struct WheelScrolls;

/// The wheel scrolls a panel that is taller than the screen.
pub fn wheel_scroll(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut panels: Query<(&mut ScrollPosition, &InheritedVisibility), With<WheelScrolls>>,
) {
    let dy: f32 = wheel
        .read()
        .map(|w| match w.unit {
            bevy::input::mouse::MouseScrollUnit::Line => w.y * 36.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => w.y,
        })
        .sum();
    if dy == 0.0 {
        return;
    }
    for (mut at, shown) in &mut panels {
        if shown.get() {
            at.0.y = (at.0.y - dy).max(0.0);
        }
    }
}

/// What a value plaque on the settings page says (`player-settings`): the
/// frame limit and the render scale, which a press steps through.
fn settings_value(action: MenuAction, player: &PlayerSettings) -> Option<String> {
    Some(match action {
        MenuAction::FrameLimit => player.frame_limit.label().to_uppercase(),
        MenuAction::RenderScale => format!("{}%", player.render_scale),
        _ => return None,
    })
}

/// Whether a checkbox on the settings page is ticked: bloom, the rain
/// shadow (off by default, CLAUDE.md "What the owner means by rain shadow")
/// and auto step.
fn settings_checked(
    action: MenuAction,
    graphics: &pbd_app::tone::Graphics,
    player: &PlayerSettings,
) -> Option<bool> {
    Some(match action {
        MenuAction::Bloom => graphics.bloom,
        MenuAction::RainShadows => player.rain_shadows,
        MenuAction::AutoStep => player.auto_step,
        _ => return None,
    })
}

/// A press on GRAPHICS' and MOVEMENT's rows changes the player's choice and
/// keeps it.
pub fn press_graphics(
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut graphics: ResMut<pbd_app::tone::Graphics>,
    mut player: ResMut<PlayerSettings>,
    file: Res<SettingsFile>,
) {
    for (interaction, action) in &rows {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let mut changed = *player;
        match action {
            MenuAction::Bloom => {
                graphics.bloom = !graphics.bloom;
                graphics.keep();
            }
            MenuAction::FrameLimit => changed.frame_limit = changed.frame_limit.next(),
            MenuAction::RenderScale => {
                changed.render_scale = pbd_app::settings::next_scale(changed.render_scale)
            }
            MenuAction::AutoStep => changed.auto_step = !changed.auto_step,
            MenuAction::RainShadows => changed.rain_shadows = !changed.rain_shadows,
            _ => {}
        }
        if changed != *player {
            *player = changed;
            file.keep(&player);
        }
    }
}

/// The settings controls say what is chosen: a checkbox shows its tick, a
/// value plaque its value.
pub fn label_graphics(
    kit: Res<UiKit>,
    graphics: Res<pbd_app::tone::Graphics>,
    player: Res<PlayerSettings>,
    mut checks: Query<(&MenuAction, &mut ImageNode), With<Check>>,
    rows: Query<(&MenuAction, &Children), With<Plaque>>,
    mut texts: Query<&mut KitText>,
) {
    for (action, mut image) in &mut checks {
        if let Some(on) = settings_checked(*action, &graphics, &player) {
            let want = kit.image(if on { "check-on" } else { "check-off" });
            if image.image != want {
                image.image = want;
            }
        }
    }
    for (action, children) in &rows {
        let Some(label) = settings_value(*action, &player) else {
            continue;
        };
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child)
                && text.text != label
            {
                text.text = label.clone();
            }
        }
    }
}

/// A checkbox on the settings page.
#[derive(Component)]
pub struct Check;

/// The settings page's tabs: Picture, Weather (the rain shadow and the
/// storm forcing), World (the clock and the globe overlay), Movement, Input.
pub const SETTINGS_TABS: [&str; 5] = ["PICTURE", "WEATHER", "WORLD", "MOVEMENT", "INPUT"];

/// Which settings tab is open.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SettingsTab(pub u8);

/// A settings tab's own oak tab, by index.
#[derive(Component)]
pub struct SettingsTabButton(pub u8);

/// The body of a settings tab, by index.
#[derive(Component)]
pub struct SettingsPage(pub u8);

/// A press on a tab opens its page; the open tab wears the oak cap, the shut
/// ones the plain stone (design 3, accent 2), and only the open page is laid
/// out.
#[allow(clippy::type_complexity)]
pub fn settings_tabs(
    kit: Res<UiKit>,
    mut tab: ResMut<SettingsTab>,
    pressed: Query<(&Interaction, &SettingsTabButton), Changed<Interaction>>,
    mut tabs: Query<(&SettingsTabButton, &mut ImageNode, &mut KitPiece, &Children)>,
    mut labels: Query<&mut KitText>,
    mut pages: Query<(&SettingsPage, &mut Node)>,
) {
    for (interaction, button) in &pressed {
        if *interaction == Interaction::Pressed && tab.0 != button.0 {
            tab.0 = button.0;
        }
    }
    if !tab.is_changed() {
        return;
    }
    for (button, mut image, mut piece, children) in &mut tabs {
        let open = button.0 == tab.0;
        let key = Set::Oak.key(if open { "tab-open" } else { "tab-shut" });
        if piece.0 != key {
            image.image = kit.image(&key);
            piece.0 = key;
        }
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                text.colour = if open {
                    ui_kit::OAK_LABEL
                } else {
                    ui_kit::LABEL
                };
            }
        }
    }
    for (page, mut node) in &mut pages {
        node.display = if page.0 == tab.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
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
        Some("pack") => Screen::Pack,
        Some("map") => Screen::Map,
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

/// Marks a panel with the screen it belongs to.
#[derive(Component)]
pub struct Panel(pub(super) Screen);

/// The full-screen layer a panel is centred in.
///
/// Centring by flex rather than by a margin of half the panel's own height,
/// because the height is the CONTENT's: the settings page grows by a row every
/// time a binding is added, and a magic offset would be wrong the day it does.
/// The first cut used `top: 50%` and the input list ran off the bottom of the
/// window.
pub(super) fn layer() -> Node {
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

/// An out-of-play menu's layer: the world dimmed behind the timber.
fn timber_layer(screen: Screen) -> impl Bundle {
    (
        layer(),
        BackgroundColor(ui_kit::DIMMER),
        GlobalZIndex(10),
        Panel(screen),
    )
}

/// A panel's frame and layout: `width` native pixels, the frame's 14 px
/// margin as its padding, children in a column. Oak out of play, the stone
/// moulding (`frame`) for an in-play panel with no baked rock of its size.
pub(super) fn framed(kit: &UiKit, set: Set, width: f32) -> impl Bundle {
    (
        kit.nine(set, "frame"),
        KitPiece(set.key("frame")),
        Node {
            width: px(width),
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
    )
}

/// The carved oak lintel over a panel, its title cut into it (design 3,
/// accent 1): pegged on the frame's top edge.
pub(super) fn lintel(parent: &mut ChildSpawnerCommands, kit: &UiKit, text: &str) {
    parent
        .spawn(Node {
            justify_content: JustifyContent::Center,
            margin: UiRect::new(px(0), px(0), px(-15), px(4)),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                kit.nine(Set::Stone, "lintel"),
                KitPiece("lintel".into()),
                Node {
                    min_height: px(16),
                    min_width: px(40),
                    padding: UiRect::new(px(11), px(11), px(5), px(4)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|board| {
                board.spawn(KitText::new(text, ui_kit::OAK_LABEL));
            });
        });
}

/// A section label inside a panel, carved in the kit's font.
pub(super) fn section(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        KitText::new(text, ui_kit::TITLE).shadow(ui_kit::INK),
        Node {
            margin: UiRect::top(px(3)),
            ..default()
        },
    ));
}

/// A line of the body face (VT323) on a stone face.
pub(super) fn body(text: impl Into<String>, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(colour),
    )
}

/// A row that can be clicked: an oak plaque the panel's width. Every one is a
/// real `Button` with a hit box and hover and press states: a control that
/// only responds to a key is a control nobody can find, which is why the
/// bindings were on screen in the first place.
pub(super) fn button(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: &str,
    action: MenuAction,
) -> Entity {
    parent
        .spawn((plaque(kit, Set::Oak, 48.0), action))
        .insert(Node {
            width: percent(100.0),
            ..plaque_node()
        })
        .with_children(|row| {
            row.spawn(plaque_label(Set::Oak, label));
        })
        .id()
}

/// A plaque's own node, as `ui_kit::plaque` lays it out.
fn plaque_node() -> Node {
    Node {
        min_width: px(30),
        height: px(22),
        padding: UiRect::new(px(10), px(10), px(8), px(0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        flex_shrink: 0.0,
        ..default()
    }
}

pub fn spawn(mut commands: Commands, kit: Res<UiKit>) {
    let kit = &*kit;
    spawn_saves(&mut commands, kit);
    commands
        // Above the slot row, which is spawned a schedule later and would
        // otherwise draw over the panel: within a UI, later is on top.
        .spawn(timber_layer(Screen::Pause))
        .with_children(|layer| {
            layer
                // The owner's menu and nothing else (`menus-and-a-quiet-screen`:
                // "Pause menu should have resume, settings, and quit", and the
                // saves): the weather, the clock and the globe overlay moved to
                // their own tabs in Settings, so the panel fits every window.
                .spawn(framed(kit, Set::Oak, 120.0))
                .with_children(|panel| {
                    lintel(panel, kit, "PAUSED");
                    let resume = button(panel, kit, "RESUME", MenuAction::Resume);
                    panel.commands().entity(resume).insert(Plaque {
                        focus: true,
                        ..Plaque::new(Set::Oak)
                    });
                    button(panel, kit, "SAVES", MenuAction::Saves);
                    button(panel, kit, "SETTINGS", MenuAction::Settings);
                    button(panel, kit, "QUIT", MenuAction::Quit);
                });
        });

    commands
        .spawn(timber_layer(Screen::Settings))
        .with_children(|layer| {
            layer
                .spawn(framed(kit, Set::Oak, 324.0))
                .with_children(|panel| {
                    lintel(panel, kit, "SETTINGS");
                    // The tabs, oak, the open one capped (design 8).
                    panel
                        .spawn(Node {
                            column_gap: px(1),
                            margin: UiRect::bottom(px(3)),
                            ..default()
                        })
                        .with_children(|tabs| {
                            for (i, name) in SETTINGS_TABS.iter().enumerate() {
                                let open = i == 0;
                                let piece = if open { "tab-open" } else { "tab-shut" };
                                tabs.spawn((
                                    Button,
                                    kit.nine(Set::Oak, piece),
                                    KitPiece(Set::Oak.key(piece)),
                                    SettingsTabButton(i as u8),
                                    Node {
                                        height: px(18),
                                        padding: UiRect::new(px(10), px(10), px(7), px(0)),
                                        flex_shrink: 0.0,
                                        ..default()
                                    },
                                ))
                                .with_children(|tab| {
                                    tab.spawn(KitText::new(
                                        *name,
                                        if open {
                                            ui_kit::OAK_LABEL
                                        } else {
                                            ui_kit::LABEL
                                        },
                                    ));
                                });
                            }
                        });
                    let page = |panel: &mut ChildSpawnerCommands, i: u8| {
                        panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(2),
                                    min_height: px(176),
                                    display: if i == 0 { Display::Flex } else { Display::None },
                                    ..default()
                                },
                                SettingsPage(i),
                            ))
                            .id()
                    };
                    let picture = page(panel, 0);
                    panel.commands().entity(picture).with_children(|rows| {
                        // `player-settings`: each control shows what is chosen,
                        // kept up by `label_graphics`.
                        check_row(
                            rows,
                            kit,
                            "Bloom",
                            "costs ~2.4 ms on a Mac",
                            MenuAction::Bloom,
                        );
                        value_row(rows, kit, "Frame limit", "", MenuAction::FrameLimit);
                        value_row(
                            rows,
                            kit,
                            "Render scale",
                            "the HUD stays sharp",
                            MenuAction::RenderScale,
                        );
                    });
                    let weather = page(panel, 1);
                    panel.commands().entity(weather).with_children(|rows| {
                        check_row(
                            rows,
                            kit,
                            "Rain shadow",
                            "the rain fog and streaks; off by default",
                            MenuAction::RainShadows,
                        );
                        rows.spawn(body(
                            "The near shower, the wet ground and the drops on the lens still draw.",
                            10.0,
                            ui_kit::TEXT_DIM,
                        ));
                        super::weather_ui::spawn(rows, kit);
                    });
                    let world = page(panel, 2);
                    panel.commands().entity(world).with_children(|rows| {
                        super::time_ui::spawn(rows, kit);
                        super::overlay_ui::spawn_row(rows, kit);
                    });
                    let movement = page(panel, 3);
                    panel.commands().entity(movement).with_children(|rows| {
                        check_row(
                            rows,
                            kit,
                            "Auto step",
                            "walk up one layer without a key",
                            MenuAction::AutoStep,
                        );
                    });
                    let input = page(panel, 4);
                    panel.commands().entity(input).with_children(|page| {
                        // Taller than the page (every binding is listed), so it is
                        // held inside it and scrolls with the wheel.
                        page.spawn((
                            Node {
                                height: px(176),
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::scroll_y(),
                                row_gap: px(1),
                                ..default()
                            },
                            ScrollPosition::default(),
                            WheelScrolls,
                        ))
                        .with_children(|list| {
                            // Off the one binding table, a heading per context and
                            // a row per action with its keys in caps.
                            for group in &BINDINGS {
                                section(list, group.heading);
                                for binding in group.rows {
                                    list.spawn(Node {
                                        justify_content: JustifyContent::SpaceBetween,
                                        align_items: AlignItems::Center,
                                        min_height: px(18),
                                        column_gap: px(4),
                                        flex_shrink: 0.0,
                                        ..default()
                                    })
                                    .with_children(|line| {
                                        line.spawn(body(binding.does, 10.0, ui_kit::TEXT));
                                        line.spawn(Node {
                                            column_gap: px(1),
                                            flex_shrink: 0.0,
                                            ..default()
                                        })
                                        .with_children(
                                            |caps| {
                                                for key in binding.keys {
                                                    key_cap(caps, kit, &key.label());
                                                }
                                            },
                                        );
                                    });
                                }
                            }
                        });
                    });
                    panel
                        .spawn(Node {
                            justify_content: JustifyContent::FlexEnd,
                            margin: UiRect::top(px(4)),
                            ..default()
                        })
                        .with_children(|foot| {
                            small(foot, kit, "BACK", MenuAction::Back);
                        });
                });
        });
}

/// A slider in the timber menus: the stone groove, the rune in it as far as
/// the value, and the oak grip (design 8). `track` is the clickable groove
/// (it carries `RelativeCursorPosition`), `fill` and `knob` are moved by the
/// caller's `show`.
pub(super) fn slider(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    width: f32,
    track: impl Bundle,
    fill: impl Bundle,
    knob: impl Bundle,
) {
    parent
        .spawn((
            kit.nine(Set::Stone, "slider-track"),
            KitPiece("slider-track".into()),
            Node {
                width: px(width),
                height: px(8),
                padding: UiRect::all(px(3)),
                margin: UiRect::vertical(px(3)),
                flex_shrink: 0.0,
                ..default()
            },
            track,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: percent(0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(ui_kit::RUNE_DIM),
                fill,
            ));
            bar.spawn((
                kit.whole(Set::Oak, "slider-knob"),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(0),
                    top: px(-2),
                    width: px(9),
                    height: px(13),
                    margin: UiRect::left(px(-4)),
                    ..default()
                },
                knob,
            ));
        });
}

/// A settings row: what it is, a dim note, and its control on the right.
fn setting_row(
    rows: &mut ChildSpawnerCommands,
    label: &str,
    note: &str,
    control: impl FnOnce(&mut ChildSpawnerCommands),
) {
    rows.spawn(Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        min_height: px(22),
        column_gap: px(4),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            column_gap: px(4),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|words| {
            words.spawn(body(label, 10.0, ui_kit::TEXT));
            if !note.is_empty() {
                words.spawn(body(format!("({note})"), 8.0, ui_kit::TEXT_DIM));
            }
        });
        control(row);
    });
}

/// A row with a checkbox: the stone well, the tick a glowing rune.
fn check_row(
    rows: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: &str,
    note: &str,
    action: MenuAction,
) {
    setting_row(rows, label, note, |row| {
        row.spawn((
            Button,
            kit.whole(Set::Stone, "check-off"),
            Node {
                width: px(12),
                height: px(12),
                flex_shrink: 0.0,
                ..default()
            },
            Check,
            action,
        ));
    });
}

/// A row whose plaque steps through values.
fn value_row(
    rows: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: &str,
    note: &str,
    action: MenuAction,
) {
    setting_row(rows, label, note, |row| {
        row.spawn((plaque(kit, Set::Oak, 64.0), action))
            .with_children(|p| {
                p.spawn(plaque_label(Set::Oak, ""));
            });
    });
}

/// A key in a cap: the oak-rimmed tooltip with the key cut in the kit's font.
fn key_cap(parent: &mut ChildSpawnerCommands, kit: &UiKit, key: &str) {
    parent
        .spawn((
            kit.nine(Set::Oak, "tooltip"),
            KitPiece(Set::Oak.key("tooltip")),
            Node {
                min_width: px(18),
                height: px(18),
                padding: UiRect::new(px(6), px(6), px(5), px(0)),
                justify_content: JustifyContent::Center,
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|cap| {
            cap.spawn(KitText::new(key, ui_kit::TEXT));
        });
}

/// The saves panel: a title, the list (rebuilt when it changes), and BACK.
fn spawn_saves(commands: &mut Commands, kit: &UiKit) {
    commands
        .spawn(timber_layer(Screen::Saves))
        .with_children(|layer| {
            layer
                .spawn(framed(kit, Set::Oak, 280.0))
                .with_children(|panel| {
                    lintel(panel, kit, "WORLDS");
                    panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(3),
                            ..default()
                        },
                        SaveList,
                    ));
                    panel
                        .spawn(Node {
                            justify_content: JustifyContent::FlexEnd,
                            margin: UiRect::top(px(4)),
                            ..default()
                        })
                        .with_children(|foot| {
                            small(foot, kit, "BACK", MenuAction::Back);
                        });
                });
        });
}

/// A line of small text, for a slot's date or a reason something did not
/// happen.
fn note(text: String, colour: Color) -> impl Bundle {
    body(text, 9.0, colour)
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
    kit: Res<UiKit>,
    index: Res<SaveIndex>,
    name: Res<NameField>,
    open: Option<Res<WorldSave>>,
    lists: Query<Entity, With<SaveList>>,
) {
    if !index.is_changed() {
        return;
    }
    let kit = &*kit;
    let playing = open
        .as_ref()
        .and_then(|save| save.slot().map(|slot| slot.id.clone()));
    for list in &lists {
        commands.entity(list).despawn_related::<Children>();
        commands.entity(list).with_children(|rows| {
            if let Some(trouble) = index.trouble.as_ref() {
                rows.spawn(note(trouble.clone(), ui_kit::TITLE));
            }
            // A delete REPLACES the list while it is pending, and says what
            // is lost rather than only asking. Tenebris's own dialog, ported
            // for its reason: a row that says "delete?" beside nine other
            // rows is a question a player answers without reading it.
            if let Some(row) = index.asking
                && let Some(slot) = index.slots.get(row)
            {
                rows.spawn(body(
                    format!("Delete '{}'?", slot.file.name),
                    11.0,
                    ui_kit::TITLE,
                ));
                rows.spawn(note(
                    "Everything dug, built and carried in it is gone for good.".into(),
                    ui_kit::TEXT,
                ));
                rows.spawn(Node {
                    height: px(4),
                    ..default()
                });
                rows.spawn(Node {
                    column_gap: px(3),
                    ..default()
                })
                .with_children(|line| {
                    small(line, kit, "DELETE FOREVER", MenuAction::Delete(row));
                    let keep = small(line, kit, "CANCEL", MenuAction::Keep);
                    line.commands().entity(keep).insert(Plaque {
                        focus: true,
                        ..Plaque::new(Set::Oak)
                    });
                });
                return;
            }
            if index.slots.is_empty() {
                rows.spawn(note("no worlds yet".into(), ui_kit::TEXT_DIM));
            }
            for (row, slot) in index.slots.iter().enumerate() {
                let here = Some(&slot.id) == playing.as_ref();
                rows.spawn(Node {
                    column_gap: px(3),
                    align_items: AlignItems::Center,
                    min_height: px(24),
                    ..default()
                })
                .with_children(|line| {
                    line.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        row_gap: px(1),
                        ..default()
                    })
                    .with_children(|words| {
                        let shown = if here {
                            format!("{} (open)", slot.file.name)
                        } else {
                            slot.file.name.clone()
                        };
                        words.spawn(KitText::new(shown, ui_kit::TITLE).shadow(ui_kit::INK));
                        words.spawn(note(since(slot.file.played_unix_s), ui_kit::TEXT_DIM));
                    });
                    if here {
                        small(line, kit, "CONTINUE", MenuAction::Resume);
                    } else {
                        small(line, kit, "LOAD", MenuAction::Load(row));
                    }
                    small(line, kit, "DELETE", MenuAction::Ask(row));
                });
            }
            rows.spawn(Node {
                height: px(3),
                ..default()
            });
            // The name for a new world, typed in the game's own box: a
            // native prompt over the window is the thing the inherited UI
            // rule forbids. Prefilled with the number the page would have
            // chosen, so Enter alone still makes one.
            rows.spawn(Node {
                column_gap: px(4),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|line| {
                line.spawn(KitText::new("NAME", ui_kit::TITLE).shadow(ui_kit::INK));
                line.spawn((
                    Button,
                    kit.nine(Set::Oak, "tooltip"),
                    KitPiece(Set::Oak.key("tooltip")),
                    Node {
                        flex_grow: 1.0,
                        height: px(18),
                        padding: UiRect::new(px(6), px(6), px(3), px(0)),
                        ..default()
                    },
                    MenuAction::EditName,
                ))
                .with_children(|field| {
                    field.spawn((body(name.shown(), 10.0, ui_kit::TEXT), NameText));
                });
            });
            rows.spawn(Node {
                justify_content: JustifyContent::FlexStart,
                ..default()
            })
            .with_children(|line| {
                small(line, kit, "NEW WORLD", MenuAction::New);
            });
        });
    }
}

/// An oak plaque that sits in a row rather than filling the panel.
pub(super) fn small(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: &str,
    action: MenuAction,
) -> Entity {
    parent
        .spawn((plaque(kit, Set::Oak, 30.0), action))
        .with_children(|row| {
            row.spawn(plaque_label(Set::Oak, label));
        })
        .id()
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
    mut overlay: ResMut<pbd_app::overlay::OverlayMode>,
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
    let seed = pbd_app::planet::terrain_config().seed;
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
            // The clock's own control answers it (`time_ui::press`).
            MenuAction::Time(_) => {}
            MenuAction::Overlay(row) => overlay.0 = super::overlay_ui::mode_for(row),
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
                        // the world they want to be in. It is made by the
                        // newest generator, and the load switches the planet
                        // to it in place if an older world's is up
                        // (`bigger-biomes` decision 8).
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
            // The guide's own system turns the page.
            MenuAction::Species(_) => {}
            // `press_graphics` acts.
            MenuAction::Bloom
            | MenuAction::FrameLimit
            | MenuAction::RenderScale
            | MenuAction::AutoStep
            | MenuAction::RainShadows => {}
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
                if let Some(why) = saves::refusal(&slot) {
                    // Opened, it would be generated by rules it was not made
                    // by: say which version this build lacks instead.
                    index.trouble = Some(format!("{} was made with {why}", slot.file.name));
                    continue;
                }
                // A world of another generator than the planet's is loaded
                // like any other: the load switches the planet in place
                // (`bigger-biomes` decision 8).
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Out of play is timber (owner, 2026-11-14: "pause menu = timber too"):
    /// the pause, settings and saves panels are framed in the carved oak, and
    /// every plaque on them is oak.
    #[test]
    fn the_timber_menus_are_oak() {
        let mut app = App::new();
        pbd_app::ui_kit::install_bare(&mut app);
        app.add_systems(Update, spawn);
        app.update();
        let world = app.world_mut();
        let oak = world.resource::<UiKit>().image("oak/frame");
        let mut panels = world.query::<(&Panel, &Children)>();
        let found: Vec<(Screen, Entity)> = panels
            .iter(world)
            .map(|(panel, children)| (panel.0, children[0]))
            .collect();
        let screens: Vec<Screen> = found.iter().map(|f| f.0).collect();
        for screen in [Screen::Pause, Screen::Settings, Screen::Saves] {
            assert!(screens.contains(&screen), "{screen:?} has a panel");
        }
        for (screen, frame) in found {
            let image = world.get::<ImageNode>(frame).expect("a framed panel");
            assert_eq!(image.image, oak, "{screen:?} is framed in oak");
        }
        let mut plaques = world.query::<&Plaque>();
        assert!(plaques.iter(world).count() >= 5);
        for plaque in plaques.iter(world) {
            assert_eq!(plaque.set, Set::Oak, "a timber menu's plaque is oak");
        }
    }

    /// Escape steps out one screen at a time, and from the world it steps IN.
    /// A settings page that resumed the game on its way past the pause menu is
    /// the reason this is one enum rather than a pair of booleans.
    #[test]
    fn escape_steps_one_screen_at_a_time() {
        assert_eq!(Screen::Playing.back(false, true), Screen::Pause);
        assert_eq!(Screen::Settings.back(false, true), Screen::Pause);
        assert_eq!(Screen::Pause.back(false, true), Screen::Playing);
        assert_eq!(Screen::Saves.back(false, true), Screen::Pause);
        assert_eq!(Screen::Guide.back(false, true), Screen::Playing);
        assert_eq!(Screen::Pack.back(false, true), Screen::Playing);
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
