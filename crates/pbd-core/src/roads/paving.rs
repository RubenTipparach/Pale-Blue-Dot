//! Roads the player marks and paves (`townsfolk` design 39 to 42l): the
//! records. The road planner ([`super::planner`]) lays a plan's roads out;
//! here are the record kinds, the schema 1 plan an old save holds (design
//! 39b, read as a schema 2 plan by [`super::planner::plans`]), the paved
//! cell's record, and what the terrain draws.
//!
//! **Marked** is derived: a plan's cell that is not paved. **Paving** a
//! marked cell makes it a `road-cell` of the player's, its slope the world
//! road's over the road's order, so paving in any order gives the same road.

use super::{BuiltCell, ROAD_CELL_RECORD};
use crate::records::{Record, Records};
use serde::{Deserialize, Serialize};

/// A plan's record kind.
pub const PLAN_RECORD: &str = "road-plan";
/// The schema 1 plan (design 39b): stakes and searched cells. Still read;
/// written as schema 2 ([`super::planner::ROADS_SCHEMA`]) when edited.
pub const PLAN_SCHEMA: u32 = 1;
/// A `road-cell` the player paved on a plan (design 40): the surface, the
/// plan, the two sides its slope runs between and whether it is raised.
/// Schema 1 (design 31's cells) still reads.
pub const PAVED_SCHEMA: u32 = 2;

/// A schema 1 plan: its stakes and its cells, by key, in order along the
/// road. Every stake is one of its cells.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    pub stakes: Vec<u32>,
    pub cells: Vec<u32>,
}

/// A schema 1 plan as its record, as design 39b wrote it (for tests of
/// old saves).
pub fn plan_v1_record(id: u64, plan: Option<&Plan>) -> Record {
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

/// A paved cell as its record (schema [`PAVED_SCHEMA`]).
pub fn paved_record(key: u32, cell: &BuiltCell) -> Record {
    Record::of(ROAD_CELL_RECORD, u64::from(key), PAVED_SCHEMA, cell)
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
/// grow there (design 44).
pub fn bare(direction: glam::Vec3) -> bool {
    with(|i| i.is_some_and(|i| i.bare.on_road(direction)))
}
