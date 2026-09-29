//! Save slots: what a world is on disk, and the one resource that owns it.
//!
//! A slot is a directory under `saves/` holding `identity.ron`, what the world
//! IS (its seed and every version that shapes it, written once);
//! `edits.v1.log`, the transaction log of everything the player changed; and
//! `world.ron`, the metadata and the pose. [`format`] says what is in them and
//! why; [`writer`] is the thread that puts them there without ever touching a
//! frame.
//!
//! [`WorldSave`] is the resource. It holds the edits in memory - the LOD
//! rebuild reads them, because a fine set built without them would undig every
//! hole the moment the player walked far enough for a new anchor - and it is
//! the only thing in the process that queues a write.

pub mod format;
pub mod migrate;
pub mod writer;

use bevy::prelude::*;
use format::{Identity, KEY_VERSION, Record, WorldFile};
use pbd_core::drops::ItemDrop;
use pbd_core::edits::{Edit, Edits};
use pbd_core::inventory::{Equipment, Slots};
use pbd_core::records::{Author, Proposal, Record as StoredRecord, Records, Refusal, YieldSet};
use pbd_core::vehicle::record::VehicleFile;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use writer::SaveWriter;

/// What the world is (`world-persistence` decision 3): written once, with a
/// barrier, before anything else in the slot.
pub const IDENTITY: &str = "identity.ron";
/// The log of everything changed, appended to per edit. Version 1 keys each
/// edit by its cell's exact key (`pbd_core::cell_key`); a save with only the
/// old `edits.log` is migrated into it once, on open ([`migrate`]).
pub const LOG: &str = "edits.v1.log";
/// The metadata and the pose, replaced whole on a timer.
pub const WORLD: &str = "world.ron";
/// The simulated atmosphere's state (`pbd_core::atmosphere`), replaced whole
/// on its own, slower timer.
pub const WEATHER: &str = "weather.bin";
/// How often the weather is written, seconds. It is simulated state rather
/// than anything a player did: losing the last minute of it costs a minute of
/// sky, and the simulation carries on from any state it is given.
pub const WEATHER_SAVE_S: f32 = 60.0;
/// Every craft in the world (`pbd_core::vehicle::record`), replaced whole
/// whenever one is made, boarded, left, made fast, cast off or comes to rest,
/// and on the pose's cadence while any moves.
pub const VEHICLES: &str = "vehicles.ron";
/// Where the slots live, relative to the working directory.
pub const ROOT: &str = "saves";
/// The longest a world's name may be, in bytes. Tenebris's `NAME_MAX`, for
/// its reason: a name is drawn in a list and used to make a path, and both
/// have a width past which they stop working.
pub const NAME_MAX: usize = 31;
/// How often the pose is written, seconds. Tenebris's own figure for the same
/// job: pose is the one part of a save that is genuinely cheap to lose, which
/// is exactly why it is the only part on a timer.
pub const AUTOSAVE_S: f32 = 5.0;

pub fn now_unix_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A slot, as the list shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    /// The directory name. Derived from the player's name and never the name
    /// itself, because a name is text and a directory is a path.
    pub id: String,
    pub file: WorldFile,
    /// What the world is, or `None` for a slot made before identities, which
    /// gains one when it is opened.
    pub identity: Option<Identity>,
}

/// A directory name for a player's name: lower case, letters and digits, and a
/// hyphen for anything else.
///
/// It is not reversible and does not need to be: the NAME lives in the world
/// file, and this only has to be a legal, stable, distinct path. A name that
/// leaves nothing behind gets one, so "???" is still a world.
pub fn slot_id(name: &str) -> String {
    let mut id: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .replace("--", "-");
    id.truncate(NAME_MAX);
    if id.is_empty() {
        id = format!("world-{}", now_unix_s());
    }
    id
}

/// Every slot on disk, newest played first.
///
/// Sorted, and the tie broken by the id, because a save list whose order comes
/// from a directory read is a list that reorders itself between launches.
pub fn list(root: &Path) -> Vec<Slot> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut slots: Vec<Slot> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if !entry.file_type().ok()?.is_dir() {
                return None;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(entry.path().join(WORLD)).ok()?;
            Some(Slot {
                file: WorldFile::from_ron(&text)?,
                identity: read_identity(&entry.path()),
                id,
            })
        })
        .collect();
    slots.sort_by(|a, b| {
        b.file
            .played_unix_s
            .cmp(&a.file.played_unix_s)
            .then_with(|| a.id.cmp(&b.id))
    });
    slots
}

/// Make a slot, writing its world file at once so it is in the list before
/// anything has been played in it. A name already taken gets a suffix rather
/// than opening somebody else's world.
pub fn create(root: &Path, name: &str, seed: u64) -> std::io::Result<Slot> {
    let name = &name[..name.len().min(NAME_MAX)];
    let base = slot_id(name);
    let mut id = base.clone();
    let mut suffix = 2;
    while root.join(&id).exists() {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    let file = WorldFile::new(name.to_string(), seed, now_unix_s());
    let identity = Identity::new(seed);
    std::fs::create_dir_all(root.join(&id))?;
    // The identity first and with a barrier, so a slot in the list always
    // says what it is.
    write_identity(&root.join(&id), &identity)?;
    std::fs::write(root.join(&id).join(WORLD), file.to_ron())?;
    Ok(Slot {
        id,
        file,
        identity: Some(identity),
    })
}

/// A slot's identity, where it has one.
pub fn read_identity(directory: &Path) -> Option<Identity> {
    let text = std::fs::read_to_string(directory.join(IDENTITY)).ok()?;
    let identity = Identity::from_ron(&text);
    if identity.is_none() {
        warn!("{}: identity.ron could not be read", directory.display());
    }
    identity
}

fn write_identity(directory: &Path, identity: &Identity) -> std::io::Result<()> {
    writer::replace_durably(&directory.join(IDENTITY), identity.to_ron().as_bytes())
}

/// The generator a slot's world is made by: its identity's, or 4 for a slot
/// from before identities, which is the only version one could have been
/// made under.
pub fn generator_of(slot: &Slot) -> u32 {
    slot.identity
        .as_ref()
        .map_or(4, |identity| identity.generator)
}

/// Why this build will not open a slot, naming the version it lacks, or
/// `None` when it will. A slot with no identity yet predates them, and was
/// made under versions every build since carries.
pub fn refusal(slot: &Slot) -> Option<String> {
    slot.identity.as_ref().and_then(Identity::refusal)
}

/// Remove a slot and everything in it. The caller is responsible for having
/// asked twice: this is the part that cannot be undone.
pub fn delete(root: &Path, id: &str) -> std::io::Result<()> {
    // Never anything but a direct child of the saves root, whatever the caller
    // believes an id is.
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "not a slot id",
        ));
    }
    std::fs::remove_dir_all(root.join(id))
}

