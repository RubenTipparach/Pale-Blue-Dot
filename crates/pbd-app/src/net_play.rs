//! Playing one world from two games (`openspec/changes/multiplayer` design
//! 20, task 6.2): a game that hosts its world on the network, and a game
//! that joined one.
//!
//! - **Hosting** (`--host <port>`): the game plays its own world as always,
//!   and remote players join the same in-process server, with this game's
//!   save as its store. Their requests are decided like the host's own; the
//!   server's messages go out each frame, and the changes addressed to the
//!   host's own player are applied to the host's tier.
//! - **Joined** (`--join <address:port>`): the world is the host's, opened
//!   in memory from the welcome. The player's digs and places are requests;
//!   every change and snapshot the server sends is applied to the game's
//!   copy of the edits and to its tier through the one APPLY.
//!
//! Without either, single player has nobody to send to, and the server's
//! messages are dropped each frame.

use crate::planet::{Applied, PlanetContact, PlanetFine};
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::edits::Edit;
use pbd_core::net::protocol::{Change, ToClient, ToServer};
use std::collections::BTreeMap;
use std::sync::Arc;

/// This game's place on the network, if it has one.
#[derive(Resource)]
pub enum NetPlay {
    /// Hosting: the listener and each connection's player.
    Host {
        host: pbd_net::Host,
        players: BTreeMap<u64, u32>,
    },
    /// Joined another's world, as this player, with the changes the server
    /// sent that the tier has not been given yet (it may not be built yet:
    /// a joining game hears its first snapshots while it loads).
    Joined {
        client: pbd_net::Client,
        player: u32,
        pending: Vec<Edit>,
    },
}

/// What the joining game learnt from the host before it built its world.
pub struct Joining {
    pub client: pbd_net::Client,
    pub player: u32,
    pub clock_s: f64,
    pub seed: u64,
    pub generator: u32,
    pub topology: u32,
    /// Messages that came with the welcome, applied once the world is up.
    pub early: Vec<ToClient>,
}

