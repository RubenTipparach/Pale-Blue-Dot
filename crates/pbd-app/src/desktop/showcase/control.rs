//! The bot's live control channel (`bot-control`): a `--showcase` run listens
//! on `127.0.0.1` for one-line JSON commands and answers each with one line
//! of JSON, so a run can be watched and nudged without a restart.
//!
//! The socket lives on its own thread; each command crosses to the game with
//! a reply sender and is acted on in the bot's own frame (`drive`), between
//! frames, as a key press would be. Nothing here touches the world: a command
//! queues an act, skips one, pauses the bot or asks for a still, and the bot
//! acts only through the player's inputs.

use bevy::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};

/// The port when `PBD_BOT_PORT` names none.
pub const PORT: u16 = 47811;

/// One command from a client, and where its answer goes.
pub struct Command {
    pub body: serde_json::Value,
    pub reply: Sender<String>,
}

/// The commands waiting for the bot's next frame.
#[derive(Resource)]
pub struct Control {
    commands: Mutex<Receiver<Command>>,
}

impl Control {
    /// Every command that has come in since the last frame.
    pub fn drain(&self) -> Vec<Command> {
        let Ok(rx) = self.commands.lock() else {
            return Vec::new();
        };
        rx.try_iter().collect()
    }
}

/// Open the channel: a thread on `127.0.0.1:<port>`, one client at a time.
/// A port already taken is logged, and the run goes on without control.
pub fn open() -> Option<Control> {
    let port = std::env::var("PBD_BOT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(PORT);
    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => listener,
        Err(error) => {
            warn!("BOT_CONTROL off: 127.0.0.1:{port} not bound ({error})");
            return None;
        }
    };
    let (tx, rx) = channel::<Command>();
    std::thread::Builder::new()
        .name("bot-control".into())
        .spawn(move || {
            for stream in listener.incoming().flatten() {
                let Ok(read) = stream.try_clone() else {
                    continue;
                };
                let mut write = stream;
                for line in BufReader::new(read).lines() {
                    let Ok(line) = line else { break };
                    if line.trim().is_empty() {
                        continue;
                    }
                    let answer = match serde_json::from_str::<serde_json::Value>(&line) {
                        Ok(body) => {
                            let (reply, back) = channel();
                            if tx.send(Command { body, reply }).is_err() {
                                return;
                            }
                            back.recv_timeout(std::time::Duration::from_secs(20))
                                .unwrap_or_else(|_| {
                                    r#"{"ok":false,"error":"the game did not answer"}"#.into()
                                })
                        }
                        Err(error) => {
                            serde_json::json!({"ok": false, "error": format!("not JSON: {error}")})
                                .to_string()
                        }
                    };
                    if writeln!(write, "{answer}").is_err() {
                        break;
                    }
                }
            }
        })
        .ok()?;
    info!("BOT_CONTROL listening on 127.0.0.1:{port}");
    Some(Control {
        commands: Mutex::new(rx),
    })
}
