//! Stations and recipes (`farming-and-smelting` decisions 6, 7 and 10): a
//! block in the world that turns inputs into outputs over world time.
//!
//! GENERIC on purpose. The forge is the first row of [`STATIONS`]; a cooking
//! pot, an oven or a campfire is another row, more recipes in
//! `assets/config/recipes.ron` and its own art, and nothing here changes.
//!
//! A station's queue is DERIVED like a crop's stage: each job is a recipe and
//! the world time it was queued, and when it starts and finishes follows
//! from those and the jobs before it. Nothing is written while a job runs.
//! Fuel is an ingredient (M3's recommendation): coal is an input, spent when
//! the job is queued.

use crate::goods::Good;
use crate::inventory::{Equipment, Item, Slots, Tier, Tool};
use crate::terrain::Material;
use serde::{Deserialize, Serialize};

/// One station kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StationRow {
    /// Its saved name, in a station record and in `recipes.ron`.
    pub code: &'static str,
    /// The block it is; `None` for the hands, whose "station" is the pack
    /// itself (a recipe made by hand, `tool-crafting`).
    pub material: Option<Material>,
    /// The most jobs it holds at once.
    pub max_jobs: usize,
}

/// Every station. **Append only.**
///
/// `hand` is the hook for recipes made from the pack with no block
/// (`tool-crafting`: planks, rope, a workbench, in a second or two): a
/// recipe whose station is `hand` is queued and collected by the same rules,
/// with its queue kept on the player rather than on a cell. Nothing in this
/// change makes anything by hand yet.
impl StationRow {
    /// Its name: `station.<code>.name` in the locale file.
    pub fn name(&self) -> crate::locale::Localized {
        crate::locale::tr(&format!("station.{}.name", self.code))
    }
}

pub const STATIONS: &[StationRow] = &[
    StationRow {
        code: "forge",
        material: Some(Material::Forge),
        max_jobs: 4,
    },
    StationRow {
        code: "hand",
        material: None,
        max_jobs: 4,
    },
    // `cooking` design S2: the kitchen's first station, also a light.
    StationRow {
        code: "campfire",
        material: Some(Material::Campfire),
        max_jobs: 4,
    },
    // tool-crafting: tools, the rod and a forge (decision 5).
    StationRow {
        code: "workbench",
        material: Some(Material::Workbench),
        max_jobs: 4,
    },
    // `storage-chests` design 1: a store, not a maker. It takes no jobs; F
    // opens its slots beside the pack (`pbd_core::chest`).
    StationRow {
        code: "chest",
        material: Some(Material::Chest),
        max_jobs: 0,
    },
    // `townsfolk` design 41: crushed stone, and gravel from it and sand.
    StationRow {
        code: "grinder",
        material: Some(Material::Grinder),
        max_jobs: 4,
    },
    // furniture design 1: every piece of furniture, and nothing else.
    StationRow {
        code: "carpenter",
        material: Some(Material::Carpenter),
        max_jobs: 4,
    },
    StationRow {
        code: "mixer",
        material: Some(Material::Mixer),
        max_jobs: 4,
    },
    // `survival-mode` design 17: a hide tanned into leather.
    StationRow {
        code: "tanning-rack",
        material: Some(Material::TanningRack),
        max_jobs: 4,
    },
    // `survival-mode` design 15 and 18: the still holds water, not jobs.
    // What it holds and boils is the survival rules' (`survival.ron`
    // `still`); F on it is theirs too.
    StationRow {
        code: "still",
        material: Some(Material::Still),
        max_jobs: 0,
    },
];

/// The record id of the hands' queue (`tool-crafting` decision 4): kept like
/// a station's, under an id no cell's [`record_id`] reaches (those stop at
/// 48 bits).
pub const HAND_RECORD_ID: u64 = u64::MAX;

/// The hands' station row.
pub fn hand() -> &'static StationRow {
    station_by_code("hand").expect("the hand row")
}

/// The station a block is, if it is one.
pub fn station_of(material: Material) -> Option<&'static StationRow> {
    STATIONS.iter().find(|s| s.material == Some(material))
}

pub fn station_by_code(code: &str) -> Option<&'static StationRow> {
    STATIONS.iter().find(|s| s.code == code)
}

/// The record kind a station's queue is stored as.
pub const STATION_KIND: &str = "station";
pub const STATION_SCHEMA: u32 = 1;

/// A station record's id: its cell and layer.
pub fn record_id(cell: u32, layer: u16) -> u64 {
    (u64::from(cell) << 16) | u64::from(layer)
}

/// A tool as `recipes.ron` names it: `tool:<tier>-<tool>`
/// (`tool:iron-pickaxe`), or `tool:<tool>` for the stone tier
/// (`tool:pickaxe`, `tool:rod`). A tool goes to the tool slot, not a stack
/// (`tool-crafting` decision 6).
pub fn tool_named(name: &str) -> Option<(Tool, Tier)> {
    let rest = name.strip_prefix("tool:")?;
    let (tier, tool) = match rest.split_once('-') {
        Some((tier, tool)) => (Tier::ALL.into_iter().find(|t| t.code() == tier)?, tool),
        None => (Tier::Stone, rest),
    };
    let tool = Tool::ALL.into_iter().find(|t| t.code() == tool)?;
    Some((tool, tier))
}

/// An item as `recipes.ron` names it: a good by its code (`coal`), a block
/// as `block:<name>` (`block:ore`), the name the inventory gives it, or a
/// tool as `tool:<name>` ([`tool_named`]), which as an item is the tool
/// without its tier.
pub fn item_named(name: &str) -> Option<Item> {
    if name.starts_with("tool:") {
        return tool_named(name).map(|(tool, _)| Item::Tool(tool));
    }
    if let Some(block) = name.strip_prefix("block:") {
        return Material::ALL
            .into_iter()
            .find(|m| crate::inventory::material_id(*m) == block)
            .map(Item::Block);
    }
    Good::by_code(name).map(Item::Good)
}

/// One recipe as the file writes it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeEntry {
    /// Its saved name, in a station's queue. Never renamed once shipped.
    pub code: String,
    /// The station code it is made at.
    pub station: String,
    /// World seconds from start to done.
    pub secs: f32,
    /// What it takes, fuel included: `(item, count)`.
    pub inputs: Vec<(String, u16)>,
    /// What it gives.
    pub outputs: Vec<(String, u16)>,
    /// Made only in a Survival world (`survival-mode` design 16 to 19): an
    /// Explorer world's book leaves it out ([`RecipeBook::for_mode`]).
    /// Written `survival: true`; left out, false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub survival: bool,
}

/// `assets/config/recipes.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeFile {
    pub recipes: Vec<RecipeEntry>,
}

