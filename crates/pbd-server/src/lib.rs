//! The server: the one authority over a shared world (`multiplayer` design
//! 4 and 5, task 4.1).
//!
//! It holds the world's edits and every player's slots. A client sends what
//! its player tried ([`ToServer::Request`]); the server decides it with the
//! same core rule a client predicts with (`pbd_core::accept`), numbers it in
//! its region (`net::ledger`), hands it to the [`Journal`], and only then
//! answers [`ToClient::Accepted`] and sends the change to every player, the
//! requester included, in that order. A refusal carries no undo: the
//! requester is sent the cell's layers and its own slots again (design 15.3).
//!
//! It is plain data and a [`Server::handle`] and [`Server::tick`] pair, so a
//! test drives it with no thread or socket, single player runs it in process
//! and a dedicated server runs it on its own thread at [`TICK_S`].

use glam::Vec3;
use pbd_core::accept::{self, Ask, Hands, Refused};
use pbd_core::column::{self, Column};
use pbd_core::edits::{Edit, Edits};
use pbd_core::inventory::{Item, Slots};
use pbd_core::net::ledger::{Ledger, Sequenced};
use pbd_core::net::protocol::{
    Change, JoinRefusal, Refusal, Request, ToClient, ToServer, Versions, agree,
};
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::Author;
use pbd_core::terrain::Material;
use pbd_core::worms::{self, WormField};
use std::collections::BTreeMap;

/// The server's tick, s: 20 a second (design 5).
pub const TICK_S: f64 = 1.0 / 20.0;
/// How often the clock is sent, s (design 15.8).
pub const CLOCK_EVERY_S: f64 = 5.0;
/// Players at most (design 14.2).
pub const MAX_PLAYERS: usize = 8;

/// Where an accepted change is made durable. The server answers a change
/// as accepted only after this has taken it (design 4, "the durability rule
/// does not move"); the app's implementation is the save's writer.
pub trait Journal {
    /// Take one accepted edit, by its author, with the author's slots after
    /// it. `false` refuses it, and the server refuses the request.
    fn edit(&mut self, author: &Author, sequenced: &Sequenced, carried: &Slots) -> bool;
}

/// A journal that keeps everything in memory: tests and a world with no
/// disk behind it.
#[derive(Clone, Debug, Default)]
pub struct MemoryJournal {
    pub entries: Vec<(Author, Sequenced, Slots)>,
    /// Refuse everything from here on, as a failed writer does.
    pub failed: bool,
}

impl Journal for MemoryJournal {
    fn edit(&mut self, author: &Author, sequenced: &Sequenced, carried: &Slots) -> bool {
        if self.failed {
            return false;
        }
        self.entries
            .push((author.clone(), *sequenced, carried.clone()));
        true
    }
}

/// Who a message is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum To {
    /// One player.
    Player(u32),
    /// A connection that has not joined (its refusal).
    Connection(u64),
}

/// The world as the server holds it: what a column is made from.
pub struct World {
    pub terrain: TerrainConfig,
    pub worms: WormField,
    /// The versions a joining build must agree with.
    pub versions: Versions,
}

/// One joined player.
#[derive(Clone, Debug)]
pub struct Player {
    pub name: String,
    pub slots: Slots,
}

pub struct Server<J: Journal> {
    world: World,
    edits: Edits,
    ledger: Ledger,
    /// By number; iterated in number order, so a broadcast's order is fixed.
    players: BTreeMap<u32, Player>,
    /// Players' install keys to their numbers, kept across leaving.
    known: Vec<([u8; 16], u32)>,
    journal: J,
    clock_s: f64,
    since_clock_s: f64,
    outbox: Vec<(To, ToClient)>,
    /// What floated out of accepted digs and has not been picked up: drops
    /// become region records in task 8.2.
    pub floating: Vec<(u32, Item)>,
}

impl<J: Journal> Server<J> {
    /// A server over `edits`, the world as its save holds them, at world
    /// time `clock_s`.
    pub fn new(world: World, edits: Edits, clock_s: f64, journal: J) -> Self {
        Self {
            world,
            ledger: Ledger::of(&edits),
            edits,
            players: BTreeMap::new(),
            known: Vec::new(),
            journal,
            clock_s,
            since_clock_s: 0.0,
            outbox: Vec::new(),
            floating: Vec::new(),
        }
    }

