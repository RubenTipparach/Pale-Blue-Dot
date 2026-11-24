//! The road planner (`townsfolk` design 42 to 42l): roads the player stakes
//! out by their two ends, each the straight hex line between its stakes,
//! checked cell by cell by the rule the world's roads use, with only the
//! cells that clash refused.
//!
//! **A plan** is a `road-plan` record of the player's holding a list of
//! roads (schema 2), each its stakes in order and its width, surface and
//! lamps. Its cells are re-derived from the stakes ([`hex_line`], a pure
//! function of the keys), so a plan reads back cell for cell whatever later
//! road versions do. A schema 1 plan (design 39b) reads as one road whose
//! cells are the ones it stored.
//!
//! **Paved** cells stay `road-cell` records naming their plan. A paved cell
//! no road runs over any more is **marked for removal**: derived, never
//! stored (design 42l).

use super::{BuiltCell, GroundFn, Kind, RoadCell, Surface, Water, centre, neighbours, paving};
use crate::locale::{Localized, tr};
use crate::records::{Record, Records};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The plan record's schema with roads (design 42h).
pub const ROADS_SCHEMA: u32 = 2;

/// The most a road climbs or drops between two neighbouring cells, layers.
/// One rule for the world's roads and the player's ([`check_step`]).
pub const MAX_STEP_LAYERS: i32 = 1;

// ---- The one rule for a cell and a step (design 42a, 42b) --------------------

/// Why a cell refuses a road. Each reds only the cell it is about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Clash {
    /// More than the grade from the cell before: the higher of the two.
    TooSteep,
    /// Water deeper than a ford (design 42l: no player's bridge yet).
    TooWideToBridge,
    /// Still water: a lake or the sea.
    OpenWater,
    /// A town's footprint: its lanes are road already.
    InTown,
    /// A tree stands on the cell.
    Tree,
    /// The player has built on the cell (not a road cell).
    InTheWay,
    /// Past the longest segment.
    TooFar,
    /// A wide road bending where the ground is not level.
    WideBend,
    /// A lane off its road's plane.
    TooSteepAcross,
}

impl Clash {
    /// The plain line the player reads (`roads.plan.*`).
    pub fn line(self) -> Localized {
        match self {
            Clash::TooSteep => tr("roads.plan.too_steep"),
            Clash::TooWideToBridge => tr("roads.plan.too_wide_to_bridge"),
            Clash::OpenWater => tr("roads.plan.open_water"),
            Clash::InTown => tr("roads.plan.in_town"),
            Clash::Tree => tr("roads.plan.tree"),
            Clash::InTheWay => tr("roads.plan.in_the_way"),
            Clash::TooFar => tr("roads.plan.too_far"),
            Clash::WideBend => tr("roads.plan.wide_bend"),
            Clash::TooSteepAcross => tr("roads.plan.too_steep_across"),
        }
    }

    /// Every reason.
    pub const ALL: [Clash; 9] = [
        Clash::TooSteep,
        Clash::TooWideToBridge,
        Clash::OpenWater,
        Clash::InTown,
        Clash::Tree,
        Clash::InTheWay,
        Clash::TooFar,
        Clash::WideBend,
        Clash::TooSteepAcross,
    ];
}

/// Whether a road may be laid on a cell of ground `g` (design 42b), a tree
/// standing on it or not: the player's work, a town, the sea, water deeper
/// than `ford_depth_m` and a tree each refuse it. The world's search asks
/// the same and enters neither the player's work nor the sea; it crosses a
/// town and bridges deep water, which the player's road does not yet.
pub fn check_cell(g: &super::Ground, tree: bool, ford_depth_m: f32) -> Option<Clash> {
    if g.player {
        return Some(Clash::InTheWay);
    }
    match g.water {
        Water::Sea => return Some(Clash::OpenWater),
        Water::River { depth_m, .. } if depth_m as f32 > ford_depth_m => {
            return Some(Clash::TooWideToBridge);
        }
        _ => {}
    }
    if g.town {
        return Some(Clash::InTown);
    }
    tree.then_some(Clash::Tree)
}

/// Whether a road may go from a cell at `from_m` to its neighbour at
/// `to_m`: never more than [`MAX_STEP_LAYERS`] between them.
pub fn check_step(from_m: i32, to_m: i32) -> Option<Clash> {
    ((from_m - to_m).abs() > MAX_STEP_LAYERS).then_some(Clash::TooSteep)
}

// ---- The line between two stakes (design 42a, 42l) -------------------------------

/// The cell nearest `d`, walked to from `key` through ever nearer
/// neighbours.
fn nearest_from(mut key: u32, d: Vec3) -> u32 {
    let far = |k: u32| centre(k).map(|c| (c.as_dvec3() - d.as_dvec3()).length_squared());
    let Some(mut dist) = far(key) else {
        return key;
    };
    loop {
        let next = neighbours(key)
            .into_iter()
            .filter_map(|n| Some((n, far(n)?)))
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        match next {
            Some((n, nd)) if nd < dist => {
                key = n;
                dist = nd;
            }
            _ => return key,
        }
    }
}

