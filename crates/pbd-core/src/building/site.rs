//! Construction sites (`building-planner` design 5, 7, 9 and 11): a plan
//! set out in the world, checked whole against it, built piece by piece,
//! changed mid-build by a diff of piece ids, and kept as records. What is
//! built is derived from the `piece` records: a site's piece is built when a
//! piece of the same id, kit and shape stands.

use super::costs::{CostTable, Goods, Locked, refund};
use super::plan::{Charted, Plan};
use super::planner::{Tunables, check_plan};
use super::rules::{Rule, check_layout, dependant};
use super::{Piece, STOREY, Shape, Site};
use crate::records::{Record, Records};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The player's blueprints (design 9).
pub const BLUEPRINT_RECORD: &str = "blueprint";
pub const BLUEPRINT_SCHEMA: u32 = 1;
/// The player's construction sites (design 9). Not `site`, which is a city
/// site's kind.
pub const SITE_RECORD: &str = "building-site";
pub const SITE_SCHEMA: u32 = 1;

/// A preset's three sizes (design 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum Size {
    Small,
    Standard,
    Large,
}

impl Size {
    pub const ALL: [Size; 3] = [Size::Small, Size::Standard, Size::Large];

    pub fn index(self) -> usize {
        self as usize
    }

    /// Its id in `blueprint.size.*`.
    pub fn id(self) -> &'static str {
        match self {
            Size::Small => "small",
            Size::Standard => "standard",
            Size::Large => "large",
        }
    }
}

/// A `blueprint` record's body: the player's own plan, or none once it is
/// thrown away.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct BlueprintBody {
    /// The player's text, not a locale key.
    pub name: String,
    /// The preset it came from, and its size.
    pub preset: Option<(String, Size)>,
    pub plan: Option<Plan>,
}

/// Where a site stands (design 9): its origin cell, its turn in sixths and
/// its ground storey's layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct SiteChart {
    pub origin: u32,
    pub turn: u8,
    pub base: u16,
}

/// A `building-site` record's body.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct SiteBody {
    /// The plan's name: the player's text, or the preset's locale key.
    pub name: String,
    /// The preset it came from, if one.
    pub preset: Option<(String, Size)>,
    /// The plan, copied in: a site never follows a later edit of the
    /// blueprint it came from.
    pub plan: Plan,
    pub chart: SiteChart,
    /// Built pieces the last change dropped, to take down, by id.
    pub marked: BTreeMap<u64, Piece>,
    /// The storeys with scaffolding standing round them.
    pub scaffold: BTreeSet<u8>,
    /// Whether the site was given up (nothing built stays a site).
    #[serde(default)]
    pub cleared: bool,
}

/// The id after the highest a kind has used: ids are counted up and never
/// reused.
pub fn next_id(records: &Records, kind: &str) -> u64 {
    records
        .of_kind(kind)
        .map(|r| r.id)
        .max()
        .map_or(1, |m| m + 1)
}

/// The record that keeps a blueprint.
pub fn blueprint_record(id: u64, body: &BlueprintBody) -> Record {
    Record::of(BLUEPRINT_RECORD, id, BLUEPRINT_SCHEMA, body)
}

/// The record that keeps a site.
pub fn site_record(id: u64, body: &SiteBody) -> Record {
    Record::of(SITE_RECORD, id, SITE_SCHEMA, body)
}

/// Every blueprint the records hold, by id; one thrown away is left out.
pub fn blueprints(records: &Records) -> BTreeMap<u64, BlueprintBody> {
    records
        .of_kind(BLUEPRINT_RECORD)
        .filter(|r| r.schema == BLUEPRINT_SCHEMA)
        .filter_map(|r| Some((r.id, r.read::<BlueprintBody>()?)))
        .filter(|(_, b)| b.plan.is_some())
        .collect()
}

/// Every site the records hold, by id; a cleared one is left out. A body
/// this build cannot read is left in the records and not shown.
pub fn sites(records: &Records) -> BTreeMap<u64, SiteBody> {
    records
        .of_kind(SITE_RECORD)
        .filter(|r| r.schema == SITE_SCHEMA)
        .filter_map(|r| Some((r.id, r.read::<SiteBody>()?)))
        .filter(|(_, s)| !s.cleared)
        .collect()
}

/// Whether `piece` stands among the world's `pieces`: the same id, kit and
/// shape.
pub fn is_built(pieces: &BTreeMap<u64, Piece>, piece: &Piece) -> bool {
    pieces.get(&piece.id()) == Some(piece)
}

