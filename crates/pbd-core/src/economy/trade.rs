//! A town's trader (`townsfolk` design 6 and 7): what it sells and buys,
//! at what price, how much it holds, and how its prices move with your
//! trade and drift back over a day.
//!
//! - **Price.** A good's rest price in a town is its value times the town's
//!   need for its group ([`TradeConfig::need`], 0.5 where the town makes it,
//!   2.0 where it lacks it). The trader sells at that and buys at 60% of it
//!   (the spread), each moved by the good's factor and the town's goodwill
//!   ([`super::goodwill`]). Coins are whole: a selling price rounds up, a
//!   buying price down, and a buying price is always at least a coin under
//!   the selling one, so buying and selling back always loses.
//! - **Stock.** A trader sells what its town makes (a group whose need is at
//!   most [`TradeConfig::makes_below`]) and its staples (bread), a stock of
//!   each sized by the town; it buys every good with a value, wanting a
//!   quantity of each.
//! - **Movement.** Each sale to the town lowers the good's factor by
//!   [`TradeConfig::step`], each purchase raises it, within
//!   [`TradeConfig::low`] and [`TradeConfig::high`]. The factor, the stock
//!   sold and the quantity bought all return to rest in a straight line over
//!   [`TradeConfig::rest_days`].
//! - **The record.** A [`Market`] is stored as `town_market` under the
//!   town's site id only once the player trades there; an unvisited town is
//!   its rest and costs nothing.

use super::coins::{Entry, Purse, Reason};
use super::values::{Group, Values, item_of};
use crate::daylight::DAY_S;
use crate::inventory::Slots;
use crate::sites::SiteKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The record kind and schema; the id is the town's site id.
pub const MARKET_KIND: &str = "town_market";
pub const MARKET_SCHEMA: u32 = 1;

/// `assets/config/trade.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TradeConfig {
    /// What the trader pays, as a share of its selling price (0.6: the 40%
    /// spread).
    pub buy_share: f32,
    /// How far one sale moves a good's factor, as a share (0.02).
    pub step: f32,
    /// The factor's range.
    pub low: f32,
    pub high: f32,
    /// Game days for a market to return to rest.
    pub rest_days: f32,
    /// A town makes, and so sells, a group whose need is at most this.
    pub makes_below: f32,
    /// What every trader sells, whatever its town makes.
    pub staples: Vec<String>,
    /// Codes no trader sells (animals caught on a line, a mussel bed).
    #[serde(default)]
    pub unsold: Vec<String>,
    /// Codes no trader buys (a delivery's parcel).
    #[serde(default)]
    pub unbought: Vec<String>,
    /// Groups no trader buys or sells.
    #[serde(default)]
    pub untraded: Vec<Group>,
    /// A size-1 town's stock of each good it sells, and its want of each
    /// good it buys.
    pub stock: u32,
    pub want: u32,
    /// Each kind of town's size (its stock and want scale by it); 1 where a
    /// kind is not named.
    #[serde(default)]
    pub size: BTreeMap<SiteKind, f32>,
    /// Each kind of town's need, by group; 1 where a group is not named.
    #[serde(default)]
    pub needs: BTreeMap<SiteKind, BTreeMap<Group, f32>>,
    pub goodwill: super::goodwill::GoodwillConfig,
}

impl TradeConfig {
    pub fn parse(text: &str) -> Result<TradeConfig, String> {
        let cfg: TradeConfig = ron::from_str(text).map_err(|e| format!("trade.ron: {e}"))?;
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), String> {
        let unit = |v: f32| v.is_finite() && v > 0.0;
        if !(unit(self.buy_share) && self.buy_share < 1.0) {
            return Err("trade: buy_share must be in (0, 1)".into());
        }
        if !(unit(self.step) && self.step < 0.5) {
            return Err("trade: step must be in (0, 0.5)".into());
        }
        if !(unit(self.low) && self.low <= 1.0 && self.high >= 1.0 && self.high.is_finite()) {
            return Err("trade: low <= 1 <= high".into());
        }
        if !unit(self.rest_days) || !self.makes_below.is_finite() {
            return Err("trade: rest_days and makes_below".into());
        }
        for (kind, needs) in &self.needs {
            for (group, need) in needs {
                if !(0.5..=2.0).contains(need) {
                    return Err(format!(
                        "trade: {kind:?} {group:?} need {need} not in 0.5..2"
                    ));
                }
            }
        }
        if self.size.values().any(|s| !unit(*s)) {
            return Err("trade: a size must be positive".into());
        }
        self.goodwill.validate()
    }

