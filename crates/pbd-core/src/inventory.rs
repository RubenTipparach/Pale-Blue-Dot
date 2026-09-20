//! What the player carries: ten slots, each holding a stack of one item kind.
//!
//! It lives in the core because what a slot holds, how a stack merges and when
//! a give is refused are rules a future multiplayer has to agree about, and
//! they depend on nothing but the material enum beside them. Drawing a slot is
//! the app's business; deciding what is in it is not.

use crate::terrain::Material;

/// How many slots the player carries. Ten, selected with the number row.
pub const SLOTS: usize = 10;

/// One thing a slot can hold.
///
/// `Tool` exists because the slots were asked for to hold "blocks and
/// equipment", and a store that can only hold one of those would have to be
/// widened later by everything that touches it. No tool is CONSTRUCTED yet: a
/// pickaxe that cannot dig is a control for a mechanic that does not exist, and
/// digging waits on the volumetric columns. The first real tool lands with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Item {
    /// A block of terrain material, placeable once there is a block to place.
    Block(Material),
    /// A piece of equipment.
    Tool(Tool),
}

/// Equipment kinds. Empty of anything usable until digging lands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tool {
    /// Breaks blocks. Placeholder: nothing constructs one until the columns
    /// change gives it something to break.
    Pick,
}

impl Item {
    /// How many of this item one slot holds.
    pub fn stack_limit(self) -> u16 {
        match self {
            // The reference's own block stack. A tool is a single thing.
            Item::Block(_) => 99,
            Item::Tool(_) => 1,
        }
    }

    /// A short name, for a tooltip or a log. NOT for the slot itself: a slot
    /// draws the item's thumbnail, because an inventory drawn as a list of
    /// names is the failure the UI rule exists to prevent.
    pub fn name(self) -> &'static str {
        match self {
            Item::Block(material) => material_name(material),
            Item::Tool(Tool::Pick) => "pick",
        }
    }
}

fn material_name(material: Material) -> &'static str {
    match material {
        Material::Air => "air",
        Material::Stone => "stone",
        Material::Soil => "soil",
        Material::Grass => "grass",
        Material::Water => "water",
        Material::Ore => "ore",
        Material::Sand => "sand",
        Material::DryGrass => "dry grass",
        Material::JungleGrass => "jungle grass",
        Material::Snow => "snow",
        Material::Rock => "rock",
        Material::Dirt => "dirt",
    }
}

/// A count of one item kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stack {
    pub item: Item,
    pub count: u16,
}

impl Stack {
    pub fn new(item: Item, count: u16) -> Self {
        Self {
            item,
            count: count.min(item.stack_limit()),
        }
    }

    /// Room left before this stack is full.
    pub fn room(&self) -> u16 {
        self.item.stack_limit().saturating_sub(self.count)
    }
}

/// The ten slots and which one is selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slots {
    slots: [Option<Stack>; SLOTS],
    selected: usize,
}

impl Default for Slots {
    fn default() -> Self {
        Self {
            slots: [None; SLOTS],
            selected: 0,
        }
    }
}