impl Default for RecipeFile {
    /// M3's forge: mixed ore at its fixed split, and bronze; then the hands'
    /// and the workbench's (`tool-crafting` decisions 4 and 5).
    fn default() -> Self {
        let pair = |name: &str, n| (name.to_string(), n);
        let recipe = |code: &str,
                      station: &str,
                      secs: f32,
                      inputs: &[(&str, u16)],
                      outputs: &[(&str, u16)]| RecipeEntry {
            code: code.into(),
            station: station.into(),
            secs,
            inputs: inputs.iter().map(|&(n, k)| pair(n, k)).collect(),
            outputs: outputs.iter().map(|&(n, k)| pair(n, k)).collect(),
            survival: false,
        };
        // A Survival world's own (`survival-mode` design 15 to 18).
        let survival = |entry: RecipeEntry| RecipeEntry {
            survival: true,
            ..entry
        };
        let stone_tool = |tool: &str| {
            recipe(
                &format!("stone-{tool}"),
                "workbench",
                3.0,
                &[("stone", 3), ("planks", 2), ("rope", 1)],
                &[(&format!("tool:stone-{tool}"), 1)],
            )
        };
        let metal_tool = |tier: &str, tool: &str| {
            recipe(
                &format!("{tier}-{tool}"),
                "workbench",
                4.0,
                &[(&format!("{tier}-ingot"), 3), ("planks", 2)],
                &[(&format!("tool:{tier}-{tool}"), 1)],
            )
        };
        let mut file = Self {
            recipes: vec![
                RecipeEntry {
                    code: "mixed-ore".into(),
                    station: "forge".into(),
                    secs: 8.0,
                    inputs: vec![pair("block:ore", 4), pair("coal", 2)],
                    outputs: vec![
                        pair("copper-ingot", 2),
                        pair("iron-ingot", 1),
                        pair("tin-ore", 1),
                    ],
                    survival: false,
                },
                RecipeEntry {
                    code: "bronze".into(),
                    station: "forge".into(),
                    secs: 5.0,
                    inputs: vec![pair("copper-ingot", 1), pair("tin-ore", 1), pair("coal", 1)],
                    outputs: vec![pair("bronze-ingot", 2)],
                    survival: false,
                },
            ],
        };
        // ---- tool-crafting: by hand, then at the workbench (M3 section 4).
        file.recipes.extend([
            recipe("planks", "hand", 1.0, &[("log", 1)], &[("planks", 4)]),
            recipe("rope", "hand", 2.0, &[("fibre", 3)], &[("rope", 1)]),
            // townsfolk design 41: survey stakes, in gravel's old row (the
            // hand recipe for gravel is retired: `RETIRED_RECIPES`).
            recipe(
                "survey-stakes",
                "hand",
                1.0,
                &[("planks", 1), ("fibre", 1)],
                &[("survey-stakes", 4)],
            ),
            recipe(
                "workbench",
                "hand",
                4.0,
                &[("planks", 6)],
                &[("block:workbench", 1)],
            ),
            stone_tool("pickaxe"),
            stone_tool("shovel"),
            // The first tool, by hand (`starter-missions` T.2, the tree's
            // trunk): loose stone, fibre and a branch.
            recipe(
                "stone-axe",
                "hand",
                3.0,
                &[("stone", 3), ("fibre", 2), ("log", 1)],
                &[("tool:stone-axe", 1)],
            ),
            // A pen's pieces and a torch (T.2): made, where the kit used to
            // deal them.
            recipe("fence", "workbench", 2.0, &[("planks", 1)], &[("fence", 2)]),
            recipe(
                "gate",
                "workbench",
                3.0,
                &[("planks", 2), ("rope", 1)],
                &[("gate", 1)],
            ),
            recipe(
                "torch",
                "hand",
                1.0,
                &[("planks", 1), ("fibre", 1)],
                &[("block:torch", 2)],
            ),
            recipe(
                "rod",
                "workbench",
                3.0,
                &[("planks", 2), ("rope", 2), ("fibre", 2)],
                &[("tool:rod", 1)],
            ),
            recipe(
                "forge",
                "workbench",
                6.0,
                &[("stone", 8), ("block:stone", 4)],
                &[("block:forge", 1)],
            ),
            recipe(
                "bucket",
                "workbench",
                3.0,
                &[("planks", 3), ("rope", 1)],
                &[("bucket", 1)],
            ),
            recipe(
                "incubator",
                "workbench",
                4.0,
                &[("planks", 4), ("rope", 2)],
                &[("incubator", 1)],
            ),
            // player-building design B8: a door, planks and an iron fitting
            // (its strap hinges and pull).
            recipe(
                "door",
                "workbench",
                4.0,
                &[("planks", 4), ("iron-ingot", 1)],
                &[("door", 1)],
            ),
            metal_tool("bronze", "pickaxe"),
            metal_tool("bronze", "shovel"),
            metal_tool("bronze", "axe"),
            metal_tool("iron", "pickaxe"),
            metal_tool("iron", "shovel"),
            metal_tool("iron", "axe"),
            // small-boats design 3: the boats. (The rowboat was here until
            // `retire-rowboat-and-cog`: `RETIRED_RECIPES`.)
            recipe(
                "inner-tube",
                "workbench",
                4.0,
                // Leather alone (small-boats design 2e).
                &[("leather", 3)],
                &[("inner-tube", 1)],
            ),
            recipe(
                "raft",
                "workbench",
                6.0,
                &[("log", 6), ("rope", 4)],
                &[("raft", 1)],
            ),
            // `storage-chests` design 1: a chest of twenty slots (`wood-chest`:
            // the carpenter's `chest` is the furniture piece).
            recipe(
                "wood-chest",
                "workbench",
                4.0,
                &[("planks", 8)],
                &[("block:chest", 1)],
            ),
            // townsfolk design 41: the gravel stations and what they make.
            recipe(
                "grinder",
                "workbench",
                6.0,
                &[("stone", 10), ("planks", 6), ("iron-ingot", 2)],
                &[("block:grinder", 1)],
            ),
            recipe(
                "mixer",
                "workbench",
                6.0,
                &[("planks", 8), ("rope", 2), ("iron-ingot", 1), ("bucket", 1)],
                &[("block:mixer", 1)],
            ),
            recipe(
                "crushed-stone",
                "grinder",
                4.0,
                &[("stone", 2)],
                &[("crushed-stone", 3)],
            ),
            recipe(
                "mixed-gravel",
                "mixer",
                6.0,
                &[("crushed-stone", 3), ("block:sand", 1)],
                &[("gravel", 4)],
            ),
            // land-transport design 10: the wheel and the two carts.
            recipe(
                "wheel",
                "workbench",
                4.0,
                &[("planks", 4), ("iron-ingot", 1)],
                &[("wheel", 1)],
            ),
            recipe(
                "hand-cart",
                "workbench",
                6.0,
                &[("wheel", 2), ("planks", 8), ("rope", 2)],
                &[("hand-cart", 1)],
            ),
            recipe(
                "horse-cart",
                "workbench",
                8.0,
                &[("wheel", 2), ("planks", 14), ("rope", 4), ("leather", 2)],
                &[("horse-cart", 1)],
            ),
            // furniture design 1 and 2: the carpenter's bench, and the
            // Furniture branch made at it and nowhere else.
            RecipeEntry {
                code: "carpenter-bench".into(),
                station: "workbench".into(),
                secs: 6.0,
                inputs: vec![pair("planks", 8), pair("log", 2), pair("iron-ingot", 1)],
                // The block's name is its English name (`material_id`).
                outputs: vec![(
                    format!(
                        "block:{}",
                        crate::inventory::material_id(Material::Carpenter)
                    ),
                    1,
                )],
                survival: false,
            },
            recipe("chair", "carpenter", 3.0, &[("planks", 3)], &[("chair", 1)]),
            recipe("stool", "carpenter", 2.0, &[("planks", 2)], &[("stool", 1)]),
            recipe("bench", "carpenter", 3.0, &[("planks", 4)], &[("bench", 1)]),
            recipe("table", "carpenter", 4.0, &[("planks", 5)], &[("table", 1)]),
            recipe(
                "bed",
                "carpenter",
                6.0,
                &[("planks", 5), ("wool", 3)],
                &[("bed", 1)],
            ),
            recipe(
                "drawers",
                "carpenter",
                5.0,
                &[("planks", 6)],
                &[("drawers", 1)],
            ),
            recipe(
                "closet",
                "carpenter",
                6.0,
                &[("planks", 8)],
                &[("closet", 1)],
            ),
            recipe(
                "chest",
                "carpenter",
                4.0,
                &[("planks", 5), ("rope", 1)],
                &[("chest", 1)],
            ),
            recipe(
                "banded-chest",
                "carpenter",
                5.0,
                &[("planks", 5), ("iron-ingot", 2)],
                &[("banded-chest", 1)],
            ),
            recipe("crate", "carpenter", 3.0, &[("planks", 3)], &[("crate", 1)]),
            // building-planner design 3 and 6: the drafting table and the
            // mallet.
            RecipeEntry {
                code: "drafting-table".into(),
                station: "workbench".into(),
                secs: 5.0,
                inputs: vec![pair("planks", 6), pair("rope", 2)],
                outputs: vec![(
                    format!(
                        "block:{}",
                        crate::inventory::material_id(Material::DraftingTable)
                    ),
                    1,
                )],
                survival: false,
            },
            recipe(
                "mallet",
                "workbench",
                2.0,
                &[("log", 1), ("stone", 1)],
                &[("mallet", 1)],
            ),
        ]);
        // sea-gathering design 6: the grinder crushes each sea piece into
        // fertiliser, at its own yield.
        file.recipes.extend(
            crate::sea_gather::SeaPiece::ALL
                .into_iter()
                .filter_map(|piece| {
                    let (n, out) = piece.crush()?;
                    Some(RecipeEntry {
                        code: piece.crush_recipe(),
                        station: "grinder".into(),
                        secs: 3.0,
                        inputs: vec![pair(piece.code(), n)],
                        outputs: vec![pair(crate::sea_gather::FERTILIZER, out)],
                        survival: false,
                    })
                }),
        );
        // ---- survival-mode design 15 to 18 (task 4.10): Survival's own, in
        // the file's order. The weapons and what they need, what is worn,
        // the bedroll and the two stations.
        let block = |m: Material| format!("block:{}", crate::inventory::material_id(m));
        let rack = block(Material::TanningRack);
        let still = block(Material::Still);
        file.recipes.extend(
            [
                recipe("straw", "hand", 1.0, &[("fibre", 2)], &[("straw", 1)]),
                recipe(
                    "stone-spear",
                    "hand",
                    3.0,
                    &[("log", 1), ("stone", 2), ("fibre", 2)],
                    &[("stone-spear", 1)],
                ),
                recipe(
                    "stone-arrows",
                    "hand",
                    2.0,
                    &[("planks", 1), ("stone", 1), ("fibre", 1)],
                    &[("stone-arrow", 4)],
                ),
                recipe(
                    "bone-meal",
                    "hand",
                    1.0,
                    &[("bone", 1)],
                    &[("fertilizer", 2)],
                ),
                recipe(
                    "straw-hat",
                    "hand",
                    3.0,
                    &[("wheat", 3)],
                    &[("straw-hat", 1)],
                ),
                recipe(
                    "fibre-hat",
                    "hand",
                    3.0,
                    &[("fibre", 6)],
                    &[("straw-hat", 1)],
                ),
                recipe(
                    "bedroll",
                    "hand",
                    4.0,
                    &[("hide", 2), ("wool", 1)],
                    &[("bedroll", 1)],
                ),
                recipe(
                    "tanning-rack",
                    "workbench",
                    5.0,
                    &[("planks", 4), ("rope", 2)],
                    &[(rack.as_str(), 1)],
                ),
                recipe(
                    "tan-hide",
                    "tanning-rack",
                    60.0,
                    &[("hide", 1)],
                    &[("leather", 1)],
                ),
                recipe(
                    "still",
                    "workbench",
                    5.0,
                    &[("stone", 6), ("planks", 2), ("rope", 1)],
                    &[(still.as_str(), 1)],
                ),
                recipe(
                    "wool-cloak",
                    "workbench",
                    4.0,
                    &[("wool", 4), ("rope", 1)],
                    &[("wool-cloak", 1)],
                ),
                recipe(
                    "bow",
                    "workbench",
                    4.0,
                    &[("planks", 2), ("rope", 2)],
                    &[("bow", 1)],
                ),
                recipe(
                    "bone-arrows",
                    "workbench",
                    2.0,
                    &[("planks", 1), ("bone", 1), ("feather", 2)],
                    &[("bone-arrow", 4)],
                ),
                recipe(
                    "bronze-arrows",
                    "workbench",
                    3.0,
                    &[("bronze-ingot", 1), ("planks", 1), ("feather", 2)],
                    &[("bronze-arrow", 8)],
                ),
                recipe(
                    "bronze-spear",
                    "workbench",
                    4.0,
                    &[("bronze-ingot", 1), ("planks", 2)],
                    &[("bronze-spear", 1)],
                ),
                recipe(
                    "iron-spear",
                    "workbench",
                    4.0,
                    &[("iron-ingot", 1), ("planks", 2)],
                    &[("iron-spear", 1)],
                ),
                recipe(
                    "bronze-hunting-axe",
                    "workbench",
                    4.0,
                    &[("bronze-ingot", 2), ("planks", 1), ("leather", 1)],
                    &[("bronze-hunting-axe", 1)],
                ),
                recipe(
                    "iron-hunting-axe",
                    "workbench",
                    4.0,
                    &[("iron-ingot", 2), ("planks", 1), ("leather", 1)],
                    &[("iron-hunting-axe", 1)],
                ),
                recipe(
                    "bronze-sword",
                    "workbench",
                    5.0,
                    &[("bronze-ingot", 3), ("planks", 1), ("leather", 1)],
                    &[("bronze-sword", 1)],
                ),
                recipe(
                    "iron-sword",
                    "workbench",
                    5.0,
                    &[("iron-ingot", 3), ("planks", 1), ("leather", 1)],
                    &[("iron-sword", 1)],
                ),
            ]
            .into_iter()
            .map(survival),
        );
        file
    }
}