/// The line from `a` to `b` in the lattice of a face both lie on: the cube
/// lerp of their lattice coordinates, rounded, every cell a neighbour of the
/// one before, nudged one way so a tie always falls the same side. `None`
/// where no face holds both.
fn lattice_line(a: u32, b: u32) -> Option<Vec<u32>> {
    use crate::cell_key::{Address, addresses, key, unpack};
    let (pa, pb) = (unpack(a)?, unpack(b)?);
    let (fa, fb) = addresses(pa).into_iter().find_map(|x| {
        addresses(pb)
            .into_iter()
            .find(|y| y.face == x.face)
            .map(|y| (x, y))
    })?;
    let (i1, j1) = (i64::from(fa.i), i64::from(fa.j));
    let (di, dj) = (i64::from(fb.i) - i1, i64::from(fb.j) - j1);
    let n = di.abs().max(dj.abs()).max((di + dj).abs());
    let mut out = Vec::with_capacity(n as usize + 1);
    for k in 0..=n {
        let t = k as f64 / n.max(1) as f64;
        let x = i1 as f64 + di as f64 * t + 1e-6;
        let z = j1 as f64 + dj as f64 * t + 2e-6;
        let y = -x - z;
        let (mut rx, ry, mut rz) = (x.round(), y.round(), z.round());
        let (ex, ey, ez) = ((rx - x).abs(), (ry - y).abs(), (rz - z).abs());
        if ex > ey && ex > ez {
            rx = -ry - rz;
        } else if ey <= ez {
            rz = -rx - ry;
        }
        let (i, j) = (rx as i64, rz as i64);
        if i < 0 || j < 0 {
            return None;
        }
        let cell = key(Address {
            face: fa.face,
            level: fa.level,
            i: i as u32,
            j: j as u32,
        })?;
        if out.last() != Some(&cell) {
            out.push(cell);
        }
    }
    Some(out)
}

/// The angle between a cell's centre and its neighbours', radians.
pub fn cell_angle(key: u32) -> f32 {
    let (Some(c), Some(n)) = (
        centre(key),
        neighbours(key).first().and_then(|&n| centre(n)),
    ) else {
        return 1e-3;
    };
    c.angle_between(n).max(1e-6)
}

/// The cells on the straight line from `from` to `to`, both included, each
/// a neighbour of the one before, the same cells both ways round (design
/// 42l): the cube lerp in the lattice of a face both lie on, from the lower
/// key to the higher; across faces, the great circle sampled every tenth of
/// a cell, the nearest cell at each, and a corner crossed by the two cells'
/// common neighbour of lower key.
pub fn hex_line(from: u32, to: u32) -> Vec<u32> {
    if from == to {
        return vec![from];
    }
    let (a, b) = (from.min(to), from.max(to));
    if let Some(mut line) = lattice_line(a, b) {
        if from > to {
            line.reverse();
        }
        return line;
    }
    let (Some(ca), Some(cb)) = (centre(a), centre(b)) else {
        return Vec::new();
    };
    // A tenth of a cell (a quarter metre on the gold standard's 2.833 m).
    let cells = ca.angle_between(cb) / cell_angle(a);
    let n = (cells * 10.0).ceil().max(1.0) as usize;
    let mut line = vec![a];
    let mut cur = a;
    for i in 1..=n {
        let t = i as f32 / n as f32;
        let p = (ca * (1.0 - t) + cb * t).normalize();
        cur = nearest_from(cur, p);
        let last = *line.last().expect("the line has its start");
        if cur == last {
            continue;
        }
        if !neighbours(last).contains(&cur) {
            let theirs = neighbours(cur);
            if let Some(&k) = neighbours(last).iter().filter(|k| theirs.contains(k)).min() {
                line.push(k);
            }
        }
        line.push(cur);
    }
    if *line.last().expect("a start") != b {
        line.push(b);
    }
    if from > to {
        line.reverse();
    }
    line
}

// ---- The planner's tunables (design 42i) -----------------------------------------

/// What an edit does with paved cells it drops (design 42f).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Removal {
    /// Marked for removal, taken up with the shovel.
    Mark,
    /// Taken up at once, their gravel back.
    Return,
}

