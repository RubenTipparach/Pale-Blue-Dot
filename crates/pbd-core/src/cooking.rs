//! Cooking (`cooking` design S1 to S6): the kitchen's recipes, the dishes and
//! what eating one does.
//!
//! The kitchen is the farming change's station machinery with more rows. A
//! campfire is a row of [`crate::station::STATIONS`]; its recipes, read from
//! `assets/config/cooking.ron`, are appended to the same
//! [`RecipeBook`](crate::station::RecipeBook) the forge's are, so a queue, a
//! collect and a station record follow one set of rules. What is the
//! kitchen's own is here:
//! - **any fish**: a recipe input that takes one caught fish of any species,
//!   resolved to a real stack when the job is queued ([`concrete`]);
//! - **pending rows**: a recipe naming a good this build lacks (the husbandry
//!   branch's eggs, milk and meat) is held aside rather than refused;
//! - **dishes and effects**: which goods are eaten, the effect each gives
//!   and for how long, and the rules for three effects at once ([`Meals`]).
//!
//! No hunger (Q1, recommendation taken): an effect is a reward for cooking,
//! and not eating costs nothing.

use crate::fauna::FishingSettings;
use crate::goods::Good;
use crate::inventory::{Item, Slots};
use crate::station::{Recipe, RecipeEntry, item_named, station_by_code};
use serde::{Deserialize, Serialize};

/// What a recipe means by `fish`: one caught fish of any species. Never in a
/// slot: [`concrete`] swaps it for the species the slots hold before a job
/// takes anything, and an output is never "any fish".
pub const ANY_FISH: Item = Item::Fish(u16::MAX);

/// The recipe file's word for [`ANY_FISH`].
pub const ANY_FISH_NAME: &str = "fish";

/// An effect a dish gives (design S5). Only effects on systems that exist:
/// glow waits for a light that follows the player, warmth for cold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Effect {
    /// Sprinting and swimming are faster.
    Stride,
    /// The line's tension rises slower while reeling; the fish tires faster.
    Steady,
    /// Blocks break faster.
    Keen,
}

impl Effect {
    pub const ALL: [Effect; 3] = [Effect::Stride, Effect::Steady, Effect::Keen];

    pub fn name(self) -> &'static str {
        match self {
            Effect::Stride => "stride",
            Effect::Steady => "steady hands",
            Effect::Keen => "keen",
        }
    }
}

/// How strong each effect is: a multiplier, 1 being no effect.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Strengths {
    /// Sprint and swim speed, times.
    pub stride: f32,
    /// Tension gained while reeling, divided by; stamina lost, times.
    pub steady: f32,
    /// Break speed, times (a break takes its time divided by this).
    pub keen: f32,
}

impl Strengths {
    pub fn of(&self, effect: Effect) -> f32 {
        match effect {
            Effect::Stride => self.stride,
            Effect::Steady => self.steady,
            Effect::Keen => self.keen,
        }
    }
}

/// One dish as the file writes it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DishEntry {
    /// The good's code.
    pub good: String,
    pub effect: Effect,
    /// Minutes of play the effect lasts (world time: a minute of play is a
    /// minute, and a closed game runs nothing down).
    pub minutes: f32,
}

/// `assets/config/cooking.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CookingFile {
    pub strengths: Strengths,
    /// The most effects running at once.
    pub max_effects: usize,
    pub dishes: Vec<DishEntry>,
    /// Goods another branch is adding (husbandry's eggs, milk, meat): a
    /// recipe that names one of these and finds it missing is pending, not
    /// wrong. Any other missing name is an error.
    pub awaited: Vec<String>,
    pub recipes: Vec<RecipeEntry>,
}

/// A dish, resolved.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dish {
    pub good: Good,
    pub effect: Effect,
    pub minutes: f32,
}

impl Dish {
    pub fn secs(&self) -> f64 {
        f64::from(self.minutes) * 60.0
    }
}

