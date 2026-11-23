//! A raft's hold (`small-boats` design 5): ten slots carried on the craft,
//! filled and emptied from the player's slots with the pack's own clicks.
//!
//! The hold is stored as a record of its own (kind [`HOLD_KIND`], the craft's
//! id), written in one line with the player's slots, so a stack moved between
//! the two is never half saved. What is here is the rule for a move; the app
//! writes the line.

use crate::inventory::{CARRIED, Slots, Stack};

/// Slots in a boat's hold (the raft's).
pub const HOLD: usize = 10;

/// The most slots any vehicle's hold has (a horse cart's, `land-transport`
/// design 10): what the HOLD panel can show.
pub const HOLD_MAX: usize = 24;

/// The record kind a hold is stored as, and its schema.
pub const HOLD_KIND: &str = "hold";
pub const HOLD_SCHEMA: u32 = 1;

/// A vehicle's hold: as many slots as its vehicle gives it (a raft's ten,
/// a hand cart's nine, a horse cart's 24).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hold {
    pub slots: Vec<Option<Stack>>,
}

impl Default for Hold {
    /// A boat's hold: [`HOLD`] slots.
    fn default() -> Self {
        Self::sized(HOLD)
    }
}

/// A slot a move names: one of the player's (hotbar and pack, as
/// [`Slots`] numbers them) or one of the hold's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Carried(usize),
    Hold(usize),
}

impl Hold {
    /// An empty hold of `slots` slots, at most [`HOLD_MAX`].
    pub fn sized(slots: usize) -> Self {
        Self {
            slots: vec![None; slots.min(HOLD_MAX)],
        }
    }

