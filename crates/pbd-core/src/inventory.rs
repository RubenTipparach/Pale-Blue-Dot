//! What the player carries: ten hotbar slots and a pack of thirty more, each
//! holding a stack of one item kind.
//!
//! It lives in the core because what a slot holds, how a stack merges and when
//! a give is refused are rules a future multiplayer has to agree about, and
//! they depend on nothing but the material enum beside them. Drawing a slot is
//! the app's business; deciding what is in it is not.

use crate::terrain::Material;

/// How many hotbar slots the player carries. Ten, selected with the number
/// row. They are the first ten of [`CARRIED`].
pub const SLOTS: usize = 10;

/// The pack's slots, three rows of ten under the hotbar (`inventory-grid`,
/// survey I2). They follow the hotbar's ten.
pub const PACK: usize = 30;

/// Every slot the player carries: the hotbar, then the pack.
pub const CARRIED: usize = SLOTS + PACK;

/// One thing a slot can hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Item {
    /// A block of terrain material, placeable once there is a block to place.
    Block(Material),
    /// A piece of equipment. The tools a player works with ride the tool slot
    /// (`Equipment`) rather than these ten; a tool as an ITEM is what a chest
    /// or a drop would hold, and the variant is kept so neither has to widen
    /// everything that touches a slot.
    Tool(Tool),
    /// A caught fish, by its species' index in the body's roster
    /// (`fauna::Roster`). An index rather than a name because a slot is `Copy`
    /// and saved as a short code; the roster is append-only for that reason.
    Fish(u16),
}

/// The four tools, in the order the picker lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tool {
    /// Casts, hooks and reels. Breaks nothing.
    Rod,
    /// Digs soil, sand and snow.
    Shovel,
    /// Breaks stone, rock and ore.
    Pickaxe,
    /// Fells trees.
    Axe,
}

impl Tool {
    /// Every tool, in the picker's order.
    pub const ALL: [Tool; 4] = [Tool::Rod, Tool::Shovel, Tool::Pickaxe, Tool::Axe];

    /// Where in `ALL` this tool stands.
    pub fn index(self) -> usize {
        Tool::ALL
            .iter()
            .position(|t| *t == self)
            .expect("every tool is in ALL")
    }

    pub fn name(self) -> &'static str {
        match self {
            Tool::Rod => "Fishing rod",
            Tool::Shovel => "Shovel",
            Tool::Pickaxe => "Pickaxe",
            Tool::Axe => "Axe",
        }
    }

    /// What it is for, in the picker's second line.
    pub fn purpose(self) -> &'static str {
        match self {
            Tool::Rod => "cast, hook, reel",
            Tool::Shovel => "digs soil, sand, snow",
            Tool::Pickaxe => "breaks stone, rock, ore",
            Tool::Axe => "chops wood; fair on turf",
        }
    }

    /// Whether the tool breaks blocks at all. A rod does not: a player
    /// holding it and clicking means to cast, never to dig.
    pub fn digs(self) -> bool {
        !matches!(self, Tool::Rod)
    }
}

/// The tool slot: which tools the player owns and which one, if any, is in
/// hand.
///
/// Its own store beside the ten slots, because a tool is not a stack and a
/// tool held in a slot would compete with blocks and fish for room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Equipment {
    owned: [bool; 4],
    /// `None` is bare hands (`inventory-grid` decision 8).
    held: Option<Tool>,
}

impl Default for Equipment {
    /// A new world's kit: all four owned, nothing in hand.
    fn default() -> Self {
        Self {
            owned: [true; 4],
            held: None,
        }
    }
}

impl Equipment {
    /// Rebuild from a save's record; a held tool that is not owned leaves the
    /// hands bare rather than handing the player something they do not have.
    pub fn from_parts(owned: [bool; 4], held: Option<Tool>) -> Self {
        let mut equipment = Self { owned, held };
        if held.is_some_and(|t| !equipment.owns(t)) {
            equipment.held = None;
        }
        equipment
    }

    /// The tool in hand, or `None` for bare hands.
    pub fn held(&self) -> Option<Tool> {
        self.held
    }

    /// The tool the left button digs with: the one in hand, if it digs.
    /// Bare hands break nothing, as the rod breaks nothing.
    pub fn digging_tool(&self) -> Option<Tool> {
        self.held.filter(|t| t.digs())
    }

