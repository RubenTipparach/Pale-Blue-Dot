//! A plan (`building-planner` design 1): a building drawn at a table, on a
//! flat hex grid of its own, before it has a place.
//!
//! Cells are axial `(q, r)` from the plan's origin, sides `0..5` are the
//! settlement templates' directions ([`crate::settlement::DIRS`]), storeys
//! count up from the ground. Rooms are painted a storey at a time; their
//! outside edges are walls by themselves ([`Plan::pieces`]), so a plan has no
//! missing wall by accident. [`chart_plan`] lays a plan onto the planet's
//! real cells at an origin, a turn in sixths and a base layer, through the
//! templates' own charting, and gives each plan piece the [`Piece`] build
//! mode would place there.

use super::{Opening, Piece, RoofShape, STOREY, Shape};
use crate::furniture::{Kind, MIDDLE};
use crate::settlement::chart::{Patch, chart_reach};
use crate::settlement::{DIRS, offset};
use crate::topology::DualCell;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// An axial cell of a plan.
pub type Hex = (i32, i32);

/// The cell across side `d` of `c`.
pub fn step(c: Hex, d: u8) -> Hex {
    let (dq, dr) = DIRS[usize::from(d % 6)];
    (c.0 + dq, c.1 + dr)
}

/// How many cells `c` is from the origin.
pub fn distance(c: Hex) -> i32 {
    c.0.abs().max(c.1.abs()).max((c.0 + c.1).abs())
}

/// `c` turned `t` sixths about the origin, the way side `d` becomes `d + t`.
pub fn turned(c: Hex, t: u8) -> Hex {
    (0..t % 6).fold(c, |(q, r), _| (-r, q + r))
}

/// A plan cell's middle on the drawing, in cell radii: `x` east, `z` toward
/// the front (the mockup's `hx`, `hz`).
pub fn middle(c: Hex) -> (f32, f32) {
    let w = 3f32.sqrt();
    (w * (c.0 as f32 + c.1 as f32 / 2.0), 1.5 * c.1 as f32)
}

/// Corner `k` of a plan cell on the drawing: side `d` runs from corner `d`
/// to corner `d + 1`.
pub fn corner(c: Hex, k: u8) -> (f32, f32) {
    let (x, z) = middle(c);
    let a = (60.0 * f32::from(k % 6) - 30.0).to_radians();
    (x + a.cos(), z + a.sin())
}

/// What a plan edge carries beyond what the room gives it: a wall on an
/// inside edge, or a door or window in a wall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum Mark {
    Wall,
    Door,
    Window,
}

/// An edge of a storey: the cell on its lower side and the side, so the
/// same edge from either cell is one key.
pub type Edge = (u8, Hex, u8);