/// Recipes this build has retired, by code, and the inputs each took as it
/// shipped (`retire-rowboat-and-cog` decisions 2 and 3): what a saved job
/// for one, a good it made, or a craft launched from that good gives back.
/// Never edited: a refund is what the player paid.
pub const RETIRED_RECIPES: [(&str, &[(&str, u16)]); 2] = [
    ("rowboat", &[("planks", 16), ("rope", 2)]),
    // `townsfolk` design 41: gravel is made at the mixer now.
    ("gravel", &[("stone", 2)]),
];

/// The inputs a retired recipe took, resolved; `None` for a code that was
/// never retired.
pub fn retired_inputs(code: &str) -> Option<Vec<(Item, u16)>> {
    let (_, inputs) = RETIRED_RECIPES.iter().find(|(c, _)| *c == code)?;
    inputs
        .iter()
        .map(|&(item, n)| item_named(item).map(|i| (i, n)))
        .collect()
}

/// Give all of `items` into `slots`, or none of them: whether they went.
pub fn give_all(slots: &mut Slots, items: &[(Item, u16)]) -> bool {
    let mut after = slots.clone();
    if items.iter().all(|&(item, n)| after.give(item, n) == 0) {
        *slots = after;
        true
    } else {
        false
    }
}

/// A recipe with its items resolved.
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    /// Its saved name; its plaque's words are `recipe.<code>.name`.
    pub code: String,
    pub station: &'static StationRow,
    pub secs: f32,
    pub inputs: Vec<(Item, u16)>,
    /// What it gives into the slots.
    pub outputs: Vec<(Item, u16)>,
    /// The tools it gives to the tool slot, with their tiers
    /// (`tool-crafting`): a `tool:` output, not in `outputs`.
    pub tools: Vec<(Tool, Tier)>,
    /// Made only in a Survival world (`RecipeEntry::survival`).
    pub survival: bool,
}