    pub fn owned(&self) -> [bool; 4] {
        self.owned
    }

    pub fn owns(&self, tool: Tool) -> bool {
        self.owned[tool.index()]
    }

    /// The owned tools, in order.
    pub fn tools(&self) -> Vec<Tool> {
        Tool::ALL.into_iter().filter(|t| self.owns(*t)).collect()
    }

    /// Put a tool in hand, or empty the hands with `None`. Refused, and
    /// `false`, when the tool is not owned or is already held: only a real
    /// change is a change worth saving.
    pub fn hold(&mut self, tool: impl Into<Option<Tool>>) -> bool {
        let tool = tool.into();
        if tool.is_some_and(|t| !self.owns(t)) || self.held == tool {
            return false;
        }
        self.held = tool;
        true
    }

    /// The entry `by` steps from `from` in the picker's list, bare hands and
    /// then the owned tools, wrapping: what the picker's wheel moves through.
    pub fn step_from(&self, from: Option<Tool>, by: i32) -> Option<Tool> {
        let entries: Vec<Option<Tool>> = std::iter::once(None)
            .chain(self.tools().into_iter().map(Some))
            .collect();
        let at = entries.iter().position(|t| *t == from).unwrap_or(0) as i32;
        let n = entries.len() as i32;
        entries[(((at + by) % n + n) % n) as usize]
    }
}

impl Item {
    /// How many of this item one slot holds.
    pub fn stack_limit(self) -> u16 {
        match self {
            // The reference's own block stack. A tool is a single thing.
            Item::Block(_) => 99,
            Item::Tool(_) => 1,
            // A creel's worth: a fish is a thing you carry a few of.
            Item::Fish(_) => 16,
        }
    }

    /// A short name, for a tooltip or a log. NOT for the slot itself: a slot
    /// draws the item's thumbnail, because an inventory drawn as a list of
    /// names is the failure the UI rule exists to prevent.
    pub fn name(self) -> &'static str {
        match self {
            Item::Block(material) => material_name(material),
            Item::Tool(tool) => tool.name(),
            // The species' own name is the roster's; the item alone only
            // knows it is a fish.
            Item::Fish(_) => "fish",
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
        Material::Torch => "torch",
        Material::LanternPost => "street lantern",
        Material::LanternWall => "wall lantern",
        Material::LanternHanging => "hanging lantern",
        Material::Brazier => "brazier",
        Material::Candle => "candle",
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

/// The hotbar and the pack, and which hotbar slot is selected. Index 0..10 is
/// the hotbar and 10..40 the pack, one array, so the order a give fills them
/// in is the index order (`inventory-grid` decision 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slots {
    slots: [Option<Stack>; CARRIED],
    selected: usize,
}

impl Default for Slots {
    fn default() -> Self {
        Self {
            slots: [None; CARRIED],
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

    /// Every slot, the hotbar's then the pack's.
    pub fn iter(&self) -> impl Iterator<Item = Option<Stack>> + '_ {
        self.slots.iter().copied()
    }

    /// The hotbar's ten slots.
    pub fn hotbar(&self) -> impl Iterator<Item = Option<Stack>> + '_ {
        self.slots[..SLOTS].iter().copied()
    }

