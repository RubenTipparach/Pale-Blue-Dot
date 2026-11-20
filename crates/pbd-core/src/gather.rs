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
    /// Earth by the water gives bait in one cell in this many
    /// (`starter-missions` G1, `dig-fixes` 2).
    pub bait_one_in: u32,
    /// By the water: a water cell within this many cells of the dug one,
    /// counted in steps across the cell's neighbours (0 is the dug cell's own
    /// column only).
    pub bait_reach_cells: u32,
    /// And that water's layer this many metres from the dug layer, the
    /// water's altitude less the dug layer's, lowest first.
    pub bait_water_m: (f32, f32),
}

impl Default for GatherSettings {
    fn default() -> Self {
        Self {
            fibre_one_in: 3,
            stone_one_in: 3,
            axe_logs: 4,
            bait_one_in: 3,
            bait_reach_cells: 3,
            bait_water_m: (-3.0, 2.0),
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
        if self.bait_reach_cells > 8 {
            return Err(format!(
                "gather.bait_reach_cells must be 0 to 8, got {}",
                self.bait_reach_cells
            ));
        }
        let (low, high) = self.bait_water_m;
        if !(low.is_finite() && high.is_finite() && low <= high && low >= -16.0 && high <= 16.0) {
            return Err(format!(
                "gather.bait_water_m must be two finite metres in [-16, 16], lowest first, \
                 got {:?}",
                self.bait_water_m
            ));
        }
        Ok(())
    }

    /// Bait out of `material` dug from `cell`/`layer`: earth, grass or sand
    /// `by_water` ([`Self::by_water`]), one cell in a few, never from a
    /// block a player `placed`. A find beside [`Self::find`], from its own
    /// hash, so the fibre and stone a cell gives do not change.
    ///
    /// Sand counts: the sea's edge is a band of beach sand, and bait there is
    /// half of what the owner asked for (`dig-fixes` 2).
    pub fn bait(
        &self,
        material: Material,
        cell: u32,
        layer: u16,
        placed: bool,
        by_water: bool,
    ) -> Option<Good> {
        // Every dirt there is: the three grasses, dirt, soil, a field's
        // tilled soil and sand (owner, 2026-11-19: "theres multiple dirt
        // types, so make sure to cover all dirt").
        let earth = matches!(
            material,
            Material::Grass
                | Material::DryGrass
                | Material::JungleGrass
                | Material::Dirt
                | Material::Soil
                | Material::Tilled
                | Material::Sand
        );
        if placed || !by_water || !earth {
            return None;
        }
        let h = mix((u64::from(cell) << 16 | u64::from(layer)) ^ 0xba17_e000_u64);
        h.is_multiple_of(u64::from(self.bait_one_in))
            .then_some(Good::BAIT)
    }

    /// Whether ground dug at `layer` of cell `from` is by the water, for
    /// bait (`dig-fixes` 2): a water cell, still or flowing, of a river or the
    /// sea, in a column within [`Self::bait_reach_cells`] steps of `from`
    /// across `neighbours` (the dug cell's own column included), whose layer
    /// lies [`Self::bait_water_m`] from `layer`.
    ///
    /// The one nearness rule. The caller hands it its neighbour table and its
    /// columns (`material` of a cell and layer, `None` where it has none):
    /// the column tier's water, which carries the rivers, the sea and every
    /// edit, is what the CPU holds authoritatively.
    pub fn by_water<K, N>(
        &self,
        from: K,
        layer: usize,
        mut neighbours: impl FnMut(K) -> N,
        mut material: impl FnMut(K, usize) -> Option<Material>,
    ) -> bool
    where
        K: Copy + PartialEq,
        N: IntoIterator<Item = K>,
    {
        let (low, high) = self.bait_water_m;
        let dug = crate::column::layer_altitude(layer);
        let layers: Vec<usize> = (0..crate::column::LAYERS)
            .filter(|&l| {
                let rise = crate::column::layer_altitude(l) - dug;
                (low..=high).contains(&rise)
            })
            .collect();
        let mut seen = vec![from];
        let mut ring = vec![from];
        for step in 0..=self.bait_reach_cells {
            for &cell in &ring {
                if layers
                    .iter()
                    .any(|&l| material(cell, l).is_some_and(Material::is_water))
                {
                    return true;
                }
            }
            if step == self.bait_reach_cells {
                break;
            }
            let mut next = Vec::new();
            for &cell in &ring {
                for n in neighbours(cell) {
                    if !seen.contains(&n) {
                        seen.push(n);
                        next.push(n);
                    }
                }
            }
            ring = next;
        }
        false
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
    /// (`by_axe` false) or an axe: either takes logs up to `axe_logs` in all,
    /// hands only slower (`tree-punch-shake` decision 1; owner, 2026-11-18:
    /// "allow punching trees to get wood logs for the first axe. Just takes
    /// longer"). `by_axe` is kept for the callers that say which it was.
    pub fn tree_gives(&self, taken: u16, _by_axe: bool) -> Result<(), TreeSpent> {
        if taken < self.axe_logs {
            Ok(())
        } else {
            Err(TreeSpent::Felled)
        }
    }
}

/// Why a tree gives no log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreeSpent {
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

    /// Bait turns up in a third of earth cells by the water, the same cells
    /// every time, and never away from it, from rock or a placed block
    /// (`starter-missions` G1, `dig-fixes` 2).
    #[test]
    fn earth_by_the_water_gives_bait_in_a_share_of_cells() {
        let g = GatherSettings::default();
        for earth in [
            Material::Sand,
            Material::Dirt,
            Material::Grass,
            Material::Soil,
        ] {
            let share = (0..30_000u32)
                .filter(|&c| g.bait(earth, c, 145, false, true) == Some(Good::BAIT))
                .count();
            assert!((9_000..11_000).contains(&share), "{earth:?}: {share}");
        }
        for c in 0..2_000 {
            assert_eq!(
                g.bait(Material::Dirt, c, 144, false, true),
                g.bait(Material::Dirt, c, 144, false, true)
            );
            assert_eq!(g.bait(Material::Dirt, c, 150, false, false), None, "dry");
            assert_eq!(g.bait(Material::Grass, c, 145, true, true), None, "placed");
            assert_eq!(g.bait(Material::Rock, c, 145, false, true), None, "rock");
            assert_eq!(g.bait(Material::Stone, c, 145, false, true), None, "stone");
            assert_eq!(g.bait(Material::Snow, c, 145, false, true), None, "snow");
        }
    }

    /// A strip of cells in a row, each the next's neighbour, and their
    /// columns: ground to `ground` (its top layer), and over cells `wet` water
    /// from `bed` to `top`, layers inclusive.
    fn strip(
        ground: impl Fn(i32) -> usize,
        wet: impl Fn(i32) -> Option<(usize, usize)>,
    ) -> impl Fn(i32, usize) -> Option<Material> {
        move |cell, layer| {
            if let Some((bed, top)) = wet(cell)
                && (bed..=top).contains(&layer)
            {
                return Some(Material::Water);
            }
            Some(if layer <= ground(cell) {
                Material::Dirt
            } else {
                Material::Air
            })
        }
    }

    fn row(cell: i32) -> [i32; 2] {
        [cell - 1, cell + 1]
    }

    /// The nearness rule (`dig-fixes` 2): a river's water inland, metres
    /// over the sea, and the sea's own, both count within the reach; past
    /// the reach, far over the water or deep under it, nothing does.
    #[test]
    fn by_the_water_is_a_few_cells_from_a_river_or_the_sea() {
        let g = GatherSettings::default();
        // An inland river: its water in cells 0 and 1, layers 160 and 161,
        // on banks whose top is layer 161, far over any sea.
        let river = strip(|_| 161, |c| (0..=1).contains(&c).then_some((160, 161)));
        let reach = g.bait_reach_cells as i32;
        for cell in 2..=1 + reach {
            assert!(g.by_water(cell, 161, row, &river), "bank cell {cell}");
            assert!(g.by_water(cell, 160, row, &river), "under the bank {cell}");
        }
        assert!(!g.by_water(2 + reach, 161, row, &river), "past the reach");
        assert!(!g.by_water(40, 161, row, &river), "far inland");
        // A cliff top over the river, and a mine under it, are dry.
        assert!(!g.by_water(2, 170, row, &river), "a cliff over it");
        assert!(!g.by_water(2, 150, row, &river), "a mine under it");
        // The sea: water from the bed up to its level, layer 140, in every
        // cell below zero; a beach rising one layer a cell from there.
        let sea = strip(
            |c| if c < 0 { 130 } else { 140 + c as usize },
            |c| (c < 0).then_some((131, 140)),
        );
        assert!(g.by_water(0, 140, row, &sea), "the beach at the water");
        assert!(g.by_water(2, 142, row, &sea), "two cells up the beach");
        assert!(!g.by_water(3, 143, row, &sea), "four cells off the water");
        assert!(!g.by_water(1, 145, row, &sea), "a cliff over the sea");
        assert!(!g.by_water(20, 160, row, &sea), "the hills behind");
        // Flowing water is water too.
        let falls = |c: i32, l: usize| {
            Some(if c == 0 && l == 150 {
                Material::flowing(3, true)
            } else if l <= 149 {
                Material::Dirt
            } else {
                Material::Air
            })
        };
        assert!(g.by_water(1, 149, row, falls));
        // A cell with no column answers nothing.
        assert!(!g.by_water(1, 149, row, |_, _| None));
        // Nothing reached when the reach is zero but the dug cell's own.
        let own = GatherSettings {
            bait_reach_cells: 0,
            ..g
        };
        assert!(!own.by_water(2, 161, row, &river));
        assert!(own.by_water(1, 161, row, &river));
        let bad = GatherSettings {
            bait_water_m: (2.0, -3.0),
            ..g
        };
        assert!(bad.validate().is_err());
        let bad = GatherSettings {
            bait_reach_cells: 99,
            ..g
        };
        assert!(bad.validate().is_err());
    }

    /// Hands and an axe both take logs up to the limit (`tree-punch-shake`).
    #[test]
    fn hands_and_an_axe_take_a_tree_s_logs_alike() {
        let g = GatherSettings::default();
        assert_eq!(g.tree_gives(0, false), Ok(()));
        assert_eq!(
            g.tree_gives(1, false),
            Ok(()),
            "hands take more than a branch"
        );
        assert_eq!(g.tree_gives(3, false), Ok(()));
        assert_eq!(g.tree_gives(4, false), Err(TreeSpent::Felled));
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