    /// A connection's first message: a [`ToServer::Hello`] joins it as a
    /// player, with `slots` if it is new (its saved slots otherwise, which
    /// the caller supplies from the player's record). Returns the player's
    /// number, or `None` with a [`ToClient::Rejected`] queued.
    pub fn join(&mut self, connection: u64, hello: &ToServer, slots: Slots) -> Option<u32> {
        let ToServer::Hello {
            versions,
            key,
            name,
        } = hello
        else {
            self.reject(connection, JoinRefusal::GameVersion);
            return None;
        };
        if let Err(reason) = agree(&self.world.versions, versions) {
            self.reject(connection, reason);
            return None;
        }
        let number = match self.known.iter().find(|(k, _)| k == key) {
            Some(&(_, n)) => n,
            None => {
                let n = self.known.len() as u32;
                self.known.push((*key, n));
                n
            }
        };
        if !self.players.contains_key(&number) && self.players.len() >= MAX_PLAYERS {
            self.reject(connection, JoinRefusal::Full);
            return None;
        }
        self.players.insert(
            number,
            Player {
                name: name.clone(),
                slots,
            },
        );
        self.send(
            To::Player(number),
            ToClient::Welcome {
                player: number,
                clock_s: self.clock_s,
            },
        );
        Some(number)
    }

    /// A player left.
    pub fn leave(&mut self, player: u32) -> Option<Player> {
        self.players.remove(&player)
    }

    /// One message from a joined player.
    pub fn handle(&mut self, player: u32, message: ToServer) {
        if !self.players.contains_key(&player) {
            return;
        }
        match message {
            ToServer::Request { id, request } => self.request(player, id, request),
            ToServer::Resync { region } => {
                let snap = self.ledger.snapshot(region, &self.edits);
                self.send(
                    To::Player(player),
                    ToClient::Snapshot {
                        region,
                        seq: snap.seq,
                        bytes: snap.bytes,
                    },
                );
            }
            // Subscription, presence, transactions and chat come with their
            // tasks (6.3, 8.1, 8.5).
            _ => {}
        }
    }

    /// Advance the world by `dt_s`. The clock runs only while a player is
    /// on (design 11).
    pub fn tick(&mut self, dt_s: f64) {
        if self.players.is_empty() {
            return;
        }
        self.clock_s += dt_s;
        self.since_clock_s += dt_s;
        if self.since_clock_s >= CLOCK_EVERY_S {
            self.since_clock_s = 0.0;
            let clock_s = self.clock_s;
            self.broadcast(ToClient::Clock { clock_s });
        }
    }

    /// Everything queued to send, in order.
    pub fn drain(&mut self) -> Vec<(To, ToClient)> {
        std::mem::take(&mut self.outbox)
    }

    pub fn edits(&self) -> &Edits {
        &self.edits
    }

    pub fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    pub fn player(&self, player: u32) -> Option<&Player> {
        self.players.get(&player)
    }

    pub fn clock_s(&self) -> f64 {
        self.clock_s
    }

    pub fn journal(&self) -> &J {
        &self.journal
    }

    pub fn journal_mut(&mut self) -> &mut J {
        &mut self.journal
    }

    /// The column a cell has now: generated from the seed, its worms and its
    /// edits, the same pure function a client's tier builds.
    pub fn column(&self, cell: u32) -> Option<Column> {
        let direction: Vec3 = pbd_core::cell_key::key_direction(cell)?;
        let near = worms::gather(&self.world.worms, &self.world.terrain, direction, 0.0);
        Some(column::generate_edited(
            &near,
            &self.world.worms,
            &self.world.terrain,
            direction,
            self.edits.for_cell(cell),
        ))
    }

