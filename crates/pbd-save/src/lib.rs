//! A world's journal on disk: its line format ([`format`]) and its writer
//! ([`writer`]), shared by the game's saves and the dedicated server
//! (`openspec/changes/multiplayer` design 27). Moved here unchanged from
//! `pbd-app`'s `saves` module, which re-exports both.

pub mod format;
pub mod writer;

use pbd_core::records::Author;

/// The append-only journal of edits and lines, in a world's folder.
pub const LOG: &str = "edits.v1.log";

/// Who the water's edits are journaled as: `@wwater:flow/0`.
pub fn water_author() -> Author {
    Author::World {
        process: "water".into(),
        kind: "flow".into(),
        id: 0,
    }
}
