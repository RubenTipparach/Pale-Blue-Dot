//! Coins, trade and town jobs (`townsfolk` design 5 to 12): the one
//! authority for what a good is worth, what a town pays for it, what the
//! purse holds, and the jobs a town posts. `pbd-app` feeds these rules and
//! stores their records; it never prices or pays by a rule of its own, so a
//! future server pays the same way.
//!
//! - [`coins`]: the purse and the one function that changes it.
//! - [`values`]: every item's value in coins, `assets/config/values.ron`.
//! - [`trade`]: a town's prices, stock and their drift back to rest.
//! - [`goodwill`]: each town's goodwill toward the player.
//! - [`derive`]: a made good's value from its recipe.
//! - [`market`]: the world's market index per good.
//! - [`jobs`]: a town's board and the jobs the player has taken.
//! - [`follow`]: how a found villager walks behind the player.

pub mod coins;
pub mod derive;
pub mod follow;
pub mod goodwill;
pub mod jobs;
pub mod market;
pub mod road_jobs;
pub mod trade;
pub mod values;

#[cfg(test)]
mod derive_tests;
#[cfg(test)]
mod tests;