/// The one key of the edge across side `d` of `c` on `storey`.
pub fn edge(storey: u8, c: Hex, d: u8) -> Edge {
    let n = step(c, d);
    if c < n {
        (storey, c, d % 6)
    } else {
        (storey, n, (d + 3) % 6)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PlanStair {
    pub foot: Hex,
    /// The head is the foot's neighbour across this side.
    pub side: u8,
    pub storey: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PlanLantern {
    pub cell: Hex,
    pub storey: u8,
}

/// A furniture slot (design 1): `side` 0 to 5 for its back to that side, or
/// [`MIDDLE`], exactly as `Shape::Furniture` stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PlanSlot {
    pub cell: Hex,
    pub storey: u8,
    pub side: u8,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub struct PlanPost {
    pub cell: Hex,
    pub corner: u8,
    pub storey: u8,
}

/// A plan.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Plan {
    /// The kit it is built of (`kits.ron`).
    pub kit: String,
    /// The top roof's shape.
    pub roof: RoofShape,
    /// Each storey's cells, the ground first.
    pub storeys: Vec<BTreeSet<Hex>>,
    /// What each marked edge carries.
    pub edges: BTreeMap<Edge, Mark>,
    pub stairs: Vec<PlanStair>,
    pub lanterns: Vec<PlanLantern>,
    pub slots: Vec<PlanSlot>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub posts: Vec<PlanPost>,
}

/// A piece of a plan, in plan coordinates (design 1).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlanPiece {
    Floor {
        cell: Hex,
        storey: u8,
    },
    Wall {
        cell: Hex,
        side: u8,
        storey: u8,
        opening: Opening,
        /// On the room's outside.
        outer: bool,
    },
    /// A doorway's door, hung with its wall.
    Door {
        cell: Hex,
        side: u8,
        storey: u8,
    },
    Stair(PlanStair),
    Post(PlanPost),
    Lantern(PlanLantern),
    Roof {
        storey: u8,
        cells: Vec<Hex>,
        shape: RoofShape,
    },
    Slot(PlanSlot),
}

impl PlanPiece {
    /// The storey it is on.
    pub fn storey(&self) -> u8 {
        match self {
            PlanPiece::Floor { storey, .. }
            | PlanPiece::Wall { storey, .. }
            | PlanPiece::Door { storey, .. }
            | PlanPiece::Roof { storey, .. } => *storey,
            PlanPiece::Stair(s) => s.storey,
            PlanPiece::Post(p) => p.storey,
            PlanPiece::Lantern(l) => l.storey,
            PlanPiece::Slot(s) => s.storey,
        }
    }

    /// The plan cells it touches: a wall's both sides, a stair's foot and
    /// head.
    pub fn cells(&self) -> Vec<Hex> {
        match self {
            PlanPiece::Floor { cell, .. } => vec![*cell],
            PlanPiece::Wall { cell, side, .. } | PlanPiece::Door { cell, side, .. } => {
                vec![*cell, step(*cell, *side)]
            }
            PlanPiece::Stair(s) => vec![s.foot, step(s.foot, s.side)],
            PlanPiece::Post(p) => vec![p.cell],
            PlanPiece::Lantern(l) => vec![l.cell],
            PlanPiece::Roof { cells, .. } => cells.clone(),
            PlanPiece::Slot(s) => vec![s.cell],
        }
    }
}

impl Plan {
    /// An empty plan in a kit.
    pub fn new(kit: &str) -> Self {
        Self {
            kit: kit.into(),
            roof: RoofShape::Gable,
            storeys: vec![BTreeSet::new()],
            edges: BTreeMap::new(),
            stairs: Vec::new(),
            lanterns: Vec::new(),
            slots: Vec::new(),
            posts: Vec::new(),
        }
    }

    /// Every cell of every storey.
    pub fn footprint(&self) -> BTreeSet<Hex> {
        self.storeys.iter().flatten().copied().collect()
    }

    /// Whether nothing is painted.
    pub fn is_empty(&self) -> bool {
        self.storeys.iter().all(BTreeSet::is_empty)
    }

    /// The cells of `storey` a stair from the storey below comes up through:
    /// left unfloored.
    pub fn wells(&self, storey: u8) -> BTreeSet<Hex> {
        if storey == 0 {
            return BTreeSet::new();
        }
        self.stairs
            .iter()
            .filter(|s| s.storey + 1 == storey)
            .flat_map(|s| [s.foot, step(s.foot, s.side)])
            .collect()
    }

    /// The plan's pieces, a storey at a time (floors, then walls with each
    /// doorway's door after it), then stairs, posts, lanterns, roofs and the
    /// furniture slots.
    pub fn pieces(&self) -> Vec<PlanPiece> {
        let mut out = Vec::new();
        for (s, cells) in self.storeys.iter().enumerate() {
            let s = s as u8;
            let wells = self.wells(s);
            for &cell in cells {
                if !wells.contains(&cell) {
                    out.push(PlanPiece::Floor { cell, storey: s });
                }
            }
            let mut seen = BTreeSet::new();
            for &cell in cells {
                for d in 0..6u8 {
                    let key = edge(s, cell, d);
                    if !seen.insert(key) {
                        continue;
                    }
                    let outer = !cells.contains(&step(cell, d));
                    let mark = self.edges.get(&key).copied();
                    let opening = match (outer, mark) {
                        (true, Some(Mark::Door)) | (false, Some(Mark::Door)) => Opening::Door,
                        (true, Some(Mark::Window)) => Opening::Window,
                        (true, _) | (false, Some(Mark::Wall)) => Opening::None,
                        (false, _) => continue,
                    };
                    out.push(PlanPiece::Wall {
                        cell,
                        side: d,
                        storey: s,
                        opening,
                        outer,
                    });
                    if opening == Opening::Door {
                        out.push(PlanPiece::Door {
                            cell,
                            side: d,
                            storey: s,
                        });
                    }
                }
            }
        }
        out.extend(self.stairs.iter().copied().map(PlanPiece::Stair));
        out.extend(self.posts.iter().copied().map(PlanPiece::Post));
        out.extend(self.lanterns.iter().copied().map(PlanPiece::Lantern));
        let top = self.storeys.len().saturating_sub(1);
        for (s, cells) in self.storeys.iter().enumerate() {
            let open: Vec<Hex> = cells
                .iter()
                .copied()
                .filter(|c| s == top || !self.storeys[s + 1].contains(c))
                .collect();
            if open.is_empty() {
                continue;
            }
            let shape = if s < top {
                RoofShape::Flat
            } else if open.len() == 1 {
                RoofShape::Cone
            } else if self.roof == RoofShape::Cone {
                RoofShape::Gable
            } else {
                self.roof
            };
            out.push(PlanPiece::Roof {
                storey: s as u8,
                cells: open,
                shape,
            });
        }
        out.extend(self.slots.iter().copied().map(PlanPiece::Slot));
        out
    }

    /// The plan turned `t` sixths about its origin.
    pub fn turned(&self, t: u8) -> Plan {
        let cell = |c: Hex| turned(c, t);
        let side = |d: u8| (d + t) % 6;
        Plan {
            kit: self.kit.clone(),
            roof: self.roof,
            storeys: self
                .storeys
                .iter()
                .map(|s| s.iter().map(|&c| cell(c)).collect())
                .collect(),
            edges: self
                .edges
                .iter()
                .map(|(&(s, c, d), &m)| (edge(s, cell(c), side(d)), m))
                .collect(),
            stairs: self
                .stairs
                .iter()
                .map(|s| PlanStair {
                    foot: cell(s.foot),
                    side: side(s.side),
                    storey: s.storey,
                })
                .collect(),
            lanterns: self
                .lanterns
                .iter()
                .map(|l| PlanLantern {
                    cell: cell(l.cell),
                    storey: l.storey,
                })
                .collect(),
            slots: self
                .slots
                .iter()
                .map(|o| PlanSlot {
                    cell: cell(o.cell),
                    side: if o.side == MIDDLE {
                        MIDDLE
                    } else {
                        side(o.side)
                    },
                    ..*o
                })
                .collect(),
            posts: self
                .posts
                .iter()
                .map(|p| PlanPost {
                    cell: cell(p.cell),
                    corner: side(p.corner),
                    storey: p.storey,
                })
                .collect(),
        }
    }

    /// Drop stairs, lanterns, slots and posts whose cells are no longer in
    /// their storey, and marks on edges no longer the room's.
    pub fn prune(&mut self) {
        let storeys = self.storeys.clone();
        let has = |s: u8, c: Hex| storeys.get(usize::from(s)).is_some_and(|x| x.contains(&c));
        self.stairs
            .retain(|t| usize::from(t.storey) < storeys.len());
        self.lanterns.retain(|l| has(l.storey, l.cell));
        self.slots.retain(|o| has(o.storey, o.cell));
        self.posts.retain(|p| has(p.storey, p.cell));
        self.edges
            .retain(|&(s, c, d), _| has(s, c) || has(s, step(c, d)));
    }
}

/// What a planner tool does where it is aimed (design 3 and M6 version 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    /// Paint a cell into the storey's room, or take it out.
    Room,
    Wall,
    Door,
    Window,
    Stair,
    Lantern,
    Slot(Kind),
    Erase,
}

