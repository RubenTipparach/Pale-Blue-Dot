//! A template's flat grid laid onto the planet's real cells
//! (`cities-in-the-world` decision 2).
//!
//! The layout's anchor cell goes on a real cell, and every other layout cell
//! is reached by walking the neighbour tables: across edge `d` of a layout
//! cell is the real cell across the matching side. Each real cell remembers
//! which of its sides is the mockup's direction 0, carried from the cell it
//! was reached from, so the walk turns with the grid however the cells are
//! distorted. Two layout neighbours are always sphere neighbours, and no real
//! cell is used twice.

use super::neighbour;
use crate::topology::DualCell;
use glam::Vec3;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// The finest cells round a site: each cell, its exact key, and its
/// neighbours as indices into the patch (`usize::MAX` for one outside it).
#[derive(Clone, Debug, Default)]
pub struct Patch {
    pub cells: Vec<DualCell>,
    pub keys: Vec<u32>,
}

impl Patch {
    /// The cell whose centre is nearest a direction.
    pub fn nearest(&self, direction: Vec3) -> Option<usize> {
        let d = direction.normalize_or_zero();
        self.cells
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.direction.dot(d).total_cmp(&b.1.direction.dot(d)))
            .map(|(i, _)| i)
    }

    /// The side of `cell` whose edge faces most nearly along `toward`, a
    /// tangent direction there.
    pub fn side_toward(&self, cell: usize, toward: Vec3) -> usize {
        let c = &self.cells[cell];
        let n = c.corners.len();
        (0..n)
            .max_by(|&a, &b| {
                let mid = |s: usize| (c.corners[s] + c.corners[(s + 1) % n]) * 0.5 - c.direction;
                mid(a).dot(toward).total_cmp(&mid(b).dot(toward))
            })
            .unwrap_or(0)
    }
}

/// Where a layout cell stands: its patch cell, and which of that cell's
/// sides is the mockup's direction 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Charted {
    pub cell: usize,
    pub d0: usize,
}

/// A layout laid onto a patch.
#[derive(Clone, Debug, Default)]
pub struct Chart {
    /// The layout cell on the site's anchor.
    pub anchor: (i32, i32),
    pub cells: BTreeMap<(i32, i32), Charted>,
}

impl Chart {
    /// The patch side across the mockup's edge `d` of layout cell `(c, r)`.
    /// The mockup's directions turn from east toward south, clockwise seen
    /// from above, where a cell's sides run counter-clockwise; so the
    /// mockup's `d + 1` is the side before.
    pub fn side(&self, c: i32, r: i32, d: usize) -> Option<usize> {
        let at = self.cells.get(&(c, r))?;
        Some((at.d0 + 6 - d % 6) % 6)
    }

    pub fn cell(&self, c: i32, r: i32) -> Option<usize> {
        self.cells.get(&(c, r)).map(|at| at.cell)
    }
}

/// Lay the layout cells in `wanted` onto `patch`: offset `anchor` on patch
/// cell `at`, with the mockup's direction 0 on its side `d0`. Every cell in
/// `wanted` must be reachable from the anchor through `wanted`, on hexagons
/// inside the patch; the first that is not is named.
pub fn chart(
    patch: &Patch,
    anchor: (i32, i32),
    at: usize,
    d0: usize,
    wanted: &BTreeSet<(i32, i32)>,
) -> Result<Chart, String> {
    let out = walk(patch, anchor, at, d0, wanted, true)?;
    if let Some(missed) = wanted.iter().find(|w| !out.cells.contains_key(w)) {
        return Err(format!(
            "layout cell {missed:?} is not reached from the anchor"
        ));
    }
    Ok(out)
}

/// As [`chart`], but every cell in `wanted` that cannot be laid (off the
/// patch, past a pentagon, where the walk would not close) is left out
/// rather than failing the chart: how far a layout could reach, for a
/// harbour weighing where to lie (slice 4d).
pub fn chart_reach(
    patch: &Patch,
    anchor: (i32, i32),
    at: usize,
    d0: usize,
    wanted: &BTreeSet<(i32, i32)>,
) -> Chart {
    walk(patch, anchor, at, d0, wanted, false).unwrap_or_default()
}

fn walk(
    patch: &Patch,
    anchor: (i32, i32),
    at: usize,
    d0: usize,
    wanted: &BTreeSet<(i32, i32)>,
    strict: bool,
) -> Result<Chart, String> {
    let mut out = Chart {
        anchor,
        cells: BTreeMap::new(),
    };
    let mut used = BTreeMap::new();
    out.cells.insert(anchor, Charted { cell: at, d0 });
    used.insert(at, anchor);
    let mut queue = VecDeque::from([anchor]);
    // A strict walk fails where a lenient one leaves the cell out.
    macro_rules! refuse {
        ($($why:tt)*) => {
            if strict {
                return Err(format!($($why)*));
            } else {
                continue;
            }
        };
    }
    while let Some((c, r)) = queue.pop_front() {
        let here = out.cells[&(c, r)];
        let cell = &patch.cells[here.cell];
        if cell.neighbors.len() != 6 {
            refuse!("layout cell ({c}, {r}) is on a pentagon");
        }
        for d in 0..6 {
            let next = neighbour(c, r, d);
            if !wanted.contains(&next) {
                continue;
            }
            let side = (here.d0 + 6 - d) % 6;
            let there = cell.neighbors[side];
            if there == usize::MAX || there >= patch.cells.len() {
                refuse!("layout cell {next:?} is off the patch");
            }
            let Some(back) = patch.cells[there]
                .neighbors
                .iter()
                .position(|&n| n == here.cell)
            else {
                refuse!("cell {there} does not list {} back", here.cell);
            };
            let reached = Charted {
                cell: there,
                d0: (back + d + 3) % 6,
            };
            match out.cells.get(&next) {
                Some(seen) if *seen != reached => {
                    refuse!("the chart does not close at {next:?}");
                }
                Some(_) => {}
                None => {
                    if let Some(other) = used.insert(there, next) {
                        used.insert(there, other);
                        refuse!("{next:?} and {other:?} land on one cell");
                    }
                    out.cells.insert(next, reached);
                    queue.push_back(next);
                }
            }
        }
    }
    Ok(out)
}