impl Slots {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, index: usize) -> Option<Stack> {
        self.slots.get(index).copied().flatten()
    }

    pub fn iter(&self) -> impl Iterator<Item = Option<Stack>> + '_ {
        self.slots.iter().copied()
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    /// What the selected slot holds, if anything.
    pub fn held(&self) -> Option<Stack> {
        self.get(self.selected)
    }

    /// Select a slot by index. Out of range is ignored rather than clamped: a
    /// key that does not name a slot should do nothing, not move the selection
    /// somewhere the player did not ask for.
    pub fn select(&mut self, index: usize) {
        if index < SLOTS {
            self.selected = index;
        }
    }

    /// Step the selection, wrapping both ways.
    pub fn step(&mut self, by: i32) {
        let n = SLOTS as i32;
        self.selected = (((self.selected as i32 + by) % n + n) % n) as usize;
    }

    /// Put items in: matching stacks with room first, then the first empty
    /// slot. Returns how many did not fit, so a caller that cannot drop the
    /// remainder on the ground knows not to take it off the world.
    pub fn give(&mut self, item: Item, mut count: u16) -> u16 {
        for slot in self.slots.iter_mut() {
            if count == 0 {
                return 0;
            }
            if let Some(stack) = slot
                && stack.item == item
                && stack.room() > 0
            {
                let moved = count.min(stack.room());
                stack.count += moved;
                count -= moved;
            }
        }
        for slot in self.slots.iter_mut() {
            if count == 0 {
                return 0;
            }
            if slot.is_none() {
                let moved = count.min(item.stack_limit());
                *slot = Some(Stack::new(item, moved));
                count -= moved;
            }
        }
        count
    }

    /// Take up to `count` from one slot. Returns how many came out; the slot
    /// holds NOTHING rather than a stack of zero when it empties.
    pub fn take(&mut self, index: usize, count: u16) -> u16 {
        let Some(slot) = self.slots.get_mut(index) else {
            return 0;
        };
        let Some(stack) = slot else { return 0 };
        let moved = count.min(stack.count);
        stack.count -= moved;
        if stack.count == 0 {
            *slot = None;
        }
        moved
    }

    /// Take one from the selected slot: what placing a block will ask for.
    pub fn take_held(&mut self) -> Option<Item> {
        let item = self.held()?.item;
        (self.take(self.selected, 1) == 1).then_some(item)
    }

    /// Put exactly this in exactly this slot, which is what LOADING a save
    /// needs and what `give` cannot express: `give` merges and spills, because
    /// that is what picking something up does, and a restore is not a pickup.
    ///
    /// A count over the item's own limit is clamped rather than refused, so a
    /// save damaged into an impossible stack loads as a legal one instead of
    /// taking the whole world with it. A count of zero is an empty slot, since
    /// nothing here ever holds a stack of nothing.
    pub fn set(&mut self, index: usize, stack: Option<Stack>) {
        let Some(slot) = self.slots.get_mut(index) else {
            return;
        };
        *slot = match stack {
            Some(stack) if stack.count > 0 => Some(Stack {
                item: stack.item,
                count: stack.count.min(stack.item.stack_limit()),
            }),
            _ => None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A restore puts a stack back exactly, which is the one thing `give`
    /// deliberately cannot do: it merges and spills, because that is what
    /// picking something up does.
    #[test]
    fn a_slot_can_be_set_exactly_and_an_impossible_stack_is_made_legal() {
        let mut slots = Slots::new();
        slots.set(3, Some(Stack::new(Item::Block(Material::Stone), 7)));
        assert_eq!(
            slots.get(3),
            Some(Stack::new(Item::Block(Material::Stone), 7))
        );
        let limit = Item::Block(Material::Stone).stack_limit();
        slots.set(
            4,
            Some(Stack::new(Item::Block(Material::Stone), limit + 50)),
        );
        assert_eq!(
            slots.get(4).map(|s| s.count),
            Some(limit),
            "a damaged save loads as a legal stack rather than an illegal one"
        );
        slots.set(3, None);
        assert!(slots.get(3).is_none(), "and a slot can be emptied");
        slots.set(5, Some(Stack::new(Item::Block(Material::Dirt), 0)));
        assert!(slots.get(5).is_none(), "nothing holds a stack of nothing");
        slots.set(999, Some(Stack::new(Item::Block(Material::Dirt), 1)));
    }

    const DIRT: Item = Item::Block(Material::Dirt);
    const STONE: Item = Item::Block(Material::Stone);

    #[test]
    fn a_give_fills_a_matching_stack_before_taking_a_new_slot() {
        let mut slots = Slots::new();
        assert_eq!(slots.give(DIRT, 10), 0);
        assert_eq!(slots.give(DIRT, 5), 0);
        assert_eq!(slots.get(0), Some(Stack::new(DIRT, 15)));
        assert_eq!(slots.get(1), None, "one kind, one stack, while it has room");
    }

    #[test]
    fn a_full_stack_spills_into_the_next_slot() {
        let mut slots = Slots::new();
        let limit = DIRT.stack_limit();
        assert_eq!(slots.give(DIRT, limit), 0);
        assert_eq!(slots.give(DIRT, 3), 0);
        assert_eq!(slots.get(0).unwrap().count, limit);
        assert_eq!(slots.get(1), Some(Stack::new(DIRT, 3)));
    }

    #[test]
    fn a_full_store_refuses_the_remainder_rather_than_dropping_it() {
        let mut slots = Slots::new();
        let limit = DIRT.stack_limit();
        for _ in 0..SLOTS {
            slots.give(DIRT, limit);
        }
        // Every slot is a full stack, so nothing fits and the caller is told
        // how much. A give that swallowed the remainder would delete material
        // the world had already given up.
        assert_eq!(slots.give(DIRT, 7), 7);
        assert_eq!(slots.give(STONE, 2), 2);
    }

    #[test]
    fn taking_the_last_one_empties_the_slot() {
        let mut slots = Slots::new();
        slots.give(STONE, 2);
        assert_eq!(slots.take(0, 1), 1);
        assert_eq!(slots.get(0), Some(Stack::new(STONE, 1)));
        assert_eq!(slots.take(0, 1), 1);
        assert_eq!(
            slots.get(0),
            None,
            "an empty slot is None, not a zero stack"
        );
        assert_eq!(slots.take(0, 1), 0, "and taking from nothing takes nothing");
    }

    #[test]
    fn a_take_never_gives_more_than_the_slot_holds() {
        let mut slots = Slots::new();
        slots.give(DIRT, 4);
        assert_eq!(slots.take(0, 99), 4);
        assert_eq!(slots.get(0), None);
    }

    #[test]
    fn the_selection_wraps_both_ways() {
        let mut slots = Slots::new();
        assert_eq!(slots.selected(), 0);
        slots.step(-1);
        assert_eq!(slots.selected(), SLOTS - 1, "stepping back from the first");
        slots.step(1);
        assert_eq!(slots.selected(), 0, "and forward from the last");
        slots.step(SLOTS as i32 + 3);
        assert_eq!(
            slots.selected(),
            3,
            "a long step is still one lap plus three"
        );
    }

    #[test]
    fn selecting_a_slot_that_is_not_there_does_nothing() {
        let mut slots = Slots::new();
        slots.select(4);
        slots.select(SLOTS);
        assert_eq!(slots.selected(), 4, "an out-of-range key must not move it");
    }

    #[test]
    fn the_held_item_is_what_the_selected_slot_holds() {
        let mut slots = Slots::new();
        slots.give(DIRT, 2);
        slots.select(3);
        assert_eq!(slots.held(), None);
        slots.select(0);
        assert_eq!(slots.held(), Some(Stack::new(DIRT, 2)));
        assert_eq!(slots.take_held(), Some(DIRT));
        assert_eq!(slots.held().unwrap().count, 1);
    }

    #[test]
    fn a_tool_takes_a_whole_slot_each() {
        let pick = Item::Tool(Tool::Pick);
        assert_eq!(pick.stack_limit(), 1);
        let mut slots = Slots::new();
        // Three picks take three slots, one each: they do not stack, and there
        // are ten slots, so all three fit. The first version of this test
        // asserted two of them had nowhere to go, which was the test being
        // wrong about how much room an empty store has rather than the store
        // being wrong about stacking.
        assert_eq!(slots.give(pick, 3), 0);
        for index in 0..3 {
            assert_eq!(slots.get(index).unwrap().count, 1);
        }
        assert_eq!(slots.get(3), None);
    }

    #[test]
    fn tools_run_out_of_slots_where_blocks_would_not() {
        let pick = Item::Tool(Tool::Pick);
        let mut slots = Slots::new();
        // One slot each means the store holds exactly SLOTS of them, against
        // 99 * SLOTS blocks. Eleven is one too many.
        assert_eq!(slots.give(pick, SLOTS as u16), 0);
        assert_eq!(slots.give(pick, 1), 1);
    }
}