/// What the player was doing, for the snapshot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub position: Vec3,
    pub heading: Vec3,
    pub pitch: f32,
    pub selected: usize,
}

/// How many of a species were caught, and the longest, cm.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CatchRecord {
    pub count: u32,
    pub best_cm: u32,
}

impl CatchRecord {
    fn add(&mut self, length_cm: u32) {
        self.count += 1;
        self.best_cm = self.best_cm.max(length_cm);
    }
}

/// What replaying a log gives back.
#[derive(Default)]
struct Replayed {
    edits: Edits,
    carried: Option<Slots>,
    kit: u32,
    catches: BTreeMap<u16, CatchRecord>,
    equipment: Option<Equipment>,
    drops: BTreeMap<u64, ItemDrop>,
    next_drop: u64,
    records: Records,
    yields: YieldSet,
    damaged: usize,
}

/// This world, in memory and on disk.
#[derive(Resource)]
pub struct WorldSave {
    /// Every edit, which the column tier and the LOD rebuild read.
    pub edits: Edits,
    /// The hotbar as the log last recorded it, for the load to restore.
    pub carried: Option<Slots>,
    /// The highest starting-kit version the log says was dealt into this
    /// world's hotbar; nought where none was.
    pub kit: u32,
    /// Where the player was, for the load to put them back.
    pub pose: Option<Pose>,
    /// The world's clock when it was last snapshotted, for the load to resume
    /// the hour and the season.
    pub world_seconds: Option<f64>,
    /// The atmosphere's saved state, for the load to restore.
    pub weather: Option<Vec<u8>>,
    /// The craft as last written, for the load to put back.
    pub vehicles: Option<VehicleFile>,
    /// The field guide's record, by species: folded from the log's catch
    /// lines, so there is no second store to keep in step with them.
    pub catches: BTreeMap<u16, CatchRecord>,
    /// The tool slot as the log last recorded it; `None` in a world that
    /// never changed tool, which opens with the new world's kit.
    pub equipment: Option<Equipment>,
    /// The drops the log says are still floating, in the order they were
    /// made, for the load to put back. Their time is judged by the world's
    /// clock once it is running, so one past its life is simply not shown.
    pub drops: Vec<ItemDrop>,
    /// What this world is: its seed and the versions that make it.
    pub identity: Identity,
    /// Every record the log holds, by kind and id: its last value
    /// (`world-persistence` decision 11). Kinds this build does not know are
    /// held as their text.
    pub records: Records,
    /// What the player has touched, which a world process may not change
    /// (decision 6).
    pub yields: YieldSet,
    /// The id the next drop is given: one past the highest the log names.
    next_drop: u64,
    slot: Option<Slot>,
    root: PathBuf,
    writer: SaveWriter,
}

impl Default for WorldSave {
    fn default() -> Self {
        Self {
            edits: Edits::new(),
            carried: None,
            kit: 0,
            pose: None,
            world_seconds: None,
            weather: None,
            vehicles: None,
            catches: BTreeMap::new(),
            equipment: None,
            drops: Vec::new(),
            records: Records::new(),
            yields: YieldSet::new(),
            // The seed is every generator version's (only the moisture moves), and
            // reading it here must not fix this run's generator before the
            // launch has chosen it from the world it opens.
            identity: Identity::new(pbd_core::planet_gen::TerrainConfig::TENEBRIS.seed),
            next_drop: 0,
            slot: None,
            root: PathBuf::from(ROOT),
            writer: SaveWriter::none(),
        }
    }
}

impl WorldSave {
    /// Open a slot: replay its log, read its pose, and start the writer on it.
    pub fn open(root: PathBuf, slot: Slot) -> Self {
        let directory = root.join(&slot.id);
        let identity = ensure_identity(&directory, &slot);
        let mut writer = SaveWriter::new(&directory);
        let text = log_text(&directory, &slot, &mut writer);
        // The key migration is the one deliberate upgrade there is: a log it
        // moved to exact keys is recorded as keyed so, once it has.
        let identity = if identity.keys < KEY_VERSION && directory.join(LOG).exists() {
            let upgraded = Identity {
                keys: KEY_VERSION,
                ..identity
            };
            if let Err(error) = write_identity(&directory, &upgraded) {
                error!("'{}': identity not upgraded: {error}", slot.id);
            }
            upgraded
        } else {
            identity
        };
        let Replayed {
            edits,
            carried,
            kit,
            catches,
            equipment,
            drops,
            next_drop,
            records,
            yields,
            damaged,
        } = replay(&text);
        if damaged > 0 {
            warn!("{damaged} damaged lines skipped in {}", slot.id);
        }
        info!(
            "world '{}' loaded: {} edits across {} cells",
            slot.file.name,
            edits.len(),
            edits.cells()
        );
        let pose = slot.file.position.map(|position| Pose {
            position: Vec3::from(position),
            heading: slot.file.heading.map_or(Vec3::X, Vec3::from),
            pitch: slot.file.pitch,
            selected: slot.file.selected,
        });
        let world_seconds = slot.file.world_seconds.filter(|s| s.is_finite());
        let weather = std::fs::read(directory.join(WEATHER)).ok();
        let vehicles = read_vehicles(&directory.join(VEHICLES));
        Self {
            edits,
            carried,
            kit,
            pose,
            world_seconds,
            weather,
            vehicles,
            catches,
            equipment,
            drops: drops.into_values().collect(),
            identity,
            records,
            yields,
            next_drop,
            slot: Some(slot),
            writer,
            root,
        }
    }

