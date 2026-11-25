//! Storage chests (`storage-chests`): a block made at the workbench and set
//! down like a station, holding twenty slots.
//!
//! A chest's store is a raft hold's type at its own length
//! ([`crate::vehicle::hold::Hold`]), filled and emptied with the hold's
//! moves. It is stored as a record of its own, kind [`CHEST_KIND`], id
//! [`crate::station::record_id`] of its cell and layer, written in one line
//! with the player's slots on every move. An empty chest is an empty record:
//! records are never removed, so breaking a chest writes it empty.

use crate::vehicle::hold::Hold;

/// Slots in a chest: two rows of ten, twice a raft's hold.
pub const CHEST: usize = 20;

/// The record kind a chest's store is saved as, and its schema.
pub const CHEST_KIND: &str = "chest";
pub const CHEST_SCHEMA: u32 = 1;

/// An empty chest's store.
pub fn empty() -> Hold {
    Hold::sized(CHEST)
}

/// A chest's record id: its cell and layer, as a station's.
pub fn record_id(cell: u32, layer: u16) -> u64 {
    crate::station::record_id(cell, layer)
}