/// `assets/config/road_planner.ron`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoadPlannerConfig {
    /// The plan's ghost, 0..1.
    pub ghost_opacity: f32,
    /// The ghost's tint, linear RGB.
    pub ghost_tint: [f32; 3],
    /// A clashing cell's tint, linear RGB.
    pub refuse_tint: [f32; 3],
    /// The steepest road: cells a layer (1 is a layer every cell).
    pub max_grade: u32,
    /// The most cells between two stakes; a longer road is staked on the
    /// way.
    pub max_segment: u32,
    /// How far a stake is driven, m.
    pub stake_reach_m: f32,
    /// How far the shovel paves, m.
    pub reach_m: f32,
    /// How near an open plan its ghost and bill show, m.
    pub ghost_reach_m: f32,
    /// Every cost, times.
    pub cost_scale: f32,
    /// Cells between lamp posts along the centre line.
    pub lamp_spacing: u32,
    pub removal: Removal,
    /// The share of a cell's gravel given back when it is taken up, 0..1.
    pub refund_share: f32,
}

impl Default for RoadPlannerConfig {
    /// The values `road_planner.ron` ships, for tests.
    fn default() -> Self {
        Self {
            ghost_opacity: 0.35,
            ghost_tint: [0.35, 0.75, 0.95],
            refuse_tint: [0.75, 0.16, 0.12],
            max_grade: 1,
            max_segment: 24,
            stake_reach_m: 6.0,
            reach_m: 4.0,
            ghost_reach_m: 60.0,
            cost_scale: 1.0,
            lamp_spacing: 6,
            removal: Removal::Mark,
            refund_share: 1.0,
        }
    }
}

impl RoadPlannerConfig {
    /// Every tunable in its range, or which is not.
    pub fn validate(&self) -> Result<(), String> {
        let unit = |name: &str, v: f32| {
            (v.is_finite() && (0.0..=1.0).contains(&v))
                .then_some(())
                .ok_or(format!("{name} must be from 0 to 1"))
        };
        unit("ghost_opacity", self.ghost_opacity)?;
        unit("refund_share", self.refund_share)?;
        for (name, c) in [
            ("ghost_tint", self.ghost_tint),
            ("refuse_tint", self.refuse_tint),
        ] {
            if !c.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)) {
                return Err(format!("{name} must be linear RGB from 0 to 1"));
            }
        }
        if !(1..=3).contains(&self.max_grade) {
            return Err("max_grade must be 1, 2 or 3 cells a layer".into());
        }
        if !(2..=64).contains(&self.max_segment) {
            return Err("max_segment must be from 2 to 64 cells".into());
        }
        for (name, v, hi) in [
            ("stake_reach_m", self.stake_reach_m, 64.0),
            ("reach_m", self.reach_m, 16.0),
            ("ghost_reach_m", self.ghost_reach_m, 500.0),
            ("cost_scale", self.cost_scale, 10.0),
        ] {
            if !(v.is_finite() && v > 0.0 && v <= hi) {
                return Err(format!("{name} must be over 0 and at most {hi}"));
            }
        }
        if !(2..=32).contains(&self.lamp_spacing) {
            return Err("lamp_spacing must be from 2 to 32 cells".into());
        }
        Ok(())
    }
}

// ---- Plans as records (design 42h) -----------------------------------------------

/// One road of a plan: its stakes in order and how it is built.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanRoad {
    pub stakes: Vec<u32>,
    /// 1, 2 or 3 cells.
    pub width: u8,
    pub surface: Surface,
    pub lamps: bool,
    /// A schema 1 plan's stored cells, which its route keeps (design 42h);
    /// `None` for a road whose cells are the hex lines between its stakes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cells: Option<Vec<u32>>,
}

impl PlanRoad {
    /// A new road of `stakes` built as `how`.
    pub fn new(stakes: Vec<u32>, how: How) -> Self {
        Self {
            stakes,
            width: how.width.clamp(1, 3),
            surface: how.surface,
            lamps: how.lamps,
            cells: None,
        }
    }

    /// Its centre line: the hex lines between its stakes, joined.
    pub fn centre(&self) -> Vec<u32> {
        if let Some(cells) = &self.cells {
            return cells.clone();
        }
        let mut out: Vec<u32> = Vec::new();
        if let [only] = self.stakes[..] {
            return vec![only];
        }
        for w in self.stakes.windows(2) {
            let line = hex_line(w[0], w[1]);
            let skip = usize::from(!out.is_empty());
            out.extend(line.into_iter().skip(skip));
        }
        out
    }
}

/// How the next road is built: the planner menu's choices (design 42c).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct How {
    pub width: u8,
    pub surface: Surface,
    pub lamps: bool,
}

impl Default for How {
    fn default() -> Self {
        Self {
            width: 1,
            surface: Surface::Gravel,
            lamps: false,
        }
    }
}

/// A plan: its roads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoadPlan {
    pub roads: Vec<PlanRoad>,
}

