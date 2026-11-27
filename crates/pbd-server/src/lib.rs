//! The server: the one authority over a shared world (`multiplayer` design
//! 4 and 5, task 4.1).
//!
//! It holds every player's slots and decides every change; the world's
//! edits, their order and the journal are its [`Store`]'s (design 18): a
//! dedicated server's own ([`MemoryStore`], then its save), single player's
//! the save it already has. A client sends what its player tried
//! ([`ToServer::Request`]); the server decides it with the same core rule a
//! client predicts with (`pbd_core::accept`), has the store number it in its
//! region (`net::ledger`) and make it durable, and only then
//! answers [`ToClient::Accepted`] and sends the change to every player, the
//! requester included, in that order. A refusal carries no undo: the
//! requester is sent the cell's layers and its own slots again (design 15.3).
//!
//! A world with crew missions (`coop-missions` design 1 and 2) has them
//! counted here: every accepted edit is counted for its author into the
//! world's one crew record, which the store journals after the edit
//! ([`Server::set_crew`]).
//!
//! It is plain data and a [`Server::handle`] and [`Server::tick`] pair, so a
//! test drives it with no thread or socket, single player runs it in process
//! and a dedicated server runs it on its own thread at [`TICK_S`].

use glam::Vec3;
use pbd_core::accept::{self, Ask, Hands, Refused};
use pbd_core::column::{self, Column};
use pbd_core::drops::ItemDrop;
use pbd_core::edits::{Edit, Edits};
use pbd_core::inventory::{Item, Slots};
use pbd_core::missions::crew::{CrewChain, CrewProgress, Hand};
use pbd_core::missions::{Happened, Kind, item_code};
use pbd_core::net::ledger::{Ledger, Sequenced};
use pbd_core::net::protocol::{
    Change, JoinRefusal, Refusal, Request, ToClient, ToServer, Versions, agree,
};
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::{Author, Record, Records};
use pbd_core::terrain::Material;
use pbd_core::worms::{self, WormField};
use std::collections::BTreeMap;

/// The server's tick, s: 20 a second (design 5).
pub const TICK_S: f64 = 1.0 / 20.0;
/// How often the clock is sent, s (design 15.8).
pub const CLOCK_EVERY_S: f64 = 5.0;
/// Players at most (design 14.2).
pub const MAX_PLAYERS: usize = 8;

/// The world's edits, their order, and where an accepted change is made
/// durable. The server answers a change as accepted only after
/// [`Store::commit`] has taken it (design 4, "the durability rule does not
/// move").
pub trait Store {
    fn edits(&self) -> &Edits;
    fn ledger(&self) -> &Ledger;
    /// Take one accepted edit by `author`, with the author's slots after it:
    /// set it in the edits, number it in its region and journal it. `None`
    /// refuses it, changing nothing, and the server refuses the request.
    fn commit(&mut self, author: &Author, edit: Edit, carried: &Slots) -> Option<Sequenced>;
    /// The drop the next [`Store::commit`] writes on its line, for `item`
    /// floating out of `cell`'s `layer` (design 23): the app's own, made
    /// before it asked, for the host's player; one the store makes now,
    /// named by its next id, for a remote player. `None` writes no drop.
    fn floating(
        &mut self,
        author: &Author,
        item: Item,
        cell: u32,
        layer: u16,
        made_s: f64,
    ) -> Option<ItemDrop>;
    /// Journal a player's slots after a move in the pack. `false` refuses
    /// it (design 19).
    fn commit_pack(&mut self, author: &Author, carried: &Slots) -> bool;
    /// A drop the world holds now, by id.
    fn drop(&self, id: u64) -> Option<ItemDrop>;
    /// Journal a pickup: `left` remain on drop `id` (none: it is gone), the
    /// rest went into `carried`. `false` refuses it.
    fn commit_pick(&mut self, author: &Author, id: u64, left: u16, carried: &Slots) -> bool;
    /// Journal a toss (design 25): a drop of `stack` at `at` (the body's
    /// frame), made at `made_s`, named by the store's next id, and the
    /// author's slots after it. `None` refuses it.
    fn commit_toss(
        &mut self,
        author: &Author,
        stack: pbd_core::inventory::Stack,
        at: glam::DVec3,
        made_s: f64,
        carried: &Slots,
    ) -> Option<ItemDrop>;
    /// Journal a world record `author`'s deed changed: the crew's progress
    /// (`coop-missions` design 1). `false` refuses it.
    fn commit_record(&mut self, author: &Author, record: Record) -> bool;
    /// A world record as the store holds it now (design 26).
    fn record(&self, kind: &str, id: u64) -> Option<Record>;
    /// Journal a record a player's deed changed with their slots after it
    /// (design 26). `false` refuses it.
    fn commit_record_with_hands(
        &mut self,
        author: &Author,
        record: Record,
        carried: &Slots,
    ) -> bool;
}

