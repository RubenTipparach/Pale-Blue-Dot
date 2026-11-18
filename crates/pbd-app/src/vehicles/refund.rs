//! What the player made of a retired boat comes back to them
//! (`retire-rowboat-and-cog` decisions 2 and 3; CLAUDE.md, "Nothing a
//! player made is dropped").
//!
//! - A held record of a retired kind the player made (`made: true`) is
//!   refunded its recipe's inputs as they shipped
//!   (`station::RETIRED_RECIPES`) and whatever its hold held, all or
//!   nothing, in ONE line with the player's slots and a `refund` record
//!   naming the craft. The record is what says it was refunded, so it is
//!   never refunded twice; the held craft record itself is never touched.
//! - A good for a retired boat in the slots is swapped for its recipe's
//!   inputs, all or nothing, and the slots written: the good is its own
//!   token, so it is swapped once.
//!
//! Neither drops anything in the world: a drop lasts five minutes and may
//! lie far from the player. While the slots cannot take it all, nothing
//! changes, and it is tried again a second later.

use super::Fleet;
use crate::hotbar::Hotbar;
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::inventory::{Item, Slots, Stack};
use pbd_core::records::Record as StoredRecord;
use pbd_core::station::{give_all, retired_inputs};
use pbd_core::vehicle::record::VehicleRecord;

/// The record kind that marks a retired craft refunded, keyed by its ID.
pub const REFUND_KIND: &str = "refund";
pub const REFUND_SCHEMA: u32 = 1;

/// How often an owed refund is tried, s.
const EVERY_S: f32 = 1.0;

/// What a held retired record is owed: its recipe's inputs if the player
/// made it, and every stack in its hold. Empty when nothing is owed: it was
/// not made and holds nothing, or it was refunded already.
pub fn owed(save: &WorldSave, record: &VehicleRecord) -> Vec<(Item, u16)> {
    if save.records.get(REFUND_KIND, record.id).is_some() {
        return Vec::new();
    }
    let mut items = Vec::new();
    if record.made {
        items.extend(retired_inputs(&record.kind).unwrap_or_default());
    }
    let hold = super::hold::read(save, record.id);
    items.extend(hold.slots.iter().flatten().map(|s| (s.item, s.count)));
    items
}

/// Refund one held record into `slots` if all of it fits, writing the
/// slots and its `refund` record in one line. Whether it was refunded.
pub fn refund_one(save: &mut WorldSave, slots: &mut Slots, record: &VehicleRecord) -> bool {
    let items = owed(save, record);
    if items.is_empty() {
        return false;
    }
    let mut after = slots.clone();
    if !give_all(&mut after, &items) {
        return false;
    }
    let given: Vec<String> = items
        .iter()
        .map(|&(item, n)| crate::saves::format::stack_text(Some(Stack::new(item, n))))
        .collect();
    let mark = StoredRecord::of(REFUND_KIND, record.id, REFUND_SCHEMA, &given);
    if !save.store_with_hands(mark, REFUND_SCHEMA, &after) {
        return false;
    }
    *slots = after;
    true
}

/// Swap every retired boat good in `slots` for its recipe's inputs, all or
/// nothing per stack. Whether anything was swapped.
pub fn swap_goods(slots: &mut Slots) -> bool {
    let mut swapped = false;
    for index in 0..pbd_core::inventory::CARRIED {
        let Some(Stack {
            item: Item::Good(good),
            count,
        }) = slots.get(index)
        else {
            continue;
        };
        let Some(inputs) = retired_inputs(good.code()) else {
            continue;
        };
        let mut after = slots.clone();
        after.take(index, count);
        let all: Vec<(Item, u16)> = inputs.iter().map(|&(i, n)| (i, n * count)).collect();
        if give_all(&mut after, &all) {
            *slots = after;
            swapped = true;
        }
    }
    swapped
}

/// Once a second while a world is open: refund what is owed, and swap any
/// retired boat good the slots hold. Only while the slots are the save's
/// (nothing on the pointer, nothing unwritten), so a refund never races a
/// move.
pub fn refund_retired(
    time: Res<Time>,
    mut due: Local<f32>,
    fleet: Option<Res<Fleet>>,
    save: Option<ResMut<WorldSave>>,
    hotbar: Option<ResMut<Hotbar>>,
) {
    *due -= time.delta_secs();
    if *due > 0.0 {
        return;
    }
    let (Some(fleet), Some(mut save), Some(mut hotbar)) = (fleet, save, hotbar) else {
        return;
    };
    if !fleet.spawned || save.carried.as_ref() != Some(&hotbar.0) {
        return;
    }
    *due = EVERY_S;
    let mut slots = hotbar.0.clone();
    for record in &fleet.retired {
        if refund_one(&mut save, &mut slots, record) {
            info!(
                "craft {}, a retired {} the player made: refunded {:?}",
                record.id,
                record.kind,
                pbd_core::station::retired_inputs(&record.kind).unwrap_or_default()
            );
        } else if !owed(&save, record).is_empty() {
            debug!(
                "craft {}, a retired {}: its refund waits for room",
                record.id, record.kind
            );
        }
    }
    let refunded = slots.clone();
    if swap_goods(&mut slots) {
        if save.record_pack(&slots) {
            info!("a retired boat good swapped for its materials");
        } else {
            slots = refunded;
        }
    }
    if slots != hotbar.0 {
        hotbar.0 = slots;
    }
}