/// Every plan the records hold, by id: schema 2 as stored, schema 1 read
/// as one road of width 1 in gravel whose cells are the ones it stored.
pub fn plans(records: &Records) -> BTreeMap<u64, RoadPlan> {
    records
        .of_kind(paving::PLAN_RECORD)
        .filter_map(|r| {
            let plan = match r.schema {
                paving::PLAN_SCHEMA => {
                    let old = r.read::<Option<paving::Plan>>()??;
                    migrate(&old)
                }
                ROADS_SCHEMA => r.read::<Option<RoadPlan>>()??,
                _ => return None,
            };
            (!plan.roads.is_empty()).then_some((r.id, plan))
        })
        .collect()
}

/// A schema 1 plan as schema 2 (design 42h).
pub fn migrate(old: &paving::Plan) -> RoadPlan {
    if old.cells.is_empty() {
        return RoadPlan::default();
    }
    RoadPlan {
        roads: vec![PlanRoad {
            stakes: old.stakes.clone(),
            width: 1,
            surface: Surface::Gravel,
            lamps: false,
            cells: Some(old.cells.clone()),
        }],
    }
}

/// A plan as its record (schema 2); `None` (or no road) is a plan taken
/// back whole.
pub fn plan_record(id: u64, plan: Option<&RoadPlan>) -> Record {
    let plan = plan.filter(|p| !p.roads.is_empty());
    Record::of(paving::PLAN_RECORD, id, ROADS_SCHEMA, &plan)
}

/// Every stake standing, each key once, in key order.
pub fn stakes(plans: &BTreeMap<u64, RoadPlan>) -> BTreeSet<u32> {
    plans
        .values()
        .flat_map(|p| &p.roads)
        .flat_map(|r| r.stakes.iter().copied())
        .collect()
}

/// The stakes of `start` to `end` (design 42k): the end, and one wherever
/// the line would pass `max_segment` cells since the last.
pub fn stake_line(start: u32, end: u32, max_segment: u32) -> Vec<u32> {
    let line = hex_line(start, end);
    let step = max_segment.max(1) as usize;
    let mut out = vec![start];
    let mut k = step;
    while k < line.len() - 1 {
        out.push(line[k]);
        k += step;
    }
    if end != start {
        out.push(end);
    }
    out
}

// ---- A road laid out: its cells, lanes, heights and clashes ---------------------

/// One cell of a laid-out road.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaidCell {
    pub key: u32,
    /// The cell of the centre line it belongs to (itself on the line).
    pub centre: u32,
    /// Across the line: 0 on it, 1 to the left, -1 to the right.
    pub lane: i8,
    /// The cells it comes in by and leaves by along the road.
    pub before: Option<u32>,
    pub after: Option<u32>,
    /// Its road top's low side, m (the ground for a ghost of a refused
    /// cell), and the neighbour it slopes up toward and from.
    pub low_m: i32,
    pub base_m: i32,
    pub raised: bool,
    pub ford: bool,
    pub up: Option<u32>,
    pub from: Option<u32>,
    pub clash: Option<Clash>,
}

impl LaidCell {
    /// As a road cell, to draw.
    pub fn drawn(&self) -> RoadCell {
        RoadCell {
            key: self.key,
            base_m: self.base_m,
            kind: if self.ford { Kind::Ford } else { Kind::Ground },
            raised: self.raised,
            cut_fill: 0,
            up: self.up,
            from: self.from,
        }
    }

    /// As the player's paved cell of plan `plan`, in `surface`.
    pub fn built(&self, plan: u64, surface: Surface) -> BuiltCell {
        BuiltCell {
            surface: if self.ford { Surface::Gravel } else { surface },
            base_m: self.base_m,
            up: self.up,
            from: self.from,
            raised: self.raised,
            plan: Some(plan),
            ford: self.ford,
        }
    }
}

/// What the planner asks of the world besides the ground: whether a tree
/// stands on a cell.
pub type TreeFn<'a> = dyn Fn(u32) -> bool + 'a;

/// The neighbour of `key` most along `dir` (a tangent), if any is ahead.
fn toward(key: u32, dir: Vec3) -> Option<u32> {
    let c = centre(key)?;
    neighbours(key)
        .into_iter()
        .filter_map(|n| Some((n, (centre(n)? - c).normalize_or_zero().dot(dir))))
        .filter(|(_, d)| *d > 0.5)
        .max_by(|a, b| a.1.total_cmp(&b.1).then(b.0.cmp(&a.0)))
        .map(|(n, _)| n)
}

