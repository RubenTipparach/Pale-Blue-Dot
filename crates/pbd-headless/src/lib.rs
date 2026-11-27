//! A dedicated server and protocol bots with no window (`multiplayer`
//! design 21, task 6.5).
//!
//! - [`serve`] runs the same [`pbd_server::Server`] the host's game runs,
//!   over a [`MemoryStore`], behind `pbd-net`'s [`Host`]: a dedicated server
//!   whose world lives in memory.
//! - [`Bot`] is a joined player with no window: it keeps its own copy of the
//!   world's edits from what the server sends, plays the hut script through
//!   the requests a player's hands send, and checks the protocol as it goes:
//!   every request answered once, each region's changes in order with no
//!   gap, and at the end its copy of every region the same as the server's.
//!
//! Pickups do not cross the network yet (task 8.2), so the server deals a
//! joining player a kit of stone to build with ([`ServeOptions::kit`]).
//!
//! locale: dev

use glam::Vec3;
use pbd_core::column;
use pbd_core::edits::{Edit, Edits};
use pbd_core::inventory::{Item, Slots};
use pbd_core::net::digest::{Regions, fnv1a64, snapshot_edits};
use pbd_core::net::protocol::{
    Change, PROTOCOL, PROTOCOL_MIN, Refusal, Request, ToClient, ToServer, Versions,
};
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::terrain::{GENERATOR_VERSION, Material, TOPOLOGY_VERSION};
use pbd_core::worms::{self, WormField};
use pbd_net::{Client, Host, HostEvent};
use pbd_server::{MemoryStore, Server, To};
use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub mod disk;

/// The game's default spawn (`desktop::spawn_direction`), where a joined
/// game stands when it opens.
pub const DEFAULT_SPAWN: Vec3 = Vec3::new(0.8776, 0.4794, 0.0);

/// How long a bot waits for any one answer before it calls the request lost.
pub const ANSWER_WAIT: Duration = Duration::from_secs(10);

/// One line of the log, with the seconds since `start`.
fn log(start: Instant, who: &str, line: &str) {
    eprintln!("[{:8.2}] {who}: {line}", start.elapsed().as_secs_f64());
}

// ---------------------------------------------------------------- serve

/// A dedicated server's settings.
#[derive(Clone, Debug)]
pub struct ServeOptions {
    pub addr: SocketAddr,
    /// The world's seed, which a joining game opens.
    pub seed: u64,
    /// World time at start, s (`daylight::DAY_S` is a day; 1200 s is 10:00).
    pub clock_s: f64,
    /// Stone blocks dealt to each joining player (task 8.2 stands in).
    pub kit: u16,
    /// Stop after this long, if given.
    pub seconds: Option<f64>,
    /// Keep the world on disk in this folder (design 27): opened if it holds
    /// one, made with `seed` if not. `None` keeps it in memory.
    pub world: Option<std::path::PathBuf>,
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 47124)),
            seed: 77,
            clock_s: 1200.0,
            kit: 64,
            seconds: None,
            world: None,
        }
    }
}

/// What a server did, counted from its outbox.
#[derive(Clone, Debug, Default)]
pub struct Served {
    pub joined: u32,
    pub requests: u64,
    pub accepted: u64,
    pub refused: u64,
    pub changes_sent: u64,
}

/// The kit a joining player is dealt: `kit` stone in the first slots, the
/// first selected.
pub fn kit(count: u16) -> Slots {
    let mut slots = Slots::new();
    slots.give(Item::Block(Material::Stone), count);
    slots.select(0);
    slots
}

/// Bind and run a dedicated server until `stop` is set or its time is up.
/// `ready` is told the address once it listens.
pub fn serve(
    options: &ServeOptions,
    stop: &AtomicBool,
    ready: Option<std::sync::mpsc::Sender<SocketAddr>>,
) -> std::io::Result<Served> {
    match &options.world {
        Some(dir) => {
            let store = disk::DiskStore::open(dir, options.seed)?;
            let world = (store.identity.seed, store.identity.generator);
            serve_on(options, world, stop, ready, store)
        }
        None => serve_on(
            options,
            (options.seed, GENERATOR_VERSION),
            stop,
            ready,
            MemoryStore::new(Edits::new()),
        ),
    }
}