impl Recipe {
    /// What the panel's plaque says.
    pub fn name(&self) -> crate::locale::Localized {
        crate::locale::tr(&format!("recipe.{}.name", self.code))
    }

    /// Everything it gives, tools as items, for a panel to draw.
    pub fn shown_outputs(&self) -> Vec<(Item, u16)> {
        let mut all = self.outputs.clone();
        all.extend(self.tools.iter().map(|&(tool, _)| (Item::Tool(tool), 1)));
        all
    }
}

/// Every recipe, resolved and checked.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RecipeBook {
    /// Every recipe this build knows, Survival's own among them: what a
    /// mission file and a saved queue are resolved against.
    pub recipes: Vec<Recipe>,
    /// Whether the world it serves is a Survival world (`survival-mode`
    /// design 16 and 19): only then does a station offer Survival's own
    /// recipes ([`RecipeBook::at`]). False, an Explorer world's, unless the
    /// world says otherwise.
    pub survival: bool,
}

impl RecipeFile {
    /// Resolve every name, or say which one is wrong.
    pub fn resolve(&self) -> Result<RecipeBook, String> {
        let mut recipes: Vec<Recipe> = Vec::new();
        for entry in &self.recipes {
            let at = &entry.code;
            if !crate::records::is_word(at) {
                return Err(format!("recipe code {at:?} is not a word"));
            }
            if recipes.iter().any(|r| &r.code == at) {
                return Err(format!("recipe {at} twice"));
            }
            let station = station_by_code(&entry.station)
                .ok_or_else(|| format!("{at}: no station {:?}", entry.station))?;
            if !(entry.secs.is_finite() && entry.secs > 0.0) {
                return Err(format!("{at}: secs must be above zero"));
            }
            let items = |list: &[(String, u16)]| -> Result<Vec<(Item, u16)>, String> {
                if list.is_empty() {
                    return Err(format!("{at}: an empty list"));
                }
                list.iter()
                    .map(|(name, n)| {
                        let item =
                            item_named(name).ok_or_else(|| format!("{at}: no item {name:?}"))?;
                        if *n == 0 || *n > item.stack_limit() {
                            return Err(format!("{at}: {n} {name} is not a stack"));
                        }
                        Ok((item, *n))
                    })
                    .collect()
            };
            let mut tools = Vec::new();
            let mut outputs = Vec::new();
            for (name, n) in &entry.outputs {
                if name.starts_with("tool:") {
                    let tool = tool_named(name).ok_or_else(|| format!("{at}: no tool {name:?}"))?;
                    if *n != 1 {
                        return Err(format!("{at}: a tool is made one at a time"));
                    }
                    tools.push(tool);
                } else {
                    outputs.push((name.clone(), *n));
                }
            }
            let outputs = if outputs.is_empty() && !tools.is_empty() {
                Vec::new()
            } else {
                items(&outputs)?
            };
            recipes.push(Recipe {
                code: entry.code.clone(),
                station,
                secs: entry.secs,
                inputs: items(&entry.inputs)?,
                outputs,
                tools,
                survival: entry.survival,
            });
        }
        Ok(RecipeBook {
            recipes,
            survival: false,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.resolve().map(|_| ())
    }
}

impl RecipeBook {
    pub fn get(&self, code: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.code == code)
    }

    /// A station's recipes this world offers, in file order: Survival's
    /// own only in a Survival world.
    pub fn at(&self, station: &str) -> impl Iterator<Item = &Recipe> {
        self.offered().filter(move |r| r.station.code == station)
    }

    /// Every recipe this world offers, in file order (`survival-mode`
    /// design 16 and 19): all but Survival's own in an Explorer world.
    pub fn offered(&self) -> impl Iterator<Item = &Recipe> {
        let survival = self.survival;
        self.recipes.iter().filter(move |r| survival || !r.survival)
    }

    /// Whether this world offers the recipe with this code.
    pub fn offers(&self, code: &str) -> bool {
        self.get(code).is_some_and(|r| self.survival || !r.survival)
    }

    /// The same book for a world of this mode: the world opening sets it
    /// (an Explorer world's is the default).
    pub fn for_mode(&self, survival: bool) -> RecipeBook {
        RecipeBook {
            recipes: self.recipes.clone(),
            survival,
        }
    }
}

/// One queued job.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Job {
    pub recipe: String,
    pub queued_s: f64,
}