    /// The world with no disk behind it: a capture, a test, a smoke run.
    /// Everything works and nothing is written, which is why the call sites
    /// need no branch.
    pub fn memory_only() -> Self {
        Self::default()
    }

    pub fn slot(&self) -> Option<&Slot> {
        self.slot.as_ref()
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// How many records are queued and not yet on disk, and what has gone
    /// wrong if anything has.
    pub fn pending(&self) -> u64 {
        self.writer.pending()
    }

    pub fn failure(&self) -> Option<String> {
        self.writer.failure()
    }

    /// Accept one edit: apply it and queue the record. Nothing here opens,
    /// writes, flushes or syncs - the frame does not wait for a disk.
    ///
    /// It refuses once the writer has REPORTED a failure, and that is the
    /// durability rule's teeth in an async writer: the first failed write is
    /// the last accepted edit, so the world in front of the player cannot go
    /// on drifting away from the world on disk.
    ///
    /// A dig's line carries the drop it made (`inventory-grid` decision 4),
    /// so the hole and the block floating beside it reach the disk together.
    pub fn accept(&mut self, edit: Edit, drop: Option<&ItemDrop>, carried: &Slots) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.edits.set(edit);
        // The player's: no world process may change this cell again.
        self.yields.touch_cell(edit.cell);
        if let Some(drop) = drop {
            self.next_drop = self.next_drop.max(drop.id + 1);
        }
        self.writer.append(format::line_of(edit, drop, carried));
        true
    }

    /// The id for the next drop. Ids are never reused, so a pickup line names
    /// one drop for the life of the world.
    pub fn next_drop_id(&self) -> u64 {
        self.next_drop
    }

    /// Record a pickup: `left` remain on drop `id` (none: it is gone), and
    /// the rest went into `carried`, which the line carries whole.
    pub fn record_pick(&mut self, id: u64, left: u16, carried: &Slots) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.carried = Some(carried.clone());
        self.writer.append(format::pick_line_of(id, left, carried));
        true
    }

    /// Record the slots after a move in the pack. The same durable path as an
    /// edit: a stack moved and not saved would be back where it was at the
    /// next load, or in two places.
    pub fn record_pack(&mut self, carried: &Slots) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.carried = Some(carried.clone());
        self.writer.append(format::pack_line_of(carried));
        true
    }

    /// Record that the kit's grants up to `version` were dealt, and the hotbar
    /// they were dealt into. The same durable path as an edit, for the same
    /// reason: a grant that showed and did not save would be dealt again on
    /// the next load, or lost, depending on which write won.
    pub fn deal_kit(&mut self, version: u32, carried: &Slots) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.kit = version;
        self.carried = Some(carried.clone());
        self.writer.append(format::kit_line_of(version, carried));
        true
    }

    /// Record a catch: the fish went into `carried`, which the line carries
    /// whole, and the field guide's record grows. The same durable path as an
    /// edit, on the frame of the catch.
    pub fn record_catch(&mut self, species: u16, length_cm: u32, carried: &Slots) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.catches.entry(species).or_default().add(length_cm);
        self.carried = Some(carried.clone());
        self.writer
            .append(format::catch_line_of(species, length_cm, carried));
        true
    }

    /// Record the tool slot after a change of tool, on the frame of it.
    pub fn record_hand(&mut self, equipment: &Equipment) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.equipment = Some(*equipment);
        self.writer.append(format::hand_line_of(equipment));
        true
    }

    /// Queue the pose and the world's clock. Whole-file, so it is a replace
    /// rather than an append.
    pub fn snapshot(&mut self, pose: Pose, world_seconds: f64) {
        if world_seconds.is_finite() {
            self.world_seconds = Some(world_seconds);
        }
        let Some(slot) = self.slot.as_mut() else {
            return;
        };
        slot.file.position = Some(pose.position.to_array());
        slot.file.heading = Some(pose.heading.to_array());
        slot.file.pitch = pose.pitch;
        slot.file.selected = pose.selected;
        slot.file.world_seconds = self.world_seconds;
        slot.file.played_unix_s = now_unix_s();
        let path = self.root.join(&slot.id).join(WORLD);
        let body = slot.file.to_ron();
        self.writer.replace(path, body);
    }

    /// Queue the atmosphere's state. Whole-file, like the pose.
    pub fn snapshot_weather(&mut self, bytes: Vec<u8>) {
        let Some(slot) = self.slot.as_ref() else {
            return;
        };
        let path = self.root.join(&slot.id).join(WEATHER);
        self.writer.replace(path, bytes.clone());
        self.weather = Some(bytes);
    }

    /// Queue every craft. Whole-file, like the pose. Refused, and reported
    /// as refused, once the writer has failed: the caller keeps the write
    /// owed rather than believing a craft is saved that is not.
    pub fn snapshot_vehicles(&mut self, file: &VehicleFile) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.vehicles = Some(file.clone());
        let Some(slot) = self.slot.as_ref() else {
            return true;
        };
        let body = ron::ser::to_string_pretty(file, ron::ser::PrettyConfig::default())
            .expect("a vehicle file is plain data");
        let path = self.root.join(&slot.id).join(VEHICLES);
        self.writer.replace(path, body);
        true
    }

    /// The highest sequence the disk has taken: a line queued with a
    /// sequence at or below it is saved. A world with no disk behind it
    /// counts everything as saved, since nothing is ever queued.
    pub fn committed(&self) -> u64 {
        self.writer.committed()
    }

    /// Store records as `author`'s (`world-persistence` decision 11): each
    /// is applied to [`Self::records`] and its line queued, in order. The
    /// last line's sequence comes back; the records are saved once
    /// [`Self::committed`] reaches it. `None`, and nothing stored, once the
    /// writer has failed.
    pub fn store(&mut self, author: &Author, records: Vec<StoredRecord>) -> Option<u64> {
        if self.writer.failure().is_some() {
            return None;
        }
        let mut last = 0;
        for record in records {
            let line = record.line(author);
            let before = self.records.put(record.clone());
            if author.is_player() {
                self.yields.touch_record(before.as_ref(), &record);
            }
            last = self.writer.append(line);
        }
        Some(last)
    }

    /// A world process's changes for a period, taken whole or refused whole
    /// (`world-persistence` decision 6): refused, with nothing written, if
    /// any of it would change a cell or a record field the player has.
    /// Taken, its edits and records are journaled under `author`.
    pub fn propose(&mut self, author: &Author, proposal: Proposal) -> Result<Option<u64>, Refusal> {
        self.yields.check(&proposal, &self.records)?;
        if self.writer.failure().is_some() {
            return Ok(None);
        }
        let mut last = 0;
        for edit in proposal.edits {
            self.edits.set(edit);
            last = self
                .writer
                .append(format::authored(author, format::world_edit_line(edit)));
        }
        if !proposal.records.is_empty() {
            last = self.store(author, proposal.records).unwrap_or(last);
        }
        Ok(Some(last))
    }

    /// Record in the identity that the world holds records of these kinds,
    /// at these schema versions: a deliberate upgrade, written once, the
    /// first time a kind is stored.
    pub fn note_record_kinds(&mut self, kinds: &[(&str, u32)]) {
        let mut changed = false;
        for &(kind, schema) in kinds {
            if self.identity.records.get(kind) != Some(&schema) {
                self.identity.records.insert(kind.to_string(), schema);
                changed = true;
            }
        }
        let Some(slot) = self.slot.as_ref().filter(|_| changed) else {
            return;
        };
        let path = self.root.join(&slot.id).join(IDENTITY);
        self.writer.replace(path, self.identity.to_ron());
    }

    /// Wait for everything queued to reach the disk. Called on the way out,
    /// which is the one place a player is already waiting.
    pub fn drain(&self) {
        self.writer.drain();
    }
}