/// The file, resolved: the live recipes (for the one recipe book), the
/// pending ones by code with what they wait for, and the dishes.
#[derive(Clone, Debug, PartialEq)]
pub struct Kitchen {
    pub strengths: Strengths,
    pub max_effects: usize,
    pub dishes: Vec<Dish>,
    pub recipes: Vec<Recipe>,
    pub pending: Vec<(String, Vec<String>)>,
}

impl Default for Kitchen {
    fn default() -> Self {
        CookingFile::default()
            .resolve()
            .expect("the defaults resolve")
    }
}

impl Kitchen {
    /// The dish a good is, if it is eaten.
    pub fn dish(&self, good: Good) -> Option<&Dish> {
        self.dishes.iter().find(|d| d.good == good)
    }
}

impl Default for CookingFile {
    /// The campfire's dishes (design S4).
    fn default() -> Self {
        let pair = |name: &str, n| (name.to_string(), n);
        let recipe = |code: &str,
                      name: &str,
                      station: &str,
                      secs: f32,
                      inputs: Vec<(String, u16)>,
                      outputs: Vec<(String, u16)>| RecipeEntry {
            code: code.into(),
            name: name.into(),
            station: station.into(),
            secs,
            inputs,
            outputs,
        };
        let dish = |good: &str, effect, minutes| DishEntry {
            good: good.into(),
            effect,
            minutes,
        };
        Self {
            strengths: Strengths {
                stride: 1.15,
                steady: 1.2,
                keen: 1.15,
            },
            max_effects: 3,
            dishes: vec![
                dish("grilled-fish", Effect::Steady, 6.0),
                dish("chili-fish", Effect::Steady, 10.0),
                dish("flatbread", Effect::Stride, 5.0),
                dish("roast-turnip", Effect::Keen, 4.0),
                dish("kale-skewer", Effect::Keen, 6.0),
                dish("fried-egg", Effect::Keen, 5.0),
                dish("roast-meat", Effect::Stride, 8.0),
            ],
            awaited: vec!["egg".into(), "milk".into(), "raw-meat".into()],
            recipes: vec![
                recipe(
                    "grilled-fish",
                    "GRILL FISH",
                    "campfire",
                    20.0,
                    vec![pair("fish", 1)],
                    vec![pair("grilled-fish", 1)],
                ),
                recipe(
                    "chili-fish",
                    "CHILI FISH",
                    "campfire",
                    25.0,
                    vec![pair("fish", 1), pair("chili", 1)],
                    vec![pair("chili-fish", 1)],
                ),
                recipe(
                    "flatbread",
                    "FLATBREAD",
                    "campfire",
                    30.0,
                    vec![pair("wheat", 3)],
                    vec![pair("flatbread", 2)],
                ),
                recipe(
                    "roast-turnip",
                    "ROAST TURNIP",
                    "campfire",
                    20.0,
                    vec![pair("turnip", 2)],
                    vec![pair("roast-turnip", 2)],
                ),
                recipe(
                    "kale-skewer",
                    "KALE SKEWER",
                    "campfire",
                    25.0,
                    vec![pair("frost-kale", 2), pair("turnip", 1)],
                    vec![pair("kale-skewer", 1)],
                ),
                recipe(
                    "fried-egg",
                    "FRIED EGG",
                    "campfire",
                    15.0,
                    vec![pair("egg", 1)],
                    vec![pair("fried-egg", 1)],
                ),
                recipe(
                    "roast-meat",
                    "ROAST MEAT",
                    "campfire",
                    30.0,
                    vec![pair("raw-meat", 1)],
                    vec![pair("roast-meat", 1)],
                ),
                recipe(
                    "campfire",
                    "CAMPFIRE",
                    "hand",
                    3.0,
                    vec![pair("block:stone", 5), pair("coal", 2)],
                    vec![pair("block:campfire", 1)],
                ),
            ],
        }
    }
}