/// Where a planner tool is aimed: a cell, the side its pointer leans to,
/// and whether the pointer is near an edge (for Erase).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aim {
    pub cell: Hex,
    pub side: u8,
    pub at_edge: bool,
}

/// Why a tool did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoUse {
    /// Paint a room here first.
    NoRoom,
    /// A window goes in an outside wall.
    InsideWindow,
    /// Past the plan's reach.
    TooFar,
}

impl NoUse {
    /// Its key in the locale file.
    pub fn key(self) -> &'static str {
        match self {
            NoUse::NoRoom => "planner.hint.no_room",
            NoUse::InsideWindow => "planner.hint.inside_window",
            NoUse::TooFar => "planner.check.plan_size",
        }
    }
}

impl Tool {
    /// Whether the tool works on edges.
    pub fn on_edge(self, aim: &Aim) -> bool {
        matches!(self, Tool::Wall | Tool::Door | Tool::Window)
            || (self == Tool::Erase && aim.at_edge)
    }
}

impl Plan {
    /// The aim a tool acts on: an edge tool aimed at a room's outer edge
    /// from the cell beyond it acts on the room's own edge (M6 version 3).
    pub fn resolve(&self, tool: Tool, storey: u8, aim: Aim) -> Aim {
        let Some(cells) = self.storeys.get(usize::from(storey)) else {
            return aim;
        };
        if tool.on_edge(&aim) && !cells.contains(&aim.cell) {
            let other = step(aim.cell, aim.side);
            if cells.contains(&other) {
                return Aim {
                    cell: other,
                    side: (aim.side + 3) % 6,
                    at_edge: true,
                };
            }
        }
        aim
    }

