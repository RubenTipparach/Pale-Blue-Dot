//! Tier 0 off the ground (`tool-crafting` decisions 2 and 3, after M3
//! section 2): what a dig turns up besides its block, and what a tree gives
//! to hands and to an axe.
//!
//! A find is DERIVED from the cell and layer, so the same cell always
//! answers the same and nothing is stored but the dig's own line. A block a
//! player placed turns up nothing, so placing and digging cannot farm it.
//! What a tree has given is a stored record (`tree`), because a tree is not
//! a block and has no edit to remember it by.

use crate::goods::Good;
use crate::terrain::Material;
use serde::{Deserialize, Serialize};

/// `assets/config/gather.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct GatherSettings {
    /// Dug grass gives fibre in one cell in this many.
    pub fibre_one_in: u32,
    /// Dug dirt, soil and sand give loose stone in one cell in this many.
    pub stone_one_in: u32,
    /// The most logs a tree gives an axe, counting the branch a hand took.
    pub axe_logs: u16,
    /// Wet ground at the shore gives bait in one cell in this many
    /// (`starter-missions` G1).
    pub bait_one_in: u32,
    /// Wet ground: a dug layer whose altitude, m from the sea's level, lies
    /// in this band, lowest first.
    pub bait_band_m: (f32, f32),
}

impl Default for GatherSettings {
    fn default() -> Self {
        Self {
            fibre_one_in: 3,
            stone_one_in: 3,
            axe_logs: 4,
            bait_one_in: 3,
            bait_band_m: (-2.0, 1.0),
        }
    }
}

impl GatherSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=64).contains(&self.fibre_one_in) {
            return Err(format!(
                "gather.fibre_one_in must be 1 to 64, got {}",
                self.fibre_one_in
            ));
        }
        if !(1..=64).contains(&self.stone_one_in) {
            return Err(format!(
                "gather.stone_one_in must be 1 to 64, got {}",
                self.stone_one_in
            ));
        }
        if !(1..=32).contains(&self.axe_logs) {
            return Err(format!(
                "gather.axe_logs must be 1 to 32, got {}",
                self.axe_logs
            ));
        }
        if !(1..=64).contains(&self.bait_one_in) {
            return Err(format!(
                "gather.bait_one_in must be 1 to 64, got {}",
                self.bait_one_in
            ));
        }
        let (low, high) = self.bait_band_m;
        if !(low.is_finite() && high.is_finite() && low < high) {
            return Err(format!(
                "gather.bait_band_m must be two finite metres, lowest first, got {:?}",
                self.bait_band_m
            ));
        }
        Ok(())
    }

    /// Bait out of `material` dug from `cell`/`layer`, whose altitude is
    /// `altitude_m` from the sea's level: earth, grass or sand in the wet
    /// band at the shore, one cell in a few, never from a block a player
    /// `placed`. A find beside [`Self::find`], from its own hash, so the
    /// fibre and stone a cell gives do not change.
    pub fn bait(
        &self,
        material: Material,
        cell: u32,
        layer: u16,
        placed: bool,
        altitude_m: f32,
    ) -> Option<Good> {
        let (low, high) = self.bait_band_m;
        let wet = altitude_m.is_finite() && (low..=high).contains(&altitude_m);
        let earth = matches!(
            material,
            Material::Grass
                | Material::DryGrass
                | Material::JungleGrass
                | Material::Dirt
                | Material::Soil
                | Material::Sand
        );
        if placed || !wet || !earth {
            return None;
        }
        let h = mix((u64::from(cell) << 16 | u64::from(layer)) ^ 0xba17_e000_u64);
        h.is_multiple_of(u64::from(self.bait_one_in))
            .then_some(Good::BAIT)
    }

    /// What digging `material` out of `cell`/`layer` turns up besides the
    /// block: fibre from grass, loose stone from earth, one cell in a few.
    /// Nothing from a block a player `placed`.
    pub fn find(&self, material: Material, cell: u32, layer: u16, placed: bool) -> Option<Good> {
        if placed {
            return None;
        }
        let (good, one_in, salt) = match material {
            Material::Grass | Material::DryGrass | Material::JungleGrass => {
                (Good::FIBRE, self.fibre_one_in, 0xf1b3_e000_u64)
            }
            Material::Dirt | Material::Soil | Material::Sand => {
                (Good::LOOSE_STONE, self.stone_one_in, 0x5701_e000_u64)
            }
            _ => return None,
        };
        let h = mix((u64::from(cell) << 16 | u64::from(layer)) ^ salt);
        h.is_multiple_of(u64::from(one_in)).then_some(good)
    }

    /// Whether a tree that has given `taken` logs gives another, to hands
    /// (`by_axe` false) or an axe (`tool-crafting` decision 3): hands take
    /// the one branch, an axe takes logs up to `axe_logs` in all.
    pub fn tree_gives(&self, taken: u16, by_axe: bool) -> Result<(), TreeSpent> {
        if by_axe {
            if taken < self.axe_logs {
                Ok(())
            } else {
                Err(TreeSpent::Felled)
            }
        } else if taken == 0 {
            Ok(())
        } else {
            Err(TreeSpent::BranchTaken)
        }
    }
}