    /// A kind of town's need for a group, 0.5 to 2.
    pub fn need(&self, kind: SiteKind, group: Group) -> f32 {
        self.needs
            .get(&kind)
            .and_then(|n| n.get(&group))
            .copied()
            .unwrap_or(1.0)
    }

    pub fn size(&self, kind: SiteKind) -> f32 {
        self.size.get(&kind).copied().unwrap_or(1.0)
    }

    fn scaled(&self, n: u32, kind: SiteKind) -> u32 {
        ((n as f32 * self.size(kind)).round() as u32).max(1)
    }
}

/// One good's state in a town that has been traded with: its factor and
/// how far its stock and want are from rest.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lot {
    /// The price's factor, 1 at rest.
    pub factor: f32,
    /// How many the town has sold since rest (its stock is down by this).
    pub sold: f32,
    /// How many the town has bought since rest (its want is down by this).
    pub bought: f32,
}

impl Default for Lot {
    fn default() -> Self {
        Self {
            factor: 1.0,
            sold: 0.0,
            bought: 0.0,
        }
    }
}

/// A town's market as the record stores it: the world time it was last
/// settled and each good that has moved from rest.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Market {
    /// World seconds when it was last settled.
    pub at_s: f64,
    pub lots: BTreeMap<String, Lot>,
}

/// A trader: the rules, the values, its town and the goodwill there.
#[derive(Clone, Copy, Debug)]
pub struct Trader<'a> {
    pub cfg: &'a TradeConfig,
    pub values: &'a Values,
    pub kind: SiteKind,
    /// The town's goodwill toward the player, points.
    pub goodwill: u32,
    /// The body's fish roster by index, to name a fish in the pack.
    pub fish_ids: &'a [String],
}

/// What the trader does with one good now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quote {
    /// What it sells one for; `None` where it does not sell it.
    pub sell: Option<u64>,
    /// What it pays for one; `None` where it will not buy it.
    pub buy: Option<u64>,
    /// How many it has to sell.
    pub stock: u32,
    /// How many more it wants to buy.
    pub wants: u32,
}

/// Why a trade was refused. Nothing changes on a refusal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The purse does not cover the price.
    Short,
    /// The trader does not sell this.
    NotSold,
    /// None left.
    SoldOut,
    /// The pack has no room for it.
    PackFull,
    /// The trader will not buy this, or wants no more.
    NotWanted,
    /// The pack holds none of it.
    NotHeld,
}

/// An accepted trade.
#[derive(Clone, Debug, PartialEq)]
pub struct Deal {
    pub code: String,
    pub coins: u64,
    pub entry: Entry,
}

impl Market {
    /// The market `now_s`: each lot moved toward rest by the days since it
    /// was last settled, and a lot back at rest dropped.
    pub fn settled(&self, now_s: f64, cfg: &TradeConfig) -> Market {
        let days = ((now_s - self.at_s) / DAY_S as f64).max(0.0) as f32;
        let keep = (1.0 - days / cfg.rest_days).clamp(0.0, 1.0);
        let lots = self
            .lots
            .iter()
            .filter_map(|(code, lot)| {
                let lot = Lot {
                    factor: 1.0 + (lot.factor - 1.0) * keep,
                    sold: lot.sold * keep,
                    bought: lot.bought * keep,
                };
                let rest = (lot.factor - 1.0).abs() < 1e-4 && lot.sold < 0.01 && lot.bought < 0.01;
                (!rest).then(|| (code.clone(), lot))
            })
            .collect();
        Market {
            at_s: now_s.max(self.at_s),
            lots,
        }
    }

    fn lot(&self, code: &str) -> Lot {
        self.lots.get(code).copied().unwrap_or_default()
    }
}

