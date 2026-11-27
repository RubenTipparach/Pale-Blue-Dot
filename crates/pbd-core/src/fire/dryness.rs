//! How dry a plant is (design 2): the fire weather of a place, each plant's
//! cure in its own hemisphere's year, and `plant_dry`, derived and never
//! saved. Green plants do not burn: below `dry_min` nothing catches.

use super::config::{Dryness, FireConfig};
use crate::tree_kit::{self, Species};
use serde::{Deserialize, Serialize};

/// What grows on a cell, as fire sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Plant {
    /// Nothing that burns: water, sand, rock, snow, soil, a road, a path, a
    /// paved or tilled cell.
    None,
    Grass,
    Shrub,
    /// Oak, birch, willow.
    Deciduous,
    /// Pine, krummholz.
    Conifer,
    /// Kapok, palm, acacia: broadleaf that keeps its leaves.
    Evergreen,
    Cactus,
}

impl Plant {
    /// The plant a tree of `species` is.
    pub fn of_tree(species: Species) -> Plant {
        match species {
            Species::Cactus => Plant::Cactus,
            Species::Pine | Species::Krummholz => Plant::Conifer,
            s if tree_kit::look(s).deciduous => Plant::Deciduous,
            _ => Plant::Evergreen,
        }
    }

    pub fn is_tree(self) -> bool {
        matches!(self, Plant::Deciduous | Plant::Conifer | Plant::Evergreen)
    }

    /// Whether anything on the cell can burn.
    pub fn burns(self) -> bool {
        !matches!(self, Plant::None | Plant::Cactus)
    }
}

pub fn smooth(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// One at `peak` on the circle of the year, nought `half` either side.
fn bump(x: f32, peak: f32, half: f32) -> f32 {
    let d = (x - peak).rem_euclid(1.0);
    1.0 - smooth(0.0, half, d.min(1.0 - d))
}

/// The fire weather at a place, 0..1 (design 2): `(1 - ground_wet) x
/// air(humidity) x warm(air_c)`.
pub fn weather(cfg: &FireConfig, ground_wet: f32, humidity: f32, air_c: f32) -> f32 {
    let w = &cfg.weather;
    let air = 1.0 - w.humid_keep * smooth(w.humid_dry, w.humid_wet, humidity);
    let warm = w.warm_floor + (1.0 - w.warm_floor) * smooth(w.warm_from_c, w.warm_full_c, air_c);
    ((1.0 - ground_wet.clamp(0.0, 1.0)) * air * warm).clamp(0.0, 1.0)
}

/// How cured the plants of a place are in its own year, 0..1, and its
/// fallen leaves: `c = max(dry(local), straw_share x straw)` (design 2).
/// `year_fraction` is the clock's (`daylight::Clock::year_fraction`),
/// `latitude_sin` the place's direction's +Y.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cured {
    pub c: f32,
    pub litter: f32,
}

pub fn cured(cfg: &FireConfig, year_fraction: f64, latitude_sin: f32) -> Cured {
    let s = &cfg.dryness.season;
    let flora = crate::flora::season_at(year_fraction, latitude_sin);
    let own = if latitude_sin < 0.0 {
        year_fraction + 0.5
    } else {
        year_fraction
    };
    let local = own.rem_euclid(1.0) as f32;
    let strength = smooth(
        crate::flora::SEASON_TROPIC[0],
        crate::flora::SEASON_TROPIC[1],
        latitude_sin.abs(),
    );
    let dry = strength * bump(local, s.dry_peak, s.dry_half);
    Cured {
        c: dry.max(s.straw_share * flora.straw).clamp(0.0, 1.0),
        litter: flora.litter,
    }
}

/// A plant's cure from the season (design 2's table).
pub fn cure(d: &Dryness, plant: Plant, cured: Cured) -> f32 {
    let row = match plant {
        Plant::None => return 0.0,
        Plant::Grass => d.cure.grass,
        Plant::Shrub => d.cure.shrub,
        Plant::Deciduous => d.cure.deciduous,
        Plant::Conifer => d.cure.conifer,
        Plant::Evergreen => d.cure.evergreen,
        Plant::Cactus => d.cure.cactus,
    };
    row.of(cured.c, cured.litter).clamp(0.0, 1.0)
}

