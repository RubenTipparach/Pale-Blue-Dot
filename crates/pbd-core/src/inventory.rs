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

/// The tool slot: which tools the player owns and which one is in hand.
///
/// Its own store beside the ten slots, because a tool is not a stack and a
/// tool held in a slot would compete with blocks and fish for room.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Equipment {
    owned: [bool; 4],
    held: Tool,
}

impl Default for Equipment {
    /// A new world's kit: all four owned, the rod in hand.
    fn default() -> Self {
        Self {
            owned: [true; 4],
            held: Tool::Rod,
        }
    }
}

impl Equipment {
    /// Rebuild from a save's record; a held tool that is not owned falls back
    /// to the first owned one rather than handing the player something they
    /// do not have.
    pub fn from_parts(owned: [bool; 4], held: Tool) -> Self {
        let mut equipment = Self { owned, held };
        if !equipment.owns(held) {
            equipment.held = Tool::ALL
                .into_iter()
                .find(|t| equipment.owns(*t))
                .unwrap_or(Tool::Rod);
        }
        equipment
    }

    pub fn held(&self) -> Tool {
        self.held
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

    /// Put a tool in hand. Refused, and `false`, when it is not owned or is
    /// already held: only a real change is a change worth saving.
    pub fn hold(&mut self, tool: Tool) -> bool {
        if !self.owns(tool) || self.held == tool {
            return false;
        }
        self.held = tool;
        true
    }

    /// The owned tool `by` steps from `from`, wrapping: what the picker's
    /// wheel moves through.
    pub fn step_from(&self, from: Tool, by: i32) -> Tool {
        let tools = self.tools();
        if tools.is_empty() {
            return from;
        }
        let at = tools.iter().position(|t| *t == from).unwrap_or(0) as i32;
        let n = tools.len() as i32;
        tools[(((at + by) % n + n) % n) as usize]
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
        // One slot each means the store holds exactly SLOTS of them, against
        // 99 * SLOTS blocks. Eleven is one too many.
        assert_eq!(slots.give(pick, SLOTS as u16), 0);
        assert_eq!(slots.give(pick, 1), 1);
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

    /// A new world's tool slot holds the rod and owns all four; holding what
    /// is already held, or what is not owned, is not a change.
    #[test]
    fn the_tool_slot_starts_on_the_rod_and_changes_only_for_real() {
        let mut kit = Equipment::default();
        assert_eq!(kit.held(), Tool::Rod);
        assert_eq!(kit.tools(), Tool::ALL.to_vec());
        assert!(!kit.hold(Tool::Rod), "already in hand");
        assert!(kit.hold(Tool::Shovel));
        assert_eq!(kit.held(), Tool::Shovel);
        let partial = Equipment::from_parts([true, false, true, false], Tool::Shovel);
        assert_eq!(
            partial.held(),
            Tool::Rod,
            "not owned falls back to the first owned"
        );
        let mut partial = partial;
        assert!(!partial.hold(Tool::Axe), "not owned");
        assert!(!Tool::Rod.digs() && Tool::Shovel.digs());
    }

    /// The picker's wheel walks the owned tools and wraps both ways.
    #[test]
    fn the_picker_steps_through_owned_tools_and_wraps() {
        let kit = Equipment::default();
        assert_eq!(kit.step_from(Tool::Rod, 1), Tool::Shovel);
        assert_eq!(kit.step_from(Tool::Rod, -1), Tool::Axe);
        assert_eq!(kit.step_from(Tool::Axe, 1), Tool::Rod);
        let partial = Equipment::from_parts([true, false, true, false], Tool::Rod);
        assert_eq!(partial.step_from(Tool::Rod, 1), Tool::Pickaxe);
        assert_eq!(partial.step_from(Tool::Pickaxe, 1), Tool::Rod);
    }
}