impl Trader<'_> {
    /// Whether this trader sells `code` at all.
    pub fn sells(&self, code: &str) -> bool {
        let Some(value) = self.values.get(code) else {
            return false;
        };
        if self.cfg.untraded.contains(&value.group) || self.cfg.unsold.iter().any(|u| u == code) {
            return false;
        }
        self.cfg.staples.iter().any(|s| s == code)
            || self.cfg.need(self.kind, value.group) <= self.cfg.makes_below
    }

    /// Whether this trader buys `code` at all.
    pub fn buys(&self, code: &str) -> bool {
        !self.cfg.unbought.iter().any(|u| u == code)
            && self
                .values
                .get(code)
                .is_some_and(|v| !self.cfg.untraded.contains(&v.group))
    }

    /// Every code it sells, in values order: the trade panel's list.
    pub fn stock_list(&self) -> Vec<String> {
        self.values
            .codes()
            .filter(|c| self.sells(c))
            .map(str::to_string)
            .collect()
    }

    /// A good's rest price here: value times need, before the factor.
    pub fn rest_price(&self, code: &str) -> Option<f32> {
        let value = self.values.get(code)?;
        Some(value.coins as f32 * self.cfg.need(self.kind, value.group))
    }

    /// What the trader does with `code` in `market` (settled already).
    pub fn quote(&self, market: &Market, code: &str) -> Quote {
        let Some(rest) = self.rest_price(code) else {
            return Quote {
                sell: None,
                buy: None,
                stock: 0,
                wants: 0,
            };
        };
        let lot = market.lot(code);
        let bonus = self.cfg.goodwill.bonus(self.goodwill);
        let raw = rest * lot.factor;
        let sell = ((raw * (1.0 - bonus)).ceil() as u64).max(1);
        let pays = (raw * self.cfg.buy_share * (1.0 + bonus)).floor() as u64;
        let pays = pays.min(sell.saturating_sub(1));
        let stock = if self.sells(code) {
            (self.cfg.scaled(self.cfg.stock, self.kind) as f32 - lot.sold)
                .floor()
                .max(0.0) as u32
        } else {
            0
        };
        let wants = if self.buys(code) {
            (self.cfg.scaled(self.cfg.want, self.kind) as f32 - lot.bought)
                .floor()
                .max(0.0) as u32
        } else {
            0
        };
        Quote {
            sell: self.sells(code).then_some(sell),
            buy: (self.buys(code) && pays > 0).then_some(pays),
            stock,
            wants,
        }
    }

    /// The player buys one `code`: the price leaves the purse, the good
    /// enters the slots, the town's stock falls and its price rises. All of
    /// it or none of it. `market` is settled to `now_s` first.
    pub fn buy(
        &self,
        market: &mut Market,
        purse: &mut Purse,
        slots: &mut Slots,
        code: &str,
        now_s: f64,
    ) -> Result<Deal, Refused> {
        let mut m = market.settled(now_s, self.cfg);
        let quote = self.quote(&m, code);
        let price = quote.sell.ok_or(Refused::NotSold)?;
        if quote.stock == 0 {
            return Err(Refused::SoldOut);
        }
        if !purse.covers(price) {
            return Err(Refused::Short);
        }
        let item = item_of(code, self.fish_ids).ok_or(Refused::NotSold)?;
        let mut s = slots.clone();
        if s.give(item, 1) != 0 {
            return Err(Refused::PackFull);
        }
        let mut p = purse.clone();
        let entry = p
            .change(Reason::Bought, price)
            .map_err(|_| Refused::Short)?;
        let lot = m.lots.entry(code.to_string()).or_default();
        lot.factor = (lot.factor * (1.0 + self.cfg.step)).min(self.cfg.high);
        lot.sold += 1.0;
        *market = m;
        *purse = p;
        *slots = s;
        Ok(Deal {
            code: code.into(),
            coins: price,
            entry,
        })
    }

    /// The player sells one `code` from the slots: the good leaves the pack,
    /// its price enters the purse, the town's want falls and so does its
    /// price. All of it or none of it.
    pub fn sell(
        &self,
        market: &mut Market,
        purse: &mut Purse,
        slots: &mut Slots,
        code: &str,
        now_s: f64,
    ) -> Result<Deal, Refused> {
        let mut m = market.settled(now_s, self.cfg);
        let quote = self.quote(&m, code);
        let price = quote.buy.ok_or(Refused::NotWanted)?;
        if quote.wants == 0 {
            return Err(Refused::NotWanted);
        }
        let item = item_of(code, self.fish_ids).ok_or(Refused::NotHeld)?;
        let mut s = slots.clone();
        if !s.take_item(item, 1) {
            return Err(Refused::NotHeld);
        }
        let mut p = purse.clone();
        let entry = p.change(Reason::Sold, price).map_err(|_| Refused::Short)?;
        let lot = m.lots.entry(code.to_string()).or_default();
        lot.factor = (lot.factor * (1.0 - self.cfg.step)).max(self.cfg.low);
        lot.bought += 1.0;
        *market = m;
        *purse = p;
        *slots = s;
        Ok(Deal {
            code: code.into(),
            coins: price,
            entry,
        })
    }
}
