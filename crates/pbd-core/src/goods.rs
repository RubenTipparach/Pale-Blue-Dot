//! Goods: every item that is not a block, a tool or a fish, in ONE
//! append-only table (`farming-and-smelting` decision 1).
//!
//! Seeds, produce, coal, ore and ingots now; food, eggs, milk, wool and feed
//! next, as rows. A row is a short code, a name, a stack limit and an icon.
//!
//! **A good is saved by its code, never its index.** [`Good`] carries the
//! row's index in memory only; the save writes `g<code>,<count>`. Two
//! branches that each append a row therefore merge as two rows in either
//! order, and no save written by one is misread by the other. A code, once
//! shipped, never changes and never names anything else.

/// One row of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoodRow {
    /// The saved name: lower-case letters, digits and hyphens.
    pub code: &'static str,
    /// What a tooltip says.
    pub name: &'static str,
    /// How many one slot holds.
    pub stack: u16,
    /// The 16 x 16 icon, under `assets/`.
    pub icon: &'static str,
}

const fn row(code: &'static str, name: &'static str, stack: u16, icon: &'static str) -> GoodRow {
    GoodRow {
        code,
        name,
        stack,
        icon,
    }
}

/// Every good. **Append rows at the end of the block they belong to's
/// section, or at the very end; never reorder, rename or remove one** - the
/// order is only memory, but the named constants below index into it and a
/// test holds them together.
pub const GOODS: &[GoodRow] = &[
    // ---- farming-and-smelting: crops (M8). Seeds and produce.
    row("wheat-seed", "wheat seed", 99, "items/goods/wheat-seed.png"),
    row("wheat", "wheat", 99, "items/goods/wheat.png"),
    row(
        "turnip-seed",
        "turnip seed",
        99,
        "items/goods/turnip-seed.png",
    ),
    row("turnip", "turnip", 99, "items/goods/turnip.png"),
    row(
        "frost-kale-seed",
        "frost kale seed",
        99,
        "items/goods/frost-kale-seed.png",
    ),
    row("frost-kale", "frost kale", 99, "items/goods/frost-kale.png"),
    row("chili-seed", "chili seed", 99, "items/goods/chili-seed.png"),
    row("chili", "chili", 99, "items/goods/chili.png"),
    // ---- farming-and-smelting: smelting (M3's first tier).
    row("coal", "coal", 99, "items/goods/coal.png"),
    row("tin-ore", "tin ore", 99, "items/goods/tin-ore.png"),
    row(
        "copper-ingot",
        "copper ingot",
        99,
        "items/goods/copper-ingot.png",
    ),
    row("iron-ingot", "iron ingot", 99, "items/goods/iron-ingot.png"),
    row(
        "bronze-ingot",
        "bronze ingot",
        99,
        "items/goods/bronze-ingot.png",
    ),
    // ---- tool-crafting: tier 0 by hand (M3 section 2) and what the hands
    // work it into. Named by code, never by index (`Good::FIBRE` and the
    // rest are found by code at compile time), so other branches' rows can
    // land above or below these in either order.
    row("fibre", "plant fibre", 99, "items/goods/fibre.png"),
    row("stone", "loose stone", 99, "items/goods/stone.png"),
    row("log", "log", 99, "items/goods/log.png"),
    row("planks", "planks", 99, "items/goods/planks.png"),
    row("rope", "rope", 99, "items/goods/rope.png"),
    // ---- Append new goods here (animal products, food, feed ...).
];

/// The row a code names, at compile time: a constant that cannot point at the
/// wrong row whatever was appended before it. Fails the build for a code the
/// table lacks.
const fn by_code_const(code: &str) -> Good {
    let mut index = 0;
    while index < GOODS.len() {
        let row = GOODS[index].code.as_bytes();
        let want = code.as_bytes();
        if row.len() == want.len() {
            let mut k = 0;
            while k < row.len() && row[k] == want[k] {
                k += 1;
            }
            if k == row.len() {
                return Good(index as u16);
            }
        }
        index += 1;
    }
    panic!("no such good")
}

