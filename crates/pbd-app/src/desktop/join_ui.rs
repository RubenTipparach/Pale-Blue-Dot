//! The title menu's MULTIPLAYER page (`multiplayer` design 28): the Pale
//! Blue Dot server and a game by its address. JOIN starts the game again
//! with `--join`, because a joined world is built from its host's seed
//! before the game starts (design 6.2).

use super::menu::{MenuAction, Screen};
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use pbd_app::ui_kit::{self, KitText, Plaque, Set, UiKit};
use pbd_core::locale::{Localized, tr, tr_with};
use std::net::{SocketAddr, ToSocketAddrs};

/// The persistent server (design 27, `deploy/fly/fly.toml`).
pub const HOSTED: &str = "pale-blue-dot.fly.dev";
/// The port every server listens on unless told another.
pub const PORT: u16 = 47123;
/// The longest address the field takes.
const ADDRESS_MAX: usize = 64;

/// The address field: what has been typed, whether the keyboard is in it,
/// whether Enter asked to join, and why the last JOIN did not go.
#[derive(Resource, Default, Clone, PartialEq, Eq, Debug)]
pub struct AddressField {
    pub text: String,
    pub focused: bool,
    pub submit: bool,
    pub trouble: Option<String>,
}

impl AddressField {
    fn typed(&mut self, key: &Key) {
        match key {
            Key::Character(text) => {
                for c in text
                    .chars()
                    .filter(|c| !c.is_control() && !c.is_whitespace())
                {
                    if self.text.chars().count() < ADDRESS_MAX {
                        self.text.push(c);
                    }
                }
            }
            Key::Backspace => {
                self.text.pop();
            }
            _ => {}
        }
    }

    fn shown(&self) -> String {
        if self.focused {
            format!("{}_", self.text)
        } else {
            self.text.clone()
        }
    }
}

/// `host:port`, or a host alone on [`PORT`], resolved to its first address.
/// `None` if it does not read or does not resolve.
pub fn resolve(text: &str) -> Option<SocketAddr> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Ok(addr) = text.parse::<SocketAddr>() {
        return Some(addr);
    }
    let with_port = if text
        .rsplit_once(':')
        .is_some_and(|(_, p)| p.parse::<u16>().is_ok())
    {
        text.to_string()
    } else {
        format!("{text}:{PORT}")
    };
    with_port.to_socket_addrs().ok()?.next()
}

/// The field's text on the page.
#[derive(Component)]
pub struct AddressText;

/// The line that says why a JOIN did not go.
#[derive(Component)]
pub struct AddressTrouble;

/// The page, hidden until the title's MULTIPLAYER opens it.
pub fn spawn(commands: &mut Commands, kit: &UiKit) {
    commands
        .spawn(super::menu::timber_layer(Screen::Multiplayer))
        .with_children(|layer| {
            layer
                .spawn(super::menu::framed(kit, Set::Oak, 260.0))
                .with_children(|panel| {
                    super::menu::timber_lintel(panel, kit, tr("join.title"));
                    panel.spawn(KitText::new(tr("join.hosted"), ui_kit::TITLE).shadow(ui_kit::INK));
                    panel
                        .spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            column_gap: px(4),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn(super::menu::body(
                                tr_with("join.address_port", &[("host", &HOSTED), ("port", &PORT)]),
                                9.0,
                                ui_kit::TEXT_DIM,
                            ));
                            let go = super::menu::small(
                                row,
                                kit,
                                tr("join.join"),
                                MenuAction::JoinHosted,
                            );
                            row.commands().entity(go).insert(Plaque {
                                focus: true,
                                ..Plaque::new(Set::Oak)
                            });
                        });
                    panel.spawn(Node {
                        height: px(4),
                        ..default()
                    });
                    panel.spawn(
                        KitText::new(tr("join.by_address"), ui_kit::TITLE).shadow(ui_kit::INK),
                    );
                    panel
                        .spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            column_gap: px(4),
                            ..default()
                        })
                        .with_children(|row| {
                            // The game's own box, as a world's name is typed.
                            row.spawn((
                                Button,
                                kit.nine(Set::Stone, "slider-track"),
                                Node {
                                    flex_grow: 1.0,
                                    height: px(18),
                                    padding: UiRect::new(px(5), px(5), px(4), px(0)),
                                    ..default()
                                },
                                MenuAction::EditAddress,
                            ))
                            .with_children(|field| {
                                field.spawn((
                                    super::menu::body(String::new(), 9.0, ui_kit::TEXT),
                                    AddressText,
                                ));
                            });
                            super::menu::small(row, kit, tr("join.join"), MenuAction::JoinAddress);
                        });
                    panel.spawn((
                        super::menu::body(String::new(), 9.0, ui_kit::TITLE),
                        AddressTrouble,
                    ));
                    panel
                        .spawn(Node {
                            justify_content: JustifyContent::FlexEnd,
                            margin: UiRect::top(px(4)),
                            ..default()
                        })
                        .with_children(|foot| {
                            super::menu::small(foot, kit, tr("menu.back"), MenuAction::Back);
                        });
                });
        });
}

