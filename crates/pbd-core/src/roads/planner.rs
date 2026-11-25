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

use super::{
    Bed, BuiltCell, GroundFn, Kind, RoadCell, Surface, Water, centre, edge_middle, neighbours,
    paving,
};
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
    /// Water past the longest bridge, or over the deck a bridge would have
    /// (design 42m).
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
/// standing on it or not: the player's work, a town, the sea and a tree
/// each refuse it. Water deeper than a ford is bridged by [`bridged`], the
/// world's rule and the player's, not refused here. The world's search asks
/// the same and enters neither the player's work nor the sea; it crosses a
/// town.
pub fn check_cell(g: &super::Ground, tree: bool) -> Option<Clash> {
    if g.player {
        return Some(Clash::InTheWay);
    }
    if g.water == Water::Sea {
        return Some(Clash::OpenWater);
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
    /// The longest bridge the player builds, cells (design 42m).
    pub max_bridge_cells: u32,
    /// Planks a bridge cell takes; it takes a log as well.
    pub bridge_planks: u32,
    /// How far over the player the eye is lifted while the planner is in
    /// hand, m (design 42n), and how long it eases in or out, s.
    pub aerial_m: f32,
    pub aerial_ease_s: f32,
}

impl Default for RoadPlannerConfig {
    /// The values `road_planner.ron` ships, for tests.
    fn default() -> Self {
        Self {
            ghost_opacity: 0.55,
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
            max_bridge_cells: 6,
            bridge_planks: 6,
            aerial_m: 15.0,
            aerial_ease_s: 0.4,
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
        if !(1..=10).contains(&self.max_bridge_cells) {
            return Err("max_bridge_cells must be from 1 to 10 cells".into());
        }
        if !(1..=32).contains(&self.bridge_planks) {
            return Err("bridge_planks must be from 1 to 32 planks".into());
        }
        if !(self.aerial_m.is_finite() && (5.0..=30.0).contains(&self.aerial_m)) {
            return Err("aerial_m must be from 5 to 30 m".into());
        }
        if !(self.aerial_ease_s.is_finite() && (0.0..=3.0).contains(&self.aerial_ease_s)) {
            return Err("aerial_ease_s must be from 0 to 3 s".into());
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
    /// A bridge's deck, m (design 42m).
    pub deck: Option<i32>,
    /// A wide road's half-height bed and rise (design 42m).
    pub bed: Bed,
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
            kind: match (self.deck, self.ford) {
                (Some(deck_m), _) => Kind::Bridge { deck_m },
                (None, true) => Kind::Ford,
                (None, false) => Kind::Ground,
            },
            raised: self.raised,
            cut_fill: 0,
            bed: self.bed,
            up: self.up,
            from: self.from,
        }
    }

    /// As the player's paved cell of plan `plan`, in `surface`.
    pub fn built(&self, plan: u64, surface: Surface) -> BuiltCell {
        let ford = self.ford && self.deck.is_none();
        BuiltCell {
            surface: if ford { Surface::Gravel } else { surface },
            base_m: self.base_m,
            up: self.up,
            from: self.from,
            raised: self.raised,
            plan: Some(plan),
            ford,
            bed: self.bed,
            deck_m: self.deck,
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

/// The bridge rule (design 10a, 42m), one for the world's search and the
/// player's planner: a river cell deeper than a ford, or any river cell
/// while a bridge is carried (`run` cells so far at `deck`), is bridged at
/// the road's height on the bank before it (`bank_m`), carried on, for at
/// most `max_cells` and only where the deck stands over the water. `None`
/// where the cell is not bridged (ground or a ford); `Some(None)` where it
/// would be and is refused; else the deck and the run with this cell.
pub fn bridged(
    water: Water,
    run: u32,
    deck: i32,
    bank_m: i32,
    ford_depth_m: f32,
    max_cells: u32,
) -> Option<Option<(i32, u32)>> {
    match water {
        Water::River { depth_m, surface_m } if depth_m as f32 > ford_depth_m || run > 0 => {
            let deck = if run > 0 { deck } else { bank_m };
            Some((deck >= surface_m && run < max_cells).then_some((deck, run + 1)))
        }
        _ => None,
    }
}

/// The centre line's heights after the profile's raise of one-cell dips
/// (design 10), and which were raised; a `fixed` cell (a bridge's) is never
/// raised.
fn heights(tops: &[i32], fixed: &[bool]) -> (Vec<i32>, Vec<bool>) {
    let n = tops.len();
    let mut h = tops.to_vec();
    let mut raised = vec![false; n];
    for _ in 0..4 {
        let mut changed = false;
        for k in 1..n.saturating_sub(1) {
            if !fixed[k] && !raised[k] && h[k] < h[k - 1] && h[k] < h[k + 1] {
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

/// The road's plane on the half-layer grid (design 42m), in half metres:
/// `p[2k]` the edge before centre cell `k` (the higher cell's height),
/// `p[2k + 1]` its middle, `p[2n]` the far end.
fn plane(h: &[i32]) -> Vec<i32> {
    let n = h.len();
    let mut p = vec![0; 2 * n + 1];
    if n == 0 {
        return p;
    }
    p[0] = 2 * h[0];
    p[2 * n] = 2 * h[n - 1];
    for k in 1..n {
        p[2 * k] = 2 * h[k - 1].max(h[k]);
    }
    for k in 0..n {
        p[2 * k + 1] = (p[2 * k] + p[2 * k + 2]).div_euclid(2);
    }
    p
}

/// The plane moved half a cell down the hill (design 42m): each point the
/// higher of its two neighbours, the ends held where they are.
fn moved_plane(p: &[i32]) -> Vec<i32> {
    let m = p.len();
    (0..m)
        .map(|q| {
            if q == 0 || q + 1 == m {
                p[q]
            } else {
                p[q - 1].max(p[q + 1])
            }
        })
        .collect()
}

/// The road's plane over the ground (design 42m): its heights on the
/// half-cell grid of its centre line ([`plane`]), read at any place by
/// where that place lies along the line. The lattice on the sphere is not
/// exactly regular, so a lane's cells are not exactly half a cell along
/// their centre cells: each cell is read where it really is.
struct Plane {
    centres: Vec<Vec3>,
    h: Vec<i32>,
}

/// Why a cell's top cannot be read off the plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Unfit {
    /// Its ground stands over the plane: a cut, which the player's road
    /// does not dig.
    Cut,
    /// Two layers or more of fill, or a plane that turns on it.
    Other,
}

/// A cell's top as read off the plane: raised a layer or not, its bed, and
/// whether it rises toward the cell it leaves by (`Some(true)`), the one it
/// comes in by (`Some(false)`), or is level.
type Read = (bool, Bed, Option<bool>);

impl Plane {
    fn new(line: &[u32], h: Vec<i32>) -> Self {
        Self {
            centres: line.iter().map(|&k| centre(k).unwrap_or(Vec3::Y)).collect(),
            h,
        }
    }

    /// Where `x` lies along the centre line, in cells from its first.
    fn along(&self, x: Vec3) -> f32 {
        let n = self.centres.len();
        if n < 2 {
            return 0.0;
        }
        let k = (0..n)
            .min_by(|&a, &b| {
                (self.centres[a] - x)
                    .length_squared()
                    .total_cmp(&(self.centres[b] - x).length_squared())
            })
            .unwrap_or(0);
        let param = |a: usize| {
            let (ca, cb) = (self.centres[a], self.centres[a + 1]);
            let d = cb - ca;
            (x - ca).dot(d) / d.length_squared().max(1e-20)
        };
        let ahead = (k + 1 < n).then(|| param(k));
        let behind = (k > 0).then(|| param(k - 1));
        match (behind, ahead) {
            (_, Some(p)) if p >= 0.0 || k == 0 => k as f32 + p,
            (Some(p), _) => (k - 1) as f32 + p,
            _ => k as f32,
        }
    }

    /// The plane's height `u` cells along, m, held level past either end.
    fn at(&self, u: f32) -> f32 {
        let q = (2.0 * u + 1.0).clamp(0.0, (self.h.len() - 1) as f32);
        let i = (q.floor() as usize).min(self.h.len() - 1);
        let j = (i + 1).min(self.h.len() - 1);
        let f = q - i as f32;
        (self.h[i] as f32 * (1.0 - f) + self.h[j] as f32 * f) / 2.0
    }

    /// The plane's lowest and highest over a cell from `foot` to `head`.
    fn span(&self, foot: Vec3, head: Vec3) -> (f32, f32) {
        let (a, c) = (self.at(self.along(foot)), self.at(self.along(head)));
        (a.min(c), a.max(c))
    }

    /// A cell's top read off the plane between its `foot` and its `head`
    /// (the middles of its sides toward the cells it comes in and leaves
    /// by) over ground at `ground_m`.
    fn read(&self, foot: Vec3, head: Vec3, ground_m: i32) -> Result<Read, Unfit> {
        let (mut u0, mut u1) = (self.along(foot), self.along(head));
        let (mut a, mut c) = (self.at(u0), self.at(u1));
        let forward = a <= c + 1e-4;
        if !forward {
            std::mem::swap(&mut u0, &mut u1);
            std::mem::swap(&mut a, &mut c);
        }
        let span = u1 - u0;
        // The plane's knots between: where its rise starts and ends.
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        if span.abs() > 1e-4 {
            for q in 0..self.h.len() {
                let u = (q as f32 - 1.0) / 2.0;
                let t = (u - u0) / span;
                if !(1e-4..1.0 - 1e-4).contains(&t) {
                    continue;
                }
                let v = self.at(u);
                if v < a - 1e-3 || v > c + 1e-3 {
                    return Err(Unfit::Other);
                }
                if v <= a + 1e-3 {
                    t0 = t0.max(t);
                }
                if v >= c - 1e-3 {
                    t1 = t1.min(t);
                }
            }
        }
        let fill = a - ground_m as f32;
        if fill < -0.01 {
            return Err(Unfit::Cut);
        }
        // A layer a cell, give or take the lattice's own unevenness.
        if fill >= 1.995 || c - a > 1.05 {
            return Err(Unfit::Other);
        }
        let raised = fill >= 0.995;
        let lift = (fill - f32::from(u8::from(raised))).max(0.0);
        let lift_mm = (lift * 1000.0).round() as u16 % 1000;
        let rise = c - a;
        let rises = (rise > 0.005).then_some(forward);
        let plain_rise = (rise - 1.0).abs() < 0.005 && t0 < 1e-3 && t1 > 1.0 - 1e-3;
        let bed = if lift_mm == 0 && (rises.is_none() || plain_rise) {
            Bed::default()
        } else {
            Bed {
                lift_mm,
                from: (t0 * 1000.0).round() as u16,
                to: (t1 * 1000.0).round() as u16,
                rise_mm: (rise * 1000.0).round() as u16,
            }
        };
        Ok((raised, bed, rises))
    }
}

/// The middles of a cell's sides toward the cells it comes in and leaves
/// by: its foot and head along the road (the centre mirrored where it has
/// no such neighbour).
fn ends(key: u32, before: Option<u32>, after: Option<u32>) -> Option<(Vec3, Vec3)> {
    let c = centre(key)?;
    let side = |n: Option<u32>| {
        n.filter(|n| neighbours(key).contains(n))
            .and_then(|n| edge_middle(key, n))
    };
    match (side(before), side(after)) {
        (Some(f), Some(h)) => Some((f, h)),
        (Some(f), None) => Some((f, (c * 2.0 - f).normalize())),
        (None, Some(h)) => Some(((c * 2.0 - h).normalize(), h)),
        (None, None) => None,
    }
}

/// Lay out a road over `ground` (design 42a to 42c, 42l, 42m): its centre
/// line and lanes, each cell's top cut by the road rule, bridges where the
/// water is deep, and each cell that clashes with why. Pure.
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
    let mut clash: Vec<Option<Clash>> = grounds
        .iter()
        .zip(&line)
        .map(|(g, &k)| match g {
            None => Some(Clash::InTheWay),
            Some(g) => check_cell(g, tree(k)),
        })
        .collect();
    // Bridges, by the world's rule, carried from the bank before.
    let mut deck: Vec<Option<i32>> = vec![None; n];
    // A refused bridge's water is held at the deck it would have had, so
    // only it is red, not the banks either side of it.
    let mut held: Vec<Option<i32>> = vec![None; n];
    let (mut run, mut carried, mut bank): (u32, i32, Option<i32>) = (0, 0, None);
    for k in 0..n {
        let Some(g) = grounds[k] else {
            (run, bank) = (0, None);
            continue;
        };
        match bridged(
            g.water,
            run,
            carried,
            bank.unwrap_or(i32::MIN),
            ford_depth_m,
            cfg.max_bridge_cells,
        ) {
            Some(Some((d, r))) if bank.is_some() => {
                deck[k] = Some(d);
                (run, carried, bank) = (r, d, Some(d));
            }
            Some(_) => {
                clash[k].get_or_insert(Clash::TooWideToBridge);
                if run == 0 {
                    carried = bank.unwrap_or(g.top_m);
                }
                held[k] = Some(carried);
                run += 1;
            }
            None => (run, bank) = (0, Some(g.top_m)),
        }
    }
    let fixed: Vec<bool> = (0..n)
        .map(|k| deck[k].is_some() || held[k].is_some())
        .collect();
    let tops_or_deck: Vec<i32> = (0..n)
        .map(|k| deck[k].or(held[k]).unwrap_or(tops[k]))
        .collect();
    let (h, raised) = heights(&tops_or_deck, &fixed);
    // Steps and the grade: the higher of two cells too far apart is red.
    let mut last_rise: Option<usize> = None;
    for k in 1..n {
        if let Some(c) = check_step(h[k - 1], h[k]) {
            let higher = if h[k] > h[k - 1] { k } else { k - 1 };
            clash[higher].get_or_insert(c);
        } else if h[k] != h[k - 1] {
            let higher = if h[k] > h[k - 1] { k } else { k - 1 };
            let lower = if higher == k { k - 1 } else { k };
            // A deck never slopes: a bank a layer over it is a step.
            if fixed[lower] {
                clash[higher].get_or_insert(Clash::TooSteep);
            }
            if let Some(r) = last_rise
                && k - r < cfg.max_grade as usize
            {
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
    // A wide road bends only on the level, and climbs straight along a row
    // of the lattice (design 10b, 42m): a turn, or a hex line's zigzag, on
    // or beside a rise is red.
    if road.width > 1 {
        let step = |a: usize, b: usize| -> Vec3 {
            match (centre(line[a]), centre(line[b])) {
                (Some(x), Some(y)) => (y - x).normalize_or_zero(),
                _ => Vec3::ZERO,
            }
        };
        for k in 1..n.saturating_sub(1) {
            if h[k - 1] == h[k] && h[k] == h[k + 1] {
                continue;
            }
            let bend = road.stakes.contains(&line[k]) && dirs[k - 1].dot(dirs[k + 1]) < 0.94;
            let zigzag = step(k - 1, k).dot(step(k, k + 1)) < 0.99;
            if bend || zigzag {
                clash[k].get_or_insert(Clash::WideBend);
            }
        }
    }
    let mut out = Vec::new();
    for k in 0..n {
        let (mut up, mut from) = (None, None);
        if deck[k].is_none() {
            for (j, o) in [(k + 1, k.wrapping_sub(1)), (k.wrapping_sub(1), k + 1)] {
                if j < n && h[j] == h[k] + 1 {
                    up = Some(line[j]);
                    from = (o < n).then(|| line[o]);
                    break;
                }
            }
        }
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
            deck: deck[k],
            bed: Bed::default(),
            up,
            from,
            clash: clash[k],
        });
    }
    if road.width > 1 && n > 0 {
        let flat = Plane::new(&line, plane(&h));
        let sides = Lanes {
            width: road.width,
            line: &line,
            dirs: &dirs,
            ground,
            tree,
            ford_depth_m,
        };
        let (mut chosen, cuts) = sides.lay(&out, &flat);
        // The plane moved half a cell down the hill at a time, at most a
        // whole cell, where its centre cells all still take it and it saves
        // a cut; the least moved that saves most.
        let (mut best, mut h) = (cuts, flat.h.clone());
        for _ in 0..2 {
            if best == 0 {
                break;
            }
            h = moved_plane(&h);
            if !h.windows(2).all(|w| (w[1] - w[0]).abs() <= 1) {
                break;
            }
            let moved = Plane::new(&line, h.clone());
            let recut: Option<Vec<LaidCell>> =
                out.iter().map(|c| recut_centre(c, &moved)).collect();
            let Some(recut) = recut else { break };
            let (lanes1, cuts1) = sides.lay(&recut, &moved);
            if cuts1 < best {
                best = cuts1;
                out = recut;
                chosen = lanes1;
            }
        }
        out.extend(chosen);
    }
    out
}

/// A centre cell re-read off the moved plane (design 42m): `None` where it
/// cannot take it (a deck the plane would tilt, a bed past its fill).
fn recut_centre(c: &LaidCell, p: &Plane) -> Option<LaidCell> {
    let (foot, head) = ends(c.key, c.before, c.after)?;
    if let Some(d) = c.deck {
        let (lo, hi) = p.span(foot, head);
        return ((lo - d as f32).abs() < 1e-3 && (hi - d as f32).abs() < 1e-3).then_some(*c);
    }
    let (raised, bed, rises) = match p.read(foot, head, c.base_m) {
        Ok(s) => s,
        Err(_) if c.clash.is_some() => return Some(*c),
        Err(_) => return None,
    };
    let (up, from) = match rises {
        Some(true) => (c.after, c.before),
        Some(false) => (c.before, c.after),
        None => (None, None),
    };
    Some(LaidCell {
        low_m: c.base_m + i32::from(raised),
        raised,
        bed,
        up,
        from,
        ..*c
    })
}

/// A cell's lattice coordinates on `face`, if it lies on it.
fn on_face(key: u32, face: u8) -> Option<(i64, i64)> {
    let a = crate::cell_key::unpack(key)?;
    crate::cell_key::addresses(a)
        .into_iter()
        .find(|x| x.face == face)
        .map(|x| (i64::from(x.i), i64::from(x.j)))
}

/// The cell `cells` steps from `key` along the row of the lattice that runs
/// most toward `to`: where a wide road can climb (design 42m). `None` off
/// the face.
pub fn along_row(key: u32, to: u32, cells: i64) -> Option<u32> {
    let (c, t) = (centre(key)?, centre(to)?);
    let addr = crate::cell_key::unpack(key)?;
    let fa = crate::cell_key::addresses(addr).into_iter().next()?;
    let (i0, j0) = (i64::from(fa.i), i64::from(fa.j));
    let step = neighbours(key)
        .into_iter()
        .filter_map(|n| {
            let (i, j) = on_face(n, fa.face)?;
            Some(((i - i0, j - j0), (centre(n)? - c).dot(t - c)))
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))?
        .0;
    translated(key, fa.face, fa.level, (step.0 * cells, step.1 * cells))
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

/// What a wide road's lanes are laid from.
struct Lanes<'a> {
    width: u8,
    line: &'a [u32],
    dirs: &'a [Vec3],
    ground: &'a GroundFn<'a>,
    tree: &'a TreeFn<'a>,
    ford_depth_m: f32,
}

impl Lanes<'_> {
    /// A wide road's lane cells (design 42l, 42m): the centre line moved
    /// one lattice step to its left (width 2) or to both sides (width 3),
    /// the step whose cell lies most square to the road, so a lane is the
    /// centre's own line beside it, cell for cell; each cell's top read off
    /// the road's plane `p` (half metres, [`plane`]) over its own ground, on
    /// a half-height bed where it falls half a layer over it. With how many
    /// would need a cut.
    fn lay(&self, on: &[LaidCell], p: &Plane) -> (Vec<LaidCell>, usize) {
        let line = self.line;
        let on_line: BTreeSet<u32> = line.iter().copied().collect();
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        let mut cuts = 0;
        let (Some(first), Some(last)) = (line.first(), line.last()) else {
            return (out, cuts);
        };
        let (Some(ca), Some(cb)) = (centre(*first), centre(*last)) else {
            return (out, cuts);
        };
        let Some(left) = ca.cross(cb).try_normalize() else {
            return (out, cuts);
        };
        let Some(addr) = crate::cell_key::unpack(*first) else {
            return (out, cuts);
        };
        let sides: &[i8] = if self.width >= 3 { &[1, -1] } else { &[1] };
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
                let centre_cell = &on[j];
                let g = (self.ground)(nb);
                let deep = |g: &super::Ground| matches!(g.water, Water::River { depth_m, .. } if depth_m as f32 > self.ford_depth_m);
                let mut clash = match g {
                    None => Some(Clash::InTheWay),
                    Some(g) => check_cell(&g, (self.tree)(nb)),
                };
                let top = g.map_or(centre_cell.low_m, |g| g.top_m);
                let along = |k: Option<usize>| k.and_then(|k| moved.get(k).copied().flatten());
                let before = along(j.checked_sub(1)).or_else(|| toward(nb, -self.dirs[j]));
                let after = along(Some(j + 1)).or_else(|| toward(nb, self.dirs[j]));
                let Some((foot, head)) = ends(nb, before, after) else {
                    continue;
                };
                let (lo, hi) = p.span(foot, head);
                let mut cell = LaidCell {
                    key: nb,
                    centre: line[j],
                    lane: sign,
                    before,
                    after,
                    low_m: centre_cell.low_m,
                    base_m: top,
                    raised: false,
                    ford: g.is_some_and(|g| matches!(g.water, Water::River { .. })),
                    deck: None,
                    bed: Bed::default(),
                    up: None,
                    from: None,
                    clash: None,
                };
                let level = hi - lo < 1e-3 && (lo - lo.round()).abs() < 1e-3;
                if centre_cell.deck.is_some() || g.as_ref().is_some_and(deep) {
                    // A wide bridge: a deck over every lane at the plane,
                    // level, standing over the water (design 10b, 42m).
                    let d = lo.round() as i32;
                    let over = match g.map(|g| g.water) {
                        Some(Water::River { surface_m, .. }) => d >= surface_m,
                        _ => true,
                    };
                    if !level || !over {
                        let why = if g.as_ref().is_some_and(deep) {
                            Clash::TooWideToBridge
                        } else {
                            Clash::TooSteepAcross
                        };
                        clash.get_or_insert(why);
                    } else if top < d || g.as_ref().is_some_and(deep) {
                        cell.deck = Some(d);
                        cell.low_m = d;
                    } else if top == d {
                        cell.low_m = d;
                    } else {
                        clash.get_or_insert(Clash::TooSteepAcross);
                    }
                } else {
                    match p.read(foot, head, top) {
                        Ok((raised, bed, rises)) => {
                            cell.raised = raised;
                            cell.bed = bed;
                            cell.low_m = top + i32::from(raised);
                            (cell.up, cell.from) = match rises {
                                Some(true) => (after, before),
                                Some(false) => (before, after),
                                None => (None, None),
                            };
                        }
                        Err(why) => {
                            if why == Unfit::Cut {
                                cuts += 1;
                            }
                            clash.get_or_insert(Clash::TooSteepAcross);
                        }
                    }
                }
                cell.clash = clash;
                out.push(cell);
            }
        }
        (out, cuts)
    }
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
        if s > 0
            && s < n - 1
            && let Some((_, why)) = first_clash(&lay(road, ground, tree, cfg, ford_depth_m))
        {
            return Err(why);
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
    /// Lamp posts not yet built (design 42e, 42m).
    pub lamps: u32,
    /// A bridge cell's planks, and its log and each lamp post's (42d).
    pub planks: u32,
    pub logs: u32,
    /// Street lanterns: one a lamp post.
    pub lanterns: u32,
    /// Cells on the plans, and of those not yet paved.
    pub cells: u32,
    pub left: u32,
    pub turns: u32,
}

/// What a bill is read against: whether a lamp post already stands on a
/// cell.
pub type LitFn<'a> = dyn Fn(u32) -> bool + 'a;

/// The bill of every plan, plus the aim's road `aim` if any (its stakes
/// past the one it starts on); `lit` says which lamp posts stand already.
#[allow(clippy::too_many_arguments)]
pub fn bill(
    plans: &BTreeMap<u64, RoadPlan>,
    aim: Option<&PlanRoad>,
    paved: &BTreeMap<u32, BuiltCell>,
    lit: &LitFn,
    ground: &GroundFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Bill {
    let mut b = Bill::default();
    let mut seen = BTreeSet::new();
    let mut posts = BTreeSet::new();
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
                if c.deck.is_some() {
                    b.planks += cfg.bridge_planks;
                    b.logs += 1;
                } else if road.surface == Surface::Gravel || c.ford {
                    gravel += cfg.cost_scale;
                }
            }
        }
        for (_, post) in lamp_posts(road, &laid, ground, cfg) {
            if posts.insert(post) && !lit(post) {
                b.lamps += 1;
            }
        }
        b.turns += turns(road);
    }
    b.logs += b.lamps;
    b.lanterns = b.lamps;
    b.planks = (b.planks as f32 * cfg.cost_scale).ceil() as u32;
    b.gravel = gravel.ceil() as u32;
    if let Some(aim) = aim {
        let standing = stakes(plans);
        b.stakes = aim.stakes.iter().filter(|s| !standing.contains(s)).count() as u32;
    }
    b
}

/// Where a road's lamp posts stand (design 42e, 42m): for every
/// `lamp_spacing`th cell of its centre line, a post on the verge to the
/// left of its outermost left cell (and, on a three-wide road, to the right
/// half a spacing on, staggered), on the neighbour of that road cell most
/// to the side that is not the road's, dry, no town's and within a layer of
/// its ground. `(the road cell beside it, the post's cell)`, in road order;
/// none for a road without lamps.
pub fn lamp_posts(
    road: &PlanRoad,
    laid: &[LaidCell],
    ground: &GroundFn,
    cfg: &RoadPlannerConfig,
) -> Vec<(u32, u32)> {
    if !road.lamps {
        return Vec::new();
    }
    let line = road.centre();
    let n = line.len();
    let on_road: BTreeSet<u32> = laid.iter().map(|c| c.key).collect();
    let (Some(ca), Some(cb)) = (
        line.first().and_then(|&k| centre(k)),
        line.last().and_then(|&k| centre(k)),
    ) else {
        return Vec::new();
    };
    let Some(left) = ca.cross(cb).try_normalize() else {
        return Vec::new();
    };
    let sp = cfg.lamp_spacing.max(1) as usize;
    let mut out = Vec::new();
    for (k, &c) in line.iter().enumerate() {
        let sides: &[i8] = if road.width >= 3 && k % sp == 0 && k > 0 && k + 1 < n {
            &[-1]
        } else if k % sp == sp / 2 {
            &[1]
        } else {
            &[]
        };
        for &sign in sides {
            // The road's outermost cell on this side at this centre cell.
            let beside = laid
                .iter()
                .filter(|l| l.centre == c && (l.lane == 0 || l.lane == sign))
                .max_by_key(|l| l.lane * sign)
                .copied();
            let Some(beside) = beside else { continue };
            let Some(bc) = centre(beside.key) else {
                continue;
            };
            let side = left * f32::from(sign);
            let mut verge: Vec<(f32, u32)> = neighbours(beside.key)
                .into_iter()
                .filter(|m| !on_road.contains(m))
                .filter_map(|m| Some(((centre(m)? - bc).dot(side), m)))
                .filter(|(d, _)| *d > 0.0)
                .collect();
            verge.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            let post = verge.into_iter().find_map(|(_, m)| {
                let g = ground(m)?;
                (g.water == Water::Dry && !g.town && (g.top_m - beside.base_m).abs() <= 1)
                    .then_some(m)
            });
            if let Some(post) = post {
                out.push((beside.key, post));
            }
        }
    }
    out
}

/// Insert a stake at `key`, a cell on a leg of a road that is not a stake
/// (design 42f, 42m): it goes between the two stakes of the leg it lies on,
/// a stake taken from the pack, and the road runs through it. `None` where
/// no leg runs over the cell; refused with the first clash on the road.
pub fn insert_stake(
    plans: &BTreeMap<u64, RoadPlan>,
    key: u32,
    ground: &GroundFn,
    tree: &TreeFn,
    cfg: &RoadPlannerConfig,
    ford_depth_m: f32,
) -> Option<Result<Edited, Clash>> {
    if stake_at(plans, key).is_some() {
        return None;
    }
    let mut after = plans.clone();
    let road = after
        .values_mut()
        .flat_map(|p| p.roads.iter_mut())
        .find_map(|road| {
            let leg = road
                .stakes
                .windows(2)
                .position(|w| hex_line(w[0], w[1]).contains(&key))?;
            Some((road, leg))
        });
    let (road, leg) = road?;
    road.stakes.insert(leg + 1, key);
    road.cells = None;
    if let Some((_, why)) = first_clash(&lay(road, ground, tree, cfg, ford_depth_m)) {
        return Some(Err(why));
    }
    let stakes = count_change(plans, &after);
    Some(Ok(Edited {
        plans: after,
        stakes,
    }))
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
