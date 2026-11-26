//! What a piece costs to build (`building-planner` design 8):
//! `assets/config/building_costs.ron`, goods per piece class and kit
//! family, the roof per cell it covers, all scaled by `cost_scale`. A kit
//! whose material the game does not make is locked, with its own reason.
//!
//! Goods are named as `recipes.ron` names items (`planks`,
//! `block:street lantern`), read by `station::item_named`.

use super::{Opening, Piece, Shape};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A list of `(item, count)`.
pub type Goods = Vec<(String, u32)>;

/// One family's costs.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FamilyCosts {
    pub wall: Goods,
    /// A wall with a doorway, and the door hung in it.
    pub door: Goods,
    pub window: Goods,
    pub floor: Goods,
    pub stair: Goods,
    pub post: Goods,
    pub lantern: Goods,
    /// A roof, per cell it covers.
    pub roof: Goods,
}

/// `assets/config/building_costs.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CostTable {
    /// Each kit's family: the column of the table it pays from.
    pub families: BTreeMap<String, String>,
    /// Kits a planned world cannot build yet, and the reason's id
    /// (`planner.locked.<id>`).
    pub locked: BTreeMap<String, String>,
    /// Each family's costs.
    pub costs: BTreeMap<String, FamilyCosts>,
}

/// Why a kit cannot be built: the locale key of its reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locked(pub String);

impl CostTable {
    /// The table the game ships.
    pub fn shipped() -> Self {
        ron::from_str(include_str!("../../../../assets/config/building_costs.ron"))
            .expect("building_costs.ron parses")
    }

    /// Every family named has its costs, and every good is one the game
    /// knows.
    pub fn validate(&self) -> Result<(), String> {
        for (kit, family) in &self.families {
            if !self.costs.contains_key(family) {
                return Err(format!("kit {kit}: no costs for family {family}"));
            }
            if self.locked.contains_key(kit) {
                return Err(format!("kit {kit} is both costed and locked"));
            }
        }
        for (family, row) in &self.costs {
            for list in [
                &row.wall,
                &row.door,
                &row.window,
                &row.floor,
                &row.stair,
                &row.post,
                &row.lantern,
                &row.roof,
            ] {
                for (item, n) in list {
                    if crate::station::item_named(item).is_none() {
                        return Err(format!("{family}: no such good {item}"));
                    }
                    if *n == 0 {
                        return Err(format!("{family}: {item} costs nothing"));
                    }
                }
            }
        }
        Ok(())
    }

    /// Whether a kit can be built, or the reason it cannot.
    pub fn kit(&self, kit: &str) -> Result<&FamilyCosts, Locked> {
        if let Some(why) = self.locked.get(kit) {
            return Err(Locked(format!("planner.locked.{why}")));
        }
        self.families
            .get(kit)
            .and_then(|f| self.costs.get(f))
            .ok_or_else(|| Locked("planner.locked.unknown".into()))
    }

    /// What `piece` costs at `scale`: each count times the cells it covers
    /// times the scale, rounded up, never below 1 where the table has one.
    pub fn cost_of(&self, piece: &Piece, scale: f32) -> Result<Goods, Locked> {
        let row = self.kit(&piece.kit)?;
        let (list, n) = match &piece.shape {
            Shape::Wall { opening, .. } => (
                match opening {
                    Opening::None => &row.wall,
                    Opening::Door => &row.door,
                    Opening::Window => &row.window,
                },
                1,
            ),
            Shape::Floor { .. } => (&row.floor, 1),
            Shape::Stair { .. } => (&row.stair, 1),
            // A beam is a post's timber laid across.
            Shape::Post { .. } | Shape::Beam { .. } => (&row.post, 1),
            Shape::Lantern { .. } => (&row.lantern, 1),
            Shape::Roof { cells, .. } => (&row.roof, cells.len().max(1) as u32),
            // Paid with its wall's door row; furniture by placing the piece.
            Shape::Door { .. } | Shape::Furniture { .. } => return Ok(Vec::new()),
        };
        Ok(list
            .iter()
            .map(|(item, count)| (item.clone(), scaled(*count * n, scale)))
            .collect())
    }

    /// The bill of materials of `pieces`, every good summed, in name order.
    pub fn bill<'a>(
        &self,
        pieces: impl IntoIterator<Item = &'a Piece>,
        scale: f32,
    ) -> Result<BTreeMap<String, u32>, Locked> {
        let mut out = BTreeMap::new();
        for piece in pieces {
            for (item, n) in self.cost_of(piece, scale)? {
                *out.entry(item).or_insert(0) += n;
            }
        }
        Ok(out)
    }
}

/// `count` at `scale`, rounded up and never below one.
pub fn scaled(count: u32, scale: f32) -> u32 {
    ((count as f32 * scale).ceil() as u32).max(1)
}

/// `share` of each good, rounded down, the goods left at nothing dropped.
pub fn refund(goods: &Goods, share: f32) -> Goods {
    goods
        .iter()
        .map(|(g, n)| (g.clone(), (*n as f32 * share).floor() as u32))
        .filter(|(_, n)| *n > 0)
        .collect()
}
