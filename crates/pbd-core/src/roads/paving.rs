//! Roads the player marks and paves (`townsfolk` design 39 to 41).
//!
//! **A plan** is a road the player means to pave: survey stakes driven as
//! waypoints, and the cells between each two found by the world's own road
//! search ([`super::route`]), at most a layer apart, off water, out of towns
//! and off the player's edits. A plan is a `road-plan` record of the
//! player's: its stakes and its cells, in order, stored as found so no
//! later search or road version moves it.
//!
//! **Marked** is derived: a plan's cell that is not paved. **Paving** a
//! marked cell makes it a `road-cell` of the player's in gravel, its slope
//! the world road's ([`super::profile`]) over the plan's order, so paving in
//! any order gives the same road.

use super::{
    BuiltCell, GroundFn, Kind, ROAD_CELL_RECORD, RoadCell, RoadsConfig, Surface, Water, centre,
    neighbours, profile, route,
};
use crate::records::{Record, Records};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A plan's record kind.
pub const PLAN_RECORD: &str = "road-plan";
/// The plan record's schema.
pub const PLAN_SCHEMA: u32 = 1;
/// A `road-cell` the player paved on a plan (design 40): the surface, the
/// plan, the two sides its slope runs between and whether it is raised.
/// Schema 1 (design 31's cells) still reads.
pub const PAVED_SCHEMA: u32 = 2;

/// One plan: its stakes and its cells, by key, in order along the road.
/// Every stake is one of its cells.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub stakes: Vec<u32>,
    pub cells: Vec<u32>,
}

impl Plan {
    /// The cells before and after `key` along the plan: the sides its mark
    /// comes in by and leaves by. `None` at an end.
    pub fn sides(&self, key: u32) -> Option<(Option<u32>, Option<u32>)> {
        let k = self.cells.iter().position(|&c| c == key)?;
        Some((
            k.checked_sub(1).map(|j| self.cells[j]),
            self.cells.get(k + 1).copied(),
        ))
    }
}

/// Every plan the records hold, by id.
pub fn plans(records: &Records) -> BTreeMap<u64, Plan> {
    records
        .of_kind(PLAN_RECORD)
        .filter(|r| r.schema == PLAN_SCHEMA)
        .filter_map(|r| Some((r.id, r.read::<Option<Plan>>()??)))
        .filter(|(_, p)| !p.cells.is_empty())
        .collect()
}

/// A plan as its record; `None` is a plan taken back whole.
pub fn plan_record(id: u64, plan: Option<&Plan>) -> Record {
    Record::of(PLAN_RECORD, id, PLAN_SCHEMA, &plan)
}

/// The next plan's id: one past every id the records have ever held, so an
/// id is never reused.
pub fn next_plan_id(records: &Records) -> u64 {
    records
        .of_kind(PLAN_RECORD)
        .map(|r| r.id + 1)
        .max()
        .unwrap_or(1)
}

/// Why a stake was not driven, or a cell not paved. Nothing is taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// No way within the stake's reach (`max_stake_cells`).
    TooFar,
    /// No way that climbs at most a layer a cell.
    TooSteep,
    /// Water between, or under the stake.
    Water,
    /// A town's footprint, or a cell the player has built on.
    NotGround,
    /// A stake stands there already.
    Staked,
    /// The cell is not marked.
    NotMarked,
}

impl Refused {
    /// The plain line the player reads.
    pub fn line(self) -> crate::locale::Localized {
        crate::locale::tr(match self {
            Refused::TooFar => "roads.mark.too_far",
            Refused::TooSteep => "roads.mark.too_steep",
            Refused::Water => "roads.mark.water",
            Refused::NotGround => "roads.mark.not_ground",
            Refused::Staked => "roads.mark.staked",
            Refused::NotMarked => "roads.pave.not_marked",
        })
    }
}

/// The stake search's own rules (design 39): no bridge (only the world
/// bridges), no cut or fill, its own small budget.
pub fn stake_config(cfg: &RoadsConfig) -> RoadsConfig {
    let mut c = cfg.clone();
    c.max_bridge_cells = 0;
    c.max_fill_layers = 0;
    c.max_cut_layers = 0;
    c.max_expanded = cfg.stake_expanded.max(1);
    c.search_weight = 1.0;
    c
}

