//! What everything is worth (`townsfolk` design 6): one number of coins for
//! every good, block, fish, dish and tool, with the group a town's need is
//! read by, in `assets/config/values.ron`.
//!
//! The file is **append-only**, like the goods table: a row, once shipped,
//! keeps its code (a value may be retuned, a row never dropped), and a new
//! item gets a new row at the end. A test fails while any item the game
//! knows has no row.
//!
//! An item is named here by its trade code ([`trade_code`]): a good's code,
//! `block:<name>`, `tool:<code>`, and `fish:<species id>` (a fish in the
//! pack is its roster index, which the caller turns into the id).

use crate::inventory::{Item, Tool};
use crate::missions::item_code;
use crate::terrain::Material;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What a town's need is read by: the kind of thing a good is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Group {
    Seed,
    Crop,
    Fruit,
    Fish,
    /// Eggs, milk, wool, hides: what kept animals give.
    Herd,
    /// Ore, coal and ingots.
    Ore,
    /// Logs, planks, sticks and what is made of them.
    Wood,
    /// Stone and earth blocks.
    Stone,
    /// A cooked dish.
    Dish,
    /// Lamps and fires.
    Light,
    /// Made things: rope, buckets, boats, stations.
    Goods,
    Tool,
}

/// `assets/config/values.ron`: `(code, coins, group)`, in the order shipped.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ValuesFile {
    pub values: Vec<(String, u32, Group)>,
}

/// One item's row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value {
    pub coins: u32,
    pub group: Group,
}

/// The values, by trade code.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Values {
    rows: BTreeMap<String, Value>,
    /// The codes in file order.
    order: Vec<String>,
}

impl ValuesFile {
    /// Check the rows and index them: a code twice, or a value of nought, is
    /// refused (an item worth nothing is one nobody priced).
    pub fn resolve(&self) -> Result<Values, String> {
        let mut values = Values::default();
        for (code, coins, group) in &self.values {
            if *coins == 0 {
                return Err(format!("values: {code} is worth nought"));
            }
            let row = Value {
                coins: *coins,
                group: *group,
            };
            if values.rows.insert(code.clone(), row).is_some() {
                return Err(format!("values: {code} twice"));
            }
            values.order.push(code.clone());
        }
        Ok(values)
    }
}

impl Values {
    /// Read `values.ron`'s text.
    pub fn parse(text: &str) -> Result<Values, String> {
        let file: ValuesFile = ron::from_str(text).map_err(|e| format!("values.ron: {e}"))?;
        file.resolve()
    }

    pub fn get(&self, code: &str) -> Option<Value> {
        self.rows.get(code).copied()
    }

    /// Every code in file order.
    pub fn codes(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}

/// An item's trade code. `fish_ids` is the body's fish roster, by index:
/// `None` for a fish index the roster lacks.
pub fn trade_code(item: Item, fish_ids: &[String]) -> Option<String> {
    match item {
        Item::Fish(index) => fish_ids.get(index as usize).map(|id| format!("fish:{id}")),
        other => Some(item_code(other)),
    }
}

/// The item a trade code names, against the same roster.
pub fn item_of(code: &str, fish_ids: &[String]) -> Option<Item> {
    if let Some(id) = code.strip_prefix("fish:") {
        return fish_ids
            .iter()
            .position(|f| f == id)
            .map(|i| Item::Fish(i as u16));
    }
    if let Some(name) = code.strip_prefix("block:") {
        return Material::ALL
            .into_iter()
            .find(|m| crate::inventory::material_id(*m) == name)
            .map(Item::Block);
    }
    if let Some(name) = code.strip_prefix("tool:") {
        return Tool::ALL
            .into_iter()
            .find(|t| t.code() == name)
            .map(Item::Tool);
    }
    crate::goods::Good::by_code(code).map(Item::Good)
}

/// Every item the game knows, by trade code: every good (dishes are goods),
/// every block a hand can hold (all but air and water), every tool, and
/// every fish in `fish_ids`.
pub fn known_codes(fish_ids: &[String]) -> Vec<String> {
    let mut codes: Vec<String> = crate::goods::Good::all()
        .map(|g| g.code().to_string())
        .collect();
    codes.extend(
        Material::ALL
            .into_iter()
            .filter(|m| !matches!(m, Material::Air | Material::Water))
            .map(|m| item_code(Item::Block(m))),
    );
    codes.extend(Tool::ALL.into_iter().map(|t| item_code(Item::Tool(t))));
    codes.extend(fish_ids.iter().map(|id| format!("fish:{id}")));
    codes
}
