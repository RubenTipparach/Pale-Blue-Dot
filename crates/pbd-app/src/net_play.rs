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
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_core::edits::Edit;
use pbd_core::inventory::Slots;
use pbd_core::net::protocol::{Change, Pose, ToClient, ToServer, forward_of, yaw_of};
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

/// How often this game sends its player's pose, s: 20 a second (design 24).
pub const PRESENCE_EVERY_S: f64 = 0.05;
/// How far behind the newest pose another player is drawn, s.
pub const DRAW_BEHIND_S: f64 = 0.1;
/// Faster than this between poses, m/s, and another player walks.
pub const WALKING_MPS: f64 = 0.3;

/// The other players this game has heard of (design 24): each one's last two
/// poses and when each arrived, on this game's clock, s. Kept in the body's
/// frame (f64), so a rebase moves nothing here.
#[derive(Resource, Default, Debug)]
pub struct Others {
    pub seen: BTreeMap<u32, Heard>,
    /// Each other player's own character, once sent (`player-character`).
    pub looks: BTreeMap<u32, pbd_core::townsfolk::look::Look>,
    /// When this game last sent its own pose, s.
    sent_s: f64,
}

/// One other player's last two poses, newest last.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heard {
    pub before: Option<(f64, Pose)>,
    pub last: (f64, Pose),
}

/// Where another player is drawn: feet in the body's frame, the way they
/// face, and whether they walk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drawn {
    pub feet: DVec3,
    pub forward: DVec3,
    pub walking: bool,
}

impl Others {
    /// Take a message about another player; `true` if it was one.
    pub fn hear(&mut self, now_s: f64, message: &ToClient) -> bool {
        match *message {
            ToClient::Presence { player, pose } => {
                let before = self.seen.get(&player).map(|h| h.last);
                self.seen.insert(
                    player,
                    Heard {
                        before,
                        last: (now_s, pose),
                    },
                );
                true
            }
            ToClient::Gone { player } => {
                self.seen.remove(&player);
                self.looks.remove(&player);
                true
            }
            ToClient::Look { player, look } => {
                self.looks.insert(player, look);
                true
            }
            _ => false,
        }
    }
}

impl Heard {
    /// Where to draw them at `now_s`: [`DRAW_BEHIND_S`] behind the newest
    /// pose, between the two along the chord, put back on the newer pose's
    /// radius; yaw turned the short way.
    pub fn at(&self, now_s: f64) -> Drawn {
        let (t1, p1) = self.last;
        let up1 = p1.at.pos.normalize();
        let Some((t0, p0)) = self.before.filter(|(t0, _)| t1 > *t0) else {
            return Drawn {
                feet: p1.at.pos,
                forward: forward_of(p1.yaw, up1),
                walking: false,
            };
        };
        let k = ((now_s - DRAW_BEHIND_S - t0) / (t1 - t0)).clamp(0.0, 1.0);
        let r = p0.at.pos.length() + (p1.at.pos.length() - p0.at.pos.length()) * k;
        let feet = p0.at.pos.lerp(p1.at.pos, k).normalize() * r;
        let mut turn = p1.yaw - p0.yaw;
        turn -= std::f32::consts::TAU * (turn / std::f32::consts::TAU).round();
        let yaw = p0.yaw + turn * k as f32;
        let speed = (p1.at.pos - p0.at.pos).length() / (t1 - t0);
        // A pose not heard for a while: they stand where they were.
        let stale = now_s - t1 > 2.0;
        Drawn {
            feet,
            forward: forward_of(yaw, feet.normalize()),
            walking: speed > WALKING_MPS && !stale,
        }
    }
}

/// This game's player's pose: the walker's feet, from the origin's own
/// methods (f64), and the camera's facing as yaw and pitch at the feet.
pub fn own_pose(origin: &pbd_core::world_frame::SimOrigin, walker: Vec3, look: Vec3) -> Pose {
    let feet = origin.with_radius(
        walker,
        origin.radius_at(walker) - f64::from(crate::walking::HALF_HEIGHT),
    );
    let at = origin.to_world(feet);
    let up = at.pos.normalize();
    let look = look.as_dvec3();
    let flat = look - up * look.dot(up);
    Pose {
        at,
        yaw: if flat.length() > 1e-6 {
            yaw_of(flat.normalize(), up)
        } else {
            0.0
        },
        pitch: look.dot(up).clamp(-1.0, 1.0).asin() as f32,
    }
}

