//! The town tree (`town-missions-tree`): a second chain, apart from the
//! starter tree, read from `assets/config/town_missions.ron` and counted by
//! the same [`Progress::record`] into its own player record,
//! `town_missions`.
//!
//! - **It opens on reaching a town** (design 1): the first `reach town`
//!   writes the record. Until then there is no record and no tab.
//! - **Its coins are paid once** (design 5): the purse's line carries the
//!   coins and the mark that the mission is paid ([`Purse::pay_once`]), and
//!   is written before the record that marks the mission done, so no crash
//!   pays a mission twice or finishes it unpaid.
//! - **An old world** that holds proof of a town visit opens the tree on
//!   load at its root, with nothing paid ([`been_to_town`]).

use super::{Chain, Progress};
use crate::economy::coins::{PURSE_ID, PURSE_KIND, Purse, Reason};
use crate::economy::follow::LOST_KIND;
use crate::economy::goodwill::GOODWILL_KIND;
use crate::economy::jobs::{JOBS_ID, JOBS_KIND, TakenJobs};
use crate::records::Records;
use serde::{Deserialize, Serialize};

/// The record kind, its schema and its one id (the player's).
pub const TOWN_MISSIONS_KIND: &str = "town_missions";
pub const TOWN_MISSIONS_SCHEMA: u32 = 1;
pub const TOWN_MISSIONS_ID: u64 = 0;

/// The starter tree's "Find a town", whose being done is proof of a visit.
pub const STARTER_TOWN: &str = "town";

/// The `town_missions` record's body: the game day the tree opened and the
/// starter tree's own kind of progress.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct TownRecord {
    pub opened: u64,
    pub progress: Progress,
}

/// Whether a world with no `town_missions` record shows that the player has
/// been to a town (design 5): a purse, any town's goodwill, a taken or
/// handed-in job, a lost villager, or the starter tree's "Find a town" done.
pub fn been_to_town(records: &Records, starter: &Progress) -> bool {
    records.get(PURSE_KIND, PURSE_ID).is_some()
        || records.of_kind(GOODWILL_KIND).next().is_some()
        || records.of_kind(LOST_KIND).next().is_some()
        || records
            .get(JOBS_KIND, JOBS_ID)
            .and_then(|r| r.read::<TakenJobs>())
            .is_some_and(|j| !j.taken.is_empty() || !j.handed_in.is_empty())
        || starter.is_done(STARTER_TOWN)
}

/// The key a town mission's coins are paid under in the purse.
pub fn paid_key(id: &str) -> String {
    format!("{TOWN_MISSIONS_KIND}:{id}")
}

/// Pay `done`'s coins into `purse`, each mission once: the coins paid. The
/// purse marks each mission paid in its own line, written before the
/// record that marks it done, so a torn write between them leaves a mission
/// paid and finished again on the next count, never paid twice.
pub fn pay(chain: &Chain, done: &[String], purse: &mut Purse) -> u64 {
    let mut paid = 0;
    for id in done {
        let Some(m) = chain.get(id).filter(|m| m.coins > 0) else {
            continue;
        };
        if purse
            .pay_once(Reason::Mission, &paid_key(id), u64::from(m.coins))
            .is_some()
        {
            paid += u64::from(m.coins);
        }
    }
    paid
}

#[cfg(test)]
mod tests;