/// A station record's body: which station, and its queue, oldest first.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct StationState {
    pub station: String,
    pub jobs: Vec<Job>,
}

/// Why a queue or a collect did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The slots do not hold every input.
    Short,
    /// The queue is full.
    Full,
    /// Not this station's recipe.
    Elsewhere,
}

impl StationState {
    pub fn new(station: &StationRow) -> Self {
        Self {
            station: station.code.to_string(),
            jobs: Vec::new(),
        }
    }

    /// When each job starts and finishes, world seconds: one after another,
    /// each from when it was queued or the last finished, whichever is
    /// later. A job whose recipe this build lacks takes no time.
    pub fn schedule(&self, book: &RecipeBook) -> Vec<(f64, f64)> {
        let mut free = f64::NEG_INFINITY;
        self.jobs
            .iter()
            .map(|job| {
                let start = job.queued_s.max(free);
                let secs = book.get(&job.recipe).map_or(0.0, |r| f64::from(r.secs));
                free = start + secs;
                (start, free)
            })
            .collect()
    }

    /// The head job's progress at `now_s`, 0..=1, or `None` with nothing
    /// queued.
    pub fn head_progress(&self, book: &RecipeBook, now_s: f64) -> Option<f32> {
        let (start, end) = *self.schedule(book).first()?;
        if end <= start {
            return Some(1.0);
        }
        Some(((now_s - start) / (end - start)).clamp(0.0, 1.0) as f32)
    }

    /// Whether any job is running or waiting.
    pub fn busy(&self) -> bool {
        !self.jobs.is_empty()
    }

    /// Whether a job is being worked at `now_s` (started and not done): what
    /// lights the forge's fire.
    pub fn working(&self, book: &RecipeBook, now_s: f64) -> bool {
        self.schedule(book)
            .iter()
            .any(|(start, end)| *start <= now_s && now_s < *end)
    }

    /// Queue `recipe` at `now_s`, taking its inputs from `slots`. All or
    /// nothing: refused, with the slots untouched, when they lack an input
    /// or the queue is full.
    pub fn queue(&mut self, slots: &mut Slots, recipe: &Recipe, now_s: f64) -> Result<(), Refused> {
        self.queue_for(slots, None, None, recipe, now_s)
    }

    /// Queue as [`StationState::queue`]. Any tool may be made as often as
    /// the inputs allow, owned or queued already: the first goes to hand,
    /// the rest to the pack, ready for when tools wear out (owner,
    /// 2026-11-22; `tool-crafting`, "A made tool goes to hand").
    pub fn queue_for(
        &mut self,
        slots: &mut Slots,
        _equipment: Option<&Equipment>,
        _book: Option<&RecipeBook>,
        recipe: &Recipe,
        now_s: f64,
    ) -> Result<(), Refused> {
        let row = station_by_code(&self.station).ok_or(Refused::Elsewhere)?;
        if recipe.station.code != row.code {
            return Err(Refused::Elsewhere);
        }
        if self.jobs.len() >= row.max_jobs {
            return Err(Refused::Full);
        }
        let mut after = slots.clone();
        for &(item, n) in &recipe.inputs {
            if !after.take_item(item, n) {
                return Err(Refused::Short);
            }
        }
        *slots = after;
        self.jobs.push(Job {
            recipe: recipe.code.clone(),
            queued_s: now_s,
        });
        Ok(())
    }

    /// Give the outputs of every finished job at the head of the queue into
    /// `slots` and take it off, stopping at the first that is not finished
    /// or whose outputs do not all fit. Returns how many were collected.
    pub fn collect(&mut self, slots: &mut Slots, book: &RecipeBook, now_s: f64) -> usize {
        self.collect_into(slots, None, book, now_s).len()
    }

    /// Collect as [`StationState::collect`], giving a recipe's tools to
    /// `equipment` (`tool-crafting` decision 6). A tool that is no longer an
    /// upgrade is given nothing and the job still comes off: a finished job
    /// is never stuck. Returns the jobs collected, oldest first.
    pub fn collect_into(
        &mut self,
        slots: &mut Slots,
        mut equipment: Option<&mut Equipment>,
        book: &RecipeBook,
        now_s: f64,
    ) -> Vec<Job> {
        let mut done = Vec::new();
        while let Some(&(_, end)) = self.schedule(book).first() {
            if end > now_s {
                break;
            }
            let mut after = slots.clone();
            let code = &self.jobs[0].recipe;
            let fits = match (book.get(code), retired_inputs(code)) {
                (Some(recipe), _) => {
                    recipe
                        .outputs
                        .iter()
                        .all(|&(item, n)| after.give(item, n) == 0)
                        // A tool already owned as well or better is a spare,
                        // and goes to the pack as any output does.
                        && recipe.tools.iter().all(|&(tool, tier)| {
                            match equipment.as_deref() {
                                Some(kit) if kit.would_take(tool, tier).is_err() => {
                                    after.give(Item::Tool(tool), 1) == 0
                                }
                                _ => true,
                            }
                        })
                }
                // A retired recipe gives back what it took
                // (`retire-rowboat-and-cog` decision 3).
                (None, Some(inputs)) => give_all(&mut after, &inputs),
                // A recipe this build lacks: kept, never thrown away.
                (None, None) => false,
            };
            if !fits {
                break;
            }
            *slots = after;
            if let (Some(equipment), Some(recipe)) =
                (equipment.as_deref_mut(), book.get(&self.jobs[0].recipe))
            {
                for &(tool, tier) in &recipe.tools {
                    // New or better: owned and put in hand at once. Not
                    // better: it went to the pack above.
                    if equipment.grant(tool, tier).is_ok() {
                        equipment.hold(tool);
                    }
                }
            }
            done.push(self.jobs.remove(0));
            // The next job started when this one finished, not when it was
            // queued: carried on it, so taking the head off does not move it.
            if let Some(next) = self.jobs.first_mut() {
                next.queued_s = next.queued_s.max(end);
            }
        }
        done
    }
}