/// [`serve`] over `store`, the world `(seed, generator)`.
fn serve_on<S: pbd_server::Store>(
    options: &ServeOptions,
    (seed, generator): (u64, u32),
    stop: &AtomicBool,
    ready: Option<std::sync::mpsc::Sender<SocketAddr>>,
    mut store: S,
) -> std::io::Result<Served> {
    let start = Instant::now();
    let host = Host::bind_to(options.addr)?;
    let addr = host.local_addr();
    let mut server = Server::new(
        pbd_server::World {
            terrain: TerrainConfig::for_version(generator).ok_or_else(|| {
                std::io::Error::other(format!("generator {generator} is not in this build"))
            })?,
            worms: WormField::DEFAULT,
            versions: Versions {
                protocol: PROTOCOL,
                protocol_min: PROTOCOL_MIN,
                topology: TOPOLOGY_VERSION,
                generator,
                content: 0,
                world: seed,
            },
        },
        options.clock_s,
    );
    let mut players: BTreeMap<u64, u32> = BTreeMap::new();
    let mut served = Served::default();
    log(
        start,
        "serve",
        &format!(
            "hosting on {addr} (seed {seed}, {})",
            options
                .world
                .as_ref()
                .map_or("world in memory".to_string(), |d| format!(
                    "world in {}",
                    d.display()
                ))
        ),
    );
    log(
        start,
        "serve",
        &format!(
            "each player is dealt {} stone: pickups wait for task 8.2",
            options.kit
        ),
    );
    if let Some(ready) = ready {
        let _ = ready.send(addr);
    }
    let tick = Duration::from_secs_f64(pbd_server::TICK_S);
    let mut last_tick = Instant::now();
    let mut last_digest = Instant::now();
    let mut last_report = Instant::now();
    loop {
        if stop.load(Ordering::Relaxed)
            || options
                .seconds
                .is_some_and(|s| start.elapsed().as_secs_f64() >= s)
        {
            break;
        }
        for event in host.poll() {
            match event {
                HostEvent::Opened(c) => log(start, "serve", &format!("a player connected ({c})")),
                HostEvent::Message(c, hello @ ToServer::Hello { .. })
                    if !players.contains_key(&c) =>
                {
                    match server.join(c, &hello, kit(options.kit)) {
                        Some(player) => {
                            served.joined += 1;
                            let name = match &hello {
                                ToServer::Hello { name, .. } if !name.is_empty() => name.clone(),
                                _ => "a game".into(),
                            };
                            log(
                                start,
                                "serve",
                                &format!("player {player} joined ({c}): {name}"),
                            );
                            drop_stale(&host, &mut players, player, |old| {
                                log(
                                    start,
                                    "serve",
                                    &format!("player {player} rejoined: dropped ({old})"),
                                )
                            });
                            players.insert(c, player);
                            server.snapshot_all(&store, player, None);
                        }
                        None => log(start, "serve", &format!("a join was refused ({c})")),
                    }
                }
                HostEvent::Message(c, message) => {
                    if let Some(&player) = players.get(&c) {
                        if matches!(message, ToServer::Request { .. }) {
                            served.requests += 1;
                        }
                        server.handle(&mut store, player, message);
                    }
                }
                HostEvent::Unreadable(c, why) => log(
                    start,
                    "serve",
                    &format!("a message could not be read ({c}): {why:?}"),
                ),
                HostEvent::Closed(c) => {
                    if let Some(player) = players.remove(&c) {
                        server.leave(player);
                        log(start, "serve", &format!("player {player} left ({c})"));
                    }
                }
            }
        }
        while last_tick.elapsed() >= tick {
            last_tick += tick;
            server.tick(pbd_server::TICK_S);
        }
        // One region's digest to everyone, in rotation (design 29).
        if !players.is_empty() && last_digest.elapsed().as_secs_f64() >= pbd_server::DIGEST_EVERY_S
        {
            last_digest = Instant::now();
            server.send_digest(&store);
        }
        for (to, message) in server.drain() {
            match &message {
                ToClient::Accepted { .. } => served.accepted += 1,
                ToClient::Refused { id, reason } => {
                    served.refused += 1;
                    if let To::Player(p) = to {
                        log(
                            start,
                            "serve",
                            &format!("player {p}: request {id} refused: {reason:?}"),
                        );
                    }
                }
                ToClient::Change { .. } => served.changes_sent += 1,
                _ => {}
            }
            match to {
                To::Player(p) => {
                    if let Some((&c, _)) = players.iter().find(|(_, n)| **n == p) {
                        host.send(c, &message);
                    }
                }
                To::Connection(c) => {
                    host.send(c, &message);
                    host.close(c);
                }
            }
        }
        if last_report.elapsed() >= Duration::from_secs(10) {
            last_report = Instant::now();
            log(
                start,
                "serve",
                &format!(
                    "SERVED players={} requests={} accepted={} refused={} changes_sent={} regions={} clock={:.0}s",
                    players.len(),
                    served.requests,
                    served.accepted,
                    served.refused,
                    served.changes_sent,
                    store.ledger().regions().regions().len(),
                    server.clock_s()
                ),
            );
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    log(
        start,
        "serve",
        &format!(
            "stopped: joined={} requests={} accepted={} refused={} changes_sent={}",
            served.joined, served.requests, served.accepted, served.refused, served.changes_sent
        ),
    );
    Ok(served)
}

/// A player who joins again (the same install's key) before the server has
/// seen their old connection close: the old connection is forgotten and
/// closed, so the player's messages go to the new one, and the old one's
/// close, when it comes, does not take the player out of the game.
pub fn drop_stale(host: &Host, players: &mut BTreeMap<u64, u32>, player: u32, said: impl Fn(u64)) {
    let stale: Vec<u64> = players
        .iter()
        .filter(|(_, p)| **p == player)
        .map(|(c, _)| *c)
        .collect();
    for c in stale {
        players.remove(&c);
        host.close(c);
        said(c);
    }
}

// ---------------------------------------------------------------- the hut

/// The cells of the build site: the centre, its ring, and the two rings
/// beyond it.
#[derive(Clone, Debug)]
pub struct Site {
    pub centre: u32,
    pub ring: Vec<u32>,
    pub ring2: Vec<u32>,
    pub ring3: Vec<u32>,
}

/// The ring of cells next to `inner` that are in none of `seen`, sorted.
fn ring_beyond(inner: &[u32], seen: &BTreeSet<u32>) -> Vec<u32> {
    let mut out = BTreeSet::new();
    for &c in inner {
        for n in pbd_core::roads::neighbours(c) {
            if !seen.contains(&n) {
                out.insert(n);
            }
        }
    }
    out.into_iter().collect()
}

impl Site {
    /// The site three cells from `near` (about 8.5 m), away from it: close
    /// enough that a game standing at `near` sees it, far enough that the
    /// hut is not built on it.
    pub fn near(near: Vec3) -> Option<Site> {
        let start = pbd_core::roads::nearest_key(near)?;
        let from = pbd_core::roads::centre(start)?;
        let mut centre = start;
        for _ in 0..3 {
            centre = pbd_core::roads::neighbours(centre)
                .into_iter()
                .max_by(|a, b| {
                    let da = pbd_core::roads::centre(*a).map_or(0.0, |d| d.distance(from));
                    let db = pbd_core::roads::centre(*b).map_or(0.0, |d| d.distance(from));
                    da.total_cmp(&db).then(a.cmp(b))
                })?;
        }
        Some(Site::at(centre))
    }

    pub fn at(centre: u32) -> Site {
        let mut seen: BTreeSet<u32> = BTreeSet::from([centre]);
        let ring = ring_beyond(&[centre], &seen);
        seen.extend(&ring);
        let ring2 = ring_beyond(&ring, &seen);
        seen.extend(&ring2);
        let ring3 = ring_beyond(&ring2, &seen);
        Site {
            centre,
            ring,
            ring2,
            ring3,
        }
    }
}

/// One step of a bot's script.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    /// A wall block `height` (1 or 2) above the cell's own ground.
    Wall { cell: u32, height: u16 },
    /// A roof block on the roof's one layer.
    Roof { cell: u32 },
}

/// The hut's pieces in build order: the walls (the ring's first cell left
/// open as the door), then the roof over the ring; the centre's roof block,
/// which every bot tries, is not in this list.
pub fn hut(site: &Site) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for height in [1, 2] {
        for &cell in site.ring.iter().skip(1) {
            pieces.push(Piece::Wall { cell, height });
        }
    }
    for &cell in &site.ring {
        pieces.push(Piece::Roof { cell });
    }
    pieces
}