    /// Use `tool` at `aim` on `storey`. `painting` is the room tool's
    /// stroke: `Some(true)` adds, `Some(false)` takes out, `None` decides
    /// from the cell (a fresh press). Returns what the stroke does next.
    pub fn apply(
        &mut self,
        tool: Tool,
        storey: u8,
        aim: Aim,
        painting: Option<bool>,
        max_radius: i32,
    ) -> Result<Option<bool>, NoUse> {
        let aim = self.resolve(tool, storey, aim);
        let s = usize::from(storey);
        if s >= self.storeys.len() {
            return Err(NoUse::NoRoom);
        }
        let cell = aim.cell;
        let has = self.storeys[s].contains(&cell);
        match tool {
            Tool::Room => {
                let add = painting.unwrap_or(!has);
                if add {
                    if distance(cell) > max_radius {
                        return Err(NoUse::TooFar);
                    }
                    self.storeys[s].insert(cell);
                } else {
                    self.storeys[s].remove(&cell);
                }
                self.prune();
                Ok(Some(add))
            }
            Tool::Wall | Tool::Door | Tool::Window => {
                if !has {
                    return Err(NoUse::NoRoom);
                }
                let key = edge(storey, cell, aim.side);
                let outer = !self.storeys[s].contains(&step(cell, aim.side));
                let now = self.edges.get(&key).copied();
                let next = match tool {
                    Tool::Wall if outer => None,
                    Tool::Wall => (now != Some(Mark::Wall)).then_some(Mark::Wall),
                    Tool::Door if now == Some(Mark::Door) => (!outer).then_some(Mark::Wall),
                    Tool::Door => Some(Mark::Door),
                    _ if !outer => return Err(NoUse::InsideWindow),
                    _ => (now != Some(Mark::Window)).then_some(Mark::Window),
                };
                match next {
                    Some(m) => self.edges.insert(key, m),
                    None => self.edges.remove(&key),
                };
                Ok(None)
            }
            Tool::Stair => {
                let here = self
                    .stairs
                    .iter()
                    .position(|t| t.storey == storey && t.foot == cell);
                match here {
                    Some(i) if self.stairs[i].side == aim.side => {
                        self.stairs.remove(i);
                    }
                    Some(i) => self.stairs[i].side = aim.side,
                    None => self.stairs.push(PlanStair {
                        foot: cell,
                        side: aim.side,
                        storey,
                    }),
                }
                Ok(None)
            }
            Tool::Lantern => {
                if !has {
                    return Err(NoUse::NoRoom);
                }
                match self
                    .lanterns
                    .iter()
                    .position(|l| l.storey == storey && l.cell == cell)
                {
                    Some(i) => {
                        self.lanterns.remove(i);
                    }
                    None => self.lanterns.push(PlanLantern { cell, storey }),
                }
                Ok(None)
            }
            Tool::Slot(kind) => {
                if !has {
                    return Err(NoUse::NoRoom);
                }
                match self
                    .slots
                    .iter()
                    .position(|o| o.storey == storey && o.cell == cell)
                {
                    Some(i) => {
                        self.slots.remove(i);
                    }
                    None => self.slots.push(PlanSlot {
                        cell,
                        storey,
                        // Facing the way pointed: its back to the side behind.
                        side: if kind == Kind::Table {
                            MIDDLE
                        } else {
                            (aim.side + 3) % 6
                        },
                        kind,
                    }),
                }
                Ok(None)
            }
            Tool::Erase => {
                if aim.at_edge {
                    self.edges.remove(&edge(storey, cell, aim.side));
                    return Ok(None);
                }
                let before = self.stairs.len() + self.lanterns.len() + self.slots.len();
                self.stairs
                    .retain(|t| !(t.storey == storey && t.foot == cell));
                self.lanterns
                    .retain(|l| !(l.storey == storey && l.cell == cell));
                self.slots
                    .retain(|o| !(o.storey == storey && o.cell == cell));
                if before == self.stairs.len() + self.lanterns.len() + self.slots.len() {
                    if !has {
                        return Err(NoUse::NoRoom);
                    }
                    self.storeys[s].remove(&cell);
                    self.prune();
                }
                Ok(None)
            }
        }
    }