/// The keyboard while it is in the address field: before `toggle`, so
/// Escape leaves the field rather than the page.
pub fn address_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    mut field: ResMut<AddressField>,
) {
    if !field.focused {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        keys.clear_just_pressed(KeyCode::Escape);
        field.focused = false;
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        keys.clear_just_pressed(KeyCode::Enter);
        keys.clear_just_pressed(KeyCode::NumpadEnter);
        field.submit = true;
    }
    for event in typed.read() {
        if event.state.is_pressed() {
            field.typed(&event.logical_key);
        }
    }
}

/// Start this game again joined to `addr`: the same program and its own
/// arguments but any world, host or join, with `--join` added.
fn rejoin(addr: SocketAddr) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut args: Vec<String> = Vec::new();
    let mut skip = 0;
    for a in std::env::args().skip(1) {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match a.as_str() {
            "--world" | "--host" | "--join" | "--menu" => skip = 1,
            _ => args.push(a),
        }
    }
    args.push("--join".into());
    args.push(addr.to_string());
    std::process::Command::new(exe)
        .args(&args)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// JOIN, the field's press and Enter.
pub fn press(
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut field: ResMut<AddressField>,
    screen: Res<Screen>,
    mut exit: MessageWriter<AppExit>,
) {
    if *screen != Screen::Multiplayer {
        if field.focused {
            field.focused = false;
        }
        return;
    }
    let mut join: Option<String> = None;
    if std::mem::take(&mut field.submit) {
        join = Some(field.text.clone());
    }
    for (interaction, action) in &rows {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            MenuAction::EditAddress => field.focused = true,
            MenuAction::JoinHosted => {
                field.focused = false;
                join = Some(format!("{HOSTED}:{PORT}"));
            }
            MenuAction::JoinAddress => {
                field.focused = false;
                join = Some(field.text.clone());
            }
            _ => field.focused = false,
        }
    }
    let Some(text) = join else { return };
    let trouble: Localized = match resolve(&text) {
        None if text.trim().is_empty() => tr("join.no_address"),
        None => tr_with("join.not_found", &[("address", &text.trim())]),
        Some(addr) => match rejoin(addr) {
            Ok(()) => {
                info!("joining {addr}: starting again with --join");
                exit.write(AppExit::Success);
                return;
            }
            Err(error) => tr_with("join.could_not_start", &[("error", &error)]),
        },
    };
    field.trouble = Some(trouble.into());
}

/// The field's text and the trouble line as they are.
pub fn show(
    field: Res<AddressField>,
    mut texts: Query<&mut Text, (With<AddressText>, Without<AddressTrouble>)>,
    mut troubles: Query<&mut Text, (With<AddressTrouble>, Without<AddressText>)>,
) {
    if !field.is_changed() {
        return;
    }
    for mut text in &mut texts {
        text.0 = field.shown();
    }
    for mut text in &mut troubles {
        text.0 = field.trouble.clone().unwrap_or_default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Design 28: an address with a port reads as it is, a host alone takes
    /// the servers' port, and nothing or rubbish reads as none.
    #[test]
    fn an_address_reads_with_or_without_its_port() {
        assert_eq!(
            resolve("127.0.0.1:5000"),
            Some("127.0.0.1:5000".parse().unwrap())
        );
        assert_eq!(
            resolve(" 127.0.0.1 "),
            Some(SocketAddr::from(([127, 0, 0, 1], PORT)))
        );
        assert_eq!(resolve("localhost:6000").map(|a| a.port()), Some(6000));
        assert_eq!(resolve(""), None);
        assert_eq!(resolve("not an address at all"), None);
    }

    #[test]
    fn the_field_takes_no_spaces_and_stops_at_its_length() {
        let mut field = AddressField::default();
        field.typed(&Key::Character("a b".into()));
        field.typed(&Key::Space);
        assert_eq!(field.text, "ab");
        for _ in 0..100 {
            field.typed(&Key::Character("x".into()));
        }
        assert_eq!(field.text.chars().count(), ADDRESS_MAX);
        field.typed(&Key::Backspace);
        assert_eq!(field.text.chars().count(), ADDRESS_MAX - 1);
    }
}
