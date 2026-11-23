//! The player builds (`player-building`, design "As built in phase 3"): the
//! town's own pieces, put on the planet's real cells one at a time, checked
//! by one set of rules ([`rules`]), cut into the same meshes and solids a
//! town's buildings are ([`cut`]), and kept as `piece` records in the
//! world's journal.
//!
//! A piece names cells by their exact key and layers by the column's own
//! index (counted from `column::BASE_M`, which never moves), so a saved piece
//! means the same place in every build. An edge is the pair of cells either
//! side of it, sorted, so the same edge seen from either cell is one slot.

pub mod blueprint;
pub mod cut;
pub mod rules;

#[cfg(test)]
mod tests;

use crate::records::Record;
use crate::settlement::chart::Patch;
use crate::topology::DualCell;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// The record kind a piece is kept as, and its schema.
pub const PIECE_RECORD: &str = "piece";
pub const PIECE_SCHEMA: u32 = 1;
/// Layers in a storey: a storey is three metres.
pub const STOREY: u16 = 3;
/// How far from a plot's anchor a piece is cut in the plot's frame, metres.
pub const PLOT_M: f32 = 24.0;

/// What is in a wall's middle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum Opening {
    None,
    Door,
    Window,
}

/// How a roof is fitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum RoofShape {
    Gable,
    Cone,
    Flat,
}

/// Where a piece is and what kind it is.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
pub enum Shape {
    /// On the edge between cells `a < b`, a storey up from `layer`.
    Wall {
        a: u32,
        b: u32,
        layer: u16,
        opening: Opening,
    },
    /// In `cell`, its top on the line under `layer`.
    Floor { cell: u32, layer: u16 },
    /// From `foot` to `head`, the cell across one of the foot's sides,
    /// climbing a storey from `layer`.
    Stair { foot: u32, head: u32, layer: u16 },
    /// At the corner the three `cells` (sorted) share, a storey high.
    Post { cells: [u32; 3], layer: u16 },
    /// A lantern standing in `cell` at `layer`.
    Lantern { cell: u32, layer: u16 },
    /// Over `cells` (sorted), its eaves on the line under `layer`.
    Roof {
        cells: Vec<u32>,
        layer: u16,
        shape: RoofShape,
    },
    /// A piece of furniture in `cell` on `layer`, its back to the cell's side
    /// `side` (0 to 5), or in its middle (`furniture::MIDDLE`): `furniture`
    /// design 3.
    Furniture {
        cell: u32,
        layer: u16,
        side: u8,
        kind: crate::furniture::Kind,
    },
}

/// A piece's class: what the slot rule and the palette speak of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Class {
    Wall,
    Floor,
    Stair,
    Post,
    Lantern,
    Roof,
    Furniture,
}

impl Class {
    /// Its id in the locale file's `build.piece.*`.
    pub fn id(self) -> &'static str {
        match self {
            Class::Wall => "wall",
            Class::Floor => "floor",
            Class::Stair => "stair",
            Class::Post => "post",
            Class::Lantern => "lantern",
            Class::Roof => "roof",
            Class::Furniture => "furniture",
        }
    }
}

impl Shape {
    pub fn class(&self) -> Class {
        match self {
            Shape::Wall { .. } => Class::Wall,
            Shape::Floor { .. } => Class::Floor,
            Shape::Stair { .. } => Class::Stair,
            Shape::Post { .. } => Class::Post,
            Shape::Lantern { .. } => Class::Lantern,
            Shape::Roof { .. } => Class::Roof,
            Shape::Furniture { .. } => Class::Furniture,
        }
    }

    /// The layer it stands on.
    pub fn layer(&self) -> u16 {
        match self {
            Shape::Wall { layer, .. }
            | Shape::Floor { layer, .. }
            | Shape::Stair { layer, .. }
            | Shape::Post { layer, .. }
            | Shape::Lantern { layer, .. }
            | Shape::Roof { layer, .. }
            | Shape::Furniture { layer, .. } => *layer,
        }
    }

    /// Every cell it touches.
    pub fn cells(&self) -> Vec<u32> {
        match self {
            Shape::Wall { a, b, .. } => vec![*a, *b],
            Shape::Floor { cell, .. }
            | Shape::Lantern { cell, .. }
            | Shape::Furniture { cell, .. } => vec![*cell],
            Shape::Stair { foot, head, .. } => vec![*foot, *head],
            Shape::Post { cells, .. } => cells.to_vec(),
            Shape::Roof { cells, .. } => cells.clone(),
        }
    }