    /// Add a storey on top, the same room as the one below.
    pub fn add_storey(&mut self) {
        let top = self.storeys.last().cloned().unwrap_or_default();
        self.storeys.push(top);
    }

    /// Take the top storey off (never the ground).
    pub fn drop_storey(&mut self) {
        if self.storeys.len() > 1 {
            self.storeys.pop();
            let n = self.storeys.len() as u8;
            self.stairs.retain(|t| t.storey + 1 < n);
            self.edges.retain(|&(s, _, _), _| s < n);
            self.prune();
        }
    }
}

/// A plan laid onto real cells: each plan piece and its piece.
pub type Charted = Vec<(PlanPiece, Piece)>;

/// The layer a storey's floor stands on, over a base.
pub fn layer_of(base: u16, storey: u8) -> u16 {
    base + u16::from(storey) * STOREY
}

/// Lay `plan` onto `patch`: its origin on patch cell `at`, its side 0 on
/// that cell's side `turn` (a sixth of a turn each), its ground storey on
/// `base` (design 1). Every plan cell must chart onto a hexagon of the
/// patch; the first that does not is named.
pub fn chart_plan(
    patch: &Patch,
    plan: &Plan,
    at: usize,
    turn: usize,
    base: u16,
) -> Result<Charted, String> {
    let footprint = plan.footprint();
    let reach = footprint.iter().map(|&c| distance(c)).max().unwrap_or(0);
    let cells = PlanCells::new(patch, at, turn, reach);
    let index = |c: Hex| cells.index(c);
    let key = |c: Hex| cells.key(c);
    let side_of = |c: Hex, d: u8| cells.side(c, d);
    let across = |c: Hex, d: u8| cells.across(c, d);
    for &c in &footprint {
        let i = index(c).ok_or_else(|| format!("plan cell {c:?} does not chart"))?;
        if patch.cells[i].corners.len() != 6 {
            return Err(format!("plan cell {c:?} is on a pentagon"));
        }
    }
    let k = plan.kit.as_str();
    let mut out = Vec::new();
    for p in plan.pieces() {
        let layer = layer_of(base, p.storey());
        let shape = match &p {
            PlanPiece::Floor { cell, .. } => Shape::Floor {
                cell: key(*cell)?,
                layer,
            },
            PlanPiece::Wall {
                cell,
                side,
                opening,
                ..
            } => Shape::wall(key(*cell)?, across(*cell, *side)?, layer, *opening),
            PlanPiece::Door { cell, side, .. } => {
                Shape::door(key(*cell)?, across(*cell, *side)?, layer)
            }
            PlanPiece::Stair(s) => Shape::Stair {
                foot: key(s.foot)?,
                head: across(s.foot, s.side)?,
                layer,
            },
            PlanPiece::Post(post) => {
                let here = key(post.cell)?;
                // Corner k of a plan cell sits between its sides k - 1 and
                // k: the cell and the two across them.
                let a = across(post.cell, (post.corner + 5) % 6)?;
                let b = across(post.cell, post.corner % 6)?;
                Shape::post([here, a, b], layer)
            }
            PlanPiece::Lantern(l) => Shape::Lantern {
                cell: key(l.cell)?,
                layer,
            },
            PlanPiece::Roof { cells, shape, .. } => Shape::roof(
                cells.iter().map(|&c| key(c)).collect::<Result<_, _>>()?,
                layer + STOREY,
                *shape,
            ),
            PlanPiece::Slot(o) => Shape::Furniture {
                cell: key(o.cell)?,
                layer,
                side: if o.side == MIDDLE {
                    MIDDLE
                } else {
                    side_of(o.cell, o.side)? as u8
                },
                kind: o.kind,
            },
        };
        out.push((p, Piece::new(k, shape)));
    }
    Ok(out)
}

/// The plan's cells laid on a patch (design 1): which patch cell each plan
/// cell is on, and which of its sides each plan side is.
pub struct PlanCells<'a> {
    patch: &'a Patch,
    chart: crate::settlement::chart::Chart,
}