// ---------------------------------------------------------------- the bot

/// A bot's settings.
#[derive(Clone, Debug)]
pub struct BotOptions {
    pub name: String,
    /// This bot's share of the hut: `role` of `of`.
    pub role: usize,
    pub of: usize,
    /// The point the site is chosen near.
    pub near: Vec3,
    /// Seconds of random digs and places round the site after the hut.
    pub stress_s: f64,
    /// Requests a second, as a player's swing.
    pub rate_hz: f64,
    /// Seconds to wait between pieces, so a watching game sees it go up.
    pub pace_s: f64,
}

impl Default for BotOptions {
    fn default() -> Self {
        Self {
            name: "bot".into(),
            role: 0,
            of: 1,
            near: DEFAULT_SPAWN.normalize(),
            stress_s: 0.0,
            rate_hz: 4.0,
            pace_s: 0.0,
        }
    }
}

/// What a bot saw.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub name: String,
    pub player: u32,
    pub requests: u64,
    pub accepted: u64,
    pub refused: BTreeMap<String, u64>,
    pub changes: u64,
    pub regions_checked: usize,
    pub rtt_ms: Vec<f64>,
    pub failures: Vec<String>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.failures.is_empty()
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.rtt_ms.is_empty() {
            return 0.0;
        }
        let mut v = self.rtt_ms.clone();
        v.sort_by(f64::total_cmp);
        v[((v.len() - 1) as f64 * p).round() as usize]
    }

    /// The one line a run is judged by.
    pub fn line(&self) -> String {
        format!(
            "{} {} player={} requests={} accepted={} refused={:?} changes={} regions_checked={} rtt_ms p50={:.1} p99={:.1} max={:.1}{}",
            if self.ok() { "BOT_OK" } else { "BOT_FAIL" },
            self.name,
            self.player,
            self.requests,
            self.accepted,
            self.refused,
            self.changes,
            self.regions_checked,
            self.percentile(0.5),
            self.percentile(0.99),
            self.percentile(1.0),
            if self.ok() {
                String::new()
            } else {
                format!(" failures={:?}", self.failures)
            }
        )
    }
}