/// Send this game's pose 20 times a second, joined or hosting (design 24).
pub fn send_presence(
    net: Option<Res<NetPlay>>,
    mut save: ResMut<WorldSave>,
    mut others: ResMut<Others>,
    time: Res<Time>,
    space: crate::SimSpace,
    walkers: Query<&Transform, With<crate::walking::Walker>>,
    cameras: Query<&GlobalTransform, With<crate::walking::WalkingCamera>>,
) {
    let Some(net) = net else { return };
    let now = time.elapsed_secs_f64();
    if now - others.sent_s < PRESENCE_EVERY_S {
        return;
    }
    let (Some(walker), Some(camera)) = (walkers.iter().next(), cameras.iter().next()) else {
        return;
    };
    others.sent_s = now;
    let pose = own_pose(&space.origin(), walker.translation, *camera.forward());
    match &*net {
        NetPlay::Host { .. } => save.bypass_change_detection().serve_presence(pose),
        NetPlay::Joined { client, .. } => client.send(&ToServer::Presence { pose }),
    }
}

/// Send this player's own look (`player-character` 4.3): once joined or
/// when a player joins this host, and again whenever it changes.
pub fn send_look(
    net: Option<ResMut<NetPlay>>,
    mut save: ResMut<WorldSave>,
    look: Option<Res<crate::character::PlayerLook>>,
    mut sent: Local<Option<pbd_core::townsfolk::look::Look>>,
    mut players_seen: Local<usize>,
) {
    let (Some(net), Some(look)) = (net, look) else {
        return;
    };
    let look = look.0;
    match &*net {
        NetPlay::Joined { client, .. } => {
            if *sent != Some(look) {
                client.send(&ToServer::Look { look });
                *sent = Some(look);
            }
        }
        NetPlay::Host { players, .. } => {
            // A new player is sent everyone's looks by the server on join,
            // the host's among them once it has given it.
            if *sent != Some(look) || players.len() > *players_seen {
                save.bypass_change_detection().serve_look(look);
                *sent = Some(look);
            }
            *players_seen = players.len();
        }
    }
}