impl<'a> PlanCells<'a> {
    /// The plan cells within `reach` of the origin, on patch cell `at` with
    /// plan side 0 on its side `turn`.
    pub fn new(patch: &'a Patch, at: usize, turn: usize, reach: i32) -> Self {
        let disc: BTreeSet<(i32, i32)> = (-reach..=reach)
            .flat_map(|q| (-reach..=reach).map(move |r| (q, r)))
            .filter(|&c| distance(c) <= reach)
            .map(|(q, r)| offset(q, r))
            .collect();
        Self {
            patch,
            chart: chart_reach(patch, (0, 0), at, turn % 6, &disc),
        }
    }

    /// The patch index a plan cell is on.
    pub fn index(&self, c: Hex) -> Option<usize> {
        let (oc, or) = offset(c.0, c.1);
        self.chart.cell(oc, or)
    }

    /// The key of the cell a plan cell is on.
    pub fn key(&self, c: Hex) -> Result<u32, String> {
        self.index(c)
            .and_then(|i| self.patch.keys.get(i).copied())
            .ok_or_else(|| format!("plan cell {c:?} does not chart"))
    }

    /// The patch side plan side `d` of `c` is.
    pub fn side(&self, c: Hex, d: u8) -> Result<usize, String> {
        let (oc, or) = offset(c.0, c.1);
        self.chart
            .side(oc, or, usize::from(d))
            .ok_or_else(|| format!("plan cell {c:?} does not chart"))
    }

    /// The key across plan side `d` of `c`, in or out of the plan.
    pub fn across(&self, c: Hex, d: u8) -> Result<u32, String> {
        if let Some(i) = self.index(step(c, d)) {
            return self
                .patch
                .keys
                .get(i)
                .copied()
                .ok_or_else(|| format!("{c:?} side {d} is off the patch"));
        }
        let here = self
            .index(c)
            .ok_or_else(|| format!("plan cell {c:?} does not chart"))?;
        let side = self.side(c, d)?;
        self.patch.cells[here]
            .neighbors
            .get(side)
            .and_then(|&n| self.patch.keys.get(n).copied())
            .ok_or_else(|| format!("{c:?} side {d} is off the patch"))
    }

    /// Every plan cell charted, by the key it is on.
    pub fn by_key(&self) -> BTreeMap<u32, Hex> {
        self.chart
            .cells
            .iter()
            .filter_map(|(&(oc, or), at)| {
                let hex = crate::settlement::axial(oc, or);
                Some((*self.patch.keys.get(at.cell)?, hex))
            })
            .collect()
    }
}