impl CookingFile {
    /// Resolve every name: a recipe naming only awaited goods it cannot find
    /// is pending; any other wrong name is an error, by name.
    pub fn resolve(&self) -> Result<Kitchen, String> {
        let s = &self.strengths;
        for (name, v) in [("stride", s.stride), ("steady", s.steady), ("keen", s.keen)] {
            if !(v.is_finite() && (1.0..=3.0).contains(&v)) {
                return Err(format!("strength {name} must be 1..=3"));
            }
        }
        if self.max_effects == 0 {
            return Err("max_effects must be at least one".into());
        }
        let mut dishes = Vec::new();
        for entry in &self.dishes {
            let good = Good::by_code(&entry.good)
                .ok_or_else(|| format!("dish {:?} is no good", entry.good))?;
            if !(entry.minutes.is_finite() && entry.minutes > 0.0) {
                return Err(format!("dish {}: minutes must be above zero", entry.good));
            }
            if dishes.iter().any(|d: &Dish| d.good == good) {
                return Err(format!("dish {} twice", entry.good));
            }
            dishes.push(Dish {
                good,
                effect: entry.effect,
                minutes: entry.minutes,
            });
        }
        let mut recipes: Vec<Recipe> = Vec::new();
        let mut pending = Vec::new();
        for entry in &self.recipes {
            let at = &entry.code;
            if !crate::records::is_word(at) {
                return Err(format!("recipe code {at:?} is not a word"));
            }
            if recipes.iter().any(|r| &r.code == at) || pending.iter().any(|(c, _)| c == at) {
                return Err(format!("recipe {at} twice"));
            }
            let station = station_by_code(&entry.station)
                .ok_or_else(|| format!("{at}: no station {:?}", entry.station))?;
            if !(entry.secs.is_finite() && entry.secs > 0.0) {
                return Err(format!("{at}: secs must be above zero"));
            }
            let mut missing = Vec::new();
            let mut items = |list: &[(String, u16)], output: bool| -> Result<_, String> {
                if list.is_empty() {
                    return Err(format!("{at}: an empty list"));
                }
                let mut out = Vec::new();
                for (name, n) in list {
                    let item = if name == ANY_FISH_NAME && !output {
                        Some(ANY_FISH)
                    } else {
                        item_named(name)
                    };
                    let Some(item) = item else {
                        if self.awaited.contains(name) {
                            missing.push(name.clone());
                            continue;
                        }
                        return Err(format!("{at}: no item {name:?}"));
                    };
                    if *n == 0 || *n > item.stack_limit() {
                        return Err(format!("{at}: {n} {name} is not a stack"));
                    }
                    out.push((item, *n));
                }
                Ok(out)
            };
            let inputs = items(&entry.inputs, false)?;
            let outputs = items(&entry.outputs, true)?;
            if !missing.is_empty() {
                pending.push((at.clone(), missing));
                continue;
            }
            recipes.push(Recipe {
                code: at.clone(),
                name: entry.name.clone(),
                station,
                secs: entry.secs,
                inputs,
                outputs,
            });
        }
        Ok(Kitchen {
            strengths: self.strengths,
            max_effects: self.max_effects,
            dishes,
            recipes,
            pending,
        })
    }

    pub fn validate(&self) -> Result<(), String> {
        self.resolve().map(|_| ())
    }
}

/// How many caught fish a recipe takes.
pub fn fish_in(recipe: &Recipe) -> u16 {
    recipe
        .inputs
        .iter()
        .filter(|(item, _)| matches!(item, Item::Fish(_)))
        .map(|(_, n)| n)
        .sum()
}