/// A row of [`GOODS`], by index. In memory only: see the module docs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Good(u16);

impl Good {
    pub const WHEAT_SEED: Good = Good(0);
    pub const WHEAT: Good = Good(1);
    pub const TURNIP_SEED: Good = Good(2);
    pub const TURNIP: Good = Good(3);
    pub const FROST_KALE_SEED: Good = Good(4);
    pub const FROST_KALE: Good = Good(5);
    pub const CHILI_SEED: Good = Good(6);
    pub const CHILI: Good = Good(7);
    pub const COAL: Good = Good(8);
    pub const TIN_ORE: Good = Good(9);
    pub const COPPER_INGOT: Good = Good(10);
    pub const IRON_INGOT: Good = Good(11);
    pub const BRONZE_INGOT: Good = Good(12);
    // tool-crafting, by code (see `by_code_const`).
    pub const FIBRE: Good = by_code_const("fibre");
    pub const LOOSE_STONE: Good = by_code_const("stone");
    pub const LOG: Good = by_code_const("log");
    pub const PLANKS: Good = by_code_const("planks");
    pub const ROPE: Good = by_code_const("rope");

    /// Every good, in table order.
    pub fn all() -> impl Iterator<Item = Good> {
        (0..GOODS.len() as u16).map(Good)
    }

    /// The good a saved code names, or `None` for a code this build lacks.
    pub fn by_code(code: &str) -> Option<Good> {
        GOODS
            .iter()
            .position(|row| row.code == code)
            .map(|index| Good(index as u16))
    }

    pub fn row(self) -> &'static GoodRow {
        &GOODS[self.0 as usize]
    }

    pub fn code(self) -> &'static str {
        self.row().code
    }

    pub fn name(self) -> &'static str {
        self.row().name
    }

    pub fn stack_limit(self) -> u16 {
        self.row().stack
    }

    pub fn icon(self) -> &'static str {
        self.row().icon
    }

    /// The row's index: for drawing tables, never for saving.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every code is a word the save line can carry (no comma, no space), and
    /// no two rows share one.
    #[test]
    fn codes_are_unique_saveable_words() {
        let mut seen = std::collections::BTreeSet::new();
        for row in GOODS {
            assert!(crate::records::is_word(row.code), "{}", row.code);
            assert!(seen.insert(row.code), "{} twice", row.code);
            assert!(row.stack > 0);
            assert!(row.icon.starts_with("items/goods/") && row.icon.ends_with(".png"));
        }
    }

    /// The named constants index the rows their names say.
    #[test]
    fn the_constants_name_their_rows() {
        for (good, code) in [
            (Good::WHEAT_SEED, "wheat-seed"),
            (Good::WHEAT, "wheat"),
            (Good::TURNIP_SEED, "turnip-seed"),
            (Good::TURNIP, "turnip"),
            (Good::FROST_KALE_SEED, "frost-kale-seed"),
            (Good::FROST_KALE, "frost-kale"),
            (Good::CHILI_SEED, "chili-seed"),
            (Good::CHILI, "chili"),
            (Good::COAL, "coal"),
            (Good::TIN_ORE, "tin-ore"),
            (Good::COPPER_INGOT, "copper-ingot"),
            (Good::IRON_INGOT, "iron-ingot"),
            (Good::BRONZE_INGOT, "bronze-ingot"),
            (Good::FIBRE, "fibre"),
            (Good::LOOSE_STONE, "stone"),
            (Good::LOG, "log"),
            (Good::PLANKS, "planks"),
            (Good::ROPE, "rope"),
        ] {
            assert_eq!(good.code(), code);
            assert_eq!(Good::by_code(code), Some(good));
        }
        assert_eq!(Good::by_code("dragon-egg"), None);
    }
}
