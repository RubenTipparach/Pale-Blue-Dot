//! What a container holds (`furniture` design 9 and 11): its slots and its
//! coins, kept as a `store` record by the container's id, written with the
//! player's slots in one line whenever a stack moves in or out.
//!
//! A town container holds its **loot** until the player first changes it:
//! goods that fit its household's trade, seeded by the container's id, so
//! the same chest holds the same things in every build. Its first change
//! writes its record, and from then on the record is what it holds; it never
//! refills (design 9).

use super::Kind;
use crate::inventory::{Item, Slots, Stack};
use crate::records::{Record, Records};
use serde::{Deserialize, Serialize};

/// The record kind and its schema.
pub const STORE_KIND: &str = "store";
pub const STORE_SCHEMA: u32 = 1;

/// A container's contents.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Store {
    pub slots: Vec<Option<Stack>>,
    pub coins: u64,
}

/// A `store` record's body: each slot as an item's name and a count, and
/// the coins.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StoreBody {
    pub slots: Vec<Option<(String, u16)>>,
    #[serde(default)]
    pub coins: u64,
}

/// An item's saved name: `recipes.ron`'s (a good's code, `block:<name>`,
/// `tool:<name>`), and a fish as `fish:<species>`.
pub fn item_name(item: Item) -> String {
    match item {
        Item::Fish(species) => format!("fish:{species}"),
        other => crate::missions::item_code(other),
    }
}

/// The item a saved name names, if this build has it.
pub fn item_of(name: &str) -> Option<Item> {
    if let Some(n) = name.strip_prefix("fish:") {
        return n.parse().ok().map(Item::Fish);
    }
    crate::station::item_named(name)
}

impl Store {
    pub fn empty(slots: usize) -> Self {
        Self {
            slots: vec![None; slots],
            coins: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.coins == 0 && self.slots.iter().all(Option::is_none)
    }

    /// Its body. A slot past `slots` is not written.
    pub fn body(&self) -> StoreBody {
        StoreBody {
            slots: self
                .slots
                .iter()
                .map(|s| s.map(|s| (item_name(s.item), s.count)))
                .collect(),
            coins: self.coins,
        }
    }

    /// From a body, `slots` long: a name this build does not know is kept
    /// out of the slots (and a later build that knows it reads it again, as
    /// the record is not rewritten until a stack moves).
    pub fn from_body(body: &StoreBody, slots: usize) -> Self {
        let mut out = Self::empty(slots.max(body.slots.len()));
        for (k, s) in body.slots.iter().enumerate() {
            out.slots[k] = s
                .as_ref()
                .and_then(|(name, count)| Some(Stack::new(item_of(name)?, *count)));
        }
        out.coins = body.coins;
        out
    }

    /// The record that keeps it under `id`.
    pub fn record(&self, id: u64) -> Record {
        Record::of(STORE_KIND, id, STORE_SCHEMA, &self.body())
    }

    /// What container `id` of `kind` holds: its record, or, with none,
    /// `fresh` (a town container's loot, or nothing).
    pub fn of(records: &Records, id: u64, kind: Kind, fresh: impl FnOnce() -> Store) -> Store {
        let slots = kind.slots().unwrap_or(0);
        match records
            .get(STORE_KIND, id)
            .filter(|r| r.schema == STORE_SCHEMA)
            .and_then(|r| r.read::<StoreBody>())
        {
            Some(body) => Store::from_body(&body, slots),
            None => {
                let mut s = fresh();
                s.slots.resize(slots, None);
                s
            }
        }
    }

    /// Put `stack` in: onto stacks of the same item first, then the first
    /// empty slot. What did not fit.
    pub fn give(&mut self, stack: Stack) -> u16 {
        let mut left = stack.count;
        let limit = stack.item.stack_limit();
        for s in self.slots.iter_mut().flatten() {
            if left == 0 {
                break;
            }
            if s.item == stack.item && s.count < limit {
                let n = left.min(limit - s.count);
                s.count += n;
                left -= n;
            }
        }
        for s in self.slots.iter_mut() {
            if left == 0 {
                break;
            }
            if s.is_none() {
                let n = left.min(limit);
                *s = Some(Stack::new(stack.item, n));
                left -= n;
            }
        }
        left
    }

    /// Move the carried stack in slot `index` in, `half` of it or all: how
    /// many moved.
    pub fn put_from(&mut self, carried: &mut Slots, index: usize, half: bool) -> u16 {
        let Some(stack) = carried.get(index) else {
            return 0;
        };
        let want = if half {
            stack.count.div_ceil(2)
        } else {
            stack.count
        };
        let left = self.give(Stack::new(stack.item, want));
        let moved = want - left;
        carried.take(index, moved);
        moved
    }

    /// Move slot `k`'s stack out into `carried`, `half` of it or all: how
    /// many moved.
    pub fn take_into(&mut self, k: usize, carried: &mut Slots, half: bool) -> u16 {
        let Some(Some(stack)) = self.slots.get(k).copied() else {
            return 0;
        };
        let want = if half {
            stack.count.div_ceil(2)
        } else {
            stack.count
        };
        let left = carried.give(stack.item, want);
        let moved = want - left;
        let rest = stack.count - moved;
        self.slots[k] = (rest > 0).then(|| Stack::new(stack.item, rest));
        moved
    }

    /// Take every coin out: how many.
    pub fn take_coins(&mut self) -> u64 {
        std::mem::take(&mut self.coins)
    }
}

/// A household's trade, for its loot (design 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trade {
    Farmer,
    Smith,
    Fisher,
    Trader,
    Hunter,
    Common,
}