/// What a dug ore cell yields (decision 10): coal where the cell and layer
/// hash to a coal seam, one in three; mixed ore elsewhere. An ore block the
/// player placed (`placed`) is mixed ore again, so placing and digging cannot
/// make coal.
pub fn ore_yield(cell: u32, layer: u16, placed: bool) -> Item {
    if placed {
        return Item::Block(Material::Ore);
    }
    let mut v = (u64::from(cell) << 16 | u64::from(layer)) ^ 0xc0a1_5ea3;
    v ^= v >> 33;
    v = v.wrapping_mul(0xff51_afd7_ed55_8ccd);
    v ^= v >> 33;
    if v % 3 == 0 {
        Item::Good(Good::COAL)
    } else {
        Item::Block(Material::Ore)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every recipe, the stations' and the campfire's, has its plaque's
    /// words in the locale file, by its code.
    #[test]
    fn every_recipe_has_its_name() {
        let english = crate::locale::Locale::english();
        let codes = RecipeFile::default()
            .recipes
            .into_iter()
            .chain(crate::cooking::CookingFile::default().recipes)
            .map(|r| r.code);
        for code in codes {
            let key = format!("recipe.{code}.name");
            assert!(english.get(&key).is_some(), "{key}");
        }
    }

    fn book() -> RecipeBook {
        RecipeFile::default().resolve().unwrap()
    }

    /// `player-building` design B8: a door is made at the workbench from 4
    /// planks and 1 iron ingot, and gives one door.
    #[test]
    fn a_door_is_planks_and_an_iron_fitting_at_the_workbench() {
        let book = book();
        let door = book.get("door").expect("the door's recipe");
        assert_eq!(door.station.code, "workbench");
        assert_eq!(
            door.inputs,
            vec![
                (Item::Good(Good::PLANKS), 4),
                (Item::Good(Good::IRON_INGOT), 1)
            ]
        );
        assert_eq!(door.outputs, vec![(Item::Good(Good::DOOR), 1)]);
        assert!(door.tools.is_empty());
    }

    fn forge() -> StationState {
        StationState::new(&STATIONS[0])
    }

    fn stocked(ore: u16, coal: u16) -> Slots {
        let mut slots = Slots::new();
        slots.give(Item::Block(Material::Ore), ore);
        slots.give(Item::Good(Good::COAL), coal);
        slots
    }

    /// The shipped file is the defaults written out, and resolves.
    #[test]
    fn the_shipped_recipes_resolve_and_agree_with_the_defaults() {
        let file: RecipeFile =
            ron::from_str(include_str!("../../../assets/config/recipes.ron")).unwrap();
        assert_eq!(file, RecipeFile::default());
        // An Explorer world's book (the default): Survival's own are
        // counted in `survival_recipes_are_offered_only_in_a_survival_world`.
        let book = file.resolve().unwrap();
        assert_eq!(book.at("forge").count(), 2);
        // Planks, rope, survey stakes (`townsfolk` design 41, in gravel's
        // old row), the workbench, the stone axe and torches by hand (`starter-missions` T.2); at the
        // bench the rest, fences and gates.
        assert_eq!(book.at("hand").count(), 6);
        assert_eq!(
            book.at("workbench").count(),
            26,
            "with the drafting table and the mallet (building-planner), the bucket (water-flow E11), the fence and gate (starter-missions T.2), the incubator (animal-husbandry E4), the tube and raft (small-boats; the rowboat retired), the door (player-building B8), the grinder and mixer (townsfolk design 41), the wheel and the two carts (land-transport), the carpenter's bench (furniture design 1), and the wood chest (storage-chests)"
        );
        assert_eq!(
            book.get("wood-chest").map(|r| r.outputs.clone()),
            Some(vec![(Item::Block(Material::Chest), 1)])
        );
        assert_eq!(
            station_of(Material::Chest).map(|s| (s.code, s.max_jobs)),
            Some(("chest", 0))
        );
        assert!(book.get("rowboat").is_none(), "the rowboat is retired");
        // `townsfolk` design 41: gravel comes from the mixer, never the hand;
        // crushed stone from the grinder. The hand recipe's code is retired.
        assert!(book.get("gravel").is_none(), "gravel by hand is retired");
        assert!(retired_inputs("gravel").is_some());
        // Crushed stone, and each sea piece crushed (`sea-gathering` 6).
        assert_eq!(book.at("grinder").count(), 13);
        assert_eq!(book.at("mixer").count(), 1);
        let mixed = book.get("mixed-gravel").unwrap();
        assert_eq!(mixed.station.code, "mixer");
        assert_eq!(
            mixed.outputs,
            vec![(Item::Good(Good::GRAVEL), 4)],
            "3 crushed stone and a sand block make 4 gravel"
        );
        assert_eq!(
            book.get("crushed-stone").unwrap().inputs,
            vec![(Item::Good(Good::LOOSE_STONE), 2)]
        );
        assert_eq!(
            station_of(Material::Grinder).map(|s| s.code),
            Some("grinder")
        );
        assert_eq!(item_named("block:ore"), Some(Item::Block(Material::Ore)));
        assert_eq!(item_named("coal"), Some(Item::Good(Good::COAL)));
        assert_eq!(item_named("unobtainium"), None);
        assert_eq!(
            item_named("tool:pickaxe"),
            Some(Item::Tool(crate::inventory::Tool::Pickaxe))
        );
        assert_eq!(station_by_code("hand").map(|s| s.material), Some(None));
        assert_eq!(book.at("carpenter").count(), 10, "the Furniture branch");
    }

    /// `survival-mode` task 4.10: the goods, recipes and stations of design
    /// 15 to 18. Every one resolves to real things and is made where the
    /// design says; an Explorer world's book offers none of them and keeps
    /// every other recipe in its order; the two new stations are blocks
    /// with their own codes, and every recipe a Survival mission unlocks is
    /// one of these.
    #[test]
    fn survival_recipes_are_offered_only_in_a_survival_world() {
        let all = book().for_mode(true);
        let survival: Vec<&Recipe> = all.recipes.iter().filter(|r| r.survival).collect();
        let codes: Vec<&str> = survival.iter().map(|r| r.code.as_str()).collect();
        for code in [
            "straw",
            "stone-spear",
            "stone-arrows",
            "bone-meal",
            "straw-hat",
            "fibre-hat",
            "bedroll",
            "tanning-rack",
            "tan-hide",
            "still",
            "wool-cloak",
            "bow",
            "bone-arrows",
            "bronze-arrows",
            "bronze-spear",
            "iron-spear",
            "bronze-hunting-axe",
            "iron-hunting-axe",
            "bronze-sword",
            "iron-sword",
        ] {
            assert!(codes.contains(&code), "{code}");
        }
        let at = |code: &str| all.get(code).unwrap();
        assert_eq!(at("straw").station.code, "hand");
        assert_eq!(at("stone-spear").station.code, "hand");
        assert_eq!(
            at("stone-spear").inputs,
            vec![
                (Item::Good(Good::LOG), 1),
                (Item::Good(Good::LOOSE_STONE), 2),
                (Item::Good(Good::FIBRE), 2),
            ]
        );
        assert_eq!(
            at("stone-arrows").outputs,
            vec![(Item::Good(Good::STONE_ARROW), 4)]
        );
        assert_eq!(
            at("bronze-arrows").outputs,
            vec![(Item::Good(Good::BRONZE_ARROW), 8)]
        );
        for code in [
            "bow",
            "bronze-sword",
            "iron-hunting-axe",
            "wool-cloak",
            "still",
        ] {
            assert_eq!(at(code).station.code, "workbench", "{code}");
        }
        let tan = at("tan-hide");
        assert_eq!(tan.station.code, "tanning-rack");
        assert_eq!(tan.secs, 60.0);
        assert_eq!(tan.inputs, vec![(Item::Good(Good::HIDE), 1)]);
        assert_eq!(
            tan.outputs,
            vec![(Item::Good(Good::by_code("leather").unwrap()), 1)]
        );
        assert_eq!(
            at("tanning-rack").outputs,
            vec![(Item::Block(Material::TanningRack), 1)]
        );
        assert_eq!(at("still").outputs, vec![(Item::Block(Material::Still), 1)]);
        assert_eq!(
            station_of(Material::TanningRack).map(|s| s.code),
            Some("tanning-rack")
        );
        assert_eq!(
            station_of(Material::Still).map(|s| (s.code, s.max_jobs)),
            Some(("still", 0))
        );
        for m in [Material::TanningRack, Material::Still] {
            let code = crate::terrain::material_code(m);
            assert_eq!(crate::terrain::material_of(code), Some(m));
            assert!((48..64).contains(&code), "{m:?} {code}");
        }
        // Each weapon and worn thing is one to a slot; arrows stack.
        for good in [
            Good::STONE_SPEAR,
            Good::BOW,
            Good::BRONZE_SWORD,
            Good::IRON_HUNTING_AXE,
            Good::STRAW,
            Good::STRAW_HAT,
            Good::WOOL_CLOAK,
        ] {
            assert_eq!(good.stack_limit(), 1, "{}", good.code());
        }
        assert!(Good::STONE_ARROW.stack_limit() >= 8);
        // Explorer (the default book): none of them at any station, and
        // every other recipe where it was; the book still knows them, so a
        // mission file resolves the same in both.
        let explorer = book();
        assert!(!explorer.survival);
        assert!(explorer.offered().all(|r| !r.survival));
        for code in &codes {
            assert!(!explorer.offers(code), "{code} in an Explorer world");
            assert!(all.offers(code), "{code} in a Survival world");
            assert!(explorer.get(code).is_some());
        }
        for row in STATIONS {
            assert!(
                explorer.at(row.code).all(|r| !r.survival),
                "{} offers a Survival recipe in Explorer",
                row.code
            );
        }
        let older: Vec<&str> = all
            .recipes
            .iter()
            .filter(|r| !r.survival)
            .map(|r| r.code.as_str())
            .collect();
        let kept: Vec<&str> = explorer.offered().map(|r| r.code.as_str()).collect();
        assert_eq!(older, kept);
        // And with the kitchen: the stew is Survival's too.
        let kitchen = crate::cooking::CookingFile::default().resolve().unwrap();
        let stew = kitchen
            .recipes
            .iter()
            .find(|r| r.code == "meat-stew")
            .unwrap();
        assert!(stew.survival);
        assert_eq!(
            stew.outputs,
            vec![
                (Item::Good(Good::MEAT_STEW), 1),
                (Item::Good(Good::BUCKET), 1)
            ]
        );
        assert!(kitchen.dish(Good::MEAT_STEW).is_some());
    }

    /// Every recipe the missions file unlocks in a Survival world names a
    /// recipe of this book (`survival-mode` design 18's unlocks).
    #[test]
    fn every_survival_unlock_in_the_missions_file_is_a_recipe() {
        let mut all = book();
        all.recipes.extend(
            crate::cooking::CookingFile::default()
                .resolve()
                .unwrap()
                .recipes,
        );
        for code in [
            "straw",
            "still",
            "straw-hat",
            "meat-stew",
            "tanning-rack",
            "bow",
            "stone-arrows",
            "bone-arrows",
            "bronze-hunting-axe",
            "bronze-sword",
            "bronze-spear",
            "bronze-arrows",
            "iron-sword",
            "iron-spear",
            "iron-hunting-axe",
            "bedroll",
            "wool-cloak",
        ] {
            assert!(
                all.get(code).is_some_and(|r| r.survival),
                "{code} is a Survival recipe"
            );
        }
    }

    #[test]
    fn a_wrong_name_is_refused_by_name() {
        let mut file = RecipeFile::default();
        file.recipes[0].inputs[1].0 = "cole".into();
        assert!(file.resolve().unwrap_err().contains("cole"));
        let mut file = RecipeFile::default();
        file.recipes[1].station = "oven".into();
        assert!(file.resolve().unwrap_err().contains("oven"));
    }

    /// Two jobs queued together run one after the other.
    #[test]
    fn two_jobs_run_in_turn() {
        let book = book();
        let recipe = book.get("mixed-ore").unwrap();
        let mut forge = forge();
        let mut slots = stocked(8, 4);
        forge.queue(&mut slots, recipe, 100.0).unwrap();
        forge.queue(&mut slots, recipe, 100.0).unwrap();
        assert_eq!(slots.count(Item::Block(Material::Ore)), 0);
        assert_eq!(slots.count(Item::Good(Good::COAL)), 0);
        assert_eq!(forge.schedule(&book), vec![(100.0, 108.0), (108.0, 116.0)]);
        assert_eq!(forge.head_progress(&book, 104.0), Some(0.5));
        assert!(forge.working(&book, 110.0));
        assert!(!forge.working(&book, 120.0));
        // At 110 one is done: two copper, an iron, a tin ore.
        assert_eq!(forge.collect(&mut slots, &book, 110.0), 1);
        assert_eq!(slots.count(Item::Good(Good::COPPER_INGOT)), 2);
        assert_eq!(slots.count(Item::Good(Good::IRON_INGOT)), 1);
        assert_eq!(slots.count(Item::Good(Good::TIN_ORE)), 1);
        assert_eq!(forge.jobs.len(), 1);
        assert_eq!(forge.collect(&mut slots, &book, 116.0), 1);
        assert!(!forge.busy());
    }

    /// Short of coal: nothing queued, nothing taken. A full queue: likewise.
    #[test]
    fn a_refused_queue_takes_nothing() {
        let book = book();
        let recipe = book.get("mixed-ore").unwrap();
        let mut forge = forge();
        let mut slots = stocked(4, 1);
        let before = slots.clone();
        assert_eq!(forge.queue(&mut slots, recipe, 0.0), Err(Refused::Short));
        assert_eq!(slots, before);
        let mut slots = stocked(99, 99);
        for _ in 0..4 {
            forge.queue(&mut slots, recipe, 0.0).unwrap();
        }
        let before = slots.clone();
        assert_eq!(forge.queue(&mut slots, recipe, 0.0), Err(Refused::Full));
        assert_eq!(slots, before);
    }

    /// A finished job whose outputs do not fit waits in the queue.
    #[test]
    fn a_collect_that_does_not_fit_waits() {
        let book = book();
        let mut forge = forge();
        let mut slots = stocked(4, 2);
        forge
            .queue(&mut slots, book.get("mixed-ore").unwrap(), 0.0)
            .unwrap();
        for k in 0..crate::inventory::CARRIED as u16 {
            slots.give(Item::Fish(k), 1);
        }
        let full = slots.clone();
        assert_eq!(forge.collect(&mut slots, &book, 100.0), 0);
        assert_eq!(slots, full);
        assert_eq!(forge.jobs.len(), 1, "kept, finished");
    }

    /// The body round-trips through RON.
    #[test]
    fn a_station_record_round_trips() {
        let mut forge = forge();
        forge.jobs.push(Job {
            recipe: "bronze".into(),
            queued_s: 12.5,
        });
        let back: StationState = ron::from_str(&ron::to_string(&forge).unwrap()).unwrap();
        assert_eq!(back, forge);
        assert_eq!(station_of(Material::Forge).map(|s| s.code), Some("forge"));
        assert_eq!(station_of(Material::Stone), None);
    }

    /// About one generated ore cell in three is coal, always the same cells;
    /// placed ore is ore.
    #[test]
    fn ore_cells_split_into_coal_seams_and_mixed_ore() {
        let coal = (0..3000u32)
            .filter(|&c| ore_yield(c, 40, false) == Item::Good(Good::COAL))
            .count();
        assert!((800..1200).contains(&coal), "{coal} of 3000");
        for c in 0..50 {
            assert_eq!(ore_yield(c, 3, false), ore_yield(c, 3, false));
            assert_eq!(ore_yield(c, 3, true), Item::Block(Material::Ore));
        }
    }

    /// Every input and output of every recipe is a known item, and every
    /// tool names a tool and a tier (`tool-crafting` task 2.1).
    #[test]
    fn every_tool_crafting_recipe_resolves_to_known_things() {
        let book = book();
        for recipe in &book.recipes {
            assert!(
                !recipe.outputs.is_empty() || !recipe.tools.is_empty(),
                "{}",
                recipe.code
            );
        }
        let made = |code: &str| book.get(code).unwrap();
        assert_eq!(made("planks").outputs, vec![(Item::Good(Good::PLANKS), 4)]);
        assert_eq!(
            made("workbench").outputs,
            vec![(Item::Block(Material::Workbench), 1)]
        );
        assert_eq!(
            made("stone-pickaxe").tools,
            vec![(Tool::Pickaxe, Tier::Stone)]
        );
        assert_eq!(made("iron-axe").tools, vec![(Tool::Axe, Tier::Iron)]);
        assert_eq!(made("rod").tools, vec![(Tool::Rod, Tier::Stone)]);
        assert_eq!(
            made("forge").outputs,
            vec![(Item::Block(Material::Forge), 1)]
        );
        assert_eq!(
            tool_named("tool:bronze-shovel"),
            Some((Tool::Shovel, Tier::Bronze))
        );
        assert_eq!(
            tool_named("tool:pickaxe"),
            Some((Tool::Pickaxe, Tier::Stone))
        );
        assert_eq!(tool_named("tool:gold-pickaxe"), None);
        assert_eq!(
            station_of(Material::Workbench).map(|s| s.code),
            Some("workbench")
        );
        assert!(HAND_RECORD_ID > record_id(u32::MAX, u16::MAX));
    }

    /// A collected tool goes to the tool slot and into hand; one already
    /// owned is a spare in the pack, so tools can be made many times over.
    #[test]
    fn a_tool_goes_to_hand_and_a_spare_to_the_pack() {
        let book = book();
        let pick = book.get("stone-pickaxe").unwrap();
        let mut bench = StationState::new(station_by_code("workbench").unwrap());
        let mut slots = Slots::new();
        slots.give(Item::Good(Good::LOOSE_STONE), 9);
        slots.give(Item::Good(Good::PLANKS), 6);
        slots.give(Item::Good(Good::ROPE), 3);
        let mut kit = Equipment::new_world();
        bench
            .queue_for(&mut slots, Some(&kit), Some(&book), pick, 0.0)
            .unwrap();
        let jobs = bench.collect_into(&mut slots, Some(&mut kit), &book, 10.0);
        assert_eq!(jobs.len(), 1);
        assert_eq!(kit.tier(Tool::Pickaxe), Some(Tier::Stone));
        assert_eq!(kit.held(), Some(Tool::Pickaxe), "made, and put in hand");
        assert_eq!(slots.count(Item::Tool(Tool::Pickaxe)), 0, "not a stack");
        // Owned already: a spare may be made, and lands in the pack.
        slots.give(Item::Good(Good::LOOSE_STONE), 9);
        slots.give(Item::Good(Good::PLANKS), 6);
        slots.give(Item::Good(Good::ROPE), 3);
        bench
            .queue_for(&mut slots, Some(&kit), Some(&book), pick, 20.0)
            .unwrap();
        let jobs = bench.collect_into(&mut slots, Some(&mut kit), &book, 30.0);
        assert_eq!(jobs.len(), 1);
        assert_eq!(
            slots.count(Item::Tool(Tool::Pickaxe)),
            1,
            "a spare, in the pack"
        );
        assert_eq!(kit.tier(Tool::Pickaxe), Some(Tier::Stone));
        assert!(!bench.busy());
    }

    /// `retire-rowboat-and-cog` decision 3: a rowboat queued at a workbench
    /// before the recipe was retired takes no time and collects as the
    /// planks and rope it took, and only when they all fit.
    #[test]
    fn a_retired_recipes_job_collects_as_its_inputs() {
        let book = book();
        let bench = STATIONS
            .iter()
            .find(|s| s.code == "workbench")
            .expect("the workbench");
        let mut state = StationState::new(bench);
        state.jobs.push(Job {
            recipe: "rowboat".into(),
            queued_s: 100.0,
        });
        let mut slots = Slots::new();
        assert_eq!(state.collect(&mut slots, &book, 100.0), 1);
        assert_eq!(slots.count(Item::Good(Good::PLANKS)), 16);
        assert_eq!(slots.count(Item::Good(Good::ROPE)), 2);
        assert!(state.jobs.is_empty());
        // A full pack takes nothing and the job waits.
        state.jobs.push(Job {
            recipe: "rowboat".into(),
            queued_s: 200.0,
        });
        let mut full = Slots::new();
        while full.give(Item::Good(Good::COAL), 99) == 0 {}
        let before = full.clone();
        assert_eq!(state.collect(&mut full, &book, 200.0), 0);
        assert_eq!(full, before);
        assert_eq!(state.jobs.len(), 1);
        assert_eq!(
            retired_inputs("rowboat"),
            Some(vec![
                (Item::Good(Good::PLANKS), 16),
                (Item::Good(Good::ROPE), 2)
            ])
        );
        assert_eq!(retired_inputs("raft"), None);
    }
}