    fn request(&mut self, player: u32, id: u32, request: Request) {
        let (cell, layer, material, hands) = match request {
            Request::Dig { cell, layer } => (cell, layer, Material::Air, Hands::Take),
            Request::Place { cell, layer, slot } => {
                let held = self.players[&player].slots.get(usize::from(slot));
                match held.map(|stack| stack.item) {
                    Some(Item::Block(material)) => (cell, layer, material, Hands::Spend),
                    _ => return self.refuse(player, id, cell, Refusal::EmptySlot),
                }
            }
        };
        let Some(column) = self.column(cell) else {
            return self.refuse(player, id, cell, Refusal::OutOfWorld);
        };
        let layer_at = usize::from(layer);
        let mut slots = self.players[&player].slots.clone();
        if let Request::Place { slot, .. } = request {
            slots.select(usize::from(slot));
        }
        let ask = Ask {
            cell,
            layer: layer_at,
            material,
            was: if accept::in_world(layer_at) {
                column.material(layer_at)
            } else {
                Material::Air
            },
            lamp_in_column: (0..column::LAYERS).any(|l| column.material(l).is_lamp()),
            hands,
        };
        let (terrain, edits) = (&self.world.terrain, &self.edits);
        let taken = match accept::edit(ask, &slots, || {
            accept::yield_of(terrain, edits, ask.was, cell, layer_at)
        }) {
            Ok(taken) => taken,
            Err(refused) => {
                let reason = match refused {
                    Refused::OutOfWorld => Refusal::OutOfWorld,
                    Refused::Unchanged => Refusal::Unchanged,
                    Refused::LampTaken => Refusal::LampTaken,
                    Refused::EmptySlot => Refusal::EmptySlot,
                };
                return self.refuse(player, id, cell, reason);
            }
        };
        let edit = Edit {
            cell,
            layer,
            material,
        };
        // Numbered and journaled before anything changes or is answered: a
        // journal that refuses leaves the world as it was. The number is the
        // one the ledger gives the edit when it takes it just below.
        let region =
            pbd_core::cell_key::region::of_key(cell).unwrap_or(pbd_core::net::ledger::NO_REGION);
        let numbered = Sequenced {
            region,
            seq: self.ledger.seq(region) + 1,
            edit,
        };
        if !self
            .journal
            .edit(&Author::Player(player), &numbered, &taken.slots)
        {
            return self.refuse(player, id, cell, Refusal::NotSaved);
        }
        self.edits.set(edit);
        let sequenced = self.ledger.accept(edit, &self.edits);
        debug_assert_eq!(sequenced, numbered);
        if let Some(player) = self.players.get_mut(&player) {
            player.slots = taken.slots;
        }
        if let Some(item) = taken.floats {
            self.floating.push((cell, item));
        }
        self.send(
            To::Player(player),
            ToClient::Accepted {
                id,
                region: sequenced.region,
                seq: sequenced.seq,
            },
        );
        self.broadcast(ToClient::Change {
            region: sequenced.region,
            seq: sequenced.seq,
            change: Change::Edit {
                cell,
                layer,
                material,
            },
        });
    }

    /// Refuse a request, and re-send what the requester may have predicted
    /// wrongly (design 15.3): the cell's region whole. The slots go when the
    /// inventory has its message (task 8.2).
    fn refuse(&mut self, player: u32, id: u32, cell: u32, reason: Refusal) {
        self.send(To::Player(player), ToClient::Refused { id, reason });
        if let Some(region) = pbd_core::cell_key::region::of_key(cell) {
            let snap = self.ledger.snapshot(region, &self.edits);
            self.send(
                To::Player(player),
                ToClient::Snapshot {
                    region,
                    seq: snap.seq,
                    bytes: snap.bytes,
                },
            );
        }
    }

    fn reject(&mut self, connection: u64, reason: JoinRefusal) {
        self.send(To::Connection(connection), ToClient::Rejected { reason });
    }

    fn send(&mut self, to: To, message: ToClient) {
        self.outbox.push((to, message));
    }

    fn broadcast(&mut self, message: ToClient) {
        let players: Vec<u32> = self.players.keys().copied().collect();
        for player in players {
            self.send(To::Player(player), message.clone());
        }
    }
}

#[cfg(test)]
mod tests;
