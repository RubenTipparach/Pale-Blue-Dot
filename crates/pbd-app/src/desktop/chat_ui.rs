//! Chat (`multiplayer` design 31): Enter opens a line at the bottom left,
//! typed in the game's own box; Enter sends it through the server and
//! Escape closes it. The last lines show above it for a while, and all of
//! them while it is open. Only with a host or a join.

use super::menu::Screen;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use pbd_app::net_play::{NetPlay, Others};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit;
use pbd_core::locale::tr_with;
use pbd_core::net::protocol::{ToClient, ToServer};

/// How long a line stays up after it is heard, s.
pub const SHOWN_S: f64 = 12.0;
/// The longest line the box takes, in characters (the server's own cut).
const LINE_MAX: usize = pbd_server::CHAT_MAX_CHARS;

/// What is being typed.
#[derive(Resource, Default, Clone, PartialEq, Eq, Debug)]
pub struct ChatLine {
    pub text: String,
}

impl ChatLine {
    fn typed(&mut self, key: &Key) {
        match key {
            Key::Character(text) => {
                for c in text.chars().filter(|c| !c.is_control()) {
                    if self.text.chars().count() < LINE_MAX {
                        self.text.push(c);
                    }
                }
            }
            Key::Space => {
                if self.text.chars().count() < LINE_MAX {
                    self.text.push(' ');
                }
            }
            Key::Backspace => {
                self.text.pop();
            }
            _ => {}
        }
    }
}

/// One of the shown lines, by its place from the oldest.
#[derive(Component)]
pub struct ChatShown(pub usize);

/// The line being typed.
#[derive(Component)]
pub struct ChatTyping;

/// The log and the line, at the bottom left over the world.
pub fn spawn(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(8),
                bottom: px(40),
                flex_direction: FlexDirection::Column,
                row_gap: px(1),
                max_width: px(300),
                ..default()
            },
            GlobalZIndex(8),
        ))
        .with_children(|log| {
            for i in 0..pbd_app::net_play::CHAT_KEPT {
                log.spawn((
                    super::menu::body(String::new(), 9.0, ui_kit::TEXT),
                    TextShadow {
                        offset: Vec2::splat(1.0),
                        color: ui_kit::INK,
                    },
                    ChatShown(i),
                    Node {
                        display: Display::None,
                        ..default()
                    },
                ));
            }
            log.spawn((
                super::menu::body(String::new(), 9.0, ui_kit::TITLE),
                TextShadow {
                    offset: Vec2::splat(1.0),
                    color: ui_kit::INK,
                },
                ChatTyping,
                Node {
                    display: Display::None,
                    ..default()
                },
            ));
        });
}

/// The keyboard: Enter opens the line in play when there is someone to talk
/// to; in it, Enter sends and Escape closes. Before `menu::toggle`, so
/// Escape closes the line rather than opening the pause menu.
#[allow(clippy::too_many_arguments)]
pub fn input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    mut screen: ResMut<Screen>,
    mut line: ResMut<ChatLine>,
    net: Option<Res<NetPlay>>,
    mut save: ResMut<WorldSave>,
    mut others: ResMut<Others>,
    time: Res<Time>,
) {
    let now = time.elapsed_secs_f64();
    let enter = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter);
    match *screen {
        Screen::Playing if enter && net.is_some() => {
            keys.clear_just_pressed(KeyCode::Enter);
            keys.clear_just_pressed(KeyCode::NumpadEnter);
            line.text.clear();
            typed.clear();
            *screen = Screen::Chat;
        }
        Screen::Chat => {
            if keys.just_pressed(KeyCode::Escape) {
                keys.clear_just_pressed(KeyCode::Escape);
                *screen = Screen::Playing;
                return;
            }
            if enter {
                keys.clear_just_pressed(KeyCode::Enter);
                keys.clear_just_pressed(KeyCode::NumpadEnter);
                let text = std::mem::take(&mut line.text);
                if !text.trim().is_empty() {
                    match net.as_deref() {
                        Some(NetPlay::Host { .. }) => {
                            // No server yet: nobody else is on, and the
                            // host's own line is heard here at once.
                            if !save.bypass_change_detection().serve_chat(&text)
                                && let Some(text) = pbd_server::clean_chat(&text)
                            {
                                others.hear(now, &ToClient::Chat { from: 0, text });
                            }
                        }
                        Some(NetPlay::Joined { client, .. }) => {
                            client.send(&ToServer::Chat { text })
                        }
                        None => {}
                    }
                }
                *screen = Screen::Playing;
                return;
            }
            for event in typed.read() {
                if event.state.is_pressed() {
                    line.typed(&event.logical_key);
                }
            }
            // Every key typed is the line's: the pack, the map, the
            // missions and the rest, read after this, do not see it.
            keys.clear();
        }
        _ => typed.clear(),
    }
}

/// The lines heard in the last [`SHOWN_S`], all of them while the line is
/// open, and the line being typed.
pub fn show(
    screen: Res<Screen>,
    others: Res<Others>,
    line: Res<ChatLine>,
    time: Res<Time>,
    mut shown: Query<(&ChatShown, &mut Text, &mut Node), Without<ChatTyping>>,
    mut typing: Query<(&mut Text, &mut Node), (With<ChatTyping>, Without<ChatShown>)>,
) {
    let now = time.elapsed_secs_f64();
    let open = *screen == Screen::Chat;
    for (row, mut text, mut node) in &mut shown {
        let heard = others
            .chat
            .get(row.0)
            .filter(|(at, _, _)| open || now - at < SHOWN_S);
        let display = if heard.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if let Some((_, from, said)) = heard {
            let want: String = tr_with("chat.line", &[("player", from), ("text", said)]).into();
            if text.0 != want {
                text.0 = want;
            }
        }
    }
    for (mut text, mut node) in &mut typing {
        let display = if open { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
        if open {
            let want: String = tr_with("chat.typing", &[("text", &line.text)]).into();
            if text.0 != want {
                text.0 = want;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_line_takes_text_and_stops_at_its_length() {
        let mut line = ChatLine::default();
        line.typed(&Key::Character("hi".into()));
        line.typed(&Key::Space);
        line.typed(&Key::Character("all".into()));
        assert_eq!(line.text, "hi all");
        line.typed(&Key::Backspace);
        assert_eq!(line.text, "hi al");
        for _ in 0..LINE_MAX + 10 {
            line.typed(&Key::Character("x".into()));
        }
        assert_eq!(line.text.chars().count(), LINE_MAX);
    }
}
