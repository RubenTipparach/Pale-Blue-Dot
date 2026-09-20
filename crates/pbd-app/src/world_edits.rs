//! What a player changed, and the file it is written to.
//!
//! The rules are `pbd_core::edits`; what is here is the one thing a core crate
//! must not know, which is where a save lives and whether the storage said
//! yes. `CLAUDE.md`: "Every accepted world mutation enters the durable
//! transaction path immediately. A queued write alone is not a durable save;
//! acknowledge commitment only after the storage backend succeeds."
//!
//! It lives in the lib rather than in the desktop binary because the LOD
//! rebuild reads it too: a fine set built without the edits would quietly
//! undig every hole the moment the player walked far enough for a new anchor.

use bevy::prelude::*;
use pbd_core::edits::{Edit, Edits};
use pbd_core::terrain::Material;
use std::io::Write;
use std::path::PathBuf;

/// Every edit in this world, and the file they are written to.
///
/// The resource owns the path rather than looking it up per edit, because the
/// write happens on the frame the edit is accepted and a path resolved then is
/// a path that can fail then.
#[derive(Resource)]
pub struct WorldEdits {
    pub edits: Edits,
    path: Option<PathBuf>,
}

impl Default for WorldEdits {
    fn default() -> Self {
        Self {
            edits: Edits::new(),
            path: None,
        }
    }
}

/// One edit, as a line of the save: cell, layer, material code.
///
/// A line per edit, appended, because the cost of writing has to be bounded by
/// the edit rather than by the size of the world, and because a torn write at
/// the end of a file then costs the last edit rather than the whole save.
fn line_of(edit: Edit) -> String {
    format!("{} {} {}\n", edit.cell, edit.layer, edit.material as u8)
}

fn parse_line(line: &str) -> Option<Edit> {
    let mut parts = line.split_whitespace();
    let cell = parts.next()?.parse().ok()?;
    let layer = parts.next()?.parse().ok()?;
    let code: u8 = parts.next()?.parse().ok()?;
    Some(Edit {
        cell,
        layer,
        material: material_of(code)?,
    })
}

/// The material a saved code names. Written out rather than transmuted: a
/// number that is not a material must fail to load rather than become one.
fn material_of(code: u8) -> Option<Material> {
    Some(match code {
        0 => Material::Air,
        1 => Material::Stone,
        2 => Material::Soil,
        3 => Material::Grass,
        4 => Material::Water,
        5 => Material::Ore,
        6 => Material::Sand,
        7 => Material::DryGrass,
        8 => Material::JungleGrass,
        9 => Material::Snow,
        10 => Material::Rock,
        11 => Material::Dirt,
        _ => return None,
    })
}

impl WorldEdits {
    /// Load a world's edits, and remember where to append the next one.
    pub fn open(path: PathBuf) -> Self {
        let mut edits = Edits::new();
        if let Ok(text) = std::fs::read_to_string(&path) {
            for line in text.lines() {
                if let Some(edit) = parse_line(line) {
                    edits.set(edit);
                }
            }
        }
        Self {
            edits,
            path: Some(path),
        }
    }

    /// Accept one edit: record it and put it on disk before returning.
    ///
    /// The write is appended AND FLUSHED, and the accept is reported only if
    /// the storage said yes, which is the rule's own wording: a queued write
    /// alone is not a durable save.
    pub fn accept(&mut self, edit: Edit) -> bool {
        let Some(path) = self.path.as_ref() else {
            // No world on disk (a headless capture, a test): the edit is real
            // in memory and nothing claims it was saved.
            self.edits.set(edit);
            return true;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let written = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| {
                file.write_all(line_of(edit).as_bytes())?;
                file.flush()?;
                file.sync_data()
            });
        match written {
            Ok(()) => {
                self.edits.set(edit);
                true
            }
            Err(error) => {
                warn!("edit not saved, so not applied: {error}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The save is the only thing that carries an edit between two runs, so
    /// what it must never do is lose one or change one on the way through.
    #[test]
    fn an_edit_survives_the_round_trip_through_the_file() {
        let dir = std::env::temp_dir().join(format!("pbd-edits-{}", std::process::id()));
        let path = dir.join("edits.txt");
        let _ = std::fs::remove_file(&path);
        let mut store = WorldEdits::open(path.clone());
        let made = [
            Edit {
                cell: 4_123_456_789,
                layer: 218,
                material: Material::Air,
            },
            Edit {
                cell: 4_123_456_789,
                layer: 217,
                material: Material::Air,
            },
            Edit {
                cell: 7,
                layer: 3,
                material: Material::Stone,
            },
        ];
        for edit in made {
            assert!(store.accept(edit), "the write is what makes it accepted");
        }
        let reloaded = WorldEdits::open(path.clone());
        assert_eq!(reloaded.edits.all(), store.edits.all());
        assert_eq!(reloaded.edits.for_cell(7), &[(3, Material::Stone)]);
        assert_eq!(
            reloaded.edits.for_cell(4_123_456_789),
            &[(218, Material::Air), (217, Material::Air)],
            "and in the order they were made, which is the order they apply"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A line that is not an edit is skipped rather than guessed at: a
    /// material code nothing names must not become a material.
    #[test]
    fn a_damaged_line_costs_that_line_and_no_other() {
        let dir = std::env::temp_dir().join(format!("pbd-edits-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("edits.txt");
        std::fs::write(&path, "5 10 1\nnonsense\n6 11 99\n7 12 0\n").unwrap();
        let store = WorldEdits::open(path);
        assert_eq!(store.edits.len(), 2, "the good lines, and only those");
        assert_eq!(store.edits.for_cell(5), &[(10, Material::Stone)]);
        assert_eq!(store.edits.for_cell(7), &[(12, Material::Air)]);
        assert!(store.edits.for_cell(6).is_empty(), "code 99 is no material");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
