//! A storage chest in the save (`storage-chests` design 2): a `chest` record
//! per chest, keyed by its cell and layer ([`pbd_core::chest::record_id`]),
//! whose body is its twenty stacks in the pack line's own stack text, as a
//! raft's hold is stored (`vehicles::hold`). Every move is written in ONE
//! line with the player's slots (`WorldSave::store_with_hands`), so a stack
//! moved between the two can never be in both or neither after a crash.
//!
//! Breaking a chest (design 4) spills it: each stored stack is written as a
//! drop at the chest first, then the record is written empty, and only then
//! does the dig's own line (with the chest's own drop) follow. A crash between
//! the two keeps every item, at worst twice, never none.

use crate::drops::Drops;
use crate::saves::WorldSave;
use crate::saves::format::{stack_of, stack_text};
use bevy::math::DVec3;
use pbd_core::chest::{CHEST, CHEST_KIND, CHEST_SCHEMA, empty, record_id};
use pbd_core::drops::ItemDrop;
use pbd_core::inventory::Slots;
use pbd_core::records::Record as StoredRecord;
use pbd_core::vehicle::hold::Hold;

/// The chest at `cell` and `layer` as the save holds it: empty if it has no
/// record, or one this build cannot read (which is kept in the save as it
/// was).
pub fn read(save: &WorldSave, cell: u32, layer: u16) -> Hold {
    let mut chest = empty();
    let Some(record) = save.records.get(CHEST_KIND, record_id(cell, layer)) else {
        return chest;
    };
    if record.schema != CHEST_SCHEMA {
        return chest;
    }
    let Ok(texts) = ron::from_str::<Vec<String>>(&record.body) else {
        return chest;
    };
    for (slot, text) in chest.slots.iter_mut().zip(texts.iter().take(CHEST)) {
        *slot = stack_of(text).flatten();
    }
    chest
}

/// The record a chest is stored as.
pub fn record(cell: u32, layer: u16, chest: &Hold) -> StoredRecord {
    let texts: Vec<String> = chest.slots.iter().map(|s| stack_text(*s)).collect();
    StoredRecord::of(CHEST_KIND, record_id(cell, layer), CHEST_SCHEMA, &texts)
}

/// Write the chest and the slots after a move, in one line. `false`, and
/// nothing kept, once the writer has failed.
pub fn store(save: &mut WorldSave, cell: u32, layer: u16, chest: &Hold, slots: &Slots) -> bool {
    save.store_with_hands(record(cell, layer, chest), CHEST_SCHEMA, slots)
}

