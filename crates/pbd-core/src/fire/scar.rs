//! What a fire leaves, and its regrowth (design 8): ash, standing charred
//! trees, then grass, shrubs, saplings and full trees by world days of play
//! since each cell burnt out. Derived from the cell's `burnt` record; the
//! ground's own blocks are never edited.

use super::config::FireConfig;
use super::dryness::Plant;
use crate::daylight::DAY_S;
use serde::{Deserialize, Serialize};

/// A burnt cell's record body, its id the cell's key.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Burnt {
    /// The blaze that burnt it.
    pub blaze: u64,
    /// The world second it burnt out.
    pub out_s: f64,
    /// What burnt.
    pub plant: Plant,
    /// A conifer that burnt in its crown.
    pub crown: bool,
}

impl Burnt {
    /// World days since it burnt out.
    pub fn days(&self, now_s: f64) -> f64 {
        ((now_s - self.out_s) / f64::from(DAY_S)).max(0.0)
    }
}

/// How a burnt cell's ground looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroundLook {
    /// Ash grey, nothing growing.
    Ash,
    /// Ash with green shoots.
    Shoots,
    /// Its grass is back.
    Grass,
}

const SALT_REGROW: u32 = 0x61;

/// The day a cell's grass is back: between `grass` and `grass_whole`, by its
/// own roll, so a scar greens in patches.
pub fn grass_back_day(cfg: &FireConfig, cell: u32) -> f64 {
    let r = &cfg.regrow_days;
    r.grass + (r.grass_whole - r.grass) * f64::from(crate::tree_kit::roll(cell, SALT_REGROW))
}

pub fn ground_look(cfg: &FireConfig, cell: u32, burnt: &Burnt, now_s: f64) -> GroundLook {
    let days = burnt.days(now_s);
    if days >= grass_back_day(cfg, cell) {
        GroundLook::Grass
    } else if days >= cfg.regrow_days.shoots {
        GroundLook::Shoots
    } else {
        GroundLook::Ash
    }
}

/// How a burnt tree stands.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreeLook {
    /// A charred snag: trunk, no crown (a crown fire).
    Snag,
    /// Standing, its foot charred and its leaves browned (a ground fire).
    Scorched,
    /// A sapling of its own species, `grown` 0 at its day to 1 a full tree.
    Sapling { grown: f32 },
    /// The tree as before.
    Whole,
}

pub fn tree_look(cfg: &FireConfig, burnt: &Burnt, now_s: f64) -> TreeLook {
    let r = &cfg.regrow_days;
    let days = burnt.days(now_s);
    if days >= r.tree {
        TreeLook::Whole
    } else if days >= r.sapling {
        TreeLook::Sapling {
            grown: ((days - r.sapling) / (r.tree - r.sapling)) as f32,
        }
    } else if burnt.crown {
        TreeLook::Snag
    } else {
        TreeLook::Scorched
    }
}

/// Whether a burnt cell's shrub is back.
pub fn shrub_back(cfg: &FireConfig, burnt: &Burnt, now_s: f64) -> bool {
    burnt.days(now_s) >= cfg.regrow_days.shrub
}

/// Whether a burnt cell has fuel again (design 8): its grass whole, or a
/// tree's cell its sapling. Until then its `plant_dry` reads 0.
pub fn burns_again(cfg: &FireConfig, burnt: &Burnt, now_s: f64) -> bool {
    let r = &cfg.regrow_days;
    let days = burnt.days(now_s);
    if burnt.plant.is_tree() {
        days >= r.sapling
    } else {
        days >= r.grass_whole
    }
}

/// What chopping a burnt tree gives while it stands charred: the snag's
/// good (`charcoal`) and its count; `None` once it has regrown.
pub fn snag_gives<'a>(cfg: &'a FireConfig, burnt: &Burnt, now_s: f64) -> Option<(&'a str, u16)> {
    (burnt.plant.is_tree()
        && matches!(
            tree_look(cfg, burnt, now_s),
            TreeLook::Snag | TreeLook::Scorched
        ))
    .then_some((cfg.snag.good.as_str(), cfg.snag.count))
}

/// Whether nothing of the scar shows any more: every part grown back.
pub fn healed(cfg: &FireConfig, cell: u32, burnt: &Burnt, now_s: f64) -> bool {
    let grass = ground_look(cfg, cell, burnt, now_s) == GroundLook::Grass;
    match burnt.plant {
        p if p.is_tree() => grass && tree_look(cfg, burnt, now_s) == TreeLook::Whole,
        Plant::Shrub => grass && shrub_back(cfg, burnt, now_s),
        _ => grass,
    }
}