    /// Whether a slot index is a hotbar slot, rather than a pack slot.
    pub fn is_hotbar(index: usize) -> bool {
        index < SLOTS
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

    /// Put items in: matching stacks with room first, the hotbar's before the
    /// pack's, then the first empty hotbar slot, then the first empty pack
    /// slot. Returns how many did not fit, which a dig leaves floating in the
    /// world (`inventory-grid` decision 4).
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

    /// Put a stack into a slot, as a click with a stack on the pointer does:
    /// into an empty slot whole; onto the same item as far as it has room;
    /// and in place of anything else, which comes back to the pointer. Returns
    /// what the pointer holds afterwards. Nothing is ever lost: whatever does
    /// not go in comes back.
    pub fn put(&mut self, index: usize, stack: Stack) -> Option<Stack> {
        let Some(slot) = self.slots.get_mut(index) else {
            return Some(stack);
        };
        match slot {
            None => {
                *slot = Some(Stack::new(stack.item, stack.count));
                (stack.count > stack.item.stack_limit())
                    .then(|| Stack::new(stack.item, stack.count - stack.item.stack_limit()))
            }
            Some(here) if here.item == stack.item => {
                let moved = stack.count.min(here.room());
                here.count += moved;
                (stack.count > moved).then(|| Stack::new(stack.item, stack.count - moved))
            }
            Some(here) => Some(std::mem::replace(here, stack)),
        }
    }

    /// Take a slot's whole stack, as a click with nothing on the pointer does.
    pub fn take_all(&mut self, index: usize) -> Option<Stack> {
        self.slots.get_mut(index)?.take()
    }

    /// Take the larger half of a slot's stack, as a right-click does. A
    /// stack of one comes up whole.
    pub fn take_half(&mut self, index: usize) -> Option<Stack> {
        let stack = self.get(index)?;
        let half = stack.count.div_ceil(2);
        let taken = self.take(index, half);
        (taken > 0).then(|| Stack::new(stack.item, taken))
    }

    /// Send a slot's stack across, as a shift-click does: from the hotbar
    /// into the pack, or from the pack into the hotbar, onto matching stacks
    /// first and then into empty slots. What does not fit stays where it was.
    pub fn send(&mut self, index: usize) {
        let Some(stack) = self.get(index) else {
            return;
        };
        let other = if Self::is_hotbar(index) {
            SLOTS..CARRIED
        } else {
            0..SLOTS
        };
        let mut left = stack.count;
        for pass in 0..2 {
            for k in other.clone() {
                if left == 0 {
                    break;
                }
                let slot = &mut self.slots[k];
                match slot {
                    Some(there) if pass == 0 && there.item == stack.item => {
                        let moved = left.min(there.room());
                        there.count += moved;
                        left -= moved;
                    }
                    None if pass == 1 => {
                        let moved = left.min(stack.item.stack_limit());
                        *slot = Some(Stack::new(stack.item, moved));
                        left -= moved;
                    }
                    _ => {}
                }
            }
        }
        self.slots[index] = (left > 0).then(|| Stack::new(stack.item, left));
    }

    /// Move `count` of one slot's stack onto another, as the pack's second
    /// click does (`inventory-grid` decision 5): into an empty slot; onto the
    /// same item as far as it has room, the rest staying behind; and in place
    /// of a different item, which swaps, but only for the whole stack, since
    /// half a stack has nowhere to put what it would displace. Nothing ever
    /// leaves the slots, so no move can lose a stack.
    pub fn shift(&mut self, from: usize, to: usize, count: u16) {
        if from == to || from >= CARRIED || to >= CARRIED {
            return;
        }
        let Some(stack) = self.get(from) else { return };
        let count = count.min(stack.count);
        if count == 0 {
            return;
        }
        match self.get(to) {
            Some(there) if there.item != stack.item => {
                if count == stack.count {
                    self.slots.swap(from, to);
                }
            }
            _ => {
                let taken = self.take(from, count);
                if let Some(back) = self.put(to, Stack::new(stack.item, taken)) {
                    // Room in `from` for what did not fit: it came from there.
                    let left = self.put(from, back);
                    debug_assert!(left.is_none(), "a shift lost {left:?}");
                }
            }
        }
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
        for _ in 0..CARRIED {
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
        let pick = Item::Tool(Tool::Pickaxe);
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
        let pick = Item::Tool(Tool::Pickaxe);
        let mut slots = Slots::new();
        // One slot each means the store holds exactly CARRIED of them,
        // against 99 * CARRIED blocks. Forty-one is one too many.
        assert_eq!(slots.give(pick, CARRIED as u16), 0);
        assert_eq!(slots.give(pick, 1), 1);
    }

    /// `inventory-grid` decision 2: matching stacks with room first, the
    /// hotbar's before the pack's; then the first empty hotbar slot; then the
    /// first empty pack slot.
    #[test]
    fn a_give_fills_the_hotbar_before_the_pack() {
        let mut slots = Slots::new();
        // A pack stack of dirt with room, and one empty hotbar slot left.
        slots.set(SLOTS + 4, Some(Stack::new(DIRT, 10)));
        for k in 0..SLOTS - 1 {
            slots.set(k, Some(Stack::new(Item::Tool(Tool::Rod), 1)));
        }
        slots.give(DIRT, 5);
        assert_eq!(
            slots.get(SLOTS + 4).unwrap().count,
            15,
            "the matching stack first"
        );
        slots.give(STONE, 3);
        assert_eq!(
            slots.get(SLOTS - 1).unwrap().item,
            STONE,
            "then the hotbar's empty slot"
        );
        slots.give(Item::Fish(2), 1);
        assert_eq!(
            slots.get(SLOTS).unwrap().item,
            Item::Fish(2),
            "then the pack's first"
        );
        assert_eq!(slots.hotbar().count(), SLOTS);
    }

    /// A click with a stack on the pointer: into an empty slot, onto the same
    /// item up to its limit, or in place of another, which comes back.
    #[test]
    fn putting_a_stack_fills_merges_or_swaps_and_never_loses_any() {
        let mut slots = Slots::new();
        assert_eq!(slots.put(12, Stack::new(DIRT, 40)), None);
        assert_eq!(slots.get(12), Some(Stack::new(DIRT, 40)));
        let back = slots.put(12, Stack::new(DIRT, 70));
        assert_eq!(slots.get(12).unwrap().count, 99);
        assert_eq!(
            back,
            Some(Stack::new(DIRT, 11)),
            "what did not fit comes back"
        );
        let swapped = slots.put(12, Stack::new(STONE, 3));
        assert_eq!(swapped, Some(Stack::new(DIRT, 99)));
        assert_eq!(slots.get(12), Some(Stack::new(STONE, 3)));
        assert_eq!(
            slots.put(CARRIED, Stack::new(STONE, 1)),
            Some(Stack::new(STONE, 1))
        );
    }

    #[test]
    fn a_right_click_takes_the_larger_half() {
        let mut slots = Slots::new();
        slots.set(3, Some(Stack::new(DIRT, 7)));
        assert_eq!(slots.take_half(3), Some(Stack::new(DIRT, 4)));
        assert_eq!(slots.get(3).unwrap().count, 3);
        slots.set(4, Some(Stack::new(DIRT, 1)));
        assert_eq!(slots.take_half(4), Some(Stack::new(DIRT, 1)));
        assert_eq!(slots.get(4), None);
        assert_eq!(slots.take_all(3), Some(Stack::new(DIRT, 3)));
        assert_eq!(slots.get(3), None);
    }

    /// A shift-click sends a stack across, onto matching stacks first, and
    /// leaves behind only what did not fit.
    #[test]
    fn a_shift_click_sends_a_stack_between_the_hotbar_and_the_pack() {
        let mut slots = Slots::new();
        slots.set(2, Some(Stack::new(DIRT, 60)));
        slots.set(SLOTS + 7, Some(Stack::new(DIRT, 90)));
        slots.send(2);
        assert_eq!(
            slots.get(SLOTS + 7).unwrap().count,
            99,
            "onto the matching stack"
        );
        assert_eq!(
            slots.get(SLOTS).unwrap(),
            Stack::new(DIRT, 51),
            "then the first empty"
        );
        assert_eq!(slots.get(2), None);
        slots.send(SLOTS);
        assert_eq!(slots.get(0), Some(Stack::new(DIRT, 51)), "and back");
        // A full hotbar keeps the stack where it was.
        for k in 0..SLOTS {
            slots.set(k, Some(Stack::new(Item::Tool(Tool::Axe), 1)));
        }
        slots.send(SLOTS + 7);
        assert_eq!(slots.get(SLOTS + 7).unwrap().count, 99);
    }

    #[test]
    fn fish_stack_to_a_creel() {
        let fish = Item::Fish(3);
        assert_eq!(fish.stack_limit(), 16);
        let mut slots = Slots::new();
        assert_eq!(slots.give(fish, 20), 0);
        assert_eq!(slots.get(0).unwrap().count, 16);
        assert_eq!(slots.get(1).unwrap().count, 4);
        assert_ne!(Item::Fish(3), Item::Fish(4), "a species is its own stack");
    }

    /// A new world's tool slot holds nothing and owns all four; holding what
    /// is already held, or what is not owned, is not a change
    /// (`inventory-grid` decision 8).
    #[test]
    fn the_tool_slot_starts_bare_handed_and_changes_only_for_real() {
        let mut kit = Equipment::default();
        assert_eq!(kit.held(), None, "a new world's hands are empty");
        assert_eq!(kit.tools(), Tool::ALL.to_vec());
        assert!(!kit.hold(None), "already empty");
        assert!(kit.hold(Tool::Shovel));
        assert_eq!(kit.held(), Some(Tool::Shovel));
        assert!(!kit.hold(Tool::Shovel), "already in hand");
        assert!(kit.hold(None), "put down");
        assert_eq!(kit.held(), None);
        let partial = Equipment::from_parts([true, false, true, false], Some(Tool::Shovel));
        assert_eq!(partial.held(), None, "not owned leaves the hands bare");
        let mut partial = partial;
        assert!(!partial.hold(Tool::Axe), "not owned");
        assert_eq!(
            Equipment::from_parts([true; 4], Some(Tool::Axe)).held(),
            Some(Tool::Axe)
        );
        // Bare hands and the rod dig nothing; the others dig.
        let with = |held| Equipment::from_parts([true; 4], held).digging_tool();
        assert_eq!(with(None), None, "bare hands break nothing");
        assert_eq!(with(Some(Tool::Rod)), None);
        assert_eq!(with(Some(Tool::Shovel)), Some(Tool::Shovel));
        assert!(!Tool::Rod.digs() && Tool::Shovel.digs());
    }

    /// The pack's second click: a whole stack moves, merges or swaps; half a
    /// stack moves or merges and never swaps; what does not fit stays where
    /// it was; and the total carried never changes.
    #[test]
    fn a_shift_moves_merges_or_swaps_and_never_loses_a_stack() {
        let dirt = Item::Block(Material::Dirt);
        let sand = Item::Block(Material::Sand);
        let total = |s: &Slots| s.iter().flatten().map(|s| u32::from(s.count)).sum::<u32>();
        let mut slots = Slots::new();
        slots.set(0, Some(Stack::new(dirt, 60)));
        slots.set(12, Some(Stack::new(dirt, 70)));
        slots.set(3, Some(Stack::new(sand, 5)));
        let before = total(&slots);

        slots.shift(0, 20, 60);
        assert_eq!(
            slots.get(20),
            Some(Stack::new(dirt, 60)),
            "into an empty slot"
        );
        assert_eq!(slots.get(0), None);

        slots.shift(20, 12, 60);
        assert_eq!(
            slots.get(12),
            Some(Stack::new(dirt, 99)),
            "merged to the limit"
        );
        assert_eq!(slots.get(20), Some(Stack::new(dirt, 31)), "the rest stays");

        slots.shift(3, 12, 5);
        assert_eq!(
            slots.get(12),
            Some(Stack::new(sand, 5)),
            "a whole stack swaps"
        );
        assert_eq!(slots.get(3), Some(Stack::new(dirt, 99)));

        slots.shift(3, 12, 50);
        assert_eq!(slots.get(12), Some(Stack::new(sand, 5)), "half never swaps");
        assert_eq!(slots.get(3), Some(Stack::new(dirt, 99)));

        slots.shift(20, 21, 16);
        assert_eq!(slots.get(21), Some(Stack::new(dirt, 16)), "half moves");
        assert_eq!(slots.get(20), Some(Stack::new(dirt, 15)));

        for (from, to) in [(0, 1), (5, 5), (3, CARRIED), (CARRIED, 3)] {
            slots.shift(from, to, 1);
        }
        assert_eq!(total(&slots), before, "nothing gained or lost");
    }

    /// The picker's wheel walks bare hands and the owned tools, and wraps
    /// both ways.
    #[test]
    fn the_picker_steps_through_bare_hands_and_owned_tools_and_wraps() {
        let kit = Equipment::default();
        assert_eq!(kit.step_from(None, 1), Some(Tool::Rod));
        assert_eq!(kit.step_from(None, -1), Some(Tool::Axe));
        assert_eq!(kit.step_from(Some(Tool::Rod), 1), Some(Tool::Shovel));
        assert_eq!(kit.step_from(Some(Tool::Rod), -1), None);
        assert_eq!(kit.step_from(Some(Tool::Axe), 1), None);
        let partial = Equipment::from_parts([true, false, true, false], Some(Tool::Rod));
        assert_eq!(partial.step_from(Some(Tool::Rod), 1), Some(Tool::Pickaxe));
        assert_eq!(partial.step_from(Some(Tool::Pickaxe), 1), None);
    }
}
