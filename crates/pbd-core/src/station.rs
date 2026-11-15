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
use crate::inventory::{Item, Slots, material_name};
use crate::terrain::Material;
use serde::{Deserialize, Serialize};

/// One station kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StationRow {
    /// Its saved name, in a station record and in `recipes.ron`.
    pub code: &'static str,
    pub name: &'static str,
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
pub const STATIONS: &[StationRow] = &[
    StationRow {
        code: "forge",
        name: "forge",
        material: Some(Material::Forge),
        max_jobs: 4,
    },
    StationRow {
        code: "hand",
        name: "hands",
        material: None,
        max_jobs: 1,
    },
    // `cooking` design S2: the kitchen's first station, also a light.
    StationRow {
        code: "campfire",
        name: "campfire",
        material: Some(Material::Campfire),
        max_jobs: 4,
    },
];

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

/// An item as `recipes.ron` names it: a good by its code (`coal`), a block
/// as `block:<name>` (`block:ore`), the name the inventory gives it, or a
/// tool as `tool:<name>` (`tool:pickaxe`). A tool output is the hook for
/// `tool-crafting`: a tool is a single thing (stack 1) that the app hands to
/// the tool slot rather than a stack; its tier is that change's to add.
pub fn item_named(name: &str) -> Option<Item> {
    if let Some(tool) = name.strip_prefix("tool:") {
        return crate::inventory::Tool::ALL
            .into_iter()
            .find(|t| t.name().eq_ignore_ascii_case(tool))
            .map(Item::Tool);
    }
    if let Some(block) = name.strip_prefix("block:") {
        return Material::ALL
            .into_iter()
            .find(|m| material_name(*m) == block)
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
    /// What the panel's plaque says.
    pub name: String,
    /// The station code it is made at.
    pub station: String,
    /// World seconds from start to done.
    pub secs: f32,
    /// What it takes, fuel included: `(item, count)`.
    pub inputs: Vec<(String, u16)>,
    /// What it gives.
    pub outputs: Vec<(String, u16)>,
}

/// `assets/config/recipes.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeFile {
    pub recipes: Vec<RecipeEntry>,
}

impl Default for RecipeFile {
    /// M3's forge: mixed ore at its fixed split, and bronze.
    fn default() -> Self {
        let pair = |name: &str, n| (name.to_string(), n);
        Self {
            recipes: vec![
                RecipeEntry {
                    code: "mixed-ore".into(),
                    name: "SMELT ORE".into(),
                    station: "forge".into(),
                    secs: 8.0,
                    inputs: vec![pair("block:ore", 4), pair("coal", 2)],
                    outputs: vec![
                        pair("copper-ingot", 2),
                        pair("iron-ingot", 1),
                        pair("tin-ore", 1),
                    ],
                },
                RecipeEntry {
                    code: "bronze".into(),
                    name: "BRONZE".into(),
                    station: "forge".into(),
                    secs: 5.0,
                    inputs: vec![pair("copper-ingot", 1), pair("tin-ore", 1), pair("coal", 1)],
                    outputs: vec![pair("bronze-ingot", 2)],
                },
            ],
        }
    }
}

/// A recipe with its items resolved.
#[derive(Clone, Debug, PartialEq)]
pub struct Recipe {
    pub code: String,
    pub name: String,
    pub station: &'static StationRow,
    pub secs: f32,
    pub inputs: Vec<(Item, u16)>,
    pub outputs: Vec<(Item, u16)>,
}

/// Every recipe, resolved and checked.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct RecipeBook {
    pub recipes: Vec<Recipe>,
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
            recipes.push(Recipe {
                code: entry.code.clone(),
                name: entry.name.clone(),
                station,
                secs: entry.secs,
                inputs: items(&entry.inputs)?,
                outputs: items(&entry.outputs)?,
            });
        }
        Ok(RecipeBook { recipes })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.resolve().map(|_| ())
    }
}

impl RecipeBook {
    pub fn get(&self, code: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.code == code)
    }

    /// A station's recipes, in file order.
    pub fn at(&self, station: &str) -> impl Iterator<Item = &Recipe> {
        self.recipes
            .iter()
            .filter(move |r| r.station.code == station)
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
        let mut done = 0;
        while let Some(&(_, end)) = self.schedule(book).first() {
            if end > now_s {
                break;
            }
            let mut after = slots.clone();
            let fits = match book.get(&self.jobs[0].recipe) {
                Some(recipe) => recipe
                    .outputs
                    .iter()
                    .all(|&(item, n)| after.give(item, n) == 0),
                // A recipe this build lacks: kept, never thrown away.
                None => false,
            };
            if !fits {
                break;
            }
            *slots = after;
            self.jobs.remove(0);
            // The next job started when this one finished, not when it was
            // queued: carried on it, so taking the head off does not move it.
            if let Some(next) = self.jobs.first_mut() {
                next.queued_s = next.queued_s.max(end);
            }
            done += 1;
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

    fn book() -> RecipeBook {
        RecipeFile::default().resolve().unwrap()
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
        let book = file.resolve().unwrap();
        assert_eq!(book.at("forge").count(), 2);
        assert_eq!(item_named("block:ore"), Some(Item::Block(Material::Ore)));
        assert_eq!(item_named("coal"), Some(Item::Good(Good::COAL)));
        assert_eq!(item_named("unobtainium"), None);
        assert_eq!(
            item_named("tool:pickaxe"),
            Some(Item::Tool(crate::inventory::Tool::Pickaxe))
        );
        assert_eq!(station_by_code("hand").map(|s| s.material), Some(None));
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
}
