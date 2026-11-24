//! The purse (`townsfolk` design 5): a whole number of coins, never below
//! nought, kept as the player record `purse`. It is not a pack slot, so a
//! full pack never drops a coin, and it has no stack limit.
//!
//! Every change goes through [`Purse::change`], which names its [`Reason`]
//! and refuses a spend greater than the purse holds. What it hands back,
//! [`Entry`], is what the game logs.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The record kind, its schema and its one id (the player's).
pub const PURSE_KIND: &str = "purse";
pub const PURSE_SCHEMA: u32 = 1;
pub const PURSE_ID: u64 = 0;

/// Why the purse changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// A good sold to a trader.
    Sold,
    /// A good bought from a trader.
    Bought,
    /// A job's reward.
    Mission,
    /// Pay for work (hired work, later).
    Wage,
    /// Coins found in a town container (`furniture` design 9).
    Found,
}

impl Reason {
    pub const ALL: [Reason; 5] = [
        Reason::Sold,
        Reason::Bought,
        Reason::Mission,
        Reason::Wage,
        Reason::Found,
    ];

    /// Its name in the log and in data.
    pub fn code(self) -> &'static str {
        match self {
            Reason::Sold => "sold",
            Reason::Bought => "bought",
            Reason::Mission => "mission",
            Reason::Wage => "wage",
            Reason::Found => "found",
        }
    }

    /// Whether this reason takes coins out rather than putting them in.
    pub fn spends(self) -> bool {
        matches!(self, Reason::Bought)
    }
}

/// The player's coins.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Purse {
    pub coins: u64,
    /// The one-off rewards already paid, by key (`town_mission:<id>`,
    /// `town-missions-tree` design 5): the purse's own line says a reward is
    /// paid, so a reward is paid once whatever line follows it. Absent from a
    /// purse written before it.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub paid: BTreeSet<String>,
}

/// One accepted change, for the log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub reason: Reason,
    /// Coins in (positive) or out (negative).
    pub delta: i64,
    /// The purse after it.
    pub after: u64,
}

impl Entry {
    /// The log's line: `sold +12 40`, the reason, the change and the purse.
    pub fn log_line(&self) -> String {
        let (reason, delta, after) = (self.reason.code(), self.delta, self.after);
        format!("{reason} {delta:+} {after}")
    }
}

/// Why a change was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// A spend greater than the purse holds.
    Short { have: u64, need: u64 },
}

impl Purse {
    pub fn new(coins: u64) -> Self {
        Self {
            coins,
            paid: BTreeSet::new(),
        }
    }

    /// Pay a one-off reward of `coins` under `key`, unless it was paid
    /// before: the entry, or `None` when it already was.
    pub fn pay_once(&mut self, reason: Reason, key: &str, coins: u64) -> Option<Entry> {
        if reason.spends() || !self.paid.insert(key.to_string()) {
            return None;
        }
        self.change(reason, coins).ok()
    }

    /// Whether the purse covers a spend of `coins`.
    pub fn covers(&self, coins: u64) -> bool {
        self.coins >= coins
    }

    /// The one rule that changes the purse: `coins` in for an earning
    /// reason, out for a spending one ([`Reason::spends`]). A spend past the
    /// purse is refused and leaves it as it was. Nought changes nothing and
    /// is accepted.
    pub fn change(&mut self, reason: Reason, coins: u64) -> Result<Entry, Refused> {
        let after = if reason.spends() {
            if !self.covers(coins) {
                return Err(Refused::Short {
                    have: self.coins,
                    need: coins,
                });
            }
            self.coins - coins
        } else {
            self.coins.saturating_add(coins)
        };
        let delta = after as i64 - self.coins as i64;
        self.coins = after;
        Ok(Entry {
            reason,
            delta,
            after,
        })
    }
}