/// A site's state from its charted pieces and the world's: what is built,
/// what is left, and whether it is finished (everything struck and nothing
/// marked; empty furniture slots never keep a site open).
pub struct Progress {
    pub built: usize,
    pub total: usize,
    pub finished: bool,
}

pub fn progress(charted: &Charted, marked: usize, pieces: &BTreeMap<u64, Piece>) -> Progress {
    let struck: Vec<&Piece> = charted
        .iter()
        .map(|(_, p)| p)
        .filter(|p| !matches!(p.shape, Shape::Furniture { .. }))
        .collect();
    let built = struck.iter().filter(|p| is_built(pieces, p)).count();
    Progress {
        built,
        total: struck.len(),
        finished: built == struck.len() && marked == 0,
    }
}

/// Why the world refuses a site (design 5), first failure first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The plan breaks one of the planner's checks.
    Plan,
    /// It does not chart here: off the cells known, or a cell under it a
    /// pentagon.
    Chart,
    /// A pentagon under or beside it.
    Pentagon,
    /// More than `max_footing` layers between its highest and lowest ground.
    Steep,
    /// A solid block, a tree, a road or a town on a block it takes.
    InTheWay,
    /// A piece of its own slot already stands, of another shape or kit.
    Taken,
    /// A rule against the world's pieces.
    Rule(Rule),
}

impl Refusal {
    /// Its reason's key in the locale file.
    pub fn key(self) -> String {
        match self {
            Refusal::Plan => "site.refused.plan".into(),
            Refusal::Chart | Refusal::Pentagon => "site.refused.pentagon".into(),
            Refusal::Steep => "site.refused.steep".into(),
            Refusal::InTheWay => "site.refused.in_the_way".into(),
            Refusal::Taken => "site.refused.taken".into(),
            Refusal::Rule(r) => format!("build.refused.{}", r.id()),
        }
    }
}

/// What the world holds round a site, for [`check_site`].
pub struct World<'a> {
    /// The cells and the ground.
    pub site: &'a Site<'a>,
    /// Whether the block of `cell` at `layer` is taken: a solid block, a
    /// tree, a road cell, a lamp or a town's footprint. Only the block, never
    /// its column (owner, 2026-11-23).
    pub blocked: &'a dyn Fn(u32, u16) -> bool,
    /// The pieces standing near, the site's own built pieces among them.
    pub pieces: &'a [Piece],
}

