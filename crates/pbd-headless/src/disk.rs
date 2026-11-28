//! A dedicated server's world on disk (`multiplayer` design 27): the same
//! folder a game's save is (`identity.ron`, `world.ron`, `edits.v1.log`),
//! written through `pbd-save`'s format and writer, so a hosted world can be
//! copied off the server and opened in the game.
//!
//! The server's state is kept in a [`MemoryStore`]; every change it takes is
//! also appended to the journal. A dedicated server has no player 0 of its
//! own, so its lines carry empty slots: the players' slots live in the
//! server until their player records (design 10).
//!
//! locale: dev

use glam::DVec3;
use pbd_core::drops::ItemDrop;
use pbd_core::edits::{Edit, Edits};
use pbd_core::inventory::{Item, Slots, Stack};
use pbd_core::net::ledger::{Ledger, Sequenced};
use pbd_core::records::{Author, Record};
use pbd_save::format::{self, Identity, Record as Line, WorldFile};
use pbd_save::writer::SaveWriter;
use pbd_server::{MemoryStore, Store};
use std::path::{Path, PathBuf};

/// The identity file, as a game's save names it.
pub const IDENTITY: &str = "identity.ron";
/// The world file, as a game's save names it.
pub const WORLD: &str = "world.ron";

/// A world on disk, served.
pub struct DiskStore {
    mem: MemoryStore,
    writer: SaveWriter,
    /// The next drop's id: never reused, as a save's are not.
    next_drop: u64,
    pub identity: Identity,
    pub dir: PathBuf,
}

impl DiskStore {
    /// Open the world in `dir`, or make one there with `seed` if the folder
    /// holds none. Its journal is replayed: edits, live drops and records.
    pub fn open(dir: &Path, seed: u64) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let identity = match std::fs::read_to_string(dir.join(IDENTITY))
            .ok()
            .and_then(|t| ron::from_str::<Identity>(&t).ok())
        {
            Some(identity) => identity,
            None => {
                let identity = Identity::new(seed);
                let text = ron::ser::to_string_pretty(&identity, ron::ser::PrettyConfig::default())
                    .map_err(std::io::Error::other)?;
                std::fs::write(dir.join(IDENTITY), text)?;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                std::fs::write(
                    dir.join(WORLD),
                    WorldFile::new("Server".into(), seed, now).to_ron(),
                )?;
                identity
            }
        };
        let text = std::fs::read_to_string(dir.join(pbd_save::LOG)).unwrap_or_default();
        let mut edits = Edits::new();
        let mut mem = MemoryStore::default();
        let mut next_drop = 0;
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let Some((_, line)) = format::parse_entry(line) else {
                continue;
            };
            match line {
                Line::Edit { edit, drop, .. } => {
                    edits.set(edit);
                    if let Some(drop) = drop {
                        next_drop = next_drop.max(drop.id + 1);
                        mem.drops.insert(drop.id, drop);
                    }
                }
                Line::Toss { drop, .. } => {
                    next_drop = next_drop.max(drop.id + 1);
                    mem.drops.insert(drop.id, drop);
                }
                Line::Pick { id, left, .. } => {
                    if left == 0 {
                        mem.drops.remove(&id);
                    } else if let Some(d) = mem.drops.get_mut(&id) {
                        d.count = left;
                    }
                }
                Line::Stored(record) | Line::Handed { record, .. } => {
                    mem.records.put(record);
                }
                _ => {}
            }
        }
        mem.ledger = Ledger::of(&edits);
        mem.edits = edits;
        Ok(Self {
            mem,
            writer: SaveWriter::new(dir),
            next_drop,
            identity,
            dir: dir.to_path_buf(),
        })
    }

    /// Wait for every line to reach the disk.
    pub fn drain(&self) {
        self.writer.drain();
    }

    /// Whether the writer has failed: nothing more is taken.
    fn failed(&self) -> bool {
        self.writer.failure().is_some()
    }

    fn take_id(&mut self) -> u64 {
        let id = self.next_drop;
        self.next_drop += 1;
        id
    }
}

/// A dedicated server's lines carry no player's slots.
fn none() -> Slots {
    Slots::new()
}

impl Store for DiskStore {
    fn edits(&self) -> &Edits {
        &self.mem.edits
    }
    fn ledger(&self) -> &Ledger {
        &self.mem.ledger
    }
    fn commit(&mut self, author: &Author, edit: Edit, carried: &Slots) -> Option<Sequenced> {
        if self.failed() {
            return None;
        }
        let drop = self.mem.floating;
        let sequenced = self.mem.commit(author, edit, carried)?;
        self.writer.append(format::authored(
            author,
            format::line_of(edit, drop.as_ref(), &none()),
        ));
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
        let direction = pbd_core::cell_key::key_direction(cell)?
            .as_dvec3()
            .normalize();
        let radius = f64::from(
            pbd_core::planet_gen::TerrainConfig::for_version(self.identity.generator)
                .map_or(4800.0, |c| c.radius_m),
        );
        let centre = direction
            * (radius + f64::from(pbd_core::column::layer_altitude(usize::from(layer))) + 0.5);
        let id = self.take_id();
        let drop = ItemDrop {
            id,
            item,
            count: 1,
            position: pbd_core::drops::scattered(centre, id),
            made_s,
        };
        self.mem.floating = Some(drop);
        Some(drop)
    }
    fn commit_pack(&mut self, author: &Author, carried: &Slots) -> bool {
        // Only a player's own slots: kept by the server, not the journal.
        !self.failed() && self.mem.commit_pack(author, carried)
    }
    fn drop(&self, id: u64) -> Option<ItemDrop> {
        self.mem.drop(id)
    }
    fn commit_pick(&mut self, author: &Author, id: u64, left: u16, carried: &Slots) -> bool {
        if self.failed() || !self.mem.commit_pick(author, id, left, carried) {
            return false;
        }
        self.writer.append(format::authored(
            author,
            format::pick_line_of(id, left, &none()),
        ));
        true
    }
    fn commit_record(&mut self, author: &Author, record: Record) -> bool {
        if self.failed() {
            return false;
        }
        self.writer.append(record.line(author));
        self.mem.commit_record(author, record)
    }
    fn record(&self, kind: &str, id: u64) -> Option<Record> {
        self.mem.record(kind, id)
    }
    fn commit_record_with_hands(
        &mut self,
        author: &Author,
        record: Record,
        carried: &Slots,
    ) -> bool {
        if self.failed() {
            return false;
        }
        self.writer.append(format::authored(
            author,
            format::with_line_of(&record, &none()),
        ));
        self.mem.commit_record_with_hands(author, record, carried)
    }
    fn commit_toss(
        &mut self,
        author: &Author,
        stack: Stack,
        at: DVec3,
        made_s: f64,
        carried: &Slots,
    ) -> Option<ItemDrop> {
        if self.failed() {
            return None;
        }
        let id = self.take_id();
        let drop = ItemDrop {
            id,
            item: stack.item,
            count: stack.count,
            position: pbd_core::drops::scattered(at, id),
            made_s,
        };
        self.mem.drops.insert(id, drop);
        self.mem.hands.push((author.clone(), None, carried.clone()));
        self.writer.append(format::authored(
            author,
            format::toss_line_of(&drop, &none()),
        ));
        Some(drop)
    }
}
