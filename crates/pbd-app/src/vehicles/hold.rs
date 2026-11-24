//! A raft's hold in the save (`small-boats` design 5): a `hold` record per
//! raft, keyed by its craft id, whose body is its ten stacks in the pack
//! line's own stack text. Every change is written in ONE line with the
//! player's slots (`WorldSave::store_with_hands`), so a stack moved between
//! the two can never be in both or neither after a crash.

use crate::saves::WorldSave;
use crate::saves::format::{stack_of, stack_text};
use pbd_core::inventory::Slots;
use pbd_core::records::Record as StoredRecord;
use pbd_core::vehicle::hold::{HOLD, HOLD_KIND, HOLD_SCHEMA, Hold};

/// The hold of craft `id` as the save holds it: empty if it has none, or one
/// this build cannot read (which is kept in the save as it was).
pub fn read(save: &WorldSave, id: u64) -> Hold {
    read_sized(save, id, HOLD)
}

/// The hold of vehicle `id`, `slots` slots big (a cart's, `land-transport`
/// design 10). A ten-slot boat record reads as it always did.
pub fn read_sized(save: &WorldSave, id: u64, slots: usize) -> Hold {
    let mut hold = Hold::sized(slots);
    let size = hold.slots.len();
    let Some(record) = save.records.get(HOLD_KIND, id) else {
        return hold;
    };
    if record.schema != HOLD_SCHEMA {
        return hold;
    }
    let Ok(texts) = ron::from_str::<Vec<String>>(&record.body) else {
        return hold;
    };
    for (slot, text) in hold.slots.iter_mut().zip(texts.iter().take(size)) {
        *slot = stack_of(text).flatten();
    }
    hold
}

/// The record a hold is stored as.
pub fn record(id: u64, hold: &Hold) -> StoredRecord {
    let texts: Vec<String> = hold.slots.iter().map(|s| stack_text(*s)).collect();
    StoredRecord::of(HOLD_KIND, id, HOLD_SCHEMA, &texts)
}

/// Write the hold and the slots after a move, in one line. `false`, and
/// nothing kept, once the writer has failed.
pub fn store(save: &mut WorldSave, id: u64, hold: &Hold, slots: &Slots) -> bool {
    save.store_with_hands(record(id, hold), HOLD_SCHEMA, slots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Stack};
    use pbd_core::vehicle::hold::{Place, shift};

    /// Logs moved into a hold are in the hold and out of the slots, in the
    /// save as in memory.
    #[test]
    fn a_hold_is_stored_with_the_slots() {
        let mut save = WorldSave::memory_only();
        let mut slots = Slots::new();
        slots.set(14, Some(Stack::new(Item::Good(Good::LOG), 12)));
        let mut hold = read(&save, 9);
        assert!(hold.is_empty());
        shift(
            &mut slots,
            &mut hold,
            Place::Carried(14),
            Place::Hold(2),
            12,
        );
        assert!(store(&mut save, 9, &hold, &slots));
        let back = read(&save, 9);
        assert_eq!(back.get(2), Some(Stack::new(Item::Good(Good::LOG), 12)));
        assert_eq!(save.carried.as_ref().and_then(|c| c.get(14)), None);
        assert!(
            read(&save, 10).is_empty(),
            "another craft's hold is its own"
        );
    }

    /// On the disk: logs moved into a raft's hold are in it, and out of the
    /// slots, when the world is opened again (`small-boats` 4.1).
    #[test]
    fn a_hold_survives_a_reload() {
        let root = std::env::temp_dir().join(format!("pbd-hold-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Hold", 77).unwrap();
        let log = Item::Good(Good::LOG);
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let mut slots = Slots::new();
            slots.set(11, Some(Stack::new(log, 30)));
            let mut hold = read(&save, 5);
            shift(
                &mut slots,
                &mut hold,
                Place::Carried(11),
                Place::Hold(0),
                30,
            );
            assert!(store(&mut save, 5, &hold, &slots));
            save.drain();
        }
        let listed = crate::saves::list(&root);
        let reopened = WorldSave::open(root.clone(), listed[0].clone());
        assert_eq!(read(&reopened, 5).get(0), Some(Stack::new(log, 30)));
        assert_eq!(reopened.carried.as_ref().map(|c| c.count(log)), Some(0));
        let _ = std::fs::remove_dir_all(&root);
    }
}