/// The recipe with "any fish" made a real species: the first fish stack, in
/// slot order (hotbar first), holding enough. `None` when the slots hold no
/// such stack. A recipe with no "any fish" comes back as it is.
pub fn concrete(recipe: &Recipe, slots: &Slots) -> Option<Recipe> {
    let mut out = recipe.clone();
    for (item, n) in out.inputs.iter_mut() {
        if *item != ANY_FISH {
            continue;
        }
        let species = slots.iter().flatten().find_map(|stack| match stack.item {
            Item::Fish(species)
                if Item::Fish(species) != ANY_FISH && slots.count(stack.item) >= u32::from(*n) =>
            {
                Some(species)
            }
            _ => None,
        })?;
        *item = Item::Fish(species);
    }
    Some(out)
}

/// Whether the slots hold every input, "any fish" counted as any species.
pub fn have(recipe: &Recipe, slots: &Slots) -> bool {
    concrete(recipe, slots).is_some_and(|r| {
        r.inputs
            .iter()
            .all(|&(item, n)| slots.count(item) >= u32::from(n))
    })
}

/// The record kind the running effects are stored as, its schema and the
/// one id it uses (the player's).
pub const MEAL_KIND: &str = "meal";
pub const MEAL_SCHEMA: u32 = 1;
pub const MEAL_ID: u64 = 0;

/// One running effect: until when, in world seconds.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Meal {
    pub effect: Effect,
    pub until_s: f64,
    /// The good eaten for it, by code: what the HUD shows.
    #[serde(default)]
    pub dish: String,
}

/// The effects running now (the `meal` record's body).
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Meals {
    pub running: Vec<Meal>,
}

impl Meals {
    /// Eat a dish at `now_s`: its effect runs for its minutes. One already
    /// running keeps the longer time (never stacks); a new effect past
    /// `max` running replaces the one with least time left.
    pub fn eat(&mut self, dish: &Dish, now_s: f64, max: usize) {
        self.running.retain(|m| m.until_s > now_s);
        let until_s = now_s + dish.secs();
        if let Some(meal) = self.running.iter_mut().find(|m| m.effect == dish.effect) {
            if until_s > meal.until_s {
                meal.until_s = until_s;
                meal.dish = dish.good.code().to_string();
            }
            return;
        }
        if self.running.len() >= max.max(1)
            && let Some(least) = self
                .running
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.until_s.total_cmp(&b.1.until_s))
                .map(|(i, _)| i)
        {
            self.running.remove(least);
        }
        self.running.push(Meal {
            effect: dish.effect,
            until_s,
            dish: dish.good.code().to_string(),
        });
    }

    /// Seconds left on an effect at `now_s`, or `None` when it is not running.
    pub fn left_s(&self, effect: Effect, now_s: f64) -> Option<f64> {
        self.running
            .iter()
            .find(|m| m.effect == effect && m.until_s > now_s)
            .map(|m| m.until_s - now_s)
    }

    /// The effect's multiplier at `now_s`: its strength while it runs, 1
    /// otherwise.
    pub fn factor(&self, effect: Effect, now_s: f64, strengths: &Strengths) -> f32 {
        if self.left_s(effect, now_s).is_some() {
            strengths.of(effect)
        } else {
            1.0
        }
    }
}