/// The cells from stake `from` to stake `to`, both included, by the road
/// search over `ground` (which answers a town or an edited cell as the
/// player's, so it is never crossed): at most `max_stake_cells` steps, no
/// water.
pub fn segment(
    from: u32,
    to: u32,
    ground: &GroundFn,
    cfg: &RoadsConfig,
    radius_m: f32,
) -> Result<Vec<u32>, Refused> {
    let reach = cfg.max_stake_cells.max(1) as usize;
    let (Some(a), Some(b)) = (centre(from), centre(to)) else {
        return Err(Refused::NotGround);
    };
    // A cell's spacing is about 2.83 m; a way of `reach` steps cannot be
    // shorter than the straight line.
    if crate::sites::arc_m(a, b, radius_m) > reach as f32 * 2.9 {
        return Err(Refused::TooFar);
    }
    let dry = |key: u32| {
        ground(key).map(|g| {
            let mut g = g;
            if g.water != Water::Dry {
                // No ford for a player's road: the search does not take it.
                g.player = true;
            }
            g
        })
    };
    let wet = |key: u32| ground(key).is_some_and(|g| g.water != Water::Dry);
    if wet(to) || wet(from) {
        return Err(Refused::Water);
    }
    match route(from, to, &dry, &stake_config(cfg), radius_m) {
        Some(steps) if steps.len() - 1 <= reach => Ok(steps.into_iter().map(|s| s.key).collect()),
        Some(_) => Err(Refused::TooFar),
        None => {
            // Why not: water across the straight way, or the climb.
            let n = (crate::sites::arc_m(a, b, radius_m) / 1.0).ceil().max(1.0) as usize;
            let water = (0..=n).any(|i| {
                let t = i as f32 / n as f32;
                super::nearest_key(a.lerp(b, t)).is_some_and(wet)
            });
            Err(if water {
                Refused::Water
            } else {
                Refused::TooSteep
            })
        }
    }
}

/// Where a stake driven on `key` goes: the plan it continues (the open
/// plan, or the plan whose last stake `key` is), or a new one. A new plan
/// on a marked cell starts there: a junction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Driven {
    pub id: u64,
    pub plan: Plan,
    /// The cells it newly marks.
    pub marked: Vec<u32>,
}

/// Drive a stake on `key` (design 39). `open` is the plan the stakes in
/// hand are laying, if any. Pure: the caller writes the plan's record with
/// the pack in the same line.
pub fn drive(
    plans: &BTreeMap<u64, Plan>,
    next_id: u64,
    open: Option<u64>,
    key: u32,
    ground: &GroundFn,
    cfg: &RoadsConfig,
    radius_m: f32,
) -> Result<Driven, Refused> {
    let g = ground(key).ok_or(Refused::NotGround)?;
    if g.water != Water::Dry {
        return Err(Refused::Water);
    }
    if g.player || g.town {
        return Err(Refused::NotGround);
    }
    if plans.values().any(|p| p.stakes.contains(&key)) {
        return Err(Refused::Staked);
    }
    let continuing = open.and_then(|id| Some((id, plans.get(&id)?)));
    if let Some((id, plan)) = continuing
        && let Some(&last) = plan.stakes.last()
    {
        let cells = segment(last, key, ground, cfg, radius_m)?;
        let mut plan = plan.clone();
        let marked: Vec<u32> = cells[1..].to_vec();
        plan.cells.extend(&marked);
        plan.stakes.push(key);
        return Ok(Driven { id, plan, marked });
    }
    Ok(Driven {
        id: next_id,
        plan: Plan {
            stakes: vec![key],
            cells: vec![key],
        },
        marked: vec![key],
    })
}

/// A plan after a stake is taken back from it (design 39): `None` for a
/// plan with no stake left. A middle stake's neighbours are joined by a
/// fresh search where there is a way; where there is none the plan splits,
/// and the second part comes back to be stored under a new id.
pub fn take_back(
    plan: &Plan,
    key: u32,
    ground: &GroundFn,
    cfg: &RoadsConfig,
    radius_m: f32,
) -> (Option<Plan>, Option<Plan>) {
    let Some(s) = plan.stakes.iter().position(|&k| k == key) else {
        return (Some(plan.clone()), None);
    };
    let at = |stake: u32| plan.cells.iter().position(|&c| c == stake);
    let n = plan.stakes.len();
    if n == 1 {
        return (None, None);
    }
    if s == 0 || s == n - 1 {
        // An end: the segment to its neighbour stake goes.
        let mut p = plan.clone();
        if s == 0 {
            let cut = at(plan.stakes[1]).unwrap_or(0);
            p.cells.drain(..cut);
            p.stakes.remove(0);
        } else {
            let cut = at(plan.stakes[n - 2]).unwrap_or(p.cells.len() - 1);
            p.cells.truncate(cut + 1);
            p.stakes.pop();
        }
        return (Some(p), None);
    }
    let (prev, next) = (plan.stakes[s - 1], plan.stakes[s + 1]);
    let (Some(i), Some(j)) = (at(prev), at(next)) else {
        return (Some(plan.clone()), None);
    };
    let mut stakes = plan.stakes.clone();
    stakes.remove(s);
    if let Ok(joined) = segment(prev, next, ground, cfg, radius_m) {
        let mut cells = plan.cells[..i].to_vec();
        cells.extend(joined);
        cells.extend(&plan.cells[j + 1..]);
        return (Some(Plan { stakes, cells }), None);
    }
    let first = Plan {
        stakes: plan.stakes[..s].to_vec(),
        cells: plan.cells[..=i].to_vec(),
    };
    let second = Plan {
        stakes: plan.stakes[s + 1..].to_vec(),
        cells: plan.cells[j..].to_vec(),
    };
    (Some(first), Some(second))
}

