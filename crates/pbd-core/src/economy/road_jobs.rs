//! Jobs on the roads (`townsfolk` design 46): a rockfall the world drops on
//! a town's road for the player to dig clear, and a trader who walks the
//! road to the next town while the player keeps up.
//!
//! Each is a stored record by the job's id (`road_rockfall`,
//! `road_escort`), so a reload finds it as it was. The rules here are pure:
//! where the rock falls, how many cells are clear, and the trader's step.

use crate::roads::{Kind as RoadKind, Road, RoadCell};
use serde::{Deserialize, Serialize};

pub const ROCKFALL_KIND: &str = "road_rockfall";
pub const ROCKFALL_SCHEMA: u32 = 1;
pub const ESCORT_KIND: &str = "road_escort";
pub const ESCORT_SCHEMA: u32 = 1;

/// How many of a road's cells out of town the rock falls from.
pub const ROCKFALL_FROM: usize = 8;
/// How near the trader the player comes before they set off, m.
pub const LEAD_START_M: f64 = 6.0;
/// Past this behind them the trader waits, m.
pub const WAIT_M: f64 = 12.0;
/// The trader's walking pace, m a second.
pub const LEAD_MPS: f64 = 1.6;

/// A rockfall on a road: its town, the cells and the layer the rock stands
/// in on each, and how many are clear and counted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rockfall {
    pub site: u32,
    pub cells: Vec<(u32, u16)>,
    pub cleared: u32,
}

/// A trader walked to the next town: their town, where they go, who they
/// are, the road's cells from their town to the other, how far along it
/// they are (cells, fractional) and whether they have set off.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Escort {
    pub site: u32,
    pub to: u32,
    pub person: u32,
    pub route: Vec<u32>,
    pub along: f64,
    pub started: bool,
}

/// A road's cells from settlement `site` outward, if the road touches it.
pub fn outward(road: &Road, site: u32) -> Option<Vec<RoadCell>> {
    if road.from_site == site {
        Some(road.cells.clone())
    } else if road.to_site == site {
        Some(road.cells.iter().rev().copied().collect())
    } else {
        None
    }
}

/// Where a rockfall lands on `cells` (a road from its town outward): from
/// the [`ROCKFALL_FROM`]th cell, the next `count` cells on the ground (not
/// a ford's or a bridge's), each with the first layer of air over its
/// ground, by `layer_of` (the altitude's layer). Fewer where the road is
/// short.
pub fn rockfall_cells(
    cells: &[RoadCell],
    count: usize,
    layer_of: impl Fn(f32) -> Option<usize>,
) -> Vec<(u32, u16)> {
    cells
        .iter()
        .skip(ROCKFALL_FROM.min(cells.len().saturating_sub(count)))
        .filter(|c| c.kind == RoadKind::Ground)
        .filter_map(|c| Some((c.key, layer_of(c.ground_m() as f32 + 0.5)? as u16)))
        .take(count)
        .collect()
}

/// How many of a rockfall's cells are clear: `material(cell, layer)` the
/// rock's layer as it stands now, air where it was dug.
pub fn cleared(fall: &Rockfall, is_air: impl Fn(u32, u16) -> bool) -> u32 {
    fall.cells.iter().filter(|(c, l)| is_air(*c, *l)).count() as u32
}

/// One step of a trader leading the player (design 46, the lost villager's
/// follow turned round): set off once the player is within
/// [`LEAD_START_M`]; then walk on at [`LEAD_MPS`] (faster if the player is
/// faster) while the player is within [`WAIT_M`], and wait where they stand
/// otherwise. `along` is in cells of a road `cells` long, each `cell_m`
/// metres; the new `along`, whether they have set off and whether they
/// moved.
pub fn lead(
    along: f64,
    started: bool,
    cells: usize,
    cell_m: f64,
    player_m: f64,
    player_mps: f64,
    dt: f64,
) -> (f64, bool, bool) {
    let started = started || player_m <= LEAD_START_M;
    let end = cells.saturating_sub(1) as f64;
    if !started || player_m > WAIT_M || !dt.is_finite() || dt <= 0.0 || along >= end {
        return (along.min(end), started, false);
    }
    let pace = LEAD_MPS.max(player_mps.min(6.0));
    let next = (along + pace * dt / cell_m.max(0.1)).min(end);
    (next, started, next > along)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell(key: u32, kind: RoadKind) -> RoadCell {
        RoadCell {
            key,
            base_m: 10,
            kind,
            raised: false,
            cut_fill: 0,
            bed: Default::default(),
            up: None,
            from: None,
        }
    }

    /// The rock falls past the town's own cells, on the ground only, in the
    /// first layer of air; a cell is clear when its rock is air again.
    #[test]
    fn a_rockfall_lands_on_the_ground_past_the_town_and_is_counted_clear() {
        let mut cells: Vec<RoadCell> = (0..20).map(|k| cell(k, RoadKind::Ground)).collect();
        cells[9].kind = RoadKind::Ford;
        let road = Road {
            from_site: 1,
            to_site: 2,
            surface: crate::roads::Surface::Gravel,
            cells,
        };
        let out = outward(&road, 2).unwrap();
        assert_eq!(out[0].key, 19);
        let fall = rockfall_cells(&outward(&road, 1).unwrap(), 3, |a| Some(a as usize));
        assert_eq!(fall, vec![(8, 10), (10, 10), (11, 10)]);
        assert!(outward(&road, 3).is_none());
        let rf = Rockfall {
            site: 1,
            cells: fall,
            cleared: 0,
        };
        assert_eq!(cleared(&rf, |c, _| c == 10), 1);
        assert_eq!(cleared(&rf, |_, _| true), 3);
    }

    /// The trader waits for the player to come, walks on while they keep
    /// up, waits when they fall behind, and stops at the road's end.
    #[test]
    fn a_trader_leads_while_the_player_keeps_up() {
        let (a, s, m) = lead(0.0, false, 10, 2.8, 20.0, 0.0, 0.1);
        assert_eq!((a, s, m), (0.0, false, false), "waits to be met");
        let (a, s, m) = lead(0.0, false, 10, 2.8, 5.0, 0.0, 1.0);
        assert!(s && m && (a - LEAD_MPS / 2.8).abs() < 1e-9);
        let (b, _, m) = lead(a, true, 10, 2.8, 15.0, 0.0, 1.0);
        assert!(!m && b == a, "waits for the player");
        let (c, _, _) = lead(8.9, true, 10, 2.8, 3.0, 4.0, 10.0);
        assert_eq!(c, 9.0, "stops at the end");
    }
}