/// Connect to `addr` and join the world there, waiting up to `wait` for the
/// welcome: before the game is built, because the world it opens is the
/// host's. `Err` with the reason in words for the log.
pub fn join(addr: std::net::SocketAddr, wait: std::time::Duration) -> Result<Joining, String> {
    use pbd_core::net::protocol::{PROTOCOL, PROTOCOL_MIN, Versions};
    let client = pbd_net::Client::connect(addr, wait).map_err(|e| e.to_string())?;
    let hello = ToServer::Hello {
        versions: Versions {
            protocol: PROTOCOL,
            protocol_min: PROTOCOL_MIN,
            topology: pbd_core::terrain::TOPOLOGY_VERSION,
            // The host's world's generator is any this build carries: the
            // join is refused only for one it does not.
            generator: 0,
            content: 0,
            world: 0,
        },
        key: install_key(),
        name: String::new(),
    };
    client.send(&hello);
    let started = std::time::Instant::now();
    let mut early = Vec::new();
    loop {
        // The welcome and the snapshots behind it come in one batch: the
        // whole batch is read before the welcome is acted on, or the world's
        // edits that came with it are lost (design 21).
        let mut welcome = None;
        for message in client.poll() {
            match message {
                Ok(ToClient::Welcome {
                    player,
                    clock_s,
                    seed,
                    generator,
                    topology,
                }) if welcome.is_none() => {
                    welcome = Some((player, clock_s, seed, generator, topology));
                }
                Ok(ToClient::Rejected { reason }) => return Err(format!("{reason:?}")),
                Ok(other) => early.push(other),
                Err(bad) => return Err(format!("{bad:?}")),
            }
        }
        if let Some((player, clock_s, seed, generator, topology)) = welcome {
            return Ok(Joining {
                client,
                player,
                clock_s,
                seed,
                generator,
                topology,
                early,
            });
        }
        if client.is_closed() {
            return Err("closed".into());
        }
        if started.elapsed() > wait {
            return Err("no welcome".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// This install's key, which a host knows its player by: kept beside the
/// saves, made the first time.
fn install_key() -> [u8; 16] {
    let path = crate::paths::saves_dir().join("install.key");
    if let Ok(bytes) = std::fs::read(&path)
        && let Ok(key) = <[u8; 16]>::try_from(bytes.as_slice())
    {
        return key;
    }
    let mut key = [0u8; 16];
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mixed = pbd_core::fauna::mix(now as u64 ^ u64::from(std::process::id()));
    key[..8].copy_from_slice(&mixed.to_le_bytes());
    key[8..].copy_from_slice(&pbd_core::fauna::mix(mixed).to_le_bytes());
    let _ = std::fs::create_dir_all(crate::paths::saves_dir());
    let _ = std::fs::write(&path, key);
    key
}

/// Each frame: the network's messages in, the server's out, and every
/// change this game's tier has not seen applied to it.
pub fn play(
    net: Option<ResMut<NetPlay>>,
    mut save: ResMut<WorldSave>,
    fine: Option<ResMut<PlanetFine>>,
    contact: Option<ResMut<PlanetContact>>,
    walkers: Query<&Transform, With<crate::walking::Walker>>,
) {
    let Some(mut net) = net else {
        // Single player: nobody to send to.
        if save.bypass_change_detection().has_outbox() {
            save.take_outbox();
        }
        return;
    };
    let mut applied: Vec<Edit> = Vec::new();
    match net.as_mut() {
        NetPlay::Host { host, players } => {
            let near = walkers
                .iter()
                .next()
                .map(|t| t.translation.normalize_or_zero());
            for event in host.poll() {
                match event {
                    pbd_net::HostEvent::Opened(c) => info!("a player connected ({c})"),
                    pbd_net::HostEvent::Message(c, hello @ ToServer::Hello { .. })
                        if !players.contains_key(&c) =>
                    {
                        let kit = crate::hotbar::Hotbar::starting_kit().0;
                        match save.serve_join(c, &hello, &kit, near) {
                            Some(player) => {
                                info!("player {player} joined ({c})");
                                // The same player joining again before their
                                // old connection is seen to close (a game
                                // restarted): the old one is forgotten and
                                // closed, so messages go to the new one and
                                // the old one's close leaves the player in.
                                let stale: Vec<u64> = players
                                    .iter()
                                    .filter(|(_, p)| **p == player)
                                    .map(|(c, _)| *c)
                                    .collect();
                                for old in stale {
                                    players.remove(&old);
                                    host.close(old);
                                    info!("player {player} rejoined: dropped ({old})");
                                }
                                players.insert(c, player);
                            }
                            None => info!("a join was refused ({c})"),
                        }
                    }
                    pbd_net::HostEvent::Message(c, message) => {
                        if let Some(&player) = players.get(&c) {
                            save.serve(player, message);
                        }
                    }
                    pbd_net::HostEvent::Unreadable(c, why) => {
                        warn!("a player's message could not be read ({c}): {why:?}")
                    }
                    pbd_net::HostEvent::Closed(c) => {
                        if let Some(player) = players.remove(&c) {
                            info!("player {player} left ({c})");
                            save.serve_leave(player);
                        }
                    }
                }
            }
            for (to, message) in save.take_outbox() {
                match to {
                    pbd_server::To::Player(0) => {
                        // The host's own player: its own edits are applied
                        // already; a remote player's reach its tier here.
                        if let ToClient::Change {
                            change:
                                Change::Edit {
                                    cell,
                                    layer,
                                    material,
                                },
                            ..
                        } = message
                        {
                            applied.push(Edit {
                                cell,
                                layer,
                                material,
                            });
                        }
                    }
                    pbd_server::To::Player(p) => {
                        if let Some((&c, _)) = players.iter().find(|(_, n)| **n == p) {
                            host.send(c, &message);
                        }
                    }
                    pbd_server::To::Connection(c) => {
                        host.send(c, &message);
                        host.close(c);
                    }
                }
            }
        }
        NetPlay::Joined {
            client,
            player,
            pending,
        } => {
            for request in save.take_requests() {
                client.send(&ToServer::Request { id: 0, request });
            }
            for message in client.poll() {
                match message {
                    Ok(message) => receive(*player, message, &mut save, pending),
                    Err(why) => warn!("the host's message could not be read: {why:?}"),
                }
            }
            // Given to the tier once there is one.
            if fine.is_some() && contact.is_some() {
                applied.append(pending);
            }
        }
    }
    if applied.is_empty() {
        return;
    }
    if let (Some(mut fine), Some(mut contact)) = (fine, contact) {
        apply_to_tier(&mut fine, &mut contact, &save, &applied);
    }
}

/// One message from the host, on a joined game.
pub fn receive(player: u32, message: ToClient, save: &mut WorldSave, applied: &mut Vec<Edit>) {
    match message {
        ToClient::Change {
            change:
                Change::Edit {
                    cell,
                    layer,
                    material,
                },
            ..
        } => {
            let edit = Edit {
                cell,
                layer,
                material,
            };
            save.take_remote(&[edit]);
            applied.push(edit);
        }
        ToClient::Snapshot { region, bytes, .. } => {
            match pbd_core::net::digest::snapshot_edits(&bytes) {
                Some((r, edits)) if r == region => {
                    save.take_remote(&edits);
                    applied.extend(edits);
                }
                _ => warn!("region {region:#x}'s snapshot could not be read"),
            }
        }
        ToClient::Refused { id, reason } => {
            save.answered();
            info!("the host refused request {id}: {reason:?}");
        }
        ToClient::Accepted { .. } => save.answered(),
        ToClient::Clock { .. } => {}
        ToClient::Rejected { reason } => warn!("the host closed the game: {reason:?}"),
        other => debug!("player {player}: from the host, not used yet: {other:?}"),
    }
}

/// Bring the tier round to edits the server sent: the one APPLY, then one
/// relight if anything was written.
fn apply_to_tier(
    fine: &mut PlanetFine,
    contact: &mut PlanetContact,
    save: &WorldSave,
    edits: &[Edit],
) {
    let mut set = (*fine.set).clone();
    let mut written = 0;
    for &edit in edits {
        if set.apply(edit, &save.edits) == Applied::Written {
            written += 1;
        }
    }
    if written == 0 {
        return;
    }
    set.finish_edits();
    let set = Arc::new(set);
    contact.set_fine(&set);
    fine.set = set;
    fine.version += 1;
    info!(
        "{written} of {} edits from the network applied to the tier",
        edits.len()
    );
}