/// The craft file, or nothing: a world with no craft yet, or a file this
/// build cannot read, which is logged and not a reason to refuse the world.
fn read_vehicles(path: &Path) -> Option<VehicleFile> {
    let text = std::fs::read_to_string(path).ok()?;
    match ron::from_str::<VehicleFile>(&text) {
        Ok(file) if file.version == pbd_core::vehicle::record::RECORD_VERSION => Some(file),
        Ok(file) => {
            warn!(
                "{}: craft layout {} is not this build's",
                path.display(),
                file.version
            );
            None
        }
        Err(error) => {
            warn!("{}: {error}", path.display());
            None
        }
    }
}

/// The slot's identity, written before anything else is done to a slot that
/// predates identities (`world/persistence`: "A save from before
/// identities"): generator 4 and topology 1, keyed exactly if its log already
/// is or it has none, by the old hash if only `edits.log` is there.
fn ensure_identity(directory: &Path, slot: &Slot) -> Identity {
    if let Some(identity) = read_identity(directory) {
        return identity;
    }
    let keys = if directory.join(LOG).exists() || !directory.join(migrate::LEGACY_LOG).exists() {
        KEY_VERSION
    } else {
        0
    };
    let identity = Identity::legacy(slot.file.seed, keys);
    match write_identity(directory, &identity) {
        Ok(()) => info!(
            "'{}' predates identities: recorded as generator {}, topology {}, keys {}",
            slot.id, identity.generator, identity.topology, identity.keys
        ),
        Err(error) => error!("'{}': identity not written: {error}", slot.id),
    }
    identity
}

/// The slot's log, as text: `edits.v1.log` where it exists; otherwise the old
/// `edits.log` migrated to exact keys, written as `edits.v1.log` through the
/// save thread and waited for before the world is shown; otherwise nothing,
/// which is a new world.
fn log_text(directory: &Path, slot: &Slot, writer: &mut SaveWriter) -> String {
    if let Ok(text) = std::fs::read_to_string(directory.join(LOG)) {
        return text;
    }
    let Ok(old) = std::fs::read_to_string(directory.join(migrate::LEGACY_LOG)) else {
        return String::new();
    };
    let started = std::time::Instant::now();
    let position = slot.file.position.map(Vec3::from);
    let (text, report) = migrate::convert(&old, position);
    writer.replace(directory.join(LOG), text.clone());
    writer.drain();
    info!(
        "'{}' migrated to exact cell keys in {:.2} s: {} edits, {} by one cell, {} by the ground, {} by position, {} kept on every candidate {:?}, {} dropped {:?}",
        slot.id,
        started.elapsed().as_secs_f64(),
        report.edits,
        report.unique,
        report.by_surface,
        report.by_position,
        report.ambiguous.len(),
        report.ambiguous,
        report.unknown.len(),
        report.unknown,
    );
    if let Some(failure) = writer.failure() {
        error!("'{}': the migrated log was not written: {failure}", slot.id);
    }
    text
}