/// The whole site checked against the world before anything is set out
/// (design 5): the first failure, and the indices of the charted pieces it
/// is on. `base` is the ground storey's layer.
pub fn check_site(
    world: &World,
    charted: &Charted,
    base: u16,
    tun: &Tunables,
    plan: &Plan,
) -> Result<(), (Refusal, Vec<usize>)> {
    if check_plan(plan, tun).iter().any(|i| i.check.refuses()) {
        return Err((Refusal::Plan, Vec::new()));
    }
    let cells = world.site.cells;
    // A pentagon under or beside it.
    let near_pentagon = |c: u32| {
        cells.pentagon(c)
            || cells
                .neighbours(c)
                .iter()
                .flatten()
                .any(|&n| cells.pentagon(n))
    };
    let hit: Vec<usize> = charted
        .iter()
        .enumerate()
        .filter(|(_, (_, p))| p.shape.cells().iter().any(|&c| near_pentagon(c)))
        .map(|(i, _)| i)
        .collect();
    if !hit.is_empty() {
        return Err((Refusal::Pentagon, hit));
    }
    // The ground under the ground floor.
    let ground_cells: Vec<(usize, u32)> = charted
        .iter()
        .enumerate()
        .filter_map(|(i, (_, p))| match p.shape {
            Shape::Floor { cell, layer } if layer == base => Some((i, cell)),
            _ => None,
        })
        .collect();
    let heights: Vec<Option<u16>> = ground_cells
        .iter()
        .map(|&(_, c)| (world.site.ground)(c))
        .collect();
    if heights.iter().any(Option::is_none) {
        return Err((Refusal::Chart, Vec::new()));
    }
    let low = heights.iter().flatten().min().copied().unwrap_or(base);
    let high = heights.iter().flatten().max().copied().unwrap_or(base);
    if high - low > tun.max_footing {
        let hit = ground_cells
            .iter()
            .zip(&heights)
            .filter(|(_, h)| h.is_some_and(|h| high - h > tun.max_footing))
            .map(|((i, _), _)| *i)
            .collect();
        return Err((Refusal::Steep, hit));
    }
    // In the way: only the blocks a piece takes on its storey, so a tree
    // refuses the ground storey's pieces in its own cell and nothing over it.
    let hit: Vec<usize> = charted
        .iter()
        .enumerate()
        .filter(|(_, (_, p))| {
            let layer = p.shape.layer();
            let blocks = |c: u32| (0..STOREY).any(|h| (world.blocked)(c, layer + h));
            match &p.shape {
                // A wall stands on its edge: the blocks either side of it.
                Shape::Wall { .. } | Shape::Door { .. } => {
                    let inside = p.shape.cells();
                    inside.iter().all(|&c| blocks(c))
                }
                Shape::Roof { .. } => false,
                _ => p.shape.cells().iter().any(|&c| blocks(c)),
            }
        })
        .map(|(i, _)| i)
        .collect();
    if !hit.is_empty() {
        return Err((Refusal::InTheWay, hit));
    }
    // Pieces already standing in its slots.
    let standing: BTreeMap<String, &Piece> =
        world.pieces.iter().map(|p| (p.shape.slot(), p)).collect();
    let hit: Vec<usize> = charted
        .iter()
        .enumerate()
        .filter(|(_, (_, p))| standing.get(&p.shape.slot()).is_some_and(|s| *s != p))
        .map(|(i, _)| i)
        .collect();
    if !hit.is_empty() {
        return Err((Refusal::Taken, hit));
    }
    // The rules, with the world's own pieces beside the plan's.
    let mine: BTreeSet<u64> = charted.iter().map(|(_, p)| p.id()).collect();
    let mut all: Vec<Piece> = charted.iter().map(|(_, p)| p.clone()).collect();
    all.extend(
        world
            .pieces
            .iter()
            .filter(|p| !mine.contains(&p.id()))
            .cloned(),
    );
    if let Err((i, rule)) = check_layout(world.site, &all) {
        let hit = if i < charted.len() {
            vec![i]
        } else {
            Vec::new()
        };
        return Err((Refusal::Rule(rule), hit));
    }
    Ok(())
}

/// What changing a site's plan does to it (design 7), piece by piece by id.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Diff {
    /// In both, the same: built stays built, a ghost stays a ghost.
    pub unchanged: Vec<Piece>,
    /// Only in the new plan (or the new kit's piece of a swapped slot).
    pub new: Vec<Piece>,
    /// Only in the old plan and not built: gone.
    pub gone: Vec<Piece>,
    /// Only in the old plan (or swapped) and built: to take down.
    pub marked: Vec<Piece>,
    /// The goods taken-down pieces give back, at `refund_share`.
    pub refund: BTreeMap<String, u32>,
}

/// Diff an old charted plan against a new one where the site stands
/// (design 7). A slot in both whose piece changed (a kit swapped, a wall
/// given a door) has its old piece marked if built, and the new one is a
/// ghost.
pub fn diff(
    old: &[Piece],
    new: &[Piece],
    pieces: &BTreeMap<u64, Piece>,
    costs: &CostTable,
    tun: &Tunables,
) -> Diff {
    let olds: BTreeMap<u64, &Piece> = old.iter().map(|p| (p.id(), p)).collect();
    let news: BTreeMap<u64, &Piece> = new.iter().map(|p| (p.id(), p)).collect();
    let mut out = Diff::default();
    for p in new {
        match olds.get(&p.id()) {
            Some(o) if *o == p => out.unchanged.push(p.clone()),
            _ => out.new.push(p.clone()),
        }
    }
    for o in old {
        if news.get(&o.id()).is_some_and(|n| *n == o) {
            continue;
        }
        if is_built(pieces, o) {
            out.marked.push(o.clone());
            for (g, n) in refund(&cost_or_none(costs, o, tun), tun.refund_share) {
                *out.refund.entry(g).or_insert(0) += n;
            }
        } else {
            out.gone.push(o.clone());
        }
    }
    out
}

fn cost_or_none(costs: &CostTable, piece: &Piece, tun: &Tunables) -> Goods {
    costs.cost_of(piece, tun.cost_scale).unwrap_or_default()
}

/// The next marked piece to take down: one nothing else standing rests on,
/// the highest first (design 7, B2's removal).
pub fn next_down<'a>(marked: &'a BTreeMap<u64, Piece>, standing: &[Piece]) -> Option<&'a Piece> {
    let mut list: Vec<&Piece> = marked.values().collect();
    list.sort_by_key(|p| std::cmp::Reverse(p.shape.layer()));
    list.into_iter().find(|p| dependant(standing, p).is_none())
}

