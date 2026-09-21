//! Save slots: what a world is on disk, and the one resource that owns it.
//!
//! A slot is a directory under `saves/` holding two files: `edits.log`, the
//! transaction log of everything the player changed, and `world.ron`, the
//! metadata and the pose. [`format`] says what is in them and why; [`writer`]
//! is the thread that puts them there without ever touching a frame.
//!
//! [`WorldSave`] is the resource. It holds the edits in memory - the LOD
//! rebuild reads them, because a fine set built without them would undig every
//! hole the moment the player walked far enough for a new anchor - and it is
//! the only thing in the process that queues a write.

pub mod format;
pub mod writer;

use bevy::prelude::*;
use format::{Record, WorldFile};
use pbd_core::edits::{Edit, Edits};
use pbd_core::inventory::Slots;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use writer::SaveWriter;

/// The log of everything changed, appended to per edit.
pub const LOG: &str = "edits.log";
/// The metadata and the pose, replaced whole on a timer.
pub const WORLD: &str = "world.ron";
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
    std::fs::create_dir_all(root.join(&id))?;
    std::fs::write(root.join(&id).join(WORLD), file.to_ron())?;
    Ok(Slot { id, file })
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
        let (edits, carried, kit, damaged) = replay(&directory.join(LOG));
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
        Self {
            edits,
            carried,
            kit,
            pose,
            slot: Some(slot),
            writer: SaveWriter::new(&directory),
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
    pub fn accept(&mut self, edit: Edit, carried: &Slots) -> bool {
        if self.writer.failure().is_some() {
            return false;
        }
        self.edits.set(edit);
        self.writer.append(format::line_of(edit, carried));
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

    /// Queue the pose. Whole-file, so it is a replace rather than an append.
    pub fn snapshot(&mut self, pose: Pose) {
        let Some(slot) = self.slot.as_mut() else {
            return;
        };
        slot.file.position = Some(pose.position.to_array());
        slot.file.heading = Some(pose.heading.to_array());
        slot.file.pitch = pose.pitch;
        slot.file.selected = pose.selected;
        slot.file.played_unix_s = now_unix_s();
        let path = self.root.join(&slot.id).join(WORLD);
        let body = slot.file.to_ron();
        self.writer.replace(path, body);
    }

    /// Wait for everything queued to reach the disk. Called on the way out,
    /// which is the one place a player is already waiting.
    pub fn drain(&self) {
        self.writer.drain();
    }
}

/// Replay a log: the edits, the hotbar as the last line that carried one left
/// it, the highest kit version dealt, and how many lines were damaged.
fn replay(path: &Path) -> (Edits, Option<Slots>, u32, usize) {
    let mut edits = Edits::new();
    let mut carried = None;
    let mut kit = 0;
    let mut damaged = 0;
    let Ok(text) = std::fs::read_to_string(path) else {
        return (edits, carried, kit, damaged);
    };
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match format::parse_line(line) {
            Some(Record::Edit { edit, slots }) => {
                edits.set(edit);
                if let Some(slots) = slots {
                    carried = Some(slots);
                }
            }
            Some(Record::Kit { version, slots }) => {
                kit = kit.max(version);
                carried = Some(slots);
            }
            None => damaged += 1,
        }
    }
    (edits, carried, kit, damaged)
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
                    &carried
                ));
            }
            save.snapshot(Pose {
                position: Vec3::new(10.0, 20.0, 30.0),
                heading: Vec3::new(0.0, 0.0, 1.0),
                pitch: -0.25,
                selected: 3,
            });
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
            &Slots::new()
        ));
        assert_eq!(save.edits.len(), 1);
        assert_eq!(save.pending(), 0);
        assert!(save.slot().is_none());
    }
}