/// Why a tree gives no log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeSpent {
    /// Hands have had its branch; an axe takes more.
    BranchTaken,
    /// It has given all its wood.
    Felled,
}

/// The record kind of what a tree has given, by the cell's key.
pub const TREE_KIND: &str = "tree";
pub const TREE_SCHEMA: u32 = 1;

/// A `tree` record's body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct TreeTaken {
    /// Logs taken from it so far, the branch included.
    pub logs: u16,
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dig::{DigSettings, Digger};
    use crate::inventory::{Equipment, Item, Slots, Tier, Tool};
    use crate::station::{RecipeFile, STATIONS, StationState, hand, station_by_code};

    /// A third of grass cells give fibre and a third of earth cells loose
    /// stone, the same cells every time, and placed blocks or rock nothing.
    #[test]
    fn grass_gives_fibre_and_earth_gives_stone_in_a_share_of_cells() {
        let g = GatherSettings::default();
        let share = |m: Material, want: Good| {
            (0..30_000u32)
                .filter(|&c| g.find(m, c, 40, false) == Some(want))
                .count()
        };
        let fibre = share(Material::Grass, Good::FIBRE);
        assert!((9_000..11_000).contains(&fibre), "{fibre}");
        let stone = share(Material::Dirt, Good::LOOSE_STONE);
        assert!((9_000..11_000).contains(&stone), "{stone}");
        for c in 0..200 {
            assert_eq!(
                g.find(Material::Soil, c, 7, false),
                g.find(Material::Soil, c, 7, false)
            );
            assert_eq!(g.find(Material::Grass, c, 7, true), None, "placed");
            assert_eq!(g.find(Material::Rock, c, 7, false), None);
            assert_eq!(g.find(Material::Tilled, c, 7, false), None);
        }
    }

    /// Bait turns up in a third of wet shore cells, the same cells every
    /// time, and never above or below the band, from rock or a placed block
    /// (`starter-missions` G1).
    #[test]
    fn wet_ground_at_the_shore_gives_bait_in_a_share_of_cells() {
        let g = GatherSettings::default();
        let share = (0..30_000u32)
            .filter(|&c| g.bait(Material::Sand, c, 145, false, 0.0) == Some(Good::BAIT))
            .count();
        assert!((9_000..11_000).contains(&share), "{share}");
        for c in 0..2_000 {
            assert_eq!(
                g.bait(Material::Dirt, c, 144, false, -1.0),
                g.bait(Material::Dirt, c, 144, false, -1.0)
            );
            assert_eq!(g.bait(Material::Dirt, c, 150, false, 5.0), None, "dry");
            assert_eq!(g.bait(Material::Dirt, c, 140, false, -6.0), None, "deep");
            assert_eq!(g.bait(Material::Grass, c, 145, true, 0.0), None, "placed");
            assert_eq!(g.bait(Material::Rock, c, 145, false, 0.0), None, "rock");
            assert_eq!(g.bait(Material::Dirt, c, 145, false, f32::NAN), None);
        }
    }

    /// Hands take one branch; an axe takes logs up to the limit.
    #[test]
    fn a_tree_gives_a_branch_by_hand_and_logs_to_an_axe() {
        let g = GatherSettings::default();
        assert_eq!(g.tree_gives(0, false), Ok(()));
        assert_eq!(g.tree_gives(1, false), Err(TreeSpent::BranchTaken));
        assert_eq!(g.tree_gives(1, true), Ok(()));
        assert_eq!(g.tree_gives(3, true), Ok(()));
        assert_eq!(g.tree_gives(4, true), Err(TreeSpent::Felled));
        g.validate().unwrap();
        let back: TreeTaken =
            ron::from_str(&ron::to_string(&TreeTaken { logs: 3 }).unwrap()).unwrap();
        assert_eq!(back.logs, 3);
    }

    /// The route (`tool-crafting` risk, task 1.4): from empty hands in a new
    /// world, hand actions alone - digging grass and earth by hand, a branch
    /// from each of a few trees, the hands' recipes, a workbench, the
    /// workbench's recipe - make a stone pickaxe. Counted, so a retune that
    /// made the climb a grind fails here.
    #[test]
    fn a_new_world_reaches_a_stone_pick_by_hand() {
        let g = GatherSettings::default();
        let dig = DigSettings::default();
        let book = RecipeFile::default().resolve().unwrap();
        let mut kit = Equipment::new_world();
        let mut slots = Slots::new();
        let mut seconds = 0.0_f32;
        // Hands dig: grass cells, then the earth under them, until there is
        // fibre for a rope and stone for a pick. Distinct cells, as a player
        // walking a meadow digs.
        let mut cell = 1_000u32;
        let mut digs = 0;
        while slots.count(Item::Good(Good::FIBRE)) < 3
            || slots.count(Item::Good(Good::LOOSE_STONE)) < 3
        {
            for (material, layer) in [(Material::Grass, 40u16), (Material::Dirt, 39)] {
                seconds += dig.secs_by(material, Digger::Hands).expect("hands dig it");
                if let Some(good) = g.find(material, cell, layer, false) {
                    slots.give(Item::Good(good), 1);
                }
                slots.give(Item::Block(material), 1);
                digs += 1;
            }
            cell += 1;
            assert!(digs < 60, "{digs} digs and still short");
        }
        // A branch from each of two trees: two logs, eight planks.
        let mut logs = 0;
        for _tree in 0..2 {
            g.tree_gives(0, false).unwrap();
            slots.give(Item::Good(Good::LOG), 1);
            logs += 1;
        }
        assert!(logs <= 3, "a new world needs at most three trees");
        // The hands' recipes, in the pack, one job at a time.
        let mut hands = StationState::new(hand());
        let mut now = 0.0_f64;
        let mut make =
            |state: &mut StationState, slots: &mut Slots, code: &str, kit: &mut Equipment| {
                let recipe = book.get(code).unwrap();
                state
                    .queue_for(slots, Some(kit), Some(&book), recipe, now)
                    .unwrap_or_else(|why| panic!("{code}: {why:?}"));
                now += f64::from(recipe.secs);
                assert_eq!(state.collect_into(slots, Some(kit), &book, now).len(), 1);
            };
        make(&mut hands, &mut slots, "planks", &mut kit);
        make(&mut hands, &mut slots, "planks", &mut kit);
        make(&mut hands, &mut slots, "rope", &mut kit);
        make(&mut hands, &mut slots, "workbench", &mut kit);
        assert_eq!(slots.count(Item::Block(Material::Workbench)), 1);
        // The workbench is placed and works.
        let mut bench = StationState::new(station_by_code("workbench").unwrap());
        make(&mut bench, &mut slots, "stone-pickaxe", &mut kit);
        assert_eq!(kit.tier(Tool::Pickaxe), Some(Tier::Stone));
        assert!(kit.hold(Tool::Pickaxe));
        assert!(
            dig.secs_by(Material::Stone, Digger::Tool(Tool::Pickaxe, Tier::Stone))
                .is_ok(),
            "and the pick breaks stone"
        );
        // A few minutes of digging, not an hour.
        assert!(seconds < 240.0, "{seconds} s of digging by hand");
        assert!(STATIONS.iter().any(|s| s.code == "workbench"));
    }
}