/// A pack or pickup a [`MemoryStore`] journaled: the author, the drop and
/// what was left on it for a pickup, and the slots after.
pub type HandsEntry = (Author, Option<(u64, u16)>, Slots);

/// A store in memory: tests, and a world with no disk behind it.
#[derive(Clone, Debug, Default)]
pub struct MemoryStore {
    pub edits: Edits,
    pub ledger: Ledger,
    /// Every commit, in order.
    pub entries: Vec<(Author, Sequenced, Slots)>,
    /// The live drops, by id.
    pub drops: BTreeMap<u64, ItemDrop>,
    /// Every pack and pickup journaled, in order: the author, the drop and
    /// what was left on it for a pickup, and the slots after.
    pub hands: Vec<HandsEntry>,
    /// Every record journaled, by kind and id: its last value.
    pub records: Records,
    /// Refuse everything from here on, as a failed writer does.
    pub failed: bool,
    /// The drop the next commit writes (design 23).
    pub floating: Option<ItemDrop>,
}

impl MemoryStore {
    pub fn new(edits: Edits) -> Self {
        Self {
            ledger: Ledger::of(&edits),
            edits,
            ..Self::default()
        }
    }
}

impl Store for MemoryStore {
    fn edits(&self) -> &Edits {
        &self.edits
    }
    fn ledger(&self) -> &Ledger {
        &self.ledger
    }
    fn commit(&mut self, author: &Author, edit: Edit, carried: &Slots) -> Option<Sequenced> {
        if self.failed {
            return None;
        }
        self.edits.set(edit);
        let sequenced = self.ledger.accept(edit, &self.edits);
        self.entries
            .push((author.clone(), sequenced, carried.clone()));
        if let Some(drop) = self.floating.take() {
            self.drops.insert(drop.id, drop);
        }
        Some(sequenced)
    }
    fn floating(
        &mut self,
        _author: &Author,
        item: Item,
        cell: u32,
        layer: u16,
        made_s: f64,
    ) -> Option<ItemDrop> {
        // In memory there is no planet: the cell's direction, as far out as
        // the layer's number, is enough to tell drops apart.
        let direction = pbd_core::cell_key::key_direction(cell)?.as_dvec3();
        let id = self.drops.keys().next_back().map_or(0, |id| id + 1);
        let drop = ItemDrop {
            id,
            item,
            count: 1,
            position: direction * f64::from(layer),
            made_s,
        };
        self.floating = Some(drop);
        Some(drop)
    }
    fn commit_pack(&mut self, author: &Author, carried: &Slots) -> bool {
        if self.failed {
            return false;
        }
        self.hands.push((author.clone(), None, carried.clone()));
        true
    }
    fn drop(&self, id: u64) -> Option<ItemDrop> {
        self.drops.get(&id).copied()
    }
    fn commit_pick(&mut self, author: &Author, id: u64, left: u16, carried: &Slots) -> bool {
        if self.failed {
            return false;
        }
        if left == 0 {
            self.drops.remove(&id);
        } else if let Some(drop) = self.drops.get_mut(&id) {
            drop.count = left;
        }
        self.hands
            .push((author.clone(), Some((id, left)), carried.clone()));
        true
    }
    fn commit_record(&mut self, _author: &Author, record: Record) -> bool {
        if self.failed {
            return false;
        }
        self.records.put(record);
        true
    }
    fn record(&self, kind: &str, id: u64) -> Option<Record> {
        self.records.get(kind, id).cloned()
    }
    fn commit_record_with_hands(
        &mut self,
        author: &Author,
        record: Record,
        carried: &Slots,
    ) -> bool {
        if self.failed {
            return false;
        }
        self.records.put(record);
        self.hands.push((author.clone(), None, carried.clone()));
        true
    }
    fn commit_toss(
        &mut self,
        author: &Author,
        stack: pbd_core::inventory::Stack,
        at: glam::DVec3,
        made_s: f64,
        carried: &Slots,
    ) -> Option<ItemDrop> {
        if self.failed {
            return None;
        }
        let id = self.drops.keys().next_back().map_or(0, |id| id + 1);
        let drop = ItemDrop {
            id,
            item: stack.item,
            count: stack.count,
            position: pbd_core::drops::scattered(at, id),
            made_s,
        };
        self.drops.insert(id, drop);
        self.hands.push((author.clone(), None, carried.clone()));
        Some(drop)
    }
}

