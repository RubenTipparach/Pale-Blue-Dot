//! What a made good is worth (`townsfolk` design 37 and 37a): the sum of
//! its inputs' values plus its labour, times the craft margin, worked out
//! from the one recipe book when the game starts, never typed in.
//!
//! - A code no recipe makes is **raw** and keeps its `values.ron` row.
//! - A recipe's cost is `(Σ inputs + labour_per_s × secs) × craft_margin`,
//!   split between its outputs by their `values.ron` rows, each output's
//!   share over its count rounded up to a whole coin (never under 1). So a
//!   recipe's outputs are always worth more than its inputs.
//! - A good two recipes make takes the cheaper.
//! - The input "fish" (any fish) is the cheapest fish row.
//! - A recipe's tools are not derived: a tool's row is tier-less and tools
//!   are not traded, so a tool keeps its row.
//! - A code in `values.ron`'s `overrides` keeps its row as typed.
//!
//! [`Deriver::priced`] runs the same derivation with the market's index
//! folded in (`super::market`): each raw value times its index, each made
//! value's result times its own, so a dear input makes a dear good.

use super::values::Values;
use crate::cooking::ANY_FISH;
use crate::inventory::Item;
use crate::missions::item_code;
use crate::station::RecipeBook;
use std::collections::{BTreeMap, BTreeSet};

/// The cheapest fish row stands for "any fish".
const ANY_FISH_CODE: &str = "fish";

/// One recipe as the derivation reads it: trade codes and counts.
#[derive(Clone, Debug, PartialEq)]
pub struct Making {
    /// The recipe's code, for a test or a log to name it.
    pub recipe: String,
    pub inputs: Vec<(String, u32)>,
    pub secs: f32,
    /// Its outputs, tools left out.
    pub outputs: Vec<(String, u32)>,
}

fn code_of(item: Item) -> String {
    if item == ANY_FISH {
        ANY_FISH_CODE.to_string()
    } else {
        item_code(item)
    }
}

/// Every recipe of `book` that makes a good or a block, as trade codes.
pub fn makings(book: &RecipeBook) -> Vec<Making> {
    book.recipes
        .iter()
        .filter(|r| !r.outputs.is_empty())
        .map(|r| Making {
            recipe: r.code.clone(),
            inputs: r
                .inputs
                .iter()
                .map(|&(i, n)| (code_of(i), u32::from(n)))
                .collect(),
            secs: r.secs,
            outputs: r
                .outputs
                .iter()
                .map(|&(i, n)| (code_of(i), u32::from(n)))
                .collect(),
        })
        .collect()
}

/// The derivation: the typed rows, the recipes, and the two rates.
#[derive(Clone, Debug, PartialEq)]
pub struct Deriver {
    base: Values,
    makings: Vec<Making>,
    labour_per_s: f64,
    margin: f64,
    /// Codes some recipe makes and no override names.
    made: BTreeSet<String>,
}

/// A derivation's result: the values, and for each made good the recipe
/// that priced it (an index into [`Deriver::makings`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Derived {
    pub values: Values,
    pub source: BTreeMap<String, usize>,
}

impl Deriver {
    /// Check that every code the recipes name has a row; `overrides` are the
    /// made codes that keep their typed row.
    pub fn new(
        base: Values,
        makings: Vec<Making>,
        labour_per_s: f32,
        margin: f32,
        overrides: &[String],
    ) -> Result<Self, String> {
        for m in &makings {
            for (code, _) in m.inputs.iter().chain(&m.outputs) {
                if code != ANY_FISH_CODE && base.get(code).is_none() {
                    return Err(format!("values: recipe {} names {code}, no row", m.recipe));
                }
            }
        }
        let made = makings
            .iter()
            .flat_map(|m| m.outputs.iter().map(|(c, _)| c.clone()))
            .filter(|c| !overrides.contains(c))
            .collect();
        Ok(Self {
            base,
            makings,
            labour_per_s: f64::from(labour_per_s),
            margin: f64::from(margin),
            made,
        })
    }

    pub fn makings(&self) -> &[Making] {
        &self.makings
    }

    /// Whether `code` keeps its typed row (no recipe makes it, or an
    /// override names it).
    pub fn is_raw(&self, code: &str) -> bool {
        !self.made.contains(code)
    }

    /// The values at rest: every index 1.
    pub fn derive(&self) -> Derived {
        self.priced(|_| 1.0)
    }

    /// The values with each good's market `index` folded in.
    pub fn priced(&self, index: impl Fn(&str) -> f32) -> Derived {
        let idx = |c: &str| f64::from(index(c));
        let mut value: BTreeMap<&str, f64> = BTreeMap::new();
        for code in self.base.codes() {
            if self.is_raw(code) {
                let coins = f64::from(self.base.get(code).map_or(1, |v| v.coins));
                value.insert(code, coins * idx(code));
            }
        }
        let mut source: BTreeMap<String, usize> = BTreeMap::new();
        // Values only fall once set (the cheaper recipe wins), and never
        // under a coin, so the passes end; the cap is a guard.
        for _ in 0..=self.makings.len() + 1 {
            let mut moved = false;
            for (at, m) in self.makings.iter().enumerate() {
                let Some(cost) = self.cost(m, &value) else {
                    continue;
                };
                let weight: f64 = m
                    .outputs
                    .iter()
                    .map(|(c, n)| self.typed(c) * f64::from(*n))
                    .sum();
                for (code, _) in &m.outputs {
                    if self.is_raw(code) {
                        continue;
                    }
                    let share = cost * self.typed(code) / weight;
                    let unit = (share - 1e-9).ceil().max(1.0) * idx(code);
                    let better = value.get(code.as_str()).is_none_or(|&v| unit < v - 1e-9);
                    if better {
                        value.insert(code.as_str(), unit);
                        source.insert(code.clone(), at);
                        moved = true;
                    }
                }
            }
            if !moved {
                break;
            }
        }
        let mut values = self.base.clone();
        for (code, v) in &value {
            values.set_coins(code, (v.round() as u32).max(1));
        }
        Derived { values, source }
    }

    /// A code's typed row, the weight it takes of a shared cost.
    fn typed(&self, code: &str) -> f64 {
        f64::from(self.base.get(code).map_or(1, |v| v.coins))
    }

    /// A recipe's cost, once every input has a value.
    fn cost(&self, m: &Making, value: &BTreeMap<&str, f64>) -> Option<f64> {
        let mut sum = 0.0;
        for (code, n) in &m.inputs {
            let v = if code == ANY_FISH_CODE {
                value
                    .iter()
                    .filter(|(c, _)| c.starts_with("fish:"))
                    .map(|(_, v)| *v)
                    .min_by(f64::total_cmp)?
            } else {
                *value.get(code.as_str())?
            };
            sum += v * f64::from(*n);
        }
        Some((sum + self.labour_per_s * f64::from(m.secs)) * self.margin)
    }
}
