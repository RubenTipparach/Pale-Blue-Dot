//! The lost villager (`townsfolk` design 9): a townsperson standing out of
//! town, who, once talked to, walks behind the player at the player's pace,
//! stops when the player stops, never swims (waits at deep water), and is
//! home once inside their town's wall.
//!
//! The record `lost_villager` (one per job, by the job's id) holds where they
//! stand, body-relative in `f64` (CLAUDE.md, the floating origin), and
//! whether they follow. The step is a pure function of positions in one
//! frame, so the game and a future server walk them the same way.

use glam::DVec3;
use serde::{Deserialize, Serialize};

pub const LOST_KIND: &str = "lost_villager";
pub const LOST_SCHEMA: u32 = 1;

/// How far out of town a lost villager stands, metres, at most.
pub const LOST_REACH_M: f64 = 300.0;
/// How far behind the player they keep, metres.
pub const GAP_M: f64 = 2.0;
/// Past this they hurry to catch up, metres.
pub const HURRY_M: f64 = 8.0;
/// Their walking pace when hurrying, metres a second.
pub const HURRY_MPS: f64 = 4.5;
/// How near they must be to talk to, metres.
pub const TALK_M: f64 = 3.0;

/// A lost villager, as stored.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lost {
    /// The town they belong to.
    pub site: u32,
    /// Where they stand, the body's frame, metres from its centre.
    pub at: [f64; 3],
    /// Whether they have been found and follow.
    pub following: bool,
    /// Who they are: their place in their town's people.
    #[serde(default)]
    pub person: u32,
}

impl Lost {
    pub fn at(&self) -> DVec3 {
        DVec3::from_array(self.at)
    }
}

/// One step: where the villager stands after `dt`, following `player`, who
/// moved at `player_mps`. `dry(p)` says whether a point is ground they can
/// stand on (not deep water); a step onto water is not taken, so they wait
/// at the edge.
pub fn step(
    villager: DVec3,
    player: DVec3,
    player_mps: f64,
    dt: f64,
    dry: impl Fn(DVec3) -> bool,
) -> DVec3 {
    let to = player - villager;
    let distance = to.length();
    if distance <= GAP_M || !dt.is_finite() || dt <= 0.0 {
        return villager;
    }
    // At the player's pace; quicker when left behind, so stopping to wait
    // for them works.
    let pace = if distance > HURRY_M {
        HURRY_MPS.max(player_mps)
    } else {
        player_mps.max(0.0)
    };
    let stride = (pace * dt).min(distance - GAP_M);
    if stride <= 0.0 {
        return villager;
    }
    let next = villager + to / distance * stride;
    if dry(next) { next } else { villager }
}

/// How near the town a lost villager may stand, metres along the ground.
pub const LOST_NEAREST_M: f64 = 120.0;

/// Where a job's lost villager stands (`townsfolk` design 28): along a
/// bearing rolled from the job's id, out from the town's anchor `anchor` (a
/// unit direction), from a reach rolled between [`LOST_NEAREST_M`] and
/// [`LOST_REACH_M`] outward, then back in from it, a few metres at a time,
/// at the first direction `open` takes. The same job gives the same place.
pub fn spot(
    job: u64,
    anchor: glam::Vec3,
    radius_m: f32,
    open: impl Fn(glam::Vec3) -> bool,
) -> Option<glam::Vec3> {
    let roll = super::jobs::mix(job ^ 0x6c6f_7374);
    let bearing = ((roll % 3600) as f32 / 10.0).to_radians();
    let first = LOST_NEAREST_M as i32 + (super::jobs::mix(roll) % 181) as i32;
    let (north, east) = crate::geo::north_east(anchor);
    let way = east * bearing.cos() + north * bearing.sin();
    let out = (first..=LOST_REACH_M as i32).step_by(3);
    let back = (LOST_NEAREST_M as i32..first).rev().step_by(3);
    out.chain(back)
        .map(|m| (anchor + way * (m as f32 / radius_m)).normalize())
        .find(|&d| open(d))
}

/// Who is lost (design 28): one of the town's people, rolled from the job's
/// id over its `people`, never its trader or job
/// giver (`keepers`). `None` for a town of no one else.
pub fn who(job: u64, people: usize, keepers: &[usize]) -> Option<u32> {
    let others: Vec<usize> = (0..people).filter(|i| !keepers.contains(i)).collect();
    if others.is_empty() {
        return None;
    }
    let pick = others[(super::jobs::mix(job ^ 0x0077_686f) % others.len() as u64) as usize];
    Some(pick as u32)
}

/// Whether a villager at `at` is home: within `wall_m` of their town's
/// anchor `anchor`, measured along the ground's plane at the anchor.
pub fn home(at: DVec3, anchor: DVec3, wall_m: f64) -> bool {
    let up = anchor.normalize_or_zero();
    let off = at - anchor;
    (off - up * off.dot(up)).length() <= wall_m
}