/// An accepted edit, as the server decided it.
#[derive(Clone, Debug, PartialEq)]
pub struct Decided {
    pub sequenced: Sequenced,
    /// The player's slots after it.
    pub slots: Slots,
    /// What floats out at the cell: a drop (a region record, task 8.2).
    pub floats: Option<Item>,
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
    /// The world time they joined at, s: a crew `each` waits for them
    /// until they have been in for its grace time.
    pub joined_s: f64,
    /// Their survival bars in a Survival world, `None` in Explorer
    /// ([`survival`], `survival-mode` task 4.1).
    pub vitals: Option<pbd_core::survival::Vitals>,
    /// Seconds of play since the vitals were last journaled.
    pub since_vitals_s: f32,
    /// Where they were last seen (design 24).
    pub pose: Option<pbd_core::net::protocol::Pose>,
}

/// A world's crew missions as the server counts them: the chain and the
/// world's one record of its progress.
#[derive(Clone, Debug)]
pub struct Crew {
    pub chain: CrewChain,
    pub progress: CrewProgress,
}

pub struct Server {
    world: World,
    /// By number; iterated in number order, so a broadcast's order is fixed.
    players: BTreeMap<u32, Player>,
    /// Players' install keys to their numbers, kept across leaving.
    known: Vec<([u8; 16], u32)>,
    /// The slots a player left with, by number, given back on a rejoin
    /// while this server runs (design 22; the save keeps them with design
    /// 10's player record).
    away: BTreeMap<u32, Slots>,
    clock_s: f64,
    since_clock_s: f64,
    outbox: Vec<(To, ToClient)>,
    /// The crew missions counted, when the world has them.
    crew: Option<Crew>,
    /// Farming's tuning, for harvests (design 26).
    farm: pbd_core::farming::FarmSettings,
}

impl Server {
    /// A server over `world`, at world time `clock_s`.
    pub fn new(world: World, clock_s: f64) -> Self {
        Self {
            world,
            players: BTreeMap::new(),
            known: Vec::new(),
            away: BTreeMap::new(),
            clock_s,
            since_clock_s: 0.0,
            outbox: Vec::new(),
            crew: None,
            farm: pbd_core::farming::FarmSettings::default(),
        }
    }

    /// Farming's tuning, as the world's config has it (design 26).
    pub fn set_farm(&mut self, farm: pbd_core::farming::FarmSettings) {
        self.farm = farm;
    }

    /// Count the crew chain `chain` from here on, from `progress` as the
    /// world's record holds it (`CrewProgress::from_records`).
    pub fn set_crew(&mut self, chain: CrewChain, progress: CrewProgress) {
        self.crew = Some(Crew { chain, progress });
    }