/// The fishing rules with steady hands at `factor` (1 is none): tension
/// gained while reeling divided by it, stamina lost times it.
pub fn steadied(fishing: &FishingSettings, factor: f32) -> FishingSettings {
    let factor = if factor.is_finite() && factor > 0.0 {
        factor
    } else {
        1.0
    };
    FishingSettings {
        tension_base: fishing.tension_base / factor,
        tension_pull: fishing.tension_pull / factor,
        stamina_drain: fishing.stamina_drain * factor,
        ..*fishing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::{RecipeBook, RecipeFile, StationState, station_by_code};
    use crate::terrain::Material;

    fn kitchen() -> Kitchen {
        CookingFile::default().resolve().unwrap()
    }

    /// The shipped file is the defaults written out, and resolves; the
    /// husbandry dishes wait for their goods.
    #[test]
    fn the_shipped_file_resolves_and_agrees_with_the_defaults() {
        let file: CookingFile =
            ron::from_str(include_str!("../../../assets/config/cooking.ron")).unwrap();
        assert_eq!(file, CookingFile::default());
        let kitchen = file.resolve().unwrap();
        let live: Vec<_> = kitchen.recipes.iter().map(|r| r.code.as_str()).collect();
        for code in [
            "grilled-fish",
            "chili-fish",
            "flatbread",
            "roast-turnip",
            "kale-skewer",
            "campfire",
        ] {
            assert!(live.contains(&code), "{code}");
        }
        // Until the husbandry branch's goods exist.
        for (code, missing) in &kitchen.pending {
            assert!(
                ["fried-egg", "roast-meat"].contains(&code.as_str()),
                "{code}"
            );
            assert!(missing.iter().all(|m| file.awaited.contains(m)));
        }
        // Every recipe's dish is a dish, and every dish is eaten.
        for recipe in kitchen
            .recipes
            .iter()
            .filter(|r| r.station.code == "campfire")
        {
            for (item, _) in &recipe.outputs {
                let Item::Good(good) = item else {
                    panic!("{}: a dish is a good", recipe.code)
                };
                assert!(kitchen.dish(*good).is_some(), "{}", recipe.code);
            }
        }
    }

    /// The kitchen's recipes and the forge's make one book, with no code
    /// twice, and the campfire is a station whose block is the campfire.
    #[test]
    fn the_kitchen_joins_the_one_recipe_book() {
        let mut book: RecipeBook = RecipeFile::default().resolve().unwrap();
        let n = book.recipes.len();
        book.recipes.extend(kitchen().recipes);
        let mut codes: Vec<_> = book.recipes.iter().map(|r| r.code.clone()).collect();
        codes.sort();
        codes.dedup();
        assert_eq!(codes.len(), book.recipes.len());
        assert!(book.recipes.len() > n);
        assert_eq!(book.at("campfire").count(), 5);
        let row = station_by_code("campfire").unwrap();
        assert_eq!(row.material, Some(Material::Campfire));
        assert_eq!(
            book.get("campfire").unwrap().outputs,
            vec![(Item::Block(Material::Campfire), 1)]
        );
    }

    /// A missing name not awaited is an error by name; an awaited one makes
    /// the recipe pending.
    #[test]
    fn only_awaited_goods_make_a_recipe_pending() {
        let mut file = CookingFile::default();
        file.recipes[0].inputs[0].0 = "trout-ish".into();
        assert!(file.resolve().unwrap_err().contains("trout-ish"));
        let mut file = CookingFile::default();
        file.awaited.push("trout-ish".into());
        file.recipes[0].inputs[0].0 = "trout-ish".into();
        let kitchen = file.resolve().unwrap();
        assert!(kitchen.pending.iter().any(|(c, _)| c == "grilled-fish"));
        assert!(kitchen.recipes.iter().all(|r| r.code != "grilled-fish"));
        // "fish" is an input only.
        let mut file = CookingFile::default();
        file.recipes[0].outputs[0].0 = "fish".into();
        assert!(file.resolve().is_err());
    }

    /// Any fish takes the first species the slots hold enough of; a queued
    /// grill takes that fish and gives a grilled fish when done.
    #[test]
    fn any_fish_is_taken_and_grilled() {
        let kitchen = kitchen();
        let mut book = RecipeBook::default();
        book.recipes.extend(kitchen.recipes.clone());
        let grill = book.get("grilled-fish").unwrap();
        let mut slots = Slots::new();
        assert!(!have(grill, &slots));
        assert!(concrete(grill, &slots).is_none());
        slots.give(Item::Fish(3), 2);
        slots.give(Item::Fish(5), 1);
        assert!(have(grill, &slots));
        assert_eq!(fish_in(grill), 1);
        let real = concrete(grill, &slots).unwrap();
        assert_eq!(real.inputs, vec![(Item::Fish(3), 1)]);
        let mut fire = StationState::new(station_by_code("campfire").unwrap());
        fire.queue(&mut slots, &real, 100.0).unwrap();
        assert_eq!(slots.count(Item::Fish(3)), 1);
        assert_eq!(fire.collect(&mut slots, &book, 110.0), 0, "not done");
        assert_eq!(fire.collect(&mut slots, &book, 120.0), 1);
        let dish = Good::by_code("grilled-fish").unwrap();
        assert_eq!(slots.count(Item::Good(dish)), 1);
        // Chili fish needs a chili as well.
        let chili = book.get("chili-fish").unwrap();
        assert!(!have(chili, &slots));
        slots.give(Item::Good(Good::CHILI), 1);
        assert!(have(chili, &slots));
    }

    fn dish(code: &str) -> Dish {
        *kitchen().dish(Good::by_code(code).unwrap()).unwrap()
    }

    /// Three effects at once; the same one keeps the longer time; a fourth
    /// replaces the one with least left; time is world time.
    #[test]
    fn three_effects_at_once_and_none_stacks() {
        let k = kitchen();
        let s = k.strengths;
        let mut meals = Meals::default();
        assert_eq!(meals.factor(Effect::Stride, 0.0, &s), 1.0);
        meals.eat(&dish("flatbread"), 0.0, 3);
        assert_eq!(meals.factor(Effect::Stride, 10.0, &s), 1.15);
        assert_eq!(meals.left_s(Effect::Stride, 0.0), Some(300.0));
        // Eaten again a minute on: the later end, not the sum.
        meals.eat(&dish("flatbread"), 60.0, 3);
        assert_eq!(meals.left_s(Effect::Stride, 60.0), Some(300.0));
        assert_eq!(meals.running.len(), 1);
        meals.eat(&dish("grilled-fish"), 60.0, 3);
        meals.eat(&dish("roast-turnip"), 60.0, 3);
        assert_eq!(meals.running.len(), 3);
        // Ends: stride 360, steady 420, keen 300. A fourth kind would push
        // keen out; with three kinds only, cap the test at two.
        meals.eat(&dish("grilled-fish"), 100.0, 2);
        assert_eq!(meals.running.len(), 3, "a running effect is refreshed");
        let mut two = Meals::default();
        two.eat(&dish("flatbread"), 0.0, 2); // stride to 300
        two.eat(&dish("roast-turnip"), 0.0, 2); // keen to 240
        two.eat(&dish("grilled-fish"), 0.0, 2); // steady: keen goes
        assert_eq!(two.left_s(Effect::Keen, 0.0), None);
        assert!(two.left_s(Effect::Stride, 0.0).is_some());
        assert!(two.left_s(Effect::Steady, 0.0).is_some());
        // Past its end, an effect is nothing.
        assert_eq!(two.factor(Effect::Stride, 301.0, &s), 1.0);
    }

    /// The record body round-trips.
    #[test]
    fn a_meal_record_round_trips() {
        let mut meals = Meals::default();
        meals.eat(&dish("chili-fish"), 12.5, 3);
        let back: Meals = ron::from_str(&ron::to_string(&meals).unwrap()).unwrap();
        assert_eq!(back, meals);
        let record = crate::records::Record::of(MEAL_KIND, MEAL_ID, MEAL_SCHEMA, &meals);
        assert_eq!(record.read::<Meals>(), Some(meals));
    }

    /// Steady hands slows the tension and tires the fish, and nothing else.
    #[test]
    fn steady_hands_eases_the_line() {
        let base = FishingSettings::default();
        let steady = steadied(&base, 1.2);
        assert!((steady.tension_base - base.tension_base / 1.2).abs() < 1e-6);
        assert!((steady.tension_pull - base.tension_pull / 1.2).abs() < 1e-6);
        assert!((steady.stamina_drain - base.stamina_drain * 1.2).abs() < 1e-6);
        assert_eq!(steady.reel_mps, base.reel_mps);
        assert_eq!(steadied(&base, 1.0), base);
    }
}
