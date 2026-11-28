//! The player's own character in the game (`openspec/changes/player-character`
//! 2.2 and 4.3): the look kept beside the saves in `character.ron`, one per
//! install, read by every world and sent to every host.
//!
//! An install with no saved look has the one [`character::rolled`] makes from
//! its key, the same every time, until its player chooses one.

use bevy::prelude::*;
use pbd_core::townsfolk::character::{self, Character};
use pbd_core::townsfolk::look::Look;
use std::path::PathBuf;

/// The file beside the saves.
const FILE: &str = "character.ron";

/// The player's look now.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct PlayerLook(pub Look);

impl PlayerLook {
    /// The saved look, or the install's rolled one.
    pub fn load() -> Self {
        Self(read(&path()).unwrap_or_else(|| character::rolled(&crate::net_play::install_key())))
    }

    /// Keep `look` beside the saves, replaced whole on disk. `false` if the
    /// look cannot be drawn or the file could not be written.
    pub fn save(look: &Look) -> bool {
        if !character::valid(look) {
            return false;
        }
        let path = path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        pbd_save::writer::replace_durably(&path, Character::of(look).to_ron().as_bytes()).is_ok()
    }
}

fn path() -> PathBuf {
    crate::paths::saves_dir().join(FILE)
}

/// A look from a `character.ron`, if it reads and can be drawn.
pub fn read(path: &std::path::Path) -> Option<Look> {
    Character::from_ron(&std::fs::read_to_string(path).ok()?)?.look()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `player-character` 2.2: a look written beside the saves reads back as
    /// it was; a file whose look cannot be drawn reads as none.
    #[test]
    fn a_saved_look_reads_back() {
        let dir = std::env::temp_dir().join(format!("pbd-character-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join(FILE);
        let look = character::rolled(&[7; 16]);
        pbd_save::writer::replace_durably(&file, Character::of(&look).to_ron().as_bytes()).unwrap();
        assert_eq!(read(&file), Some(look));
        let mut bad = Character::of(&look);
        bad.height_m = 9.0;
        std::fs::write(&file, bad.to_ron()).unwrap();
        assert_eq!(read(&file), None);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