    /// The slot it takes: one piece of a class per slot. A wall's opening
    /// is not part of it (a door and a wall on one edge are one slot), nor a
    /// roof's shape.
    pub fn slot(&self) -> String {
        match self {
            Shape::Wall { a, b, layer, .. } => format!("wall:{a}:{b}:{layer}"),
            Shape::Floor { cell, layer } => format!("floor:{cell}:{layer}"),
            Shape::Stair { foot, layer, .. } => format!("stair:{foot}:{layer}"),
            Shape::Post { cells, layer } => {
                format!("post:{}:{}:{}:{layer}", cells[0], cells[1], cells[2])
            }
            Shape::Lantern { cell, layer } => format!("lantern:{cell}:{layer}"),
            Shape::Roof { cells, layer, .. } => {
                format!("roof:{}:{layer}", cells.first().copied().unwrap_or(0))
            }
            Shape::Furniture {
                cell, layer, side, ..
            } => format!("furniture:{cell}:{side}:{layer}"),
        }
    }

    /// Its record id: its slot hashed (design decision 4: the same place
    /// and kind is the same id).
    pub fn id(&self) -> u64 {
        crate::river_cache::fnv(self.slot().as_bytes())
    }

    /// A wall on the edge between `a` and `b`, the pair sorted.
    pub fn wall(a: u32, b: u32, layer: u16, opening: Opening) -> Self {
        Shape::Wall {
            a: a.min(b),
            b: a.max(b),
            layer,
            opening,
        }
    }

    /// A roof over `cells`, sorted.
    pub fn roof(mut cells: Vec<u32>, layer: u16, shape: RoofShape) -> Self {
        cells.sort_unstable();
        cells.dedup();
        Shape::Roof {
            cells,
            layer,
            shape,
        }
    }

    /// A post at `cells`' corner, sorted.
    pub fn post(mut cells: [u32; 3], layer: u16) -> Self {
        cells.sort_unstable();
        Shape::Post { cells, layer }
    }
}

/// A piece: where and what, and the kit it is made of.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct Piece {
    pub kit: String,
    pub shape: Shape,
}

impl Piece {
    pub fn new(kit: &str, shape: Shape) -> Self {
        Self {
            kit: kit.into(),
            shape,
        }
    }

    pub fn id(&self) -> u64 {
        self.shape.id()
    }
}

/// A `piece` record's body: the piece, or none once it is taken away.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct PieceBody {
    pub piece: Option<Piece>,
}

/// The record that places `piece`.
pub fn placed(piece: &Piece) -> Record {
    Record::of(
        PIECE_RECORD,
        piece.id(),
        PIECE_SCHEMA,
        &PieceBody {
            piece: Some(piece.clone()),
        },
    )
}

/// The record that takes `piece` away.
pub fn removed(piece: &Piece) -> Record {
    Record::of(
        PIECE_RECORD,
        piece.id(),
        PIECE_SCHEMA,
        &PieceBody { piece: None },
    )
}

/// Every piece the records hold, by id. A body this build cannot read (a
/// later schema) is left in the records as it was and not built.
pub fn from_records(records: &crate::records::Records) -> BTreeMap<u64, Piece> {
    records
        .of_kind(PIECE_RECORD)
        .filter(|r| r.schema == PIECE_SCHEMA)
        .filter_map(|r| r.read::<PieceBody>()?.piece.map(|p| (r.id, p)))
        .collect()
}

/// The finest cells round a plot, by key: what the rules and the cut ask of
/// the planet's topology.
pub struct Cells<'a> {
    pub patch: &'a Patch,
    index: HashMap<u32, usize>,
}

impl<'a> Cells<'a> {
    pub fn new(patch: &'a Patch) -> Self {
        let index = patch
            .keys
            .iter()
            .enumerate()
            .map(|(i, &k)| (k, i))
            .collect();
        Self { patch, index }
    }

    pub fn index(&self, key: u32) -> Option<usize> {
        self.index.get(&key).copied()
    }