/// A request's answer: its change's region and sequence, or the refusal.
type Answer = Result<(u32, u64), Refusal>;

/// A joined player with no window.
pub struct Bot {
    client: Client,
    options: BotOptions,
    start: Instant,
    terrain: TerrainConfig,
    worms: WormField,
    edits: Edits,
    regions: Regions,
    seqs: BTreeMap<u32, u64>,
    /// Requests sent and not answered, by id.
    pending: BTreeMap<u32, Instant>,
    answers: BTreeMap<u32, Answer>,
    /// Resync replies waited for.
    checking: BTreeSet<u32>,
    next_id: u32,
    kit_left: u16,
    rng: u64,
    pub report: Report,
}

impl Bot {
    /// Connect to `addr`, join with the name, and take the welcome and the
    /// snapshots that follow it.
    pub fn join(addr: SocketAddr, options: BotOptions, kit: u16) -> Result<Bot, String> {
        let start = Instant::now();
        let client = Client::connect(addr, Duration::from_secs(30)).map_err(|e| e.to_string())?;
        let mut key = [0u8; 16];
        let h = fnv1a64(options.name.as_bytes());
        key[..8].copy_from_slice(&h.to_le_bytes());
        key[8..].copy_from_slice(&pbd_core::fauna::mix(h).to_le_bytes());
        client.send(&ToServer::Hello {
            versions: Versions {
                protocol: PROTOCOL,
                protocol_min: PROTOCOL_MIN,
                topology: TOPOLOGY_VERSION,
                // Any world this build carries, as a joining game names it.
                generator: 0,
                content: 0,
                world: 0,
            },
            key,
            name: options.name.clone(),
        });
        let mut early = Vec::new();
        let (player, generator) = loop {
            let mut got = None;
            for message in client.poll() {
                match message {
                    Ok(ToClient::Welcome {
                        player, generator, ..
                    }) if got.is_none() => got = Some((player, generator)),
                    Ok(ToClient::Rejected { reason }) => {
                        return Err(format!("rejected: {reason:?}"));
                    }
                    Ok(other) => early.push(other),
                    Err(bad) => return Err(format!("unreadable: {bad:?}")),
                }
            }
            if let Some(got) = got {
                break got;
            }
            if client.is_closed() {
                return Err("closed before the welcome".into());
            }
            if start.elapsed() > Duration::from_secs(30) {
                return Err("no welcome".into());
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        let terrain = TerrainConfig::for_version(generator)
            .ok_or_else(|| format!("generator {generator} is not in this build"))?;
        let rng = pbd_core::fauna::mix(fnv1a64(options.name.as_bytes()) ^ 0x9e37);
        let mut bot = Bot {
            client,
            report: Report {
                name: options.name.clone(),
                player,
                ..Report::default()
            },
            options,
            start,
            terrain,
            worms: WormField::DEFAULT,
            edits: Edits::new(),
            regions: Regions::default(),
            seqs: BTreeMap::new(),
            pending: BTreeMap::new(),
            answers: BTreeMap::new(),
            checking: BTreeSet::new(),
            next_id: 1,
            kit_left: kit,
            rng,
        };
        bot.say(&format!("joined as player {player}"));
        for message in early {
            bot.receive(message);
        }
        Ok(bot)
    }

    fn say(&self, line: &str) {
        log(self.start, &self.options.name, line);
    }

    fn fail(&mut self, why: String) {
        self.say(&format!("FAIL {why}"));
        self.report.failures.push(why);
    }

    fn random(&mut self) -> u64 {
        self.rng = pbd_core::fauna::mix(self.rng);
        self.rng
    }

    /// Fold one message from the server into the copy and the answers.
    fn receive(&mut self, message: ToClient) {
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
                self.report.changes += 1;
                let last = self.seqs.get(&region).copied().unwrap_or(0);
                if seq != last + 1 {
                    self.fail(format!("region {region:#x}: change {seq} after {last}"));
                }
                self.seqs.insert(region, seq);
                self.edits.set(Edit {
                    cell,
                    layer,
                    material,
                });
                self.regions.note(cell, &self.edits);
            }
            ToClient::Snapshot { region, seq, bytes } => {
                if self.checking.remove(&region) {
                    // The server's region as of everything it sent before
                    // it, which the copy has taken in order.
                    let mine = self.regions.digest(region);
                    let theirs = fnv1a64(&bytes);
                    let my_seq = self.seqs.get(&region).copied().unwrap_or(0);
                    if mine != theirs || my_seq != seq {
                        self.fail(format!(
                            "region {region:#x}: copy {mine:#x} at {my_seq}, server {theirs:#x} at {seq}"
                        ));
                    }
                    self.report.regions_checked += 1;
                    return;
                }
                match snapshot_edits(&bytes) {
                    Some((r, edits)) if r == region => {
                        for edit in edits {
                            self.edits.set(edit);
                            self.regions.note(edit.cell, &self.edits);
                        }
                        self.seqs.insert(region, seq);
                    }
                    _ => self.fail(format!("region {region:#x}: snapshot unreadable")),
                }
            }
            ToClient::Accepted { id, region, seq } => self.answer(id, Ok((region, seq))),
            ToClient::Refused { id, reason } => self.answer(id, Err(reason)),
            ToClient::Rejected { reason } => self.fail(format!("closed by the server: {reason:?}")),
            _ => {}
        }
    }