/// What a site's goods still need: the bill of what is not built.
pub fn still_needed(
    charted: &Charted,
    pieces: &BTreeMap<u64, Piece>,
    costs: &CostTable,
    tun: &Tunables,
) -> Result<BTreeMap<String, u32>, Locked> {
    costs.bill(
        charted
            .iter()
            .map(|(_, p)| p)
            .filter(|p| !is_built(pieces, p)),
        tun.cost_scale,
    )
}

/// The world's building setting (design 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum BuildingSetting {
    /// The planner, presets, sites and costs; no single piece outside a site.
    Planned,
    /// Single pieces and whole blueprints at no cost.
    Free,
}

impl BuildingSetting {
    /// The setting a world names, or Free for a world made before there was
    /// one: that is how those worlds were built.
    pub fn of_saved(saved: Option<BuildingSetting>) -> Self {
        saved.unwrap_or(BuildingSetting::Free)
    }

    /// What a world this build makes is set to.
    pub const NEW_WORLD: BuildingSetting = BuildingSetting::Planned;
}

/// The scaffolding round a storey (design 6): the bays, one outside each
/// outer edge of the storey, as `(cell, side)` in plan coordinates.
pub fn scaffold_bays(plan: &Plan, storey: u8) -> Vec<(super::plan::Hex, u8)> {
    let Some(cells) = plan.storeys.get(usize::from(storey)) else {
        return Vec::new();
    };
    cells
        .iter()
        .flat_map(|&c| (0..6u8).map(move |d| (c, d)))
        .filter(|&(c, d)| !cells.contains(&super::plan::step(c, d)))
        .collect()
}

/// What scaffolding of `bays` costs.
pub fn scaffold_cost(bays: usize, tun: &Tunables) -> Goods {
    let bays = bays as u32;
    vec![
        ("planks".into(), tun.scaffold_cost.planks_per_bay * bays),
        (
            "rope".into(),
            bays.div_ceil(tun.scaffold_cost.bays_per_rope),
        ),
    ]
}

/// Whether a storey wants scaffolding: it is above the ground, the walls of
/// the storey below are all built, and its own walls are not.
pub fn wants_scaffold(
    charted: &Charted,
    storey: u8,
    pieces: &BTreeMap<u64, Piece>,
    tun: &Tunables,
) -> bool {
    if !tun.scaffold || storey == 0 {
        return false;
    }
    let walls_done = |s: u8| {
        charted
            .iter()
            .filter(|(pp, p)| pp.storey() == s && matches!(p.shape, Shape::Wall { .. }))
            .all(|(_, p)| is_built(pieces, p))
    };
    walls_done(storey - 1) && !walls_done(storey)
}

/// The ground storey's cells along its outside, in order round the room
/// (design 5's lime line): each cell with an outer side, by its angle about
/// the room's middle.
pub fn outline(plan: &Plan) -> Vec<super::plan::Hex> {
    let Some(cells) = plan.storeys.first() else {
        return Vec::new();
    };
    let n = cells.len().max(1) as f32;
    let (cx, cz) = cells.iter().fold((0.0, 0.0), |(x, z), &c| {
        let (a, b) = super::plan::middle(c);
        (x + a / n, z + b / n)
    });
    let mut out: Vec<(f32, super::plan::Hex)> = cells
        .iter()
        .copied()
        .filter(|&c| (0..6u8).any(|d| !cells.contains(&super::plan::step(c, d))))
        .map(|c| {
            let (x, z) = super::plan::middle(c);
            ((z - cz).atan2(x - cx), c)
        })
        .collect();
    out.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    out.into_iter().map(|(_, c)| c).collect()
}

/// Where the survey stakes stand (design 5): outside each corner of the
/// ground storey, in the cell beyond a corner cell's middle outer side.
pub fn stake_spots(plan: &Plan) -> Vec<(super::plan::Hex, u8)> {
    let Some(cells) = plan.storeys.first() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for &c in cells {
        let outer: Vec<u8> = (0..6u8)
            .filter(|&d| !cells.contains(&super::plan::step(c, d)))
            .collect();
        if outer.len() >= 3 || cells.len() == 1 {
            out.push((c, outer[outer.len() / 2]));
        }
    }
    out
}