    pub fn crew(&self) -> Option<&Crew> {
        self.crew.as_ref()
    }

    /// The players in the world now, and when each joined.
    pub fn present(&self) -> Vec<Hand> {
        self.players
            .iter()
            .map(|(&player, p)| Hand {
                player,
                since_s: p.joined_s,
            })
            .collect()
    }

    /// Count `happened`, done by `player`, into the crew's record, and have
    /// the store journal the record when it moved. A record the store
    /// refuses leaves the crew's progress as it was.
    fn count_crew(&mut self, store: &mut impl Store, player: u32, happened: &Happened) {
        let present = self.present();
        let now_s = self.clock_s;
        let Some(crew) = self.crew.as_mut() else {
            return;
        };
        let before = crew.progress.clone();
        let step = crew
            .progress
            .count(&crew.chain, player, happened, &present, now_s);
        if step.changed && !store.commit_record(&Author::Player(player), crew.progress.record()) {
            crew.progress = before;
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
        // A player already known keeps what they carry: still in (a game
        // restarted before its old connection closed), or away (design 22).
        let slots = match self.players.get(&number) {
            Some(player) => player.slots.clone(),
            None => self.away.remove(&number).unwrap_or(slots),
        };
        self.players.insert(
            number,
            Player {
                name: name.clone(),
                slots,
                joined_s: self.clock_s,
                vitals: None,
                since_vitals_s: 0.0,
                pose: None,
            },
        );
        self.send(
            To::Player(number),
            ToClient::Welcome {
                player: number,
                clock_s: self.clock_s,
                seed: self.world.versions.world,
                generator: self.world.versions.generator,
                topology: self.world.versions.topology,
            },
        );
        self.send_slots(number);
        Some(number)
    }

    /// Every region with an edit, sent whole to `player`, nearest to
    /// `near` first when it is given: what a joining player is sent until
    /// region subscription (task 6.3) sends only the regions near them
    /// (design 20).
    pub fn snapshot_all(&mut self, store: &impl Store, player: u32, near: Option<Vec3>) {
        let mut regions = store.ledger().regions().regions();
        if let Some(near) = near {
            let distance = |r: &u32| -> f32 {
                pbd_core::cell_key::region::address(*r, 0)
                    .or_else(|| pbd_core::cell_key::region::address(*r, 63 << 6))
                    .map(|a| pbd_core::cell_key::direction(a).distance(near))
                    .unwrap_or(f32::MAX)
            };
            regions.sort_by(|a, b| distance(a).total_cmp(&distance(b)));
        }
        for region in regions {
            let snap = store.ledger().snapshot(region, store.edits());
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

    /// A player left.
    pub fn leave(&mut self, player: u32) -> Option<Player> {
        let left = self.players.remove(&player)?;
        self.away.insert(player, left.slots.clone());
        // Everyone still in stops drawing them (design 24).
        self.broadcast(ToClient::Gone { player });
        Some(left)
    }

    /// Where `player` is now (design 24): kept, and passed to every other
    /// player, never back to them. The host's own player's comes this way
    /// too, so the host is seen as any player is.
    pub fn presence(&mut self, player: u32, pose: pbd_core::net::protocol::Pose) {
        let Some(p) = self.players.get_mut(&player) else {
            return;
        };
        p.pose = Some(pose);
        let others: Vec<u32> = self
            .players
            .keys()
            .copied()
            .filter(|&n| n != player)
            .collect();
        for to in others {
            self.send(To::Player(to), ToClient::Presence { player, pose });
        }
    }

    /// One message from a joined player.
    pub fn handle(&mut self, store: &mut impl Store, player: u32, message: ToServer) {
        if !self.players.contains_key(&player) {
            return;
        }
        match message {
            ToServer::Request { id, request } => {
                // What floats out becomes a drop record with task 8.2.
                let _ = self.request(store, player, id, request);
                // Taken or refused, the player's slots as the server holds
                // them: what a dig gave, or a prediction put back (design
                // 22, 15.3).
                self.send_slots(player);
            }
            ToServer::Resync { region } => {
                let snap = store.ledger().snapshot(region, store.edits());
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
            ToServer::Presence { pose } => self.presence(player, pose),
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

    /// Whether anything is queued to send.
    pub fn has_outbox(&self) -> bool {
        !self.outbox.is_empty()
    }

    /// Everything queued to send, in order.
    pub fn drain(&mut self) -> Vec<(To, ToClient)> {
        std::mem::take(&mut self.outbox)
    }

    pub fn player(&self, player: u32) -> Option<&Player> {
        self.players.get(&player)
    }

    pub fn clock_s(&self) -> f64 {
        self.clock_s
    }

    /// The column a cell has now: generated from the seed, its worms and its
    /// edits, the same pure function a client's tier builds.
    pub fn column(&self, edits: &Edits, cell: u32) -> Option<Column> {
        let direction: Vec3 = pbd_core::cell_key::key_direction(cell)?;
        let near = worms::gather(&self.world.worms, &self.world.terrain, direction, 0.0);
        Some(column::generate_edited(
            &near,
            &self.world.worms,
            &self.world.terrain,
            direction,
            edits.for_cell(cell),
        ))
    }

    fn request(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        request: Request,
    ) -> Result<Decided, Refusal> {
        let (cell, layer, material, hands, slot) = match request {
            Request::Dig { cell, layer } => (cell, layer, Material::Air, Hands::Take, None),
            Request::Place { cell, layer, slot } => {
                let held = self.players[&player].slots.get(usize::from(slot));
                match held.map(|stack| stack.item) {
                    Some(Item::Block(material)) => {
                        (cell, layer, material, Hands::Spend, Some(slot))
                    }
                    _ => return Err(self.refuse(store, player, id, cell, Refusal::EmptySlot)),
                }
            }
            Request::Pack(mv) => {
                let slots = self.players[&player].slots.clone();
                return self
                    .pack(store, player, id, mv, slots)
                    .map(|slots| Decided {
                        sequenced: Sequenced {
                            region: pbd_core::net::ledger::NO_REGION,
                            seq: 0,
                            edit: Edit {
                                cell: 0,
                                layer: 0,
                                material: Material::Air,
                            },
                        },
                        slots,
                        floats: None,
                    });
            }
            Request::Plant { cell, layer, slot } => {
                let slots = self.players[&player].slots.clone();
                let made_s = self.clock_s;
                return self
                    .plant(
                        store,
                        player,
                        id,
                        (cell, layer, usize::from(slot)),
                        made_s,
                        slots,
                    )
                    .map(no_edit);
            }
            Request::Harvest { cell } => {
                let slots = self.players[&player].slots.clone();
                let made_s = self.clock_s;
                return self
                    .harvest(store, player, id, cell, made_s, slots)
                    .map(no_edit);
            }
            Request::Toss { slot, count, at_mm } => {
                let slots = self.players[&player].slots.clone();
                let at =
                    glam::DVec3::new(at_mm[0] as f64, at_mm[1] as f64, at_mm[2] as f64) / 1000.0;
                let made_s = self.clock_s;
                return self
                    .toss(
                        store,
                        player,
                        id,
                        (usize::from(slot), count),
                        at,
                        made_s,
                        slots,
                    )
                    .map(|(slots, _)| Decided {
                        sequenced: Sequenced {
                            region: pbd_core::net::ledger::NO_REGION,
                            seq: 0,
                            edit: Edit {
                                cell: 0,
                                layer: 0,
                                material: Material::Air,
                            },
                        },
                        slots,
                        floats: None,
                    });
            }
            Request::Pick { drop } => {
                let slots = self.players[&player].slots.clone();
                return self
                    .pick(store, player, id, drop, slots)
                    .map(|(slots, _)| Decided {
                        sequenced: Sequenced {
                            region: pbd_core::net::ledger::NO_REGION,
                            seq: 0,
                            edit: Edit {
                                cell: 0,
                                layer: 0,
                                material: Material::Air,
                            },
                        },
                        slots,
                        floats: None,
                    });
            }
        };
        let mut slots = self.players[&player].slots.clone();
        if let Some(slot) = slot {
            slots.select(usize::from(slot));
        }
        self.edit(
            store,
            player,
            id,
            Edit {
                cell,
                layer,
                material,
            },
            hands,
            slots,
        )
    }

    /// Decide one edit by `player` with the hands as asked, from `slots`:
    /// the path every edit takes, a remote request's and single player's
    /// alike. In process (design 18) the player's slots are the app's as
    /// they stand, because the game's other writers still change them
    /// outside the server until they move (task 2.2), and the hands are the
    /// ones the app settled (a dig's finds, a till's seed), which only a
    /// remote client is held to the server's own roll for (task 8.7).
    pub fn edit(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        edit: Edit,
        hands: Hands,
        slots: Slots,
    ) -> Result<Decided, Refusal> {
        let Edit {
            cell,
            layer,
            material,
        } = edit;
        if !self.players.contains_key(&player) {
            return Err(Refusal::Cheat);
        }
        let Some(column) = self.column(store.edits(), cell) else {
            return Err(self.refuse(store, player, id, cell, Refusal::OutOfWorld));
        };
        let layer_at = usize::from(layer);
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
        let terrain = &self.world.terrain;
        let taken = match accept::edit(ask, &slots, || {
            accept::yield_of(terrain, store.edits(), ask.was, cell, layer_at)
        }) {
            Ok(taken) => taken,
            Err(refused) => {
                let reason = match refused {
                    Refused::OutOfWorld => Refusal::OutOfWorld,
                    Refused::Unchanged => Refusal::Unchanged,
                    Refused::LampTaken => Refusal::LampTaken,
                    Refused::EmptySlot => Refusal::EmptySlot,
                };
                return Err(self.refuse(store, player, id, cell, reason));
            }
        };
        // What floats out is a drop, written on the dig's line (design 23).
        let made = taken.floats.and_then(|item| {
            store.floating(&Author::Player(player), item, cell, layer, self.clock_s)
        });
        // Made durable and numbered before anything is answered: a store
        // that refuses leaves the world as it was.
        let Some(sequenced) = store.commit(&Author::Player(player), edit, &taken.slots) else {
            return Err(self.refuse(store, player, id, cell, Refusal::NotSaved));
        };
        if let Some(player) = self.players.get_mut(&player) {
            player.slots = taken.slots.clone();
        }
        // The deed, as the missions count it: a block dug or placed.
        let happened = if material == Material::Air {
            Happened::new(Kind::Gather, item_code(Item::Block(ask.was)), 1)
        } else {
            Happened::new(Kind::Place, item_code(Item::Block(material)), 1)
        };
        self.count_crew(store, player, &happened);
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
        if let Some(drop) = made {
            self.broadcast(ToClient::Drop { drop });
        }
        Ok(Decided {
            sequenced,
            slots: taken.slots,
            floats: taken.floats,
        })
    }

    /// Decide a move in `player`'s pack, from `slots` (design 19): the core
    /// rule, then the store journals the slots after it. In process the
    /// slots are the app's as they stand (design 18).
    pub fn pack(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        mv: pbd_core::accept::PackMove,
        slots: Slots,
    ) -> Result<Slots, Refusal> {
        if !self.players.contains_key(&player) {
            return Err(Refusal::Cheat);
        }
        let Ok(moved) = accept::pack(&slots, mv) else {
            return Err(self.refuse_hands(player, id, Refusal::EmptySlot));
        };
        if !store.commit_pack(&Author::Player(player), &moved) {
            return Err(self.refuse_hands(player, id, Refusal::NotSaved));
        }
        self.players.get_mut(&player).expect("checked").slots = moved.clone();
        self.send(
            To::Player(player),
            ToClient::Accepted {
                id,
                region: 0,
                seq: 0,
            },
        );
        Ok(moved)
    }

    /// Decide a pickup of drop `drop_id` by `player`, from `slots` (design
    /// 19): the drop must be in the store; it gives by the core's give order
    /// and the store journals what is left on it. Returns the slots after
    /// and what is left. A drop that is gone is refused, so two players
    /// reaching one drop are given it once.
    pub fn pick(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        drop_id: u64,
        slots: Slots,
    ) -> Result<(Slots, u16), Refusal> {
        if !self.players.contains_key(&player) {
            return Err(Refusal::Cheat);
        }
        let Some(drop) = store.drop(drop_id).filter(|d| d.count > 0) else {
            return Err(self.refuse_hands(player, id, Refusal::Gone));
        };
        let (taken, left) = accept::pick(&drop, &slots);
        if !store.commit_pick(&Author::Player(player), drop_id, left, &taken) {
            return Err(self.refuse_hands(player, id, Refusal::NotSaved));
        }
        self.players.get_mut(&player).expect("checked").slots = taken.clone();
        self.send(
            To::Player(player),
            ToClient::Accepted {
                id,
                region: 0,
                seq: 0,
            },
        );
        // Everyone's copy of the drop follows (design 23).
        self.broadcast(ToClient::DropLeft { id: drop_id, left });
        Ok((taken, left))
    }

    /// Decide a toss by `player` from `slots` (design 25): `count` of what
    /// slot `what.0` holds, to land at `at` (the body's frame), made at
    /// `made_s`. The store writes the toss line; everyone is told of the drop.
    #[allow(clippy::too_many_arguments)]
    pub fn toss(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        what: (usize, u16),
        at: glam::DVec3,
        made_s: f64,
        slots: Slots,
    ) -> Result<(Slots, ItemDrop), Refusal> {
        if !self.players.contains_key(&player) || !at.is_finite() {
            return Err(Refusal::Cheat);
        }
        let Ok((after, stack)) = accept::toss(&slots, what.0, what.1) else {
            return Err(self.refuse_hands(player, id, Refusal::EmptySlot));
        };
        let Some(drop) = store.commit_toss(&Author::Player(player), stack, at, made_s, &after)
        else {
            return Err(self.refuse_hands(player, id, Refusal::NotSaved));
        };
        self.players.get_mut(&player).expect("checked").slots = after.clone();
        self.send(
            To::Player(player),
            ToClient::Accepted {
                id,
                region: 0,
                seq: 0,
            },
        );
        self.broadcast(ToClient::Drop { drop });
        Ok((after, drop))
    }

    /// Decide a sowing by `player` from `slots` (design 26): the seed in
    /// slot `at.2` on the tilled soil at cell `at.0`'s layer `at.1`, sown at
    /// `now_s`. The plot record is written with the slots and sent to all.
    pub fn plant(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        at: (u32, u16, usize),
        now_s: f64,
        slots: Slots,
    ) -> Result<Slots, Refusal> {
        let (cell, layer, slot) = at;
        if !self.players.contains_key(&player) {
            return Err(Refusal::Cheat);
        }
        let Some(column) = self.column(store.edits(), cell) else {
            return Err(self.refuse_hands(player, id, Refusal::OutOfWorld));
        };
        let l = usize::from(layer);
        let (ground, over) = (column.material(l), column.material(l + 1));
        let growing = plot_of(store, cell).is_some_and(|p| {
            let pl = usize::from(p.layer);
            p.crop().is_some()
                && column.material(pl) == Material::Tilled
                && column.material(pl + 1) == Material::Air
        });
        let climate = pbd_core::cell_key::key_direction(cell).and_then(|d| {
            let height = column::layer_altitude(l) + 1.0;
            pbd_core::farming::Climate::of(pbd_core::planet_gen::biome_at(
                &self.world.terrain,
                d,
                height,
            ))
        });
        let (plot, after, _) = match pbd_core::farming::accept::plant(
            ground, over, growing, climate, &slots, slot, layer, now_s,
        ) {
            Ok(done) => done,
            Err(refused) => return Err(self.refuse_hands(player, id, refused.into())),
        };
        self.commit_plot(store, player, id, cell, &plot, after)
    }

    /// Decide a harvest of `cell` by `player` from `slots` (design 26).
    pub fn harvest(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        cell: u32,
        now_s: f64,
        slots: Slots,
    ) -> Result<Slots, Refusal> {
        if !self.players.contains_key(&player) {
            return Err(Refusal::Cheat);
        }
        let Some(plot) = plot_of(store, cell) else {
            return Err(self.refuse_hands(player, id, Refusal::NoCrop));
        };
        let (bare, after, ..) =
            match pbd_core::farming::accept::harvest(&plot, &self.farm, &slots, now_s) {
                Ok(done) => done,
                Err(refused) => return Err(self.refuse_hands(player, id, refused.into())),
            };
        self.commit_plot(store, player, id, cell, &bare, after)
    }

    /// Write a plot with the slots after, answer, and send the record to all.
    fn commit_plot(
        &mut self,
        store: &mut impl Store,
        player: u32,
        id: u32,
        cell: u32,
        plot: &pbd_core::farming::Plot,
        after: Slots,
    ) -> Result<Slots, Refusal> {
        use pbd_core::farming::{PLOT_KIND, PLOT_SCHEMA};
        let record = Record::of(PLOT_KIND, u64::from(cell), PLOT_SCHEMA, plot);
        if !store.commit_record_with_hands(&Author::Player(player), record.clone(), &after) {
            return Err(self.refuse_hands(player, id, Refusal::NotSaved));
        }
        self.players.get_mut(&player).expect("checked").slots = after.clone();
        self.send(
            To::Player(player),
            ToClient::Accepted {
                id,
                region: 0,
                seq: 0,
            },
        );
        self.broadcast(ToClient::Record { record });
        Ok(after)
    }

    /// Refuse a request about the hands. The slots go back with the
    /// inventory's message (task 8.2); until then the client keeps its own.
    fn refuse_hands(&mut self, player: u32, id: u32, reason: Refusal) -> Refusal {
        self.send(To::Player(player), ToClient::Refused { id, reason });
        reason
    }

    /// Refuse a request, and re-send what the requester may have predicted
    /// wrongly (design 15.3): the cell's region whole. The slots go when the
    /// inventory has its message (task 8.2).
    fn refuse(
        &mut self,
        store: &impl Store,
        player: u32,
        id: u32,
        cell: u32,
        reason: Refusal,
    ) -> Refusal {
        self.send(To::Player(player), ToClient::Refused { id, reason });
        if let Some(region) = pbd_core::cell_key::region::of_key(cell) {
            let snap = store.ledger().snapshot(region, store.edits());
            self.send(
                To::Player(player),
                ToClient::Snapshot {
                    region,
                    seq: snap.seq,
                    bytes: snap.bytes,
                },
            );
        }
        reason
    }

    /// Send a remote player their own slots (design 22). The host's own
    /// player, 0, holds the app's slots already (design 18).
    fn send_slots(&mut self, player: u32) {
        if player == 0 {
            return;
        }
        if let Some(slots) = self.players.get(&player).map(|p| p.slots.clone()) {
            self.send(To::Player(player), ToClient::Slots { slots });
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

/// A cell's plot record, read (design 26).
fn plot_of(store: &impl Store, cell: u32) -> Option<pbd_core::farming::Plot> {
    store
        .record(pbd_core::farming::PLOT_KIND, u64::from(cell))?
        .read()
}

/// A decision that changed the hands only, as a [`Decided`].
fn no_edit(slots: Slots) -> Decided {
    Decided {
        sequenced: Sequenced {
            region: pbd_core::net::ledger::NO_REGION,
            seq: 0,
            edit: Edit {
                cell: 0,
                layer: 0,
                material: Material::Air,
            },
        },
        slots,
        floats: None,
    }
}

pub mod survival;

#[cfg(test)]
mod tests;