/// Every factor of `plant_dry`, each already worked out (design 2's table).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Factors {
    pub plant: Plant,
    pub cured: Cured,
    pub weather: f32,
    pub shade: f32,
    pub water: f32,
    pub hollow: f32,
    pub patch: f32,
    pub grazed: f32,
    /// Snow lies, the cell is flooded, or it burnt and has not grown back.
    pub zeroed: bool,
}

impl Factors {
    /// An open, level cell with no neighbour trees, no water near and
    /// `patch` 1: design 2's examples' place.
    pub fn open(plant: Plant, cured: Cured, weather: f32) -> Self {
        Self {
            plant,
            cured,
            weather,
            shade: 1.0,
            water: 1.0,
            hollow: 1.0,
            patch: 1.0,
            grazed: 1.0,
            zeroed: false,
        }
    }
}

/// `plant_dry`, 0..1 (design 2).
pub fn plant_dry(cfg: &FireConfig, f: &Factors) -> f32 {
    if f.zeroed || !f.plant.burns() {
        return 0.0;
    }
    (cure(&cfg.dryness, f.plant, f.cured)
        * f.weather
        * f.shade
        * f.water
        * f.hollow
        * f.patch
        * f.grazed)
        .clamp(0.0, 1.0)
}

/// How much of a fully dry cell's hazard a cell of dryness `d` carries:
/// nought below `dry_min`, then the margin's square (design 2).
pub fn burn(cfg: &FireConfig, d: f32) -> f32 {
    let lo = cfg.dryness.dry_min;
    if d < lo {
        return 0.0;
    }
    ((d - lo) / (1.0 - lo))
        .clamp(0.0, 1.0)
        .powf(cfg.dryness.burn_power)
}

/// Whether a cell of dryness `d` can catch at all.
pub fn can_catch(cfg: &FireConfig, d: f32) -> bool {
    d >= cfg.dryness.dry_min
}

/// Grass and shrubs under trees: `1 - per_tree x trees round`, times
/// `own_tree` under a tree of its own.
pub fn shade(d: &Dryness, plant: Plant, trees_round: usize, own_tree: bool) -> f32 {
    if !matches!(plant, Plant::Grass | Plant::Shrub) {
        return 1.0;
    }
    let s = (1.0 - d.shade.per_tree * trees_round as f32).max(0.0);
    if own_tree { s * d.shade.own_tree } else { s }
}

/// Water near: `bank + (1 - bank) x smooth(from, to, cells to water)`;
/// `cells` is `None` where no water is within reach (it counts as far).
pub fn water(d: &Dryness, cells: Option<u32>) -> f32 {
    let w = &d.water;
    let at = cells.map_or(w.to_cells, |c| c as f32);
    w.bank + (1.0 - w.bank) * smooth(w.from_cells, w.to_cells, at)
}

/// A hollow stays greener, a rise dries first: `1 + per_layer x clamp(rise)`.
pub fn hollow(d: &Dryness, layers_above_mean: f32) -> f32 {
    let h = &d.hollow;
    1.0 + h.per_layer * layers_above_mean.clamp(-h.clamp_layers, h.clamp_layers)
}

/// The soil's own patches: a smooth field `n` (0..1) and a cell's roll.
pub fn patch(d: &Dryness, n: f32, roll: f32) -> f32 {
    let p = &d.patch;
    (p.base + p.span * n.clamp(0.0, 1.0)) * (p.cell_base + p.cell_span * roll.clamp(0.0, 1.0))
}

/// Grazed short: `mult` on and near a pen's cells, for grass.
pub fn grazed(d: &Dryness, plant: Plant, near_pen: bool) -> f32 {
    if near_pen && plant == Plant::Grass {
        d.grazed.mult
    } else {
        1.0
    }
}