    /// Slots holding something.
    pub fn filled(&self) -> usize {
        self.slots.iter().flatten().count()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    pub fn get(&self, index: usize) -> Option<Stack> {
        self.slots.get(index).copied().flatten()
    }
}

fn get(slots: &Slots, hold: &Hold, at: Place) -> Option<Stack> {
    match at {
        Place::Carried(i) => slots.get(i),
        Place::Hold(i) => hold.get(i),
    }
}

fn set(slots: &mut Slots, hold: &mut Hold, at: Place, stack: Option<Stack>) {
    let stack = stack.filter(|s| s.count > 0);
    match at {
        Place::Carried(i) => slots.set(i, stack),
        Place::Hold(i) => {
            if let Some(slot) = hold.slots.get_mut(i) {
                *slot = stack;
            }
        }
    }
}

fn valid(hold: &Hold, at: Place) -> bool {
    match at {
        Place::Carried(i) => i < CARRIED,
        Place::Hold(i) => i < hold.slots.len(),
    }
}

/// Move `count` of the stack at `from` onto `to`, as the pack moves a stack
/// between two of its own slots (`Slots::shift`): onto an empty slot or the
/// same item it goes (as many as fit, the rest staying where it was); onto a
/// different item the two swap, if the whole stack was moved; otherwise
/// nothing happens. Nothing is ever lost or made.
pub fn shift(slots: &mut Slots, hold: &mut Hold, from: Place, to: Place, count: u16) {
    if from == to || !valid(hold, from) || !valid(hold, to) {
        return;
    }
    let Some(stack) = get(slots, hold, from) else {
        return;
    };
    let count = count.min(stack.count);
    if count == 0 {
        return;
    }
    match get(slots, hold, to) {
        Some(there) if there.item != stack.item => {
            if count == stack.count {
                set(slots, hold, from, Some(there));
                set(slots, hold, to, Some(stack));
            }
        }
        there => {
            let have = there.map_or(0, |t| t.count);
            let room = stack.item.stack_limit().saturating_sub(have);
            let moved = count.min(room);
            if moved == 0 {
                return;
            }
            set(slots, hold, to, Some(Stack::new(stack.item, have + moved)));
            set(
                slots,
                hold,
                from,
                Some(Stack::new(stack.item, stack.count - moved)),
            );
        }
    }
}

/// Shift-click: send the whole stack at `from` to the other side - from the
/// hold into the player's slots (pack first, then hotbar, merging first), or
/// from the player's slots into the hold. As much as fits goes.
pub fn send(slots: &mut Slots, hold: &mut Hold, from: Place) {
    let Some(stack) = get(slots, hold, from) else {
        return;
    };
    let targets: Vec<Place> = match from {
        Place::Hold(_) => (crate::inventory::SLOTS..CARRIED)
            .chain(0..crate::inventory::SLOTS)
            .map(Place::Carried)
            .collect(),
        Place::Carried(_) => (0..hold.slots.len()).map(Place::Hold).collect(),
    };
    // Onto the same item first, then into empty slots.
    for merge in [true, false] {
        for &to in &targets {
            let Some(left) = get(slots, hold, from) else {
                return;
            };
            let there = get(slots, hold, to);
            let fits = match there {
                Some(t) => merge && t.item == stack.item,
                None => !merge,
            };
            if fits {
                shift(slots, hold, from, to, left.count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goods::Good;
    use crate::inventory::Item;

    fn logs(n: u16) -> Stack {
        Stack::new(Item::Good(Good::LOG), n)
    }

    fn total(slots: &Slots, hold: &Hold) -> u32 {
        let item = Item::Good(Good::LOG);
        slots.count(item)
            + hold
                .slots
                .iter()
                .flatten()
                .filter(|s| s.item == item)
                .map(|s| u32::from(s.count))
                .sum::<u32>()
    }

    /// Stacks go into the hold and back, merge, and swap, and nothing is
    /// ever made or lost.
    #[test]
    fn moves_between_the_slots_and_the_hold_keep_every_item() {
        let mut slots = Slots::new();
        slots.set(12, Some(logs(40)));
        slots.set(13, Some(Stack::new(Item::Good(Good::ROPE), 5)));
        let mut hold = Hold::default();
        shift(
            &mut slots,
            &mut hold,
            Place::Carried(12),
            Place::Hold(0),
            40,
        );
        assert_eq!(hold.get(0), Some(logs(40)));
        assert_eq!(slots.get(12), None);
        // Half back.
        shift(
            &mut slots,
            &mut hold,
            Place::Hold(0),
            Place::Carried(20),
            20,
        );
        assert_eq!(hold.get(0), Some(logs(20)));
        assert_eq!(slots.get(20), Some(logs(20)));
        // A merge that overfills keeps the rest where it was.
        slots.set(21, Some(logs(90)));
        shift(
            &mut slots,
            &mut hold,
            Place::Carried(21),
            Place::Hold(0),
            90,
        );
        assert_eq!(hold.get(0), Some(logs(99)));
        assert_eq!(slots.get(21), Some(logs(11)));
        assert_eq!(total(&slots, &hold), 130);
        // A whole stack onto another item swaps.
        shift(&mut slots, &mut hold, Place::Carried(13), Place::Hold(0), 5);
        assert_eq!(hold.get(0).map(|s| s.item), Some(Item::Good(Good::ROPE)));
        assert_eq!(slots.get(13), Some(logs(99)));
        assert_eq!(total(&slots, &hold), 130);
        assert!(!hold.is_empty());
    }

    /// Shift-click sends a stack across, merging first.
    #[test]
    fn a_shift_click_sends_a_stack_across() {
        let mut slots = Slots::new();
        slots.set(15, Some(logs(30)));
        let mut hold = Hold::default();
        hold.slots[3] = Some(logs(80));
        send(&mut slots, &mut hold, Place::Carried(15));
        assert_eq!(hold.get(3), Some(logs(99)));
        assert_eq!(hold.get(0), Some(logs(11)));
        assert_eq!(slots.get(15), None);
        send(&mut slots, &mut hold, Place::Hold(3));
        assert_eq!(hold.get(3), None);
        assert_eq!(total(&slots, &hold), 110);
    }
}
