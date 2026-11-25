//! The world's market (`townsfolk` design 37 and 37a): an index per good,
//! 1.0 at rest, moved by the player's trade and by the towns' making and
//! using each day, drifting back toward rest over days. It is a stored
//! record per world (`market`), written on a trade or at a day boundary
//! that moved a row; a world with none reads every index as 1.0.
//!
//! - **Trade.** Each unit sold to any trader lowers its good's index by
//!   [`MarketConfig::step`] as a share, each unit bought raises it, within
//!   [`MarketConfig::low`] and [`MarketConfig::high`].
//! - **A day.** Each index moves back toward 1.0 by `1 / rest_days` of its
//!   distance, then by [`MarketConfig::town_pull`] times the towns'
//!   [`balance`] for the good: the mean over every town of its need for the
//!   good's group less one.
//! - **Settling** is lazy and the same however it is read: the record steps
//!   day by day from its day to today, at most `rest_days × 8` days.
//!
//! The phase 6 stock market (M12) reads this index as its price ledger.

use super::trade::TradeConfig;
use super::values::Values;
use crate::sites::SiteKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The record kind, its one id and its schema.
pub const WORLD_MARKET_KIND: &str = "market";
pub const WORLD_MARKET_ID: u64 = 0;
pub const WORLD_MARKET_SCHEMA: u32 = 1;

/// An index this close to 1.0 is at rest and has no row.
const REST: f32 = 1e-4;
/// A row moved by more than this at a day boundary is written.
pub const WORTH_WRITING: f32 = 1e-3;

/// `trade.ron`'s `market` block.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MarketConfig {
    /// How far one unit traded moves its good's index, as a share.
    pub step: f32,
    /// The index's range.
    pub low: f32,
    pub high: f32,
    /// Game days over which an index returns to rest.
    pub rest_days: f32,
    /// How far a day of the towns' making and using moves an index, per
    /// unit of balance.
    pub town_pull: f32,
}

impl Default for MarketConfig {
    fn default() -> Self {
        Self {
            step: 0.005,
            low: 0.5,
            high: 2.0,
            rest_days: 8.0,
            town_pull: 0.02,
        }
    }
}

impl MarketConfig {
    pub fn validate(&self) -> Result<(), String> {
        let unit = |v: f32| v.is_finite() && v > 0.0;
        if !(unit(self.step) && self.step < 0.5) {
            return Err("trade: market.step must be in (0, 0.5)".into());
        }
        if !(unit(self.low) && self.low <= 1.0 && self.high >= 1.0 && self.high.is_finite()) {
            return Err("trade: market.low <= 1 <= market.high".into());
        }
        if !(self.rest_days.is_finite() && self.rest_days >= 1.0) {
            return Err("trade: market.rest_days must be at least 1".into());
        }
        if !(self.town_pull.is_finite() && (0.0..0.5).contains(&self.town_pull)) {
            return Err("trade: market.town_pull must be in [0, 0.5)".into());
        }
        Ok(())
    }

    /// The most days a settle steps.
    fn most_days(&self) -> u64 {
        (self.rest_days * 8.0).ceil() as u64
    }
}

/// The record's body: the game day it was last settled and each good's
/// index that is not at rest.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WorldMarket {
    pub day: u64,
    pub index: BTreeMap<String, f32>,
}

/// The towns' making and using of a good's group: the mean over `towns`
/// of its need less one. 0 with no towns or no row.
pub fn balance(cfg: &TradeConfig, values: &Values, towns: &[SiteKind], code: &str) -> f32 {
    let Some(value) = values.get(code) else {
        return 0.0;
    };
    if towns.is_empty() {
        return 0.0;
    }
    let mut counts: BTreeMap<SiteKind, u32> = BTreeMap::new();
    for kind in towns {
        *counts.entry(*kind).or_default() += 1;
    }
    let sum: f32 = counts
        .iter()
        .map(|(kind, n)| (cfg.need(*kind, value.group) - 1.0) * *n as f32)
        .sum();
    sum / towns.len() as f32
}

impl WorldMarket {
    /// A market at rest, settled to `day`: what a world with no record reads.
    pub fn fresh(day: u64) -> Self {
        Self {
            day,
            index: BTreeMap::new(),
        }
    }

    /// A good's index, 1.0 at rest.
    pub fn index(&self, code: &str) -> f32 {
        self.index.get(code).copied().unwrap_or(1.0)
    }

    /// The market on `today`: each day from its own stepped back toward
    /// rest and by the towns' `balance`. Every code of `values` is stepped,
    /// in their order, so it is the same however it is read.
    pub fn settled(
        &self,
        today: u64,
        cfg: &MarketConfig,
        values: &Values,
        balance: impl Fn(&str) -> f32,
    ) -> WorldMarket {
        if today <= self.day {
            return self.clone();
        }
        let days = (today - self.day).min(cfg.most_days());
        let keep = 1.0 - 1.0 / cfg.rest_days;
        let pulls: Vec<(&str, f32)> = values
            .codes()
            .map(|c| (c, cfg.town_pull * balance(c)))
            .filter(|(c, p)| *p != 0.0 || self.index.contains_key(*c))
            .collect();
        let mut index = self.index.clone();
        for _ in 0..days {
            for &(code, pull) in &pulls {
                let now = index.get(code).copied().unwrap_or(1.0);
                let next = (1.0 + (now - 1.0) * keep + pull).clamp(cfg.low, cfg.high);
                if (next - 1.0).abs() < REST {
                    index.remove(code);
                } else {
                    index.insert(code.to_string(), next);
                }
            }
        }
        // A code the values no longer name keeps its index as it was.
        WorldMarket { day: today, index }
    }

    /// `n` of `code` sold to a trader (`sold`) or bought from one.
    pub fn traded(&mut self, code: &str, n: u32, sold: bool, cfg: &MarketConfig) {
        let mut now = self.index(code);
        let by = if sold { 1.0 - cfg.step } else { 1.0 + cfg.step };
        for _ in 0..n {
            now = (now * by).clamp(cfg.low, cfg.high);
        }
        if (now - 1.0).abs() < REST {
            self.index.remove(code);
        } else {
            self.index.insert(code.to_string(), now);
        }
    }

    /// Whether any index differs from `other`'s by more than
    /// [`WORTH_WRITING`]: a day's settle worth a line.
    pub fn moved_from(&self, other: &WorldMarket) -> bool {
        self.index
            .keys()
            .chain(other.index.keys())
            .any(|c| (self.index(c) - other.index(c)).abs() > WORTH_WRITING)
    }
}