/// The centre line's heights after the profile's raise of one-cell dips
/// (design 10), and which were raised.
fn heights(tops: &[i32]) -> (Vec<i32>, Vec<bool>) {
    let n = tops.len();
    let mut h = tops.to_vec();
    let mut raised = vec![false; n];
    for _ in 0..4 {
        let mut changed = false;
        for k in 1..n.saturating_sub(1) {
            if !raised[k] && h[k] < h[k - 1] && h[k] < h[k + 1] {
                h[k] += 1;
                raised[k] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    (h, raised)
}

/// Lay out a road over `ground` (design 42a to 42c, 42l): its centre line
/// and lanes, each cell's top cut by the road rule, and each cell that
/// clashes with why. Pure.
pub fn lay(
    road: &PlanRoad,
    ground: &GroundFn,
    tree: &TreeFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Vec<LaidCell> {
    let line = road.centre();
    let n = line.len();
    let grounds: Vec<Option<super::Ground>> = line.iter().map(|&k| ground(k)).collect();
    // A ford's road is on its bed, the ground's top under the water.
    let tops: Vec<i32> = grounds.iter().map(|g| g.map_or(0, |g| g.top_m)).collect();
    let (h, raised) = heights(&tops);
    let mut clash: Vec<Option<Clash>> = grounds
        .iter()
        .zip(&line)
        .map(|(g, &k)| match g {
            None => Some(Clash::InTheWay),
            Some(g) => check_cell(g, tree(k), ford_depth_m),
        })
        .collect();
    // Steps and the grade: the higher of two cells too far apart is red.
    let mut last_rise: Option<usize> = None;
    for k in 1..n {
        if let Some(c) = check_step(h[k - 1], h[k]) {
            let higher = if h[k] > h[k - 1] { k } else { k - 1 };
            clash[higher].get_or_insert(c);
        } else if h[k] != h[k - 1] {
            if let Some(r) = last_rise
                && k - r < cfg.max_grade as usize
            {
                let higher = if h[k] > h[k - 1] { k } else { k - 1 };
                clash[higher].get_or_insert(Clash::TooSteep);
            }
            last_rise = Some(k);
        }
    }
    // Past the longest segment between two stakes (a road staked by the
    // planner never is).
    let mut since = 0usize;
    for (k, &key) in line.iter().enumerate() {
        if road.stakes.contains(&key) {
            since = 0;
        } else {
            since += 1;
            if since > cfg.max_segment as usize {
                clash[k].get_or_insert(Clash::TooFar);
            }
        }
    }
    let dirs: Vec<Vec3> = (0..n)
        .map(|k| {
            let a = centre(line[k.saturating_sub(1)]).unwrap_or(Vec3::Y);
            let b = centre(line[(k + 1).min(n - 1)]).unwrap_or(Vec3::Y);
            let up = centre(line[k]).unwrap_or(Vec3::Y);
            let d = b - a;
            (d - up * d.dot(up)).normalize_or_zero()
        })
        .collect();
    // A wide road bends only on the level.
    if road.width > 1 {
        for k in 1..n.saturating_sub(1) {
            let turn = dirs[k - 1].dot(dirs[k + 1]) < 0.94;
            let level = h[k - 1] == h[k] && h[k] == h[k + 1];
            if turn && road.stakes.contains(&line[k]) && !level {
                clash[k].get_or_insert(Clash::WideBend);
            }
        }
    }
    let mut out = Vec::new();
    let mut ups: Vec<(Option<u32>, Option<u32>)> = Vec::with_capacity(n);
    for k in 0..n {
        let (mut up, mut from) = (None, None);
        for (j, o) in [(k + 1, k.wrapping_sub(1)), (k.wrapping_sub(1), k + 1)] {
            if j < n && h[j] == h[k] + 1 {
                up = Some(line[j]);
                from = (o < n).then(|| line[o]);
                break;
            }
        }
        ups.push((up, from));
        out.push(LaidCell {
            key: line[k],
            centre: line[k],
            lane: 0,
            before: k.checked_sub(1).map(|j| line[j]),
            after: line.get(k + 1).copied(),
            low_m: h[k],
            base_m: tops[k],
            raised: raised[k],
            ford: grounds[k].is_some_and(|g| matches!(g.water, Water::River { .. })),
            up,
            from,
            clash: clash[k],
        });
    }
    if road.width > 1 {
        out.extend(lanes(
            road.width,
            &line,
            &dirs,
            &out,
            ground,
            tree,
            ford_depth_m,
        ));
    }
    out
}

/// A cell's lattice coordinates on `face`, if it lies on it.
fn on_face(key: u32, face: u8) -> Option<(i64, i64)> {
    let a = crate::cell_key::unpack(key)?;
    crate::cell_key::addresses(a)
        .into_iter()
        .find(|x| x.face == face)
        .map(|x| (i64::from(x.i), i64::from(x.j)))
}

/// The cell `(di, dj)` lattice steps from `key` on `face`.
fn translated(key: u32, face: u8, level: u8, (di, dj): (i64, i64)) -> Option<u32> {
    let (i, j) = on_face(key, face)?;
    let (i, j) = (i + di, j + dj);
    if i < 0 || j < 0 {
        return None;
    }
    crate::cell_key::key(crate::cell_key::Address {
        face,
        level,
        i: i as u32,
        j: j as u32,
    })
}

/// A wide road's lane cells (design 42l): the centre line moved one
/// lattice step to its left (width 2) or to both sides (width 3), the step
/// whose cell lies most square to the road, so a lane is the centre's own
/// line beside it, cell for cell; each on its centre cell's height.
fn lanes(
    width: u8,
    line: &[u32],
    dirs: &[Vec3],
    on: &[LaidCell],
    ground: &GroundFn,
    tree: &TreeFn,
    ford_depth_m: f32,
) -> Vec<LaidCell> {
    let on_line: BTreeSet<u32> = line.iter().copied().collect();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    let (Some(first), Some(last)) = (line.first(), line.last()) else {
        return out;
    };
    let (Some(ca), Some(cb)) = (centre(*first), centre(*last)) else {
        return out;
    };
    let Some(left) = ca.cross(cb).try_normalize() else {
        return out;
    };
    let Some(addr) = crate::cell_key::unpack(*first) else {
        return out;
    };
    let sides: &[i8] = if width >= 3 { &[1, -1] } else { &[1] };
    for &sign in sides {
        // The step from the first cell whose neighbour lies most to this
        // side, on a face the whole line lies on.
        let pick = crate::cell_key::addresses(addr).into_iter().find_map(|fa| {
            if !line.iter().all(|&k| on_face(k, fa.face).is_some()) {
                return None;
            }
            let (i0, j0) = (i64::from(fa.i), i64::from(fa.j));
            neighbours(*first)
                .into_iter()
                .filter_map(|n| {
                    let (i, j) = on_face(n, fa.face)?;
                    let side = centre(n)?.dot(left) * f32::from(sign);
                    Some(((i - i0, j - j0), side))
                })
                .max_by(|x, y| x.1.total_cmp(&y.1))
                .map(|(d, _)| (fa.face, fa.level, d))
        });
        let Some((face, level, step)) = pick else {
            continue;
        };
        let moved: Vec<Option<u32>> = line
            .iter()
            .map(|&k| translated(k, face, level, step))
            .collect();
        for (j, cell) in moved.iter().enumerate() {
            let Some(nb) = *cell else { continue };
            if on_line.contains(&nb) || !seen.insert(nb) {
                continue;
            }
            let plane = &on[j];
            let g = ground(nb);
            let mut clash = match g {
                None => Some(Clash::InTheWay),
                Some(g) => check_cell(&g, tree(nb), ford_depth_m),
            };
            let top = g.map_or(plane.low_m, |g| g.top_m);
            // On a slope a lane beside a sloped cell would need a bed half a
            // layer high to share its plane (design 10b): not built yet, so
            // refused rather than drawn with a seam (design 42l).
            if plane.up.is_some() {
                clash.get_or_insert(Clash::TooSteepAcross);
            }
            let (base_m, raised) = if top == plane.low_m {
                (top, false)
            } else if top == plane.low_m - 1 {
                (top, true)
            } else {
                clash.get_or_insert(Clash::TooSteepAcross);
                (top, false)
            };
            let along = |k: Option<usize>| k.and_then(|k| moved.get(k).copied().flatten());
            out.push(LaidCell {
                key: nb,
                centre: line[j],
                lane: sign,
                before: along(j.checked_sub(1)).or_else(|| toward(nb, -dirs[j])),
                after: along(Some(j + 1)).or_else(|| toward(nb, dirs[j])),
                low_m: plane.low_m,
                base_m,
                raised,
                ford: g.is_some_and(|g| matches!(g.water, Water::River { .. })),
                up: None,
                from: None,
                clash,
            });
        }
    }
    out
}

/// The first clash of a laid-out road, by its order along the road.
pub fn first_clash(laid: &[LaidCell]) -> Option<(u32, Clash)> {
    laid.iter().find_map(|c| Some((c.key, c.clash?)))
}

/// A whole road paved cell `key` of, as the player's paved cell of plan
/// `plan` (design 40, 42g): `None` where the cell is not on the road or
/// clashes.
pub fn paved_cell(
    plan: u64,
    road: &PlanRoad,
    key: u32,
    ground: &GroundFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Option<BuiltCell> {
    // Paving never asks of trees: a tree on a planned cell was there when
    // the road was staked, or it would not have been.
    let laid = lay(road, ground, &|_| false, cfg, ford_depth_m);
    let cell = laid.iter().find(|c| c.key == key)?;
    if cell.clash.is_some() {
        return None;
    }
    Some(cell.built(plan, road.surface))
}

// ---- Every plan's cells ------------------------------------------------------------

/// Every cell every plan's roads run over, by key: the plan, the road's
/// index in it and the laid cell (the first plan and road to reach a cell).
pub fn every_cell(
    plans: &BTreeMap<u64, RoadPlan>,
    ground: &GroundFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> BTreeMap<u32, (u64, usize, LaidCell)> {
    let mut out = BTreeMap::new();
    for (&id, plan) in plans {
        for (r, road) in plan.roads.iter().enumerate() {
            for cell in lay(road, ground, &|_| false, cfg, ford_depth_m) {
                out.entry(cell.key).or_insert((id, r, cell));
            }
        }
    }
    out
}

/// Every marked cell (a plan's cell not yet paved) with the cells before
/// and after it, for the band (design 39a).
pub fn marked(
    cells: &BTreeMap<u32, (u64, usize, LaidCell)>,
    paved: &BTreeMap<u32, BuiltCell>,
) -> BTreeMap<u32, (Option<u32>, Option<u32>)> {
    cells
        .iter()
        .filter(|(k, _)| !paved.contains_key(k))
        .map(|(&k, (_, _, c))| (k, (c.before, c.after)))
        .collect()
}

/// The paved cells of a plan no road runs over any more: marked for
/// removal (design 42f, 42l).
pub fn for_removal(
    cells: &BTreeMap<u32, (u64, usize, LaidCell)>,
    paved: &BTreeMap<u32, BuiltCell>,
) -> Vec<u32> {
    paved
        .iter()
        .filter(|(k, c)| c.plan.is_some() && !cells.contains_key(k))
        .map(|(&k, _)| k)
        .collect()
}

// ---- Editing (design 42f, 42k) ---------------------------------------------------

/// Where a stake stands: its plan, road and place among the road's stakes.
pub fn stake_at(plans: &BTreeMap<u64, RoadPlan>, key: u32) -> Option<(u64, usize, usize)> {
    plans.iter().find_map(|(&id, p)| {
        p.roads.iter().enumerate().find_map(|(r, road)| {
            road.stakes
                .iter()
                .position(|&s| s == key)
                .map(|s| (id, r, s))
        })
    })
}

/// The road whose cells (lanes included) hold `key`: its plan and index.
pub fn road_at(
    plans: &BTreeMap<u64, RoadPlan>,
    key: u32,
    ground: &GroundFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Option<(u64, usize)> {
    plans.iter().find_map(|(&id, p)| {
        p.roads.iter().enumerate().find_map(|(r, road)| {
            lay(road, ground, &|_| false, cfg, ford_depth_m)
                .iter()
                .any(|c| c.key == key)
                .then_some((id, r))
        })
    })
}

/// The plan a new road from `start` goes into (design 42l): the plan whose
/// stake it starts on, else the newest, else `next_id`.
pub fn plan_for(plans: &BTreeMap<u64, RoadPlan>, start: u32, next_id: u64) -> u64 {
    stake_at(plans, start)
        .map(|(id, _, _)| id)
        .or_else(|| plans.keys().next_back().copied())
        .unwrap_or(next_id)
}

/// The plans after an edit, and what it does to the pack: stakes given
/// (more than nought) or taken (less).
#[derive(Clone, Debug, PartialEq)]
pub struct Edited {
    pub plans: BTreeMap<u64, RoadPlan>,
    pub stakes: i32,
}

fn count_change(before: &BTreeMap<u64, RoadPlan>, after: &BTreeMap<u64, RoadPlan>) -> i32 {
    stakes(before).len() as i32 - stakes(after).len() as i32
}

/// Add a road from `start` to `end` (design 42k): staked every
/// `max_segment` cells, joined to any stake already at either end. Refused
/// with the first clash on it.
#[allow(clippy::too_many_arguments)]
pub fn add_road(
    plans: &BTreeMap<u64, RoadPlan>,
    next_id: u64,
    start: u32,
    end: u32,
    how: How,
    ground: &GroundFn,
    tree: &TreeFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Result<Edited, Clash> {
    let road = PlanRoad::new(stake_line(start, end, cfg.max_segment), how);
    if let Some((_, why)) = first_clash(&lay(&road, ground, tree, cfg, ford_depth_m)) {
        return Err(why);
    }
    let id = plan_for(plans, start, next_id);
    let mut after = plans.clone();
    after.entry(id).or_default().roads.push(road);
    let stakes = count_change(plans, &after);
    Ok(Edited {
        plans: after,
        stakes,
    })
}

/// Erase the road `(plan, index)` (design 42k): its stakes come back where
/// no other road holds them.
pub fn erase(plans: &BTreeMap<u64, RoadPlan>, plan: u64, index: usize) -> Edited {
    let mut after = plans.clone();
    if let Some(p) = after.get_mut(&plan)
        && index < p.roads.len()
    {
        p.roads.remove(index);
    }
    after.retain(|_, p| !p.roads.is_empty());
    let stakes = count_change(plans, &after);
    Edited {
        plans: after,
        stakes,
    }
}

/// Move the stake at `from` to `to` (design 42f, 42k: shift and a click
/// picks it up): every road holding it follows, refused with the first
/// clash on any of them.
pub fn move_stake(
    plans: &BTreeMap<u64, RoadPlan>,
    from: u32,
    to: u32,
    ground: &GroundFn,
    tree: &TreeFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Result<Edited, Clash> {
    let mut after = plans.clone();
    for road in after.values_mut().flat_map(|p| p.roads.iter_mut()) {
        if road.stakes.contains(&from) {
            for s in road.stakes.iter_mut().filter(|s| **s == from) {
                *s = to;
            }
            road.cells = None;
            road.stakes.dedup();
            if let Some((_, why)) = first_clash(&lay(road, ground, tree, cfg, ford_depth_m)) {
                return Err(why);
            }
        }
    }
    let stakes = count_change(plans, &after);
    Ok(Edited {
        plans: after,
        stakes,
    })
}

/// Take the stake at `key` back (design 42f, 42k): a road ending on it is
/// shortened, and goes whole when two stakes are all it has; a middle stake's
/// neighbours are joined by one leg, refused with its clash where that leg
/// would be.
pub fn take_back(
    plans: &BTreeMap<u64, RoadPlan>,
    key: u32,
    ground: &GroundFn,
    tree: &TreeFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Result<Edited, Clash> {
    let mut after = plans.clone();
    for road in after.values_mut().flat_map(|p| p.roads.iter_mut()) {
        let Some(s) = road.stakes.iter().position(|&k| k == key) else {
            continue;
        };
        let n = road.stakes.len();
        if n <= 2 {
            road.stakes.clear();
            continue;
        }
        road.stakes.remove(s);
        road.cells = None;
        if s > 0 && s < n - 1 {
            if let Some((_, why)) = first_clash(&lay(road, ground, tree, cfg, ford_depth_m)) {
                return Err(why);
            }
        }
    }
    for p in after.values_mut() {
        p.roads.retain(|r| !r.stakes.is_empty());
    }
    after.retain(|_, p| !p.roads.is_empty());
    let stakes = count_change(plans, &after);
    Ok(Edited {
        plans: after,
        stakes,
    })
}

// ---- The bill and the counts (design 42d) ------------------------------------

/// What is still to build, and the counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bill {
    pub stakes: u32,
    pub gravel: u32,
    pub lamps: u32,
    /// Cells on the plans, and of those not yet paved.
    pub cells: u32,
    pub left: u32,
    pub turns: u32,
}

/// The bill of every plan, plus the aim's road `aim` if any (its stakes
/// past the one it starts on).
pub fn bill(
    plans: &BTreeMap<u64, RoadPlan>,
    aim: Option<&PlanRoad>,
    paved: &BTreeMap<u32, BuiltCell>,
    ground: &GroundFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Bill {
    let mut b = Bill::default();
    let mut seen = BTreeSet::new();
    let mut gravel = 0.0f32;
    let roads = plans.values().flat_map(|p| &p.roads).chain(aim);
    for road in roads {
        let laid = lay(road, ground, &|_| false, cfg, ford_depth_m);
        for c in &laid {
            if !seen.insert(c.key) {
                continue;
            }
            b.cells += 1;
            if !paved.contains_key(&c.key) {
                b.left += 1;
                if road.surface == Surface::Gravel || c.ford {
                    gravel += cfg.cost_scale;
                }
            }
        }
        b.lamps += lamp_cells(road, cfg).len() as u32;
        b.turns += turns(road);
    }
    b.gravel = gravel.ceil() as u32;
    if let Some(aim) = aim {
        let standing = stakes(plans);
        b.stakes = aim.stakes.iter().filter(|s| !standing.contains(s)).count() as u32;
    }
    b
}

/// The waypoints of a road where its heading changes (design 42d): a hex
/// line's own zigzag is not a turn.
pub fn turns(road: &PlanRoad) -> u32 {
    road.stakes
        .windows(3)
        .filter(|w| {
            let (Some(a), Some(b), Some(c)) = (centre(w[0]), centre(w[1]), centre(w[2])) else {
                return false;
            };
            let (d1, d2) = ((b - a).normalize_or_zero(), (c - b).normalize_or_zero());
            d1.dot(d2) < 0.97
        })
        .count() as u32
}

/// The cells a lamp post stands beside along a road with lamps (design
/// 42e): every `lamp_spacing`th cell of the centre line, from the start.
pub fn lamp_cells(road: &PlanRoad, cfg: &RoadPlannerConfig) -> Vec<u32> {
    if !road.lamps {
        return Vec::new();
    }
    road.centre()
        .into_iter()
        .enumerate()
        .filter(|(k, _)| k % cfg.lamp_spacing.max(1) as usize == cfg.lamp_spacing as usize / 2)
        .map(|(_, c)| c)
        .collect()
}

#[cfg(test)]
mod tests;