impl Trade {
    /// The trade a townsperson's role keeps.
    pub fn of_role(role: &str) -> Trade {
        let has = |w: &[&str]| w.iter().any(|x| role.contains(x));
        if has(&["farmer", "peasant", "forager", "reeve"]) {
            Trade::Farmer
        } else if has(&["smith", "carter", "boatwright"]) {
            Trade::Smith
        } else if has(&["fish", "net", "sailor", "boatman", "porter"]) {
            Trade::Fisher
        } else if has(&["trader", "merchant", "stallholder", "caravan", "innkeeper", "keeper"]) {
            Trade::Trader
        } else if has(&["hunter", "lookout", "watcher", "guard"]) {
            Trade::Hunter
        } else {
            Trade::Common
        }
    }

    /// What its households keep: goods by code, each with the most of it.
    fn goods(self) -> &'static [(&'static str, u16)] {
        match self {
            Trade::Farmer => &[("wheat-seed", 6), ("turnip-seed", 5), ("wheat", 8), ("turnip", 6), ("fibre", 8)],
            Trade::Smith => &[("coal", 8), ("iron-ingot", 3), ("copper-ingot", 3), ("stone", 10)],
            Trade::Fisher => &[("rope", 4), ("bait", 6), ("fibre", 8), ("log", 3)],
            Trade::Trader => &[("planks", 8), ("rope", 3), ("wool", 4), ("leather", 3)],
            Trade::Hunter => &[("leather", 4), ("rope", 3), ("fibre", 6)],
            Trade::Common => &[("planks", 6), ("rope", 2), ("fibre", 6), ("log", 3), ("stone", 6)],
        }
    }
}

/// A small, stable generator off a container's id.
fn roll(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// A town container's loot: one to five stacks by its size, drawn from its
/// trade's goods and a few of anyone's, and coins in a trader's or by one
/// chance in four elsewhere. Fixed by `id`.
pub fn loot(id: u64, trade: Trade, kind: Kind) -> Store {
    let slots = kind.slots().unwrap_or(0);
    let mut out = Store::empty(slots);
    if slots == 0 {
        return out;
    }
    let mut s = id ^ 0x5eed_f00d;
    let (lo, hi): (usize, usize) = match kind {
        Kind::Crate => (1, 2),
        Kind::Drawers => (1, 3),
        Kind::Chest | Kind::Closet => (2, 4),
        _ => (3, 5),
    };
    let n = lo + (roll(&mut s) % (hi - lo + 1) as u64) as usize;
    let goods = trade.goods();
    let common = Trade::Common.goods();
    for k in 0..n {
        let from = if k == 0 || roll(&mut s) % 3 != 0 {
            goods
        } else {
            common
        };
        let (code, most) = from[(roll(&mut s) % from.len() as u64) as usize];
        let Some(item) = item_of(code) else { continue };
        let count = 1 + (roll(&mut s) % u64::from(most)) as u16;
        out.give(Stack::new(item, count));
    }
    let chance = if trade == Trade::Trader { 1 } else { 4 };
    if roll(&mut s) % chance == 0 {
        out.coins = 2 + roll(&mut s) % 14;
    }
    out
}