/// A plan of pieces already standing (design 7, "adopting a house"): read
/// in plan coordinates from patch cell `at`, turn 0, its ground storey on
/// `base`. Floors make the rooms; doors, windows and inner walls mark their
/// edges; stairs, lanterns, furniture and posts are kept; the top roof's
/// shape is the plan's. Pieces off the chart are left out.
pub fn adopt(patch: &Patch, at: usize, base: u16, pieces: &[Piece]) -> Plan {
    let cells = PlanCells::new(patch, at, 0, 8);
    let hex = cells.by_key();
    let storey = |layer: u16| -> Option<u8> {
        (layer >= base && (layer - base).is_multiple_of(STOREY))
            .then(|| ((layer - base) / STOREY) as u8)
    };
    let side_to = |a: Hex, b: Hex| (0..6u8).find(|&d| step(a, d) == b);
    let kit = pieces
        .first()
        .map_or_else(|| "timber".to_string(), |p| p.kit.clone());
    let mut plan = Plan::new(&kit);
    let grow = |plan: &mut Plan, s: u8| {
        while plan.storeys.len() <= usize::from(s) {
            plan.storeys.push(BTreeSet::new());
        }
    };
    for p in pieces {
        if let Shape::Floor { cell, layer } = p.shape
            && let (Some(&h), Some(s)) = (hex.get(&cell), storey(layer))
        {
            grow(&mut plan, s);
            plan.storeys[usize::from(s)].insert(h);
        }
    }
    let mut top_roof: Option<(u16, RoofShape)> = None;
    for p in pieces {
        match p.shape {
            Shape::Stair { foot, head, layer } => {
                if let (Some(&f), Some(&h), Some(s)) =
                    (hex.get(&foot), hex.get(&head), storey(layer))
                    && let Some(d) = side_to(f, h)
                {
                    grow(&mut plan, s + 1);
                    // The well's cells are the storey above's though unfloored.
                    plan.storeys[usize::from(s) + 1].extend([f, h]);
                    plan.stairs.push(PlanStair {
                        foot: f,
                        side: d,
                        storey: s,
                    });
                }
            }
            Shape::Wall {
                a,
                b,
                layer,
                opening,
            } => {
                let (Some(&ha), Some(&hb), Some(s)) = (hex.get(&a), hex.get(&b), storey(layer))
                else {
                    continue;
                };
                let Some(d) = side_to(ha, hb) else {
                    continue;
                };
                let room = plan.storeys.get(usize::from(s));
                let inner = room.is_some_and(|r| r.contains(&ha) && r.contains(&hb));
                let mark = match opening {
                    Opening::Door => Some(Mark::Door),
                    Opening::Window => Some(Mark::Window),
                    Opening::None if inner => Some(Mark::Wall),
                    Opening::None => None,
                };
                if let Some(m) = mark {
                    plan.edges.insert(edge(s, ha, d), m);
                }
            }
            Shape::Lantern { cell, layer } => {
                if let (Some(&h), Some(s)) = (hex.get(&cell), storey(layer)) {
                    plan.lanterns.push(PlanLantern { cell: h, storey: s });
                }
            }
            Shape::Furniture {
                cell,
                layer,
                side,
                kind,
            } => {
                if let (Some(&h), Some(s)) = (hex.get(&cell), storey(layer)) {
                    let plan_side = if side == MIDDLE {
                        Some(MIDDLE)
                    } else {
                        (0..6u8).find(|&d| cells.side(h, d).ok() == Some(usize::from(side)))
                    };
                    if let Some(side) = plan_side {
                        plan.slots.push(PlanSlot {
                            cell: h,
                            storey: s,
                            side,
                            kind,
                        });
                    }
                }
            }
            Shape::Roof { layer, shape, .. } if top_roof.is_none_or(|(l, _)| layer > l) => {
                top_roof = Some((layer, shape));
            }
            _ => {}
        }
    }
    if let Some((_, shape)) = top_roof {
        plan.roof = shape;
    }
    plan.stairs.sort();
    plan.stairs.dedup();
    plan
}

/// The keys of [`flat_patch`] start here, clear of any real cell's.
pub const FLAT_KEY: u32 = 0xE000_0000;

/// A flat hex disc of `radius` cells round the origin (design 2's stand-in
/// patch): no pentagons, every neighbour and corner exact, its cells at the
/// real tile width on a body of the gold standard's radius. Cell `i` is key
/// `FLAT_KEY + i`; cell 0 is the origin, side 0 of every cell is plan side 0.
pub fn flat_patch(radius: i32) -> Patch {
    let cells: Vec<Hex> = std::iter::once((0, 0))
        .chain(
            (-radius..=radius)
                .flat_map(|q| (-radius..=radius).map(move |r| (q, r)))
                .filter(|&c| c != (0, 0) && distance(c) <= radius),
        )
        .collect();
    let index: BTreeMap<Hex, usize> = cells.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    // Tile width 2.833 m on a 300 m body: a cell radius of 1.636 m.
    const CELL_M: f32 = 1.636;
    const BODY_M: f32 = 300.0;
    let lift = |(x, z): (f32, f32)| Vec3::new(x * CELL_M, BODY_M, z * CELL_M).normalize();
    // The charting walk turns plan side d onto patch side (d0 - d): with d0
    // 0 everywhere, patch side s is plan side -s, and its edge runs from
    // the drawing's corner 1 - s to corner -s (counter-clockwise seen from
    // above, as a real cell's sides run).
    let patch_cells = cells
        .iter()
        .map(|&c| DualCell {
            direction: lift(middle(c)),
            corners: (0..6u8).map(|s| lift(corner(c, (7 - s) % 6))).collect(),
            neighbors: (0..6u8)
                .map(|s| {
                    index
                        .get(&step(c, (6 - s) % 6))
                        .copied()
                        .unwrap_or(usize::MAX)
                })
                .collect(),
        })
        .collect();
    Patch {
        cells: patch_cells,
        keys: (0..cells.len() as u32).map(|i| FLAT_KEY + i).collect(),
    }
}