    fn answer(&mut self, id: u32, answer: Answer) {
        match self.pending.remove(&id) {
            Some(sent) => {
                self.report
                    .rtt_ms
                    .push(sent.elapsed().as_secs_f64() * 1000.0);
                match &answer {
                    Ok(_) => self.report.accepted += 1,
                    Err(reason) => {
                        *self
                            .report
                            .refused
                            .entry(format!("{reason:?}"))
                            .or_default() += 1
                    }
                }
                self.answers.insert(id, answer);
            }
            None => self.fail(format!("request {id} answered twice, or never asked")),
        }
    }

    fn pump(&mut self) {
        for message in self.client.poll() {
            match message {
                Ok(message) => self.receive(message),
                Err(bad) => self.fail(format!("unreadable: {bad:?}")),
            }
        }
        if self.client.is_closed()
            && !self
                .report
                .failures
                .iter()
                .any(|f| f == "connection closed")
        {
            self.fail("connection closed".into());
        }
    }

    /// Send one request and wait for its answer and, if accepted, for its
    /// change to come back.
    fn ask(&mut self, request: Request) -> Option<Answer> {
        let id = self.next_id;
        self.next_id += 1;
        self.pending.insert(id, Instant::now());
        self.report.requests += 1;
        self.client.send(&ToServer::Request { id, request });
        let sent = Instant::now();
        let answer = loop {
            self.pump();
            if let Some(answer) = self.answers.remove(&id) {
                break answer;
            }
            if sent.elapsed() > ANSWER_WAIT || self.client.is_closed() {
                self.pending.remove(&id);
                self.fail(format!("request {id} ({request:?}) never answered"));
                return None;
            }
            std::thread::sleep(Duration::from_millis(1));
        };
        if let Ok((region, seq)) = answer {
            while self.seqs.get(&region).copied().unwrap_or(0) < seq {
                self.pump();
                if sent.elapsed() > ANSWER_WAIT || self.client.is_closed() {
                    self.fail(format!(
                        "request {id}: change {seq} of {region:#x} never came"
                    ));
                    return Some(answer);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        Some(answer)
    }

    /// The cell's column as this bot's copy has it.
    fn column(&self, cell: u32) -> Option<column::Column> {
        let direction = pbd_core::cell_key::key_direction(cell)?;
        let near = worms::gather(&self.worms, &self.terrain, direction, 0.0);
        Some(column::generate_edited(
            &near,
            &self.worms,
            &self.terrain,
            direction,
            self.edits.for_cell(cell),
        ))
    }

    fn surface(&self, cell: u32) -> Option<u16> {
        self.column(cell)?.surface().map(|l| l as u16)
    }

    fn open(&self, cell: u32, layer: u16) -> bool {
        self.column(cell).is_some_and(|c| {
            let m = c.material(usize::from(layer));
            m == Material::Air || m.is_water()
        })
    }

    /// Place stone at the layer if it is open and the kit has any; a refusal
    /// other than `Unchanged` (a race with another hand) is a failure.
    fn place(&mut self, cell: u32, layer: u16) {
        if self.kit_left == 0 || !self.open(cell, layer) {
            return;
        }
        match self.ask(Request::Place {
            cell,
            layer,
            slot: 0,
        }) {
            Some(Ok(_)) => self.kit_left -= 1,
            Some(Err(Refusal::Unchanged)) | None => {}
            Some(Err(other)) => self.fail(format!("place {cell:#x}/{layer} refused: {other:?}")),
        }
    }

    fn dig(&mut self, cell: u32, layer: u16) -> Option<Answer> {
        let answer = self.ask(Request::Dig { cell, layer })?;
        match answer {
            Ok(_) | Err(Refusal::Unchanged) => {}
            Err(other) => self.fail(format!("dig {cell:#x}/{layer} refused: {other:?}")),
        }
        Some(answer)
    }

    fn pace(&mut self) {
        let until = Instant::now() + Duration::from_secs_f64(self.options.pace_s);
        while Instant::now() < until {
            self.pump();
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    /// The whole script: a pit, this bot's share of the hut, the centre's
    /// roof block, stress, then the check. Returns the report.
    pub fn play(mut self) -> Report {
        let Some(site) = Site::near(self.options.near) else {
            self.fail("no site near the given point".into());
            return self.report;
        };
        let (role, of) = (self.options.role, self.options.of.max(1));
        self.say(&format!(
            "site {:#x}, ring {}, role {role} of {of}",
            site.centre,
            site.ring.len()
        ));
        // A pit of two layers in this bot's own cell, and the bottom dug
        // again: the second must be refused, the cell already being air.
        if !site.ring2.is_empty() {
            let pit = site.ring2[(role * 3) % site.ring2.len()];
            if let Some(top) = self.surface(pit) {
                self.dig(pit, top);
                self.pace();
                self.dig(pit, top - 1);
                self.pace();
                match self.dig(pit, top - 1) {
                    Some(Err(Refusal::Unchanged)) => {}
                    other => self.fail(format!("a dig of air was answered {other:?}")),
                }
            }
        }
        // The hut: walls on the ring at each cell's own ground, the roof one
        // above the highest wall.
        let pieces = hut(&site);
        let wall_top = site
            .ring
            .iter()
            .filter_map(|&c| self.surface(c))
            .max()
            .unwrap_or(0);
        let roof = wall_top + 3;
        for (index, piece) in pieces.iter().enumerate() {
            if index % of != role {
                continue;
            }
            match *piece {
                Piece::Wall { cell, height } => {
                    if let Some(ground) = self.surface_unbuilt(cell, &site) {
                        self.place(cell, ground + height);
                    }
                }
                Piece::Roof { cell } => self.place(cell, roof),
            }
            self.pace();
        }
        // Everyone tries the centre's roof block: one is taken, the rest
        // refused as `Unchanged`.
        self.place(site.centre, roof);
        // Stress: random digs and places in the third ring at the swing rate.
        if self.options.stress_s > 0.0 && !site.ring3.is_empty() {
            let until = Instant::now() + Duration::from_secs_f64(self.options.stress_s);
            let gap = Duration::from_secs_f64(1.0 / self.options.rate_hz.max(0.1));
            while Instant::now() < until && !self.client.is_closed() {
                let at = Instant::now();
                let cell = site.ring3[(self.random() % site.ring3.len() as u64) as usize];
                if let Some(top) = self.surface(cell) {
                    if self.random().is_multiple_of(2) && self.kit_left > 0 {
                        self.place(cell, top + 1);
                    } else if top > 1 {
                        self.dig(cell, top);
                    }
                }
                while at.elapsed() < gap {
                    self.pump();
                    std::thread::sleep(Duration::from_millis(2));
                }
            }
        }
        self.check();
        self.report
    }

    /// The ground a wall stands on: the cell's surface, less any of the
    /// hut's own stone already on it (so a second course sits on the
    /// ground's line, not on the first).
    fn surface_unbuilt(&self, cell: u32, _site: &Site) -> Option<u16> {
        let direction = pbd_core::cell_key::key_direction(cell)?;
        let near = worms::gather(&self.worms, &self.terrain, direction, 0.0);
        let ground = column::generate_edited(&near, &self.worms, &self.terrain, direction, &[]);
        ground.surface().map(|l| l as u16)
    }

    /// Ask for every region the copy holds and compare each with the copy.
    pub fn check(&mut self) {
        let regions = self.regions.regions();
        for &region in &regions {
            self.checking.insert(region);
            self.client.send(&ToServer::Resync { region });
        }
        let sent = Instant::now();
        while !self.checking.is_empty() {
            self.pump();
            if sent.elapsed() > ANSWER_WAIT || self.client.is_closed() {
                let left = self.checking.len();
                self.fail(format!("{left} regions never came back to be checked"));
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        self.say(&self.report.line());
    }
}

// ---------------------------------------------------------------- soak

/// A server and `bots` bots in one process over loopback: every bot's
/// report, after the server has stopped.
pub fn soak(bots: usize, stress_s: f64, kit: u16) -> Vec<Report> {
    let stop = Arc::new(AtomicBool::new(false));
    let (ready_tx, ready) = std::sync::mpsc::channel();
    let options = ServeOptions {
        addr: SocketAddr::from(([127, 0, 0, 1], 0)),
        kit,
        ..ServeOptions::default()
    };
    let server = {
        let stop = stop.clone();
        std::thread::spawn(move || serve(&options, &stop, Some(ready_tx)))
    };
    let addr = ready
        .recv_timeout(Duration::from_secs(10))
        .expect("the server listens");
    let handles: Vec<_> = (0..bots)
        .map(|role| {
            std::thread::spawn(move || {
                let options = BotOptions {
                    name: format!("bot{role}"),
                    role,
                    of: bots,
                    stress_s,
                    ..BotOptions::default()
                };
                match Bot::join(addr, options, kit) {
                    Ok(bot) => bot.play(),
                    Err(why) => Report {
                        name: format!("bot{role}"),
                        failures: vec![why],
                        ..Report::default()
                    },
                }
            })
        })
        .collect();
    let reports = handles
        .into_iter()
        .map(|h| h.join().expect("a bot thread"))
        .collect();
    stop.store(true, Ordering::Relaxed);
    let _ = server.join();
    reports
}

#[cfg(test)]
mod tests;