/// Spill the chest at `cell` and `layer` as it is broken (design 4): each
/// stored stack a drop at `centre` (the body's frame, f64), made at world
/// time `now_s`, each written with `slots` unchanged; then the record written
/// empty. The drops written are returned for the live list. `None`, and the
/// chest left as it was, if the save refused the first line; an empty chest
/// writes nothing.
pub fn spill(
    save: &mut WorldSave,
    cell: u32,
    layer: u16,
    centre: DVec3,
    now_s: f64,
    slots: &Slots,
) -> Option<Vec<ItemDrop>> {
    let chest = read(save, cell, layer);
    if chest.is_empty() {
        return Some(Vec::new());
    }
    let mut dropped = Vec::new();
    for stack in chest.slots.iter().flatten() {
        let mut drop = Drops::make(save, stack.item, centre, now_s);
        drop.count = stack.count;
        if !save.record_toss(&drop, slots) {
            // What was written already floats; the rest stays in the record,
            // which is not cleared, so nothing is lost.
            return (!dropped.is_empty()).then_some(dropped);
        }
        dropped.push(drop);
    }
    if !store(save, cell, layer, &empty(), slots) {
        return Some(dropped);
    }
    Some(dropped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::goods::Good;
    use pbd_core::inventory::{Item, Stack};
    use pbd_core::terrain::Material;
    use pbd_core::vehicle::hold::{Place, send, shift};

    /// Stacks moved into a chest are in it and out of the slots, in the save
    /// as in memory; a chest at another cell or layer is its own.
    #[test]
    fn a_chest_is_stored_with_the_slots() {
        let mut save = WorldSave::memory_only();
        let mut slots = Slots::new();
        slots.set(14, Some(Stack::new(Item::Block(Material::Stone), 99)));
        slots.set(15, Some(Stack::new(Item::Good(Good::LOG), 12)));
        let mut chest = read(&save, 700, 41);
        assert_eq!(chest.slots.len(), CHEST);
        assert!(chest.is_empty());
        shift(
            &mut slots,
            &mut chest,
            Place::Carried(14),
            Place::Hold(19),
            99,
        );
        send(&mut slots, &mut chest, Place::Carried(15));
        assert!(store(&mut save, 700, 41, &chest, &slots));
        let back = read(&save, 700, 41);
        assert_eq!(
            back.get(19),
            Some(Stack::new(Item::Block(Material::Stone), 99))
        );
        assert_eq!(back.get(0), Some(Stack::new(Item::Good(Good::LOG), 12)));
        assert_eq!(save.carried.as_ref().and_then(|c| c.get(14)), None);
        assert_eq!(save.carried.as_ref().and_then(|c| c.get(15)), None);
        assert!(
            read(&save, 700, 42).is_empty(),
            "another layer's is its own"
        );
        assert!(read(&save, 701, 41).is_empty(), "another cell's is its own");
    }

    /// On the disk: a stack put in a chest is in it, and out of the slots,
    /// when the world is opened again (`storage-chests` spec, "A stack
    /// stored and kept").
    #[test]
    fn a_chest_survives_a_reload() {
        let root = std::env::temp_dir().join(format!("pbd-chest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Chest", 78).unwrap();
        let sand = Item::Block(Material::Sand);
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let mut slots = Slots::new();
            slots.set(11, Some(Stack::new(sand, 99)));
            let mut chest = read(&save, 1234, 60);
            shift(
                &mut slots,
                &mut chest,
                Place::Carried(11),
                Place::Hold(7),
                99,
            );
            assert!(store(&mut save, 1234, 60, &chest, &slots));
            save.drain();
        }
        let listed = crate::saves::list(&root);
        let reopened = WorldSave::open(root.clone(), listed[0].clone());
        assert_eq!(read(&reopened, 1234, 60).get(7), Some(Stack::new(sand, 99)));
        assert_eq!(reopened.carried.as_ref().map(|c| c.count(sand)), Some(0));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Breaking a chest drops each stack it holds as a drop of its whole
    /// count, and leaves its record empty; the drops are in the save before
    /// the record is cleared, and are there when the world is opened again
    /// (`storage-chests` spec, "A broken chest drops its contents").
    #[test]
    fn a_spilled_chest_drops_every_stack_and_is_left_empty() {
        let root = std::env::temp_dir().join(format!("pbd-chest-spill-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Spill", 79).unwrap();
        let log = Item::Good(Good::LOG);
        let stone = Item::Block(Material::Stone);
        let centre = DVec3::new(0.0, 1000.5, 0.0);
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            let slots = Slots::new();
            let mut chest = empty();
            chest.slots[2] = Some(Stack::new(log, 30));
            chest.slots[17] = Some(Stack::new(stone, 99));
            assert!(store(&mut save, 55, 9, &chest, &slots));
            let dropped = spill(&mut save, 55, 9, centre, 10.0, &slots).expect("spilled");
            assert_eq!(dropped.len(), 2);
            assert_eq!((dropped[0].item, dropped[0].count), (log, 30));
            assert_eq!((dropped[1].item, dropped[1].count), (stone, 99));
            assert_ne!(dropped[0].id, dropped[1].id);
            assert!(
                dropped.iter().all(|d| (d.position - centre).length() < 1.0),
                "at the chest"
            );
            assert!(read(&save, 55, 9).is_empty(), "the record is cleared");
            // The next drop (the chest's own, on the dig's line) takes a
            // fresh id.
            assert!(save.next_drop_id() > dropped[1].id);
            // An empty chest spills nothing.
            assert_eq!(spill(&mut save, 55, 9, centre, 11.0, &slots), Some(vec![]));
            save.drain();
        }
        let listed = crate::saves::list(&root);
        let reopened = WorldSave::open(root.clone(), listed[0].clone());
        assert!(read(&reopened, 55, 9).is_empty());
        let counts: Vec<_> = reopened.drops.iter().map(|d| (d.item, d.count)).collect();
        assert!(counts.contains(&(log, 30)), "{counts:?}");
        assert!(counts.contains(&(stone, 99)), "{counts:?}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
