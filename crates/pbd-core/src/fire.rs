//! Fire in a Survival world (`openspec/changes/wildfire`): how dry each
//! plant is, small forest fires that start in the dry season and spread by
//! one rule the server runs, what stops them, the scar they leave and its
//! regrowth, and torches the rain puts out.
//!
//! - [`dryness`]: the fire weather and `plant_dry` (design 2), derived and
//!   never saved; green plants (below `dry_min`) never catch.
//! - [`blaze`]: the `blaze` record, its one-second tick in cell-key order
//!   with seeded draws, ignition and what stops it (design 3 to 7).
//! - [`scar`]: the `burnt` records' looks by days since (design 8).
//! - [`torch`]: torches in the rain (design 9).
//!
//! The ground's wetness, the one new state, is the atmosphere's
//! (`Atmosphere::ground_wet`). Every number is `assets/config/fire.ron`'s
//! ([`FireConfig`]). None of it runs in an Explorer world.

pub mod blaze;
pub mod config;
pub mod dryness;
pub mod scar;
#[cfg(test)]
mod tests;
pub mod torch;

pub use blaze::{Air, Blaze, Burning, Ground, Land, Lighting, LitBy, State, Step, World};
pub use config::{FireConfig, GroundWetRates};
pub use dryness::{Factors, Plant};
pub use scar::{Burnt, GroundLook, TreeLook};

/// Every record kind fire writes, for the save's kind list.
pub fn record_kinds() -> Vec<(&'static str, u32)> {
    blaze::KINDS
        .iter()
        .chain(torch::KINDS.iter())
        .copied()
        .collect()
}

/// Health lost a second (design 10): in a burning cell, beside one, or
/// neither.
pub fn harm_per_s(cfg: &FireConfig, in_cell: bool, beside: bool) -> f32 {
    if in_cell {
        cfg.harm.in_cell_per_s
    } else if beside {
        cfg.harm.beside_per_s
    } else {
        0.0
    }
}