/// Draw the other players as townsfolk figures where they are now
/// (design 24): stood and driven each frame, gone when they leave.
pub fn draw_others(
    others: Res<Others>,
    time: Res<Time>,
    folk: Option<ResMut<crate::townsfolk::Townsfolk>>,
    assets: Option<Res<crate::towns::TownAssets>>,
    mut shown: Local<Vec<u32>>,
) {
    let Some(mut folk) = folk else { return };
    let now = time.elapsed_secs_f64();
    // A tunic the figures' atlas holds: the village's first person's.
    let tunic = assets
        .as_deref()
        .and_then(|a| a.village.people.first().map(|p| p.tunic))
        .unwrap_or(0x4a6a8a);
    for (&player, heard) in &others.seen {
        let d = heard.at(now);
        let chosen = others.looks.get(&player).copied();
        folk.visit(player, d.feet, d.forward, d.walking, chosen, tunic);
    }
    for gone in shown.iter().filter(|p| !others.seen.contains_key(p)) {
        folk.unvisit(*gone);
    }
    *shown = others.seen.keys().copied().collect();
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
pub fn install_key() -> [u8; 16] {
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
    hotbar: Option<ResMut<crate::hotbar::Hotbar>>,
    drops: Option<ResMut<crate::drops::Drops>>,
    mut others: ResMut<Others>,
    time: Res<Time>,
    mut digest_s: Local<f64>,
    sun: Option<ResMut<crate::sky::Sun>>,
) {
    let mut drops = drops;
    let now = time.elapsed_secs_f64();
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
            // One region's digest to everyone, in rotation (design 29).
            if !players.is_empty() && now - *digest_s >= pbd_server::DIGEST_EVERY_S {
                *digest_s = now;
                save.serve_digest();
            }
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
                    pbd_server::To::Player(0) if others.hear(now, &message) => {}
                    pbd_server::To::Player(0) => {
                        // A guest's drop, or a pickup by anyone: the host's
                        // own live drops follow (design 23).
                        if let Some(drops) = drops.as_mut() {
                            apply_drop_news(drops, &message);
                        }
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
            for region in save.take_resyncs() {
                info!("region {region:#x} differs from the host's: asked for it whole");
                client.send(&ToServer::Resync { region });
            }
            for message in client.poll() {
                match message {
                    Ok(message) if others.hear(now, &message) => {}
                    Ok(message) => receive(*player, message, &mut save, pending),
                    Err(why) => warn!("the host's message could not be read: {why:?}"),
                }
            }
            // The host's clock, every 5 s: this game runs it forward between,
            // and is set to it only when it has drifted (design 15.8).
            if let (Some(clock_s), Some(mut sun)) = (save.remote_clock(), sun) {
                follow_clock(&mut sun.clock.seconds, clock_s);
            }
            if let (Some(slots), Some(mut hotbar)) = (save.remote_slots(), hotbar) {
                follow(&mut hotbar.0, slots);
            }
            if let Some(drops) = drops.as_mut() {
                for news in save.drop_news() {
                    apply_drop_news(drops, &news);
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
            region,
            seq,
            change:
                Change::Edit {
                    cell,
                    layer,
                    material,
                },
        } => {
            let edit = Edit {
                cell,
                layer,
                material,
            };
            save.take_change(region, seq, edit);
            applied.push(edit);
        }
        ToClient::Snapshot { region, seq, bytes } => {
            // A region whole: replaced, not merged (design 29).
            match pbd_core::net::digest::snapshot_edits(&bytes) {
                Some((r, edits)) if r == region => {
                    applied.extend(save.take_snapshot(region, seq, &edits));
                }
                _ => warn!("region {region:#x}'s snapshot could not be read"),
            }
        }
        ToClient::Digest {
            region,
            seq,
            digest,
        } => save.hear_digest(region, seq, digest),
        ToClient::Refused { id, reason } => {
            save.answered();
            info!("the host refused request {id}: {reason:?}");
        }
        ToClient::Accepted { .. } => save.answered(),
        ToClient::Slots { slots } => save.take_remote_slots(slots),
        news @ (ToClient::Drop { .. } | ToClient::DropLeft { .. }) => save.take_drop_news(news),
        ToClient::Record { record } => save.take_remote_record(record),
        ToClient::Clock { clock_s } => save.take_remote_clock(clock_s),
        ToClient::Rejected { reason } => warn!("the host closed the game: {reason:?}"),
        other => debug!("player {player}: from the host, not used yet: {other:?}"),
    }
}

/// Apply news of a drop to the live drops (design 23): a drop made is added
/// unless it is already held (the host's own dig), and a pickup sets what is
/// left, taking it away at 0. Anything else is ignored.
pub fn apply_drop_news(drops: &mut crate::drops::Drops, news: &ToClient) {
    match *news {
        ToClient::Drop { drop } => {
            if !drops.live.iter().any(|d| d.id == drop.id) {
                drops.live.push(drop);
            }
        }
        ToClient::DropLeft { id, left } => {
            if left == 0 {
                drops.live.retain(|d| d.id != id);
            } else if let Some(d) = drops.live.iter_mut().find(|d| d.id == id) {
                d.count = left;
            }
        }
        _ => {}
    }
}

/// How far this game's clock may run from the host's before it is set to
/// it, s: closer than this, the two run on together and the sky never jumps.
pub const CLOCK_DRIFT_S: f64 = 0.5;

/// A joined game's clock follows the host's (design 15.8): set to it when
/// they differ by more than [`CLOCK_DRIFT_S`], left to run otherwise.
pub fn follow_clock(mine: &mut f64, host: f64) {
    if (*mine - host).abs() > CLOCK_DRIFT_S {
        *mine = host;
    }
}

/// The hotbar takes the slots the server holds for this player (design 22),
/// keeping its own selected slot: the hand is the player's, not the server's.
pub fn follow(hotbar: &mut Slots, from_server: Slots) {
    let selected = hotbar.selected();
    *hotbar = from_server;
    hotbar.select(selected);
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

#[cfg(test)]
mod follow_tests {
    use super::*;
    use pbd_core::inventory::{Item, Stack};
    use pbd_core::terrain::Material;

    /// Design 15.8: a joined game's clock is set to the host's only when
    /// it has drifted past half a second.
    #[test]
    fn the_clock_follows_the_host_only_when_it_drifts() {
        let mut mine = 1200.2;
        follow_clock(&mut mine, 1200.0);
        assert_eq!(mine, 1200.2, "close enough: runs on");
        follow_clock(&mut mine, 1210.0);
        assert_eq!(mine, 1210.0, "drifted: set to the host's");
    }

    /// Design 22: the joined game's hotbar takes the server's slots whole
    /// and keeps the slot its player has selected.
    #[test]
    fn the_hotbar_takes_the_servers_slots_and_keeps_its_selection() {
        let mut hotbar = Slots::new();
        hotbar.set(1, Some(Stack::new(Item::Block(Material::Dirt), 4)));
        hotbar.select(6);
        let mut from_server = Slots::new();
        from_server.set(0, Some(Stack::new(Item::Block(Material::Stone), 3)));
        follow(&mut hotbar, from_server.clone());
        assert_eq!(hotbar.selected(), 6);
        assert_eq!(hotbar.get(0), from_server.get(0));
        assert_eq!(hotbar.get(1), None);
    }

    /// Design 23: a drop the host already holds is not added twice, and a
    /// pickup sets what is left, taking the drop away at 0.
    #[test]
    fn drop_news_adds_a_drop_once_and_a_pickup_sets_what_is_left() {
        let drop = pbd_core::drops::ItemDrop {
            id: 3,
            item: Item::Block(Material::Dirt),
            count: 2,
            position: bevy::math::DVec3::new(1.0, 2.0, 3.0),
            made_s: 4.0,
        };
        let mut drops = crate::drops::Drops::default();
        apply_drop_news(&mut drops, &ToClient::Drop { drop });
        apply_drop_news(&mut drops, &ToClient::Drop { drop });
        assert_eq!(drops.live, vec![drop]);
        apply_drop_news(&mut drops, &ToClient::DropLeft { id: 3, left: 1 });
        assert_eq!(drops.live[0].count, 1);
        apply_drop_news(&mut drops, &ToClient::DropLeft { id: 3, left: 0 });
        assert!(drops.live.is_empty());
    }

    fn pose(at: DVec3, yaw: f32) -> Pose {
        Pose {
            at: pbd_core::world_frame::WorldPos::new(pbd_core::world_frame::FrameId(0), at),
            yaw,
            pitch: 0.0,
        }
    }

    /// Design 24: another player is drawn 100 ms behind the newest pose,
    /// between the two on the sphere, walking when they move; the first pose
    /// alone stands them still; `Gone` forgets them.
    #[test]
    fn another_player_is_drawn_behind_between_poses_on_the_sphere() {
        let r = 4800.0;
        let a = DVec3::new(0.0, r, 0.0);
        let b = (DVec3::new(2.0, r, 0.0)).normalize() * r;
        let mut others = Others::default();
        assert!(others.hear(
            1.0,
            &ToClient::Presence {
                player: 2,
                pose: pose(a, 0.0)
            }
        ));
        let first = others.seen[&2].at(1.0);
        assert_eq!(first.feet, a);
        assert!(!first.walking);
        others.hear(
            1.05,
            &ToClient::Presence {
                player: 2,
                pose: pose(b, 0.4),
            },
        );
        // At 1.125 the drawn time is 1.025: halfway between the two poses.
        let mid = others.seen[&2].at(1.125);
        assert!((mid.feet.length() - r).abs() < 1e-6, "on the sphere");
        let half = (a.lerp(b, 0.5)).normalize() * r;
        assert!((mid.feet - half).length() < 1e-6);
        assert!(mid.walking, "2 m in 50 ms is walking");
        assert!(!others.hear(1.1, &ToClient::Clock { clock_s: 0.0 }));
        assert!(others.hear(1.2, &ToClient::Gone { player: 2 }));
        assert!(others.seen.is_empty());
    }

    /// Design 24: two games whose origins differ (one has rebased) put the
    /// same pose at the same world point, and their own pose round trips.
    #[test]
    fn a_pose_does_not_depend_on_either_games_origin() {
        use pbd_core::world_frame::SimOrigin;
        let walker_world = DVec3::new(30.0, 4800.9, -12.0);
        let here = SimOrigin {
            frame: pbd_core::world_frame::FrameId(0),
            origin: DVec3::ZERO,
        };
        let there = SimOrigin {
            origin: DVec3::new(25.0, 4790.0, -10.0),
            ..here
        };
        let look = Vec3::new(0.3, -0.2, 0.9).normalize();
        let from = |o: &SimOrigin| {
            let local = o.to_local(pbd_core::world_frame::WorldPos::new(o.frame, walker_world));
            own_pose(o, local, look)
        };
        let (p, q) = (from(&here), from(&there));
        assert!((p.at.pos - q.at.pos).length() < 1e-3);
        assert!((p.yaw - q.yaw).abs() < 1e-4);
        assert!((p.at.pos.length() - (walker_world.length() - 0.9)).abs() < 1e-3);
    }
}
