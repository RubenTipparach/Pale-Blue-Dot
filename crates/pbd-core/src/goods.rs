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

use crate::locale::{Localized, tr};

/// One row of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GoodRow {
    /// The saved name: lower-case letters, digits and hyphens.
    pub code: &'static str,
    /// How many one slot holds.
    pub stack: u16,
    /// The 16 x 16 icon, under `assets/`.
    pub icon: &'static str,
}

const fn row(code: &'static str, stack: u16, icon: &'static str) -> GoodRow {
    GoodRow { code, stack, icon }
}

impl GoodRow {
    /// What a tooltip says: `item.<code>.name` in the locale file.
    pub fn name(&self) -> Localized {
        tr(&format!("item.{}.name", self.code))
    }
}

/// Every good. **Append rows at the end of the block they belong to's
/// section, or at the very end; never reorder, rename or remove one** - the
/// order is only memory, but the named constants below index into it and a
/// test holds them together.
pub const GOODS: &[GoodRow] = &[
    // ---- farming-and-smelting: crops (M8). Seeds and produce.
    row("wheat-seed", 99, "items/goods/wheat-seed.png"),
    row("wheat", 99, "items/goods/wheat.png"),
    row("turnip-seed", 99, "items/goods/turnip-seed.png"),
    row("turnip", 99, "items/goods/turnip.png"),
    row("frost-kale-seed", 99, "items/goods/frost-kale-seed.png"),
    row("frost-kale", 99, "items/goods/frost-kale.png"),
    row("chili-seed", 99, "items/goods/chili-seed.png"),
    row("chili", 99, "items/goods/chili.png"),
    // ---- farming-and-smelting: smelting (M3's first tier).
    row("coal", 99, "items/goods/coal.png"),
    row("tin-ore", 99, "items/goods/tin-ore.png"),
    row("copper-ingot", 99, "items/goods/copper-ingot.png"),
    row("iron-ingot", 99, "items/goods/iron-ingot.png"),
    row("bronze-ingot", 99, "items/goods/bronze-ingot.png"),
    // ---- animal-husbandry: a pen's pieces (fences and gates stand on hex
    // edges as `fence` records, not as blocks) and what kept animals give.
    row("fence", 64, "items/goods/fence.png"),
    row("gate", 64, "items/goods/gate.png"),
    row("egg", 32, "items/goods/egg.png"),
    row("wool", 32, "items/goods/wool.png"),
    row("milk", 16, "items/goods/milk.png"),
    // ---- tool-crafting: tier 0 by hand (M3 section 2) and what the hands
    // work it into. Named by code, never by index (`Good::FIBRE` and the
    // rest are found by code at compile time), so other branches' rows can
    // land above or below these in either order.
    row("fibre", 99, "items/goods/fibre.png"),
    row("stone", 99, "items/goods/stone.png"),
    row("log", 99, "items/goods/log.png"),
    row("planks", 99, "items/goods/planks.png"),
    row("rope", 99, "items/goods/rope.png"),
    // ---- Append new goods here (food, feed ...).
    // cooking (`cooking` design S1): the campfire's dishes. Saved by code.
    row("grilled-fish", 16, "items/goods/grilled-fish.png"),
    row("chili-fish", 16, "items/goods/chili-fish.png"),
    row("flatbread", 32, "items/goods/flatbread.png"),
    row("roast-turnip", 32, "items/goods/roast-turnip.png"),
    row("kale-skewer", 16, "items/goods/kale-skewer.png"),
    row("fried-egg", 16, "items/goods/fried-egg.png"),
    row("roast-meat", 16, "items/goods/roast-meat.png"),
    // ---- water-flow E11: the wooden bucket, empty and full.
    row("bucket", 4, "items/goods/bucket.png"),
    row("water-bucket", 1, "items/goods/water-bucket.png"),
    // ---- animal-husbandry design 13: a hen's egg with a cockerel in her
    // pen (its parentage rides in a `brood` record), and the incubator piece.
    row("fertile-egg", 32, "items/goods/fertile-egg.png"),
    row("incubator", 8, "items/goods/incubator.png"),
    // ---- flora-v2 design 12.5: the trees' fruit, picked by hand, in
    // `pbd_core::flora::FRUIT_KINDS` order.
    row("apple", 32, "items/goods/apple.png"),
    row("pear", 32, "items/goods/pear.png"),
    row("plum", 32, "items/goods/plum.png"),
    row("cherry", 32, "items/goods/cherry.png"),
    row("mango", 32, "items/goods/mango.png"),
    row("papaya", 32, "items/goods/papaya.png"),
    row("marula", 32, "items/goods/marula.png"),
    row("baobab", 32, "items/goods/baobab.png"),
    row("pine-cone", 32, "items/goods/pine-cone.png"),
    // ---- starter-missions G1: worms dug from wet ground at the shore, put
    // on the hook when a cast is made with them in the pack.
    row("bait", 99, "items/goods/bait.png"),
    // ---- small-boats design 3: a made boat, carried until it is launched.
    row("inner-tube", 1, "items/goods/inner-tube.png"),
    row("raft", 1, "items/goods/raft.png"),
    row("rowboat", 1, "items/goods/rowboat.png"),
    // ---- glowing-mushrooms design 3: the glowing plant the meadows grow,
    // picked with F and eaten for the body glow.
    row("glowing-mushroom", 32, "items/goods/glowing-mushroom.png"),
    // ---- small-boats design 2e: a cow's hide, tanned; what an inner tube is
    // made of (owner, 2026-11-18: "tubes: would probably need just leather",
    // "cows and horses should").
    row("leather", 32, "items/goods/leather.png"),
    // ---- townsfolk design 9: a delivery job's parcel, handed to the named
    // town's people.
    row("parcel", 1, "items/goods/parcel.png"),
    // ---- townsfolk design 31: crushed stone, laid as a gravel road.
    row("gravel", 99, "items/goods/gravel.png"),
    // ---- townsfolk design 41: the grinder's crushed stone, which the mixer
    // makes gravel of, and survey stakes, a road's waypoints.
    row("crushed-stone", 99, "items/goods/crushed-stone.png"),
    row("survey-stakes", 32, "items/goods/survey-stakes.png"),
    // ---- player-building design B8: a door, made at the workbench and hung
    // in a doorway.
    row("door", 8, "items/goods/door.png"),
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
    // animal-husbandry
    pub const FENCE: Good = Good(13);
    pub const GATE: Good = Good(14);
    pub const EGG: Good = Good(15);
    pub const WOOL: Good = Good(16);
    pub const MILK: Good = Good(17);
    // tool-crafting, by code (see `by_code_const`).
    pub const FIBRE: Good = by_code_const("fibre");
    pub const LOOSE_STONE: Good = by_code_const("stone");
    pub const LOG: Good = by_code_const("log");
    pub const PLANKS: Good = by_code_const("planks");
    pub const ROPE: Good = by_code_const("rope");
    pub const BUCKET: Good = by_code_const("bucket");
    pub const WATER_BUCKET: Good = by_code_const("water-bucket");
    pub const FERTILE_EGG: Good = by_code_const("fertile-egg");
    pub const INCUBATOR: Good = by_code_const("incubator");
    pub const BAIT: Good = by_code_const("bait");
    // small-boats: the made boats.
    pub const INNER_TUBE: Good = by_code_const("inner-tube");
    pub const RAFT: Good = by_code_const("raft");
    pub const ROWBOAT: Good = by_code_const("rowboat");
    // glowing-mushrooms: the glowing plant, picked.
    pub const GLOWING_MUSHROOM: Good = by_code_const("glowing-mushroom");
    // townsfolk: a delivery's parcel.
    pub const PARCEL: Good = by_code_const("parcel");
    // townsfolk: crushed stone for a gravel road.
    pub const GRAVEL: Good = by_code_const("gravel");
    // townsfolk design 41: the grinder's output, and a road's waypoints.
    pub const CRUSHED_STONE: Good = by_code_const("crushed-stone");
    pub const SURVEY_STAKES: Good = by_code_const("survey-stakes");
    // player-building: a door to hang in a doorway.
    pub const DOOR: Good = by_code_const("door");

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

    pub fn name(self) -> Localized {
        self.row().name()
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
    /// Every good, crop and station has its name in the locale file, by
    /// its code.
    #[test]
    fn every_good_crop_and_station_has_its_name() {
        let english = crate::locale::Locale::english();
        let keys = super::GOODS
            .iter()
            .map(|r| format!("item.{}.name", r.code))
            .chain(
                crate::farming::CROPS
                    .iter()
                    .map(|r| format!("crop.{}.name", r.code)),
            )
            .chain(
                crate::station::STATIONS
                    .iter()
                    .map(|r| format!("station.{}.name", r.code)),
            )
            .chain(
                crate::flora::FRUIT_GOODS
                    .iter()
                    .map(|c| format!("item.{c}.many")),
            );
        for key in keys {
            assert!(english.get(&key).is_some(), "{key}");
        }
    }

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
            (Good::FENCE, "fence"),
            (Good::GATE, "gate"),
            (Good::EGG, "egg"),
            (Good::WOOL, "wool"),
            (Good::MILK, "milk"),
            (Good::FIBRE, "fibre"),
            (Good::LOOSE_STONE, "stone"),
            (Good::LOG, "log"),
            (Good::PLANKS, "planks"),
            (Good::ROPE, "rope"),
            (Good::FERTILE_EGG, "fertile-egg"),
            (Good::INCUBATOR, "incubator"),
            (Good::BAIT, "bait"),
            (Good::GLOWING_MUSHROOM, "glowing-mushroom"),
            (Good::DOOR, "door"),
        ] {
            assert_eq!(good.code(), code);
            assert_eq!(Good::by_code(code), Some(good));
        }
        assert_eq!(Good::by_code("dragon-egg"), None);
    }
}