    pub fn cell(&self, key: u32) -> Option<&'a DualCell> {
        self.index(key).map(|i| &self.patch.cells[i])
    }

    pub fn has(&self, key: u32) -> bool {
        self.index.contains_key(&key)
    }

    /// The key of the patch cell at `index`, if it is one.
    fn key_at(&self, index: usize) -> Option<u32> {
        self.patch.keys.get(index).copied()
    }

    /// A cell's neighbours' keys, by side; `None` for one off the patch.
    pub fn neighbours(&self, key: u32) -> Vec<Option<u32>> {
        self.cell(key)
            .map(|c| c.neighbors.iter().map(|&n| self.key_at(n)).collect())
            .unwrap_or_default()
    }

    /// Whether a cell is a pentagon. A cell off the patch is not known to be
    /// one, and the rules refuse it as off the plot.
    pub fn pentagon(&self, key: u32) -> bool {
        self.cell(key).is_some_and(|c| c.corners.len() != 6)
    }

    /// The side of `a` that `b` is across.
    pub fn side(&self, a: u32, b: u32) -> Option<usize> {
        self.neighbours(a).iter().position(|&n| n == Some(b))
    }

    /// Whether two cells share an edge.
    pub fn adjacent(&self, a: u32, b: u32) -> bool {
        self.side(a, b).is_some()
    }

    /// The cell straight on from `from` through `head`: across the side of
    /// `head` opposite the one `from` is across. Hexagons only.
    pub fn straight_on(&self, from: u32, head: u32) -> Option<u32> {
        let cell = self.cell(head)?;
        if cell.corners.len() != 6 {
            return None;
        }
        let s = self.side(head, from)?;
        self.neighbours(head)[(s + 3) % 6]
    }

    /// The three cells that meet at corner `k` of `key`: the cell, and the
    /// neighbours across the sides either side of the corner.
    pub fn corner_cells(&self, key: u32, k: usize) -> Option<[u32; 3]> {
        let n = self.neighbours(key);
        let m = n.len();
        if m == 0 {
            return None;
        }
        let before = n[(k + m - 1) % m]?;
        let after = n[k % m]?;
        let mut cells = [key, before, after];
        cells.sort_unstable();
        Some(cells)
    }

    /// The two corners (unit directions) of the edge between `a` and `b`,
    /// counter-clockwise about `a`.
    pub fn edge(&self, a: u32, b: u32) -> Option<(glam::Vec3, glam::Vec3)> {
        let cell = self.cell(a)?;
        let s = self.side(a, b)?;
        let n = cell.corners.len();
        Some((cell.corners[s], cell.corners[(s + 1) % n]))
    }

    /// The corner (a unit direction) the three cells share.
    pub fn corner(&self, cells: [u32; 3]) -> Option<glam::Vec3> {
        let cell = self.cell(cells[0])?;
        (0..cell.corners.len())
            .find(|&k| self.corner_cells(cells[0], k) == Some(cells))
            .map(|k| cell.corners[k])
    }
}

/// What the rules are asked about: the cells, and where the ground is.
pub struct Site<'a> {
    pub cells: &'a Cells<'a>,
    /// The first layer over the ground in a cell: the layer a ground-storey
    /// floor stands on. `None` where it is not known, which holds nothing up.
    pub ground: &'a dyn Fn(u32) -> Option<u16>,
}

impl Site<'_> {
    /// Whether a piece at `layer` in `cell` stands on the ground.
    pub fn on_ground(&self, cell: u32, layer: u16) -> bool {
        (self.ground)(cell).is_some_and(|g| layer <= g)
    }
}

/// The pieces grouped into plots: each plot's anchor cell and the ids of the
/// pieces cut in its frame. Pieces are taken by their lowest cell key, so the
/// same pieces always make the same plots; a piece joins the first plot
/// whose anchor is within [`PLOT_M`] (`radius_m` is the planet's), else it
/// anchors a plot of its own. `direction` is a cell's centre by its key
/// (`cell_key::key_direction` in the game).
pub fn plots(
    pieces: &BTreeMap<u64, Piece>,
    radius_m: f32,
    direction: impl Fn(u32) -> Option<glam::Vec3>,
) -> Vec<(u32, Vec<u64>)> {
    let mut order: Vec<(u32, u64)> = pieces
        .iter()
        .map(|(&id, p)| (p.shape.cells().into_iter().min().unwrap_or(0), id))
        .collect();
    order.sort_unstable();
    let reach = (PLOT_M / radius_m.max(1.0)).cos();
    let mut out: Vec<(u32, glam::Vec3, Vec<u64>)> = Vec::new();
    for (cell, id) in order {
        let Some(d) = direction(cell) else {
            continue;
        };
        match out.iter_mut().find(|(_, a, _)| a.dot(d) > reach) {
            Some(plot) => plot.2.push(id),
            None => out.push((cell, d, vec![id])),
        }
    }
    out.into_iter().map(|(a, _, ids)| (a, ids)).collect()
}