/// Every marked cell (a plan's cell not yet paved) with the cells before
/// and after it along its plan, for the mark's look (design 39a). A cell on
/// two plans takes the first's.
pub fn marked(
    plans: &BTreeMap<u64, Plan>,
    paved: &BTreeMap<u32, BuiltCell>,
) -> BTreeMap<u32, (Option<u32>, Option<u32>)> {
    let mut out = BTreeMap::new();
    for plan in plans.values() {
        for (k, &cell) in plan.cells.iter().enumerate() {
            if paved.contains_key(&cell) || out.contains_key(&cell) {
                continue;
            }
            let before = k.checked_sub(1).map(|j| plan.cells[j]);
            let after = plan.cells.get(k + 1).copied();
            out.insert(cell, (before, after));
        }
    }
    out
}

/// The plan a marked cell is on, if any (the first, by id).
pub fn plan_of(plans: &BTreeMap<u64, Plan>, key: u32) -> Option<(u64, &Plan)> {
    plans
        .iter()
        .find(|(_, p)| p.cells.contains(&key))
        .map(|(id, p)| (*id, p))
}

/// The paved cell `key` of plan `plan_id` (design 40): its slope the world
/// road's, cut over the plan's whole order from `ground`, so paving the
/// cells in any order gives the same road.
pub fn paved_cell(
    plan_id: u64,
    plan: &Plan,
    key: u32,
    ground: &GroundFn,
) -> Result<BuiltCell, Refused> {
    let steps: Vec<super::Step> = plan
        .cells
        .iter()
        .map(|&k| {
            Some(super::Step {
                key: k,
                ground: {
                    let mut g = ground(k)?;
                    // Every cell of a plan is laid: a town's footprint never
                    // is one, and the profile must not drop any.
                    g.town = false;
                    g
                },
                deck: None,
                lift: 0,
            })
        })
        .collect::<Option<_>>()
        .ok_or(Refused::NotMarked)?;
    let cells: Vec<RoadCell> = profile(&steps).ok_or(Refused::TooSteep)?;
    let cell = cells
        .into_iter()
        .find(|c| c.key == key)
        .ok_or(Refused::NotMarked)?;
    Ok(BuiltCell {
        surface: Surface::Gravel,
        base_m: cell.base_m,
        up: cell.up,
        from: cell.from,
        raised: cell.raised,
        plan: Some(plan_id),
    })
}

/// A paved cell as its record (schema [`PAVED_SCHEMA`]).
pub fn paved_record(key: u32, cell: &BuiltCell) -> Record {
    Record::of(ROAD_CELL_RECORD, u64::from(key), PAVED_SCHEMA, cell)
}

/// Whether two cells are neighbours (share a side).
pub fn touching(a: u32, b: u32) -> bool {
    neighbours(a).contains(&b)
}

/// A built cell as the road it draws: a ground cell, raised where it is.
pub fn drawn(key: u32, cell: &BuiltCell) -> RoadCell {
    RoadCell {
        key,
        base_m: cell.base_m,
        kind: Kind::Ground,
        raised: cell.raised,
        cut_fill: 0,
        up: cell.up,
        from: cell.from,
    }
}

// ---- What the terrain draws (design 39a and 43) -------------------------------

/// The marks and the player's road cells as the terrain draws them: each
/// marked cell's sides (the cells before and after it on its plan), and
/// where the clutter grows no tufts (a marked or paved cell). Installed
/// whenever they change, read by the terrain's build threads.
#[derive(Clone, Debug, Default)]
pub struct Installed {
    pub marks: std::collections::HashMap<u32, (Option<u32>, Option<u32>)>,
    pub bare: super::Cleared,
}

static INSTALLED: std::sync::RwLock<Option<std::sync::Arc<Installed>>> =
    std::sync::RwLock::new(None);
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

thread_local! {
    static LOCAL: std::cell::RefCell<(u64, Option<std::sync::Arc<Installed>>)> =
        const { std::cell::RefCell::new((u64::MAX, None)) };
}

/// Install the open world's marks and bare cells, or none.
pub fn install(installed: Option<Installed>) {
    *INSTALLED.write().unwrap_or_else(|e| e.into_inner()) = installed.map(std::sync::Arc::new);
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
}

fn with<T>(f: impl FnOnce(Option<&Installed>) -> T) -> T {
    let generation = GENERATION.load(std::sync::atomic::Ordering::Acquire);
    LOCAL.with(|local| {
        let mut local = local.borrow_mut();
        if local.0 != generation {
            *local = (
                generation,
                INSTALLED.read().unwrap_or_else(|e| e.into_inner()).clone(),
            );
        }
        f(local.1.as_deref())
    })
}

/// A marked cell's sides, by key: `None` where the cell is not marked.
pub fn mark_of(key: u32) -> Option<(Option<u32>, Option<u32>)> {
    with(|i| i.and_then(|i| i.marks.get(&key).copied()))
}

/// Whether a direction lies on a cell the player marked or paved: no tufts
/// grow there (design 43).
pub fn bare(direction: glam::Vec3) -> bool {
    with(|i| i.is_some_and(|i| i.bare.on_road(direction)))
}

#[cfg(test)]
mod tests;