/// Replay a log: the edits, the hotbar as the last line that carried one left
/// it, the highest kit version dealt, the catches, the last tool slot, the
/// records and what the player touched, and how many lines were damaged.
fn replay(text: &str) -> Replayed {
    let mut out = Replayed::default();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let Some((author, record)) = format::parse_entry(line) else {
            out.damaged += 1;
            continue;
        };
        match record {
            Record::Edit { edit, drop, slots } => {
                out.edits.set(edit);
                if author.is_player() {
                    out.yields.touch_cell(edit.cell);
                }
                if let Some(drop) = drop {
                    out.next_drop = out.next_drop.max(drop.id + 1);
                    out.drops.insert(drop.id, drop);
                }
                if let Some(slots) = slots {
                    out.carried = Some(slots);
                }
            }
            Record::Pick { id, left, slots } => {
                if left == 0 {
                    out.drops.remove(&id);
                } else if let Some(drop) = out.drops.get_mut(&id) {
                    drop.count = left;
                }
                out.carried = Some(slots);
            }
            Record::Pack { slots } => out.carried = Some(slots),
            Record::Kit { version, slots } => {
                out.kit = out.kit.max(version);
                out.carried = Some(slots);
            }
            Record::Catch {
                species,
                length_cm,
                slots,
            } => {
                out.catches.entry(species).or_default().add(length_cm);
                out.carried = Some(slots);
            }
            Record::Hand { equipment } => out.equipment = Some(equipment),
            Record::Stored(record) => {
                if author.is_player() {
                    let before = out.records.get(&record.kind, record.id).cloned();
                    out.yields.touch_record(before.as_ref(), &record);
                }
                out.records.put(record);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::inventory::Item;
    use pbd_core::terrain::Material;

    fn temporary(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pbd-slots-{name}-{}-{}",
            std::process::id(),
            now_unix_s()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// A name is text and a directory is a path, and the one thing the
    /// translation must never do is fail to produce one.
    #[test]
    fn every_name_becomes_a_legal_distinct_directory() {
        assert_eq!(slot_id("Caves"), "caves");
        assert_eq!(slot_id("My World 2"), "my-world-2");
        assert_eq!(slot_id("  spaced  "), "spaced");
        assert_eq!(slot_id("../../etc"), "etc");
        assert!(!slot_id("???").is_empty(), "a name still gets a world");
        assert!(slot_id(&"x".repeat(200)).len() <= NAME_MAX);
    }

    /// Create, list, delete. The list's ORDER is the part worth pinning: a
    /// save list that reorders itself between launches is one a player cannot
    /// learn.
    #[test]
    fn slots_are_made_listed_in_a_stable_order_and_removed() {
        let root = temporary("index");
        let first = create(&root, "Alpha", 7).unwrap();
        let mut second = create(&root, "Beta", 7).unwrap();
        // Beta was played later, so it is first.
        second.file.played_unix_s = first.file.played_unix_s + 10;
        std::fs::write(root.join(&second.id).join(WORLD), second.file.to_ron()).unwrap();
        let listed = list(&root);
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, second.id, "newest played first");
        assert_eq!(list(&root), listed, "and the same order every time");

        // A second world of the same name is its own world, never the first.
        let twin = create(&root, "Alpha", 7).unwrap();
        assert_ne!(twin.id, first.id);
        assert_eq!(list(&root).len(), 3);

        delete(&root, &first.id).unwrap();
        assert_eq!(list(&root).len(), 2);
        assert!(list(&root).iter().all(|slot| slot.id != first.id));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Nothing outside the saves root is deletable, whatever a caller believes
    /// an id to be.
    #[test]
    fn delete_refuses_anything_that_is_not_a_slot() {
        let root = temporary("escape");
        std::fs::create_dir_all(&root).unwrap();
        for id in ["", "..", "../..", "a/b", "a\\b"] {
            assert!(delete(&root, id).is_err(), "{id:?} was allowed");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// One craft left at anchor: what a vehicle file holds.
    fn parked() -> VehicleFile {
        use pbd_core::vehicle::{Craft, Hulls, Kind, spec::VehicleSpecs};
        let specs = std::sync::Arc::new(VehicleSpecs::default());
        let hulls = Hulls::new(&specs);
        let mut craft = Craft::new(
            Kind::Tern,
            7,
            specs,
            hulls,
            pbd_core::DVec3::new(0.0, 4799.5, 0.0),
            pbd_core::DQuat::IDENTITY,
        );
        craft.mooring = Some(pbd_core::vehicle::Mooring {
            at: pbd_core::DVec3::new(0.0, 4790.0, -3.0),
            length: 30.0,
            anchored: true,
        });
        VehicleFile {
            version: pbd_core::vehicle::record::RECORD_VERSION,
            next_id: 8,
            vehicles: vec![craft.record()],
        }
    }

    /// The whole feature, end to end and on the disk: dig, put the pose down,
    /// and open it again as another run would.
    #[test]
    fn a_world_comes_back_with_its_edits_its_hotbar_and_its_pose() {
        let root = temporary("round");
        let slot = create(&root, "Round Trip", 4242).unwrap();
        let mut carried = Slots::new();
        carried.give(Item::Block(Material::Grass), 3);
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            for layer in [218u16, 217] {
                carried.give(Item::Block(Material::Dirt), 1);
                assert!(save.accept(
                    Edit {
                        cell: 4_123_456_789,
                        layer,
                        material: Material::Air,
                    },
                    None,
                    &carried
                ));
            }
            save.snapshot(
                Pose {
                    position: Vec3::new(10.0, 20.0, 30.0),
                    heading: Vec3::new(0.0, 0.0, 1.0),
                    pitch: -0.25,
                    selected: 3,
                },
                40.5 * 2880.0,
            );
            save.snapshot_weather(vec![7, 1, 2, 3]);
            assert!(save.snapshot_vehicles(&parked()));
            save.drain();
        }
        let listed = list(&root);
        let reopened = WorldSave::open(root.clone(), listed[0].clone());
        assert_eq!(
            reopened.edits.for_cell(4_123_456_789),
            &[(218, Material::Air), (217, Material::Air)],
            "every edit, in the order they apply"
        );
        assert_eq!(reopened.carried.as_ref(), Some(&carried), "and the hotbar");
        assert_eq!(reopened.kit, 0, "no kit line was written");
        let pose = reopened.pose.expect("and the pose");
        assert_eq!(pose.position, Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(pose.selected, 3);
        assert!((pose.pitch + 0.25).abs() < 1e-6);
        assert_eq!(reopened.world_seconds, Some(40.5 * 2880.0), "and the clock");
        assert_eq!(reopened.weather, Some(vec![7, 1, 2, 3]), "and the weather");
        assert_eq!(
            reopened.vehicles,
            Some(parked()),
            "and every craft, where it was left"
        );
        assert_eq!(reopened.slot().unwrap().file.seed, 4242, "and its world");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A kit line is the last word on the hotbar and on the version dealt,
    /// and it survives the reopen like an edit does.
    #[test]
    fn a_dealt_kit_comes_back_with_its_version_and_its_hotbar() {
        let root = temporary("kit");
        let slot = create(&root, "Kit", 9).unwrap();
        let mut carried = Slots::new();
        carried.give(Item::Block(Material::Grass), 3);
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            assert!(save.accept(
                Edit {
                    cell: 77,
                    layer: 200,
                    material: Material::Air,
                },
                None,
                &carried
            ));
            carried.give(Item::Block(Material::Torch), 16);
            assert!(save.deal_kit(3, &carried));
            assert_eq!(save.kit, 3);
            save.drain();
        }
        let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
        assert_eq!(reopened.kit, 3);
        assert_eq!(reopened.carried.as_ref(), Some(&carried));
        assert_eq!(reopened.edits.for_cell(77), &[(200, Material::Air)]);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A catch and a change of tool come back from the disk: the fish in the
    /// hotbar, the field guide's count and best, and the tool in hand.
    #[test]
    fn catches_and_the_tool_in_hand_come_back() {
        use pbd_core::inventory::Tool;
        let root = temporary("catch");
        let slot = create(&root, "Catch", 11).unwrap();
        let mut carried = Slots::new();
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            carried.give(Item::Fish(5), 1);
            assert!(save.record_catch(5, 44, &carried));
            carried.give(Item::Fish(5), 1);
            assert!(save.record_catch(5, 39, &carried));
            carried.give(Item::Fish(0), 1);
            assert!(save.record_catch(0, 15, &carried));
            let mut hand = Equipment::default();
            hand.hold(Tool::Shovel);
            assert!(save.record_hand(&hand));
            save.drain();
        }
        let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
        assert_eq!(reopened.carried.as_ref(), Some(&carried));
        assert_eq!(
            reopened.catches.get(&5),
            Some(&CatchRecord {
                count: 2,
                best_cm: 44
            })
        );
        assert_eq!(reopened.catches.get(&0).map(|r| r.count), Some(1));
        assert_eq!(reopened.equipment.map(|e| e.held()), Some(Tool::Shovel));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A dig's drop floats on across a reopen with what is left on it; a
    /// drop picked up whole does not; a pack move's slots come back; and the
    /// next drop's id is past every id the log names, so a pickup line never
    /// names two drops.
    #[test]
    fn drops_pickups_and_pack_moves_come_back() {
        use pbd_core::drops::ItemDrop;
        let root = temporary("drops");
        let slot = create(&root, "Drops", 12).unwrap();
        let drop = |id: u64, material| ItemDrop {
            id,
            item: Item::Block(material),
            count: 1,
            position: Vec3::new(1.25, 300.5, -2.0),
            made_s: 1000.0 + id as f64,
        };
        let dig = |layer| Edit {
            cell: 55,
            layer,
            material: Material::Air,
        };
        let mut carried = Slots::new();
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            assert_eq!(save.next_drop_id(), 0);
            assert!(save.accept(dig(200), Some(&drop(0, Material::Dirt)), &carried));
            assert!(save.accept(dig(199), Some(&drop(1, Material::Stone)), &carried));
            assert_eq!(save.next_drop_id(), 2);
            carried.give(Item::Block(Material::Dirt), 1);
            assert!(save.record_pick(0, 0, &carried));
            carried.send(0);
            assert!(save.record_pack(&carried));
            save.drain();
        }
        let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
        assert_eq!(reopened.drops, vec![drop(1, Material::Stone)]);
        assert_eq!(reopened.next_drop_id(), 2);
        assert_eq!(reopened.carried.as_ref(), Some(&carried));
        assert!(
            reopened.carried.unwrap().get(10).is_some(),
            "sent to the pack"
        );
        assert_eq!(reopened.edits.for_cell(55).len(), 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The memory-only world is what a capture and a test run on: everything
    /// works, nothing is written, and nothing claims to have been.
    #[test]
    fn a_world_with_no_disk_accepts_edits_and_saves_nothing() {
        let mut save = WorldSave::memory_only();
        assert!(save.accept(
            Edit {
                cell: 1,
                layer: 2,
                material: Material::Air
            },
            None,
            &Slots::new()
        ));
        assert_eq!(save.edits.len(), 1);
        assert_eq!(save.pending(), 0);
        assert!(save.slot().is_none());
    }

    /// An old save's `edits.log` is migrated once to `edits.v1.log`
    /// (`exact-cell-keys` 4.1, 4.2): its edit lands on the exact key and its
    /// other lines come along; `edits.log` is left byte for byte; a second
    /// open reads the new log and does not migrate again; and new edits go to
    /// the new log alone.
    #[test]
    fn an_old_save_is_migrated_once_and_its_old_log_is_left_alone() {
        use pbd_core::cell_key::{self, Address};
        let root = temporary("migrate");
        let slot = create(&root, "Old World", 4242).unwrap();
        let directory = root.join(&slot.id);
        let cell = Address {
            face: 7,
            level: 11,
            i: 300,
            j: 400,
        };
        let (hash, key) = (cell_key::old_hash(cell), cell_key::key(cell).unwrap());
        let kit = "kit 3 - - - - - - - - - -";
        let old = format!("{hash} 150 0\n{kit}\n");
        let legacy = directory.join(migrate::LEGACY_LOG);
        std::fs::write(&legacy, &old).unwrap();
        {
            let save = WorldSave::open(root.clone(), slot.clone());
            assert_eq!(save.edits.for_cell(key), &[(150, Material::Air)]);
            assert!(save.edits.for_cell(hash).is_empty(), "nothing by the hash");
            assert_eq!(save.kit, 3, "the other lines came along");
        }
        assert_eq!(
            std::fs::read_to_string(directory.join(LOG)).unwrap(),
            format!("{key} 150 0\n{kit}\n")
        );
        assert_eq!(std::fs::read_to_string(&legacy).unwrap(), old, "untouched");

        // A line added to the old log now is never read: the second open
        // takes the new log and does not migrate again.
        let grown = format!("{old}{hash} 149 0\n");
        std::fs::write(&legacy, &grown).unwrap();
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            assert_eq!(save.edits.for_cell(key), &[(150, Material::Air)]);
            assert!(save.accept(
                Edit {
                    cell: key,
                    layer: 151,
                    material: Material::Stone,
                },
                None,
                &Slots::new()
            ));
            save.drain();
        }
        let v1 = std::fs::read_to_string(directory.join(LOG)).unwrap();
        assert_eq!(v1.lines().count(), 3);
        assert!(
            v1.lines()
                .last()
                .unwrap()
                .starts_with(&format!("{key} 151 1 "))
        );
        assert_eq!(
            std::fs::read_to_string(&legacy).unwrap(),
            grown,
            "never written"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A new world is made with its identity, written before its world file,
    /// naming this build's versions; the list reads it back
    /// (`world-persistence` 2.1).
    #[test]
    fn a_new_world_is_made_with_its_identity() {
        let root = temporary("identity");
        let slot = create(&root, "Named", 4242).unwrap();
        assert_eq!(slot.identity, Some(Identity::new(4242)));
        let listed = list(&root);
        assert_eq!(listed[0].identity, Some(Identity::new(4242)));
        assert_eq!(refusal(&listed[0]), None);
        assert_eq!(
            generator_of(&listed[0]),
            pbd_core::terrain::GENERATOR_VERSION,
            "made on this build's generator"
        );
        let save = WorldSave::open(root.clone(), listed[0].clone());
        assert_eq!(save.identity, Identity::new(4242));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A slot from before identities gains one the first time it is opened:
    /// generator 4, topology 1. One with only the old hash-keyed log is
    /// recorded as keyed by the hash first, and as exactly keyed once the
    /// open has migrated it (`world/persistence`: "A save from before
    /// identities").
    #[test]
    fn a_world_from_before_identities_gains_one_when_opened() {
        use pbd_core::cell_key::{self, Address};
        let root = temporary("legacy-identity");
        let slot = create(&root, "Before", 4242).unwrap();
        let directory = root.join(&slot.id);
        std::fs::remove_file(directory.join(IDENTITY)).unwrap();
        let cell = Address {
            face: 3,
            level: 11,
            i: 12,
            j: 34,
        };
        std::fs::write(
            directory.join(migrate::LEGACY_LOG),
            format!("{} 150 0\n", cell_key::old_hash(cell)),
        )
        .unwrap();
        let listed = list(&root);
        assert_eq!(listed[0].identity, None, "none yet");
        assert_eq!(refusal(&listed[0]), None, "and it opens");
        let save = WorldSave::open(root.clone(), listed[0].clone());
        let written = read_identity(&directory).expect("written on open");
        assert_eq!(written, Identity::legacy(4242, KEY_VERSION));
        assert_eq!(save.identity, written);
        assert_eq!(written.generator, 4);
        assert_eq!(written.topology, 1);
        assert_eq!(generator_of(&listed[0]), 4, "made on version 4");
        assert_eq!(generator_of(&list(&root)[0]), 4, "and it stays so");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A world whose identity names a version this build lacks is refused
    /// with the version named, and is never opened: its identity file is
    /// left as it was.
    #[test]
    fn a_world_of_a_version_this_build_lacks_is_refused_by_name() {
        let root = temporary("future");
        let slot = create(&root, "Future", 4242).unwrap();
        let directory = root.join(&slot.id);
        let mut future = Identity::new(4242);
        future.generator = 99;
        std::fs::write(directory.join(IDENTITY), future.to_ron()).unwrap();
        let listed = list(&root);
        let why = refusal(&listed[0]).expect("refused");
        assert!(why.contains("generator version 99"), "{why}");
        assert_eq!(read_identity(&directory), Some(future));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// "A newer save in an older build": a record of a kind this build has
    /// never heard of is held as its text, and after a dig and a quit it is
    /// still in the log byte for byte, and read back the same (task 3.1).
    #[test]
    fn a_record_of_an_unknown_kind_survives_a_dig_and_a_quit_byte_for_byte() {
        let root = temporary("unknown-record");
        let slot = create(&root, "Landmarks", 21).unwrap();
        let landmark =
            r#"rec @c landmark 9 4 Landmark(name: "Old Tower", keeps: {"bell": [1, 2]})"#;
        std::fs::write(root.join(&slot.id).join(LOG), format!("{landmark}\n")).unwrap();
        let mut carried = Slots::new();
        carried.give(Item::Block(Material::Dirt), 1);
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let held = save
                .records
                .get("landmark", 9)
                .expect("held, not understood");
            assert_eq!(held.schema, 4);
            assert!(save.accept(
                Edit {
                    cell: 77,
                    layer: 200,
                    material: Material::Air,
                },
                None,
                &carried
            ));
            save.drain();
        }
        let text = std::fs::read_to_string(root.join(&slot.id).join(LOG)).unwrap();
        assert_eq!(text.lines().next(), Some(landmark), "byte for byte");
        let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
        assert_eq!(
            reopened
                .records
                .get("landmark", 9)
                .map(|r| r.line(&Author::Creation)),
            Some(format!("{landmark}\n"))
        );
        assert_eq!(reopened.edits.for_cell(77), &[(200, Material::Air)]);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Records a world stores come back from the disk, the last value for
    /// each key; a log from before authors replays as it always did and is
    /// the player's; a torn record line at the end costs that line alone
    /// (task 3.2).
    #[test]
    fn records_come_back_and_a_torn_record_line_costs_only_itself() {
        let root = temporary("records");
        let slot = create(&root, "Records", 22).unwrap();
        let old = "77 200 0\n78 150 0\n";
        std::fs::write(root.join(&slot.id).join(LOG), old).unwrap();
        let site = |id, name: &str| StoredRecord {
            kind: "site".into(),
            id,
            schema: 1,
            body: format!("(name:\"{name}\")"),
        };
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            assert!(
                save.yields.has_cell(77) && save.yields.has_cell(78),
                "the player's"
            );
            assert!(
                save.store(
                    &Author::Creation,
                    vec![site(1, "Holbrook"), site(2, "Corford")]
                )
                .is_some()
            );
            assert!(
                save.store(&Author::Creation, vec![site(2, "Cormouth")])
                    .is_some()
            );
            save.drain();
        }
        let path = root.join(&slot.id).join(LOG);
        let mut text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with(old), "the old lines are untouched");
        text.push_str("rec @c site 3 1 (name:\"Ash");
        std::fs::write(&path, &text).unwrap();
        let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
        let names: Vec<String> = reopened
            .records
            .of_kind("site")
            .map(|r| r.body.clone())
            .collect();
        assert_eq!(names, vec!["(name:\"Holbrook\")", "(name:\"Cormouth\")"]);
        assert_eq!(reopened.edits.for_cell(78), &[(150, Material::Air)]);
        assert!(!reopened.yields.has_cell(1), "records are not cells");
        let _ = std::fs::remove_dir_all(&root);
    }

    /// "The world yields to the player": a proposal touching a cell the
    /// player dug is refused whole and writes nothing; one that does not is
    /// journaled under its process, and a reload has it, not as the
    /// player's (task 3.3).
    #[test]
    fn a_world_proposal_on_the_players_work_is_refused_and_writes_nothing() {
        let root = temporary("yield");
        let slot = create(&root, "Yield", 23).unwrap();
        let dig = |cell| Edit {
            cell,
            layer: 100,
            material: Material::Air,
        };
        let process = Author::World {
            process: "test".into(),
            kind: "building".into(),
            id: 40,
        };
        let house = |state: u8| StoredRecord {
            kind: "building".into(),
            id: 40,
            schema: 1,
            body: format!("(state:{state},door:0)"),
        };
        let mut carried = Slots::new();
        carried.give(Item::Block(Material::Dirt), 1);
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            assert!(save.store(&Author::Creation, vec![house(0)]).is_some());
            assert!(save.accept(dig(5), None, &carried));
            save.drain();
            let before = std::fs::read_to_string(root.join(&slot.id).join(LOG)).unwrap();
            let refused = save.propose(
                &process,
                Proposal {
                    edits: vec![dig(6), dig(5)],
                    records: vec![house(1)],
                },
            );
            assert_eq!(refused, Err(Refusal::Cell(5)));
            save.drain();
            let after = std::fs::read_to_string(root.join(&slot.id).join(LOG)).unwrap();
            assert_eq!(before, after, "nothing of it written");
            assert!(save.edits.for_cell(6).is_empty(), "nor applied");
            let taken = save.propose(
                &process,
                Proposal {
                    edits: vec![dig(6)],
                    records: vec![house(1)],
                },
            );
            assert!(matches!(taken, Ok(Some(_))), "{taken:?}");
            save.drain();
        }
        let text = std::fs::read_to_string(root.join(&slot.id).join(LOG)).unwrap();
        assert!(text.contains("@wtest:building/40 6 100 0\n"), "{text}");
        let reopened = WorldSave::open(root.clone(), list(&root)[0].clone());
        assert_eq!(reopened.edits.for_cell(6), &[(100, Material::Air)]);
        assert!(
            !reopened.yields.has_cell(6),
            "the world's dig is not the player's"
        );
        assert!(reopened.yields.has_cell(5));
        assert_eq!(
            reopened
                .records
                .get("building", 40)
                .map(|r| r.body.as_str()),
            Some("(state:1,door:0)")
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
